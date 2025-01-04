use serde::{Deserialize, Serialize};
use worker::{Error, Method, ObjectNamespace, Request, RequestInit, Storage};
use worker::wasm_bindgen::JsValue;
use prelude::async_trait;
use prelude::domain::{Transaction, Versioned};
use prelude::runtime::repository::Reply;
use prelude::runtime::repository::Reply::{MalformedPrompt, NotFound, Success, VersionConflict};
use scheduling::aggregate::{Airfield, AirfieldId, Airship, AirshipId, Flight, FlightId};
use scheduling::repository::{AirfieldRepository, AirfieldRepositoryError, AirshipRepository, AirshipRepositoryError, FlightRepository, FlightRepositoryError};
use crate::runtime::repository::DurableObjectSchedulingRepositoryProtocol::*;

#[derive(Serialize, Deserialize)]
pub enum DurableObjectSchedulingRepositoryProtocol {
    // airfield
    GetAirfieldPrompt(AirfieldId),
    GetAirfieldReply(Reply<Versioned<Airfield>>),

    SetAirfieldPrompt(Versioned<Airfield>),
    SetAirfieldReply(Reply<()>),

    // airship
    GetAirshipPrompt(AirshipId),
    GetAirshipReply(Reply<Versioned<Airship>>),

    SetAirshipPrompt(Versioned<Airship>),
    SetAirshipReply(Reply<()>),

    // flight
    GetFlightPrompt(FlightId),
    GetFlightReply(Reply<Versioned<Flight>>),

    SetFlightPrompt(Versioned<Flight>),
    SetFlightReply(Reply<()>),
}

pub struct DurableObjectSchedulingRepository {
    namespace: ObjectNamespace
}

impl DurableObjectSchedulingRepository {
    pub fn new(namespace: ObjectNamespace) -> Self {
        Self {
            namespace
        }
    }

    pub async fn handle(prompt: DurableObjectSchedulingRepositoryProtocol, storage: &mut Storage) -> Result<DurableObjectSchedulingRepositoryProtocol, Error> {
        match prompt {
            GetAirfieldPrompt(id) => {
                let key = ["airfield:", &id.to_string()].concat();
                let value =  storage.get::<Versioned<Airfield>>(&key).await.ok();

                if let Some(airfield) = value{
                    Ok(GetAirfieldReply(Success(airfield)))
                }
                else {
                    Ok(GetAirfieldReply(NotFound))
                }
            }

            SetAirfieldPrompt(airfield) => {
                let id = airfield.value_ref().id.clone();
                let key = ["airfield:", &id.to_string()].concat();
                let value: Option<Versioned<Airfield>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if airfield.version() == existing_version + 1 {
                    storage.put(&key, &airfield).await?;
                    Ok(SetAirfieldReply(Success(())))
                }
                else {
                    Ok(SetAirfieldReply(VersionConflict))
                }
            }

            GetAirshipPrompt(id) => {
                let key = ["airship:", &id.to_string()].concat();
                let value =  storage.get::<Versioned<Airship>>(&key).await.ok();

                if let Some(airship) = value{
                    Ok(GetAirshipReply(Success(airship)))
                }
                else {
                    Ok(GetAirshipReply(NotFound))
                }
            }

            SetAirshipPrompt(airship) => {
                let id = airship.value_ref().id.clone();
                let key = ["airship:", &id.to_string()].concat();
                let value: Option<Versioned<Airship>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if airship.version() == existing_version + 1 {
                    storage.put(&key, &airship).await?;
                    Ok(SetAirshipReply(Success(())))
                }
                else {
                    Ok(SetAirshipReply(VersionConflict))
                }
            }

            GetFlightPrompt(id) => {
                let key = ["flight:", &id.to_string()].concat();
                let value =  storage.get::<Versioned<Flight>>(&key).await.ok();

                if let Some(airship) = value{
                    Ok(GetFlightReply(Success(airship)))
                }
                else {
                    Ok(GetFlightReply(NotFound))
                }
            }

            SetFlightPrompt(airship) => {
                let id = airship.value_ref().id;
                let key = ["flight:", &id.to_string()].concat();
                let value: Option<Versioned<FlightId>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if airship.version() == existing_version + 1 {
                    storage.put(&key, &airship).await?;
                    Ok(SetFlightReply(Success(())))
                }
                else {
                    Ok(SetFlightReply(VersionConflict))
                }
            }

            _ => Err(Error::BadEncoding)
        }
    }
}


#[async_trait(?Send)]
impl AirfieldRepository for DurableObjectSchedulingRepository {
    async fn get(&self, id: &AirfieldId) -> Result<Option<Airfield>, AirfieldRepositoryError> {
        let reply = self
            .dispatch(GetAirfieldPrompt(id.clone())).await
            .map_err(|e| AirfieldRepositoryError::IoError(e.to_string()))?;

        if let GetAirfieldReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(AirfieldRepositoryError::VersionConflict),
                MalformedPrompt => Err(AirfieldRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(AirfieldRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: &AirfieldId) -> Result<Transaction<AirfieldId, Airfield>, AirfieldRepositoryError> {
        let reply = self
            .dispatch(GetAirfieldPrompt(id.clone())).await
            .map_err(|e| AirfieldRepositoryError::IoError(e.to_string()))?;

        if let GetAirfieldReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Transaction::from_versioned(id.clone(), result)),
                NotFound => Ok(Transaction::new(id.clone())),
                VersionConflict => Err(AirfieldRepositoryError::VersionConflict),
                MalformedPrompt => Err(AirfieldRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(AirfieldRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<AirfieldId, Airfield>) -> Result<(), AirfieldRepositoryError> {
        if let Some(airship) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetAirfieldPrompt(airship)).await
                .map_err(|e| AirfieldRepositoryError::IoError(e.to_string()))?;

            if let SetAirfieldReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(AirfieldRepositoryError::NotFound),
                    VersionConflict => Err(AirfieldRepositoryError::VersionConflict),
                    MalformedPrompt => Err(AirfieldRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(AirfieldRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}


#[async_trait(?Send)]
impl AirshipRepository for DurableObjectSchedulingRepository {
    async fn get(&self, id: &AirshipId) -> Result<Option<Airship>, AirshipRepositoryError> {
        let reply = self
            .dispatch(GetAirshipPrompt(id.clone())).await
            .map_err(|e| AirshipRepositoryError::IoError(e.to_string()))?;

        if let GetAirshipReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(AirshipRepositoryError::VersionConflict),
                MalformedPrompt => Err(AirshipRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(AirshipRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: &AirshipId) -> Result<Transaction<AirshipId, Airship>, AirshipRepositoryError> {
        let reply = self
            .dispatch(GetAirshipPrompt(id.clone())).await
            .map_err(|e| AirshipRepositoryError::IoError(e.to_string()))?;

        if let GetAirshipReply(get_reply) = reply {
            match get_reply {
                Success(result) =>  Ok(Transaction::from_versioned(id.clone(), result)),
                NotFound => Ok(Transaction::new(id.clone())),
                VersionConflict => Err(AirshipRepositoryError::VersionConflict),
                MalformedPrompt => Err(AirshipRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(AirshipRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<AirshipId, Airship>) -> Result<(), AirshipRepositoryError> {
        if let Some(airship) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetAirshipPrompt(airship)).await
                .map_err(|e| AirshipRepositoryError::IoError(e.to_string()))?;

            if let SetAirshipReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(AirshipRepositoryError::NotFound),
                    VersionConflict => Err(AirshipRepositoryError::VersionConflict),
                    MalformedPrompt => Err(AirshipRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(AirshipRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}

#[async_trait(?Send)]
impl FlightRepository for DurableObjectSchedulingRepository {
    async fn get(&self, id: FlightId) -> Result<Option<Flight>, FlightRepositoryError> {
        let reply = self
            .dispatch(GetFlightPrompt(id)).await
            .map_err(|e| FlightRepositoryError::IoError(e.to_string()))?;

        if let GetFlightReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(FlightRepositoryError::VersionConflict),
                MalformedPrompt => Err(FlightRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(FlightRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: FlightId) -> Result<Transaction<FlightId, Flight>, FlightRepositoryError> {
        let reply = self
            .dispatch(GetFlightPrompt(id)).await
            .map_err(|e| FlightRepositoryError::IoError(e.to_string()))?;

        if let GetFlightReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Transaction::from_versioned(id, result)),
                NotFound => Ok(Transaction::new(id)),
                VersionConflict => Err(FlightRepositoryError::VersionConflict),
                MalformedPrompt => Err(FlightRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(FlightRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<FlightId, Flight>) -> Result<(), FlightRepositoryError> {
        if let Some(airship) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetFlightPrompt(airship)).await
                .map_err(|e| FlightRepositoryError::IoError(e.to_string()))?;

            if let SetFlightReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(FlightRepositoryError::NotFound),
                    VersionConflict => Err(FlightRepositoryError::VersionConflict),
                    MalformedPrompt => Err(FlightRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(FlightRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}

impl DurableObjectSchedulingRepository {
    async fn dispatch(&self, prompt: DurableObjectSchedulingRepositoryProtocol) -> Result<DurableObjectSchedulingRepositoryProtocol, Error> {
        let object = self.namespace
            .id_from_name("default")?;

        let request_body = serde_json::to_string(&prompt)
            .map(JsValue::from)
            .map(Some)?;

        let request = Request::new_with_init(
            "http://do/",
            RequestInit::new()
                .with_method(Method::Post)
                .with_body(request_body)
        )?;

        object.get_stub()?
            .fetch_with_request(request).await?
            .json().await
    }
}
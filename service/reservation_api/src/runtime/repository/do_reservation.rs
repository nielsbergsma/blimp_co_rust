use serde::{Deserialize, Serialize};
use worker::{Error, Method, ObjectNamespace, Request, RequestInit, Storage};
use worker::wasm_bindgen::JsValue;
use prelude::async_trait;
use prelude::domain::{Transaction, Versioned};
use prelude::runtime::repository::Reply;
use prelude::runtime::repository::Reply::{MalformedPrompt, NotFound, Success, VersionConflict};
use reservation::aggregate::{Airfield, AirfieldId, FlightAvailability, FlightId, Journey, JourneyId, Reservation, ReservationId};
use reservation::repository::{AirfieldRepository, AirfieldRepositoryError, FlightAvailabilityRepository, FlightAvailabilityRepositoryError, JourneyRepository, JourneyRepositoryError, ReservationRepository, ReservationRepositoryError};
use DurableObjectReservationRepositoryProtocol::*;

#[derive(Serialize, Deserialize)]
pub enum DurableObjectReservationRepositoryProtocol {
    // journey
    GetJourneyPrompt(JourneyId),
    GetJourneyReply(Reply<Versioned<Journey>>),

    SetJourneyPrompt(Versioned<Journey>),
    SetJourneyReply(Reply<()>),

    // airfield
    GetAirfieldPrompt(AirfieldId),
    GetAirfieldReply(Reply<Versioned<Airfield>>),

    SetAirfieldPrompt(Versioned<Airfield>),
    SetAirfieldReply(Reply<()>),

    // flight availability
    GetFlightAvailabilityPrompt(FlightId),
    GetFlightAvailabilityReply(Reply<Versioned<FlightAvailability>>),

    SetFlightAvailabilityPrompt(Versioned<FlightAvailability>),
    SetFlightAvailabilityReply(Reply<()>),

    // reservation
    GetReservationPrompt(ReservationId),
    GetReservationReply(Reply<Versioned<Reservation>>),

    SetReservationPrompt(Versioned<Reservation>),
    SetReservationReply(Reply<()>),
}

pub struct DurableObjectReservationRepository {
    namespace: ObjectNamespace
}

impl DurableObjectReservationRepository {
    pub fn new(namespace: ObjectNamespace) -> Self {
        Self {
            namespace
        }
    }

    pub async fn handle(prompt: DurableObjectReservationRepositoryProtocol, storage: &mut Storage) -> Result<DurableObjectReservationRepositoryProtocol, Error> {
        match prompt {
            GetJourneyPrompt(id) => {
                let key = ["journey:", &id.to_string()].concat();
                let value =  storage.get::<Versioned<Journey>>(&key).await.ok();

                if let Some(journey) = value{
                    Ok(GetJourneyReply(Success(journey)))
                }
                else {
                    Ok(GetJourneyReply(NotFound))
                }
            }

            SetJourneyPrompt(journey) => {
                let id = journey.value_ref().id;
                let key = ["journey:", &id.to_string()].concat();
                let value: Option<Versioned<Journey>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if journey.version() == existing_version + 1 {
                    storage.put(&key, &journey).await?;
                    Ok(SetJourneyReply(Success(())))
                }
                else {
                    Ok(SetJourneyReply(VersionConflict))
                }
            }

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

            GetFlightAvailabilityPrompt(flight_id) => {
                let key = ["flight_availability:", &flight_id.to_string()].concat();
                let value =  storage.get::<Versioned<FlightAvailability>>(&key).await.ok();

                if let Some(flight_availability) = value{
                    Ok(GetFlightAvailabilityReply(Success(flight_availability)))
                }
                else {
                    Ok(GetFlightAvailabilityReply(NotFound))
                }
            }

            SetFlightAvailabilityPrompt(flight_availability) => {
                let id = &flight_availability.value_ref().flight.id;
                let key = ["flight_availability:", &id.to_string()].concat();
                let value: Option<Versioned<FlightAvailability>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if flight_availability.version() == existing_version + 1 {
                    storage.put(&key, &flight_availability).await?;
                    Ok(SetFlightAvailabilityReply(Success(())))
                }
                else {
                    Ok(SetFlightAvailabilityReply(VersionConflict))
                }
            }

            GetReservationPrompt(id) => {
                let key = ["reservation:", &id.to_string()].concat();
                let value =  storage.get::<Versioned<Reservation>>(&key).await.ok();

                if let Some(reservation) = value{
                    Ok(GetReservationReply(Success(reservation)))
                }
                else {
                    Ok(GetReservationReply(NotFound))
                }
            }

            SetReservationPrompt(reservation) => {
                let id = &reservation.value_ref().id();
                let key = ["reservation:", &id.to_string()].concat();
                let value: Option<Versioned<Reservation>> = storage.get(&key).await.ok();

                let existing_version = value
                    .map(|va| va.version())
                    .unwrap_or_default();

                if reservation.version() == existing_version + 1 {
                    storage.put(&key, &reservation).await?;
                    Ok(SetReservationReply(Success(())))
                }
                else {
                    Ok(SetReservationReply(VersionConflict))
                }
            }

            _ => Err(Error::BadEncoding)
        }
    }
}

#[async_trait(?Send)]
impl JourneyRepository for DurableObjectReservationRepository {
    async fn get(&self, id: &JourneyId) -> Result<Option<Journey>, JourneyRepositoryError> {
        let reply = self
            .dispatch(GetJourneyPrompt(*id)).await
            .map_err(|e| JourneyRepositoryError::IoError(e.to_string()))?;

        if let GetJourneyReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(JourneyRepositoryError::VersionConflict),
                MalformedPrompt => Err(JourneyRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(JourneyRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: &JourneyId) -> Result<Transaction<JourneyId, Journey>, JourneyRepositoryError> {
        let reply = self
            .dispatch(GetJourneyPrompt(*id)).await
            .map_err(|e| JourneyRepositoryError::IoError(e.to_string()))?;

        if let GetJourneyReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Transaction::from_versioned(*id, result)),
                NotFound => Ok(Transaction::new(*id)),
                VersionConflict => Err(JourneyRepositoryError::VersionConflict),
                MalformedPrompt => Err(JourneyRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(JourneyRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<JourneyId, Journey>) -> Result<(), JourneyRepositoryError> {
        if let Some(journey) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetJourneyPrompt(journey)).await
                .map_err(|e| JourneyRepositoryError::IoError(e.to_string()))?;

            if let SetJourneyReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(JourneyRepositoryError::NotFound),
                    VersionConflict => Err(JourneyRepositoryError::VersionConflict),
                    MalformedPrompt => Err(JourneyRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(JourneyRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}

#[async_trait(?Send)]
impl AirfieldRepository for DurableObjectReservationRepository {
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
impl FlightAvailabilityRepository for DurableObjectReservationRepository {
    async fn get(&self, id: &FlightId) -> Result<Option<FlightAvailability>, FlightAvailabilityRepositoryError> {
        let reply = self
            .dispatch(GetFlightAvailabilityPrompt(id.clone())).await
            .map_err(|e| FlightAvailabilityRepositoryError::IoError(e.to_string()))?;

        if let GetFlightAvailabilityReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(FlightAvailabilityRepositoryError::VersionConflict),
                MalformedPrompt => Err(FlightAvailabilityRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(FlightAvailabilityRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: &FlightId) -> Result<Transaction<FlightId, FlightAvailability>, FlightAvailabilityRepositoryError> {
        let reply = self
            .dispatch(GetFlightAvailabilityPrompt(id.clone())).await
            .map_err(|e| FlightAvailabilityRepositoryError::IoError(e.to_string()))?;

        if let GetFlightAvailabilityReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Transaction::from_versioned(id.clone(), result)),
                NotFound => Ok(Transaction::new(id.clone())),
                VersionConflict => Err(FlightAvailabilityRepositoryError::VersionConflict),
                MalformedPrompt => Err(FlightAvailabilityRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(FlightAvailabilityRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<FlightId, FlightAvailability>) -> Result<(), FlightAvailabilityRepositoryError> {
        if let Some(flight_availability) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetFlightAvailabilityPrompt(flight_availability)).await
                .map_err(|e| FlightAvailabilityRepositoryError::IoError(e.to_string()))?;

            if let SetFlightAvailabilityReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(FlightAvailabilityRepositoryError::NotFound),
                    VersionConflict => Err(FlightAvailabilityRepositoryError::VersionConflict),
                    MalformedPrompt => Err(FlightAvailabilityRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(FlightAvailabilityRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}


#[async_trait(?Send)]
impl ReservationRepository for DurableObjectReservationRepository {
    async fn get(&self, id: &ReservationId) -> Result<Option<Reservation>, ReservationRepositoryError> {
        let reply = self
            .dispatch(GetReservationPrompt(*id)).await
            .map_err(|e| ReservationRepositoryError::IoError(e.to_string()))?;

        if let GetReservationReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Some(result.value())),
                NotFound => Ok(None),
                VersionConflict => Err(ReservationRepositoryError::VersionConflict),
                MalformedPrompt => Err(ReservationRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(ReservationRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_begin(&self, id: &ReservationId) -> Result<Transaction<ReservationId, Reservation>, ReservationRepositoryError> {
        let reply = self
            .dispatch(GetReservationPrompt(*id)).await
            .map_err(|e| ReservationRepositoryError::IoError(e.to_string()))?;

        if let GetReservationReply(get_reply) = reply {
            match get_reply {
                Success(result) => Ok(Transaction::from_versioned(*id, result)),
                NotFound => Ok(Transaction::new(*id)),
                VersionConflict => Err(ReservationRepositoryError::VersionConflict),
                MalformedPrompt => Err(ReservationRepositoryError::IoError("malformed prompt".to_owned()))
            }
        }
        else {
            Err(ReservationRepositoryError::IoError("unexpected reply".to_owned()))
        }
    }

    async fn set_commit(&self, transaction: Transaction<ReservationId, Reservation>) -> Result<(), ReservationRepositoryError> {
        if let Some(reservation) = transaction.next_versioned_value() {
            let reply = self
                .dispatch(SetReservationPrompt(reservation)).await
                .map_err(|e| ReservationRepositoryError::IoError(e.to_string()))?;

            if let SetReservationReply(set_reply) = reply {
                match set_reply {
                    Success(result) => Ok(result),
                    NotFound => Err(ReservationRepositoryError::NotFound),
                    VersionConflict => Err(ReservationRepositoryError::VersionConflict),
                    MalformedPrompt => Err(ReservationRepositoryError::IoError("malformed prompt".to_owned()))
                }
            } else {
                Err(ReservationRepositoryError::IoError("unexpected reply".to_owned()))
            }
        }
        else {
            Ok(())
        }
    }
}

impl DurableObjectReservationRepository {
    async fn dispatch(&self, prompt: DurableObjectReservationRepositoryProtocol) -> Result<DurableObjectReservationRepositoryProtocol, Error> {
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
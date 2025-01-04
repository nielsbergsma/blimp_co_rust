use thiserror::Error;
use prelude::domain::EventTryIntoError;
use crate::aggregate::{AirshipModel, AirshipName, AirshipNumberOfSeats, AirshipId};
use crate::repository::AirshipRepositoryError;

pub struct AddAirshipToFleet {
    pub id: AirshipId,
    pub name: AirshipName,
    pub model: AirshipModel,
    pub number_of_seats: AirshipNumberOfSeats,
}

#[derive(Error, Debug, PartialEq)]
pub enum AddAirshipToFleetError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("unknown airship")]
    UnknownAirship,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    OtherError(String),
}

pub type AddAirshipToFleetResult = Result<AirshipId, AddAirshipToFleetError>;


// transformers
impl From<AirshipRepositoryError> for AddAirshipToFleetError {
    fn from(value: AirshipRepositoryError) -> Self {
        match value {
            AirshipRepositoryError::IoError(reason) => Self::IoError(reason),
            AirshipRepositoryError::NotFound => Self::UnknownAirship,
            AirshipRepositoryError::VersionConflict => Self::VersionConflict,
        }
    }
}

impl From<EventTryIntoError> for AddAirshipToFleetError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
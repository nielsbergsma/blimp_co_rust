use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{Flight};
use crate::repository::FlightAvailabilityRepositoryError;

#[derive(Error, Debug, PartialEq)]
pub enum MakeFlightAvailableError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    OtherError(String),
}

pub type MakeFlightAvailableResult = Result<(), MakeFlightAvailableError>;

pub struct MakeFlightAvailable {
    pub flight: Flight
}

// transformers
impl From<FlightAvailabilityRepositoryError> for MakeFlightAvailableError {
    fn from(value: FlightAvailabilityRepositoryError) -> Self {
        MakeFlightAvailableError::IoError(value.to_string())
    }
}

impl From<EventTryIntoError> for MakeFlightAvailableError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}

impl From<EventPublishError> for MakeFlightAvailableError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}
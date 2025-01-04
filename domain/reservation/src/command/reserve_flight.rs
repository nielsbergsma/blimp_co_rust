use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError, Versioned};
use crate::aggregate::{ FlightId, NumberOfSeats, ReservationId};
use crate::repository::FlightAvailabilityRepositoryError;

pub struct ReserveFlight {
    pub reservation: Versioned<ReservationId>,
    pub flight: FlightId,
    pub seats: NumberOfSeats
}

#[derive(Error, Debug, PartialEq)]
pub enum ReserveFlightError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("unknown flight: {0}")]
    UnknownFlight(FlightId),

    #[error("version conflict")]
    VersionConflict,
}

pub type ReserveFlightResult = Result<(), ReserveFlightError>;

// transformers
impl From<FlightAvailabilityRepositoryError> for ReserveFlightError {
    fn from(value: FlightAvailabilityRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventPublishError> for ReserveFlightError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventTryIntoError> for ReserveFlightError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
use std::io::Error;
use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{Passengers, ReservationId};
use crate::policy::ReservationPolicyError;
use crate::repository::ReservationRepositoryError;

#[derive(Error, Debug, PartialEq)]
pub enum RevisePassengersError {
    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

       #[error("{0}")]
    PolicyError(ReservationPolicyError),

    #[error("unknown reservation")]
    UnknownReservation,

    #[error("other: {0}")]
    OtherError(String),
}

pub type RevisePassengersResult = Result<(), RevisePassengersError>;

pub struct RevisePassengers {
    pub reservation: ReservationId,
    pub passengers: Passengers,
}

// transformers
impl From<ReservationRepositoryError> for RevisePassengersError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<ReservationPolicyError> for RevisePassengersError {
    fn from(value: ReservationPolicyError) -> Self {
        Self::PolicyError(value)
    }
}

impl From<std::io::Error> for RevisePassengersError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventPublishError> for RevisePassengersError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventTryIntoError> for RevisePassengersError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
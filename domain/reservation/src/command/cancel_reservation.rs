use std::io::Error;
use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{ReservationId};
use crate::policy::ReservationPolicyError;
use crate::repository::ReservationRepositoryError;

pub type CancelReservationResult = Result<(), CancelReservationError>;

pub struct CancelReservation {
    pub id: ReservationId
}

#[derive(Error, Debug, PartialEq)]
pub enum CancelReservationError {
    #[error("id conflict")]
    IdConflict,

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

impl From<ReservationRepositoryError> for CancelReservationError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<ReservationPolicyError> for CancelReservationError {
    fn from(value: ReservationPolicyError) -> Self {
        Self::PolicyError(value)
    }
}

impl From<std::io::Error> for CancelReservationError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventPublishError> for CancelReservationError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventTryIntoError> for CancelReservationError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}

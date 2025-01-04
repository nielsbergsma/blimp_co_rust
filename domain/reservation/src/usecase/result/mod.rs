use std::io::Error;
use thiserror::Error;
use crate::aggregate::ReservationId;
use crate::repository::ReservationRepositoryError;

// handle flight reserved
#[derive(Error, Debug, PartialEq)]
pub enum HandleFlightReservedError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("unknown reservation")]
    UnknownReservation(ReservationId)
}

pub type HandleFlightReservedResult = Result<(), HandleFlightReservedError>;

impl From<ReservationRepositoryError> for HandleFlightReservedError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<std::io::Error> for HandleFlightReservedError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum HandleFlightReservationFailedError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("unknown reservation")]
    UnknownReservation(ReservationId)
}

pub type HandleFlightReservationFailedResult = Result<(), HandleFlightReservationFailedError>;

impl From<ReservationRepositoryError> for HandleFlightReservationFailedError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<std::io::Error> for HandleFlightReservationFailedError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}
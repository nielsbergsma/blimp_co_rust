use thiserror::Error;
use crate::aggregate::{Reservation, ReservationId};
use crate::repository::ReservationRepositoryError;

pub struct GetReservation {
    pub id: ReservationId
}

#[derive(Error, Debug, PartialEq)]
pub enum GetReservationError {
    #[error("unknown reservation")]
    UnknownReservation,

    #[error("I/O error: {0}")]
    IoError(String),
}

impl From<ReservationRepositoryError> for GetReservationError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

pub type GetReservationResult = Result<Reservation, GetReservationError>;
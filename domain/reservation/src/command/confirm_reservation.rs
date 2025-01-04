use std::io::Error;
use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{Contact, ItineraryError, JourneyId, PassengerArrangement, ReservationId};
use crate::command::{Itinerary, ReferencedItineraryStage};
use crate::policy::ReservationPolicyError;
use crate::repository::{JourneyRepositoryError, ReservationRepositoryError};

#[derive(Error, Debug, PartialEq)]
pub enum ConfirmReservationError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    ItineraryError(ItineraryError),

    #[error("{0}")]
    PolicyError(ReservationPolicyError),

    #[error("unknown journey")]
    UnknownJourney,

    #[error("other: {0}")]
    OtherError(String),
}

pub type ConfirmReservationResult = Result<ReservationId, ConfirmReservationError>;

pub struct ConfirmReservation {
    pub journey: JourneyId,
    pub contact: Contact,
    pub passengers: PassengerArrangement,
    pub itinerary: Itinerary<ReferencedItineraryStage>,
}

// transformers
impl From<JourneyRepositoryError> for ConfirmReservationError {
    fn from(value: JourneyRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<ReservationRepositoryError> for ConfirmReservationError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<ItineraryError> for ConfirmReservationError {
    fn from(value: ItineraryError) -> Self {
        Self::ItineraryError(value)
    }
}

impl From<ReservationPolicyError> for ConfirmReservationError {
    fn from(value: ReservationPolicyError) -> Self {
        Self::PolicyError(value)
    }
}

impl From<EventTryIntoError> for ConfirmReservationError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}

impl From<EventPublishError> for ConfirmReservationError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<std::io::Error> for ConfirmReservationError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}
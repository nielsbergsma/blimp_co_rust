use std::io::Error;
use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{ItineraryError, ReservationId};
use crate::command::{Itinerary, ReferencedItineraryStage};
use crate::policy::ReservationPolicyError;
use crate::repository::{JourneyRepositoryError, ReservationRepositoryError};

#[derive(Error, Debug, PartialEq)]
pub enum ReviseItineraryError {
    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    ItineraryError(ItineraryError),

    #[error("{0}")]
    PolicyError(ReservationPolicyError),

    #[error("unknown reservation")]
    UnknownReservation,

    #[error("unknown journey")]
    UnknownJourney,

    #[error("other: {0}")]
    OtherError(String),
}

pub type ReviseItineraryResult = Result<(), ReviseItineraryError>;

pub struct ReviseItinerary {
    pub reservation: ReservationId,
    pub itinerary: Itinerary<ReferencedItineraryStage>,
}

// transformers
impl From<ReservationRepositoryError> for ReviseItineraryError {
    fn from(value: ReservationRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<JourneyRepositoryError> for ReviseItineraryError {
    fn from(value: JourneyRepositoryError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<ReservationPolicyError> for ReviseItineraryError {
    fn from(value: ReservationPolicyError) -> Self {
        Self::PolicyError(value)
    }
}

impl From<ItineraryError> for ReviseItineraryError {
    fn from(value: ItineraryError) -> Self {
        Self::ItineraryError(value)
    }
}

impl From<std::io::Error> for ReviseItineraryError {
    fn from(value: Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventPublishError> for ReviseItineraryError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<EventTryIntoError> for ReviseItineraryError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
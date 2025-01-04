use chrono::{DateTime, FixedOffset};
use thiserror::Error;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{AirfieldId, AirshipId, FlightError, FlightId};
use crate::command::{AddAirshipToFleetError, RegisterAirfieldError};
use crate::repository::{AirfieldRepositoryError, AirshipRepositoryError, FlightRepositoryError};

pub struct ScheduleFlight {
    pub departure_location: AirfieldId,
    pub departure_time: DateTime<FixedOffset>,
    pub arrival_location: AirfieldId,
    pub arrival_time: DateTime<FixedOffset>,
    pub airship: AirshipId,
}

#[derive(Error, Debug, PartialEq)]
pub enum ScheduleFlightError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("id conflict error")]
    IdConflict,

    #[error("unknown airfield")]
    UnknownAirfield,

    #[error("unknown airship")]
    UnknownAirship,

    #[error("malformed flight: {0}")]
    MalformedFlight(FlightError),

    #[error("version conflict")]
    VersionConflict,
}

pub type ScheduleFlightResult = Result<FlightId, ScheduleFlightError>;

// transformers
impl From<FlightRepositoryError> for ScheduleFlightError {
    fn from(value: FlightRepositoryError) -> Self {
        match value {
            FlightRepositoryError::IoError(reason) => Self::IoError(reason),
            FlightRepositoryError::NotFound => Self::IdConflict,
            FlightRepositoryError::VersionConflict => Self::VersionConflict,
        }
    }
}

impl From<AirfieldRepositoryError> for ScheduleFlightError {
    fn from(value: AirfieldRepositoryError) -> Self {
        match value {
            AirfieldRepositoryError::IoError(reason) => Self::IoError(reason),
            AirfieldRepositoryError::NotFound => Self::UnknownAirfield,
            AirfieldRepositoryError::VersionConflict => Self::UnknownAirfield,
        }
    }
}

impl From<AirshipRepositoryError> for ScheduleFlightError {
    fn from(value: AirshipRepositoryError) -> Self {
        match value {
            AirshipRepositoryError::IoError(reason) => Self::IoError(reason),
            AirshipRepositoryError::NotFound => Self::UnknownAirship,
            AirshipRepositoryError::VersionConflict => Self::UnknownAirship,
        }
    }
}

impl From<FlightError> for ScheduleFlightError {
    fn from(value: FlightError) -> Self {
        Self::MalformedFlight(value)
    }
}

impl From<EventTryIntoError> for ScheduleFlightError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to serialize event".to_owned())
    }
}

impl From<EventPublishError> for ScheduleFlightError {
    fn from(_: EventPublishError) -> Self {
        Self::IoError("unable to publish event".to_owned())
    }
}

impl From<EventPublishError> for AddAirshipToFleetError {
    fn from(_: EventPublishError) -> Self {
        Self::IoError("unable to publish event".to_owned())
    }
}

impl From<EventPublishError> for RegisterAirfieldError {
    fn from(_: EventPublishError) -> Self {
        Self::IoError("unable to publish event".to_owned())
    }
}
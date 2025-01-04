use thiserror::Error;
use prelude::collection::SortedSet;
use prelude::domain::{EventPublishError, EventTryIntoError};
use crate::aggregate::{AirfieldId, JourneyError, JourneyId, JourneyName, Segment};
use crate::repository::{AirfieldRepositoryError, JourneyRepositoryError};

pub struct PublishJourney {
    pub name: JourneyName,
    pub segments: SortedSet<Segment>,
}

#[derive(Error, Debug, PartialEq)]
pub enum PublishJourneyError {
    #[error("id conflict")]
    IdConflict,

    #[error("version conflict")]
    VersionConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    JourneyError(JourneyError),

    #[error("{0}")]
    OtherError(String),

    #[error("unregistered airfield: {0}")]
    UnknownAirfield(AirfieldId)
}

pub type PublishJourneyResult = Result<JourneyId, PublishJourneyError>;

// transformers
impl From<JourneyRepositoryError> for PublishJourneyError {
    fn from(value: JourneyRepositoryError) -> Self {
        Self::OtherError(value.to_string())
    }
}

impl From<AirfieldRepositoryError> for PublishJourneyError {
    fn from(value: AirfieldRepositoryError) -> Self {
        Self::OtherError(value.to_string())
    }
}

impl From<JourneyError> for PublishJourneyError {
    fn from(value: JourneyError) -> Self {
        Self::JourneyError(value)
    }
}

impl From<EventTryIntoError> for PublishJourneyError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}

impl From<EventPublishError> for PublishJourneyError {
    fn from(value: EventPublishError) -> Self {
        Self::IoError(value.to_string())
    }
}
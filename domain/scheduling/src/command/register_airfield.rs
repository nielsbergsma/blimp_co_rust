use thiserror::Error;
use prelude::data::GeoHash;
use prelude::domain::EventTryIntoError;
use crate::aggregate::{AirfieldId, AirfieldName};
use crate::repository::AirfieldRepositoryError;

pub struct RegisterAirfield {
    pub id: AirfieldId,
    pub name: AirfieldName,
    pub location: GeoHash,
}

#[derive(Error, Debug, PartialEq)]
pub enum RegisterAirfieldError {
    #[error("id conflict")]
    IdConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    OtherError(String),

    #[error("unknown airfield")]
    UnknownAirfield,

    #[error("version conflict")]
    VersionConflict,
}

pub type RegisterAirfieldResult = Result<AirfieldId, RegisterAirfieldError>;

// transformers
impl From<AirfieldRepositoryError> for RegisterAirfieldError {
    fn from(value: AirfieldRepositoryError) -> Self {
        match value {
            AirfieldRepositoryError::IoError(reason) => Self::IoError(reason),
            AirfieldRepositoryError::NotFound => Self::UnknownAirfield,
            AirfieldRepositoryError::VersionConflict => Self::VersionConflict,
        }
    }
}

impl From<EventTryIntoError> for RegisterAirfieldError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
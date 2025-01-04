use thiserror::Error;
use prelude::data::GeoHash;
use prelude::domain::EventTryIntoError;
use crate::aggregate::{AirfieldId};
use crate::repository::AirfieldRepositoryError;

pub struct RegisterAirfield {
    pub id: AirfieldId,
    pub name: String,
    pub location: GeoHash,
}

#[derive(Error, Debug, PartialEq)]
pub enum RegisterAirfieldError {
    #[error("id conflict")]
    IdConflict,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("unknown airfield")]
    UnknownAirfield,

    #[error("{0}")]
    OtherError(String),

    #[error("version conflict")]
    VersionConflict,
}

pub type RegisterAirfieldResult = Result<AirfieldId, RegisterAirfieldError>;

// transformers
impl From<AirfieldRepositoryError> for RegisterAirfieldError {
    fn from(value: AirfieldRepositoryError) -> Self {
        match value {
            AirfieldRepositoryError::IoError(reason) => RegisterAirfieldError::IoError(reason),
            AirfieldRepositoryError::NotFound => RegisterAirfieldError::UnknownAirfield,
            AirfieldRepositoryError::VersionConflict => RegisterAirfieldError::VersionConflict,
        }
    }
}

impl From<EventTryIntoError> for RegisterAirfieldError {
    fn from(_: EventTryIntoError) -> Self {
        Self::IoError("unable to marshal event".to_owned())
    }
}
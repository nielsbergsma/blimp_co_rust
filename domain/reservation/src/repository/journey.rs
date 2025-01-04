use thiserror::Error;
use prelude::async_trait;
use prelude::domain::{Transaction};
use crate::aggregate::{Journey, JourneyId};

#[derive(Error, Debug, PartialEq)]
pub enum JourneyRepositoryError {
    #[error("I/O error {0}")]
    IoError(String),

    #[error("not found")]
    NotFound,

    #[error("version conflict")]
    VersionConflict,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait JourneyRepository {
    async fn get(&self, id: &JourneyId) -> Result<Option<Journey>, JourneyRepositoryError>;
    async fn set_begin(&self, id: &JourneyId) -> Result<Transaction<JourneyId, Journey>, JourneyRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<JourneyId, Journey>) -> Result<(), JourneyRepositoryError>;
}

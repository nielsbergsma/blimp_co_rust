use thiserror::Error;
use prelude::async_trait;
use prelude::domain::Transaction;
use crate::aggregate::{Airship, AirshipId};

#[derive(Error, Debug, PartialEq)]
pub enum AirshipRepositoryError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("not found")]
    NotFound,

    #[error("version conflict")]
    VersionConflict,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait AirshipRepository {
    async fn get(&self, id: &AirshipId) -> Result<Option<Airship>, AirshipRepositoryError>;
    async fn set_begin(&self, id: &AirshipId) -> Result<Transaction<AirshipId, Airship>, AirshipRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<AirshipId, Airship>) -> Result<(), AirshipRepositoryError>;
}
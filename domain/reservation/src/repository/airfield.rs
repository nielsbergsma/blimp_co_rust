use thiserror::Error;
use prelude::async_trait;
use prelude::domain::{Transaction};
use crate::aggregate::{Airfield, AirfieldId};

#[derive(Error, Debug, PartialEq)]
pub enum AirfieldRepositoryError {
    #[error("I/O error {0}")]
    IoError(String),

    #[error("not found")]
    NotFound,

    #[error("version conflict")]
    VersionConflict,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait AirfieldRepository {
    async fn get(&self, id: &AirfieldId) -> Result<Option<Airfield>, AirfieldRepositoryError>;
    async fn set_begin(&self, id: &AirfieldId) -> Result<Transaction<AirfieldId, Airfield>, AirfieldRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<AirfieldId, Airfield>) -> Result<(), AirfieldRepositoryError>;
}
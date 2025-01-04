use thiserror::Error;
use prelude::async_trait;
use prelude::domain::Transaction;
use crate::aggregate::{Flight, FlightId};

#[derive(Error, Debug, PartialEq)]
pub enum FlightRepositoryError {
    #[error("I/O error {0}")]
    IoError(String),

    #[error("version conflict")]
    VersionConflict,

    #[error("not found")]
    NotFound
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait FlightRepository {
    async fn get(&self, id: FlightId) -> Result<Option<Flight>, FlightRepositoryError>;
    async fn set_begin(&self, id: FlightId) -> Result<Transaction<FlightId, Flight>, FlightRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<FlightId, Flight>) -> Result<(), FlightRepositoryError>;
}
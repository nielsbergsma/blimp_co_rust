use thiserror::Error;
use prelude::async_trait;
use prelude::domain::{Transaction};
use crate::aggregate::{FlightAvailability, FlightId};

#[derive(Error, Debug, PartialEq)]
pub enum FlightAvailabilityRepositoryError {
    #[error("I/O error {0}")]
    IoError(String),

    #[error("not found")]
    NotFound,

    #[error("version conflict")]
    VersionConflict,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait FlightAvailabilityRepository {
    async fn get(&self, id: &FlightId) -> Result<Option<FlightAvailability>, FlightAvailabilityRepositoryError>;
    async fn set_begin(&self, id: &FlightId) -> Result<Transaction<FlightId, FlightAvailability>, FlightAvailabilityRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<FlightId, FlightAvailability>) -> Result<(), FlightAvailabilityRepositoryError>;
}

use thiserror::Error;
use prelude::async_trait;
use prelude::domain::{Transaction};
use crate::aggregate::{Reservation, ReservationId};

#[derive(Error, Debug, PartialEq)]
pub enum ReservationRepositoryError {
    #[error("I/O error {0}")]
    IoError(String),

    #[error("not found")]
    NotFound,

    #[error("version conflict")]
    VersionConflict,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ReservationRepository {
    async fn get(&self, id: &ReservationId) -> Result<Option<Reservation>, ReservationRepositoryError>;
    async fn set_begin(&self, id: &ReservationId) -> Result<Transaction<ReservationId, Reservation>, ReservationRepositoryError>;
    async fn set_commit(&self, transaction: Transaction<ReservationId, Reservation>) -> Result<(), ReservationRepositoryError>;
}

use serde::{Deserialize, Serialize};
use prelude::data::GeoHash;
use crate::aggregate::AirfieldId;

#[derive(Serialize, Deserialize)]
pub struct AirfieldRegisteredV1 {
    pub id: AirfieldId,
    pub name: String,
    pub location: GeoHash
}
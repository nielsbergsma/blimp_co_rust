use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use prelude::data::GeoHash;
use crate::aggregate::{AirfieldId, AirfieldName};

#[derive(Serialize, Deserialize)]
pub struct AirfieldRegisteredV1 {
    pub id: AirfieldId,
    pub name: AirfieldName,
    pub location: GeoHash,
}

impl TryInto<Event> for AirfieldRegisteredV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
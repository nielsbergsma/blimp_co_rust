use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{AirshipModel, AirshipName, AirshipNumberOfSeats, AirshipId};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AirshipAddedToFleetV1 {
    pub id: AirshipId,
    pub name: AirshipName,
    pub model: AirshipModel,
    pub number_of_seats: AirshipNumberOfSeats,
}

impl TryInto<Event> for AirshipAddedToFleetV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{Itinerary, JourneyId, Passengers, ReservationId};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReservationRevisedV1 {
    pub id: ReservationId,
    pub journey: JourneyId,
    pub passengers: Passengers,
    pub itinerary: Itinerary
}

impl TryInto<Event> for ReservationRevisedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
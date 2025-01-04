use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{Contact, Itinerary, JourneyId, Passengers, ReservationId, Revision};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReservationConfirmedV1 {
    pub id: ReservationId,
    pub journey: JourneyId,
    pub contact: Contact,
    pub passengers: Passengers,
    pub itinerary: Itinerary,
    pub revisions: Vec<Revision>
}

impl TryInto<Event> for ReservationConfirmedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
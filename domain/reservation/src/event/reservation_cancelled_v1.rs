use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{Contact, JourneyId, ReservationId};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReservationCancelledV1 {
    pub id: ReservationId,
    pub journey: JourneyId,
    pub contact: Contact
}

impl TryInto<Event> for ReservationCancelledV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
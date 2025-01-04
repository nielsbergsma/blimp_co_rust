use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError, Versioned};
use crate::aggregate::{FlightId, NumberOfSeats, ReservationId};

#[derive(Serialize, Deserialize)]
pub struct FlightReservationRequestedV1 {
    pub reservation: Versioned<ReservationId>,
    pub flight: FlightId,
    pub seats: NumberOfSeats
}

impl TryInto<Event> for FlightReservationRequestedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
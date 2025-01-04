use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError, Versioned};
use crate::aggregate::{FlightId, NumberOfSeats, ReservationId};

#[derive(Serialize, Deserialize)]
pub struct FlightReservedV1 {
    pub reservation: Versioned<ReservationId>,
    pub flight: FlightId,
    pub seats: NumberOfSeats,
}

impl FlightReservedV1 {
    pub fn annulled(&self) -> bool {
        self.seats == 0
    }
}

impl TryInto<Event> for FlightReservedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
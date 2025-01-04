use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError, Versioned};
use crate::aggregate::{AvailabilityFlightError, FlightId, ReservationId};

#[derive(Serialize, Deserialize)]
pub struct FlightReservationFailedV1 {
    pub reservation: Versioned<ReservationId>,
    pub flight: FlightId,
    pub reason: AvailabilityFlightError,
}

impl TryInto<Event> for FlightReservationFailedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
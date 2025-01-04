use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{FlightId, FlightRoute, NumberOfSeats};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FlightAvailabilityChangedV1 {
    pub flight: FlightId,
    pub route: FlightRoute,
    pub departure: DateTime<FixedOffset>,
    pub arrival: DateTime<FixedOffset>,
    pub seats_available: NumberOfSeats,
}

impl TryInto<Event> for FlightAvailabilityChangedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
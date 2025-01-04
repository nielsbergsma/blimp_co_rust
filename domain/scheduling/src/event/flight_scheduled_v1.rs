use serde::{Deserialize, Serialize};
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{AirshipId, AirshipNumberOfSeats, FlightArrival, FlightDeparture, FlightId};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FlightScheduledV1 {
    pub id: FlightId,
    pub departure: FlightDeparture,
    pub arrival: FlightArrival,
    pub airship: Airship,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Airship {
    pub id: AirshipId,
    pub number_of_seats: AirshipNumberOfSeats
}

impl TryInto<Event> for FlightScheduledV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
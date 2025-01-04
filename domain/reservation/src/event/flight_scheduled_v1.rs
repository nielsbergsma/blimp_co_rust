use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use crate::aggregate::{AirfieldId, Flight, FlightId, FlightRoute, NumberOfSeats};

#[derive(Serialize, Deserialize)]
pub struct FlightScheduledV1 {
    pub id: FlightId,
    pub departure: FlightDeparture,
    pub arrival: FlightArrival,
    pub airship: Airship,
}

impl From<FlightScheduledV1> for Flight {
    fn from(value: FlightScheduledV1) -> Self {
        Flight {
            id: value.id,
            route: FlightRoute {
                departure: value.departure.location,
                arrival: value.arrival.location,
            },
            departure: value.departure.time,
            arrival: value.arrival.time,
            seats: value.airship.number_of_seats,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FlightDeparture {
    pub location: AirfieldId,
    pub time: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize)]
pub struct FlightArrival {
    pub location: AirfieldId,
    pub time: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Airship {
    pub number_of_seats: NumberOfSeats
}
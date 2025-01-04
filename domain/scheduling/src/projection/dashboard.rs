use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use crate::event::Event;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Dashboard {
    airfields: Vec<Airfield>,
    airships: Vec<Airship>,
    flights: Vec<Flight>,
}

impl Dashboard {
    pub fn apply(mut self, event: Event) -> Self {
        match event {
            Event::AirfieldRegisteredV1(event) => {
                let airfield = Airfield{
                    id: event.id.to_string(),
                    name: event.name.to_string(),
                    location: event.location.to_string(),
                };

                self.airfields.retain(|a| a.id != airfield.id);
                self.airfields.push(airfield);
            }

            Event::AirshipAddedToFleetV1(event) => {
                let airship = Airship {
                    id: event.id.to_string(),
                    name: event.name.to_string(),
                    model: event.model.to_string(),
                    number_of_seats: event.number_of_seats.as_u8(),
                };

                self.airships.retain(|a| a.id != airship.id);
                self.airships.push(airship);
            }

            Event::FlightScheduledV1(event) => {
                let flight = Flight {
                    id: event.id.to_string(),
                    departure: FlightDeparture {
                        time: event.departure.time,
                        location: event.departure.location.to_string(),
                    },
                    arrival: FlightArrival {
                        time: event.arrival.time,
                        location: event.arrival.location.to_string(),
                    },
                    airship: event.airship.id.to_string(),
                };

                self.flights.retain(|f| f.id != flight.id);
                self.flights.push(flight);
            }
        }

        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Airfield {
    id: String,
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeoPosition {
    latitude: f32,
    longitude: f32
}

#[derive(Serialize, Deserialize, Debug)]
struct Airship {
    id: String,
    name: String,
    model: String,
    number_of_seats: u8
}

#[derive(Serialize, Deserialize, Debug)]
struct Flight {
    id: String,
    departure: FlightDeparture,
    arrival: FlightArrival,
    airship: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlightDeparture {
    time: DateTime<FixedOffset>,
    location: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlightArrival {
    time: DateTime<FixedOffset>,
    location: String,
}
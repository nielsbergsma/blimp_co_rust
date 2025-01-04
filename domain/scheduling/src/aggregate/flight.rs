use std::hash::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{Airfield, Airship, AirshipId};
use crate::aggregate::flight_arrival::FlightArrival;
use crate::aggregate::flight_departure::FlightDeparture;
use crate::aggregate::flight_id::FlightId;
use crate::event::{FlightScheduledV1, Airship as FlightScheduledV1Airship};

#[derive(Error, Debug, PartialEq)]
pub enum FlightError {
    #[error("departure and arrival location are the same")]
    SameDepartureAndArrivalLocation,

    #[error("departure is later then arrival")]
    DepartureIsLaterThenArrival,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Flight {
    pub id: FlightId,
    departure: FlightDeparture,
    arrival: FlightArrival,
    airship: AirshipId,
}

impl Flight {
    pub fn build(
        id: FlightId,
        departure_location: Airfield,
        departure_time: DateTime<FixedOffset>,
        arrival_location: Airfield,
        arrival_time: DateTime<FixedOffset>,
        airship: Airship
    ) -> Result<(Self, FlightScheduledV1), FlightError> {

        if departure_location == arrival_location {
            return Err(FlightError::SameDepartureAndArrivalLocation);
        }

        if departure_time >= arrival_time {
            return Err(FlightError::DepartureIsLaterThenArrival);
        }

        let flight = Self {
            id,
            departure: FlightDeparture {
                location: departure_location.id.clone(),
                time: departure_time,
            },
            arrival: FlightArrival {
                location: arrival_location.id.clone(),
                time: arrival_time,
            },
            airship: airship.id.clone(),
        };

        let event = FlightScheduledV1 {
            id,
            departure: FlightDeparture {
                location: departure_location.id,
                time: departure_time,
            },
            arrival: FlightArrival {
                location: arrival_location.id,
                time: arrival_time,
            },
            airship: FlightScheduledV1Airship {
                id: airship.id,
                number_of_seats: airship.number_of_seats,
            },
        };

        Ok((flight, event))
    }
}

impl PartialEq for Flight {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Flight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};
    use crate::aggregate::{Airfield, Airship, AirshipNumberOfSeats, FlightId};
    use crate::aggregate::flight::{Flight, FlightError};
    use prelude::domain::Event;

    #[test]
    fn equals_by_id() {
        let (flight1, _) = Flight::build(
            flight_id(),
            airfield_eham(),
            datetime_departure(),
            airfield_enli(),
            datetime_arrival(),
            airship(),
        ).unwrap();

        let (flight2, _) = Flight::build(
            flight_id(),
            airfield_enli(),
            datetime_departure(),
            airfield_eham(),
            datetime_arrival(),
            airship(),
        ).unwrap();

        let (flight3, _) = Flight::build(
            "5EPFciXgSxB70tAE8iERl9".parse().unwrap(),
            airfield_eham(),
            datetime_departure(),
            airfield_enli(),
            datetime_arrival(),
            airship(),
        ).unwrap();

        assert_eq!(flight1, flight2);
        assert_ne!(flight1, flight3);
    }

    #[test]
    fn errors_on_malformed_input() {
        // errors if departure and arrival is same location
        let result = Flight::build(
            flight_id(),
            airfield_eham(),
            datetime_departure(),
            airfield_eham(),
            datetime_arrival(),
            airship(),
        );
        assert_eq!(result, Err(FlightError::SameDepartureAndArrivalLocation));

        // errors if departure is later or equal then arrival
        let result = Flight::build(
            flight_id(),
            airfield_eham(),
            datetime_departure(),
            airfield_enli(),
            datetime_departure(),
            airship(),
        );
        assert_eq!(result, Err(FlightError::DepartureIsLaterThenArrival));
    }

    #[test]
    fn is_serializable() {
        let (original, _) = Flight::build(
            flight_id(),
            airfield_eham(),
            datetime_departure(),
            airfield_enli(),
            datetime_arrival(),
            airship(),
        ).unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Flight = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.departure, deserialized.departure);
        assert_eq!(original.arrival, deserialized.arrival);
        assert_eq!(original.airship, deserialized.airship);
    }

    #[test]
    fn returns_event_on_build() {
        let (_, event) = Flight::build(
            flight_id(),
            airfield_eham(),
            datetime_departure(),
            airfield_enli(),
            datetime_arrival(),
            airship(),
        ).unwrap();

        assert_eq!(event.id, flight_id());
        assert_eq!(event.departure.location, airfield_eham().id);
        assert_eq!(event.arrival.location, airfield_enli().id);
        assert_eq!(event.airship.id, airship().id);

        let data: Result<Event, _> = event.try_into();
        assert!(data.is_ok());
    }

    // test data
    fn flight_id() -> FlightId {
        "5EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn airship() -> Airship {
        let (airship, _) = Airship::build(
            "PH-1A1".parse().unwrap(),
            "Galaxy One".parse().unwrap(),
            "Blimp 1".parse().unwrap(),
            AirshipNumberOfSeats::try_from(10).unwrap(),
        );

        airship
    }

    fn datetime_departure() -> DateTime<FixedOffset> {
        "2024-01-08T09:00:00+05:00".parse().unwrap()
    }

    fn datetime_arrival() -> DateTime<FixedOffset> {
        "2024-01-08T11:00:00+05:00".parse().unwrap()
    }

    fn airfield_enli() -> Airfield {
        let (airfield, _) = Airfield::build(
            "ENLI".parse().unwrap(),
            "Farsund Airport".parse().unwrap(),
            "u4kdwc".parse().unwrap()
        );

        airfield
    }

    fn airfield_eham() -> Airfield {
        let (airfield, _) = Airfield::build(
            "EHAM".parse().unwrap(),
            "Amsterdam Airport".parse().unwrap(),
            "u173se".parse().unwrap()
        );

        airfield
    }
}
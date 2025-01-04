use std::hash::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{FlightId, FlightRoute, NumberOfSeats};

#[derive(Error, Debug, PartialEq)]
pub enum FlightError {
    #[error("arrival before departure")]
    ArrivalBeforeDeparture,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flight {
    pub id: FlightId,
    pub route: FlightRoute,
    pub departure: DateTime<FixedOffset>,
    pub arrival: DateTime<FixedOffset>,
    pub seats: NumberOfSeats,
}

impl Flight {
    pub fn build(id: FlightId, route: FlightRoute, departure: DateTime<FixedOffset>, arrival: DateTime<FixedOffset>, seats: NumberOfSeats) -> Result<Self, FlightError> {
        if departure >= arrival {
            return Err(FlightError::ArrivalBeforeDeparture);
        }

        Ok(Self{
            id,
            route,
            departure,
            arrival,
            seats,
        })
    }
}

impl PartialEq for Flight {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Flight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};
    use crate::aggregate::{Flight, FlightError, FlightId, FlightRoute, NumberOfSeats};

    #[test]
    fn is_buildable() {
        let flight = Flight::build(id(), route_eham_enli(), departure(), arrival(), seats());
        assert!(flight.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        let flight = Flight::build(id(), route_eham_enli(), departure(), departure(), seats());
        assert_eq!(flight, Err(FlightError::ArrivalBeforeDeparture));
    }

    #[test]
    fn equals_by_id() {
        let flight1 = Flight::build(id(), route_eham_enli(), departure(), arrival(), seats()).unwrap();
        let flight2 = Flight::build(id(), route_eli_eham(), departure(), arrival(), seats()).unwrap();
        assert_eq!(flight1, flight2);

        let flight3 = Flight::build(id2(), route_eham_enli(), departure(), arrival(), seats()).unwrap();
        assert_ne!(flight1, flight3);
    }

    #[test]
    fn is_serializable() {
        let original = Flight::build(id(), route_eham_enli(), departure(), arrival(), seats()).unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Flight = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.route, deserialized.route);
        assert_eq!(original.departure, deserialized.departure);
        assert_eq!(original.arrival, deserialized.arrival);
        assert_eq!(original.seats, deserialized.seats);
    }

    fn id() -> FlightId {
        "5EPFciXgSxB70tAE8iERl6".to_owned()
    }

    fn id2() -> FlightId {
        "5EPFciXgSxB70tAE8iERl7".to_owned()
    }

    fn route_eham_enli() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENLI".parse().unwrap()
        ).unwrap()
    }

    fn route_eli_eham() -> FlightRoute {
        FlightRoute::build(
            "ENLI".parse().unwrap(),
            "EHAM".parse().unwrap()
        ).unwrap()
    }

    fn departure() -> DateTime<FixedOffset> {
        "2024-01-08T09:00:00+05:00".parse().unwrap()
    }

    fn arrival() -> DateTime<FixedOffset> {
        "2024-01-08T11:00:00+05:00".parse().unwrap()
    }

    fn seats() -> NumberOfSeats {
        60u8
    }
}
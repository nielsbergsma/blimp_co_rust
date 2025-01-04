use std::iter;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{Flight, NumberOfSeats, ReservationId};
use crate::event::FlightAvailabilityChangedV1;

#[derive(Serialize, Deserialize, Error, Debug, PartialEq)]
pub enum AvailabilityFlightError {
    #[error("insufficient seats")]
    InsufficientSeats
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FlightAvailability {
    pub flight: Flight,
    pub seat_reservations: Vec<ReservationId>,
}

impl FlightAvailability {
    pub fn from_flight(flight: Flight) -> (FlightAvailability, FlightAvailabilityChangedV1) {
        let availability = FlightAvailability {
            flight: flight.clone(),
            seat_reservations: Vec::new(),
        };

        let event = FlightAvailabilityChangedV1 {
            flight: flight.id,
            route: flight.route,
            departure: flight.departure,
            arrival: flight.arrival,
            seats_available: availability.seats_available(),
        };

        (availability, event)
    }

    pub fn seats_available(&self) -> NumberOfSeats {
        self.flight.seats - self.seat_reservations.len() as NumberOfSeats
    }

    pub fn reserve(self, id: &ReservationId, seats: NumberOfSeats) -> Result<(Self, FlightAvailabilityChangedV1), AvailabilityFlightError> {
        let seats_reserved: Vec<ReservationId> = self.seat_reservations.clone()
            .into_iter()
            .filter(|r| r != id)
            .collect();

        let seats_available: NumberOfSeats = self.flight.seats - seats_reserved.len() as u8;
        if seats_available >= seats {
            let reservation_seats: Vec<ReservationId> = iter::repeat(*id).take(seats as usize).collect();

            let availability = Self {
                seat_reservations: [seats_reserved, reservation_seats].concat(),
                ..self.clone()
            };

            let event = FlightAvailabilityChangedV1 {
                flight: self.flight.id,
                route: self.flight.route,
                departure: self.flight.departure,
                arrival: self.flight.arrival,
                seats_available: availability.seats_available(),
            };

            Ok((availability, event))
        }
        else {
            Err(AvailabilityFlightError::InsufficientSeats)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{FlightAvailability, Flight, FlightRoute, ReservationId, AvailabilityFlightError};

    #[test]
    fn equality_by_flight() {
        let (availability1, _) = FlightAvailability::from_flight(flight());
        let (availability2, _) = FlightAvailability::from_flight(flight());
        let (availability2, _) = availability2.reserve(&reservation(), 2).unwrap();
        assert_eq!(availability1.flight, availability2.flight);

        let (availability3, _) = FlightAvailability::from_flight(flight2());
        assert_ne!(availability1.flight, availability3.flight);
    }

    #[test]
    fn is_serializable() {
        let (original, _) = FlightAvailability::from_flight(flight());
        let (original, _) = original.reserve(&reservation(), 2).unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: FlightAvailability = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.flight, deserialized.flight);
        assert_eq!(original.seat_reservations, deserialized.seat_reservations);
    }

    #[test]
    fn can_reserve_seats() {
        let (availability, _) = FlightAvailability::from_flight(flight());
        assert_eq!(availability.seats_available(), 10u8);

        let (availability, _) = availability.reserve(&reservation(), 4).unwrap();
        assert_eq!(availability.seats_available(), 6u8);
    }

    #[test]
    fn errors_on_insufficient_seats() {
        let (availability, _) = FlightAvailability::from_flight(flight());
        let result = availability.reserve(&reservation(), 255);
        assert_eq!(result, Err(AvailabilityFlightError::InsufficientSeats));
    }

    #[test]
    fn reserving_seats_is_idempotent() {
        let (availability, _) = FlightAvailability::from_flight(flight());
        let (availability, _) = availability.reserve(&reservation(), 4).unwrap();
        let (availability, _) = availability.reserve(&reservation(), 4).unwrap();
        let (availability, _) = availability.reserve(&reservation(), 4).unwrap();

        assert_eq!(availability.seats_available(), 6u8);
    }

    #[test]
    fn updating_reservation_is_absolute() {
        let (availability, _) = FlightAvailability::from_flight(flight());

        let (availability, _) = availability.reserve(&reservation(), 4).unwrap();
        assert_eq!(availability.seats_available(), 6u8);

        // change to 6 (expand); expect 4 remaining
        let (availability, _) = availability.reserve(&reservation(), 6).unwrap();
        assert_eq!(availability.seats_available(), 4u8);

        // change to 0 (cancel); expect 10 remaining
        let (availability, _) = availability.reserve(&reservation(), 0).unwrap();
        assert_eq!(availability.seats_available(), 10u8);
    }

    fn flight() -> Flight {
        let id = "5EPFciXgSxB70tAE8iERl6".to_owned();
        let route = FlightRoute::build("EHAM".parse().unwrap(), "ENLI".parse().unwrap()).unwrap();
        let departure = "2024-01-08T09:00:00+05:00".parse().unwrap();
        let arrival = "2024-01-08T11:00:00+05:00".parse().unwrap();
        let seats = 10u8;

        Flight::build(id, route, departure, arrival, seats).unwrap()
    }

    fn flight2() -> Flight {
        let id = "5EPFciXgSxB70tAE8iERl7".to_owned();
        let route = FlightRoute::build("ENLI".parse().unwrap(), "EHAM".parse().unwrap()).unwrap();
        let departure = "2024-02-08T09:00:00+05:00".parse().unwrap();
        let arrival = "2024-02-08T11:00:00+05:00".parse().unwrap();
        let seats = 10u8;

        Flight::build(id, route, departure, arrival, seats).unwrap()
    }

    fn reservation() -> ReservationId {
        "6APFciXgSxB70tAE8iERl1".parse().unwrap()
    }
}
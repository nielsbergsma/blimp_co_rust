use chrono::Utc;
use thiserror::Error;
use crate::aggregate::{ConfirmedReservation, Itinerary, Passengers, Reservation};

#[derive(Error, Debug, PartialEq)]
pub enum ReservationPolicyError {
    #[error("not allowed to confirm reservation shortly before departure")]
    NotAllowedToConfirmReservationShortlyBeforeDeparture,

    #[error("reservation is cancelled")]
    ReservationIsAlreadyCancelled,

    #[error("not allowed to revise reservation anymore")]
    NotAllowedToReviseReservationAnymore,

    #[error("not allowed to cancel reservation anymore")]
    NotAllowedToCancelReservationAnymore,

    #[error("malformed itinerary")]
    MalformedItinerary
}

pub type ReservationRevisionResult<T> = Result<T, ReservationPolicyError>;

#[derive(Clone, Copy)]
pub struct ReservationPolicy {
    pub revise_days_before_departure: i64,
    pub cancel_days_before_departure: i64,
}

impl ReservationPolicy {
    pub fn passenger() -> Self {
        Self {
            revise_days_before_departure: 7,
            cancel_days_before_departure: 1,
        }
    }

    pub fn agent() -> Self {
        Self {
            revise_days_before_departure: -365, // 1 year after
            cancel_days_before_departure: -365,
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            revise_days_before_departure: i64::MIN,
            cancel_days_before_departure: i64::MIN,
        }
    }
}

impl ReservationPolicy {
    pub fn new_confirmed<F, R>(&self, reservation: ConfirmedReservation, f: F) -> ReservationRevisionResult<R>
        where F: FnOnce(ConfirmedReservation) -> R {

        if self.within_revision_period(&reservation.itinerary) {
            Ok(f(reservation))
        }
        else {
            Err(ReservationPolicyError::NotAllowedToConfirmReservationShortlyBeforeDeparture)
        }
    }

    pub fn revise_passengers<F, R>(&self, reservation: Reservation, passengers: Passengers, f: F) -> ReservationRevisionResult<R>
        where F: FnOnce(Reservation, Passengers) -> R {

        match reservation {
            Reservation::Confirmed(reservation) => {
                if self.within_revision_period(&reservation.itinerary) {
                    let confirmed = Reservation::Confirmed(reservation);
                    Ok(f(confirmed, passengers))
                }
                else {
                    Err(ReservationPolicyError::NotAllowedToReviseReservationAnymore)
                }
            }

            Reservation::Cancelled(_) => {
                Err(ReservationPolicyError::ReservationIsAlreadyCancelled)
            }
        }
    }

    pub fn revise_itinerary<F, R>(&self, reservation: Reservation, itinerary: Itinerary, f: F) -> ReservationRevisionResult<R>
        where F: FnOnce(Reservation, Itinerary) -> R {

        match reservation {
            Reservation::Confirmed(reservation) => {
                if self.within_revision_period(&reservation.itinerary) && self.within_revision_period(&itinerary) {
                    let confirmed = Reservation::Confirmed(reservation);
                    Ok(f(confirmed, itinerary))
                }
                else {
                    Err(ReservationPolicyError::NotAllowedToReviseReservationAnymore)
                }
            }

            Reservation::Cancelled(_) => {
                Err(ReservationPolicyError::ReservationIsAlreadyCancelled)
            }
        }
    }

    pub fn cancel<F, R>(&self, reservation: Reservation, f: F) -> ReservationRevisionResult<R>
        where F: FnOnce(Reservation) -> R {

        match reservation {
            Reservation::Confirmed(reservation) => {
                if self.within_cancellation_period(&reservation.itinerary) {
                    let confirmed = Reservation::Confirmed(reservation);
                    Ok(f(confirmed))
                }
                else {
                    Err(ReservationPolicyError::NotAllowedToCancelReservationAnymore)
                }
            }

            Reservation::Cancelled(_) => {
                Err(ReservationPolicyError::ReservationIsAlreadyCancelled)
            }
        }
    }

    fn within_revision_period(&self, itinerary: &Itinerary) -> bool {
        let today = Utc::now().date_naive();
        let days_before_departure = (itinerary.departure_date() - today).num_days();

        days_before_departure >= self.revise_days_before_departure
    }

    fn within_cancellation_period(&self, itinerary: &Itinerary) -> bool {
        let today = Utc::now().date_naive();
        let days_before_departure = (itinerary.departure_date() - today).num_days();

        days_before_departure >= self.cancel_days_before_departure
    }
}


#[cfg(test)]
mod tests {
    use std::collections::{LinkedList};
    use chrono::{Days, NaiveDate, Utc};
    use prelude::collection::SortedSet;
    use crate::aggregate::{Contact, Flight, FlightRoute, Itinerary, Journey, JourneyId, PassengerArrangement, Passengers, Reservation, ReservationId, Segment};
    use crate::policy::{ReservationPolicy, ReservationPolicyError};

    #[test]
    fn passengers_can_confirm_reservation_7days_before_departure() {
        let today = Utc::now().date_naive();

        // 7 days before is allowed
        let departure_date = today.checked_add_days(Days::new(7)).unwrap();
        let reservation = Reservation::new_confirmed(
            &ReservationPolicy::passenger(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        );
        assert!(reservation.is_ok());

        // 6 days before is not allowed
        let departure_date = today.checked_add_days(Days::new(6)).unwrap();
        let reservation = Reservation::new_confirmed(
            &ReservationPolicy::passenger(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        );
        assert_eq!(reservation, Err(ReservationPolicyError::NotAllowedToConfirmReservationShortlyBeforeDeparture));
    }

    #[test]
    fn passengers_can_revise_reservation_7days_before_departure() {
        let today = Utc::now().date_naive();

        // 7 days before is allowed
        let departure_date = today.checked_add_days(Days::new(7)).unwrap();
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        ).unwrap();

        let revised = reservation.revise_passengers(
            &ReservationPolicy::passenger(),
            passengers2()
        );
        assert!(revised.is_ok());

        // 6 days before is not allowed
        let departure_date = today.checked_add_days(Days::new(6)).unwrap();
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        ).unwrap();

        let revised = reservation.revise_passengers(
            &ReservationPolicy::passenger(),
            passengers2()
        );
        assert_eq!(revised, Err(ReservationPolicyError::NotAllowedToReviseReservationAnymore));
    }

    #[test]
    fn passengers_can_cancel_reservation_before_departure() {
        let today = Utc::now().date_naive();

        // 7 days before is allowed
        let departure_date = today.checked_add_days(Days::new(1)).unwrap();
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        ).unwrap();

        let cancelled = reservation.cancel(&ReservationPolicy::passenger());
        assert!(cancelled.is_ok());

        // 6 days before is not allowed
        let departure_date = today.checked_add_days(Days::new(0)).unwrap();
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary(departure_date)
        ).unwrap();

        let cancelled = reservation.cancel(&ReservationPolicy::passenger());
        assert_eq!(cancelled, Err(ReservationPolicyError::NotAllowedToCancelReservationAnymore));
    }

    fn id() -> ReservationId {
        "5FFFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn itinerary(date: NaiveDate) -> Itinerary {
        let stages = LinkedList::from([
            (flight_eham_enli(date), None),
        ]);

        let journey = journey();
        journey.parse_itinerary(stages).unwrap()
    }

    fn contact() -> Contact {
        Contact::new(
            "Niels Bergsma".parse().unwrap(),
            "n.bergsma@internet.com".parse().unwrap(),
            None,
        )
    }

    fn passengers() -> PassengerArrangement {
        PassengerArrangement::build(2, 0).unwrap()
    }

    fn passengers2() -> Passengers {
        Passengers::new(PassengerArrangement::build(4, 0).unwrap())
    }

    fn journey_id() -> JourneyId {
        "5EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn journey() -> Journey {
        let (journey, _) = Journey::build(
            journey_id(),
            "Journey Around North Atlantic".parse().unwrap(),
            SortedSet::empty()
                .insert(Segment::build(route_eham_enli(), SortedSet::empty()).unwrap())
                .insert(Segment::build(route_enli_eham(), SortedSet::empty()).unwrap())
        ).unwrap();

        journey
    }

    fn route_eham_enli() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENLI".parse().unwrap(),
        ).unwrap()
    }

    fn route_enli_eham() -> FlightRoute {
        FlightRoute::build(
            "ENLI".parse().unwrap(),
            "EHAM".parse().unwrap(),
        ).unwrap()
    }

    fn flight_eham_enli(date: NaiveDate) -> Flight {
        let date_as_rfc3339 = date.to_string();

        Flight::build(
            "9EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_eham_enli(),
            [&date_as_rfc3339, "T09:00:00+05:00"].concat().parse().unwrap(),
            [&date_as_rfc3339, "T11:00:00+05:00"].concat().parse().unwrap(),
            10u8,
        ).unwrap()
    }
}
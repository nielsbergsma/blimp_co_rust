use std::cmp::Ordering;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use prelude::domain::Version;
use crate::aggregate::{Contact, EmailVerificationError, FlightId, Itinerary, ItineraryStageError, JourneyId, PassengerArrangement, Passengers, PhoneNumber, ReservationId};
use crate::event::{ReservationCancelledV1, ReservationConfirmedV1, ReservationRevisedV1};
use crate::policy::{ReservationPolicy, ReservationRevisionResult};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Reservation {
    Confirmed(ConfirmedReservation),
    Cancelled(CancelledReservation),
}

impl Reservation {
    pub fn new_confirmed(policy: &ReservationPolicy, id: ReservationId, journey: JourneyId, contact: Contact, passengers: PassengerArrangement, itinerary: Itinerary) -> ReservationRevisionResult<(Self, ReservationConfirmedV1)> {
        let reservation = ConfirmedReservation{
            id,
            journey,
            contact: contact.clone(),
            passengers: Passengers::Arrangement(passengers.clone()),
            itinerary: itinerary.clone(),
            revisions: Vec::default(),
        };

        let reservation = policy.new_confirmed(reservation, |reservation| {
            Self::Confirmed(reservation)
        })?;

        let event =  ReservationConfirmedV1 {
            id,
            journey,
            contact,
            passengers: Passengers::Arrangement(passengers),
            itinerary,
            revisions: Vec::default(),
        };

        Ok((reservation, event))
    }

    pub fn id(&self) -> ReservationId {
        match self {
            Self::Confirmed(reservation) => reservation.id,
            Self::Cancelled(reservation) => reservation.id,
        }
    }

    pub fn journey(&self) -> JourneyId {
        match self {
            Self::Confirmed(reservation) => reservation.journey,
            Self::Cancelled(reservation) => reservation.journey,
        }
    }

    pub fn revise_itinerary(self, policy: &ReservationPolicy, itinerary: Itinerary) -> ReservationRevisionResult<(Self, Option<ReservationRevisedV1>)> {
        policy.revise_itinerary(self, itinerary, |reservation, itinerary| {
            match reservation {
                Self::Confirmed(reservation) if !reservation.itinerary.equivalent(&itinerary) => {
                    let state = Self::Confirmed(ConfirmedReservation{
                        itinerary: itinerary.clone(),
                        passengers: reservation.passengers.clone(),
                        revisions: [
                            vec!((reservation.revisions.len() as Version, reservation.passengers.clone(), reservation.itinerary.clone())),
                            reservation.revisions
                        ].concat(),
                        ..reservation
                    });

                    let event = ReservationRevisedV1 {
                        id: reservation.id,
                        journey: reservation.journey,
                        passengers: reservation.passengers,
                        itinerary,
                    };

                    (state, Some(event))
                }

                confirmed@Self::Confirmed(_) => {
                    (confirmed, None)
                }

                cancelled@Self::Cancelled(_) => {
                    (cancelled, None)
                }
            }
        })
    }

    pub fn revise_passengers(self, policy: &ReservationPolicy, passengers: Passengers) -> ReservationRevisionResult<(Self, Option<ReservationRevisedV1>)> {
        policy.revise_passengers(self, passengers,|reservation, passengers| {
            match reservation {
                Self::Confirmed(reservation) if reservation.passengers != passengers => {
                    let state = Self::Confirmed(ConfirmedReservation {
                        passengers: passengers.clone(),
                        itinerary: reservation.itinerary.planned(),
                        revisions: [
                            vec!((reservation.revisions.len() as Version, reservation.passengers, reservation.itinerary.clone())),
                            reservation.revisions
                        ].concat(),
                        ..reservation
                    });

                    let event = ReservationRevisedV1 {
                        id: reservation.id,
                        journey: reservation.journey,
                        passengers,
                        itinerary: reservation.itinerary.planned(),
                    };

                    (state, Some(event))
                }

                confirmed@Self::Confirmed(_) => {
                    (confirmed, None)
                }

                cancelled@Self::Cancelled(_) => {
                    (cancelled, None)
                }
            }
        })
    }

    pub fn cancel(self, policy: &ReservationPolicy) -> ReservationRevisionResult<(Self, Option<ReservationCancelledV1>)> {
        policy.cancel(self, |reservation| {
            match reservation {
                Self::Confirmed(reservation) => {
                    let state = Self::Cancelled(CancelledReservation {
                        id: reservation.id,
                        journey: reservation.journey,
                        contact: reservation.contact.clone(),
                        revisions: [
                            vec!((reservation.revisions.len() as Version, reservation.passengers, reservation.itinerary.clone())),
                            reservation.revisions
                        ].concat(),
                        time: Utc::now(),
                    });

                    let event = ReservationCancelledV1 {
                        id: reservation.id,
                        journey: reservation.journey,
                        contact: reservation.contact,
                    };

                    (state, Some(event))
                }

                cancelled@Self::Cancelled(_) => {
                    (cancelled, None)
                }
            }
        })
    }

    pub fn contact_email_verify_challenge(&self) -> Option<String> {
        match self {
            Self::Confirmed(reservation) => {
                reservation.contact.email_verify_challenge()
            }

            Self::Cancelled(reservation) => {
                reservation.contact.email_verify_challenge()
            }
        }
    }

    pub fn verify_contact_email(self, challenge: String) -> Result<Reservation, EmailVerificationError> {
        match self {
            Self::Confirmed(reservation) => {
                Ok(Self::Confirmed(ConfirmedReservation {
                    contact: reservation.contact.verify_email(challenge)?,
                    ..reservation
                }))
            }

            Self::Cancelled(reservation) => {
                Ok(Self::Cancelled(CancelledReservation {
                    contact: reservation.contact.verify_email(challenge)?,
                    ..reservation
                }))
            }
        }
    }

    pub fn set_contact_phone(self, phone: PhoneNumber) -> Self {
        match self {
            Self::Confirmed(reservation) => {
                Self::Confirmed(ConfirmedReservation {
                    contact: reservation.contact.set_phone(phone),
                    ..reservation
                })
            }

            Self::Cancelled(reservation) => {
                Self::Cancelled(CancelledReservation {
                    contact: reservation.contact.set_phone(phone),
                    ..reservation
                })
            }
        }
    }

    pub fn mark_flight_as_reserved(self, flight: &FlightId, version: Version) -> Self {
        self.mark_flight(flight, version, |itinerary| itinerary.mark_flight_as_reserved(flight))
    }

    pub fn mark_flight_as_reserved_failed(self, flight: &FlightId, reason: &ItineraryStageError, version: Version) -> Self {
        self.mark_flight(flight, version, |itinerary| itinerary.mark_flight_as_reserved_failed(flight, reason))
    }

    pub fn mark_flight_as_annulled(self, flight: &FlightId, version: Version) -> Self {
        self.mark_flight(flight, version, |itinerary| itinerary.mark_flight_as_annulled(flight))
    }

    fn mark_flight<F>(self, flight: &FlightId, version: Version, mark: F) -> Self
        where F: Fn(Itinerary) -> Itinerary {
        let current_version = self.version();

        match self {
            Self::Confirmed(mut reservation) => {
                if current_version == version {
                    reservation.itinerary = mark(reservation.itinerary);
                }

                reservation.revisions = reservation.revisions.into_iter()
                    .map(|(revision_version, passengers, itinerary)| {
                        match revision_version.cmp(&version) {
                            Ordering::Less => (revision_version, passengers, itinerary.mark_flight_as_annulled(flight)),
                            Ordering::Equal => (revision_version, passengers, mark(itinerary)),
                            Ordering::Greater => (revision_version, passengers, itinerary),
                        }
                    })
                    .collect();

                Self::Confirmed(reservation)
            }

            Self::Cancelled(mut reservation) => {
                reservation.revisions = reservation.revisions.into_iter()
                    .map(|(revision_version, passengers, itinerary)| {
                        match revision_version.cmp(&version) {
                            Ordering::Less => (revision_version, passengers, itinerary.mark_flight_as_annulled(flight)),
                            Ordering::Equal => (revision_version, passengers, mark(itinerary)),
                            Ordering::Greater => (revision_version, passengers, itinerary),
                        }
                    })
                    .collect();

                Self::Cancelled(reservation)
            }
        }
    }

    pub fn version(&self) -> Version {
        match self {
            Self::Confirmed(reservation) => reservation.revisions.len() as Version,
            Self::Cancelled(reservation) => reservation.revisions.len() as Version
        }
    }
}

impl PartialEq for Reservation {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmedReservation {
    pub id: ReservationId,
    pub journey: JourneyId,
    pub contact: Contact,
    pub passengers: Passengers,
    pub itinerary: Itinerary,
    pub revisions: Vec<Revision>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelledReservation {
    pub id: ReservationId,
    pub journey: JourneyId,
    pub contact: Contact,
    pub time: DateTime<Utc>,
    pub revisions: Vec<Revision>
}

pub type Revision = (Version, Passengers, Itinerary);


#[cfg(test)]
mod tests {
    use std::collections::LinkedList;
    use prelude::collection::SortedSet;
    use crate::aggregate::{Contact, Flight, FlightId, FlightRoute, Itinerary, ItineraryStage, ItineraryStageError, Journey, JourneyId, PassengerArrangement, Passengers, PhoneNumber, Reservation, ReservationId, Segment};
    use crate::policy::ReservationPolicy;

    #[test]
    fn can_create_new_confirmed_reservation() {
        let reservation = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        );
        assert!(reservation.is_ok())
    }

    #[test]
    fn equals_by_id() {
        let (reservation1, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();
        let (reservation2, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary2()
        ).unwrap();
        assert_eq!(reservation1, reservation2);

        let (reservation3, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id2(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();
        assert_ne!(reservation1, reservation3);
    }

    #[test]
    fn is_serializable() {
        let (original, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Reservation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized)
    }

    #[test]
    fn tracks_revisions() {
        // starts with 0
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();
        assert_eq!(reservation.version(), 0);

        // change passengers
        let (reservation, event) = reservation.revise_passengers(
            &ReservationPolicy::test(),
            passengers2()
        ).unwrap();
        assert_eq!(reservation.version(), 1);
        assert!(event.is_some());

        // change itinerary
        let (reservation, event) = reservation.revise_itinerary(
            &ReservationPolicy::test(),
            itinerary2()
        ).unwrap();
        assert_eq!(reservation.version(), 2);
        assert!(event.is_some());

        // cancel
        let (reservation, event) = reservation.cancel(
            &ReservationPolicy::test()
        ).unwrap();
        assert_eq!(reservation.version(), 3);
        assert!(event.is_some())
    }

    #[test]
    fn can_revise_passengers() {
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let (reservation, _) = reservation.revise_passengers(
            &ReservationPolicy::test(),
            passengers2()
        ).unwrap();

        assert!(match reservation {
            Reservation::Confirmed(confirmed) => confirmed.passengers == passengers2(),
            _ => false
        });
    }

    #[test]
    fn can_revise_itinerary() {
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let (reservation, _) = reservation.revise_itinerary(
            &ReservationPolicy::test(),
            itinerary2()
        ).unwrap();

        assert!(match reservation {
            Reservation::Confirmed(confirmed) => confirmed.itinerary == itinerary2(),
            _ => false
        });
    }

    #[test]
    fn can_cancel() {
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let (reservation, _) = reservation.cancel(
            &ReservationPolicy::test(),
        ).unwrap();

        assert!(match reservation {
            Reservation::Cancelled(_) => true,
            _ => false
        });
    }

    #[test]
    fn can_verify_contact_email_address() {
        let (reservation, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let challenge = reservation.contact_email_verify_challenge();
        assert_eq!(challenge, Some("9531101938472543805".to_owned()));

        let result = reservation.verify_contact_email("9531101938472543805".to_owned());
        assert!(result.is_ok());

        let reservation = result.unwrap();
        assert!(match reservation {
            Reservation::Confirmed(confirmed) => confirmed.contact.email_is_verified(),
            _ => false
        });
    }

    #[test]
    fn can_set_contact_phone() {
        let (before, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        assert!(match &before {
            Reservation::Confirmed(confirmed) => !confirmed.contact.phone_is_present(),
            _ => false
        });

        let after = before.set_contact_phone(phone());
        assert!(match &after {
            Reservation::Confirmed(confirmed) => confirmed.contact.phone_is_present(),
            _ => false
        });
    }

    #[test]
    fn can_reserve_flight_in_itinerary() {
        let (before, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let after = before.mark_flight_as_reserved(
            &flight_id_eham_enli_8may(), 0
        );

        assert!(match &after {
            Reservation::Confirmed(confirmed) =>
                match confirmed.itinerary.first_stage() {
                    ItineraryStage::Reserved(_, _) => true,
                    _ => false
                }
            _ => false
        });

        // is idempotent
        let after2 = after.mark_flight_as_reserved(
            &flight_id_eham_enli_8may(), 0
        );

        assert!(match &after2 {
            Reservation::Confirmed(confirmed) =>
                match confirmed.itinerary.first_stage() {
                    ItineraryStage::Reserved(_, _) => true,
                    _ => false
                }
            _ => false
        });
    }

    #[test]
    fn skip_reserve_flights_for_other_versions() {
        let (before, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let after = before.mark_flight_as_reserved(
            &flight_id_eham_enli_8may(), 99
        );

        assert!(match &after {
            Reservation::Confirmed(confirmed) =>
                match confirmed.itinerary.first_stage() {
                    ItineraryStage::Planned(_, _) => true,
                    _ => false
                }
            _ => false
        });
    }

    #[test]
    fn can_fail_reserve_flight_in_itinerary() {
        let (before, _) = Reservation::new_confirmed(
            &ReservationPolicy::test(),
            id(),
            journey_id(),
            contact(),
            passengers(),
            itinerary()
        ).unwrap();

        let after = before.mark_flight_as_reserved_failed(
            &flight_id_eham_enli_8may(), &ItineraryStageError::InsufficientSeats, 0
        );

        assert!(match &after {
            Reservation::Confirmed(confirmed) =>
                match confirmed.itinerary.first_stage() {
                    ItineraryStage::ReservedFailed(_, _, reason) => reason == &ItineraryStageError::InsufficientSeats,
                    _ => false
                }
            _ => false
        });

        // is idempotent
        let after2 = after.mark_flight_as_reserved_failed(
            &flight_id_eham_enli_8may(), &ItineraryStageError::InsufficientSeats, 0
        );

        assert!(match &after2 {
            Reservation::Confirmed(confirmed) =>
                match confirmed.itinerary.first_stage() {
                    ItineraryStage::ReservedFailed(_, _, reason) => reason == &ItineraryStageError::InsufficientSeats,
                    _ => false
                }
            _ => false
        });
    }

    fn id() -> ReservationId {
        "5FFFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn id2() -> ReservationId {
        "5FFFciXgSxB70tAE8iERl7".parse().unwrap()
    }

    fn itinerary() -> Itinerary {
        let stages = LinkedList::from([
            (flight_eham_enli_8may(), None),
            (flight_enli_eham_14may(), None),
        ]);

        let journey = journey();
        journey.parse_itinerary(stages).unwrap()
    }

    fn itinerary2() -> Itinerary {
        let stages = LinkedList::from([
            (flight_enli_eham_14may(), None),
            (flight_eham_enli_20may(), None)
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
        Passengers::Arrangement(PassengerArrangement::build(4, 0).unwrap())
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

    fn flight_id_eham_enli_8may() -> FlightId {
        "9EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn flight_eham_enli_8may() -> Flight {
        Flight::build(
            flight_id_eham_enli_8may(),
            route_eham_enli(),
            "2024-05-08T09:00:00+05:00".parse().unwrap(),
            "2024-05-08T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_id_enli_eham_14may() -> FlightId {
        "8EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn flight_enli_eham_14may() -> Flight {
        Flight::build(
            flight_id_enli_eham_14may(),
            route_enli_eham(),
            "2024-05-14T09:00:00+05:00".parse().unwrap(),
            "2024-05-14T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_eham_enli_20may() -> Flight {
        Flight::build(
            "9EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_eham_enli(),
            "2024-05-20T09:00:00+05:00".parse().unwrap(),
            "2024-05-20T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn phone() -> PhoneNumber {
        "+31653321799".parse().unwrap()
    }
}

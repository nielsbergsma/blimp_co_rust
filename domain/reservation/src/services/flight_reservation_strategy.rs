use std::slice::Iter;
use prelude::domain::{Version, Versioned};
use crate::aggregate::{Itinerary, ItineraryStage, Passengers, Reservation, ReservationId, Revision};
use crate::event::FlightReservationRequestedV1;

/// next request inspects the current state of a reservation against
/// the target state, and returns a single action towards that.
///
/// for a confirmed reservation that means:
/// - stages in latest itinerary = reserved
/// - stages in older revisions = annulled
///
/// for a cancelled reservation that means:
/// - all stages from older revisions = annulled
pub fn next_request(reservation: &Reservation) -> Option<FlightReservationRequestedV1> {
    let id = reservation.id();
    let version = reservation.version();

    match reservation {
        Reservation::Confirmed(reservation) => {
            next_reserved(&id, &version, &reservation.itinerary, &reservation.passengers)
                .or_else(|| next_annulment(&id, reservation.revisions.iter()))
        }

        Reservation::Cancelled(reservation) => {
            next_annulment(&id, reservation.revisions.iter())
        }
    }
}

fn next_reserved(id: &ReservationId, version: &Version, itinerary: &Itinerary, passengers: &Passengers,) -> Option<FlightReservationRequestedV1> {
    itinerary.stages()
        .filter_map(|stage| {
            match stage {
                ItineraryStage::Planned(flight, _) | ItineraryStage::Annulled(flight, _) => {
                    Some(FlightReservationRequestedV1 {
                        reservation: Versioned::from_version(*id, *version),
                        flight: flight.id.clone(),
                        seats: passengers.count(),
                    })
                }

                ItineraryStage::Reserved(_, _) | ItineraryStage::ReservedFailed(_, _, _)=> {
                    None
                }
            }
        })
        .next()
}

fn next_annulment(id: &ReservationId, revisions: Iter<Revision>) -> Option<FlightReservationRequestedV1> {
    revisions
        .flat_map(|(version, _, itinerary)| {
            itinerary.stages().map(move |stage| (version, stage))
        })
        .filter_map(|(version, stage)| {
            match stage {
                ItineraryStage::Planned(flight, _)
                | ItineraryStage::Reserved(flight, _)
                | ItineraryStage::ReservedFailed(flight, _, _) => {
                    Some(FlightReservationRequestedV1 {
                        reservation: Versioned::from_version(*id, *version),
                        flight: flight.clone().id,
                        seats: 0,
                    })
                }

                ItineraryStage::Annulled(_, _) => {
                    None
                }
            }
        })
        .next()
}
mod publish_journey;
mod make_flight_available;
mod register_airfield;
mod confirm_reservation;
mod reserve_flight;
mod get_reservation;
mod cancel_reservation;
mod revise_passengers;
mod revise_itinerary;

use std::collections::LinkedList;
pub use publish_journey::*;
pub use make_flight_available::*;
pub use register_airfield::*;
pub use confirm_reservation::*;
pub use reserve_flight::*;
pub use get_reservation::*;
pub use cancel_reservation::*;
pub use revise_passengers::*;
pub use revise_itinerary::*;
use crate::aggregate::{Accommodation, AccommodationId, Flight, FlightId};

pub type ReferencedItineraryStage = (FlightId, Option<AccommodationId>);
pub type ResolvedItineraryStage = (Flight, Option<Accommodation>);
pub type Itinerary<T> = LinkedList<T>;
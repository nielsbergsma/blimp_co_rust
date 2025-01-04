mod journey_published_v1;
mod flight_scheduled_v1;
mod flight_availability_changed_v1;
mod airfield_registered_v1;
mod reservation_confirmed_v1;
mod flight_reservation_requested_v1;
mod flight_reserved_v1;
mod flight_reservation_failed_v1;
mod reservation_cancelled_v1;
mod reservation_revised_v1;

use serde::Deserialize;
pub use journey_published_v1::*;
pub use flight_scheduled_v1::*;
pub use flight_availability_changed_v1::*;
pub use airfield_registered_v1::*;
pub use reservation_confirmed_v1::*;
pub use reservation_revised_v1::*;
pub use reservation_cancelled_v1::*;
pub use flight_reservation_requested_v1::*;
pub use flight_reserved_v1::*;
pub use flight_reservation_failed_v1::*;

#[derive(Deserialize)]
pub struct RawEvent(String);

impl RawEvent {
    pub fn deserialize(&self) -> Result<Event, serde_json::Error> {
        serde_json::from_str(&self.0)
    }
}

#[derive(Deserialize)]
pub enum Event {
    JourneyPublishedV1(JourneyPublishedV1),
    AirfieldRegisteredV1(AirfieldRegisteredV1),
    FlightScheduledV1(FlightScheduledV1),
    FlightAvailabilityChangedV1(FlightAvailabilityChangedV1),
    ReservationConfirmedV1(ReservationConfirmedV1),
    ReservationRevisedV1(ReservationRevisedV1),
    ReservationCancelledV1(ReservationCancelledV1),
    FlightReservationRequestedV1(FlightReservationRequestedV1),
    FlightReservedV1(FlightReservedV1),
    FlightReservationFailedV1(FlightReservationFailedV1),
}
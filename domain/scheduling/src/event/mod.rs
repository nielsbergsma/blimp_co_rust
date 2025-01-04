mod flight_scheduled_v1;
mod airship_added_to_fleet_v1;
mod airfield_registered_v1;

use serde::Deserialize;
pub use airfield_registered_v1::*;
pub use flight_scheduled_v1::*;
pub use airship_added_to_fleet_v1::*;

#[derive(Deserialize)]
pub struct RawEvent(String);

impl RawEvent {
    pub fn deserialize(&self) -> Result<Event, serde_json::Error> {
        serde_json::from_str(&self.0)
    }
}

#[derive(Deserialize)]
pub enum Event {
    AirfieldRegisteredV1(AirfieldRegisteredV1),
    AirshipAddedToFleetV1(AirshipAddedToFleetV1),
    FlightScheduledV1(FlightScheduledV1),
}
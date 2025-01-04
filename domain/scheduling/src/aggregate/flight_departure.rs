use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use crate::aggregate::airfield_id::AirfieldId;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FlightDeparture {
    pub location: AirfieldId,
    pub time: DateTime<FixedOffset>,
}

impl FlightDeparture {
    pub fn build(location: AirfieldId, time: DateTime<FixedOffset>) -> Self {
        Self {
            location,
            time,
        }
    }
}
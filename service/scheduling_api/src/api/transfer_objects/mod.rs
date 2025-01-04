use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub error: String
}

#[derive(Serialize, Deserialize)]
pub struct PostAirshipsRequest {
    pub id: String,
    pub name: String,
    pub model: String,
    pub number_of_seats: u8,
}

#[derive(Serialize, Deserialize)]
pub struct PostAirshipsResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostAirfieldsRequest {
    pub id: String,
    pub name: String,
    pub location: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostAirfieldsResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostFlightsRequest {
    pub departure_location: String,
    pub departure_time: DateTime<FixedOffset>,
    pub arrival_location: String,
    pub arrival_time: DateTime<FixedOffset>,
    pub airship: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostFlightsResponse {
    pub id: String
}

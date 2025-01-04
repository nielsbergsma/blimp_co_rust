use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use chrono::{Datelike, DateTime, FixedOffset, Month, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeSeq;
use crate::event::Event;

#[derive(Serialize, Deserialize)]
pub struct Availability {
    period: YearMonth,
    flights: HashMap<FlightRoute, FlightRouteAvailability>
}

impl Availability {
    pub fn from_period(period: YearMonth) -> Self {
        Self {
            period,
            flights: HashMap::new(),
        }
    }

    pub fn period(&self) -> YearMonth {
        self.period
    }

    pub fn apply(mut self, event: Event) -> Self {
        match event {
            Event::FlightAvailabilityChangedV1(flight) => {
                let route = FlightRoute::from_aggregate(flight.route);
                let availability = FlightAvailability {
                    id: flight.flight.to_string(),
                    departure: flight.departure,
                    arrival: flight.arrival,
                    seats_available: flight.seats_available,
                };

                self.flights.entry(route)
                    .and_modify(|a| { a.replace(availability.clone()); })
                    .or_insert(HashSet::from([availability]));
            }

            _ => {
                // ignore unknown events
            }
        }

        self
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct YearMonth(i32, Month);

impl YearMonth {
    pub fn from_naive_date(value: NaiveDate) -> Self {
        let year = value.year();
        let month = (value.month() as u8).try_into()
            .expect("should be a valid month number");

        Self(year, month)
    }

    pub fn from_datetime(value: DateTime<FixedOffset>) -> Self {
        Self::from_naive_date(value.date_naive())
    }

    pub fn year(&self) -> i32 {
        self.0
    }

    pub fn month(&self) -> Month {
        self.1
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
struct FlightRoute(String);

impl FlightRoute {
    fn from_aggregate(value: crate::aggregate::FlightRoute) -> Self {
        Self([value.departure.to_string(), "-".to_owned(), value.arrival.to_string()].concat())
    }
}

type FlightRouteAvailability = HashSet<FlightAvailability>;

#[derive(Eq, Clone)]
struct FlightAvailability {
    id: String,
    departure: DateTime<FixedOffset>,
    arrival: DateTime<FixedOffset>,
    seats_available: u8
}

impl PartialEq for FlightAvailability{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for FlightAvailability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Serialize for FlightAvailability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.departure)?;
        seq.serialize_element(&self.arrival)?;
        seq.serialize_element(&self.seats_available)?;
        seq.end()
    }
}

impl <'de> Deserialize<'de> for FlightAvailability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let fields: (String, DateTime<FixedOffset>, DateTime<FixedOffset>, u8) = Deserialize::deserialize(deserializer)?;

        Ok(FlightAvailability {
            id: fields.0,
            departure: fields.1,
            arrival: fields.2,
            seats_available: fields.3,
        })
    }
}
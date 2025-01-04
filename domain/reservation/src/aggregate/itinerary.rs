use std::collections::linked_list::Iter;
use std::collections::LinkedList;
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{Accommodation, AccommodationId, Flight, FlightId};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Itinerary(LinkedList<ItineraryStage>);

impl Itinerary {
    pub(super) fn from_iter(stages: impl IntoIterator<Item=(Flight, Option<Accommodation>)>) -> Result<Self, ItineraryError> {
        let unconfirmed_stages: LinkedList<ItineraryStage> = stages.into_iter()
            .map(|(flight, accommodation)| ItineraryStage::Planned(flight, accommodation))
            .collect();

        if !unconfirmed_stages.is_empty() {
            Ok(Self(unconfirmed_stages))
        } else {
            Err(ItineraryError::NoStages)
        }
    }

    pub fn stages(&self) -> Iter<ItineraryStage> {
        self.0.iter()
    }

    /// self is equivalent to other, if planned stages are equal
    pub fn equivalent(&self, other: &Self) -> bool {
        self.planned() == other.planned()
    }

    pub fn planned(&self) -> Self {
        let stages = self.stages()
            .map(|stage| stage.clone().planned())
            .collect();

        Self(stages)
    }

    pub fn first_stage(&self) -> &ItineraryStage {
        self.0.front()
            .expect("stages should always be non-empty")
    }

    pub fn last_stage(&self) -> &ItineraryStage {
        self.0.back()
            .expect("stages should always be non-empty")
    }

    pub fn departure_date(&self) -> NaiveDate {
        match self.first_stage() {
            ItineraryStage::Planned(flight, _)
            | ItineraryStage::Reserved(flight, _)
            | ItineraryStage::ReservedFailed(flight, _, _)
            | ItineraryStage::Annulled(flight, _) => flight.departure.date_naive()
        }
    }

    pub fn duration(&self) -> Duration {
        let departure = match self.first_stage() {
            ItineraryStage::Planned(flight, _)
            | ItineraryStage::Reserved(flight, _)
            | ItineraryStage::ReservedFailed(flight, _, _)
            | ItineraryStage::Annulled(flight, _) => flight.departure,
        };

        let arrival = match self.last_stage() {
            ItineraryStage::Planned(flight, _)
            | ItineraryStage::Reserved(flight, _)
            | ItineraryStage::ReservedFailed(flight, _, _)
            | ItineraryStage::Annulled(flight, _) => flight.arrival,
        };

        arrival - departure
    }

    pub fn mark_flight_as_reserved(self, flight: &FlightId) -> Self {
        self.mark_flight(flight, |stage| stage.mark_flight_as_reserved())
    }

    pub fn mark_flight_as_reserved_failed(self, flight: &FlightId, reason: &ItineraryStageError) -> Self {
        self.mark_flight(flight, |stage| stage.mark_flight_as_reserved_failed(reason))
    }

    pub fn mark_flight_as_annulled(self, flight: &FlightId) -> Self {
        self.mark_flight(flight, |stage| stage.mark_flight_as_annulled())
    }

    fn mark_flight<F>(self, flight: &FlightId, mark: F) -> Self
        where F: Fn(ItineraryStage) -> ItineraryStage {

        Itinerary(self.0
            .into_iter()
            .map(|stage|
                if &stage.flight() == flight {
                    mark(stage)
                } else {
                    stage
                })
            .collect()
        )
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ItineraryStage {
    Planned(Flight, Option<Accommodation>),
    Reserved(Flight, Option<Accommodation>),
    ReservedFailed(Flight, Option<Accommodation>, ItineraryStageError),
    Annulled(Flight, Option<Accommodation>),
}

impl ItineraryStage {
    pub fn flight(&self) -> FlightId {
        match self {
            ItineraryStage::Planned(flight, _) => flight.id.clone(),
            ItineraryStage::Reserved(flight, _) => flight.id.clone(),
            ItineraryStage::ReservedFailed(flight, _, _) => flight.id.clone(),
            ItineraryStage::Annulled(flight, _) => flight.id.clone(),
        }
    }

    pub fn planned(self) -> Self {
        match self {
            ItineraryStage::Planned(flight, accommodation)
            | ItineraryStage::Reserved(flight, accommodation)
            | ItineraryStage::ReservedFailed(flight, accommodation, _)
            | ItineraryStage::Annulled(flight, accommodation) => {
                ItineraryStage::Planned(flight, accommodation)
            }
        }
    }

    pub fn mark_flight_as_reserved(self) -> Self {
        match self {
            ItineraryStage::Planned(flight, accommodation)
            | ItineraryStage::Reserved(flight, accommodation)
            | ItineraryStage::ReservedFailed(flight, accommodation, _)
            | ItineraryStage::Annulled(flight, accommodation) => {
                ItineraryStage::Reserved(flight, accommodation)
            }
        }
    }

    pub fn mark_flight_as_reserved_failed(self, reason: &ItineraryStageError) -> Self {
        match self {
            ItineraryStage::Planned(flight, accommodation)
            | ItineraryStage::Reserved(flight, accommodation)
            | ItineraryStage::ReservedFailed(flight, accommodation, _)
            | ItineraryStage::Annulled(flight, accommodation) => {
                ItineraryStage::ReservedFailed(flight, accommodation, reason.clone())
            }
        }
    }

    pub fn mark_flight_as_annulled(self) -> Self {
        match self {
            ItineraryStage::Planned(flight, accommodation)
            | ItineraryStage::Reserved(flight, accommodation)
            | ItineraryStage::ReservedFailed(flight, accommodation, _)
            | ItineraryStage::Annulled(flight, accommodation) => {
                ItineraryStage::Annulled(flight, accommodation)
            }
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ItineraryError {
    #[error("no stages")]
    NoStages,

    #[error("unknown flight")]
    UnknownFlight(FlightId),

    #[error("unknown accommodation")]
    UnknownAccommodation(AccommodationId),

    #[error("last stage has accommodation")]
    LastStageHasAccommodation,

    #[error("malformed flight route(s)")]
    MalformedRoute,

    #[error("accommodation not in stage")]
    AccommodationNotInStage,

    #[error("flights are not consecutive")]
    FlightsAreNotConsecutive,

    #[error("days in accommodation is too short")]
    DaysInAccommodationIsTooShort,

    #[error("days in accommodation is too long")]
    DaysInAccommodationIsTooLong,

    #[error("I/O error: {0}")]
    IoError(String)
}

#[derive(Error, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ItineraryStageError {
    #[error("insufficient seats")]
    InsufficientSeats,
}
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::event::{Event, JourneyPublishedV1};
use crate::repository::AirfieldRepository;
use crate::aggregate;


#[derive(Error, Debug, PartialEq)]
pub enum JourneysError {
    #[error("unable to resolve airfield: {0}")]
    UnableToResolveAirfield(aggregate::AirfieldId),

    #[error("I/O error: {0}")]
    IoError(String),
}

pub type JourneysApplyResult<T> = Result<T, JourneysError>;

#[derive(Serialize, Deserialize, Default)]
pub struct Journeys(Vec<Journey>);

impl Journeys {
    pub async fn apply(mut self, context: &JourneysApplyContext, event: Event) -> JourneysApplyResult<Journeys> {
        match event {
            Event::JourneyPublishedV1(event) => {
                let journey = Journey::from(context, event).await?;

                self.0.retain(|j| j.id != journey.id);
                self.0.push(journey);
            }

            _ => {
                // ignore unknown event types
            }
        }

        Ok(self)
    }
}

#[derive(Serialize, Deserialize)]
struct Journey {
    id: String,
    name: String,
    segments: Vec<Segment>,
}

impl Journey {
    async fn from(context: &JourneysApplyContext, value: JourneyPublishedV1) -> JourneysApplyResult<Self> {
        let id = value.id.to_string();
        let name = value.name.to_string();
        let mut segments = Vec::new();
        for segment in value.segments.into_iter() {
            segments.push(
                Segment::from(context, segment).await?
            );
        }

        Ok(Self {
            id,
            name,
            segments
        })
    }
}

#[derive(Serialize, Deserialize)]
struct Segment {
    flight: FlightRoute,
    accommodations: Vec<Accommodation>
}

impl Segment {
    async fn from(context: &JourneysApplyContext, value: aggregate::Segment) -> JourneysApplyResult<Self> {
        let flight = FlightRoute::from(context, value.flight).await?;
        let accommodations = value.accommodations.into_iter()
            .map(Accommodation::from)
            .collect();

        Ok(Self {
            flight,
            accommodations
        })
    }
}

#[derive(Serialize, Deserialize)]
struct FlightRoute {
    departure: Airfield,
    arrival: Airfield
}

impl FlightRoute {
    async fn from(context: &JourneysApplyContext, value: aggregate::FlightRoute) -> JourneysApplyResult<Self> {
        let departure = context.resolve_airfield(&value.departure).await?;
        let arrival = context.resolve_airfield(&value.arrival).await?;

        Ok(Self {
            departure,
            arrival
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Airfield {
    id: String,
    name: String,
    location: String
}

impl From<aggregate::Airfield> for Airfield {
    fn from(value: aggregate::Airfield) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            location: value.location.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Accommodation {
    id: String,
    name: String,
    place: Place,
    pictures: Vec<Picture>,
}

impl From<aggregate::Accommodation> for Accommodation {
    fn from(value: aggregate::Accommodation) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            place: value.place.into(),
            pictures: value.pictures.into_iter()
                .map(Picture::from)
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Place {
    name: String,
    location: String,
}

impl From<aggregate::Place> for Place {
    fn from(value: aggregate::Place) -> Self {
        Place {
            name: value.name.to_string(),
            location: value.location.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Picture {
    url: String,
    caption: String,
}

impl From<aggregate::Picture> for Picture {
    fn from(value: aggregate::Picture) -> Self {
        Self {
            url: value.url.to_string(),
            caption: value.caption.to_string(),
        }
    }
}

// apply context
pub struct JourneysApplyContext {
    airfield_repository: Rc<dyn AirfieldRepository>
}

impl JourneysApplyContext {
    pub fn new(airfield_repository: Rc<dyn AirfieldRepository>) -> Self {
        Self {
            airfield_repository
        }
    }

    pub async fn resolve_airfield(&self, id: &aggregate::AirfieldId) -> Result<Airfield, JourneysError> {
        self.airfield_repository.get(id).await
            .map_err(|error| JourneysError::IoError(error.to_string()))?
            .ok_or(JourneysError::UnableToResolveAirfield(id.clone()))
            .map(|airfield| airfield.into())
    }
}
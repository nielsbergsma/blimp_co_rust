use std::collections::HashMap;
use std::rc::Rc;
use prelude::domain::{Event, EventPublisher};
use crate::aggregate::{Airfield, Journey, JourneyId};
use crate::command::{PublishJourney, PublishJourneyError, PublishJourneyResult, RegisterAirfield, RegisterAirfieldResult};
use crate::repository::{AirfieldRepository, JourneyRepository};

pub struct JourneyUseCase {
    journey_repository: Rc<dyn JourneyRepository>,
    airfield_repository: Rc<dyn AirfieldRepository>,
    event_publisher: Rc<dyn EventPublisher>,
}

impl JourneyUseCase {
    pub fn new(journey_repository: Rc<dyn JourneyRepository>, airfield_repository: Rc<dyn AirfieldRepository>, event_publisher: Rc<dyn EventPublisher>) -> Self {
        Self {
            journey_repository,
            airfield_repository,
            event_publisher,
        }
    }

    pub async fn register_airfield(&self, command: RegisterAirfield) -> RegisterAirfieldResult {
        let transaction = self.airfield_repository.set_begin(&command.id).await?;

        let airfield = Airfield::build(command.id.clone(), command.name, command.location);
        self.airfield_repository.set_commit(transaction.with_value(airfield)).await?;

        Ok(command.id)
    }

    pub async fn publish(&self, command: PublishJourney) -> PublishJourneyResult {
        let id = JourneyId::new_random();

        let mut airfield_lookup = HashMap::new();
        for segment in command.segments.iter() {
            let id = segment.flight.departure.clone();
            let airfield = self.airfield_repository.get(&id).await?
                .ok_or(PublishJourneyError::UnknownAirfield(id.clone()))?;

            airfield_lookup.insert(id, airfield);
        }

        let transaction = self.journey_repository.set_begin(&id).await?
            .expect_empty(PublishJourneyError::IdConflict)?;

        let (journey, journey_published) = Journey::build(
            id,
            command.name,
            command.segments
        )?;
        let event: Event = journey_published.try_into()?;

        self.journey_repository.set_commit(transaction.with_value(journey)).await?;
        self.event_publisher.send(event).await?;

        Ok(id)
    }
}
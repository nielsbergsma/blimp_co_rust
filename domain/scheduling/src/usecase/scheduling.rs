use std::rc::Rc;
use prelude::domain::{Event, EventPublisher};
use crate::aggregate::{Airfield, Airship, Flight, FlightId};
use crate::command::{AddAirshipToFleet, AddAirshipToFleetError, AddAirshipToFleetResult, RegisterAirfield, RegisterAirfieldError, RegisterAirfieldResult, ScheduleFlight, ScheduleFlightError, ScheduleFlightResult};
use crate::repository::{AirfieldRepository, AirshipRepository, FlightRepository};

pub struct SchedulingUseCase {
    airfield_repository: Rc<dyn AirfieldRepository>,
    airship_repository: Rc<dyn AirshipRepository>,
    flight_repository: Rc<dyn FlightRepository>,
    event_publisher: Rc<dyn EventPublisher>,
}

impl SchedulingUseCase {
    pub fn new(
        airfield_repository: Rc<dyn AirfieldRepository>,
        airship_repository: Rc<dyn AirshipRepository>,
        flight_repository: Rc<dyn FlightRepository>,
        event_publisher: Rc<dyn EventPublisher>,
    ) -> Self {

        Self {
            airfield_repository,
            airship_repository,
            flight_repository,
            event_publisher,
        }
    }

    pub async fn add_airship_to_fleet(&self, command: AddAirshipToFleet) -> AddAirshipToFleetResult {
        let transaction = self.airship_repository.set_begin(&command.id).await?
            .expect_empty(AddAirshipToFleetError::IdConflict)?;

        let (airship, airship_registered) = Airship::build(
            command.id.clone(),
            command.name,
            command.model,
            command.number_of_seats
        );
        let event: Event = airship_registered.try_into()?;

        self.airship_repository.set_commit(transaction.with_value(airship)).await?;
        self.event_publisher.send(event).await?;

        Ok(command.id)
    }

    pub async fn register_airfield(&self, command: RegisterAirfield) -> RegisterAirfieldResult {
        let transaction = self.airfield_repository.set_begin(&command.id).await?
            .expect_empty(RegisterAirfieldError::IdConflict)?;

        let (airfield, airfield_registered) = Airfield::build(
            command.id.to_owned(),
            command.name,
            command.location,
        );
        let event: Event = airfield_registered.try_into()?;

        self.airfield_repository.set_commit(transaction.with_value(airfield)).await?;
        self.event_publisher.send(event).await?;

        Ok(command.id)
    }

    pub async fn schedule_flight(&self, command: ScheduleFlight) -> ScheduleFlightResult {
        let id = FlightId::new_random();

        let transaction = self.flight_repository.set_begin(id).await?
            .expect_empty(ScheduleFlightError::IdConflict)?;

        // resolve dependencies
        let departure_location = self.airfield_repository
            .get(&command.departure_location).await?
            .ok_or(ScheduleFlightError::UnknownAirfield)?;

        let arrival_location = self.airfield_repository
            .get(&command.arrival_location).await?
            .ok_or(ScheduleFlightError::UnknownAirfield)?;

        let airship = self.airship_repository
            .get(&command.airship).await?
            .ok_or(ScheduleFlightError::UnknownAirship)?;

        let (flight, flight_scheduled) = Flight::build(
            id,
            departure_location,
            command.departure_time,
            arrival_location,
            command.arrival_time,
            airship,
        )?;
        let event: Event = flight_scheduled.try_into()?;

        self.flight_repository.set_commit(transaction.with_value(flight)).await?;
        self.event_publisher.send(event).await?;

        Ok(id)
    }
}

use std::rc::Rc;
use prelude::domain::{Event, EventPublisher};
use crate::aggregate::{FlightAvailability};
use crate::command::{MakeFlightAvailableError, MakeFlightAvailable, MakeFlightAvailableResult, ReserveFlight, ReserveFlightResult, ReserveFlightError};
use crate::event::{FlightReservationFailedV1, FlightReservedV1};
use crate::repository::{FlightAvailabilityRepository};

pub struct AvailabilityUseCase {
    flight_availability_repository: Rc<dyn FlightAvailabilityRepository>,
    event_publisher: Rc<dyn EventPublisher>,
}

impl AvailabilityUseCase {
    pub fn new(flight_availability_repository: Rc<dyn FlightAvailabilityRepository>,event_publisher: Rc<dyn EventPublisher>) -> Self {
        Self {
            flight_availability_repository,
            event_publisher,
        }
    }

    pub async fn make_flight_available(&self, command: MakeFlightAvailable) -> MakeFlightAvailableResult {
        let transaction = self.flight_availability_repository.set_begin(&command.flight.id).await?
            .expect_empty(MakeFlightAvailableError::IdConflict)?;

        let (flight_availability, availability_changed) = FlightAvailability::from_flight(command.flight);
        let event: Event = availability_changed.try_into()?;

        self.flight_availability_repository.set_commit(transaction.with_value(flight_availability)).await?;
        self.event_publisher.send(event).await?;

        Ok(())
    }

    pub async fn reserve_flight(&self, command: ReserveFlight) -> ReserveFlightResult {
        let transaction = self.flight_availability_repository
            .set_begin(&command.flight).await?
            .expect_non_empty(ReserveFlightError::UnknownFlight(command.flight.clone()))?;

        let availability = transaction.value.clone()
            .ok_or(ReserveFlightError::UnknownFlight(command.flight.clone()))?;

        match availability.reserve(command.reservation.value_ref(), command.seats) {
            Ok((availability, availability_changed)) => {
                self.flight_availability_repository.set_commit(
                    transaction.with_value(availability)
                ).await?;

                let event: Event = availability_changed.try_into()?;
                self.event_publisher.send(event).await?;

                let reserved_flight = FlightReservedV1 {
                    reservation: command.reservation,
                    flight: command.flight,
                    seats: command.seats,
                };
                let event: Event = reserved_flight.try_into()?;
                self.event_publisher.send(event).await?;

                Ok(())
            }

            Err(reason) => {
                let reserved_flight_failed = FlightReservationFailedV1 {
                    reservation: command.reservation,
                    flight: command.flight,
                    reason,
                };

                let event: Event = reserved_flight_failed.try_into()?;
                self.event_publisher.send(event).await?;

                Ok(())
            }
        }
    }
}
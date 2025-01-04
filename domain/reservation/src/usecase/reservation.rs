use std::collections::LinkedList;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use prelude::domain::{Event, EventPublisher};
use crate::aggregate::{AccommodationId, AvailabilityFlightError, FlightId, ItineraryError, ItineraryStageError, Journey, Reservation, ReservationId};
use crate::command::{CancelReservation, CancelReservationError, CancelReservationResult, ConfirmReservation, ConfirmReservationError, ConfirmReservationResult, GetReservation, GetReservationError, GetReservationResult, ReferencedItineraryStage, ResolvedItineraryStage, ReviseItinerary, ReviseItineraryError, ReviseItineraryResult, RevisePassengers, RevisePassengersError, RevisePassengersResult};
use crate::event::{FlightReservationFailedV1, FlightReservedV1};
use crate::policy::ReservationPolicy;
use crate::repository::{FlightAvailabilityRepository, JourneyRepository, ReservationRepository};
use crate::services::flight_reservation_strategy;
use crate::usecase::result::{HandleFlightReservationFailedError, HandleFlightReservationFailedResult, HandleFlightReservedError, HandleFlightReservedResult};

pub struct ReservationUseCase {
    reservation_repository: Rc<dyn ReservationRepository>,
    flight_availability_repository: Rc<dyn FlightAvailabilityRepository>,
    journey_repository: Rc<dyn JourneyRepository>,
    event_publisher: Rc<dyn EventPublisher>
}

impl ReservationUseCase {
    pub fn new(
        reservation_repository: Rc<dyn ReservationRepository>,
        flight_availability_repository: Rc<dyn FlightAvailabilityRepository>,
        journey_repository: Rc<dyn JourneyRepository>,
        event_publisher: Rc<dyn EventPublisher>
    ) -> Self {
        Self {
            reservation_repository,
            flight_availability_repository,
            journey_repository,
            event_publisher,
        }
    }

    pub async fn get(&self, command: GetReservation) -> GetReservationResult {
        match self.reservation_repository.get(&command.id).await? {
            Some(reservation) => Ok(reservation),
            None => Err(GetReservationError::UnknownReservation),
        }
    }

    pub async fn confirm(&self, command: ConfirmReservation, policy: &ReservationPolicy) -> ConfirmReservationResult {
        let id = ReservationId::new_random();

        let journey = self.journey_repository.get(&command.journey).await?
            .ok_or(ConfirmReservationError::UnknownJourney)?;

        let itinerary = journey.parse_itinerary(
            self.resolve_itinerary_stages(&journey, command.itinerary).await?
        )?;

        let transaction = self.reservation_repository.set_begin(&id).await?
            .expect_empty(ConfirmReservationError::IdConflict)?;

        let (reservation, reservation_confirmed) = Reservation::new_confirmed(
            policy,
            id,
            command.journey,
            command.contact,
            command.passengers,
            itinerary,
        )?;
        let event: Event = reservation_confirmed.try_into()?;

        self.reservation_repository.set_commit(transaction.with_value(reservation.clone())).await?;
        self.event_publisher.send(event).await?;

        // kick-off resolving flights in reservations (saga)
        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
            .map(|_| id)
    }

    pub async fn revise_passengers(&self, command: RevisePassengers, policy: &ReservationPolicy) -> RevisePassengersResult {
        let transaction = self.reservation_repository.set_begin(&command.reservation).await?;

        let (reservation, reservation_revised) = transaction
            .value_or(RevisePassengersError::UnknownReservation)?
            .revise_passengers(policy, command.passengers)?;

        self.reservation_repository.set_commit(
            transaction.with_value(reservation.clone())
        ).await?;

        if let Some(event) = reservation_revised {
            let event = event.try_into()?;
            self.event_publisher.send(event).await?;
        }

        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
    }

    pub async fn revise_itinerary(&self, command: ReviseItinerary, policy: &ReservationPolicy) -> ReviseItineraryResult {
        let transaction = self.reservation_repository
            .set_begin(&command.reservation).await?;

        let reservation = transaction
            .value_or(ReviseItineraryError::UnknownReservation)?;

        let journey = self.journey_repository.get(&reservation.journey()).await?
            .ok_or(ReviseItineraryError::UnknownJourney)?;

        let itinerary = journey.parse_itinerary(
            self.resolve_itinerary_stages(&journey, command.itinerary).await?
        )?;

        let (reservation, reservation_revised) = reservation.revise_itinerary(policy, itinerary)?;

        self.reservation_repository.set_commit(
            transaction.with_value(reservation.clone())
        ).await?;

        if let Some(event) = reservation_revised {
            let event = event.try_into()?;
            self.event_publisher.send(event).await?;
        }

        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
    }

    pub async fn cancel(&self, command: CancelReservation, policy: &ReservationPolicy) -> CancelReservationResult {
        let transaction = self.reservation_repository
            .set_begin(&command.id).await?;

        let (reservation, reservation_cancelled) = transaction
            .value_or(CancelReservationError::UnknownReservation)?
            .cancel(policy)?;

        self.reservation_repository.set_commit(
            transaction.with_value(reservation.clone())
        ).await?;

        if let Some(event) = reservation_cancelled {
            let event = event.try_into()?;
            self.event_publisher.send(event).await?;
        }

        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
    }

    pub async fn handle_flight_reserved(&self, event: &FlightReservedV1) -> HandleFlightReservedResult {
        let id = event.reservation.value_ref();
        let version = event.reservation.version();
        let transaction = self.reservation_repository.set_begin(id).await?;

        let reservation = if event.annulled() {
            transaction
                .value_or(HandleFlightReservedError::UnknownReservation(*id))?
                .mark_flight_as_annulled(&event.flight, version)
        }
        else {
            transaction
                .value_or(HandleFlightReservedError::UnknownReservation(*id))?
                .mark_flight_as_reserved(&event.flight, version)
        };

        self.reservation_repository.set_commit(
            transaction.with_value(reservation.clone())
        ).await?;

        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
    }

    pub async fn handle_flight_reservation_failed(&self, event: &FlightReservationFailedV1) -> HandleFlightReservationFailedResult {
        let id = event.reservation.value_ref();
        let version = event.reservation.version();
        let reason = match event.reason {
            AvailabilityFlightError::InsufficientSeats => ItineraryStageError::InsufficientSeats,
        };
        let transaction = self.reservation_repository.set_begin(id).await?;

        let reservation = transaction
            .value_or(HandleFlightReservationFailedError::UnknownReservation(*id))?
            .mark_flight_as_reserved_failed(&event.flight, &reason, version);

        self.reservation_repository.set_commit(
            transaction.with_value(reservation.clone())
        ).await?;

        self.resolve_flights(&reservation).await
            .map_err(|error| error.into())
    }

    async fn resolve_itinerary_stages(&self, journey: &Journey, stages: LinkedList<ReferencedItineraryStage>) -> Result<LinkedList<ResolvedItineraryStage>, ItineraryError> {
        let mut resolved_stages = LinkedList::default();
        for (flight_id, accommodation_id) in stages {
            resolved_stages.push_back(
                self.resolve_itinerary_stage(journey, flight_id, accommodation_id).await?
            );
        }

        Ok(resolved_stages)
    }

    async fn resolve_itinerary_stage(&self, journey: &Journey, flight: FlightId, accommodation: Option<AccommodationId>) -> Result<ResolvedItineraryStage, ItineraryError> {
        let flight = self.flight_availability_repository.get(&flight).await
            .map_err(|error| ItineraryError::IoError(error.to_string()))?
            .map(|availability| availability.flight)
            .ok_or(ItineraryError::UnknownFlight(flight))?;

        let accommodation = match accommodation {
            Some(id) => {
                let location = flight.route.arrival.clone();

                journey.find_accommodation(&location, &id)
                    .map(|accommodation| Some(accommodation.clone()))
                    .ok_or(ItineraryError::UnknownAccommodation(id))?
            }

            None => None
        };

        Ok((flight, accommodation))
    }

    /// resolve flights of a reservation (reserve, and/or annul);
    /// works similar to an orchestration-based saga, meaning 1 aggregate at a time is changed in a single transaction (event driven)
    /// converses reservation to the correct state, 1 event at a time
    async fn resolve_flights(&self, reservation: &Reservation) -> Result<(), Error> {
        if let Some(request) = flight_reservation_strategy::next_request(reservation) {
            let event: Event = request.try_into()
                .map_err(|error| Error::new(ErrorKind::Other, error))?;

            self.event_publisher.send(event).await
                .map_err(|error| Error::new(ErrorKind::Other, error))?;
        }
        Ok(())
    }
}

use std::rc::Rc;
use worker::*;
use prelude::{durable_object_repository, queue_publisher};
use reservation::command::{MakeFlightAvailable, RegisterAirfield, ReserveFlight};
use reservation::event::{Event, RawEvent};
use reservation::projection::{JourneysApplyContext, YearMonth};
use reservation::usecase::{AvailabilityUseCase, JourneyUseCase, ReservationUseCase};
use crate::api::RouteData;
use crate::runtime::repository::{
    DurableObjectReservationRepository,
    DurableObjectReservationRepositoryProtocol,
    R2AvailabilityRepository,
    R2JourneysRepository
};

mod api;
mod runtime;

durable_object_repository!(ReservationRepository, DurableObjectReservationRepository, DurableObjectReservationRepositoryProtocol);

#[event(queue)]
pub async fn main(message_batch: MessageBatch<RawEvent>, env: Env, _ctx: Context) -> Result<()> {
    let reservation_repository = Rc::new(DurableObjectReservationRepository::new(
        env.durable_object("reservation_objects")?
    ));
    let journeys_repository = R2JourneysRepository::build(
        env.bucket("reservation_rs_bucket")?
    );
    let availability_repository = R2AvailabilityRepository::build(
        env.bucket("reservation_rs_bucket")?
    );

    let event_publisher = Rc::new(queue_publisher!(env));

    let journey_usecase = JourneyUseCase::new(
        reservation_repository.clone(),
        reservation_repository.clone(),
        event_publisher.clone()
    );

    let availability_usecase = AvailabilityUseCase::new(
        reservation_repository.clone(),
        event_publisher.clone(),
    );

    let reservation_usecase = ReservationUseCase::new(
        reservation_repository.clone(),
        reservation_repository.clone(),
        reservation_repository.clone(),
        event_publisher.clone(),
    );

    for message in message_batch.messages()? {
        match message.body().deserialize()? {
            event@Event::JourneyPublishedV1(_) => {
                let context = JourneysApplyContext::new(reservation_repository.clone());

                let journeys = journeys_repository.get().await?;
                let journeys = journeys.apply(&context, event).await
                    .map_err(|error| error.to_string())?;
                journeys_repository.set(&journeys).await?;
            }

            Event::FlightScheduledV1(event) => {
                let command = MakeFlightAvailable {
                    flight: event.into(),
                };

                availability_usecase.make_flight_available(command).await
                    .map_err(|error| error.to_string())?;
            },

            Event::FlightAvailabilityChangedV1(flight) => {
                let period = YearMonth::from_datetime(flight.departure);
                let event = Event::FlightAvailabilityChangedV1(flight);

                let availability = availability_repository.get(period).await?;
                let availability = availability.apply(event);
                availability_repository.set(availability).await?;
            }

            Event::AirfieldRegisteredV1(airfield) => {
                let command = RegisterAirfield {
                    id: airfield.id,
                    name: airfield.name,
                    location: airfield.location,
                };

                journey_usecase.register_airfield(command).await
                    .map_err(|error| error.to_string())?;
            }

            Event::FlightReservationRequestedV1(request) => {
                let command = ReserveFlight {
                    reservation: request.reservation,
                    flight: request.flight,
                    seats: request.seats,
                };

                availability_usecase.reserve_flight(command).await
                    .map_err(|error| error.to_string())?;
            }

            Event::FlightReservedV1(event) => {
                reservation_usecase.handle_flight_reserved(&event).await
                    .map_err(|error| error.to_string())?;
            }

            Event::FlightReservationFailedV1(event) => {
                reservation_usecase.handle_flight_reservation_failed(&event).await
                    .map_err(|error| error.to_string())?;
            }

            _ => {
                // ignore other events
            }
        }
    }

    Ok(())
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let api_key = env.secret("api_key")?;
    let repository = Rc::new(DurableObjectReservationRepository::new(
        env.durable_object("reservation_objects")?
    ));
    let event_publisher = Rc::new(queue_publisher!(env));

    let route_data = RouteData {
        journey_usecase: JourneyUseCase::new(
            repository.clone(),
            repository.clone(),
            event_publisher.clone(),
        ),
        reservation_usecase: ReservationUseCase::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            event_publisher.clone()
        ),
        api_key: api_key.to_string(),
    };

    api::route(req, env, route_data).await
}
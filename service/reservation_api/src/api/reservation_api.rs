use worker::{Request, RouteContext};
use reservation::command::{CancelReservation, ConfirmReservation, GetReservation, ReviseItinerary, RevisePassengers};
use crate::api::{ApiResult, RouteData};
use crate::api::security::{Claims, reservation_policy_from_claims};
use crate::api::transfer_objects::{DeleteReservationResponse, GetReservationResponse, parse_contact, parse_itinerary, parse_passenger_arrangement, parse_passengers, PostReservationsRequest, PostReservationsResponse, PutReservationItineraryRequest, PutReservationItineraryResponse, PutReservationPassengersRequest, PutReservationPassengersResponse, reservation_to_transfer_object};

pub async fn post_reservations(mut req: Request, claims: Option<Claims>, ctx: RouteContext<RouteData>) -> ApiResult<PostReservationsResponse> {
    let usecase =  ctx.data.reservation_usecase;
    let body: PostReservationsRequest = req.json().await?;

    let command = ConfirmReservation {
        journey: body.journey.parse()?,
        contact: parse_contact(body.contact)?,
        passengers: parse_passenger_arrangement(body.passengers)?,
        itinerary: parse_itinerary(body.itinerary)?,
    };

    let policy = reservation_policy_from_claims(claims);

    let result = usecase.confirm(command, &policy).await?;
    Ok(PostReservationsResponse{
        id: result.to_string(),
    })
}

pub async fn get_reservation(_: Request, ctx: RouteContext<RouteData>) -> ApiResult<GetReservationResponse> {
    let id = ctx.param("id").unwrap_or(&String::default()).parse()?;
    let command = GetReservation {
        id
    };

    let reservation = ctx.data.reservation_usecase.get(command).await?;
    Ok(reservation_to_transfer_object(reservation))
}

pub async fn put_reservation_passengers(mut req: Request, claims: Option<Claims>, ctx: RouteContext<RouteData>) -> ApiResult<PutReservationPassengersResponse> {
    let id = ctx.param("id").unwrap_or(&String::default()).parse()?;
    let body: PutReservationPassengersRequest = req.json().await?;

    let usecase =  ctx.data.reservation_usecase;

    let command = RevisePassengers {
        reservation: id,
        passengers: parse_passengers(body)?
    };

    let policy = reservation_policy_from_claims(claims);
    usecase.revise_passengers(command, &policy).await?;

    Ok(PutReservationPassengersResponse{
        id,
    })
}

pub async fn put_reservation_itinerary(mut req: Request, claims: Option<Claims>, ctx: RouteContext<RouteData>) -> ApiResult<PutReservationItineraryResponse> {
    let id = ctx.param("id").unwrap_or(&String::default()).parse()?;
    let body: PutReservationItineraryRequest = req.json().await?;

    let usecase =  ctx.data.reservation_usecase;

    let command = ReviseItinerary {
        reservation: id,
        itinerary: parse_itinerary(body)?,
    };

    let policy = reservation_policy_from_claims(claims);
    usecase.revise_itinerary(command, &policy).await?;

    Ok(PutReservationItineraryResponse{
        id,
    })
}

pub async fn delete_reservation(_: Request, claims: Option<Claims>, ctx: RouteContext<RouteData>) -> ApiResult<DeleteReservationResponse> {
    let id = ctx.param("id").unwrap_or(&String::default()).parse()?;
    let usecase =  ctx.data.reservation_usecase;

    let command = CancelReservation {
        id
    };

    let policy = reservation_policy_from_claims(claims);
    usecase.cancel(command, &policy).await?;

    Ok(DeleteReservationResponse{
        id,
    })
}
use std::future::Future;
use serde::Serialize;
use worker::{Env, Request, Response, Router, Result, RouteContext};
use reservation::usecase::{JourneyUseCase, ReservationUseCase};
use crate::api::security::{Claims, parse_bearer_token};

mod transfer_objects;
mod journey_api;
mod reservation_api;
mod security;


type ApiResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct RouteData {
    pub journey_usecase: JourneyUseCase,
    pub reservation_usecase: ReservationUseCase,
    pub api_key: String,
}

pub async fn route(request: Request, environment: Env, route_data: RouteData) -> Result<Response> {
    Router::with_data(route_data)
        .post_async("/journeys", |req, ctx| async move {
            authorize_with_api_key(req, ctx, |req, ctx| async move {
                to_response(journey_api::post_journeys(req, ctx).await)
            }).await
        })
        .post_async("/reservations", |req, ctx| async move {
            authorize_with_optional_bearer_token(req, ctx, |req, cls, ctx| async move {
                to_response(reservation_api::post_reservations(req, cls, ctx).await)
            }).await
        })
        .put_async("/reservations/:id/passengers", |req, ctx| async move {
            authorize_with_optional_bearer_token(req, ctx, |req, cls, ctx| async move {
                to_response(reservation_api::put_reservation_passengers(req, cls, ctx).await)
            }).await
        })
        .put_async("/reservations/:id/itinerary", |req, ctx| async move {
            authorize_with_optional_bearer_token(req, ctx, |req, cls, ctx| async move {
                to_response(reservation_api::put_reservation_itinerary(req, cls, ctx).await)
            }).await
        })
        .get_async("/reservations/:id", |req, ctx| async move {
            to_response(reservation_api::get_reservation(req, ctx).await)
        })
        .delete_async("/reservations/:id", |req, ctx| async move {
            authorize_with_optional_bearer_token(req, ctx, |req, cls, ctx| async move {
                to_response(reservation_api::delete_reservation(req, cls, ctx).await)
            }).await
        })
        .run(request, environment)
        .await
}

async fn authorize_with_api_key<T: Future<Output=Result<Response>>>(
    request: Request,
    ctx: RouteContext<RouteData>,
    next: fn(Request, RouteContext<RouteData>) -> T) -> Result<Response> {

    if let Some(value) = request.headers().get("Authorization")? {
        if let Some(api_key) = value.strip_prefix("Key ") {
            return if api_key == ctx.data.api_key {
                next(request, ctx).await
            }
            else {
                Response::error("forbidden", 403)
            }
        }
    }
    Response::error("unauthorized", 401)
}

async fn authorize_with_optional_bearer_token<T: Future<Output=Result<Response>>>(
    request: Request,
    ctx: RouteContext<RouteData>,
    next: fn(Request, Option<Claims>, RouteContext<RouteData>) -> T) -> Result<Response> {

    if let Some(value) = request.headers().get("Authorization")? {
        if let Some(token) = value.strip_prefix("Bearer ") {
            let claims = parse_bearer_token(token);
            return next(request, Some(claims), ctx).await;
        }
    }
    next(request, None, ctx).await
}

fn to_response<T:Serialize>(result: ApiResult<T>) -> Result<Response> {
    match result {
        Ok(success) => Response::from_json(&success),
        Err(error) => {
            let body = transfer_objects::Error {
                error: error.to_string(),
            };
            Response::from_json(&body).map(|r| r.with_status(400))
        }
    }
}
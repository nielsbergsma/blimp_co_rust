use std::future::Future;
use serde::Serialize;
use worker::{Env, Request, Response, Router, Result, RouteContext};
use scheduling::usecase::SchedulingUseCase;

mod transfer_objects;
mod airship_api;
mod airfield_api;
mod flight_api;

type ApiResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct RouteData {
    pub usecase: SchedulingUseCase,
    pub api_key: String,
}

pub async fn route(request: Request, environment: Env, route_data: RouteData) -> Result<Response> {
    Router::with_data(route_data)
        .post_async("/airships", |req, ctx| async move {
            authorize_with_api_key(req, ctx, |req, ctx| async move {
                to_response(airship_api::post_airships(req, ctx).await)
            }).await
        })
        .post_async("/airfields", |req, ctx| async move {
            authorize_with_api_key(req, ctx, |req, ctx| async move {
                to_response(airfield_api::post_airfields(req, ctx).await)
            }).await
        })
        .post_async("/flights", |req, ctx| async move {
            authorize_with_api_key(req, ctx, |req, ctx| async move {
                to_response(flight_api::post_flights(req, ctx).await)
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
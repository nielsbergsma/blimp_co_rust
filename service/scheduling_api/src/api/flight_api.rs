use worker::{Request, RouteContext};
use scheduling::command::{ScheduleFlight};
use crate::api::{ApiResult, RouteData};
use crate::api::transfer_objects::{PostFlightsRequest, PostFlightsResponse};


pub async fn post_flights(mut req: Request, ctx: RouteContext<RouteData>) -> ApiResult<PostFlightsResponse> {
    let usecase =  ctx.data.usecase;
    let body: PostFlightsRequest = req.json().await?;

    let command = ScheduleFlight {
        departure_location: body.departure_location.parse()?,
        departure_time: body.departure_time,
        arrival_location: body.arrival_location.parse()?,
        arrival_time: body.arrival_time,
        airship: body.airship.parse()?,
    };

    let result = usecase.schedule_flight(command).await?;
    Ok(PostFlightsResponse {
        id: result.to_string()
    })
}

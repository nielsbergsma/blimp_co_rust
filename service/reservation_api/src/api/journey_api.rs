use worker::{Request, RouteContext};
use reservation::command::{PublishJourney};
use crate::api::{ApiResult, RouteData};
use crate::api::transfer_objects::*;

pub async fn post_journeys(mut req: Request, ctx: RouteContext<RouteData>) -> ApiResult<PostJourneyResponse> {
    let usecase =  ctx.data.journey_usecase;
    let body: PostJourneyRequest = req.json().await?;

    let command = PublishJourney {
        name: body.name.parse()?,
        segments: parse_segments(body.segments)?,
    };

    let result = usecase.publish(command).await?;
    Ok(PostJourneyResponse {
        id: result.to_string()
    })
}



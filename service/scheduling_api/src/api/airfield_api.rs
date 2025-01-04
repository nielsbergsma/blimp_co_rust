use worker::{Request, RouteContext};
use scheduling::command::{RegisterAirfield};
use crate::api::{ApiResult, RouteData};
use crate::api::transfer_objects::{PostAirfieldsRequest, PostAirfieldsResponse};


pub async fn post_airfields(mut req: Request, ctx: RouteContext<RouteData>) -> ApiResult<PostAirfieldsResponse> {
    let usecase =  ctx.data.usecase;
    let body: PostAirfieldsRequest = req.json().await?;

    let command = RegisterAirfield {
        id: body.id.parse()?,
        name: body.name.parse()?,
        location: body.location.parse()?,
    };

    let result = usecase.register_airfield(command).await?;
    Ok(PostAirfieldsResponse {
        id: result.to_string()
    })
}

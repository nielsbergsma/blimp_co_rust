use worker::{Request, RouteContext};
use scheduling::command::AddAirshipToFleet;
use crate::api::{ApiResult, RouteData};
use crate::api::transfer_objects::{PostAirshipsRequest, PostAirshipsResponse};


pub async fn post_airships(mut req: Request, ctx: RouteContext<RouteData>) -> ApiResult<PostAirshipsResponse> {
    let usecase =  ctx.data.usecase;
    let body: PostAirshipsRequest = req.json().await?;

    let command = AddAirshipToFleet {
        id: body.id.parse()?,
        name: body.name.parse()?,
        model: body.model.parse()?,
        number_of_seats: body.number_of_seats.try_into()?,
    };

    let result = usecase.add_airship_to_fleet(command).await?;
    Ok(PostAirshipsResponse {
       id: result.to_string()
    })
}

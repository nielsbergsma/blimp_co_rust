mod api;
mod runtime;

use std::rc::Rc;
use worker::*;
use prelude::{durable_object_repository, queue_publisher};
use scheduling::event::RawEvent;
use scheduling::usecase::SchedulingUseCase;
use crate::api::RouteData;
use crate::runtime::repository::{DurableObjectSchedulingRepository, DurableObjectSchedulingRepositoryProtocol, R2DashboardRepository};

durable_object_repository!(SchedulingRepository, DurableObjectSchedulingRepository, DurableObjectSchedulingRepositoryProtocol);

#[event(queue)]
pub async fn main(message_batch: MessageBatch<RawEvent>, env: Env, _ctx: Context) -> Result<()> {
    let repository = R2DashboardRepository::build(
        env.bucket("scheduling_rs_bucket")?
    );

    for message in message_batch.messages()? {
        let event = message.body().deserialize()?;

        let dashboard = repository.get().await?;
        let dashboard = dashboard.apply(event);
        repository.set(dashboard).await?;
    }

    Ok(())
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let api_key = env.secret("api_key")?;
    let repository = Rc::new(DurableObjectSchedulingRepository::new(
        env.durable_object("scheduling_objects")?
    ));
    let event_publisher = Rc::new(queue_publisher!(env));

    let route_data = RouteData {
        usecase: SchedulingUseCase::new(
            repository.clone(),
            repository.clone(),
            repository,
            event_publisher,
        ),
        api_key: api_key.to_string(),
    };

    api::route(req, env, route_data).await
}
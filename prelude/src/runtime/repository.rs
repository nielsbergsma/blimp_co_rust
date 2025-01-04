use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Reply<T> {
    Success(T),
    NotFound,
    VersionConflict,
    MalformedPrompt
}

#[macro_export]
macro_rules! durable_object_repository {
    ($name: ident, $repository: ty, $protocol: ty) => {
        #[durable_object]
        pub struct $name {
            state: State,
            env: Env,
        }

        #[durable_object]
        impl DurableObject for $name {
            fn new(state: State, env: Env) -> Self {
                Self { state, env }
            }

            async fn fetch(&mut self, mut request: Request) -> Result<Response> {
                let mut storage = self.state.storage();

                let prompt: $protocol = request.json().await?;
                let result = <$repository>::handle(prompt, &mut storage).await?;
                Response::from_json(&result)
            }
        }
    };
}
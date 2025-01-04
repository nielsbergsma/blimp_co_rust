use proc_macro::TokenStream;
use std::collections::HashMap;
use std::env::{var};
use std::fs::File;
use quote::quote;
use serde::{Deserialize, Serialize};

type EventBindings = HashMap<String, Vec<String>>;
type ServiceBindings = HashMap<String, EventBindings>;

#[derive(Serialize, Deserialize, Debug)]
struct EventMap {
    service: ServiceBindings
}

#[proc_macro]
pub fn queue_publisher(_input: TokenStream) -> TokenStream {
    let event_map_path = var("EVENT_MAP_PATH")
        .expect("EVENT_MAP_PATH not set");

    let event_map_data = File::open(event_map_path)
        .expect("unable to open event map file");

    let event_map: EventMap = serde_json::from_reader(event_map_data)
        .expect("malformed event map file");

    let service_name = var("CARGO_PKG_NAME")
        .expect("unknown service name");

    let event_bindings: Vec<_> = event_map.service.get(&service_name)
        .expect("service not in event map")
        .iter()
        .map(|(key, value)| quote!((#key, vec!(#(env.queue(#value)?),*))))
        .collect();


    quote!{
        {
            use prelude::domain::{Event, EventPublisher, EventPublishError};

            pub struct QueuePublisher {
                routes: Vec<(&'static str, Vec<worker::Queue>)>
            }

            #[prelude::async_trait(?Send)]
            impl EventPublisher for QueuePublisher {
                async fn send(&self, event: Event) -> std::result::Result<(), EventPublishError> {
                    for (event_name, queues) in self.routes.iter() {
                        if *event_name == event.name() {
                            let data = serde_json::to_string(&event.data())
                                .map_err(|e| EventPublishError::IoError(e.to_string()))?;

                            for queue in queues {
                                queue.send(&data).await
                                    .map_err(|e| EventPublishError::IoError(e.to_string()))?;
                            }

                            return Ok(());
                        }
                    }
                    Ok(())
                }
            }

            QueuePublisher {
                routes: vec!(#(#event_bindings),*)
            }
        }
    }.into()
}
use async_trait::async_trait;
use serde::{Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[macro_export]
macro_rules! event_name {
    ($type: ty) => {
        std::any::type_name::<$type>()
            .rsplit_once("::")
            .expect("invalid type of event")
            .1
    };
}

pub struct Event(String, Value);

pub type EventTryIntoError = serde_json::Error;

impl Event {
    pub fn try_into<T: Serialize>(value: T) -> Result<Event, EventTryIntoError> {
        let type_name = event_name!(T);
        let data = json!({
            type_name: value
        });

        Ok(Event(type_name.to_owned(), serde_json::to_value(data)?))
    }

    pub fn name(&self) -> String {
        self.0.clone()
    }

    pub fn data(&self) -> Value {
        self.1.clone()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum EventPublishError {
    #[error("I/O Error {0}")]
    IoError(String),
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait EventPublisher {
    async fn send(&self, event: Event) -> Result<(), EventPublishError>;
}
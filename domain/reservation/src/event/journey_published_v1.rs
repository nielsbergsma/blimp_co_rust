use serde::{Deserialize, Serialize};
use prelude::collection::SortedSet;
use prelude::domain::{Event, EventTryIntoError};
use crate::aggregate::{JourneyId, JourneyName, Segment};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JourneyPublishedV1 {
    pub id: JourneyId,
    pub name: JourneyName,
    pub segments: SortedSet<Segment>,
}

impl TryInto<Event> for JourneyPublishedV1 {
    type Error = EventTryIntoError;

    fn try_into(self) -> Result<Event, Self::Error> {
        Event::try_into(self)
    }
}
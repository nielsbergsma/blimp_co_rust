use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use prelude::data::GeoHash;
use crate::aggregate::airfield_id::AirfieldId;
use crate::aggregate::airfield_name::AirfieldName;
use crate::event::AirfieldRegisteredV1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Airfield {
    pub id: AirfieldId,
    pub name: AirfieldName,
    pub location: GeoHash,
}

impl Airfield {
    pub fn build(id: AirfieldId, name: AirfieldName, location: GeoHash) -> (Self, AirfieldRegisteredV1) {
        let airfield = Self {
            id: id.clone(),
            name: name.clone(),
            location: location.clone(),
        };

        let event = AirfieldRegisteredV1 {
            id,
            name,
            location,
        };

        (airfield, event)
    }
}

impl PartialEq for Airfield {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Airfield {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use prelude::domain::Event;
    use prelude::data::GeoHash;
    use crate::aggregate::airfield::Airfield;
    use crate::aggregate::{AirfieldId, AirfieldName};

    #[test]
    fn is_serializable() {
        let (original, _) = Airfield::build(
            airfield_id(),
            airfield_name(),
            location(),
        );

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Airfield = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.location, deserialized.location);
    }

    #[test]
    fn equals_by_id() {
        let (field1, _) = Airfield::build(
            airfield_id(),
            airfield_name(),
            location(),
        );

        let (field2, _) = Airfield::build(
            airfield_id(),
            "Amsterdam".parse().unwrap(),
            location(),
        );

        let (field3, _) = Airfield::build(
            "AAAA".parse().unwrap(),
            airfield_name(),
            location(),
        );

        assert_eq!(field1, field2);
        assert_ne!(field1, field3);
    }

    #[test]
    fn returns_event_on_build() {
        let (_, event) = Airfield::build(
            airfield_id(),
            airfield_name(),
            location(),
        );

        let data: Result<Event, _> = event.try_into();
        assert!(data.is_ok());
    }

    // test data
    fn airfield_id() -> AirfieldId {
        "LTCJ".parse().unwrap()
    }

    fn airfield_name() -> AirfieldName {
        "Batman Airport".parse().unwrap()
    }

    fn location() -> GeoHash {
        "sytpz2".parse().unwrap()
    }
}
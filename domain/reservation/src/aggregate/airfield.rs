use serde::{Deserialize, Serialize};
use prelude::data::GeoHash;
use crate::aggregate::AirfieldId;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Airfield {
    pub id: AirfieldId,
    pub(crate) name: String,
    pub(crate) location: GeoHash
}

impl Airfield {
    pub fn build(id: AirfieldId, name: String, location: GeoHash) -> Self {
        Self {
            id,
            name,
            location,
        }
    }
}

impl PartialEq for Airfield {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use prelude::data::GeoHash;
    use crate::aggregate::{Airfield, AirfieldId};

    #[test]
    fn equality_by_id() {
        let airfield1 = Airfield::build(id(), name(), location());
        let airfield2 = Airfield::build(id(), name2(), location2());
        assert_eq!(airfield1, airfield2);

        let airfield3 = Airfield::build(id2(), name2(), location2());
        assert_ne!(airfield1, airfield3);
    }

    #[test]
    fn is_serializable() {
        let original = Airfield::build(id(), name(), location());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Airfield = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.location, deserialized.location);
    }

    fn id() -> AirfieldId {
        "EHAM".parse().unwrap()
    }

    fn id2() -> AirfieldId {
        "ENLI".parse().unwrap()
    }

    fn name() -> String {
        "Schiphol Airport".to_owned()
    }

    fn name2() -> String {
        "Farsund Airport".to_owned()
    }

    fn location2() -> GeoHash {
        "sytpz2".parse().unwrap()
    }

    fn location() -> GeoHash {
        "sytpz3".parse().unwrap()
    }
}
use serde::{Deserialize, Serialize};
use prelude::data::GeoHash;
use crate::aggregate::{PlaceName};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Place {
    pub name: PlaceName,
    pub location: GeoHash,
}

impl Place {
    pub fn new(name: PlaceName, location: GeoHash) -> Self {
        Self {
            name,
            location,
        }
    }
}

impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

#[cfg(test)]
mod tests {
    use prelude::data::GeoHash;
    use crate::aggregate::{Place, PlaceName};

    # [test]
    fn equals_by_location() {
        let place1 = Place::new(name(), location());
        let place2 = Place::new(name2(), location());
        assert_eq!(place1, place2);

        let place3 = Place::new(name(), location2());
        assert_ne!(place1, place3);
    }

    #[test]
    fn is_serializable() {
        let original = Place::new(name(), location());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Place = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.location, deserialized.location);
        assert_eq!(original.name, deserialized.name);
    }

    fn location() -> GeoHash {
        "u4kf6x".parse().unwrap()
    }

    fn location2() -> GeoHash {
        "u4kf6z".parse().unwrap()
    }

    fn name() -> PlaceName {
        "Farsund, Norway".parse().unwrap()
    }

    fn name2() -> PlaceName {
        "Bergen, Norway".parse().unwrap()
    }
}


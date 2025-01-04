use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use geohash::GeohashError;
use geohash::Coord;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct GeoHash(String);

impl Display for GeoHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for GeoHash {
    type Err = GeohashError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let _ = geohash::decode(input)?;
        Ok(GeoHash(input.to_owned()))
    }
}

impl GeoHash {
    pub fn from_latitude_longitude(latitude: f32, longitude: f32, precision: usize) -> Result<GeoHash, GeohashError> {
        let value = geohash::encode(Coord::from((latitude as f64, longitude as f64)), precision)?;
        Ok(GeoHash(value))
    }
}

impl Serialize for GeoHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GeoHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value: String = Deserialize::deserialize(deserializer)?;
        Ok(GeoHash(value))
    }
}
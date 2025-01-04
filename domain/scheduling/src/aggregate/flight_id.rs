use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use prelude::data::{Uid, UidParseError};

#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Clone, Copy)]
pub struct FlightId(Uid);

pub type FlightIdError = UidParseError;

impl FlightId {
    pub fn new_random() -> Self {
        FlightId(Uid::new_random())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for FlightId {
    type Err = FlightIdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = input.parse()?;
        Ok(Self(value))
    }
}

impl Display for FlightId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::flight_id::{FlightId, FlightIdError};

    #[test]
    fn is_parseable() {
        let result: Result<FlightId, FlightIdError> = "5EPFciXgSxB70tAE8iERl6".parse();
        assert!(result.is_ok())
    }

    #[test]
    fn is_serializable() {
        let original: FlightId = "5EPFciXgSxB70tAE8iERl6".parse().unwrap();
        let serialized = serde_json::to_value(&original).unwrap();
        let deserialized: FlightId = serde_json::from_value(serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}

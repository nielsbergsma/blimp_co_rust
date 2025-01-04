use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use prelude::data::{Uid, UidParseError};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ReservationId(Uid);

pub type ReservationIdError = UidParseError;

impl ReservationId {
    pub fn new_random() -> Self {
        ReservationId(Uid::new_random())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for ReservationId {
    type Err = ReservationIdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = input.parse()?;
        Ok(Self(value))
    }
}

impl Display for ReservationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{ReservationId, ReservationIdError};

    #[test]
    fn is_parseable() {
        let result: Result<ReservationId, ReservationIdError> = "5EPFciXgSxB70tAE8iERl6".parse();
        assert!(result.is_ok())
    }

    #[test]
    fn is_serializable() {
        let original: ReservationId = "5EPFciXgSxB70tAE8iERl6".parse().unwrap();
        let serialized = serde_json::to_value(&original).unwrap();
        let deserialized: ReservationId = serde_json::from_value(serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}

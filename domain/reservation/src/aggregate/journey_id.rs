use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use prelude::data::{Uid, UidParseError};

#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Clone, Copy)]
pub struct JourneyId(Uid);

pub type JourneyIdError = UidParseError;

impl JourneyId {
    pub fn new_random() -> Self {
        JourneyId(Uid::new_random())
    }
}

impl FromStr for JourneyId {
    type Err = JourneyIdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = input.parse()?;
        Ok(Self(value))
    }
}

impl Display for JourneyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::journey_id::{JourneyId, JourneyIdError};

    #[test]
    fn is_parseable() {
        let result: Result<JourneyId, JourneyIdError> = "5EPFciXgSxB70tAE8iERl6".parse();
        assert!(result.is_ok())
    }

    #[test]
    fn is_serializable() {
        let original: JourneyId = "5EPFciXgSxB70tAE8iERl6".parse().unwrap();
        let serialized = serde_json::to_value(&original).unwrap();
        let deserialized: JourneyId = serde_json::from_value(serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}

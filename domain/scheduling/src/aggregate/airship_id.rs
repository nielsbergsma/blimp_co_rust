use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{end, is_a, Parser, sym};

#[derive(Error, Debug, PartialEq)]
pub enum AirshipIdError {
    #[error("malformed value")]
    MalformedValue,
}

// uses aircraft registration id as identity (https://en.wikipedia.org/wiki/List_of_aircraft_registration_prefixes)
#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Clone)]
pub struct AirshipId(String);

impl AirshipId {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (is_a(|c| c.is_ascii_uppercase() || c.is_ascii_digit()).repeat(1..4)
            + sym('-').opt()
            + is_a(|c| c.is_ascii_uppercase() || c.is_ascii_digit()).repeat(2..8)
            + end::<char>()
        ).collect()
    }
}

impl FromStr for AirshipId {
    type Err = AirshipIdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| AirshipIdError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for AirshipId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::airship_id::{AirshipId, AirshipIdError};

    #[test]
    fn is_parsable() {
        let result: Result<AirshipId, AirshipIdError> = "PH-1A1".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should only contain uppercase, digits and dashes
        let result: Result<AirshipId, AirshipIdError> = "ph-1a1".parse();
        assert_eq!(result, Err(AirshipIdError::MalformedValue));

        // no symbols, except for dashes
        let result: Result<AirshipId, AirshipIdError> = "N/A".parse();
        assert_eq!(result, Err(AirshipIdError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: AirshipId = "PH-1A1".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: AirshipId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
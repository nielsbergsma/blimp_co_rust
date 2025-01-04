use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, Parser, sym};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct JourneyName(String);

#[derive(Error, Debug, PartialEq)]
pub enum JourneyNameError {
    #[error("malformed value")]
    MalformedValue,
}

impl JourneyName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (alphanumeric() | sym(' ') | sym('-')).repeat(0..99).collect()
            + end::<char>()
        ).collect()
    }
}

impl FromStr for JourneyName {
    type Err = JourneyNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| JourneyNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for JourneyName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::journey_name::{JourneyName, JourneyNameError};

    #[test]
    fn is_parsable() {
        let result: Result<JourneyName, JourneyNameError> = "Journey Around North Atlantic".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should start with a capital
        let result: Result<JourneyName, JourneyNameError> = "journey around the backyard".parse();
        assert_eq!(result, Err(JourneyNameError::MalformedValue));

        // no symbols, except for spaces and dashes
        let result: Result<JourneyName, JourneyNameError> = "N/A".parse();
        assert_eq!(result, Err(JourneyNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: JourneyName = "Journey Around North Atlantic".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
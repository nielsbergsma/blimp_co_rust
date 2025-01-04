use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, Parser, sym};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlaceName(String);

#[derive(Error, Debug, PartialEq)]
pub enum PlaceNameError {
    #[error("malformed value")]
    MalformedValue,
}

impl PlaceName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (alphanumeric() | sym(' ') | sym(',') | sym('-')).repeat(0..99).collect()
            + end::<char>()
        ).collect()
    }
}

impl FromStr for PlaceName {
    type Err = PlaceNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| PlaceNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for PlaceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::place_name::{PlaceName, PlaceNameError};

    #[test]
    fn is_parsable() {
        let result: Result<PlaceName, PlaceNameError> = "Farsund".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should start with a capital
        let result: Result<PlaceName, PlaceNameError> = "farsund".parse();
        assert_eq!(result, Err(PlaceNameError::MalformedValue));

        // no symbols, except for spaces and dashes
        let result: Result<PlaceName, PlaceNameError> = "N/A".parse();
        assert_eq!(result, Err(PlaceNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: PlaceName = "Farsund".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
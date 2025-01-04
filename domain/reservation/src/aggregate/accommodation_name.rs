use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, Parser, sym};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AccommodationName(String);

#[derive(Error, Debug, PartialEq)]
pub enum AccommodationNameError {
    #[error("malformed value")]
    MalformedValue,
}

impl AccommodationName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (alphanumeric() | sym(' ') | sym('-')).repeat(0..99).collect()
            + end::<char>()
        ).collect()
    }
}

impl FromStr for AccommodationName {
    type Err = AccommodationNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| AccommodationNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for AccommodationName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::accommodation_name::{AccommodationName, AccommodationNameError};

    #[test]
    fn is_parsable() {
        let result: Result<AccommodationName, AccommodationNameError> = "Farsund Fjordhotel".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should start with a capital
        let result: Result<AccommodationName, AccommodationNameError> = "farsund fjordhotel".parse();
        assert_eq!(result, Err(AccommodationNameError::MalformedValue));

        // no symbols, except for spaces and dashes
        let result: Result<AccommodationName, AccommodationNameError> = "N/A".parse();
        assert_eq!(result, Err(AccommodationNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: AccommodationName = "Farsund Fjordhotel".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
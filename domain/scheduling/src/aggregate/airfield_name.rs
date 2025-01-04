use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, one_of, Parser, spaces};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AirfieldName(String);

#[derive(Error, Debug, PartialEq)]
pub enum AirfieldNameError {
    #[error("malformed value")]
    MalformedValue,
}

impl AirfieldName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (spaces().opt() + (one_of("-.") | alphanumeric())).repeat(0..99).collect()
            + end::<char>()
        ).collect()
    }
}

impl FromStr for AirfieldName {
    type Err = AirfieldNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| AirfieldNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for AirfieldName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::airfield_name::{AirfieldName, AirfieldNameError};

    #[test]
    fn is_parsable() {
        let result: Result<AirfieldName, AirfieldNameError> = "Bat-Man Airport".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should start with a capital
        let result: Result<AirfieldName, AirfieldNameError> = "amsterdam".parse();
        assert_eq!(result, Err(AirfieldNameError::MalformedValue));

        // no symbols, except for dashes
        let result: Result<AirfieldName, AirfieldNameError> = "N/A".parse();
        assert_eq!(result, Err(AirfieldNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: AirfieldName = "Amsterdam Airport".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: AirfieldName = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
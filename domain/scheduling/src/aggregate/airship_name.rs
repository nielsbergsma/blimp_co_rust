use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, Parser, spaces, sym};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AirshipName(String);

#[derive(Error, Debug, PartialEq)]
pub enum AirshipNameError {
    #[error("malformed value")]
    MalformedValue,
}

impl AirshipName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (spaces().opt() + (sym('-') | alphanumeric())).repeat(0..99).collect()
            + end::<char>()
        ).collect()
    }
}

impl FromStr for AirshipName {
    type Err = AirshipNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| AirshipNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for AirshipName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::airship_name::{AirshipName, AirshipNameError};

    #[test]
    fn is_parsable() {
        let result: Result<AirshipName, AirshipNameError> = "Galaxy 1".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // should start with a capital
        let result: Result<AirshipName, AirshipNameError> = "the galaxy".parse();
        assert_eq!(result, Err(AirshipNameError::MalformedValue));

        // no symbols, except for dashes
        let result: Result<AirshipName, AirshipNameError> = "N/A".parse();
        assert_eq!(result, Err(AirshipNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: AirshipName = "Galaxy 1".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
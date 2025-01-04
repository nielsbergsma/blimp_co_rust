use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{Parser, capital, end};

#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Clone)]
pub struct IcaoCode(String);

#[derive(Error, Debug, PartialEq)]
pub enum IcaoCodeError {
    #[error("malformed value")]
    MalformedValue,
}

impl IcaoCode {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (capital().repeat(4) + end::<char>()).collect()
    }
}

impl FromStr for IcaoCode {
    type Err = IcaoCodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = IcaoCode::parser()
            .parse_str(input)
            .map_err(|_| IcaoCodeError::MalformedValue)?
            .to_owned();

        Ok(Self(parsed))
    }
}

impl Display for IcaoCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}


#[cfg(test)]
mod tests {
    use crate::aggregate::icao_code::{IcaoCode, IcaoCodeError};

    #[test]
    fn is_parseable() {
        let code: Result<IcaoCode, IcaoCodeError> = "LTCJ".parse();
        assert!(code.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // empty string
        let result: Result<IcaoCode, IcaoCodeError> = "".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));

        // too few letters
        let result: Result<IcaoCode, IcaoCodeError> = "AMS".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));

        // use of symbols
        let result: Result<IcaoCode, IcaoCodeError> = "AM-S".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));

        // use of numbers
        let result: Result<IcaoCode, IcaoCodeError> = "AMS0".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));

        // too many letters
        let result: Result<IcaoCode, IcaoCodeError> = "AMSTE".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));

        // uses lowercase letters
        let result: Result<IcaoCode, IcaoCodeError> = "abcd".parse();
        assert_eq!(result, Err(IcaoCodeError::MalformedValue));
    }
}

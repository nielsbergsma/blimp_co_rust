use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{end, is_a, Parser, sym};

#[derive(Error, Debug, PartialEq, Clone)]
pub enum PhoneNumberError {
    #[error("malformed value")]
    MalformedValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (sym('+') + is_a(|c| c.is_ascii_digit()).repeat(4..15) + end::<char>()).collect()
    }
}

impl FromStr for PhoneNumber {
    type Err = PhoneNumberError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = Self::parser()
            .parse_str(input)
            .map_err(|_| PhoneNumberError::MalformedValue)?
            .to_owned();

        Ok(Self(parsed))
    }
}


#[cfg(test)]
mod tests {
    use crate::aggregate::{PhoneNumber, PhoneNumberError};

    #[test]
    fn is_parseable() {
        let number: Result<PhoneNumber, PhoneNumberError> = "+31653321799".parse();
        assert!(number.is_ok());
    }

    #[test]
    fn errors_on_malformed_values() {
        // must start with a '+' symbol
        let number: Result<PhoneNumber, PhoneNumberError> = "31653321799".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));

        // spaces are not allowed
        let number: Result<PhoneNumber, PhoneNumberError> = "+31 6533 217 99".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));

        // dashes are not allowed
        let number: Result<PhoneNumber, PhoneNumberError> = "+31-6533-217-99".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));

        // letters are not allowed
        let number: Result<PhoneNumber, PhoneNumberError> = "+31fivefivefive".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));

        // can't be less than 4 numbers
        let number: Result<PhoneNumber, PhoneNumberError> = "+310".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));

        // can't be more than 15 numbers
        let number: Result<PhoneNumber, PhoneNumberError> = "+1234567890123456".parse();
        assert_eq!(number, Err(PhoneNumberError::MalformedValue));
    }
}
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, end, one_of, Parser, sym};

#[derive(Error, Debug, PartialEq)]
pub enum PersonFullNameError {
    #[error("malformed value")]
    MalformedValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PersonFullName(String);

impl PersonFullName {
    fn parser<'a>() -> Parser<'a, &'a str> {
        (
            (alphanumeric() | one_of("-'.")).repeat(1..25)
                + (sym(' ') + (alphanumeric() | one_of(" -'.")).repeat(1..25)).repeat(1..10)
                + end::<char>()
        ).collect()
    }
}

impl FromStr for PersonFullName {
    type Err = PersonFullNameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = Self::parser()
            .parse_str(input)
            .map_err(|_| PersonFullNameError::MalformedValue)?
            .to_owned();

        Ok(Self(value))
    }
}

impl Display for PersonFullName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{PersonFullName, PersonFullNameError};

    #[test]
    fn is_parseable() {
        let name: Result<PersonFullName, PersonFullNameError> = "Niels Bergsma".parse();
        assert!(name.is_ok());

        let name: Result<PersonFullName, PersonFullNameError> = "Karina Cecilia Sands Sánchez".parse();
        assert!(name.is_ok());

        let name: Result<PersonFullName, PersonFullNameError> = "James O'Conner".parse();
        assert!(name.is_ok());

        let name: Result<PersonFullName, PersonFullNameError> = "John Smith-Jones".parse();
        assert!(name.is_ok());
    }

    #[test]
    fn errors_on_malformed_value() {
        // single names
        let name: Result<PersonFullName, PersonFullNameError> = "Niels".parse();
        assert_eq!(name, Err(PersonFullNameError::MalformedValue));

        // symbols are not allowed
        let name: Result<PersonFullName, PersonFullNameError> = ">|John|<".parse();
        assert_eq!(name, Err(PersonFullNameError::MalformedValue));

        // emojis are not allowed
        let name: Result<PersonFullName, PersonFullNameError> = "Don 🍩".parse();
        assert_eq!(name, Err(PersonFullNameError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: PersonFullName = "Niels Bergsma".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: PersonFullName = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}


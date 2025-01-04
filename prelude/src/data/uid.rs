use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;
use crate::encode::base62;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct Uid(u128);


#[derive(Error, Debug, PartialEq)]
pub enum UidParseError {
    #[error("malformed input (invalid characters)")]
    MalformedInput,

    #[error("value is empty")]
    ValueIsEmpty,
}


impl Uid {
    pub fn new_random() -> Uid {
        let id = Uuid::new_v4();
        Uid(id.as_u128())
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0u128
    }
}

impl FromStr for Uid {
    type Err = UidParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match base62::decode(input) {
            Some(0) => Err(UidParseError::ValueIsEmpty),
            Some(value) => Ok(Uid(value)),
            None => Err(UidParseError::MalformedInput),
        }
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = base62::encode(self.0);
        f.write_str(&encoded)
    }
}

impl Serialize for Uid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let encoded = base62::encode(self.0);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for Uid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value: Value = Deserialize::deserialize(deserializer)?;
        let value_str = value.as_str().ok_or_else(|| {
            serde::de::Error::custom("expected a string")
        })?;
        let value_u128 = base62::decode(value_str).ok_or_else(|| {
            serde::de::Error::custom("malformed value")
        })?;
        Ok(Uid(value_u128))
    }
}

#[cfg(test)]
mod tests {
    use crate::data::uid::{Uid, UidParseError};

    #[test]
    fn can_displayed() {
        let generated = Uid::new_random();
        let result = generated.to_string();
        assert!(result.len() > 10 && result.len() < 36);
    }

    #[test]
    fn generated_id_can_be_parsed() {
        let generated = Uid::new_random();
        let parsed: Uid = generated.to_string().parse().unwrap();
        assert_eq!(generated, parsed);
    }

    #[test]
    fn invalid_parse_input_results_in_error() {
        let parsed1: Result<Uid, UidParseError> = "#@$".parse();
        assert_eq!(parsed1, Err(UidParseError::MalformedInput));

        let parsed2: Result<Uid, UidParseError> = "AAAAAAAAAAAAAAAAAAAAAAAAAAAh".parse();
        assert_eq!(parsed2, Err(UidParseError::MalformedInput));

        let parsed3: Result<Uid, UidParseError> = "ZZZZZZZZZZZZZZZZZZZZZZ".parse();
        assert_eq!(parsed3, Err(UidParseError::MalformedInput));
    }

    #[test]
    fn is_serializable() {
        let id: Uid = "5EPFciXgSxB70tAE8iERl6".parse().unwrap();

        let serialized = serde_json::to_value(id.clone());
        assert!(serialized.is_ok());

        let deserialized: Result<Uid, serde_json::Error> = serde_json::from_value(serialized.unwrap());
        assert!(deserialized.is_ok());

        assert_eq!(id, deserialized.unwrap());
    }
}

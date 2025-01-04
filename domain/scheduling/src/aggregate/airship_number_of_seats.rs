use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AirshipNumberOfSeats(u8);

#[derive(Error, Debug, PartialEq)]
pub enum AirshipNumberOfSeatsError {
    #[error("value out of range")]
    OutOfRangeValue,
}

impl TryFrom<u8> for AirshipNumberOfSeats {
    type Error = AirshipNumberOfSeatsError;

    fn try_from(input: u8) -> Result<Self, Self::Error> {
        if input == 0 {
            return Err(AirshipNumberOfSeatsError::OutOfRangeValue)
        }
        Ok(Self(input))
    }
}

impl AirshipNumberOfSeats {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{AirshipNumberOfSeats, AirshipNumberOfSeatsError};

    #[test]
    fn initiate_from_u8() {
        let result: Result<AirshipNumberOfSeats, _> = AirshipNumberOfSeats::try_from(10);
        assert!(result.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // zero value is not allowed
        let result: Result<AirshipNumberOfSeats, _> = AirshipNumberOfSeats::try_from(0);
        assert_eq!(result, Err(AirshipNumberOfSeatsError::OutOfRangeValue));
    }
}
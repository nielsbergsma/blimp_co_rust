use chrono::{NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::collection::SortedSet;
use crate::aggregate::PersonFullName;
use prelude::data::chrono::*;

#[derive(Error, Debug, PartialEq)]
pub enum PassengersError {
    #[error("no passengers")]
    NoPassengers,

    #[error("too many passengers")]
    TooManyPassengers,

    #[error("number of passengers are different")]
    NumberOfPassengersAreDifferent,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Passengers {
    Arrangement(PassengerArrangement),
    List(SortedSet<Passenger>),
}

impl Passengers {
    pub fn new(arrangement: PassengerArrangement) -> Self {
        Self::Arrangement(arrangement)
    }

    pub fn list(self, passengers: SortedSet<Passenger>) -> Result<Self, PassengersError> {
        match self {
            Self::Arrangement(arrangement) => {
                if arrangement.count() != passengers.len() as u8 {
                    return Err(PassengersError::NumberOfPassengersAreDifferent);
                }
                Ok(Passengers::List(passengers))
            }

            Self::List(list) => {
                if list.len() != passengers.len() {
                    return Err(PassengersError::NumberOfPassengersAreDifferent);
                }
                Ok(Passengers::List(passengers))
            }
        }
    }

    pub fn arrangement(&self, adults_as_of_date: NaiveDate) -> PassengerArrangement {
        match self {
            Self::Arrangement(arrangement) => {
                arrangement.clone()
            }

            Self::List(list) => {
                let adults = list.iter()
                    .filter(|p| years_between(p.date_of_birth, adults_as_of_date) >= 18)
                    .count();

                let children = list.len() - adults;

                PassengerArrangement {
                    adults: adults as u8,
                    children: children as u8,
                }
            }
        }
    }

    pub fn count(&self) -> u8 {
        match self {
            Passengers::Arrangement(arrangement) => {
                arrangement.count()
            }

            Passengers::List(list) => {
                list.len() as u8
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PassengerArrangement {
    pub adults: u8,
    pub children: u8,
}

impl PassengerArrangement {
    pub fn build(adults: u8, children: u8) -> Result<Self, PassengersError> {
        let total = adults as u16 + children as u16;
        if total < 1 {
            return Err(PassengersError::NoPassengers);
        }
        if total > 255u16 {
            return Err(PassengersError::TooManyPassengers);
        }

        Ok(Self{adults, children})
    }

    pub fn count(&self) -> u8 {
        self.adults + self.children
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Passenger {
    name: PersonFullName,
    date_of_birth: NaiveDate
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use prelude::collection::SortedSet;
    use crate::aggregate::{Passenger, PassengerArrangement, Passengers, PassengersError};

    #[test]
    fn arrangements_are_buildable() {
        let passengers = PassengerArrangement::build(2,2);
        assert!(passengers.is_ok());
    }

    #[test]
    fn arrangements_error_on_malformed_input() {
        // no people is not allowed
        let passengers = PassengerArrangement::build(0,0);
        assert_eq!(passengers, Err(PassengersError::NoPassengers));

        // > 255
        let passengers = PassengerArrangement::build(255,255);
        assert_eq!(passengers, Err(PassengersError::TooManyPassengers));
    }

    #[test]
    fn arrangement_returns_count_of_passengers() {
        let arrangement = PassengerArrangement::build(2,3).unwrap();
        assert_eq!(arrangement.count(), 5);
    }

    #[test]
    fn passengers_can_only_be_initiated_by_arrangement() {
        let passengers = Passengers::new(arrangement());
        assert_eq!(passengers, Passengers::Arrangement(arrangement()));
    }

    #[test]
    fn list_changes_arrangement_to_list() {
        let before = Passengers::new(arrangement());
        let after = before.list(SortedSet::empty().insert(passenger()).insert(passenger2()));
        assert!(after.is_ok());
    }

    #[test]
    fn list_errors_on_malformed_input() {
        let before = Passengers::new(arrangement());

        // passenger count differs
        let after = before.list(SortedSet::empty());
        assert_eq!(after, Err(PassengersError::NumberOfPassengersAreDifferent));
    }

    #[test]
    fn can_get_arrangement_from_list() {
        let list = Passengers::new(arrangement())
            .list(SortedSet::empty().insert(passenger()).insert(passenger2()))
            .unwrap();

        let today = Utc::now().date_naive();
        let list_arrangement = list.arrangement(today);
        assert_eq!(list_arrangement, arrangement());
    }

    fn arrangement() -> PassengerArrangement {
        PassengerArrangement::build(2,0).unwrap()
    }

    fn passenger() -> Passenger {
        Passenger {
            name: "Niels Bergsma".parse().unwrap(),
            date_of_birth: "1983-10-21".parse().unwrap(),
        }
    }

    fn passenger2() -> Passenger {
        Passenger {
            name: "Karina Sands".parse().unwrap(),
            date_of_birth: "1980-09-03".parse().unwrap(),
        }
    }
}
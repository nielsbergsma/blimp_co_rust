use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use crate::aggregate::airship_id::AirshipId;
use crate::aggregate::AirshipModel;
use crate::aggregate::AirshipName;
use crate::aggregate::AirshipNumberOfSeats;
use crate::event::AirshipAddedToFleetV1;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Airship {
    pub id: AirshipId,
    name: AirshipName,
    model: AirshipModel,
    pub(crate) number_of_seats: AirshipNumberOfSeats,
}

impl Airship {
    pub fn build(id: AirshipId, name: AirshipName, model: AirshipModel, number_of_seats: AirshipNumberOfSeats) -> (Self, AirshipAddedToFleetV1) {
        let airship = Self {
            id: id.clone(),
            name: name.clone(),
            model: model.clone(),
            number_of_seats: number_of_seats.clone(),
        };

        let event = AirshipAddedToFleetV1 {
            id,
            name,
            model,
            number_of_seats,
        };

        (airship, event)
    }
}

impl Hash for Airship {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq<Self> for Airship {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use prelude::domain::Event;
    use crate::aggregate::{Airship, AirshipModel, AirshipName, AirshipNumberOfSeats, AirshipId};

    #[test]
    fn is_serializable() {
            let (result, _) = Airship::build(
                airship_id(),
                airship_name(),
                airship_model(),
                number_of_seats()
            );

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: Airship = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.id, deserialized.id);
        assert_eq!(result.name, deserialized.name);
        assert_eq!(result.model, deserialized.model);
        assert_eq!(result.number_of_seats, deserialized.number_of_seats);
    }

    #[test]
    fn equals_by_id() {
        let (ship1, _) = Airship::build(
            airship_id(),
            airship_name(),
            airship_model(),
            number_of_seats()
        );

        let (ship2, _) = Airship::build(
            airship_id(),
            "Galaxy 2".parse().unwrap(),
            airship_model(),
            number_of_seats()
        );

        let (ship3, _) = Airship::build(
            "PH-2B2".parse().unwrap(),
            airship_name(),
            airship_model(),
            number_of_seats()
        );

        // ship 1 and 2 have the same id, should to be equal
        assert_eq!(ship1, ship2);

        // ship 1 and 3 have different ids, should not be equal
        assert_ne!(ship1, ship3);
    }

    #[test]
    fn returns_event_on_build() {
        let (_, event) = Airship::build(
            airship_id(),
            airship_name(),
            airship_model(),
            number_of_seats()
        );

        let data: Result<Event, _> = event.try_into();
        assert!(data.is_ok());
    }

    // test data
    fn airship_id() -> AirshipId {
        "PH-1A1".parse().unwrap()
    }

    fn airship_name() -> AirshipName {
        "Galaxy One".parse().unwrap()
    }

    fn airship_model() -> AirshipModel {
        "Blimp 1".parse().unwrap()
    }

    fn number_of_seats() -> AirshipNumberOfSeats {
        AirshipNumberOfSeats::try_from(10).unwrap()
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{AirfieldId};

#[derive(Error, Debug, PartialEq)]
pub enum FlightRouteError {
    #[error("departure and arrival are the same")]
    DepartureAndArrivalAreTheSame,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct FlightRoute {
    pub departure: AirfieldId,
    pub arrival: AirfieldId,
}

impl PartialEq for FlightRoute {
    fn eq(&self, other: &Self) -> bool {
        self.departure == other.departure && self.arrival == other.arrival
    }
}

impl FlightRoute {
    pub fn build(departure: AirfieldId, arrival: AirfieldId) -> Result<Self, FlightRouteError> {
        if departure == arrival {
            return Err(FlightRouteError::DepartureAndArrivalAreTheSame);
        }

        Ok(Self {
            departure,
            arrival,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{AirfieldId, FlightRouteError};
    use crate::aggregate::flight_route::FlightRoute;

    #[test]
    fn is_buildable() {
        let route= FlightRoute::build(airfield_enli(), airfield_eham());
        assert!(route.is_ok())
    }

    #[test]
    fn errors_on_malformed_input() {
        // same departure and arrival airfield
        let route= FlightRoute::build(airfield_eham(), airfield_eham());
        assert_eq!(route, Err(FlightRouteError::DepartureAndArrivalAreTheSame));
    }

    #[test]
    fn equality_by_values() {
        let route1 = FlightRoute::build(airfield_eham(), airfield_enli()).unwrap();
        let route2 = FlightRoute::build(airfield_eham(), airfield_enli()).unwrap();
        assert_eq!(route1, route2);

        let route3 = FlightRoute::build(airfield_eham(), airfield_enbr()).unwrap();
        assert_ne!(route1, route3);
    }

    #[test]
    fn is_serializable() {
        let route = FlightRoute::build(airfield_eham(), airfield_enli()).unwrap();
        let serialized = serde_json::to_string(&route).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(route, deserialized);
    }

    fn airfield_eham() -> AirfieldId {
        "EHAM".parse().unwrap()
    }

    fn airfield_enli() -> AirfieldId {
        "ENLI".parse().unwrap()
    }

    fn airfield_enbr() -> AirfieldId {
        "ENBR".parse().unwrap()
    }
}


use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::collection::{SortedSet};
use crate::aggregate::{Accommodation, AirfieldId, FlightRoute};

#[derive(Error, Debug, PartialEq)]
pub enum SegmentError {
    #[error("too many accommodations")]
    TooManyAccommodations,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segment {
    pub(crate) flight: FlightRoute,
    pub(crate) accommodations: SortedSet<Accommodation>
}

impl Segment {
    pub fn build(flight: FlightRoute, accommodations: SortedSet<Accommodation>) -> Result<Self, SegmentError> {
        if accommodations.len() > 10 {
            return Err(SegmentError::TooManyAccommodations);
        }

        Ok(Self{
            flight,
            accommodations,
        })
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.flight == other.flight
    }
}

impl Segment {
    pub fn departs_from(&self) -> AirfieldId {
        self.flight.departure.clone()
    }

    pub fn arrives_at(&self) -> AirfieldId {
        self.flight.arrival.clone()
    }
}

#[cfg(test)]
mod tests {
    use prelude::collection::SortedSet;
    use crate::aggregate::{Accommodation, AccommodationId, FlightRoute, Picture, Place, Segment, SegmentError};

    #[test]
    fn is_buildable() {
        let segment = Segment::build(flight(), SortedSet::singleton(accommodation()));
        assert!(segment.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // too many accommodations
        let segment = Segment::build(flight(), accommodations(11));
        assert_eq!(segment, Err(SegmentError::TooManyAccommodations));
    }

    #[test]
    fn equals_by_id() {
        let segment1 = Segment::build(flight(), SortedSet::singleton(accommodation())).unwrap();
        let segment2 = Segment::build(flight(), SortedSet::empty()).unwrap();
        assert_eq!(segment1, segment2);

        let segment3 = Segment::build(flight2(), SortedSet::singleton(accommodation())).unwrap();
        assert_ne!(segment1, segment3);
    }

    #[test]
    fn is_serializable() {
        let original = Segment::build(flight(), SortedSet::singleton(accommodation())).unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Segment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.flight, deserialized.flight);
        assert_eq!(original.accommodations, deserialized.accommodations);
    }

    fn flight() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENLI".parse().unwrap(),
        ).unwrap()
    }

    fn flight2() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENBR".parse().unwrap(),
        ).unwrap()
    }

    fn accommodations(count: u8) -> SortedSet<Accommodation> {
        let mut set = SortedSet::empty();
        for _ in 0..count {
            let accommodation = Accommodation::build(
                AccommodationId::new_random(),
                "Farsund Fjordhotel".parse().unwrap(),
                Place::new(
                    "Farsund, Norway".parse().unwrap(),
                    "u4kf6x".parse().unwrap(),
                ),
                SortedSet::singleton(Picture::build(
                    "https://www.visitnorway.com/img/farsund.jpg".parse().unwrap(),
                    "Farsund Resort".to_owned(),
                ).unwrap())
            ).unwrap();

            set = set.insert(accommodation);
        }
        set
    }

    fn accommodation() -> Accommodation {
        Accommodation::build(
            "5EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            "Farsund Fjordhotel".parse().unwrap(),
            Place::new(
                "Farsund, Norway".parse().unwrap(),
                "u4kf6x".parse().unwrap(),
            ),
            SortedSet::singleton(Picture::build(
                "https://www.visitnorway.com/img/farsund.jpg".parse().unwrap(),
                "Farsund Resort".to_owned(),
            ).unwrap())
        ).unwrap()
    }
}
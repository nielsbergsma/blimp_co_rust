use std::collections::{LinkedList};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::collection::SortedSet;
use crate::aggregate::{Accommodation, AccommodationId, AirfieldId, Flight, Itinerary, ItineraryError, JourneyId, JourneyName, Segment};
use crate::event::JourneyPublishedV1;

pub const JOURNEY_MIN_DAYS_IN_ACCOMMODATION: i64 = 3;
pub const JOURNEY_MAX_DAYS_IN_ACCOMMODATION: i64 = 21;

#[derive(Error, Debug, PartialEq)]
pub enum JourneyError {
    #[error("too few segments")]
    TooFewSegments,

    #[error("too many segments")]
    TooManySegments,

    #[error("segments don't form a round trip")]
    SegmentsDontFormARoundTrip,
}

/// Journey is a "blueprint" for the actual itinerary
/// it consists of segments of flight routes, with resp. accommodation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Journey {
    pub id: JourneyId,
    name: JourneyName,
    segments: SortedSet<Segment>,
}

impl PartialEq for Journey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Journey {
    pub fn build(id: JourneyId, name: JourneyName, segments: SortedSet<Segment>) -> Result<(Self, JourneyPublishedV1), JourneyError> {
        if segments.len() < 2 {
            return Err(JourneyError::TooFewSegments);
        }
        if segments.len() > 20 {
            return Err(JourneyError::TooManySegments);
        }
        if !Self::segments_form_round_trip(&segments) {
            return Err(JourneyError::SegmentsDontFormARoundTrip);
        }

        let journey = Self {
            id,
            name: name.clone(),
            segments: segments.clone(),
        };

        let event = JourneyPublishedV1 {
            id,
            name,
            segments,
        };

        Ok((journey, event))
    }

    pub fn parse_itinerary(&self, stages: LinkedList<(Flight, Option<Accommodation>)>) -> Result<Itinerary, ItineraryError> {
        let first_stage = stages.front().ok_or(ItineraryError::NoStages)?;
        let last_stage = stages.back().ok_or(ItineraryError::NoStages)?;

        // not allowed to have a stay after the last flight
        if last_stage.1.is_some() {
            return Err(ItineraryError::LastStageHasAccommodation);
        }

        let route = self
            .resolve_route(&first_stage.0.route.departure, &last_stage.0.route.arrival)
            .ok_or(ItineraryError::MalformedRoute)?;

        if route.len() != stages.len() {
            return Err(ItineraryError::MalformedRoute);
        }

        // verify stages follow journey (flights) and accommodations in segment
        for (segment, (stage_flight, stage_accommodation)) in route.iter().zip(stages.clone()) {
            if segment.flight != stage_flight.route {
               return Err(ItineraryError::MalformedRoute);
            }

            if let Some(accommodation) = stage_accommodation {
                if !segment.accommodations.contains(&accommodation) {
                    return Err(ItineraryError::AccommodationNotInStage);
                }
            }
        }

        // verify time between flights, and accommodation align and are within bounds
        for ((arrive, accommodation), (depart, _)) in stages.iter().zip(stages.iter().skip(1)) {
            let duration_between_flights = depart.departure - arrive.arrival;
            let days_between_flights = (depart.departure.date_naive() - arrive.arrival.date_naive()).num_days();

            if duration_between_flights < Duration::zero() {
                return Err(ItineraryError::FlightsAreNotConsecutive);
            }

            if accommodation.is_some() && days_between_flights < JOURNEY_MIN_DAYS_IN_ACCOMMODATION {
                return Err(ItineraryError::DaysInAccommodationIsTooShort);
            }

            if days_between_flights > JOURNEY_MAX_DAYS_IN_ACCOMMODATION {
                return Err(ItineraryError::DaysInAccommodationIsTooLong);
            }
        }

        Itinerary::from_iter(stages)
    }

    pub fn find_accommodation(&self, location: &AirfieldId, id: &AccommodationId) -> Option<&Accommodation> {
        self.segments.iter()
            .filter(|segment| segment.flight.arrival == location.clone())
            .flat_map(|segment| segment.accommodations.iter())
            .find(|accommodation| accommodation.id == *id)
    }

    fn resolve_route(&self, departure: &AirfieldId, arrival: &AirfieldId) -> Option<LinkedList<&Segment>> {
        let mut route = LinkedList::new();
        let mut departs_from = departure.clone();

        let mut resolved = false;
        while !resolved {
            let segment = self.segments.find(|p| p.departs_from() == departs_from)?;

            route.push_back(segment);
            if route.len() > self.segments.len() {
                return None;
            }

            departs_from = segment.arrives_at();
            resolved = &departs_from == arrival;
        }

        Some(route)
    }

    fn segments_form_round_trip(segments: &SortedSet<Segment>) -> bool {
        for segment in segments.iter() {
            let connects_exactly_once = segments
                .iter()
                .filter(|s| s.departs_from() == segment.arrives_at())
                .count() == 1;

            if !connects_exactly_once {
                return false;
            }
        }

        segments.len() > 1
    }
}


#[cfg(test)]
mod tests {
    use std::collections::{LinkedList};
    use prelude::collection::SortedSet;
    use crate::aggregate::{Accommodation, Flight, FlightRoute, ItineraryError, Journey, JourneyError, JourneyId, JourneyName, Picture, Place, Segment};

    #[test]
    fn is_buildable() {
        let journey = Journey::build(id(), name(), segments());
        assert!(journey.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // too few segments
        let journey = Journey::build(id(), name(), SortedSet::empty());
        assert_eq!(journey, Err(JourneyError::TooFewSegments));

        let journey = Journey::build(id(), name(), SortedSet::singleton(segment_eham_enli()));
        assert_eq!(journey, Err(JourneyError::TooFewSegments));

        // segments don't form round trip
        let segments = SortedSet::empty()
            .insert(segment_eham_enli())
            .insert(segment_eham_enbr());
        let journey = Journey::build(id(), name(), segments);
        assert_eq!(journey, Err(JourneyError::SegmentsDontFormARoundTrip));
    }

    #[test]
    fn equality_on_id() {
        let (journey1, _) = Journey::build(id(), name(), segments()).unwrap();
        let (journey2, _) = Journey::build(id(), name2(), segments()).unwrap();
        assert_eq!(journey1, journey2);

        let (journey3, _) = Journey::build(id2(), name(), segments()).unwrap();
        assert_ne!(journey1, journey3);
    }

    #[test]
    fn is_serializable() {
        let (original, _) = Journey::build(id(), name(), segments()).unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Journey = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.segments, deserialized.segments);
    }

    #[test]
    fn stages_can_be_parsed_into_a_itinerary() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), Some(accommodation_enli())),
            (flight_enli_eham_14jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert!(result.is_ok());
    }

    #[test]
    fn stages_can_start_anywhere() {
        let stages = LinkedList::from([
            (flight_enli_eham_14jan(), None),
            (flight_eham_enli_20jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert!(result.is_ok());
    }

    #[test]
    fn errors_if_no_stages_are_given() {
        let stages = LinkedList::new();

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::NoStages));
    }

    #[test]
    fn errors_if_last_stage_has_accommodation() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), Some(accommodation_enli())),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::LastStageHasAccommodation));
    }

    #[test]
    fn errors_if_flight_is_not_part_of_journey() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), None),
            (flight_enbr_eham_14jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::MalformedRoute));
    }

    #[test]
    fn errors_if_not_a_round_trip() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), None),
            (flight_enli_eham_14jan(), None),
            (flight_eham_enli_20jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::MalformedRoute));
    }

    #[test]
    fn errors_if_flights_are_not_consecutive() {
        let stages = LinkedList::from([
            (flight_enli_eham_14jan(), None),
            (flight_eham_enli_8jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::FlightsAreNotConsecutive));
    }

    #[test]
    fn errors_stay_is_shorter_then_minimal() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), Some(accommodation_enli())),
            (flight_enli_eham_9jan(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::DaysInAccommodationIsTooShort));
    }

    #[test]
    fn errors_stay_is_longer_then_maximum() {
        let stages = LinkedList::from([
            (flight_eham_enli_8jan(), Some(accommodation_enli())),
            (flight_enli_eham_1apr(), None),
        ]);

        let journey = journey();
        let result = journey.parse_itinerary(stages);
        assert_eq!(result, Err(ItineraryError::DaysInAccommodationIsTooLong));
    }

    // data generators
    fn id() -> JourneyId {
        "5EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn id2() -> JourneyId {
        "5EPFciXgSxB70tAE8iERl7".parse().unwrap()
    }

    fn name() -> JourneyName {
        "Journey Around North Atlantic".parse().unwrap()
    }

    fn name2() -> JourneyName {
        "Journey Around South Atlantic".parse().unwrap()
    }

    fn segments() -> SortedSet<Segment> {
        SortedSet::empty()
            .insert(segment_eham_enli())
            .insert(segment_enli_eham())
    }

    fn segment_eham_enli() -> Segment {
        let flight = route_eham_enli();
        let accommodation = accommodation_enli();
        Segment::build(flight, SortedSet::singleton(accommodation)).unwrap()
    }

    fn segment_eham_enbr() -> Segment {
        Segment::build(route_eham_enbr(), SortedSet::empty()).unwrap()
    }

    fn segment_enli_eham() -> Segment {
        Segment::build(route_enli_eham(), SortedSet::empty()).unwrap()
    }

    fn journey() -> Journey {
        let (journey, _) = Journey::build(id(), name(), segments()).unwrap();
        journey
    }

    fn route_eham_enli() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENLI".parse().unwrap(),
        ).unwrap()
    }

    fn route_eham_enbr() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENBR".parse().unwrap(),
        ).unwrap()
    }

    fn route_enbr_eham() -> FlightRoute {
        FlightRoute::build(
            "ENBR".parse().unwrap(),
            "EHAM".parse().unwrap(),
        ).unwrap()
    }

    fn route_enli_eham() -> FlightRoute {
        FlightRoute::build(
            "ENLI".parse().unwrap(),
            "EHAM".parse().unwrap(),
        ).unwrap()
    }

    fn flight_eham_enli_8jan() -> Flight {
        Flight::build(
            "9EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_eham_enli(),
            "2024-01-08T09:00:00+05:00".parse().unwrap(),
            "2024-01-08T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_eham_enli_20jan() -> Flight {
        Flight::build(
            "9EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_eham_enli(),
            "2024-01-20T09:00:00+05:00".parse().unwrap(),
            "2024-01-20T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_enbr_eham_14jan() -> Flight {
        Flight::build(
            "8EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_enbr_eham(),
            "2024-01-14T09:00:00+05:00".parse().unwrap(),
            "2024-01-14T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_enli_eham_9jan() -> Flight {
        Flight::build(
            "8EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_enli_eham(),
            "2024-01-9T09:00:00+05:00".parse().unwrap(),
            "2024-01-9T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_enli_eham_14jan() -> Flight {
        Flight::build(
            "8EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_enli_eham(),
            "2024-01-14T09:00:00+05:00".parse().unwrap(),
            "2024-01-14T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_enli_eham_1apr() -> Flight {
        Flight::build(
            "8EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_enli_eham(),
            "2024-04-01T09:00:00+05:00".parse().unwrap(),
            "2024-04-01T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn accommodation_enli() -> Accommodation {
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
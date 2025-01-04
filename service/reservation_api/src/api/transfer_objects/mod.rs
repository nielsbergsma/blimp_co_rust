use std::collections::LinkedList;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use prelude::collection::SortedSet;
use reservation::aggregate;
use reservation::aggregate::ReservationId;
use crate::api::ApiResult;

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub error: String
}

#[derive(Serialize, Deserialize)]
pub struct PostJourneyRequest {
    pub name: String,
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize)]
pub struct PostJourneyResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Segment {
    pub flight: FlightRoute,
    pub accommodations: Vec<Accommodation>
}

pub fn parse_segments(value: Vec<Segment>) -> ApiResult<SortedSet<aggregate::Segment>> {
    let mut segments = SortedSet::empty();
    for segment in value {
        segments = segments.insert(parse_segment(segment)?);
    }
    Ok(segments)
}

pub fn parse_segment(value: Segment) -> ApiResult<aggregate::Segment> {
    let flight_route = parse_flight_route(value.flight)?;
    let accommodations = parse_accommodations(value.accommodations)?;

    aggregate::Segment::build(flight_route, accommodations)
        .map_err(|error| error.into())
}

#[derive(Serialize, Deserialize)]
pub struct FlightRoute {
    pub departure: String,
    pub arrival: String
}

pub fn parse_flight_route(value: FlightRoute) -> ApiResult<aggregate::FlightRoute> {
    let departure = value.departure.parse()?;
    let arrival= value.arrival.parse()?;

    aggregate::FlightRoute::build(departure, arrival)
        .map_err(|error| error.into())
}

#[derive(Serialize, Deserialize)]
pub struct Accommodation {
    pub name: String,
    pub place: Place,
    pub pictures: Vec<Picture>,
}

pub fn accommodation_to_transfer_object(value: aggregate::Accommodation) -> Accommodation {
    Accommodation {
        name: value.name.to_string(),
        place: place_to_transfer_object(value.place),
        pictures: value.pictures.into_iter()
            .map(picture_to_transfer_object)
            .collect()
    }
}

pub fn parse_accommodations(value: Vec<Accommodation>) -> ApiResult<SortedSet<aggregate::Accommodation>> {
    let mut accommodations = SortedSet::empty();
    for accommodation in value {
        accommodations = accommodations.insert(parse_accommodation(accommodation)?);
    }
    Ok(accommodations)
}

pub fn parse_accommodation(value: Accommodation) -> ApiResult<aggregate::Accommodation> {
    let id = aggregate::AccommodationId::new_random();
    let name = value.name.parse()?;
    let place = parse_place(value.place)?;
    let pictures = parse_pictures(value.pictures)?;

    aggregate::Accommodation::build(id, name, place, pictures)
        .map_err(|error| error.into())
}

#[derive(Serialize, Deserialize)]
pub struct Place {
    pub name: String,
    pub location: String,
}

pub fn place_to_transfer_object(value: aggregate::Place) -> Place {
    Place {
        name: value.name.to_string(),
        location: value.location.to_string(),
    }
}

pub fn parse_place(value: Place) -> ApiResult<aggregate::Place> {
    let id = value.location.parse()?;
    let name = value.name.parse()?;

    Ok(aggregate::Place::new(name, id))
}

#[derive(Serialize, Deserialize)]
pub struct Picture {
    pub url: String,
    pub caption: String,
}

pub fn picture_to_transfer_object(value: aggregate::Picture) -> Picture {
    Picture {
        url: value.url.to_string(),
        caption: value.caption.to_string(),
    }
}

pub fn parse_pictures(value: Vec<Picture>) -> ApiResult<SortedSet<aggregate::Picture>> {
    let mut pictures = SortedSet::empty();
    for picture in value {
        pictures = pictures.insert(parse_picture(picture)?);
    }
    Ok(pictures)
}

pub fn parse_picture(value: Picture) -> ApiResult<aggregate::Picture> {
    let url =  value.url.parse()?;
    let caption = value.caption;

    aggregate::Picture::build(url, caption)
        .map_err(|error| error.into())
}

#[derive(Serialize, Deserialize)]
pub struct PostReservationsRequest {
    pub journey: String,
    pub contact: Contact,
    pub passengers: PassengerArrangement,
    pub itinerary: Vec<ItineraryStage>,
}

#[derive(Serialize, Deserialize)]
pub struct PostReservationsResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub email: String,
}

pub fn parse_contact(value: Contact) -> ApiResult<aggregate::Contact> {
    Ok(aggregate::Contact::new(value.name.parse()?, value.email.parse()?, None))
}

pub fn contact_to_transfer_object(value: aggregate::Contact) -> Contact {
    Contact {
        name: value.name.to_string(),
        email: value.email.to_string(),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Passengers {
    Arrangement(PassengerArrangement)
}

pub fn parse_passengers(value: Passengers) -> ApiResult<aggregate::Passengers> {
    match value {
        Passengers::Arrangement(arrangement_value) => {
            Ok(aggregate::Passengers::new(
                parse_passenger_arrangement(arrangement_value)?
            ))
        }
    }
}


pub fn passengers_to_transfer_object(value: aggregate::Passengers) -> Passengers {
    let today = Utc::now().date_naive();

    Passengers::Arrangement(passengers_arrangement_to_transfer_object(
        value.arrangement(today)
    ))
}

#[derive(Serialize, Deserialize)]
pub struct PassengerArrangement {
    pub adults: u8,
    pub children: u8,
}

pub fn passengers_arrangement_to_transfer_object(value: aggregate::PassengerArrangement) -> PassengerArrangement {
    PassengerArrangement {
        adults: value.adults,
        children: value.children,
    }
}

pub fn parse_passenger_arrangement(value: PassengerArrangement) -> ApiResult<aggregate::PassengerArrangement> {
    aggregate::PassengerArrangement::build(value.adults, value.children)
        .map_err(|error| error.into())
}

pub type Itinerary = Vec<FullItineraryStage>;

pub fn itinerary_to_transfer_object(value: aggregate::Itinerary) -> Itinerary {
    value.stages()
        .map(|stage| itinerary_stage_to_transfer_object(stage.clone()))
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FullItineraryStage {
    Planned(FlightAndAccommodation),
    Reserved(FlightAndAccommodation),
    ReservedFailed(FlightAndAccommodation),
    Annulled(FlightAndAccommodation)
}

pub fn itinerary_stage_to_transfer_object(value: aggregate::ItineraryStage) -> FullItineraryStage {
    match value {
        aggregate::ItineraryStage::Planned(flight, accommodation) => {
            let flight_and_accommodation = FlightAndAccommodation {
                flight: flight_to_transfer_object(flight),
                accommodation: accommodation.map(accommodation_to_transfer_object)
            };
            FullItineraryStage::Planned(flight_and_accommodation)
        }

        aggregate::ItineraryStage::Reserved(flight, accommodation) => {
            let flight_and_accommodation = FlightAndAccommodation {
                flight: flight_to_transfer_object(flight),
                accommodation: accommodation.map(accommodation_to_transfer_object)
            };
            FullItineraryStage::Reserved(flight_and_accommodation)
        }

        aggregate::ItineraryStage::ReservedFailed(flight, accommodation, _) => {
            let flight_and_accommodation = FlightAndAccommodation {
                flight: flight_to_transfer_object(flight),
                accommodation: accommodation.map(accommodation_to_transfer_object)
            };
            FullItineraryStage::ReservedFailed(flight_and_accommodation)
        }

        aggregate::ItineraryStage::Annulled(flight, accommodation) => {
            let flight_and_accommodation = FlightAndAccommodation {
                flight: flight_to_transfer_object(flight),
                accommodation: accommodation.map(accommodation_to_transfer_object)
            };
            FullItineraryStage::Annulled(flight_and_accommodation)
        }
    }
}

#[derive(Serialize)]
pub struct FlightAndAccommodation {
    flight: Flight,
    accommodation: Option<Accommodation>
}

#[derive(Serialize, Deserialize)]
pub struct ItineraryStage {
    pub flight: String,
    pub accommodation: Option<String>
}

pub fn parse_itinerary(value: Vec<ItineraryStage>) -> ApiResult<LinkedList<(aggregate::FlightId, Option<aggregate::AccommodationId>)>> {
    let mut result = LinkedList::default();
    for element in value.into_iter() {
        let flight = element.flight.parse()?;
        let accommodation = match element.accommodation {
            Some(id) => Some(id.parse()?),
            None => None,
        };

        result.push_back((flight, accommodation));
    }

    Ok(result)
}

#[derive(Serialize)]
pub struct Flight {
    id: String,
    departure: FlightDeparture,
    arrival: FlightArrival,
}

#[derive(Serialize)]
pub struct FlightDeparture {
    airfield: String,
    time: DateTime<FixedOffset>
}

#[derive(Serialize)]
pub struct FlightArrival {
    airfield: String,
    time: DateTime<FixedOffset>
}

pub fn flight_to_transfer_object(value: aggregate::Flight) -> Flight {
    Flight {
        id: value.id.to_string(),
        departure: FlightDeparture {
            airfield: value.route.departure.to_string(),
            time: value.departure,
        },
        arrival: FlightArrival {
            airfield: value.route.arrival.to_string(),
            time: value.arrival,
        },
    }
}

pub type GetReservationResponse = Reservation;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reservation {
    Confirmed(ConfirmedReservation),
    Cancelled(CancelledReservation),
}

pub fn reservation_to_transfer_object(value: aggregate::Reservation) -> Reservation {
    match value {
        aggregate::Reservation::Confirmed(confirmed) => {
            Reservation::Confirmed(confirmed_reservation_to_transfer_object(confirmed))
        }

        aggregate::Reservation::Cancelled(cancelled) => {
            Reservation::Cancelled(cancelled_reservation_to_transfer_object(cancelled))
        }
    }
}

#[derive(Serialize)]
pub struct Revision {
    passengers: Passengers,
    itinerary: Itinerary,
}

pub fn revision_to_transfer_object(value: aggregate::Revision) -> Revision {
    Revision {
        passengers: passengers_to_transfer_object(value.1),
        itinerary: itinerary_to_transfer_object(value.2),
    }
}

#[derive(Serialize)]
pub struct ConfirmedReservation {
    pub id: String,
    pub journey: String,
    pub contact: Contact,
    pub passengers: Passengers,
    pub itinerary: Itinerary,
    pub revisions: Vec<Revision>
}

pub fn confirmed_reservation_to_transfer_object(value: aggregate::ConfirmedReservation) -> ConfirmedReservation {
    ConfirmedReservation {
        id: value.id.to_string(),
        journey: value.journey.to_string(),
        contact: contact_to_transfer_object(value.contact),
        passengers: passengers_to_transfer_object(value.passengers),
        itinerary: itinerary_to_transfer_object(value.itinerary),
        revisions: value.revisions.into_iter()
            .map(revision_to_transfer_object)
            .collect()
    }
}

#[derive(Serialize)]
pub struct CancelledReservation {
    pub id: String,
    pub journey: String,
    pub contact: Contact,
    pub time: DateTime<Utc>,
    pub revisions: Vec<Revision>
}

pub fn cancelled_reservation_to_transfer_object(value: aggregate::CancelledReservation) -> CancelledReservation {
    CancelledReservation {
        id: value.id.to_string(),
        journey: value.journey.to_string(),
        contact: contact_to_transfer_object(value.contact),
        time: value.time,
        revisions: value.revisions.into_iter()
            .map(revision_to_transfer_object)
            .collect()
    }
}

#[derive(Deserialize)]
pub struct DeleteReservationRequest {

}

#[derive(Serialize)]
pub struct DeleteReservationResponse {
    pub id: ReservationId
}

pub type PutReservationPassengersRequest = Passengers;

#[derive(Serialize)]
pub struct PutReservationPassengersResponse {
    pub id: ReservationId
}


pub type PutReservationItineraryRequest = Vec<ItineraryStage>;

#[derive(Serialize)]
pub struct PutReservationItineraryResponse {
    pub id: ReservationId
}
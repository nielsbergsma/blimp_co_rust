use chrono::{NaiveDate};
use prelude::data::Money;
use crate::aggregate::{Itinerary, Reservation};

fn price_per_day_per_passenger() -> Money {
    Money::usd(12000)
}

/// pricing a reservation is based on the number of days of travel
///   the formula is: amount = price per day * days of travel * passengers
///
/// cancellation fee always apply, and depend on the number of days before travel,
/// raging from 0% (full amount is returned) - 100% (nothing is returned)
pub fn price(reservation: &Reservation) -> Money {
    match reservation {
        Reservation::Confirmed(confirmed) => {
            price_itinerary(&confirmed.itinerary, confirmed.passengers.count(), None)
        }

        Reservation::Cancelled(cancelled) => {
            if let Some((_, passengers, itinerary)) = cancelled.revisions.first() {
                price_itinerary(itinerary, passengers.count(), Some(cancelled.time.date_naive()))
            }
            else {
                Money::usd(0)
            }
        }
    }
}

fn price_itinerary(itinerary: &Itinerary, passengers: u8, cancelled: Option<NaiveDate>) -> Money {
    let number_of_days = 1 + itinerary.duration().num_days();

    let price = price_per_day_per_passenger()
        .mul(number_of_days)
        .mul(passengers as i64);

    if let Some(date) = cancelled {
        let number_of_days_before_departure = (itinerary.departure_date() - date).num_days();
        let return_percentage = match number_of_days_before_departure {
            15.. => 100,
            10..=14 => 75,
            5..=9 => 50,
            _ => 0
        };

        price.percentage(100 - return_percentage)
    }
    else {
        price
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{LinkedList};
    use chrono::NaiveDate;
    use prelude::collection::SortedSet;
    use prelude::data::Money;
    use crate::aggregate::{Flight, FlightRoute, Itinerary, Journey, Segment};
    use crate::services::reservation_pricing_strategy::price_itinerary;

    #[test]
    fn price_reservation_is_based_on_length_and_passengers() {
        // 7 days itinerary, 2 persons
        let result = price_itinerary(&itinerary(), 2, None);
        assert_eq!(result, Money::usd(1680_00));

        // 7 days itinerary, 4 persons
        let result = price_itinerary(&itinerary(), 4, None);
        assert_eq!(result, Money::usd(3360_00));
    }

    #[test]
    fn cancellation_refund_on_date() {
        // cancelled >= 15 days, 100% refunded
        let cancellation_date: NaiveDate = "2024-03-08".parse().unwrap();
        let result = price_itinerary(&itinerary(), 2, Some(cancellation_date));
        assert_eq!(result, Money::usd(0_00));

        // cancelled >= 10 days, 75% refunded
        let cancellation_date: NaiveDate = "2024-04-24".parse().unwrap();
        let result = price_itinerary(&itinerary(), 2, Some(cancellation_date));
        assert_eq!(result, Money::usd(420_00));

        // cancelled >= 5 days, 50% refunded
        let cancellation_date: NaiveDate = "2024-04-30".parse().unwrap();
        let result = price_itinerary(&itinerary(), 2, Some(cancellation_date));
        assert_eq!(result, Money::usd(840_00));

        // cancelled < 5 days
        let cancellation_date: NaiveDate = "2024-05-05".parse().unwrap();
        let result = price_itinerary(&itinerary(), 2, Some(cancellation_date));
        assert_eq!(result, Money::usd(1680_00));
    }

    fn itinerary() -> Itinerary {
        let stages = LinkedList::from([
            (flight_eham_enli_8may(), None),
            (flight_enli_eham_14may(), None),
        ]);

        let journey = journey();
        journey.parse_itinerary(stages).unwrap()
    }

    fn journey() -> Journey {
        let (journey, _) = Journey::build(
            "5EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            "Journey Around North Atlantic".parse().unwrap(),
            SortedSet::empty()
                .insert(Segment::build(route_eham_enli(), SortedSet::empty()).unwrap())
                .insert(Segment::build(route_enli_eham(), SortedSet::empty()).unwrap())
        ).unwrap();

        journey
    }

    fn route_eham_enli() -> FlightRoute {
        FlightRoute::build(
            "EHAM".parse().unwrap(),
            "ENLI".parse().unwrap(),
        ).unwrap()
    }

    fn route_enli_eham() -> FlightRoute {
        FlightRoute::build(
            "ENLI".parse().unwrap(),
            "EHAM".parse().unwrap(),
        ).unwrap()
    }

    fn flight_eham_enli_8may() -> Flight {
        Flight::build(
            "9EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_eham_enli(),
            "2024-05-08T09:00:00+05:00".parse().unwrap(),
            "2024-05-08T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }

    fn flight_enli_eham_14may() -> Flight {
        Flight::build(
            "8EPFciXgSxB70tAE8iERl6".parse().unwrap(),
            route_enli_eham(),
            "2024-05-14T09:00:00+05:00".parse().unwrap(),
            "2024-05-14T11:00:00+05:00".parse().unwrap(),
            10u8,
        ).unwrap()
    }
}
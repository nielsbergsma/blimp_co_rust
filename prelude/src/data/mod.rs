mod uid;
mod geohash;
mod money;

pub use uid::*;
pub use geohash::*;
pub use money::*;

pub mod chrono {
    use chrono::NaiveDate;

    pub fn years_between(oldest: NaiveDate, newest: NaiveDate) -> i64 {
        let number_of_days = newest.signed_duration_since(oldest).num_days() as f64;
        let number_of_years = number_of_days / 365.2425;

        number_of_years as i64
    }
}
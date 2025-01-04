use std::ops::{Div, Mul};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Currency {
    USD
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Money(Decimal, Currency);

impl Money {
    pub fn usd(cents: i64) -> Self {
        Self(Decimal::from(cents), Currency::USD)
    }

    pub fn currency(&self) -> Currency {
        self.1
    }

    pub fn mul(&self, times: i64) -> Self {
        let times = Decimal::from(times);
        Self(self.0.mul(times), self.1)
    }

    pub fn percentage(&self, percentage: u8) -> Self {
        let percentage = Decimal::from(percentage).div(Decimal::from(100));
        Self(self.0.mul(percentage), self.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::Money;

    #[test]
    fn equals_by_amount_and_currency() {
        let money1 = Money::usd(100);
        let money2 = Money::usd(100);
        assert_eq!(money1, money2);

        let money3 = Money::usd(101);
        assert_ne!(money1, money3);
    }

    #[test]
    fn is_serializable() {
        let original = Money::usd(100);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Money = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn can_multiply() {
        let money = Money::usd(100)
            .mul(2);
        assert_eq!(money, Money::usd(200));
    }

    #[test]
    fn can_get_percentage() {
        let money = Money::usd(100);
        assert_eq!(money.percentage(50), Money::usd(50));
        assert_eq!(money.percentage(25), Money::usd(25));
    }
}
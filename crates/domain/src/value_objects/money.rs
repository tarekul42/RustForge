use serde::{Deserialize, Serialize};
use std::fmt;

/// Monetary amount stored as i64 cents.
///
/// This avoids floating-point precision issues inherent in `f64`
/// and is more efficient than a `Decimal` type for this project's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    /// Create a new Money amount from cents.
    pub fn from_cents(cents: i64) -> Self {
        Self(cents)
    }

    /// Create a new Money amount from dollars (converted to cents).
    pub fn from_dollars(dollars: i64) -> Self {
        Self(dollars * 100)
    }

    /// Return the amount in cents.
    pub fn cents(&self) -> i64 {
        self.0
    }

    /// Return the amount in dollars (integer division, truncates cents).
    pub fn dollars(&self) -> i64 {
        self.0 / 100
    }

    /// Check if this amount is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Add two Money amounts.
    pub fn add(&self, other: &Money) -> Money {
        Money(self.0 + other.0)
    }

    /// Subtract two Money amounts. Returns None if result would be negative.
    pub fn sub(&self, other: &Money) -> Option<Money> {
        if self.0 < other.0 {
            None
        } else {
            Some(Money(self.0 - other.0))
        }
    }

    /// Multiply by a scalar.
    pub fn mul(&self, scalar: i64) -> Money {
        Money(self.0 * scalar)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dollars = self.0 / 100;
        let cents = (self.0 % 100).abs();
        write!(f, "{}.{:02}", dollars, cents)
    }
}

impl From<i64> for Money {
    fn from(cents: i64) -> Self {
        Self(cents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_cents() {
        let m = Money::from_cents(100);
        assert_eq!(m.cents(), 100);
        assert_eq!(m.dollars(), 1);
    }

    #[test]
    fn from_dollars() {
        let m = Money::from_dollars(10);
        assert_eq!(m.cents(), 1000);
    }

    #[test]
    fn display() {
        assert_eq!(Money::from_cents(1050).to_string(), "10.50");
        assert_eq!(Money::from_cents(0).to_string(), "0.00");
    }

    #[test]
    fn add() {
        let a = Money::from_cents(100);
        let b = Money::from_cents(200);
        assert_eq!(a.add(&b).cents(), 300);
    }

    #[test]
    fn sub_success() {
        let a = Money::from_cents(300);
        let b = Money::from_cents(100);
        assert_eq!(a.sub(&b).unwrap().cents(), 200);
    }

    #[test]
    fn sub_negative_fails() {
        let a = Money::from_cents(100);
        let b = Money::from_cents(300);
        assert!(a.sub(&b).is_none());
    }
}

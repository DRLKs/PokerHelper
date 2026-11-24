use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub};

/// An amount of chips in the table's smallest unit.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Chips(pub u64);

/// An amount expressed in big blinds, used for display and chart lookups.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BigBlinds(pub f64);

impl Chips {
    pub const ZERO: Chips = Chips(0);

    pub fn saturating_sub(self, other: Chips) -> Chips {
        Chips(self.0.saturating_sub(other.0))
    }

    /// Converts to big blinds. `big_blind` must not be zero; [`TableState`]
    /// guarantees that for its own blinds.
    ///
    /// [`TableState`]: crate::table::TableState
    pub fn in_big_blinds(self, big_blind: Chips) -> BigBlinds {
        BigBlinds(self.0 as f64 / big_blind.0 as f64)
    }

    /// Rounds `amount` big blinds to the nearest whole chip.
    pub fn from_big_blinds(amount: BigBlinds, big_blind: Chips) -> Chips {
        Chips((amount.0 * big_blind.0 as f64).round().max(0.0) as u64)
    }

    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl Add for Chips {
    type Output = Chips;

    fn add(self, other: Chips) -> Chips {
        Chips(self.0 + other.0)
    }
}

impl AddAssign for Chips {
    fn add_assign(&mut self, other: Chips) {
        self.0 += other.0;
    }
}

impl Sub for Chips {
    type Output = Chips;

    /// Panics on underflow, like `u64`. Use [`Chips::saturating_sub`] when the
    /// result can be negative.
    fn sub(self, other: Chips) -> Chips {
        Chips(self.0 - other.0)
    }
}

impl Sum for Chips {
    fn sum<I: Iterator<Item = Chips>>(amounts: I) -> Chips {
        amounts.fold(Chips::ZERO, Add::add)
    }
}

impl fmt::Display for Chips {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for BigBlinds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.1} BB", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_chips_and_big_blinds() {
        let big_blind = Chips(20);
        assert_eq!(Chips(50).in_big_blinds(big_blind), BigBlinds(2.5));
        assert_eq!(Chips::from_big_blinds(BigBlinds(2.5), big_blind), Chips(50));
        assert_eq!(
            Chips::from_big_blinds(BigBlinds(3.76), big_blind),
            Chips(75)
        );
        assert_eq!(BigBlinds(7.5).to_string(), "7.5 BB");
    }

    #[test]
    fn sums_and_subtracts() {
        let total: Chips = [Chips(1), Chips(2), Chips(3)].into_iter().sum();
        assert_eq!(total, Chips(6));
        assert_eq!(Chips(5).saturating_sub(Chips(9)), Chips::ZERO);
        assert_eq!(Chips(9) - Chips(5), Chips(4));
    }
}

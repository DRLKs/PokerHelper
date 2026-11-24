use crate::table::chips::Chips;
use serde::{Deserialize, Serialize};

/// Index of a physical seat at the table, starting at 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SeatIndex(pub u8);

impl SeatIndex {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatStatus {
    Empty,
    SittingOut,
    Active,
    Folded,
    AllIn,
}

impl SeatStatus {
    /// Whether the seat received cards this hand, even if it folded since.
    pub fn is_dealt_in(self) -> bool {
        match self {
            SeatStatus::Active | SeatStatus::Folded | SeatStatus::AllIn => true,
            SeatStatus::Empty | SeatStatus::SittingOut => false,
        }
    }

    /// Whether the seat can still win the pot.
    pub fn is_in_hand(self) -> bool {
        match self {
            SeatStatus::Active | SeatStatus::AllIn => true,
            SeatStatus::Empty | SeatStatus::SittingOut | SeatStatus::Folded => false,
        }
    }

    /// Whether the seat still has decisions to make.
    pub fn can_act(self) -> bool {
        match self {
            SeatStatus::Active => true,
            SeatStatus::Empty | SeatStatus::SittingOut | SeatStatus::Folded | SeatStatus::AllIn => {
                false
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seat {
    /// Chips behind, not counting what is already committed this street.
    pub stack: Chips,
    pub committed_this_street: Chips,
    pub status: SeatStatus,
}

impl Seat {
    /// Chips behind plus chips committed this street.
    pub fn stack_at_street_start(&self) -> Chips {
        self.stack + self.committed_this_street
    }
}

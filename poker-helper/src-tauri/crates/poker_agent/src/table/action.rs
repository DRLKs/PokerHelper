use crate::table::chips::Chips;
use crate::table::seat::SeatIndex;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
}

impl Street {
    /// Street implied by the number of community cards, or `None` for a
    /// count that no street has (1, 2 or more than 5).
    pub fn from_board_len(board_len: usize) -> Option<Street> {
        match board_len {
            0 => Some(Street::Preflop),
            3 => Some(Street::Flop),
            4 => Some(Street::Turn),
            5 => Some(Street::River),
            _ => None,
        }
    }

    /// Number of community cards on this street.
    pub fn board_len(self) -> usize {
        match self {
            Street::Preflop => 0,
            Street::Flop => 3,
            Street::Turn => 4,
            Street::River => 5,
        }
    }
}

/// A voluntary action. Amounts are the player's total commitment on the
/// street after the action, so a raise from 20 to 60 is `Raise { to: 60 }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PlayerAction {
    Fold,
    Check,
    Call { to: Chips },
    Bet { to: Chips },
    Raise { to: Chips },
}

impl PlayerAction {
    pub fn is_aggressive(self) -> bool {
        match self {
            PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => true,
            PlayerAction::Fold | PlayerAction::Check | PlayerAction::Call { .. } => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RecordedAction {
    pub seat: SeatIndex,
    pub street: Street,
    pub action: PlayerAction,
}

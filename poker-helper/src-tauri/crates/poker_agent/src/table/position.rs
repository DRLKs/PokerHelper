use serde::Serialize;
use std::fmt;

/// Smallest number of dealt-in players that makes a hand.
pub const MIN_PLAYERS: usize = 2;
/// Largest table this engine supports (6-max).
pub const MAX_PLAYERS: usize = 6;

/// Seat position relative to the button at a 6-max table.
///
/// With fewer than six players the earliest positions disappear first: five
/// players have no `UnderTheGun`, four start at the `Cutoff`, and heads-up the
/// `Button` posts the small blind and plays against the `BigBlind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Position {
    UnderTheGun,
    Hijack,
    Cutoff,
    Button,
    SmallBlind,
    BigBlind,
}

impl Position {
    /// Positions of `player_count` dealt-in players in clockwise seat order,
    /// starting with the button. `None` outside `MIN_PLAYERS..=MAX_PLAYERS`.
    pub fn clockwise_from_button(player_count: usize) -> Option<&'static [Position]> {
        use Position::*;
        match player_count {
            2 => Some(&[Button, BigBlind]),
            3 => Some(&[Button, SmallBlind, BigBlind]),
            4 => Some(&[Button, SmallBlind, BigBlind, Cutoff]),
            5 => Some(&[Button, SmallBlind, BigBlind, Hijack, Cutoff]),
            6 => Some(&[Button, SmallBlind, BigBlind, UnderTheGun, Hijack, Cutoff]),
            _ => None,
        }
    }

    /// Order of action before the flop: lower acts first.
    pub fn preflop_order(self) -> u8 {
        match self {
            Position::UnderTheGun => 0,
            Position::Hijack => 1,
            Position::Cutoff => 2,
            Position::Button => 3,
            Position::SmallBlind => 4,
            Position::BigBlind => 5,
        }
    }

    /// Order of action from the flop on: lower acts first.
    pub fn postflop_order(self) -> u8 {
        match self {
            Position::SmallBlind => 0,
            Position::BigBlind => 1,
            Position::UnderTheGun => 2,
            Position::Hijack => 3,
            Position::Cutoff => 4,
            Position::Button => 5,
        }
    }

    /// Whether `self` acts after `other` on every street after the flop.
    pub fn is_in_position_against(self, other: Position) -> bool {
        self.postflop_order() > other.postflop_order()
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Position::UnderTheGun => "UTG",
            Position::Hijack => "HJ",
            Position::Cutoff => "CO",
            Position::Button => "BTN",
            Position::SmallBlind => "SB",
            Position::BigBlind => "BB",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            Position::UnderTheGun => "under the gun",
            Position::Hijack => "hijack",
            Position::Cutoff => "cutoff",
            Position::Button => "button",
            Position::SmallBlind => "small blind",
            Position::BigBlind => "big blind",
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.short_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_size_has_button_and_big_blind() {
        for player_count in MIN_PLAYERS..=MAX_PLAYERS {
            let positions = Position::clockwise_from_button(player_count).unwrap();
            assert_eq!(positions.len(), player_count);
            assert_eq!(positions[0], Position::Button);
            assert!(positions.contains(&Position::BigBlind));
        }
        assert!(Position::clockwise_from_button(1).is_none());
        assert!(Position::clockwise_from_button(7).is_none());
    }

    #[test]
    fn button_is_in_position_against_the_blinds() {
        assert!(Position::Button.is_in_position_against(Position::BigBlind));
        assert!(Position::BigBlind.is_in_position_against(Position::SmallBlind));
        assert!(!Position::UnderTheGun.is_in_position_against(Position::Cutoff));
    }
}

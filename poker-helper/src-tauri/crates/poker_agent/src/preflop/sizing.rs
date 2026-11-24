use crate::preflop::charts::Seating;
use crate::table::{BigBlinds, Chips, Position};

const OPEN_SIZE: f64 = 2.5;
const SMALL_BLIND_OPEN_SIZE: f64 = 3.0;
const ISOLATION_BASE_SIZE: f64 = 3.0;
const EXTRA_PER_LIMPER: f64 = 1.0;
const EXTRA_OUT_OF_POSITION: f64 = 1.0;
const THREE_BET_IN_POSITION: f64 = 3.0;
const THREE_BET_OUT_OF_POSITION: f64 = 3.75;
const FOUR_BET_MULTIPLIER: f64 = 2.3;

/// Opening raise size when everyone before folded: 2.5 big blinds, or 3 from
/// the small blind outside heads-up play.
pub fn open_size(position: Position, heads_up: bool) -> BigBlinds {
    let base = match (position, heads_up) {
        (Position::SmallBlind, false) => SMALL_BLIND_OPEN_SIZE,
        _ => OPEN_SIZE,
    };
    BigBlinds(base)
}

/// Raise size over limpers: 3 big blinds, plus 1 per limper, plus 1 when the
/// raiser will be out of position.
pub fn isolation_size(limpers: usize, seating: Seating) -> BigBlinds {
    let out_of_position_extra = match seating {
        Seating::InPosition => 0.0,
        Seating::OutOfPosition => EXTRA_OUT_OF_POSITION,
    };
    BigBlinds(ISOLATION_BASE_SIZE + EXTRA_PER_LIMPER * limpers as f64 + out_of_position_extra)
}

/// 3-bet size: 3 times the open in position, 3.75 times out of position, plus
/// one more open for each player who called it.
pub fn three_bet_to(open_to: Chips, seating: Seating, callers: usize) -> Chips {
    let multiplier = match seating {
        Seating::InPosition => THREE_BET_IN_POSITION,
        Seating::OutOfPosition => THREE_BET_OUT_OF_POSITION,
    };
    Chips((open_to.as_f64() * (multiplier + callers as f64)).round() as u64)
}

/// 4-bet size: 2.3 times the 3-bet.
pub fn four_bet_to(three_bet_to: Chips) -> Chips {
    Chips((three_bet_to.as_f64() * FOUR_BET_MULTIPLIER).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_follow_the_plan() {
        assert_eq!(open_size(Position::Cutoff, false), BigBlinds(2.5));
        assert_eq!(open_size(Position::SmallBlind, false), BigBlinds(3.0));
        assert_eq!(open_size(Position::Button, true), BigBlinds(2.5));
        assert_eq!(isolation_size(2, Seating::InPosition), BigBlinds(5.0));
        assert_eq!(isolation_size(1, Seating::OutOfPosition), BigBlinds(5.0));
        assert_eq!(three_bet_to(Chips(50), Seating::InPosition, 0), Chips(150));
        assert_eq!(
            three_bet_to(Chips(50), Seating::OutOfPosition, 0),
            Chips(188)
        );
        assert_eq!(three_bet_to(Chips(50), Seating::InPosition, 1), Chips(200));
        assert_eq!(four_bet_to(Chips(150)), Chips(345));
    }
}

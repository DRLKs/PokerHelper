use crate::table::{Chips, PlayerAction, SeatIndex, Street, TableState};
use serde::Serialize;

/// Raise sizes allowed by No-Limit rules, as the total commitment on the
/// street after raising. `max_to` is all-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RaiseBounds {
    pub min_to: Chips,
    pub max_to: Chips,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LegalActions {
    /// Chips needed to call, already capped at the player's stack.
    pub to_call: Chips,
    /// `None` when the player cannot bet or raise.
    pub raise: Option<RaiseBounds>,
}

impl LegalActions {
    pub fn can_check(&self) -> bool {
        self.to_call == Chips::ZERO
    }
}

/// What `seat` may do now, or `None` if it cannot act (folded, all-in,
/// empty or out of range).
///
/// A raise must be at least as large as the last full bet or raise on the
/// street (the big blind when there is none). An all-in for less than that
/// does not reopen the betting for players who already acted.
pub fn legal_actions(table: &TableState, seat: SeatIndex) -> Option<LegalActions> {
    let player = table.seat(seat)?;
    if !player.status.can_act() {
        return None;
    }
    let to_call = table.amount_to_call(seat);
    let current_bet = table.current_bet();
    let all_in_to = player.committed_this_street + player.stack;
    let someone_can_respond = table.opponents_in_hand(seat).into_iter().any(|opponent| {
        table
            .seat(opponent)
            .is_some_and(|other| other.status.can_act())
    });

    let betting = street_betting(table, seat);
    let raise =
        (all_in_to > current_bet && someone_can_respond && betting.is_open_to_seat).then(|| {
            RaiseBounds {
                min_to: (current_bet + betting.minimum_raise_increment).min(all_in_to),
                max_to: all_in_to,
            }
        });
    Some(LegalActions { to_call, raise })
}

struct StreetBetting {
    minimum_raise_increment: Chips,
    is_open_to_seat: bool,
}

fn street_betting(table: &TableState, seat: SeatIndex) -> StreetBetting {
    let street = table.street();
    let opening_bet = match street {
        Street::Preflop => table.big_blind(),
        Street::Flop | Street::Turn | Street::River => Chips::ZERO,
    };
    let mut bet_level = opening_bet;
    let mut full_increment = table.big_blind();
    let mut last_full_raise_turn = None;
    let mut last_turn_of_seat = None;

    for (turn, recorded) in table.actions_on(street).enumerate() {
        if recorded.seat == seat {
            last_turn_of_seat = Some(turn);
        }
        let (PlayerAction::Bet { to } | PlayerAction::Raise { to }) = recorded.action else {
            continue;
        };
        let increment = to.saturating_sub(bet_level);
        if increment >= full_increment {
            full_increment = increment;
            last_full_raise_turn = Some(turn);
        }
        bet_level = bet_level.max(to);
    }

    let history_misses_the_current_bet = table.current_bet() > bet_level;
    if history_misses_the_current_bet {
        full_increment = full_increment.max(table.current_bet() - bet_level);
    }
    let is_open_to_seat = match last_turn_of_seat {
        None => true,
        Some(seat_turn) => last_full_raise_turn.is_some_and(|raise_turn| raise_turn > seat_turn),
    };
    StreetBetting {
        minimum_raise_increment: full_increment,
        is_open_to_seat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::{cards, seat, six_max_preflop};
    use crate::table::{RecordedAction, SeatStatus, TableSetup};

    fn raise(seat: u8, to: u64) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street: Street::Preflop,
            action: PlayerAction::Raise { to: Chips(to) },
        }
    }

    fn legal(setup: TableSetup, seat: u8) -> Option<LegalActions> {
        legal_actions(&TableState::new(setup).unwrap(), SeatIndex(seat))
    }

    fn bounds(min_to: u64, max_to: u64) -> Option<RaiseBounds> {
        Some(RaiseBounds {
            min_to: Chips(min_to),
            max_to: Chips(max_to),
        })
    }

    #[test]
    fn first_player_preflop_calls_the_big_blind_or_raises_to_two() {
        let actions = legal(six_max_preflop(3, "Ah Kd"), 3).unwrap();
        assert_eq!(actions.to_call, Chips(2));
        assert_eq!(actions.raise, bounds(4, 200));
        assert!(!actions.can_check());
    }

    #[test]
    fn a_re_raise_must_be_at_least_the_previous_raise() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[3] = seat(194, 6, SeatStatus::Active);
        setup.history = vec![raise(3, 6)];
        assert_eq!(legal(setup.clone(), 5).unwrap().raise, bounds(10, 200));

        setup.history.clear();
        assert_eq!(legal(setup, 5).unwrap().raise, bounds(10, 200));
    }

    #[test]
    fn postflop_with_no_bet_the_minimum_bet_is_the_big_blind() {
        let mut setup = six_max_preflop(2, "Ah Kd");
        setup.board = cards("2c 7d Ts");
        for index in 0..6 {
            setup.seats[index] = seat(190, 0, SeatStatus::Active);
        }
        let actions = legal(setup, 2).unwrap();
        assert!(actions.can_check());
        assert_eq!(actions.raise, bounds(2, 190));
    }

    #[test]
    fn a_short_stack_can_only_raise_all_in() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[3] = seat(194, 6, SeatStatus::Active);
        setup.seats[5] = seat(8, 0, SeatStatus::Active);
        assert_eq!(legal(setup.clone(), 5).unwrap().raise, bounds(8, 8));

        setup.seats[5] = seat(5, 0, SeatStatus::Active);
        let actions = legal(setup, 5).unwrap();
        assert_eq!(actions.to_call, Chips(5));
        assert_eq!(actions.raise, None);
    }

    #[test]
    fn a_short_all_in_does_not_reopen_the_betting() {
        let mut setup = six_max_preflop(3, "Ah Kd");
        setup.seats[3] = seat(194, 6, SeatStatus::Active);
        setup.seats[5] = seat(0, 8, SeatStatus::AllIn);
        setup.history = vec![raise(3, 6), raise(5, 8)];
        let actions = legal(setup, 3).unwrap();
        assert_eq!(actions.to_call, Chips(2));
        assert_eq!(actions.raise, None);
    }

    #[test]
    fn nobody_left_to_raise_against() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        for index in 0..5 {
            setup.seats[index].status = SeatStatus::Folded;
        }
        setup.seats[4] = seat(0, 100, SeatStatus::AllIn);
        let actions = legal(setup, 5).unwrap();
        assert_eq!(actions.to_call, Chips(100));
        assert_eq!(actions.raise, None);
    }

    #[test]
    fn folded_players_have_no_actions() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[5].status = SeatStatus::Folded;
        assert_eq!(legal(setup, 5), None);
    }
}

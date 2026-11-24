use crate::table::{PlayerAction, Position, SeatIndex, Street, TableState};

/// The preflop situation the hero faces, read from the actions so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreflopSpot {
    Unopened,
    Limped {
        limpers: usize,
    },
    FacingOpen {
        opener: Position,
        callers: usize,
        hero_limped: bool,
    },
    FacingThreeBet {
        opener: Position,
        three_bettor: Position,
        hero_is_opener: bool,
    },
    FacingFourBet {
        hero_is_three_bettor: bool,
    },
    FacingFiveBetOrMore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Raiser {
    pub seat: SeatIndex,
    pub position: Position,
}

/// The spot and the last player who raised, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreflopSituation {
    pub spot: PreflopSpot,
    pub last_raiser: Option<Raiser>,
}

pub fn classify(table: &TableState) -> PreflopSituation {
    let hero = table.hero();
    let mut raisers: Vec<Raiser> = Vec::new();
    let mut limpers = 0;
    let mut callers_of_last_raise = 0;
    let mut hero_limped = false;

    for recorded in table.actions_on(Street::Preflop) {
        let Some(position) = table.position_of(recorded.seat) else {
            continue;
        };
        match recorded.action {
            PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => {
                raisers.push(Raiser {
                    seat: recorded.seat,
                    position,
                });
                callers_of_last_raise = 0;
            }
            PlayerAction::Call { .. } if raisers.is_empty() => {
                limpers += 1;
                hero_limped |= recorded.seat == hero;
            }
            PlayerAction::Call { .. } => callers_of_last_raise += 1,
            PlayerAction::Fold | PlayerAction::Check => {}
        }
    }

    if raisers.is_empty() {
        if let Some(missed_raiser) = raiser_missing_from_history(table) {
            raisers.push(missed_raiser);
        }
    }

    let spot = match raisers.as_slice() {
        [] if limpers == 0 => PreflopSpot::Unopened,
        [] => PreflopSpot::Limped { limpers },
        [opener] => PreflopSpot::FacingOpen {
            opener: opener.position,
            callers: callers_of_last_raise,
            hero_limped,
        },
        [opener, three_bettor] => PreflopSpot::FacingThreeBet {
            opener: opener.position,
            three_bettor: three_bettor.position,
            hero_is_opener: opener.seat == hero,
        },
        [_, three_bettor, _] => PreflopSpot::FacingFourBet {
            hero_is_three_bettor: three_bettor.seat == hero,
        },
        _ => PreflopSpot::FacingFiveBetOrMore,
    };
    PreflopSituation {
        spot,
        last_raiser: raisers.last().copied(),
    }
}

/// When frames were missed the history can lack a raise that the chips
/// show. The seat with the largest commitment above the big blind raised.
fn raiser_missing_from_history(table: &TableState) -> Option<Raiser> {
    if table.current_bet() <= table.big_blind() {
        return None;
    }
    table
        .seats_in_action_order(Street::Preflop)
        .into_iter()
        .find(|(seat, _)| {
            table
                .seat(*seat)
                .is_some_and(|player| player.committed_this_street == table.current_bet())
        })
        .map(|(seat, position)| Raiser { seat, position })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::{seat, six_max_preflop};
    use crate::table::{Chips, RecordedAction, SeatStatus, TableSetup};

    fn act(seat: u8, action: PlayerAction) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street: Street::Preflop,
            action,
        }
    }

    fn raise(seat: u8, to: u64) -> RecordedAction {
        act(seat, PlayerAction::Raise { to: Chips(to) })
    }

    fn call(seat: u8, to: u64) -> RecordedAction {
        act(seat, PlayerAction::Call { to: Chips(to) })
    }

    fn spot(mut setup: TableSetup, history: Vec<RecordedAction>) -> PreflopSpot {
        setup.history = history;
        classify(&TableState::new(setup).unwrap()).spot
    }

    #[test]
    fn recognises_each_spot() {
        let cutoff = || six_max_preflop(5, "Ah Kd");
        assert_eq!(spot(cutoff(), vec![]), PreflopSpot::Unopened);
        assert_eq!(
            spot(cutoff(), vec![call(3, 2), call(4, 2)]),
            PreflopSpot::Limped { limpers: 2 }
        );
        assert_eq!(
            spot(cutoff(), vec![raise(3, 6), call(4, 6)]),
            PreflopSpot::FacingOpen {
                opener: Position::UnderTheGun,
                callers: 1,
                hero_limped: false,
            }
        );
        assert_eq!(
            spot(cutoff(), vec![raise(5, 6), raise(0, 18)]),
            PreflopSpot::FacingThreeBet {
                opener: Position::Cutoff,
                three_bettor: Position::Button,
                hero_is_opener: true,
            }
        );
        assert_eq!(
            spot(cutoff(), vec![raise(3, 6), raise(5, 18), raise(3, 42)]),
            PreflopSpot::FacingFourBet {
                hero_is_three_bettor: true
            }
        );
        assert_eq!(
            spot(
                cutoff(),
                vec![raise(3, 6), raise(4, 18), raise(5, 42), raise(4, 200)]
            ),
            PreflopSpot::FacingFiveBetOrMore
        );
    }

    #[test]
    fn a_limper_facing_a_raise_is_marked() {
        let history = vec![call(5, 2), raise(0, 10)];
        assert_eq!(
            spot(six_max_preflop(5, "Ah Kd"), history),
            PreflopSpot::FacingOpen {
                opener: Position::Button,
                callers: 0,
                hero_limped: true,
            }
        );
    }

    #[test]
    fn falls_back_to_the_chips_when_the_history_misses_a_raise() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[4] = seat(194, 6, SeatStatus::Active);
        let situation = classify(&TableState::new(setup).unwrap());
        assert_eq!(
            situation.spot,
            PreflopSpot::FacingOpen {
                opener: Position::Hijack,
                callers: 0,
                hero_limped: false,
            }
        );
        assert_eq!(situation.last_raiser.unwrap().seat, SeatIndex(4));
    }
}

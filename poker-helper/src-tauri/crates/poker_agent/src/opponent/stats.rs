use crate::math::Probability;
use crate::table::{PlayerAction, RecordedAction, SeatIndex, Street, TableState};
use serde::{Deserialize, Serialize};

/// How many chances at the population rate a prior is worth before any
/// observation: about 30 hands, as in `docs/STRATEGY_PLAN.md`.
pub const PRIOR_WEIGHT: f64 = 30.0;

/// How often something happened out of the chances it had to happen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frequency {
    pub hits: u32,
    pub chances: u32,
}

impl Frequency {
    pub fn record(&mut self, hit: bool) {
        self.chances += 1;
        if hit {
            self.hits += 1;
        }
    }

    /// The observed rate pulled toward `prior` as if [`PRIOR_WEIGHT`] chances
    /// at the prior rate had come first (Bayesian shrinkage). With no
    /// chances it is the prior; with many it approaches the observed rate.
    pub fn shrunk_toward(self, prior: Probability) -> Probability {
        let hits = self.hits as f64 + prior.fraction() * PRIOR_WEIGHT;
        let chances = self.chances as f64 + PRIOR_WEIGHT;
        Probability::saturating(hits / chances)
    }
}

/// Counts of one player's habits over the hands recorded so far.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Hands dealt in where the player called or raised before the flop.
    pub voluntarily_played: Frequency,
    /// Hands dealt in where the player raised before the flop.
    pub raised_preflop: Frequency,
    /// Re-raises out of the chances to re-raise a single preflop raise.
    pub three_bet: Frequency,
    /// Bets and raises out of bets, raises and calls after the flop.
    pub postflop_aggression: Frequency,
    /// Folds out of the flop continuation bets the player faced.
    pub folded_to_continuation_bet: Frequency,
    /// Hands still in at the last frame of the river, out of flops seen.
    pub went_to_showdown: Frequency,
}

impl PlayerStats {
    pub fn hands(&self) -> u32 {
        self.voluntarily_played.chances
    }

    /// Adds what `seat` did in the hand shown by `table`, which should be the
    /// last frame of a finished hand. Does nothing for a seat that was not
    /// dealt in.
    ///
    /// Showdowns are inferred: a player still in the hand when the last frame
    /// shows the river counts as going to showdown.
    pub fn record_hand(&mut self, table: &TableState, seat: SeatIndex) {
        if table.position_of(seat).is_none() {
            return;
        }
        let preflop: Vec<&RecordedAction> = table.actions_on(Street::Preflop).collect();
        let acted = |predicate: fn(PlayerAction) -> bool| {
            preflop
                .iter()
                .any(|recorded| recorded.seat == seat && predicate(recorded.action))
        };
        self.voluntarily_played.record(acted(|action| match action {
            PlayerAction::Call { .. } | PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => {
                true
            }
            PlayerAction::Fold | PlayerAction::Check => false,
        }));
        self.raised_preflop
            .record(acted(PlayerAction::is_aggressive));
        if let Some(three_bet) = three_bet_chance(&preflop, seat) {
            self.three_bet.record(three_bet);
        }
        self.record_postflop(table, seat, &preflop);
    }

    fn record_postflop(
        &mut self,
        table: &TableState,
        seat: SeatIndex,
        preflop: &[&RecordedAction],
    ) {
        let folded_preflop = preflop
            .iter()
            .any(|recorded| recorded.seat == seat && recorded.action == PlayerAction::Fold);
        if table.street() == Street::Preflop || folded_preflop {
            return;
        }
        let postflop = table
            .history()
            .iter()
            .filter(|recorded| recorded.street != Street::Preflop && recorded.seat == seat);
        for recorded in postflop {
            match recorded.action {
                PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => {
                    self.postflop_aggression.record(true)
                }
                PlayerAction::Call { .. } => self.postflop_aggression.record(false),
                PlayerAction::Fold | PlayerAction::Check => {}
            }
        }
        if let Some(folded) = response_to_continuation_bet(table, preflop, seat) {
            self.folded_to_continuation_bet.record(folded);
        }
        let still_in = table
            .seat(seat)
            .is_some_and(|player| player.status.is_in_hand());
        self.went_to_showdown
            .record(table.street() == Street::River && still_in);
    }
}

/// Whether `seat` re-raised at its first chance to act facing exactly one
/// preflop raise, or `None` when it never had that chance.
fn three_bet_chance(preflop: &[&RecordedAction], seat: SeatIndex) -> Option<bool> {
    let mut raises = 0;
    for recorded in preflop {
        if recorded.seat == seat && raises == 1 {
            return Some(recorded.action.is_aggressive());
        }
        if recorded.action.is_aggressive() {
            raises += 1;
        }
    }
    None
}

/// Whether `seat` folded to the flop bet of the last preflop raiser, or
/// `None` when there was no such bet or `seat` did not act after it.
fn response_to_continuation_bet(
    table: &TableState,
    preflop: &[&RecordedAction],
    seat: SeatIndex,
) -> Option<bool> {
    let preflop_raiser = preflop
        .iter()
        .rev()
        .find(|recorded| recorded.action.is_aggressive())
        .map(|recorded| recorded.seat)?;
    if preflop_raiser == seat {
        return None;
    }
    let mut flop = table.actions_on(Street::Flop);
    let first_bet = flop.find(|recorded| recorded.action.is_aggressive())?;
    let is_continuation_bet =
        first_bet.seat == preflop_raiser && matches!(first_bet.action, PlayerAction::Bet { .. });
    if !is_continuation_bet {
        return None;
    }
    flop.find(|recorded| recorded.seat == seat)
        .map(|recorded| recorded.action == PlayerAction::Fold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::{cards, seat, six_max_preflop};
    use crate::table::{Chips, SeatStatus, TableSetup};

    const BUTTON: u8 = 0;
    const SMALL_BLIND: u8 = 1;
    const BIG_BLIND: u8 = 2;
    const UNDER_THE_GUN: u8 = 3;
    const CUTOFF: u8 = 5;

    fn act(seat: u8, street: Street, action: PlayerAction) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street,
            action,
        }
    }

    fn three_bet_pot_on_the_turn() -> TableState {
        let mut setup: TableSetup = six_max_preflop(BUTTON, "Ah Kd");
        setup.history = vec![
            act(UNDER_THE_GUN, Street::Preflop, PlayerAction::Fold),
            act(4, Street::Preflop, PlayerAction::Fold),
            act(
                CUTOFF,
                Street::Preflop,
                PlayerAction::Raise { to: Chips(5) },
            ),
            act(
                BUTTON,
                Street::Preflop,
                PlayerAction::Raise { to: Chips(15) },
            ),
            act(SMALL_BLIND, Street::Preflop, PlayerAction::Fold),
            act(
                BIG_BLIND,
                Street::Preflop,
                PlayerAction::Call { to: Chips(15) },
            ),
            act(
                CUTOFF,
                Street::Preflop,
                PlayerAction::Call { to: Chips(15) },
            ),
            act(BIG_BLIND, Street::Flop, PlayerAction::Check),
            act(CUTOFF, Street::Flop, PlayerAction::Check),
            act(BUTTON, Street::Flop, PlayerAction::Bet { to: Chips(20) }),
            act(BIG_BLIND, Street::Flop, PlayerAction::Fold),
            act(CUTOFF, Street::Flop, PlayerAction::Call { to: Chips(20) }),
        ];
        for index in [SMALL_BLIND, BIG_BLIND, UNDER_THE_GUN, 4] {
            setup.seats[index as usize].status = SeatStatus::Folded;
        }
        setup.seats[BUTTON as usize] = seat(165, 0, SeatStatus::Active);
        setup.seats[CUTOFF as usize] = seat(165, 0, SeatStatus::Active);
        setup.board = cards("Kh 7h 2c 9s");
        setup.pot_from_previous_streets = Chips(86);
        TableState::new(setup).unwrap()
    }

    fn stats_of(table: &TableState, seat: u8) -> PlayerStats {
        let mut stats = PlayerStats::default();
        stats.record_hand(table, SeatIndex(seat));
        stats
    }

    fn frequency(hits: u32, chances: u32) -> Frequency {
        Frequency { hits, chances }
    }

    #[test]
    fn counts_preflop_habits() {
        let table = three_bet_pot_on_the_turn();
        let button = stats_of(&table, BUTTON);
        assert_eq!(button.voluntarily_played, frequency(1, 1));
        assert_eq!(button.raised_preflop, frequency(1, 1));
        assert_eq!(button.three_bet, frequency(1, 1));

        let big_blind = stats_of(&table, BIG_BLIND);
        assert_eq!(big_blind.voluntarily_played, frequency(1, 1));
        assert_eq!(big_blind.raised_preflop, frequency(0, 1));
        assert_eq!(big_blind.three_bet, Frequency::default());

        let cutoff = stats_of(&table, CUTOFF);
        assert_eq!(cutoff.raised_preflop, frequency(1, 1));
        assert_eq!(cutoff.three_bet, Frequency::default());

        let under_the_gun = stats_of(&table, UNDER_THE_GUN);
        assert_eq!(under_the_gun.voluntarily_played, frequency(0, 1));
        assert_eq!(under_the_gun.went_to_showdown, Frequency::default());
    }

    #[test]
    fn counts_continuation_bets_and_aggression() {
        let table = three_bet_pot_on_the_turn();
        assert_eq!(
            stats_of(&table, BIG_BLIND).folded_to_continuation_bet,
            frequency(1, 1)
        );
        let cutoff = stats_of(&table, CUTOFF);
        assert_eq!(cutoff.folded_to_continuation_bet, frequency(0, 1));
        assert_eq!(cutoff.postflop_aggression, frequency(0, 1));
        assert_eq!(cutoff.went_to_showdown, frequency(0, 1));
        let button = stats_of(&table, BUTTON);
        assert_eq!(button.postflop_aggression, frequency(1, 1));
        assert_eq!(button.folded_to_continuation_bet, Frequency::default());
    }

    #[test]
    fn shrinkage_starts_at_the_prior_and_converges_to_the_observed_rate() {
        let prior = Probability::saturating(0.4);
        assert_eq!(Frequency::default().shrunk_toward(prior), prior);
        let small_sample = frequency(1, 10).shrunk_toward(prior);
        assert!(small_sample.fraction() > 0.3);
        let large_sample = frequency(1_000, 10_000).shrunk_toward(prior);
        assert!((large_sample.fraction() - 0.1).abs() < 0.001);
    }
}

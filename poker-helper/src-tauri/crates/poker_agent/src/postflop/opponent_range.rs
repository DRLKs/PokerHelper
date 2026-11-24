use crate::evaluator::fast_eval::CardMask;
use crate::postflop::narrowing::{narrow, Narrowing};
use crate::postflop::validated_cards_mask;
use crate::preflop::charts::{ChartError, Charts, OpenerGroup, Seating, SpotChart};
use crate::preflop::decision::chart_position;
use crate::preflop::spot::Raiser;
use crate::range::Range;
use crate::table::{PlayerAction, Position, SeatIndex, Street, TableState};

const POSTFLOP_STREETS: [Street; 3] = [Street::Flop, Street::Turn, Street::River];

/// The likely hands of each seat in `seats`, in the same order: the preflop
/// chart range that matches the seat's last preflop action, cut down by each
/// call, bet and raise it made after the flop. Combos that use a card in
/// `dead_cards` are left in; equity functions remove them.
///
/// A seat with no recorded preflop action (missed frames) is given its
/// opening range, or every hand in the big blind. If a cut would leave no
/// live combo, the range from before that cut is kept.
pub fn estimate_ranges(
    table: &TableState,
    charts: &Charts,
    seats: &[SeatIndex],
    dead_cards: CardMask,
) -> Result<Vec<Range>, ChartError> {
    let mut preflop = preflop_ranges(table, charts)?;
    seats
        .iter()
        .map(|&seat| {
            let recorded = preflop.get_mut(seat.index()).and_then(Option::take);
            let range = match recorded {
                Some(range) => range,
                None => default_range(table, charts, seat)?,
            };
            let range = keep_if_live(range, Range::full(), dead_cards);
            Ok(narrow_after_the_flop(range, table, seat, dead_cards))
        })
        .collect()
}

fn default_range(
    table: &TableState,
    charts: &Charts,
    seat: SeatIndex,
) -> Result<Range, ChartError> {
    match table.position_of(seat) {
        None | Some(Position::BigBlind) => Ok(Range::full()),
        Some(position) => charts
            .raise_first_in(position, table.is_heads_up())
            .cloned(),
    }
}

fn narrow_after_the_flop(
    mut range: Range,
    table: &TableState,
    seat: SeatIndex,
    dead_cards: CardMask,
) -> Range {
    let streets = POSTFLOP_STREETS
        .into_iter()
        .filter(|&street| street <= table.street());
    for street in streets {
        let board = validated_cards_mask(&table.board()[..street.board_len()]);
        let actions = table
            .actions_on(street)
            .filter(|recorded| recorded.seat == seat);
        for recorded in actions {
            let narrowing = match recorded.action {
                PlayerAction::Fold | PlayerAction::Check => continue,
                PlayerAction::Call { .. } => Narrowing::CALL,
                PlayerAction::Bet { .. } => Narrowing::BET,
                PlayerAction::Raise { .. } => Narrowing::RAISE,
            };
            let narrowed = narrow(&range, board, dead_cards, narrowing).range;
            range = keep_if_live(narrowed, range, dead_cards);
        }
    }
    range
}

fn keep_if_live(candidate: Range, fallback: Range, dead_cards: CardMask) -> Range {
    if candidate.without_cards(dead_cards).combo_count() > 0.0 {
        candidate
    } else {
        fallback
    }
}

/// The range each seat shows with its last preflop action, indexed by seat.
fn preflop_ranges(table: &TableState, charts: &Charts) -> Result<Vec<Option<Range>>, ChartError> {
    let heads_up = table.is_heads_up();
    let mut ranges = vec![None; table.seats().len()];
    let mut betting = PreflopBetting::default();
    for recorded in table.actions_on(Street::Preflop) {
        let Some(position) = table.position_of(recorded.seat) else {
            continue;
        };
        let actor = Raiser {
            seat: recorded.seat,
            position,
        };
        if let Some(range) = betting.range_shown_by(actor, recorded.action, charts, heads_up)? {
            ranges[actor.seat.index()] = Some(range);
        }
        betting.record(actor, recorded.action);
    }
    Ok(ranges)
}

/// The preflop betting so far, as seen by the next player to act.
#[derive(Default)]
struct PreflopBetting {
    raisers: Vec<Raiser>,
    limpers: Vec<SeatIndex>,
    callers_of_last_raise: usize,
}

impl PreflopBetting {
    fn range_shown_by(
        &self,
        actor: Raiser,
        action: PlayerAction,
        charts: &Charts,
        heads_up: bool,
    ) -> Result<Option<Range>, ChartError> {
        let unraised = self.raisers.is_empty();
        let range = match action {
            PlayerAction::Fold => return Ok(None),
            PlayerAction::Check => {
                Range::full().minus(charts.versus_limpers(Position::BigBlind)?.raise)
            }
            PlayerAction::Call { .. } if unraised => limping_range(actor, charts, heads_up)?,
            PlayerAction::Call { .. } => calling_range(self.chart_facing(actor, charts, heads_up)?),
            PlayerAction::Bet { .. } | PlayerAction::Raise { .. }
                if unraised && self.limpers.is_empty() =>
            {
                charts.raise_first_in(actor.position, heads_up)?.clone()
            }
            PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => {
                self.chart_facing(actor, charts, heads_up)?.raise.clone()
            }
        };
        Ok(Some(range))
    }

    fn chart_facing<'a>(
        &self,
        actor: Raiser,
        charts: &'a Charts,
        heads_up: bool,
    ) -> Result<SpotChart<'a>, ChartError> {
        match self.raisers.as_slice() {
            [] => charts.versus_limpers(chart_position(actor.position, heads_up)),
            [_] if self.limpers.contains(&actor.seat) => charts.limp_then_raised(),
            [_] if self.callers_of_last_raise > 0 => charts.squeeze(),
            [opener] => charts.versus_open(
                chart_position(actor.position, heads_up),
                chart_position(opener.position, heads_up),
            ),
            [opener, three_bettor] if opener.seat == actor.seat => charts.versus_three_bet(
                OpenerGroup::of(opener.position),
                Seating::of(actor.position, three_bettor.position),
            ),
            [_, _] => charts.cold_four_bet(),
            [_, _, _] => charts.versus_four_bet(),
            _ => charts.versus_five_bet(),
        }
    }

    fn record(&mut self, actor: Raiser, action: PlayerAction) {
        match action {
            PlayerAction::Bet { .. } | PlayerAction::Raise { .. } => {
                self.raisers.push(actor);
                self.callers_of_last_raise = 0;
            }
            PlayerAction::Call { .. } if self.raisers.is_empty() => self.limpers.push(actor.seat),
            PlayerAction::Call { .. } => self.callers_of_last_raise += 1,
            PlayerAction::Fold | PlayerAction::Check => {}
        }
    }
}

/// A limper holds the hands it plays but would not open with.
fn limping_range(actor: Raiser, charts: &Charts, heads_up: bool) -> Result<Range, ChartError> {
    match actor.position {
        Position::BigBlind => Ok(Range::full()),
        position => Ok(Range::full().minus(charts.raise_first_in(position, heads_up)?)),
    }
}

/// The calling range of a chart, or its raising range when the chart never
/// calls: a player who called anyway is assumed to hold a strong hand.
fn calling_range(chart: SpotChart) -> Range {
    if chart.call.combo_count() > 0.0 {
        chart.call.clone()
    } else {
        chart.raise.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::Combo;
    use crate::table::state::tests::{cards, seat, six_max_preflop};
    use crate::table::{Chips, RecordedAction, SeatStatus, TableSetup};

    const BUTTON: u8 = 0;
    const BIG_BLIND: u8 = 2;
    const CUTOFF: u8 = 5;

    fn act(seat: u8, street: Street, action: PlayerAction) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street,
            action,
        }
    }

    fn combo(text: &str) -> Combo {
        let parsed = cards(text);
        Combo::of([parsed[0], parsed[1]]).unwrap()
    }

    fn flop_after_button_open(history: Vec<RecordedAction>) -> TableSetup {
        let mut setup = six_max_preflop(BUTTON, "Ah Kd");
        for index in [1, 3, 4, 5] {
            setup.seats[index].status = SeatStatus::Folded;
        }
        setup.seats[BUTTON as usize] = seat(195, 0, SeatStatus::Active);
        setup.seats[BIG_BLIND as usize] = seat(195, 0, SeatStatus::Active);
        setup.board = cards("Kh 7h 2c");
        setup.pot_from_previous_streets = Chips(11);
        setup.history = history;
        setup
    }

    fn estimate(setup: TableSetup, seat: u8) -> Range {
        let table = TableState::new(setup).unwrap();
        let dead = validated_cards_mask(&table.hero_cards()) | validated_cards_mask(table.board());
        let charts = Charts::embedded().unwrap();
        estimate_ranges(&table, charts, &[SeatIndex(seat)], dead)
            .unwrap()
            .remove(0)
    }

    fn open_and_call() -> Vec<RecordedAction> {
        vec![
            act(
                BUTTON,
                Street::Preflop,
                PlayerAction::Raise { to: Chips(5) },
            ),
            act(
                BIG_BLIND,
                Street::Preflop,
                PlayerAction::Call { to: Chips(5) },
            ),
        ]
    }

    #[test]
    fn a_big_blind_caller_holds_the_calling_chart() {
        let range = estimate(flop_after_button_open(open_and_call()), BIG_BLIND);
        let charts = Charts::embedded().unwrap();
        let chart = charts
            .versus_open(Position::BigBlind, Position::Button)
            .unwrap();
        assert_eq!(&range, chart.call);
    }

    #[test]
    fn a_three_bettor_holds_fewer_hands_than_a_caller() {
        let mut history = open_and_call();
        history[1] = act(
            BIG_BLIND,
            Street::Preflop,
            PlayerAction::Raise { to: Chips(18) },
        );
        history.push(act(
            BUTTON,
            Street::Preflop,
            PlayerAction::Call { to: Chips(18) },
        ));
        let three_bettor = estimate(flop_after_button_open(history), BIG_BLIND);
        let caller = estimate(flop_after_button_open(open_and_call()), BIG_BLIND);
        assert!(three_bettor.combo_count() * 2.0 < caller.combo_count());
    }

    #[test]
    fn a_flop_bet_and_turn_call_narrow_the_range() {
        let mut history = open_and_call();
        history.push(act(
            BIG_BLIND,
            Street::Flop,
            PlayerAction::Bet { to: Chips(6) },
        ));
        history.push(act(
            BUTTON,
            Street::Flop,
            PlayerAction::Call { to: Chips(6) },
        ));
        let preflop = estimate(flop_after_button_open(open_and_call()), BIG_BLIND);
        let after_bet = estimate(flop_after_button_open(history), BIG_BLIND);
        assert!(after_bet.combo_count() < preflop.combo_count() * 0.7);
        assert!(after_bet.weight(combo("Ks Qs")) > 0.0);
        assert_eq!(after_bet.weight(combo("4d 3d")), 0.0);
    }

    #[test]
    fn missing_history_falls_back_to_wide_ranges() {
        let big_blind = estimate(flop_after_button_open(vec![]), BIG_BLIND);
        assert_eq!(big_blind, Range::full());
        let button = estimate(flop_after_button_open(vec![]), BUTTON);
        let charts = Charts::embedded().unwrap();
        assert_eq!(
            &button,
            charts.raise_first_in(Position::Button, false).unwrap()
        );
    }

    #[test]
    fn a_limper_does_not_hold_its_opening_hands() {
        let mut setup = flop_after_button_open(vec![
            act(CUTOFF, Street::Preflop, PlayerAction::Call { to: Chips(2) }),
            act(BIG_BLIND, Street::Preflop, PlayerAction::Check),
        ]);
        setup.seats[CUTOFF as usize].status = SeatStatus::Active;
        let limper = estimate(setup, CUTOFF);
        assert_eq!(limper.weight(combo("As Ac")), 0.0);
        assert_eq!(limper.weight(combo("7d 3s")), 1.0);
    }
}

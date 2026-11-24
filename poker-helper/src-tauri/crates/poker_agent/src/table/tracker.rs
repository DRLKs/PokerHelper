use crate::table::action::{PlayerAction, RecordedAction, Street};
use crate::table::chips::Chips;
use crate::table::position::Position;
use crate::table::seat::{SeatIndex, SeatStatus};
use crate::table::state::{TableError, TableSetup, TableState};

/// Rebuilds the action history of a hand from consecutive table frames.
///
/// Each frame is compared with the previous one: chips that a seat added
/// become a call, bet or raise, and a seat that turned `Folded` becomes a
/// fold. A new hand starts when the button or the hero's cards change.
///
/// Limits: if one player acts twice between two frames (for example raises
/// and then calls a re-raise), only the last action is recorded, and checks
/// are only recorded once the street ends.
#[derive(Debug, Default)]
pub struct HandTracker {
    current: Option<TableState>,
    finished: Option<TableState>,
    chips_mismatch: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeatView {
    committed: Chips,
    status: SeatStatus,
}

impl HandTracker {
    pub fn new() -> HandTracker {
        HandTracker::default()
    }

    pub fn current(&self) -> Option<&TableState> {
        self.current.as_ref()
    }

    /// Whether the stacks, bets and pot have added up to the same total in
    /// every frame of the current hand. A frame that breaks this was likely
    /// misread.
    pub fn chips_add_up(&self) -> bool {
        !self.chips_mismatch
    }

    /// The last frame of the most recent hand that ended, once. Feed it to
    /// [`OpponentBook::record_hand`](crate::opponent::OpponentBook::record_hand).
    pub fn take_finished_hand(&mut self) -> Option<TableState> {
        self.finished.take()
    }

    /// Validates the frame and returns it with the history of the hand so far.
    /// On error the tracker keeps its previous state.
    pub fn observe(&mut self, setup: TableSetup) -> Result<&TableState, TableError> {
        let observed = TableState::new(setup)?;
        let history = match self.current.take() {
            Some(previous) if previous.is_same_hand_as(&observed) => {
                if chips_on_table(&previous) != chips_on_table(&observed) {
                    self.chips_mismatch = true;
                }
                continue_history(&previous, &observed)
            }
            Some(previous) => {
                self.finished = Some(previous);
                self.chips_mismatch = false;
                history_from_hand_start(&observed)
            }
            None => history_from_hand_start(&observed),
        };
        Ok(self.current.insert(observed.with_history(history)))
    }
}

fn chips_on_table(table: &TableState) -> Chips {
    table
        .seats()
        .iter()
        .map(|seat| seat.stack_at_street_start())
        .sum::<Chips>()
        + table.pot_from_previous_streets()
}

fn views_of(table: &TableState) -> Vec<SeatView> {
    table
        .seats()
        .iter()
        .map(|seat| SeatView {
            committed: seat.committed_this_street,
            status: seat.status,
        })
        .collect()
}

fn opening_bet(street: Street, table: &TableState) -> Chips {
    match street {
        Street::Preflop => table.big_blind(),
        Street::Flop | Street::Turn | Street::River => Chips::ZERO,
    }
}

fn history_from_hand_start(observed: &TableState) -> Vec<RecordedAction> {
    let street = observed.street();
    let before = match street {
        Street::Preflop => views_after_blinds(observed),
        Street::Flop | Street::Turn | Street::River => views_at_street_start(&views_of(observed)),
    };
    diff_street(street, observed, &before, &views_of(observed), &[])
}

fn views_after_blinds(table: &TableState) -> Vec<SeatView> {
    let blinds = table.blinds();
    let heads_up = table.is_heads_up();
    table
        .seats()
        .iter()
        .enumerate()
        .map(|(index, seat)| {
            let posted = match table.position_of(SeatIndex(index as u8)) {
                Some(Position::SmallBlind) => blinds.small,
                Some(Position::Button) if heads_up => blinds.small,
                Some(Position::BigBlind) => blinds.big,
                Some(_) | None => Chips::ZERO,
            };
            let status = match seat.status {
                SeatStatus::Folded | SeatStatus::AllIn => SeatStatus::Active,
                other => other,
            };
            SeatView {
                committed: posted.min(seat.stack_at_street_start()),
                status,
            }
        })
        .collect()
}

fn views_at_street_start(views: &[SeatView]) -> Vec<SeatView> {
    views
        .iter()
        .map(|view| SeatView {
            committed: Chips::ZERO,
            status: view.status,
        })
        .collect()
}

fn continue_history(previous: &TableState, observed: &TableState) -> Vec<RecordedAction> {
    let mut history = previous.history().to_vec();
    let previous_views = views_of(previous);
    if observed.street() == previous.street() {
        let new_actions = diff_street(
            observed.street(),
            observed,
            &previous_views,
            &views_of(observed),
            &history,
        );
        history.extend(new_actions);
        return history;
    }

    let closing_views = closing_views(previous, observed);
    let closing_actions = diff_street(
        previous.street(),
        previous,
        &previous_views,
        &closing_views,
        &history,
    );
    history.extend(closing_actions);
    let checks = checks_at_street_end(previous.street(), previous, &closing_views, &history);
    history.extend(checks);

    let street_start = views_at_street_start(&closing_views);
    let new_street_actions = diff_street(
        observed.street(),
        observed,
        &street_start,
        &views_of(observed),
        &history,
    );
    history.extend(new_street_actions);
    history
}

/// Commitments at the end of the previous street, recovered from the chips
/// each seat lost between the two frames.
fn closing_views(previous: &TableState, observed: &TableState) -> Vec<SeatView> {
    let final_commitments: Vec<Chips> = previous
        .seats()
        .iter()
        .zip(observed.seats())
        .map(|(before, after)| {
            let added_since = before
                .stack
                .saturating_sub(after.stack)
                .saturating_sub(after.committed_this_street);
            before.committed_this_street + added_since
        })
        .collect();
    let final_bet = final_commitments
        .iter()
        .copied()
        .max()
        .unwrap_or(Chips::ZERO);

    previous
        .seats()
        .iter()
        .zip(observed.seats())
        .zip(&final_commitments)
        .map(|((before, after), &committed)| {
            let folded_facing_a_bet = after.status == SeatStatus::Folded && committed < final_bet;
            let status = if folded_facing_a_bet || after.status == SeatStatus::AllIn {
                after.status
            } else {
                before.status
            };
            SeatView { committed, status }
        })
        .collect()
}

fn checks_at_street_end(
    street: Street,
    table: &TableState,
    closing: &[SeatView],
    history: &[RecordedAction],
) -> Vec<RecordedAction> {
    let final_bet = closing
        .iter()
        .map(|view| view.committed)
        .max()
        .unwrap_or(Chips::ZERO);
    let nobody_added_chips = final_bet == opening_bet(street, table);
    if !nobody_added_chips {
        return Vec::new();
    }
    table
        .seats_in_action_order(street)
        .into_iter()
        .filter(|&(seat, _)| {
            let view = closing[seat.index()];
            let already_acted = history
                .iter()
                .any(|recorded| recorded.street == street && recorded.seat == seat);
            view.status.can_act() && view.committed == final_bet && !already_acted
        })
        .map(|(seat, _)| RecordedAction {
            seat,
            street,
            action: PlayerAction::Check,
        })
        .collect()
}

/// Actions that turn `before` into `after` on one street. Chip increases are
/// replayed in ascending order of the new commitment, because every bet or
/// raise is larger than the one before it. Ties go to the first seat in
/// action order after the last aggressor, which is the one that raised.
fn diff_street(
    street: Street,
    table: &TableState,
    before: &[SeatView],
    after: &[SeatView],
    history: &[RecordedAction],
) -> Vec<RecordedAction> {
    let action_order: Vec<SeatIndex> = table
        .seats_in_action_order(street)
        .into_iter()
        .map(|(seat, _)| seat)
        .collect();
    let mut last_aggressor = history
        .iter()
        .rev()
        .find(|recorded| recorded.street == street && recorded.action.is_aggressive())
        .map(|recorded| recorded.seat);
    let mut running_bet = before
        .iter()
        .map(|view| view.committed)
        .max()
        .unwrap_or(Chips::ZERO)
        .max(opening_bet(street, table));

    let mut increases: Vec<(SeatIndex, Chips)> = action_order
        .iter()
        .filter(|seat| after[seat.index()].committed > before[seat.index()].committed)
        .map(|&seat| (seat, after[seat.index()].committed))
        .collect();
    increases.sort_by_key(|&(_, to)| to);

    let mut actions = Vec::new();
    for group in increases.chunk_by(|first, second| first.1 == second.1) {
        let mut group = group.to_vec();
        group.sort_by_key(|&(seat, _)| turns_after(&action_order, last_aggressor, seat));
        for (seat, to) in group {
            let action = if to <= running_bet {
                PlayerAction::Call { to }
            } else if running_bet == Chips::ZERO {
                PlayerAction::Bet { to }
            } else {
                PlayerAction::Raise { to }
            };
            if action.is_aggressive() {
                running_bet = to;
                last_aggressor = Some(seat);
            }
            actions.push(RecordedAction {
                seat,
                street,
                action,
            });
        }
    }

    let folds = action_order
        .iter()
        .filter(|seat| {
            after[seat.index()].status == SeatStatus::Folded
                && before[seat.index()].status != SeatStatus::Folded
        })
        .map(|&seat| RecordedAction {
            seat,
            street,
            action: PlayerAction::Fold,
        });
    actions.extend(folds);
    actions
}

/// How many turns after `reference` the `seat` acts, or its plain place in
/// the order when there is no reference.
fn turns_after(action_order: &[SeatIndex], reference: Option<SeatIndex>, seat: SeatIndex) -> usize {
    let place_of = |target: SeatIndex| {
        action_order
            .iter()
            .position(|&candidate| candidate == target)
    };
    let seat_place = place_of(seat).unwrap_or(action_order.len());
    match reference.and_then(place_of) {
        Some(reference_place) => {
            (seat_place + action_order.len() - reference_place - 1) % action_order.len()
        }
        None => seat_place,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::{cards, seat, six_max_preflop};

    fn actions(table: &TableState) -> Vec<(u8, PlayerAction)> {
        table
            .history()
            .iter()
            .map(|recorded| (recorded.seat.0, recorded.action))
            .collect()
    }

    #[test]
    fn reads_an_open_a_call_and_a_fold_from_the_first_frame() {
        let mut frame = six_max_preflop(0, "Ah Kd");
        frame.seats[3] = seat(194, 6, SeatStatus::Active);
        frame.seats[4] = seat(200, 0, SeatStatus::Folded);
        frame.seats[5] = seat(194, 6, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        let table = tracker.observe(frame).unwrap();
        assert_eq!(
            actions(table),
            [
                (3, PlayerAction::Raise { to: Chips(6) }),
                (5, PlayerAction::Call { to: Chips(6) }),
                (4, PlayerAction::Fold),
            ]
        );
    }

    #[test]
    fn a_limp_is_a_call_of_the_big_blind() {
        let mut frame = six_max_preflop(0, "Ah Kd");
        frame.seats[3] = seat(198, 2, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        let table = tracker.observe(frame).unwrap();
        assert_eq!(actions(table), [(3, PlayerAction::Call { to: Chips(2) })]);
    }

    #[test]
    fn orders_a_three_bet_and_the_call_seen_in_the_same_frame() {
        let mut first = six_max_preflop(2, "Ah Kd");
        first.seats[3] = seat(194, 6, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        tracker.observe(first.clone()).unwrap();

        let mut second = first;
        second.seats[4] = seat(200, 0, SeatStatus::Folded);
        second.seats[5] = seat(200, 0, SeatStatus::Folded);
        second.seats[0] = seat(180, 20, SeatStatus::Active);
        second.seats[3] = seat(180, 20, SeatStatus::Active);
        let table = tracker.observe(second).unwrap();
        assert_eq!(
            actions(table),
            [
                (3, PlayerAction::Raise { to: Chips(6) }),
                (0, PlayerAction::Raise { to: Chips(20) }),
                (3, PlayerAction::Call { to: Chips(20) }),
                (4, PlayerAction::Fold),
                (5, PlayerAction::Fold),
            ]
        );
    }

    #[test]
    fn closes_the_previous_street_when_the_flop_appears() {
        let mut preflop = six_max_preflop(2, "Ah Kd");
        preflop.seats[3] = seat(194, 6, SeatStatus::Active);
        for index in [0, 4, 5] {
            preflop.seats[index].status = SeatStatus::Folded;
        }
        let mut tracker = HandTracker::new();
        tracker.observe(preflop.clone()).unwrap();

        let mut flop = preflop;
        flop.board = cards("2c 7d Ts");
        flop.pot_from_previous_streets = Chips(13);
        flop.seats[1] = seat(199, 0, SeatStatus::Folded);
        flop.seats[2] = seat(184, 10, SeatStatus::Active);
        flop.seats[3] = seat(194, 0, SeatStatus::Active);
        let table = tracker.observe(flop).unwrap();
        assert_eq!(
            actions(table)[4..],
            [
                (2, PlayerAction::Call { to: Chips(6) }),
                (1, PlayerAction::Fold),
                (2, PlayerAction::Bet { to: Chips(10) }),
            ]
        );
        assert_eq!(table.history()[6].street, Street::Flop);
    }

    #[test]
    fn records_checks_when_a_street_ends_without_bets() {
        let mut flop = six_max_preflop(2, "Ah Kd");
        flop.board = cards("2c 7d Ts");
        flop.pot_from_previous_streets = Chips(12);
        for index in [0, 1, 4, 5] {
            flop.seats[index] = seat(194, 0, SeatStatus::Folded);
        }
        flop.seats[2] = seat(194, 0, SeatStatus::Active);
        flop.seats[3] = seat(194, 0, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        tracker.observe(flop.clone()).unwrap();

        let mut turn = flop;
        turn.board = cards("2c 7d Ts Qh");
        let table = tracker.observe(turn).unwrap();
        assert_eq!(
            actions(table),
            [(2, PlayerAction::Check), (3, PlayerAction::Check)]
        );
    }

    #[test]
    fn a_new_hand_starts_a_new_history() {
        let mut first = six_max_preflop(0, "Ah Kd");
        first.seats[3] = seat(194, 6, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        tracker.observe(first).unwrap();

        let next_hand = six_max_preflop(0, "2c 3d");
        let table = tracker.observe(next_hand).unwrap();
        assert!(table.history().is_empty());
        let finished = tracker.take_finished_hand().unwrap();
        assert_eq!(finished.history().len(), 1);
        assert!(tracker.take_finished_hand().is_none());
    }

    #[test]
    fn notices_chips_that_do_not_add_up_until_the_next_hand() {
        let mut first = six_max_preflop(0, "Ah Kd");
        first.seats[3] = seat(194, 6, SeatStatus::Active);
        let mut tracker = HandTracker::new();
        tracker.observe(first.clone()).unwrap();
        assert!(tracker.chips_add_up());

        let mut misread = first;
        misread.seats[4] = seat(150, 0, SeatStatus::Active);
        tracker.observe(misread).unwrap();
        assert!(!tracker.chips_add_up());

        tracker.observe(six_max_preflop(1, "2c 3d")).unwrap();
        assert!(tracker.chips_add_up());
    }

    #[test]
    fn keeps_the_previous_state_when_a_frame_is_invalid() {
        let mut tracker = HandTracker::new();
        tracker.observe(six_max_preflop(0, "Ah Kd")).unwrap();
        assert!(tracker.observe(six_max_preflop(0, "Ah Ah")).is_err());
        assert!(tracker.current().is_some());
    }
}

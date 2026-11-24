use crate::core::card::Card;
use crate::evaluator::fast_eval::{card_bit, CardMask};
use crate::table::action::{RecordedAction, Street};
use crate::table::chips::Chips;
use crate::table::position::{Position, MAX_PLAYERS, MIN_PLAYERS};
use crate::table::seat::{Seat, SeatIndex};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blinds {
    pub small: Chips,
    pub big: Chips,
    #[serde(default)]
    pub ante: Chips,
}

/// Raw description of a table, checked by [`TableState::new`].
#[derive(Clone, Debug, PartialEq)]
pub struct TableSetup {
    pub seats: Vec<Seat>,
    pub button: SeatIndex,
    pub hero: SeatIndex,
    pub hero_cards: [Card; 2],
    pub board: Vec<Card>,
    pub blinds: Blinds,
    /// Chips collected from earlier streets and antes, without the chips
    /// committed on the current street.
    pub pot_from_previous_streets: Chips,
    pub history: Vec<RecordedAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError {
    WrongPlayerCount(usize),
    SeatOutOfRange(SeatIndex),
    ButtonNotDealtIn(SeatIndex),
    HeroNotDealtIn(SeatIndex),
    InvalidBoardSize(usize),
    InvalidCard(Card),
    DuplicateCard(Card),
    InvalidBlinds(Blinds),
}

impl fmt::Display for TableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::WrongPlayerCount(count) => write!(
                formatter,
                "{count} players are dealt in, expected {MIN_PLAYERS} to {MAX_PLAYERS}"
            ),
            TableError::SeatOutOfRange(seat) => write!(formatter, "seat {} does not exist", seat.0),
            TableError::ButtonNotDealtIn(seat) => write!(formatter, "the button seat {} is not dealt in", seat.0),
            TableError::HeroNotDealtIn(seat) => write!(formatter, "the hero seat {} is not dealt in", seat.0),
            TableError::InvalidBoardSize(len) => write!(formatter, "{len} community cards is not a valid board"),
            TableError::InvalidCard(card) => write!(formatter, "invalid card {card:?}"),
            TableError::DuplicateCard(card) => write!(formatter, "card {card} appears twice"),
            TableError::InvalidBlinds(blinds) => write!(
                formatter,
                "invalid blinds {}/{}: the big blind must be positive and not smaller than the small blind",
                blinds.small, blinds.big
            ),
        }
    }
}

impl std::error::Error for TableError {}

/// A validated table: 2 to 6 dealt-in players, the button and the hero are
/// dealt in, the board has 0, 3, 4 or 5 cards, every card is valid and unique,
/// and the big blind is positive.
#[derive(Clone, Debug, PartialEq)]
pub struct TableState {
    seats: Vec<Seat>,
    positions: Vec<Option<Position>>,
    button: SeatIndex,
    hero: SeatIndex,
    hero_cards: [Card; 2],
    board: Vec<Card>,
    street: Street,
    blinds: Blinds,
    pot_from_previous_streets: Chips,
    history: Vec<RecordedAction>,
}

impl TableState {
    pub fn new(setup: TableSetup) -> Result<TableState, TableError> {
        let blinds = setup.blinds;
        if blinds.big == Chips::ZERO || blinds.small > blinds.big {
            return Err(TableError::InvalidBlinds(blinds));
        }
        let street = Street::from_board_len(setup.board.len())
            .ok_or(TableError::InvalidBoardSize(setup.board.len()))?;
        ensure_cards_are_valid_and_unique(setup.hero_cards.iter().chain(&setup.board))?;
        let positions = assign_positions(&setup.seats, setup.button)?;
        let hero_position = positions
            .get(setup.hero.index())
            .ok_or(TableError::SeatOutOfRange(setup.hero))?;
        if hero_position.is_none() {
            return Err(TableError::HeroNotDealtIn(setup.hero));
        }

        Ok(TableState {
            seats: setup.seats,
            positions,
            button: setup.button,
            hero: setup.hero,
            hero_cards: setup.hero_cards,
            board: setup.board,
            street,
            blinds,
            pot_from_previous_streets: setup.pot_from_previous_streets,
            history: setup.history,
        })
    }

    pub fn with_history(mut self, history: Vec<RecordedAction>) -> TableState {
        self.history = history;
        self
    }

    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    pub fn seat(&self, seat: SeatIndex) -> Option<&Seat> {
        self.seats.get(seat.index())
    }

    pub fn hero(&self) -> SeatIndex {
        self.hero
    }

    pub fn hero_seat(&self) -> &Seat {
        &self.seats[self.hero.index()]
    }

    pub fn hero_cards(&self) -> [Card; 2] {
        self.hero_cards
    }

    pub fn hero_position(&self) -> Position {
        self.positions[self.hero.index()]
            .expect("TableState::new rejects a hero who is not dealt in")
    }

    pub fn button(&self) -> SeatIndex {
        self.button
    }

    pub fn board(&self) -> &[Card] {
        &self.board
    }

    pub fn street(&self) -> Street {
        self.street
    }

    pub fn blinds(&self) -> Blinds {
        self.blinds
    }

    pub fn big_blind(&self) -> Chips {
        self.blinds.big
    }

    pub fn pot_from_previous_streets(&self) -> Chips {
        self.pot_from_previous_streets
    }

    pub fn history(&self) -> &[RecordedAction] {
        &self.history
    }

    pub fn actions_on(&self, street: Street) -> impl Iterator<Item = &RecordedAction> {
        self.history
            .iter()
            .filter(move |recorded| recorded.street == street)
    }

    /// `None` when the seat does not exist or was not dealt in.
    pub fn position_of(&self, seat: SeatIndex) -> Option<Position> {
        self.positions.get(seat.index()).copied().flatten()
    }

    pub fn dealt_in_player_count(&self) -> usize {
        self.positions.iter().flatten().count()
    }

    pub fn is_heads_up(&self) -> bool {
        self.dealt_in_player_count() == MIN_PLAYERS
    }

    /// Dealt-in seats in the order they act on `street`.
    pub fn seats_in_action_order(&self, street: Street) -> Vec<(SeatIndex, Position)> {
        let mut ordered: Vec<(SeatIndex, Position)> = self
            .positions
            .iter()
            .enumerate()
            .filter_map(|(index, position)| {
                position.map(|position| (SeatIndex(index as u8), position))
            })
            .collect();
        match street {
            Street::Preflop => ordered.sort_by_key(|(_, position)| position.preflop_order()),
            Street::Flop | Street::Turn | Street::River => {
                ordered.sort_by_key(|(_, position)| position.postflop_order())
            }
        }
        ordered
    }

    /// Largest amount any player has committed on the current street.
    pub fn current_bet(&self) -> Chips {
        self.seats
            .iter()
            .map(|seat| seat.committed_this_street)
            .max()
            .unwrap_or(Chips::ZERO)
    }

    /// Chips `seat` must add to call, capped at its stack.
    pub fn amount_to_call(&self, seat: SeatIndex) -> Chips {
        let Some(player) = self.seat(seat) else {
            return Chips::ZERO;
        };
        self.current_bet()
            .saturating_sub(player.committed_this_street)
            .min(player.stack)
    }

    /// Everything in the middle, including the chips committed this street.
    pub fn total_pot(&self) -> Chips {
        self.pot_from_previous_streets
            + self
                .seats
                .iter()
                .map(|seat| seat.committed_this_street)
                .sum()
    }

    pub fn opponents_in_hand(&self, seat: SeatIndex) -> Vec<SeatIndex> {
        (0..self.seats.len())
            .map(|index| SeatIndex(index as u8))
            .filter(|&other| other != seat && self.seats[other.index()].status.is_in_hand())
            .collect()
    }

    /// The most `seat` can win or lose from the start of this street: its own
    /// stack, capped by the largest stack among opponents still in the hand.
    pub fn effective_stack(&self, seat: SeatIndex) -> Chips {
        let Some(player) = self.seat(seat) else {
            return Chips::ZERO;
        };
        let largest_opponent_stack = self
            .opponents_in_hand(seat)
            .into_iter()
            .map(|opponent| self.seats[opponent.index()].stack_at_street_start())
            .max()
            .unwrap_or(Chips::ZERO);
        player.stack_at_street_start().min(largest_opponent_stack)
    }

    pub fn is_same_hand_as(&self, other: &TableState) -> bool {
        self.button == other.button && self.hero_cards == other.hero_cards
    }
}

fn ensure_cards_are_valid_and_unique<'a>(
    cards: impl Iterator<Item = &'a Card>,
) -> Result<(), TableError> {
    let mut seen: CardMask = 0;
    for card in cards {
        let bit = card_bit(card).ok_or(TableError::InvalidCard(*card))?;
        let flag = 1u64 << bit;
        if seen & flag != 0 {
            return Err(TableError::DuplicateCard(*card));
        }
        seen |= flag;
    }
    Ok(())
}

fn assign_positions(
    seats: &[Seat],
    button: SeatIndex,
) -> Result<Vec<Option<Position>>, TableError> {
    let button_seat = seats
        .get(button.index())
        .ok_or(TableError::SeatOutOfRange(button))?;
    if !button_seat.status.is_dealt_in() {
        return Err(TableError::ButtonNotDealtIn(button));
    }

    let dealt_in_clockwise_from_button: Vec<usize> = (0..seats.len())
        .map(|offset| (button.index() + offset) % seats.len())
        .filter(|&index| seats[index].status.is_dealt_in())
        .collect();
    let player_count = dealt_in_clockwise_from_button.len();
    let table_positions = Position::clockwise_from_button(player_count)
        .ok_or(TableError::WrongPlayerCount(player_count))?;

    let mut positions = vec![None; seats.len()];
    for (&seat_index, &position) in dealt_in_clockwise_from_button.iter().zip(table_positions) {
        positions[seat_index] = Some(position);
    }
    Ok(positions)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::table::seat::SeatStatus;

    pub(crate) fn seat(stack: u64, committed: u64, status: SeatStatus) -> Seat {
        Seat {
            stack: Chips(stack),
            committed_this_street: Chips(committed),
            status,
        }
    }

    pub(crate) fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace()
            .map(|card| card.parse().unwrap())
            .collect()
    }

    pub(crate) fn six_max_preflop(hero: u8, hero_cards: &str) -> TableSetup {
        let hero_cards = cards(hero_cards);
        TableSetup {
            seats: vec![
                seat(200, 0, SeatStatus::Active),
                seat(199, 1, SeatStatus::Active),
                seat(198, 2, SeatStatus::Active),
                seat(200, 0, SeatStatus::Active),
                seat(200, 0, SeatStatus::Active),
                seat(200, 0, SeatStatus::Active),
            ],
            button: SeatIndex(0),
            hero: SeatIndex(hero),
            hero_cards: [hero_cards[0], hero_cards[1]],
            board: vec![],
            blinds: Blinds {
                small: Chips(1),
                big: Chips(2),
                ante: Chips::ZERO,
            },
            pot_from_previous_streets: Chips::ZERO,
            history: vec![],
        }
    }

    #[test]
    fn assigns_six_max_positions_clockwise_from_the_button() {
        let mut setup = six_max_preflop(3, "Ah Kd");
        setup.button = SeatIndex(4);
        let table = TableState::new(setup).unwrap();
        let positions: Vec<Position> = (0..6)
            .map(|seat| table.position_of(SeatIndex(seat)).unwrap())
            .collect();
        use Position::*;
        assert_eq!(
            positions,
            [BigBlind, UnderTheGun, Hijack, Cutoff, Button, SmallBlind]
        );
        assert_eq!(table.hero_position(), Cutoff);
    }

    #[test]
    fn players_sitting_out_shrink_the_table() {
        let mut setup = six_max_preflop(0, "Ah Kd");
        setup.seats[3].status = SeatStatus::SittingOut;
        setup.seats[4].status = SeatStatus::Empty;
        let table = TableState::new(setup).unwrap();
        assert_eq!(table.dealt_in_player_count(), 4);
        assert_eq!(table.position_of(SeatIndex(5)), Some(Position::Cutoff));
        assert_eq!(table.position_of(SeatIndex(3)), None);
    }

    #[test]
    fn heads_up_has_button_and_big_blind() {
        let mut setup = six_max_preflop(0, "Ah Kd");
        for index in 2..6 {
            setup.seats[index].status = SeatStatus::Empty;
        }
        let table = TableState::new(setup).unwrap();
        assert!(table.is_heads_up());
        assert_eq!(table.position_of(SeatIndex(0)), Some(Position::Button));
        assert_eq!(table.position_of(SeatIndex(1)), Some(Position::BigBlind));
    }

    #[test]
    fn computes_pot_call_and_effective_stack() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[3] = seat(194, 6, SeatStatus::Active);
        setup.seats[4] = seat(200, 0, SeatStatus::Folded);
        setup.seats[2] = seat(50, 2, SeatStatus::Active);
        let table = TableState::new(setup).unwrap();
        assert_eq!(table.current_bet(), Chips(6));
        assert_eq!(table.amount_to_call(SeatIndex(5)), Chips(6));
        assert_eq!(table.total_pot(), Chips(9));
        assert_eq!(table.effective_stack(SeatIndex(5)), Chips(200));
        assert_eq!(table.effective_stack(SeatIndex(2)), Chips(52));
        assert_eq!(table.opponents_in_hand(SeatIndex(5)).len(), 4);
    }

    #[test]
    fn call_is_capped_by_the_stack() {
        let mut setup = six_max_preflop(5, "Ah Kd");
        setup.seats[3] = seat(0, 200, SeatStatus::AllIn);
        setup.seats[5] = seat(40, 0, SeatStatus::Active);
        let table = TableState::new(setup).unwrap();
        assert_eq!(table.amount_to_call(SeatIndex(5)), Chips(40));
    }

    #[test]
    fn rejects_impossible_tables() {
        let mut duplicate = six_max_preflop(3, "Ah Ah");
        assert_eq!(
            TableState::new(duplicate.clone()).unwrap_err(),
            TableError::DuplicateCard("Ah".parse().unwrap())
        );
        duplicate.hero_cards = [Card::new('x', 14), Card::new('h', 2)];
        assert!(matches!(
            TableState::new(duplicate).unwrap_err(),
            TableError::InvalidCard(_)
        ));

        let mut bad_board = six_max_preflop(3, "Ah Kd");
        bad_board.board = cards("2c 3c");
        assert_eq!(
            TableState::new(bad_board).unwrap_err(),
            TableError::InvalidBoardSize(2)
        );

        let mut folded_out_hero = six_max_preflop(3, "Ah Kd");
        folded_out_hero.seats[3].status = SeatStatus::SittingOut;
        assert_eq!(
            TableState::new(folded_out_hero).unwrap_err(),
            TableError::HeroNotDealtIn(SeatIndex(3))
        );

        let mut lonely = six_max_preflop(0, "Ah Kd");
        for index in 1..6 {
            lonely.seats[index].status = SeatStatus::Empty;
        }
        assert_eq!(
            TableState::new(lonely).unwrap_err(),
            TableError::WrongPlayerCount(1)
        );

        let mut empty_button = six_max_preflop(3, "Ah Kd");
        empty_button.seats[0].status = SeatStatus::Empty;
        assert_eq!(
            TableState::new(empty_button).unwrap_err(),
            TableError::ButtonNotDealtIn(SeatIndex(0))
        );

        let mut no_big_blind = six_max_preflop(3, "Ah Kd");
        no_big_blind.blinds.big = Chips::ZERO;
        assert!(matches!(
            TableState::new(no_big_blind).unwrap_err(),
            TableError::InvalidBlinds(_)
        ));
    }
}

use crate::core::card::Card;
use crate::evaluator::fast_eval::{card_bit, evaluate, CardMask};
use crate::math::Probability;
use crate::range::combo::{CardIndex, DECK_SIZE};
use crate::range::hand_range::Range;
use crate::table::position::MAX_PLAYERS;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::fmt;

/// Most opponents a 6-max hand can have.
pub const MAX_OPPONENTS: usize = MAX_PLAYERS - 1;
const FULL_BOARD: usize = 5;
const CHUNK_SIZE: u32 = 2048;
const MAX_SAMPLING_ATTEMPTS: u32 = 64;
const RANKS_PER_SUIT: usize = 13;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquityError {
    InvalidCard(Card),
    DuplicateCard(Card),
    TooManyBoardCards(usize),
    TooManyOpponents(usize),
    NoIterations,
    /// The range of this opponent has no combo left once the known cards are removed.
    EmptyRange {
        opponent: usize,
    },
    /// The ranges overlap so much that no deal gives every opponent a hand.
    RangesCannotBeDealt,
}

impl fmt::Display for EquityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EquityError::InvalidCard(card) => write!(formatter, "invalid card {card:?}"),
            EquityError::DuplicateCard(card) => write!(formatter, "card {card} appears twice"),
            EquityError::TooManyBoardCards(count) => {
                write!(formatter, "{count} community cards, at most {FULL_BOARD}")
            }
            EquityError::TooManyOpponents(count) => {
                write!(formatter, "{count} opponents, at most {MAX_OPPONENTS}")
            }
            EquityError::NoIterations => write!(formatter, "iterations must be greater than 0"),
            EquityError::EmptyRange { opponent } => {
                write!(formatter, "opponent {opponent} has no possible hand")
            }
            EquityError::RangesCannotBeDealt => {
                write!(formatter, "the opponent ranges cannot be dealt together")
            }
        }
    }
}

impl std::error::Error for EquityError {}

/// Share of the pot `hero` wins on average against one hand from each of
/// `opponents`, drawn by weight. Split pots count as `1/k` of a win.
///
/// Against one opponent with a flop, turn or river on the board the result is
/// exact. Otherwise it is a Monte Carlo estimate over `iterations` deals; an
/// opponent combo that collides with cards already dealt is redrawn, which
/// slightly favours combos that collide less, as in most poker tools.
pub fn equity_vs_ranges(
    hero: [Card; 2],
    board: &[Card],
    opponents: &[Range],
    iterations: u32,
) -> Result<Probability, EquityError> {
    if opponents.is_empty() {
        return Ok(Probability::ONE);
    }
    if opponents.len() > MAX_OPPONENTS {
        return Err(EquityError::TooManyOpponents(opponents.len()));
    }
    if board.len() > FULL_BOARD {
        return Err(EquityError::TooManyBoardCards(board.len()));
    }
    if iterations == 0 {
        return Err(EquityError::NoIterations);
    }

    let hero_mask = mask_of(&hero, 0)?;
    let board_mask = mask_of(board, hero_mask)?;
    let dead_cards = hero_mask | board_mask;
    let mut live_ranges = Vec::with_capacity(opponents.len());
    for (opponent, range) in opponents.iter().enumerate() {
        let live = LiveCombos::new(range, dead_cards);
        if live.total_weight <= 0.0 {
            return Err(EquityError::EmptyRange { opponent });
        }
        live_ranges.push(live);
    }

    let deal = Deal {
        hero_mask,
        board_mask,
        board_missing: FULL_BOARD - board.len(),
    };
    let equity = match live_ranges.as_slice() {
        [single_opponent] if board.len() >= 3 => deal.exact_heads_up(single_opponent),
        _ => deal.simulate(&live_ranges, iterations)?,
    };
    Ok(Probability::saturating(equity))
}

fn mask_of(cards: &[Card], already_used: CardMask) -> Result<CardMask, EquityError> {
    let mut mask = 0;
    for card in cards {
        let flag = 1u64 << card_bit(card).ok_or(EquityError::InvalidCard(*card))?;
        if (mask | already_used) & flag != 0 {
            return Err(EquityError::DuplicateCard(*card));
        }
        mask |= flag;
    }
    Ok(mask)
}

/// The combos of a range that do not use a known card, with cumulative
/// weights for sampling.
struct LiveCombos {
    masks: Vec<CardMask>,
    weights: Vec<f64>,
    cumulative_weights: Vec<f64>,
    total_weight: f64,
}

impl LiveCombos {
    fn new(range: &Range, dead_cards: CardMask) -> LiveCombos {
        let (masks, weights): (Vec<CardMask>, Vec<f64>) = range
            .weighted_combos()
            .filter(|(combo, _)| combo.mask() & dead_cards == 0)
            .map(|(combo, weight)| (combo.mask(), weight as f64))
            .unzip();
        let cumulative_weights: Vec<f64> = weights
            .iter()
            .scan(0.0, |running, &weight| {
                *running += weight;
                Some(*running)
            })
            .collect();
        let total_weight = cumulative_weights.last().copied().unwrap_or(0.0);
        LiveCombos {
            masks,
            weights,
            cumulative_weights,
            total_weight,
        }
    }

    fn sample(&self, rng: &mut SmallRng) -> CardMask {
        let target = rng.gen::<f64>() * self.total_weight;
        let index = self
            .cumulative_weights
            .partition_point(|&cumulative| cumulative <= target);
        self.masks[index.min(self.masks.len() - 1)]
    }
}

struct Deal {
    hero_mask: CardMask,
    board_mask: CardMask,
    board_missing: usize,
}

impl Deal {
    /// Enumerates every board completion for every opponent combo.
    fn exact_heads_up(&self, opponent: &LiveCombos) -> f64 {
        let deck: Vec<CardMask> = CardIndex::all()
            .map(CardIndex::mask)
            .filter(|&card| card & (self.hero_mask | self.board_mask) == 0)
            .collect();
        let weighted_share: f64 = opponent
            .masks
            .par_iter()
            .zip(&opponent.weights)
            .map(|(&opponent_mask, &weight)| {
                weight * self.average_share_against(opponent_mask, &deck)
            })
            .sum();
        weighted_share / opponent.total_weight
    }

    fn average_share_against(&self, opponent_mask: CardMask, deck: &[CardMask]) -> f64 {
        let share = |board: CardMask| {
            showdown_share(
                evaluate(board | self.hero_mask),
                evaluate(board | opponent_mask),
            )
        };
        let live = |card: &&CardMask| **card & opponent_mask == 0;
        match self.board_missing {
            0 => share(self.board_mask),
            1 => {
                let (total, count) = deck
                    .iter()
                    .filter(live)
                    .fold((0.0, 0u32), |(total, count), &river| {
                        (total + share(self.board_mask | river), count + 1)
                    });
                total / count as f64
            }
            _ => {
                let live_deck: Vec<CardMask> = deck.iter().filter(live).copied().collect();
                let mut total = 0.0;
                let mut count = 0u32;
                for (position, &turn) in live_deck.iter().enumerate() {
                    for &river in &live_deck[position + 1..] {
                        total += share(self.board_mask | turn | river);
                        count += 1;
                    }
                }
                total / count as f64
            }
        }
    }

    fn simulate(&self, opponents: &[LiveCombos], iterations: u32) -> Result<f64, EquityError> {
        let chunks = iterations.div_ceil(CHUNK_SIZE);
        let base_seed: u64 = rand::thread_rng().gen();
        let (won, dealt) = (0..chunks)
            .into_par_iter()
            .map(|chunk| {
                let chunk_iterations = CHUNK_SIZE.min(iterations - chunk * CHUNK_SIZE);
                let mut rng = SmallRng::seed_from_u64(base_seed.wrapping_add(chunk as u64));
                self.simulate_chunk(opponents, chunk_iterations, &mut rng)
            })
            .reduce(
                || (0.0, 0u32),
                |left, right| (left.0 + right.0, left.1 + right.1),
            );
        if dealt == 0 {
            return Err(EquityError::RangesCannotBeDealt);
        }
        Ok(won / dealt as f64)
    }

    /// Returns the pot share won and the number of deals that succeeded.
    fn simulate_chunk(
        &self,
        opponents: &[LiveCombos],
        iterations: u32,
        rng: &mut SmallRng,
    ) -> (f64, u32) {
        let mut opponent_hands = [0u64; MAX_OPPONENTS];
        let mut won = 0.0;
        let mut dealt = 0u32;
        for _ in 0..iterations {
            let mut used = self.hero_mask | self.board_mask;
            let all_dealt = opponents
                .iter()
                .zip(opponent_hands.iter_mut())
                .all(
                    |(range, hand)| match sample_without_collision(range, used, rng) {
                        Some(mask) => {
                            *hand = mask;
                            used |= mask;
                            true
                        }
                        None => false,
                    },
                );
            if !all_dealt {
                continue;
            }

            let mut board = self.board_mask;
            for _ in 0..self.board_missing {
                let card = draw_unused_card(used, rng);
                used |= card;
                board |= card;
            }
            won += multiway_share(
                evaluate(board | self.hero_mask),
                &opponent_hands[..opponents.len()],
                board,
            );
            dealt += 1;
        }
        (won, dealt)
    }
}

fn sample_without_collision(
    range: &LiveCombos,
    used: CardMask,
    rng: &mut SmallRng,
) -> Option<CardMask> {
    (0..MAX_SAMPLING_ATTEMPTS)
        .map(|_| range.sample(rng))
        .find(|&mask| mask & used == 0)
}

fn draw_unused_card(used: CardMask, rng: &mut SmallRng) -> CardMask {
    loop {
        let card = rng.gen_range(0..DECK_SIZE);
        let card_mask = 1u64 << ((card / RANKS_PER_SUIT) * 16 + card % RANKS_PER_SUIT);
        if card_mask & used == 0 {
            return card_mask;
        }
    }
}

fn showdown_share(hero_strength: u32, opponent_strength: u32) -> f64 {
    match hero_strength.cmp(&opponent_strength) {
        std::cmp::Ordering::Greater => 1.0,
        std::cmp::Ordering::Equal => 0.5,
        std::cmp::Ordering::Less => 0.0,
    }
}

fn multiway_share(hero_strength: u32, opponent_hands: &[CardMask], board: CardMask) -> f64 {
    let mut ties = 0u32;
    for &hand in opponent_hands {
        let opponent_strength = evaluate(board | hand);
        if opponent_strength > hero_strength {
            return 0.0;
        }
        ties += (opponent_strength == hero_strength) as u32;
    }
    1.0 / (ties + 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::community_cards::CommunityCards;
    use crate::core::hand::Hand;
    use crate::probability::calculate_equity;

    const SIMULATION: u32 = 200_000;

    fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace()
            .map(|card| card.parse().unwrap())
            .collect()
    }

    fn hero(text: &str) -> [Card; 2] {
        let parsed = cards(text);
        [parsed[0], parsed[1]]
    }

    fn range(text: &str) -> Range {
        text.parse().unwrap()
    }

    fn equity(hero_cards: &str, board: &str, opponents: &[Range], iterations: u32) -> f64 {
        equity_vs_ranges(hero(hero_cards), &cards(board), opponents, iterations)
            .unwrap()
            .percent()
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64, label: &str) {
        assert!(
            (actual - expected).abs() < tolerance,
            "{label}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn exact_equity_against_a_full_range_matches_the_random_hand_calculator() {
        for (hand, board) in [
            ("Ah Kd", "2h 9c Td Ks Qh"),
            ("Ah Kd", "2h 9c Td Ks"),
            ("7s 7c", "2h 9c Td"),
        ] {
            let against_full_range = equity(hand, board, &[Range::full()], 1);
            let against_random_hand = calculate_equity(
                Hand::new(hero(hand)),
                CommunityCards::new(cards(board)),
                1,
                u32::MAX,
            )
            .unwrap();
            assert_close(against_full_range, against_random_hand, 1e-9, board);
        }
    }

    #[test]
    fn preflop_matchups_match_published_values() {
        assert_close(
            equity("As Ah", "", &[Range::full()], SIMULATION),
            85.2,
            1.0,
            "AA vs any",
        );
        assert_close(
            equity("As Ks", "", &[range("QQ")], SIMULATION),
            46.0,
            1.0,
            "AKs vs QQ",
        );
        assert_close(
            equity("Qs Qh", "", &[range("AKs")], SIMULATION),
            54.0,
            1.0,
            "QQ vs AKs",
        );
    }

    #[test]
    fn a_raise_range_cuts_equity_compared_with_random_hands() {
        let against_random = equity("As Jd", "", &[Range::full()], SIMULATION);
        let against_tight = equity("As Jd", "", &[range("TT+, AQ+")], SIMULATION);
        assert!(
            against_tight + 25.0 < against_random,
            "AJo: {against_tight} vs {against_random}"
        );
    }

    #[test]
    fn range_equity_is_the_combo_weighted_average() {
        let board = "2h 7c Td";
        let queens = range("QQ");
        let deuces = range("22");
        let equity_vs_queens = equity("Ah Ad", board, std::slice::from_ref(&queens), 1);
        let equity_vs_deuces = equity("Ah Ad", board, std::slice::from_ref(&deuces), 1);
        let live_queens = 6.0;
        let live_deuces = 3.0;
        let expected = (live_queens * equity_vs_queens + live_deuces * equity_vs_deuces)
            / (live_queens + live_deuces);
        let combined = equity("Ah Ad", board, &[queens.union(&deuces)], 1);
        assert_close(combined, expected, 1e-9, "weighted average");
    }

    #[test]
    fn multiway_equity_matches_the_random_hand_calculator() {
        let against_ranges = equity(
            "Jh Jd",
            "2c 7s Kd",
            &[Range::full(), Range::full()],
            SIMULATION,
        );
        let against_random_hands = calculate_equity(
            Hand::new(hero("Jh Jd")),
            CommunityCards::new(cards("2c 7s Kd")),
            2,
            SIMULATION,
        )
        .unwrap();
        assert_close(
            against_ranges,
            against_random_hands,
            1.0,
            "JJ vs two random hands",
        );
    }

    #[test]
    fn reports_impossible_inputs() {
        let error = |hero_cards: &str, board: &str, opponents: &[Range]| {
            equity_vs_ranges(hero(hero_cards), &cards(board), opponents, 1000).unwrap_err()
        };
        assert_eq!(
            error("Ah Ah", "", &[Range::full()]),
            EquityError::DuplicateCard("Ah".parse().unwrap())
        );
        assert_eq!(
            error("Ah Kd", "", &[range("AA, KK"), range("")]),
            EquityError::EmptyRange { opponent: 1 }
        );
        assert_eq!(
            error("Ah Kd", "Ac As 2c", &[range("AA")]),
            EquityError::EmptyRange { opponent: 0 }
        );
        assert_eq!(
            error("Ah Kd", "2c 3c 4c 5c 6c 7c", &[Range::full()]),
            EquityError::TooManyBoardCards(6)
        );
        assert_eq!(
            error("Ah Ad", "", &[range("AA"), range("AA")]),
            EquityError::RangesCannotBeDealt
        );
        assert_eq!(
            equity_vs_ranges(hero("Ah Kd"), &[], &vec![Range::full(); 6], 1000).unwrap_err(),
            EquityError::TooManyOpponents(6)
        );
        assert_eq!(
            equity_vs_ranges(hero("Ah Kd"), &[], &[], 1000).unwrap(),
            Probability::ONE
        );
    }
}

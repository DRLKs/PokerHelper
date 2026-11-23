use crate::core::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::core::hand::Hand;
use crate::evaluator::fast_eval::{card_bit, evaluate, CardMask};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

/// Iterations simulated by one parallel work unit. Small enough to keep all
/// cores busy for typical requests (10k iterations -> 5 chunks), large enough
/// that scheduling overhead is negligible.
const CHUNK_SIZE: u32 = 2048;

const HAND_SIZE: usize = 2;

/// Equity (in %) of `hand` against `num_opponents` random hands. Split pots
/// count as `1/k` of a win.
///
/// When the number of distinct deals (board completions x opponent hands) is
/// not larger than `iterations`, every deal is enumerated and the result is
/// exact (e.g. turn or river vs one opponent). Otherwise `iterations` random
/// deals are simulated (Monte Carlo).
pub fn calculate_equity(hand: Hand, community_cards: CommunityCards, num_opponents: u8, iterations: u32) -> Result<f64, String> {
    if num_opponents == 0 {
        return Ok(100.0);
    }
    if iterations == 0 {
        return Err("Iterations must be greater than 0".to_string());
    }

    let board = community_cards.get_cards();
    if board.len() > 5 {
        return Err(format!("Too many community cards: {}", board.len()));
    }

    let mut hero_mask: CardMask = 0;
    let mut board_mask: CardMask = 0;
    let mut used: CardMask = 0;
    for (position, card) in hand.get_cards().iter().chain(board.iter()).enumerate() {
        let bit = card_bit(card).ok_or_else(|| format!("Invalid card: {:?}", card))?;
        let card_flag = 1u64 << bit;
        if used & card_flag != 0 {
            return Err(format!("Duplicate card: {:?}", card));
        }
        used |= card_flag;
        let is_hero_card = position < HAND_SIZE;
        if is_hero_card { hero_mask |= card_flag } else { board_mask |= card_flag }
    }

    let deck: Vec<u8> = (0..64u8)
        .filter(|&bit| bit % 16 < 13 && used & (1u64 << bit) == 0)
        .collect();

    let board_missing = 5 - board.len();
    let needed = board_missing + HAND_SIZE * num_opponents as usize;
    if needed > deck.len() {
        return Err(format!("Not enough cards left for {} opponents", num_opponents));
    }

    let sim = Simulation { deck, hero_mask, board_mask, board_missing, opponents: num_opponents as usize };

    let total_deals = exact_deal_count(sim.deck.len(), board_missing, sim.opponents);
    if total_deals <= iterations as f64 {
        return Ok(sim.exact() / total_deals * 100.0);
    }

    let chunks = iterations.div_ceil(CHUNK_SIZE);
    let base_seed: u64 = rand::thread_rng().gen();
    let won: f64 = (0..chunks)
        .into_par_iter()
        .map(|chunk| {
            let chunk_iterations = CHUNK_SIZE.min(iterations - chunk * CHUNK_SIZE);
            let rng = SmallRng::seed_from_u64(base_seed.wrapping_add(chunk as u64));
            sim.run(chunk_iterations, rng)
        })
        .sum();

    Ok(won / iterations as f64 * 100.0)
}

struct Simulation {
    deck: Vec<u8>,
    hero_mask: CardMask,
    board_mask: CardMask,
    board_missing: usize,
    opponents: usize,
}

/// Number of distinct deals: ways to complete the board from `deck_len` cards
/// times the ways to give `opponents` (indistinguishable) players two cards each.
fn exact_deal_count(deck_len: usize, board_missing: usize, opponents: usize) -> f64 {
    fn choose(n: usize, r: usize) -> f64 {
        if r > n {
            return 0.0;
        }
        (0..r).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
    }
    let cards_after_board = deck_len - board_missing;
    let opponent_cards = HAND_SIZE * opponents;
    let pairings_of_opponent_cards: f64 = (1..=opponents).map(|i| (2 * i - 1) as f64).product();
    (choose(deck_len, board_missing) * choose(cards_after_board, opponent_cards) * pairings_of_opponent_cards).round()
}

/// All ways to pick `r` cards from `deck[start..]`, as masks.
fn combinations(deck: &[u8], start: usize, r: usize, acc: CardMask, out: &mut Vec<CardMask>) {
    if r == 0 {
        out.push(acc);
        return;
    }
    for i in start..=deck.len() - r {
        combinations(deck, i + 1, r - 1, acc | (1u64 << deck[i]), out);
    }
}

/// Partial Fisher-Yates shuffle: moves `count` random cards to the front of
/// `deck`. The slice stays a permutation of the same cards, so it can be
/// reused across iterations without being reset.
#[inline]
fn shuffle_front(deck: &mut [u8], count: usize, rng: &mut SmallRng) {
    let len = deck.len();
    for i in 0..count {
        let j = rng.gen_range(i..len);
        deck.swap(i, j);
    }
}

impl Simulation {
    /// Enumerates every deal and returns the (fractional) number of pots won.
    /// Divide by [`exact_deal_count`] to obtain the exact equity.
    fn exact(&self) -> f64 {
        let mut board_completions = Vec::new();
        combinations(&self.deck, 0, self.board_missing, 0, &mut board_completions);

        board_completions
            .par_iter()
            .map(|&extra| {
                let board = self.board_mask | extra;
                let hero_strength = evaluate(board | self.hero_mask);
                let available: Vec<u8> = self.deck.iter().copied().filter(|&bit| extra & (1u64 << bit) == 0).collect();
                won_in_opponent_deals(&available, 0, 0, self.opponents, board, hero_strength, 0)
            })
            .sum()
    }

    /// Runs `iterations` random deals and returns the (fractional) number of pots won.
    fn run(&self, iterations: u32, mut rng: SmallRng) -> f64 {
        let mut deck = self.deck.clone();
        let needed = self.board_missing + HAND_SIZE * self.opponents;
        let mut won = 0.0;

        for _ in 0..iterations {
            shuffle_front(&mut deck, needed, &mut rng);

            let mut board = self.board_mask;
            for &bit in &deck[..self.board_missing] {
                board |= 1u64 << bit;
            }

            let hero_strength = evaluate(board | self.hero_mask);
            let mut ties = 0u32;
            let mut lost = false;
            for opponent_cards in deck[self.board_missing..needed].chunks_exact(HAND_SIZE) {
                let opponent_strength = evaluate(board | (1u64 << opponent_cards[0]) | (1u64 << opponent_cards[1]));
                if opponent_strength > hero_strength {
                    lost = true;
                    break;
                }
                ties += (opponent_strength == hero_strength) as u32;
            }

            if !lost {
                won += 1.0 / (ties + 1) as f64;
            }
        }

        won
    }
}

/// Sums the hero's pot share over every unordered set of `opponents_left`
/// two-card hands drawn from `available` (indices not in `used`). Each set is
/// visited once by requiring the opponents' lowest cards to be increasing.
/// Branches where an opponent already beats the hero are pruned: they all
/// contribute 0.
fn won_in_opponent_deals(
    available: &[u8],
    start: usize,
    used: u64,
    opponents_left: usize,
    board: CardMask,
    hero_strength: u32,
    ties: u32,
) -> f64 {
    if opponents_left == 0 {
        return 1.0 / (ties + 1) as f64;
    }
    let mut won = 0.0;
    for first in start..available.len() {
        if used & (1u64 << first) != 0 {
            continue;
        }
        for second in (first + 1)..available.len() {
            if used & (1u64 << second) != 0 {
                continue;
            }
            let opponent_strength = evaluate(board | (1u64 << available[first]) | (1u64 << available[second]));
            if opponent_strength > hero_strength {
                continue;
            }
            won += won_in_opponent_deals(
                available,
                first + 1,
                used | (1u64 << first) | (1u64 << second),
                opponents_left - 1,
                board,
                hero_strength,
                ties + (opponent_strength == hero_strength) as u32,
            );
        }
    }
    won
}

pub fn calculate_pot_odds(pot_size: f64, call_amount: f64, win_percentage: f64) -> f64 {
    (win_percentage * (pot_size + call_amount)) / 100.0
}


#[cfg(test)]
mod tests {
    use crate::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
    use super::*;

    const FULL_DECK: usize = 52;
    const RIVER_DECK: usize = FULL_DECK - HAND_SIZE - 5;
    const TURN_DECK: usize = RIVER_DECK + 1;

    fn equity(cards: [Card; 2], board: Vec<Card>, opponents: u8) -> f64 {
        calculate_equity(Hand::new(cards), CommunityCards::new(board), opponents, 200_000).unwrap()
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64, label: &str) {
        assert!((actual - expected).abs() < tolerance, "{label}: expected {expected}, got {actual}");
    }

    #[test]
    fn preflop_equity_matches_known_values() {
        let aces = equity([Card::new(SPADES, 14), Card::new(HEARTS, 14)], vec![], 1);
        assert_close(aces, 85.2, 1.0, "AA");
        let ace_king_offsuit = equity([Card::new(SPADES, 14), Card::new(HEARTS, 13)], vec![], 1);
        assert_close(ace_king_offsuit, 65.4, 1.0, "AKo");
        let seven_deuce_offsuit = equity([Card::new(SPADES, 7), Card::new(HEARTS, 2)], vec![], 1);
        assert_close(seven_deuce_offsuit, 34.6, 1.0, "72o");
    }

    #[test]
    fn royal_flush_on_board_splits_the_pot_between_everyone() {
        let royal_flush_board = vec![
            Card::new(CLUBS, 10), Card::new(CLUBS, 11), Card::new(CLUBS, 12),
            Card::new(CLUBS, 13), Card::new(CLUBS, 14),
        ];
        let opponents = 3;
        let hero_equity = equity([Card::new(SPADES, 2), Card::new(HEARTS, 3)], royal_flush_board, opponents);
        let even_split = 100.0 / (opponents as f64 + 1.0);
        assert_close(hero_equity, even_split, 1e-9, "split equity");
    }

    #[test]
    fn deal_count() {
        let river_opponent_hands = 990.0;
        assert_eq!(exact_deal_count(RIVER_DECK, 0, 1), river_opponent_hands);
        assert_eq!(exact_deal_count(TURN_DECK, 1, 1), TURN_DECK as f64 * river_opponent_hands);
        assert_eq!(exact_deal_count(RIVER_DECK, 0, 2), river_opponent_hands * 903.0 / 2.0);
        let ways_to_split_six_cards_into_three_pairs = 15.0;
        assert_eq!(exact_deal_count(6, 0, 3), ways_to_split_six_cards_into_three_pairs);
    }

    #[test]
    fn exact_matches_naive_enumeration() {
        let hero = [Card::new(SPADES, 14), Card::new(HEARTS, 13)];
        let board = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10), Card::new(SPADES, 13)];
        let exact = calculate_equity(Hand::new(hero), CommunityCards::new(board.clone()), 1, 50_000).unwrap();

        let mask = |cards: &[Card]| cards.iter().fold(0u64, |acc, card| acc | 1u64 << card_bit(card).unwrap());
        let used = mask(&hero) | mask(&board);
        let deck: Vec<u64> = (0..64).filter(|bit| bit % 16 < 13 && used & (1 << bit) == 0).map(|bit| 1u64 << bit).collect();
        let (mut won, mut total) = (0.0, 0.0);
        for &river in &deck {
            let full_board = mask(&board) | river;
            let hero_strength = evaluate(full_board | mask(&hero));
            for &first in &deck {
                for &second in &deck {
                    if first == river || second == river || first >= second {
                        continue;
                    }
                    let opponent_strength = evaluate(full_board | first | second);
                    total += 1.0;
                    won += if hero_strength > opponent_strength { 1.0 } else if hero_strength == opponent_strength { 0.5 } else { 0.0 };
                }
            }
        }
        assert_close(exact, won / total * 100.0, 1e-9, "exact vs naive");

        let repeated = calculate_equity(Hand::new(hero), CommunityCards::new(board), 1, 50_000).unwrap();
        assert_eq!(exact, repeated);
    }

    #[test]
    fn exact_with_several_opponents_matches_simulation() {
        let hero = [Card::new(SPADES, 12), Card::new(HEARTS, 12)];
        let board = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10), Card::new(SPADES, 13), Card::new(CLUBS, 4)];
        let river_deals_vs_two_opponents = exact_deal_count(RIVER_DECK, 0, 2) as u32;
        let exact = calculate_equity(Hand::new(hero), CommunityCards::new(board.clone()), 2, river_deals_vs_two_opponents).unwrap();
        let simulated = calculate_equity(Hand::new(hero), CommunityCards::new(board), 2, river_deals_vs_two_opponents - 1).unwrap();
        assert_close(exact, simulated, 0.5, "exact vs simulated");
    }

    #[test]
    fn rejects_invalid_input() {
        let hand = Hand::new([Card::new(SPADES, 14), Card::new(SPADES, 14)]);
        assert!(calculate_equity(hand, CommunityCards::empty(), 1, 100).is_err());

        let hand = Hand::new([Card::new(SPADES, 14), Card::new(HEARTS, 14)]);
        assert!(calculate_equity(hand.clone(), CommunityCards::empty(), 1, 0).is_err());
        assert!(calculate_equity(hand.clone(), CommunityCards::empty(), 30, 100).is_err());
        let bad = Hand::new([Card::new('x', 14), Card::new(HEARTS, 1)]);
        assert!(calculate_equity(bad, CommunityCards::empty(), 1, 100).is_err());

        let max_opponents = ((FULL_DECK - HAND_SIZE - 5) / HAND_SIZE) as u8;
        assert!(calculate_equity(hand, CommunityCards::empty(), max_opponents, 100).is_ok());
    }

    #[test]
    fn pot_odd_zero_probability(){
        let pot_size= 100.0;
        let call_amount= 10.0;
        let win_percentage = 0.0;
        assert_eq!( calculate_pot_odds(pot_size,call_amount,win_percentage), 0.0 );
    }

    #[test]
    fn pot_odd_100_probability(){
        let pot_size= 100.0;
        let call_amount= 30.0;
        let win_percentage = 100.0;
        assert!( calculate_pot_odds(pot_size,call_amount,win_percentage) > call_amount );
    }

    #[test]
    fn calculate_equitity_always_in_the_range(){
        let hand: Hand = Hand::new([Card::new(DIAMONDS, 7), Card::new(SPADES, 2)]);
        let num_opponents = 3;

        let result = calculate_equity(hand, CommunityCards::empty(), num_opponents, 100 );
        assert!(result.is_ok());

        let prob_victory = result.unwrap();
        assert!( 100.0 >= prob_victory && prob_victory >=0.0 );
    }

    #[test]
    fn calculate_equity_always_win(){
        let pocket_aces: Hand = Hand::new([Card::new(DIAMONDS, 14), Card::new(SPADES, 14)]);
        let board_with_other_aces = CommunityCards::new_array([
            Card::new(CLUBS, 14),
            Card::new(HEARTS, 14),
            Card::new(HEARTS, 3),
            Card::new(DIAMONDS, 7),
            Card::new(SPADES, 5),
        ]);

        let num_opponents = 3;
        let result = calculate_equity(pocket_aces, board_with_other_aces, num_opponents, 100 );
        assert!(result.is_ok());

        let prob_victory = result.unwrap();
        assert!( 100.0 == prob_victory );
    }
}

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
    for (i, card) in hand.get_cards().iter().chain(board.iter()).enumerate() {
        let bit = card_bit(card).ok_or_else(|| format!("Invalid card: {:?}", card))?;
        let b = 1u64 << bit;
        if used & b != 0 {
            return Err(format!("Duplicate card: {:?}", card));
        }
        used |= b;
        if i < 2 { hero_mask |= b } else { board_mask |= b }
    }

    let deck: Vec<u8> = (0..64u8)
        .filter(|&bit| bit % 16 < 13 && used & (1u64 << bit) == 0)
        .collect();

    let board_missing = 5 - board.len();
    let needed = board_missing + 2 * num_opponents as usize;
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
            let n = CHUNK_SIZE.min(iterations - chunk * CHUNK_SIZE);
            let rng = SmallRng::seed_from_u64(base_seed.wrapping_add(chunk as u64));
            sim.run(n, rng)
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

/// Number of distinct deals: ways to complete the board from `n` cards times
/// the ways to give `k` (indistinguishable) opponents two cards each.
fn exact_deal_count(n: usize, board_missing: usize, k: usize) -> f64 {
    fn choose(n: usize, r: usize) -> f64 {
        if r > n {
            return 0.0;
        }
        (0..r).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
    }
    // Unordered sets of k disjoint pairs out of m cards: C(m, 2k) * (2k - 1)!!
    let m = n - board_missing;
    let double_factorial: f64 = (1..=k).map(|i| (2 * i - 1) as f64).product();
    (choose(n, board_missing) * choose(m, 2 * k) * double_factorial).round()
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

impl Simulation {
    /// Enumerates every deal and returns the (fractional) number of pots won.
    /// Divide by [`exact_deal_count`] to obtain the exact equity.
    fn exact(&self) -> f64 {
        let mut boards = Vec::new();
        combinations(&self.deck, 0, self.board_missing, 0, &mut boards);

        boards
            .par_iter()
            .map(|&extra| {
                let board = self.board_mask | extra;
                let hero = evaluate(board | self.hero_mask);
                let available: Vec<u8> = self.deck.iter().copied().filter(|&bit| extra & (1u64 << bit) == 0).collect();
                won_in_opponent_deals(&available, 0, 0, self.opponents, board, hero, 0)
            })
            .sum()
    }

    /// Runs `iterations` random deals and returns the (fractional) number of pots won.
    fn run(&self, iterations: u32, mut rng: SmallRng) -> f64 {
        let mut deck = self.deck.clone();
        let len = deck.len();
        let needed = self.board_missing + 2 * self.opponents;
        let mut won = 0.0;

        for _ in 0..iterations {
            // Partial Fisher-Yates: only the first `needed` positions are drawn.
            // The array stays a permutation of the remaining cards, so it never
            // needs to be reset between iterations.
            for i in 0..needed {
                let j = rng.gen_range(i..len);
                deck.swap(i, j);
            }

            let mut board = self.board_mask;
            for &bit in &deck[..self.board_missing] {
                board |= 1u64 << bit;
            }

            let hero = evaluate(board | self.hero_mask);
            let mut ties = 0u32;
            let mut lost = false;
            for opp in deck[self.board_missing..needed].chunks_exact(2) {
                let value = evaluate(board | (1u64 << opp[0]) | (1u64 << opp[1]));
                if value > hero {
                    lost = true;
                    break;
                }
                ties += (value == hero) as u32;
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
    hero: u32,
    ties: u32,
) -> f64 {
    if opponents_left == 0 {
        return 1.0 / (ties + 1) as f64;
    }
    let mut won = 0.0;
    for a in start..available.len() {
        if used & (1u64 << a) != 0 {
            continue;
        }
        for b in (a + 1)..available.len() {
            if used & (1u64 << b) != 0 {
                continue;
            }
            let value = evaluate(board | (1u64 << available[a]) | (1u64 << available[b]));
            if value > hero {
                continue;
            }
            won += won_in_opponent_deals(
                available,
                a + 1,
                used | (1u64 << a) | (1u64 << b),
                opponents_left - 1,
                board,
                hero,
                ties + (value == hero) as u32,
            );
        }
    }
    won
}

pub fn calculate_pot_odds(pot_size: f64, call_amount: f64, ptg_win: f64) -> f64 {
    // TODO: Implementar evaluación de manos, es dudosa la actual
    (ptg_win * (pot_size + call_amount)) / 100.0
}


#[cfg(test)]
mod tests {
    use crate::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
    use super::*;

    fn equity(cards: [Card; 2], board: Vec<Card>, opponents: u8) -> f64 {
        calculate_equity(Hand::new(cards), CommunityCards::new(board), opponents, 200_000).unwrap()
    }

    #[test]
    fn preflop_equity_matches_known_values() {
        // Reference heads-up equities vs a random hand: AA 85.2%, AKo 65.4%, 72o 34.6%
        let aa = equity([Card::new(SPADES, 14), Card::new(HEARTS, 14)], vec![], 1);
        assert!((aa - 85.2).abs() < 1.0, "AA = {aa}");
        let ako = equity([Card::new(SPADES, 14), Card::new(HEARTS, 13)], vec![], 1);
        assert!((ako - 65.4).abs() < 1.0, "AKo = {ako}");
        let sevtwo = equity([Card::new(SPADES, 7), Card::new(HEARTS, 2)], vec![], 1);
        assert!((sevtwo - 34.6).abs() < 1.0, "72o = {sevtwo}");
    }

    #[test]
    fn board_plays_splits_the_pot() {
        // Royal flush on the board: everybody splits, equity = 1/(opponents+1)
        let board = vec![
            Card::new(CLUBS, 10), Card::new(CLUBS, 11), Card::new(CLUBS, 12),
            Card::new(CLUBS, 13), Card::new(CLUBS, 14),
        ];
        let e = equity([Card::new(SPADES, 2), Card::new(HEARTS, 3)], board, 3);
        assert!((e - 25.0).abs() < 1e-9, "equity = {e}");
    }

    #[test]
    fn deal_count() {
        assert_eq!(exact_deal_count(45, 0, 1), 990.0); // river, 1 opponent
        assert_eq!(exact_deal_count(46, 1, 1), 46.0 * 990.0); // turn, 1 opponent
        assert_eq!(exact_deal_count(45, 0, 2), 990.0 * 903.0 / 2.0); // river, 2 opponents
        assert_eq!(exact_deal_count(6, 0, 3), 15.0); // 6 cards into 3 pairs
    }

    /// Exact enumeration must match a naive ordered double loop.
    #[test]
    fn exact_matches_naive_enumeration() {
        let hero = [Card::new(SPADES, 14), Card::new(HEARTS, 13)];
        let board = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10), Card::new(SPADES, 13)];
        let exact = calculate_equity(Hand::new(hero), CommunityCards::new(board.clone()), 1, 50_000).unwrap();

        let mask = |cs: &[Card]| cs.iter().fold(0u64, |m, c| m | 1u64 << card_bit(c).unwrap());
        let used = mask(&hero) | mask(&board);
        let deck: Vec<u64> = (0..64).filter(|b| b % 16 < 13 && used & (1 << b) == 0).map(|b| 1u64 << b).collect();
        let (mut won, mut total) = (0.0, 0.0);
        for &river in &deck {
            let full = mask(&board) | river;
            let h = evaluate(full | mask(&hero));
            for &x in &deck {
                for &y in &deck {
                    if x == river || y == river || x >= y {
                        continue;
                    }
                    let o = evaluate(full | x | y);
                    total += 1.0;
                    won += if h > o { 1.0 } else if h == o { 0.5 } else { 0.0 };
                }
            }
        }
        assert!((exact - won / total * 100.0).abs() < 1e-9);
        // Deterministic: no randomness involved
        assert_eq!(exact, calculate_equity(Hand::new(hero), CommunityCards::new(board), 1, 50_000).unwrap());
    }

    #[test]
    fn exact_with_several_opponents_matches_simulation() {
        // River, 2 opponents: 446,985 deals -> exact with a large budget
        let hero = [Card::new(SPADES, 12), Card::new(HEARTS, 12)];
        let board = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10), Card::new(SPADES, 13), Card::new(CLUBS, 4)];
        let exact = calculate_equity(Hand::new(hero), CommunityCards::new(board.clone()), 2, 500_000).unwrap();
        let simulated = calculate_equity(Hand::new(hero), CommunityCards::new(board), 2, 446_984).unwrap();
        assert!((exact - simulated).abs() < 0.5, "exact {exact} vs simulated {simulated}");
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
        // Maximum table size still works (2 + 5 + 22*2 = 51 cards)
        assert!(calculate_equity(hand, CommunityCards::empty(), 22, 100).is_ok());
    }

    #[test]
    fn pot_odd_zero_probability(){
        let pot_size= 100.0;
        let call_amount= 10.0;
        let ptg_win = 0.0;
        assert_eq!( calculate_pot_odds(pot_size,call_amount,ptg_win), 0.0 );
    }

    #[test]
    fn pot_odd_100_probability(){
        let pot_size= 100.0;
        let call_amount= 30.0;
        let ptg_win = 100.0;
        assert!( calculate_pot_odds(pot_size,call_amount,ptg_win) > call_amount );
    }

    #[test]
    fn calculate_equitity_always_in_the_range(){

        // Hand cards
        let card1:Card = Card::new(DIAMONDS, 7);
        let card2: Card = Card::new(SPADES, 2);

        // Hand
        let hand: Hand = Hand::new([card1,card2]);

        let num_opponents = 3;

        let result = calculate_equity(hand, CommunityCards::empty(), num_opponents, 100 );
        assert!(result.is_ok());

        let prob_victory = result.unwrap();
        assert!( 100.0 >= prob_victory && prob_victory >=0.0 );
    }

    #[test]
    fn calculate_equity_always_win(){
        // Hand cards
        let card1: Card = Card::new(DIAMONDS, 14);
        let card2: Card = Card::new(SPADES, 14);

        // Hand
        let hand: Hand = Hand::new([card1,card2]);

        // Community cards cards
        let card3: Card = Card::new(CLUBS, 14);
        let card4: Card = Card::new(HEARTS, 14);
        let card5: Card = Card::new(HEARTS, 3);
        let card6: Card = Card::new(DIAMONDS, 7);
        let card7: Card = Card::new(SPADES, 5);

        let community_cards = CommunityCards::new_array([card3,card4,card5,card6,card7]);

        let num_opponents = 3;
        let result = calculate_equity(hand, community_cards, num_opponents, 100 );
        assert!(result.is_ok());

        let prob_victory = result.unwrap();
        assert!( 100.0 == prob_victory );
    }
}
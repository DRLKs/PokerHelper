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

/// Monte Carlo equity (in %) of `hand` against `num_opponents` random hands,
/// completing the board on every iteration. Split pots count as `1/k` of a win.
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

impl Simulation {
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
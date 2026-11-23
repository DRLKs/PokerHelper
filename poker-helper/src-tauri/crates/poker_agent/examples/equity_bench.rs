//! Quick throughput check for the Monte Carlo equity simulation.
//! Run with: `cargo run --release --example equity_bench`
//! (set `RAYON_NUM_THREADS=1` to measure single-core performance, e.g. to
//! approximate a low-end phone).
use poker_agent::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::core::hand::Hand;
use poker_agent::probability::calculate_equity;
use std::time::Instant;

fn main() {
    let hand = Hand::new([Card::new(DIAMONDS, 14), Card::new(SPADES, 13)]);
    let flop = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10)];
    let runs = 50;

    for (street, board) in [("preflop", vec![]), ("flop", flop)] {
        for opponents in [1u8, 5] {
            for iterations in [10_000u32, 100_000] {
                let board = CommunityCards::new(board.clone());
                // Warm-up (spins up the rayon thread pool)
                calculate_equity(hand.clone(), board.clone(), opponents, iterations).unwrap();
                let start = Instant::now();
                let mut equity = 0.0;
                for _ in 0..runs {
                    equity = calculate_equity(hand.clone(), board.clone(), opponents, iterations).unwrap();
                }
                println!(
                    "{street:<8} opponents={opponents} iterations={iterations:<7} {:>10.3?}/call  equity={equity:.2}%",
                    start.elapsed() / runs
                );
            }
        }
    }
}

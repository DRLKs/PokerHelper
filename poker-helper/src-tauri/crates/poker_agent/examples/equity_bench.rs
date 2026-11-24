//! Quick throughput check for the Monte Carlo equity simulation.
//! Run with: `cargo run --release --example equity_bench`
//! (set `RAYON_NUM_THREADS=1` to measure single-core performance, e.g. to
//! approximate a low-end phone).
use poker_agent::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::core::hand::Hand;
use poker_agent::probability::calculate_equity;
use poker_agent::range::{equity_vs_ranges, Range};
use std::time::Instant;

fn main() {
    let hand = Hand::new([Card::new(DIAMONDS, 14), Card::new(SPADES, 13)]);
    let flop = vec![Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10)];
    let mut turn = flop.clone();
    turn.push(Card::new(SPADES, 4));
    let mut river = turn.clone();
    river.push(Card::new(HEARTS, 13));
    let runs = 50;

    for (street, board) in [("preflop", vec![]), ("flop", flop), ("turn", turn), ("river", river)] {
        for opponents in [1u8, 5] {
            for iterations in [50_000u32] {
                let board = CommunityCards::new(board.clone());
                let _warm_up_thread_pool = calculate_equity(hand.clone(), board.clone(), opponents, iterations).unwrap();
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

    let opening_range: Range = "22+, A2s+, K9s+, Q9s+, J9s+, T9s, 98s, 87s, ATo+, KJo+".parse().unwrap();
    let hero = [Card::new(DIAMONDS, 14), Card::new(SPADES, 13)];
    let flop = [Card::new(HEARTS, 2), Card::new(CLUBS, 9), Card::new(DIAMONDS, 10)];
    for (street, board) in [("preflop", &flop[..0]), ("flop", &flop[..])] {
        for opponents in [1usize, 3] {
            let ranges = vec![opening_range.clone(); opponents];
            let _warm_up_thread_pool = equity_vs_ranges(hero, board, &ranges, 20_000).unwrap();
            let start = Instant::now();
            let mut equity = 0.0;
            for _ in 0..runs {
                equity = equity_vs_ranges(hero, board, &ranges, 20_000).unwrap().percent();
            }
            println!(
                "range {street:<8} opponents={opponents} iterations=20000   {:>10.3?}/call  equity={equity:.2}%",
                start.elapsed() / runs
            );
        }
    }
}

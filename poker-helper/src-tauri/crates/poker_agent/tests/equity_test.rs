use poker_agent::core::card::{Card, DIAMONDS, SPADES};
use poker_agent::core::hand::Hand;
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::probability::calculate_equity;

#[test]
fn test_equity_calculation_integration() {
    let pocket_aces = Hand::new([Card::new(DIAMONDS, 14), Card::new(SPADES, 14)]);
    let preflop = CommunityCards::empty();

    let result = calculate_equity(pocket_aces, preflop, 1, 1000);

    assert!(result.is_ok());
    let equity = result.unwrap();

    println!("Equity for AA pre-flop vs 1 opponent: {}%", equity);
    assert!(equity > 70.0);
    assert!(equity <= 100.0);
}

#[test]
fn bad_cards_test() {
    let hand = Hand::new([Card::new(DIAMONDS, 2), Card::new(SPADES, 7)]);
    let community = CommunityCards::new_array([
        Card::new(DIAMONDS, 14),
        Card::new(DIAMONDS, 13),
        Card::new(DIAMONDS, 12),
        Card::new(SPADES, 10),
        Card::new(SPADES, 8),
    ]);

    let result = calculate_equity(hand, community, 4, 1000);

    assert!(result.is_ok());
    let equity = result.unwrap();

    println!("Equity for very bad cards vs 4 opponent: {}%", equity);
    assert!(equity < 40.0);
    assert!(equity >= 0.0);
}

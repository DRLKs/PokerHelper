use poker_agent::core::card::{Card, DIAMONDS, SPADES};
use poker_agent::core::hand::Hand;
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::probability::calculate_equity;

#[test]
fn test_equity_calculation_integration() {
    // Setup a hand: Pair of Aces
    let card1 = Card::new(DIAMONDS, 14); // Ace of Diamonds
    let card2 = Card::new(SPADES, 14); // Ace of Spades
    let hand = Hand::new([card1, card2]);

    // Setup community cards (empty for pre-flop)
    let community = CommunityCards::empty();

    // Calculate equity against 1 opponent with 1000 iterations
    let result = calculate_equity(hand, community, 1, 1000);
    
    assert!(result.is_ok());
    let equity = result.unwrap();
    
    // Aces pre-flop against 1 random hand should have high equity (usually around 85%)
    println!("Equity for AA pre-flop vs 1 opponent: {}%", equity);
    assert!(equity > 70.0);
    assert!(equity <= 100.0);
}

#[test]
fn bad_cards_test() {

    // My hand
    let card1 = Card::new(DIAMONDS, 2);
    let card2 = Card::new(SPADES, 7);
    let hand = Hand::new([card1, card2]);

    // Community cards
    let card3 = Card::new(DIAMONDS, 14);
    let card4 = Card::new(DIAMONDS, 13);
    let card5 = Card::new(DIAMONDS, 12);
    let card6 = Card::new(SPADES, 10);
    let card7 = Card::new(SPADES, 8);

    let community = CommunityCards::new_array([card3,card4,card5,card6,card7]);

    let result = calculate_equity(hand, community, 4, 1000);

    assert!(result.is_ok());
    let equity = result.unwrap();

    println!("Equity for very bad cards vs 4 opponent: {}%", equity);
    assert!(equity < 40.0);
    assert!(equity >= 0.0);
}

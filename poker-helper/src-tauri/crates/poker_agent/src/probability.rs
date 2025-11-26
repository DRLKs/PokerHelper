use crate::core::card::Card;
use crate::core::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::core::deck::Deck;
use crate::core::hand::Hand;
use crate::evaluator::hand_analysis::HandAnalysis;

pub fn calculate_equity(hand: Hand, community_cards: CommunityCards, num_opponents: u8, iterations: u16) -> Result<f64, String> {
    if num_opponents <= 0 {
        return Ok(100.0);
    }
    let mut cards_excluded: Vec<Card> = Vec::new();
    (&mut cards_excluded).extend_from_slice(hand.get_cards());
    (&mut cards_excluded).extend_from_slice(community_cards.get_cards());

    let base_deck: Deck = Deck::new_excluding(&cards_excluded);

    let mut games_won: u32 = 0;
    for _ in 0..iterations {
        let mut deck = base_deck.clone();
        deck.shuffle();

        if am_i_the_winner(&hand, &community_cards, deck.give_n_hands(num_opponents)) {
            games_won += 1;
        }
    }

    let ptg_victory: f64 = games_won as f64 / iterations as f64;

    Ok(ptg_victory * 100.0)
}


pub fn calculate_pot_odds(pot_size: f64, call_amount: f64, ptg_win: f64) -> f64 {
    // TODO: Implementar evaluación de manos, es dudosa la actual
    (ptg_win * (pot_size + call_amount)) / 100.0
}

fn am_i_the_winner(my_hand: &Hand, community_cards: &CommunityCards, opponents: Vec<Hand>) -> bool {
    let my_analysis = HandAnalysis::analyse(my_hand, community_cards);

    for opponent_hand in opponents {
        let opponent_analysis = HandAnalysis::analyse(&opponent_hand, community_cards);

        if opponent_analysis.is_better(&my_analysis) {
            return false;
        }
    }

    true
}


#[cfg(test)]
mod tests {
    use crate::core::card::{CLUBS, DIAMONDS, HEARTS, SPADES};
    use super::*;

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
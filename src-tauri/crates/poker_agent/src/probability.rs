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

    Ok(ptg_victory)
}

pub fn calculate_pot_odds(pot_size: f64, call_amount: f64) -> f64 {
    // TODO: Implementar evaluación de manos
    0.0
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

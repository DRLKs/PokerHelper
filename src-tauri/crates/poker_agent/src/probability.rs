
use crate::utils::card::{Card, CardTrait};
use crate::utils::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::utils::hand::{Hand, HandTrait};
use crate::utils::deck::Deck;
use crate::utils::hand_analysis::HandAnalysis;

use crate::game_state::SetOfHands;
pub fn calculate_equity(hand: Hand, community_cards: CommunityCards, num_opponents: u8, iterations: u32) -> Result<f64, String> {
    
    if num_opponents <= 0  {
        return Ok(100.0);
    }
    
    let mut cards_excluded: Vec<Card> = Vec::new();
    (&mut cards_excluded).extend_from_slice( hand.get_cards() );
    (&mut cards_excluded).extend_from_slice( community_cards.get_cards() );
    
    let mut deck: Deck = Deck::new_excluding(&cards_excluded);

    let mut games_won: u32 = 0;
    for _ in 0..iterations {

        (&mut deck).shuffle();

        if determine_winner(hand.clone(), community_cards.clone(), deck.give_n_hands(num_opponents)){
            games_won += 1;
        }
    }

    let ptg_victory: f64 = iterations as f64 / games_won as f64;

    Ok(ptg_victory)
}

pub fn calculate_pot_odds(pot_size: f64, call_amount: f64) -> f64 {
    // TODO: Implementar evaluación de manos
    0.0
}

fn determine_winner(my_hand: Hand, community_cards: CommunityCards, opponents: Vec<Hand>) -> bool {
    // TODO: Implementar evaluación de manos
    
    
    
    
    false
}

fn have_pair(hand: Hand, community_cards: CommunityCards) -> bool{
    if hand.is_pair() {
        return true;
    } else{
        let cards = hand.get_cards();
        for c  in community_cards.get_cards() {
            if cards[0].get_rank() == c.get_rank() || cards[1].get_rank() == c.get_rank() {
                return true;
            }
        }   
    }
    false
}

fn get_set_of_hands(hand: Hand, community_cards: CommunityCards) -> HandAnalysis {
    
    let mut hand_analysis = HandAnalysis::new();

    if hand.is_pair(){
        get_set_of_hands_are_pair(hand.get_cards()[0], hand.get_cards()[1].get_suit(), community_cards, &mut hand_analysis);
    }else if hand.is_same_suit(){
        ctt_first_card_suits = 2;
        ctt_second_card_suits = 0;
    }else{

    }
    

    
    hand_analysis
}

fn get_set_of_hands_are_pair(card: Card, suit_second_card: char, community_cards: CommunityCards, hand_analysis: &mut HandAnalysis) {

    hand_analysis.add( SetOfHands::Pair, card.get_rank() );

    let mut ctt_first_card_suits: u8 = 1;
    let mut ctt_second_card_suits: u8 = 1;
    let mut ctt_same_rank: u8 = 2;

    for community_card in community_cards.get_cards() {

        if card.same_rank(community_card) {
            ctt_same_rank += 1;
        }

        if card.same_suit(community_card) {
            ctt_first_card_suits += 1;
        }else if community_card.same_suit_by_char(suit_second_card) {
            ctt_second_card_suits += 1;
        }

    }

    if ctt_same_rank >= 3 {
        hand_analysis.add( SetOfHands::ThreeOfAKind, card.get_rank() );
    }

    if ctt_first_card_suits >= 4 {
        hand_analysis.add( SetOfHands::FourOfAKind, card.get_rank() );
    }
}

fn get_set_of_hands_are_same_suit(card: Card, community_cards: CommunityCards, hand_analysis: &mut HandAnalysis) {

    hand_analysis.add( SetOfHands::Pair, card.get_rank() );

    for card in community_cards.get_cards() {


    }
}
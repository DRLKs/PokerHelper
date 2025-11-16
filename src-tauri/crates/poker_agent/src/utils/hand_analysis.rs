use crate::utils::card::{Card, CardTrait, HIGHEST_RANK};
use crate::utils::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::utils::hand::{Hand, HandTrait, };
use serde::Serialize;
use std::collections::HashMap;

const NECESSARY_CARDS_FLUSH: u8 = 5;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetOfHands {
    Pair,         // Pareja
    TwoPair,      // 2 Parejas
    ThreeOfAKind, // Trio
    Straight,
    Flush, // Color
    FullHouse,
    FourOfAKind,
    StraightFLush,
    RoyalStraight,
}

#[derive(Debug, Clone)]
pub struct HandAnalysis {
    high_card: u8,
    sets: HashMap<SetOfHands, u8>,
}

impl HandAnalysis {
    pub fn new() -> Self {
        Self {
            high_card: 0,
            sets: HashMap::new(),
        }
    }

    pub fn analyse(hand: &Hand, community_cards: &CommunityCards) -> Self {
        let mut analysis = Self::new();

        if hand.are_pair() {
            analyse_pair_hand(hand, community_cards, &mut analysis);
        } else if hand.is_same_suit() {
        }

        analysis
    }

    pub fn set_high_card(&mut self, high_card: u8) {
        self.high_card = high_card;
    }

    pub fn add(&mut self, hand_type: SetOfHands, high_card_of_the_set: u8) {
        self.sets.insert(hand_type, high_card_of_the_set);
    }

    pub fn has_set(&self, hand_type: &SetOfHands) -> bool {
        self.sets.contains_key(hand_type)
    }

    pub fn get_high_card(&self) -> u8 {
        self.high_card
    }

    pub fn get_sets(&self) -> &HashMap<SetOfHands, u8> {
        &self.sets
    }

    pub fn get_highest_card_set(&self, set: &SetOfHands) -> u8 {
        if self.has_set(set) {
            return self.sets[set]
        }
        0   // There isn't 
    }

    pub fn is_better(&self, other: &HandAnalysis) -> bool {
        
        if other.has_set(&SetOfHands::RoyalStraight){
            return false;
        }else if self.has_set(&SetOfHands::RoyalStraight){
            return true;
        }
        
        if other.high_card > self.high_card {
            return false;
        }
        true
    }
}

impl Default for HandAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

fn analyse_pair_hand(hand: &Hand, community_cards: &CommunityCards, analysis: &mut HandAnalysis) {
    let mut ctt_same_rank_cards = 2;
    let rank_of_the_cards = hand.get_cards()[0].get_rank();

    analysis.high_card = rank_of_the_cards;
    analysis.add(SetOfHands::Pair, analysis.high_card);

    ctt_same_rank_cards += community_cards.number_of_cards_by_rank(rank_of_the_cards);

    // Same of a kind
    if ctt_same_rank_cards == 3 {
        analysis.add(SetOfHands::ThreeOfAKind, rank_of_the_cards);
    } else if ctt_same_rank_cards == 4 {
        analysis.add(SetOfHands::FourOfAKind, rank_of_the_cards);
    }

    // Straight
    let straight_high_card = analyse_straight(rank_of_the_cards, community_cards);

    if straight_high_card != 0 {
        // Exists straight
        analysis.add(SetOfHands::Straight, straight_high_card);
    }

    // Flush
    let mut flush_high_card;
    for idx in [0,1] {

        flush_high_card = analyse_flush(hand.get_cards()[idx].get_suit(), 1,community_cards);

        if flush_high_card != 0 {
            if flush_high_card < rank_of_the_cards {
                flush_high_card = rank_of_the_cards;
            }
            analysis.add(SetOfHands::Flush, flush_high_card);

            break;  // We got a flush. It's impossible to get another
        }
    }

    // Full House
    let full_high_card = analyse_full_house(analysis, community_cards);
    if full_high_card != 0 {
        analysis.add(SetOfHands::FullHouse, full_high_card);
    }

    // Straight Flush
    if analysis.has_set(&SetOfHands::Straight) && analysis.has_set(&SetOfHands::Flush)   {
        let straight_flush_high_card = analyse_straight_flush(hand, community_cards);

        if straight_flush_high_card != 0 {
            analysis.add(SetOfHands::StraightFLush, straight_flush_high_card);
        }
    }

    // Royal Straight
    if analysis.get_highest_card_set(&SetOfHands::StraightFLush) == HIGHEST_RANK {
        analysis.add(SetOfHands::StraightFLush, HIGHEST_RANK);
    }
}



/// Determines if there is a straight in the hand.
///
/// # Returns
///
/// * `0` - If there is no straight.
/// * `n` - If there is a straight; returns the rank of the highest card in the straight.
fn analyse_straight(rank: u8, community_cards: &CommunityCards) -> u8 {

    let mut ctt: u8 = 0;
    let mut high_card_of_the_set: u8 = 0;
    for card_rank in bounded_range(rank) {
        if community_cards.number_of_cards_by_rank(card_rank) > 0 || card_rank == rank {
            ctt += 1;
        }else {
            ctt = 0;
        }

        if ctt >= 5 {
            high_card_of_the_set = card_rank;
        }
    }

    high_card_of_the_set
}

/// Using this function we obtain the ranks of the cards that can make a straight
fn bounded_range(pivot: u8) -> Vec<u8> {
    let min = 2;
    let max = 14;

    let start = (pivot - 4).max(min);
    let end = (pivot + 4).min(max);

    (start..=end).collect()
}


fn analyse_straight_flush(hand: &Hand, community_cards: &CommunityCards) -> u8 {

    let mut high_card_of_the_set :u8 = 0;

    for card in hand.get_cards(){
        let mut ctt:u8 = 0;
        let rank = card.get_rank();
        let suit = card.get_suit();

        for card_rank in bounded_range(rank) {
            if community_cards.contains(card_rank,suit) || card_rank == rank {
                ctt += 1;
            }else {
                ctt = 0;
            }

            if ctt >= 5 && high_card_of_the_set < card_rank {
                high_card_of_the_set = card_rank;
            }
        }
    }

    high_card_of_the_set

}

/// Determines if there is a flush in the hand. Find 4 or more cards of the same type (suit)
///
/// # Returns
///
/// * `0` - If there is no flush.
/// * `n` - If there is a flush; returns the rank of the highest card in the flush.
fn analyse_flush(suit: char, cards_same_suit: u8 , community_cards: &CommunityCards) -> u8 {

    let community_cards_same_suit: u8;
    let highest: u8;
    (community_cards_same_suit, highest) = community_cards.get_by_suit_and_highest(suit);

    if cards_same_suit + community_cards_same_suit >= NECESSARY_CARDS_FLUSH {
           return highest;
    }
    0       // Returns 0
}

/// Determines if there is a full house in the hand.
///
/// # Returns
///
/// * `0` - If there is no full house.
/// * `n` - If there is a full house; returns the rank of the three of a kind or the pair or of the pair, it depends on which hand is missing.
///     This will be properly analyzed later, when we have all the sets.
fn analyse_full_house(analysis: &HandAnalysis, community_cards: &CommunityCards) -> u8 {

    let high_card_pair = analysis.get_highest_card_set(&SetOfHands::Pair);
    let high_card_three_of_a_kind = analysis.get_highest_card_set(&SetOfHands::ThreeOfAKind);
    let repeated_ranks_necessary: u8;
    if high_card_pair == 0 {
        return 0;
    }else if high_card_pair == high_card_three_of_a_kind {      // Only Three of a kind
        repeated_ranks_necessary = 2;
    }else{      // Only Pair
        repeated_ranks_necessary = 3;
    }

    for rank in community_cards.get_cards_ranks(){
        if repeated_ranks_necessary <= community_cards.number_of_cards_by_rank(rank) {
            return rank
        }
    }


    0
}

#[cfg(test)]
mod tests {
    use crate::utils::card::{Card, DIAMONDS, HEARTS};
    use super::*;
    #[test]
    fn hand_pair_and_flush(){

        let pair_rank = 5;

        // Hand Cards
        let hand_card_0 = Card::new(DIAMONDS,pair_rank);
        let hand_card_1 = Card::new(HEARTS, pair_rank);

        // Community Cards
        let community_card_0 = Card::new(HEARTS, 2);
        let community_card_1 = Card::new(HEARTS, 3);
        let community_card_2 = Card::new(HEARTS, 6);
        let community_card_3 = Card::new(HEARTS, 8);
        let community_card_4 = Card::new(HEARTS, 9);

        let hand = Hand::new([hand_card_0,hand_card_1]);
        let community_cards = CommunityCards::new(vec![community_card_0,community_card_1,community_card_2,community_card_3,community_card_4]);

        let analysis = HandAnalysis::analyse(&hand, &community_cards);

        assert_eq!(analysis.high_card, pair_rank);
        assert!(analysis.has_set(&SetOfHands::Pair));
        assert!(analysis.has_set(&SetOfHands::Flush));
        assert_eq!(analysis.get_highest_card_set(&SetOfHands::Flush), 9);
    }


    #[test]
    fn hand_pair_straight_flush(){

        let pair_rank = 5;

        // Hand Cards
        let hand_card_0 = Card::new(DIAMONDS,pair_rank);
        let hand_card_1 = Card::new(HEARTS, pair_rank);

        // Community Cards
        let community_card_0 = Card::new(HEARTS, 3);
        let community_card_1 = Card::new(HEARTS, 4);
        let community_card_2 = Card::new(HEARTS, 6);
        let community_card_3 = Card::new(HEARTS, 7);
        let community_card_4 = Card::new(HEARTS, 8);

        let hand = Hand::new([hand_card_0,hand_card_1]);
        let community_cards = CommunityCards::new(vec![community_card_0,community_card_1,community_card_2,community_card_3,community_card_4]);

        let analysis = HandAnalysis::analyse(&hand, &community_cards);

        assert_eq!(analysis.high_card, pair_rank);
        assert!(analysis.has_set(&SetOfHands::Pair));
        assert!(analysis.has_set(&SetOfHands::Flush));
        assert!(analysis.has_set(&SetOfHands::Straight));
        assert!(analysis.has_set(&SetOfHands::StraightFLush));
        assert_eq!(analysis.get_highest_card_set(&SetOfHands::Flush), 8);
        assert_eq!(analysis.get_highest_card_set(&SetOfHands::StraightFLush), 8);
    }
}
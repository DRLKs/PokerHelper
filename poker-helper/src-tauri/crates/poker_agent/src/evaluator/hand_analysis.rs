use std::cmp::max;
use crate::core::card::{CardTrait, HIGHEST_RANK};
use crate::core::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::core::hand::{Hand, HandTrait, };
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
            analyse_same_suit_hand(hand, community_cards, &mut analysis);
        }else{
            analyse_hand_other_case(hand, community_cards, &mut analysis);
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
        
        if self.has_set(&SetOfHands::RoyalStraight){
            return true;
        }else if other.has_set(&SetOfHands::RoyalStraight){
            return false;
        }

        if self.has_set(&SetOfHands::StraightFLush){
            return other.has_set(&SetOfHands::StraightFLush) && self.get_highest_card_set(&SetOfHands::StraightFLush) < other.get_highest_card_set(&SetOfHands::Straight);
        }else if other.has_set(&SetOfHands::StraightFLush){
            return false;
        }

        if self.has_set(&SetOfHands::FourOfAKind){
            return other.has_set(&SetOfHands::FourOfAKind) && self.get_highest_card_set(&SetOfHands::FourOfAKind) < other.get_highest_card_set(&SetOfHands::FourOfAKind);
        }else if other.has_set(&SetOfHands::FourOfAKind){
            return false;
        }

        if self.has_set(&SetOfHands::FullHouse){

            if other.has_set(&SetOfHands::FullHouse) {

                if self.get_highest_card_set(&SetOfHands::FullHouse) < other.get_highest_card_set(&SetOfHands::FullHouse) {
                    return false;
                } else if self.get_highest_card_set(&SetOfHands::FullHouse) < other.get_highest_card_set(&SetOfHands::FullHouse) {
                    return true;
                }
            }
        }else if other.has_set(&SetOfHands::FullHouse){
            return false;
        }

        if self.has_set(&SetOfHands::Flush){
            return other.has_set(&SetOfHands::Flush) && self.get_highest_card_set(&SetOfHands::Flush) < other.get_highest_card_set(&SetOfHands::Flush);
        }else if other.has_set(&SetOfHands::Flush){
            return false;
        }

        if self.has_set(&SetOfHands::Straight){

            if other.has_set(&SetOfHands::Straight){
                if self.get_highest_card_set(&SetOfHands::Straight) > other.get_highest_card_set(&SetOfHands::Straight){
                    return true;
                }else if self.get_highest_card_set(&SetOfHands::Straight) < other.get_highest_card_set(&SetOfHands::Straight){
                    return false;
                }
            }

        }else if other.has_set(&SetOfHands::Straight){
            return false;
        }

        if self.has_set(&SetOfHands::ThreeOfAKind){
            return other.has_set(&SetOfHands::ThreeOfAKind) && self.get_highest_card_set(&SetOfHands::ThreeOfAKind) < other.get_highest_card_set(&SetOfHands::ThreeOfAKind);
        }else if other.has_set(&SetOfHands::ThreeOfAKind){
            return false;
        }


        // PAIRS, thats a litle bit different
        if self.has_set(&SetOfHands::Pair){

            if self.has_set(&SetOfHands::TwoPair){

                // The addition of the highest card of the two pairs are higher
                if other.has_set(&SetOfHands::TwoPair) &&
                    (self.get_highest_card_set(&SetOfHands::TwoPair) + self.get_highest_card_set(&SetOfHands::Pair))
                        >
                        (other.get_highest_card_set(&SetOfHands::TwoPair) + other.get_highest_card_set(&SetOfHands::Pair)){
                    return true;
                }else{
                    return false;
                }

            }else if other.has_set(&SetOfHands::TwoPair){
                return false;
            }

            if other.has_set(&SetOfHands::Pair) {

                if self.get_highest_card_set(&SetOfHands::Pair) < other.get_highest_card_set(&SetOfHands::Pair) {
                    return false;
                }else if self.get_highest_card_set(&SetOfHands::Pair) > other.get_highest_card_set(&SetOfHands::Pair){
                    return true;
                }
            }
        }else if other.has_set(&SetOfHands::Pair){
            return false;
        }

        if other.high_card > self.high_card {
            return false;
        }
        true
    }

    fn am_i_better_at_this_set(&self, set: &SetOfHands, other: &HandAnalysis) -> bool {

        if self.has_set(set) {
            return other.has_set(set) && self.get_highest_card_set(set) < other.get_highest_card_set(set);
        }else {
            false
        }

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


fn analyse_same_suit_hand(hand: &Hand, community_cards: &CommunityCards, analysis: &mut HandAnalysis) {

    let rank_card_1: u8 = hand.get_cards()[0].get_rank();
    let rank_card_2: u8 = hand.get_cards()[1].get_rank();

    // Set High Card
    analysis.set_high_card( max(rank_card_1, rank_card_2) );

    let ctt_same_suit = analyse_flush(hand.get_cards()[0].get_suit(), 2, community_cards);
    if ctt_same_suit != 0 {
        analysis.add(SetOfHands::Flush, ctt_same_suit);
    }

    let mut ctt_same_rank_card: u8;
    // Pair, Double Pair, Three of a kind, Four of a kind
    for rank_card in [rank_card_1, rank_card_2].iter().cloned() {
        ctt_same_rank_card = 1 + community_cards.number_of_cards_by_rank(rank_card);

        if ctt_same_rank_card >= 2 {
            if !analysis.has_set(&SetOfHands::Pair) {
                analysis.add(SetOfHands::Pair, ctt_same_rank_card);
            }else{
                let card_pair = analysis.get_highest_card_set(&SetOfHands::Pair);
                if card_pair < rank_card {
                    analysis.add(SetOfHands::Pair, rank_card);
                    analysis.add(SetOfHands::TwoPair, card_pair);
                }else{
                    analysis.add(SetOfHands::Pair, rank_card);
                }
            }
        }

        if ctt_same_rank_card >= 3 {
            if !analysis.has_set(&SetOfHands::ThreeOfAKind) {
                analysis.add(SetOfHands::ThreeOfAKind, ctt_same_rank_card);
            }else if analysis.get_highest_card_set(&SetOfHands::ThreeOfAKind) < rank_card {
                analysis.add(SetOfHands::ThreeOfAKind, rank_card);
            }
        }

        // POKER - FourOfAKind
        if ctt_same_rank_card == 4 {
            analysis.add(SetOfHands::FourOfAKind, ctt_same_rank_card);
        }

        // Straight
        let straight_high_card = analyse_straight(rank_card, community_cards);
        if straight_high_card != 0 {
            if analysis.get_highest_card_set(&SetOfHands::StraightFLush) > straight_high_card { // The straight is weaker or don`t exists
                analysis.add(SetOfHands::Straight, straight_high_card);
            }
        }
    }

    // Straight Flush
    if analysis.has_set(&SetOfHands::Straight) && analysis.has_set(&SetOfHands::Flush) {
        let straight_flush_high_card = analyse_straight_flush(hand, community_cards);
        if straight_flush_high_card != 0 {
            analysis.add(SetOfHands::Straight, straight_flush_high_card);
        }
    }

    // RoyalStraight
    if analysis.get_highest_card_set(&SetOfHands::StraightFLush) == HIGHEST_RANK {
        analysis.add(SetOfHands::RoyalStraight, HIGHEST_RANK);
    }
}

fn analyse_hand_other_case(hand: &Hand, community_cards: &CommunityCards, analysis: &mut HandAnalysis){

    let rank_card_1: u8 = hand.get_cards()[0].get_rank();
    let rank_card_2: u8 = hand.get_cards()[1].get_rank();

    let suit_card_1: char = hand.get_cards()[0].get_suit();
    let suit_card_2: char = hand.get_cards()[1].get_suit();

    // Set High Card
    analysis.set_high_card( max(rank_card_1, rank_card_2) );

    // Flush
    let mut ctt_same_suit;
    for suit_card in [suit_card_1, suit_card_2].iter().cloned() {
        ctt_same_suit = analyse_flush(suit_card, 1, community_cards);
        if ctt_same_suit != 0 {
            analysis.add(SetOfHands::Flush, ctt_same_suit);
            break
        }
    }

    let mut ctt_same_rank_card: u8;
    // Pair, Double Pair, Three of a kind, Four of a kind
    for rank_card in [rank_card_1, rank_card_2].iter().cloned() {
        ctt_same_rank_card = 1 + community_cards.number_of_cards_by_rank(rank_card);

        if ctt_same_rank_card >= 2 {
            if !analysis.has_set(&SetOfHands::Pair) {
                analysis.add(SetOfHands::Pair, ctt_same_rank_card);
            }else{
                let card_pair = analysis.get_highest_card_set(&SetOfHands::Pair);
                if card_pair < rank_card {
                    analysis.add(SetOfHands::Pair, rank_card);
                    analysis.add(SetOfHands::TwoPair, card_pair);
                }else{
                    analysis.add(SetOfHands::Pair, rank_card);
                }
            }
        }

        if ctt_same_rank_card >= 3 {
            if !analysis.has_set(&SetOfHands::ThreeOfAKind) {
                analysis.add(SetOfHands::ThreeOfAKind, ctt_same_rank_card);
            }else if analysis.get_highest_card_set(&SetOfHands::ThreeOfAKind) < rank_card {
                analysis.add(SetOfHands::ThreeOfAKind, rank_card);
            }
        }

        // POKER - FourOfAKind
        if ctt_same_rank_card == 4 {
            analysis.add(SetOfHands::FourOfAKind, ctt_same_rank_card);
        }

        // Straight
        let straight_high_card = analyse_straight(rank_card, community_cards);
        if straight_high_card != 0 {
            if analysis.get_highest_card_set(&SetOfHands::StraightFLush) > straight_high_card { // The straight is weaker or don`t exists
                analysis.add(SetOfHands::Straight, straight_high_card);
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

    let start = pivot.saturating_sub(4).max(min);
    let end = (pivot + 4).min(max);

    (start..=end).collect()
}

/// Determines if there is a straight flush in the hand.
///
/// # Returns
///
/// * `0` - If there is no straight flush.
/// * `n` - If there is a straight flush; returns the rank of the highest card in the straight.
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
    use crate::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
    use crate::evaluator::hand_analysis::SetOfHands::{FullHouse, Pair, Straight, TwoPair};
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

    #[test]
    fn same_suit_hand_flush(){
        // Hand Cards
        let hand_card_0 = Card::new(HEARTS,4);
        let hand_card_1 = Card::new(HEARTS, 6);

        // Community Cards
        let community_card_0 = Card::new(DIAMONDS, 3);
        let community_card_1 = Card::new(DIAMONDS, 4);
        let community_card_2 = Card::new(HEARTS, 2);
        let community_card_3 = Card::new(HEARTS, 7);
        let community_card_4 = Card::new(HEARTS, 8);

        let hand = Hand::new([hand_card_0,hand_card_1]);
        let community_cards = CommunityCards::new(vec![community_card_0,community_card_1,community_card_2,community_card_3,community_card_4]);

        let analysis = HandAnalysis::analyse(&hand, &community_cards);

        assert!(analysis.has_set(&SetOfHands::Flush));
        assert_eq!(analysis.get_highest_card_set(&SetOfHands::Flush), 8);
    }

    #[test]
    fn same_suit_hand_poker(){
        // Hand Cards
        let hand_card_0 = Card::new(HEARTS,4);
        let hand_card_1 = Card::new(HEARTS, 6);

        // Community Cards
        let community_card_0 = Card::new(DIAMONDS, 9);
        let community_card_1 = Card::new(DIAMONDS, 4);
        let community_card_2 = Card::new(SPADES, 4);
        let community_card_3 = Card::new(CLUBS, 4);
        let community_card_4 = Card::new(HEARTS, 8);

        let hand = Hand::new([hand_card_0,hand_card_1]);
        let community_cards = CommunityCards::new(vec![community_card_0,community_card_1,community_card_2,community_card_3,community_card_4]);

        let analysis = HandAnalysis::analyse(&hand, &community_cards);
        assert!(analysis.has_set(&SetOfHands::Pair));
        assert!(analysis.has_set(&SetOfHands::ThreeOfAKind));
        assert!(analysis.has_set(&SetOfHands::FourOfAKind));
        assert_eq!(analysis.get_highest_card_set(&SetOfHands::FourOfAKind), 4);
    }

    #[test]
    fn better_hand_two_pairs(){
        // Self analysis
        let mut self_hand_analysis = HandAnalysis::new();
        self_hand_analysis.add(Pair, 14);
        self_hand_analysis.add(TwoPair, 12);

        // Other analysis
        let mut other_hand_analysis = HandAnalysis::new();
        other_hand_analysis.add(Pair, 13);
        other_hand_analysis.add(TwoPair, 9);

        assert!(self_hand_analysis.is_better(&other_hand_analysis));
    }

    #[test]
    fn better_hand_same_pair_high_card(){
        // Self analysis
        let mut self_hand_analysis = HandAnalysis::new();
        self_hand_analysis.add(Pair, 12);
        self_hand_analysis.set_high_card(14);

        // Other analysis
        let mut other_hand_analysis = HandAnalysis::new();
        other_hand_analysis.add(Pair, 12);
        other_hand_analysis.set_high_card(12);

        assert!(self_hand_analysis.is_better(&other_hand_analysis));
    }

    #[test]
    fn better_hand_same_straight_high_card(){
        // Self analysis
        let mut self_hand_analysis = HandAnalysis::new();
        self_hand_analysis.add(Straight, 12);
        self_hand_analysis.set_high_card(14);

        // Other analysis
        let mut other_hand_analysis = HandAnalysis::new();
        other_hand_analysis.add(Straight, 12);
        other_hand_analysis.set_high_card(12);

        assert!(self_hand_analysis.is_better(&other_hand_analysis));
    }

    #[test]
    fn better_hand_same_full_high_card(){
        // Self analysis
        let mut self_hand_analysis = HandAnalysis::new();
        self_hand_analysis.add(FullHouse, 12);
        self_hand_analysis.set_high_card(14);

        // Other analysis
        let mut other_hand_analysis = HandAnalysis::new();
        other_hand_analysis.add(FullHouse, 12);
        other_hand_analysis.set_high_card(12);

        assert!(self_hand_analysis.is_better(&other_hand_analysis));
    }

    #[test]
    fn better_hand_same_full_high_card_other(){
        // Self analysis
        let mut self_hand_analysis = HandAnalysis::new();
        self_hand_analysis.add(FullHouse, 12);
        self_hand_analysis.set_high_card(12);

        // Other analysis
        let mut other_hand_analysis = HandAnalysis::new();
        other_hand_analysis.add(FullHouse, 12);
        other_hand_analysis.set_high_card(14);

        assert!(!self_hand_analysis.is_better(&other_hand_analysis));
    }
}
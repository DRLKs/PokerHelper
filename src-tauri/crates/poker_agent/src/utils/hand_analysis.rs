use std::collections::HashMap;
use serde::Serialize;
use crate::utils::card::CardTrait;
use crate::utils::community_cards::{CommunityCards, CommunityCardsTrait};
use crate::utils::hand::{Hand, HandTrait};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetOfHands {
    Pair,           // Pareja
    TwoPair,        // 2 Parejas
    ThreeOfAKind,   // Trio
    Straight,
    Flush,          // Color
    FullHouse,
    FourOfAKind,
    StraightFLush,
    RoyalStraight
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

        }else if hand.is_same_suit() {

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

    pub fn is_better(&self, other: &HandAnalysis) -> bool {

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
    let rank = hand.get_cards()[0].get_rank();

    analysis.high_card = rank;
    analysis.sets.insert(SetOfHands::Pair, analysis.high_card);

    ctt_same_rank_cards +=  community_cards.get_by_rank(rank);

    // Same of a kind
    if ctt_same_rank_cards == 3 {
        analysis.sets.insert(SetOfHands::ThreeOfAKind, rank);
    }else if ctt_same_rank_cards == 4 {
        analysis.sets.insert(SetOfHands::FourOfAKind, rank);
    }

    // Straight
    let straight_high_card = analyse_straight(rank,community_cards);

    if straight_high_card != 0{     // Exists straight
        analysis.sets.insert(SetOfHands::Straight, straight_high_card);
    }

    // Flush

}

fn analyse_straight(rank: u8, community_cards: &CommunityCards) -> u8 {

    // TODO

    let mut possible_straight_cards = Vec::new();
    for rank_of_the_card in community_cards.get_cards_ranks(){

        if rank_of_the_card != rank && rank_of_the_card <= rank + 4 && rank_of_the_card >= rank - 4   {
            possible_straight_cards.push(rank_of_the_card);
        }

    }

    return 0
}

fn analyse_flush(suit: char, community_cards: &CommunityCards) -> u8 {

    // TODO

    if community_cards.get_by_suit(suit) == 4 {
        return 1;
    }

    0
}

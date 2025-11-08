use std::collections::HashMap;
use serde::Deserialize;
use super::card::{Card, CardTrait};

pub trait CommunityCardsTrait {
    fn get_cards(&self) -> &[Card];

    fn number_of_cards(&self) -> usize;

    fn get_by_rank(&self, rank: u8) -> u8;

    fn get_by_suit(&self, suit: char) -> u8;
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommunityCards {
    /// Map with the card number that appears and a vector with his suits
    cards_map: HashMap<u8, Vec<char>>,
    /// Cached vector of cards for efficient access
    cards: Vec<Card>
}

impl CommunityCards {

    pub fn new(cards: Vec<Card>) -> Self {
        let mut community = Self::empty();

        for card in cards {
            community.add_card(card);
        }

        community
    }

    pub fn empty() -> Self {
        Self { 
            cards_map: HashMap::new(),
            cards: Vec::new(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.cards_map.is_empty()
    }

    pub fn add_card(&mut self, card: Card) {
        let rank = card.get_rank();
        if !self.cards_map.contains_key(&rank) {
            self.cards_map.insert(rank, vec![]);
        }
        self.cards_map.get_mut(&rank).unwrap().push(card.get_suit());
        self.cards.push(card);
    }

    pub fn get_cards_ranks(&self) -> Vec<u8> {
        let mut cards_ranks = vec![];
        for rank in self.cards_map.keys() {
            cards_ranks.push(*rank);
        }
        cards_ranks
    }
}

impl CommunityCardsTrait for CommunityCards {
    fn get_cards(&self) -> &[Card] {
        &self.cards
    }

    fn number_of_cards(&self) -> usize {
        let mut size = 0;
        for values in self.cards_map.iter() {
            size += values.1.len();
        }
        size
    }

    fn get_by_rank(&self, rank: u8) -> u8 {

        let value: Option<&Vec<char>> = self.cards_map.get(&rank);
        if value.is_some() {
               value.unwrap().len() as u8
        }else {
            0
        }
    }

    fn get_by_suit(&self, suit: char) -> u8 {
        let mut ctt: u8 = 0;
        for values in self.cards_map.iter() {
            for s in values.1.iter().copied() {
                if s == suit {
                    ctt += 1;
                    break;
                }
            }
        }
        ctt
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::card::{DIAMONDS, HEARTS};
    use super::*;

    #[test]
    pub fn test_empty_constructor(){
        let community_cards = CommunityCards::empty();
        assert_eq!(community_cards.number_of_cards(), 0);
        assert_eq!(community_cards.is_empty(), true);
    }

    #[test]
    pub fn test_add_card(){
        let mut community_cards = CommunityCards::empty();
        let rank_card = 10;
        community_cards.add_card( Card::new(DIAMONDS,rank_card) );
        assert_eq!(community_cards.number_of_cards(), 1);
        assert_eq!(community_cards.is_empty(), false);

        assert_eq!(community_cards.get_by_suit(HEARTS),0);
        assert_eq!(community_cards.get_by_suit(DIAMONDS), 1 );
        assert_eq!(community_cards.get_by_rank(rank_card), 1);
    }

    #[test]
    pub fn test_add_some_card_same_rank(){
        let mut community_cards = CommunityCards::empty();
        let rank_card = 10;
        community_cards.add_card( Card::new(DIAMONDS,rank_card) );
        community_cards.add_card( Card::new(HEARTS,rank_card) );
        assert_eq!(community_cards.number_of_cards(), 2);
        assert_eq!(community_cards.is_empty(), false);

        assert_eq!(community_cards.get_by_suit(HEARTS),1);
        assert_eq!(community_cards.get_by_suit(DIAMONDS), 1 );
        assert_eq!(community_cards.get_by_rank(rank_card), 2);
    }

    #[test]
    pub fn test_get_cards_but_no_cards(){
        let community_cards: CommunityCards = CommunityCards::empty();
        let rank_card: u8 = 10;

        assert_eq!(community_cards.get_by_rank(rank_card), 0);
    }
}
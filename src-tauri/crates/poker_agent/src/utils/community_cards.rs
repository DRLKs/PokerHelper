use std::collections::HashMap;
use serde::Deserialize;
use super::card::{Card, CardTrait};

pub trait CommunityCardsTrait {
    fn get_cards(&self) -> &[Card];

    fn to_array(&self) -> [Card; 2];
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommunityCards {
    cards_map: HashMap<u8, Vec<char>>,
    cards: Vec<Card>,
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
        self.cards.push(card);
        let rank = card.get_rank();
        if !self.cards_map.contains_key(&rank) {
            self.cards_map.insert(rank, vec![]);
        }
        self.cards_map.get_mut(&rank).unwrap().push(card.get_suit());
    }
}

impl CommunityCardsTrait for CommunityCards {
    fn get_cards(&self) -> &[Card] {
        &self.cards
    }

    fn to_array(&self) -> [Card; 2] {
        [self.cards[0], self.cards[1]]
    }
}

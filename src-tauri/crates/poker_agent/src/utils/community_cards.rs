use serde::Deserialize;
use super::card::Card;

pub trait CommunityCardsTrait {
    fn get_cards(&self) -> &[Card];
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommunityCards {
    cards: Vec<Card>  // Puede tener 0, 3, 4 o 5 cartas
}

impl CommunityCards {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }
    
    pub fn empty() -> Self {
        Self { cards: Vec::new() }
    }
    
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl CommunityCardsTrait for CommunityCards {
    fn get_cards(&self) -> &[Card] {
        &self.cards
    }
}
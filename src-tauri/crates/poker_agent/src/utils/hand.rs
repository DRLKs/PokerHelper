use serde::Deserialize;
use super::card::{Card, CardTrait};

pub trait HandTrait {
    fn is_pair(&self) -> bool;
    fn is_same_suit(&self) -> bool;
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hand {
    cards: [Card; 2]
}

impl Hand {
    pub fn new(cards: [Card; 2]) -> Self {
        Self { cards }
    }

    pub fn get_cards(&self) -> &[Card; 2] {
        &self.cards
    }
}

impl HandTrait for Hand {
    fn is_pair(&self) -> bool {
        self.cards[0].get_rank() == self.cards[1].get_rank()
    }

    fn is_same_suit(&self) -> bool {
        self.cards[0].get_suit() == self.cards[1].get_suit()
    }
}
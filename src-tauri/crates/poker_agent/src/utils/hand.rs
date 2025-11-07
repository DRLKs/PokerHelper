use serde::Deserialize;
use super::card::{Card, CardTrait};



pub trait HandTrait {
    fn is_pair(&self) -> bool;
    fn is_same_suit(&self) -> bool;

    fn get_cards(&self) -> [Card; 2];
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hand {
    cards: [Card; 2]
}

impl Hand {
    pub fn new(cards: [Card; 2]) -> Self {
        Self { cards }
    }

    pub fn random_hand() -> Self{

        let card1: Card = Card::new_random();
        let card2: Card = Card::new_random();

        Hand::new([card1, card2])
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

    fn get_cards(&self) -> [Card; 2] {
        [self.cards[0].clone(), self.cards[1].clone()]
    }
}
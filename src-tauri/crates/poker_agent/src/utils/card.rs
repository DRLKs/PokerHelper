use serde::{Deserialize, Serialize};

pub trait CardTrait {
    fn get_suit(&self) -> char;
    fn get_rank(&self) -> u8;
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    suit: char,  // 'h', 'd', 'c', 's'
    rank: u8     // 2-14 ( 11=J, 12=Q, 13=K, 14=A )
}

impl Card {
    pub fn new(suit: char, rank: u8) -> Self {
        Card { suit, rank }
    }
}

impl CardTrait for Card {
    fn get_suit(&self) -> char {
        self.suit
    }

    fn get_rank(&self) -> u8 {
        self.rank
    }
}
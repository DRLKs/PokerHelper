use crate::cons::r#const::ALL_CARDS;
use rand::Rng;
use serde::Deserialize;


pub const DIAMONDS: char = 'd';
pub const CLUBS: char = 'c';
pub const HEARTS: char = 'h';
pub const SPADES: char = 's';

pub trait CardTrait {
    fn get_suit(&self) -> char;
    fn get_rank(&self) -> u8;

    fn same_rank(&self, other: &Card) -> bool;

    fn same_suit(&self, other: &Card) -> bool;

    fn same_suit_by_char(&self, suit: char) -> bool;
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    suit: char,  // 'h', 'd', 'c', 's'
    rank: u8     // 2-14 ( 11=J, 12=Q, 13=K, 14=A )
}

impl Card {
    pub const fn new(suit: char, rank: u8) -> Self {
        Card { suit, rank }
    }

    /// Crea una carta aleatoria (puede repetirse)
    /// Para cartas sin repetición, usa Deck::draw_random()
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let rdm_idx = rng.gen_range(0..ALL_CARDS.len());
        ALL_CARDS[rdm_idx]
    }
}

impl CardTrait for Card {
    fn get_suit(&self) -> char {
        self.suit
    }

    fn get_rank(&self) -> u8 {
        self.rank
    }

    fn same_rank(&self, other: &Card) -> bool {
        self.rank == other.rank
    }

    fn same_suit(&self, other: &Card) -> bool {
        self.suit == other.suit
    }

    fn same_suit_by_char(&self, suit: char) -> bool {
        self.suit == suit
    }
}

#[cfg(test)]
mod tests {
    use crate::cons::r#const::{ALL_RANKS, ALL_SUITS};
    use super::*;

    #[test]
    fn test_create_card(){
        let rank = 2;
        let card = Card::new(DIAMONDS, rank);
        assert_eq!(card.suit, DIAMONDS);
        assert_eq!(card.rank, rank);
    }

    /// Nunca debería ocurrir, pero prefiero no manejar el error y que no explote el programa si no ocurre
    /// Es una mala práctica
    #[test]
    fn test_create_card_with_wrong_values(){
        let card = Card::new(DIAMONDS, 0);
        assert_eq!(card.suit, DIAMONDS);
        assert_eq!(card.rank, 0);
    }

    #[test]
    fn test_create_random_card(){
        let random_card = Card::new_random();
        assert!( ALL_SUITS.contains(&random_card.suit) );
        assert!( ALL_RANKS.contains(&random_card.rank) );
    }
}
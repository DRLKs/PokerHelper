use crate::r#const::r#const::ALL_CARDS;
use rand::Rng;
use serde::Deserialize;

pub const DIAMONDS: char = 'd';
pub const CLUBS: char = 'c';
pub const HEARTS: char = 'h';
pub const SPADES: char = 's';
pub const HIGHEST_RANK: u8 = 14;

pub trait CardTrait {
    fn get_suit(&self) -> char;
    fn get_rank(&self) -> u8;

    fn same_rank(&self, other: &Card) -> bool;

    fn same_suit(&self, other: &Card) -> bool;

    fn same_suit_by_char(&self, suit: char) -> bool;
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    suit: char,
    rank: u8,
}

impl Card {
    pub const fn new(suit: char, rank: u8) -> Self {
        Card { suit, rank }
    }

    /// Creates a random card. Repeated calls can return the same card;
    /// use `Deck::draw_random()` to draw without repetition.
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..ALL_CARDS.len());
        ALL_CARDS[random_index]
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
    use super::*;
    use crate::r#const::r#const::{ALL_RANKS, ALL_SUITS};

    #[test]
    fn test_create_card() {
        let rank = 2;
        let card = Card::new(DIAMONDS, rank);
        assert_eq!(card.suit, DIAMONDS);
        assert_eq!(card.rank, rank);
    }

    #[test]
    fn test_create_card_with_wrong_values_does_not_panic() {
        let card = Card::new(DIAMONDS, 0);
        assert_eq!(card.suit, DIAMONDS);
        assert_eq!(card.rank, 0);
    }

    #[test]
    fn test_create_random_card() {
        let random_card = Card::new_random();
        assert!(ALL_SUITS.contains(&random_card.suit));
        assert!(ALL_RANKS.contains(&random_card.rank));
    }
}

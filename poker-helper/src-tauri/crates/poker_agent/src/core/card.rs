use crate::r#const::r#const::ALL_CARDS;
use rand::Rng;
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

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

/// Error returned when a text such as `"Ah"` or `"10d"` is not a valid card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCardError(pub String);

impl fmt::Display for ParseCardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid card: {:?}", self.0)
    }
}

impl std::error::Error for ParseCardError {}

impl FromStr for Card {
    type Err = ParseCardError;

    /// Parses a rank (`2`-`9`, `T` or `10`, `J`, `Q`, `K`, `A`) followed by a
    /// suit (`h`, `d`, `c`, `s`), for example `"Ah"` or `"10d"`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || ParseCardError(text.to_string());
        let mut characters = text.chars();
        let suit = characters.next_back().ok_or_else(invalid)?;
        let rank = match characters.as_str() {
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            "9" => 9,
            "T" | "10" => 10,
            "J" => 11,
            "Q" => 12,
            "K" => 13,
            "A" => 14,
            _ => return Err(invalid()),
        };
        match suit {
            HEARTS | DIAMONDS | CLUBS | SPADES => Ok(Card::new(suit, rank)),
            _ => Err(invalid()),
        }
    }
}

/// Character used for a rank in card and hand notation (`T` for ten), or `?`
/// for a rank outside 2..=14.
pub fn rank_symbol(rank: u8) -> char {
    match rank {
        2..=9 => (b'0' + rank) as char,
        10 => 'T',
        11 => 'J',
        12 => 'Q',
        13 => 'K',
        14 => 'A',
        _ => '?',
    }
}

impl fmt::Display for Card {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", rank_symbol(self.rank), self.suit)
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
    fn parses_and_prints_cards() {
        assert_eq!("Ah".parse::<Card>(), Ok(Card::new(HEARTS, 14)));
        assert_eq!("10d".parse::<Card>(), Ok(Card::new(DIAMONDS, 10)));
        assert_eq!("Td".parse::<Card>().unwrap().to_string(), "Td");
        assert_eq!("7c".parse::<Card>().unwrap().to_string(), "7c");
        for invalid in ["", "A", "1h", "Ax", "AAh"] {
            assert!(invalid.parse::<Card>().is_err(), "{invalid} should be rejected");
        }
    }

    #[test]
    fn test_create_random_card() {
        let random_card = Card::new_random();
        assert!(ALL_SUITS.contains(&random_card.suit));
        assert!(ALL_RANKS.contains(&random_card.rank));
    }
}

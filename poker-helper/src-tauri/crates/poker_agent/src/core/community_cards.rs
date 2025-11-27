use super::card::{Card, CardTrait};
use serde::Deserialize;
use std::collections::HashMap;

pub trait CommunityCardsTrait {
    fn get_cards(&self) -> &[Card];

    fn number_of_cards(&self) -> usize;

    fn number_of_cards_by_rank(&self, rank: u8) -> u8;

    fn number_of_cards_by_suit(&self, suit: char) -> u8;
    fn get_by_suit_and_highest(&self, suit: char) -> (u8, u8);

    fn contains(&self, rank: u8, suit: char) -> bool;
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommunityCards {
    /// Map with the card number that appears and a vector with his suits
    cards_map: HashMap<u8, Vec<char>>,
    /// Cached vector of cards for efficient access
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

    pub fn new_array(cards: [Card;5]) -> Self {
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

    fn number_of_cards_by_rank(&self, rank: u8) -> u8 {
        let value: Option<&Vec<char>> = self.cards_map.get(&rank);
        if value.is_some() {
            value.unwrap().len() as u8
        } else {
            0
        }
    }


    fn number_of_cards_by_suit(&self, suit: char) -> u8 {
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

    /// Returns: Number of cards with that suit and the highest rank of that cards
    fn get_by_suit_and_highest(&self, suit: char) -> (u8, u8) {
        let mut highest: u8 = 0;
        let mut ctt: u8 = 0;
        for values in self.cards_map.iter() {
            for s in values.1.iter().copied() {
                if s == suit {
                    ctt += 1;
                    if values.0 > &highest {
                        highest = *values.0;
                    }

                    break;
                }
            }
        }
        (ctt,highest)
    }

    fn contains(&self, rank: u8, suit: char) -> bool {
        self.cards_map.contains_key(&rank) && self.cards_map.get(&rank).unwrap().contains(&suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::card::{CLUBS, DIAMONDS, HEARTS, SPADES};

    #[test]
    pub fn test_empty_constructor() {
        let community_cards = CommunityCards::empty();
        assert_eq!(community_cards.number_of_cards(), 0);
        assert_eq!(community_cards.is_empty(), true);
        let carta = Card::new_random();
        assert!(!community_cards.contains(carta.get_rank(), carta.get_suit()));
    }

    #[test]
    pub fn test_add_card() {
        let mut community_cards: CommunityCards = CommunityCards::empty();
        let carta = Card::new_random();
        (&mut community_cards).add_card(Card::new(carta.get_suit(), carta.get_rank()));

        assert_eq!(community_cards.number_of_cards(), 1);
        assert_eq!(community_cards.is_empty(), false);

        assert_eq!(community_cards.number_of_cards_by_suit(carta.get_suit()), 1);
        assert_eq!(community_cards.number_of_cards_by_rank(carta.get_rank()), 1);
        assert!(community_cards.contains(carta.get_rank(), carta.get_suit()));
    }

    #[test]
    pub fn test_add_some_card_same_rank() {
        let mut community_cards: CommunityCards = CommunityCards::empty();
        let rank_card = 10;
        let card1 = Card::new(DIAMONDS, rank_card);
        let card2 = Card::new(HEARTS, rank_card);

        (&mut community_cards).add_card( card1 );
        (&mut community_cards).add_card(card2);
        assert_eq!(community_cards.number_of_cards(), 2);
        assert_eq!(community_cards.is_empty(), false);

        assert_eq!(community_cards.number_of_cards_by_suit(HEARTS), 1);
        assert_eq!(community_cards.number_of_cards_by_suit(DIAMONDS), 1);
        assert_eq!(community_cards.number_of_cards_by_rank(rank_card), 2);
    }

    #[test]
    pub fn test_get_cards_but_no_cards() {
        let community_cards: CommunityCards = CommunityCards::empty();
        let rank_card: u8 = 10;

        assert_eq!(community_cards.number_of_cards_by_rank(rank_card), 0);
        assert_eq!(community_cards.number_of_cards_by_suit(DIAMONDS), 0);
        assert_eq!(community_cards.number_of_cards_by_suit(SPADES), 0);
        assert_eq!(community_cards.number_of_cards_by_suit(HEARTS), 0);
        assert_eq!(community_cards.number_of_cards_by_suit(CLUBS), 0);
    }

    #[test]
    pub fn test_get_cards_of_one_suit() {
        let mut community_cards: CommunityCards = CommunityCards::empty();
        community_cards.add_card(Card::new(DIAMONDS, 10));
        community_cards.add_card(Card::new(DIAMONDS, 9));
        community_cards.add_card(Card::new(DIAMONDS, 8));

        let number_of_cards;
        let highest;
        (number_of_cards, highest) =  community_cards.get_by_suit_and_highest(DIAMONDS);

        assert_eq!(number_of_cards, 3);
        assert_eq!(highest, 10);
    }
}

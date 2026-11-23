use super::card::Card;
use crate::r#const::r#const::ALL_CARDS;
use crate::core::hand::Hand;
use rand::Rng;
use std::collections::HashSet;

/// Deck of cards with constant-time removal.
#[derive(Debug, Clone)]
pub struct Deck {
    /// Remaining cards. Removal uses `swap_remove`, so it is O(1) and does not keep the order.
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a full 52-card deck.
    pub fn new_full() -> Self {
        let cards = ALL_CARDS.to_vec();
        Self { cards }
    }

    /// Creates an empty deck.
    pub fn new_empty() -> Self {
        Self { cards: Vec::new() }
    }

    /// Creates a full deck without the given cards.
    pub fn new_excluding(excluded: &[Card]) -> Self {
        let excluded_set: HashSet<Card> = excluded.iter().copied().collect();

        let cards: Vec<Card> = ALL_CARDS
            .iter()
            .copied()
            .filter(|card| !excluded_set.contains(card))
            .collect();

        Self { cards }
    }

    /// Shuffles the deck.
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let len = self.cards.len();

        for i in 0..len {
            let j = rng.gen_range(i..len);
            self.cards.swap(i, j);
        }
    }

    /// Removes and returns a random card, or `None` if the deck is empty. O(1).
    pub fn draw_random(&mut self) -> Option<Card> {
        if self.cards.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.cards.len());

        Some(self.cards.swap_remove(index))
    }

    /// Removes and returns the card at `index`, or `None` if it is out of bounds. O(1).
    pub fn draw_at(&mut self, index: usize) -> Option<Card> {
        if index >= self.cards.len() {
            return None;
        }

        Some(self.cards.swap_remove(index))
    }

    /// Removes and returns the top card, as in a real deck.
    pub fn draw_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Removes and returns up to `n` random cards.
    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let count = n.min(self.cards.len());
        (0..count).filter_map(|_| self.draw_random()).collect()
    }

    /// Puts a card back into the deck.
    pub fn return_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Returns whether the deck contains `card`.
    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    /// Number of cards left.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns whether the deck has no cards left.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// The cards left in the deck.
    pub fn remaining_cards(&self) -> &[Card] {
        &self.cards
    }

    /// Restores the deck to the full 52 cards.
    pub fn reset_full(&mut self) {
        *self = Self::new_full();
    }

    pub fn give_n_hands(&mut self, n: u8) -> Vec<Hand> {
        let mut hands: Vec<Hand> = vec![];

        for _ in 0..n {
            let cards: Vec<Card> = self.draw_n(2);
            hands.push(Hand::new([cards[0].clone(), cards[1].clone()]));
        }

        hands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_deck_has_52_cards() {
        let deck = Deck::new_full();
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn test_draw_random() {
        let mut deck = Deck::new_full();
        let initial_len = deck.len();

        let card = deck.draw_random();
        assert!(card.is_some());
        assert_eq!(deck.len(), initial_len - 1);
    }

    #[test]
    fn test_draw_all_cards() {
        let mut deck = Deck::new_full();

        for _ in 0..52 {
            assert!(deck.draw_random().is_some());
        }

        assert!(deck.is_empty());
        assert!(deck.draw_random().is_none());
    }

    #[test]
    fn test_excluding_cards() {
        let ace_of_hearts = Card::new('h', 14);
        let ace_of_diamonds = Card::new('d', 14);
        let excluded = vec![ace_of_hearts, ace_of_diamonds];

        let deck = Deck::new_excluding(&excluded);
        assert_eq!(deck.len(), 50);

        for card in &excluded {
            assert!(!deck.contains(card));
        }
    }

    #[test]
    fn test_draw_n() {
        let mut deck = Deck::new_full();
        let cards = deck.draw_n(5);

        assert_eq!(cards.len(), 5);
        assert_eq!(deck.len(), 47);
    }

    #[test]
    fn test_shuffle_keeps_every_card() {
        let ordered = Deck::new_full();
        let mut shuffled = ordered.clone();

        shuffled.shuffle();

        assert_eq!(ordered.len(), shuffled.len());
    }
}

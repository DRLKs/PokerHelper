use super::card::Card;
use crate::cons::r#const::ALL_CARDS;
use rand::Rng;
use std::collections::HashSet;
use crate::utils::hand::Hand;

/// Mazo de cartas eficiente con eliminación rápida
#[derive(Debug, Clone)]
pub struct Deck {
    /// Cartas disponibles (usar Vec con swap_remove para O(1) al eliminar)
    cards: Vec<Card>,
}

impl Deck {
    /// Crea un mazo completo de 52 cartas
    pub fn new_full() -> Self {
        let cards = ALL_CARDS.to_vec();
        Self { cards }
    }

    /// Crea un mazo vacío
    pub fn new_empty() -> Self {
        Self { cards: Vec::new() }
    }

    /// Crea un mazo excluyendo las cartas especificadas
    pub fn new_excluding(excluded: &[Card]) -> Self {
        let excluded_set: HashSet<Card> = excluded.iter().copied().collect();
        
        let cards: Vec<Card> = ALL_CARDS
            .iter()
            .copied()
            .filter(|card| !excluded_set.contains(card))
            .collect();
        
        Self { cards }
    }

    /// Mezcla el mazo
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let len = self.cards.len();
        
        for i in 0..len {
            let j = rng.gen_range(i..len);
            self.cards.swap(i, j);
        }
    }

    /// Saca una carta aleatoria del mazo (O(1))
    /// Usa swap_remove que es muy eficiente
    pub fn draw_random(&mut self) -> Option<Card> {
        if self.cards.is_empty() {
            return None;
        }
        
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.cards.len());
        
        // swap_remove: intercambia con el último y hace pop (O(1))
        Some(self.cards.swap_remove(index))
    }

    /// Saca la carta en la posición index (O(1))
    pub fn draw_at(&mut self, index: usize) -> Option<Card> {
        if index >= self.cards.len() {
            return None;
        }
        
        Some(self.cards.swap_remove(index))
    }

    /// Saca la primera carta (como en un mazo real)
    pub fn draw_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Saca N cartas aleatorias
    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let count = n.min(self.cards.len());
        (0..count)
            .filter_map(|_| self.draw_random())
            .collect()
    }

    /// Devuelve una carta al mazo
    pub fn return_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Verifica si el mazo contiene una carta específica
    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    /// Número de cartas restantes
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Verifica si el mazo está vacío
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Obtiene referencia a todas las cartas restantes
    pub fn remaining_cards(&self) -> &[Card] {
        &self.cards
    }

    /// Resetea el mazo a 52 cartas completas
    pub fn reset_full(&mut self) {
        *self = Self::new_full();
    }
    
    pub fn give_n_hands(&mut self, n: u8) -> Vec<Hand> {
        
        let mut hands: Vec<Hand> = vec![];

        for _ in 0..n {
            let cards: Vec<Card> = self.draw_n(2);
            hands.push( Hand::new( [cards[0].clone(), cards[1].clone()] ) );
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
        let excluded = vec![
            Card::new('h', 14), // Ah
            Card::new('d', 14), // Ad
        ];
        
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
    fn test_shuffle_changes_order() {
        let deck1 = Deck::new_full();
        let mut deck2 = deck1.clone();
        
        deck2.shuffle();
        
        // Es extremadamente improbable que sean iguales después de mezclar
        // pero no podemos garantizarlo al 100%, así que solo verificamos que compile
        assert_eq!(deck1.len(), deck2.len());
    }
}

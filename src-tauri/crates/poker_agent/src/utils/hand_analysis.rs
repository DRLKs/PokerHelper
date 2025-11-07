use std::collections::HashMap;
use crate::game_state::SetOfHands;

pub trait CardAnalysisTrait {
    fn get_high_card(&self) -> u8;
    fn get_sets(&self) -> &HashMap<SetOfHands, u8>;
}

#[derive(Debug, Clone)]
pub struct HandAnalysis {
    high_card: u8,
    sets: HashMap<SetOfHands, u8>,
}

impl HandAnalysis {
    pub fn new() -> Self {
        Self {
            high_card: 0,
            sets: HashMap::new(),
        }
    }

    /// Establece la carta alta
    pub fn set_high_card(&mut self, high_card: u8) {
        self.high_card = high_card;
    }

    /// Añade o actualiza un set (pareja, trío, etc.)
    pub fn add(&mut self, hand_type: SetOfHands, high_card_of_the_set: u8) {
        self.sets.insert(hand_type, high_card_of_the_set);
    }

    /// Obtiene la carta alta
    pub fn get_high_card(&self) -> u8 {
        self.high_card
    }

    /// Obtiene referencia a todos los sets
    pub fn get_sets(&self) -> &HashMap<SetOfHands, u8> {
        &self.sets
    }

    /// Verifica si tiene un tipo de mano específico
    pub fn has_set(&self, hand_type: &SetOfHands) -> bool {
        self.sets.contains_key(hand_type)
    }
}

impl Default for HandAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl CardAnalysisTrait for HandAnalysis {
    fn get_high_card(&self) -> u8 {
        self.high_card
    }

    fn get_sets(&self) -> &HashMap<SetOfHands, u8> {
        &self.sets
    }
}
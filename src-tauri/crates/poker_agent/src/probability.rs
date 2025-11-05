// TODO: Implementar cálculos de probabilidad y equity

use crate::utils::community_cards::CommunityCards;
use crate::utils::hand::Hand;

pub fn calculate_equity(hand: Hand, community_cards: CommunityCards) -> Result<f64, String> {
    // Tu implementación aquí
    Ok(0.5)
}

pub fn calculate_pot_odds(pot_size: f64, call_amount: f64) -> f64 {
    // Tu implementación aquí
    if call_amount <= 0.0 {
        0.0
    } else {
        call_amount / (pot_size + call_amount)
    }
}

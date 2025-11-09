use crate::utils::community_cards::CommunityCards;
use crate::utils::hand::Hand;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct CvState {
    pub status: String,
    pub small_blind: f64,
    pub pot: f64,
    pub opponents: Vec<OpponentState>,
    pub hand: Hand,
    pub community_cards: CommunityCards,
}

#[derive(Deserialize, Debug)]
pub struct OpponentState {
    pub opponent_bet: f64,
}

#[derive(Serialize, Debug)]
pub enum PokerAction {
    Fold,
    Check,
    Call,
    Bet(f64),
    Raise(f64),
}

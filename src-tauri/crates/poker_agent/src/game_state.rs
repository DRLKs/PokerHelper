use serde::{Deserialize, Serialize};
use crate::utils::hand::Hand;
use crate::utils::community_cards::CommunityCards;

#[derive(Deserialize, Debug)]
pub struct CvState {
    pub status: String,
    pub pot: f64,
    pub opponent_bet: f64,
    pub hand: Hand,
    pub community_cards: CommunityCards,
}

#[derive(Serialize, Debug)]
pub enum PokerAction {
    Fold,
    Check,
    Call,
    Bet(f64),
    Raise(f64),
}

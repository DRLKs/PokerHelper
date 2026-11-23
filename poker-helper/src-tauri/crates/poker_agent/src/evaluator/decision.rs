use crate::evaluator::game_state::{CvState, PokerAction};
use crate::probability::calculate_equity;

const BET_EQUITY_THRESHOLD_PERCENT: f64 = 50.0;

/// `iterations`: number of Monte Carlo deals used to estimate equity.
pub fn make_decision(state: CvState, iterations: u32) -> Result<PokerAction, String> {
    let equity = calculate_equity(
        state.hand,
        state.community_cards,
        state.opponents.len() as u8,
        iterations,
    )?;

    if equity > BET_EQUITY_THRESHOLD_PERCENT {
        Ok(PokerAction::Bet(200.0))
    } else {
        Ok(PokerAction::Fold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::card::{Card, CLUBS, DIAMONDS, HEARTS, SPADES};
    use crate::core::community_cards::CommunityCards;
    use crate::core::hand::Hand;
    use crate::evaluator::game_state::OpponentState;

    const ITERATIONS: u32 = 20_000;

    fn state_with(hand: [Card; 2], board: Vec<Card>, opponent_count: usize) -> CvState {
        CvState {
            status: "playing".to_string(),
            small_blind: 10.0,
            pot: 100.0,
            opponents: (0..opponent_count).map(|_| OpponentState { opponent_bet: 0.0 }).collect(),
            hand: Hand::new(hand),
            community_cards: CommunityCards::new(board),
        }
    }

    #[test]
    fn bets_with_royal_flush() {
        let hand = [Card::new(SPADES, 14), Card::new(SPADES, 13)];
        let board = vec![
            Card::new(SPADES, 12), Card::new(SPADES, 11), Card::new(SPADES, 10),
            Card::new(HEARTS, 2), Card::new(CLUBS, 7),
        ];
        let action = make_decision(state_with(hand, board, 1), ITERATIONS).unwrap();
        assert!(matches!(action, PokerAction::Bet(_)), "expected Bet, got {action:?}");
    }

    #[test]
    fn folds_seven_deuce_offsuit_against_five_opponents() {
        let hand = [Card::new(SPADES, 7), Card::new(DIAMONDS, 2)];
        let action = make_decision(state_with(hand, vec![], 5), ITERATIONS).unwrap();
        assert!(matches!(action, PokerAction::Fold), "expected Fold, got {action:?}");
    }
}

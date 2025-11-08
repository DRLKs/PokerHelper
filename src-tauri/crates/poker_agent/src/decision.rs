// TODO: Implementar lógica de toma de decisiones

use crate::game_state::{CvState, PokerAction};
use crate::probability::calculate_equity;

/// Max iterations 2¹⁶ = 65536
pub fn make_decision(state: CvState, iterations: u16) -> Result<PokerAction, String> {
    // TODO: Tu implementación aquí

    let result = calculate_equity(state.hand,state.community_cards,state.opponents.len() as u8, iterations);

    if result.is_err() {
        return Err(result.err().unwrap());
    }

    let value_result = result?;

    if value_result > 0.5 {
        Ok(PokerAction::Bet(200.0))
    }else{
        Ok(PokerAction::Fold)
    }

}

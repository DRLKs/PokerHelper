// TODO: Implementar lógica de toma de decisiones

use crate::game_state::{CvState, PokerAction};

pub fn make_decision(state: CvState) -> Result<PokerAction, String> {
    // Tu implementación aquí
    Ok(PokerAction::Check)
}

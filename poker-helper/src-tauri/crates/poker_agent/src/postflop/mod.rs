pub mod decision;
pub mod narrowing;
pub mod opponent_range;
pub mod reading;
pub mod texture;

pub use decision::{advise, PostflopStreet};
pub use narrowing::{narrow, Narrowed, Narrowing};
pub use opponent_range::estimate_ranges;
pub use reading::{HandCategory, HandReading};
pub use texture::{BoardTexture, FlushThreat};

use crate::core::card::Card;
use crate::evaluator::fast_eval::{card_bit, CardMask};

/// Mask of cards that [`TableState::new`](crate::table::TableState::new)
/// has already validated.
pub(crate) fn validated_cards_mask(cards: &[Card]) -> CardMask {
    cards.iter().fold(0, |mask, card| {
        let bit = card_bit(card).expect("TableState::new rejects invalid cards");
        mask | (1u64 << bit)
    })
}

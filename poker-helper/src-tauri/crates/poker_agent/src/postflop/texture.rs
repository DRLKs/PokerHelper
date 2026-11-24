use crate::evaluator::fast_eval::CardMask;

const SUIT_COUNT: u32 = 4;
const SUIT_LANE_WIDTH: u32 = 16;
const RANK_LANE: u64 = 0x1FFF;
const STRAIGHT_LENGTH: u32 = 5;
const STRAIGHT_WINDOW: u32 = 0b1_1111;
const LOWEST_STRAIGHT_WINDOW_START: u32 = 0;
const HIGHEST_STRAIGHT_WINDOW_START: u32 = 9;
const CONNECTED_RANKS_IN_WINDOW: u32 = 3;

/// How close the board is to making a flush.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlushThreat {
    /// No two cards of the same suit.
    NoFlushDraw,
    /// Two cards of one suit: a flush draw on the flop or turn.
    FlushDraw,
    /// Three or more cards of one suit: a flush is already possible.
    FlushPossible,
}

/// The features of the board that change how often and how big to bet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardTexture {
    pub flush: FlushThreat,
    pub paired: bool,
    /// Three or more board ranks fit inside one straight.
    pub connected: bool,
}

impl BoardTexture {
    pub fn of(board: CardMask) -> BoardTexture {
        let suit_lanes =
            (0..SUIT_COUNT).map(|suit| (board >> (suit * SUIT_LANE_WIDTH)) & RANK_LANE);
        let most_cards_of_one_suit = suit_lanes.clone().map(u64::count_ones).max().unwrap_or(0);
        let ranks = suit_lanes.fold(0, |ranks, lane| ranks | lane) as u32;
        let flush = match most_cards_of_one_suit {
            0 | 1 => FlushThreat::NoFlushDraw,
            2 => FlushThreat::FlushDraw,
            _ => FlushThreat::FlushPossible,
        };
        BoardTexture {
            flush,
            paired: ranks.count_ones() < board.count_ones(),
            connected: most_ranks_in_one_straight(ranks) >= CONNECTED_RANKS_IN_WINDOW,
        }
    }

    /// A board where many hands have draws, so equities run close and bets
    /// should be bigger.
    pub fn is_wet(self) -> bool {
        let flush_draws_exist = match self.flush {
            FlushThreat::NoFlushDraw => false,
            FlushThreat::FlushDraw | FlushThreat::FlushPossible => true,
        };
        self.connected || flush_draws_exist
    }
}

fn most_ranks_in_one_straight(ranks: u32) -> u32 {
    let ranks_with_low_ace = (ranks << 1) | (ranks >> 12);
    (LOWEST_STRAIGHT_WINDOW_START..=HIGHEST_STRAIGHT_WINDOW_START)
        .map(|lowest| ((ranks_with_low_ace >> lowest) & STRAIGHT_WINDOW).count_ones())
        .max()
        .unwrap_or(0)
        .min(STRAIGHT_LENGTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postflop::validated_cards_mask;
    use crate::table::state::tests::cards;

    fn texture(board: &str) -> BoardTexture {
        BoardTexture::of(validated_cards_mask(&cards(board)))
    }

    #[test]
    fn a_rainbow_ace_high_board_is_dry() {
        let board = texture("Ah 7d 2c");
        assert_eq!(board.flush, FlushThreat::NoFlushDraw);
        assert!(!board.paired);
        assert!(!board.connected);
        assert!(!board.is_wet());
    }

    #[test]
    fn a_two_tone_connected_board_is_wet() {
        let board = texture("9h 8h 7c");
        assert_eq!(board.flush, FlushThreat::FlushDraw);
        assert!(board.connected);
        assert!(board.is_wet());
    }

    #[test]
    fn the_ace_counts_low_for_straights() {
        assert!(texture("Ah 2d 4c").connected);
        assert!(!texture("Ah 6d Tc").connected);
    }

    #[test]
    fn recognises_paired_and_monotone_boards() {
        assert!(texture("Kh Kd 4c").paired);
        assert_eq!(texture("Kh 9h 4h").flush, FlushThreat::FlushPossible);
    }
}

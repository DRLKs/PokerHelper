use crate::evaluator::fast_eval::{
    category, evaluate, CardMask, FLUSH, FOUR_OF_A_KIND, FULL_HOUSE, HIGH_CARD, PAIR, STRAIGHT,
    STRAIGHT_FLUSH, THREE_OF_A_KIND, TWO_PAIR,
};
use crate::range::CardIndex;

const FULL_BOARD: u32 = 5;
const RANK_COUNT: usize = 13;
const LOWEST_RANK: u8 = 2;

/// Poker hand categories from weakest to strongest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandCategory {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl HandCategory {
    /// Category of a strength returned by [`evaluate`].
    pub fn of(strength: u32) -> HandCategory {
        match category(strength) {
            HIGH_CARD => HandCategory::HighCard,
            PAIR => HandCategory::Pair,
            TWO_PAIR => HandCategory::TwoPair,
            THREE_OF_A_KIND => HandCategory::ThreeOfAKind,
            STRAIGHT => HandCategory::Straight,
            FLUSH => HandCategory::Flush,
            FULL_HOUSE => HandCategory::FullHouse,
            FOUR_OF_A_KIND => HandCategory::FourOfAKind,
            STRAIGHT_FLUSH => HandCategory::StraightFlush,
            other => {
                unreachable!("evaluate returns categories up to a straight flush, got {other}")
            }
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            HandCategory::HighCard => "no pair",
            HandCategory::Pair => "a pair",
            HandCategory::TwoPair => "two pair",
            HandCategory::ThreeOfAKind => "three of a kind",
            HandCategory::Straight => "a straight",
            HandCategory::Flush => "a flush",
            HandCategory::FullHouse => "a full house",
            HandCategory::FourOfAKind => "four of a kind",
            HandCategory::StraightFlush => "a straight flush",
        }
    }
}

/// What two hole cards make on a board of 3 to 5 cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandReading {
    pub category: HandCategory,
    /// The hole cards add nothing to the pair, trips or better that the
    /// board shows by itself (on the river, nothing at all).
    pub plays_the_board: bool,
    /// Unseen cards that would make a straight or better on the next street.
    pub outs: u8,
}

impl HandReading {
    pub fn of(hole: CardMask, board: CardMask) -> HandReading {
        let strength = evaluate(hole | board);
        HandReading {
            category: HandCategory::of(strength),
            plays_the_board: plays_the_board(strength, board),
            outs: outs(hole, board),
        }
    }

    /// A short phrase such as `a pair and 9 outs to a straight or better`.
    pub fn describe(self) -> String {
        let made_hand = if self.plays_the_board {
            "only what the board shows"
        } else {
            self.category.description()
        };
        match self.outs {
            0 => made_hand.to_string(),
            outs => format!("{made_hand} and {outs} outs to a straight or better"),
        }
    }
}

/// Unseen cards that would give `hole` a straight or better, beating its
/// current category and, on a five-card board, the board itself. Zero on the
/// river.
pub fn outs(hole: CardMask, board: CardMask) -> u8 {
    if board.count_ones() >= FULL_BOARD {
        return 0;
    }
    let used = hole | board;
    let current = HandCategory::of(evaluate(used));
    let completing_cards = CardIndex::all()
        .map(CardIndex::mask)
        .filter(|&card| card & used == 0)
        .filter(|&card| completes_a_draw(hole, board | card, current))
        .count();
    completing_cards as u8
}

fn completes_a_draw(hole: CardMask, board: CardMask, current: HandCategory) -> bool {
    let improved = evaluate(hole | board);
    let improved_category = HandCategory::of(improved);
    let beats_the_board = board.count_ones() < FULL_BOARD || improved > evaluate(board);
    improved_category >= HandCategory::Straight && improved_category > current && beats_the_board
}

fn plays_the_board(strength: u32, board: CardMask) -> bool {
    if board.count_ones() >= FULL_BOARD {
        return strength == evaluate(board);
    }
    let category = HandCategory::of(strength);
    category != HandCategory::HighCard && category == category_of_board_ranks(board)
}

/// Category made by repeated ranks on a board too short to evaluate.
fn category_of_board_ranks(board: CardMask) -> HandCategory {
    let mut rank_counts = [0u8; RANK_COUNT];
    for card in CardIndex::all().filter(|card| card.mask() & board != 0) {
        rank_counts[(card.rank() - LOWEST_RANK) as usize] += 1;
    }
    let pairs = rank_counts.iter().filter(|&&count| count == 2).count();
    let largest_group = rank_counts.iter().copied().max().unwrap_or(0);
    match (largest_group, pairs) {
        (4, _) => HandCategory::FourOfAKind,
        (3, 0) => HandCategory::ThreeOfAKind,
        (3, _) => HandCategory::FullHouse,
        (_, 2) => HandCategory::TwoPair,
        (_, 1) => HandCategory::Pair,
        _ => HandCategory::HighCard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postflop::validated_cards_mask;
    use crate::table::state::tests::cards;

    fn reading(hole: &str, board: &str) -> HandReading {
        HandReading::of(
            validated_cards_mask(&cards(hole)),
            validated_cards_mask(&cards(board)),
        )
    }

    #[test]
    fn counts_the_outs_of_common_draws() {
        assert_eq!(reading("Ah Kh", "7h 2h Qc").outs, 9);
        assert_eq!(reading("9c 8d", "7h 6s 2c").outs, 8);
        assert_eq!(reading("9c 8d", "Jh 7s 2c").outs, 4);
        assert_eq!(reading("9h 8h", "7h 6h 2c").outs, 15);
        assert_eq!(reading("Ah Kd", "7c 7s 2d").outs, 0);
        assert_eq!(reading("Ah Kh", "7h 2h Qc 3d 9s").outs, 0);
    }

    #[test]
    fn a_card_that_makes_a_straight_on_the_board_is_not_an_out() {
        assert_eq!(reading("Ac Ad", "5h 6s 7c 8d").outs, 0);
    }

    #[test]
    fn tells_when_the_hole_cards_add_nothing() {
        let board_pair = reading("Ah Kd", "7c 7s 2d");
        assert_eq!(board_pair.category, HandCategory::Pair);
        assert!(board_pair.plays_the_board);
        assert_eq!(board_pair.describe(), "only what the board shows");

        let trips = reading("Ah 7d", "7c 7s 2d");
        assert_eq!(trips.category, HandCategory::ThreeOfAKind);
        assert!(!trips.plays_the_board);

        assert!(reading("2c 3d", "Th Jh Qh Kh Ah").plays_the_board);
        assert!(!reading("9h 3d", "Th Jh Qh Kh 2c").plays_the_board);
    }

    #[test]
    fn describes_made_hands_and_draws() {
        assert_eq!(
            reading("Ah Kh", "7h 2h Kc").describe(),
            "a pair and 9 outs to a straight or better"
        );
        assert_eq!(reading("Ah Kd", "Qh 7c 2s").describe(), "no pair");
        assert_eq!(
            reading("Ah Ad", "7h 7c 2s").describe(),
            "two pair and 4 outs to a straight or better"
        );
    }
}

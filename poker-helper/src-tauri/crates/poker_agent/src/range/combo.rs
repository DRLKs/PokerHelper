use crate::core::card::{rank_symbol, Card};
use crate::evaluator::fast_eval::{card_bit, CardMask};
use std::fmt;

pub const DECK_SIZE: usize = 52;
/// Number of distinct two-card hands: 52 choose 2.
pub const COMBO_COUNT: usize = 1326;
/// Number of starting-hand classes: 13 pairs, 78 suited and 78 offsuit hands.
pub const HAND_CLASS_COUNT: usize = 169;

const RANKS_PER_SUIT: u8 = 13;
const SUIT_COUNT: u8 = 4;
const LOWEST_RANK: u8 = 2;
pub const ACE: u8 = 14;

/// A card numbered `0..52` as `suit * 13 + (rank - 2)`, with suits in the
/// order of the hand evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardIndex(u8);

impl CardIndex {
    pub fn of(card: &Card) -> Option<CardIndex> {
        card_bit(card).map(|bit| CardIndex((bit / 16) * RANKS_PER_SUIT + bit % 16))
    }

    pub fn all() -> impl Iterator<Item = CardIndex> {
        (0..DECK_SIZE as u8).map(CardIndex)
    }

    fn from_rank_and_suit(rank: u8, suit: u8) -> CardIndex {
        CardIndex(suit * RANKS_PER_SUIT + rank - LOWEST_RANK)
    }

    pub fn rank(self) -> u8 {
        self.0 % RANKS_PER_SUIT + LOWEST_RANK
    }

    pub fn suit(self) -> u8 {
        self.0 / RANKS_PER_SUIT
    }

    pub fn mask(self) -> CardMask {
        1u64 << (self.suit() as u32 * 16 + (self.rank() - LOWEST_RANK) as u32)
    }
}

/// Two distinct cards, stored with the lower [`CardIndex`] first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Combo {
    first: CardIndex,
    second: CardIndex,
}

impl Combo {
    /// `None` when both cards are the same.
    pub fn new(one: CardIndex, other: CardIndex) -> Option<Combo> {
        match one.cmp(&other) {
            std::cmp::Ordering::Less => Some(Combo {
                first: one,
                second: other,
            }),
            std::cmp::Ordering::Greater => Some(Combo {
                first: other,
                second: one,
            }),
            std::cmp::Ordering::Equal => None,
        }
    }

    pub fn of(cards: [Card; 2]) -> Option<Combo> {
        Combo::new(CardIndex::of(&cards[0])?, CardIndex::of(&cards[1])?)
    }

    pub fn all() -> impl Iterator<Item = Combo> {
        CardIndex::all().flat_map(|first| {
            (first.0 + 1..DECK_SIZE as u8).map(move |second| Combo {
                first,
                second: CardIndex(second),
            })
        })
    }

    /// Position in `0..COMBO_COUNT`, in the same order as [`Combo::all`].
    pub fn index(self) -> usize {
        let first = self.first.0 as usize;
        let second = self.second.0 as usize;
        let combos_before_first = first * (2 * DECK_SIZE - 1 - first) / 2;
        combos_before_first + (second - first - 1)
    }

    pub fn cards(self) -> [CardIndex; 2] {
        [self.first, self.second]
    }

    pub fn mask(self) -> CardMask {
        self.first.mask() | self.second.mask()
    }

    pub fn hand_class(self) -> HandClass {
        HandClass::of_cards(self.first, self.second)
    }
}

/// One of the 169 starting-hand classes, such as `AA`, `AKs` or `T9o`.
/// Ranks go from 2 to 14 (ace) and `high > low` for non-pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandClass {
    Pair { rank: u8 },
    Suited { high: u8, low: u8 },
    Offsuit { high: u8, low: u8 },
}

impl HandClass {
    pub fn of_cards(one: CardIndex, other: CardIndex) -> HandClass {
        let high = one.rank().max(other.rank());
        let low = one.rank().min(other.rank());
        if high == low {
            HandClass::Pair { rank: high }
        } else if one.suit() == other.suit() {
            HandClass::Suited { high, low }
        } else {
            HandClass::Offsuit { high, low }
        }
    }

    pub fn of(cards: [Card; 2]) -> Option<HandClass> {
        Combo::of(cards).map(Combo::hand_class)
    }

    pub fn all() -> impl Iterator<Item = HandClass> {
        let ranks = LOWEST_RANK..=ACE;
        ranks.flat_map(|high| {
            (LOWEST_RANK..=high).flat_map(move |low| {
                if high == low {
                    vec![HandClass::Pair { rank: high }]
                } else {
                    vec![
                        HandClass::Suited { high, low },
                        HandClass::Offsuit { high, low },
                    ]
                }
            })
        })
    }

    /// Every combo of this class.
    pub fn combos(self) -> impl Iterator<Item = Combo> {
        let (high, low) = self.ranks();
        let suits = 0..SUIT_COUNT;
        suits
            .flat_map(move |high_suit| (0..SUIT_COUNT).map(move |low_suit| (high_suit, low_suit)))
            .filter(move |&(high_suit, low_suit)| match self {
                HandClass::Pair { .. } => high_suit < low_suit,
                HandClass::Suited { .. } => high_suit == low_suit,
                HandClass::Offsuit { .. } => high_suit != low_suit,
            })
            .filter_map(move |(high_suit, low_suit)| {
                Combo::new(
                    CardIndex::from_rank_and_suit(high, high_suit),
                    CardIndex::from_rank_and_suit(low, low_suit),
                )
            })
    }

    pub fn combo_count(self) -> usize {
        match self {
            HandClass::Pair { .. } => 6,
            HandClass::Suited { .. } => 4,
            HandClass::Offsuit { .. } => 12,
        }
    }

    /// `(high, low)` ranks; equal for a pair.
    pub fn ranks(self) -> (u8, u8) {
        match self {
            HandClass::Pair { rank } => (rank, rank),
            HandClass::Suited { high, low } | HandClass::Offsuit { high, low } => (high, low),
        }
    }
}

impl fmt::Display for HandClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (high, low) = self.ranks();
        let suffix = match self {
            HandClass::Pair { .. } => "",
            HandClass::Suited { .. } => "s",
            HandClass::Offsuit { .. } => "o",
        };
        write!(
            formatter,
            "{}{}{}",
            rank_symbol(high),
            rank_symbol(low),
            suffix
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn combo_indexes_are_a_bijection_onto_zero_to_1325() {
        let indexes: Vec<usize> = Combo::all().map(Combo::index).collect();
        assert_eq!(indexes.len(), COMBO_COUNT);
        assert!(indexes
            .iter()
            .enumerate()
            .all(|(position, &index)| position == index));
    }

    #[test]
    fn card_index_matches_the_evaluator_bit() {
        for card in crate::r#const::r#const::ALL_CARDS {
            let index = CardIndex::of(&card).unwrap();
            assert_eq!(index.mask(), 1u64 << card_bit(&card).unwrap());
            assert_eq!(index.rank(), crate::core::card::CardTrait::get_rank(&card));
        }
    }

    #[test]
    fn there_are_169_classes_covering_every_combo_once() {
        let classes: Vec<HandClass> = HandClass::all().collect();
        assert_eq!(classes.len(), HAND_CLASS_COUNT);
        let mut seen = HashSet::new();
        for class in classes {
            let combos: Vec<Combo> = class.combos().collect();
            assert_eq!(combos.len(), class.combo_count(), "{class}");
            for combo in combos {
                assert_eq!(combo.hand_class(), class);
                assert!(seen.insert(combo));
            }
        }
        assert_eq!(seen.len(), COMBO_COUNT);
    }

    #[test]
    fn names_classes_in_standard_notation() {
        let ace_king_suited =
            HandClass::of(["Ah".parse().unwrap(), "Kh".parse().unwrap()]).unwrap();
        assert_eq!(ace_king_suited.to_string(), "AKs");
        let ten_nine_offsuit =
            HandClass::of(["9c".parse().unwrap(), "Td".parse().unwrap()]).unwrap();
        assert_eq!(ten_nine_offsuit.to_string(), "T9o");
        assert_eq!(HandClass::Pair { rank: 7 }.to_string(), "77");
    }
}

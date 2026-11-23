//! Allocation-free 7-card hand evaluator based on bitmasks.
//!
//! A set of cards is a `u64` where each suit owns a 16-bit lane and the
//! bit inside the lane is the rank (`rank - 2`, so bit 0 = deuce, bit 12 = ace).
//! [`evaluate`] returns a `u32` strength: a higher value always means a
//! stronger hand and equal values mean a split pot.
//!
//! Layout of the strength value:
//! `category << 26 | primary_ranks << 13 | kicker_ranks`
//! where both rank fields are 13-bit rank masks, which compare correctly as
//! plain integers (the highest differing rank decides).

use crate::core::card::{Card, CardTrait, CLUBS, DIAMONDS, HEARTS, SPADES};

/// Bitmask of a set of cards (see module docs for the layout).
pub type CardMask = u64;

pub const HIGH_CARD: u32 = 0;
pub const PAIR: u32 = 1;
pub const TWO_PAIR: u32 = 2;
pub const THREE_OF_A_KIND: u32 = 3;
pub const STRAIGHT: u32 = 4;
pub const FLUSH: u32 = 5;
pub const FULL_HOUSE: u32 = 6;
pub const FOUR_OF_A_KIND: u32 = 7;
pub const STRAIGHT_FLUSH: u32 = 8;

const CATEGORY_SHIFT: u32 = 26;
const PRIMARY_SHIFT: u32 = 13;
const RANK_LANE: u64 = 0x1FFF;

/// Bit index (0..64) of a card inside a [`CardMask`], or `None` if the card
/// has an invalid suit or rank.
#[inline]
pub fn card_bit(card: &Card) -> Option<u8> {
    let suit_idx: u8 = match card.get_suit() {
        HEARTS => 0,
        DIAMONDS => 1,
        CLUBS => 2,
        SPADES => 3,
        _ => return None,
    };
    let rank = card.get_rank();
    if !(2..=14).contains(&rank) {
        return None;
    }
    Some(suit_idx * 16 + (rank - 2))
}

/// Category (`HIGH_CARD` ..= `STRAIGHT_FLUSH`) of a strength value.
#[allow(dead_code)]
#[inline]
pub fn category(strength: u32) -> u32 {
    strength >> CATEGORY_SHIFT
}

#[inline(always)]
fn highest_bit(mask: u32) -> u32 {
    debug_assert!(mask != 0);
    1 << (31 - mask.leading_zeros())
}

/// Keeps only the `n` highest set bits of `mask`.
#[inline(always)]
fn keep_top(mut mask: u32, n: u32) -> u32 {
    while mask.count_ones() > n {
        mask &= mask - 1; // clear lowest set bit
    }
    mask
}

/// Returns a non-zero, monotonic value identifying the highest straight in
/// `ranks` (13-bit rank mask), or 0 if there is none. The wheel (A-2-3-4-5)
/// is the lowest straight.
#[inline(always)]
fn straight_high(ranks: u32) -> u32 {
    // Shift up one bit and copy the ace into bit 0 so it can play low.
    let r = (ranks << 1) | (ranks >> 12);
    let runs = r & (r << 1) & (r << 2) & (r << 3) & (r << 4);
    if runs == 0 {
        0
    } else {
        // Index of the top card (4 = wheel .. 13 = broadway); kept as an index
        // rather than a bit so it fits in the 13-bit primary field.
        31 - runs.leading_zeros()
    }
}

#[inline(always)]
fn make(category: u32, primary: u32, kickers: u32) -> u32 {
    (category << CATEGORY_SHIFT) | (primary << PRIMARY_SHIFT) | kickers
}

/// Evaluates the best 5-card hand contained in `cards` (5 to 7 cards).
#[inline]
pub fn evaluate(cards: CardMask) -> u32 {
    let h = (cards & RANK_LANE) as u32;
    let d = ((cards >> 16) & RANK_LANE) as u32;
    let c = ((cards >> 32) & RANK_LANE) as u32;
    let s = ((cards >> 48) & RANK_LANE) as u32;

    // With 7 cards a flush excludes quads and full house, so it can be
    // resolved first.
    for suited in [h, d, c, s] {
        if suited.count_ones() >= 5 {
            let sf = straight_high(suited);
            return if sf != 0 {
                make(STRAIGHT_FLUSH, sf, 0)
            } else {
                make(FLUSH, keep_top(suited, 5), 0)
            };
        }
    }

    let ranks = h | d | c | s;
    let quads = h & d & c & s;
    let trips_plus = (h & d & c) | (h & d & s) | (h & c & s) | (d & c & s);
    let pairs_plus = (h & d) | (h & c) | (h & s) | (d & c) | (d & s) | (c & s);

    if quads != 0 {
        let q = highest_bit(quads);
        return make(FOUR_OF_A_KIND, q, keep_top(ranks & !q, 1));
    }

    let trips = trips_plus;
    let pairs = pairs_plus & !trips_plus;

    if trips != 0 {
        let t = highest_bit(trips);
        let rest = (trips & !t) | pairs;
        if rest != 0 {
            return make(FULL_HOUSE, t, highest_bit(rest));
        }
    }

    let st = straight_high(ranks);
    if st != 0 {
        return make(STRAIGHT, st, 0);
    }

    if trips != 0 {
        let t = highest_bit(trips);
        return make(THREE_OF_A_KIND, t, keep_top(ranks & !t, 2));
    }

    match pairs.count_ones() {
        0 => make(HIGH_CARD, 0, keep_top(ranks, 5)),
        1 => make(PAIR, pairs, keep_top(ranks & !pairs, 3)),
        _ => {
            let top_two = keep_top(pairs, 2);
            make(TWO_PAIR, top_two, keep_top(ranks & !top_two, 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    /// Parses "Ah Kd 10c ..." / "As Td" into a mask.
    fn m(s: &str) -> CardMask {
        s.split_whitespace()
            .map(|c| {
                let (r, suit) = c.split_at(c.len() - 1);
                let rank = match r {
                    "A" => 14,
                    "K" => 13,
                    "Q" => 12,
                    "J" => 11,
                    "T" => 10,
                    n => n.parse().unwrap(),
                };
                1u64 << card_bit(&Card::new(suit.chars().next().unwrap(), rank)).unwrap()
            })
            .fold(0, |a, b| {
                assert_eq!(a & b, 0, "duplicate card");
                a | b
            })
    }

    fn cat(s: &str) -> u32 {
        category(evaluate(m(s)))
    }

    #[test]
    fn categories() {
        assert_eq!(cat("Ah Kd 9c 7s 5h 3d 2c"), HIGH_CARD);
        assert_eq!(cat("Ah Ad 9c 7s 5h 3d 2c"), PAIR);
        assert_eq!(cat("Ah Ad 9c 9s 5h 3d 2c"), TWO_PAIR);
        assert_eq!(cat("Ah Ad Ac 9s 5h 3d 2c"), THREE_OF_A_KIND);
        assert_eq!(cat("Ah 2d 3c 4s 5h 9d Kc"), STRAIGHT);
        assert_eq!(cat("Ah 2h 7h 4h 9h 9d Kc"), FLUSH);
        assert_eq!(cat("Ah Ad Ac 9s 9h 3d 2c"), FULL_HOUSE);
        assert_eq!(cat("Ah Ad Ac As 9h 3d 2c"), FOUR_OF_A_KIND);
        assert_eq!(cat("5h 6h 7h 8h 9h 9d Kc"), STRAIGHT_FLUSH);
        assert_eq!(cat("Th Jh Qh Kh Ah 9d Kc"), STRAIGHT_FLUSH);
    }

    #[test]
    fn wheel_is_lowest_straight() {
        assert!(evaluate(m("Ah 2d 3c 4s 5h Kd Qc")) < evaluate(m("2d 3c 4s 5h 6d Kd Qc")));
        assert!(evaluate(m("Ah 2h 3h 4h 5h Kd Qc")) < evaluate(m("2h 3h 4h 5h 6h Kd Qc")));
    }

    #[test]
    fn two_trips_make_full_house_with_highest_trips() {
        assert!(evaluate(m("Ah Ad Ac Ks Kh Kd 2c")) > evaluate(m("Qh Qd Qc Ks Kh 3d 2c")));
        assert_eq!(evaluate(m("Ah Ad Ac Ks Kh Kd 2c")), evaluate(m("Ah Ad Ac Ks Kh 7d 2c")));
    }

    #[test]
    fn kickers_and_splits() {
        // Same pair, kicker decides
        assert!(evaluate(m("Ah Ad Kc 7s 5h 3d 2c")) > evaluate(m("Ah Ad Qc 7s 5h 3d 2c")));
        // Only the best five cards count: the 6th/7th cards do not break ties
        assert_eq!(evaluate(m("Ah Kd Qc Js 9h 3d 2c")), evaluate(m("Ah Kd Qc Js 9h 4d 3c")));
        // Three pairs: third pair can be the kicker
        assert!(evaluate(m("Ah Ad Kc Ks Qh Qd 2c")) > evaluate(m("Ah Ad Kc Ks Jh Jd 2c")));
        // Better flush wins
        assert!(evaluate(m("Ah 2h 7h 4h 9h")) > evaluate(m("Kh Qh Jh 9h 7h")));
    }

    // ---- Reference (brute-force) evaluator used for cross-checking ----

    fn naive_five(cards: &[(u8, u8)]) -> (u32, Vec<u8>) {
        let mut ranks: Vec<u8> = cards.iter().map(|c| c.0).collect();
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        let flush = cards.iter().all(|c| c.1 == cards[0].1);
        let mut uniq = ranks.clone();
        uniq.dedup();
        let straight_top = if uniq.len() == 5 && uniq[0] - uniq[4] == 4 {
            Some(uniq[0])
        } else if uniq == vec![14, 5, 4, 3, 2] {
            Some(5)
        } else {
            None
        };
        // groups sorted by (count desc, rank desc)
        let mut groups: Vec<(u8, u8)> = uniq
            .iter()
            .map(|&r| (ranks.iter().filter(|&&x| x == r).count() as u8, r))
            .collect();
        groups.sort_unstable_by(|a, b| b.cmp(a));
        let order: Vec<u8> = groups.iter().map(|g| g.1).collect();
        let counts: Vec<u8> = groups.iter().map(|g| g.0).collect();
        let cat = match (straight_top, flush, counts.as_slice()) {
            (Some(_), true, _) => STRAIGHT_FLUSH,
            (_, _, [4, 1]) => FOUR_OF_A_KIND,
            (_, _, [3, 2]) => FULL_HOUSE,
            (_, true, _) => FLUSH,
            (Some(_), _, _) => STRAIGHT,
            (_, _, [3, 1, 1]) => THREE_OF_A_KIND,
            (_, _, [2, 2, 1]) => TWO_PAIR,
            (_, _, [2, 1, 1, 1]) => PAIR,
            _ => HIGH_CARD,
        };
        match straight_top {
            Some(t) if cat == STRAIGHT || cat == STRAIGHT_FLUSH => (cat, vec![t]),
            _ => (cat, order),
        }
    }

    fn naive_seven(cards: &[(u8, u8)]) -> (u32, Vec<u8>) {
        let mut best = (0, vec![]);
        for a in 0..7 {
            for b in (a + 1)..7 {
                let five: Vec<(u8, u8)> = (0..7)
                    .filter(|&i| i != a && i != b)
                    .map(|i| cards[i])
                    .collect();
                let v = naive_five(&five);
                if v > best {
                    best = v;
                }
            }
        }
        best
    }

    #[test]
    fn matches_brute_force_on_random_hands() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
        let mut deck: Vec<(u8, u8)> = (0..4u8)
            .flat_map(|s| (2..=14u8).map(move |r| (r, s)))
            .collect();
        let to_mask = |cs: &[(u8, u8)]| -> CardMask {
            cs.iter()
                .map(|&(r, s)| 1u64 << (s as u64 * 16 + (r - 2) as u64))
                .sum()
        };
        for _ in 0..20_000 {
            deck.shuffle(&mut rng);
            let (a, b) = (&deck[0..7], &deck[7..14]);
            let (fa, fb) = (evaluate(to_mask(a)), evaluate(to_mask(b)));
            let (na, nb) = (naive_seven(a), naive_seven(b));
            assert_eq!(category(fa), na.0, "category mismatch for {:?}", a);
            assert_eq!(fa.cmp(&fb), na.cmp(&nb), "order mismatch {:?} vs {:?}", a, b);
        }
    }
}

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
        mask = without_lowest_bit(mask);
    }
    mask
}

#[inline(always)]
fn without_lowest_bit(mask: u32) -> u32 {
    mask & (mask - 1)
}

/// Returns the index of the top card of the highest straight in `ranks`
/// (13-bit rank mask), from 4 for the wheel (A-2-3-4-5) to 13 for broadway,
/// or 0 if there is none. An index rather than a bit keeps the value inside
/// the 13-bit primary field.
#[inline(always)]
fn straight_high(ranks: u32) -> u32 {
    let ranks_with_low_ace = (ranks << 1) | (ranks >> 12);
    let straight_tops = ranks_with_low_ace
        & (ranks_with_low_ace << 1)
        & (ranks_with_low_ace << 2)
        & (ranks_with_low_ace << 3)
        & (ranks_with_low_ace << 4);
    if straight_tops == 0 {
        0
    } else {
        31 - straight_tops.leading_zeros()
    }
}

#[inline(always)]
fn make(category: u32, primary: u32, kickers: u32) -> u32 {
    (category << CATEGORY_SHIFT) | (primary << PRIMARY_SHIFT) | kickers
}

/// Evaluates the best 5-card hand contained in `cards` (5 to 7 cards).
///
/// Flushes are resolved first: with at most 7 cards, a flush leaves too few
/// cards for four of a kind or a full house.
#[inline]
pub fn evaluate(cards: CardMask) -> u32 {
    let hearts = (cards & RANK_LANE) as u32;
    let diamonds = ((cards >> 16) & RANK_LANE) as u32;
    let clubs = ((cards >> 32) & RANK_LANE) as u32;
    let spades = ((cards >> 48) & RANK_LANE) as u32;

    for suited in [hearts, diamonds, clubs, spades] {
        if suited.count_ones() >= 5 {
            let straight_flush_top = straight_high(suited);
            return if straight_flush_top != 0 {
                make(STRAIGHT_FLUSH, straight_flush_top, 0)
            } else {
                make(FLUSH, keep_top(suited, 5), 0)
            };
        }
    }

    let ranks = hearts | diamonds | clubs | spades;
    let quads = hearts & diamonds & clubs & spades;
    let trips = (hearts & diamonds & clubs)
        | (hearts & diamonds & spades)
        | (hearts & clubs & spades)
        | (diamonds & clubs & spades);
    let pairs_or_better = (hearts & diamonds)
        | (hearts & clubs)
        | (hearts & spades)
        | (diamonds & clubs)
        | (diamonds & spades)
        | (clubs & spades);
    let pairs = pairs_or_better & !trips;

    if quads != 0 {
        let quad_rank = highest_bit(quads);
        return make(FOUR_OF_A_KIND, quad_rank, keep_top(ranks & !quad_rank, 1));
    }

    if trips != 0 {
        let trip_rank = highest_bit(trips);
        let full_house_pair_candidates = (trips & !trip_rank) | pairs;
        if full_house_pair_candidates != 0 {
            return make(FULL_HOUSE, trip_rank, highest_bit(full_house_pair_candidates));
        }
    }

    let straight_top = straight_high(ranks);
    if straight_top != 0 {
        return make(STRAIGHT, straight_top, 0);
    }

    if trips != 0 {
        let trip_rank = highest_bit(trips);
        return make(THREE_OF_A_KIND, trip_rank, keep_top(ranks & !trip_rank, 2));
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
    fn parse_cards(s: &str) -> CardMask {
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

    fn category_of(s: &str) -> u32 {
        category(evaluate(parse_cards(s)))
    }

    #[test]
    fn categories() {
        assert_eq!(category_of("Ah Kd 9c 7s 5h 3d 2c"), HIGH_CARD);
        assert_eq!(category_of("Ah Ad 9c 7s 5h 3d 2c"), PAIR);
        assert_eq!(category_of("Ah Ad 9c 9s 5h 3d 2c"), TWO_PAIR);
        assert_eq!(category_of("Ah Ad Ac 9s 5h 3d 2c"), THREE_OF_A_KIND);
        assert_eq!(category_of("Ah 2d 3c 4s 5h 9d Kc"), STRAIGHT);
        assert_eq!(category_of("Ah 2h 7h 4h 9h 9d Kc"), FLUSH);
        assert_eq!(category_of("Ah Ad Ac 9s 9h 3d 2c"), FULL_HOUSE);
        assert_eq!(category_of("Ah Ad Ac As 9h 3d 2c"), FOUR_OF_A_KIND);
        assert_eq!(category_of("5h 6h 7h 8h 9h 9d Kc"), STRAIGHT_FLUSH);
        assert_eq!(category_of("Th Jh Qh Kh Ah 9d Kc"), STRAIGHT_FLUSH);
    }

    #[test]
    fn wheel_is_lowest_straight() {
        assert!(evaluate(parse_cards("Ah 2d 3c 4s 5h Kd Qc")) < evaluate(parse_cards("2d 3c 4s 5h 6d Kd Qc")));
        assert!(evaluate(parse_cards("Ah 2h 3h 4h 5h Kd Qc")) < evaluate(parse_cards("2h 3h 4h 5h 6h Kd Qc")));
    }

    #[test]
    fn two_trips_make_full_house_with_highest_trips() {
        assert!(evaluate(parse_cards("Ah Ad Ac Ks Kh Kd 2c")) > evaluate(parse_cards("Qh Qd Qc Ks Kh 3d 2c")));
        assert_eq!(evaluate(parse_cards("Ah Ad Ac Ks Kh Kd 2c")), evaluate(parse_cards("Ah Ad Ac Ks Kh 7d 2c")));
    }

    #[test]
    fn kicker_decides_between_equal_pairs() {
        assert!(evaluate(parse_cards("Ah Ad Kc 7s 5h 3d 2c")) > evaluate(parse_cards("Ah Ad Qc 7s 5h 3d 2c")));
    }

    #[test]
    fn cards_outside_the_best_five_do_not_break_ties() {
        assert_eq!(evaluate(parse_cards("Ah Kd Qc Js 9h 3d 2c")), evaluate(parse_cards("Ah Kd Qc Js 9h 4d 3c")));
    }

    #[test]
    fn third_pair_can_play_as_kicker() {
        assert!(evaluate(parse_cards("Ah Ad Kc Ks Qh Qd 2c")) > evaluate(parse_cards("Ah Ad Kc Ks Jh Jd 2c")));
    }

    #[test]
    fn higher_flush_wins() {
        assert!(evaluate(parse_cards("Ah 2h 7h 4h 9h")) > evaluate(parse_cards("Kh Qh Jh 9h 7h")));
    }

    fn brute_force_five(cards: &[(u8, u8)]) -> (u32, Vec<u8>) {
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

    fn brute_force_seven(cards: &[(u8, u8)]) -> (u32, Vec<u8>) {
        let mut best = (0, vec![]);
        for a in 0..7 {
            for b in (a + 1)..7 {
                let five: Vec<(u8, u8)> = (0..7)
                    .filter(|&i| i != a && i != b)
                    .map(|i| cards[i])
                    .collect();
                let v = brute_force_five(&five);
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
            let (na, nb) = (brute_force_seven(a), brute_force_seven(b));
            assert_eq!(category(fa), na.0, "category mismatch for {:?}", a);
            assert_eq!(fa.cmp(&fb), na.cmp(&nb), "order mismatch {:?} vs {:?}", a, b);
        }
    }
}

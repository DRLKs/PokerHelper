use crate::evaluator::fast_eval::{evaluate, CardMask};
use crate::math::Probability;
use crate::postflop::reading::outs;
use crate::range::{Combo, Range};

/// Outs from which a draw is strong enough to keep betting or calling.
pub const STRONG_DRAW_OUTS: u8 = 8;

/// How much of a range is left after an action on a board: the strongest
/// `keep_share` of the combos by made hand, plus every combo below that cut
/// with a strong draw at `draw_weight` of its weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Narrowing {
    pub keep_share: f64,
    pub draw_weight: f32,
}

impl Narrowing {
    /// Starting values, to be tuned by simulation (phase 7 of the plan).
    pub const CALL: Narrowing = Narrowing {
        keep_share: 0.55,
        draw_weight: 1.0,
    };
    pub const BET: Narrowing = Narrowing {
        keep_share: 0.45,
        draw_weight: 1.0,
    };
    pub const RAISE: Narrowing = Narrowing {
        keep_share: 0.25,
        draw_weight: 0.5,
    };

    /// The same narrowing keeping `factor` times as much of the range, at
    /// most all of it.
    pub fn scaled(self, factor: f64) -> Narrowing {
        Narrowing {
            keep_share: (self.keep_share * factor).clamp(0.0, 1.0),
            draw_weight: self.draw_weight,
        }
    }

    /// The hands that keep playing against a bet: `share` of the range by
    /// made hand, plus strong draws.
    pub fn continuing(share: Probability) -> Narrowing {
        Narrowing {
            keep_share: share.fraction(),
            draw_weight: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Narrowed {
    pub range: Range,
    /// Weight of the narrowed range over the weight of the live input range.
    pub kept_share: Probability,
}

/// Cuts `range` down to the hands that take an action on `board`. Combos
/// that use a card in `dead_cards` or on the board are dropped first.
///
/// The cut always keeps at least the strongest live combo when the live
/// range is not empty.
pub fn narrow(
    range: &Range,
    board: CardMask,
    dead_cards: CardMask,
    narrowing: Narrowing,
) -> Narrowed {
    let dead_cards = dead_cards | board;
    let mut by_strength: Vec<(Combo, f32, u32)> = range
        .weighted_combos()
        .filter(|(combo, _)| combo.mask() & dead_cards == 0)
        .map(|(combo, weight)| (combo, weight, evaluate(board | combo.mask())))
        .collect();
    by_strength.sort_unstable_by(|left, right| right.2.cmp(&left.2));

    let live_weight: f64 = by_strength
        .iter()
        .map(|&(_, weight, _)| weight as f64)
        .sum();
    let target_weight = live_weight * narrowing.keep_share.clamp(0.0, 1.0);
    let mut narrowed = Range::empty();
    let mut kept_weight = 0.0;
    let mut weight_above_cut = 0.0;
    for &(combo, weight, _) in &by_strength {
        let kept = if weight_above_cut < target_weight {
            weight_above_cut += weight as f64;
            weight
        } else if narrowing.draw_weight > 0.0 && outs(combo.mask(), board) >= STRONG_DRAW_OUTS {
            weight * narrowing.draw_weight
        } else {
            continue;
        };
        narrowed.set_weight(combo, kept);
        kept_weight += kept as f64;
    }

    let kept_share = if live_weight > 0.0 {
        Probability::saturating(kept_weight / live_weight)
    } else {
        Probability::ZERO
    };
    Narrowed {
        range: narrowed,
        kept_share,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postflop::validated_cards_mask;
    use crate::table::state::tests::cards;

    fn mask(text: &str) -> CardMask {
        validated_cards_mask(&cards(text))
    }

    fn combo(text: &str) -> Combo {
        let parsed = cards(text);
        Combo::of([parsed[0], parsed[1]]).unwrap()
    }

    #[test]
    fn keeps_the_strongest_hands_and_the_strong_draws() {
        let board = mask("Kh 7h 2c");
        let range: Range = "AA, KK, 77, AKo, 98o, 65o, A5s".parse().unwrap();
        let narrowed = narrow(
            &range,
            board,
            0,
            Narrowing {
                keep_share: 0.2,
                draw_weight: 1.0,
            },
        );
        assert!(narrowed.range.weight(combo("Kd Ks")) > 0.0);
        assert!(narrowed.range.weight(combo("7d 7s")) > 0.0);
        assert_eq!(narrowed.range.weight(combo("9c 8d")), 0.0);
        assert_eq!(narrowed.range.weight(combo("6c 5d")), 0.0);
        assert_eq!(narrowed.range.weight(combo("Ah 5h")), 1.0);
        assert_eq!(narrowed.range.weight(combo("Ac 5c")), 0.0);
        assert!(narrowed.kept_share.fraction() < 0.4);
    }

    #[test]
    fn removes_dead_cards_and_measures_the_share_of_live_combos() {
        let board = mask("Kh 7h 2c");
        let range: Range = "AA, QQ".parse().unwrap();
        let narrowed = narrow(
            &range,
            board,
            mask("Ad Ac"),
            Narrowing::continuing(Probability::ONE),
        );
        assert_eq!(narrowed.range.combo_count(), 7.0);
        assert_eq!(narrowed.kept_share, Probability::ONE);

        let tenth = narrow(
            &range,
            board,
            mask("Ad Ac"),
            Narrowing::continuing(Probability::new(0.1).unwrap()),
        );
        assert_eq!(tenth.range.combo_count(), 1.0);
        assert_eq!(tenth.range.weight(combo("Ah As")), 1.0);
    }

    #[test]
    fn an_empty_range_stays_empty() {
        let narrowed = narrow(&Range::empty(), mask("Kh 7h 2c"), 0, Narrowing::CALL);
        assert_eq!(narrowed.range.combo_count(), 0.0);
        assert_eq!(narrowed.kept_share, Probability::ZERO);
    }
}

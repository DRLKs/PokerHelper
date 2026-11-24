use crate::evaluator::fast_eval::{evaluate, CardMask};
use crate::range::combo::{CardIndex, HandClass, COMBO_COUNT, DECK_SIZE};
use crate::range::hand_range::Range;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::sync::LazyLock;

const DEALS_PER_CLASS: u32 = 4_000;
const SEED: u64 = 0x005E_ED0F_0DD5;
const BOARD_AND_OPPONENT_CARDS: usize = 7;
const UNSEEN_CARDS: usize = DECK_SIZE - 2;

static STRENGTH_ORDER: LazyLock<Vec<HandClass>> =
    LazyLock::new(rank_by_equity_against_a_random_hand);

/// The 169 starting-hand classes from strongest to weakest, ranked by their
/// equity against one random hand. The ranking is computed once, with a
/// fixed seed, so it is the same on every run.
pub fn preflop_strength_order() -> &'static [HandClass] {
    &STRENGTH_ORDER
}

fn rank_by_equity_against_a_random_hand() -> Vec<HandClass> {
    let mut ranked: Vec<(HandClass, f64)> = HandClass::all()
        .collect::<Vec<_>>()
        .into_par_iter()
        .enumerate()
        .map(|(position, class)| {
            let seed = SEED.wrapping_add(position as u64);
            (class, equity_against_a_random_hand(class, seed))
        })
        .collect();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    ranked.into_iter().map(|(class, _)| class).collect()
}

fn equity_against_a_random_hand(class: HandClass, seed: u64) -> f64 {
    let hero = class
        .combos()
        .next()
        .expect("every hand class has at least one combo");
    let hero_mask = hero.mask();
    let mut unseen = [0 as CardMask; UNSEEN_CARDS];
    let unseen_masks = CardIndex::all()
        .map(CardIndex::mask)
        .filter(|&mask| mask & hero_mask == 0);
    for (slot, mask) in unseen.iter_mut().zip(unseen_masks) {
        *slot = mask;
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut won = 0.0;
    for _ in 0..DEALS_PER_CLASS {
        for drawn in 0..BOARD_AND_OPPONENT_CARDS {
            let pick = rng.gen_range(drawn..UNSEEN_CARDS);
            unseen.swap(drawn, pick);
        }
        let opponent = unseen[0] | unseen[1];
        let board = unseen[2..BOARD_AND_OPPONENT_CARDS]
            .iter()
            .fold(0, |mask, &card| mask | card);
        let hero_strength = evaluate(hero_mask | board);
        let opponent_strength = evaluate(opponent | board);
        won += match hero_strength.cmp(&opponent_strength) {
            std::cmp::Ordering::Greater => 1.0,
            std::cmp::Ordering::Equal => 0.5,
            std::cmp::Ordering::Less => 0.0,
        };
    }
    won / DEALS_PER_CLASS as f64
}

impl Range {
    /// This range resized to `factor` times its combos, capped at every hand.
    ///
    /// A larger range adds the strongest hands it does not hold yet; a
    /// smaller one keeps its strongest hands. Strength follows
    /// [`preflop_strength_order`], and the hand class at the boundary is
    /// held in part.
    pub fn rescaled(&self, factor: f64) -> Range {
        let current = self.combo_count();
        let target = (current * factor.max(0.0)).min(COMBO_COUNT as f64);
        if target >= current {
            self.grown_by(target - current)
        } else {
            self.cut_to(target)
        }
    }

    fn grown_by(&self, mut combos_to_add: f64) -> Range {
        let mut grown = self.clone();
        for &class in preflop_strength_order() {
            if combos_to_add <= 0.0 {
                break;
            }
            let missing: f64 = class
                .combos()
                .map(|combo| 1.0 - grown.weight(combo) as f64)
                .sum();
            if missing <= 0.0 {
                continue;
            }
            let share_added = (combos_to_add / missing).min(1.0);
            for combo in class.combos() {
                let weight = grown.weight(combo) as f64;
                grown.set_weight(combo, (weight + (1.0 - weight) * share_added) as f32);
            }
            combos_to_add -= missing * share_added;
        }
        grown
    }

    fn cut_to(&self, mut combos_to_keep: f64) -> Range {
        let mut cut = Range::empty();
        for &class in preflop_strength_order() {
            if combos_to_keep <= 0.0 {
                break;
            }
            let held: f64 = class.combos().map(|combo| self.weight(combo) as f64).sum();
            if held <= 0.0 {
                continue;
            }
            let share_kept = (combos_to_keep / held).min(1.0);
            for combo in class.combos() {
                cut.set_weight(combo, (self.weight(combo) as f64 * share_kept) as f32);
            }
            combos_to_keep -= held * share_kept;
        }
        cut
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::combo::HAND_CLASS_COUNT;

    fn class(text: &str) -> HandClass {
        let range: Range = text.parse().unwrap();
        HandClass::all()
            .find(|&candidate| range.class_weight(candidate) > 0.0)
            .unwrap()
    }

    fn rank_of(text: &str) -> usize {
        let wanted = class(text);
        preflop_strength_order()
            .iter()
            .position(|&candidate| candidate == wanted)
            .unwrap()
    }

    #[test]
    fn aces_are_strongest_and_seven_deuce_offsuit_is_near_the_bottom() {
        let order = preflop_strength_order();
        assert_eq!(order.len(), HAND_CLASS_COUNT);
        assert_eq!(rank_of("AA"), 0);
        assert!(rank_of("KK") < 3);
        assert!(rank_of("AKs") < rank_of("AKo"));
        assert!(rank_of("72o") > HAND_CLASS_COUNT - 6);
    }

    #[test]
    fn growing_adds_the_strongest_missing_hands() {
        let range: Range = "55-22".parse().unwrap();
        let grown = range.rescaled(2.0);
        assert!((grown.combo_count() - range.combo_count() * 2.0).abs() < 0.01);
        assert_eq!(grown.class_weight(class("AA")), 1.0);
        assert_eq!(grown.class_weight(class("22")), 1.0);
        assert_eq!(grown.class_weight(class("72o")), 0.0);
    }

    #[test]
    fn cutting_keeps_the_strongest_hands() {
        let range: Range = "22+,A2s+".parse().unwrap();
        let cut = range.rescaled(0.5);
        assert!((cut.combo_count() - range.combo_count() * 0.5).abs() < 0.01);
        assert_eq!(cut.class_weight(class("AA")), 1.0);
        assert_eq!(cut.class_weight(class("22")), 0.0);
    }

    #[test]
    fn growing_stops_at_every_hand() {
        let range: Range = "AA".parse().unwrap();
        assert_eq!(range.rescaled(5_000.0), Range::full());
    }
}

use crate::evaluator::fast_eval::CardMask;
use crate::math::Probability;
use crate::range::combo::{Combo, HandClass, ACE, COMBO_COUNT};
use std::fmt;
use std::str::FromStr;

/// The hands a player can hold, as a weight between 0 and 1 for each of the
/// 1326 two-card combos. A weight of 0.5 means the player holds that combo
/// half of the time they reach this spot.
#[derive(Clone, PartialEq)]
pub struct Range {
    weights: [f32; COMBO_COUNT],
}

impl Range {
    pub fn empty() -> Range {
        Range {
            weights: [0.0; COMBO_COUNT],
        }
    }

    /// Every combo with weight 1: a player whose cards are unknown.
    pub fn full() -> Range {
        Range {
            weights: [1.0; COMBO_COUNT],
        }
    }

    pub fn weight(&self, combo: Combo) -> f32 {
        self.weights[combo.index()]
    }

    /// Sets the weight of one combo, clamped to `0.0..=1.0`.
    pub fn set_weight(&mut self, combo: Combo, weight: f32) {
        self.weights[combo.index()] = weight.clamp(0.0, 1.0);
    }

    pub fn set_class(&mut self, class: HandClass, weight: f32) {
        for combo in class.combos() {
            self.set_weight(combo, weight);
        }
    }

    /// Average weight of the combos of `class`.
    pub fn class_weight(&self, class: HandClass) -> f32 {
        let total: f32 = class.combos().map(|combo| self.weight(combo)).sum();
        total / class.combo_count() as f32
    }

    /// Sum of all weights: the number of combos the range holds.
    pub fn combo_count(&self) -> f64 {
        self.weights.iter().map(|&weight| weight as f64).sum()
    }

    /// Share of all starting hands this range plays.
    pub fn share_of_all_hands(&self) -> Probability {
        Probability::saturating(self.combo_count() / COMBO_COUNT as f64)
    }

    /// The same range without the combos that use any card in `dead_cards`.
    pub fn without_cards(&self, dead_cards: CardMask) -> Range {
        let mut live = self.clone();
        for combo in Combo::all().filter(|combo| combo.mask() & dead_cards != 0) {
            live.weights[combo.index()] = 0.0;
        }
        live
    }

    /// Combos in either range, with the larger weight of the two.
    pub fn union(&self, other: &Range) -> Range {
        let mut combined = self.clone();
        for (weight, other_weight) in combined.weights.iter_mut().zip(&other.weights) {
            *weight = weight.max(*other_weight);
        }
        combined
    }

    /// This range with `other` taken out: each weight is multiplied by the
    /// share of the combo that `other` does not hold.
    pub fn minus(&self, other: &Range) -> Range {
        let mut remaining = self.clone();
        for (weight, other_weight) in remaining.weights.iter_mut().zip(&other.weights) {
            *weight *= 1.0 - other_weight;
        }
        remaining
    }

    /// Combos with a positive weight.
    pub fn weighted_combos(&self) -> impl Iterator<Item = (Combo, f32)> + '_ {
        Combo::all()
            .map(|combo| (combo, self.weight(combo)))
            .filter(|&(_, weight)| weight > 0.0)
    }
}

impl fmt::Debug for Range {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Range({:.1} combos)", self.combo_count())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeParseProblem {
    UnknownRank,
    UnknownSuitedness,
    PairCannotBeSuited,
    MismatchedSpan,
    InvalidWeight,
}

/// A token of range notation that could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeParseError {
    pub token: String,
    pub problem: RangeParseProblem,
}

impl fmt::Display for RangeParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let problem = match self.problem {
            RangeParseProblem::UnknownRank => "unknown rank",
            RangeParseProblem::UnknownSuitedness => "expected 's' or 'o' after the ranks",
            RangeParseProblem::PairCannotBeSuited => "a pair cannot be suited or offsuit",
            RangeParseProblem::MismatchedSpan => {
                "both ends of a span must share the first rank and the suitedness"
            }
            RangeParseProblem::InvalidWeight => "the weight must be a number between 0 and 1",
        };
        write!(formatter, "invalid range token {:?}: {problem}", self.token)
    }
}

impl std::error::Error for RangeParseError {}

impl FromStr for Range {
    type Err = RangeParseError;

    /// Parses comma-separated hand notation:
    ///
    /// - `AA`, `AKs`, `AKo`, `AK` (suited and offsuit)
    /// - `77+` (sevens and every higher pair), `KTs+` (KTs, KJs, KQs)
    /// - `JJ-88`, `A5s-A2s` (inclusive spans)
    /// - `:0.5` after any token sets its weight, for example `AQo:0.5`
    fn from_str(text: &str) -> Result<Range, RangeParseError> {
        let mut range = Range::empty();
        for token in text
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            let fail = |problem| RangeParseError {
                token: token.to_string(),
                problem,
            };
            let (pattern, weight) =
                split_weight(token).ok_or_else(|| fail(RangeParseProblem::InvalidWeight))?;
            for class in expand(pattern).map_err(fail)? {
                range.set_class(class, weight);
            }
        }
        Ok(range)
    }
}

fn split_weight(token: &str) -> Option<(&str, f32)> {
    let Some((pattern, weight)) = token.split_once(':') else {
        return Some((token, 1.0));
    };
    let weight: f32 = weight.trim().parse().ok()?;
    (0.0..=1.0)
        .contains(&weight)
        .then_some((pattern.trim(), weight))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suitedness {
    Pair,
    Suited,
    Offsuit,
    Either,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pattern {
    high: u8,
    low: u8,
    suitedness: Suitedness,
}

fn expand(text: &str) -> Result<Vec<HandClass>, RangeParseProblem> {
    if let Some(start) = text.strip_suffix('+') {
        let pattern = parse_pattern(start)?;
        let top = match pattern.suitedness {
            Suitedness::Pair => Pattern {
                high: ACE,
                low: ACE,
                ..pattern
            },
            Suitedness::Suited | Suitedness::Offsuit | Suitedness::Either => Pattern {
                low: pattern.high - 1,
                ..pattern
            },
        };
        return span(pattern, top);
    }
    if let Some((first, last)) = text.split_once('-') {
        return span(parse_pattern(first)?, parse_pattern(last)?);
    }
    let pattern = parse_pattern(text)?;
    span(pattern, pattern)
}

fn span(one_end: Pattern, other_end: Pattern) -> Result<Vec<HandClass>, RangeParseProblem> {
    if one_end.suitedness != other_end.suitedness {
        return Err(RangeParseProblem::MismatchedSpan);
    }
    let lowest = one_end.low.min(other_end.low);
    let highest = one_end.low.max(other_end.low);
    let classes = match one_end.suitedness {
        Suitedness::Pair => (lowest..=highest)
            .map(|rank| HandClass::Pair { rank })
            .collect(),
        Suitedness::Suited | Suitedness::Offsuit | Suitedness::Either => {
            if one_end.high != other_end.high || highest >= one_end.high {
                return Err(RangeParseProblem::MismatchedSpan);
            }
            let high = one_end.high;
            (lowest..=highest)
                .flat_map(|low| classes_for(high, low, one_end.suitedness))
                .collect()
        }
    };
    Ok(classes)
}

fn classes_for(high: u8, low: u8, suitedness: Suitedness) -> Vec<HandClass> {
    match suitedness {
        Suitedness::Pair => vec![HandClass::Pair { rank: high }],
        Suitedness::Suited => vec![HandClass::Suited { high, low }],
        Suitedness::Offsuit => vec![HandClass::Offsuit { high, low }],
        Suitedness::Either => vec![
            HandClass::Suited { high, low },
            HandClass::Offsuit { high, low },
        ],
    }
}

fn parse_pattern(text: &str) -> Result<Pattern, RangeParseProblem> {
    let mut characters = text.trim().chars();
    let first = characters
        .next()
        .and_then(parse_rank)
        .ok_or(RangeParseProblem::UnknownRank)?;
    let second = characters
        .next()
        .and_then(parse_rank)
        .ok_or(RangeParseProblem::UnknownRank)?;
    let suffix = characters.as_str();
    let high = first.max(second);
    let low = first.min(second);
    let suitedness = match (high == low, suffix) {
        (true, "") => Suitedness::Pair,
        (true, _) => return Err(RangeParseProblem::PairCannotBeSuited),
        (false, "s") => Suitedness::Suited,
        (false, "o") => Suitedness::Offsuit,
        (false, "") => Suitedness::Either,
        (false, _) => return Err(RangeParseProblem::UnknownSuitedness),
    };
    Ok(Pattern {
        high,
        low,
        suitedness,
    })
}

fn parse_rank(symbol: char) -> Option<u8> {
    match symbol {
        '2'..='9' => Some(symbol as u8 - b'0'),
        'T' => Some(10),
        'J' => Some(11),
        'Q' => Some(12),
        'K' => Some(13),
        'A' => Some(14),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(text: &str) -> Range {
        text.parse().unwrap()
    }

    fn class(text: &str) -> HandClass {
        let classes = expand(text).unwrap();
        assert_eq!(classes.len(), 1);
        classes[0]
    }

    #[test]
    fn counts_combos_of_common_ranges() {
        assert_eq!(range("AA").combo_count(), 6.0);
        assert_eq!(range("AKs").combo_count(), 4.0);
        assert_eq!(range("AKo").combo_count(), 12.0);
        assert_eq!(range("AK").combo_count(), 16.0);
        assert_eq!(range("22+").combo_count(), 78.0);
        assert_eq!(range("A2s+").combo_count(), 48.0);
        assert_eq!(range("KTs+").combo_count(), 12.0);
        assert_eq!(range("JJ-88").combo_count(), 24.0);
        assert_eq!(range("A5s-A2s").combo_count(), 16.0);
        assert_eq!(range("AQo:0.5").combo_count(), 6.0);
        assert_eq!(Range::full().combo_count(), 1326.0);
    }

    #[test]
    fn the_planned_under_the_gun_range_is_about_sixteen_percent() {
        let under_the_gun = range("22+, A2s+, KTs+, QTs+, JTs, T9s, 98s, AJo+, KQo");
        assert_eq!(under_the_gun.combo_count(), 206.0);
        assert!((under_the_gun.share_of_all_hands().percent() - 15.5).abs() < 0.1);
    }

    #[test]
    fn plus_keeps_the_high_card_and_raises_the_kicker() {
        let range = range("KTs+");
        for expected in ["KTs", "KJs", "KQs"] {
            assert_eq!(range.class_weight(class(expected)), 1.0, "{expected}");
        }
        assert_eq!(range.class_weight(class("K9s")), 0.0);
        assert_eq!(range.class_weight(class("KTo")), 0.0);
        assert_eq!(range.class_weight(class("AKs")), 0.0);
    }

    #[test]
    fn rejects_malformed_tokens() {
        let problem = |text: &str| text.parse::<Range>().unwrap_err().problem;
        assert_eq!(problem("AXs"), RangeParseProblem::UnknownRank);
        assert_eq!(problem("A"), RangeParseProblem::UnknownRank);
        assert_eq!(problem("AKx"), RangeParseProblem::UnknownSuitedness);
        assert_eq!(problem("AAs"), RangeParseProblem::PairCannotBeSuited);
        assert_eq!(problem("A5s-K2s"), RangeParseProblem::MismatchedSpan);
        assert_eq!(problem("A5s-A2o"), RangeParseProblem::MismatchedSpan);
        assert_eq!(problem("AK:2"), RangeParseProblem::InvalidWeight);
        assert_eq!(problem("AK:x"), RangeParseProblem::InvalidWeight);
        assert_eq!("".parse::<Range>().unwrap().combo_count(), 0.0);
    }

    #[test]
    fn removes_dead_cards_and_combines_ranges() {
        let aces = range("AA");
        let ace_of_hearts = crate::range::combo::CardIndex::of(&"Ah".parse().unwrap())
            .unwrap()
            .mask();
        assert_eq!(aces.without_cards(ace_of_hearts).combo_count(), 3.0);

        let premium = range("QQ+, AK");
        assert_eq!(premium.union(&range("JJ")).combo_count(), 40.0);
        assert_eq!(premium.minus(&range("AKo")).combo_count(), 22.0);
        assert_eq!(premium.minus(&range("AKo:0.5")).combo_count(), 28.0);
    }
}

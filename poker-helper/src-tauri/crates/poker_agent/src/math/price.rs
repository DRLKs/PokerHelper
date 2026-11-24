use crate::table::Chips;
use serde::Serialize;
use std::fmt;

/// A probability or share of the pot between 0 and 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Probability(f64);

impl Probability {
    pub const ZERO: Probability = Probability(0.0);
    pub const ONE: Probability = Probability(1.0);

    /// `None` unless `fraction` is a finite number in `0.0..=1.0`.
    pub fn new(fraction: f64) -> Option<Probability> {
        (0.0..=1.0)
            .contains(&fraction)
            .then_some(Probability(fraction))
    }

    /// Clamps any finite value into `0.0..=1.0`; `NaN` becomes zero.
    pub fn saturating(fraction: f64) -> Probability {
        if fraction.is_nan() {
            return Probability::ZERO;
        }
        Probability(fraction.clamp(0.0, 1.0))
    }

    /// `None` unless `percent` is in `0.0..=100.0`.
    pub fn from_percent(percent: f64) -> Option<Probability> {
        Probability::new(percent / 100.0)
    }

    pub fn fraction(self) -> f64 {
        self.0
    }

    pub fn percent(self) -> f64 {
        self.0 * 100.0
    }

    pub fn complement(self) -> Probability {
        Probability(1.0 - self.0)
    }
}

impl fmt::Display for Probability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.0}%", self.percent())
    }
}

/// The largest bet an opponent can make before calling it loses money.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CallLimit {
    AnyBet,
    UpTo(Chips),
}

/// Share of the final pot a call must win to break even:
/// `call / (pot_before_call + call)`. `pot_before_call` includes the bet
/// being called. Zero when there is nothing to call.
pub fn required_equity(call: Chips, pot_before_call: Chips) -> Probability {
    let final_pot = pot_before_call + call;
    if call == Chips::ZERO || final_pot == Chips::ZERO {
        return Probability::ZERO;
    }
    Probability::saturating(call.as_f64() / final_pot.as_f64())
}

/// Largest bet into `pot_before_bet` that a player with `equity` can call
/// without losing money, assuming they have nothing committed on the street.
///
/// Calling a bet `B` needs `B / (P + 2B) <= equity`, so the limit is
/// `equity * P / (1 - 2 * equity)`; from 50% equity on, any bet is a call.
pub fn max_callable_bet(pot_before_bet: Chips, equity: Probability) -> CallLimit {
    let equity = equity.fraction();
    if equity >= 0.5 {
        return CallLimit::AnyBet;
    }
    let limit = equity * pot_before_bet.as_f64() / (1.0 - 2.0 * equity);
    CallLimit::UpTo(Chips(limit.floor() as u64))
}

/// Share of its range a player must continue with against a bet so that a
/// bluff with any two cards does not profit: `P / (P + B)`.
pub fn minimum_defence_frequency(pot_before_bet: Chips, bet: Chips) -> Probability {
    let total = pot_before_bet + bet;
    if total == Chips::ZERO {
        return Probability::ONE;
    }
    Probability::saturating(pot_before_bet.as_f64() / total.as_f64())
}

/// How often a pure bluff of `bet` into `pot_before_bet` must make everyone
/// fold to break even: `B / (P + B)`.
pub fn bluff_breakeven(pot_before_bet: Chips, bet: Chips) -> Probability {
    let total = pot_before_bet + bet;
    if total == Chips::ZERO {
        return Probability::ZERO;
    }
    Probability::saturating(bet.as_f64() / total.as_f64())
}

/// Share of bluffs in a balanced river betting range, which leaves the
/// caller indifferent: `B / (P + 2B)`.
pub fn river_bluff_share(pot_before_bet: Chips, bet: Chips) -> Probability {
    required_equity(bet, pot_before_bet + bet)
}

/// Stack-to-pot ratio, or `None` for an empty pot.
pub fn stack_to_pot_ratio(effective_stack: Chips, pot: Chips) -> Option<f64> {
    (pot != Chips::ZERO).then(|| effective_stack.as_f64() / pot.as_f64())
}

/// Chips a drawing hand must win on later streets for a call to break even,
/// given `equity_when_hit`, the chance that the draw completes:
/// `call / equity_when_hit - (pot_before_call + call)`. `None` when the draw
/// can never complete.
pub fn implied_odds_needed(
    call: Chips,
    pot_before_call: Chips,
    equity_when_hit: Probability,
) -> Option<Chips> {
    if equity_when_hit == Probability::ZERO {
        return None;
    }
    let needed = call.as_f64() / equity_when_hit.fraction() - (pot_before_call + call).as_f64();
    Some(Chips(needed.max(0.0).ceil() as u64))
}

/// Expected chips won by calling compared with folding:
/// `equity * pot_before_call - (1 - equity) * call`.
pub fn ev_of_call(pot_before_call: Chips, call: Chips, equity: Probability) -> f64 {
    equity.fraction() * pot_before_call.as_f64() - equity.complement().fraction() * call.as_f64()
}

/// Expected chips won by betting `bet` into `pot_before_bet` compared with
/// checking down nothing, when the opponent folds with `fold_probability`
/// and otherwise calls, and the bettor's equity against the calling range is
/// `equity_when_called`:
/// `F * P + (1 - F) * (E * (P + 2B) - B)`.
pub fn ev_of_bet(
    pot_before_bet: Chips,
    bet: Chips,
    fold_probability: Probability,
    equity_when_called: Probability,
) -> f64 {
    let pot = pot_before_bet.as_f64();
    let bet = bet.as_f64();
    let when_called = equity_when_called.fraction() * (pot + 2.0 * bet) - bet;
    fold_probability.fraction() * pot + fold_probability.complement().fraction() * when_called
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probability(fraction: f64) -> Probability {
        Probability::new(fraction).unwrap()
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn probability_rejects_values_outside_zero_to_one() {
        assert!(Probability::new(1.2).is_none());
        assert!(Probability::new(-0.1).is_none());
        assert!(Probability::new(f64::NAN).is_none());
        assert_eq!(Probability::from_percent(25.0), Some(probability(0.25)));
        assert_eq!(Probability::saturating(f64::NAN), Probability::ZERO);
        assert_eq!(probability(0.384).to_string(), "38%");
    }

    #[test]
    fn calling_fifty_into_one_hundred_fifty_needs_a_quarter() {
        assert_close(required_equity(Chips(50), Chips(150)).fraction(), 0.25);
        assert_eq!(required_equity(Chips::ZERO, Chips(150)), Probability::ZERO);
    }

    #[test]
    fn thirty_percent_equity_calls_up_to_three_quarters_of_the_pot() {
        assert_eq!(
            max_callable_bet(Chips(100), probability(0.3)),
            CallLimit::UpTo(Chips(75))
        );
        assert_eq!(
            max_callable_bet(Chips(100), probability(0.5)),
            CallLimit::AnyBet
        );
        assert_eq!(
            max_callable_bet(Chips(100), Probability::ZERO),
            CallLimit::UpTo(Chips::ZERO)
        );
    }

    #[test]
    fn the_call_limit_is_exactly_break_even() {
        let pot = Chips(100);
        let equity = probability(0.3);
        let CallLimit::UpTo(limit) = max_callable_bet(pot, equity) else {
            panic!("30% equity has a limit");
        };
        assert!(ev_of_call(pot + limit, limit, equity) >= -1e-9);
        let one_more = limit + Chips(1);
        assert!(ev_of_call(pot + one_more, one_more, equity) < 0.0);
    }

    #[test]
    fn defence_and_bluff_frequencies_match_the_textbook_values() {
        assert_close(
            minimum_defence_frequency(Chips(100), Chips(100)).fraction(),
            0.5,
        );
        assert_close(
            minimum_defence_frequency(Chips(100), Chips(50)).fraction(),
            2.0 / 3.0,
        );
        assert_close(bluff_breakeven(Chips(100), Chips(50)).fraction(), 1.0 / 3.0);
        assert_close(river_bluff_share(Chips(100), Chips(50)).fraction(), 0.25);
        assert_close(
            river_bluff_share(Chips(100), Chips(100)).fraction(),
            1.0 / 3.0,
        );
        assert_close(river_bluff_share(Chips(100), Chips(200)).fraction(), 0.4);
    }

    #[test]
    fn stack_to_pot_ratio_and_implied_odds() {
        assert_eq!(stack_to_pot_ratio(Chips(300), Chips(100)), Some(3.0));
        assert_eq!(stack_to_pot_ratio(Chips(300), Chips::ZERO), None);
        assert_eq!(
            implied_odds_needed(Chips(20), Chips(60), probability(0.2)),
            Some(Chips(20))
        );
        assert_eq!(
            implied_odds_needed(Chips(20), Chips(100), probability(0.2)),
            Some(Chips::ZERO)
        );
        assert_eq!(
            implied_odds_needed(Chips(20), Chips(60), Probability::ZERO),
            None
        );
    }

    #[test]
    fn expected_values_of_calls_and_bets() {
        assert_close(ev_of_call(Chips(150), Chips(50), probability(0.25)), 0.0);
        assert_close(ev_of_call(Chips(150), Chips(50), probability(0.5)), 50.0);
        assert_close(
            ev_of_bet(Chips(100), Chips(50), Probability::ONE, Probability::ZERO),
            100.0,
        );
        assert_close(
            ev_of_bet(Chips(100), Chips(50), Probability::ZERO, Probability::ONE),
            150.0,
        );
        let breakeven_bluff = bluff_breakeven(Chips(100), Chips(50));
        assert_close(
            ev_of_bet(Chips(100), Chips(50), breakeven_bluff, Probability::ZERO),
            0.0,
        );
    }
}

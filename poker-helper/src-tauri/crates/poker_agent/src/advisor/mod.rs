use crate::math::{legal_actions, LegalActions};
use crate::opponent::TableReads;
use crate::postflop::{self, PostflopStreet};
use crate::preflop::{self, ChartError, Charts};
use crate::range::EquityError;
use crate::table::{Chips, Seat, Street, TableState};
use serde::Serialize;
use std::fmt;

pub mod report;

pub use report::{
    ActionValue, Confidence, ConfidenceIssue, ConfidenceLevel, PlannedBet, ResponsePlan,
};

/// The exact action to take now. Amounts are chips; `total` is the hero's
/// whole commitment on the street after acting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Recommendation {
    Fold,
    Check,
    Call {
        amount: Chips,
    },
    /// The first bet on a street after the flop.
    BetTo {
        total: Chips,
    },
    RaiseTo {
        total: Chips,
    },
    AllIn {
        total: Chips,
    },
}

impl Recommendation {
    /// One line the user can follow, such as `Raise to 6 (3.0 BB)`.
    pub fn instruction(self, big_blind: Chips) -> String {
        match self {
            Recommendation::Fold => "Fold".to_string(),
            Recommendation::Check => "Check".to_string(),
            Recommendation::Call { amount } => {
                format!("Call {amount} ({})", amount.in_big_blinds(big_blind))
            }
            Recommendation::BetTo { total } => {
                format!("Bet {total} ({})", total.in_big_blinds(big_blind))
            }
            Recommendation::RaiseTo { total } => {
                format!("Raise to {total} ({})", total.in_big_blinds(big_blind))
            }
            Recommendation::AllIn { total } => {
                format!("All-in: {total} ({})", total.in_big_blinds(big_blind))
            }
        }
    }
}

/// What to do if an opponent raises after the recommended bet or raise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedResponse {
    Fold,
    Call,
    ReRaise,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Advice {
    pub recommendation: Recommendation,
    pub instruction: String,
    pub reason: String,
    /// Set only when the recommendation is a bet or raise that leaves chips
    /// behind.
    pub if_reraised: Option<PlannedResponse>,
    /// Set after the flop when the hero checks with an opponent still to
    /// act, or bets or raises and keeps chips behind.
    pub response_plan: Option<ResponsePlan>,
    /// Every action the engine compared, best first. Empty before the flop,
    /// where the charts decide.
    pub ev_by_action: Vec<ActionValue>,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdviceError {
    /// The hero has folded, is all-in, or is not dealt in.
    HeroCannotAct,
    Charts(ChartError),
    Equity(EquityError),
}

impl fmt::Display for AdviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdviceError::HeroCannotAct => write!(formatter, "the hero has no decision to make"),
            AdviceError::Charts(error) => write!(formatter, "{error}"),
            AdviceError::Equity(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for AdviceError {}

impl From<ChartError> for AdviceError {
    fn from(error: ChartError) -> AdviceError {
        AdviceError::Charts(error)
    }
}

impl From<EquityError> for AdviceError {
    fn from(error: EquityError) -> AdviceError {
        AdviceError::Equity(error)
    }
}

/// Advice for the hero, assuming it is the hero's turn to act, with no
/// reads on the opponents.
///
/// Preflop advice follows the charts; later streets pick the action with the
/// highest expected value against the opponents' likely hands.
pub fn advise(table: &TableState) -> Result<Advice, AdviceError> {
    advise_with_reads(table, &TableReads::none())
}

/// Like [`advise`], with the opponents' likely hands and fold rates adjusted
/// to the habits in `reads`. Preflop, the reads change only the decision to
/// call an all-in; the charts stay as they are.
pub fn advise_with_reads(table: &TableState, reads: &TableReads) -> Result<Advice, AdviceError> {
    let legal = legal_actions(table, table.hero()).ok_or(AdviceError::HeroCannotAct)?;
    let charts = Charts::embedded()?;
    let mut advice = match table.street() {
        Street::Preflop => preflop::advise(table, &legal, charts, reads)?,
        Street::Flop => postflop::advise(table, &legal, charts, PostflopStreet::Flop, reads)?,
        Street::Turn => postflop::advise(table, &legal, charts, PostflopStreet::Turn, reads)?,
        Street::River => postflop::advise(table, &legal, charts, PostflopStreet::River, reads)?,
    };
    advice.confidence = Confidence::of(table);
    Ok(advice)
}

/// What a strategy wants to do, before the table's rules turn it into an
/// exact [`Recommendation`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Intent {
    Fold,
    Call,
    RaiseTo(Chips),
    AllIn,
}

/// Applies the betting rules to `intent`: folding becomes checking when it
/// is free, a raise is clamped to the legal sizes and becomes all-in when it
/// would commit the stack, and a raise that is not allowed becomes a call.
pub(crate) fn recommendation_for(
    intent: Intent,
    legal: &LegalActions,
    hero: &Seat,
) -> Recommendation {
    let all_in_total = hero.committed_this_street + hero.stack;
    let call_or_check = || {
        if legal.can_check() {
            Recommendation::Check
        } else if legal.to_call == hero.stack {
            Recommendation::AllIn {
                total: all_in_total,
            }
        } else {
            Recommendation::Call {
                amount: legal.to_call,
            }
        }
    };
    match (intent, legal.raise) {
        (Intent::Fold, _) if legal.can_check() => Recommendation::Check,
        (Intent::Fold, _) => Recommendation::Fold,
        (Intent::Call, _) | (Intent::RaiseTo(_) | Intent::AllIn, None) => call_or_check(),
        (Intent::AllIn, Some(bounds)) => Recommendation::AllIn {
            total: bounds.max_to,
        },
        (Intent::RaiseTo(target), Some(bounds)) => {
            let total = target.clamp(bounds.min_to, bounds.max_to);
            if total == bounds.max_to || commits_the_stack(total, hero.stack_at_street_start()) {
                Recommendation::AllIn {
                    total: bounds.max_to,
                }
            } else {
                Recommendation::RaiseTo { total }
            }
        }
    }
}

/// Whether raising to `raise_to` puts in more than a third of the stack the
/// player started the street with. Going all-in is then better than keeping
/// a stack too small to fold.
pub(crate) fn commits_the_stack(raise_to: Chips, stack_at_street_start: Chips) -> bool {
    raise_to.0 * 3 > stack_at_street_start.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::RaiseBounds;
    use crate::table::SeatStatus;

    fn hero(stack: u64, committed: u64) -> Seat {
        Seat {
            stack: Chips(stack),
            committed_this_street: Chips(committed),
            status: SeatStatus::Active,
        }
    }

    fn legal(to_call: u64, raise: Option<(u64, u64)>) -> LegalActions {
        LegalActions {
            to_call: Chips(to_call),
            raise: raise.map(|(min_to, max_to)| RaiseBounds {
                min_to: Chips(min_to),
                max_to: Chips(max_to),
            }),
        }
    }

    #[test]
    fn folding_is_a_check_when_it_is_free() {
        let player = hero(198, 2);
        assert_eq!(
            recommendation_for(Intent::Fold, &legal(0, Some((4, 200))), &player),
            Recommendation::Check
        );
        assert_eq!(
            recommendation_for(Intent::Fold, &legal(4, Some((10, 200))), &player),
            Recommendation::Fold
        );
    }

    #[test]
    fn raises_are_clamped_and_turn_all_in_when_they_commit_the_stack() {
        let deep = hero(200, 0);
        let bounds = legal(2, Some((4, 200)));
        assert_eq!(
            recommendation_for(Intent::RaiseTo(Chips(5)), &bounds, &deep),
            Recommendation::RaiseTo { total: Chips(5) }
        );
        assert_eq!(
            recommendation_for(Intent::RaiseTo(Chips(3)), &bounds, &deep),
            Recommendation::RaiseTo { total: Chips(4) }
        );
        assert_eq!(
            recommendation_for(Intent::RaiseTo(Chips(70)), &bounds, &deep),
            Recommendation::AllIn { total: Chips(200) }
        );
    }

    #[test]
    fn a_raise_that_is_not_allowed_becomes_a_call() {
        let player = hero(100, 0);
        assert_eq!(
            recommendation_for(Intent::RaiseTo(Chips(30)), &legal(10, None), &player),
            Recommendation::Call { amount: Chips(10) }
        );
        assert_eq!(
            recommendation_for(Intent::AllIn, &legal(100, None), &player),
            Recommendation::AllIn { total: Chips(100) }
        );
    }

    #[test]
    fn instructions_show_chips_and_big_blinds() {
        let big_blind = Chips(2);
        assert_eq!(
            Recommendation::RaiseTo { total: Chips(5) }.instruction(big_blind),
            "Raise to 5 (2.5 BB)"
        );
        assert_eq!(
            Recommendation::Call { amount: Chips(4) }.instruction(big_blind),
            "Call 4 (2.0 BB)"
        );
        assert_eq!(
            Recommendation::AllIn { total: Chips(20) }.instruction(big_blind),
            "All-in: 20 (10.0 BB)"
        );
        assert_eq!(
            Recommendation::BetTo { total: Chips(7) }.instruction(big_blind),
            "Bet 7 (3.5 BB)"
        );
        assert_eq!(Recommendation::Fold.instruction(big_blind), "Fold");
    }
}

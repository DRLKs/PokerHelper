use crate::advisor::{PlannedResponse, Recommendation};
use crate::math::CallLimit;
use crate::table::{Chips, SeatIndex, Street, TableState};
use serde::Serialize;

/// What to do once an opponent acts after the recommended check, bet or
/// raise, prepared before they act.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResponsePlan {
    /// The largest bet or raise, as the opponent's total on the street, that
    /// the hero can still call; anything above it is a fold.
    pub fold_above: CallLimit,
    /// The response to each typical size, smallest first.
    pub responses: Vec<PlannedBet>,
    /// One line the user can follow, such as `Fold if they bet more than 40 (20.0 BB).`
    pub summary: String,
}

/// The response to an opponent betting or raising to `total` on the street.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlannedBet {
    pub total: Chips,
    pub response: PlannedResponse,
}

/// The expected value of one candidate action, in chips won from now on
/// compared with folding.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActionValue {
    pub recommendation: Recommendation,
    pub instruction: String,
    pub ev: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

/// A reason to trust the advice less.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConfidenceIssue {
    /// An opponent reached the flop with no recorded preflop action, so its
    /// range is a wide guess.
    MissingHistory { seat: SeatIndex },
    /// Three or more players are in the pot, where ranges and equities are
    /// rougher estimates.
    ManyOpponents,
    /// The chips on the table changed between two frames of the same hand
    /// without adding up, so the table reading may be wrong.
    ChipsDoNotAddUp,
}

impl ConfidenceIssue {
    fn level(self) -> ConfidenceLevel {
        match self {
            ConfidenceIssue::MissingHistory { .. } | ConfidenceIssue::ChipsDoNotAddUp => {
                ConfidenceLevel::Low
            }
            ConfidenceIssue::ManyOpponents => ConfidenceLevel::Medium,
        }
    }
}

/// How far the advice can be trusted: the lowest level of its issues, or
/// high when there are none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Confidence {
    pub level: ConfidenceLevel,
    pub issues: Vec<ConfidenceIssue>,
}

impl Default for Confidence {
    fn default() -> Confidence {
        Confidence {
            level: ConfidenceLevel::High,
            issues: Vec::new(),
        }
    }
}

impl Confidence {
    /// The issues that the table alone shows.
    pub fn of(table: &TableState) -> Confidence {
        let opponents = table.opponents_in_hand(table.hero());
        let mut confidence = Confidence::default();
        if opponents.len() >= 2 {
            confidence.add(ConfidenceIssue::ManyOpponents);
        }
        if table.street() == Street::Preflop {
            return confidence;
        }
        for seat in opponents {
            let acted_preflop = table
                .actions_on(Street::Preflop)
                .any(|recorded| recorded.seat == seat);
            if !acted_preflop {
                confidence.add(ConfidenceIssue::MissingHistory { seat });
            }
        }
        confidence
    }

    pub fn add(&mut self, issue: ConfidenceIssue) {
        if self.issues.contains(&issue) {
            return;
        }
        self.issues.push(issue);
        self.level = self.level.min(issue.level());
    }
}

/// `Fold if they bet more than X` for a hero who checked or bet, or `raise
/// to more than X` after a raise.
pub(crate) fn fold_above_summary(
    fold_above: CallLimit,
    after: Recommendation,
    big_blind: Chips,
) -> String {
    let (noun, verb) = match after {
        Recommendation::Check => ("bet", "bet"),
        Recommendation::Fold
        | Recommendation::Call { .. }
        | Recommendation::BetTo { .. }
        | Recommendation::RaiseTo { .. }
        | Recommendation::AllIn { .. } => ("raise", "raise to"),
    };
    match fold_above {
        CallLimit::AnyBet => format!("Call any {noun}."),
        CallLimit::UpTo(limit) => format!(
            "Fold if they {verb} more than {limit} ({}).",
            limit.in_big_blinds(big_blind)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::{cards, six_max_preflop};
    use crate::table::{PlayerAction, RecordedAction, SeatStatus};

    #[test]
    fn the_lowest_issue_sets_the_level() {
        let mut confidence = Confidence::default();
        assert_eq!(confidence.level, ConfidenceLevel::High);
        confidence.add(ConfidenceIssue::ManyOpponents);
        assert_eq!(confidence.level, ConfidenceLevel::Medium);
        confidence.add(ConfidenceIssue::ChipsDoNotAddUp);
        confidence.add(ConfidenceIssue::ManyOpponents);
        assert_eq!(confidence.level, ConfidenceLevel::Low);
        assert_eq!(confidence.issues.len(), 2);
    }

    #[test]
    fn flags_an_opponent_on_the_flop_without_preflop_history() {
        let mut setup = six_max_preflop(0, "Ah Kd");
        for index in [1, 3, 4, 5] {
            setup.seats[index].status = SeatStatus::Folded;
        }
        setup.board = cards("2c 7d Ts");
        setup.seats[2].committed_this_street = Chips::ZERO;
        setup.seats[0].committed_this_street = Chips::ZERO;
        let without_history = TableState::new(setup.clone()).unwrap();
        assert_eq!(
            Confidence::of(&without_history).issues,
            [ConfidenceIssue::MissingHistory { seat: SeatIndex(2) }]
        );

        setup.history = vec![RecordedAction {
            seat: SeatIndex(2),
            street: Street::Preflop,
            action: PlayerAction::Call { to: Chips(5) },
        }];
        let with_history = TableState::new(setup).unwrap();
        assert_eq!(Confidence::of(&with_history), Confidence::default());
    }

    #[test]
    fn summarises_the_fold_limit() {
        let big_blind = Chips(2);
        assert_eq!(
            fold_above_summary(CallLimit::UpTo(Chips(40)), Recommendation::Check, big_blind),
            "Fold if they bet more than 40 (20.0 BB)."
        );
        assert_eq!(
            fold_above_summary(
                CallLimit::AnyBet,
                Recommendation::BetTo { total: Chips(6) },
                big_blind
            ),
            "Call any raise."
        );
    }
}

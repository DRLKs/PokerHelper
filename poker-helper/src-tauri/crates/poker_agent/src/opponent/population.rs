use crate::math::Probability;
use serde::{Deserialize, Serialize};

/// The kind of players a table usually holds. Every stat of a player starts
/// from the rate of this population and moves toward the player's own rate
/// as hands come in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Population {
    /// Real-money regulars: tight and aggressive.
    Regular,
    /// Play-money tables and home games: loose and passive.
    #[default]
    PlayMoney,
}

/// Average rates of a population, from the starting values in
/// `docs/STRATEGY_PLAN.md` (phase 5), to be tuned by simulation in phase 7.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tendencies {
    pub voluntarily_played: Probability,
    pub raised_preflop: Probability,
    pub three_bet: Probability,
    pub postflop_aggression: Probability,
    pub folded_to_continuation_bet: Probability,
    pub went_to_showdown: Probability,
}

impl Population {
    pub fn tendencies(self) -> Tendencies {
        let rates = match self {
            Population::Regular => [0.23, 0.18, 0.07, 0.65, 0.45, 0.27],
            Population::PlayMoney => [0.40, 0.12, 0.04, 0.45, 0.30, 0.35],
        };
        let [voluntarily_played, raised_preflop, three_bet, postflop_aggression, folded_to_continuation_bet, went_to_showdown] =
            rates.map(Probability::saturating);
        Tendencies {
            voluntarily_played,
            raised_preflop,
            three_bet,
            postflop_aggression,
            folded_to_continuation_bet,
            went_to_showdown,
        }
    }
}

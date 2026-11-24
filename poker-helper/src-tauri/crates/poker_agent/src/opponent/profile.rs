use crate::math::Probability;
use crate::opponent::population::Population;
use crate::opponent::stats::PlayerStats;
use serde::Serialize;

/// The preflop charts describe regulars, so preflop ranges are resized by
/// how a player compares with that population.
const CHART_POPULATION: Population = Population::Regular;
/// The postflop engine already assumes loose, passive callers (see the
/// status section of `docs/STRATEGY_PLAN.md`), so postflop habits are
/// compared with that population.
const POSTFLOP_POPULATION: Population = Population::PlayMoney;

const MIN_PREFLOP_RANGE_SCALE: f64 = 0.5;
const MAX_PREFLOP_RANGE_SCALE: f64 = 3.0;
const MIN_POSTFLOP_SCALE: f64 = 0.5;
const MAX_POSTFLOP_SCALE: f64 = 2.0;

/// A rough label for how a player plays, from the thresholds in
/// `docs/STRATEGY_PLAN.md` (phase 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerType {
    Nit,
    Regular,
    LooseAggressive,
    CallingStation,
    Maniac,
}

impl PlayerType {
    pub fn of(voluntarily_played: Probability, raised_preflop: Probability) -> PlayerType {
        let vpip = voluntarily_played.percent();
        let pfr = raised_preflop.percent();
        if vpip > 45.0 && pfr > 35.0 {
            PlayerType::Maniac
        } else if vpip > 35.0 && pfr < 15.0 {
            PlayerType::CallingStation
        } else if vpip > 25.0 && pfr > 20.0 {
            PlayerType::LooseAggressive
        } else if vpip < 16.0 {
            PlayerType::Nit
        } else {
            PlayerType::Regular
        }
    }
}

/// What a preflop action says about the hands behind it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreflopRangeKind {
    /// Opening, isolating or shoving into a pot nobody raised.
    FirstRaise,
    /// Re-raising a raise.
    ReRaise,
    /// Calling or limping.
    Call,
}

/// A player's estimated rates: each observed rate shrunk toward the
/// population rate, so a player with few hands looks like the population.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PlayerProfile {
    pub hands: u32,
    pub voluntarily_played: Probability,
    pub raised_preflop: Probability,
    pub three_bet: Probability,
    pub postflop_aggression: Probability,
    pub folded_to_continuation_bet: Probability,
    pub went_to_showdown: Probability,
    pub player_type: PlayerType,
}

impl PlayerProfile {
    pub fn estimate(stats: &PlayerStats, population: Population) -> PlayerProfile {
        let prior = population.tendencies();
        let voluntarily_played = stats
            .voluntarily_played
            .shrunk_toward(prior.voluntarily_played);
        let raised_preflop = stats.raised_preflop.shrunk_toward(prior.raised_preflop);
        PlayerProfile {
            hands: stats.hands(),
            voluntarily_played,
            raised_preflop,
            three_bet: stats.three_bet.shrunk_toward(prior.three_bet),
            postflop_aggression: stats
                .postflop_aggression
                .shrunk_toward(prior.postflop_aggression),
            folded_to_continuation_bet: stats
                .folded_to_continuation_bet
                .shrunk_toward(prior.folded_to_continuation_bet),
            went_to_showdown: stats.went_to_showdown.shrunk_toward(prior.went_to_showdown),
            player_type: PlayerType::of(voluntarily_played, raised_preflop),
        }
    }

    /// How many times larger than the chart range this player's range is
    /// for `kind`: the player's rate over the regulars' rate.
    pub fn preflop_range_scale(&self, kind: PreflopRangeKind) -> f64 {
        let reference = CHART_POPULATION.tendencies();
        let (player_rate, reference_rate) = match kind {
            PreflopRangeKind::FirstRaise => (self.raised_preflop, reference.raised_preflop),
            PreflopRangeKind::ReRaise => (self.three_bet, reference.three_bet),
            PreflopRangeKind::Call => (self.voluntarily_played, reference.voluntarily_played),
        };
        ratio(player_rate, reference_rate).clamp(MIN_PREFLOP_RANGE_SCALE, MAX_PREFLOP_RANGE_SCALE)
    }

    /// How much more of its range this player keeps when facing a bet than
    /// the postflop engine assumes: its rate of not folding to a
    /// continuation bet over the reference rate.
    pub fn continuing_scale(&self) -> f64 {
        let reference = POSTFLOP_POPULATION.tendencies();
        ratio(
            self.folded_to_continuation_bet.complement(),
            reference.folded_to_continuation_bet.complement(),
        )
        .clamp(MIN_POSTFLOP_SCALE, MAX_POSTFLOP_SCALE)
    }

    /// How much more of its range this player bets and raises with after the
    /// flop than the postflop engine assumes.
    pub fn betting_scale(&self) -> f64 {
        let reference = POSTFLOP_POPULATION.tendencies();
        ratio(self.postflop_aggression, reference.postflop_aggression)
            .clamp(MIN_POSTFLOP_SCALE, MAX_POSTFLOP_SCALE)
    }
}

fn ratio(rate: Probability, reference: Probability) -> f64 {
    if reference.fraction() <= 0.0 {
        return 1.0;
    }
    rate.fraction() / reference.fraction()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opponent::stats::Frequency;

    fn rate(hits: u32, chances: u32) -> Frequency {
        Frequency { hits, chances }
    }

    fn calling_station() -> PlayerStats {
        PlayerStats {
            voluntarily_played: rate(300, 500),
            raised_preflop: rate(25, 500),
            three_bet: rate(5, 200),
            postflop_aggression: rate(40, 200),
            folded_to_continuation_bet: rate(10, 100),
            went_to_showdown: rate(120, 250),
        }
    }

    fn nit() -> PlayerStats {
        PlayerStats {
            voluntarily_played: rate(55, 500),
            raised_preflop: rate(45, 500),
            three_bet: rate(4, 200),
            postflop_aggression: rate(30, 50),
            folded_to_continuation_bet: rate(70, 100),
            went_to_showdown: rate(10, 50),
        }
    }

    #[test]
    fn labels_players_by_their_preflop_rates() {
        let percent = |value| Probability::from_percent(value).unwrap();
        assert_eq!(
            PlayerType::of(percent(12.0), percent(10.0)),
            PlayerType::Nit
        );
        assert_eq!(
            PlayerType::of(percent(22.0), percent(18.0)),
            PlayerType::Regular
        );
        assert_eq!(
            PlayerType::of(percent(30.0), percent(25.0)),
            PlayerType::LooseAggressive
        );
        assert_eq!(
            PlayerType::of(percent(50.0), percent(8.0)),
            PlayerType::CallingStation
        );
        assert_eq!(
            PlayerType::of(percent(60.0), percent(40.0)),
            PlayerType::Maniac
        );
    }

    #[test]
    fn a_new_player_looks_like_the_population() {
        let regular = PlayerProfile::estimate(&PlayerStats::default(), Population::Regular);
        assert_eq!(regular.player_type, PlayerType::Regular);
        assert!((regular.preflop_range_scale(PreflopRangeKind::Call) - 1.0).abs() < 1e-9);
        let play_money = PlayerProfile::estimate(&PlayerStats::default(), Population::PlayMoney);
        assert_eq!(play_money.player_type, PlayerType::CallingStation);
        assert!((play_money.continuing_scale() - 1.0).abs() < 1e-9);
        assert!((play_money.betting_scale() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_calling_station_continues_more_and_a_nit_less() {
        let station = PlayerProfile::estimate(&calling_station(), Population::PlayMoney);
        assert_eq!(station.player_type, PlayerType::CallingStation);
        assert!(station.continuing_scale() > 1.2);
        assert!(station.betting_scale() < 0.6);
        assert!(station.preflop_range_scale(PreflopRangeKind::Call) > 2.0);

        let nit = PlayerProfile::estimate(&nit(), Population::PlayMoney);
        assert_eq!(nit.player_type, PlayerType::Nit);
        assert!(nit.continuing_scale() < 0.6);
        assert!(nit.preflop_range_scale(PreflopRangeKind::Call) < 0.6);
    }
}

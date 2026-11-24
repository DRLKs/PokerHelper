use crate::range::{HandClass, Range, RangeParseError};
use crate::table::Position;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

/// Starting ranges for 6-max cash games at 100 big blinds, plus push/fold
/// ranges for stacks of 15 big blinds or less. They are hand-written
/// approximations of solver output, see `docs/STRATEGY_PLAN.md`.
const EMBEDDED_CHARTS: &str = include_str!("../../data/preflop/charts.ranges");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChartError {
    MalformedLine { line: usize },
    DuplicateKey(String),
    InvalidRange { key: String, error: RangeParseError },
    MissingChart(String),
}

impl fmt::Display for ChartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartError::MalformedLine { line } => {
                write!(formatter, "chart line {line} is not `key = range`")
            }
            ChartError::DuplicateKey(key) => write!(formatter, "chart {key} is defined twice"),
            ChartError::InvalidRange { key, error } => write!(formatter, "chart {key}: {error}"),
            ChartError::MissingChart(key) => write!(formatter, "there is no chart {key}"),
        }
    }
}

impl std::error::Error for ChartError {}

/// What a chart says to do with a hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartAction {
    Raise,
    Call,
    Fold,
}

/// The raising and calling ranges of one preflop spot.
#[derive(Clone, Copy, Debug)]
pub struct SpotChart<'a> {
    pub raise: &'a Range,
    pub call: &'a Range,
}

impl SpotChart<'_> {
    /// The most frequent action for `class`. Ties go to the more aggressive
    /// action, so a hand listed in both ranges is raised.
    pub fn action_for(&self, class: HandClass) -> ChartAction {
        let raise = self.raise.class_weight(class);
        let call = self.call.class_weight(class);
        let fold = (1.0 - raise - call).max(0.0);
        if raise > 0.0 && raise >= call && raise >= fold {
            ChartAction::Raise
        } else if call > 0.0 && call >= fold {
            ChartAction::Call
        } else {
            ChartAction::Fold
        }
    }
}

/// Which side of the opener a player opened from, for facing a 3-bet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenerGroup {
    Early,
    Late,
}

impl OpenerGroup {
    pub fn of(position: Position) -> OpenerGroup {
        match position {
            Position::UnderTheGun | Position::Hijack => OpenerGroup::Early,
            Position::Cutoff | Position::Button | Position::SmallBlind | Position::BigBlind => {
                OpenerGroup::Late
            }
        }
    }

    fn key(self) -> &'static str {
        match self {
            OpenerGroup::Early => "early",
            OpenerGroup::Late => "late",
        }
    }
}

/// Whether a player acts after the opponent on the flop and later streets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seating {
    InPosition,
    OutOfPosition,
}

impl Seating {
    pub fn of(player: Position, opponent: Position) -> Seating {
        if player.is_in_position_against(opponent) {
            Seating::InPosition
        } else {
            Seating::OutOfPosition
        }
    }

    fn key(self) -> &'static str {
        match self {
            Seating::InPosition => "in_position",
            Seating::OutOfPosition => "out_of_position",
        }
    }
}

/// Stack depth buckets of the push/fold charts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortStackDepth {
    UpToSix,
    UpToTen,
    UpToFifteen,
}

impl ShortStackDepth {
    /// `None` above 15 big blinds, where the deep charts apply.
    pub fn of(big_blinds: f64) -> Option<ShortStackDepth> {
        if big_blinds <= 6.0 {
            Some(ShortStackDepth::UpToSix)
        } else if big_blinds <= 10.0 {
            Some(ShortStackDepth::UpToTen)
        } else if big_blinds <= 15.0 {
            Some(ShortStackDepth::UpToFifteen)
        } else {
            None
        }
    }

    /// Like [`ShortStackDepth::of`], with deeper stacks in the deepest bucket.
    pub fn at_most_fifteen(big_blinds: f64) -> ShortStackDepth {
        ShortStackDepth::of(big_blinds).unwrap_or(ShortStackDepth::UpToFifteen)
    }

    fn key(self) -> &'static str {
        match self {
            ShortStackDepth::UpToSix => "6",
            ShortStackDepth::UpToTen => "10",
            ShortStackDepth::UpToFifteen => "15",
        }
    }
}

#[derive(Debug)]
pub struct Charts {
    ranges: HashMap<String, Range>,
}

impl Charts {
    /// The charts compiled into the crate, parsed once.
    pub fn embedded() -> Result<&'static Charts, ChartError> {
        static CHARTS: OnceLock<Result<Charts, ChartError>> = OnceLock::new();
        CHARTS
            .get_or_init(|| Charts::parse(EMBEDDED_CHARTS))
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Parses lines of `key = range notation`. Blank lines are ignored.
    pub fn parse(source: &str) -> Result<Charts, ChartError> {
        let mut ranges = HashMap::new();
        for (line_index, line) in source.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let (key, notation) = line.split_once('=').ok_or(ChartError::MalformedLine {
                line: line_index + 1,
            })?;
            let key = key.trim().to_string();
            let range = notation
                .parse::<Range>()
                .map_err(|error| ChartError::InvalidRange {
                    key: key.clone(),
                    error,
                })?;
            if ranges.insert(key.clone(), range).is_some() {
                return Err(ChartError::DuplicateKey(key));
            }
        }
        Ok(Charts { ranges })
    }

    fn range(&self, key: &str) -> Result<&Range, ChartError> {
        self.ranges
            .get(key)
            .ok_or_else(|| ChartError::MissingChart(key.to_string()))
    }

    fn spot(&self, prefix: &str) -> Result<SpotChart<'_>, ChartError> {
        Ok(SpotChart {
            raise: self.range(&format!("{prefix}.raise"))?,
            call: self.range(&format!("{prefix}.call"))?,
        })
    }

    /// Opening range when everyone before folded. Heads-up, the button has
    /// its own wider chart.
    pub fn raise_first_in(&self, position: Position, heads_up: bool) -> Result<&Range, ChartError> {
        if heads_up && position == Position::Button {
            return self.range("rfi.HU_BTN");
        }
        self.range(&format!("rfi.{}", position.short_name()))
    }

    /// Facing a single open from `opener` with no callers.
    pub fn versus_open(
        &self,
        hero: Position,
        opener: Position,
    ) -> Result<SpotChart<'_>, ChartError> {
        self.spot(&format!(
            "vs_open.{}.{}",
            hero.short_name(),
            opener.short_name()
        ))
    }

    /// The opener facing a 3-bet.
    pub fn versus_three_bet(
        &self,
        opener: OpenerGroup,
        seating: Seating,
    ) -> Result<SpotChart<'_>, ChartError> {
        self.spot(&format!("vs_3bet.{}.{}", opener.key(), seating.key()))
    }

    /// The 3-bettor facing a 4-bet; the raise range goes all-in.
    pub fn versus_four_bet(&self) -> Result<SpotChart<'_>, ChartError> {
        self.spot("vs_4bet")
    }

    /// Facing a 5-bet or more; the raise range goes all-in.
    pub fn versus_five_bet(&self) -> Result<SpotChart<'_>, ChartError> {
        self.spot("vs_5bet")
    }

    /// Facing a 3-bet without having put money in yet.
    pub fn cold_four_bet(&self) -> Result<SpotChart<'_>, ChartError> {
        self.spot("cold_4bet")
    }

    /// Facing an open that someone already called.
    pub fn squeeze(&self) -> Result<SpotChart<'_>, ChartError> {
        self.spot("squeeze")
    }

    /// A player who limped and then faces a raise.
    pub fn limp_then_raised(&self) -> Result<SpotChart<'_>, ChartError> {
        self.spot("limp_then_raised")
    }

    /// Facing one or more limpers.
    pub fn versus_limpers(&self, hero: Position) -> Result<SpotChart<'_>, ChartError> {
        self.spot(&format!("vs_limp.{}", hero.short_name()))
    }

    /// All-in range when everyone before folded or limped, with a short stack.
    pub fn push(&self, position: Position, depth: ShortStackDepth) -> Result<&Range, ChartError> {
        self.range(&format!("push.{}.{}", position.short_name(), depth.key()))
    }

    /// All-in range over an open, with a short stack.
    pub fn resteal(&self, depth: ShortStackDepth) -> Result<&Range, ChartError> {
        self.range(&format!("resteal.{}", depth.key()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::Combo;

    fn class(cards: &str) -> HandClass {
        let parsed: Vec<_> = cards
            .split_whitespace()
            .map(|card| card.parse().unwrap())
            .collect();
        Combo::of([parsed[0], parsed[1]]).unwrap().hand_class()
    }

    const OPENERS: [Position; 5] = [
        Position::UnderTheGun,
        Position::Hijack,
        Position::Cutoff,
        Position::Button,
        Position::SmallBlind,
    ];

    #[test]
    fn every_chart_the_engine_can_ask_for_exists() {
        let charts = Charts::embedded().unwrap();
        for position in OPENERS {
            charts.raise_first_in(position, false).unwrap();
            for depth in [
                ShortStackDepth::UpToSix,
                ShortStackDepth::UpToTen,
                ShortStackDepth::UpToFifteen,
            ] {
                charts.push(position, depth).unwrap();
                charts.resteal(depth).unwrap();
            }
        }
        charts.raise_first_in(Position::Button, true).unwrap();
        for (opener_index, &opener) in OPENERS.iter().enumerate() {
            let later = OPENERS[opener_index + 1..]
                .iter()
                .chain([&Position::BigBlind]);
            for &hero in later {
                charts.versus_open(hero, opener).unwrap();
            }
        }
        for hero in [
            Position::Hijack,
            Position::Cutoff,
            Position::Button,
            Position::SmallBlind,
            Position::BigBlind,
        ] {
            charts.versus_limpers(hero).unwrap();
        }
        for group in [OpenerGroup::Early, OpenerGroup::Late] {
            for seating in [Seating::InPosition, Seating::OutOfPosition] {
                charts.versus_three_bet(group, seating).unwrap();
            }
        }
        charts.versus_four_bet().unwrap();
        charts.versus_five_bet().unwrap();
        charts.cold_four_bet().unwrap();
        charts.squeeze().unwrap();
        charts.limp_then_raised().unwrap();
    }

    #[test]
    fn opening_ranges_widen_toward_the_button() {
        let charts = Charts::embedded().unwrap();
        let shares: Vec<f64> = [
            Position::UnderTheGun,
            Position::Hijack,
            Position::Cutoff,
            Position::Button,
        ]
        .into_iter()
        .map(|position| {
            charts
                .raise_first_in(position, false)
                .unwrap()
                .share_of_all_hands()
                .percent()
        })
        .collect();
        assert!(
            shares.windows(2).all(|pair| pair[0] < pair[1]),
            "{shares:?}"
        );
        assert!((14.0..18.0).contains(&shares[0]), "UTG {}", shares[0]);
        assert!((40.0..50.0).contains(&shares[3]), "BTN {}", shares[3]);
    }

    #[test]
    fn push_ranges_widen_as_the_stack_shrinks() {
        let charts = Charts::embedded().unwrap();
        for position in OPENERS {
            let share = |depth| charts.push(position, depth).unwrap().combo_count();
            assert!(
                share(ShortStackDepth::UpToSix) > share(ShortStackDepth::UpToTen),
                "{position}"
            );
            assert!(
                share(ShortStackDepth::UpToTen) > share(ShortStackDepth::UpToFifteen),
                "{position}"
            );
        }
    }

    #[test]
    fn a_hand_in_both_ranges_is_raised() {
        let charts = Charts::embedded().unwrap();
        let big_blind_versus_utg = charts
            .versus_open(Position::BigBlind, Position::UnderTheGun)
            .unwrap();
        assert_eq!(
            big_blind_versus_utg.action_for(class("5s As")),
            ChartAction::Raise
        );
        assert_eq!(
            big_blind_versus_utg.action_for(class("9s As")),
            ChartAction::Call
        );
        assert_eq!(
            big_blind_versus_utg.action_for(class("7d 2s")),
            ChartAction::Fold
        );
    }

    #[test]
    fn reports_bad_chart_sources() {
        assert_eq!(
            Charts::parse("rfi.UTG 22+").unwrap_err(),
            ChartError::MalformedLine { line: 1 }
        );
        assert_eq!(
            Charts::parse("a = AA\na = KK").unwrap_err(),
            ChartError::DuplicateKey("a".to_string())
        );
        assert!(matches!(
            Charts::parse("a = AX").unwrap_err(),
            ChartError::InvalidRange { .. }
        ));
        let charts = Charts::parse("").unwrap();
        assert_eq!(
            charts.raise_first_in(Position::Cutoff, false).unwrap_err(),
            ChartError::MissingChart("rfi.CO".to_string())
        );
    }
}

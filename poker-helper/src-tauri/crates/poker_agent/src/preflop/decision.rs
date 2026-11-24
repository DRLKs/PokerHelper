use crate::advisor::{
    recommendation_for, Advice, AdviceError, Confidence, Intent, PlannedResponse, Recommendation,
};
use crate::math::price::required_equity;
use crate::math::LegalActions;
use crate::opponent::{PreflopRangeKind, TableReads};
use crate::preflop::charts::{
    ChartAction, Charts, OpenerGroup, Seating, ShortStackDepth, SpotChart,
};
use crate::preflop::sizing::{four_bet_to, isolation_size, open_size, three_bet_to};
use crate::preflop::spot::{classify, PreflopSituation, PreflopSpot, Raiser};
use crate::range::{equity_vs_ranges, HandClass, Range};
use crate::table::{BigBlinds, Chips, Position, SeatIndex, SeatStatus, TableState};

const EQUITY_ITERATIONS: u32 = 20_000;

/// The table facts every preflop rule needs.
struct Spot<'a> {
    table: &'a TableState,
    charts: &'a Charts,
    legal: &'a LegalActions,
    reads: &'a TableReads,
    hand: HandClass,
    position: Position,
    heads_up: bool,
    situation: PreflopSituation,
}

struct Plan {
    intent: Intent,
    reason: String,
    if_reraised: Option<PlannedResponse>,
}

pub fn advise(
    table: &TableState,
    legal: &LegalActions,
    charts: &Charts,
    reads: &TableReads,
) -> Result<Advice, AdviceError> {
    let spot = Spot {
        table,
        charts,
        legal,
        reads,
        hand: HandClass::of(table.hero_cards())
            .expect("TableState::new rejects invalid or repeated hero cards"),
        position: table.hero_position(),
        heads_up: table.is_heads_up(),
        situation: classify(table),
    };
    let effective_stack = table
        .effective_stack(table.hero())
        .in_big_blinds(table.big_blind());
    let plan = match ShortStackDepth::of(effective_stack.0) {
        Some(depth) => short_stack_plan(&spot, depth, effective_stack)?,
        None => deep_stack_plan(&spot)?,
    };

    let recommendation = recommendation_for(plan.intent, legal, table.hero_seat());
    let if_reraised = match recommendation {
        Recommendation::RaiseTo { .. } => plan.if_reraised,
        Recommendation::Fold
        | Recommendation::Check
        | Recommendation::Call { .. }
        | Recommendation::BetTo { .. }
        | Recommendation::AllIn { .. } => None,
    };
    Ok(Advice {
        instruction: recommendation.instruction(table.big_blind()),
        recommendation,
        reason: plan.reason,
        if_reraised,
        response_plan: None,
        ev_by_action: Vec::new(),
        confidence: Confidence::default(),
    })
}

fn deep_stack_plan(spot: &Spot) -> Result<Plan, AdviceError> {
    let charts = spot.charts;
    let current_bet = spot.table.current_bet();
    match spot.situation.spot {
        PreflopSpot::Unopened => open_plan(spot),
        PreflopSpot::Limped { limpers } => {
            let seating = isolation_seating(spot.position);
            let raise_to = in_chips(spot, isolation_size(limpers, seating));
            let if_reraised = response(
                spot,
                charts.versus_three_bet(OpenerGroup::of(spot.position), seating)?,
            );
            let context = format!(
                "from the {} against {}",
                spot.position.full_name(),
                plural(limpers, "limper")
            );
            Ok(chart_plan(
                spot,
                charts.versus_limpers(spot.position)?,
                Intent::RaiseTo(raise_to),
                if_reraised,
                &context,
            ))
        }
        PreflopSpot::FacingOpen {
            opener,
            callers,
            hero_limped,
        } => {
            let seating = Seating::of(spot.position, opener);
            let raise_to = Intent::RaiseTo(three_bet_to(current_bet, seating, callers));
            let if_reraised = response(spot, charts.versus_four_bet()?);
            let (chart, context) = if hero_limped {
                (
                    charts.limp_then_raised()?,
                    format!("after limping into a raise from the {}", opener.full_name()),
                )
            } else if callers > 0 {
                (
                    charts.squeeze()?,
                    format!(
                        "against an open from the {} with {}",
                        opener.full_name(),
                        plural(callers, "caller")
                    ),
                )
            } else {
                (
                    charts.versus_open(
                        chart_position(spot.position, spot.heads_up),
                        chart_position(opener, spot.heads_up),
                    )?,
                    format!(
                        "from the {} against an open from the {}",
                        spot.position.full_name(),
                        opener.full_name()
                    ),
                )
            };
            Ok(chart_plan(spot, chart, raise_to, if_reraised, &context))
        }
        PreflopSpot::FacingThreeBet {
            opener,
            three_bettor,
            hero_is_opener,
        } => {
            let raise_to = Intent::RaiseTo(four_bet_to(current_bet));
            let if_reraised = response(spot, charts.versus_five_bet()?);
            let chart = if hero_is_opener {
                charts.versus_three_bet(
                    OpenerGroup::of(opener),
                    Seating::of(spot.position, three_bettor),
                )?
            } else {
                charts.cold_four_bet()?
            };
            let context = format!("against a 3-bet from the {}", three_bettor.full_name());
            Ok(chart_plan(spot, chart, raise_to, if_reraised, &context))
        }
        PreflopSpot::FacingFourBet {
            hero_is_three_bettor,
        } => {
            let chart = if hero_is_three_bettor {
                charts.versus_four_bet()?
            } else {
                charts.versus_five_bet()?
            };
            Ok(chart_plan(
                spot,
                chart,
                Intent::AllIn,
                None,
                "against a 4-bet",
            ))
        }
        PreflopSpot::FacingFiveBetOrMore => Ok(chart_plan(
            spot,
            charts.versus_five_bet()?,
            Intent::AllIn,
            None,
            "against a 5-bet",
        )),
    }
}

fn open_plan(spot: &Spot) -> Result<Plan, AdviceError> {
    if spot.position == Position::BigBlind {
        return Ok(Plan {
            intent: Intent::Fold,
            reason: "Nobody raised, so checking is free.".to_string(),
            if_reraised: None,
        });
    }
    let range = spot.charts.raise_first_in(spot.position, spot.heads_up)?;
    let share = range.share_of_all_hands();
    if !holds(range, spot.hand) {
        return Ok(Plan {
            intent: Intent::Fold,
            reason: format!(
                "{} is not in the {} opening range ({share} of hands).",
                spot.hand,
                spot.position.full_name()
            ),
            if_reraised: None,
        });
    }
    let seating_against_three_bet = match spot.position {
        Position::Button => Seating::InPosition,
        Position::UnderTheGun
        | Position::Hijack
        | Position::Cutoff
        | Position::SmallBlind
        | Position::BigBlind => Seating::OutOfPosition,
    };
    let if_reraised = response(
        spot,
        spot.charts
            .versus_three_bet(OpenerGroup::of(spot.position), seating_against_three_bet)?,
    );
    Ok(Plan {
        intent: Intent::RaiseTo(in_chips(spot, open_size(spot.position, spot.heads_up))),
        reason: format!(
            "{} is in the {} opening range ({share} of hands).",
            spot.hand,
            spot.position.full_name()
        ),
        if_reraised,
    })
}

fn chart_plan(
    spot: &Spot,
    chart: SpotChart,
    raise: Intent,
    if_reraised: Option<PlannedResponse>,
    context: &str,
) -> Plan {
    let hand = spot.hand;
    match chart.action_for(hand) {
        ChartAction::Raise => Plan {
            intent: raise,
            reason: format!(
                "{hand} is in the raising range {context} ({} of hands).",
                chart.raise.share_of_all_hands()
            ),
            if_reraised,
        },
        ChartAction::Call => Plan {
            intent: Intent::Call,
            reason: format!("{hand} is in the calling range {context}."),
            if_reraised: None,
        },
        ChartAction::Fold => Plan {
            intent: Intent::Fold,
            reason: format!("{hand} is too weak to play {context}."),
            if_reraised: None,
        },
    }
}

fn short_stack_plan(
    spot: &Spot,
    depth: ShortStackDepth,
    effective_stack: BigBlinds,
) -> Result<Plan, AdviceError> {
    let hand = spot.hand;
    let raiser = match (spot.situation.spot, spot.situation.last_raiser) {
        (PreflopSpot::Unopened, _) if spot.position == Position::BigBlind => {
            return open_plan(spot)
        }
        (PreflopSpot::Unopened | PreflopSpot::Limped { .. }, _) | (_, None) => {
            let (range, description) = match push_chart_position(spot.position, spot.heads_up) {
                Some(position) => (
                    spot.charts.push(position, depth)?,
                    format!("{} all-in range", spot.position.full_name()),
                ),
                None => (
                    spot.charts.resteal(depth)?,
                    "all-in range over limpers".to_string(),
                ),
            };
            return Ok(all_in_or_fold(range, hand, effective_stack, &description));
        }
        (_, Some(raiser)) => raiser,
    };

    let hero_stack = spot.table.hero_seat().stack_at_street_start();
    let raiser_is_all_in = spot
        .table
        .seat(raiser.seat)
        .is_some_and(|seat| seat.status == SeatStatus::AllIn);
    let call_commits_the_stack = spot.legal.to_call.0 * 3 >= hero_stack.0;
    if raiser_is_all_in || spot.legal.raise.is_none() || call_commits_the_stack {
        return call_or_fold_by_equity(spot, raiser);
    }
    Ok(all_in_or_fold(
        spot.charts.resteal(depth)?,
        hand,
        effective_stack,
        "re-steal range",
    ))
}

fn all_in_or_fold(
    range: &Range,
    hand: HandClass,
    effective_stack: BigBlinds,
    description: &str,
) -> Plan {
    let share = range.share_of_all_hands();
    let (intent, verdict) = if holds(range, hand) {
        (Intent::AllIn, "is in")
    } else {
        (Intent::Fold, "is not in")
    };
    Plan {
        intent,
        reason: format!(
            "With {effective_stack} {hand} {verdict} the {description} ({share} of hands)."
        ),
        if_reraised: None,
    }
}

/// Calls an all-in when the hand's equity against the likely ranges of
/// everyone who put chips in beats the price of the call.
fn call_or_fold_by_equity(spot: &Spot, raiser: Raiser) -> Result<Plan, AdviceError> {
    let table = spot.table;
    let opponent_ranges = committed_opponents(table)
        .into_iter()
        .map(|seat| estimated_all_in_range(spot, raiser, seat))
        .collect::<Result<Vec<Range>, AdviceError>>()?;
    let equity = equity_vs_ranges(
        table.hero_cards(),
        table.board(),
        &opponent_ranges,
        EQUITY_ITERATIONS,
    )?;
    let needed = required_equity(spot.legal.to_call, table.total_pot());
    let (intent, verdict) = if equity >= needed {
        (Intent::Call, "call")
    } else {
        (Intent::Fold, "fold")
    };
    Ok(Plan {
        intent,
        reason: format!(
            "Calling {} needs {needed} equity and {} has {equity} against the likely all-in range, so {verdict}.",
            spot.legal.to_call, spot.hand
        ),
        if_reraised: None,
    })
}

fn committed_opponents(table: &TableState) -> Vec<SeatIndex> {
    table
        .opponents_in_hand(table.hero())
        .into_iter()
        .filter(|&seat| {
            table
                .seat(seat)
                .is_some_and(|player| player.committed_this_street > table.big_blind())
        })
        .collect()
}

/// A short-stack opener's all-in range comes from the push chart of its
/// position; anyone re-raising or calling an all-in is given the tighter
/// re-steal range. Both use the depth of that player's own stack, and are
/// resized by how often that player raises or re-raises when it has a
/// profile in the reads.
fn estimated_all_in_range(
    spot: &Spot,
    raiser: Raiser,
    seat: SeatIndex,
) -> Result<Range, AdviceError> {
    let table = spot.table;
    let stack = table
        .seat(seat)
        .map(|player| player.stack_at_street_start())
        .unwrap_or(Chips::ZERO);
    let depth = ShortStackDepth::at_most_fifteen(stack.in_big_blinds(table.big_blind()).0);
    let opened_an_unraised_pot =
        matches!(spot.situation.spot, PreflopSpot::FacingOpen { .. }) && seat == raiser.seat;
    let push_position =
        push_chart_position(raiser.position, spot.heads_up).filter(|_| opened_an_unraised_pot);
    let (range, kind) = match push_position {
        Some(position) => (
            spot.charts.push(position, depth)?,
            PreflopRangeKind::FirstRaise,
        ),
        None => (spot.charts.resteal(depth)?, PreflopRangeKind::ReRaise),
    };
    let scale = spot
        .reads
        .profile(seat)
        .map_or(1.0, |profile| profile.preflop_range_scale(kind));
    Ok(range.rescaled(scale))
}

fn response(spot: &Spot, chart: SpotChart) -> Option<PlannedResponse> {
    Some(match chart.action_for(spot.hand) {
        ChartAction::Raise => PlannedResponse::ReRaise,
        ChartAction::Call => PlannedResponse::Call,
        ChartAction::Fold => PlannedResponse::Fold,
    })
}

fn holds(range: &Range, hand: HandClass) -> bool {
    range.class_weight(hand) >= 0.5
}

fn in_chips(spot: &Spot, amount: BigBlinds) -> Chips {
    Chips::from_big_blinds(amount, spot.table.big_blind())
}

/// The heads-up button posts the small blind, so charts for the small blind
/// describe it when it faces or makes a raise.
pub(crate) fn chart_position(position: Position, heads_up: bool) -> Position {
    match (position, heads_up) {
        (Position::Button, true) => Position::SmallBlind,
        _ => position,
    }
}

/// Position whose push chart applies, or `None` for the big blind, which
/// has no unopened push chart.
fn push_chart_position(position: Position, heads_up: bool) -> Option<Position> {
    match chart_position(position, heads_up) {
        Position::BigBlind => None,
        other => Some(other),
    }
}

/// Raising over limpers from the blinds means playing out of position.
fn isolation_seating(position: Position) -> Seating {
    match position {
        Position::SmallBlind | Position::BigBlind => Seating::OutOfPosition,
        Position::UnderTheGun | Position::Hijack | Position::Cutoff | Position::Button => {
            Seating::InPosition
        }
    }
}

fn plural(count: usize, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        _ => format!("{count} {noun}s"),
    }
}

#[cfg(test)]
mod tests {
    use crate::advisor::{
        advise, advise_with_reads, Advice, AdviceError, PlannedResponse, Recommendation,
    };
    use crate::core::card::rank_symbol;
    use crate::opponent::{Frequency, PlayerProfile, PlayerStats, Population, TableReads};
    use crate::range::HandClass;
    use crate::table::state::tests::{cards, seat, six_max_preflop};
    use crate::table::{
        Chips, PlayerAction, RecordedAction, SeatIndex, SeatStatus, Street, TableSetup, TableState,
    };

    const BUTTON: u8 = 0;
    const SMALL_BLIND: u8 = 1;
    const BIG_BLIND: u8 = 2;
    const UNDER_THE_GUN: u8 = 3;
    const HIJACK: u8 = 4;
    const CUTOFF: u8 = 5;

    fn advice(setup: TableSetup) -> Advice {
        advise(&TableState::new(setup).unwrap()).unwrap()
    }

    fn raise(seat: u8, to: u64) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street: Street::Preflop,
            action: PlayerAction::Raise { to: Chips(to) },
        }
    }

    fn limp(seat: u8) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street: Street::Preflop,
            action: PlayerAction::Call { to: Chips(2) },
        }
    }

    fn fold_seats(setup: &mut TableSetup, seats: &[u8]) {
        for &index in seats {
            setup.seats[index as usize].status = SeatStatus::Folded;
        }
    }

    fn cards_of(class: HandClass) -> String {
        let (high, low) = class.ranks();
        let (high_suit, low_suit) = match class {
            HandClass::Suited { .. } => ('h', 'h'),
            HandClass::Pair { .. } | HandClass::Offsuit { .. } => ('h', 'd'),
        };
        format!(
            "{}{high_suit} {}{low_suit}",
            rank_symbol(high),
            rank_symbol(low)
        )
    }

    fn facing_cutoff_open_then_button_three_bet(hero_cards: &str) -> TableSetup {
        let mut setup = six_max_preflop(CUTOFF, hero_cards);
        fold_seats(&mut setup, &[SMALL_BLIND, BIG_BLIND, UNDER_THE_GUN, HIJACK]);
        setup.seats[CUTOFF as usize] = seat(195, 5, SeatStatus::Active);
        setup.seats[BUTTON as usize] = seat(185, 15, SeatStatus::Active);
        setup.history = vec![raise(CUTOFF, 5), raise(BUTTON, 15)];
        setup
    }

    #[test]
    fn opens_aces_and_plans_to_four_bet_them() {
        let advice = advice(six_max_preflop(UNDER_THE_GUN, "Ah Ad"));
        assert_eq!(
            advice.recommendation,
            Recommendation::RaiseTo { total: Chips(5) }
        );
        assert_eq!(advice.instruction, "Raise to 5 (2.5 BB)");
        assert_eq!(advice.if_reraised, Some(PlannedResponse::ReRaise));
    }

    #[test]
    fn folds_seven_deuce_under_the_gun() {
        let advice = advice(six_max_preflop(UNDER_THE_GUN, "7h 2d"));
        assert_eq!(advice.recommendation, Recommendation::Fold);
        assert!(
            advice
                .reason
                .contains("not in the first position (UTG) opening range"),
            "{}",
            advice.reason
        );
    }

    #[test]
    fn every_hand_gets_an_opening_decision_from_every_position() {
        let mut raised_hands = Vec::new();
        for (hero, position) in [
            (UNDER_THE_GUN, "UTG"),
            (HIJACK, "HJ"),
            (CUTOFF, "CO"),
            (BUTTON, "BTN"),
            (SMALL_BLIND, "SB"),
        ] {
            let mut raised = 0;
            for class in HandClass::all() {
                let recommendation = advice(six_max_preflop(hero, &cards_of(class))).recommendation;
                match recommendation {
                    Recommendation::RaiseTo { .. } => raised += 1,
                    Recommendation::Fold => {}
                    other => panic!("{position} {class}: unexpected {other:?}"),
                }
                if class == (HandClass::Pair { rank: 14 }) {
                    assert!(
                        matches!(recommendation, Recommendation::RaiseTo { .. }),
                        "{position} folds AA"
                    );
                }
            }
            raised_hands.push(raised);
        }
        assert!(
            raised_hands[0] < raised_hands[3],
            "UTG opens {} classes, BTN {}",
            raised_hands[0],
            raised_hands[3]
        );
    }

    #[test]
    fn three_bets_ace_king_from_the_cutoff_against_an_early_open() {
        let mut setup = six_max_preflop(CUTOFF, "Ah Kd");
        setup.seats[UNDER_THE_GUN as usize] = seat(194, 6, SeatStatus::Active);
        fold_seats(&mut setup, &[HIJACK]);
        setup.history = vec![raise(UNDER_THE_GUN, 6)];
        let advice = advice(setup);
        assert_eq!(
            advice.recommendation,
            Recommendation::RaiseTo { total: Chips(18) }
        );
        assert_eq!(advice.if_reraised, Some(PlannedResponse::ReRaise));
    }

    #[test]
    fn big_blind_defends_suited_connectors_against_the_button() {
        let mut setup = six_max_preflop(BIG_BLIND, "9h 8h");
        fold_seats(&mut setup, &[UNDER_THE_GUN, HIJACK, CUTOFF, SMALL_BLIND]);
        setup.seats[BUTTON as usize] = seat(195, 5, SeatStatus::Active);
        setup.history = vec![raise(BUTTON, 5)];
        assert_eq!(
            advice(setup).recommendation,
            Recommendation::Call { amount: Chips(3) }
        );
    }

    #[test]
    fn the_opener_responds_to_a_three_bet_by_hand_strength() {
        let expected = [
            ("Qh Qd", Recommendation::RaiseTo { total: Chips(35) }),
            ("Th Td", Recommendation::Call { amount: Chips(10) }),
            ("Ah Jd", Recommendation::Fold),
        ];
        for (hand, recommendation) in expected {
            let advice = advice(facing_cutoff_open_then_button_three_bet(hand));
            assert_eq!(advice.recommendation, recommendation, "{hand}");
        }
    }

    #[test]
    fn the_three_bettor_goes_all_in_with_kings_against_a_four_bet() {
        let mut setup = facing_cutoff_open_then_button_three_bet("Kh Kd");
        setup.hero = SeatIndex(BUTTON);
        setup.hero_cards = [cards("Kh")[0], cards("Kd")[0]];
        setup.seats[CUTOFF as usize] = seat(165, 35, SeatStatus::Active);
        setup.history.push(raise(CUTOFF, 35));
        assert_eq!(
            advice(setup.clone()).recommendation,
            Recommendation::AllIn { total: Chips(200) }
        );

        setup.hero_cards = [cards("Ah")[0], cards("Qh")[0]];
        assert_eq!(
            advice(setup).recommendation,
            Recommendation::Call { amount: Chips(20) }
        );
    }

    #[test]
    fn a_ten_big_blind_button_pushes_or_folds() {
        let mut setup = six_max_preflop(BUTTON, "Kh 3h");
        fold_seats(&mut setup, &[UNDER_THE_GUN, HIJACK, CUTOFF]);
        setup.seats[BUTTON as usize] = seat(20, 0, SeatStatus::Active);
        assert_eq!(
            advice(setup.clone()).recommendation,
            Recommendation::AllIn { total: Chips(20) }
        );

        setup.hero_cards = [cards("7h")[0], cards("2d")[0]];
        assert_eq!(advice(setup).recommendation, Recommendation::Fold);
    }

    #[test]
    fn calls_a_short_all_in_only_with_enough_equity() {
        let shove = |hand: &str| {
            let mut setup = six_max_preflop(BIG_BLIND, hand);
            fold_seats(&mut setup, &[UNDER_THE_GUN, HIJACK, CUTOFF, SMALL_BLIND]);
            setup.seats[BUTTON as usize] = seat(0, 20, SeatStatus::AllIn);
            setup.seats[BIG_BLIND as usize] = seat(18, 2, SeatStatus::Active);
            setup.history = vec![raise(BUTTON, 20)];
            advice(setup)
        };
        let ace_jack = shove("Ah Jd");
        assert_eq!(
            ace_jack.recommendation,
            Recommendation::AllIn { total: Chips(20) }
        );
        assert!(
            ace_jack.reason.contains("needs 44% equity"),
            "{}",
            ace_jack.reason
        );
        assert_eq!(shove("7h 2d").recommendation, Recommendation::Fold);
    }

    #[test]
    fn calls_a_maniac_all_in_wider() {
        let under_the_gun_shoves_into = |hand: &str| {
            let mut setup = six_max_preflop(BIG_BLIND, hand);
            fold_seats(&mut setup, &[HIJACK, CUTOFF, BUTTON, SMALL_BLIND]);
            setup.seats[UNDER_THE_GUN as usize] = seat(0, 20, SeatStatus::AllIn);
            setup.seats[BIG_BLIND as usize] = seat(18, 2, SeatStatus::Active);
            setup.history = vec![raise(UNDER_THE_GUN, 20)];
            TableState::new(setup).unwrap()
        };
        let maniac = PlayerStats {
            voluntarily_played: Frequency {
                hits: 300,
                chances: 500,
            },
            raised_preflop: Frequency {
                hits: 250,
                chances: 500,
            },
            ..PlayerStats::default()
        };
        let mut profiles = vec![None; 6];
        profiles[UNDER_THE_GUN as usize] =
            Some(PlayerProfile::estimate(&maniac, Population::PlayMoney));
        let reads = TableReads::from_profiles(profiles);

        let table = under_the_gun_shoves_into("Ah 8d");
        assert_eq!(advise(&table).unwrap().recommendation, Recommendation::Fold);
        assert_eq!(
            advise_with_reads(&table, &reads).unwrap().recommendation,
            Recommendation::AllIn { total: Chips(20) }
        );
    }

    #[test]
    fn isolates_limpers_and_checks_the_big_blind_option() {
        let mut setup = six_max_preflop(BUTTON, "Ah Kd");
        fold_seats(&mut setup, &[HIJACK, CUTOFF]);
        setup.seats[UNDER_THE_GUN as usize] = seat(198, 2, SeatStatus::Active);
        setup.history = vec![limp(UNDER_THE_GUN)];
        assert_eq!(
            advice(setup.clone()).recommendation,
            Recommendation::RaiseTo { total: Chips(8) }
        );

        setup.hero = SeatIndex(BIG_BLIND);
        setup.hero_cards = [cards("7h")[0], cards("2d")[0]];
        fold_seats(&mut setup, &[BUTTON, SMALL_BLIND]);
        assert_eq!(advice(setup).recommendation, Recommendation::Check);
    }

    #[test]
    fn heads_up_the_button_opens_wide_and_the_big_blind_defends_wide() {
        let mut setup = six_max_preflop(BUTTON, "Ah 2d");
        for index in [BIG_BLIND, UNDER_THE_GUN, HIJACK, CUTOFF] {
            setup.seats[index as usize].status = SeatStatus::Empty;
        }
        setup.seats[BUTTON as usize] = seat(199, 1, SeatStatus::Active);
        setup.seats[SMALL_BLIND as usize] = seat(198, 2, SeatStatus::Active);
        assert_eq!(
            advice(setup.clone()).recommendation,
            Recommendation::RaiseTo { total: Chips(5) }
        );

        setup.hero = SeatIndex(SMALL_BLIND);
        setup.hero_cards = [cards("Qh")[0], cards("2h")[0]];
        setup.seats[BUTTON as usize] = seat(195, 5, SeatStatus::Active);
        setup.history = vec![raise(BUTTON, 5)];
        assert_eq!(
            advice(setup).recommendation,
            Recommendation::Call { amount: Chips(3) }
        );
    }

    #[test]
    fn reports_decisions_it_cannot_make() {
        let mut folded = six_max_preflop(BIG_BLIND, "Ah Kd");
        folded.seats[BIG_BLIND as usize].status = SeatStatus::Folded;
        let table = TableState::new(folded).unwrap();
        assert_eq!(advise(&table).unwrap_err(), AdviceError::HeroCannotAct);
    }
}

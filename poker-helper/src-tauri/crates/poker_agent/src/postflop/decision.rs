use crate::advisor::{
    recommendation_for, Advice, AdviceError, Intent, PlannedResponse, Recommendation,
};
use crate::evaluator::fast_eval::CardMask;
use crate::math::price::{implied_odds_needed, required_equity};
use crate::math::{LegalActions, Probability};
use crate::postflop::narrowing::{narrow, Narrowing};
use crate::postflop::opponent_range::estimate_ranges;
use crate::postflop::reading::HandReading;
use crate::postflop::texture::BoardTexture;
use crate::postflop::validated_cards_mask;
use crate::preflop::Charts;
use crate::range::combo::DECK_SIZE;
use crate::range::{equity_vs_ranges, Range};
use crate::table::{Chips, Seat, TableState};

const EQUITY_ITERATIONS: u32 = 20_000;
/// Share of its equity a hand wins out of position, from the realization
/// table in `docs/STRATEGY_PLAN.md`.
const OUT_OF_POSITION_REALIZATION: f64 = 0.85;
/// A bet or raise must beat the best passive line by this share of the pot.
const SIMPLER_LINE_MARGIN: f64 = 0.01;
const RAISE_MULTIPLIER: u64 = 3;
const ALL_IN_MAX_POT_MULTIPLE: f64 = 1.5;
const RIVER_ALL_IN_MAX_POT_MULTIPLE: f64 = 2.0;
const MIN_DRAW_OUTS: u8 = 4;
/// Share of the stack behind a draw may expect to win when it completes.
const IMPLIED_ODDS_STACK_SHARE: f64 = 0.25;
const RERAISE_EQUITY: f64 = 0.6;

const DRY_FLOP_BET_SIZES: [f64; 3] = [0.33, 0.5, 0.75];
const WET_FLOP_BET_SIZES: [f64; 2] = [0.5, 0.75];
const TURN_BET_SIZES: [f64; 3] = [0.5, 0.75, 1.0];
const RIVER_BET_SIZES: [f64; 3] = [0.5, 0.75, 1.25];

/// A street after the flop is dealt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostflopStreet {
    Flop,
    Turn,
    River,
}

impl PostflopStreet {
    fn is_river(self) -> bool {
        match self {
            PostflopStreet::River => true,
            PostflopStreet::Flop | PostflopStreet::Turn => false,
        }
    }

    fn bet_sizes(self, texture: BoardTexture) -> &'static [f64] {
        match self {
            PostflopStreet::Flop if texture.is_wet() => &WET_FLOP_BET_SIZES,
            PostflopStreet::Flop => &DRY_FLOP_BET_SIZES,
            PostflopStreet::Turn => &TURN_BET_SIZES,
            PostflopStreet::River => &RIVER_BET_SIZES,
        }
    }

    fn all_in_max_pot_multiple(self) -> f64 {
        match self {
            PostflopStreet::River => RIVER_ALL_IN_MAX_POT_MULTIPLE,
            PostflopStreet::Flop | PostflopStreet::Turn => ALL_IN_MAX_POT_MULTIPLE,
        }
    }
}

/// Picks the action with the highest expected value against the likely
/// hands of every opponent still in the hand.
///
/// Fold, check and call are compared with two to four bet or raise sizes.
/// A bet must beat the best passive line by 1% of the pot. A draw that
/// cannot call on price alone still calls when the chips it can win later
/// cover the difference.
pub fn advise(
    table: &TableState,
    legal: &LegalActions,
    charts: &Charts,
    street: PostflopStreet,
) -> Result<Advice, AdviceError> {
    let spot = Spot::read(table, legal, charts, street)?;
    let ranges: Vec<Range> = spot
        .opponents
        .iter()
        .map(|opponent| opponent.range.clone())
        .collect();
    let equity = spot.equity_against(&ranges)?;

    let passive = spot.best_passive_line(equity);
    let chosen = match spot.best_aggressive_line()? {
        Some(aggressive)
            if aggressive.ev > passive.ev + SIMPLER_LINE_MARGIN * spot.pot.as_f64() =>
        {
            aggressive
        }
        _ => spot.implied_odds_call(&passive, equity).unwrap_or(passive),
    };
    let if_reraised = spot.response_to_a_raise(chosen.recommendation)?;
    Ok(Advice {
        instruction: chosen.recommendation.instruction(table.big_blind()),
        recommendation: chosen.recommendation,
        reason: spot.reason(&chosen, equity),
        if_reraised,
    })
}

struct Opponent {
    seat: Seat,
    range: Range,
}

impl Opponent {
    fn stack_at_street_start(&self) -> Chips {
        self.seat.stack_at_street_start()
    }
}

#[derive(Clone, Copy, Debug)]
enum Line {
    Fold,
    Check,
    Call,
    ImpliedOddsCall {
        needed_later: Chips,
    },
    Aggressive {
        fold_probability: Probability,
        equity_when_called: Probability,
    },
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    recommendation: Recommendation,
    line: Line,
    /// Expected chips won from now on, compared with folding.
    ev: f64,
}

/// The facts of one postflop decision.
struct Spot<'a> {
    table: &'a TableState,
    legal: &'a LegalActions,
    street: PostflopStreet,
    hero: Seat,
    board: CardMask,
    dead_cards: CardMask,
    pot: Chips,
    opponents: Vec<Opponent>,
    reading: HandReading,
    texture: BoardTexture,
    realization: f64,
}

impl<'a> Spot<'a> {
    fn read(
        table: &'a TableState,
        legal: &'a LegalActions,
        charts: &Charts,
        street: PostflopStreet,
    ) -> Result<Spot<'a>, AdviceError> {
        let hole = validated_cards_mask(&table.hero_cards());
        let board = validated_cards_mask(table.board());
        let dead_cards = hole | board;
        let opponent_seats = table.opponents_in_hand(table.hero());
        let ranges = estimate_ranges(table, charts, &opponent_seats, dead_cards)?;
        let opponents = opponent_seats
            .iter()
            .zip(ranges)
            .map(|(&seat, range)| Opponent {
                seat: *table
                    .seat(seat)
                    .expect("opponents_in_hand returns seats of the table"),
                range,
            })
            .collect();
        Ok(Spot {
            table,
            legal,
            street,
            hero: *table.hero_seat(),
            board,
            dead_cards,
            pot: table.total_pot(),
            opponents,
            reading: HandReading::of(hole, board),
            texture: BoardTexture::of(board),
            realization: realization(table, street),
        })
    }

    fn equity_against(&self, ranges: &[Range]) -> Result<Probability, AdviceError> {
        Ok(equity_vs_ranges(
            self.table.hero_cards(),
            self.table.board(),
            ranges,
            EQUITY_ITERATIONS,
        )?)
    }

    /// Realization for a line that ends the hero's decisions when it puts
    /// the hero all-in.
    fn realization_for(&self, puts_hero_all_in: bool) -> f64 {
        if puts_hero_all_in {
            1.0
        } else {
            self.realization
        }
    }

    fn best_passive_line(&self, equity: Probability) -> Candidate {
        if self.legal.can_check() {
            return Candidate {
                recommendation: Recommendation::Check,
                line: Line::Check,
                ev: self.realization * equity.fraction() * self.pot.as_f64(),
            };
        }
        let call = self.legal.to_call;
        let realized = self.realization_for(call == self.hero.stack) * equity.fraction();
        let call_ev = realized * (self.pot + call).as_f64() - call.as_f64();
        if call_ev > 0.0 {
            Candidate {
                recommendation: recommendation_for(Intent::Call, self.legal, &self.hero),
                line: Line::Call,
                ev: call_ev,
            }
        } else {
            Candidate {
                recommendation: Recommendation::Fold,
                line: Line::Fold,
                ev: 0.0,
            }
        }
    }

    fn best_aggressive_line(&self) -> Result<Option<Candidate>, AdviceError> {
        let mut best: Option<Candidate> = None;
        let mut evaluated_totals: Vec<Chips> = Vec::new();
        for target in self.raise_targets() {
            let recommendation = self.named_for_the_street(recommendation_for(
                Intent::RaiseTo(target),
                self.legal,
                &self.hero,
            ));
            let Some(total) = raise_total(recommendation) else {
                continue;
            };
            if evaluated_totals.contains(&total) {
                continue;
            }
            evaluated_totals.push(total);
            let candidate = self.aggressive_line(recommendation, total)?;
            if best.is_none_or(|current| candidate.ev > current.ev) {
                best = Some(candidate);
            }
        }
        Ok(best)
    }

    /// Bet sizes as shares of the pot when nobody has bet, or a raise to 3
    /// times the bet; plus all-in when the stack is small next to the pot.
    fn raise_targets(&self) -> Vec<Chips> {
        let Some(bounds) = self.legal.raise else {
            return Vec::new();
        };
        let current_bet = self.table.current_bet();
        let mut targets: Vec<Chips> = if current_bet == Chips::ZERO {
            self.street
                .bet_sizes(self.texture)
                .iter()
                .map(|share| Chips((share * self.pot.as_f64()).round() as u64))
                .collect()
        } else {
            vec![Chips(current_bet.0 * RAISE_MULTIPLIER)]
        };
        let all_in_adds = bounds
            .max_to
            .saturating_sub(self.hero.committed_this_street);
        if all_in_adds.as_f64() <= self.street.all_in_max_pot_multiple() * self.pot.as_f64() {
            targets.push(bounds.max_to);
        }
        targets
    }

    /// A raise when nobody has bet on the street is a bet.
    fn named_for_the_street(&self, recommendation: Recommendation) -> Recommendation {
        match recommendation {
            Recommendation::RaiseTo { total } if self.table.current_bet() == Chips::ZERO => {
                Recommendation::BetTo { total }
            }
            other => other,
        }
    }

    /// Expected value of betting or raising to `total`.
    ///
    /// Each opponent who can act continues with `1 - r` of its range by made
    /// hand, plus strong draws, where `r` is the equity it needs to call.
    /// That is the share of hands whose equity beats the price if equities
    /// spread evenly across the range. It is always above the minimum
    /// defence frequency, so pure bluffs never show a profit: the right
    /// baseline against the loose players of home and play-money games.
    /// When anyone continues, the hero plays against all of them.
    fn aggressive_line(
        &self,
        recommendation: Recommendation,
        total: Chips,
    ) -> Result<Candidate, AdviceError> {
        let largest_opponent_total = self
            .opponents
            .iter()
            .map(Opponent::stack_at_street_start)
            .max()
            .unwrap_or(Chips::ZERO);
        let hero_adds = total
            .min(largest_opponent_total)
            .saturating_sub(self.hero.committed_this_street);
        let pot_after_hero = self.pot + hero_adds;

        let mut everyone_folds = 1.0;
        let mut chips_matched = Chips::ZERO;
        let mut continuing = Vec::with_capacity(self.opponents.len());
        for opponent in &self.opponents {
            if !opponent.seat.status.can_act() {
                everyone_folds = 0.0;
                continuing.push(opponent.range.clone());
                continue;
            }
            let to_match = total
                .saturating_sub(opponent.seat.committed_this_street)
                .min(opponent.seat.stack);
            let continuing_share = required_equity(to_match, pot_after_hero).complement();
            let narrowed = narrow(
                &opponent.range,
                self.board,
                self.dead_cards,
                Narrowing::continuing(continuing_share),
            );
            everyone_folds *= narrowed.kept_share.complement().fraction();
            chips_matched += to_match;
            continuing.push(narrowed.range);
        }

        let equity_when_called = self.equity_against(&continuing)?;
        let puts_hero_all_in = matches!(recommendation, Recommendation::AllIn { .. });
        let realized = self.realization_for(puts_hero_all_in) * equity_when_called.fraction();
        let pot_when_called = pot_after_hero + chips_matched;
        let ev = everyone_folds * self.pot.as_f64()
            + (1.0 - everyone_folds) * (realized * pot_when_called.as_f64() - hero_adds.as_f64());
        Ok(Candidate {
            recommendation,
            line: Line::Aggressive {
                fold_probability: Probability::saturating(everyone_folds),
                equity_when_called,
            },
            ev,
        })
    }

    /// Turns a fold into a call when a draw of at least 4 outs needs to win
    /// no more than a quarter of the stack behind on later streets.
    fn implied_odds_call(&self, passive: &Candidate, equity: Probability) -> Option<Candidate> {
        let draw_can_still_come = !self.street.is_river() && self.reading.outs >= MIN_DRAW_OUTS;
        let call = self.legal.to_call;
        let call_leaves_chips_behind = call < self.hero.stack;
        if !matches!(passive.line, Line::Fold) || !draw_can_still_come || !call_leaves_chips_behind
        {
            return None;
        }
        let unseen_cards = DECK_SIZE as u32 - self.dead_cards.count_ones();
        let hit = Probability::saturating(self.reading.outs as f64 / unseen_cards as f64);
        let needed_later = implied_odds_needed(call, self.pot, hit)?;
        let behind_after_call = self
            .table
            .effective_stack(self.table.hero())
            .saturating_sub(self.hero.committed_this_street + call);
        let affordable =
            needed_later.as_f64() <= IMPLIED_ODDS_STACK_SHARE * behind_after_call.as_f64();
        affordable.then(|| Candidate {
            recommendation: recommendation_for(Intent::Call, self.legal, &self.hero),
            line: Line::ImpliedOddsCall { needed_later },
            ev: self.realization * equity.fraction() * (self.pot + call).as_f64() - call.as_f64(),
        })
    }

    /// What to do if an opponent raises the recommended bet or raise to 3
    /// times its size. `None` when the recommendation is not a bet or raise
    /// that leaves chips behind.
    fn response_to_a_raise(
        &self,
        recommendation: Recommendation,
    ) -> Result<Option<PlannedResponse>, AdviceError> {
        let total = match recommendation {
            Recommendation::BetTo { total } | Recommendation::RaiseTo { total } => total,
            Recommendation::Fold
            | Recommendation::Check
            | Recommendation::Call { .. }
            | Recommendation::AllIn { .. } => return Ok(None),
        };
        let raising_ranges: Vec<Range> = self
            .opponents
            .iter()
            .filter(|opponent| opponent.seat.status.can_act())
            .map(|opponent| {
                narrow(
                    &opponent.range,
                    self.board,
                    self.dead_cards,
                    Narrowing::RAISE,
                )
                .range
            })
            .collect();
        if raising_ranges.is_empty() {
            return Ok(None);
        }
        let equity = self.equity_against(&raising_ranges)?;

        let hero_adds = total.saturating_sub(self.hero.committed_this_street);
        let behind_after_bet = self.hero.stack.saturating_sub(hero_adds);
        let reraise_to = Chips(total.0 * RAISE_MULTIPLIER);
        let call = reraise_to.saturating_sub(total).min(behind_after_bet);
        let raiser_adds = reraise_to.saturating_sub(self.table.current_bet());
        let pot_before_call = self.pot + hero_adds + raiser_adds;
        let calling_is_all_in = call == behind_after_bet;
        let realized =
            Probability::saturating(self.realization_for(calling_is_all_in) * equity.fraction());

        let response = if !calling_is_all_in && realized.fraction() >= RERAISE_EQUITY {
            PlannedResponse::ReRaise
        } else if realized >= required_equity(call, pot_before_call) {
            PlannedResponse::Call
        } else {
            PlannedResponse::Fold
        };
        Ok(Some(response))
    }

    fn reason(&self, chosen: &Candidate, equity: Probability) -> String {
        let hand = self.reading.describe();
        let call = self.legal.to_call;
        let needed = required_equity(call, self.pot);
        let realized = Probability::saturating(
            self.realization_for(call == self.hero.stack) * equity.fraction(),
        );
        match chosen.line {
            Line::Check => format!(
                "With {hand} you have about {equity} equity against the likely hands, and betting does not earn more than checking."
            ),
            Line::Call => format!(
                "You need {needed} equity to call and have about {realized} with {hand}."
            ),
            Line::Fold => format!(
                "You need {needed} equity to call and have only about {realized} with {hand}."
            ),
            Line::ImpliedOddsCall { needed_later } => format!(
                "You need {needed} equity to call and have about {realized} with {hand}, but the call pays off if the draw wins {needed_later} more chips later."
            ),
            Line::Aggressive {
                fold_probability,
                equity_when_called,
            } => format!(
                "With {hand}, {} makes them fold about {fold_probability} of the time, and you have about {equity_when_called} equity when called.",
                self.describe_aggression(chosen.recommendation)
            ),
        }
    }

    fn describe_aggression(&self, recommendation: Recommendation) -> String {
        match recommendation {
            Recommendation::BetTo { total } => {
                let share_of_pot = total.as_f64() / self.pot.as_f64() * 100.0;
                format!("betting {share_of_pot:.0}% of the pot")
            }
            Recommendation::RaiseTo { total } => format!("raising to {total}"),
            Recommendation::AllIn { .. } => "going all-in".to_string(),
            Recommendation::Fold | Recommendation::Check | Recommendation::Call { .. } => {
                "playing passively".to_string()
            }
        }
    }
}

fn raise_total(recommendation: Recommendation) -> Option<Chips> {
    match recommendation {
        Recommendation::BetTo { total }
        | Recommendation::RaiseTo { total }
        | Recommendation::AllIn { total } => Some(total),
        Recommendation::Fold | Recommendation::Check | Recommendation::Call { .. } => None,
    }
}

/// Share of its equity the hero wins by showdown: all of it on the river or
/// when acting last against everyone who can still act, less out of position.
fn realization(table: &TableState, street: PostflopStreet) -> f64 {
    let hero_position = table.hero_position();
    let acts_last = table
        .opponents_in_hand(table.hero())
        .into_iter()
        .filter(|&seat| {
            table
                .seat(seat)
                .is_some_and(|player| player.status.can_act())
        })
        .filter_map(|seat| table.position_of(seat))
        .all(|opponent| hero_position.is_in_position_against(opponent));
    if street.is_river() || acts_last {
        1.0
    } else {
        OUT_OF_POSITION_REALIZATION
    }
}

#[cfg(test)]
mod tests {
    use crate::advisor::{advise, Advice, PlannedResponse, Recommendation};
    use crate::math::legal_actions;
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

    fn act(seat: u8, street: Street, action: PlayerAction) -> RecordedAction {
        RecordedAction {
            seat: SeatIndex(seat),
            street,
            action,
        }
    }

    fn street_of(setup: &TableSetup) -> Street {
        Street::from_board_len(setup.board.len()).unwrap()
    }

    /// The button opened to 5 and the big blind called; both have 195 left.
    fn button_against_big_blind(hero: u8, hero_cards: &str, board: &str) -> TableSetup {
        let mut setup = six_max_preflop(hero, hero_cards);
        for index in [SMALL_BLIND, UNDER_THE_GUN, HIJACK, CUTOFF] {
            setup.seats[index as usize] = seat(200, 0, SeatStatus::Folded);
        }
        setup.seats[BUTTON as usize] = seat(195, 0, SeatStatus::Active);
        setup.seats[BIG_BLIND as usize] = seat(195, 0, SeatStatus::Active);
        setup.board = cards(board);
        setup.pot_from_previous_streets = Chips(11);
        setup.history = vec![
            act(
                BUTTON,
                Street::Preflop,
                PlayerAction::Raise { to: Chips(5) },
            ),
            act(
                BIG_BLIND,
                Street::Preflop,
                PlayerAction::Call { to: Chips(5) },
            ),
        ];
        setup
    }

    fn check(mut setup: TableSetup, seat: u8) -> TableSetup {
        let street = street_of(&setup);
        setup.history.push(act(seat, street, PlayerAction::Check));
        setup
    }

    fn bet(mut setup: TableSetup, bettor: u8, amount: u64) -> TableSetup {
        let street = street_of(&setup);
        let player = &mut setup.seats[bettor as usize];
        player.stack = player.stack - Chips(amount);
        player.committed_this_street = Chips(amount);
        setup
            .history
            .push(act(bettor, street, PlayerAction::Bet { to: Chips(amount) }));
        setup
    }

    fn advice(setup: TableSetup) -> Advice {
        advise(&TableState::new(setup).unwrap()).unwrap()
    }

    fn is_aggressive(recommendation: Recommendation) -> bool {
        match recommendation {
            Recommendation::BetTo { .. }
            | Recommendation::RaiseTo { .. }
            | Recommendation::AllIn { .. } => true,
            Recommendation::Fold | Recommendation::Check | Recommendation::Call { .. } => false,
        }
    }

    #[test]
    fn checks_air_when_checking_is_free() {
        let advice = advice(button_against_big_blind(BIG_BLIND, "3c 4d", "Kh Qh Jc"));
        assert_eq!(
            advice.recommendation,
            Recommendation::Check,
            "{}",
            advice.reason
        );
        assert_eq!(advice.if_reraised, None);
    }

    #[test]
    fn bets_the_nut_flush_on_the_river() {
        let setup = check(
            button_against_big_blind(BUTTON, "Ah 5h", "Kh 7h 2c 9h 4s"),
            BIG_BLIND,
        );
        let advice = advice(setup);
        assert!(is_aggressive(advice.recommendation), "{advice:?}");
        assert!(advice.reason.contains("a flush"), "{}", advice.reason);
    }

    #[test]
    fn folds_air_to_a_pot_sized_river_bet() {
        let setup = bet(
            button_against_big_blind(BUTTON, "6c 5c", "Kh 7h 2d 9s Jd"),
            BIG_BLIND,
            11,
        );
        let advice = advice(setup);
        assert_eq!(advice.recommendation, Recommendation::Fold);
        assert!(
            advice.reason.contains("You need 33% equity"),
            "{}",
            advice.reason
        );
    }

    #[test]
    fn continues_with_a_nut_flush_draw_against_a_small_bet() {
        let setup = bet(
            button_against_big_blind(BUTTON, "Ah 5h", "Kh 7h 2c"),
            BIG_BLIND,
            4,
        );
        let advice = advice(setup);
        assert_ne!(
            advice.recommendation,
            Recommendation::Fold,
            "{}",
            advice.reason
        );
    }

    #[test]
    fn a_set_never_folds_to_a_flop_bet() {
        let setup = bet(
            check(
                button_against_big_blind(BIG_BLIND, "7d 7s", "Kh 7h 2c"),
                BIG_BLIND,
            ),
            BUTTON,
            8,
        );
        let advice = advice(setup);
        assert!(
            matches!(
                advice.recommendation,
                Recommendation::Call { .. }
                    | Recommendation::RaiseTo { .. }
                    | Recommendation::AllIn { .. }
            ),
            "{advice:?}"
        );
    }

    #[test]
    fn a_gutshot_calls_on_implied_odds_with_deep_stacks() {
        let mut setup = button_against_big_blind(BUTTON, "9c 8d", "Jh 7s 2c 3d");
        setup.seats[BUTTON as usize].stack = Chips(395);
        setup.seats[BIG_BLIND as usize].stack = Chips(395);
        let advice = advice(bet(setup, BIG_BLIND, 6));
        assert_eq!(
            advice.recommendation,
            Recommendation::Call { amount: Chips(6) },
            "{}",
            advice.reason
        );
        assert!(
            advice.reason.contains("more chips later"),
            "{}",
            advice.reason
        );
    }

    #[test]
    fn a_gutshot_folds_when_stacks_are_short() {
        let mut setup = button_against_big_blind(BUTTON, "9c 8d", "Jh 7s 2c 3d");
        setup.seats[BUTTON as usize].stack = Chips(30);
        setup.seats[BIG_BLIND as usize].stack = Chips(30);
        let advice = advice(bet(setup, BIG_BLIND, 6));
        assert_eq!(
            advice.recommendation,
            Recommendation::Fold,
            "{}",
            advice.reason
        );
    }

    #[test]
    fn bets_and_raises_are_legal_and_named_for_the_street() {
        let spots = [
            check(
                button_against_big_blind(BUTTON, "Ah Kd", "Kh 7c 2s"),
                BIG_BLIND,
            ),
            check(
                button_against_big_blind(BUTTON, "Qh Qd", "Jh 7c 2s 3d"),
                BIG_BLIND,
            ),
            bet(
                button_against_big_blind(BUTTON, "Kc Kd", "Kh 7c 2s"),
                BIG_BLIND,
                3,
            ),
            bet(
                button_against_big_blind(BIG_BLIND, "7c 2d", "7h 7d 2s"),
                BUTTON,
                6,
            ),
        ];
        for setup in spots {
            let table = TableState::new(setup).unwrap();
            let legal = legal_actions(&table, table.hero()).unwrap();
            let advice = advise(&table).unwrap();
            match advice.recommendation {
                Recommendation::BetTo { total } => {
                    assert_eq!(table.current_bet(), Chips::ZERO);
                    assert!(total >= legal.raise.unwrap().min_to, "{advice:?}");
                    assert!(advice.instruction.starts_with("Bet "), "{advice:?}");
                    assert!(advice.if_reraised.is_some());
                }
                Recommendation::RaiseTo { total } => {
                    assert!(table.current_bet() > Chips::ZERO);
                    assert!(total >= legal.raise.unwrap().min_to, "{advice:?}");
                    assert!(advice.if_reraised.is_some());
                }
                Recommendation::Fold => panic!("folded a strong hand: {advice:?}"),
                Recommendation::Check
                | Recommendation::Call { .. }
                | Recommendation::AllIn { .. } => {}
            }
        }
    }

    #[test]
    fn plans_to_keep_going_with_quads_when_raised() {
        let setup = check(
            button_against_big_blind(BUTTON, "7c 7s", "7h 7d 2s"),
            BIG_BLIND,
        );
        let advice = advice(setup);
        if is_aggressive(advice.recommendation) {
            assert_ne!(
                advice.if_reraised,
                Some(PlannedResponse::Fold),
                "{advice:?}"
            );
        }
    }
}

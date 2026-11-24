use crate::advisor::report::fold_above_summary;
use crate::advisor::{
    recommendation_for, ActionValue, Advice, AdviceError, Confidence, Intent, PlannedBet,
    PlannedResponse, Recommendation, ResponsePlan,
};
use crate::evaluator::fast_eval::CardMask;
use crate::math::price::{implied_odds_needed, max_callable_bet, required_equity};
use crate::math::{CallLimit, LegalActions, Probability};
use crate::opponent::TableReads;
use crate::postflop::narrowing::{narrow, Narrowing};
use crate::postflop::opponent_range::estimate_ranges;
use crate::postflop::reading::HandReading;
use crate::postflop::texture::BoardTexture;
use crate::postflop::validated_cards_mask;
use crate::preflop::Charts;
use crate::range::combo::DECK_SIZE;
use crate::range::{equity_vs_ranges, Range};
use crate::table::{Chips, Seat, SeatIndex, TableState};

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
/// Opponent bet sizes, as shares of the pot, that the plan after a check covers.
const PLANNED_BET_SIZES: [f64; 4] = [0.33, 0.5, 0.75, 1.0];
/// Opponent raise sizes, as multiples of the hero's bet, that the plan after
/// a bet or raise covers.
const PLANNED_RAISE_MULTIPLES: [f64; 3] = [2.5, 3.0, 4.0];

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
///
/// Opponents with a profile in `reads` get ranges and fold rates that follow
/// their own habits; see [`estimate_ranges`].
pub fn advise(
    table: &TableState,
    legal: &LegalActions,
    charts: &Charts,
    street: PostflopStreet,
    reads: &TableReads,
) -> Result<Advice, AdviceError> {
    let spot = Spot::read(table, legal, charts, street, reads)?;
    let ranges: Vec<Range> = spot
        .opponents
        .iter()
        .map(|opponent| opponent.range.clone())
        .collect();
    let equity = spot.equity_against(&ranges)?;

    let passive_lines = spot.passive_lines(equity);
    let aggressive_lines = spot.aggressive_lines()?;
    let passive = best_passive_line(&passive_lines);
    let best_aggressive = aggressive_lines.iter().copied().reduce(|best, candidate| {
        if candidate.ev > best.ev {
            candidate
        } else {
            best
        }
    });
    let chosen = match best_aggressive {
        Some(aggressive)
            if aggressive.ev > passive.ev + SIMPLER_LINE_MARGIN * spot.pot.as_f64() =>
        {
            aggressive
        }
        _ => spot.implied_odds_call(&passive, equity).unwrap_or(passive),
    };
    let response_plan = spot.response_plan(chosen.recommendation)?;
    let if_reraised = match chosen.recommendation {
        Recommendation::BetTo { total } | Recommendation::RaiseTo { total } => response_plan
            .as_ref()
            .and_then(|plan| planned_response_to(plan, Chips(total.0 * RAISE_MULTIPLIER))),
        Recommendation::Fold
        | Recommendation::Check
        | Recommendation::Call { .. }
        | Recommendation::AllIn { .. } => None,
    };
    let big_blind = table.big_blind();
    Ok(Advice {
        instruction: chosen.recommendation.instruction(big_blind),
        recommendation: chosen.recommendation,
        reason: spot.reason(&chosen, equity),
        if_reraised,
        response_plan,
        ev_by_action: values_best_first(passive_lines.iter().chain(&aggressive_lines), big_blind),
        confidence: Confidence::default(),
    })
}

/// Checks when it is free; otherwise calls when calling wins chips, and
/// folds when it does not.
fn best_passive_line(passive_lines: &[Candidate]) -> Candidate {
    let call = passive_lines
        .iter()
        .find(|candidate| matches!(candidate.line, Line::Check | Line::Call));
    match call {
        Some(&candidate) if matches!(candidate.line, Line::Check) || candidate.ev > 0.0 => {
            candidate
        }
        _ => passive_lines
            .iter()
            .copied()
            .find(|candidate| matches!(candidate.line, Line::Fold))
            .expect("passive_lines holds a fold whenever checking is not free"),
    }
}

fn values_best_first<'a>(
    candidates: impl Iterator<Item = &'a Candidate>,
    big_blind: Chips,
) -> Vec<ActionValue> {
    let mut values: Vec<ActionValue> = candidates
        .map(|candidate| ActionValue {
            recommendation: candidate.recommendation,
            instruction: candidate.recommendation.instruction(big_blind),
            ev: candidate.ev,
        })
        .collect();
    values.sort_by(|left, right| right.ev.total_cmp(&left.ev));
    values
}

/// The planned response to the smallest planned size of at least `total`,
/// or to the largest one when `total` is above them all.
fn planned_response_to(plan: &ResponsePlan, total: Chips) -> Option<PlannedResponse> {
    plan.responses
        .iter()
        .find(|planned| planned.total >= total)
        .or(plan.responses.last())
        .map(|planned| planned.response)
}

struct Opponent {
    index: SeatIndex,
    seat: Seat,
    range: Range,
    continuing_scale: f64,
    betting_scale: f64,
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
        reads: &TableReads,
    ) -> Result<Spot<'a>, AdviceError> {
        let hole = validated_cards_mask(&table.hero_cards());
        let board = validated_cards_mask(table.board());
        let dead_cards = hole | board;
        let opponent_seats = table.opponents_in_hand(table.hero());
        let ranges = estimate_ranges(table, charts, &opponent_seats, dead_cards, reads)?;
        let opponents = opponent_seats
            .iter()
            .zip(ranges)
            .map(|(&seat, range)| {
                let profile = reads.profile(seat);
                Opponent {
                    index: seat,
                    seat: *table
                        .seat(seat)
                        .expect("opponents_in_hand returns seats of the table"),
                    range,
                    continuing_scale: profile.map_or(1.0, |profile| profile.continuing_scale()),
                    betting_scale: profile.map_or(1.0, |profile| profile.betting_scale()),
                }
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

    /// Check when it is free; otherwise fold and call, whatever the call is
    /// worth.
    fn passive_lines(&self, equity: Probability) -> Vec<Candidate> {
        if self.legal.can_check() {
            return vec![Candidate {
                recommendation: Recommendation::Check,
                line: Line::Check,
                ev: self.realization * equity.fraction() * self.pot.as_f64(),
            }];
        }
        let call = self.legal.to_call;
        let realized = self.realization_for(call == self.hero.stack) * equity.fraction();
        vec![
            Candidate {
                recommendation: Recommendation::Fold,
                line: Line::Fold,
                ev: 0.0,
            },
            Candidate {
                recommendation: recommendation_for(Intent::Call, self.legal, &self.hero),
                line: Line::Call,
                ev: realized * (self.pot + call).as_f64() - call.as_f64(),
            },
        ]
    }

    /// One candidate per distinct legal bet or raise size.
    fn aggressive_lines(&self) -> Result<Vec<Candidate>, AdviceError> {
        let mut lines: Vec<Candidate> = Vec::new();
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
            lines.push(self.aggressive_line(recommendation, total)?);
        }
        Ok(lines)
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
    /// A player who folds to bets more or less often than that keeps a
    /// smaller or larger share. When anyone continues, the hero plays
    /// against all of them.
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
                Narrowing::continuing(continuing_share).scaled(opponent.continuing_scale),
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

    /// What to do when an opponent bets after a check, or raises after a
    /// bet or raise. `None` for other recommendations, for a check that ends
    /// the street, and for a bet that leaves no chips behind.
    fn response_plan(
        &self,
        recommendation: Recommendation,
    ) -> Result<Option<ResponsePlan>, AdviceError> {
        let plan = match recommendation {
            Recommendation::Check => self.plan_after_check()?,
            Recommendation::BetTo { total } | Recommendation::RaiseTo { total } => {
                self.plan_after_raise(total)?
            }
            Recommendation::Fold | Recommendation::Call { .. } | Recommendation::AllIn { .. } => {
                None
            }
        };
        Ok(plan.map(|(fold_above, responses)| ResponsePlan {
            summary: fold_above_summary(fold_above, recommendation, self.table.big_blind()),
            fold_above,
            responses,
        }))
    }

    /// Opponents who still act on this street after the hero checks are
    /// assumed to bet with their betting range; the others keep theirs.
    fn plan_after_check(&self) -> Result<Option<(CallLimit, Vec<PlannedBet>)>, AdviceError> {
        let hero_position = self.table.hero_position();
        let acts_after_hero = |opponent: &Opponent| {
            opponent.seat.status.can_act()
                && self
                    .table
                    .position_of(opponent.index)
                    .is_some_and(|position| !hero_position.is_in_position_against(position))
        };
        let mut anyone_can_bet = false;
        let mut ranges = Vec::with_capacity(self.opponents.len());
        for opponent in &self.opponents {
            if acts_after_hero(opponent) {
                anyone_can_bet = true;
                ranges.push(self.narrowed(
                    &opponent.range,
                    Narrowing::BET.scaled(opponent.betting_scale),
                ));
            } else {
                ranges.push(opponent.range.clone());
            }
        }
        if !anyone_can_bet {
            return Ok(None);
        }
        let equity = self.equity_against(&ranges)?;
        let all_in = self.hero.stack;
        let mut sizes: Vec<Chips> = PLANNED_BET_SIZES
            .iter()
            .map(|share| Chips((share * self.pot.as_f64()).round() as u64).min(all_in))
            .chain([all_in])
            .filter(|&size| size > Chips::ZERO)
            .collect();
        sizes.dedup();
        let responses = sizes
            .into_iter()
            .map(|bet| PlannedBet {
                total: bet,
                response: self.respond(bet.min(all_in), self.pot + bet, bet >= all_in, equity),
            })
            .collect();
        let realized = Probability::saturating(self.realization * equity.fraction());
        let fold_above = match max_callable_bet(self.pot, realized) {
            CallLimit::UpTo(limit) if limit < all_in => CallLimit::UpTo(limit),
            CallLimit::UpTo(_) | CallLimit::AnyBet => CallLimit::AnyBet,
        };
        Ok(Some((fold_above, responses)))
    }

    /// Raises to 2.5, 3 and 4 times `total` and all-in, against each
    /// opponent's raising range.
    fn plan_after_raise(
        &self,
        total: Chips,
    ) -> Result<Option<(CallLimit, Vec<PlannedBet>)>, AdviceError> {
        let hero_adds = total.saturating_sub(self.hero.committed_this_street);
        let behind_after_bet = self.hero.stack.saturating_sub(hero_adds);
        let raisers: Vec<&Opponent> = self
            .opponents
            .iter()
            .filter(|opponent| opponent.seat.status.can_act())
            .collect();
        if raisers.is_empty() || behind_after_bet == Chips::ZERO {
            return Ok(None);
        }
        let raising_ranges: Vec<Range> = raisers
            .iter()
            .map(|opponent| {
                self.narrowed(
                    &opponent.range,
                    Narrowing::RAISE.scaled(opponent.betting_scale),
                )
            })
            .collect();
        let equity = self.equity_against(&raising_ranges)?;

        let hero_all_in = total + behind_after_bet;
        let largest_raise = raisers
            .iter()
            .map(|opponent| opponent.stack_at_street_start())
            .max()
            .unwrap_or(Chips::ZERO)
            .min(hero_all_in);
        let pot_after_bet = self.pot + hero_adds;
        let raiser_committed = self.table.current_bet();
        let mut sizes: Vec<Chips> = PLANNED_RAISE_MULTIPLES
            .iter()
            .map(|multiple| Chips((multiple * total.as_f64()).round() as u64))
            .chain([Chips(total.0 * RAISE_MULTIPLIER)])
            .map(|raise_to| raise_to.min(largest_raise))
            .chain([largest_raise])
            .filter(|&raise_to| raise_to > total)
            .collect();
        sizes.sort();
        sizes.dedup();
        let responses = sizes
            .into_iter()
            .map(|raise_to| {
                let call = raise_to.saturating_sub(total).min(behind_after_bet);
                let raiser_adds = raise_to.saturating_sub(raiser_committed);
                PlannedBet {
                    total: raise_to,
                    response: self.respond(
                        call,
                        pot_after_bet + raiser_adds,
                        call == behind_after_bet,
                        equity,
                    ),
                }
            })
            .collect();
        let realized = self.realization * equity.fraction();
        let fold_above = match max_callable_raise(total, pot_after_bet, raiser_committed, realized)
        {
            Some(limit) if limit < hero_all_in => CallLimit::UpTo(limit),
            Some(_) | None => CallLimit::AnyBet,
        };
        Ok(Some((fold_above, responses)))
    }

    fn narrowed(&self, range: &Range, narrowing: Narrowing) -> Range {
        narrow(range, self.board, self.dead_cards, narrowing).range
    }

    /// Re-raise with 60% realized equity when chips stay behind, call when
    /// the equity beats the price, fold otherwise.
    fn respond(
        &self,
        call: Chips,
        pot_before_call: Chips,
        calling_is_all_in: bool,
        equity: Probability,
    ) -> PlannedResponse {
        let realized =
            Probability::saturating(self.realization_for(calling_is_all_in) * equity.fraction());
        if !calling_is_all_in && realized.fraction() >= RERAISE_EQUITY {
            PlannedResponse::ReRaise
        } else if realized >= required_equity(call, pot_before_call) {
            PlannedResponse::Call
        } else {
            PlannedResponse::Fold
        }
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

/// The largest raise-to an opponent can make over the hero's `total` that
/// the hero can still call with `equity`, or `None` when any raise is a call.
///
/// Calling a raise to `R` costs `R - total` into a pot of
/// `pot_after_bet + (R - raiser_committed)`, so the call breaks even while
/// `R * (1 - 2e) <= total + e * (pot_after_bet - raiser_committed - total)`.
fn max_callable_raise(
    total: Chips,
    pot_after_bet: Chips,
    raiser_committed: Chips,
    equity: f64,
) -> Option<Chips> {
    if equity >= 0.5 {
        return None;
    }
    let numerator = total.as_f64()
        + equity * (pot_after_bet.as_f64() - raiser_committed.as_f64() - total.as_f64());
    let limit = numerator / (1.0 - 2.0 * equity);
    Some(Chips(limit.max(total.as_f64()).floor() as u64))
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
    use crate::advisor::{advise, advise_with_reads, Advice, PlannedResponse, Recommendation};
    use crate::math::{legal_actions, CallLimit};
    use crate::opponent::{Frequency, PlayerProfile, PlayerStats, Population, TableReads};
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

    fn advice_against_big_blind(setup: TableSetup, big_blind: PlayerStats) -> Advice {
        let mut profiles = vec![None; setup.seats.len()];
        profiles[BIG_BLIND as usize] =
            Some(PlayerProfile::estimate(&big_blind, Population::PlayMoney));
        let reads = TableReads::from_profiles(profiles);
        advise_with_reads(&TableState::new(setup).unwrap(), &reads).unwrap()
    }

    fn rate(hits: u32, chances: u32) -> Frequency {
        Frequency { hits, chances }
    }

    fn nit() -> PlayerStats {
        PlayerStats {
            voluntarily_played: rate(60, 500),
            raised_preflop: rate(50, 500),
            three_bet: rate(5, 200),
            postflop_aggression: rate(30, 60),
            folded_to_continuation_bet: rate(80, 100),
            went_to_showdown: rate(10, 60),
        }
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

    fn is_aggressive(recommendation: Recommendation) -> bool {
        match recommendation {
            Recommendation::BetTo { .. }
            | Recommendation::RaiseTo { .. }
            | Recommendation::AllIn { .. } => true,
            Recommendation::Fold | Recommendation::Check | Recommendation::Call { .. } => false,
        }
    }

    #[test]
    fn bluffs_a_nit_but_not_a_calling_station() {
        let river_checked_to_hero = || {
            check(
                button_against_big_blind(BUTTON, "6c 5c", "Kh 7h 2d 9s Jd"),
                BIG_BLIND,
            )
        };
        let against_nit = advice_against_big_blind(river_checked_to_hero(), nit());
        assert!(
            is_aggressive(against_nit.recommendation),
            "{}",
            against_nit.reason
        );
        let against_station = advice_against_big_blind(river_checked_to_hero(), calling_station());
        assert_eq!(
            against_station.recommendation,
            Recommendation::Check,
            "{}",
            against_station.reason
        );
    }

    #[test]
    fn value_bets_thinner_and_bigger_against_a_calling_station() {
        let river_checked_to_hero = |hero_cards| {
            check(
                button_against_big_blind(BUTTON, hero_cards, "Kh 7h 2d 9s Qd"),
                BIG_BLIND,
            )
        };
        let third_pair = "9c 8c";
        let without_reads = advice(river_checked_to_hero(third_pair));
        assert_eq!(without_reads.recommendation, Recommendation::Check);
        let against_station =
            advice_against_big_blind(river_checked_to_hero(third_pair), calling_station());
        assert!(
            is_aggressive(against_station.recommendation),
            "{}",
            against_station.reason
        );

        let second_pair = "Qc Td";
        let bet_total = |advice: Advice| match advice.recommendation {
            Recommendation::BetTo { total } => total,
            other => panic!("expected a bet, got {other:?}"),
        };
        let usual_bet = bet_total(advice(river_checked_to_hero(second_pair)));
        let station_bet = bet_total(advice_against_big_blind(
            river_checked_to_hero(second_pair),
            calling_station(),
        ));
        assert!(station_bet > usual_bet);
    }

    #[test]
    fn plans_the_fold_limit_after_checking_a_medium_hand_out_of_position() {
        let advice = advice(button_against_big_blind(
            BIG_BLIND,
            "9c 8c",
            "Kh 7h 2d 9s Qd",
        ));
        assert_eq!(
            advice.recommendation,
            Recommendation::Check,
            "{}",
            advice.reason
        );
        let plan = advice.response_plan.expect("the button can still bet");
        let CallLimit::UpTo(limit) = plan.fold_above else {
            panic!("second pair cannot call every bet: {plan:?}");
        };
        assert!(
            plan.summary.starts_with("Fold if they bet more than"),
            "{}",
            plan.summary
        );
        assert!(plan
            .responses
            .windows(2)
            .all(|pair| pair[0].total < pair[1].total));
        for planned in &plan.responses {
            let folds = planned.response == PlannedResponse::Fold;
            assert_eq!(folds, planned.total > limit, "{planned:?} against {limit}");
        }
    }

    #[test]
    fn has_no_plan_when_checking_ends_the_street() {
        let advice = advice(check(
            button_against_big_blind(BUTTON, "3c 4d", "Kh Qh Jc 8s 2d"),
            BIG_BLIND,
        ));
        assert_eq!(
            advice.recommendation,
            Recommendation::Check,
            "{}",
            advice.reason
        );
        assert_eq!(advice.response_plan, None);
    }

    #[test]
    fn lists_every_compared_action_best_first() {
        let setup = bet(
            button_against_big_blind(BUTTON, "Ah 5h", "Kh 7h 2c"),
            BIG_BLIND,
            4,
        );
        let advice = advice(setup);
        let values = &advice.ev_by_action;
        assert!(values.windows(2).all(|pair| pair[0].ev >= pair[1].ev));
        assert!(values
            .iter()
            .any(|value| value.recommendation == Recommendation::Fold && value.ev == 0.0));
        assert!(values
            .iter()
            .any(|value| matches!(value.recommendation, Recommendation::Call { .. })));
        assert!(values
            .iter()
            .any(|value| matches!(value.recommendation, Recommendation::RaiseTo { .. })));
        if is_aggressive(advice.recommendation) {
            assert_eq!(values[0].recommendation, advice.recommendation);
        }
    }

    #[test]
    fn calls_any_raise_with_the_nut_flush() {
        let advice = advice(check(
            button_against_big_blind(BUTTON, "Ah 5h", "Kh 7h 2c 9h 4s"),
            BIG_BLIND,
        ));
        let plan = advice.response_plan.expect("a bet keeps chips behind");
        assert_eq!(plan.fold_above, CallLimit::AnyBet);
        assert_eq!(plan.summary, "Call any raise.");
        assert!(plan
            .responses
            .iter()
            .all(|planned| planned.response != PlannedResponse::Fold));
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

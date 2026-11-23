# Strategy engine plan

This plan turns the poker agent from an equity calculator into an advisor that tells the user exactly what to do on every decision: fold, check, call, or bet/raise to a specific amount, plus how much an opponent has to bet before the user should give up the hand.

Scope agreed for this branch (`feature/strategy-engine`):

- Use: study, play-money tables, home games and games against bots. Real-money sites ban real-time assistance tools, so the plan adds nothing for hiding the app from them.
- Format: No-Limit Hold'em cash game, 6-max, 100 big blind stacks as the reference depth. Fewer players at the table (5, 4, 3, heads-up) must work from day one because players leave and sit out. Tournaments and 9-max come later (phase 8).
- Strategy: a game-theory baseline (GTO) that cannot be exploited, plus exploitative adjustments once the app has seen enough hands from an opponent.

## 1. What "always win" can mean

No strategy wins every hand or every session. The cards are random, and an 80% favourite still loses one time in five. What the engine can do is take the action with the highest expected value (EV) every time. A player who does that wins money over many hands against anyone who does not. The engine is measured in big blinds won per 100 hands (bb/100) over simulated millions of hands, never by single results (see phase 7).

The user also needs to understand variance so they do not stop following the advice after a bad run. The UI will show the EV of each decision, not just the result of the hand.

## 2. Where the code stands today

| Area | Current state | Problem |
|---|---|---|
| `evaluator/decision.rs` | `equity > 50%` returns `Bet(200.0)`, otherwise `Fold` | Never checks or calls, fixed bet size, ignores price, position and stacks |
| `probability::calculate_equity` | Exact or Monte Carlo equity against N random hands | Opponents who raised do not hold random hands, so equity is overestimated in every contested pot |
| `probability::calculate_pot_odds` | Returns `equity * (pot + call)` | Name does not match what it computes, and it is not compared with anything |
| `evaluator/game_state.rs` `CvState` | `status: String`, `f64` money, opponent has only `opponent_bet` | No seats, button, stacks, folded players, blinds per seat, street, or action history |
| `cv_sidecar.py` | `count_active_opponents` returns `1` | The brain never sees the real number of players |
| Frontend | Manual entry of cards, opponents, pot, call | No position, no stacks, no advice beyond call or fold |

The fast hand evaluator and the exact/Monte Carlo equity code are good foundations. They stay, and range equity builds on them.

## 3. Inputs and how much each one weighs

These are the facts the engine needs, ordered by how much they change the correct decision in 6-max cash games.

| Input | Weight | Why it matters |
|---|---|---|
| Hole cards and board | Highest | Base hand strength and draws |
| Opponent range (what they can hold given their actions) | Highest | Equity against a raise is far lower than against random cards. This is the biggest error in the current code |
| Price: pot size and amount to call | Very high | Decides whether a call is profitable: required equity = call / (pot after call) |
| Position (who acts last after the flop) | High | The player in position wins more of their equity. Opening ranges go from about 15% of hands in the first seat to about 45% on the button |
| Number of players still in the hand | High | Every extra opponent cuts equity, cuts bluff success, and calls for stronger hands |
| Effective stack in big blinds and stack-to-pot ratio (SPR) | High | Decides how big the bets can be, whether implied odds exist, and when to go all-in |
| Action history this hand (who raised, how many times, sizes) | High | Narrows ranges. A 3-bet range is about 3 times narrower than an opening range |
| Board texture (dry, wet, paired, monotone) | Medium | Sets bet size and how often to bet |
| Opponent tendencies (VPIP, PFR, aggression, fold to c-bet) | Medium, grows with sample size | Exploit adjustments. Worth little until about 100 hands per player |
| Blind and ante sizes | Medium | Everything is normalised to big blinds. Antes and straddles make the pot bigger and widen ranges |
| Players who folded or sit out | Medium | Changes positions (the table "shrinks") and the number of opponents |
| Rake | Low for this scope | Play money and home games have none. Kept as a setting at 0 |

Everything is converted to big blinds before any decision so that the same logic works at any stake.

## 4. Core formulas

All money is kept as integer chips in a `Chips(u64)` newtype and converted to `BigBlinds(f64)` only for display and for table lookups.

### Price of a call

```
required_equity = call / (pot_before_call + call)
```

`pot_before_call` includes the opponent's bet. Calling 50 into a pot of 150 needs 50 / 200 = 25%.

### Maximum bet the user can call (the "fold above X" number)

The user asked how much an opponent has to bet before the user should fold. For a pot `P` before the opponent bets and a hero equity `e` (already reduced by equity realization, below):

```
call is profitable while  B / (P + 2B) <= e
max_callable_bet = e * P / (1 - 2e)      for e < 0.5
any bet can be called                     for e >= 0.5
```

With 30% equity in a pot of 100, the user can call bets up to 0.3 * 100 / 0.4 = 75. The advice screen shows this number every time the user checks or bets, so the user already knows the answer when the opponent acts.

### Equity realization

A hand does not win its full equity because it often gets folded before showdown. Realized equity = equity x R, with R as the starting values below (published solver studies give similar figures; to be tuned in phase 7):

| Situation | R |
|---|---|
| In position, suited or connected hands | 1.00 to 1.10 |
| In position, offsuit hands | 0.90 |
| Out of position, suited or connected | 0.85 |
| Out of position, offsuit or weak pairs | 0.70 to 0.80 |
| All-in (no more decisions) | 1.00 |

### Implied odds

When a draw is too weak for the current price, extra money won later can still make the call correct:

```
extra_needed_later = call / e_hit - (pot_before_call + call)
```

The call is allowed only if `extra_needed_later` is at most a realistic share of the remaining effective stack (starting value: 25% of it, higher against loose players).

### Bluffing and defending

| Formula | Meaning |
|---|---|
| `bluff_breakeven = B / (P + B)` | How often a bluff of size B into pot P must make the opponent fold to break even |
| `minimum_defence_frequency = P / (P + B)` | How much of our range we must continue with so the opponent cannot bluff profitably with any two cards |
| `river_bluff_share = B / (P + 2B)` | Share of bluffs in a balanced river betting range: 25% for half pot, 33% for pot, 40% for twice pot |

### EV of a bet or raise

```
EV(bet) = fold_probability * P
        + (1 - fold_probability) * (realized_equity * (P + B + call) - B)
```

The engine computes the EV of fold (0), check or call, and two to four bet sizes, and picks the highest. This is how it chooses both the action and the amount.

## 5. Phases

Each phase ends with passing tests and the checks in AGENTS.md (fmt, clippy, tests, `pnpm build`). Each phase is one pull request.

### Phase 0: Table model and data contract

Goal: the brain knows the full table, not just two cards and an opponent count.

New types in `poker_agent` (names are proposals):

- `Chips(u64)`, `BigBlinds(f64)`
- `SeatIndex(u8)`, `Seat { stack, committed_this_street, status }`
- `SeatStatus { Empty, SittingOut, Active, Folded, AllIn }`
- `Position { UnderTheGun, Hijack, Cutoff, Button, SmallBlind, BigBlind }`, assigned from the button and the active seats. With fewer than 6 players, positions are removed from the front: 5 players have no UTG, 4 players start at the cutoff, heads-up is button/small blind against big blind.
- `Street { Preflop, Flop, Turn, River }`
- `Blinds { small, big, ante, straddle }`
- `PlayerAction { Fold, Check, Call(Chips), Bet(Chips), Raise { to: Chips }, AllIn(Chips) }`
- `HandHistory`: ordered actions per street, used to build ranges
- `TableState`: everything above plus hero seat, hole cards, board, pot, amount to call, minimum raise

`CvState` is replaced by a versioned JSON contract (`"version": 2`) that the sidecar must fill: button seat, hero seat, each seat's stack, bet and status, blinds, board, hero cards. A `HandTracker` in Rust compares consecutive frames to rebuild the action history (a stack that drops by X on seat 3 means seat 3 put in X). It starts a new hand when the hole cards or the button change.

Manual entry stays as a first-class input path, because CV will not read every table correctly. The frontend gets a "table" form: seats, button, stacks, who folded.

Tests: position assignment for 2 to 6 players, effective stack, action reconstruction from frame diffs, rejection of impossible states (two buttons, negative stacks).

### Phase 1: Pot mathematics

Goal: correct price logic, independent of ranges.

- `required_equity`, `max_callable_bet`, `minimum_defence_frequency`, `bluff_breakeven`, `stack_to_pot_ratio`, `effective_stack`, implied odds.
- Remove the misnamed `calculate_pot_odds`.
- Legal action generation: minimum raise is the previous raise size, all-in rules, short all-in that does not reopen betting.

Tests: worked examples from section 4 with exact numbers, and edge cases (call larger than stack becomes all-in for less).

### Phase 2: Ranges and range equity

Goal: equity against what the opponent can actually hold.

- `Range`: weights for the 1326 two-card combos (`[f32; 1326]`, no heap), built from the 169 hand classes (AA, AKs, AKo...). Card removal: combos that use known cards get weight 0.
- Parser for the usual notation (`22+, A2s+, KTs+, AJo+`) so charts are readable data files.
- `equity_vs_ranges(hero, board, &[Range])`: exact on the river and turn when cheap, weighted Monte Carlo otherwise. Reuses `fast_eval`.
- Precomputed 169 x 169 preflop equity table, generated once and embedded with `include_bytes!`.
- Performance budget: under 150 ms for one opponent on the flop, under 400 ms for three opponents. Measured with `examples/equity_bench.rs`.

Tests: AA against a random range matches the current function; known matchups (AKs against QQ about 46%); range equity is the weighted average of combo equities.

### Phase 3: Preflop engine

Goal: a correct action for every preflop spot. Preflop is where a beginner loses most money, and it is solvable with lookup tables.

Spots, each a chart keyed by position and stack depth (100 bb first, then 40 bb and 20 bb):

1. Raise first in (everyone folded to us)
2. Facing an open: fold, call, or 3-bet
3. Facing a 3-bet: fold, call, or 4-bet
4. Facing a 4-bet: fold or all-in
5. Squeeze (open plus callers before us)
6. Facing limpers (frequent in home and play-money games): raise or fold, rarely limp behind
7. Big blind defence, small blind strategy
8. Short stack (effective stack 15 bb or less): push or fold from Nash tables

Starting opening ranges for 6-max at 100 bb, to be replaced by solver output:

| Position | Share of hands | Range |
|---|---|---|
| UTG | about 16% | 22+, A2s+, KTs+, QTs+, JTs, T9s, 98s, AJo+, KQo |
| Hijack | about 19% | UTG plus K9s, Q9s, J9s, 87s, ATo, KJo |
| Cutoff | about 27% | 22+, A2s+, K6s+, Q8s+, J8s+, T8s+, 97s+, 86s+, 76s, 65s, 54s, A8o+, KTo+, QTo+, JTo |
| Button | about 45% | 22+, A2s+, K2s+, Q4s+, J6s+, T6s+, 96s+, 85s+, 74s+, 64s+, 53s+, 43s, A2o+, K8o+, Q9o+, J9o+, T8o+, 98o |
| Small blind | about 40% | raise or fold, close to the cutoff range plus suited kings and queens |

Sizing rules:

| Spot | Size |
|---|---|
| Open | 2.5 bb (small blind 3 bb), plus 1 bb per limper |
| 3-bet in position | 3 x the open |
| 3-bet out of position | 3.5 to 4 x the open |
| Squeeze | 3-bet size plus 1 open per caller |
| 4-bet | 2.2 to 2.5 x the 3-bet; all-in when that commits more than a third of the stack |

Chart source: charts are data files (`data/preflop/*.ranges`) loaded at build time. The first version uses the hand-written starting ranges above. A later task generates them with an open-source solver run offline. Candidates are TexasSolver and the `postflop-solver` crate; both are AGPL licensed as far as I know, so they run as separate tools that produce data files and are never linked into the app. The license must be checked before use.

Tests: every hand in every position gets exactly one action; AA never folds preflop; 72o never opens from UTG; the advice matches the chart for a table of known spots.

### Phase 4: Postflop engine

Goal: for each flop, turn and river decision, pick the action and size with the highest EV against the opponent's range.

Steps for each decision:

1. Build each opponent's range: preflop range from phase 3 given their actions, then narrowed street by street (a player who calls a big turn bet keeps made hands and strong draws, and drops air).
2. Classify the board: high or low, dry or wet, paired, monotone, and who has the range advantage (more strong hands) and nut advantage (more of the very best hands).
3. Classify the hero hand against the board: nuts, strong value, medium showdown value, draw (with outs and whether they are clean), air.
4. Compute EV of each candidate action with section 4 formulas, using range equity, realization factor and the opponent's fold probability.
5. Choose the action. Tie-break toward the simpler line (check over a marginal bet) so the advice is easier to follow.

Starting heuristics that steer candidate sizes (EV still decides):

| Situation | Typical action |
|---|---|
| Preflop raiser, dry high board (A72 rainbow), heads-up | Small c-bet, 25 to 33% pot, with most of the range |
| Preflop raiser, wet middle board (987 two-tone) | Check more often; when betting, 66 to 75% pot |
| Multiway pot (3+ players) | Bet much less often, mostly strong hands and strong draws |
| Low SPR (under 3) with top pair or better | Plan to get all-in |
| River, polarised range | Large bets (75% pot to all-in); bluff share from section 4 |
| River, medium hand | Check, then call or fold against the `max_callable_bet` |

Facing a bet: compare realized equity with required equity, add implied odds for draws, and check the defence frequency so the hero does not over-fold. Raise when EV(raise) beats EV(call).

Tests: invariants (never fold when checking is free; never bet below the minimum; never call when all-in EV is clearly negative), plus a set of hand-picked spots with the expected answer.

### Phase 5: Opponent model and exploits

Goal: earn more than the GTO baseline against the weak players common in play-money and home games.

Statistics tracked per player name or seat:

| Stat | Meaning | Hands needed before trusting it |
|---|---|---|
| VPIP | Share of hands where the player puts money in voluntarily | about 50 to 100 |
| PFR | Share of hands raised preflop | about 50 to 100 |
| 3-bet % | Re-raise frequency preflop | about 300 |
| Aggression factor | (bets + raises) / calls | about 200 |
| Fold to c-bet | Folds when facing a continuation bet | about 50 occurrences |
| Went to showdown | Reaches showdown after seeing the flop | about 200 |

Small samples are handled with Bayesian shrinkage instead of hard cut-offs: each stat starts from a population prior worth about 30 hands and moves toward the observed value as hands come in. There are two population priors: "regular" and "play money / home game" (much looser and more passive: VPIP around 40%, fold to c-bet low).

Player types and starting thresholds:

| Type | VPIP | PFR | Main adjustment |
|---|---|---|---|
| Nit | under 16 | under 13 | Steal their blinds more; fold to their raises unless strong; bluff more |
| Regular (TAG) | 18 to 25 | 15 to 22 | Stay close to the baseline |
| Loose aggressive | 26 to 35 | 21 to 30 | 3-bet and call down a bit wider |
| Calling station | over 35 | under 15 | Never bluff; value bet thinner and bigger |
| Maniac | over 45 | over 35 | Let them bet; call down lighter; trap with strong hands |

The exploit layer changes two things only: the opponent ranges used in range equity, and the fold probability used in the EV formula. The decision logic stays the same, so exploits never bypass the math.

Tests: stats from a synthetic hand log; shrinkage converges to the observed rate; a calling station profile removes bluffs from the advice.

### Phase 6: Advice output and UI

Goal: a person with no poker knowledge can follow the instructions without thinking.

`Advice` returned by the brain:

- `action`: Fold, Check, Call, Bet or Raise, All-in
- `amount`: exact chips and big blinds ("Raise to 750 (7.5 BB)"), with raises always expressed as "raise to"
- `response_plan`: what to do after the opponent acts, prepared before they act. For each typical bet size (33%, 50%, 75%, pot, all-in): call, fold or raise. Includes the single "fold if they bet more than X" number.
- `reason`: one plain sentence ("You need 25% to call and have about 38%")
- `ev_by_action`: shown in a collapsible detail view for study
- `confidence`: low when the CV reading is uncertain or the opponent range is wide guesswork

UI changes: a large single instruction with a colour per action, the response plan under it, a warning banner when the table reading looks wrong, and a study view that replays the last hands with the EV of each choice. Text in the UI follows the Humanizer rules in AGENTS.md.

### Phase 7: Validation by simulation

Goal: prove the engine wins, with numbers.

- A small No-Limit Hold'em game engine in Rust (new crate `poker_sim`) that deals hands and enforces the rules from phase 1.
- Bot pool: random, calling station, nit, maniac, the old `make_decision`, and the new engine itself.
- Run millions of hands per matchup with fixed seeds and duplicate dealing (both sides play the same cards) to cut variance.
- Report bb/100 with a 95% confidence interval.

Acceptance targets: clearly positive against every exploitable bot, positive against the old engine, close to 0 against itself. Tuning of R, bet sizes and thresholds in phases 4 and 5 happens here, against these numbers.

### Phase 8: Later extensions

- Tournaments: blind levels, ICM (chip value is not linear near the money), push/fold charts by stack depth.
- 9-max: wider position set (UTG+1, UTG+2, lojack) and tighter early ranges.
- Hand history import (PokerStars and GGPoker text format) for study and to build opponent stats offline.

## 6. Proposed module layout

```
src-tauri/crates/poker_agent/src/
├── core/            existing cards, deck, hand
├── table/           Chips, Position, Seat, TableState, HandTracker      (phase 0)
├── math/            pot odds, MDF, SPR, implied odds, legal actions     (phase 1)
├── range/           Range, notation parser, range equity                (phase 2)
├── preflop/         chart loading and preflop decisions                 (phase 3)
├── postflop/        board texture, hand class, EV of actions            (phase 4)
├── opponent/        stats, priors, player types                         (phase 5)
├── advisor/         Advice, response plan, top-level decide()           (phase 6)
└── evaluator/       existing fast_eval; decision.rs replaced by advisor
data/preflop/        range files per position and spot
src-tauri/crates/poker_sim/                                              (phase 7)
```

## 7. Risks and open points

- CV accuracy limits everything. Wrong stacks or missed folds produce wrong ranges. Mitigation: the `confidence` field, a consistency check (chips must add up between frames), and manual correction in the UI.
- Reconstructing actions from screenshots misses fast actions between frames. The sidecar may need to read the action labels that poker clients show next to each seat.
- Starting ranges and realization factors are approximations. Phase 7 is where they get tuned; until then the advice is sound but not optimal.
- Range equity on the flop with several opponents is the most expensive computation. If it exceeds the time budget, fall back to precomputed equity buckets.
- Solver licenses (AGPL) must be checked before any solver output is committed.

## 8. Order of work

Phases 0, 1 and 2 come first and can be built and tested without any UI change. Phase 3 alone already fixes most beginner losses. Phase 7 can start in parallel with phase 4, since the simulator only needs phases 0 and 1.

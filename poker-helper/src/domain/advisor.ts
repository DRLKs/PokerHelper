export type Chips = number;
export type SeatIndex = number;

export type SeatStatus = 'empty' | 'sitting_out' | 'active' | 'folded' | 'all_in';

export interface Blinds {
  small: Chips;
  big: Chips;
  ante: Chips;
}

export interface SnapshotSeat {
  stack: Chips;
  committed_this_street: Chips;
  status: SeatStatus;
  name?: string;
}

export const SNAPSHOT_VERSION = 2;

export interface TableSnapshot {
  version: typeof SNAPSHOT_VERSION;
  button_seat: SeatIndex;
  hero_seat: SeatIndex;
  blinds: Blinds;
  seats: SnapshotSeat[];
  hero_cards: [string, string];
  board: string[];
  pot: Chips;
}

export type Street = 'Preflop' | 'Flop' | 'Turn' | 'River';

export type PlayerAction =
  | 'Fold'
  | 'Check'
  | { Call: { to: Chips } }
  | { Bet: { to: Chips } }
  | { Raise: { to: Chips } };

export interface RecordedAction {
  seat: SeatIndex;
  street: Street;
  action: PlayerAction;
}

export type Recommendation =
  | { kind: 'fold' }
  | { kind: 'check' }
  | { kind: 'call'; amount: Chips }
  | { kind: 'bet_to'; total: Chips }
  | { kind: 'raise_to'; total: Chips }
  | { kind: 'all_in'; total: Chips };

export type RecommendationKind = Recommendation['kind'];

export type PlannedResponse = 'fold' | 'call' | 're_raise';

export type CallLimit = 'AnyBet' | { UpTo: Chips };

export interface PlannedBet {
  total: Chips;
  response: PlannedResponse;
}

export interface ResponsePlan {
  fold_above: CallLimit;
  responses: PlannedBet[];
  summary: string;
}

export interface ActionValue {
  recommendation: Recommendation;
  instruction: string;
  ev: number;
}

export type ConfidenceLevel = 'low' | 'medium' | 'high';

export type ConfidenceIssue =
  | { kind: 'missing_history'; seat: SeatIndex }
  | { kind: 'many_opponents' }
  | { kind: 'chips_do_not_add_up' };

export interface Confidence {
  level: ConfidenceLevel;
  issues: ConfidenceIssue[];
}

export interface Advice {
  recommendation: Recommendation;
  instruction: string;
  reason: string;
  if_reraised: PlannedResponse | null;
  response_plan: ResponsePlan | null;
  ev_by_action: ActionValue[];
  confidence: Confidence;
}

export type PlayerType = 'nit' | 'regular' | 'loose_aggressive' | 'calling_station' | 'maniac';

export interface PlayerProfile {
  hands: number;
  voluntarily_played: number;
  raised_preflop: number;
  three_bet: number;
  postflop_aggression: number;
  folded_to_continuation_bet: number;
  went_to_showdown: number;
  player_type: PlayerType;
}

export interface SeatRead {
  seat: SeatIndex;
  player: string;
  profile: PlayerProfile;
}

export interface TableUpdate {
  advice: Advice | null;
  advice_error: string | null;
  street: Street;
  history: RecordedAction[];
  opponents: SeatRead[];
  recorded_a_finished_hand: boolean;
}

export type Population = 'play_money' | 'regular';

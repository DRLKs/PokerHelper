import {
  Chips,
  ConfidenceIssue,
  PlannedResponse,
  PlayerAction,
  PlayerType,
  RecommendationKind,
  SeatIndex,
  SeatStatus,
} from '../domain/advisor';

export const seatLabel = (seat: SeatIndex): string => `Seat ${seat + 1}`;

export const chipsWithBigBlinds = (chips: Chips, bigBlind: Chips): string => {
  if (bigBlind <= 0) return `${chips}`;
  return `${chips} (${(chips / bigBlind).toFixed(1)} BB)`;
};

export const percent = (fraction: number): string => `${Math.round(fraction * 100)}%`;

export const signedChips = (ev: number): string => {
  const rounded = ev.toFixed(1);
  return ev > 0 ? `+${rounded}` : rounded;
};

export const seatStatusLabels: Record<SeatStatus, string> = {
  active: 'Playing',
  folded: 'Folded',
  all_in: 'All-in',
  sitting_out: 'Sitting out',
  empty: 'Empty',
};

export const playerTypeLabels: Record<PlayerType, string> = {
  nit: 'Nit',
  regular: 'Regular',
  loose_aggressive: 'Loose aggressive',
  calling_station: 'Calling station',
  maniac: 'Maniac',
};

export const plannedResponseLabels: Record<PlannedResponse, string> = {
  fold: 'Fold',
  call: 'Call',
  re_raise: 'Raise',
};

export const recommendationTone = (kind: RecommendationKind): string => {
  switch (kind) {
    case 'fold':
      return 'advice-fold';
    case 'check':
      return 'advice-check';
    case 'call':
      return 'advice-call';
    case 'bet_to':
    case 'raise_to':
      return 'advice-raise';
    case 'all_in':
      return 'advice-all-in';
  }
};

export const describeAction = (action: PlayerAction): string => {
  if (action === 'Fold') return 'folds';
  if (action === 'Check') return 'checks';
  if ('Call' in action) return `calls ${action.Call.to}`;
  if ('Bet' in action) return `bets ${action.Bet.to}`;
  return `raises to ${action.Raise.to}`;
};

export const describeIssue = (issue: ConfidenceIssue): string => {
  switch (issue.kind) {
    case 'missing_history':
      return `${seatLabel(issue.seat)} reached the flop with no preflop action on record, so its range is a guess.`;
    case 'many_opponents':
      return 'Three or more players are in the pot, so the estimates are rougher.';
    case 'chips_do_not_add_up':
      return 'The chips changed since the last update without adding up. Check the stacks and bets.';
  }
};

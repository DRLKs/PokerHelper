import { Blinds, Chips, SeatIndex, SeatStatus, SNAPSHOT_VERSION, SnapshotSeat, TableSnapshot } from './advisor';
import { Card } from './types';

export const SEAT_COUNT = 6;
export const HOLE_CARD_COUNT = 2;
export const MAX_BOARD_CARDS = 5;

const VALID_BOARD_SIZES = [0, 3, 4, 5];
const DEFAULT_BLINDS: Blinds = { small: 1, big: 2, ante: 0 };
const DEFAULT_STACK_IN_BIG_BLINDS = 100;
const DEFAULT_HERO_SEAT = 3;

export interface SeatDraft {
  name: string;
  status: SeatStatus;
  stack: Chips;
  committed: Chips;
}

/** The table as the user is entering it, before it becomes a snapshot. */
export interface TableDraft {
  seats: SeatDraft[];
  buttonSeat: SeatIndex;
  heroSeat: SeatIndex;
  blinds: Blinds;
  heroCards: Card[];
  board: Card[];
  /** Chips from earlier streets and antes, not the bets in front of the players. */
  pot: Chips;
}

export type SnapshotResult =
  | { kind: 'ready'; snapshot: TableSnapshot }
  | { kind: 'incomplete'; problem: string };

export const cardCode = (card: Card): string => `${card.rank}${card.suit}`;

export const isDealtIn = (status: SeatStatus): boolean => {
  switch (status) {
    case 'active':
    case 'folded':
    case 'all_in':
      return true;
    case 'empty':
    case 'sitting_out':
      return false;
  }
};

const toSnapshotSeat = (seat: SeatDraft): SnapshotSeat => {
  const name = seat.name.trim();
  const snapshotSeat: SnapshotSeat = {
    stack: seat.stack,
    committed_this_street: seat.committed,
    status: seat.status,
  };
  return name === '' ? snapshotSeat : { ...snapshotSeat, name };
};

/**
 * Turns the draft into a version 2 snapshot, or names the first thing the
 * user still has to fill in. The backend checks everything else.
 */
export const toSnapshot = (draft: TableDraft): SnapshotResult => {
  if (draft.heroCards.length !== HOLE_CARD_COUNT) {
    return { kind: 'incomplete', problem: 'Pick both of your cards.' };
  }
  if (!VALID_BOARD_SIZES.includes(draft.board.length)) {
    return { kind: 'incomplete', problem: 'The board needs 0, 3, 4 or 5 cards.' };
  }
  const [firstCard, secondCard] = draft.heroCards;
  return {
    kind: 'ready',
    snapshot: {
      version: SNAPSHOT_VERSION,
      button_seat: draft.buttonSeat,
      hero_seat: draft.heroSeat,
      blinds: draft.blinds,
      seats: draft.seats.map(toSnapshotSeat),
      hero_cards: [cardCode(firstCard), cardCode(secondCard)],
      board: draft.board.map(cardCode),
      pot: draft.pot,
    },
  };
};

const nextDealtInSeat = (seats: SeatDraft[], from: SeatIndex): SeatIndex | null => {
  for (let offset = 1; offset <= seats.length; offset++) {
    const index = (from + offset) % seats.length;
    if (isDealtIn(seats[index].status)) return index;
  }
  return null;
};

const payFromStack = (seat: SeatDraft, amount: Chips): { seat: SeatDraft; paid: Chips } => {
  const paid = Math.min(seat.stack, amount);
  const stack = seat.stack - paid;
  return { seat: { ...seat, stack, status: stack === 0 ? 'all_in' : seat.status }, paid };
};

const postAntes = (draft: TableDraft): TableDraft => {
  let pot = draft.pot;
  const seats = draft.seats.map((seat) => {
    if (!isDealtIn(seat.status)) return seat;
    const { seat: paidSeat, paid } = payFromStack(seat, draft.blinds.ante);
    pot += paid;
    return paidSeat;
  });
  return { ...draft, seats, pot };
};

const postBlind = (seats: SeatDraft[], index: SeatIndex, amount: Chips): SeatDraft[] =>
  seats.map((seat, seatIndex) => {
    if (seatIndex !== index) return seat;
    const { seat: paidSeat, paid } = payFromStack(seat, amount);
    return { ...paidSeat, committed: paidSeat.committed + paid };
  });

const blindSeats = (seats: SeatDraft[], button: SeatIndex): { small: SeatIndex; big: SeatIndex } | null => {
  const headsUp = seats.filter((seat) => isDealtIn(seat.status)).length === 2;
  const small = headsUp ? button : nextDealtInSeat(seats, button);
  if (small === null) return null;
  const big = nextDealtInSeat(seats, small);
  if (big === null) return null;
  return { small, big };
};

const postForcedBets = (draft: TableDraft): TableDraft => {
  const withAntes = postAntes(draft);
  const blinds = blindSeats(withAntes.seats, withAntes.buttonSeat);
  if (blinds === null) return withAntes;
  const afterSmallBlind = postBlind(withAntes.seats, blinds.small, draft.blinds.small);
  const seats = postBlind(afterSmallBlind, blinds.big, draft.blinds.big);
  return { ...withAntes, seats };
};

const resetSeatForNewHand = (seat: SeatDraft): SeatDraft => {
  const cleared = { ...seat, committed: 0 };
  if (!isDealtIn(seat.status)) return cleared;
  return { ...cleared, status: seat.stack === 0 ? 'sitting_out' : 'active' };
};

/**
 * Starts the next hand: clears the cards, bets and pot, moves the button to
 * the next player, and posts the antes and blinds. Players left without
 * chips sit out.
 */
export const startNewHand = (draft: TableDraft): TableDraft => {
  const seats = draft.seats.map(resetSeatForNewHand);
  const buttonSeat = nextDealtInSeat(seats, draft.buttonSeat) ?? draft.buttonSeat;
  return postForcedBets({ ...draft, seats, buttonSeat, heroCards: [], board: [], pot: 0 });
};

/** Moves the bets in front of the players into the pot. */
export const collectBets = (draft: TableDraft): TableDraft => {
  const collected = draft.seats.reduce((total, seat) => total + seat.committed, 0);
  return {
    ...draft,
    pot: draft.pot + collected,
    seats: draft.seats.map((seat) => ({ ...seat, committed: 0 })),
  };
};

export const initialDraft = (): TableDraft => {
  const stack = DEFAULT_BLINDS.big * DEFAULT_STACK_IN_BIG_BLINDS;
  const seats = Array.from({ length: SEAT_COUNT }, (): SeatDraft => ({
    name: '',
    status: 'active',
    stack,
    committed: 0,
  }));
  return postForcedBets({
    seats,
    buttonSeat: 0,
    heroSeat: DEFAULT_HERO_SEAT,
    blinds: DEFAULT_BLINDS,
    heroCards: [],
    board: [],
    pot: 0,
  });
};

export interface Card {
  rank: string; // "2", "3", ..., "T", "J", "Q", "K", "A"
  suit: string; // "h", "d", "c", "s"
}

export interface GameState {
  myCards: Card[];
  communityCards: Card[];
  numOpponents: number;
}

export interface ProbabilityResult {
  equity: number;
}

export type Rank = "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "T" | "J" | "Q" | "K" | "A";
export type Suit = "h" | "d" | "c" | "s";

export interface Card {
  rank: Rank;
  suit: Suit;
}

export interface GameState {
  myCards: Card[];
  communityCards: Card[];
  numOpponents: number;
}

export interface ProbabilityResult {
  equity: number;
}

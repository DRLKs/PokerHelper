export type Card = {
  suit: 'hearts' | 'diamonds' | 'clubs' | 'spades' | '';
  rank: 'A' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '10' | 'J' | 'Q' | 'K' | '';
};

export type Suit = {
  value: 'hearts' | 'diamonds' | 'clubs' | 'spades';
  label: string;
  name: string;
};

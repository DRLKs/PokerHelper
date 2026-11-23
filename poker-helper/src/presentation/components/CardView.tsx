import React from 'react';
import { Card } from '../../domain/types';

interface Props {
  card: Card;
  onClick?: () => void;
}

const suits: Record<string, { symbol: string; name: string; color: string }> = {
  h: { symbol: '♥', name: 'hearts', color: 'text-[#d64b55]' },
  d: { symbol: '♦', name: 'diamonds', color: 'text-[#4387d7]' },
  c: { symbol: '♣', name: 'clubs', color: 'text-[#24856a]' },
  s: { symbol: '♠', name: 'spades', color: 'text-[#202329]' },
};

export const CardView: React.FC<Props> = ({ card, onClick }) => {
  const suit = suits[card.suit] ?? suits.s;
  const label = `${card.rank} of ${suit.name}`;

  return (
    <button
      type="button"
      onClick={onClick}
      className="playing-card group"
      aria-label={onClick ? `Remove ${label}` : label}
      title={onClick ? `Remove ${label}` : label}
    >
      <span className={`card-corner ${suit.color}`}>
        <span>{card.rank}</span>
        <span className="text-[0.72em] leading-none">{suit.symbol}</span>
      </span>
      <span className={`card-suit ${suit.color}`} aria-hidden="true">
        {suit.symbol}
      </span>
      {onClick && (
        <span className="card-remove" aria-hidden="true">×</span>
      )}
    </button>
  );
};

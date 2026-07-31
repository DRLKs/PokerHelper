import React, { useMemo, useState } from 'react';
import { Card } from '../../domain/types';

interface Props {
  onSelect: (card: Card) => void;
  unavailableCards?: Card[];
  destinationLabel: string;
}

const ranks = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];
const suits = [
  { value: 'h', symbol: '♥', name: 'Hearts', className: 'suit-hearts' },
  { value: 'd', symbol: '♦', name: 'Diamonds', className: 'suit-diamonds' },
  { value: 'c', symbol: '♣', name: 'Clubs', className: 'suit-clubs' },
  { value: 's', symbol: '♠', name: 'Spades', className: 'suit-spades' },
];

export const CardSelector: React.FC<Props> = ({
  onSelect,
  unavailableCards = [],
  destinationLabel,
}) => {
  const [selectedRank, setSelectedRank] = useState('A');
  const unavailable = useMemo(
    () => new Set(unavailableCards.map((card) => `${card.rank}${card.suit}`)),
    [unavailableCards],
  );

  return (
    <section className="panel p-5 sm:p-6" aria-labelledby="card-picker-title">
      <div className="mb-5 flex items-start justify-between gap-4">
        <div>
          <p className="eyebrow">Card picker</p>
          <h2 id="card-picker-title" className="mt-1 text-lg font-semibold text-white">
            Add to {destinationLabel}
          </h2>
        </div>
        <span className="selection-chip">{selectedRank} selected</span>
      </div>

      <div className="rank-grid" aria-label="Select a rank">
        {ranks.map((rank) => (
          <button
            type="button"
            key={rank}
            onClick={() => setSelectedRank(rank)}
            className={`rank-button ${selectedRank === rank ? 'rank-button-active' : ''}`}
            aria-pressed={selectedRank === rank}
          >
            {rank}
          </button>
        ))}
      </div>

      <div className="mt-4 grid grid-cols-4 gap-2" aria-label="Select a suit">
        {suits.map((suit) => {
          const disabled = unavailable.has(`${selectedRank}${suit.value}`);
          return (
            <button
              type="button"
              key={suit.value}
              onClick={() => onSelect({ rank: selectedRank, suit: suit.value })}
              disabled={disabled}
              className={`suit-button ${suit.className}`}
              aria-label={`${selectedRank} of ${suit.name}`}
            >
              <span className="text-2xl leading-none" aria-hidden="true">{suit.symbol}</span>
              <span className="hidden text-xs font-medium sm:block">{suit.name}</span>
            </button>
          );
        })}
      </div>
      <p className="mt-3 text-xs text-slate-500">Cards already in play are disabled.</p>
    </section>
  );
};

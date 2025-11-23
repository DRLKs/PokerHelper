import React, { useState } from 'react';
import { Card } from '../../domain/types';

interface Props {
  onSelect: (card: Card) => void;
}

const ranks = ['2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];
const suits = [
  { value: 'h', symbol: '♥', color: 'text-red-500' },
  { value: 'd', symbol: '♦', color: 'text-blue-500' },
  { value: 'c', symbol: '♣', color: 'text-green-500' },
  { value: 's', symbol: '♠', color: 'text-gray-300' },
];

export const CardSelector: React.FC<Props> = ({ onSelect }) => {
  const [selectedRank, setSelectedRank] = useState<string | null>(null);

  const handleRankClick = (rank: string) => {
    setSelectedRank(rank);
  };

  const handleSuitClick = (suit: string) => {
    if (selectedRank) {
      onSelect({ rank: selectedRank, suit });
      setSelectedRank(null); // Reset after selection
    }
  };

  return (
    <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
      <h3 className="text-gray-400 mb-2 text-sm uppercase tracking-wider">Select Card</h3>
      
      {/* Ranks */}
      <div className="grid grid-cols-7 gap-2 mb-4">
        {ranks.map(rank => (
          <button
            key={rank}
            onClick={() => handleRankClick(rank)}
            className={`
              p-2 rounded text-center font-bold transition-colors
              ${selectedRank === rank 
                ? 'bg-poker-accent text-black' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}
            `}
          >
            {rank}
          </button>
        ))}
      </div>

      {/* Suits (only show if rank selected) */}
      {selectedRank && (
        <div className="flex justify-center gap-4 animate-fade-in">
          {suits.map(suit => (
            <button
              key={suit.value}
              onClick={() => handleSuitClick(suit.value)}
              className={`
                w-12 h-12 rounded-full bg-gray-700 flex items-center justify-center text-2xl
                hover:bg-gray-600 transition-transform hover:scale-110
                ${suit.color}
              `}
            >
              {suit.symbol}
            </button>
          ))}
        </div>
      )}
      {!selectedRank && (
        <div className="text-center text-gray-500 text-sm italic h-12 flex items-center justify-center">
          Pick a rank first...
        </div>
      )}
    </div>
  );
};

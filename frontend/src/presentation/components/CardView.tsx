import React from 'react';
import { Card } from '../../domain/types';

interface Props {
  card: Card;
  onClick?: () => void;
}

const suitColors: Record<string, string> = {
  'h': 'text-red-500',
  'd': 'text-blue-500', // Diamonds usually red, but let's use blue for contrast or stick to red? Poker usually uses 4 colors for online: H=Red, D=Blue, C=Green, S=Black. Or standard Red/Black.
  // Let's use standard 4-color deck for better visibility on dark mode.
  // Hearts: Red, Diamonds: Blue, Clubs: Green, Spades: Gray/White
  'c': 'text-green-500',
  's': 'text-gray-300',
};

const suitSymbols: Record<string, string> = {
  'h': '♥',
  'd': '♦',
  'c': '♣',
  's': '♠',
};

export const CardView: React.FC<Props> = ({ card, onClick }) => {
  const colorClass = suitColors[card.suit] || 'text-white';
  
  return (
    <div 
      onClick={onClick}
      className={`
        w-16 h-24 bg-gray-800 border-2 border-gray-600 rounded-lg 
        flex flex-col items-center justify-center cursor-pointer 
        hover:bg-gray-700 transition-colors select-none
        ${colorClass}
      `}
    >
      <span className="text-xl font-bold">{card.rank}</span>
      <span className="text-3xl">{suitSymbols[card.suit]}</span>
    </div>
  );
};

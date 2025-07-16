import { Card } from '../types/Card';

interface PokerCardProps {
  card: Card;
  index: number;
  onCardClick: (index: number) => void;
  getSuitColor: (suit: Card['suit']) => string;
  getCardDisplay: (card: Card) => string;
}

export const PokerCard = ({ 
  card, 
  index, 
  onCardClick, 
  getSuitColor, 
  getCardDisplay 
}: PokerCardProps) => {
  return (
    <div 
      className={`poker-card ${card.suit && card.rank ? 'filled' : 'empty'}`}
      onClick={() => onCardClick(index)}
      style={{ color: getSuitColor(card.suit) }}
    >
      {card.suit && card.rank ? (
        <span className="card-display">{getCardDisplay(card)}</span>
      ) : (
        <span className="card-placeholder">+</span>
      )}
    </div>
  );
};

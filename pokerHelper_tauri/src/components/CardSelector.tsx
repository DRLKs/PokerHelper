import { Card, Suit } from '../types/Card';
import { useLanguage } from '../contexts/LanguageContext';

interface CardSelectorProps {
  card: Card;
  index: number;
  suits: Suit[];
  ranks: string[];
  onUpdateCard: (index: number, suit: Card['suit'], rank: Card['rank']) => void;
  onClose: () => void;
  getSuitColor: (suit: Card['suit']) => string;
}

export const CardSelector = ({
  card,
  index,
  suits,
  ranks,
  onUpdateCard,
  onClose,
  getSuitColor
}: CardSelectorProps) => {
  const { t } = useLanguage();

  return (
    <div className="dropdown-overlay" onClick={onClose}>
      <div className="dropdown-menu" onClick={(e) => e.stopPropagation()}>
        <div className="dropdown-header">{t.selectCard}</div>
        <div className="suits-grid">
          {suits.map(suit => (
            <div key={suit.value} className="suit-section">
              <div className="suit-header">
                <span className="suit-icon">{suit.label}</span>
                <span className="suit-name">{suit.name}</span>
              </div>
              <div className="ranks-grid">
                {ranks.map(rank => (
                  <button
                    key={`${suit.value}-${rank}`}
                    className="rank-button"
                    onClick={() => onUpdateCard(index, suit.value as Card['suit'], rank as Card['rank'])}
                    style={{ color: getSuitColor(suit.value as Card['suit']) }}
                  >
                    {rank}{suit.label}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
        {card.suit && card.rank && (
          <button 
            className="clear-button"
            onClick={() => onUpdateCard(index, '', '')}
          >
            {t.clear}
          </button>
        )}
      </div>
    </div>
  );
};

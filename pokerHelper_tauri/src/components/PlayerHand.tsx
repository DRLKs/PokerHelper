import { PokerCard } from './PokerCard';
import { CardSelector } from './CardSelector';
import { usePlayerHand } from '../hooks/usePlayerHand';
import { useLanguage } from '../contexts/LanguageContext';

export const PlayerHand = () => {
  const { t } = useLanguage();
  const {
    cards,
    openDropdown,
    updateCard,
    toggleDropdown,
    getCardDisplay,
    getSuitColor,
    closeDropdown,
    suits,
    ranks
  } = usePlayerHand();

  return (
    <div className="player-hand">
      <h2 className="hand-title">{t.playerHandTitle}</h2>
      <div className="player-cards-container">
        {cards.map((card, index) => (
          <div key={index} className="card-wrapper">
            <PokerCard
              card={card}
              index={index}
              onCardClick={toggleDropdown}
              getSuitColor={getSuitColor}
              getCardDisplay={getCardDisplay}
            />
            
            {openDropdown === index && (
              <CardSelector
                card={card}
                index={index}
                suits={suits}
                ranks={ranks}
                onUpdateCard={updateCard}
                onClose={closeDropdown}
                getSuitColor={getSuitColor}
              />
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

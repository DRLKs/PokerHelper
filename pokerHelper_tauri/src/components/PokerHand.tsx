import { PokerCard } from './PokerCard';
import { CardSelector } from './CardSelector';
import { usePokerHand } from '../hooks/usePokerHand';
import { useLanguage } from '../contexts/LanguageContext';

export const PokerHand = () => {
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
  } = usePokerHand();

  return (
    <div className="poker-hand">
      <h2 className="hand-title">{t.communityCardsTitle}</h2>
      <div className="cards-container">
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

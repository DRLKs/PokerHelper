import { useState } from 'react';
import { Card } from '../types/Card';
import { useLanguage } from '../contexts/LanguageContext';
import { usePokerGame } from '../contexts/PokerGameContext';

export const usePlayerHand = () => {
  const { t } = useLanguage();
  const { playerHand, updatePlayerCard } = usePokerGame();
  
  const [openDropdown, setOpenDropdown] = useState<number | null>(null);

  const suits = [
    { value: 'hearts' as const, label: '♥️', name: t.suits.hearts },
    { value: 'diamonds' as const, label: '♦️', name: t.suits.diamonds },
    { value: 'clubs' as const, label: '♣️', name: t.suits.clubs },
    { value: 'spades' as const, label: '♠️', name: t.suits.spades }
  ];

  const updateCard = (index: number, suit: Card['suit'], rank: Card['rank']) => {
    updatePlayerCard(index, suit, rank);
    setOpenDropdown(null);
  };

  const toggleDropdown = (index: number) => {
    setOpenDropdown(openDropdown === index ? null : index);
  };

  const getCardDisplay = (card: Card) => {
    if (!card.suit || !card.rank) return '';
    const suit = suits.find(s => s.value === card.suit);
    return `${card.rank}${suit?.label}`;
  };

  const getSuitColor = (suit: Card['suit']) => {
    return suit === 'hearts' || suit === 'diamonds' ? '#dc2626' : '#1f2937';
  };

  const closeDropdown = () => setOpenDropdown(null);

  return {
    cards: playerHand,
    openDropdown,
    updateCard,
    toggleDropdown,
    getCardDisplay,
    getSuitColor,
    closeDropdown,
    suits,
    ranks: ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']
  };
};

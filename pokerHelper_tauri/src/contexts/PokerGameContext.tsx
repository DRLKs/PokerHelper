import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Card } from '../types/Card';
import { usePokerAnalysis, PokerAnalysisOptions } from '../hooks/usePokerAnalysis';
import { PokerHandAnalysis } from '../services/pokerAPI';

interface PokerGameContextType {
  // Estado de las cartas
  playerHand: Card[];
  communityCards: Card[];
  
  // Funciones para actualizar cartas
  updatePlayerCard: (index: number, suit: Card['suit'], rank: Card['rank']) => void;
  updateCommunityCard: (index: number, suit: Card['suit'], rank: Card['rank']) => void;
  updateAllCards: (playerHand: Card[], communityCards: Card[]) => void;
  
  // Estado del análisis
  analysis: PokerHandAnalysis | null;
  isLoading: boolean;
  error: string | null;
  
  // Opciones de análisis
  analysisOptions: PokerAnalysisOptions;
  updateAnalysisOptions: (options: Partial<PokerAnalysisOptions>) => void;
  
  // Función para limpiar todo
  clearAll: () => void;
}

const PokerGameContext = createContext<PokerGameContextType | undefined>(undefined);

interface PokerGameProviderProps {
  children: ReactNode;
}

export const PokerGameProvider: React.FC<PokerGameProviderProps> = ({ children }) => {
  // Estado de las cartas del jugador (2 cartas)
  const [playerHand, setPlayerHand] = useState<Card[]>([
    { suit: '', rank: '' },
    { suit: '', rank: '' }
  ]);

  // Estado de las cartas comunitarias (5 cartas)
  const [communityCards, setCommunityCards] = useState<Card[]>([
    { suit: '', rank: '' },
    { suit: '', rank: '' },
    { suit: '', rank: '' },
    { suit: '', rank: '' },
    { suit: '', rank: '' }
  ]);

  // Opciones de análisis
  const [analysisOptions, setAnalysisOptions] = useState<PokerAnalysisOptions>({
    numberOfOpponents: 3,
    smallBlind: 5,
    accumulatedBet: 0
  });

  // Hook para el análisis
  const { analysis, isLoading, error, analyzeCards, clearAnalysis } = usePokerAnalysis();

  // Función para actualizar carta del jugador
  const updatePlayerCard = (index: number, suit: Card['suit'], rank: Card['rank']) => {
    // Primero remover duplicados si la carta no está vacía
    if (suit && rank) {
      removeDuplicateCard(suit, rank, index, undefined);
    }
    
    setPlayerHand(prev => {
      const newHand = [...prev];
      newHand[index] = { suit, rank };
      return newHand;
    });
  };

  // Función para actualizar carta comunitaria
  const updateCommunityCard = (index: number, suit: Card['suit'], rank: Card['rank']) => {
    // Primero remover duplicados si la carta no está vacía
    if (suit && rank) {
      removeDuplicateCard(suit, rank, undefined, index);
    }
    
    setCommunityCards(prev => {
      const newCards = [...prev];
      newCards[index] = { suit, rank };
      return newCards;
    });
  };

  // Función para actualizar todas las cartas de una vez
  // Esta función será utilizada para sincronizar el estados de la cartas con el futuro servicio Python
  const updateAllCards = (newPlayerHand: Card[], newCommunityCards: Card[]) => {
    // Validar que playerHand tenga exactamente 2 cartas
    if (newPlayerHand.length !== 2) {
      console.warn('Player hand must have exactly 2 cards');
      return;
    }

    // Validar que communityCards tenga exactamente 5 cartas
    if (newCommunityCards.length !== 5) {
      console.warn('Community cards must have exactly 5 cards');
      return;
    }

    // Verificar duplicados entre todas las cartas
    const allCards = [...newPlayerHand, ...newCommunityCards];
    const validCards = allCards.filter(card => card.suit && card.rank);
    const duplicates = validCards.filter((card, index) => 
      validCards.findIndex(c => c.suit === card.suit && c.rank === card.rank) !== index
    );

    if (duplicates.length > 0) {
      console.warn('Duplicate cards detected:', duplicates);
      return;
    }

    // Actualizar ambos estados
    setPlayerHand([...newPlayerHand]);
    setCommunityCards([...newCommunityCards]);
  };

  // Función para actualizar opciones de análisis
  const updateAnalysisOptions = (newOptions: Partial<PokerAnalysisOptions>) => {
    setAnalysisOptions(prev => ({ ...prev, ...newOptions }));
  };

  // Función para limpiar todo
  const clearAll = () => {
    setPlayerHand([
      { suit: '', rank: '' },
      { suit: '', rank: '' }
    ]);
    setCommunityCards([
      { suit: '', rank: '' },
      { suit: '', rank: '' },
      { suit: '', rank: '' },
      { suit: '', rank: '' },
      { suit: '', rank: '' }
    ]);
    clearAnalysis();
  };

  // Función para verificar y eliminar cartas duplicadas
  const removeDuplicateCard = (newSuit: string, newRank: string, excludePlayerIndex?: number, excludeCommunityIndex?: number) => {
    // Solo verificar si la carta no está vacía
    if (!newSuit || !newRank) return;

    // Verificar duplicados en playerHand
    setPlayerHand(prev => prev.map((card, index) => {
      // Si la carta es igual pero en un índice diferente, eliminar
      if (card.suit === newSuit && card.rank === newRank && index !== excludePlayerIndex) {
        return { suit: '', rank: '' };
      }
      return card;
    }));

    // Verificar duplicados en communityCards
    setCommunityCards(prev => prev.map((card, index) => {
      // Si la carta es igual pero en un índice diferente, eliminar
      if (card.suit === newSuit && card.rank === newRank && index !== excludeCommunityIndex) {
        return { suit: '', rank: '' };
      }
      return card;
    }));
  };

  // Efecto para hacer petición cuando cambien las cartas o las opciones
  useEffect(() => {
    // Debounce para evitar demasiadas peticiones
    const timeoutId = setTimeout(() => {
      analyzeCards(playerHand, communityCards, analysisOptions);
    }, 300); // Esperar 300ms después del último cambio

    return () => clearTimeout(timeoutId);
  }, [playerHand, communityCards, analysisOptions, analyzeCards]);

  const value: PokerGameContextType = {
    playerHand,
    communityCards,
    updatePlayerCard,
    updateCommunityCard,
    updateAllCards,
    analysis,
    isLoading,
    error,
    analysisOptions,
    updateAnalysisOptions,
    clearAll
  };

  return (
    <PokerGameContext.Provider value={value}>
      {children}
    </PokerGameContext.Provider>
  );
};

export const usePokerGame = (): PokerGameContextType => {
  const context = useContext(PokerGameContext);
  if (!context) {
    throw new Error('usePokerGame must be used within a PokerGameProvider');
  }
  return context;
};

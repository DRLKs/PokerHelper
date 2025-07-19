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
    setPlayerHand(prev => {
      const newHand = [...prev];
      newHand[index] = { suit, rank };
      return newHand;
    });
  };

  // Función para actualizar carta comunitaria
  const updateCommunityCard = (index: number, suit: Card['suit'], rank: Card['rank']) => {
    setCommunityCards(prev => {
      const newCards = [...prev];
      newCards[index] = { suit, rank };
      return newCards;
    });
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

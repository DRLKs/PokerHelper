import { useState, useEffect, useCallback } from 'react';
import { Card } from '../types/Card';
import { PokerHandAnalysis, pokerAPIService } from '../services/pokerAPI';

export interface PokerAnalysisOptions {
  numberOfOpponents?: number;
  smallBlind?: number;
  accumulatedBet?: number;
}

interface UsePokerAnalysisReturn {
  analysis: PokerHandAnalysis | null;
  isLoading: boolean;
  error: string | null;
  analyzeCards: (playerHand: Card[], communityCards: Card[], options?: PokerAnalysisOptions) => Promise<void>;
  clearAnalysis: () => void;
}

export const usePokerAnalysis = (): UsePokerAnalysisReturn => {
  const [analysis, setAnalysis] = useState<PokerHandAnalysis | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const analyzeCards = useCallback(async (
    playerHand: Card[], 
    communityCards: Card[], 
    options: PokerAnalysisOptions = {}
  ) => {
    // Valores por defecto para el análisis
    const {
      numberOfOpponents = 3,
      smallBlind = 5,
      accumulatedBet = 0
    } = options;

    // Filtrar cartas válidas
    const validPlayerHand = playerHand.filter(card => card.suit && card.rank);
    const validCommunityCards = communityCards.filter(card => card.suit && card.rank);

    console.log('usePokerAnalysis.analyzeCards called:', {
      validPlayerHandCount: validPlayerHand.length,
      validCommunityCardsCount: validCommunityCards.length,
      options: { numberOfOpponents, smallBlind, accumulatedBet }
    });

    // Si no hay cartas del jugador, limpiar análisis
    if (validPlayerHand.length === 0) {
      setAnalysis(null);
      setError(null);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await pokerAPIService.analyzeHand(
        validPlayerHand, 
        validCommunityCards,
        numberOfOpponents,
        smallBlind,
        accumulatedBet
      );
      
      // Si la petición fue cancelada o es obsoleta, result será null
      if (result !== null) {
        setAnalysis(result);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Error desconocido';
      console.error('Error in usePokerAnalysis:', err);
      setError(errorMessage);
      setAnalysis(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const clearAnalysis = useCallback(() => {
    pokerAPIService.cancelAllRequests();
    setAnalysis(null);
    setError(null);
    setIsLoading(false);
  }, []);

  // Cleanup al desmontar el componente
  useEffect(() => {
    return () => {
      pokerAPIService.cancelAllRequests();
    };
  }, []);

  return {
    analysis,
    isLoading,
    error,
    analyzeCards,
    clearAnalysis
  };
};

import { useState, useCallback } from 'react';
import { Card } from '../domain/types';
import { calculateEquity } from '../infrastructure/pokerApi';

export const usePoker = () => {
  const [myCards, setMyCards] = useState<Card[]>([]);
  const [communityCards, setCommunityCards] = useState<Card[]>([]);
  const [numOpponents, setNumOpponents] = useState<number>(1);
  const [potSize, setPotSize] = useState<number>(0);
  const [callAmount, setCallAmount] = useState<number>(0);
  const [equity, setEquity] = useState<number | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const calculate = async () => {
    if (myCards.length !== 2) {
        setError("You must have exactly 2 cards in hand.");
        return;
    }
    setLoading(true);
    setError(null);
    try {
      const result = await calculateEquity(myCards, communityCards, numOpponents);
      setEquity(result);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  const addMyCard = useCallback((card: Card) => {
    setMyCards((prev) => (prev.length < 2 ? [...prev, card] : prev));
    setEquity(null);
  }, []);

  const removeMyCard = useCallback((index: number) => {
    setMyCards((prev) => prev.filter((_, i) => i !== index));
    setEquity(null);
  }, []);

  const addCommunityCard = useCallback((card: Card) => {
    setCommunityCards((prev) => (prev.length < 5 ? [...prev, card] : prev));
    setEquity(null);
  }, []);

  const removeCommunityCard = useCallback((index: number) => {
    setCommunityCards((prev) => prev.filter((_, i) => i !== index));
    setEquity(null);
  }, []);

  const updateNumOpponents = useCallback((value: number) => {
    setNumOpponents(value);
    setEquity(null);
  }, []);

  return {
    myCards,
    addMyCard,
    removeMyCard,
    communityCards,
    addCommunityCard,
    removeCommunityCard,
    numOpponents,
    setNumOpponents: updateNumOpponents,
    potSize,
    setPotSize,
    callAmount,
    setCallAmount,
    equity,
    loading,
    error,
    calculate
  };
};

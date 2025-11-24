import { useState, useEffect } from 'react';
import { Card } from '../domain/types';
import { calculateEquity, startSidecar, listWindows } from '../infrastructure/pokerApi';

export const usePoker = () => {
  const [myCards, setMyCards] = useState<Card[]>([]);
  const [communityCards, setCommunityCards] = useState<Card[]>([]);
  const [numOpponents, setNumOpponents] = useState<number>(1);
  const [equity, setEquity] = useState<number | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  
  // Vision state
  const [visionEnabled, setVisionEnabled] = useState(false);
  const [availableWindows, setAvailableWindows] = useState<string[]>([]);
  const [selectedWindow, setSelectedWindow] = useState<string>("");

  useEffect(() => {
    // Initialize sidecar on mount
    startSidecar().catch(console.error);
  }, []);

  const toggleVision = async () => {
    if (!visionEnabled) {
        try {
            const windows = await listWindows();
            setAvailableWindows(windows);
            setVisionEnabled(true);
        } catch (e: any) {
            setError("Failed to list windows: " + e.toString());
        }
    } else {
        setVisionEnabled(false);
    }
  };

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

  const addMyCard = (card: Card) => {
    if (myCards.length < 2) {
        setMyCards([...myCards, card]);
    }
  };

  const removeMyCard = (index: number) => {
    const newCards = [...myCards];
    newCards.splice(index, 1);
    setMyCards(newCards);
  };

  const addCommunityCard = (card: Card) => {
    if (communityCards.length < 5) {
        setCommunityCards([...communityCards, card]);
    }
  };

  const removeCommunityCard = (index: number) => {
    const newCards = [...communityCards];
    newCards.splice(index, 1);
    setCommunityCards(newCards);
  };

  return {
    myCards,
    addMyCard,
    removeMyCard,
    communityCards,
    addCommunityCard,
    removeCommunityCard,
    numOpponents,
    setNumOpponents,
    equity,
    loading,
    error,
    calculate,
    visionEnabled,
    toggleVision,
    availableWindows,
    selectedWindow,
    setSelectedWindow
  };
};

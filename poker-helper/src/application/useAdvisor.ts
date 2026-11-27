import { useCallback, useMemo, useState } from 'react';
import { Blinds, Chips, SeatIndex, TableUpdate } from '../domain/advisor';
import {
  collectBets,
  HOLE_CARD_COUNT,
  initialDraft,
  MAX_BOARD_CARDS,
  SeatDraft,
  startNewHand,
  TableDraft,
  toSnapshot,
} from '../domain/tableDraft';
import { Card } from '../domain/types';
import { observeTable } from '../infrastructure/advisorApi';

const replaceAt = <T,>(items: T[], index: number, replacement: T): T[] =>
  items.map((item, itemIndex) => (itemIndex === index ? replacement : item));

export const useAdvisor = () => {
  const [draft, setDraft] = useState<TableDraft>(initialDraft);
  const [update, setUpdate] = useState<TableUpdate | null>(null);
  const [adviceDraft, setAdviceDraft] = useState<TableDraft | null>(null);
  const [requesting, setRequesting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const snapshotResult = useMemo(() => toSnapshot(draft), [draft]);
  const adviceMatchesTable = adviceDraft === draft;

  const updateSeat = useCallback((index: SeatIndex, change: Partial<SeatDraft>) => {
    setDraft((current) => ({
      ...current,
      seats: replaceAt(current.seats, index, { ...current.seats[index], ...change }),
    }));
  }, []);

  const setButtonSeat = useCallback((buttonSeat: SeatIndex) => {
    setDraft((current) => ({ ...current, buttonSeat }));
  }, []);

  const setHeroSeat = useCallback((heroSeat: SeatIndex) => {
    setDraft((current) => ({ ...current, heroSeat }));
  }, []);

  const updateBlinds = useCallback((change: Partial<Blinds>) => {
    setDraft((current) => ({ ...current, blinds: { ...current.blinds, ...change } }));
  }, []);

  const setPot = useCallback((pot: Chips) => {
    setDraft((current) => ({ ...current, pot }));
  }, []);

  const addHeroCard = useCallback((card: Card) => {
    setDraft((current) =>
      current.heroCards.length < HOLE_CARD_COUNT
        ? { ...current, heroCards: [...current.heroCards, card] }
        : current,
    );
  }, []);

  const removeHeroCard = useCallback((index: number) => {
    setDraft((current) => ({
      ...current,
      heroCards: current.heroCards.filter((_, cardIndex) => cardIndex !== index),
    }));
  }, []);

  const addBoardCard = useCallback((card: Card) => {
    setDraft((current) =>
      current.board.length < MAX_BOARD_CARDS ? { ...current, board: [...current.board, card] } : current,
    );
  }, []);

  const removeBoardCard = useCallback((index: number) => {
    setDraft((current) => ({
      ...current,
      board: current.board.filter((_, cardIndex) => cardIndex !== index),
    }));
  }, []);

  const newHand = useCallback(() => setDraft(startNewHand), []);
  const nextStreet = useCallback(() => setDraft(collectBets), []);

  const requestAdvice = useCallback(async () => {
    if (snapshotResult.kind !== 'ready') return;
    setRequesting(true);
    setError(null);
    try {
      setUpdate(await observeTable(snapshotResult.snapshot));
      setAdviceDraft(draft);
    } catch (reason: unknown) {
      setError(String(reason));
    } finally {
      setRequesting(false);
    }
  }, [draft, snapshotResult]);

  return {
    draft,
    updateSeat,
    setButtonSeat,
    setHeroSeat,
    updateBlinds,
    setPot,
    addHeroCard,
    removeHeroCard,
    addBoardCard,
    removeBoardCard,
    newHand,
    nextStreet,
    snapshotResult,
    requestAdvice,
    requesting,
    update,
    adviceMatchesTable,
    error,
  };
};

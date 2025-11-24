import { invoke } from '@tauri-apps/api/tauri';
import { Card } from '../domain/types';

export const calculateEquity = async (
  myCards: Card[],
  communityCards: Card[],
  numOpponents: number
): Promise<number> => {
  const myCardsStr = myCards.map(c => `${c.rank}${c.suit}`);
  const communityCardsStr = communityCards.map(c => `${c.rank}${c.suit}`);

  return await invoke('calculate_equity_command', {
    myCards: myCardsStr,
    communityCards: communityCardsStr,
    numOpponents,
  });
};

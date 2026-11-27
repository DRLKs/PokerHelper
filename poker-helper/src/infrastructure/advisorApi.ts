import { invoke } from '@tauri-apps/api/tauri';
import { Population, TableSnapshot, TableUpdate } from '../domain/advisor';

export const observeTable = (snapshot: TableSnapshot): Promise<TableUpdate> =>
  invoke('observe_table', { snapshot });

export const fetchOpponentPopulation = (): Promise<Population> => invoke('opponent_population');

export const saveOpponentPopulation = (population: Population): Promise<void> =>
  invoke('set_opponent_population', { population });

export const resetOpponents = (): Promise<void> => invoke('reset_opponents');

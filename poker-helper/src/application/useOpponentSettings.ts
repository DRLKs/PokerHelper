import { useCallback, useEffect, useState } from 'react';
import { Population } from '../domain/advisor';
import { fetchOpponentPopulation, resetOpponents, saveOpponentPopulation } from '../infrastructure/advisorApi';

export const useOpponentSettings = () => {
  const [population, setPopulation] = useState<Population | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchOpponentPopulation()
      .then(setPopulation)
      .catch((reason: unknown) => setError(String(reason)));
  }, []);

  const changePopulation = useCallback(async (next: Population) => {
    setError(null);
    try {
      await saveOpponentPopulation(next);
      setPopulation(next);
    } catch (reason: unknown) {
      setError(String(reason));
    }
  }, []);

  const forgetOpponents = useCallback(async () => {
    setError(null);
    try {
      await resetOpponents();
    } catch (reason: unknown) {
      setError(String(reason));
    }
  }, []);

  return { population, changePopulation, forgetOpponents, error };
};

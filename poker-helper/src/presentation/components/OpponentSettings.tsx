import React, { useState } from 'react';
import { Population } from '../../domain/advisor';

interface Props {
  population: Population | null;
  onChangePopulation: (population: Population) => void;
  onForgetOpponents: () => void;
  error: string | null;
}

const populations: { value: Population; label: string; hint: string }[] = [
  { value: 'play_money', label: 'Play money', hint: 'Loose, passive players who rarely fold' },
  { value: 'regular', label: 'Regular', hint: 'Tighter players who fold to bets more often' },
];

export const OpponentSettings = React.memo<Props>(({ population, onChangePopulation, onForgetOpponents, error }) => {
  const [confirmingReset, setConfirmingReset] = useState(false);

  return (
    <section className="panel p-5" aria-labelledby="opponents-title">
      <p className="eyebrow">Opponents</p>
      <h2 id="opponents-title" className="mt-1 text-base font-semibold text-white">Unknown players are like</h2>
      <div className="mt-3 grid grid-cols-2 gap-2" role="radiogroup" aria-labelledby="opponents-title">
        {populations.map((option) => (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={population === option.value}
            disabled={population === null}
            onClick={() => onChangePopulation(option.value)}
            className={`population-option ${population === option.value ? 'population-option-on' : ''}`}
          >
            <span className="block text-sm font-semibold">{option.label}</span>
            <span className="mt-0.5 block text-[11px] text-slate-500">{option.hint}</span>
          </button>
        ))}
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-2">
        {!confirmingReset && (
          <button type="button" className="secondary-button" onClick={() => setConfirmingReset(true)}>
            Reset opponent stats
          </button>
        )}
        {confirmingReset && (
          <>
            <span className="text-xs text-slate-400">Forget every player's stats?</span>
            <button
              type="button"
              className="secondary-button secondary-button-danger"
              onClick={() => {
                onForgetOpponents();
                setConfirmingReset(false);
              }}
            >
              Reset
            </button>
            <button type="button" className="secondary-button" onClick={() => setConfirmingReset(false)}>
              Cancel
            </button>
          </>
        )}
      </div>
      {error && <div className="error-message mt-3" role="alert">{error}</div>}
    </section>
  );
});

import React from 'react';

interface Props {
  id: string;
  label: string;
  hint: string;
  value: number;
  onChange: (value: number) => void;
}

const normalize = (value: number) => Math.max(0, Math.round(value * 100) / 100);

export const AmountInput: React.FC<Props> = ({ id, label, hint, value, onChange }) => {
  const step = value >= 100 ? 10 : value >= 10 ? 5 : 1;

  return (
    <div className="amount-field">
      <label htmlFor={id} className="block">
        <span className="text-sm font-semibold text-slate-200">{label}</span>
        <span className="mt-0.5 block text-xs text-slate-500">{hint}</span>
      </label>
      <div className="mt-3 flex items-center gap-2">
        <button
          type="button"
          className="stepper-button"
          onClick={() => onChange(normalize(value - step))}
          aria-label={`Decrease ${label}`}
        >
          −
        </button>
        <div className="amount-input-wrap">
          <input
            id={id}
            type="number"
            min="0"
            step="0.01"
            inputMode="decimal"
            value={value}
            onChange={(event) => onChange(normalize(Number(event.target.value) || 0))}
            className="amount-input"
          />
          <span className="text-[10px] font-semibold uppercase tracking-wider text-slate-600">chips</span>
        </div>
        <button
          type="button"
          className="stepper-button"
          onClick={() => onChange(normalize(value + step))}
          aria-label={`Increase ${label}`}
        >
          +
        </button>
      </div>
    </div>
  );
};

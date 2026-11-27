import React from 'react';
import { Chips } from '../../domain/advisor';

interface Props {
  id: string;
  label: string;
  value: Chips;
  onChange: (value: Chips) => void;
  disabled?: boolean;
}

const toWholeChips = (text: string): Chips => Math.max(0, Math.floor(Number(text) || 0));

export const ChipInput = React.memo<Props>(({ id, label, value, onChange, disabled = false }) => (
  <label htmlFor={id} className="chip-field">
    <span className="chip-field-label">{label}</span>
    <input
      id={id}
      type="number"
      min="0"
      step="1"
      inputMode="numeric"
      value={value}
      disabled={disabled}
      onChange={(event) => onChange(toWholeChips(event.target.value))}
      className="chip-input"
    />
  </label>
));

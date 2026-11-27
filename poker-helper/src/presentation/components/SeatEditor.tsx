import React from 'react';
import { SeatIndex, SeatRead, SeatStatus } from '../../domain/advisor';
import { SeatDraft } from '../../domain/tableDraft';
import { percent, playerTypeLabels, seatLabel, seatStatusLabels } from '../format';
import { ChipInput } from './ChipInput';

interface Props {
  index: SeatIndex;
  seat: SeatDraft;
  isButton: boolean;
  isHero: boolean;
  read: SeatRead | undefined;
  onChange: (index: SeatIndex, change: Partial<SeatDraft>) => void;
  onMakeButton: (index: SeatIndex) => void;
  onMakeHero: (index: SeatIndex) => void;
}

const statuses: SeatStatus[] = ['active', 'folded', 'all_in', 'sitting_out', 'empty'];

const ProfileLine = ({ read }: { read: SeatRead }) => {
  const { profile } = read;
  const hands = profile.hands === 1 ? '1 hand' : `${profile.hands} hands`;
  return (
    <p className="seat-profile" title={`VPIP ${percent(profile.voluntarily_played)}, PFR ${percent(profile.raised_preflop)}, 3-bet ${percent(profile.three_bet)}, fold to c-bet ${percent(profile.folded_to_continuation_bet)}`}>
      <span className="seat-profile-type">{playerTypeLabels[profile.player_type]}</span>
      <span>{hands} · VPIP {percent(profile.voluntarily_played)} · PFR {percent(profile.raised_preflop)}</span>
    </p>
  );
};

export const SeatEditor = React.memo<Props>(({
  index,
  seat,
  isButton,
  isHero,
  read,
  onChange,
  onMakeButton,
  onMakeHero,
}) => {
  const isEmpty = seat.status === 'empty';
  const idPrefix = `seat-${index}`;

  return (
    <div className={`seat-card ${isHero ? 'seat-card-hero' : ''} ${isEmpty ? 'seat-card-empty' : ''}`}>
      <div className="flex items-center justify-between gap-2">
        <span className="text-sm font-semibold text-slate-200">{seatLabel(index)}</span>
        <div className="flex gap-1.5">
          <button
            type="button"
            className={`seat-toggle ${isButton ? 'seat-toggle-on' : ''}`}
            aria-pressed={isButton}
            onClick={() => onMakeButton(index)}
          >
            Dealer
          </button>
          <button
            type="button"
            className={`seat-toggle ${isHero ? 'seat-toggle-on' : ''}`}
            aria-pressed={isHero}
            onClick={() => onMakeHero(index)}
          >
            You
          </button>
        </div>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-2">
        <input
          type="text"
          value={seat.name}
          placeholder="Name"
          aria-label={`${seatLabel(index)} player name`}
          onChange={(event) => onChange(index, { name: event.target.value })}
          className="seat-text-input"
        />
        <select
          value={seat.status}
          aria-label={`${seatLabel(index)} status`}
          onChange={(event) => onChange(index, { status: event.target.value as SeatStatus })}
          className="seat-select"
        >
          {statuses.map((status) => (
            <option key={status} value={status}>{seatStatusLabels[status]}</option>
          ))}
        </select>
        <ChipInput
          id={`${idPrefix}-stack`}
          label="Stack"
          value={seat.stack}
          disabled={isEmpty}
          onChange={(stack) => onChange(index, { stack })}
        />
        <ChipInput
          id={`${idPrefix}-bet`}
          label="Bet"
          value={seat.committed}
          disabled={isEmpty}
          onChange={(committed) => onChange(index, { committed })}
        />
      </div>

      {read && <ProfileLine read={read} />}
    </div>
  );
});

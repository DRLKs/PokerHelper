import React from 'react';
import { RecordedAction, Street } from '../../domain/advisor';
import { describeAction, seatLabel } from '../format';

interface Props {
  history: RecordedAction[];
}

const streets: Street[] = ['Preflop', 'Flop', 'Turn', 'River'];

export const HandHistory = React.memo<Props>(({ history }) => {
  const playedStreets = streets.filter((street) => history.some((recorded) => recorded.street === street));

  return (
    <section className="panel p-5" aria-labelledby="history-title">
      <p className="eyebrow">This hand</p>
      <h2 id="history-title" className="mt-1 text-base font-semibold text-white">Actions so far</h2>
      {playedStreets.length === 0 && <p className="mt-3 text-sm text-slate-500">No actions recorded yet.</p>}
      {playedStreets.map((street) => (
        <div key={street} className="mt-3">
          <p className="text-xs font-semibold text-slate-400">{street}</p>
          <ol className="mt-1 space-y-0.5 text-sm text-slate-300">
            {history
              .filter((recorded) => recorded.street === street)
              .map((recorded, index) => (
                <li key={index}>{seatLabel(recorded.seat)} {describeAction(recorded.action)}</li>
              ))}
          </ol>
        </div>
      ))}
    </section>
  );
});

import React from 'react';

interface Props {
  equity: number;
  requiredEquity: number;
}

const clampPercentage = (value: number) => Math.min(100, Math.max(0, value));

export const EquityChart = React.memo<Props>(({ equity, requiredEquity }) => {
  const win = clampPercentage(equity);
  const notWin = 100 - win;
  const threshold = clampPercentage(requiredEquity);
  const edge = equity - requiredEquity;
  const isProfitable = edge >= 0;

  return (
    <div className="equity-visualization">
      <div
        className="equity-donut"
        role="img"
        aria-label={`${win.toFixed(1)} percent estimated chance to win and ${notWin.toFixed(1)} percent chance not to win`}
      >
        <svg viewBox="0 0 120 120" aria-hidden="true">
          <circle className="equity-donut-track" cx="60" cy="60" r="48" pathLength="100" />
          <circle
            className="equity-donut-value"
            cx="60"
            cy="60"
            r="48"
            pathLength="100"
            strokeDasharray={`${win} ${100 - win}`}
          />
        </svg>
        <div className="equity-donut-label">
          <strong>{win.toFixed(1)}%</strong>
          <span>Win</span>
        </div>
      </div>

      <div className="equity-chart-details">
        <div className="probability-legend" aria-label="Probability breakdown">
          <div className="probability-legend-item">
            <span className="legend-dot legend-dot-win" aria-hidden="true" />
            <span>Win</span>
            <strong>{win.toFixed(1)}%</strong>
          </div>
          <div className="probability-legend-item">
            <span className="legend-dot legend-dot-rest" aria-hidden="true" />
            <span>Not win</span>
            <strong>{notWin.toFixed(1)}%</strong>
          </div>
        </div>

        <div className="equity-comparison">
          <div className="mb-2 flex items-center justify-between gap-3 text-[11px]">
            <span className="text-slate-500">Equity vs. required</span>
            <strong className={isProfitable ? 'text-emerald-400' : 'text-rose-400'}>
              {edge >= 0 ? '+' : ''}{edge.toFixed(1)} pts
            </strong>
          </div>
          <div
            className="equity-scale"
            role="img"
            aria-label={`Your equity is ${win.toFixed(1)} percent. Required equity is ${threshold.toFixed(1)} percent.`}
          >
            <span className="equity-scale-fill" style={{ width: `${win}%` }} />
            {requiredEquity > 0 && requiredEquity < 100 && (
              <span className="equity-scale-threshold" style={{ left: `${threshold}%` }} />
            )}
          </div>
          <div className="mt-2 flex items-center justify-between text-[10px] text-slate-600">
            <span>0%</span>
            <span>{requiredEquity > 0 ? `Required ${threshold.toFixed(1)}%` : 'No call required'}</span>
            <span>100%</span>
          </div>
        </div>
      </div>
    </div>
  );
});

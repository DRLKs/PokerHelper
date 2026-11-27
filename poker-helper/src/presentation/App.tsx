import { useState } from 'react';
import { useVision } from '../application/useVision';
import { AdvisorPage } from './pages/AdvisorPage';
import { EquityPage } from './pages/EquityPage';

type Tab = 'advisor' | 'equity';

const tabs: { value: Tab; label: string }[] = [
  { value: 'advisor', label: 'Advisor' },
  { value: 'equity', label: 'Equity calculator' },
];

const App = () => {
  const { visionEnabled, toggleVision, availableWindows, selectedWindow, setSelectedWindow, error } = useVision();
  const [tab, setTab] = useState<Tab>('advisor');

  return (
    <div className="app-shell min-h-screen text-white">
      <div className="ambient-glow" aria-hidden="true" />
      <main className="relative mx-auto w-full max-w-[1180px] px-4 py-5 sm:px-6 sm:py-8 lg:px-8">
        <header className="mb-6 flex flex-wrap items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="brand-mark" aria-hidden="true">♠</div>
            <div>
              <h1 className="text-base font-semibold tracking-[0.16em] text-white">POKER HELPER</h1>
              <p className="mt-0.5 text-xs text-slate-500">Texas Hold'em decision support</p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {visionEnabled && (
              <select
                value={selectedWindow}
                onChange={(event) => setSelectedWindow(event.target.value)}
                className="control-select max-w-[220px]"
                aria-label="Window to analyze"
              >
                <option value="">Select a window</option>
                {availableWindows.map((windowName) => (
                  <option key={windowName} value={windowName}>{windowName}</option>
                ))}
              </select>
            )}
            <button
              type="button"
              onClick={toggleVision}
              className={`vision-button ${visionEnabled ? 'vision-button-active' : ''}`}
              aria-pressed={visionEnabled}
            >
              <span className="status-dot" />
              {visionEnabled ? 'Vision on' : 'Enable vision'}
            </button>
          </div>
        </header>

        <nav className="tab-bar mb-5" role="tablist" aria-label="Mode">
          {tabs.map((option) => (
            <button
              key={option.value}
              type="button"
              role="tab"
              aria-selected={tab === option.value}
              onClick={() => setTab(option.value)}
              className={`tab-button ${tab === option.value ? 'tab-button-active' : ''}`}
            >
              {option.label}
            </button>
          ))}
        </nav>

        {error && <div className="error-message mb-5" role="alert">{error}</div>}

        <div role="tabpanel" hidden={tab !== 'advisor'}><AdvisorPage /></div>
        <div role="tabpanel" hidden={tab !== 'equity'}><EquityPage /></div>

        <p className="mt-6 text-center text-[11px] leading-relaxed text-slate-600">
          Development build · Results are estimates, not financial advice.
        </p>
      </main>
    </div>
  );
};

export default App;

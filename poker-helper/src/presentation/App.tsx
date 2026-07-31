import React, { useState } from 'react';
import { usePoker } from '../application/usePoker';
import { Card } from '../domain/types';
import { AmountInput } from './components/AmountInput';
import { CardSelector } from './components/CardSelector';
import { CardView } from './components/CardView';

type CardTarget = 'hand' | 'community';

const CardSlot = () => (
  <div className="card-slot" aria-hidden="true">
    <span>+</span>
  </div>
);

const App = () => {
  const {
    myCards, addMyCard, removeMyCard,
    communityCards, addCommunityCard, removeCommunityCard,
    numOpponents, setNumOpponents,
    potSize, setPotSize, callAmount, setCallAmount,
    equity, calculate, loading, error,
    visionEnabled, toggleVision, availableWindows, selectedWindow, setSelectedWindow,
  } = usePoker();
  const [target, setTarget] = useState<CardTarget>('hand');

  const allCards = [...myCards, ...communityCards];
  const potOdds = callAmount > 0 ? (callAmount / (potSize + callAmount)) * 100 : 0;
  const hasRecommendation = equity !== null && callAmount > 0;
  const isProfitableCall = equity !== null && equity >= potOdds;

  const handleCardSelect = (card: Card) => {
    if (target === 'hand' && myCards.length < 2) {
      addMyCard(card);
      if (myCards.length === 1) setTarget('community');
      return;
    }
    if (target === 'community' && communityCards.length < 5) addCommunityCard(card);
  };

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

        <div className="grid gap-5 lg:grid-cols-[minmax(0,1.42fr)_minmax(330px,0.8fr)]">
          <div className="space-y-5">
            <section className="table-surface">
              <div className="table-vignette" aria-hidden="true" />
              <div className="relative z-10">
                <div className="mb-8 flex items-start justify-between gap-4">
                  <div>
                    <p className="eyebrow text-emerald-200/50">Current hand</p>
                    <h2 className="mt-1 text-xl font-semibold">The table</h2>
                  </div>
                  <div className="rounded-full border border-white/10 bg-black/15 px-3 py-1.5 text-xs text-emerald-50/70 backdrop-blur-sm">
                    {numOpponents} {numOpponents === 1 ? 'opponent' : 'opponents'}
                  </div>
                </div>

                <div
                  className={`card-zone community-zone ${target === 'community' ? 'card-zone-active' : ''}`}
                  onClick={() => setTarget('community')}
                >
                  <button type="button" className="zone-label" onClick={() => setTarget('community')}>Community cards</button>
                  <span className="card-row">
                    {communityCards.map((card, index) => (
                      <CardView key={`${card.rank}${card.suit}`} card={card} onClick={() => removeCommunityCard(index)} />
                    ))}
                    {Array.from({ length: 5 - communityCards.length }, (_, index) => <CardSlot key={index} />)}
                  </span>
                </div>

                <div className="table-divider"><span>BOARD</span></div>

                <div
                  className={`card-zone hand-zone ${target === 'hand' ? 'card-zone-active' : ''}`}
                  onClick={() => setTarget('hand')}
                >
                  <button type="button" className="zone-label" onClick={() => setTarget('hand')}>Your hand</button>
                  <span className="card-row">
                    {myCards.map((card, index) => (
                      <CardView key={`${card.rank}${card.suit}`} card={card} onClick={() => removeMyCard(index)} />
                    ))}
                    {Array.from({ length: 2 - myCards.length }, (_, index) => <CardSlot key={index} />)}
                  </span>
                </div>
              </div>
            </section>

            <CardSelector
              onSelect={handleCardSelect}
              unavailableCards={allCards}
              destinationLabel={target === 'hand' ? 'your hand' : 'the board'}
            />
          </div>

          <aside className="space-y-5">
            <section className="panel p-5 sm:p-6" aria-labelledby="game-details-title">
              <p className="eyebrow">Game details</p>
              <h2 id="game-details-title" className="mt-1 text-lg font-semibold">Set the action</h2>

              <div className="mt-6">
                <div className="mb-3 flex items-center justify-between">
                  <div>
                    <label htmlFor="opponents" className="text-sm font-semibold text-slate-200">Opponents</label>
                    <p className="mt-0.5 text-xs text-slate-500">Players still in the hand</p>
                  </div>
                  <output htmlFor="opponents" className="opponent-count">{numOpponents}</output>
                </div>
                <input
                  id="opponents"
                  type="range"
                  min="1"
                  max="9"
                  value={numOpponents}
                  onChange={(event) => setNumOpponents(Number(event.target.value))}
                  className="range-input"
                />
                <div className="mt-1.5 flex justify-between text-[10px] text-slate-600"><span>1</span><span>9</span></div>
              </div>

              <div className="my-6 h-px bg-white/[0.06]" />

              <div className="space-y-5">
                <AmountInput id="pot-size" label="Pot size" hint="Pot before your call" value={potSize} onChange={setPotSize} />
                <AmountInput id="to-call" label="Amount to call" hint="Additional chips required" value={callAmount} onChange={setCallAmount} />
              </div>

              <div className="mt-5 flex items-center justify-between rounded-xl border border-white/[0.06] bg-black/15 px-4 py-3">
                <span className="text-xs text-slate-500">Required equity</span>
                <span className="font-mono text-sm font-semibold text-slate-200">{potOdds.toFixed(1)}%</span>
              </div>
            </section>

            <button
              type="button"
              onClick={calculate}
              disabled={loading || myCards.length !== 2}
              className="calculate-button"
            >
              <span>{loading ? 'Calculating…' : 'Calculate equity'}</span>
              {!loading && <span aria-hidden="true">→</span>}
            </button>
            {myCards.length !== 2 && (
              <p className="-mt-2 text-center text-xs text-slate-600">Select both hole cards to continue.</p>
            )}

            {equity !== null && (
              <section className="result-panel" aria-live="polite">
                <div className="flex items-end justify-between gap-4">
                  <div>
                    <p className="eyebrow">Estimated equity</p>
                    <p className="mt-2 text-5xl font-semibold tracking-[-0.04em] text-white">
                      {equity.toFixed(1)}<span className="ml-1 text-2xl text-emerald-400">%</span>
                    </p>
                  </div>
                  {hasRecommendation && (
                    <span className={`recommendation ${isProfitableCall ? 'recommendation-call' : 'recommendation-fold'}`}>
                      {isProfitableCall ? 'Call' : 'Fold'}
                    </span>
                  )}
                </div>
                {hasRecommendation && (
                  <div className="mt-5 grid grid-cols-2 gap-3 border-t border-white/[0.06] pt-4 text-xs">
                    <div><span className="block text-slate-500">Required</span><strong className="mt-1 block text-slate-200">{potOdds.toFixed(1)}%</strong></div>
                    <div><span className="block text-slate-500">Equity edge</span><strong className={`mt-1 block ${isProfitableCall ? 'text-emerald-400' : 'text-rose-400'}`}>{(equity - potOdds).toFixed(1)}%</strong></div>
                  </div>
                )}
              </section>
            )}

            {error && <div className="error-message" role="alert">{error}</div>}

            <p className="text-center text-[11px] leading-relaxed text-slate-600">
              Development build · Results are estimates, not financial advice.
            </p>
          </aside>
        </div>
      </main>
    </div>
  );
};

export default App;

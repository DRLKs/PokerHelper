import { useCallback, useMemo, useState } from 'react';
import { useAdvisor } from '../../application/useAdvisor';
import { useOpponentSettings } from '../../application/useOpponentSettings';
import { HOLE_CARD_COUNT, MAX_BOARD_CARDS } from '../../domain/tableDraft';
import { Card } from '../../domain/types';
import { AdvicePanel } from '../components/AdvicePanel';
import { CardSelector } from '../components/CardSelector';
import { CardSlot } from '../components/CardSlot';
import { CardView } from '../components/CardView';
import { ChipInput } from '../components/ChipInput';
import { HandHistory } from '../components/HandHistory';
import { OpponentSettings } from '../components/OpponentSettings';
import { SeatEditor } from '../components/SeatEditor';

type CardTarget = 'hand' | 'board';

export const AdvisorPage = () => {
  const advisor = useAdvisor();
  const opponents = useOpponentSettings();
  const { draft, addHeroCard, addBoardCard } = advisor;
  const [target, setTarget] = useState<CardTarget>('hand');

  const usedCards = useMemo(() => [...draft.heroCards, ...draft.board], [draft.heroCards, draft.board]);
  const betsOnTheTable = draft.seats.reduce((total, seat) => total + seat.committed, 0);
  const readsBySeat = useMemo(
    () => new Map((advisor.update?.opponents ?? []).map((read) => [read.seat, read])),
    [advisor.update],
  );

  const handleCardSelect = useCallback((card: Card) => {
    if (target === 'hand' && draft.heroCards.length < HOLE_CARD_COUNT) {
      addHeroCard(card);
      if (draft.heroCards.length === HOLE_CARD_COUNT - 1) setTarget('board');
      return;
    }
    if (target === 'board' && draft.board.length < MAX_BOARD_CARDS) addBoardCard(card);
  }, [target, draft.heroCards.length, draft.board.length, addHeroCard, addBoardCard]);

  return (
    <div className="grid gap-5 lg:grid-cols-[minmax(0,1.42fr)_minmax(330px,0.8fr)]">
      <div className="space-y-5">
        <section className="table-surface table-surface-compact">
          <div className="table-vignette" aria-hidden="true" />
          <div className="relative z-10">
            <div className="mb-4 flex flex-wrap items-start justify-between gap-3">
              <div>
                <p className="eyebrow text-emerald-200/50">Current hand</p>
                <h2 className="mt-1 text-xl font-semibold">The table</h2>
              </div>
              <div className="flex gap-2">
                <button type="button" className="secondary-button" onClick={advisor.nextStreet}>Next street</button>
                <button type="button" className="secondary-button" onClick={advisor.newHand}>New hand</button>
              </div>
            </div>

            <div
              className={`card-zone ${target === 'board' ? 'card-zone-active' : ''}`}
              onClick={() => setTarget('board')}
            >
              <button type="button" className="zone-label" onClick={() => setTarget('board')}>Board</button>
              <span className="card-row">
                {draft.board.map((card, index) => (
                  <CardView key={`${card.rank}${card.suit}`} card={card} onClick={() => advisor.removeBoardCard(index)} />
                ))}
                {Array.from({ length: MAX_BOARD_CARDS - draft.board.length }, (_, index) => <CardSlot key={index} />)}
              </span>
            </div>

            <div
              className={`card-zone ${target === 'hand' ? 'card-zone-active' : ''}`}
              onClick={() => setTarget('hand')}
            >
              <button type="button" className="zone-label" onClick={() => setTarget('hand')}>Your hand</button>
              <span className="card-row">
                {draft.heroCards.map((card, index) => (
                  <CardView key={`${card.rank}${card.suit}`} card={card} onClick={() => advisor.removeHeroCard(index)} />
                ))}
                {Array.from({ length: HOLE_CARD_COUNT - draft.heroCards.length }, (_, index) => <CardSlot key={index} />)}
              </span>
            </div>

            <div className="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-4">
              <ChipInput id="small-blind" label="Small blind" value={draft.blinds.small} onChange={(small) => advisor.updateBlinds({ small })} />
              <ChipInput id="big-blind" label="Big blind" value={draft.blinds.big} onChange={(big) => advisor.updateBlinds({ big })} />
              <ChipInput id="ante" label="Ante" value={draft.blinds.ante} onChange={(ante) => advisor.updateBlinds({ ante })} />
              <ChipInput id="pot" label="Pot" value={draft.pot} onChange={advisor.setPot} />
            </div>
            <p className="mt-2 text-xs text-emerald-50/50">
              Pot holds the chips from earlier streets and antes. Bets in front of the players this street: {betsOnTheTable}.
            </p>
          </div>
        </section>

        <section className="panel p-5 sm:p-6" aria-labelledby="seats-title">
          <p className="eyebrow">Seats</p>
          <h2 id="seats-title" className="mt-1 text-lg font-semibold">Players, stacks and bets</h2>
          <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {draft.seats.map((seat, index) => (
              <SeatEditor
                key={index}
                index={index}
                seat={seat}
                isButton={draft.buttonSeat === index}
                isHero={draft.heroSeat === index}
                read={readsBySeat.get(index)}
                onChange={advisor.updateSeat}
                onMakeButton={advisor.setButtonSeat}
                onMakeHero={advisor.setHeroSeat}
              />
            ))}
          </div>
        </section>

        <CardSelector
          onSelect={handleCardSelect}
          unavailableCards={usedCards}
          destinationLabel={target === 'hand' ? 'your hand' : 'the board'}
        />
      </div>

      <aside className="space-y-5">
        <button
          type="button"
          onClick={advisor.requestAdvice}
          disabled={advisor.requesting || advisor.snapshotResult.kind !== 'ready'}
          className="calculate-button"
        >
          <span>{advisor.requesting ? 'Thinking…' : 'Get advice'}</span>
          {!advisor.requesting && <span aria-hidden="true">→</span>}
        </button>
        {advisor.snapshotResult.kind === 'incomplete' && (
          <p className="-mt-2 text-center text-xs text-slate-600">{advisor.snapshotResult.problem}</p>
        )}
        {advisor.error && <div className="error-message" role="alert">{advisor.error}</div>}

        <AdvicePanel
          update={advisor.update}
          adviceMatchesTable={advisor.adviceMatchesTable}
          bigBlind={draft.blinds.big}
        />
        {advisor.update && <HandHistory history={advisor.update.history} />}
        <OpponentSettings
          population={opponents.population}
          onChangePopulation={opponents.changePopulation}
          onForgetOpponents={opponents.forgetOpponents}
          error={opponents.error}
        />
      </aside>
    </div>
  );
};

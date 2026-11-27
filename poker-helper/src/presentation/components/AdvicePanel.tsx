import React from 'react';
import { ActionValue, Advice, Chips, Confidence, PlannedResponse, ResponsePlan, TableUpdate } from '../../domain/advisor';
import {
  chipsWithBigBlinds,
  describeIssue,
  plannedResponseLabels,
  recommendationTone,
  signedChips,
} from '../format';

interface Props {
  update: TableUpdate | null;
  adviceMatchesTable: boolean;
  bigBlind: Chips;
}

const ConfidenceNotice = ({ confidence }: { confidence: Confidence }) => {
  if (confidence.level === 'high') return null;
  const title = confidence.level === 'low' ? 'Low confidence' : 'Medium confidence';
  return (
    <div className={`confidence-notice confidence-${confidence.level}`} role="status">
      <p className="font-semibold">{title}</p>
      <ul className="mt-1 space-y-1">
        {confidence.issues.map((issue) => (
          <li key={JSON.stringify(issue)}>{describeIssue(issue)}</li>
        ))}
      </ul>
    </div>
  );
};

const ResponsePlanView = ({ plan, opponentBets, bigBlind }: {
  plan: ResponsePlan;
  opponentBets: boolean;
  bigBlind: Chips;
}) => (
  <div className="advice-section">
    <p className="eyebrow">If they act</p>
    <p className="mt-2 text-sm font-semibold text-slate-100">{plan.summary}</p>
    <ul className="mt-3 space-y-1.5">
      {plan.responses.map((planned, index) => (
        <li key={index} className="plan-row">
          <span>{opponentBets ? 'They bet' : 'They raise to'} {chipsWithBigBlinds(planned.total, bigBlind)}</span>
          <span className={`plan-response plan-response-${planned.response}`}>
            {plannedResponseLabels[planned.response]}
          </span>
        </li>
      ))}
    </ul>
  </div>
);

const ReraisePlan = ({ response }: { response: PlannedResponse }) => (
  <div className="advice-section">
    <p className="eyebrow">If someone raises</p>
    <p className="mt-2 text-sm font-semibold text-slate-100">{plannedResponseLabels[response]}</p>
  </div>
);

const EvBreakdown = ({ values }: { values: ActionValue[] }) => (
  <details className="advice-section ev-details">
    <summary>Compare actions</summary>
    <p className="mt-2 text-xs text-slate-500">Chips won from here on compared with folding, best first.</p>
    <ul className="mt-2 space-y-1">
      {values.map((value, index) => (
        <li key={index} className="plan-row">
          <span>{value.instruction}</span>
          <span className="font-mono">{signedChips(value.ev)}</span>
        </li>
      ))}
    </ul>
  </details>
);

const AdviceView = ({ advice, bigBlind }: { advice: Advice; bigBlind: Chips }) => (
  <>
    <div className={`advice-instruction ${recommendationTone(advice.recommendation.kind)}`}>
      {advice.instruction}
    </div>
    <p className="mt-3 text-sm leading-relaxed text-slate-300">{advice.reason}</p>
    <ConfidenceNotice confidence={advice.confidence} />
    {advice.response_plan && (
      <ResponsePlanView
        plan={advice.response_plan}
        opponentBets={advice.recommendation.kind === 'check'}
        bigBlind={bigBlind}
      />
    )}
    {!advice.response_plan && advice.if_reraised && <ReraisePlan response={advice.if_reraised} />}
    {advice.ev_by_action.length > 0 && <EvBreakdown values={advice.ev_by_action} />}
  </>
);

const UpdateBody = ({ update, bigBlind }: { update: TableUpdate; bigBlind: Chips }) => {
  if (update.advice_error) {
    return <div className="error-message" role="alert">{update.advice_error}</div>;
  }
  if (!update.advice) {
    return <p className="text-sm text-slate-400">You have no decision to make right now.</p>;
  }
  return <AdviceView advice={update.advice} bigBlind={bigBlind} />;
};

export const AdvicePanel = React.memo<Props>(({ update, adviceMatchesTable, bigBlind }) => (
  <section className="result-panel" aria-live="polite" aria-labelledby="advice-title">
    <div className="flex items-center justify-between gap-3">
      <div>
        <p className="eyebrow">Advice</p>
        <h2 id="advice-title" className="mt-1 text-lg font-semibold text-white">What to do</h2>
      </div>
      {update && <span className="street-chip">{update.street}</span>}
    </div>
    <div className="mt-4">
      {update === null && (
        <p className="text-sm text-slate-400">Enter the table and press Get advice.</p>
      )}
      {update !== null && !adviceMatchesTable && (
        <p className="stale-note">The table changed since this advice. Press Get advice again.</p>
      )}
      {update !== null && <UpdateBody update={update} bigBlind={bigBlind} />}
      {update?.recorded_a_finished_hand && (
        <p className="mt-4 text-xs text-slate-500">The previous hand was added to the opponent stats.</p>
      )}
    </div>
  </section>
));

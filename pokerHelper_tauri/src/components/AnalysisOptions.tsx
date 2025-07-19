import React from 'react';
import { usePokerGame } from '../contexts/PokerGameContext';
import '../styles/analysis-options.css';

export const AnalysisOptions: React.FC = () => {
  const { analysisOptions, updateAnalysisOptions } = usePokerGame();

  const handleNumberOfOpponentsChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    updateAnalysisOptions({ numberOfOpponents: parseInt(e.target.value) });
  };

  const handleSmallBlindChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value) && value > 0) {
      updateAnalysisOptions({ smallBlind: value });
    }
  };

  const handleAccumulatedBetChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value) && value >= 0) {
      updateAnalysisOptions({ accumulatedBet: value });
    }
  };

  return (
    <div className="analysis-options">
      <h4>Analysis Settings</h4>
      
      <div className="option-group">
        <label htmlFor="opponents">Number of Opponents:</label>
        <select
          id="opponents"
          value={analysisOptions.numberOfOpponents}
          onChange={handleNumberOfOpponentsChange}
        >
          {[1, 2, 3, 4, 5, 6, 7, 8, 9].map(num => (
            <option key={num} value={num}>{num}</option>
          ))}
        </select>
      </div>

      <div className="option-group">
        <label htmlFor="small-blind">Small Blind:</label>
        <input
          id="small-blind"
          type="number"
          min="1"
          step="0.5"
          value={analysisOptions.smallBlind}
          onChange={handleSmallBlindChange}
        />
      </div>

      <div className="option-group">
        <label htmlFor="accumulated-bet">Accumulated Bet:</label>
        <input
          id="accumulated-bet"
          type="number"
          min="0"
          step="0.5"
          value={analysisOptions.accumulatedBet}
          onChange={handleAccumulatedBetChange}
        />
      </div>

      <div className="options-summary">
        <small>
          Current settings: {analysisOptions.numberOfOpponents} opponents, 
          ${analysisOptions.smallBlind} small blind, 
          ${analysisOptions.accumulatedBet} accumulated
        </small>
      </div>
    </div>
  );
};

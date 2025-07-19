import React, { useState } from 'react';
import { useLanguage } from '../contexts/LanguageContext';
import { usePokerGame } from '../contexts/PokerGameContext';
import { pokerAPIService } from '../services/pokerAPI';
import '../styles/analysis-display.css';

export const AnalysisDisplay: React.FC = () => {
  const { t } = useLanguage();
  const { analysis: pokerAnalysis, isLoading: isLoadingPoker, error: pokerError } = usePokerGame();
  const [testResult, setTestResult] = useState<string>('');
  const [isTesting, setIsTesting] = useState(false);

  const formatProbability = (prob: number) => `${(prob * 100).toFixed(1)}%`;

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult('Testing connection...');
    
    try {
      const result = await pokerAPIService.testConnection();
      if (result.success) {
        setTestResult(`✅ Connection successful: ${result.message}`);
        console.log('Connection test details:', result.details);
      } else {
        setTestResult(`❌ Connection failed: ${result.message}`);
        console.error('Connection test error:', result.details);
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      setTestResult(`❌ Test failed: ${errorMsg}`);
      console.error('Connection test exception:', error);
    } finally {
      setIsTesting(false);
    }
  };

  if (pokerError) {
    return (
      <div className="analysis-container error">
        <h3 className="analysis-title">{t.analysis.error}</h3>
        <p className="error-message">{pokerError}</p>
        
        <div className="debug-section">
          <button 
            onClick={handleTestConnection} 
            disabled={isTesting}
            style={{
              marginTop: '10px',
              padding: '8px 16px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isTesting ? 'not-allowed' : 'pointer',
              opacity: isTesting ? 0.6 : 1
            }}
          >
            {isTesting ? 'Testing...' : 'Test Connection'}
          </button>
          
          {testResult && (
            <div style={{ 
              marginTop: '10px', 
              padding: '8px', 
              backgroundColor: '#f8f9fa', 
              border: '1px solid #dee2e6',
              borderRadius: '4px',
              fontSize: '14px',
              whiteSpace: 'pre-wrap'
            }}>
              {testResult}
            </div>
          )}
        </div>
      </div>
    );
  }

  if (isLoadingPoker) {
    return (
      <div className="analysis-container loading">
        <h3 className="analysis-title">{t.analysis.title}</h3>
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>{t.analysis.loading}</p>
        </div>
      </div>
    );
  }

  if (!pokerAnalysis) {
    return (
      <div className="analysis-container placeholder">
        <h3 className="analysis-title">{t.analysis.title}</h3>
        <p className="placeholder-text">{t.analysis.selectCards}</p>
      </div>
    );
  }

  return (
    <div className="analysis-container">
      <div className="analysis-layout">
        {/* Left side - Poker Analysis */}
        <div className="poker-analysis-panel">
          <h3 className="analysis-title">{t.analysis.title}</h3>
          
          {/* Player Probabilities */}
          <div className="probability-section">
            <h4 className="section-title">Player Probabilities</h4>
            <div className="probability-list">
              <div className="probability-item">
                <span className="prob-label">Pair:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.pair)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Three of a Kind:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.threeOfAKind)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Straight:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.straight)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.flush)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Full House:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.fullHouse)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Four of a Kind:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.fourOfAKind)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Straight Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.straightFlush)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Royal Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.royalFlush)}</span>
              </div>
            </div>
          </div>

          {/* Opponent Probabilities */}
          <div className="probability-section">
            <h4 className="section-title">Opponent Probabilities</h4>
            <div className="probability-list">
              <div className="probability-item">
                <span className="prob-label">Pair:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.pair)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Three of a Kind:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.threeOfAKind)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Straight:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.straight)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.flush)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Full House:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.fullHouse)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Four of a Kind:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.fourOfAKind)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Straight Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.straightFlush)}</span>
              </div>
              <div className="probability-item">
                <span className="prob-label">Royal Flush:</span>
                <span className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.royalFlush)}</span>
              </div>
            </div>
          </div>

          {/* Decision */}
          <div className="decision-section">
            <h4 className="section-title">Recommended Action</h4>
            <div className="decision-info">
              <div className="decision-item">
                <span className="decision-label">Action:</span>
                <span className={`decision-value action-${pokerAnalysis.decision.action.toLowerCase()}`}>
                  {pokerAnalysis.decision.action}
                </span>
              </div>
              {pokerAnalysis.decision.betAmount > 0 && (
                <div className="decision-item">
                  <span className="decision-label">Bet Amount:</span>
                  <span className="decision-value">${pokerAnalysis.decision.betAmount}</span>
                </div>
              )}
              <div className="decision-description">
                <p>{pokerAnalysis.decision.description}</p>
              </div>
            </div>
          </div>

          <div className="analysis-timestamp">
            Updated: {new Date(pokerAnalysis.timestamp).toLocaleTimeString()}
          </div>
        </div>

        {/* Right side - Additional content area */}
        <div className="additional-content-panel">
          {/* This area can be used for other components or information */}
        </div>
      </div>
    </div>
  );
};

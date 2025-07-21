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
    <div className="analysis-container compact">
      <h3 className="analysis-title">{t.analysis.title}</h3>
      
      <div className="analysis-grid">
        {/* Player vs Opponent Comparison */}
        <div className="probabilities-comparison">
          <div className="prob-grid">
            <div className="prob-header">Hand</div>
            <div className="prob-header">You</div>
            <div className="prob-header">Opponent</div>
            
            <div className="prob-label">Pair</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.pair)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.pair)}</div>
            
            <div className="prob-label">3 of Kind</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.threeOfAKind)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.threeOfAKind)}</div>
            
            <div className="prob-label">Straight</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.straight)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.straight)}</div>
            
            <div className="prob-label">Flush</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.flush)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.flush)}</div>
            
            <div className="prob-label">Full House</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.fullHouse)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.fullHouse)}</div>
            
            <div className="prob-label">4 of Kind</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.fourOfAKind)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.fourOfAKind)}</div>
            
            <div className="prob-label">Str. Flush</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.straightFlush)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.straightFlush)}</div>
            
            <div className="prob-label">Royal</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.playerProbabilities.royalFlush)}</div>
            <div className="prob-value">{formatProbability(pokerAnalysis.opponentProbabilities.royalFlush)}</div>
          </div>
        </div>

        {/* Decision */}
        <div className="decision-compact">
          <div className="decision-action">
            <span className={`action-badge action-${pokerAnalysis.decision.action.toLowerCase()}`}>
              {pokerAnalysis.decision.action}
            </span>
            {pokerAnalysis.decision.betAmount > 0 && (
              <span className="bet-amount">${pokerAnalysis.decision.betAmount}</span>
            )}
          </div>
          <p className="decision-description">{pokerAnalysis.decision.description}</p>
          <div className="timestamp">
            {new Date(pokerAnalysis.timestamp).toLocaleTimeString()}
          </div>
        </div>
      </div>
    </div>
  );
};

import React from 'react';
import { useLanguage } from '../contexts/LanguageContext';
import { usePokerAnalysis } from '../hooks/usePokerAnalysis';
import '../styles/analysis-display.css';

export const AnalysisDisplay: React.FC = () => {
  const { t } = useLanguage();
  const { analysis, isLoading, error } = usePokerAnalysis();

  if (error) {
    return (
      <div className="analysis-container error">
        <h3 className="analysis-title">{t.analysis.error}</h3>
        <p className="error-message">{error}</p>
      </div>
    );
  }

  if (isLoading) {
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

  if (!analysis) {
    return (
      <div className="analysis-container placeholder">
        <h3 className="analysis-title">{t.analysis.title}</h3>
        <p className="placeholder-text">{t.analysis.selectCards}</p>
      </div>
    );
  }

  return (
    <div className="analysis-container">
      <h3 className="analysis-title">{t.analysis.title}</h3>
      
      <div className="analysis-grid">
        <div className="analysis-item">
          <span className="analysis-label">{t.analysis.handType}:</span>
          <span className="analysis-value hand-type">{analysis.handType}</span>
        </div>
        
        <div className="analysis-item">
          <span className="analysis-label">{t.analysis.strength}:</span>
          <div className="strength-bar">
            <div 
              className="strength-fill" 
              style={{ width: `${analysis.handStrength}%` }}
            ></div>
            <span className="strength-text">{analysis.handStrength.toFixed(1)}%</span>
          </div>
        </div>
        
        <div className="analysis-item">
          <span className="analysis-label">{t.analysis.winProbability}:</span>
          <span className={`analysis-value win-probability ${getWinProbabilityClass(analysis.winProbability)}`}>
            {analysis.winProbability.toFixed(1)}%
          </span>
        </div>
      </div>

      {analysis.recommendations && analysis.recommendations.length > 0 && (
        <div className="recommendations">
          <h4 className="recommendations-title">{t.analysis.recommendations}:</h4>
          <ul className="recommendations-list">
            {analysis.recommendations.map((recommendation, index) => (
              <li key={index} className="recommendation-item">
                {recommendation}
              </li>
            ))}
          </ul>
        </div>
      )}
      
      <div className="analysis-timestamp">
        {t.analysis.updated}: {new Date(analysis.timestamp).toLocaleTimeString()}
      </div>
    </div>
  );
};

// Función auxiliar para determinar la clase CSS basada en la probabilidad
const getWinProbabilityClass = (probability: number): string => {
  if (probability >= 70) return 'high';
  if (probability >= 40) return 'medium';
  return 'low';
};

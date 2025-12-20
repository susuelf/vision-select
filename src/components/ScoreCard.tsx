// VisionSelect AI - Score Card Component
// AI pontszám megjelenítő komponens

import React from 'react';
import { QualityScore, formatScore, getScoreColor } from '../types';
import './ScoreCard.css';

interface ScoreCardProps {
  score: QualityScore;
  showDetails?: boolean;
  compact?: boolean;
}

/**
 * AI pontszám megjelenítő kártya
 * Megjeleníti az élesség, expozíció, kontraszt és összes pontszámot
 */
export const ScoreCard: React.FC<ScoreCardProps> = ({ 
  score, 
  showDetails = true,
  compact = false 
}) => {
  if (compact) {
    return (
      <div className="score-card-compact">
        <div 
          className="score-badge"
          style={{ backgroundColor: getScoreColor(score.overall) }}
        >
          {formatScore(score.overall)}
        </div>
      </div>
    );
  }

  return (
    <div className="score-card">
      <div className="score-card-header">
        <span className="score-label">AI Pontszám</span>
        <span 
          className="score-overall"
          style={{ color: getScoreColor(score.overall) }}
        >
          {formatScore(score.overall)}
        </span>
      </div>

      {showDetails && (
        <div className="score-details">
          <ScoreBar label="Élesség" value={score.sharpness} />
          <ScoreBar label="Expozíció" value={score.exposure} />
          <ScoreBar label="Kontraszt" value={score.contrast} />
          <ScoreBar label="Zajszint" value={score.noise_estimate} />
        </div>
      )}
    </div>
  );
};

interface ScoreBarProps {
  label: string;
  value: number;
}

const ScoreBar: React.FC<ScoreBarProps> = ({ label, value }) => {
  const percentage = Math.round(value * 100);
  const color = getScoreColor(value);

  return (
    <div className="score-bar-container">
      <div className="score-bar-label">
        <span>{label}</span>
        <span>{percentage}%</span>
      </div>
      <div className="score-bar-track">
        <div 
          className="score-bar-fill"
          style={{ 
            width: `${percentage}%`,
            backgroundColor: color 
          }}
        />
      </div>
    </div>
  );
};

export default ScoreCard;

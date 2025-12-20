// VisionSelect AI - GroupView Component
// Csoportosított képek megjelenítése és kezelése

import React, { useState } from 'react';
import type { ImageGroup, QualityScore, RawFile } from '../types';
import { formatScore, getScoreColor } from '../types';
import './GroupView.css';

interface GroupViewProps {
  groups: ImageGroup[];
  scores: Record<string, QualityScore>;
  onSelectGroup?: (groupId: number) => void;
  onApproveWinner?: (groupId: number, imagePath: string) => void;
  onOverrideWinner?: (groupId: number, newWinnerPath: string) => void;
}

/**
 * Csoportosított képek megjelenítő nézet
 * Burst/bracket sorozatokat mutat, az AI által javasolt győztes kiemelésével
 */
export const GroupView: React.FC<GroupViewProps> = ({
  groups,
  scores,
  onSelectGroup,
  onApproveWinner,
  onOverrideWinner,
}) => {
  const [expandedGroupId, setExpandedGroupId] = useState<number | null>(null);

  // Csak a többelemes csoportok (burst/bracket)
  const multiImageGroups = groups.filter(g => g.images.length > 1);
  const singleImages = groups.filter(g => g.images.length === 1);

  const handleGroupClick = (groupId: number) => {
    setExpandedGroupId(expandedGroupId === groupId ? null : groupId);
    onSelectGroup?.(groupId);
  };

  if (multiImageGroups.length === 0) {
    return (
      <div className="group-view-empty">
        <div className="empty-icon">📷</div>
        <p>Nincs csoportosítható sorozatfelvétel</p>
        <span className="empty-hint">
          {singleImages.length} egyedi kép található
        </span>
      </div>
    );
  }

  return (
    <div className="group-view">
      <div className="group-view-header">
        <h2>Sorozatfelvételek ({multiImageGroups.length})</h2>
        <span className="hint">Kattints egy csoportra a részletekért</span>
      </div>

      <div className="group-list">
        {multiImageGroups.map((group) => (
          <GroupCard
            key={group.id}
            group={group}
            scores={scores}
            isExpanded={expandedGroupId === group.id}
            onClick={() => handleGroupClick(group.id)}
            onApproveWinner={onApproveWinner}
            onOverrideWinner={onOverrideWinner}
          />
        ))}
      </div>
    </div>
  );
};

interface GroupCardProps {
  group: ImageGroup;
  scores: Record<string, QualityScore>;
  isExpanded: boolean;
  onClick: () => void;
  onApproveWinner?: (groupId: number, imagePath: string) => void;
  onOverrideWinner?: (groupId: number, newWinnerPath: string) => void;
}

const GroupCard: React.FC<GroupCardProps> = ({
  group,
  scores,
  isExpanded,
  onClick,
  onApproveWinner,
  onOverrideWinner,
}) => {
  // Legjobb kép meghatározása
  const rankedImages = [...group.images].sort((a, b) => {
    const scoreA = scores[a]?.overall ?? 0;
    const scoreB = scores[b]?.overall ?? 0;
    return scoreB - scoreA;
  });

  const winner = rankedImages[0];
  const winnerScore = scores[winner]?.overall ?? 0;
  const alternatives = rankedImages.slice(1, 4); // Top 3 alternatíva

  const getFileName = (path: string) => {
    return path.split(/[/\\]/).pop() || path;
  };

  return (
    <div className={`group-card ${isExpanded ? 'expanded' : ''}`}>
      <div className="group-card-header" onClick={onClick}>
        <div className="group-info">
          <span className="group-type-badge">
            {group.group_type === 'Burst' ? '📸 Sorozat' : '🔲 Bracket'}
          </span>
          <span className="group-count">{group.images.length} kép</span>
        </div>
        
        <div className="winner-preview">
          <span className="winner-name">{getFileName(winner)}</span>
          <div 
            className="winner-score"
            style={{ backgroundColor: getScoreColor(winnerScore) }}
          >
            {formatScore(winnerScore)}
          </div>
        </div>

        <div className="expand-icon">{isExpanded ? '▼' : '▶'}</div>
      </div>

      {isExpanded && (
        <div className="group-card-content">
          <div className="winner-section">
            <h4>🏆 AI Javaslat</h4>
            <div className="winner-card">
              <div className="winner-thumbnail">
                {/* Thumbnail itt lenne betöltve */}
                <div className="placeholder-thumb">📷</div>
              </div>
              <div className="winner-info">
                <span className="filename">{getFileName(winner)}</span>
                <div className="score-details">
                  {scores[winner] && (
                    <>
                      <span>Élesség: {formatScore(scores[winner].sharpness)}</span>
                      <span>Expozíció: {formatScore(scores[winner].exposure)}</span>
                    </>
                  )}
                </div>
              </div>
              <button 
                className="approve-btn"
                onClick={() => onApproveWinner?.(group.id, winner)}
              >
                ✓ Elfogad
              </button>
            </div>
          </div>

          {alternatives.length > 0 && (
            <div className="alternatives-section">
              <h4>Alternatívák</h4>
              <div className="alternatives-grid">
                {alternatives.map((alt, idx) => (
                  <div 
                    key={alt} 
                    className="alternative-card"
                    onClick={() => onOverrideWinner?.(group.id, alt)}
                  >
                    <div className="alt-rank">#{idx + 2}</div>
                    <div className="placeholder-thumb small">📷</div>
                    <span className="alt-name">{getFileName(alt)}</span>
                    <div 
                      className="alt-score"
                      style={{ backgroundColor: getScoreColor(scores[alt]?.overall ?? 0) }}
                    >
                      {formatScore(scores[alt]?.overall ?? 0)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default GroupView;

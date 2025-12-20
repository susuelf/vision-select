// VisionSelect AI - ImageCard Component
// Integrált AI elemzés és ScoreCard megjelenítés

import { useState, useEffect, useRef, useCallback } from 'react';
import { getThumbnail, analyzeImage } from '../api';
import type { RawFile, ThumbnailData, QualityScore } from '../types';
import { formatFileSize, formatScore, getScoreColor } from '../types';

interface ImageCardProps {
  file: RawFile;
  isSelected?: boolean;
  onClick?: () => void;
  onDoubleClick?: () => void;
  showScore?: boolean;
}

export function ImageCard({ 
  file, 
  isSelected = false, 
  onClick, 
  onDoubleClick,
  showScore = true
}: ImageCardProps) {
  const [thumbnail, setThumbnail] = useState<ThumbnailData | null>(null);
  const [score, setScore] = useState<QualityScore | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [hasError, setHasError] = useState(false);
  const cardRef = useRef<HTMLDivElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loadingRef = useRef(false);

  // Thumbnail betöltése
  const loadThumbnail = useCallback(async (signal: AbortSignal) => {
    if (loadingRef.current) return;
    
    loadingRef.current = true;
    setIsLoading(true);
    setHasError(false);
    
    try {
      const thumb = await getThumbnail(file.path, 256);
      
      if (!signal.aborted) {
        setThumbnail(thumb);
        
        // AI elemzés indítása miután a thumbnail betöltődött
        if (showScore) {
          setIsAnalyzing(true);
          try {
            const qualityScore = await analyzeImage(file.path);
            if (!signal.aborted) {
              setScore(qualityScore);
            }
          } catch (err) {
            console.warn('AI analysis error:', file.filename);
          } finally {
            if (!signal.aborted) {
              setIsAnalyzing(false);
            }
          }
        }
      }
    } catch (error) {
      if (!signal.aborted) {
        console.warn('Thumbnail error:', file.filename);
        setHasError(true);
      }
    } finally {
      if (!signal.aborted) {
        loadingRef.current = false;
        setIsLoading(false);
      }
    }
  }, [file.path, file.filename, showScore]);
  
  // IntersectionObserver setup
  useEffect(() => {
    setThumbnail(null);
    setScore(null);
    setHasError(false);
    setIsLoading(false);
    setIsAnalyzing(false);
    loadingRef.current = false;
    
    const abortController = new AbortController();
    
    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          loadThumbnail(abortController.signal);
          observerRef.current?.disconnect();
        }
      },
      { 
        rootMargin: '200px',
        threshold: 0
      }
    );
    
    if (cardRef.current) {
      observerRef.current.observe(cardRef.current);
    }
    
    return () => {
      abortController.abort();
      observerRef.current?.disconnect();
      loadingRef.current = false;
    };
  }, [file.path, loadThumbnail]);
  
  return (
    <div 
      ref={cardRef}
      className={`image-card ${isSelected ? 'selected' : ''}`}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      style={{ minHeight: '200px' }}
    >
      <div className="image-card-thumbnail">
        {isLoading && (
          <div className="thumbnail-loading">
            <div className="loading-spinner" />
          </div>
        )}
        
        {hasError && !isLoading && (
          <div className="thumbnail-error">
            <span>⚠️</span>
          </div>
        )}
        
        {thumbnail && !isLoading && (
          <img 
            src={thumbnail.data_base64} 
            alt={file.filename}
            loading="eager"
            decoding="async"
          />
        )}
        
        {!thumbnail && !isLoading && !hasError && (
          <div className="thumbnail-placeholder" />
        )}

        {/* AI Score Badge Overlay */}
        {score && !isAnalyzing && (
          <div 
            className="score-badge-overlay"
            title={`Élesség: ${formatScore(score.sharpness)} | Expozíció: ${formatScore(score.exposure)}`}
          >
            <div 
              className="score-badge"
              style={{ backgroundColor: getScoreColor(score.overall) }}
            >
              {formatScore(score.overall)}
            </div>
          </div>
        )}

        {/* Analyzing indicator */}
        {isAnalyzing && (
          <div className="analyzing-indicator">
            <div className="analyzing-pulse" />
          </div>
        )}
      </div>
      
      <div className="image-card-info">
        <div className="filename" title={file.filename}>
          {file.filename}
        </div>
        <div className="meta">
          <span className="extension">{file.extension.toUpperCase()}</span>
          <span className="size">{formatFileSize(file.size_bytes)}</span>
        </div>
      </div>
    </div>
  );
}

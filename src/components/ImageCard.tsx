// VisionSelect AI - ImageCard Component
// Optimalizált: nincs mesterséges késleltetés, megfelelő race condition kezelés

import { useState, useEffect, useRef, useCallback } from 'react';
import { getThumbnail } from '../api';
import type { RawFile, ThumbnailData } from '../types';
import { formatFileSize } from '../types';

interface ImageCardProps {
  file: RawFile;
  isSelected?: boolean;
  onClick?: () => void;
  onDoubleClick?: () => void;
}

export function ImageCard({ 
  file, 
  isSelected = false, 
  onClick, 
  onDoubleClick 
}: ImageCardProps) {
  const [thumbnail, setThumbnail] = useState<ThumbnailData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [hasError, setHasError] = useState(false);
  const cardRef = useRef<HTMLDivElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loadingRef = useRef(false); // Ref a loading state követésére dependency nélkül

  // Thumbnail betöltése - STABIL referenciával (nem függ az isLoading state-től)
  const loadThumbnail = useCallback(async (signal: AbortSignal) => {
    if (loadingRef.current) return;
    
    loadingRef.current = true;
    setIsLoading(true);
    setHasError(false);
    
    try {
      const thumb = await getThumbnail(file.path, 256);
      
      if (!signal.aborted) {
        setThumbnail(thumb);
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
  }, [file.path, file.filename]); // Csak a file változásakor jön létre újra
  
  // IntersectionObserver setup
  useEffect(() => {
    // Reset state when file changes
    setThumbnail(null);
    setHasError(false);
    setIsLoading(false);
    loadingRef.current = false;
    
    const abortController = new AbortController();
    
    // Observer létrehozása
    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          loadThumbnail(abortController.signal);
          
          // Lecsatlakozás az observerről - csak egyszer kell betölteni
          observerRef.current?.disconnect();
        }
      },
      { 
        rootMargin: '200px', // Növelt előtöltési zóna
        threshold: 0
      }
    );
    
    if (cardRef.current) {
      observerRef.current.observe(cardRef.current);
    }
    
    // Cleanup
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
      style={{ minHeight: '200px' }} // Layout shift prevention
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
            loading="eager" // Már lazy loadoltuk az observer-rel
            decoding="async"
          />
        )}
        
        {!thumbnail && !isLoading && !hasError && (
          <div className="thumbnail-placeholder" />
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

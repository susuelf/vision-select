// VisionSelect AI - Image Card Component
// Egyedi kép kártya thumbnail-lel és EXIF infóval

import { useState, useEffect } from 'react';
import { getThumbnail, getExifSummary } from '../api';
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
  const [exifSummary, setExifSummary] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  
  useEffect(() => {
    let isMounted = true;
    
    async function loadData() {
      setIsLoading(true);
      setHasError(false);
      
      try {
        // Thumbnail betöltése
        const thumb = await getThumbnail(file.path, 256);
        if (isMounted) {
          setThumbnail(thumb);
        }
        
        // EXIF betöltése (opcionális, nem blokkol)
        try {
          const exif = await getExifSummary(file.path);
          if (isMounted) {
            setExifSummary(exif);
          }
        } catch {
          // EXIF hiba nem kritikus
        }
      } catch (error) {
        console.error('Thumbnail load error:', error);
        if (isMounted) {
          setHasError(true);
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }
    
    loadData();
    
    return () => {
      isMounted = false;
    };
  }, [file.path]);
  
  return (
    <div 
      className={`image-card ${isSelected ? 'selected' : ''}`}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
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
            loading="lazy"
          />
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
        {exifSummary && (
          <div className="exif-summary">{exifSummary}</div>
        )}
      </div>
    </div>
  );
}

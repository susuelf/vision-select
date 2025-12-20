// VisionSelect AI - ImageGrid Component
// Egyszerűsített: nincs staggered loading, nincs mesterséges késleltetés

import type { RawFile } from '../types';
import { ImageCard } from './ImageCard';

interface ImageGridProps {
  files: RawFile[];
  selectedPath: string | null;
  onImageSelect: (file: RawFile) => void;
  onImageDoubleClick?: (file: RawFile) => void;
}

export function ImageGrid({ 
  files, 
  selectedPath, 
  onImageSelect,
  onImageDoubleClick 
}: ImageGridProps) {
  if (files.length === 0) {
    return (
      <div className="image-grid-empty">
        <div className="empty-icon">📷</div>
        <h3>Nincs megjeleníthető kép</h3>
        <p>Válassz ki egy mappát a RAW fájlok betöltéséhez</p>
      </div>
    );
  }
  
  return (
    <div className="image-grid">
      {files.map((file) => (
        <ImageCard
          key={file.path}
          file={file}
          isSelected={file.path === selectedPath}
          onClick={() => onImageSelect(file)}
          onDoubleClick={() => onImageDoubleClick?.(file)}
        />
      ))}
    </div>
  );
}

// VisionSelect AI - Folder Picker Component
// Mappa kiválasztó gomb komponens

import { useState } from 'react';
import { selectFolder } from '../api';

interface FolderPickerProps {
  onFolderSelected: (path: string) => void;
  disabled?: boolean;
}

export function FolderPicker({ onFolderSelected, disabled = false }: FolderPickerProps) {
  const [isLoading, setIsLoading] = useState(false);
  
  async function handleClick() {
    setIsLoading(true);
    try {
      const path = await selectFolder();
      if (path) {
        onFolderSelected(path);
      }
    } catch (error) {
      console.error('Folder selection error:', error);
    } finally {
      setIsLoading(false);
    }
  }
  
  return (
    <button 
      className="folder-picker-btn"
      onClick={handleClick}
      disabled={disabled || isLoading}
    >
      <svg 
        className="folder-icon" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        strokeWidth="2"
      >
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      {isLoading ? 'Betöltés...' : 'Mappa kiválasztása'}
    </button>
  );
}

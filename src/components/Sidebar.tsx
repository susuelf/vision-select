// VisionSelect AI - Sidebar Component
// Oldalsáv mappák és statisztikák megjelenítéséhez

import type { ScanResult } from '../types';
import { formatFileSize } from '../types';
import { FolderPicker } from './FolderPicker';

interface SidebarProps {
  currentFolder: string | null;
  scanResult: ScanResult | null;
  isScanning: boolean;
  onFolderSelected: (path: string) => void;
}

export function Sidebar({ 
  currentFolder, 
  scanResult, 
  isScanning,
  onFolderSelected 
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1 className="app-title">
          <span className="app-icon">📸</span>
          VisionSelect AI
        </h1>
      </div>
      
      <div className="sidebar-section">
        <h2>Mappa</h2>
        <FolderPicker 
          onFolderSelected={onFolderSelected}
          disabled={isScanning}
        />
        
        {currentFolder && (
          <div className="current-folder">
            <div className="folder-path" title={currentFolder}>
              {currentFolder}
            </div>
          </div>
        )}
      </div>
      
      {scanResult && (
        <div className="sidebar-section">
          <h2>Statisztikák</h2>
          <div className="stats">
            <div className="stat-item">
              <span className="stat-label">Képek száma</span>
              <span className="stat-value">{scanResult.total_count}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Összméret</span>
              <span className="stat-value">{formatFileSize(scanResult.total_size_bytes)}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Szkennelési idő</span>
              <span className="stat-value">{scanResult.scan_duration_ms} ms</span>
            </div>
          </div>
        </div>
      )}
      
      {isScanning && (
        <div className="sidebar-section">
          <div className="scanning-indicator">
            <div className="loading-spinner" />
            <span>Szkennelés folyamatban...</span>
          </div>
        </div>
      )}
      
      <div className="sidebar-footer">
        <div className="version">v0.1.0</div>
      </div>
    </aside>
  );
}

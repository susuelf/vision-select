// VisionSelect AI - Main Application
// Fő alkalmazás komponens - Tab váltás (Grid/Groups)

import { useState } from 'react';
import { Sidebar, ImageGrid } from './components';
import { GroupView } from './components/GroupView';
import { scanFolder, getImageGroups } from './api';
import type { RawFile, ScanResult, ImageGroup, QualityScore } from './types';
import './App.css';

type ViewMode = 'grid' | 'groups';

function App() {
  const [currentFolder, setCurrentFolder] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [selectedFile, setSelectedFile] = useState<RawFile | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Phase II: AI states
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [groups, setGroups] = useState<ImageGroup[]>([]);
  const [scores, setScores] = useState<Record<string, QualityScore>>({});
  const [isLoadingGroups, setIsLoadingGroups] = useState(false);

  async function handleFolderSelected(path: string) {
    setCurrentFolder(path);
    setIsScanning(true);
    setError(null);
    setSelectedFile(null);
    setGroups([]);
    setScores({});
    
    try {
      const result = await scanFolder(path, true);
      setScanResult(result);
      
      // Automatikus csoportosítás betöltése
      loadGroups(path);
    } catch (err) {
      console.error('Scan error:', err);
      setError(err instanceof Error ? err.message : 'Ismeretlen hiba');
      setScanResult(null);
    } finally {
      setIsScanning(false);
    }
  }

  async function loadGroups(folder: string) {
    setIsLoadingGroups(true);
    try {
      const imageGroups = await getImageGroups(folder);
      setGroups(imageGroups);
    } catch (err) {
      console.warn('Groups loading error:', err);
    } finally {
      setIsLoadingGroups(false);
    }
  }

  function handleImageSelect(file: RawFile) {
    setSelectedFile(file);
  }

  function handleImageDoubleClick(file: RawFile) {
    console.log('Double click:', file.filename);
  }

  function handleApproveWinner(groupId: number, imagePath: string) {
    console.log('Approved winner:', imagePath, 'for group:', groupId);
    // TODO: Mentés adatbázisba
  }

  function handleOverrideWinner(groupId: number, newWinnerPath: string) {
    console.log('Override winner:', newWinnerPath, 'for group:', groupId);
    // TODO: Frissítés és mentés
  }

  // Burst csoportok száma
  const burstGroupCount = groups.filter(g => g.images.length > 1).length;

  return (
    <div className="app">
      <Sidebar
        currentFolder={currentFolder}
        scanResult={scanResult}
        isScanning={isScanning}
        onFolderSelected={handleFolderSelected}
      />
      
      <main className="main-content">
        {/* View Mode Tabs */}
        <div className="view-tabs">
          <button 
            className={`view-tab ${viewMode === 'grid' ? 'active' : ''}`}
            onClick={() => setViewMode('grid')}
          >
            🖼️ Összes kép
          </button>
          <button 
            className={`view-tab ${viewMode === 'groups' ? 'active' : ''}`}
            onClick={() => setViewMode('groups')}
          >
            📸 Sorozatok {isLoadingGroups ? '...' : burstGroupCount > 0 && <span className="tab-badge">{burstGroupCount}</span>}
          </button>
        </div>

        {error && (
          <div className="error-banner">
            <span>⚠️ {error}</span>
            <button onClick={() => setError(null)}>✕</button>
          </div>
        )}
        
        {viewMode === 'grid' && (
          <ImageGrid
            files={scanResult?.files ?? []}
            selectedPath={selectedFile?.path ?? null}
            onImageSelect={handleImageSelect}
            onImageDoubleClick={handleImageDoubleClick}
          />
        )}

        {viewMode === 'groups' && (
          <GroupView
            groups={groups}
            scores={scores}
            onApproveWinner={handleApproveWinner}
            onOverrideWinner={handleOverrideWinner}
          />
        )}
      </main>
    </div>
  );
}

export default App;

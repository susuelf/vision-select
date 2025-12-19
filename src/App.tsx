// VisionSelect AI - Main Application
// Fő alkalmazás komponens

import { useState } from 'react';
import { Sidebar, ImageGrid } from './components';
import { scanAndSave } from './api';
import type { RawFile, ScanResult } from './types';
import './App.css';

function App() {
  const [currentFolder, setCurrentFolder] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [selectedFile, setSelectedFile] = useState<RawFile | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleFolderSelected(path: string) {
    setCurrentFolder(path);
    setIsScanning(true);
    setError(null);
    setSelectedFile(null);
    
    try {
      const result = await scanAndSave(path, true);
      setScanResult(result);
    } catch (err) {
      console.error('Scan error:', err);
      setError(err instanceof Error ? err.message : 'Ismeretlen hiba');
      setScanResult(null);
    } finally {
      setIsScanning(false);
    }
  }

  function handleImageSelect(file: RawFile) {
    setSelectedFile(file);
  }

  function handleImageDoubleClick(file: RawFile) {
    // TODO: Teljes képnézet megnyitása
    console.log('Double click:', file.filename);
  }

  return (
    <div className="app">
      <Sidebar
        currentFolder={currentFolder}
        scanResult={scanResult}
        isScanning={isScanning}
        onFolderSelected={handleFolderSelected}
      />
      
      <main className="main-content">
        {error && (
          <div className="error-banner">
            <span>⚠️ {error}</span>
            <button onClick={() => setError(null)}>✕</button>
          </div>
        )}
        
        <ImageGrid
          files={scanResult?.files ?? []}
          selectedPath={selectedFile?.path ?? null}
          onImageSelect={handleImageSelect}
          onImageDoubleClick={handleImageDoubleClick}
        />
      </main>
    </div>
  );
}

export default App;

// VisionSelect AI - Tauri API Wrapper
// Frontend API hívások a Rust backend felé

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { RawFile, ScanResult, ThumbnailData, ExifData, ImageRecord } from './types';

/**
 * Mappa kiválasztó dialógus megnyitása
 */
export async function selectFolder(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Válassz mappát'
  });
  
  return selected as string | null;
}

/**
 * Mappa szkennelése RAW fájlokért
 */
export async function scanFolder(path: string, recursive: boolean = true): Promise<ScanResult> {
  return invoke<ScanResult>('scan_folder', { path, recursive });
}

/**
 * Thumbnail lekérése egy képhez
 */
export async function getThumbnail(path: string, maxSize: number = 512): Promise<ThumbnailData> {
  return invoke<ThumbnailData>('get_thumbnail', { path, maxSize });
}

/**
 * Több thumbnail lekérése egyszerre
 */
export async function getThumbnails(
  paths: string[], 
  maxSize: number = 512
): Promise<Array<ThumbnailData | null>> {
  const results = await invoke<Array<{ Ok?: ThumbnailData; Err?: string }>>('get_thumbnails', { 
    paths, 
    maxSize 
  });
  
  return results.map(r => r.Ok ?? null);
}

/**
 * EXIF adatok lekérése
 */
export async function getExif(path: string): Promise<ExifData> {
  return invoke<ExifData>('get_exif', { path });
}

/**
 * EXIF összefoglaló string lekérése
 */
export async function getExifSummary(path: string): Promise<string> {
  return invoke<string>('get_exif_summary', { path });
}

/**
 * Kép mentése az adatbázisba
 */
export async function saveImage(file: RawFile): Promise<number> {
  return invoke<number>('save_image', { file });
}

/**
 * Képek lekérése mappából (adatbázisból)
 */
export async function getImagesFromDb(folder: string): Promise<ImageRecord[]> {
  return invoke<ImageRecord[]>('get_images_from_db', { folder });
}

/**
 * Szkennelés és adatbázisba mentés egyben
 */
export async function scanAndSave(path: string, recursive: boolean = true): Promise<ScanResult> {
  return invoke<ScanResult>('scan_and_save', { path, recursive });
}

/**
 * App verzió lekérdezése
 */
export async function getAppVersion(): Promise<string> {
  return invoke<string>('get_app_version');
}

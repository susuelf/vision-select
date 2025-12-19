// VisionSelect AI - TypeScript Types
// Frontend típusdefiníciók a Tauri API-hoz

/**
 * RAW fájl alapinformációk
 */
export interface RawFile {
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  modified_at: string | null;
}

/**
 * EXIF metaadatok
 */
export interface ExifData {
  iso: number | null;
  shutter_speed: string | null;
  aperture: number | null;
  focal_length: number | null;
  capture_date: string | null;
  camera_make: string | null;
  camera_model: string | null;
  lens_model: string | null;
  orientation: number | null;
}

/**
 * Kép rekord az adatbázisban
 */
export interface ImageRecord {
  id: number | null;
  file_path: string;
  filename: string;
  extension: string;
  file_size: number;
  file_hash: string | null;
  thumbnail_path: string | null;
  exif: ExifData | null;
  created_at: string | null;
}

/**
 * Thumbnail adat base64 kódolással
 */
export interface ThumbnailData {
  image_path: string;
  data_base64: string;
  width: number;
  height: number;
}

/**
 * Szkennelési eredmény
 */
export interface ScanResult {
  files: RawFile[];
  total_count: number;
  total_size_bytes: number;
  scan_duration_ms: number;
}

/**
 * Támogatott RAW formátumok
 */
export const RAW_EXTENSIONS = [
  "arw", // Sony
  "cr2", // Canon
  "cr3", // Canon (új)
  "nef", // Nikon
  "nrw", // Nikon (kompakt)
  "raf", // Fujifilm
  "orf", // Olympus
  "rw2", // Panasonic
  "pef", // Pentax
  "srw", // Samsung
  "dng", // Adobe DNG
] as const;

export type RawExtension = (typeof RAW_EXTENSIONS)[number];

/**
 * Ellenőrzi, hogy a kiterjesztés RAW formátum-e
 */
export function isRawExtension(ext: string): boolean {
  const extLower = ext.toLowerCase().replace(/^\./, "");
  return RAW_EXTENSIONS.includes(extLower as RawExtension);
}

/**
 * Fájlméret formázása olvasható formátumba
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * EXIF összefoglaló formázása
 */
export function formatExifSummary(exif: ExifData | null): string {
  if (!exif) return "";

  const parts: string[] = [];

  if (exif.iso) parts.push(`ISO ${exif.iso}`);
  if (exif.shutter_speed) parts.push(exif.shutter_speed);
  if (exif.aperture) parts.push(`f/${exif.aperture.toFixed(1)}`);
  if (exif.focal_length) parts.push(`${Math.round(exif.focal_length)}mm`);

  return parts.join(" | ");
}

# VisionSelect AI - Részletes Megvalósítási Terv

## Áttekintés

Ez a dokumentum a VisionSelect AI projekt teljes megvalósítási tervét tartalmazza, fázisokra és azokon belül konkrét megvalósítandó funkciókra bontva. Minden funkció tartalmazza a technikai követelményeket, bemeneti/kimeneti specifikációkat és a becsült komplexitást.

---

## 🏗️ I. FÁZIS: Infrastruktúra és Data Pipeline (1-3. hét)

### Célkitűzés

Az alkalmazás vázának megépítése, amely képes nagy adatmennyiséget kezelni lefagyás nélkül.

---

### 1.1 Tauri Projekt Struktúra Felállítása

#### 1.1.1 `init_tauri_project()`

**Leírás**: A Tauri keretrendszer alapkonfigurációjának beállítása.

| Paraméter       | Típus  | Leírás                                |
| --------------- | ------ | ------------------------------------- |
| `app_name`      | String | Az alkalmazás neve: "VisionSelect AI" |
| `window_config` | Object | Ablak méret, cím, téma                |

**Felelősségek**:

- [x] Tauri CLI inicializálás
- [ ] `tauri.conf.json` testreszabása
- [ ] Frontend-backend kommunikációs csatorna beállítása
- [ ] Hot-reload fejlesztői környezet konfigurálása

**Kimenet**: Működő fejlesztői környezet React frontendel és Rust backenddel.

---

#### 1.1.2 `setup_build_pipeline()`

**Leírás**: Build és package folyamat konfigurálása.

**Felelősségek**:

- [ ] Cargo.toml függőségek definiálása
- [ ] Release optimalizációk beállítása
- [ ] Platform-specifikus build targetek (Windows/macOS)

---

### 1.2 Fájlrendszer Integráció (Rust Backend)

#### 1.2.1 `scan_directory(path: &Path) -> Vec<RawFile>`

**Leírás**: Rekurzívan bejárja a megadott mappát és összegyűjti a RAW fájlokat.

| Paraméter    | Típus       | Leírás                            |
| ------------ | ----------- | --------------------------------- |
| `path`       | &Path       | A szkennelendő mappa útvonala     |
| `recursive`  | bool        | Almappák bejárása                 |
| `extensions` | Vec<String> | Támogatott kiterjesztések listája |

**Támogatott formátumok**:

```
.ARW (Sony), .CR2/.CR3 (Canon), .NEF (Nikon),
.RAF (Fujifilm), .DNG (Adobe), .ORF (Olympus), .RW2 (Panasonic)
```

**Visszatérési érték**:

```rust
struct RawFile {
    path: PathBuf,
    filename: String,
    extension: String,
    size_bytes: u64,
    modified_at: DateTime<Utc>,
}
```

**Komplexitás**: Közepes

---

#### 1.2.2 `extract_exif_data(file: &RawFile) -> ExifData`

**Leírás**: EXIF metaadatok kinyerése a RAW fájlokból.

**Kinyerendő adatok**:

```rust
struct ExifData {
    // Felvételi adatok
    iso: u32,
    shutter_speed: String,      // pl. "1/250"
    aperture: f32,              // pl. 2.8
    focal_length: f32,          // mm

    // Időbélyeg
    capture_date: DateTime<Utc>,

    // Kamera info
    camera_make: String,
    camera_model: String,
    lens_model: Option<String>,

    // Egyéb
    orientation: u8,
    gps_coordinates: Option<(f64, f64)>,
}
```

**Függőség**: `kamadak-exif` crate

---

#### 1.2.3 `file_watcher_service() -> FileWatcher`

**Leírás**: Valós idejű fájlrendszer-figyelő szolgáltatás.

**Események**:

- `FileCreated` - Új fájl hozzáadva
- `FileModified` - Fájl módosítva
- `FileDeleted` - Fájl törölve

**Megvalósítás**: `notify` crate használata

---

### 1.3 RAW Thumbnail Engine

#### 1.3.1 `extract_thumbnail(file: &RawFile) -> Result<Image, Error>`

**Leírás**: A RAW fájlba ágyazott preview kép kinyerése.

| Paraméter     | Típus      | Leírás                         |
| ------------- | ---------- | ------------------------------ |
| `file`        | &RawFile   | A RAW fájl referenciája        |
| `target_size` | (u32, u32) | Célméret (szélesség, magasság) |

**Prioritási sorrend**:

1. Teljes méretű beágyazott JPEG (ha elérhető)
2. Közepes méretű preview
3. Thumbnail (utolsó lehetőség)

**Függőség**: `libraw-rs` Rust binding

**Teljesítmény cél**: < 50ms / kép

---

#### 1.3.2 `cache_thumbnail(image: &Image, cache_path: &Path)`

**Leírás**: Az előnézeti képek cachelése a gyorsabb újratöltéshez.

**Cache stratégia**:

```rust
struct ThumbnailCache {
    path: PathBuf,
    format: ImageFormat::WebP,  // Kis méret, jó minőség
    quality: 85,
    max_dimension: 1024,
}
```

---

#### 1.3.3 `generate_blurhash(image: &Image) -> String`

**Leírás**: BlurHash generálása placeholder-hez lazy loading során.

**Kimenet**: Base83-kódolt string (pl. "LEHV6nWB2yk8pyo0adR\*.7kCMdnj")

---

### 1.4 Adatbázis Réteg (SQLite)

#### 1.4.1 `init_database() -> Database`

**Leírás**: SQLite adatbázis inicializálása és séma létrehozása.

**Séma definíció**:

```sql
-- Képek táblája
CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT UNIQUE NOT NULL,
    filename TEXT NOT NULL,
    extension TEXT NOT NULL,
    file_size INTEGER,
    file_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME
);

-- EXIF adatok
CREATE TABLE exif_data (
    image_id INTEGER PRIMARY KEY,
    iso INTEGER,
    shutter_speed TEXT,
    aperture REAL,
    focal_length REAL,
    capture_date DATETIME,
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,
    FOREIGN KEY (image_id) REFERENCES images(id)
);

-- AI elemzési eredmények
CREATE TABLE ai_scores (
    image_id INTEGER PRIMARY KEY,
    category TEXT,               -- 'portrait', 'landscape', etc.
    overall_score REAL,
    sharpness_score REAL,
    exposure_score REAL,
    composition_score REAL,
    face_score REAL,             -- NULL ha nincs arc
    analyzed_at DATETIME,
    model_version TEXT,
    FOREIGN KEY (image_id) REFERENCES images(id)
);

-- Csoportok (sorozatfelvételek)
CREATE TABLE image_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE group_members (
    group_id INTEGER,
    image_id INTEGER,
    is_winner BOOLEAN DEFAULT FALSE,
    rank INTEGER,
    PRIMARY KEY (group_id, image_id),
    FOREIGN KEY (group_id) REFERENCES image_groups(id),
    FOREIGN KEY (image_id) REFERENCES images(id)
);

-- Indexek a gyors kereséshez
CREATE INDEX idx_images_path ON images(file_path);
CREATE INDEX idx_exif_date ON exif_data(capture_date);
CREATE INDEX idx_scores_overall ON ai_scores(overall_score DESC);
```

---

#### 1.4.2 `upsert_image(image: &ImageRecord) -> Result<i64, Error>`

**Leírás**: Kép rekord beszúrása vagy frissítése.

---

#### 1.4.3 `get_images_by_folder(folder: &Path) -> Vec<ImageRecord>`

**Leírás**: Adott mappához tartozó képek lekérdezése.

---

#### 1.4.4 `get_images_needing_analysis() -> Vec<ImageRecord>`

**Leírás**: Még nem elemzett képek lekérdezése.

---

### 1.5 Frontend Alapok (React)

#### 1.5.1 `<ImageGrid />` komponens

**Leírás**: Virtualizált képrács nagy mennyiségű kép megjelenítéséhez.

**Props**:

```typescript
interface ImageGridProps {
  images: ImageRecord[];
  columnCount: number;
  onImageSelect: (id: number) => void;
  onImageDoubleClick: (id: number) => void;
  selectedIds: Set<number>;
}
```

**Technikai követelmények**:

- [ ] Virtualizált lista (`react-window` vagy `@tanstack/virtual`)
- [ ] Lazy loading (csak látható képek betöltése)
- [ ] Pre-fetching (következő 20 kép előre betöltése)
- [ ] BlurHash placeholder-ek

---

#### 1.5.2 `useFolderSelection()` hook

**Leírás**: Mappa kiválasztó dialógus Tauri API-val.

```typescript
function useFolderSelection() {
    const selectFolder = async (): Promise<string | null>;
    const isLoading: boolean;
    const error: Error | null;
}
```

---

#### 1.5.3 `useImageLoader()` hook

**Leírás**: Képek betöltésének kezelése Rust backend-ről.

```typescript
function useImageLoader(imagePaths: string[]) {
  const loadedImages: Map<string, ImageData>;
  const loadingStates: Map<string, "loading" | "loaded" | "error">;
  const progress: number; // 0-100
}
```

---

## 🧠 II. FÁZIS: AI Mag és Intelligens Csoportosítás (4-8. hét)

### Célkitűzés

A projekt "agya" - az intelligens elemzési és döntéshozatali rendszer megépítése.

---

### 2.1 Kategória Osztályozó (Classifier)

#### 2.1.1 `classify_image(image: &Image) -> ImageCategory`

**Leírás**: Meghatározza a kép típusát a megfelelő szakértő modell kiválasztásához.

**Kategóriák**:

```rust
enum ImageCategory {
    Portrait,       // Emberek, arcok
    Landscape,      // Tájképek
    Event,          // Esküvő, rendezvény
    Product,        // Tárgyfotó
    Macro,          // Makró fotók
    Wildlife,       // Állatok
    Architecture,   // Épületek
    Street,         // Street photography
    Unknown,        // Nem besorolható
}
```

**Modell**: MobileNetV3 (ONNX formátum)

- Input: 224x224 RGB kép
- Output: Softmax valószínűségek (9 osztály)
- Méret: ~15MB

---

#### 2.1.2 `load_classifier_model() -> OnnxModel`

**Leírás**: Az osztályozó modell betöltése ONNX Runtime-mal.

**Konfiguráció**:

```rust
struct ModelConfig {
    model_path: PathBuf,
    use_gpu: bool,
    execution_provider: ExecutionProvider, // CUDA, DirectML, CPU
    thread_count: usize,
}
```

---

### 2.2 Szakértő Modellek (Expert Models)

#### 2.2.1 Portré Szakértő Modul

##### `detect_faces(image: &Image) -> Vec<Face>`

**Leírás**: Arcok detektálása és landmark pontok kinyerése.

```rust
struct Face {
    bounding_box: Rectangle,
    confidence: f32,
    landmarks: FaceLandmarks,
}

struct FaceLandmarks {
    left_eye: Point2D,
    right_eye: Point2D,
    nose_tip: Point2D,
    mouth_left: Point2D,
    mouth_right: Point2D,
}
```

**Modell**: RetinaFace (ONNX)

---

##### `measure_eye_sharpness(image: &Image, face: &Face) -> f32`

**Leírás**: A szemek élességének mérése (a portréfotózás legkritikusabb faktora).

**Algoritmus**:

1. Szem régiók kivágása (ROI)
2. Laplacian variance számítása
3. Normalizálás 0-1 skálára

**Visszatérési érték**: 0.0 (életlenség) - 1.0 (tökéletes élesség)

---

##### `detect_blink(image: &Image, face: &Face) -> BlinkResult`

**Leírás**: Pislogás detektálása (Eye Aspect Ratio alapján).

```rust
struct BlinkResult {
    is_blinking: bool,
    left_eye_open_ratio: f32,  // 0.0 = zárt, 1.0 = nyitott
    right_eye_open_ratio: f32,
    confidence: f32,
}
```

**Küszöbérték**: EAR < 0.2 → pislogás

---

##### `detect_smile(image: &Image, face: &Face) -> SmileResult`

**Leírás**: Mosoly detektálása és intenzitása.

```rust
struct SmileResult {
    is_smiling: bool,
    smile_intensity: f32,  // 0.0 - 1.0
    is_genuine: bool,      // Duchenne smile detektálás
}
```

---

##### `analyze_portrait(image: &Image) -> PortraitAnalysis`

**Leírás**: Teljes portré elemzés, minden metrika aggregálása.

```rust
struct PortraitAnalysis {
    faces: Vec<FaceAnalysis>,
    main_subject: Option<usize>,  // Fő alany indexe
    group_photo: bool,
    overall_score: f32,
}

struct FaceAnalysis {
    face: Face,
    eye_sharpness: f32,
    blink_result: BlinkResult,
    smile_result: SmileResult,
    expression_score: f32,
}
```

---

#### 2.2.2 Általános Minőségi Szakértő

##### `measure_sharpness(image: &Image) -> SharpnessAnalysis`

**Leírás**: Globális és lokális élesség mérése.

```rust
struct SharpnessAnalysis {
    global_score: f32,           // Teljes kép élessége
    focus_regions: Vec<FocusRegion>, // Éles régiók
    sharpest_point: Point2D,     // Legélesebb pont
    blur_type: Option<BlurType>, // motion/out-of-focus/none
}

enum BlurType {
    MotionBlur,     // Mozgás okozta
    OutOfFocus,     // Rossz fókusz
    None,
}
```

**Algoritmus**: Laplacian Variance + FFT analízis

---

##### `analyze_exposure(image: &Image) -> ExposureAnalysis`

**Leírás**: Expozíció elemzése hisztogram alapján.

```rust
struct ExposureAnalysis {
    is_properly_exposed: bool,
    overexposed_percentage: f32,  // Kiégett területek %
    underexposed_percentage: f32, // Fekete területek %
    dynamic_range_score: f32,
    histogram: [u32; 256],
}
```

**Küszöbértékek**:

- Túlexponált: > 5% pixel értéke 250+
- Alulexponált: > 10% pixel értéke < 10

---

##### `analyze_composition(image: &Image) -> CompositionScore`

**Leírás**: Opcionális kompozíciós elemzés.

**Elemzett szempontok**:

- Harmadolás szabálya
- Szimmetria
- Vezető vonalak
- Térkitöltés

---

### 2.3 Csoportosító Algoritmus

#### 2.3.1 `group_images_by_burst(images: Vec<&Image>) -> Vec<ImageGroup>`

**Leírás**: Sorozatfelvételek automatikus csoportosítása.

**Csoportosítási kritériumok**:

1. **Időbélyeg**: < 3 másodperc különbség egymás között
2. **Fókusztávolság**: Azonos (± 5mm)
3. **Helyszín**: Azonos GPS koordináták (ha elérhető)

```rust
struct ImageGroup {
    id: u64,
    images: Vec<ImageId>,
    group_type: GroupType,
    capture_start: DateTime<Utc>,
    capture_end: DateTime<Utc>,
}

enum GroupType {
    BurstMode,      // Sorozatfelvétel
    Bracket,        // Expozíció bracket
    TimeLapse,      // Time-lapse szekvencia
    Manual,         // Felhasználó által létrehozott
}
```

---

#### 2.3.2 `calculate_visual_similarity(img1: &Image, img2: &Image) -> f32`

**Leírás**: Vizuális hasonlóság mérése perceptuális hash-sel.

**Módszer**: pHash (perceptual hash) + Hamming-távolság

**Kimenet**: 0.0 (teljesen különböző) - 1.0 (azonos)

---

#### 2.3.3 `merge_groups(groups: Vec<ImageGroup>, threshold: f32) -> Vec<ImageGroup>`

**Leírás**: Hasonló csoportok összevonása.

---

### 2.4 Döntéshozó Logika (Decision Engine)

#### 2.4.1 `rank_images_in_group(group: &ImageGroup) -> RankedGroup`

**Leírás**: A csoport képeinek rangsorolása AI pontszámok alapján.

```rust
struct RankedGroup {
    group: ImageGroup,
    winner: ImageId,
    runners_up: Vec<ImageId>,  // Top 3 alternatíva
    scores: HashMap<ImageId, DetailedScore>,
}

struct DetailedScore {
    overall: f32,
    sharpness: f32,
    exposure: f32,
    face_score: Option<f32>,
    category_specific: HashMap<String, f32>,
}
```

---

#### 2.4.2 `apply_scoring_weights(category: ImageCategory) -> ScoringWeights`

**Leírás**: Kategória-specifikus súlyozás alkalmazása.

**Súlyozási példák**:

```rust
// Portré
ScoringWeights {
    sharpness: 0.25,
    exposure: 0.15,
    face_sharpness: 0.35,
    expression: 0.20,
    composition: 0.05,
}

// Táj
ScoringWeights {
    sharpness: 0.40,
    exposure: 0.25,
    composition: 0.25,
    dynamic_range: 0.10,
}
```

---

#### 2.4.3 `resolve_ties(candidates: Vec<&ScoredImage>) -> ImageId`

**Leírás**: Döntetlen feloldása másodlagos kritériumok alapján.

**Döntési sorrend**:

1. Legélesebb szem (portréknál)
2. Legjobb expozíció
3. Legkisebb ISO (zajcsökkentés)
4. Legkorábbi felvételi időpont

---

### 2.5 AI Futtatási Infrastruktúra

#### 2.5.1 `AIService` struct

**Leírás**: Központi AI szolgáltatás az összes modell kezeléséhez.

```rust
struct AIService {
    classifier: ClassifierModel,
    portrait_expert: PortraitExpert,
    quality_expert: QualityExpert,
    executor: ThreadPool,
    gpu_available: bool,
}

impl AIService {
    fn analyze_batch(&self, images: Vec<Image>) -> Vec<AnalysisResult>;
    fn analyze_single(&self, image: &Image) -> AnalysisResult;
    fn get_progress(&self) -> ProgressInfo;
}
```

---

#### 2.5.2 `AnalysisQueue`

**Leírás**: Háttérben futó elemzési sor.

```rust
struct AnalysisQueue {
    pending: VecDeque<ImageId>,
    processing: HashSet<ImageId>,
    completed: HashMap<ImageId, AnalysisResult>,

    fn enqueue(&mut self, images: Vec<ImageId>);
    fn get_progress(&self) -> (usize, usize); // (completed, total)
    fn pause(&mut self);
    fn resume(&mut self);
}
```

---

## 🎨 III. FÁZIS: Felhasználói Élmény (UX) és Interfész (9-12. hét)

### Célkitűzés

A legjobb AI is haszontalan, ha a kezelése nehézkes - itt finomítjuk a React felületet.

---

### 3.1 Döntési Nézet (Decision View)

#### 3.1.1 `<DecisionView />` komponens

**Leírás**: A fő válogatási interfész, ahol a felhasználó áttekinti az AI javaslatait.

**Layout**:

```
┌─────────────────────────────────────────────────────────┐
│                    GYŐZTES KÉP                          │
│                    (nagy méret)                         │
│                                                         │
├────────────┬────────────┬────────────┬──────────────────┤
│ Alternatív │ Alternatív │ Alternatív │   Metadata       │
│     #1     │     #2     │     #3     │   Panel          │
└────────────┴────────────┴────────────┴──────────────────┘
```

**Props**:

```typescript
interface DecisionViewProps {
  group: ImageGroup;
  winner: ImageRecord;
  alternatives: ImageRecord[];
  scores: Map<number, DetailedScore>;
  onApprove: () => void;
  onOverride: (newWinnerId: number) => void;
  onReject: () => void;
  onSkip: () => void;
}
```

---

#### 3.1.2 `<ScoreOverlay />` komponens

**Leírás**: AI pontszámok megjelenítése a képeken.

```typescript
interface ScoreOverlayProps {
  score: DetailedScore;
  showDetails: boolean;
  position: "top-left" | "top-right" | "bottom";
}
```

---

#### 3.1.3 `<ComparisonSlider />` komponens

**Leírás**: Két kép összehasonlítása csúszkával.

---

### 3.2 Smart Zoom Funkció

#### 3.2.1 `useSmartZoom()` hook

**Leírás**: Szinkronizált nagyítás implementálása.

```typescript
function useSmartZoom(images: ImageRecord[]) {
  const zoomLevel: number; // 1.0 - 10.0
  const panOffset: { x: number; y: number };
  const activeImage: number;

  const zoomToFace: (faceId: number) => void;
  const syncZoom: () => void; // Összes kép ugyanoda zoomol
  const resetZoom: () => void;
}
```

**Működés**:

1. Felhasználó nagyít az egyik képen
2. Automatikusan kiszámítjuk a megfelelő pozíciót a többi képen
3. Arc-alapú igazítás (ha vannak arcok)

---

#### 3.2.2 `<ZoomableImage />` komponens

**Leírás**: Nagyítható és görgethet kép megjelenítő.

**Funkciók**:

- Görgetés (scroll) zoom
- Dupla kattintás 100%-os nézet
- Drag-to-pan
- Arc-indikátorok overlay

---

### 3.3 Billentyűzet Vezérlés

#### 3.3.1 `useKeyboardNavigation()` hook

**Leírás**: Teljes billentyűzet-vezérelhetőség.

**Billentyű-kiosztás**:

```typescript
const keyBindings = {
  // Navigáció
  ArrowRight: "nextGroup",
  ArrowLeft: "previousGroup",
  ArrowUp: "previousImage",
  ArrowDown: "nextImage",

  // Műveletek
  Space: "approveWinner",
  Enter: "approveWinner",
  "1": "selectAlternative1",
  "2": "selectAlternative2",
  "3": "selectAlternative3",
  x: "rejectCurrent",
  Delete: "rejectCurrent",
  s: "skipGroup",

  // Zoom
  "+": "zoomIn",
  "-": "zoomOut",
  "0": "resetZoom",
  f: "zoomToFace",

  // Panel
  i: "toggleInfo",
  h: "toggleHistory",
};
```

---

#### 3.3.2 `<KeyboardShortcutsPanel />` komponens

**Leírás**: Billentyűparancsok súgó panel.

---

### 3.4 Valós Idejű Visszajelzés

#### 3.4.1 `<AnalysisProgress />` komponens

**Leírás**: AI elemzési folyamat vizualizálása.

```typescript
interface AnalysisProgressProps {
  total: number;
  completed: number;
  currentImage: string;
  estimatedTimeRemaining: number; // másodpercben
  status: "analyzing" | "paused" | "completed" | "error";
}
```

---

#### 3.4.2 `useAnalysisStatus()` hook

**Leírás**: Backend elemzési állapot lekérdezése.

```typescript
function useAnalysisStatus() {
  const progress: AnalysisProgress;
  const pause: () => void;
  const resume: () => void;
  const cancel: () => void;
}
```

---

### 3.5 UI/UX Finomhangolás

#### 3.5.1 Sötét Mód

**Leírás**: Fotós-barát sötét téma.

**Szín paletta**:

```css
:root {
  --bg-primary: #1a1a1a;
  --bg-secondary: #242424;
  --bg-tertiary: #2d2d2d;
  --text-primary: #e0e0e0;
  --text-secondary: #888888;
  --accent: #4caf50;
  --warning: #ffc107;
  --error: #f44336;
}
```

---

#### 3.5.2 `<Sidebar />` komponens

**Leírás**: Navigációs oldalsáv.

**Szekciók**:

- Mappák fa nézet
- Szűrők (kategória, értékelés, dátum)
- Gyors statisztikák

---

#### 3.5.3 Érintés támogatás (Tablet)

**Leírás**: Touch gesztusok implementálása.

**Gesztusok**:

- Pinch-to-zoom
- Swipe navigáció
- Long-press kontextus menü

---

## 📤 IV. FÁZIS: Export és Integráció (13-15. hét)

### Célkitűzés

A munkafolyamat lezárása, hogy a fotós folytatni tudja a munkát a megszokott szoftvereiben.

---

### 4.1 XMP Sidecar Generátor

#### 4.1.1 `generate_xmp_sidecar(image: &ImageRecord, rating: Rating) -> String`

**Leírás**: XMP sidecar fájl generálása Lightroom/Capture One kompatibilitással.

**XMP struktúra**:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/"
            xmp:Rating="5"
            xmp:Label="Green">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
```

**Rating mapping**:

```rust
enum Rating {
    Winner => 5,      // 5 csillag + Zöld címke
    Alternative => 3, // 3 csillag
    Rejected => -1,   // Elutasított flag
    Unrated => 0,
}
```

---

#### 4.1.2 `write_xmp_file(path: &Path, content: &str) -> Result<(), Error>`

**Leírás**: XMP fájl írása a lemezre.

**Fájlnév konvenció**: `IMG_1234.ARW` → `IMG_1234.xmp`

---

#### 4.1.3 `batch_export_xmp(images: Vec<&ImageRecord>) -> ExportResult`

**Leírás**: Tömeges XMP export.

```rust
struct ExportResult {
    successful: usize,
    failed: Vec<(PathBuf, Error)>,
    duration: Duration,
}
```

---

### 4.2 Fájlkezelési Műveletek

#### 4.2.1 `move_to_rejected(images: Vec<&ImageRecord>, target_folder: &Path)`

**Leírás**: Elutasított képek áthelyezése.

**Opciók**:

- Áthelyezés "Rejected" mappába
- Törlésre jelölés (nem véglegesen töröl)
- XMP-ben Rejected flag beállítása

---

#### 4.2.2 `organize_by_rating(source: &Path, destination: &Path)`

**Leírás**: Képek rendezése értékelés alapján mappákba.

**Struktúra**:

```
destination/
├── 5_stars/
├── 4_stars/
├── 3_stars/
├── unrated/
└── rejected/
```

---

#### 4.2.3 `generate_session_report(session: &CullingSession) -> Report`

**Leírás**: Válogatási munkamenet összefoglaló.

```rust
struct Report {
    total_images: usize,
    groups_reviewed: usize,
    winners_selected: usize,
    rejected_count: usize,
    time_spent: Duration,
    ai_accuracy: f32,  // Hányszor fogadta el a javaslatot
}
```

---

### 4.3 Teljesítmény Optimalizálás

#### 4.3.1 Multi-threading Konfigurálás

**Leírás**: Párhuzamos feldolgozás finomhangolása.

```rust
struct ThreadingConfig {
    io_threads: usize,       // Fájl műveletek
    ai_threads: usize,       // AI inference
    ui_priority: bool,       // UI thread elsőbbsége
    max_memory_mb: usize,    // RAM limit
}
```

---

#### 4.3.2 GPU Gyorsítás

**Leírás**: ONNX Runtime GPU backend használata.

**Támogatott backend-ek**:

- CUDA (NVIDIA)
- DirectML (Windows, AMD/Intel)
- CoreML (macOS)

---

#### 4.3.3 Memória Menedzsment

**Leírás**: Automatikus memória felszabadítás.

**Stratégia**:

- LRU cache a betöltött képekhez
- Automatikus thumbnail downscale alacsony RAM esetén
- Explicit GC hívások kritikus pontokon

---

### 4.4 Telepítő és Disztribúció

#### 4.4.1 Windows Telepítő

**Leírás**: NSIS vagy WiX telepítő készítése.

**Tartalmazza**:

- Főalkalmazás
- AI modellek
- Visual C++ Redistributable
- Asztali ikon és Start menü bejegyzés

---

#### 4.4.2 macOS Bundle

**Leírás**: .app bundle és DMG készítése.

**Követelmények**:

- Code signing
- Notarization
- Universal Binary (Intel + Apple Silicon)

---

#### 4.4.3 Auto-Update Rendszer

**Leírás**: Automatikus frissítések kezelése.

```rust
struct UpdateConfig {
    check_interval_hours: u32,
    update_url: String,
    auto_install: bool,
}
```

---

## 📊 Összefoglaló Táblázat

| Fázis | Időtartam  | Fő Komponensek            | Kritikus Függőségek     |
| ----- | ---------- | ------------------------- | ----------------------- |
| I.    | 1-3. hét   | Tauri, SQLite, LibRaw     | Rust toolchain, Node.js |
| II.   | 4-8. hét   | ONNX Runtime, AI modellek | GPU driver (opcionális) |
| III.  | 9-12. hét  | React komponensek         | I. és II. fázis         |
| IV.   | 13-15. hét | XMP, telepítők            | Teljes alkalmazás       |

---

## Mérföldkövek

1. **3. hét vége**: Működő fájlböngésző és thumbnail megjelenítés
2. **8. hét vége**: AI működik, képes rangsorolni egy sorozatot
3. **12. hét vége**: Teljes UI, használható béta verzió
4. **15. hét vége**: Kiadásra kész v1.0

---

## Kockázatkezelés

| Kockázat                    | Valószínűség | Hatás    | Megoldás                        |
| --------------------------- | ------------ | -------- | ------------------------------- |
| Lassú AI feldolgozás        | Közepes      | Magas    | GPU gyorsítás, batch processing |
| RAW formátum kompatibilitás | Alacsony     | Magas    | LibRaw rendszeres frissítése    |
| Memória túlfogyasztás       | Közepes      | Közepes  | Lazy loading, LRU cache         |
| macOS code signing          | Alacsony     | Alacsony | Apple Developer fiók beszerzése |

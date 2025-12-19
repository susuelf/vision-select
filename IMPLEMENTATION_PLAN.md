# VisionSelect AI: Részletes Megvalósítási Terv

## 1. Fázis: Az Infrastruktúra és a "Data Pipeline" (1-3. hét)

Ebben a szakaszban építjük meg az applikáció vázát, ami képes hatalmas adatmennyiséget kezelni lefagyás nélkül.

- **Fájlrendszer Integráció (Rust)**: Olyan háttérfolyamat készítése, amely képes beszkennelni egy mappát, kiszűrni a RAW kiterjesztéseket (.ARW, .CR2, .NEF stb.), és kinyerni az EXIF adatokat (ISO, záridő, dátum).
- **RAW Thumbnail Engine**: A LibRaw segítségével a beágyazott preview képek kinyerése.
- **Kihívás**: A képeket nem szabad egyszerre a memóriába tölteni. Egy "Lazy Loading" listát kell építeni a frontendnél.
- **Adatbázis séma**: SQLite beállítása a képek metaadatainak és az AI pontszámoknak a tárolására (hogy ne kelljen minden indításkor újrakalkulálni).

## 2. Fázis: Az AI Mag és az Intelligens Csoportosítás (4-8. hét)

Ez a projekt "agya". Itt dől el, hogy a szoftver valóban megkönnyíti-e a fotós dolgát.

- **Kategória Osztályozó (Classifier)**: Egy könnyű modell (pl. MobileNet), ami eldönti, mi van a képen.
- **Csoportosító Algoritmus**: Időbélyeg (timestamp) és vizuális hasonlóság alapján a sorozatfelvételek egy blokkba rendezése.
- **Szakértő Modellek Integrálása**:
  - **Portré szakértő**: Arcdetektálás, szem-élesség mérése, pislogás felismerése.
  - **Általános szakértő**: Élesség (Laplacian variance) és expozíció elemzés.
- **A "Döntéshozó" Logika**: Egy algoritmus, ami a csoporton belül súlyozza a pontszámokat (pl. ha mindenki mosolyog, az élesség dönt; ha senki sem mosolyog, a legkevésbé rosszat választja).

## 3. Fázis: A Felhasználói Élmény (UX) és Interfész (9-12. hét)

A legjobb AI is haszontalan, ha a kezelése nehézkes. Itt finomítjuk a React felületet.

- **Ellenőrző Nézet (Decision View)**: A felhasználó elé tárjuk a "Győztest" és a 3 alternatívát.
- **Smart Zoom**: Amikor a felhasználó a győztes kép arcára közelít, a másik 3 kép is automatikusan ugyanoda ugrik, hogy azonnali legyen az összehasonlítás.
- **Billentyűzet-fókuszált vezérlés**: Space a jóváhagyáshoz, 1-4 a győztes felülbírálásához, X az elutasításhoz.
- **Valós idejű visszajelzés**: Progress bar az AI elemzési folyamatáról.

## 4. Fázis: Export és Integráció (13-15. hét)

A munkafolyamat lezárása, hogy a fotós folytatni tudja a munkát a megszokott szoftvereiben.

- **XMP Sidecar Generátor**: Olyan fájlok írása, amelyeket a Lightroom és a Capture One automatikusan felismer (Rating és Color Label tagek).
- **Fájlkezelési műveletek**: A "Rejected" képek áthelyezése egy külön mappába vagy törlésre jelölése.
- **Teljesítmény optimalizálás**: Multi-threading finomhangolása, hogy az elemzés ne akadályozza a UI-t.

---

# VisionSelect AI - Projekt Specifikáció és Megvalósítási Terv

## 1. Projekt Összefoglaló

A VisionSelect AI egy nagy teljesítményű, mesterséges intelligenciával támogatott asztali alkalmazás, amely a fotósok culling (válogatási) munkafolyamatát automatizálja. A szoftver célja, hogy a több ezer elkészült nyers (RAW) kép közül kiválassza a technikailag és esztétikailag legjobb felvételeket, drasztikusan csökkentve az utómunkára fordított időt.

## 2. Termékfunkciók (Product Features)

### 2.1. Adatkezelés és Megjelenítés

- **Ultra-gyors RAW Motor**: A RAW fájlokba ágyazott teljes méretű preview képek azonnali megjelenítése.
- **Lazy Loading & Pre-fetching**: Csak a látható képek betöltése, miközben a háttérben az algoritmus előre olvassa a következő fájlokat.
- **Minden formátum támogatása**: Canon (.CR2, .CR3), Nikon (.NEF), Sony (.ARW), Fujifilm (.RAF) és univerzális .DNG támogatás.

### 2.2. AI-alapú Analitika (Multi-Expert Engine)

- **Intelligens Osztályozó (Classifier)**: A képek kategóriákba sorolása (Portré, Tájkép, Esemény, Tárgy, Makró).
- **Szakértői Modellek (Expert Models)**:
  - **Portré Modul**: Arcfelismerés, szem-élesség mérése, pislogás- és mosolyvizsgálat.
  - **Élesség Analízis**: Objektív kontrasztmérés (Laplacian variance) a fókuszhíba kiszűrésére.
  - **Expozíció Ellenőrzés**: Túl- vagy alulexponált képek jelölése.
- **Helyi Feldolgozás**: Minden AI folyamat a felhasználó saját GPU-ján/CPU-ján fut (adatvédelmi és sebességi okokból).

### 2.3. Intelligens Munkafolyamat (Workflow)

- **Automatikus Csoportosítás**: Sorozatfelvételek felismerése időbélyeg és vizuális hasonlóság alapján.
- **AI Javaslat (Winner & Runners-up)**: Csoportonként 1 győztes kép és a 3 legjobb alternatíva automatikus felkínálása.
- **Smart Sync Zoom**: Több kép egyidejű nagyítása az összehasonlításhoz, arcra fókuszált automatikus igazítással.

### 2.4. Integráció és Export

- **XMP Sidecar támogatás**: Csillagok és színes címkék mentése külső fájlba (Lightroom/Capture One kompatibilitás).
- **Fájlkezelés**: Elutasított képek automatikus áthelyezése "Rejected" mappába.

## 3. Technológiai Stack

- **Backend**: Rust (Tauri framework)
- **Frontend**: React.js + Tailwind CSS
- **AI Runtime**: ONNX Runtime (GPU gyorsítással)
- **Adatbázis**: SQLite (lokális metaadat tárolás)
- **Képfeldolgozás**: LibRaw & OpenCV (Rust bindings)

## 4. Megvalósítási Ütemterv (Roadmap)

### I. Fázis: Alapok és Adatkezelés (1-3. hét)

- [ ] Tauri projektstruktúra felállítása.
- [ ] Rust alapú fájlböngésző és RAW-thumbnail kinyerő motor.
- [ ] SQLite adatbázis séma implementálása.
- [ ] Alapszintű képrács (grid view) megjelenítése React-ben.

### II. Fázis: AI Mag és Logika (4-8. hét)

- [ ] Képkategória-osztályozó integrálása.
- [ ] Élességmérő és arcfelismerő modulok beépítése.
- [ ] Idő- és tartalom alapú csoportosító algoritmus fejlesztése.
- [ ] Mérföldkő: Működő prototípus, amely képes rangsorolni egy sorozatot.

### III. Fázis: Felhasználói Interfész (9-12. hét)

- [ ] "Döntési Nézet" (Győztes + 3 alternatíva) fejlesztése.
- [ ] Smart Zoom funkció és billentyűzet-vezérelt válogatás.
- [ ] UI/UX finomhangolás (Sötét mód, performáns görgetés).

### IV. Fázis: Finiselés és Export (13-15. hét)

- [ ] XMP generáló modul fejlesztése.
- [ ] Batch (tömeges) feldolgozási funkciók.
- [ ] Telepítőcsomagok készítése (Windows/macOS).
- [ ] Mérföldkő: Végleges, kiadásra kész v1.0 verzió.

## 5. Kockázatok és Megoldások

- **Lassú feldolgozás**: Megoldás: Multi-threading és GPU gyorsítás használata.
- **AI tévedések**: Megoldás: Az AI csak javaslatot tesz, a végső döntés mindig a fotósé (egyszerű felülbírálati lehetőség).
- **RAM igény**: Megoldás: Csak az aktuálisan nézett képek teljes felbontású tárolása a memóriában.

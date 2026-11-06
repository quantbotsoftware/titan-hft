# 🔍 MARKET DISCOVERY — Manual Complet de Utilizare

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


**Versiune:** V8 FMP-Enriched (Discovery V8 Engine)  
**Ultima actualizare:** 2 Aprilie 2026 (Sesiunea 64 — S15: Tiered Scanner Top-K)  
**Locație în UI:** Sidebar stâng → Secțiunea 8 (colapsabilă)  
**Surse de date:** Alpaca Markets (prețuri, volume) + Financial Modeling Prep (float, market cap, insider trading, earnings, price targets)

---

## CUPRINS

1. [Ce este Market Discovery?](#1-ce-este-market-discovery)
2. [Unde găsesc Market Discovery în interfață?](#2-unde-gasesc-market-discovery-in-interfata)
3. [Parametrii de Filtrare (Câmpuri de intrare)](#3-parametrii-de-filtrare)
4. [Butoanele de Acțiune](#4-butoanele-de-actiune)
5. [Presetări (Quick Presets)](#5-presetari-quick-presets)
6. [Score Mode — Moduri de Scoring](#6-score-mode)
7. [Rezultatele Scanării](#7-rezultatele-scanarii)
8. [Coloana Δ (Delta) — DIFF ALERT](#8-coloana-delta-diff-alert)
9. [NEW MOVER — Alerte pentru Simboluri Noi](#9-new-mover-alerte)
10. [Auto Scan — Scanare Automată](#10-auto-scan)
11. [Ross Cameron Preset](#11-ross-cameron-preset)
12. [Inject în Watchlist / Signal CFG](#12-inject-in-watchlist)
13. [Cum Funcționează Scanneru pe Interior (Backend)](#13-cum-functioneaza-backend)
14. [FMP Data Integration — Date Fundamentale GRATUITE](#14-fmp-data-integration)
15. [Scenarii Practice de Utilizare](#15-scenarii-practice)
16. [Sfaturi de Expert pentru Începători](#16-sfaturi-expert)
17. [Probleme Frecvente și Soluții](#17-probleme-frecvente)
18. [Glosar de Termeni](#18-glosar)

---

## 1. CE ESTE MARKET DISCOVERY?

Market Discovery este **scanneru de piață** integrat în Titan HFT. El scanează **toate acțiunile listate pe bursele americane** (NYSE, NASDAQ, AMEX, ARCA) și le filtrează conform criteriilor tale, returnând doar cele care se mișcă cel mai mult și au cel mai mare potențial de tranzacționare.

### Ce face concret?

Imaginează-ți că ai **10.000+ de acțiuni** listate pe piața americană. Nu poți urmări toate manual. Market Discovery face asta pentru tine:

1. **Descarcă** lista tuturor acțiunilor active de pe Alpaca Markets
2. **Filtrează** doar cele de pe bursele americane (NYSE, NASDAQ, AMEX, ARCA)
3. **Preia** snapshot-ul de preț curent (preț, volum, schimbare %) pentru toate
4. **Aplică** filtrele tale (preț, volum, change %, RVOL, gap)
5. **Scorează** fiecare candidat pe o scală de 0-100
6. **Sortează** rezultatele după scor (cele mai bune primele)
7. **Afișează** lista în sidebar

### La ce ajută?

- **Găsești cele mai "fierbinți" acțiuni din ziua respectivă** — cele care se mișcă cel mai mult
- **Identifici oportunități de trading** pe care nu le-ai fi văzut altfel
- **Economisești timp** — în loc de 2 ore de cercetare manuală, scannerul face totul în 30 secunde

---

## 2. UNDE GĂSESC MARKET DISCOVERY ÎN INTERFAȚĂ?

Market Discovery se află în **sidebar-ul stâng** al aplicației, ca o secțiune colapsabilă cu headerul:

```
MARKET DISCOVERY ▸
```

- **Click** pe header pentru a deschide/închide secțiunea
- Headerul este **cyan neon** cu glow, stil cyber-HUD
- Când secțiunea este deschisă, se vede **▾** (rotit 90°)
- Când este o scanare nouă cu simboluri noi, apare un **badge roșu** (ex: 🔴3) pe header

---

## 3. PARAMETRII DE FILTRARE

Aceștia sunt câmpurile unde setezi **CE CAUȚI**. Fiecare parametru elimină acțiunile care nu se potrivesc criteriilor tale.

### 3.1 Preț Min ($) — `md-price-min`

**Ce face:** Setează limita inferioară de preț. Orice acțiune cu prețul UNDER această valoare este eliminată.

| Câmp | Valoare default | Interval |
|------|----------------|----------|
| Preț Min | $1 | $0.01 - $99,999 |

**Exemple:**
- `Preț Min = 1` → Elimină penny stocks sub $1 (protecție)
- `Preț Min = 10` → Cauți doar acțiuni peste $10
- `Preț Min = 0.10` → Incluzi micro-caps (risc foarte mare!)

**Sfat:** Începătorii ar trebui să folosească minimum $1 — acțiunile sub $1 sunt extrem de volatile și manipulate.

---

### 3.2 Preț Max ($) — `md-price-max`

**Ce face:** Setează limita superioară de preț. Orice acțiune cu prețul PESTE această valoare este eliminată.

| Câmp | Valoare default | Interval |
|------|----------------|----------|
| Preț Max | $500 | $1 - $99,999 |

**Exemple:**
- `Preț Max = 20` → Doar acțiuni ieftine (small-caps), ideale pentru day trading agresiv
- `Preț Max = 100` → Exclude acțiuni super-scumpe (AMZN, etc.)
- `Preț Max = 500` → Include aproape totul mai puțin BRK.A ($600K+)

**Sfat:** Pentru day trading, intervalul $1-$50 este cel mai folosit. Acțiunile scumpe ($200+) au spread-uri mari și necesită capital mare.

---

### 3.3 Volum Min (M) — `md-vol-min`

**Ce face:** Setează volumul minim zilnic în **milioane**. Acțiunile cu volum sub această valoare sunt eliminate.

| Câmp | Valoare default | Interval |
|------|----------------|----------|
| Volum Min | 0.5M (500,000) | 0.1M - 100M |

**Ce înseamnă volumul?** Numărul total de acțiuni tranzacționate în ziua respectivă. Volum mare = mulți traderi = lichiditate bună.

**Exemple:**
- `0.5M` (500K) → Standard, elimină acțiunile cu lichiditate scăzută
- `1M` → Doar acțiuni cu lichiditate medie-mare
- `5M` → Doar acțiuni foarte lichide (large-cap, ETF-uri populare)
- `0.1M` (100K) → Include și micro-caps (risc: spread-uri mari, slippage)

**De ce contează volumul?**
- Volum mic = greu de cumpărat/vândut rapid → **slippage** (prești mai mult decât vrei)
- Volum mare = poți intra/ieși instant fără impact pe preț
- Regulă de aur: **nu tranzacționa niciodată o acțiune cu volum sub 200K**

---

### 3.4 Min % Schimbare — `md-change-min` (Slider)

**Ce face:** Setează procentul minim de schimbare a prețului față de închiderea zilei anterioare. Acțiuni care s-au mișcat mai puțin sunt eliminate.

| Câmp | Valoare default | Interval | Control |
|------|----------------|----------|---------|
| Min % Schimbare | 1% | 0% - 50% | Slider (pas 0.5%) |

**Ce arată schimbarea?** Dacă o acțiune a închis ieri la $10 și acum e la $11, schimbarea este +10%.

**Exemple:**
- `1%` → Include aproape orice se mișcă (conservator)
- `5%` → Doar acțiuni cu mișcare semnificativă
- `10%` → Doar acțiuni "fierbinți" — gap-up-uri, știri, earnings
- `20%` → Extrem de agresiv — doar mișcări explozive (penny stocks, squeeze-uri)

**Comportament special:**
- Dacă **Short Selling este dezactivat** în setări: scannerul caută DOAR schimbări pozitive (+5% da, -5% nu)
- Dacă **Short Selling este activat**: scannerul caută schimbări în ambele direcții (|change| >= min), deci și -15% trece filtrul de 10%

**Sfat:** Valoarea citită din slider se afișează în galben (#FFD700) în dreapta slider-ului.

---

### 3.5 RVOL Minim (x) — `md-rvol-min`

**Ce face:** Setează volumul relativ minim. **RVOL** (Relative Volume) compară volumul de azi cu volumul zilei precedente.

| Câmp | Valoare default | Interval |
|------|----------------|----------|
| RVOL Min | 1x | 0 - 50x |

**Formula:** `RVOL = Volumul de azi / Volumul de ieri`

**Ce înseamnă?**

| RVOL | Interpretare |
|------|-------------|
| 0.5x | Volum la jumătate față de ieri — zi liniștită |
| 1.0x | Volum normal — ca ieri |
| 2.0x | Dublu față de ieri — interesant |
| 5.0x | De 5 ori mai mult decât ieri — **ceva se întâmplă!** |
| 10x+ | Exploziv — probabil știre majoră, FDA, earnings |

**Exemple practice:**
- `RVOL = 1` → Acceptă orice (fără filtru real)
- `RVOL = 2` → Doar acțiuni cu activitate peste normal
- `RVOL = 5` → Doar acțiuni cu activitate explozivă (recomandat Ross Cameron)
- `RVOL = 10` → Ultra-selectiv — câteva acțiuni pe zi

**De ce e important RVOL?**

RVOL este **cel mai important indicator pentru un day trader**. O acțiune cu RVOL mare înseamnă:
- Traderi instituționali sunt activi
- Știre sau catalizator important
- Mișcarea prețului are "combustibil" real (nu e doar noise)
- Spread-uri mai mici (lichiditate mai bună)

**Regulă de aur:** Nu tranzacționa NICIODATĂ o acțiune cu RVOL sub 2. Acelea sunt "mort" — fără energie, fără oportunitate.

---

### 3.6 Gap Min (%) — `md-gap-min`

**Ce face:** Setează procentul minim de gap. **Gap-ul** este diferența dintre prețul de deschidere de azi și prețul de închidere de ieri.

| Câmp | Valoare default | Interval |
|------|----------------|----------|
| Gap Min | 0% | 0% - 50% |

**Formula:** `Gap% = (Open azi - Close ieri) / Close ieri × 100`

**Ce este un Gap?**

Un gap apare când o acțiune **deschide** la un preț semnificativ diferit față de cum a **închis** ieri:

```
Ieri: Acțiunea X a închis la $10.00
Azi:  Acțiunea X a deschis la $12.00 → Gap-up de +20%
```

**Tipuri de gap:**
- **Gap-up** (+) → Prețul a sărit în SUS peste noapte (știre bună, earnings bune)
- **Gap-down** (-) → Prețul a căzut în JOS peste noapte (știre rea, earnings slabe)

**Exemple:**
- `Gap = 0%` → Fără filtru (include totul, este default-ul)
- `Gap = 2%` → Doar acțiuni care au deschis cu minim 2% diferență
- `Gap = 5%` → Gap semnificativ — de obicei știri sau earnings
- `Gap = 10%` → Gap major — acțiuni care vor fi în centrul atenției toată ziua

**Comportament special:** Ca și la Change, dacă Short Selling e dezactivat, doar gap-uri pozitive trec filtrul.

---

### 3.7 Float Max (M) — `md-float-max` 🆕

**Ce face:** Setează numărul maxim de acțiuni în circulație liberă (float) în **milioane**. Acțiunile cu float mai mare sunt eliminate.

| Câmp | Valoare default | Interval | Sursă date |
|------|----------------|----------|------------|
| Float Max | 0 (dezactivat) | 0 - 10,000M | FMP API |

**Ce este Float-ul?** Float-ul este numărul de acțiuni disponibile public pentru tranzacționare. **NU** include acțiunile deținute de insideri, fonduri blocate sau acțiuni restricționate.

**De ce contează Float-ul?**

| Float | Clasificare | Efect pe Preț |
|-------|------------|---------------|
| < 5M | **Ultra-low float** | Mișcări EXTREME (±50-100%/zi). Risc maxim. |
| 5-20M | **Low float** | Mișcări mari (±10-30%/zi). Ross Cameron target. |
| 20-100M | **Mediu** | Mișcări moderate. Lichiditate OK. |
| > 100M | **Mare** | Mișcări mici. Lichiditate excelentă. |

**Exemple:**
- `Float Max = 0` → Filtrul este dezactivat (include totul)
- `Float Max = 20` → **Ross Cameron style** — doar low float stocks
- `Float Max = 50` → Include și medium float
- `Float Max = 100` → Exclude doar large-cap-uri masive

**Cum funcționează tehnic:**
- Datele vin de la **Financial Modeling Prep (FMP)** API
- Cache 24h (float-ul se schimbă rar — doar la dilution/offerings)
- Dacă FMP nu are date pentru un simbol, acesta **NU este eliminat** (trece filtrul)
- Necesită FMP API Key (vezi secțiunea API & WALLET)

**Culori în tabel:**
- 🟣 **Mov** — Float ≤ 20M (low float, Ross Cameron target)
- ⚪ **Gri** — Float > 20M

**Sfat:** Float-ul mic + RVOL mare = **cocktail exploziv**. Când mulți cumpărători se luptă pe puține acțiuni disponibile, prețul EXPLODEAZĂ. Dar poate cădea la fel de repede!

---

### 3.8 Market Cap Max (B) — `md-cap-max` 🆕

**Ce face:** Setează capitalizarea de piață maximă în **miliarde**. Companiile mai mari sunt eliminate.

| Câmp | Valoare default | Interval | Sursă date |
|------|----------------|----------|------------|
| Cap Max | 0 (dezactivat) | 0 - 10,000B | FMP API |

**Ce este Market Cap?** `Market Cap = Preț × Total Acțiuni`. Măsoară "mărimea" companiei.

| Market Cap | Clasificare | Culoare în tabel |
|-----------|------------|------------------|
| < 300M | **Micro Cap** | 🟣 Mov |
| 300M - 2B | **Small Cap** | 🟡 Galben |
| 2B - 10B | **Mid Cap** | ⚪ Gri |
| > 10B | **Large Cap** | ⚫ Gri închis |

**Exemple:**
- `Cap Max = 0` → Filtrul dezactivat
- `Cap Max = 2` → **Ross Cameron style** — doar micro + small cap
- `Cap Max = 10` → Exclude large-caps (AAPL, MSFT, GOOGL)

**De ce contează?** Companiile mici se mișcă mai mult procentual. AAPL trebuie să crească cu $300B în cap pentru +10%. O companie de $50M trebuie doar $5M — de aceea small-caps au mișcări explozive.

**Tooltip:** Hover pe coloana CAP arată **sectorul** companiei (Technology, Healthcare, Energy, etc.)

---

### 3.9 REQUIRE CATALYST — Toggle (Checkbox)

**Ce face:** Când este activat, scannerul va marca acțiunile care au un catalizator (știre, earnings). 

| Câmp | Valoare default | Control |
|------|----------------|---------|
| Require Catalyst | OFF | Checkbox toggle |

**Ce este un catalizator?**

Un catalizator este o **știre sau un eveniment** care cauzează mișcarea prețului:
- 📰 Earnings report (raport financiar trimestrial)
- 💊 Aprobarea FDA (pentru companii pharma)
- 🤝 Parteneriat sau achiziție anunțată
- 📊 Upgrade/downgrade de la un analist
- 🏛️ Legislație care afectează sectorul
- 🟢 **Insider buying** (CEO/CFO cumpără acțiuni — acum detectat automat via FMP!)

**De ce contează?** O acțiune care se mișcă FĂRĂ catalizator este adesea doar "noise" sau manipulare. O acțiune cu catalizator are un motiv fundamental pentru mișcare — și mișcarea tinde să continue.

---

### 3.10 EXCLUDE EARNINGS ±3d — Toggle (Checkbox) 🆕

**Ce face:** Când este activat, elimină acțiunile care au earnings report în următoarele ±3 zile.

| Câmp | Valoare default | Control | Sursă date |
|------|----------------|---------|------------|
| Exclude Earnings | OFF | Checkbox toggle | FMP API |

**De ce ai exclude earnings?**
- Acțiunile fac mișcări de **±20-50%** la earnings — impredictibil!
- Dacă ești day trader conservator, earnings = **riscul nu merită**
- Spread-urile cresc dramatic înainte de earnings
- Implied volatility este umflată (opțiuni scumpe)

**Când să-l activezi:**
- ✅ Dacă ești începător și nu vrei riscuri extreme
- ✅ Dacă tranzacționezi cu capital pe care nu-l poți pierde

**Când să-l dezactivezi:**
- ❌ Dacă cauți explicit earnings plays (strategii gap-and-go post-earnings)
- ❌ Dacă vrei să vezi toate oportunitățile

**Cum funcționează:** FMP returnează calendarul cu UN SINGUR API call (super-eficient). Cache 12h.

---

### 3.8 Max Candidates — Slider

**Ce face:** Limitează numărul maxim de rezultate returnate de scanner.

| Câmp | Valoare default | Interval | Control |
|------|----------------|----------|---------|
| Max Candidates | 100 | 10 - 300 | Slider (pas 10) |

**Exemple:**
- `50` → Top 50 cele mai bune oportunități (rapid, concentrat)
- `100` → Standard — bun echilibru între vizibilitate și viteză
- `200` → Vedere largă — include și candidați mai slabi
- `300` → Maxim — tot ce trece filtrele

**Sfat:** Mai mulți candidați = scanare mai lentă (mai multe snapshots de preluat). Pentru scanări rapide, folosește 50.

---

### 3.9 Auto-Scan Interval (min) — Slider

**Ce face:** Setează intervalul în minute pentru scanarea automată (când Auto Scan este activat).

| Câmp | Valoare default | Interval | Control |
|------|----------------|----------|---------|
| Auto-Scan Interval | 30 min | 5 - 60 min | Slider (pas 5) |

**Exemple:**
- `5 min` → Scanare foarte frecventă (ideal pentru day trading activ, dar consumă mai mult API)
- `15 min` → Bun echilibru
- `30 min` → Standard — recomandat
- `60 min` → Scanare rară — pentru swing trading sau monitorizare pasivă

---

## 4. BUTOANELE DE ACȚIUNE

### 4.1 ⚡ SCAN (Albastru cyan, mare)

**Ce face:** Pornește o scanare manuală a pieței ACUM, folosind toți parametrii setați de tine.

**Cum funcționează:**
1. Click pe buton → Textul se schimbă în "⚡ SCANNING..."
2. Butonul se dezactivează (previne double-click)
3. Backend-ul descarcă toate acțiunile active (~10,000)
4. Le filtrează conform parametrilor tăi
5. Le scorează cu formula de scoring selectată
6. Returnează rezultatele sortate
7. Butonul revine la "⚡ SCAN"

**Durata:** 20-60 secunde (depinde de conexiunea la internet și numărul de acțiuni)

**Când să scanezi:** Cel mai bun moment este între **9:30 - 10:30 AM New York** (primele 60 minute de trading). Aici sunt cele mai multe oportunități.

---

### 4.2 ✕ CLEAR (Roșu)

**Ce face:** Șterge toate rezultatele afișate din panelul de rezultate.

**Nu afectează:** Parametrii de filtrare, presetările sau setările. Doar curăță lista de rezultate.

---

### 4.3 AUTO SCAN: OFF / ON (Cyan)

**Ce face:** Activează/dezactivează scanarea automată periodică.

**Când este ON:**
- Textul se schimbă în "AUTO SCAN: ON" cu culoare verde
- Scannerul rulează automat la intervalul setat (default 30 min)
- Scanarea se face doar **luni-vineri, între 9:00 - 17:00 ora New York**
- NU scanează în weekend (economisește API calls pe date stale de vineri)

**Când este OFF:**
- Textul arată "AUTO SCAN: OFF"
- Trebuie să apeși manual "⚡ SCAN"

---

### 4.4 ⚔️ ROSS CAMERON (Galben/Auriu)

**Ce face:** Aplică presetul profesional Ross Cameron (fondatorul Warrior Trading). Vezi [secțiunea 11](#11-ross-cameron-preset) pentru detalii complete.

---

### 4.5 INJECT → WATCHLIST (Cyan)

**Ce face:** Adaugă TOATE acțiunile din rezultatele scanării în lista de watchlist/favorites.

**Exemplu:** Dacă scanarea a returnat 15 acțiuni, toate 15 se adaugă în watchlist-ul tău cu un singur click.

---

### 4.6 AUTO INJECT

**Ce face:** Activează/dezactivează injectarea automată a top rezultatelor în watchlist după fiecare scanare.

---

## 5. PRESETĂRI (QUICK PRESETS)

Presetările sunt **configurații predefinite** care setează automat toți parametrii de filtrare cu un singur click. Sunt afișate în grid 2×2, fiecare cu culoare distinctă.

### 5.1 🚀 PENNY ROCKETS (Portocaliu) — `btn-md-penny-rockets`

**Strategie:** Acțiuni foarte ieftine cu mișcări procentuale uriașe.

| Parametru | Valoare | De ce? |
|-----------|---------|--------|
| Preț Min | $0.10 | Include penny stocks |
| Preț Max | $5.00 | Doar acțiuni ultra-ieftine |
| Volum Min | 2M | Volum mare (lichiditate necesară la penny) |
| RVOL Min | 5x | Activitate explozivă |
| Gap Min | 0% | Nu filtrează gap |
| Change Min | 20% | Doar mișcări masive |
| Max Candidates | 50 | Concentrat pe top |
| Catalyst | OFF | Nu necesită |
| Score Mode | MOMENTUM | Prioritizează schimbarea de preț |

**Când să folosești:** Când cauți acțiuni sub $5 care sar 20-100% într-o zi. Profil de risc: **FOARTE RIDICAT**. Doar cu bani pe care îți permiți să-i pierzi.

**Exemplu real:** O acțiune la $0.50 care sare la $1.50 într-o zi = +200%. Dar poate la fel de ușor să cadă la $0.20 (-60%).

---

### 5.2 📈 SWING SETUP (Mov/Purple) — `btn-md-swing-setup`

**Strategie:** Acțiuni mid-cap cu gap-uri semnificative, ideale pentru swing trading (țineri de 1-5 zile).

| Parametru | Valoare | De ce? |
|-----------|---------|--------|
| Preț Min | $10 | Exclude penny stocks |
| Preț Max | $100 | Mid-cap range |
| Volum Min | 1M | Lichiditate bună |
| RVOL Min | 3x | Activitate peste normal |
| Gap Min | 2% | Gap semnificativ |
| Change Min | 3% | Mișcare moderată |
| Max Candidates | 100 | Vedere largă |
| Catalyst | OFF | Nu necesită |
| Score Mode | GAP SCANNER | Prioritizează gap-ul |

**Când să folosești:** Seara (after hours) sau dimineața devreme, pentru a identifica acțiuni care au deschis cu gap și ar putea continua mișcarea.

**Exemplu:** O acțiune la $30 deschide la $33 (gap +10%) pe earnings bune. Tu intri în prima oră, setezi stop-loss la $31.50, și ținteiti $36 în 2-3 zile.

---

### 5.3 🏛️ LARGE CAP (Albastru) — `btn-md-large-cap`

**Strategie:** Acțiuni mari, lichide, stabile — ideale pentru traderi conservatori.

| Parametru | Valoare | De ce? |
|-----------|---------|--------|
| Preț Min | $50 | Doar companii mari |
| Preț Max | $500 | Include AAPL, GOOGL, MSFT |
| Volum Min | 5M | Ultra-lichid |
| RVOL Min | 1x | Volum normal e OK |
| Gap Min | 0% | Nu filtrează gap |
| Change Min | 3% | Mișcare semnificativă pentru large-cap |
| Max Candidates | 50 | Focusat |
| Catalyst | OFF | Nu necesită |
| Score Mode | VOLUME HUNTER | Prioritizează volumul |

**Când să folosești:** Oricând. Large-caps se mișcă mai puțin procentual, dar au spread-uri mici și slippage minim. Perfect pentru începători.

**Exemplu:** AAPL scade 4% pe știri de supply chain → intri long cu stop tight, așteptând rebound.

---

### 5.4 🔥 HOT MOMENTUM (Roșu) — `btn-md-hot-momentum`

**Strategie:** Cele mai "fierbinți" acțiuni din piață — combinație de change mare și RVOL ridicat.

| Parametru | Valoare | De ce? |
|-----------|---------|--------|
| Preț Min | $1 | Include small-caps |
| Preț Max | $50 | Exclude acțiuni scumpe |
| Volum Min | 1M | Lichiditate decentă |
| RVOL Min | 3x | Minim 3x volumul normal |
| Gap Min | 0% | Include totul |
| Change Min | 10% | Doar mișcări puternice |
| Max Candidates | 100 | Vedere largă |
| Catalyst | OFF | Nu necesită |
| Score Mode | MOMENTUM | Prioritizează schimbarea de preț |

**Când să folosești:** În primele 2 ore de market open (9:30-11:30 ET). Aceasta e fereastra de "momentum" — acțiunile care se mișcă tind să continue.

---

### 5.5 Ce se întâmplă când apeși un preset?

1. **Toți parametrii** se actualizează instant în UI (culorile slider-ilor se schimbă)
2. **Valorile se salvează** în backend (persisten între sesiuni)
3. **Score Mode** se schimbă automat la modul potrivit presetului
4. **Butonul** se iluminează cyan scurt (0.8 secunde) ca confirmare
5. **Mesaj status:** "Penny Rockets preset applied" (la baza secțiunii)

**⚠️ Important:** Apăsarea unui preset NU pornește scanarea automată. Trebuie să apeși "⚡ SCAN" separat.

---

## 6. SCORE MODE — MODURI DE SCORING

Score Mode controlează **formula de scoring** — adică cum se calculează scorul (0-100) pentru fiecare acțiune candidat. Diferite moduri pun accent pe metrici diferite.

### 6.1 Componentele scorului

Scorul total are **4 componente**, fiecare cu o pondere. Suma ponderilor = 100:

| Componentă | Ce măsoară | Exemplu valoare bună |
|-----------|-----------|---------------------|
| **CHG** (Change) | Cât s-a mișcat prețul procentual | +15% |
| **RVOL** (Relative Volume) | Cât de activ e față de normal | 8x |
| **GAP** (Gap) | Cât a deschis diferit de ieri | +5% |
| **VOL** (Volume absolut) | Volumul brut în acțiuni | 12M |

### 6.2 Cele 4 moduri disponibile

#### ⚖️ BALANCED — 25 / 25 / 25 / 25

**Toate componentele au importanță egală.** Cel mai neutru mod. Nu favorizează nimic.

**Când:** Când nu știi ce cauți. Bun pentru explorare generală.

**Exemplu scoring:**
```
Acțiune: XYZ
Change: +8% → Score: (8/20) × 25 = 10.0
RVOL: 4x  → Score: (3/10) × 25 = 7.5
Gap: +3%  → Score: (3/10) × 25 = 7.5
Volume: 3M → Score: 25 × 0.72 = 18.0
────────────────────────────────────────
TOTAL: 43.0 / 100
```

#### 🚀 MOMENTUM — 50 / 20 / 10 / 20

**Schimbarea de preț contează DE 2 ORI mai mult.** Ideal pentru day trading agresiv.

**Când:** Cauți acțiunile cu cele mai mari mișcări procentuale.

**Exemplu:** Acțiune cu +25% change dar volum modest va fi scorată mai sus decât una cu +5% dar volum enorm.

#### 📊 VOLUME HUNTER — 15 / 20 / 10 / 55

**Volumul absolut contează DUBLU.** Ideal pentru traderi care prioritizează lichiditate.

**Când:** Cauți acțiuni cu cel mai mare volum tranzacționat. Bun dacă tranzacționezi cu sume mari.

**Exemplu:** Apple (AAPL) cu 50M volume și +3% change va fi scorat mai sus decât un penny cu 500K volume și +30% change.

#### 📉 GAP SCANNER — 10 / 15 / 55 / 20

**Gap-ul contează DE 2 ORI mai mult.** Ideal pentru gap trading.

**Când:** Cauți acțiuni care au deschis cu gap semnificativ. Strategia: cumperi gap-ul și aștepți continuare sau fill.

**Exemplu:** O acțiune care a deschis +8% de la close-ul de ieri va fi scorată foarte sus, chiar dacă change-ul actual e doar +5% (s-a retras din gap).

### 6.3 Cum schimb Score Mode?

1. Deschide dropdown-ul "**◈ SCORE MODE**"
2. Selectează modul dorit
3. Se salvează automat în backend
4. Mesajul status arată: "Score mode: MOMENTUM" (sau ce ai ales)
5. **Efectul se aplică la următoarea scanare** — nu modifică rezultatele deja afișate

### 6.4 Legenda "CHG / RVOL / GAP / VOL"

Sub dropdown apare o legendă mică gri:

```
CHG / RVOL / GAP / VOL
```

Aceasta îți arată ordinea componentelor din cifrele afișate. De exemplu, "50 / 20 / 10 / 20" înseamnă:
- CHG (Change) = 50 puncte maxim
- RVOL = 20 puncte maxim
- GAP = 10 puncte maxim
- VOL (Volume) = 20 puncte maxim

---

## 7. REZULTATELE SCANĂRII

După apăsarea "⚡ SCAN", rezultatele apar într-un tabel cu următoarele coloane:

### 7.1 Coloanele tabelului

| Coloană | Ce arată | Culori | Sursă |
|---------|---------|--------|-------|
| **SYMBOL** | Numele acțiunii (ex: AAPL) + 📊 badge earnings | Click → schimbă simbolul activ | Alpaca |
| **PRICE** | Prețul curent | Alb/gri | Alpaca |
| **CHG%** | Schimbare % față de ieri | 🔵Cyan(≥10%), 🟡Galben(≥0%), 🔴Roșu(<0%) | Alpaca |
| **RVOL** | Volumul relativ (ex: 5.2x) | 🔵Cyan(≥5x), ⚪Gri(<5x) | Alpaca |
| **VOL** | Volumul absolut (format scurt) | Gri | Alpaca |
| **$VOL** 🆕 | Dollar Volume (preț × volume) | 🟡Gold(≥$10M), ⚪Dim(<$10M) | Calcul |
| **FLOAT** | Float shares în milioane | 🟣Mov(≤20M), ⚪Gri(>20M) | FMP |
| **CAP** 🆕 | Market cap (hover=sector) | 🟣Mov(<300M), 🟡Galben(<2B), ⚪Gri(<10B), ⚫(<10B+) | FMP |
| **INS** 🆕 | Insider trading signal | 🟢(BUY), 🔴(SELL), —(none). Hover=detalii. | FMP |
| **SCORE** | Scorul calculat (0-100) | 🔵Cyan(≥60), 🟡Galben(≥30), ⚪Gri(<30) | Calcul |
| **Δ** | Delta față de scanarea precedentă | Vezi secțiunea 8 | Calcul |
| **+** | Buton adăugare în watchlist | 🟡Galben → 🔵Cyan(✓) la confirmare | — |

### 7.2 Coloane FMP (apar doar cu FMP API Key)

Coloanele FLOAT, CAP și INS sunt **vizibile doar dacă FMP API Key este configurat** și datele sunt disponibile. Fără key, scannerul funcționează normal dar fără aceste date suplimentare.

**Earnings Badge (📊):**
Dacă o acțiune are earnings report în următoarele ±3 zile, apare un badge galben 📊 lângă numele simbolului. Hover pe badge arată "Earnings ±3 days".

**Insider Signal (INS):**
- 🟢 = Insider **BUY** recente (CEO, CFO, director au cumpărat acțiuni)
- 🔴 = Insider **SELL** recente
- — = Fără date insider
- **Hover** pe emoji arată detalii: *"Tim Cook BUY 50000sh @$182.50"*

### 7.2 Formatarea volumului

Volumul se afișează în format scurt pentru lizibilitate:

| Volum Real | Afișat Ca |
|------------|-----------|
| 500,000 | 500K |
| 1,200,000 | 1.2M |
| 15,700,000 | 15.7M |
| 250,000,000 | 250M |

### 7.3 Interacțiuni pe rânduri

- **Click pe simbol** → Setează acțiunea ca simbol activ (graficul se schimbă, analiza se refocusează)
- **Click pe +** → Adaugă acțiunea în watchlist/favorites

---

## 8. COLOANA Δ (DELTA) — DIFF ALERT

### Ce este Delta?

Delta (Δ) este **diferența de scor** între scanarea curentă și scanarea anterioară pentru aceeași acțiune.

### Cum funcționează?

1. **Prima scanare:** Coloana Δ nu apare (nu are referință anterioară)
2. **A doua scanare (și după):** Fiecare acțiune arată diferența:
   - Dacă XYZ avea scor 45 acum 30 min și acum are 72 → Δ = **+27**
   - Dacă ABC avea scor 60 și acum are 50 → Δ = **-10**
3. **Acțiuni noi** (care nu erau în scanarea anterioară) → Se afișează "—" (liniuță)

### Culorile Delta

| Delta | Culoare | Înseamnă |
|-------|---------|----------|
| **> +5** | 🟢 Verde (#00FF88) | Accelerare puternică — ATENȚIE! |
| **> 0** | 🔵 Cyan (#00BFFF) | Îmbunătățire moderată |
| **= 0** | ⚪ Gri (#555) | Stabil |
| **< 0** | 🟠 Portocaliu (#FF8800) | Decelerare |
| **< -5** | 🔴 Roșu (#FF4444) | Scădere semnificativă |

### De ce e important Delta?

**Delta este cel mai puternic semnal intraday.** De ce?

- Un scor absolut de 70 poate fi o acțiune "mereu fierbinte" (ex: GME, AMC)
- Dar un delta de **+25** (scorul a sărit de la 45 la 70 în 30 min) înseamnă: **CEVA S-A SCHIMBAT ACUM**
- Aceasta indică **accelerare** — acțiunea câștigă momentum CHIAR ACUM
- Este exact momentul în care traderii profesioniști intră

### Exemplu practic

```
Scanare 1 (10:00 AM):
  MARA  $22.50  +5%   2.1x  3.2M  +2%   42     —
  RIOT  $15.30  +3%   1.8x  2.5M  +1%   35     —

Scanare 2 (10:30 AM):
  MARA  $24.10  +12%  4.5x  8.1M  +2%   67    +25 🟢  ← ACCELERARE!
  RIOT  $15.80  +6%   2.3x  3.1M  +1%   41    +6  🟢
  NVDA  $850    +4%   3.2x  25M   +1%   55     — (NEW)
```

MARA a avut un delta de +25 → A sărit de la scor 42 la 67 în 30 minute. Ceva se întâmplă — posibil știre despre Bitcoin, mișcare sectorială, short squeeze.

---

## 9. NEW MOVER — ALERTE PENTRU SIMBOLURI NOI

### Ce este un New Mover?

Un "New Mover" este o acțiune care **apare în rezultate PENTRU PRIMA DATĂ** — nu era în scanarea anterioară dar acum trece toate filtrele.

### Cum sunt afișate?

#### 9.1 Badge "NEW" pe rând

Simbolurile noi au un badge verde pulsant **NEW** înaintea numelui:

```
🟢NEW NVDA  $850.00  +4%  3.2x  25M  +1%  55
```

- Badge-ul are animație de **pulse** (glow verde care pulsează)
- Rândul are un **border stâng verde** și **background subtil verde**

#### 9.2 Toast Notification

Când apar simboluri noi, un **toast** (notificare) slide-in apare în **colțul dreapta-sus** al ecranului:

```
✔ 3 NEW MOVERS: NVDA, MARA, RIVN
```

- Folosește sistemul unificat TitanToasts (`showCyberToast`, tip `success`)
- Glassmorphism cu gradient verde și border neon
- Se auto-închide după **5 secunde**, click pe X pentru dismiss manual
- Afișează maximum 4 simboluri + "+X" pentru restul

#### 9.3 Badge pe Header

Headerul "MARKET DISCOVERY" primește un **badge roșu** cu numărul de new movers:

```
MARKET DISCOVERY 🔴3
```

- Badge-ul are animație de "pop" la apariție (scale 0 → 1.3 → 1)
- Glow roșu (box-shadow)
- Se resetează la următoarea scanare (sau devine 0)

### De ce contează New Movers?

**Un New Mover este cel mai periculos/oportun eveniment intraday:**

- O acțiune care NU era în scanner acum 30 min dar ACUM trece toate filtrele înseamnă:
  - Tocmai a avut o explozie de volum
  - A trecut un prag de preț/change care l-a "activat"
  - Probabil un catalizator a declanșat mișcarea ACUM
- Aceștia sunt candidații cu cel mai mare potențial **PE TERMEN SCURT** (15-60 min)

---

## 10. AUTO SCAN — SCANARE AUTOMATĂ

### Cum activez Auto Scan?

1. Click pe butonul **"AUTO SCAN: OFF"**
2. Se schimbă în **"AUTO SCAN: ON"** cu culoare verde
3. Scannerul rulează automat la fiecare `Auto-Scan Interval` minute

### Reguli de funcționare

- **Luni-Vineri:** Scanează doar între orele 9:00 - 17:00 ora New York
- **Weekend (Sâmbătă/Duminică):** NU scanează (bursele sunt închise)
- **Outside hours:** NU scanează (datele nu se schimbă)
- **Parametrii folosiți:** Aceiași parametri pe care îi vezi în UI

### Fluxul Auto-Scan

```
Timer expirat (30 min)
    ↓
Verifică: E luni-vineri? ✓
Verifică: Ora 9-17 ET?  ✓
    ↓
Fetch acțiuni active
    ↓
Filtrare cu quant_prefilter()
    ↓
Scoring cu score_weights_for_preset()
    ↓
Rezultate afișate în UI
    ↓
Reset timer → Așteaptă încă 30 min
```

---

## 11. ROSS CAMERON PRESET

### Cine este Ross Cameron?

Ross Cameron este fondatorul **Warrior Trading** și unul dintre cei mai respectați day traderi din lume. Strategia sa se concentrează pe acțiuni ieftine cu mișcări explosive. Cele 5 criterii ale sale sunt: **Preț scăzut, Volum relativ mare, Catalizator, Float mic, Change mare.**

### Parametrii Ross Cameron (V8 — Complet cu FMP)

| # | Parametru | Valoare | Criteriu Ross Cameron | Sursă |
|---|-----------|---------|----------------------|-------|
| 1 | Preț Min | $1 | ✅ Penny stocks incluse | Alpaca |
| 2 | Preț Max | $20 | ✅ Small-cap focus | Alpaca |
| 3 | Volum Min | 1M | ✅ Lichiditate minimă | Alpaca |
| 4 | RVOL Min | 5x | ✅ **"Stocks that are already UP"** | Alpaca |
| 5 | Gap Min | 0% | Nu filtrează gap (Change e suficient) | Alpaca |
| 6 | Change Min | 10% | ✅ **"Stocks already up 10%"** | Alpaca |
| 7 | **Float Max** | **20M** | ✅ **"Low float stocks"** | 🆕 FMP |
| 8 | **Cap Max** | **2B** | ✅ **Small cap only** | 🆕 FMP |
| 9 | Catalyst | ON | ✅ **"There must be a reason"** | Alpaca |
| 10 | Exclude Earnings | OFF | Nu exclude (earnings = catalizator valid) | 🆕 FMP |

### Cele 5 criterii Ross Cameron vs implementarea Titan:

| Criteriu | Descriere | Titan V7 | Titan V8 (FMP) |
|----------|-----------|:--------:|:--------------:|
| 1. RVOL | Volum relativ > 5x | ✅ | ✅ |
| 2. Change | Deja UP 10%+ | ✅ | ✅ |
| 3. Catalyst | Motiv pentru mișcare | ✅ | ✅+ (insider) |
| 4. Float | Float mic (< 20M) | ❌ lipsea | ✅ **FMP** |
| 5. Price | $1 - $20 | ✅ | ✅ |
| 6. Market Cap | Small cap | ❌ lipsea | ✅ **FMP** |

**Titan V8 implementează TOATE criteriile Ross Cameron — 100% complet.**

### De ce funcționează?

Combinația **preț mic + volum exploziv + float mic + cap mic** identifică acțiunile care:
- Au un catalizator (știre, earnings, insider buying)
- Sunt în centrul atenției traderilor
- Au float mic = ofertă limitată + cerere mare = **explozie de preț**
- Au potențial de +20-50% intraday
- Dar și risc de -20-30% dacă momentum-ul se inversează

### ⚠️ Atenție

**Strategia Ross Cameron este AGRESIVĂ.** Nu e potrivită pentru începători cu capital mic. Riscul de pierdere este la fel de mare ca potențialul de câștig.

---

## 12. INJECT ÎN WATCHLIST / SIGNAL CFG

### INJECT → WATCHLIST (Cyan)

Click pe acest buton adaugă **TOATE** acțiunile din rezultatele curente în lista ta de watchlist (favorites).

**Ce se întâmplă:**
1. Fiecare simbol din rezultate este trimis la backend cu `add_favorite`
2. Watchlist-ul se actualizează instant
3. Simbolurile devin vizibile în tab-ul WATCHLIST din sidebar

### INJECT → SIGNAL CFG (Magenta/Mov) 🆕

Click pe acest buton injectează rezultatele direct în **Signal CFG** — lista de asseturi pentru strategia de trading automată.

**Ce se întâmplă:**
1. Simbolurile sunt trimise la backend cu `inject_to_signal_assets`
2. Signal CFG primește noile asseturi pentru monitorizare
3. Butonul face flash magenta ca confirmare

**Diferența:**
- **WATCHLIST** = lista ta personală de urmărit (pasiv)
- **SIGNAL CFG** = lista activă pe care botul o folosește pentru trading automat

### AUTO INJECT

Când este activat, topul rezultatelor se injectează automat după fiecare scanare (manuală sau auto).

### Butonul "+" per rând

Fiecare rând din rezultate are un buton **"+"** în dreapta care adaugă DOAR acel simbol în watchlist:
- Click "+" → Se schimbă în "✓" (cyan) pentru 1.5 secunde
- Simbolul apare instant în watchlist

---

## 13. CUM FUNCȚIONEAZĂ SCANNERUL PE INTERIOR (BACKEND)

### Pipeline-ul complet (de la click la rezultate)

```
[UI] Click "⚡ SCAN"
    ↓
[JS] titan_watchlist.js → invoke('run_market_scan', { strategy: 'DISCOVERY' })
    ↓
[Rust] scanner_cmd.rs → run_market_scan()
    │  Citește scan_params din state
    │  Sincronizează allow_negative_change ← allow_short
    ↓
[Rust] scanner.rs → scan_discovery(&key, &secret, &scan_params, &fmp_key)
    │
    ├── 1. fetch_active_assets() → ~10,000 acțiuni
    │   └── Filtrare: tradable=true, exchange ∈ {NYSE, NASDAQ, AMEX, ARCA}
    │
    ├── 2. fetch_snapshots_batch() → Snapshot-uri de preț
    │   └── Procesate în batch-uri de ~5,000 simboluri
    │   └── Delay 300ms între batch-uri (respectare rate limit Alpaca)
    │
    ├── 3. quant_prefilter(&snapshots, &scan_params)
    │   ├── Filtru: close ∈ [price_min, price_max]
    │   ├── Filtru: volume ≥ volume_min
    │   ├── Filtru: price × volume ≥ min_dollar_volume (default $5M) ← S04.1
    │   ├── Filtru: change_pct ≥ change_min (sau abs() dacă shorts=ON)
    │   ├── Filtru: rvol ≥ rvol_min
    │   ├── Filtru: gap_pct ≥ gap_min (sau abs() dacă shorts=ON)
    │   ├── Scoring: compute_candidate_score_weighted() cu ponderile din preset
    │   ├── Sort: prefilter_score DESC, symbol ASC (deterministic)
    │   ├── ═══ TIERED SCAN (S15 — dacă tiered_scan_enabled) ═══
    │   ├── Pre-rank by dollar volume DESC
    │   ├── Tier 1: top tier1_count (default 50) → scorate complet
    │   ├── Tier 2: următoarele tier2_count (default 100) → scorate DOAR dacă RVOL ≥ 1.5 SAU change% ≥ 3%
    │   ├── Merge: union tier1+tier2, re-sort by prefilter score
    │   ├── Log: [TIERED_SCAN] tier1={} tier2_scored={} total={}
    │   └── Truncare: la max_candidates (~100 candidați)
    │
    ├── 4. FMP ENRICHMENT (post-filter — doar pe ~100 candidați, salvează API calls)
    │   ├── fetch_float_batch()     → Float shares per simbol (cache 24h)
    │   ├── fetch_profile_batch()   → Market cap + sector per simbol (cache 24h)
    │   ├── fetch_insider_batch()   → Insider buy/sell signal (cache 24h)
    │   ├── fetch_earnings_calendar() → Earnings ±3 zile (cache 12h, 1 call total!)
    │   └── fetch_targets_batch()   → Analyst upside % (cache 7 zile)
    │
    ├── 5. POST-FILTER (filtre FMP aplicate DUPĂ enrichment)
    │   ├── float_max: elimină dacă float > float_max (și float_max > 0)
    │   ├── cap_max: elimină dacă market_cap > cap_max (și cap_max > 0)
    │   └── exclude_earnings: elimină dacă has_earnings_soon == true
    │
    └── 6. Conversie PreFilterCandidate → ScanResult (cu date FMP atașate)
        └── strategy = "DISCOVERY_V8"
    ↓
[JS] Primește Vec<ScanResult> (cu float_shares, market_cap, sector, insider_signal, etc.)
    │
    ├── computeScanDiffs() → Calculează delta față de scan anterior
    │   ├── Compară scorul curent cu _prevScanScores
    │   ├── Identifică new movers (nu erau în _prevScanSymbols)
    │   └── Actualizează cache-urile pentru următoarea scanare
    │
    ├── renderScanResults() → Afișează tabelul cu Δ și NEW badges
    ├── showNewMoverToast() → Toast via showCyberToast (dacă sunt new movers)
    └── updateMdBadge() → Badge pe header
```

### Formula de scoring detaliată

```
Score = change_score + rvol_score + gap_score + vol_score

change_score = min(|change_pct|, 20) / 20 × W_change
    Exemplu: change=15%, W=25 → (15/20)×25 = 18.75

rvol_score = dacă rvol > 1.0:
                min(rvol - 1.0, 10.0) / 10.0 × W_rvol
             altfel: 0
    Exemplu: rvol=4x, W=25 → ((4-1)/10)×25 = 7.5

gap_score = min(|gap_pct|, 10.0) / 10.0 × W_gap
    Exemplu: gap=6%, W=25 → (6/10)×25 = 15.0

vol_score = dacă volume > 5M   → W_volume × 1.00
            dacă volume > 2M   → W_volume × 0.72
            dacă volume > 2×prev → W_volume × 0.48
            altfel              → (volume/5M) × W_volume × 0.40
    Exemplu: volume=3.5M, W=25 → 25×0.72 = 18.0
```

---

## 14. FMP DATA INTEGRATION — DATE FUNDAMENTALE GRATUITE

### 14.1 Ce este FMP (Financial Modeling Prep)?

FMP este un serviciu API care oferă **date fundamentale despre companii** — date pe care Alpaca ($99/mo) **NU le oferă**. Planul gratuit include **250 API calls/zi**, suficient pentru utilizarea Titan.

### 14.2 Setup FMP API Key

1. Creează cont gratuit pe [financialmodelingprep.com](https://financialmodelingprep.com)
2. Copiază API Key-ul din dashboard
3. În Titan, deschide **API & WALLET** din sidebar
4. Lipește key-ul în câmpul **FMP API KEY**
5. Click **SAVE FMP KEY** → Butonul devine verde "✓ SAVED"
6. Key-ul se salvează persistent în `titan_keys.json`

### 14.3 Cele 5 Module FMP Integrate

| # | Modul | Ce oferă | Cache | API Calls |
|---|-------|---------|-------|----------|
| 1 | **Float** | Nr. acțiuni în float (milioane) | 24h | 1/simbol |
| 2 | **Company Profile** | Market cap + sector | 24h | 1/simbol |
| 3 | **Insider Trading** | Cumpărări/vânzări CEO, CFO, etc. | 24h | 1/simbol |
| 4 | **Earnings Calendar** | Earnings ±3 zile | 12h | 1 call TOTAL |
| 5 | **Price Targets** | Consensus analiști (upside %) | 7 zile | 1/simbol |

### 14.4 Budget API Calls (250/zi gratuit)

Cu cache agresiv, consumul tipic:

| Scenariu | Prima scanare | Scanări ulterioare | Total/zi |
|----------|:------------:|:------------------:|:--------:|
| 1 scan manual | ~150 | — | ~150 ✅ |
| Auto-Scan 30 min | ~150 | ~20-30 each | ~200 ✅ |
| Auto-Scan 15 min | ~150 | ~20-30 each | ~230 ✅ |
| Auto-Scan 5 min | ~150 | ~20-30 each | ~250 ⚠️ |

**De ce e eficient?** Cache-ul face ca a doua scanare să consume doar calls pentru simboluri NOI (care nu erau în scanarea anterioară). Float-ul, profilele și targetele rămân cached.

### 14.5 Insider Trading — Cel mai puternic semnal

**Ce sunt insider trades?** Când CEO-ul, CFO-ul sau directorii unei companii **cumpără sau vând acțiuni cu banii lor personali**. Aceste tranzacții sunt raportate obligatoriu la SEC (Form 4).

**De ce contează?**
- **Insider BUY** = Persoana din interior crede că acțiunea va CREȘTE
- Este considerat **cel mai credibil semnal bullish** din piață
- Studiile arată că insider buys bat piața cu 6-10% pe an
- **Insider SELL** = Mai puțin semnificativ (pot vinde din diverse motive)

**Exemplu în tabel:**
```
SYMBOL  PRICE  CHG%  RVOL  VOL   FLOAT  CAP    INS   SCORE
MULN    $2.35  +45%  12x   5.2M  8.2M   120M   🟢    87.3
                                                 ^
                                          Hover: "John Smith BUY 50000sh @$1.85"
```

### 14.6 Earnings Calendar — Protecție sau Oportunitate?

Badge-ul 📊 apare automat lângă simbolul acțiunilor cu earnings ±3 zile:

```
MULN📊  $2.35  +45%  ...
```

**Două abordări:**
1. **Conservator:** Activează "EXCLUDE EARNINGS" → Evită riscul
2. **Agresiv:** Caută explicit acțiunile cu 📊 → Earnings gap-and-go

### 14.7 Price Targets — Upside Potențial

Scorul `analyst_upside_pct` este calculat automat:
```
Upside% = (Consensus Target - Preț Actual) / Preț Actual × 100
```

Exemplu: Dacă analiștii au target $50 și acțiunea e la $10 → Upside = +400%.

---

## 15. SCENARII PRACTICE DE UTILIZARE

### Scenariul 1: "Sunt începător, ce fac?"

1. Deschide **Market Discovery**
2. Apasă **🏛️ LARGE CAP**
3. Apasă **⚡ SCAN**
4. Uită-te la primele 5 rezultate — toate sunt companii mari, lichide
5. Click pe simbol → Graficul se încarcă
6. Analizează graficul și decide

### Scenariul 2: "Vreau acțiuni la open (9:30 AM)"

1. La 9:35 AM ET, apasă **🔥 HOT MOMENTUM**
2. Apasă **⚡ SCAN**
3. Sortează mental după RVOL (cele cu 5x+ sunt cele mai interesante)
4. Click pe top 3 → Analizează graficul rapid

### Scenariul 3: "Vreau penny stocks explozive"

1. Apasă **🚀 PENNY ROCKETS**
2. Apasă **⚡ SCAN**
3. Caută acțiuni cu RVOL > 10x și Change > 30%
4. **⚠️ RISC EXTREM** — pune stop-loss STRÂNS

### Scenariul 4: "Vreau scanare continuă" (monitorizare pasivă)

1. Setează parametrii doriti (sau apasă un preset)
2. Setează **Auto-Scan Interval** la 15 min
3. Apasă **AUTO SCAN: OFF** → devine **ON**
4. Lasă-l să scaneze singur
5. Urmărește badge-ul 🔴 și toast-urile pentru NEW MOVERS
6. Când vezi ceva interesant → Click pe simbol

### Scenariul 5: "Vreau să compar două scanări"

1. Apasă **⚡ SCAN** la 10:00 AM → Primele rezultate
2. Apasă **⚡ SCAN** la 10:30 AM → Rezultate cu coloana Δ
3. Sortează mental după Δ:
   - **Δ = +20** → Acțiunea accelerează ACUM → Investighhează!
   - **Δ = -15** → Momentum-ul scade → Evită
   - **NEW** → Acțiune complet nouă → Verifică imediat

---

### Scenariul 6: "Vreau Ross Cameron COMPLET" (V8 cu FMP) 🆕

1. Setează FMP API Key (o singură dată)
2. Apasă **⚔️ ROSS CAMERON** → Setează automat: Float ≤20M, Cap ≤2B, RVOL ≥5x, Change ≥10%
3. Apasă **⚡ SCAN**
4. Coloanele FLOAT (mov), CAP (galben), INS (🟢/🔴) apar automat
5. Caută acțiuni cu: **Score >60, Float <10M, Cap <500M, INS=🟢**
6. Aceste sunt candidații PERFECȚI Ross Cameron — low float, small cap, insider confidence

### Scenariul 7: "Vreau să evit earnings" 🆕

1. Activează **EXCLUDE EARNINGS ±3d**
2. Scanează normal
3. Toate acțiunile cu 📊 sunt eliminate automat
4. Tranzacționezi fără surprize de earnings

---

## 16. SFATURI DE EXPERT PENTRU ÎNCEPĂTORI

### ✅ DO (Ce să faci)

1. **Scanează în primele 30 min** (9:30-10:00 ET) — aici e cel mai mult volum
2. **Folosește RVOL ≥ 3** — acțiunile cu RVOL sub 2 sunt "moarte"
3. **Verifică graficul** înainte de a tranzacționa — scannerul găsește candidați, NU semnale
4. **Setează stop-loss** ÎNAINTE de a intra — decide cât ești dispus să pierzi
5. **Folosește Score Mode corect** — dacă ești day trader, folosește MOMENTUM; dacă ești swing trader, GAP SCANNER
6. **Scanează de 2+ ori** — coloana Δ apare doar de la a doua scanare

### ❌ DON'T (Ce să NU faci)

1. **NU tranzacționa penny stocks** fără experiență — sunt extrem de volatile
2. **NU ignora volumul** — acțiuni cu volum mic au spread-uri mari
3. **NU intra pe baza scorului singur** — scorul indică potențial, NU garanție
4. **NU scanezi în weekend** — datele sunt de vineri, nu sunt relevante
5. **NU lăsa Auto-Scan pe 5 min** dacă nu ai nevoie — consumă API rate limits
6. **NU ignora Δ negativ** — un delta de -20 înseamnă că momentumul dispare

### 📊 Reguli practice

| Regulă | Explicație |
|--------|-----------|
| **Scor > 60** | Candidat strong — merită analizat |
| **Scor 30-60** | Candidat mediu — verifică doar dacă ai timp |
| **Scor < 30** | Slab — probabil nu merită atenție |
| **RVOL > 5** | Activitate explozivă — ceva se întâmplă |
| **Gap > 5%** | Gap semnificativ — potențial real de continuare |
| **Δ > +10** | Accelerare — ACUM e momentul |
| **NEW badge** | Acțiune proaspătă — verifică imediat |

| **FLOAT < 10M** | Ultra low float — mișcări de ±30-50% | |
| **CAP < 300M** | Micro cap — zona Ross Cameron |
| **INS = 🟢** | Insider buying — cel mai puternic bullish signal |
| **📊 badge** | Are earnings ±3 zile — ATENȚIE la risc! |

---

## 17. PROBLEME FRECVENTE ȘI SOLUȚII

### P: "Am apăsat SCAN dar nu primesc rezultate"

**Soluții posibile:**
1. **Filtrele sunt prea restrictive** — Setează Change Min la 1%, RVOL la 1, Gap la 0%
2. **Piața e închisă** — Scannerul funcționează cel mai bine 9:30-16:00 ET luni-vineri
3. **API keys lipsă** — Verifică secțiunea API & WALLET din sidebar
4. **Weekend** — Datele de vineri nu vor avea aceleași mișcări

### P: "Coloana Δ nu apare"

**Explicație:** Δ apare doar de la **A DOUA scanare** în sesiunea curentă. La prima scanare nu are cu ce compara. Scanează din nou și va apărea.

### P: "Am primit toast cu NEW MOVERS dar nu văd rândul"

**Explicație:** Scroll-ează în jos în panelul de rezultate. New Movers pot fi mai jos în listă (scorul lor poate fi mai mic decât al celor vechi).

### P: "Scanarea durează foarte mult"

**Cauze posibile:**
1. **Max Candidates = 300** → Setează la 50-100 pentru viteză
2. **Conexiune internet lentă** → Snapshot-urile necesită date de la Alpaca
3. **API rate limiting** → Alpaca are limită de requests pe minut. Dacă ai mai multe scanuri simultane, se încetinesc reciproc.

### P: "Rezultatele de dimineață vs seară sunt diferite"

**Normal!** Dimineața (9:30-10:30 ET) ai cele mai multe acțiuni cu mișcări mari. Spre seară (15:00+), volumul scade și mai puține acțiuni trec filtrele.

### P: "Score-ul unei acțiuni s-a schimbat dramatic"

**Normal!** Scorul se recalculează la fiecare scanare cu datele LIVE. Dacă o acțiune a continuat să crească (change +5% → +15%), scorul va crește proporțional.

### P: "Coloanele FLOAT, CAP, INS nu apar" 🆕

**Explicație:** Aceste coloane necesită **FMP API Key**. Dacă nu ai setat key-ul, scannerul funcționează normal dar fără date FMP. Setează key-ul la API & WALLET → FMP API KEY → SAVE FMP KEY.

### P: "Văd 'FMP limit reached' în log" 🆕

**Explicație:** Ai depășit 250 API calls/zi. Soluții:
1. Crește intervalul Auto-Scan la 30+ min
2. Datele cached rămân valide — scannerează continuă cu cache
3. Mâine counter-ul se resetează
4. Opțional: upgrade la planul Starter FMP ($19/mo) pentru 300 calls/minut

---

## 18. GLOSAR DE TERMENI

| Termen | Definiție |
|--------|----------|
| **Ask** | Cel mai mic preț la care cineva vinde acțiunea |
| **Bid** | Cel mai mare preț la care cineva cumpără acțiunea |
| **Catalyst** | Eveniment sau știre care cauzează mișcarea prețului |
| **Change %** | Diferența procentuală între prețul curent și close-ul de ieri |
| **Day Trading** | Cumpărare și vânzare în aceeași zi |
| **Delta (Δ)** | Diferența de scor între două scanări consecutive |
| **Gap** | Diferența între open-ul de azi și close-ul de ieri |
| **Gap-up** | Acțiunea a deschis mai sus decât a închis ieri |
| **Gap-down** | Acțiunea a deschis mai jos decât a închis ieri |
| **Large Cap** | Companie cu capitalizare de piață peste $10 miliarde |
| **Lichiditate** | Cât de ușor poți cumpăra/vinde fără a afecta prețul |
| **Mid Cap** | Companie cu capitalizare între $2-10 miliarde |
| **Momentum** | Tendința prețului de a continua în aceeași direcție |
| **New Mover** | Acțiune care apare pentru prima dată în rezultatele scanării |
| **Penny Stock** | Acțiune cu prețul sub $5 |
| **Preset** | Configurație predefinită care setează toți parametrii cu un click |
| **RVOL** | Relative Volume — volumul de azi împărțit la volumul de ieri |
| **Score** | Scor calculat (0-100) bazat pe change, rvol, gap și volume |
| **Score Mode** | Modul de ponderare al componentelor scorului |
| **Short Selling** | Vânzarea unei acțiuni pe care nu o deții (profiți din scădere) |
| **Slippage** | Diferența între prețul dorit și prețul efectiv de execuție |
| **Small Cap** | Companie cu capitalizare sub $2 miliarde |
| **Snapshot** | Captură instantanee a prețului, volumului și altor date |
| **Spread** | Diferența dintre Bid și Ask |
| **Swing Trading** | Tranzacții ținute 1-5 zile |
| **Toast** | Notificare temporară care apare și dispare automat |
| **Volume** | Numărul total de acțiuni tranzacționate |
| **Float** 🆕 | Numărul de acțiuni disponibile public pentru tranzacționare |
| **Market Cap** 🆕 | Capitalizarea de piață = Preț × Total Acțiuni |
| **Insider Trading** 🆕 | Tranzacții făcute de persoane din interiorul companiei (CEO, CFO, etc.) |
| **Insider BUY** 🆕 | CEO/CFO/director cumpără acțiuni cu bani proprii — semnal bullish |
| **Insider SELL** 🆕 | CEO/CFO/director vinde acțiuni — poate fi normal (diversificare) |
| **Earnings** 🆕 | Raportul financiar trimestrial al companiei |
| **FMP** 🆕 | Financial Modeling Prep — serviciu API gratuit pentru date fundamentale |
| **Micro Cap** 🆕 | Companie cu market cap sub $300M |
| **Small Cap** 🆕 | Companie cu market cap $300M - $2B |
| **SEC Form 4** 🆕 | Formularul SEC care raportează insider trades |
| **Price Target** 🆕 | Prețul țintă estimat de analiștii de pe Wall Street |
| **Upside** 🆕 | Diferența procentuală între prețul curent și price target |

---

## REFERINȚE TEHNICE

### Fișiere relevante

| Fișier | Ce conține |
|--------|-----------|
| `src-tauri/src/scanner.rs` | Motorul de scanare, filtrare și scoring |
| `src-tauri/src/commands/scanner_cmd.rs` | Comanda Tauri `run_market_scan` |
| `src-tauri/src/commands/settings.rs` | Salvarea parametrilor de scanare |
| `src-tauri/src/loops/ai_scan_loop.rs` | Auto-scan loop cu weekend guard |
| `src/titan_watchlist.js` | Frontend: SCAN button, presets, DIFF ALERT, NEW MOVER |
| `src/titan_manual_trade.js` | Restaurarea setărilor la pornire |
| `src/index.html` | HTML pentru panoul Market Discovery |
| `src/styles.css` | Stiluri cyber-HUD |

### Structuri de date (Rust)

```rust
// Parametrii de scanare (salvați în backend)
struct ScanParams {
    price_min: f64,             // Preț minim filtru
    price_max: f64,             // Preț maxim filtru
    volume_min: f64,            // Volum minim (absolut, ex: 500000)
    change_min: f64,            // Change % minim
    rvol_min: f64,              // RVOL minim
    gap_min: f64,               // Gap % minim
    require_catalyst: bool,     // Necesită catalizator?
    max_candidates: usize,      // Nr maxim rezultate
    scan_interval_minutes: u32, // Interval auto-scan (minute)
    allow_negative_change: bool, // Permite change negativ (shorts)
    score_preset: String,       // "balanced"|"momentum"|"volume_hunter"|"gap_scanner"
    float_max: f64,             // Max float milioane (0=dezactivat)
    cap_max: f64,               // Max market cap miliarde (0=dezactivat)
    tiered_scan_enabled: bool,  // S15: Activează tiered scan pre-ranking
    tier1_count: usize,         // S15: Top N by dollar volume (default 50)
    tier2_count: usize,         // S15: Următoarele M candidați (default 100)
    exclude_earnings: bool,     // 🆕 Exclude acțiuni cu earnings ±3 zile
}

// Rezultatul scanării (returnat la frontend)
struct ScanResult {
    symbol: String,             // ex: "AAPL"
    price: f64,                 // ex: 175.50
    change_percent: f64,        // ex: 5.2
    volume: f64,                // ex: 12500000
    rvol: f64,                  // ex: 3.5
    gap_percent: f64,           // ex: 2.1
    score: f64,                 // ex: 67.3
    strategy: String,           // "DISCOVERY_V8"
    float_shares: f64,          // 🆕 ex: 8.2 (milioane)
    market_cap: f64,            // 🆕 ex: 120.5 (milioane)
    sector: String,             // 🆕 ex: "Technology"
    insider_signal: String,     // 🆕 "BUY", "SELL", sau ""
    insider_detail: String,     // 🆕 "Tim Cook BUY 50000sh @$182.50"
    has_earnings_soon: bool,    // 🆕 true dacă earnings ±3 zile
    analyst_upside_pct: f64,    // 🆕 ex: 45.2 (upside %)
}
```

### Cache-uri FMP (Backend)

```rust
FLOAT_CACHE    → HashMap<Symbol, (float_m, Instant)>    // TTL: 24h
PROFILE_CACHE  → HashMap<Symbol, (ProfileData, Instant)> // TTL: 24h  
INSIDER_CACHE  → HashMap<Symbol, (InsiderSignal, Instant)> // TTL: 24h
EARNINGS_CACHE → Option<(Instant, HashMap<Symbol, bool>)> // TTL: 12h (1 call!)
TARGETS_CACHE  → HashMap<Symbol, (upside_pct, Instant)>  // TTL: 7 zile
```
```

---

*Titan HFT System — Manual Market Discovery, Martie 2026*  
*Versiune: V8 FMP-Enriched*  
*Surse date: Alpaca Markets + Financial Modeling Prep (FMP)*

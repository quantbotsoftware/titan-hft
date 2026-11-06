# 🤖 GPU NEXUS — Manual de Utilizare

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


## Cuprins

1. [Ce este GPU Nexus?](#ce-este-gpu-nexus)
2. [Cele 3 Moduri de Strategie](#cele-3-moduri-de-strategie)
3. [Panoul GPU Nexus (UI)](#panoul-gpu-nexus-ui)
4. [Cum Funcționează Agenții](#cum-funcționează-agenții)
5. [Slider-ul Fusion Alpha (α)](#slider-ul-fusion-alpha-α)
6. [Interpretarea Consensului](#interpretarea-consensului)
7. [Sfaturi de Utilizare](#sfaturi-de-utilizare)

---

## Ce este GPU Nexus?

GPU Nexus este un **sistem masiv multi-agent** care rulează **150.000 de agenți autonomi de trading** direct pe placa grafică (GPU) folosind tehnologia WGPU. Fiecare agent ia decizii independente de buy/sell bazate pe datele de piață în timp real, iar rezultatul final este un **consens agregat** care indică direcția dominantă a pieței.

### De ce 150.000 de agenți?

- **Diversitate** — Fiecare agent are un "ADN de risc" unic generat aleator
- **Robusteț** — Mii de opinii independente reduc zgomotul și semnalele false
- **Viteză** — GPU-ul procesează toți agenții în paralel, de obicei sub 500μs

---

## Cele 3 Moduri de Strategie

Aplicația oferă **3 moduri** de operare, selectabile din panoul Nexus:

### 🧠 ORACLE (Mod Implicit)

- Folosește **exclusiv** motorul matematic existent (Kalman, Lyapunov, Wavelet, VPIN etc.)
- GPU Nexus este **dezactivat** — niciun calcul GPU suplimentar
- **Când să-l folosești**: Dacă vrei strategie conservatoare, dovedită, bazată pe analiză matematică pură

### 🤖 GPU (GPU Nexus)

- Folosește **exclusiv** consensul celor 150K agenți GPU
- Motorul matematic Oracle continuă să ruleze dar **nu influențează** deciziile
- **Când să-l folosești**: Dacă vrei să testezi performanța agenților GPU independent, sau dacă piața este foarte volatilă și vrei diversitate maximă de opinii

### ⚡ FUSION (Mod Hibrid)

- **Combină ambele strategii** folosind formula:

  ```
  Scor Final = α × Scor Oracle + (1 − α) × Scor GPU Nexus
  ```

- Slider-ul **Fusion α** controlează ponderea fiecărei strategii
- **Când să-l folosești**: Recomandat pentru utilizare zilnică — obții beneficiile ambelor sisteme

---

## Panoul GPU Nexus (UI)

```
┌─────────────────────────────────────────────────────────────┐
│  🤖 GPU NEXUS — 150K AGENTS        [🧠 ORACLE] [🤖 GPU] [⚡ FUSION]  │
│  ┌──────────────────────┬───────────────────────────┐       │
│  │ ██████ 55.3% BUY ██  │ ████ 44.7% SELL ████████ │  BUY  │
│  └──────────────────────┴───────────────────────────┘       │
│  AGENTS: 150,000   GPU: 342μs   MODE: FUSION               │
│  ⚡ FUSION α  ═══════════●══════════  0.50  GPU ◄ ► ORACLE  │
└─────────────────────────────────────────────────────────────┘
```

### Elemente

| Element | Descriere |
|---------|-----------|
| **Toggle 3-way** | Butoanele ORACLE / GPU / FUSION — click pentru a schimba modul |
| **Bara BUY/SELL** | Bară vizuală cu procentajul BUY (verde) vs SELL (roșu) |
| **Eticheta Consens** | STRONG_BUY, BUY, NEUTRAL, SELL, STRONG_SELL |
| **AGENTS** | Numărul total de agenți activi (150,000) |
| **GPU** | Latența ultimului dispatch GPU în microsecunde (μs) |
| **MODE** | Modul curent activ |
| **Fusion α slider** | Apare doar în modul FUSION — controlează ponderea |

---

## Cum Funcționează Agenții

Cei 150.000 de agenți sunt împărțiți în **3 tipuri de strategie**:

### 1. 📈 Momentum (Agenți 0 – 49.999)

- Urmăresc **direcția trendului** (velocity)
- Cumpără când prețul accelerează în sus
- Vând când prețul accelerează în jos
- Formula: `signal = velocity × risk_dna × 100.0`

### 2. 📊 Mean-Reversion (Agenți 50.000 – 99.999)

- Caută **reveniri la medie**
- Cumpără când prețul este sub medie (subextins)
- Vând când prețul este supra-extins
- Formula: `signal = −deviation × risk_dna × 50.0`

### 3. 🚀 Breakout (Agenți 100.000 – 149.999)

- Detectează **spargeri de volatilitate**
- Cumpără la breakout ascendent cu volatilitate mare
- Vând la breakout descendent
- Formula: `signal = velocity × volatility × risk_dna × 200.0`

### ADN-ul de Risc

Fiecare agent are un **risk_dna** unic (valoare între 0.1 și 1.0) care determină cât de agresiv reacționează la semnale. Agenții cu risk_dna mare sunt mai agresivi; cei cu risk_dna mic sunt mai conservatori. Acest ADN este generat aleator la inițializare.

### Procesul de Vot

1. Fiecare agent calculează un semnal bazat pe strategia sa
2. Dacă semnalul > **prag** → votează **BUY** (cu ponderea = risk_dna × 100)
3. Dacă semnalul < **−prag** → votează **SELL** (cu ponderea = risk_dna × 100)
4. Dacă semnalul este între praguri → agentul rămâne **neutru** (nu votează)
5. Voturile sunt agregate atomic pe GPU și readback-ul conține doar 8 bytes (buy_weight + sell_weight)

---

## Slider-ul Fusion Alpha (α)

Slider-ul α apare **doar în modul FUSION** și controlează ponderea relativă:

| Valoare α | Comportament |
|-----------|-------------|
| **0.00** | 100% GPU Nexus, 0% Oracle — doar agenții decid |
| **0.25** | 75% GPU + 25% Oracle |
| **0.50** | Echilibru 50/50 (recomandat pentru început) |
| **0.75** | 25% GPU + 75% Oracle |
| **1.00** | 0% GPU, 100% Oracle — doar math-ul decide |

### Recomandări

- **Start**: Începe cu α = 0.50 (echilibru perfect)
- **Piață trending**: Crește α spre 0.70+ (Oracle detectează mai bine trend-urile lungi)
- **Piață laterală/choppy**: Scade α spre 0.30 (GPU-ul are mai multă diversitate de opinii)
- **Volatilitate extremă**: α = 0.40 (GPU Breakout agents excelează în aceste condiții)

---

## Interpretarea Consensului

Eticheta de consens se calculează din procentajul BUY/SELL al agenților:

| Consens | Condiție | Semnificație |
|---------|----------|-------------|
| 🟢 **STRONG BUY** | BUY > 70% | Majoritate covârșitoare bullish — semnal puternic de cumpărare |
| 🟢 **BUY** | BUY > 55% | Majoritate moderată bullish — semnal de cumpărare |
| ⚪ **NEUTRAL** | 45% – 55% | Piață indecisă — agenții sunt împărțiți, prudență |
| 🔴 **SELL** | SELL > 55% | Majoritate moderată bearish — semnal de vânzare |
| 🔴 **STRONG SELL** | SELL > 70% | Majoritate covârșitoare bearish — semnal puternic de vânzare |

### Cum să citești bara

- **Verde mare, roșu mic** → Agenții sunt majoritar bullish
- **Roșu mare, verde mic** → Agenții sunt majoritar bearish
- **Aproximativ egal** → Piață fără direcție clară, evită tranzacțiile

---

## Sfaturi de Utilizare

### ✅ Best Practices

1. **Începe cu modul ORACLE** pentru a te obișnui cu interfața
2. **Treci pe FUSION cu α = 0.50** când vrei să testezi GPU Nexus
3. **Monitorizează GPU μs** — dacă depășește 2000μs, GPU-ul este supraîncărcat
4. **Nu ignora NEUTRAL** — când agenții sunt 50/50, piața este indecisă
5. **Folosește FUSION pentru trading real** — combinația ambelor strategii este mai robustă

### ⚠️ Atenție

- GPU Nexus funcționează cel mai bine cu **date live** (tick-uri reale)
- Consensul se actualizează la **fiecare tick** (~100ms)
- Primele câteva tick-uri pot arăta NEUTRAL până când agenții acumulează date
- **STRONG_BUY/STRONG_SELL** sunt semnale rare — când apar, au semnificație mare

### 🔧 Troubleshooting

| Problemă | Soluție |
|----------|---------|
| GPU: --μs (nu se actualizează) | Modul este pe ORACLE — treci pe GPU sau FUSION |
| Sempre NEUTRAL | Normal la primele secunde; dacă persistă, verifică feed-ul de date |
| Latență GPU > 5000μs | Închide alte aplicații GPU-intensive |
| Batonul nu se mișcă | Verifică că scan-ul rulează (LIVE FEED activ) |

---

## Arhitectură Tehnică (Rezumat)

```
Tick (100ms) → main.rs (tick loop)
                  ↓
              nexus_dispatch() → GPU Write (16B uniform)
                  ↓
              WGSL Kernel → 150K agents × 3 strategii
                  ↓
              Atomic Vote → atomicAdd(buy_weight, sell_weight)
                  ↓
              8B Readback → NexusIntentRaw { buy: u32, sell: u32 }
                  ↓
              Consens → "STRONG_BUY" / "BUY" / "NEUTRAL" / "SELL" / "STRONG_SELL"
                  ↓
              Frontend → nexus-update event → Bara BUY/SELL + Label
```

**Fișiere cheie:**

- `gpu.rs` — NexusAgent, nexus_dispatch(), readback logic
- `nexus_kernel.wgsl` — Compute shader cu cele 3 strategii
- `main.rs` — StrategyMode enum, Tauri commands, tick loop integration
- `index.html` — Panoul Nexus UI + JavaScript handlers

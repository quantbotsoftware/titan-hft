# 🔮 TITAN ORACLE — Documentație Tehnică Completă

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


## Ghid al Indicatorului Predictiv cu 7 Straturi

TITAN ORACLE este un **super-indicator predictiv** care combină 7 algoritmi matematici avansați într-un singur scor (`oracle_score`) între **-100** și **+100**. Spre deosebire de indicatorii tradiționali (EMA, RSI, MACD) care sunt **lagging** (privesc în trecut), TITAN ORACLE este **leading** — prezice viitorul.

---

## Arhitectura Generală

```
┌─────────────────────────────────────────────────────────────────┐
│                        DATE INTRARE                             │
│            candles[] → close, high, low, volume                 │
└─────────────────────┬───────────────────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    ▼                 ▼                 ▼
┌────────┐      ┌──────────┐     ┌──────────┐
│ LAYER 1│      │ LAYER 2  │     │ LAYER 3  │
│ Kalman │      │ Entropy  │     │  Hurst   │
│ Filter │      │  Flow    │     │ Exponent │
└───┬────┘      └────┬─────┘     └────┬─────┘
    │                │                │
    ▼                ▼                ▼
┌────────┐      ┌──────────┐     ┌──────────┐     ┌──────────┐
│ LAYER 4│      │ LAYER 5  │     │ LAYER 6  │     │ LAYER 7  │
│Lyapunov│      │ Wavelet  │     │  VPIN    │     │Changepoint│
│ Chaos  │      │ Denoise  │     │  Volume  │     │ Bayesian │
└───┬────┘      └────┬─────┘     └────┬─────┘     └────┬─────┘
    │                │                │                 │
    └────────────────┼────────────────┼─────────────────┘
                     ▼
            ┌────────────────┐
            │  FUSION LAYER  │
            │  Ponderi       │
            │  Adaptive      │
            └───────┬────────┘
                    ▼
         oracle_score [-100, +100]
         confidence   [0.0, 1.0]
         regime: Trending/MeanReverting/Chaotic
```

---

## LAYER 1: Filtrul Kalman (Kalman Adaptive Predictor)

### Ce face?
Prezice **prețul viitor** și **viteza de schimbare** a prețului. Filtrul Kalman este utilizat în navigarea rachetelor, GPS și piloți automați — acum îl folosim pentru piață.

### Model Matematic

Starea sistemului este un vector 2D:
```
x = [preț, viteză]
```

**Ecuația de Stare (Predicție):**
```
x_predicted = F · x_previous
```
unde matricea de tranziție F este:
```
F = | 1  dt |    (dt = 1 bar)
    | 0   1 |
```
Aceasta spune: `preț_nou = preț_vechi + viteză × dt`

**Ecuația de Observație (Corecție):**
```
z = H · x + noise
```
unde `H = [1, 0]` (observăm doar prețul, nu viteza).

**Update Kalman în 5 pași:**
1. **Predict State:** `x̂ = F · x`
2. **Predict Covariance:** `P̂ = F · P · Fᵀ + Q`
3. **Kalman Gain:** `K = P̂ · Hᵀ · (H · P̂ · Hᵀ + R)⁻¹`
4. **Update State:** `x = x̂ + K · (z - H · x̂)`
5. **Update Covariance:** `P = (I - K · H) · P̂`

### Parametri
| Parametru | Valoare | Descriere |
|-----------|---------|-----------|
| `Q` (process noise) | `[[0.01, 0], [0, 0.01]]` | Cât de mult se schimbă piața "intern" |
| `R` (measurement noise) | `1.0` | Zgomotul observațiilor (prețul brut) |
| `P₀` (initial covariance) | `[[1, 0], [0, 1]]` | Incertitudinea inițială |

### Output
| Câmp | Tip | Descriere |
|------|-----|-----------|
| `predicted_price` | `f64` | Prețul prezis pentru bara următoare |
| `predicted_velocity` | `f64` | Viteza de schimbare (pozitivă = crește, negativă = scade) |
| `uncertainty` | `f64` | Incertitudinea predicției (P[0][0]) |

### Semnal de Trading
- `velocity > 0` → **BULLISH** (prețul accelerează în sus)
- `velocity < 0` → **BEARISH** (prețul decelărează/scade)
- Semnalul brut: `kalman_signal = (velocity × 1000).clamp(-1, +1)`

---

## LAYER 2: Shannon Entropy Flow Regime Detector

### Ce face?
Detectează **regimul pieței**: Trending, Mean Reverting sau Chaotic. Entropia Shannon măsoară "dezordinea" din returnuri — piețele ordonate (trending) au entropie joasă, cele haotice au entropie maximă.

### Formula Matematică

**1. Calculul Returnurilor:**
```
r[i] = (close[i] - close[i-1]) / close[i-1]
```

**2. Distribuția pe Bins:**
Returnurile sunt clasificate în 10 bin-uri (histogramă). Entropia Shannon:
```
H = -Σ p(i) · log₂(p(i))    pentru toate bin-urile cu p(i) > 0
```
unde `p(i)` = probabilitatea de a fi în bin-ul i.

**3. Normalizare:**
```
H_normalized = H / log₂(num_bins)    → [0, 1]
```
- `H ≈ 0` = Toate returnurile sunt identice (trend perfect)
- `H ≈ 1` = Returnuri complet aleatorii (noise pur)

**4. Detectarea Regimului prin Autocorrelație:**
```
autocorr = Σ(r[i] · r[i-1]) / Σ(r[i]²)
```
- `autocorr > 0.3` → **TRENDING** (mișcările se propagă)
- `autocorr < -0.3` → **MEAN REVERTING** (mișcările se inversează)
- `altfel` → **CHAOTIC** (imprevizibil)

### Parametri
| Parametru | Valoare | Descriere |
|-----------|---------|-----------|
| `window` | 50 | Numărul de bare analizate |
| `num_bins` | 10 | Numărul de bin-uri pentru histograma Shannon |
| `autocorr threshold` | ±0.3 | Pragul de clasificare a regimului |

### Output
| Câmp | Tip | Descriere |
|------|-----|-----------|
| `entropy` | `f64` | Entropia normalizată [0, 1] |
| `regime` | `MarketRegime` | Trending / MeanReverting / Chaotic / Unknown |
| `clarity_score` | `f64` | `1 - entropy` (claritatea semnalului) |

---

## LAYER 3: Exponentul Hurst (Fractal Memory Analyzer)

### Ce face?
Detectează dacă piața are **memorie** (trendul va continua) sau dacă evoluează aleatoriu. Este bazat pe analiza R/S (Rescaled Range) inventată de hidrologul Harold Edwin Hurst.

### Formula Matematică

**Metoda R/S (Rescaled Range):**

Pentru fiecare dimensiune de sub-serie `n`:
1. Împarte seria de N elemente în sub-serii de lungime `n`
2. Pentru fiecare sub-serie:
   - Calculează media: `μ = Σx / n`
   - Calculează deviația cumulativă: `Y[i] = Σ(x[j] - μ), j=1..i`
   - Calculează range-ul: `R = max(Y) - min(Y)`
   - Calculează deviația standard: `S = sqrt(Σ(x[j] - μ)² / n)`
   - `R/S = R / S` (dacă S > 0)
3. Calculează media R/S peste toate sub-seriile

**Regresia log-log:**
```
log(R/S) = H · log(n) + c
```
Exponentul Hurst `H` este panta regresiei liniare pe scala log-log.

### Interpretare
| Hurst | Semnificație |
|-------|-------------|
| `H > 0.5` | **Persistent** — trendul va continua |
| `H = 0.5` | **Random Walk** — fără memorie |
| `H < 0.5` | **Anti-persistent** — trendul se va inversa |

### Semnal
```
persistence_signal = (H - 0.5) × 2.0    → [-1, +1]
```
Se multiplică cu direcția ultimei mișcări:
- Dacă prețul a crescut recent ȘI `H > 0.5` → persistence_signal devine pozitiv (continuare bullish)
- Dacă prețul a scăzut recent ȘI `H > 0.5` → persistence_signal devine negativ (continuare bearish)

### Parametri
| Parametru | Valoare | Descriere |
|-----------|---------|-----------|
| `window` | 128 | Numărul maxim de bare analizate |
| `min_group_sizes` | [8, 16, 32, 64, 128] | Dimensiunile sub-seriilor |

---

## LAYER 4: Exponentul Lyapunov (Chaos Detector)

### Ce face?
Măsoară cât de **haotică sau predictibilă** este piața. Dacă traiectorii similare diverge rapid → piața este haotică și impredictibilă. Dacă rămân similare → piața este stabilă și predictibilă.

### Formula Matematică

**1. Reconstrucția spațiului de faze (Takens' Theorem):**
```
v[i] = [x[i], x[i+lag], x[i+2·lag], ..., x[i+(dim-1)·lag]]
```
cu `dim = 3`, `lag = 2`

**2. Găsirea vecinilor:**
Pentru fiecare vector v[i], se găsește cel mai apropiat vecin v[j] (cu separare temporală minimă = `2 × lag`):
```
dist_0 = ||v[i] - v[j]||₂    (distanța Euclidiană)
```

**3. Evoluția divergenței:**
Se urmărește cum evoluează distanța după `k` pași:
```
dist_k = ||v[i+k] - v[j+k]||₂
```

**4. Exponentul Lyapunov:**
```
λ = (1/N) · Σ ln(dist_k / dist_0) / k
```

### Interpretare
| λ | Semnificație |
|---|-------------|
| `λ > 0.05` | **HAOTIC** — predicțiile sunt nesigure |
| `λ ≈ 0` | **STABIL** — piața este predictibilă |
| `λ < 0` | **CONVERGENT** — foarte predictibilă |

### Output
```
predictability = exp(-λ × 10).clamp(0, 1)
```
- `predictability ≈ 1.0` → Piața este predictibilă
- `predictability ≈ 0.0` → Piața este haotică

### Parametri
| Parametru | Valoare | Descriere |
|-----------|---------|-----------|
| `embedding_dim` | 3 | Dimensiunea spațiului de faze |
| `lag` | 2 | Lag-ul pentru embedding |
| `evolution_steps` | 5 | Câți pași se urmărește divergența |

---

## LAYER 5: Descompunerea Haar Wavelet

### Ce face?
Separă semnalul prețului în **componenta de trend** (frecvență joasă) și **zgomot** (frecvență înaltă). Practic "denoisează" prețul, arătând trendul real ascuns sub fluctuații.

### Formula Matematică

**Transformata Haar la nivel l:**

Semnalul este împărțit recursiv:
```
approx[i] = (signal[2i] + signal[2i+1]) / √2    (aproximare — trend)
detail[i] = (signal[2i] - signal[2i+1]) / √2     (detaliu — zgomot)
```

Acest proces se repetă pe `max_levels = min(6, log₂(N))` niveluri.

**Reconstrucție (Denoise):**
Se zerorizează detaliile de frecvență înaltă (primele 2 niveluri) și se aplică transformata inversă:
```
signal_reconst[2i]     = (approx[i] + detail[i]) / √2
signal_reconst[2i+1]   = (approx[i] - detail[i]) / √2
```

**Noise Ratio:**
```
noise_ratio = (energy_detail_level_1 + energy_detail_level_2) / total_energy
```
unde `energy = Σ d[i]²`

### Output
| Câmp | Tip | Descriere |
|------|-----|-----------|
| `denoised_trend` | `f64` | Prețul denoised (trend curat) |
| `trend_direction` | `f64` | Panta ultimelor 5 valori denoised |
| `dominant_cycle` | `usize` | Ciclul dominant (2^nivel) |
| `noise_ratio` | `f64` | Proporția de zgomot [0, 1] |

---

## LAYER 6: VPIN (Volume-Synchronized Probability of Informed Trading)

### Ce face?
Detectează activitatea **smart money** (bani informați). VPIN mare = cineva cu informații privilegiate tranzacționează agresiv. Dezvoltat la Universitatea Cornell pentru detectarea flash crash-urilor.

### Formula Matematică

**1. Clasificarea volumului (Bulk Volume Classification):**
```
buy_fraction = (close - low) / (high - low)
buy_volume = volume × buy_fraction
sell_volume = volume × (1 - buy_fraction)
```

**2. Volumul în Bucket-uri:**
Volumul total se împarte în `n_buckets` bucket-uri egale:
```
bucket_volume = total_volume / n_buckets
```

Fiecare bucket acumulează volum de buy și sell până la umplere.

**3. Calculul VPIN:**
```
VPIN = Σ|buy_bucket[i] - sell_bucket[i]| / Σ(buy_bucket[i] + sell_bucket[i])
```
peste ultimele `n_buckets` bucket-uri.

### Output
| Câmp | Tip | Descriere |
|------|-----|-----------|
| `vpin` | `f64` | Probabilitatea de trading informat [0, 1] |
| `toxicity_alert` | `bool` | `true` dacă VPIN > 0.6 (smart money activ!) |
| `buy_pressure` | `f64` | Presiunea de cumpărare [0, 1] |

### Interpretare
| VPIN | Semnificație |
|------|-------------|
| `< 0.3` | Normal — fără activitate smart money |
| `0.3 - 0.6` | Moderat — atenție |
| `> 0.6` | **ALERT** — smart money tranzacționează agresiv |

---

## LAYER 7: Bayesian Online Changepoint Detection (BOCPD)

### Ce face?
Detectează **schimbări structurale** în timp real. Bazat pe paper-ul Adams & MacKay (2007). Când piața trece de la un regim la altul (de ex: din trend în crash), BOCPD detectează momentul exact.

### Formula Matematică

**Run Length = câte bare au trecut de la ultimul changepoint.**

La fiecare observație `x_t`:

**1. Probabilitatea Predictivă (Student-t):**
```
π(x_t | r) = Student-t(x_t; μ_n, σ²_n, 2α_n)
```
cu parametrii Normal-Inverse-Gamma actualizați online.

**2. Creștere vs Changepoint:**
```
growth_prob[r+1]     = joint[r] × (1 - 1/λ)
changepoint_prob[0] += joint[r] × (1/λ)
```
unde `λ = hazard_lambda` (cât de des apar changepoint-uri).

**3. Normalizare:**
```
P(r_t | data) = probs / Σ probs
```

**4. MAP Run Length:**
```
best_run_length = argmax P(r_t | data)
```

### Parametri
| Parametru | Valoare | Descriere |
|-----------|---------|-----------|
| `hazard_lambda` | 100.0 | Intervalul mediu între changepoint-uri (bare) |
| `μ₀` | 0.0 | Media prior |
| `κ₀` | 1.0 | Forța prior-ului |
| `α₀` | 1.0 | Forma prior |
| `β₀` | 0.0001 | Scala prior |

### Output
| Câmp | Tip | Descriere |
|------|-----|-----------|
| `changepoint_probability` | `f64` | Probabilitatea unui changepoint ACUM [0, 1] |
| `run_length` | `usize` | Câte bare de la ultimul changepoint |
| `regime_stable` | `bool` | `true` dacă changepoint_prob < 0.3 |

---

## FUSION LAYER — Combinarea Adaptivă

### Ce face?
Combină semnalele tuturor celor 7 straturi cu **ponderi adaptive** — straturile care sunt mai sigure primesc pondere mai mare.

### Formula de Fuziune

**1. Semnale Individuale → [-1, +1]:**
```
kalman_signal  = (velocity × 1000).clamp(-1, +1)
hurst_signal   = persistence_signal                    (deja [-1, +1])
wavelet_signal = (trend_direction × 100).clamp(-1, +1)
vpin_signal    = (buy_pressure - 0.5) × 2             (map [0,1] → [-1,+1])
```

**2. Ponderi Fixe (Bază):**
| Strat | Pondere | Justificare |
|-------|---------|-------------|
| Kalman | 0.30 | Cel mai direct predictor (preț + viteză) |
| Wavelet | 0.25 | Trend denoised de calitate |
| Hurst | 0.20 | Memorie / persistență |
| VPIN | 0.15 | Activitate smart money |
| Entropy echo | 0.10 | Claritate × Kalman (confirmare) |

**3. Meta-Confidence (Modificator Global):**
```
meta_confidence = (lyapunov.predictability × entropy.clarity × stability_factor).clamp(0, 1)
```
unde `stability_factor = 1.0` dacă regimul e stabil, `0.3` dacă nu.

**4. Regime Boost:**
```
Trending      → × 1.3  (boost — semnalele direcționale sunt de încredere)
MeanReverting → × 0.7  (reduce — trendul se poate inversa)
Chaotic       → × 0.3  (penalizare severă — imprevizibil)
```

**5. Scorul Final:**
```
raw = w_kalman × kalman + w_wavelet × wavelet + w_hurst × hurst + w_vpin × vpin + w_entropy × kalman × clarity
oracle_score = (raw × regime_boost × meta_confidence × 100).clamp(-100, +100)
```

**6. Confidence finală:**
```
confidence = meta_confidence × max(0, 1 - noise_ratio)
```

---

## Interpretarea oracle_score

| Scor | Interpretare | Acțiune |
|------|-------------|---------|
| **+15 to +100** | Bullish | 🟢 LONG |
| **-15 to +15** | Neutru / Incert | ⚪ HOLD |
| **-100 to -15** | Bearish | 🔴 SHORT |

> **NOTĂ OPTIMIZARE V2:** Calculul istoric a fost optimizat de la O(n²) la **O(n)** folosind serii pre-calculate și fuziune punct-cu-punct. Aceasta permite calculul instantaneu pentru mii de bare fără timeout. Pragurile au fost ajustate la ±15 pentru a reflecta scala reală a scorurilor compuse.

### Chaos Kill Switch
Dacă `confidence < 0.15`, piața este în **haos complet**:
- Se aplică o penalizare de **-50 puncte** la scorul de strategie
- Aceasta blochează efectiv orice intrare în tranzacție
- Motivul: în haos, NICIUN indicator nu este de încredere

---

## Diagrama Fluxului de Date

```
Candle[] ─── close[] ──┬── Kalman Filter ──── predicted_price, velocity
                       ├── Entropy Regime ──── entropy, regime, clarity
                       ├── Hurst R/S ───────── hurst, persistence
                       ├── Lyapunov ────────── exponent, predictability
                       └── Wavelet Haar ────── denoised_trend, noise_ratio

Candle[] ─── volume ───── VPIN ─────────────── vpin, buy_pressure, toxicity
            close[] ───── BOCPD ────────────── changepoint_prob, run_length

                              │
                              ▼
                        FUSION LAYER
                    adaptive weights × regime
                              │
                              ▼
                     oracle_score [-100, +100]
                     confidence  [0.0, 1.0]
                     dominant_regime
```

---

## Avantaje față de Indicatorii Tradiționali

| Aspect | Indicatori Clasici (EMA, RSI) | TITAN ORACLE |
|--------|------------------------------|--------------|
| **Timing** | Lagging (privesc în trecut) | Leading (prezic viitorul) |
| **Noise** | Sensibili la zgomot | Wavelet denoisează semnalul |
| **Regim** | Nu detectează | Entropy + Changepoint detectează |
| **Smart Money** | Invizibil | VPIN detectează activitate informată |
| **Haos** | Ignorat | Lyapunov + Kill Switch protejează |
| **Memorie** | Nu analizează | Hurst detectează persistența |
| **Adaptivitate** | Parametri ficși | Ponderi adaptive pe baza condițiilor |

---

## Fișiere Sursă

| Fișier | Descriere |
|--------|-----------|
| `src-tauri/crates/titan-indicators/src/oracle.rs` | Implementarea completă (toate 7 straturile + fuziune + teste unitare) |
| `src-tauri/crates/titan-indicators/benches/bench_oracle.rs` | Benchmark-uri Criterion pentru pipeline-ul Oracle |
| `src-tauri/src/titan_oracle.rs` | Re-export al API-ului Oracle din `titan-indicators` în crate-ul principal |
| `src-tauri/src/strategy.rs` | Logica de decizie bazată pe `oracle_score` |
| `src-tauri/src/commands/backtest.rs` | Comanda Tauri `get_oracle_snapshot` (înregistrată în `main.rs`) |

---

*TITAN ORACLE v1.0 — Precizie. Predicție. Profit.* 🔮

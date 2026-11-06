# 🔮 TITAN ORACLE — Complete Technical Documentation

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


## Guide to the 7-Layer Predictive Indicator

TITAN ORACLE is a **predictive super-indicator** that combines 7 advanced mathematical algorithms into a single score (`oracle_score`) ranging from **-100** to **+100**. Unlike traditional indicators (EMA, RSI, MACD), which are **lagging** (they look at the past), TITAN ORACLE is **leading** — it forecasts what comes next.

---

## General Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         INPUT DATA                              │
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
            │  Adaptive      │
            │  Weights       │
            └───────┬────────┘
                    ▼
         oracle_score [-100, +100]
         confidence   [0.0, 1.0]
         regime: Trending/MeanReverting/Chaotic
```

---

## LAYER 1: Kalman Filter (Kalman Adaptive Predictor)

### What does it do?
It predicts the **future price** and the **rate of change** of the price. The Kalman filter is used in missile guidance, GPS and autopilots — here we apply it to the market.

### Mathematical Model

The system state is a 2D vector:
```
x = [price, velocity]
```

**State Equation (Prediction):**
```
x_predicted = F · x_previous
```
where the transition matrix F is:
```
F = | 1  dt |    (dt = 1 bar)
    | 0   1 |
```
This says: `new_price = old_price + velocity × dt`

**Observation Equation (Correction):**
```
z = H · x + noise
```
where `H = [1, 0]` (we observe only the price, not the velocity).

**Kalman update in 5 steps:**
1. **Predict State:** `x̂ = F · x`
2. **Predict Covariance:** `P̂ = F · P · Fᵀ + Q`
3. **Kalman Gain:** `K = P̂ · Hᵀ · (H · P̂ · Hᵀ + R)⁻¹`
4. **Update State:** `x = x̂ + K · (z - H · x̂)`
5. **Update Covariance:** `P = (I - K · H) · P̂`

### Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| `Q` (process noise) | `[[0.01, 0], [0, 0.01]]` | How much the market changes "internally" |
| `R` (measurement noise) | `1.0` | Observation noise (raw price) |
| `P₀` (initial covariance) | `[[1, 0], [0, 1]]` | Initial uncertainty |

### Output
| Field | Type | Description |
|-------|------|-------------|
| `predicted_price` | `f64` | Predicted price for the next bar |
| `predicted_velocity` | `f64` | Rate of change (positive = rising, negative = falling) |
| `uncertainty` | `f64` | Prediction uncertainty (P[0][0]) |

### Trading Signal
- `velocity > 0` → **BULLISH** (price is accelerating upward)
- `velocity < 0` → **BEARISH** (price is decelerating / falling)
- Raw signal: `kalman_signal = (velocity × 1000).clamp(-1, +1)`

---

## LAYER 2: Shannon Entropy Flow Regime Detector

### What does it do?
It detects the **market regime**: Trending, Mean Reverting or Chaotic. Shannon entropy measures the "disorder" of returns — orderly (trending) markets have low entropy, chaotic ones have maximum entropy.

### Mathematical Formula

**1. Return Calculation:**
```
r[i] = (close[i] - close[i-1]) / close[i-1]
```

**2. Bin Distribution:**
Returns are sorted into 10 bins (histogram). Shannon entropy:
```
H = -Σ p(i) · log₂(p(i))    for all bins with p(i) > 0
```
where `p(i)` = probability of falling in bin i.

**3. Normalisation:**
```
H_normalized = H / log₂(num_bins)    → [0, 1]
```
- `H ≈ 0` = All returns are identical (perfect trend)
- `H ≈ 1` = Completely random returns (pure noise)

**4. Regime Detection via Autocorrelation:**
```
autocorr = Σ(r[i] · r[i-1]) / Σ(r[i]²)
```
- `autocorr > 0.3` → **TRENDING** (moves propagate)
- `autocorr < -0.3` → **MEAN REVERTING** (moves reverse)
- `otherwise` → **CHAOTIC** (unpredictable)

### Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| `window` | 50 | Number of bars analysed |
| `num_bins` | 10 | Number of bins for the Shannon histogram |
| `autocorr threshold` | ±0.3 | Regime classification threshold |

### Output
| Field | Type | Description |
|-------|------|-------------|
| `entropy` | `f64` | Normalised entropy [0, 1] |
| `regime` | `MarketRegime` | Trending / MeanReverting / Chaotic / Unknown |
| `clarity_score` | `f64` | `1 - entropy` (signal clarity) |

---

## LAYER 3: Hurst Exponent (Fractal Memory Analyzer)

### What does it do?
It detects whether the market has **memory** (the trend will continue) or evolves randomly. It is based on R/S (Rescaled Range) analysis, invented by the hydrologist Harold Edwin Hurst.

### Mathematical Formula

**R/S (Rescaled Range) Method:**

For each sub-series size `n`:
1. Split the N-element series into sub-series of length `n`
2. For each sub-series:
   - Compute the mean: `μ = Σx / n`
   - Compute the cumulative deviation: `Y[i] = Σ(x[j] - μ), j=1..i`
   - Compute the range: `R = max(Y) - min(Y)`
   - Compute the standard deviation: `S = sqrt(Σ(x[j] - μ)² / n)`
   - `R/S = R / S` (if S > 0)
3. Average R/S over all sub-series

**Log-log regression:**
```
log(R/S) = H · log(n) + c
```
The Hurst exponent `H` is the slope of the linear regression on the log-log scale.

### Interpretation
| Hurst | Meaning |
|-------|---------|
| `H > 0.5` | **Persistent** — the trend will continue |
| `H = 0.5` | **Random Walk** — no memory |
| `H < 0.5` | **Anti-persistent** — the trend will reverse |

### Signal
```
persistence_signal = (H - 0.5) × 2.0    → [-1, +1]
```
Multiplied by the direction of the most recent move:
- If price rose recently AND `H > 0.5` → persistence_signal becomes positive (bullish continuation)
- If price fell recently AND `H > 0.5` → persistence_signal becomes negative (bearish continuation)

### Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| `window` | 128 | Maximum number of bars analysed |
| `min_group_sizes` | [8, 16, 32, 64, 128] | Sub-series sizes |

---

## LAYER 4: Lyapunov Exponent (Chaos Detector)

### What does it do?
It measures how **chaotic or predictable** the market is. If similar trajectories diverge quickly → the market is chaotic and unpredictable. If they stay similar → the market is stable and predictable.

### Mathematical Formula

**1. Phase-space reconstruction (Takens' Theorem):**
```
v[i] = [x[i], x[i+lag], x[i+2·lag], ..., x[i+(dim-1)·lag]]
```
with `dim = 3`, `lag = 2`

**2. Nearest-neighbour search:**
For each vector v[i], find the nearest neighbour v[j] (with a minimum temporal separation of `2 × lag`):
```
dist_0 = ||v[i] - v[j]||₂    (Euclidean distance)
```

**3. Divergence evolution:**
Track how the distance evolves after `k` steps:
```
dist_k = ||v[i+k] - v[j+k]||₂
```

**4. Lyapunov exponent:**
```
λ = (1/N) · Σ ln(dist_k / dist_0) / k
```

### Interpretation
| λ | Meaning |
|---|---------|
| `λ > 0.05` | **CHAOTIC** — predictions are unreliable |
| `λ ≈ 0` | **STABLE** — the market is predictable |
| `λ < 0` | **CONVERGENT** — highly predictable |

### Output
```
predictability = exp(-λ × 10).clamp(0, 1)
```
- `predictability ≈ 1.0` → The market is predictable
- `predictability ≈ 0.0` → The market is chaotic

### Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| `embedding_dim` | 3 | Phase-space dimension |
| `lag` | 2 | Embedding lag |
| `evolution_steps` | 5 | How many steps divergence is tracked |

---

## LAYER 5: Haar Wavelet Decomposition

### What does it do?
It separates the price signal into a **trend component** (low frequency) and **noise** (high frequency). In effect it "denoises" the price, revealing the real trend hidden under the fluctuations.

### Mathematical Formula

**Haar transform at level l:**

The signal is split recursively:
```
approx[i] = (signal[2i] + signal[2i+1]) / √2    (approximation — trend)
detail[i] = (signal[2i] - signal[2i+1]) / √2     (detail — noise)
```

This process is repeated over `max_levels = min(6, log₂(N))` levels.

**Reconstruction (Denoise):**
The high-frequency details (first 2 levels) are zeroed and the inverse transform is applied:
```
signal_reconst[2i]     = (approx[i] + detail[i]) / √2
signal_reconst[2i+1]   = (approx[i] - detail[i]) / √2
```

**Noise Ratio:**
```
noise_ratio = (energy_detail_level_1 + energy_detail_level_2) / total_energy
```
where `energy = Σ d[i]²`

### Output
| Field | Type | Description |
|-------|------|-------------|
| `denoised_trend` | `f64` | Denoised price (clean trend) |
| `trend_direction` | `f64` | Slope of the last 5 denoised values |
| `dominant_cycle` | `usize` | Dominant cycle (2^level) |
| `noise_ratio` | `f64` | Proportion of noise [0, 1] |

---

## LAYER 6: VPIN (Volume-Synchronized Probability of Informed Trading)

### What does it do?
It detects **smart money** (informed) activity. High VPIN = someone with privileged information is trading aggressively. Developed at Cornell University for flash-crash detection.

### Mathematical Formula

**1. Volume classification (Bulk Volume Classification):**
```
buy_fraction = (close - low) / (high - low)
buy_volume = volume × buy_fraction
sell_volume = volume × (1 - buy_fraction)
```

**2. Volume Buckets:**
Total volume is split into `n_buckets` equal buckets:
```
bucket_volume = total_volume / n_buckets
```

Each bucket accumulates buy and sell volume until it is full.

**3. VPIN Calculation:**
```
VPIN = Σ|buy_bucket[i] - sell_bucket[i]| / Σ(buy_bucket[i] + sell_bucket[i])
```
over the last `n_buckets` buckets.

### Output
| Field | Type | Description |
|-------|------|-------------|
| `vpin` | `f64` | Probability of informed trading [0, 1] |
| `toxicity_alert` | `bool` | `true` if VPIN > 0.6 (smart money active!) |
| `buy_pressure` | `f64` | Buying pressure [0, 1] |

### Interpretation
| VPIN | Meaning |
|------|---------|
| `< 0.3` | Normal — no smart-money activity |
| `0.3 - 0.6` | Moderate — caution |
| `> 0.6` | **ALERT** — smart money is trading aggressively |

---

## LAYER 7: Bayesian Online Changepoint Detection (BOCPD)

### What does it do?
It detects **structural changes** in real time. Based on the Adams & MacKay (2007) paper. When the market shifts from one regime to another (e.g. from trend to crash), BOCPD pinpoints the exact moment.

### Mathematical Formula

**Run Length = how many bars have elapsed since the last changepoint.**

For each observation `x_t`:

**1. Predictive Probability (Student-t):**
```
π(x_t | r) = Student-t(x_t; μ_n, σ²_n, 2α_n)
```
with Normal-Inverse-Gamma parameters updated online.

**2. Growth vs Changepoint:**
```
growth_prob[r+1]     = joint[r] × (1 - 1/λ)
changepoint_prob[0] += joint[r] × (1/λ)
```
where `λ = hazard_lambda` (how often changepoints occur).

**3. Normalisation:**
```
P(r_t | data) = probs / Σ probs
```

**4. MAP Run Length:**
```
best_run_length = argmax P(r_t | data)
```

### Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| `hazard_lambda` | 100.0 | Mean interval between changepoints (bars) |
| `μ₀` | 0.0 | Prior mean |
| `κ₀` | 1.0 | Prior strength |
| `α₀` | 1.0 | Prior shape |
| `β₀` | 0.0001 | Prior scale |

### Output
| Field | Type | Description |
|-------|------|-------------|
| `changepoint_probability` | `f64` | Probability of a changepoint NOW [0, 1] |
| `run_length` | `usize` | Bars since the last changepoint |
| `regime_stable` | `bool` | `true` if changepoint_prob < 0.3 |

---

## FUSION LAYER — Adaptive Combination

### What does it do?
It combines the signals of all 7 layers with **adaptive weights** — layers that are more confident receive a higher weight.

### Fusion Formula

**1. Individual Signals → [-1, +1]:**
```
kalman_signal  = (velocity × 1000).clamp(-1, +1)
hurst_signal   = persistence_signal                    (already [-1, +1])
wavelet_signal = (trend_direction × 100).clamp(-1, +1)
vpin_signal    = (buy_pressure - 0.5) × 2             (map [0,1] → [-1,+1])
```

**2. Fixed (Base) Weights:**
| Layer | Weight | Rationale |
|-------|--------|-----------|
| Kalman | 0.30 | The most direct predictor (price + velocity) |
| Wavelet | 0.25 | High-quality denoised trend |
| Hurst | 0.20 | Memory / persistence |
| VPIN | 0.15 | Smart-money activity |
| Entropy echo | 0.10 | Clarity × Kalman (confirmation) |

**3. Meta-Confidence (Global Modifier):**
```
meta_confidence = (lyapunov.predictability × entropy.clarity × stability_factor).clamp(0, 1)
```
where `stability_factor = 1.0` if the regime is stable, `0.3` otherwise.

**4. Regime Boost:**
```
Trending      → × 1.3  (boost — directional signals are trustworthy)
MeanReverting → × 0.7  (reduce — the trend may reverse)
Chaotic       → × 0.3  (severe penalty — unpredictable)
```

**5. Final Score:**
```
raw = w_kalman × kalman + w_wavelet × wavelet + w_hurst × hurst + w_vpin × vpin + w_entropy × kalman × clarity
oracle_score = (raw × regime_boost × meta_confidence × 100).clamp(-100, +100)
```

**6. Final Confidence:**
```
confidence = meta_confidence × max(0, 1 - noise_ratio)
```

---

## Interpreting oracle_score

| Score | Interpretation | Action |
|-------|----------------|--------|
| **+15 to +100** | Bullish | 🟢 LONG |
| **-15 to +15** | Neutral / Uncertain | ⚪ HOLD |
| **-100 to -15** | Bearish | 🔴 SHORT |

> **V2 OPTIMISATION NOTE:** The historical computation was optimised from O(n²) to **O(n)** using pre-computed series and point-by-point fusion. This allows instant computation over thousands of bars without timeouts. The thresholds were adjusted to ±15 to reflect the real scale of the composite scores.

### Chaos Kill Switch
If `confidence < 0.15`, the market is in **complete chaos**:
- A **-50 point** penalty is applied to the strategy score
- This effectively blocks any trade entry
- Rationale: in chaos, NO indicator can be trusted

---

## Data Flow Diagram

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

## Advantages over Traditional Indicators

| Aspect | Classic Indicators (EMA, RSI) | TITAN ORACLE |
|--------|-------------------------------|--------------|
| **Timing** | Lagging (look at the past) | Leading (forecast the future) |
| **Noise** | Sensitive to noise | Wavelet denoises the signal |
| **Regime** | Not detected | Entropy + Changepoint detect it |
| **Smart Money** | Invisible | VPIN detects informed activity |
| **Chaos** | Ignored | Lyapunov + Kill Switch protect |
| **Memory** | Not analysed | Hurst detects persistence |
| **Adaptivity** | Fixed parameters | Adaptive weights based on conditions |

---

## Source Files

| File | Description |
|------|-------------|
| `src-tauri/crates/titan-indicators/src/oracle.rs` | Full implementation (all 7 layers + fusion + unit tests) |
| `src-tauri/crates/titan-indicators/benches/bench_oracle.rs` | Criterion benchmarks for the Oracle pipeline |
| `src-tauri/src/titan_oracle.rs` | Re-export of the `titan-indicators` Oracle API into the main crate |
| `src-tauri/src/strategy.rs` | Decision logic driven by `oracle_score` |
| `src-tauri/src/commands/backtest.rs` | Tauri command `get_oracle_snapshot` (registered in `main.rs`) |

---

*TITAN ORACLE v1.0 — Precision. Prediction. Profit.* 🔮

# 🤖 GPU NEXUS — User Manual

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


## Table of Contents

1. [What is GPU Nexus?](#ce-este-gpu-nexus)
2. [The 3 Strategy Modes](#cele-3-moduri-de-strategie)
3. [GPU Nexus Panel (UI)](#panoul-gpu-nexus-ui)
4. [How it Works Agents](#cum-funcționează-agenții)
5. [Fusion Alpha Slider (α)](#slider-ul-fusion-alpha-α)
6. [Consensus Interpretation](#interpretarea-consensului)
7. [Usage Tips](#sfaturi-de-utilizare)

---

## What is GPU Nexus?

GPU Nexus is a **massive multi-agent system** running **150,000 autonomous agents of trading** directly on the graphics card (GPU) using WGPU technology. Each agent makes independent buy/sell decisions based on real-time market data, and the end result is an **aggregate consensus** that indicates the dominant market direction.

### Why 150,000 agents?

- **Diversity** — Each agent has a "risk DNA" unique randomly generated
- **Robustness** — Thousands of independent opinions reduce noise and false signals
- **Speed** — GPU processes all agents in parallel, typically under 500μs

---

## The 3 Strategy Modes

The application offers **3 modes** of operation, selectable from the Nexus panel:

### 🧠 ORACLE (Default Mode)

- Uses **exclusively** existing math engine (Kalman, Lyapunov, Wavelet, VPIN, etc.)
- Nexus GPU is **disabled** — no additional GPU computation
- **When to use it**: If you want conservative, proven strategy based on math analysis pure

### 🤖 GPU (GPU Nexus)

- Uses **exclusively** consensus of 150K GPU agents
- Oracle math engine continues to run but **does not influence** decisions
- **When to use it**: If you want to test the performance of GPU agents independently, or if the market is very volatile and you want maximum diversity of opinions

### ⚡ FUSION (Hybrid Mode)

- **Combines both strategies** using the formula:

  ```
  Scor Final = α × Scor Oracle + (1 − α) × Scor GPU Nexus
  ```

- The **Fusion α** slider controls the weight of each strategy
- **When to use it**: Recommended for daily use — get the benefits of both systems

---

## Nexus GPU Panel (UI)

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

### Elements

| Element | Description |
|---------|-----------|
| **Toggle 3-way** | ORACLE / GPU / FUSION buttons — click to change mode |
| **BUY/SELL bar** | Visual bar with BUY percentage (green) vs SELL (red) |
| **Consensus Tag** | STRONG_BUY, BUY, NEUTRAL, SELL, STRONG_SELL |
| **AGENTS** | Total number of active agents (150,000) |
| **GPU** | Last GPU dispatch latency in microseconds (μs) |
| **MODE** | Current active mode |
| **Fusion α slider** | Appears only in FUSION mode — controls weight |

---

## How Agents Work

The 150,000 agents are divided into **3 types of strategy**:

### 1. 📈 Momentum (Agents 0 – 49,999)

- I follow the **trend direction** (velocity)
- Buy when the price accelerates up
- Sell when the price accelerates down
- Formula: `signal = velocity × risk_dna × 100.0`

### 2. 📊 Mean-Reversion (Agents 50,000 – 99,999)

- Look for **reversions to the mean**
- Buy when the price is below the mean (underextended)
- Sell when the price is overextended
- Formula: `signal = −deviation × risk_dna × 50.0`

### 3. 🚀 Breakout (Agents 100,000 – 149,999)

- Detect **volatility breakouts**
- Buy on high volatility bullish breakout
- Sell on bearish breakout
- Formula: `signal = velocity × volatility × risk_dna × 200.0`

### The DNA of Risk

Each agent has a unique **risk_dna** (value between 0.1 and 1.0) that determines how aggressively it reacts to signals. Agents with high risk_dna are more aggressive; those with low risk_dna are more conservative. This DNA is randomly generated at initialization.

### Voting Process

1. Each agent calculates a signal based on its strategy
2. If the signal > **threshold** → vote **BUY** (with weight = risk_dna × 100)
3. If the signal < **−threshold** → vote **SELL** (with weight = risk_dna × 100)
4. If the signal is between the thresholds → the agent remains **neutral** (does not vote)
5. The votes are aggregated atomically on the GPU and the readback contains only 8 bytes (buy_weight + sell_weight)

---

## The Fusion Alpha Slider (α)

The Slider α appears **only in FUSION mode** and controls the relative weight:

| α value | Behavior |
|-----------|-------------|
| **0.00** | 100% GPU Nexus, 0% Oracle — only agents decide |
| **0.25** | 75% GPU + 25% Oracle |
| **0.50** | Balance 50/50 (recommended to start) |
| **0.75** | 25% GPU + 75% Oracle |
| **1.00** | 0% GPU, 100% Oracle — only the math decides |

### Recommendations

- **Start**: Start with α = 0.50 (perfect balance)
- **Trending market**: Increase α to 0.70+ (Oracle detects long trends better)
- **Side/choppy market**: Decrease α to 0.30 (GPU has more diversity of opinion)
- **Extreme volatility**: α = 0.40 (GPU Breakout agents excel in these conditions)

---

## Consensus Interpretation

The consensus tag is calculated from the agents' BUY/SELL percentage:

| Consensus | Condition | Meaning |
|---------|----------|-------------|
| 🟢 **STRONG BUY** | BUY > 70% | Overwhelmingly bullish — strong buy signal |
| 🟢 **BUY** | BUY > 55% | Moderate majority bullish — buy signal |
| ⚪ **NEUTRAL** | 45% – 55% | Undecided market — agents are divided, caution |
| 🔴 **SELL** | SELL > 55% | Moderately bearish majority — sell signal |
| 🔴 **STRONG SELL** | SELL > 70% | Overwhelmingly bearish — strong sell signal |

### How to read the bar

- **Big green, little red** → Dealers are mostly bullish
- **Big red, little green** → Dealers are mostly bearish
- **About equal** → Market without clear direction, avoid transactions

---

## Usage Tips

### ✅ Best Practices

1. **Start with ORACLE mode** to get used to the interface
2. **Switch to FUSION with α = 0.50** when you want to test GPU Nexus
3. **Monitor GPU μs** — if it exceeds 2000μs, GPU is overloaded
4. **Don't ignore NEUTRAL** — when agents are 50/50, market is indecisive
5. **Use FUSION for real trading** — combination of both strategies is more robust

### ⚠️ Caution

- Nexus GPU works best with **live data** (ticks reals)
- Consensus updates **every tick** (~100ms)
- The first few ticks may look NEUTRAL until agents accumulate data
- **STRONG_BUY/STRONG_SELL** are rare signals — when they occur, they are significant

### 🔧 Troubleshooting

| Problem | Solution |
|----------|---------|
| GPU: --μs (does not update) | Mode is on ORACLE — switch to GPU or FUSION |
| Always NEUTRAL | Normal for first few seconds; if persistent, check data feed |
| GPU Latency > 5000μs | Close other GPU-intensive applications |
| Stick not moving | Verify scan is running (LIVE FEED active) |

---

## Technical Architecture (Summary)

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

**Key files:**

- `gpu.rs` — NexusAgent, nexus_dispatch(), readback logic
- `nexus_kernel.wgsl` — Compute shader with the 3 strategies
- `main.rs` — StrategyMode enum, Tauri commands, tick loop integration
- `index.html` — Nexus UI panel + JavaScript handlers

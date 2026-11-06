# TITAN HFT (SHARK V7 RELOADED) — Full User Manual

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


**Version:** V7 Reloaded
**Engine:** Rust (Tauri) + WGPU + HTML/JS Frontend
**Platform:** Alpaca Markets (Paper + Live)
**Supported Assets:** Crypto (BTC/USD, ETH/USD, etc.) + US Stocks
**Date:** April 2026 (updated April 3 — Session 69: Wave 2B SmartTrend Correctness — 108 Chapters)

---

## CONTENTS

1. [Startup and Initial Configuration](#1-pornire-si-configurare-initiala)
2. [System Architecture](#2-arhitectura-sistemului)
3. [User Interface (UI)](#3-interfata-utilizator-ui)
4. [Restructured Sidebar (Full Audit)](#4-sidebar-ul-restructurat-full-audit)
5. [System of Strategy (3 Modes)](#5-sistemul-de-strategie-3-moduri)
6. [How the Bot Chooses Assets](#6-cum-alege-botul-asset-urile)
7. [TITAN Oracle — 7 AI Indicators](#7-titan-oracle--7-indicatori-ai)
8. [GPU Nexus — 150,000 Agents](#8-gpu-nexus--150000-agenti)
9. [L2 Orderbook and OFI (Order Flow Imbalance)](#9-l2-orderbook-si-ofi-order-flow-imbalance)
10. [Autopilot — Automatic Trading Engine](#10-autopilot--motorul-de-trading-automat)
11. [Safety Filters (Safety Gates)](#11-filtrele-de-siguranta-safety-gates)
12. [Genetic Optimizer — Automatic Evolution of Parameters](#12-genetic-optimizer--evolutie-automata-a-parametrilor)
13. [Circuit Breaker — System of Safety](#13-circuit-breaker--sistemul-de-siguranta)
14. [Dry Run Mode — Simulation without Risk](#14-dry-run-mode--simulare-fara-risc)
15. [Watchdog and Telemetry](#15-watchdog-si-telemetrie)
16. [Logging and Reports](#16-logging-si-rapoarte)
17. [Backtesting](#17-backtesting)
18. [Professional Chart](#18-graficul-profesional)
19. [New Tabs (AI News, Journal, HFT Engine, Performance)](#19-tab-uri-noi)
20. [AI Scanner — FUSION Asset Discovery](#20-ai-scanner--fusion-asset-discovery)
21. [Positions and Funds Management](#21-managementul-pozitiilor-si-fondurilor)
22. [All Background Loops](#22-toate-loop-urile-din-background)
23. [All Available Commands](#23-toate-comenzile-disponibile)
24. [Structure Files](#24-structura-fisierelor)
25. [Frequent Problems and Solutions](#25-probleme-frecvente-si-solutii)
26. [Catalyst Watchdog — Sentient News Feed](#26-catalyst-watchdog--sentient-news-feed)
27. [Chart Signals and Arrows](#27-semnale-si-săgeți-pe-grafic)
28. [Telegram Alerts — Notifications Mobile](#28-telegram-alerts--notificari-mobile)
29. [Risk Correlation Matrix](#29-risk-correlation-matrix)
30. [Portfolio Performance & Rebalancing](#30-portfolio-performance--rebalancing)
31. [ATR Dynamic Stop-Loss & Partial Profit Taking](#31-atr-dynamic-stop-loss--partial-profit-taking)
32. [Web Dashboard — Remote Mobile Access](#32-web-dashboard--acces-remote-mobile)
33. [State Persistence — Crash Recovery](#33-state-persistence--crash-recovery)
34. [Smart Score V4 — Institutional Ranking](#34-smart-score-v4--ranking-institutional)
35. [Auto Entry — Entry Control Automatic](#35-auto-entry--control-intrari-automate)
36. [Signal Alignment — FUSION-Aware Autopilot & Causal Chart](#36-signal-alignment--fusion-aware-autopilot--causal-chart)
37. [Auto-Resume Scan — Continue from Breakpoint](#37-auto-resume-scan--continuare-de-la-punctul-de-intrerupere)
38. [Multi-Symbol Entry Fix & DEL ALL](#38-multi-symbol-entry-fix--del-all)
39. [Entry Gate Scoring Pipeline — Debottlenecking](#39-entry-gate-scoring-pipeline--debottlenecking)
40. [Scalping Mode — Toggles per Strategy](#40-scalping-mode--toggle-uri-per-strategie)
41. [Regime-Adaptive Trading Signals](#41-regime-adaptive-trading-signals)
42. [Autopilot Alignment — check_entry + Oracle Exits](#42-autopilot-alignment--check_entry--oracle-exits)
43. [WARRIOR PRO — Ross Cameron Scanner](#43-warrior-pro--ross-cameron-scanner)
44. [Position Sizing Safety — Hardcaps & Duplicate Prevention](#44-position-sizing-safety--hardcaps--duplicate-prevention)
45. [Scalping Position Marker](#45-scalping-position-marker)
46. [AI Scanner Enhancements — Cancellation, Shared Scoring, Rate Limiting](#46-ai-scanner-enhancements)
47. [SMART PICKS 3-Pillar Scoring](#47-smart-picks-3-pillar-scoring)
48. [Market Discovery — Advanced Parameters and Auto-Scan](#48-market-discovery--parametri-avansati-si-auto-scan)
49. [Manual Trade — Manual USD Transaction](#49-manual-trade--tranzactionare-manuala-usd)
50. [Force Manual Override — Total Control](#50-force-manual-override--control-total)
51. [Live Positions — Auto/Manual Mode per Asset](#51-live-positions--mod-automanual-per-asset)
52. [Drawing Tools V2 — All Tools](#52-drawing-tools-v2--toate-instrumentele)
53. [Aggressive Mode — Full Audit](#53-aggressive-mode--audit-complet)
54. [Extended Hours — Full Audit](#54-extended-hours--audit-complet)
55. [Session 7 Eliminations](#55-eliminari-din-sesiunea-7)
56. [AI Scanner Parallel Scoring (Rayon)](#56-ai-scanner-parallel-scoring-rayon)
57. [Deep Audit Reasoning — Modal Window](#57-deep-audit-reasoning--fereastra-modal)
58. [Manual Trade USD vs Shares Toggle](#58-manual-trade-usd-vs-shares-toggle)
59. [SIGNAL Scalping Kit — Alerts and Exact Journal](#83-signal-scalping-kit-sesiunea-17--alerte-realtime--journal-exact)
60. [All Configurable Trading Parameters (Session 18)](#84-toti-parametrii-de-trading-configurabili-sesiunea-18)
61. [Deep Audit + 12 Smart Features + Styling (Session 19)](#85-deep-audit--12-smart-features--styling-sesiunea-19)
62. [Session 26: TA Entry Gates with Soft/Hard Mode (RF-B)](#cap-91--sesiunea-26-ta-entry-gates-cu-softhard-mode-rf-b)
63. [Portfolio VaR Engine + HHI Concentration Gate (S9)](#98-portfolio-var-engine--hhi-concentration-gate-s9--sesiunea-33)
64. [Microstructure Alpha Signals — Stream VPIN, Spread Regime, Dual VPIN Divergence (S10-lite)](#100-microstructure-alpha-signals)
65. [Microstructure Alpha Signals — Stream VPIN, Spread Regime, Dual VPIN Divergence (S10-lite)](#100-microstructure-alpha-signals)
66. [N4 Strategy Configurable Thresholds (Session 35-36)](#101-n4-strategy-thresholds-configurabile)
67. [CI Hardening + Criterion Benchmarks (S19, Session 37)](#102-ci-hardening--criterion-benchmarks-s19--sesiunea-37)
68. [Online Oracle Weight Adaptation (S15, Session 38)](#103-online-oracle-weight-adaptation-s15--sesiunea-38)
69. [Exit Inflight Guard — Anti-Loop Protection (Session 39)](#104-exit-inflight-guard--protectie-anti-loop-la-iesiri-sesiunea-39)
70. [Configuration Coherence — apply_runtime_config (Session 44)](#105-configuration-coherence--apply_runtime_config-sesiunea-44)
71. [Wave 1 MUST-FIX — 9 Critical Fixes + 4 Bonuses (Session 67)](#106-wave-1-must-fix--9-fix-uri-critice--4-bonusuri-sesiunea-67)
72. [Wave 2A SmartTrend Safety — 8 Fixes + 4 Bonuses (Session 68)](#107-wave-2a-smarttrend-safety--8-fix-uri--4-bonusuri-sesiunea-68)
73. [Wave 2B SmartTrend Correctness — 9 Fixes + 5 Bonuses (Session 69)](#108-wave-2b-smarttrend-correctness--9-fix-uri--5-bonusuri-sesiunea-69)

---

## 1. START-UP AND INITIAL CONFIGURATION

### 1.1 System Requirements

- **OS:** Linux (Pop!_OS, Ubuntu, Fedora)
- **GPU:** NVIDIA with Vulkan support (RTX 3060+)
- **RAM:** 8 GB minimum, 16 GB recommended
- **Rust:** Installed with rustup
- **Redis:** Optional but recommended (`redis-server` on port 6379)
- **Alpaca Account:** Paper or Live with Algo Trader Plus ($99/mo for SIP data)

### 1.2 Starting the application

```bash
${TITAN_HFT_ROOT:-.}/src-tauri/target/release/titan_hft_rust
```

### 1.3 At startup, Titan does automatically

1. **Initialize logging** — write in `$HOME/HFT_Bot/logs/titan.log.YYYY-MM-DD`
2. **Load API Keys** from `$HOME/HFT_Bot/titan_keys.json`
3. **Full Health Check** — check:
   - API Keys (loaded and non-empty)
   - Alpaca API (connection, equity, account status)
   - Redis (connection)
   - GPU WGPU (Vulkan adapter found)
   - Settings (values ​​within safe limits)
   - Log directory (writable)
4. **Start all loops** (9 loops + watchdog supervisor)
5. **Synchronize positions** on Alpaca (Reconciliation)
6. **Displays the Health Check result in the Activity Log**

### 1.4 Configuring API Keys

In the interface, in the left sidebar, expand the section **API & WALLET**:

1. Enter **API KEY ID** (26 characters)
2. Enter **API SECRET** (44 characters)
3. Click **SAVE KEYS**

The keys are saved in `$HOME/HFT_Bot/titan_keys.json` and persist between sessions.

---

## 2. SYSTEM ARCHITECTURE

### 2.1 Rust Modules (Backend)

| File | Responsibility |
|--------|-----------------|
| `main.rs` | Central orchestrator, state management, ~70 Tauri commands, 14 background loops |
| `titan_oracle.rs` | 7 advanced AI indicators (Kalman, Hurst, Entropy, Wavelet, VPIN, Lyapunov, BOCPD) |
| `gpu.rs` | WGPU compute shaders, 150K GPU Nexus agents, EMA/RSI on GPU |
| `risk.rs` | Circuit Breaker (4 levels), SessionAnalytics, OFI, risk-based positioning |
| `genetic.rs` | Genetic optimizer with walk-forward backtest (70/30 IS/OOS) |
| `l2_heatmap.rs` | L2 orderbook live, OFI from full depth, crypto v1beta3 + SIP stocks |
| `watchdog.rs` | Heartbeat monitoring, telemetry per loop, session reports |
| `titan_log.rs` | Structured logging with tracing, daily rotation |
| `indicators.rs` | Classic technical indicators (EMA, RSI, MACD, ADX, BB, VWAP) |
| `strategy.rs` | The logic of strategy and signals |
| `streamer.rs` | Price streaming and heatmap |
| `scanner.rs` | Market scanner (quant_prefilter + warrior_prefilter) |
| `monitor.rs` | Hardware monitoring (CPU, GPU, RAM, Network) |
| Sentiment analysis + Catalyst check | Sentiment analysis + Catalyst check |
| `app_state.rs` | Definition of application states |
| Pearson correlation, portfolio performance, rebalancing | Pearson correlation, portfolio performance, rebalancing |
| `telegram.rs` | Telegram Bot alerts (fill, breaker, catalyst, daily) |
| `journal.rs` | Persistent trade journal (JSON, stats, CSV export) |

### 2.2 GPU Shaders (WGSL)

| Shaders | what is he doing |
|--------|---------|
| `nexus_kernel.wgsl` | 150K agents: each calculates intent (buy/sell), votes, adjusts capital |
| `gpu_indicators.wgsl` | EMA fast, EMA slow, RSI for a single series |
| `gpu_indicators_batch.wgsl` | The same thing but for several series simultaneously |
| `gpu_dot.wgsl` | Parallel dot product (GPU reduction) |

### 2.3 Front end

- **index.html** — Single-page application with TradingView LightweightCharts
- **Communication:** Bulls `invoke()` for commands, `listen()` for push events

---

## 3. USER INTERFACE (UI)

### 3.1 Header

- **Title:** "TITAN V4.37 (SHARK V7 RELOADED)"
- **Status indicator:** CLOSED / LIVE
- **NY Time:** New York time updated every second

### 3.2 Central Area — Tabs

| Tab | Content |
|-----|----------|
| **Complex Chart** (tab-0) | Professional OHLC chart with EMA, BB, VWAP, RSI, MACD, ADX, Oracle |
| **News AI** (tab-1) | News feed with sentiment analysis (gauge, timeline, cards) |
| **Backtest** (tab-2) | Backtesting engine with multi-timeframe and multi-symbol sweep, equity curve |
| **Diary** (tab-3) | Persistent transaction history with filters, CSV export, equity/PnL graphs |
| **HFT Engine** (tab-4) | Advanced HFT engine settings: entry/exit params, Discord alerts, safety limits |

### 3.3 Upper Central Panel

**GPU Nexus — 150K Agents:**

- Consensus bar: green (BUY) vs red (SELL)
- Mode buttons: ORACLE / GPU / FUSION
- Slider Fusion Alpha (0.0 = full Oracle, 1.0 = full GPU)
- Shows: number of agents, GPU latency (microseconds), current mode

### 3.4 The TITAN SAFETY panel

Displays in real time:

- **Circuit Breaker Level:** NORMAL (green) / CAUTION (yellow) / HALT (orange) / PANIC (red)
- **Daily PnL:** Current day's profit/loss
- **Max DD:** Maximum drawdown (percentage)
- **Win Rate:** Win rate
- **Trades:** Total number of transactions
- **Sharpe:** Sharpe ratio of the session
- **Profit Factor:** The profit/loss ratio
- **Equity:** The total value of the account

### 3.5 EVOLUTION panel

- **Gen:** The current generation of the genetic optimizer
- **Fitness:** The champion's fitness score
- **Sharpe:** Sharpe ratio of the champion
- **TP/SL:** Take Profit / Stop Loss of active parameters
- **FREEZE / UNFREEZE:** Stops/starts genetic evolution
- **BACKTEST NOW:** Run an instant backtest with the current parameters

### 3.6 OFI LIVE panel

- **SPR:** The live spread in basis points (bps)
- **OFI bar:** Graphical visualization of Order Flow Imbalance
- **BID depth / ASK depth:** Orderbook depth
- **IMB:** Imbalance ratio

### 3.7 ACTIVITY LOG panel

Log in real time with color-coding:

- **Green:** Success messages (BOOT, OK, PASS)
- **Red:** Errors and warnings
- **Yellow:** Warnings
- **Cyan:** Trade information

### 3.8 LOOP HEARTBEATS panel

- **Global status:** "ALL ALIVE" (green) or "X DEAD LOOPS" (red)
- **Per loop:** Name, time since last heartbeat, number of ticks
- Updated every 10 seconds

### 3.9 Control Buttons

| Button | Function |
|-------|---------|
| **DRY RUN: OFF/ON** | Activate/deactivate simulation (do not send orders to Alpaca) |
| **SHUTDOWN** | Graceful stop — saves the report, stops all loops |
| **LIQUIDATE ALL** | Close ALL open positions and cancel orders |

### 3.10 Left Sidebar (Summary)

The sidebar has been completely restructured into 8 logical sections (see [Section 4](#4-sidebar-ul-restructurat-full-audit) for full details).

---

## 4. THE RESTRUCTURED SIDEBAR (FULL AUDIT)

The left sidebar has been completely reorganized into 8 logical sections, each with a visible header and consistent content.

### 4.1 Section 1 — SYMBOL & FAVORITES

- **Symbol input:** The box where the active symbol is entered (ex: BTC/USD, AAPL)
- **Symbol buttons:** BTC/USD, ETH/USD, SPY, AAPL, TSLA (quick change)
- **Action buttons:** BUY / SELL manually
- **FAVORITES list:** Persistently saved symbols (kept between sessions)
  - **+** button to add, **x** button to delete
  - Click on the favorite → it becomes the active symbol
  - Favorites are stored in `titan_settings.json` (backend), not in localStorage

### 4.2 Section 2 — SYMBOL TABS (WATCHLIST / HFT ENGINE)

Two sub-tabs with interactive buttons:

| Tab | Content |
|-----|----------|
| **WATCHLIST** | List of favorite symbols with quick selection |
| **HFT ENGINE** | Target symbol list for multi-symbol autopilot |

HFT ENGINE tab displays:

- List of configured target symbols
- Buttons for adding/deleting from the HFT list
- Status per symbol (active/inactive)

### 4.3 Section 3 — HARDWARE MONITOR (Collapsible)

Section `<details>` open by default. Display:

**GPU:**

- Model name (ex: NVIDIA GeForce RTX 3060)
- Temperature, usage %, memory used/total
- Power consumed, fan speed

**CPU:**

- Usage %, temperature, frequency
- Fan speed, RAM used/total

**Network:**

- Active interface, download/upload speed
- Total data transfer

### 4.4 Section 4 — BANK & ACCOUNTS

Show funds in real time:

| Field | Description |
|------|-----------|
| **Cash (Power)** | Available purchasing power |
| **Equity (Total)** | **Bot Alloc** |
| **Bot Alloc** | Funds allocated to the bot |
| **User Deposit** | The user's virtual storage |
| **PnL** | Daily profit/loss |
| **Bot Trade Size** | Order size (persistent in the backend) |
| **Circuit Breaker Limit** | Circuit breaker limit (persistent in backend) |

Transfer buttons:

- **TOP UP BOT** — Allocate funds from user to bot
- **WITHDRAW** — Withdraw funds from the bot to the user
- **TOPUP USER** — Increases virtual storage

### 4.5 Section 5 — API & WALLET (Foldable)

Section `<details>` open by default.

- **API KEY ID** and **API SECRET** — editable fields
- **SAVE KEYS** — Save in `titan_keys.json`
- **Wallet info:** Show wallet status

### 4.6 Section 6 — RISK & PARAMS (Collapsible)

Section `<details>` open by default. Contain:

**Toggles (checkboxes):**

| Toggle | Effect |
|--------|-------|
| **Scan Active** | Activate the market scanner |
| **Kelly** | Kelly Criterion positioning |
| **Extended Hours** | Trading outside normal hours |
| **Manual Result** | Manual result per trade |
| **Force Manual** | Force manual mode (disable autopilot) |
| **Warrior** | Aggressive mode |
| **Auto-Pilot** | Enable/disable autopilot |
| **Aggressive** | Aggressive parameters |
| **Short Selling** | Allows short selling |
| **Multi-Symbol** | Enable simultaneous trading on multiple symbols |

**Sliders:**

- **Risk per trade** (0-100%) — bright yellow slider
- All sliders have yellow track (#FFD700 → #FFEA00) with glow and golden thumb

### 4.7 Section 7 — MARKET DISCOVERY (Collapsible)

Section `<details>` open by default.

- **Market Scanner:** Search for new symbols based on criteria (volume, volatility)
- **Discovery results:** List of symbols found

### 4.8 Section 8 — CONTROL BUTTONS

Control buttons at the base of the sidebar:

| Button | Color | Effect |
|-------|---------|-------|
| **DRY RUN: OFF/ON** | Yellow | Activate/deactivate simulation |
| **SHUTDOWN** | Gray | Shutdown with save report |
| **LIQUIDATE ALL** | Red | Kill switch — close everything |

### 4.9 Persistence of Settings

After Full Audit, the following settings are persistently stored in `titan_settings.json` (Rust backend), NOT in localStorage:

| Setting | Description |
|--------|-----------|
| `dry_run_enabled` | Dry Run Status (true/false) |
| `strategy_mode_name` | Strategy Mode (ORACLE/GPU/FUSION) |
| `bot_trade_size` | Order Size |
| `circuit_breaker_limit` | Circuit breaker limit |
| `favorites` | List of favorite symbols |

Settings are loaded automatically at startup and saved instantly when modified.

### 4.10 Styling

- **Inputs/Selectors:** Black Background (`#000`), Gold Text (`#FFD700`), Gray Border (`#333`), JetBrains Mono Font
- **Sliders:** Glow Yellow Track, Gold Thumb with White Border
- **Buttons:** Uniform Classes — `.btn-primary` (accent), `.btn-danger` (red), `.btn-secondary` (grey)
- **Toggles:** Yellow hover effect on `.chk-row:hover`
- **Section headers:** `.sb-header` with uppercase font, yellow border-bottom

---

## 5. STRATEGY SYSTEM (3 MODES)

Titan operates in 3 strategy modes, selectable from the UI:

### 5.1 ORACLE Mode

Exclusively uses **TITAN Oracle** (7 AI indicators):

- Signal comes from Oracle score (-100 to +100)
- Ideal for trend following and regime detection
- 100% processing on CPU (native Rust)

### 5.2 GPU NEXUS Mode

Uses exclusively **GPU Nexus** (150K agents on RTX):

- 150,000 autonomous agents vote simultaneously on GPU
- Consensus derived from weighted majority vote
- Latency under 1ms (typically ~500-700 microseconds)
- Ideal for quick market reaction

### 5.3 FUSION Mode (Recommended)

Combine Oracle + GPU Nexus with an adjustable blend:

- **Fusion Alpha slider:** 0.0 = 100% Oracle, 1.0 = 100% GPU
- **Absolute default:** The system always starts in FUSION mode by default (persistent setting UI + Backend). Initial Blend: 0.50.
- Combine deep analysis (Oracle) with fast response (GPU)
- Input parameters are controlled by the genetic optimizer

### Change mode

- From UI: click on the buttons **ORACLE**, **GPU**, **FUSION**
- The alpha slider is only visible in FUSION mode

---

## 6. HOW THE BOT CHOOSES ASSETS

### 6.1 Two Modes of Selection

Titan does not scan the market itself to discover what to trade. Assets are defined by the user:

**Module 1 — Unique Symbol (Default):**

- The bot ONLY trades the active symbol in the sidebar (ex: BTC/USD)
- Change the symbol → the bot focuses on the new symbol
- All signals (Oracle, GPU Nexus) are calculated on that symbol

**Module 2 — Multi-Symbol (HFT Engine):**

- Activated by the **Multi-Symbol** toggle in the sidebar
- The bot trades simultaneously on the list of symbols in the HFT ENGINE tab
- Each symbol receives its own analysis and independent signals
- The autopilot runs the entry/exit cycle for ALL symbols in the list

### 6.2 Decision Flow (per symbol)

```
Simbol Activ → Fetch OHLCV bars
                    ↓
            Calcul Oracle Score (7 AI indicators)
            Calcul GPU Nexus Consensus (150K agents)
                    ↓
            Scor final (fusion_threshold check)
                    ↓
         ╔══════════════════════════════════════╗
         ║  SAFETY GATES (toate trebuie trecute):║
         ║  1. Circuit Breaker = NORMAL          ║
         ║  2. Spread Guard < limita             ║
         ║  3. OFI confirma directia             ║
         ║  4. Cooldown elapsed                  ║
         ║  5. Max positions nu e atins          ║
         ║  6. Sentiment filter (optional)       ║
         ╚══════════════════════════════════════╝
                    ↓
            Risk-based position sizing
                    ↓
            SEND ORDER (sau DRY RUN log)
```

### 6.3 Strategy — Filters relationship

The strategy (Oracle/GPU/Fusion) generates the **signal** and the **score**. But the signal does NOT guarantee a trade. The safety filters are **independent** and can block the entry even if the score is high:

- Strategy says "BUY with score 85" → Spread Guard blocks (spread too large)
- Strategy says "SELL with score 70" → Circuit Breaker is on HALT → blocked
- Strategy says "BUY with score 90" → OFI is negative → the signal is not confirmed

The strategy is the **signal creator**. Filters are the **gatekeepers** who protect the capital.

---

## 7. TITAN ORACLE — 7 AI INDICATORS

The Oracle combines 7 advanced indicators into a final score (-100 to +100):

### 7.1 Kalman Adaptive Predictor (KAP) — Weight 30%

- **What it does:** Kalman filter with constant velocity model
- **How ​​it works:** Estimates price and speed (trend). The Q matrix scales with volatility.
- **Signal:** If the predicted price > current price → bullish. Reverse → bearish.
- **Usefulness:** The most stable predictor on trending markets

### 7.2 Haar Wavelet Multi-Resolution — Weight 25%

- **What it does:** Breaks down the price into trend + noise
- **How ​​it works:** It applies the Haar wavelet transform, keeping only the low frequencies
- **Signal:** The denoised trend indicates the real direction
- **Utility:** Filters market noise

### 7.3 Fractal Hurst Momentum (FHM) — Weight 20%

- **What it does:** R/S (Range/Scale) analysis for regime detection
- **How ​​it works:** Calculates the Hurst exponent on the price window
- **Interpretation:**
  - H > 0.5 → Trending market (momentum works)
  - H < 0.5 → Mean-reverting market (contrarian works)
  - H = 0.5 → Random walk
- **Utility:** Decide if the momentum or contrarian strategy is more appropriate

### 7.4 VPIN (Volume-Weighted Probability of Informed Trading) — Weight 15%

- **What it does:** It measures the probability of "informed trading"
- **How ​​it works:** Division into volume buckets, buy/sell imbalance calculation
- **Interpretation:** High VPIN → active informed traders, possible big move
- **Usefulness:** Early warning for volatility spikes

### 7.5 Shannon Entropy Flow Regime Detector (EFRD) — Weight 10%

- **What it does:** Measures the entropy of the returns
- **How ​​it works:** The returns are binned in 20 categories, the Shannon entropy is calculated
- **Interpretation:**
  - Low entropy → Clear, directional regime
  - High entropy → Chaotic, directionless market
- **Utility:** Market clarity meta-indicator

### 7.6 Lyapunov Chaos Detector

- **What it does:** Detects chaos in the price series
- **How it works:** Nearest-neighbor divergence analysis in phase space
- **Interpretation:**
  - Positive lambda → Chaos (hard to predict)
  - Negative lambda → Predictable
- **Utility:** Adjusts meta-confidence — if the market is chaotic, reduces the signal

### 7.7 Bayesian Online Changepoint Detector (BOCPD)

- **What it does:** Detects regime changes in real time
- **How it works:** The Adams algorithm & MacKay (2007), Bayesian changepoint probability
- **Interpretation:** Changepoint probability > 0.3 → Reduces the signal by 70%
- **Usefulness:** Prevents entry into transactions even when the regime changes

### 7.8 Final Fusion Formula

```
raw_signal = (Kalman * 0.30) + (Wavelet * 0.25) + (Hurst * 0.20) + (VPIN * 0.15) + (Entropy * 0.10)

regime_boost = 1.3 daca trending | 0.7 daca mean-reverting | 0.3 daca chaotic

meta_confidence = (predictability * clarity * stability_factor).cbrt()
                  ^ media geometrica previne colapsul catre zero

stability_factor = 0.3 daca changepoint_prob > 0.3 | 1.0 altfel

oracle_score = raw_signal * regime_boost * meta_confidence * 100

confidence = meta_confidence * (1 - noise_ratio * 0.5).max(0.25)
             ^ noise penalty injumatatit, floor 25%
```

**Final range:** -100 (extremely bearish) to +100 (extremely bullish)

---

## 8. GPU NEXUS — 150,000 AGENTS

### 8.1 How it works

- **150,000 agents** run simultaneously on NVIDIA GPU (RTX 3060)
- Each agent has:
  - **Type:** Momentum (40%), Mean-Reversion (35%), Breakout (25%)
  - **Virtual capital:** Increases by +0.2% for a correct prediction, decreases by -0.2% for a wrong one
  - **Intent:** BUY or SELL, weighted by the agent's capital
- **Latency:** ~500-700 microseconds per dispatch (under 1ms)

### 8.2 Consensus

At each tick (500ms):

1. Send `MarketState` (price, volumes, indicators) to GPU
2. All agents simultaneously calculate intent
3. Summary `buy_weight` and `sell_weight`
4. Calculate percentages and consensus

| Consensus | Condition |
|---------|----------|
| **STRONG BUY** | buy_pct > 70% |
| **BUY** | buy_pct > 55% |
| **NEUTRAL** | None of the above |
| **SELL** | sell_pct > 55% |
| **STRONG SELL** | sell_pct > 70% |

### 8.3 Evolution of Agents

Agents do not have static parameters — their capital adjusts automatically:

- Agents with good predictions → increase in influence
- Agents with wrong predictions → decrease
- This mechanism creates a "survival of the fittest" natural

---

## 9. L2 ORDERBOOK AND OFI (ORDER FLOW IMBALANCE)

### 9.1 Data source

- **Crypto (BTC/USD etc.):** `wss://stream.data.alpaca.markets/v1beta3/crypto/us`
  - Subscription: **orderbooks** (L2 full depth) + quotes + trades
  - Real orderbook with multiple bids/asks levels
- **Stocks:** `wss://stream.data.alpaca.markets/v2/sip` (requires Algo Trader Plus)
  - Subscription: quotes + trades
  - OFI calculated from quote size changes

### 9.2 Automatic detection of crypto vs stocks

The symbol is detected automatically:

- Contains `/` (ex: BTC/USD) → Crypto
- Start with BTC, ETH, LTC, DOGE, SOL, AVAX, UNI → Crypto
- Other → Stocks (SIP)

### 9.3 Orderbook Management (Crypto)

- **BTreeMap** ordered by price for bids and asks
- Message `"o"` with `r=true` → Full reset (snapshot)
- Message `"o"` with `r=false` → Delta (add/update/remove)
  - `s == 0` → Delete price level
  - `s > 0` → Add or update
- Top 20 levels extracts for OFI and heatmap

### 9.4 OFI (Order Flow Imbalance)

**Formula:**

```
OFI = Σ(bid_size_deltas) - Σ(ask_size_deltas)  pe top 10 nivele
```

- **OFI positive** → Net buying pressure (bullish)
- **OFI negative** → Net selling pressure (bearish)
- **Smoothing:** EMA with alpha = 0.3

**OFI as input filter:**

- Buy signal: OFI > threshold (confirm buying pressure)
- Sell signal: OFI < -threshold (confirm selling pressure)
- If OFI does not confirm the signal, the entry is blocked

### 9.5 Spread Safety

- **Formula:** `spread_pct = (ask - bid) / mid * 100`
- If the spread exceeds the configured limit, entries are blocked
- It prevents transactions in low liquidity conditions

### 9.6 Consolidation of the L2 Stream

L2 data is processed through a **single** WebSocket in `streamer.rs`. Initially there were two separate connections (`streamer.rs` and `l2_heatmap.rs`), which caused the "connection limit exceeded (code 406)" error. After consolidation:

- `streamer.rs` opens a single WebSocket connection
- Process messages of type `"o"` (orderbook) and `"b"` (bar)
- Calculates OFI and issues `l2-update` and `ofi-update` to the frontend
- `l2_heatmap.rs` remains available as a module but no longer opens its own connections

### 9.7 What you see in the UI

- **SPR:** The current spread in basis points
- **OFI bar:** Green = buying pressure, Red = selling pressure
- **IMB:** Imbalance ratio (-1 to +1)
- **IMB:** Imbalance ratio (-1 to +1)

---

## 10. AUTOPILOT — THE AUTOMATIC TRADING ENGINE

### 10.1 Trading cycle (every 1.5 seconds)

1. **Check Circuit Breaker** — If it is not NORMAL, do not enter new positions
2. **Fetch bars** (OHLCV) from Alpaca
3. **Calculate Oracle Score** and/or **GPU Nexus Consensus**
4. **Check OFI** — The signal must be confirmed by the order flow
5. **Check Spread** — The spread must be within safe limits
6. **Check Cooldown** — Minimum time between entries
7. **Calculate the position** — Risk-based sizing
8. **Send the order** or **simulate** (if Dry Run is ON)

### 10.2 Entries (Entry)

Cumulative conditions (ALL must be met):

- Circuit breaker = NORMAL and entries_blocked = false
- Score >= fusion_threshold (from genetic optimizer)
- Confidence >= min_confidence (from genetic optimizer)
- OFI confirms direction (if ofi_filter_enabled)
- Spread is within limits
- Cooldown elapsed (minimum time since last entry)
- Valid price (> 0)
- Max open positions is not reached

### 10.3 Exits (Exit)

Checks on existing positions (at each tick):

- **Take Profit (TP):** The price has reached the profit target
- **Stop Loss (SL):** The price has reached the stop loss
- **Trailing Stop:** Dynamic trail gap — follows the favorable price and exits if it withdraws with trail_gap_pct
- **Circuit Breaker exits:** If HALT or PANIC is activated

### 10.4 Position Sizing (Risk-Based)

**Formula:**

```
risk_amount = equity * (risk_per_trade / 100) * breaker_multiplier
qty = risk_amount / |entry_price - stop_loss_price|
```

**Safety checks:**

- Maximum notional = 95% of equity
- Minimum Qty = 0.0001
- If the price is 0 → Order rejected
- If the keys are empty → Order rejected

### 10.5 Trailing Stop

- `trail_high` (for long) or `trail_low` (for short) per symbol is maintained
- At each tick, the trail is updated if the price goes favorably
- If the price retreats by > `trail_gap_pct` from trail → EXIT
- When the position is closed, the trail data is automatically deleted

---

### 10.6 Crypto vs Stocks — Automatic Detection

When sending an order, the bot automatically detects the type of asset:

**Crypto detection** (any of):

- The symbol contains `USD` or `/`
- Symbol starts with: BTC, ETH, LTC, DOGE, SOL, AVAX, UNI, AAVE, LINK, DOT

**Key Difference — Time In Force:**

- **Crypto:** `time_in_force: "gtc"` (Good Till Canceled)
- **Stocks/ETFs:** `time_in_force: "day"`

Sending a crypto order with `"day"` causes error `422 Unprocessable Entity: invalid crypto time_in_force` on Alpaca. Titan automatically manages this difference.

### 10.7 Handling Permanent Errors at Exit (Fix 403 Forbidden)

A critical issue was the `403 Forbidden: insufficient qty` bug when the bot tried to close a position (Market Sell) while the stock was blocked by a previously opened Limit (Take Profit) order.

**Titan V7 Solution (March 10, 2026):**
At every Exit signal (Stop Loss, Take Profit or Manual Sell), the bot now performs a proactive **"Blind Cancel"**:

1. Wait **500 milliseconds** (to ensure synchronization of Alpaca servers).
2. Wait **500 milliseconds** (to ensure synchronization of Alpaca servers).
3. Send the liquidation Market Sell order.

This logic releases the blocked amount of shares and is designed to make the liquidation succeed reliably, eliminating the 403 error.

### 10.8 Seeding Price Cache at Exit

The autopilot uses `state.last_prices` for risk checks. When exiting a position, the current price may not be in the cache (if the symbol is no longer actively streaming). The solution:

Before sending the close command, the autopilot copies `current_price` from the Alpaca position data to `last_prices`. This prevents the `"NO PRICE DATA for [SYMBOL]"` error.

---

## 11. SAFETY GATES

The safety filters are **independent** of the strategy and run before each potential entry. Even if the strategy generates a strong signal, if a filter blocks, the entry is NOT made.

### 11.1 Full List of Filters

| # | Filter | When it Blocks | Message in Log |
|---|--------|---------------|--------------|
| 1 | **Circuit Breaker** | Level != NORMAL | `CIRCUIT BREAKER: [level]` |
| 2 | **Spread Guard** | Spread > configured limit | `SPREAD GUARD: spread [x] > max [y]` |
| 3 | **OFI Filter** | OFI does not confirm the direction of the signal | `OFI FILTER: signal blocked` |
| 4 | **Cooldown** | Insufficient time since last entry | `COOLDOWN: [x]s remaining` |
| 5 | **Max Positions** | Maximum number of positions reached | `MAX POSITIONS: [n] reached` |
| 6 | **Feeling Filter** | Feeling too negative to buy (or too positive to sell) | `SENTIMENT FILTER: blocked` |
| 7 | **Price Validity** | The price is 0 or invalid | `NO PRICE DATA` |

### 11.2 Spread Guard — Details

**What is the Spread?** The difference between the best bid and the best ask in the orderbook.

**Formula:** `spread_pct = (best_ask - best_bid) / midpoint * 100`

**Configurable threshold:** The SPREAD GUARD threshold is controlled by `max_spread_bps` in the **HFT Engine tab** (Safety section). Default: **50 bps** (0.50%). Configurable range: 1 — 500 bps. Previously it was hardcoded to 15 bps; now it directly uses the value from settings.

**Conversion:** `max_spread_bps / 100 = procent maxim spread acceptat`. Example: 50 bps = 0.50%, 200 bps = 2.00%.

**Why it's important:** A large spread means you pay an "entry cost" high. On liquid markets (broad-cap, BTC/USD), the spread is 5-20 bps. On low-volume micro-caps, it can be 100-300 bps. Adjust `max_spread_bps` according to the type of traded assets.

**SPREAD GUARD messages in the log:**

- `SPREAD GUARD: 152.7 bps — too wide, skipping entries` → The current spread exceeds the configured limit
- These are **informational messages**, not errors. It means that the bot protects the capital while waiting for better conditions.

**Recommendations per asset type:**

- Large-cap (AAPL, MSFT, NVDA): 20-50 bps
- Mid-head: 50-100 bps
- Small/micro-cap (SOC, QBTS): 150-250 bps
- Crypto (BTC/USD): 10-30 bps

**Overnight/No-Data Exception:** If the reported spread is 0.0 bps (no data outside of business hours), the bot automatically becomes "safe" allowing transactions to position themselves instead of being artificially blocked by the lack of deep connection.

### 11.3 Advanced Safety Gates (S04.1) 🆕

Session 48 added 4 new configurable safety gates from the **⬡ SAFETY GATES** panel in the HFT Engine tab:

| # | trappings | Settings | Effect | Effect |
|---|------|---------|---------|-------|
| 8 | **VaR Fail-Closed** | `var_fail_closed` | MR | Block entry if there are no return data (fail-safe) |
| 9 | **Asset Check Hard** | `asset_check_hard_gate` | MR | Reject orders when the assets API is unavailable |
| 10 | **Blackout First** | `blackout_first_mins` | 5 | Block entry in the first N minutes after open (9:30am ET) |
| 11 | **Blackout Last** | `blackout_last_mins` | 5 | Block entry in the last N minutes before close (4:00 PM ET) |

**Blackout Exempt Symbols:** Symbols that bypass the blackout (ex: `SPY, QQQ`). Configurable in the UI.

**Min Dollar Volume:** Scanner filter — remove tickers with `price × volume < $5M` (default). Prevents trading on illiquid instruments.

**Validation Layer:** Automatic warnings if:
- Total blackout >= 195 min (half of the trading day)
- Dollar volume < $100K (illiquid risk)
- VaR gate ON but fail-closed OFF (risk gap)

### 11.4 How to Interpret the Log

When you see many SPREAD GUARD or OFI FILTER messages, it does not mean that the bot is broken. It means:

- The market does not offer optimal entry conditions at that moment
- The filters work correctly protecting the capital
- When the conditions improve, the bot will enter automatically

---

## 12. GENETIC OPTIMIZER — AUTOMATIC EVOLUTION OF PARAMETERS

### 12.1 What optimizes

| Parameter | Default | rank |
|-----------|---------|-------|
| `fusion_threshold` | 50.0 | ±15% of safety rails |
| `tp_percent` (Take Profit) | 2.5% | 0.5% — rails*1.15 |
| `sl_percent` (Stop Loss) | 2.5% | 0.5% — rails*1.15 |
| `trail_gap_pct` | 1.5% | 0.3% — rails*1.15 |
| `min_confidence` | 0.30 | 0.10 — 0.90 |
| `ofi_filter_enabled` | false | 90% keep, 10% flip |
| `nexus_buy_threshold_pct` | 60% | 50 — 90 |

### 12.2 How it works

**Population:** 20 individuals (sets of parameters)

**Evolution cycle (every 60 seconds):**

1. **Fitness evaluation** on in-sample data (first 70% of candles)
2. **Sort** by fitness
3. **Selection:** Top 5 = elites (survive intact)
4. **Reproduction:** 15 new offspring from combinations of elites
   - 70% chance: crossover (2 parents) + mutation
   - 30% chance: direct mutation (1 parent)
5. **Mutation:** Each parameter ±15% with random probability
6. **OOS validation:** The champion is tested on the out-of-sample data (the last 30%)
7. **Promotion:** The champion is ONLY promoted if:
   - Fitness IS > current fitness champion
   - Sharp IS > 0
   - Sharpe OOS > 0 and PnL OOS > 0

### 12.3 Walk-Forward Backtest

- **In-Sample (70%):** Fitness assessment
- **Out-of-Sample (30%):** Anti-overfitting validation
- **Logic:** SMA(5) vs SMA(20) crossover + volatility + threshold
- **Fitness formula:** `sharpe * min(profit_factor, 5) + trade_bonus - dd_penalty`

### 12.4 Safety Rails

All parameters are clamped within strict limits:

- TP/SL cannot be below 0.5%
- Trail gap cannot be below 0.3%
- Confidence between 0.10 and 0.90
- Threshold between 50 and 90

### 12.5 What you see in the UI

- **Gender:** Current generation (increases with each evolution)
- **Fitness:** The best score in the population
- **Sharpe:** The champion's Sharpe
- **TP/SL:** Active Take Profit / Stop Loss parameters
- **FREEZE:** Stops the evolution (keeps the current parameters)
- 13. CIRCUIT BREAKER — SECURITY SYSTEM

---

## Effect

### 13.1 The 4 Levels

| Level | Trigger | Effect | UI color |
|-------|---------|-------|------------|
| **NORMAL** | Daily PnL > -2% and DD > -5% | Normal trading, full size | Verdant |
| **WARNING** | Daily PnL ≤ -2% | Halved positions (50%) | Yellow |
| **HALT** | Daily PnL ≤ -3% | New entries BLOCKED | Orange |
| **PANIC** | DD ≤ -5% OR Daily PnL ≤ -5% | Kill switch — LIQUIDATE EVERYTHING | Red |

### 13.2 How it works

- **Continuous evaluation** at every tick of the autopilot
- **Configurable Thresholds** from the Settings sidebar
- **Kill Switch:** In PANIC mode, it activates automatically and closes ALL positions
- **Size reduction:** In CAUTION, orders are at 50% of the normal size

### 13.3 Reset

- **Automatic:** At daily reset (09:30 ET)
- **Manual:** The RESET button available in the UI
- **From code:** Order `reset_circuit_breaker`

### 13.4 Emergency Liquidation

The **LIQUIDATE ALL** button in the UI:

1. Activate the kill switch
2. Block the entrances
3. Cancel ALL open orders
4. Close ALL positions
5. Issue event "breaker-triggered" with PANIC level

---

## 14. DRY RUN MODE — SIMULATION WITHOUT RISK

### 14.1 What it does

When Dry Run is activated:

- **NO** orders are sent to Alpaca
- Instead, it is logged and displayed in the Activity Log as "DRY RUN ORDER"
- All other functions (OFI, Oracle, Circuit Breaker) work normally
- Ideal for testing in real market conditions

### 14.2 Activation/Deactivation

- **From the UI:** Click on the **DRY RUN: OFF** button → it becomes **DRY RUN: ON**
- **From environment:** Setting `TITAN_DRY_RUN=1` before starting
- **From code:** Order `set_dry_run(true/false)`

**Persistence:** The Dry Run state is saved in `titan_settings.json`. When restarting, the bot remembers whether it was in Dry Run or not.

**Default (S72):** Fresh install starts with `dry_run_enabled: true`. Live trading must be activated explicitly.

**Visual guard (S72):** When dry_run is OFF (live trading), the UI displays a pulsating red border on the entire screen + banner `LIVE TRADING — DRY RUN OFF` to prevent accidental trading.

### 14.3 What happens in Dry Run

- `send_order_internal` → Log the order, issue `dry-run-order`, return OK
- `liquidate_all_internal` → Log in, return without action
- Activity Log shows: "DRY RUN: BUY 0.5 BTC/USD (simulated)"

---

## 15. WATCHDOG AND TELEMETRY

### 15.1 Watchdog — Monitoring Loops

Each loop in the background sends a **heartbeat** at each iteration. The watchdog monitors:

- **tick_count:** How many iterations the loop did
- **last_heartbeat:** When was the last heartbeat
- **last_tick_ms:** Duration of the last iteration
- **error_count:** How many errors occurred

**Watched loops (13):** autopilot, nexus, volatility, equity_sync, position_recon, trade_updates, ai_scanner, daily_reset, genetic_evo, catalyst_watchdog, streamer, news, telegram.

### 15.2 Dead Loop Detection

- **Threshold:** 120 seconds without heartbeat = loop considered DEAD
- **Watchdog Supervisor:** Checks every 30 seconds
- **Alert:** Issue `watchdog-alert` to the UI with the list of dead loops
- **UI:** The LOOP HEARTBEATS panel shows "ALL ALIVE" (green) or "X DEAD LOOPS" (red)

### 15.3 Cancel/Join Contract on Respawn (S03)

The watchdog stores one `AbortHandle` per loop. On respawn:

1. **Abort** -- the old task is explicitly aborted (it no longer runs in parallel)
2. **Spawn** -- newly started task and stored handle
3. **Log** -- "Aborted old task handle before respawn"

This eliminates the risk of **split-brain** when a "dead" loop it is still running and a new spawn duplicates it.

### 15.4 Generation ID — Automatic Invalidation (S03)

**`session_generation`** is a counter `AtomicU64` on `TitanState`, incremented on respawn of critical loops: **streamer**, **autopilot**, **trade_updates**.

Each critical loop captures the generation at the start. At each tick, check `state.current_generation() == my_generation`. If it does not coincide, the loop automatically exits with a log:

```
"Generation mismatch — exiting stale autopilot instance"
```

**Effect:** operations from the previous generation are automatically invalidated (fail-closed). There is no window in which two instances of the same loop process orders simultaneously.

### 15.5 What you see in the UI (LOOP HEARTBEATS)

```
LOOP HEARTBEATS  ALL ALIVE
autopilot      1s/214        (214 tick-uri, ultimul acum 1s)
nexus          0s/641        (641 tick-uri, ultimul acum <1s)
l2_stream      0s/79991      (79991 mesaje procesate)
trade_updates  13s/22        (22 mesaje, ultimul acum 13s)
news           120s/5        (5 fetch-uri, interval 300s)
telegram       0s/48         (48 poll-uri)
...
```

### 15.6 Detailed Telemetry

The command `get_telemetry` returns per loop:

- `name`, `tick_count`, `error_count`
- `last_tick_ms`, `avg_tick_ms`
- `last_heartbeat_secs_ago`
- `is_alive` (true/false)
- `uptime_secs` — how many seconds does the loop run without restarting (reset on respawn)

### 15.7 Watchdog Health Event (S03b)

At every 30-second tick, the watchdog supervisor issues an event **`watchdog-health`** to the frontend with the complete snapshot of all loops, the current generation and the number of generation mismatches:

```json
{
  "loops": [ { "name": "autopilot", "tick_count": 214, "uptime_secs": 3600.5, ... } ],
  "generation": 3,
  "generation_mismatches": 1
}
```

This gives continuous visibility on the status of all loops, not just the dead ones.

### 15.8 Generation Mismatch Counter (S03b)

**`generation_mismatch_count`** is a `AtomicU64` on `TitanState`. Automatically increased in the 3 critical loops (autopilot, trade_updates, streamer) when it detects a generation mismatch and exits. The value appears in the `watchdog-health` event under the `generation_mismatches` key.

### 15.9 Snapshot Integrity Hash (S03b)

At each `save_state_snapshot`, after JSON serialization, SHA-256 is calculated on the payload and written in a `.sha256` file along with the snapshot. At `load_state_snapshot`, if the file `.sha256` exists, the hash is checked. Mismatch = log error + snapshot rejected (corrupted data is not restored).

---

## 16. LOGGING AND REPORTS

### 16.1 Structured Logging

- **Library:** `tracing` + `tracing-subscriber` + `tracing-appender`
- **Director:** `$HOME/HFT_Bot/logs/`
- **Format:** `titan.log.YYYY-MM-DD` (automatic daily rotation)
- **Content:** Timestamps UTC RFC-3339, thread IDs, target module, level (DEBUG/INFO/WARN/ERROR)
- **Two simultaneous outputs:**
  - File: Full detail with target and thread
  - Stdout: Compact (visible in the terminal)

### 16.2 Session Reports

At **shutdown** and **daily reset** (09:30 ET), a JSON report is saved:

**Location:** `$HOME/HFT_Bot/reports/session_YYYY-MM-DD.json`

**Content:**

- Session date
- Total trades, winning, losing
- Daily PnL, Max Drawdown
- Win Rate, Profit Factor, Sharpe
- Peak Equity, Final Equity
- Champion params (from genetic optimizer)
- Loop telemetry (all loops)
- Breaker events count
- Dry run status

### 16.3 Activity Log (UI)

All important events appear in the ACTIVITY LOG panel:

- Boot checks
- Trade fills
- Circuit breaker events
- WebSocket connections/disconnections
- OFI updates
- errors

---

## 17. BACKTESTING

### 17.1 Types of Backtest

| Type | Description | Command |
|-----|-----------|---------|
| **Single Backtest** | A symbol, a timeframe | `run_backtest` |
| **TF Sweeps** | One symbol, all timeframes | `run_backtest_sweep` |
| **Multi Sweep** | More symbols, all TFs | `run_backtest_sweep_multi` |
| **Quick Backtest** | With the active parameters and the current symbol | `run_backtest_now` |

### 1m, 5m, 15m, 30m, 1h, 4h, 1d

1m, 5m, 15m, 30m, 1h, 4h, 1d

### 17.3 Export

- **CSV:** Tabular data for Excel/Google Sheets
- **JSON:** Structured data for programmatic analysis
- **HTML:** Visual report with graphics
- **Multi-Sweep CSV:** Comparison between symbols and timeframes

### 17.4 Auto-Optimize

The **AUTO TUNE TF** button runs `optimize_timeframe`:

- Test all timeframes for the last 30 days
- Select the timeframe with the best Sharpe ratio
- Automatically updates the active timeframe

---

## 18. PROFESSIONAL SCHEDULE

### 18.1 Bookstore

**TradingView LightweightCharts** — professional chart with:

- Candlestick OHLC
- Volume histogram
- Multi-pane layout (chart, volume, RSI, MACD, ADX)

### 18.2 Available Indicators

| Indicator | fried | Configurator |
|-----------|------|-------------|
| **EMA 9** | Main chart | Period |
| **EMA 50** | Main chart | Period |
| **EMA 200** | Main chart | Period |
| **VWAP** | Main chart | - |
| **Bollinger Bands** | Main chart | Period, StdDev |
| **RSI** | Sub-chart | Period (lines 30/50/70) |
| **MACD** | Sub-chart | Fast/Slow/Signal |
| **ADX** | Sub-chart | Period (lines 20/25) |
| **Oracle Score** | Sub-chart | - |

### 18.3 Drawing Tools

- Crosshair, Trendline, Ray, Horizontal Line, Vertical Line
- Fibonacci Retracement
- Text annotation
- Measure tool
- Long/Short position markers
- Delete All, Reset

### 18.4 Special Functions

- **Replay Mode:** Simulates the price movement in real time (adjustable speed)
- **Magnet:** Snap to OHLC
- **Screenshot:** Chart capture
- **Fullscreen:** Full screen chart
- **Resize:** Panes are resizable (P+/P-, H+/H-)
- **Timeframes:** 1m, 5m, 15m, 30m, 1h, 4h, 1d

### 18.5 TITAN Oracle Fullscreen

The Oracle histogram can be viewed fullscreen:

- **FS Button** (upper-right corner of the Oracle panel)
- Fullscreen covers the entire viewport (position: fixed)
- The graph is automatically resized to the viewport
- Click FS again to return to normal size

### 18.6 Live Feed

Below the chart is displayed:

- **BID / ASK** live (from L2 stream)
- **SPR** (Spread)
- **BIDS / ASKS** lists (orderbook top levels)
- **Orderbook Heatmap** (depth view)

---

## 19. NEW TABS (NEWS AI, JOURNAL, HFT ENGINE, PERFORMANCE)

### 19.1 News AI (tab-1)

Tab dedicated to monitoring news with sentiment analysis.

**Components:**

- **Sentiment Gauge:** Circular visual indicator (-100 to +100) with gradient colors
- **News Timeline:** Chronological flow of news with timestamps
- **News Cards:** Each news item displays:
  - Title and source
  - Sentiment score (positive/negative/neutral)
  - Color: green (bullish), red (bearish), gray (neutral)
- **Statistics:** Average sentiment over the last X hours

**Sentiment Analysis:**

- Keyword-based scoring with extended lexicon
- Bullish words: "rally", "surge", "breakthrough", "uptick", "bullish", etc.
- Bearish words: "crash", "plunge", "selloff", "bearish", "decline", etc.
- The score is calculated per headline/summary and averaged

### 19.2 Journal (tab-3)

Persistent transaction history.

**Functionality:**

- **Transaction table:** Date, symbol, direction, quantity, price, PnL
- **Filters:** By symbol, date, direction (buy/sell), result (win/loss)
- **Graphs:** Equity curve, PnL per trade, distribution results
- **Export:** CSV export button for external analysis
- **Persistence:** Data is stored in `journal.rs` (Rust backend), not in the browser

### 19.3 HFT Engine (tab-4)

Advanced settings for the trading engine:

**Entry Parameters:**

- Fusion threshold, min confidence
- OFI filter toggle
- Cooldown interval

**Exit Parameters:**

- Take Profit %, Stop Loss %
- Trail gap %
- Max holding time

**Safety:**

- Circuit breaker thresholds (editable)
- Max open positions
- Risk per trade (short slider, 15% of width)

**Alerting:**

- Discord webhook URL
- Event selection: fills, breaker events, daily summary
- Test alert button

**Sliders in HFT Engine:** They have reduced width (15% of normal) to take up less space and look compact.

**SPREAD GUARD — Connection to Settings:**
Parameter `max_spread_bps` in HFT Engine directly controls the SPREAD GUARD threshold in the autopilot. Previously it was hardcoded at 15 bps (0.15%), now it uses the value from settings (default 50 bps = 0.50%). This value determines the maximum bid-ask spread accepted to enter a trade. Assets with low liquidity (micro-caps) have spreads of 100-300 bps and require an increase in this value.

### 19.4 Performance Dashboard

Integrated in the TITAN SAFETY panel and in the backtest tabs:

**KPI Cards:**

- Daily PnL with trend arrow
- Win Rate with progress bar
- Sharpe Ratio
- Profit Factor
- Max Drawdown
- Total Trades

---

## 20. AI SCANNER — FUSION ASSET DISCOVERY

### 20.1 What is AI Scanner

AI Scanner is an intelligent engine for discovering profitable assets. It combines quantitative filters (quant) with FUSION scoring (Oracle + GPU Nexus) and automatic backtest on all timeframes to find and classify the most promising US stocks.

Accessible from the **AI Scanner** tab (tab-6) in the tab bar in the central panel.

### 20.2 The Scanning Pipeline

Scanning takes place in 4 phases:

1. **Fetch Assets** — Download the complete list of active stocks from Alpaca (~8000+)
2. **Snapshots** — Load daily data (price, volume, previous day) in batches of 1000
3. **Pre-filter Quant** — Apply quantitative filters and sort by composite score
4. **FUSION Scoring + Sweep Backtest** — For each candidate:
   - Download 5m candles
   - Run Oracle 7-layer (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, Changepoint)
   - Dispatches GPU Nexus (150,000 agents vote)
   - Calculate FUSION score (Oracle *(1-alpha) + Nexus* alpha)
   - Run backtest sweep on **8 timeframes**: 1m, 5m, 10m, 15m, 30m, 1h, 4h, 1D
   - Keep the best timeframe (after Sharpe + Return combined)
   - Calculate composite AI Score

### 20.3 Filters Quantitative (Pre-filter)

| Filter | Condition |
|--------|----------|
| Price | $1 - $500 |
| Minimum Volume | 500,000 |
| Minimum Change | > 1% (absolute) |
| RVOL | volume_today / volume_yesterday |
| Gap % | Volume spikes |
| Volume spikes | volumes > 2x prev_volume |

**Pre-filter scoring:**

- Absolute change% (max 20%) * 2
- RVOL > 2x: (RVOL-1) * 5 (max 50 points)
- Gap > 1%: gap * 3 (max 30 points)
- Volume spike bonus: +15 points
- High volume bonus: +5 (>2M) or +10 (>5M)

### 20.4 AI Score — Composite Formula

```
AI Score = 0.40 * FUSION_normalized +
           0.25 * Sharpe_normalized +
           0.20 * Win_Rate +
           0.15 * ProfitFactor_normalized
```

Conditions: minimum 3 trades in backtest and Oracle confidence > 0.15. Otherwise, AI Score = FUSION * 0.5.

### 20.5 Multi-Timeframe Sweep Backtest

Each scanned asset is backtested on all 8 timeframes. The timeframe with the best combined score (Sharpe + Return * 0.01) is kept. The **BEST TF** column in the table shows the optimal timeframe.

The depth of the backtest depends on the data available from Alpaca (up to 365 days, limit 1000 candles per timeframe).

### 20.6 AI Scanner Table Columns

| Column | Description |
|---------|-----------|
| # | Rank by AI Score |
| SYMBOL | Action symbol |
| PRICES | Current price |
| CHG% | Percentage change compared to the previous day |
| RVOL | Relative Volume (today / yesterday) |
| GAP% | Gap at the opening compared to the previous close |
| VOL | Volume of the day |
| FLOAT | Estimated float category: LOW / MED / HIGH |
| ORACLES | Oracle Score (-100 to +100) |
| diet | Market regime: Trending / MeanReverting / Chaotic |
| NEXUS | GPU Nexus buy % (150K agents) |
| FUSION | Combined FUSION Score (-100 to +100) |
| SIGNAL | Signal: LONG / SHORT / NEUTRAL |
| BEST TF | The optimal timeframe from the sweep backtest |
| BETRAYED | Number of transactions from the backtest |
| WIN% | Win rate from backtest |
| SNAKE | RET% |
| RET% | Return % from backtest |
| EXPECT | Expectancy per trade from backtest |
| YOU GOT SCORE | Final Composite Score (0-100) |

### 20.7 UI Functionalities

- **AI AUTO-SCAN (switch ON/OFF):** In the header of the AI ​​Scanner tab, the "AI AUTO-SCAN" switch enable/disable automatic background scanning (every 30 minutes). When it is OFF, only SCAN NOW manually runs. The condition persists in `titan_settings.json` (`ai_scan_active`).
- **Candidate switch (50/100/200):** Controls the number of assets passed through the pre-filter
- **SCAN NOW:** Starts the manual scan (lasts 2-10 minutes depending on the number of candidates)
- **Progress bar:** Displays the current phase and the processed symbol in real time
- **Client-side filters:** Symbol search, Signal (LONG/SHORT/NEUTRAL), Regime
- **Sorting:** Click on any column header to sort asc/desc
- **Color coding:** Green (LONG), red (SHORT), neutral (gray) rows
- **Checkbox per row:** Select the desired assets
- **INJECT SELECTED → HFT:** The green button adds the selected assets directly to the HFT Engine targets and activates multi-symbol mode

### 20.8 SMART PICKS (Premium Filtered Assets)

The **⚡ SMART PICKS** panel is a sub-section of AI Scanner that cleans out the noise and shows only the "cream" market.
The filter performs a POST-scan and retains exclusively the momentum (high-reward) candidates:

- **Trades:** Minimum 5 trades in backtest (Eliminate accidental results from only 2-3 trades)
- **Trades:** Minimum 5 trades in backtest (Eliminate accidental results from only 2-3 trades)
- **Smart Picks functionalities:**

**Smart Picks functionalities:**

- Includes the **BEST TF** column indicating the exact time interval where profitability has been demonstrated in history.
- **Top N Selector:** Allows the isolation of the first 3, 5, 10 or 20 assets (Top 20 offers a solid margin of error against macro risk).
- **MANUAL Mode:** The user can manually review the Premium list and inject using **INJECT ALL -> HFT** (buttons indicating visual instant `✓ ACTIVE`).
- **AUTO Mode:** Automatically injects into the HFT Engine the configured number (Top N) of coins directly from the scanner upon its completion (total ease in the background).

### 20.9 Auto-Scan (Background)

- `ai_scan_loop` runs automatically every **30 minutes** during market hours (09:00-17:00 ET) — **only if the AI ​​AUTO-SCAN switch is ON**
- The switch in the header allows stopping the auto-scan without stopping the application (saves API calls and CPU)
- Scan the top 100 candidates
- Issue `ai-scan-update` event — the table is updated automatically
- Heartbeat to watchdog every 60 seconds
- It does not scan outside of market hours

### 20.10 Injection into HFT Engine

After scanning, select the desired assets with the checkbox and press **INJECT SELECTED → HFT**. This:

1. Add symbols to list `hft_targets` from `titan_settings.json`
2. Activate `multi_symbol_enabled = true`
3. The autopilot will start evaluating these symbols on the next iteration

---

## 21. MANAGEMENT OF POSITIONS AND FUNDS

### 21.1 Positions

- **Automatic synchronization:** Every 15 seconds, the positions are synchronized from the Alpaca
- **Display:** In the "Active Portfolio" panel with symbol, price, TF, type, invested, P/L
- **Actions per position:** SELL button for manual closing

### 21.2 Equity Sync

- Every 30 seconds, equity is synced from Alpaca
- Display in the TITAN SAFETY panel and in the sidebar

### 21.3 Trade Updates (Fill Notifications)

- WebSocket connected to Alpaca Trade Updates stream
- Receive real-time notifications for:
  - Fills
  - Partially executed orders
  - Canceled orders
- Display in the Activity Log: symbol, quantity, average price

### 21.4 Funds

| Action | Button | what is he doing |
|---------|-------|---------|
| **Top Up User** | TOPUP USER | Add to the user's virtual storage |
| **Top Up Bot** | TOP UP | Allocate funds to the bot from the user deposit |
| **Withdraw** | WITHDRAW | Withdraw funds from the bot to the user |

---

## 22. ALL THE LOOPS IN THE BACKGROUND

| # | Loop | Interval | what is he doing |
|---|------|----------|---------|
| 1 | **autopilot_loop** | 1.5s | Automatic trading: analysis, entries, exits, trailing stop |
| 2 | **nexus_loop** | 500 ms | GPU Nexus: 150K agents vote, calculated consensus |
| 3 | **update_volatility_loop** | 30s | Updates the volatility cache |
| 4 | **equity_sync_loop** | 30s | Synchronize equity on Alpaca |
| 5 | **position_reconciliation_loop** | 15s | 6 |
| 6 | **trade_updates_stream** | continuously | WebSocket: fill notifications in real time |
| 7 | continuously | continuously | L2 orderbook stream (crypto v1beta3 / stocks SIP) |
| 8 | **daily_reset_loop** | 60s (check) | Resets daily at 9:30am ET |
| 9 | **genetic_evolution_loop** | 60s | Genetic evolution of trading parameters |
| 10 | **watchdog_supervisor** | 30s | Detects dead loops |
| 11 | **startup_health_check** | once (boot) | Check all dependencies |
| 12 | **streamer::run_streamer** | Event Driven | Price streaming and heatmap |
| 13 | **ai_scan_loop** | 30min (heartbeat 60s) | AI Scanner: pre-filter + FUSION + sweep backtest |
| 14 | **state_snapshot_loop** | 30s | Snapshot state on disk (crash recovery) |

---

## 23. ALL ORDERS AVAILABLE

### 23.1 Trading

| Command | what is he doing |
|---------|---------|
| `submit_order` | Send order (through safety checks + risk sizing) |
| `liquidate_all_positions` | Close all positions |
| `cancel_all_orders` | Cancel all open orders |
| `emergency_liquidate` | Kill switch + liquidated |
| `set_dry_run` | Enables/disables the simulation |

### 23.2 Strategy

| Command | what is he doing |
|---------|---------|
| `set_strategy_mode` | Set Oracle / GPU / Fusion |
| `toggle_strategy_mode` | Cycle between modes |
| `set_nexus_alpha` | Adjust the Fusion blend (0-1) |
| `get_nexus_status` | Nexus GPU status |

### 23.3 Genetic Optimizer

| Command | what is he doing |
|---------|---------|
| `freeze_evolution` | Stops Fusion evolution |
| `resume_evolution` | Fusion evolution starts |
| `get_evolution_stats` | Fusion optimizer statistics |
| `get_active_trading_params` | Active Fusion parameters |
| `run_backtest_now` | Quick backtest |

#### SmartTrend Genetic (S13)

| Command | what is he doing |
|---------|---------|
| `get_st_evolution_stats` | SmartTrend optimizer statistics (gen, fitness, sharpe, diversity, champion params) |
| `get_st_fitness_history` | IS/OOS fitness history per generation (for convergence chart) |
| `get_st_champion_params` | Drill-down champion: each param with value, default, delta%, min, max |
| `run_comparison_backtest_cmd` | Run Fusion + SmartTrend on the same candle, return metrics side-by-side |
| `freeze_st_evolution` | Stops SmartTrend evolution independently |
| `resume_st_evolution` | SmartTrend evolution starts |

**Conditional behavior:** When `StrategyMode::SmartTrend` is active, `genetic_evolution_loop` automatically uses `SmartTrendParams` and `smarttrend_walk_forward_backtest` (real FSM: Flat/Stalking/Entry/InTrade/Trail). In any other mode, run the classic Fusion optimizer with `TradingParams`. The two optimizers are completely independent.

**Evolvable SmartTrendParams (12):** macro_weight, micro_weight, momentum_weight, entry_threshold, stalking_threshold, changepoint_reset, trail_tighten_pct, hysteresis_margin, bayesian_halflife, stabilization_window, dominant_cycle_filter, risk_per_trade. Each has safety rails (min/max) hardcoded.

**Fitness SmartTrend:** `sharpe * min(PF, 5) + trade_count_bonus + win_rate_bonus + stability_bonus - dd_penalty`. Win rate active bonus over 55%. Stability bonus when avg_bars_in_trade between 3-30. OOS validation: champion must be sharpe > 0 and total_pnl > 0 on 30% out-of-sample.

**UI Evolution Panel (S13):**
- **Convergence Chart:** Canvas in the EVOLUTION panel — line IS fitness (cyan) + OOS (gold dashed) on generations, green dots on promoted champion, diversity bar at the base
- **PARAMS button:** Click -> modal drill-down with all champion params, delta% vs default, heat-color bar proportional to magnitude
- **DIV badge:** Badge "DIV: N%" in the header panel — red with pulses below 15% (premature convergence), cyan 15-40%, green above 40%
- **Comparison Panel:** New section in tab-13 SMARTTREND CFG — RUN COMPARISON runs both strategies on the same data, grid with winner highlighting, verdict banner

### 23.4 Risk & Safety

| Command | what is he doing |
|---------|---------|
| `get_risk_status` | Circuit breaker + session analytics |
| `reset_circuit_breaker` | Manual breaker reset |
| `reset_daily_analytics` | Reset daily analytics |
| `get_ofi_status` | OFI status live |

### 23.5 Account & fund

| Command | what is he doing |
|---------|---------|
| `connect_api` / `update_api_keys` | Connect/update API keys |
| `get_account_snapshot` | Alpaca account snapshot |
| `topup_user` / `topup_bot` / `withdraw_bot` | Fund management |
| `get_wallet_snapshot` | Wallet status |

### 23.6 Backtesting

| Command | what is he doing |
|---------|---------|
| `run_backtest` | Backtest alone |
| `run_backtest_sweep` | Sweep all TFs |
| `run_backtest_sweep_multi` | Multi-symbol sweep |
| `optimize_timeframe` | TF auto-optimization |
| `export_backtest_csv/json/html` | Export results |

### 23.7 Monitoring

| Command | what is he doing |
|---------|---------|
| `get_telemetry` | Telemetry loops |
| `get_monitor_snapshot` | Hardware (CPU, GPU, RAM, NET) |
| `graceful_shutdown` | Gracious stop |

### 23.8 Settings & Persistence

| Command | what is he doing |
|---------|---------|
| `update_settings` | Updates any setting (field + value) — includes `bot_trade_size`, `circuit_breaker_limit`, `dry_run_enabled`, `strategy_mode_name` |
| `get_favorites` | Returns the list of favorite symbols |
| `add_favorite` | Add symbol to favorites (persistent) |
| `remove_favorite` | Delete symbol from favorites (persistent) |

### 23.9 AI Scanner

| Command | what is he doing |
|---------|---------|
| `run_ai_scan` | Start AI scan with configurable number of candidates (50/100/200) |
| `activate_scan_targets` | Injects selected symbols into HFT Engine targets |

---

## 24. STRUCTURE OF FILES

```
$HOME/HFT_Bot/
├── titan_keys.json              ← API keys (CONFIDENTIAL)
├── titan_settings.json          ← Setari persistente (dry_run, strategy_mode, favorites, etc.)
├── titan_wallet.json            ← Stare portofel (user deposit, bot alloc)
├── titan_state_snapshot.json    ← State snapshot (crash recovery, auto-resume)
├── titan_position_blacklist.json ← Blacklist pozitii force-closed
├── TITAN_MANUAL.md              ← This manual
├── logs/
│   └── titan.log.YYYY-MM-DD    ← Loguri zilnice (rotatie automata)
├── reports/
│   └── session_YYYY-MM-DD.json ← Rapoarte zilnice
├── titan_hft_rust/
│   ├── src/
│   │   └── index.html           ← Frontend UI (sursa)
│   ├── dist/
│   │   └── index.html           ← Frontend build (embedded in binary)
│   └── src-tauri/
│       ├── Cargo.toml            ← Dependente Rust
│       ├── tauri.conf.json       ← Configurare Tauri (distDir → ../dist)
│       ├── src/
│       │   ├── main.rs           ← Orchestrator (~60 comenzi Tauri, 12 loop-uri)
│       │   ├── titan_oracle.rs   ← 7 indicatori AI
│       │   ├── gpu.rs            ← GPU compute (WGPU)
│       │   ├── risk.rs           ← Circuit breaker, risk management
│       │   ├── genetic.rs        ← Optimizator genetic
│       │   ├── l2_heatmap.rs     ← L2 orderbook module (date procesate in streamer.rs)
│       │   ├── watchdog.rs       ← Loop monitoring, telemetrie
│       │   ├── titan_log.rs      ← Logging structurat
│       │   ├── indicators.rs     ← Indicatori clasici
│       │   ├── strategy.rs       ← Logica strategie
│       │   ├── streamer.rs       ← Price streaming + L2/OFI processing (WebSocket unic)
│       │   ├── scanner.rs        ← Market scanner
│       │   ├── monitor.rs        ← Hardware monitor
│       │   ├── sentiment.rs      ← Sentiment analysis (keyword lexicon)
│       │   ├── journal.rs        ← Jurnal tranzactii persistent (NOU)
│       │   ├── app_state.rs      ← State definitions + AppSettings
│       │   ├── nexus_kernel.wgsl ← GPU shader: 150K agenti
│       │   ├── gpu_indicators.wgsl
│       │   ├── gpu_indicators_batch.wgsl
│       │   └── gpu_dot.wgsl
│       └── target/
│           └── release/
│               └── titan_hft_rust  ← EXECUTABIL FINAL
```

---

## 25. FREQUENT PROBLEMS AND SOLUTIONS

### 25.1 L2 Stream does not receive data

**Symptom:** OFI LIVE shows 0.0, BID/ASK = 0

**Possible causes:**

- **Crypto:** Endpoint must be `v1beta3/crypto/us` (not `v2/sip`)
- **Stocks:** Requires **Algo Trader Plus** plan ($99/mo) for SIP
- **API Keys:** Check that they are correct and without spaces

**Verification:** In the terminal, search for:

```
L2 SUB RESPONSE: [{"T":"subscription","orderbooks":["BTC/USD"],...}]
```

### 25.2 Trade Updates does not authenticate

**Symptom:** "Auth failed" in the Activity Log

**Solution:** Titan automatically tries two auth formats:

1. `authenticate` with `key_id`/`secret_key` (format v2)
2. `auth` with `key`/`secret` (legacy format)

Check the Activity Log for the exact message from the server.

### 25.3 Circuit Breaker activates

**Symptom:** TITAN SAFETY shows CAUTION/HALT/PANIC

**Solution:**

- **CAUTION:** Normal — reduces positions automatically. Wait for recovery.
- **HALT:** Inputs are blocked. Review the strategy. Manual reset if necessary.
- **PANIC:** Automatic liquidation. Investigate the cause. Reset after analysis.

### 25.4 Dead loop (DEAD)

**Symptom:** LOOP HEARTBEATS shows a loop with `> 120s`

**Solution:** Restart the application. If it persists, check the logs for errors specific to the respective loop.

### 25.5 GPU is not detected

**Symptom:** Health Check shows "GPU (WGPU): FAILED"

**Solution:**

- Check NVIDIA drivers: `nvidia-smi`
- Check Vulkan support: `vulkaninfo`
- Set `WEBKIT_DISABLE_COMPOSITING_MODE=1` (automatically done)

### 25.6 EGL spam in terminal

**Symptom:** Repetitive EGL/WebKit messages in terminal

**Solution:** Titan automatically sets `WEBKIT_DISABLE_COMPOSITING_MODE=1` at startup. If it persists, add to `.bashrc`:

```bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
```

### 25.7 "Connection limit exceeded (code 406)"

**Symptom:** In Activity Log appears repeatedly:

```
L2 STREAM AUTH ERROR: [{"T":"error","code":406,"msg":"connection limit exceeded"}]
L2 STREAM: Quick disconnect — failure #N, backoff Xs
```

**Cause:** Alpaca allows a limited number of simultaneous WebSocket connections per account. If two components open connections to the same stream, the limit is reached.

**Solution:** This problem was solved by consolidating all L2 connections in `streamer.rs`. If they reappear:

1. Check that there are no multiple instances of the application
2. Restart the application (kill → re-launch)
3. Wait ~60 seconds for Alpaca to release the old connections

### 25.8 "AUTOPILOT EXIT FAILED: NO PRICE DATA"

**Symptom:** The autopilot repeatedly tries to close a position but fails.

**Cause (resolved):** The internal cache `last_prices` did not have the price of the symbol if it was not actively streaming.

**Solution:** The autopilot now automatically copies `current_price` from the cached Alpaca position data before sending the exit command. If it still appears:

1. Check that the L2 stream is connected
2. Check that the symbol is valid on Alpaca

### 25.9 "422 Unprocessable Entity: invalid crypto time_in_force"

**Symptom:** Crypto orders (BTC/USD, ETH/USD) fail with error 422.

**Cause (resolved):** Alpaca requires `time_in_force: "gtc"` for crypto, not `"day"`.

**Solution:** Titan automatically detects crypto vs stocks and sets TIF correctly. This problem should not occur again.

### 25.10 The application runs old version after rebuild

**Symptom:** After changes in `index.html`, the application displays the old UI.

**Possible causes:**

1. **Tauri incremental build** — Do not re-embed the frontend if only HTML has changed
2. **WebKit cache** — The internal browser caches the old styles

**Solution:**

```bash
# 1. Sterge build cache-ul Tauri
rm -rf ${TITAN_HFT_ROOT:-.}/src-tauri/target/release/build/titan_hft_rust-*

# 2. Sterge WebKit cache
rm -rf ~/.cache/titan_hft_rust/WebKitCache

# 3. Rebuild
cd ${TITAN_HFT_ROOT:-.}/src-tauri && cargo build --release

# 4. Relanseaza aplicatia
```

### 25.11 Buttons/Tabs do not work (CSP)

**Symptom:** Clicking on a button (ex: HFT ENGINE tab) does nothing.

**Cause:** Tauri WebView blocks `onclick="..."` inline for reasons of Content Security Policy.

**Solution:** Use `data-` attributes and `addEventListener` instead of `onclick`:

```html
<!-- Corect: -->
<button data-symtab="hft">HFT ENGINE</button>

<!-- Gresit (blocat de CSP): -->
<button onclick="switchSymbolTab('hft')">HFT ENGINE</button>
```

JavaScript attaches event listeners to `DOMContentLoaded`.

### 25.12 Invisible text in inputs/selections

**Symptom:** You cannot see the text in the input boxes in the sidebar.

**Cause:** WebKit applies native styles (white background, black text) that override the application's dark theme.

**Solution:** Applied with `color-scheme: dark !important` on all input/select elements in the sidebar:

```css
#sidebar input, #sidebar select {
    background: #000 !important;
    color: #FFD700 !important;
    color-scheme: dark !important;
}
```

If it happens again after the rebuild, delete the WebKit cache (see 25.10).

### 25.13 SPREAD GUARD blocks all entries on an asset

**Symptom:** The log constantly shows `SPREAD GUARD: X bps — too wide, skipping entries` and the bot never enters.

**Cause:** The asset has a natural spread higher than the configured `max_spread_bps` limit. Micro-caps (SOC, penny stocks) often have spreads of 100-300 bps.

**Solution:** Increase `max_spread_bps` from HFT Engine tab. Recommendations: 50 bps for large-caps, 100-200 bps for small/micro-caps. Alternatively, choose assets with better liquidity (volume > 2M, spread < 50 bps).

### 25.14 WATCHDOG: Dead loops — ai_scanner or autopilot

**Symptom:** Red messages `WATCHDOG: Dead loops: ai_scanner` or `autopilot` in Activity Log.

**Cause ai_scanner (resolved):** Previously, `ai_scan_loop` sent heartbeat only every 30 minutes, exceeding the watchdog threshold of 120 seconds.

**ai_scanner solution:** The loop now sends a heartbeat every 60 seconds (under the 120s threshold), independent of the 30 minute scan interval.

**Autopilot Cause:** Autopilot may seem "dead" if SPREAD GUARD blocks constantly and the loop spends all the time in `continue` without reaching the effective execution. The heartbeat is sent at the beginning of each iteration (1.5s), so it shouldn't appear normally. If it appears, restart the application.

### 25.15 AI Scanner — the scan takes a long time

**Symptom:** After clicking SCAN NOW, the progress bar advances very slowly (especially with 200 candidates).

**Reason:** Each candidate requires ~9 Alpaca API requests (1 for 5m candles + 8 for sweep backtest on all timeframes). With 200 candidates = ~1800 applications.

**Solution:** Alpaca rate limit is ~200 req/min. The scan with 200 candidates can take 10-15 minutes. Use 50 candidates for fast results (~3 minutes).

---

## FINAL NOTE

Titan HFT is a complex system with multiple layers of security. The main recommendations:

1. **Always start in DRY RUN** until you are comfortable with the behavior
2. **Monitor the Activity Log** for anomalies
3. **Do not manually modify the thresholds** Circuit Breaker without understanding the implications
4. **Let the Genetic Optimizer** run for at least a few hours before trading live
5. **Check LOOP HEARTBEATS** periodically — all loops must be ALIVE
6. **Session Reports** from `$HOME/HFT_Bot/reports/` are the source of truth for performance
7. **SPREAD GUARD** messages are normal — it means that the filters work and protect capital
8. **After frontend changes**, delete WebKit cache and do a clean build (see section 25.10)
9. **Settings persist automatically** in `titan_settings.json` — dry_run, strategy_mode, favorites, trade size
10. **Strategy generates the signal, filters validate it** — see section 6 and 11 for details

---

## 26. CATALYST WATCHDOG — SENTIENT NEWS FEED

Advanced real-time news monitoring system designed to identify "Black Swan" or major momentums in the **Stocks** market before they are fully digested by the market.

### 26.1 How it works

- **Polling Loop:** Runs in the background (`sentiment.rs`) and checks the Alpaca News feed every 60 seconds.
- **OnceLock Lexicon:** The lexicon (~100 terms + regex) is compiled only once via `std::sync::OnceLock`, eliminating overhead every iteration.
- **Parallel Check:** `check_catalyst` processes symbols in parallel (`futures_util::join_all`), chunking to 5 symbols with 10s timeout.
- **Crypto Filter:** Completely ignores crypto news.
- **Sentiment Engine:** Advanced stock lexicon with negation "lookback" for **Confidence Score** (0.00 - 1.00).
- **Bearish Support:** Alerts both bullish (score ≥ 0.50) and bearish (score ≤ -0.50).
- **Per-Symbol Cooldown:** 5 minute cooldown per symbol against spam alert.

### 26.2 Severity Levels

| slag | Severity | Color |
|-------|----------|---------|
| ≥ 0.85 or ≤ -0.85 | **EXPLOSIVES** | Green / Intense red |
| ≥ 0.50 or ≤ -0.50 | **STRONG** | Green / Red |
| ≥ 0.35 | **MODERATE** | Yellow |
| < 0.35 | **MILD** | Gray |

### 26.3 Tab 7 — Catalyst Watchdog Dashboard

**Header:**
- Magenta title with badge `alerts` count
- Buttons in a separate row: 🔥 TEST BULLISH (magenta gradient) | 🔻 BEARISH TEST (red gradient) | CLEAR LOG
- Buttons have `white-space:nowrap` — do not truncate

**Stats Bar (5 responsive cards):**
- **Total Catalysts** — total alerts counter
- **Bullish** — positive alert counter (green)
- **Bearish** — negative alert counter (red)
- **Last Alert** — last symbol + time
- **Watchdog Status** — LIVE (green pulse) / NO_KEYS / DOWN

**Log Table (7 columns):**

| Column | Description |
|---------|-----------|
| TIME | Alert time (HH:MM AM/PM) |
| SYMBOL | Ticker (click to open on the chart) |
| HEADLINE | News title (clickable link) |
| SEVERITY | Colored badge (EXPLOSIVE/STRONG/MODERATE/MILD) |
| slag | Sentiment score (colored) |
| FEELING | BULLISH (green) / BEARISH (red) |
| act | INJECT button (add to HFT Targets) |

**Filters:** Symbol, Minimum Score, Sentiment (All/Bullish/Bearish)
**Max rows:** 100 (auto-eviction oldest)

### 26.4 Toast Notifications V3

- **Bullish toast:** Green/cyan, 2-tone sound
- **Bearish toast:** Red/magenta, deep bass sound
- **Stacking:** Counter badge for multiple alerts (+2, +3 etc.)
- **Auto-dismiss:** 8 seconds with progress bar
- **Buttons:** VIEW (open Tab 7), INJECT (add to HFT), X (close)

### 26.5 Backend Commands

| Command | Description |
|---------|-----------|
| `test_catalyst_alert` | Issue bullish mock alert (NVDA, +0.98) |
| `test_catalyst_bearish` | Issue bearish mock alert (TSLA, -0.92) |
| `check_catalyst` | Check news by token, return `CatalystCheckResult` |
| `get_news_feed` | Returns `SentimentSnapshot` with `label_from_score()` |

---

## 27. SIGNALS AND ARROWS ON CHART

The visual signaling system is strategy-specific — the arrows reflect the active strategy mode:

### 27.1 Oracle Mode — Green/Red Arrows (Threshold ±8.0)

- **BUY (green, ▲ below bar):** Oracle score crosses +8.0 from bottom to top.
- **SELL (red, ▼ above bar):** Oracle score crosses -8.0 from top to bottom.
- Recalibrated from ±15.0 (too high for current volatility) to ±8.0.

### 27.2 Nexus GPU Mod — Cyan/Pink Arrows (Momentum ±0.02)

- **GPU BUY (cyan `#00BFFF`, ▲ under bar):** GPU momentum crosses +0.02.
- **GPU SELL (pink `#FF6EC7`, ▼ above bar):** GPU momentum crosses -0.02.
- Momentum is calculated per candle from dot product weighted by returns (64/128/256 candle windows).

### 27.3 Fusion Mod — Gold/Orange Arrows (Adaptive Threshold)

- **FUSION BUY (gold `#FFD700`, ▲ under bar):** Fusion score crosses adaptive threshold.
- **FUSION SELL (orange `#FF4500`, ▼ above bar):** Fusion score crosses -threshold.
- **Fusion Formula:** `oracle_score × (1 - alpha) + gpu_momentum × 100 × alpha`
- 27.4 Changepoint Markers (All Modes)

### 27.4 Changepoint Markers (All Modes)

- **Orange dots (`#FF8800`, ● above bar, text "BREAK"):** Indicates regime breaks detected by BOCPD.
- **Trigger:** Probability changepoint > 0.7.
- Active in all 3 strategy modes.

### 27.5 Visibility Toggle

- The **"Signals"** checkbox in the chart legend controls the visibility of the arrows.
- Toggle state persists between symbol/timeframe changes.

### 27.6 Colors per Mode (Quick Reference)

| Mode | BUY | Sella | Threshold |
|-----|-----|------|-----------|
| **Oracle** | Green `#00FF41` | Red `#FF2B2B` | ±8.0 |
| **GPU Nexus** | Cyan `#00BFFF` | Pink `#FF6EC7` | ±0.02 momentum |
| **Fusion** | Gold `#FFD700` | Orange `#FF4500` | Adaptive (8.0×(1-α) + 2.0×α) |
| **Changepoint** | — | — | BOCPD prob > 0.7 |

---

## 28. TELEGRAM ALERTS — MOBILE NOTIFICATIONS

Real-time alert system on the mobile phone via Telegram Bot API.

### 28.1 Initial Setup (BotFather)

1. Open Telegram and search for **@BotFather**.
2. Send `/newbot` and follow the steps to create a new bot.
3. Copy **Bot Token** (format: `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`).
4. Send a message to your newly created bot (to initiate the conversation).
5. Access `https://api.telegram.org/bot<TOKEN>/getUpdates` in the browser.
6. Copy the **chat_id** from the JSON response (the number from `"chat":{"id":123456789}`).

### 28.2 Configuration in Titan

In the **HFT Engine (tab-4)** tab, the **TELEGRAM ALERTS** section:

- **Bot Token:** Enter the BotFather token.
- **Chat ID:** Enter your chat_id.
- **Enable:** Checkbox for activation/deactivation.
- **TEST:** Button to send a test message ("Titan HFT Connected!").

### 28.3 Types of Alerts

| Event | When it is sent | Information included |
|-----------|----------------|--------------------|
| **Trade Fill** | When executing an order | Symbol, side (BUY/SELL), quantity, price, PnL |
| **Circuit Breaker** | Upon activation of PANIC | The breaker level, the reason |
| **Catalyst** | When detecting a major news | Symbol, score, headline |
| **Daily Summary** | At daily reset (9:30 a.m. ET) | Total trades, win rate, PnL, profit factor |

### 28.4 Security

- Messages are sent via HTTPS to the Telegram API.
- The token is stored locally in `titan_settings.json`, it is not transmitted elsewhere.
- Alerts are sent only if `telegram_enabled = true` and token/chat_id are non-empty.

---

## 29. RISK CORRELATION MATRIX

Correlation analysis between simultaneously traded symbols in Multi-Symbol mode, to avoid redundant exposure.

### 29.1 What is Correlation

The Pearson correlation measures how similar two assets move:

- **+1.0** — It moves identically (double exposure to the same risk).
- **0.0** — No connection (perfect diversification).
- **-1.0** — Moves opposite (natural hedge).

### 29.2 How it Works in Titan

At each entry attempt in Multi-Symbol mode:

1. Titan downloads recent candles for the candidate symbol and already owned symbols.
2. Calculates percentage returns from close prices.
3. Apply `pearson_correlation` between the candidate and each existing position.
4. If the correlation exceeds `max_correlation` (default 0.70), the entry is **blocked**.

**Example:** If you own AAPL and try to enter MSFT (correlation ~0.85), Titan blocks the entry because both are Large-Cap Tech and move similarly.

### 29.3 Configuration

In the **HFT Engine (tab-4)** tab:

- **Slider "Max Corr":** Sets the maximum threshold of accepted correlation (0.30 - 0.95, default 0.70).
- **Correlation Window:** The number of candles used for the calculation (default 60).

### 29.4 Viewing in the UI

The **RISK CORRELATION MATRIX** section in tab-4 displays:

- **Matrix table:** Pairs of symbols with the calculated correlation (red = risk, green = safe).
- **REFRESH button:** Recalculates the matrix based on the current data.

---

## 30. PERFORMANCE PORTFOLIO & REBALANCE

Analysis and optimization of the portfolio based on the historical performance in the Trade Journal.

### 30.1 Metrics Per Symbol

From the log data, Titan calculates for each symbol:

| Metra | Description |
|---------|-----------|
| **Trades** | The total number of transactions |
| **Win Rate** | The percentage of profitable transactions |
| **Total PnL** | Cumulative profit/loss |
| **Avg PnL** | Average profit per transaction |
| **Performance Score** | Composite score: `win_rate * 40 + pnl_magnitude * 30 + trade_bonus * 30` |
| **Tag** | Classification: STAR / GOOD / AVERAGE / UNDERPERFORMER |

### 30.2 Risk Multiplier

Historical performance directly influences the size of future positions:

| Tag | Risk Multiplier | Effect |
|-----|----------------|-------|
| **STAR** | 1.5x | Higher positions on performing symbols |
| **GOOD** | 1.2x | Slightly higher positions |
| **AVERAGE** | 1.0x | Standard size |
| **UNDERPERFORMER** | 0.5x | Halved positions on weak symbols |

### 30.3 Rebalancing

The **REBALANCE** button in tab-4 analyzes the portfolio and suggests:

- **Remove:** Symbols with consistent losses (underperformer tag, PnL < 0, >3 trades).
- **Demote:** Symbols with below average but not disastrous performance.
- **Promote:** STAR symbols that are not yet in HFT targets.

### 30.4 Viewing in the UI

**PORTFOLIO PERFORMANCE** section from tab-4:

- **Table:** Symbol, Trades, Win Rate, PnL, Tag (colored), Risk Multiplier.
- **REFRESH button:** Recalculates metrics from the log.
- **REBALANCE button:** Displays suggested actions.

---

## 31. ATR DYNAMIC STOP-LOSS & PARTIAL PROFIT TAKING

Volatility adaptive exit system, replacing fixed stop-loss/take-profit with dynamic values ​​based on ATR.

### 31.1 What is ATR (Average True Range)

ATR measures the average volatility of an instrument over a given period. True Range = max(High-Low, |High-PrevClose|, |Low-PrevClose|). ATR = average True Range over N periods (default 14).

### 31.2 How Stop-Loss ATR Works

Instead of a fixed stop-loss (eg -2.5%), Titan calculates:

```
stop_loss_price = entry_price - ATR * atr_stop_multiplier    (pentru LONG)
stop_loss_price = entry_price + ATR * atr_stop_multiplier    (pentru SHORT)

take_profit_price = entry_price + ATR * atr_take_multiplier  (pentru LONG)
take_profit_price = entry_price - ATR * atr_take_multiplier  (pentru SHORT)
```

**Advantage:** On volatile markets, the stop is automatically widened (prevents premature stop-out). In calm markets, the stop is tightened (protects the profit).

### 31.3 Configuration

In the **HFT Engine (tab-4)** tab, the **ATR DYNAMIC STOPS** section:

| Parameter | Default | rank | Description |
|-----------|---------|-------|-----------|
| **Stop-Loss (x ATR)** | 2.0 | 0.5 - 5.0 | ATR multiplier for stop-loss |
| 3.0 | 3.0 | 1.0 - 10.0 | ATR multiplier for take-profit |
| **Partial Take %** | 50% | 25% - 75% | The percentage of the partially sold position |
| **Partial Profit Enable** | OFF | ON/OFF | Enables/disables partial profit |

### 31.4 Partial Profit Taking

When the profit reaches 50% of the take-profit target:

1. Titan automatically sells `partial_take_pct` (ex: 50%) of the position.
2. The rest of the position continues with active trailing stop.
3. "PARTIAL TAKE: balance X% of SYMBOL" is logged. in the Activity Log.
4. Telegram alert is sent (if enabled).

**Example:** Entry AAPL $180, TP = $186 (ATR * 3.0 = $6). At $183 (50% of $6 = $3 profit), sell 50% of the position. The rest remains with a trailing stop.

### 31.5 Trailing Stop with ATR

The trailing stop is calculated based on ATR:

```
trail_pct = ATR / entry_price * atr_stop_multiplier * 0.5
```

`trail_high` (LONG) or `trail_low` (SHORT) is updated at each tick. Exit if the price withdraws by > trail_pct from peak.

---

## 32. WEB DASHBOARD — LOCAL ACCESS

Web dashboard served by an embedded HTTP server (axum) directly from Titan, accessible on localhost.

### 32.1 Access

- **URL:** `http://127.0.0.1:7777` (localhost only — not exposed on LAN)
- **Compatibility:** Any browser (Chrome, Safari, Firefox).
- **Authentication:** All endpoints (including `/` and `/ws`) are protected with Bearer token.

### 32.2 What It Displays

- **Cards:** Equity, Daily PnL, Win Rate, Open positions, Autopilot, Strategy, Breaker, Trades.
- **Positions Table:** Symbol, side, quantity, entry price, current price, PnL (green/red colored with neon glow).
- **Trades table:** The last 10 transactions with timestamp and PnL.
- **WebSocket Live:** Status push every 2 seconds, trade/alert events, auto-reconnect (zero polling).
- **Design:** Cyber-HUD dark theme (Share Tech Mono, neon cyan/gold/green palette, scan-line animation), mobile-responsive.

### 32.3 API Endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-----------|
| `/api/health` | GET | Not | Health check: status, uptime, version |
| `/api/build-info` | GET | Not | Build info: version, git hash, build date, uptime (S04) |
| `get_state_health` | bulls | -- | State decomposition health: lock contention per sub-state (S04.5) |
| `/` | GET | Bearer | Complete HTML cyber-HUD dashboard |
| `/api/status` | GET | Bearer | Equity, autopilot, strategy, breaker, dry_run |
| `/api/positions` | GET | Bearer | List of positions with PnL |
| `/api/trades` | GET | Bearer | The last 20 trades |
| `/api/stats` | GET | Bearer | Win rate, PnL, profit factor |
| `/api/autopilot` | POST | Bearer | Toggle autopilot |
| `/ws` | GET | Bearer | WebSocket live push (status, trade, alert) |

### 32.4 Security

- **Token:** All endpoints (except `/api/health`) are protected with `Authorization: Bearer <token>`. The token is `web_dashboard_token` (automatically generated on first run, 64 hex chars).
- **Token Rotation:** The Tauri `rotate_dashboard_token` command regenerates the token instantly and persists on disk.
- **Access:** The server listens exclusively on `127.0.0.1` (localhost). Not accessible from LAN.
- **CORS:** Only allows `http://127.0.0.1:<port>` and `http://localhost:<port>` origins.
- **Rate Limit:** 60 sliding window requests/minute on all protected routes (including WS upgrade).
- **WS Rate Limit:** Maximum 10 client messages per tick (2s). Exceeding disconnects the client.
- **Auth Logging:** Authentication failures are logged; every 5 consecutive failures a brute-force alert is issued.

### 32.5 Configuration

In the tab **HFT Engine (tab-4)**, section **WEB DASHBOARD**:

- **Enable:** Toggle server activation/deactivation.
- **Port:** HTTP port (default 7777, range 1024-65535).
- **Token:** Displayed in UI, COPY URL button. Can be rotated with `rotate_dashboard_token`.

---

## 32.6 State Architecture (S04.5)

Starting with V4.0, `TitanState` is decomposed into 4 distinct sub-states, each with documented internal lock ordering:

| Sub-states | Module | Fields | Purpose |
|-----------|-------|---------|------|
| **TradingState** | `trading_state.rs` | 10 Mutex | Trail tracking, entry metadata, exit inflight, blacklist, cooldowns |
| **MarketDataState** | `market_state.rs` | 5 Mutex + ArcSwap | Prices, candle buffers, indicator cache, vol/bars cache, OFI |
| **RiskState** | `risk_state.rs` | 3 Mutex + atomics | Circuit breaker, session analytics, drawdown, flash crash |
| **ScanState** | `scan_state.rs` | 5 Mutex | AI scan pipeline: best-TF, results, progress, prefilter, cancel |

**Lock ordering global:** 40 slots in 9 groups, fully documented in `state.rs`. Any purchase of Mutex must respect the order.

**State Health Dots:** In the STATUS sidebar, 4 mini-dots (TRD/MKT/RSK/SCN) show the health of the sub-states in real time:
- Green: all Mutexes free
- Yellow: 1 Mutex contended (normal lock contention)
- Red: 2+ Mutexes contended (potential bottleneck)

Hover on each dot displays details (locks free / contended, session generation).

**Tauri command:** `get_state_health` returns per sub-state: name, fields, locks_ok, locks_contended, plus session_generation and uptime.

---

## 33. STATE PERSISTENCE — CRASH RECOVERY

Complete snapshot/restore system that ensures that the bot survives crashes and restarts without losing context.

### 33.1 What is Saved

Every **30 seconds**, Titan writes a complete snapshot to `$HOME/HFT_Bot/titan_state_snapshot.json`:

| Date | Description |
|------|-----------|
| **trail_highs / trail_lows** | Trailing stop tracking per symbol |
| **partial_taken** | Partial profit flags per symbol |
| **last_prices** | Cache prices per symbol |
| **symbol_best_tf** | Optimum timeframe per symbol from AI scan |
| **nexus_fusion_alpha** | Current fusion alpha value |
| **session_analytics** | Equity, trades, PnL, drawdown, trade_pnls complete |
| **dd_velocity_ring** | Ring buffer drawdown velocity (timestamp, dd_pct) — restored on restart (S03) |
| **session_generation** | Session generation counter — monotonic, stale order invalidation (S03) |
| **last_scan_results** | All AI scan table (all rows) |
| **scan_meta** | Was there a scan in progress? How many processed? Total required? |

### 33.2 Auto-Restore at Startup

At startup, Titan searches for the snapshot and restores it automatically:

1. Read `$HOME/HFT_Bot/titan_state_snapshot.json`
2. Check age — ignore snapshots older than **12 hours** (data would be too stale)
3. Populates trail stops, prices, analytics, scan results in memory
4. Issues `ai-scan-update` to the frontend — the scan table appears **instant** without re-scanning
5. Displays in the Activity Log: "SNAPSHOT RESTORE: X scan results loaded, Y trail stops, Z cached prices"

### 33.3 Auto-Resume Scan (Continued from Point of Interruption)

If the snapshot indicates that a scan was in progress when the application stopped:

1. Partial results appear **instant** in the UI (ex: 158 symbols already processed)
2. Wait 5 seconds (for the APIs to connect)
3. Continue the scan **where it left off** (ex: from 158/200), processing only the remaining symbols
4. The progress bar starts from the breakpoint (eg: 158/200)
5. At the end, it goes through the old + new results, re-sorts, and displays the complete list
6. Displays: "AUTO-RESUME: Continuing scan from 158/200 (42 remaining)"

**How it works technically:** The pre-filter queue (the complete list of symbols to be processed) is saved in the snapshot. In summary, a HashSet is built from the already processed symbols and only the remaining ones are processed. The results are incremental (saved along the way in states for safety).

### 33.4 Graceful Shutdown

When pressing the **SHUTDOWN** button, Titan saves a last snapshot before shutdown, ensuring that absolutely all the state is on the disk.

### 33.5 Atomic Writing

The snapshot is written atomically (write to `.tmp` file, then rename) to prevent corruption during a crash during writing. The Journal (`TradeJournal::save()`) uses the same pattern (S72).

### 33.5b Delta Compression (S72)

`save_state_snapshot` compares the SHA-256 hash of the new snapshot with the previous hash (stored in `.sha256` sidecar file). If they are identical, writing to disk is omitted. Significantly reduces I/O during idle periods.

### 33.6 What NOT to Save (Intentional)

| Date | Reason |
|------|-------|
| `bars_cache` | Stale candles, re-fetch on request |
| `gpu_score_cache` | Recalculated per tick |
| `ofi_state` | Live L2 data, meaningless after restart |
| `nexus_consensus` | Recalculated by GPU agents |
| `news_cache` | Re-fetch from background loop |
| `circuit_breaker` | Safer to reset after crash |

---

## 34. SMART SCORE V4 — INSTITUTIONAL RANKING

The AI ​​Scanner asset ranking formula, optimized to favor quality and consistency.

### 34.1 Basic Formula

```
base = 0.35 * Win% + 0.22 * Sharpe_norm + 0.15 * Trades_norm
     + 0.10 * PF_norm + 0.10 * Expect_norm + 0.08 * Fusion_norm
```

### 34.2 Multiplier Win% (Most Important)

| Win Rate | Multiplier | Effect |
|----------|-----------|-------|
| < 30% | 0.25x | Aggressive Removal |
| 30-38% | 0.45x | Severe Penalty |
| 38-45% | 0.60x | Penalty moderate |
| 45-50% | 0.82x | Sub-par but acceptable |
| 50-60% | 1.00x | Standard |
| 60-70% | 1.12x | Bonus |
| 70%+ | 1.20x | Maximum Bonus |

### 34.3 Other Multipliers

| Factor | Condition | Multiplier |
|--------|----------|-----------|
| **Trades** | < 5 | 0.35x |
| **Trades** | 5-8 | 0.60x |
| **Trades** | 8-12 | 0.85x |
| **Trades** | 20+ | 1.05x (modest) |
| **Sharpe** | < 0.3 | 0.45x |
| **Sharpe** | 0.3-0.7 | 0.70x |
| **Sharpe** | 0.7-1.0 | 0.88x |
| **Sharpe** | 2.0+ | 1.12x |
| **Profit Factor** | < 0.8 | 0.55x |
| **Expectancy** | < 0 | 0.40x |

### 34.4 Consistency Bonus (NEW in V4)

Symbols that excel on **all** axes receive an additional bonus:

| Condition | Bonus |
|----------|-------|
| Win >= 50% AND Sharpe >= 1.0 AND Trades >= 10 | 1.15x |
| Win >= 55% AND Sharpe >= 1.0 | 1.08x |

### 34.5 Hard Kill

If any of the following conditions are true, the SMART score is reduced to `fusion_normalized * 0.10`:

- Trades < 3
- Sharpe <= 0
- Win Rate < 15%
- Expectancy < -0.01

### 34.6 Result

The formula produces a ranking where the top 20 SMART PICKS are predominantly symbols with Win% > 50%, Sharpe > 1.0, and enough trades for statistical significance. Symbols with Win% below 40% can no longer compensate by a large number of trades.

---

## 35. AUTO ENTRY — AUTOMATIC ENTRY CONTROL

### 35.1 What It Is

The **AUTO ENTRY** toggle in the sidebar (TRADING CONTROLS section) controls whether the bot opens new positions automatically.

### 35.2 AUTOPILOT vs AUTO ENTRY distinction

| Toggle | What it does | Default |
|--------|---------|---------|
| **AUTOPILOT** | Manages exits: stop-loss, take-profit, trailing stop on existing positions | ON |
| **AUTO ENTRY** | Open new positions automatically based on FUSION/Nexus signals | OFF |

**Important:** Both must be ON for fully automatic trading. AUTOPILOT alone = the bot closes positions but does not open new ones.

### 35.3 Safety Gates for Entry

When AUTO ENTRY is ON, the bot checks the following before opening a position:

1. Circuit Breaker = NORMAL
2. Spread Guard < limit
3. Cooldown elapsed
4. Max positions not reached
5. Strategy score >= threshold
6. OFI confirms direction
7. Nexus consensus >= threshold
8. Correlation check (if Multi-Symbol)

### 35.4 Recommendation

Activate AUTO ENTRY **only after** what:
- You checked that FUSION signals appear correctly on the chart
- You ran at least one scan and checked SMART PICKS
- You configured risk_per_trade and max_open_positions
- You tested in DRY RUN

---

## 36. SIGNAL ALIGNMENT — FUSION-AWARE AUTOPILOT & CAUSAL CHART

### 36.1 Previous Problem

Two distinct problems affected the quality of the signals:

1. **Autopilot ignores GPU/Fusion:** Function `compute_strategy_score` generates the LONG/SHORT/NEUTRAL signal using ONLY Oracle with threshold +/-30. In FUSION mode, the Nexus GPU was completely ignored. On symbols with low Oracle score (~0.03), the signal was always NEUTRAL — the bot never enters.

2. **Chart signals with hindsight:** `adaptive_thresholds` calculates 80/20 percentiles on ALL scores in the visible window (hindsight). A SELL signal at $17.60 appeared only because the algorithm "saw" already the fall from $18.80 — hindsight bias.

### 36.2 Solution: compute_strategy_score_with_mode

Function `compute_strategy_score` was refactored into `compute_strategy_score_with_mode(candles, mode, alpha, gpu_momentum)`:

| Mode | Source Signal | Threshold |
|-----|-------------|-----------|
| **ORACLE** | Oracle score | +/-8.0 (reduced from +/-15), confidence > 0.10 |
| **GPU** | GPU momentum | +/-0.003 (reduced from +/-0.005) |
| **FUSION** | `oracle_norm * (1-alpha) + gpu_signal * alpha` | Adaptive, max 6.0 |

36.3 Solution: Causal Rolling Lookback (Chart)

### 36.3 Solution: Causal Rolling Lookback (Chart)

Replaced `adaptive_thresholds` (retrospective) with **rolling lookback** on last 40 bars:

- At each bar `i`, the thresholds are calculated from `scores[max(0, i-40)..i]`
- Only **previous** bars — no information from the future
- The signals are honest and reflect the real conditions at the time of generation
- Completely removed hindsight bias from chart markers

### 36.4 Impact

- The autopilot now generates correct signals in FUSION mode (most used)
- The chart shows realistic signals, not "perfect hindsight"
- The signals from the chart and from the autopilot are now aligned (same logic)

---

## 37. AUTO-RESUME SCAN — CONTINUED FROM INTERRUPTION POINT

### 37.1 The problem

When restarting after an interrupted scan (eg: 158/200), auto-resume started from 0/200. The pre-filter pipeline (fetch assets -> snapshots -> quant_prefilter) is non-deterministic (depends on live data), so the same list of symbols on restart could not be guaranteed.

### 37.2 The solution

Saving the pre-filter queue (full list of symbols to be processed) in StateSnapshot:

1. **`PreFilterCandidate`** receives `Serialize, Deserialize` derives
2. **`scan_prefilter_queue`** added to `TitanState` and `StateSnapshot`
3. The queue is saved after `quant_prefilter` and is deleted when completed
4. **`auto_resume_scan`** completely rewritten:
   - Accept `saved_queue` + `partial_results`
   - Skip already processed symbols (HashSet lookup)
   - Progress from `done_count` (ex: 158/200)
   - It goes incrementally along the way
   - Final: merge + re-sort + re-rank + issue

### 37.3 Behavior on Restart

| Step | what is happening |
|-----|---------------|
| 1 | The partial results (158) appear instantly in the UI |
| 2 | Log: "AUTO-RESUME: Continuing scan from 158/200 (42 remaining)" |
| 3 | Progress bar starts from 158/200 |
| 4 | Process only the remaining 42 symbols |
| 5 | The complete list of 200 ranked symbols appears in the UI |

### 37.4 Safety

- New results are saved incrementally in states (the periodic snapshot captures them)
- If the application crashes again during the resume, at the next restart it will continue where it left off (eg: 178/200)
- `#[serde(default)]` on `scan_prefilter_queue` ensures backward compatibility with old snapshots

---

## 38. MULTI-SYMBOL ENTRY FIX & DEL ALL

### 38.1 Multi-Symbol Entry Fix

**Problem:** With 21 symbols in HFT Engine and Multi-Symbol activated, no open position in 4 hours.

**Identified causes:**

1. `max_open_positions = 1` — only 1 simultaneous position allowed
2. OFI filter calculated on `active_symbol` but applied to ALL symbols
3. Nexus consensus filter — same, only on active_symbol but applied globally
4. **Applied solutions:**

**Applied solutions:**

- `max_open_positions` increased to 5
- OFI filter: in multi-symbol mode, applies ONLY to `active_symbol`; other symbols pass without OFI gate (OFI data is irrelevant for them)
- Nexus consensus filter: same, only on `active_symbol`
- Diagnostic logging: `ENTRY BLOCKED: auto_entry=OFF / breaker blocking / spread too wide` (0.8% sampling rate)

### 38.2 DEL ALL Button

Quick button to delete all symbols from the HFT Engine list:

- **Location:** In the header of the HFT targets list, next to the toggle arrow
- **Style:** Cyberpunk Red on Dark Background (`#1a0800`)
- **Behavior:** Delete only IDLE symbols — active positions are **protected** automatically
- **Confirm dialog:** "Delete all IDLE targets? Active positions will be kept."
- **Backend:** `clear_all_hft_targets` — `hft_targets.retain(|s| active_set.contains(s))`

---

## 39. ENTRY GATE SCORING PIPELINE — DEBOTTLENECKING

### 39.1 The Problem (March 14, 2026)

The entry gate did not allow any entry. The analysis of 42 logs (sampled at 1%, representing ~4200 real evaluations) showed score=50.0 and signal=NEUTRAL on all symbols, in a window of 8 minutes (06:23-06:31). The score was stuck at the neutral baseline of 50.0 due to a chain of 5 multiplicative bottlenecks.

### 39.2 The 5 Bottlenecks (Before the Fix)

The scoring pipeline had the following steps, each reducing the signal:

```
Oracle Layers (Kalman, Wavelet, Hurst, VPIN, Entropy)
    → raw_signal (weighted sum, range [-1, +1])
    → final_signal = raw_signal * regime_boost * meta_confidence    ← BOTTLENECK 1+2
    → oracle_score = final_signal * 100
    → confidence = meta_confidence * (1 - noise_ratio)              ← BOTTLENECK 3
    → entry score = 50 + oracle_val * 0.5 * confidence              ← BOTTLENECK 4
    → signal = LONG/SHORT/NEUTRAL (threshold check)                 ← BOTTLENECK 5
    → entry_side = None daca NEUTRAL (blocat)
```

### 39.3 Applied Fixes

**Fix 1 — meta_confidence: product → geometric mean**

| Before | After |
|---------|------|
| `predictability * clarity * stability` | `(predictability * clarity * stability).cbrt()` |
| Example: 0.4 * 0.5 * 0.3 = **0.06** | Example: (0.4 * 0.5 * 0.3)^(1/3) = **0.39** |

The geometric mean keeps the intent (all 3 must be reasonable) but does not collapse to near-zero.

**Fix 2 — noise_ratio: penalty halved with floor**

| Before | After |
|---------|------|
| `meta_conf * (1 - noise_ratio).max(0)` | `meta_conf * (1 - noise_ratio * 0.5).max(0.25)` |
| noise_ratio=0.7 → factor 0.3 | noise_ratio=0.7 → factor 0.65 |

Floor of 0.25 prevents the total death of the signal in high noise conditions (pre-market).

**Fix 3 — score formula: removed redundant multiplier 0.5**

| Mode | Before | After |
|-----|---------|------|
| MathOracle | `50 + oracle_val * 0.5 * oracle_conf` | `50 + oracle_val * oracle_conf` |
| GpuNexus | `50 + gpu_scaled * 0.5` | `50 + gpu_scaled` |
| Fusion | `50 + fusion_val * 0.5 * fusion_conf` | `50 + fusion_val * fusion_conf` |

**Fix 4 — reduced signal thresholds**

| Mode | Before | After |
|-----|---------|------|
| MathOracle | oracle_val > 8.0, conf > 0.10 | oracle_val > 8.0, conf > 0.10 |
| GpuNexus | gpu_momentum > 0.005 | gpu_momentum > 0.003 |
| Fusion | headless fusion_thresh | fusion_thresh with .min(6.0) |

**Fix 5 — correct log message**

| Situation | Before | After |
|----------|---------|------|
| NEUTRAL signal | `score 50.0 < threshold 40.0` (misleading) | `signal NEUTRAL (score=50.0, no directional conviction)` |
| Score below threshold | Same as above | `score X < threshold Y (signal=LONG)` |

### 39.4 Corrected Flow

```
Oracle Layers → raw_signal [-1, +1]
    → meta_confidence = (pred * clarity * stab).cbrt()        ← geometric mean
    → final_signal = raw_signal * regime_boost * meta_conf
    → oracle_score = final_signal * 100
    → confidence = meta_conf * (1 - noise * 0.5).max(0.25)   ← halved + floor
    → entry score = 50 + oracle_val * confidence              ← no 0.5x
    → signal = LONG if oracle_val > 8.0 && conf > 0.10       ← lower thresholds
    → entry_side = Some("buy") daca LONG && score >= threshold
```

### 39.5 Numerical Impact

With typical values ​​(predictability=0.4, clarity=0.5, stability=0.3, noise_ratio=0.7, raw_signal=0.5):

| Metra | Before | After |
|---------|---------|------|
| meta_confidence | 0.06 | 0.39 |
| confidence (final) | 0.018 | 0.25 |
| oracle_score | 2.4 | 15.6 |
| entry score | 50.02 | 53.9 |
| signal | NEUTRAL (locked) | LONG (pass) |

### 39.6 Modified Files

| File | Location | CHANGE |
|--------|---------|-----------|
| `titan_oracle.rs` | line 1171 | meta_confidence: `.cbrt()` |
| `titan_oracle.rs` | line 1202 | confidences: `* 0.5`, `.max(0.25)` |
| `titan_oracle.rs` | line 1431 | series path: `.cbrt()` (consistent with fuse_oracle) |
| `main.rs` | lines 2690-2716 | formula score without 0.5, reduced thresholds |
| `main.rs` | lines 4905-4916 | log message differentiated NEUTRAL vs score<threshold |

---

## 40. ORACLE HISTORY FIXES (S05)

### 40.1 What was fixed

Five critical bugs in `calculate_history()` (used for chart overlay, backtest, SmartTrend) were fixed:

| Bug | Effect before | Fix |
|-----|---------------|-----|
| **Entropy double-normalization** | clarity ~0.88-1.0 constant (wrong) | clarity = `1 - normalized_entropy` (correct [0,1]) |
| **noise_ratio hardcoded 0.3** | Identical confidence regardless of noise | Uses real noise_ratio from wavelet decomposition |
| **VPIN buy_pressure = VPIN score** | buy_pressure was toxicity, not BVC fraction | BVC buy fraction: `(close-low)/(high-low)` per-bara |
| **Changepoint threshold 0.5 vs 0.3** | More permissive history with stability | Both paths use 0.3 (shared constant) |
| **Backtest exit exact match** | Prefixed exits ("O EXIT") not detected | `contains("EXIT")` instead of `== "EXIT"` |

### 40.2 Impact

- **History oracle scores** are now consistent with live scores
- **Backtest** correctly detects all exit markers (including those from SignalStrategy with O/G/F prefix)
- **SmartTrend** (future) will receive correct data from `OracleHistory`

### 40.3 New series available

`OracleHistory` now contains 5 additional series:

| Series | Source | Utility |
|-------|-------|-----------|
| `kalman_gain_series` | Kalman K[0] per-bar | How much the filter adjusts to new data |
| `price_surprise_series` | innovation/predicted | Kalman unexpected price movements |
| `run_length_series` | BOCPD MAP run length | How many bars since the last changepoint |
| `dominant_cycle_series` | Wavelet max-energy level | Dominant cycle (2^n bars) |
| `noise_ratio_series` | Wavelet detail/total energy | Noise/signal ratio |

---

## FINAL NOTE

Titan HFT is a complex system with multiple security layers. The main recommendations:

1. **Always start in DRY RUN** until you are comfortable with the behavior
2. **Monitor the Activity Log** for anomalies
3. **Do not manually modify the thresholds** Circuit Breaker without understanding the implications
4. **Let the Genetic Optimizer** run for at least a few hours before trading live
5. **Check LOOP HEARTBEATS** periodically — all loops must be ALIVE
6. **Session Reports** from `$HOME/HFT_Bot/reports/` are the source of truth for performance
7. **SPREAD GUARD** messages are normal — it means that the filters work and protect capital
8. **After frontend changes**, delete WebKit cache and do a clean build (see section 25.10)
9. **Settings persist automatically** in `titan_settings.json` — dry_run, strategy_mode, favorites, trade size
10. **Strategy generates the signal, filters validate it** — see section 6 and 11 for details
11. **Telegram Alerts** — configure BotFather and activate from tab-4 for alerts on mobile
12. **Correlation Matrix** — adjust Max Corr from tab-4 for optimal diversification in Multi-Symbol
13. **ATR Dynamic Stops** — leave the default multipliers (2.0x SL, 3.0x TP) until you understand the behavior on your assets
14. **State Persistence** — Upon crash/restart, everything is automatically restored from the snapshot (trail stops, analytics, scan results)
15. **AUTO ENTRY** — It must be activated explicitly from the sidebar for the bot to open new positions
16. **Signal Alignment** — Signals from the chart and autopilot are now aligned and FUSION-aware (Chap. 36)
17. **Auto-Resume Scan** — Upon restart, the scan continues from where it left off, not from 0 (Chap. 37)
18. **DEL ALL** — Deletes only IDLE symbols from HFT Engine; active positions are protected (Ch. 38)
19. **Entry Gate Scoring** — The scoring pipeline has been unlocked: meta_confidence with cbrt, noise floor, reduced thresholds (Ch. 39)
20. **Scalping Mode** — Enable scalping per strategy for more frequent signals on 1m-5m (Ch. 40)
21. **WARRIOR PRO** — Ross Cameron criteria filter strictly: RVOL>=5x, change>=10%, $1-$20, with catalyst check (Chap. 43)
22. **Position Sizing Safety** — Max 25% equity per trade, 1000 share cap, duplicate order prevention (Chap. 44)

---

## 40. SCALPING MODE — TOGGLES PER STRATEGY

### 40.1 What It Is

Each of the 3 strategies (Oracle, GPU Nexus, Fusion) can be independently switched to scalping mode. Scalping mode generates more frequent signals, optimized for 1m and 5m timeframes with holding times from seconds to 30 minutes.

### 40.2 Toggles in the UI

In the NEXUS GPU panel in the interface:

| Toggle | Setting | Default |
|--------|---------|---------|
| **SCALP ORACLE** | `scalping_oracle` | ON |
| **SCALP GPU** | `scalping_gpu` | ON |
| **SCALP FUSION** | `scalping_fusion` | ON |

### 40.3 Differences Scalping ON vs OFF

| Parameter | Scalping ON | Scalping OFF |
|-----------|------------|-------------|
| Lookback | 15 bars | 30 bars |
| Cooldown | 3 bars | 8 bars |
| Score threshold | 0.4 | 0.55 |
| Confidence gate | 0.08 | 0.15 |
| Exit SL multiplier | 0.70x | 1.0x |
| Exit TP multiplier | 0.60x | 1.0x |
| Exit Trail multiplier | 0.50x | 1.0x |

### 40.4 Impact on Chart

When scalping is active, the chart shows significantly more BUY/SELL/EXIT markers. When it is disabled, the signals are rarer and more conservative.

---

## 41. REGIME-ADAPTIVE TRADING SIGNALS

### 41.1 What It Is

Trading signals automatically adapt to the market regime detected by Oracle (Trending, MeanReverting, Chaotic, Unknown).

### 41.2 Thresholds per Regime

| Regime | Score Threshold | Confidence Min | Behavior |
|-------|----------------|---------------|-------------|
| **Trending** | 0.35 (scalp) / 0.50 (swing) | 0.06 / 0.12 | More sensitive signals, follow the trend |
| **MeanReverting** | 0.50 / 0.60 | 0.10 / 0.18 | Moderate thresholds, stricter confirmation |
| **Chaotic** | 0.65 / 0.75 | 0.15 / 0.25 | Very selective, avoid noise |
| **Unknown** | 0.45 / 0.55 | 0.08 / 0.15 | Default balanced |

### 41.3 Explicit Exit Signals

Special exit markers appear on the chart:

| Marker | Color | Condition |
|--------|---------|----------|
| **TRAIL** | Yellow #FFD700 | Trailing stop reached |
| **EXIT** | White #FFFFFF | SL/TP or opposite signal |
| **CHAOS** | Orange #FF6600 | Chaotic regime with loss |

### 41.4 Multi-Timeframe Confirmation

Signals are validated against Oracle direction on higher timeframe. A BUY on 1m is stronger if Oracle on 5m is also bullish.

---

## 42. AUTOPILOT ALIGNMENT — CHECK_ENTRY + ORACLE EXITS

### 42.2 Solution: check_entry()

The optimizations on the chart (adaptive regimes, scalping, VPIN gate) did not affect the live autopilot. The chart and the autopilot had different logics.

### 42.2 Solution: check_entry()

VPIN toxicity gate

- VPIN toxicity gate
- RVOL institutional volume check
- Regime-adaptive entry thresholds
- Scalping-specific parameters

### 42.3 Additional Exits

| exit | Condition | Description |
|------|----------|-----------|
| **Oracle Flip** | Oracle score < -10 (long) sau > 10 (short), confidence > 0.20 | Oracle reversed itself strongly |
| **Chaos Exit** | Chaotic regime + negative PnL + confidence > 0.25 | Do not maintain position in chaos |
| **Scalping Exit** | Active scalping | SL * 0.70, TP * 0.60, Trail * 0.50 |

---

## 43. WARRIOR PRO — ROSS CAMERON SCANNER

### 43.1 What It Is

WARRIOR PRO is a scanning mode based on Ross Cameron (Warrior Trading) criteria, optimized for day trading on low-float stocks.

### 43.2 Pre-filter criteria (warrior_prefilter)

| Criterion | Value | Explanation |
|----------|---------|-----------|
| **Price** | $1 - $20 | Low-price stocks with high movement potential |
| **Change%** | >= 10% | Only stocks in significant movement |
| **RVOL** | >= 5x | Relatively extraordinary volume compared to the previous day |
| **Volume** | > 1M | Minimum liquidity |

### 43.3 Catalyst Check

When WARRIOR PRO is active, each candidate is checked for recent news via the Alpaca News API:

| condition | Effect on Score |
|-------|---------------|
| **With catalyst (news detected)** | +25% bonus to ai_score and smart_score |
| **No catalyst** | -40% penalty |

Additional bonuses:
- RVOL >= 10x: +15%
- Change >= 20%: +10%
- Price <= $10: +10%

### 43.4 Activation

Two activation modes (synchronized):
1. **Toggle `WARRIOR PRO`** in AI Scanner panel (tab-6) — red button
2. **Toggle `Warrior`** in the sidebar Risk & Params

Both control the same `warrior_mode` in the backend.

### 43.5 Badge Catalyst

In the AI ​​Scanner table, symbols with recently detected news have a newspaper icon next to their name.

### 43.6 Warrior Scanner Sidebar

In Market Discovery (sidebar), the WARRIOR scanner now has:
- Ross Cameron Criteria (price $1-$20, change >= 10%, RVOL >= 5x)
- Column header: SYMBOL | PRICE | CHG% | RVOL | VOL
- Color coding: green for CHG% >= 10% and RVOL >= 5x

---

## 44. POSITION SIZING SAFETY — HARDCAPS & DUPLICATE PREVENTION

### 44.1 The problem

The autopilot places repeated orders of 8000+ shares on the same symbol (95% of equity per trade), without checking pending orders or market hours.

### 44.2 Solutions

| Fixed | Details |
|-----|---------|
| **Notional Max** | `equity * 0.25` (was `equity * 0.95`) |
| **Share head** | Maximum 1000 shares per order |
| **Pending order check** | `has_pending_orders_for_symbol()` — query Alpaca `/v2/orders?status=open` |
| **Market hours check** | `is_market_open()` — query Alpaca `/v2/clock` |
| **Entry cooldown** | 60 seconds (was 30) |

### 44.3 Conduct

- If a symbol already has pending orders → it is no longer placed
- If the market is closed and `enable_extended` is OFF → skip
- If the calculated qty > 1000 → is limited to 1000
- If notional > 25% equity → is reduced proportionally

---

## 45. SCALPING POSITION MARKER

### 45.1 What It Is

Open positions in scalping mode have a distinct visual marker in the positions table.

### **Yellow "SCALP" badge** — next to the position symbol

- **Yellow "SCALP" badge** — next to the position symbol
- **Regime badge** — ex: "Trending_LONG", "MeanReverting_SHORT" — next to timeframe

### 45.3 Technical Implementation

- `entry_tags: Mutex<HashMap<String, String>>` in `TitanState`
- At autopilot entry: `SCALP|Trending_LONG` or `MeanReverting_SHORT`
- At close: the tag is automatically deleted
- Frontend parses the tag and displays colored badges

---

## 46. ​​AI SCANNER ENHANCEMENTS

### 46.1 Scan Cancellation

The **CANCEL** button in progress on the AI ​​Scanner bar allows stopping a scan in progress. Backend: `CancellationToken` checked on each candidate.

### 46.2 Shared score_candidate()

The `score_candidate()` function centralizes the scoring logic (Oracle + GPU Nexus + Backtest Sweep + AI Score + Smart Score + Catalyst Check). Used by `run_ai_scan`, `ai_scan_loop`, and `auto_resume_scan`.

### 46.3 HFT Targets Cleanup

On auto-inject, symbols with `smart_score < 10.0` are automatically removed from `hft_targets`. Max cap: 20 symbols.

### 46.4 Timezone Fixed

Market hours calculated correctly with DST: EDT (UTC-4) March-October, EST (UTC-5) November-February.

### 46.5 Rate Limiting

500ms delay between batches of snapshots (1000 symbols/batch) to respect Alpaca rate limits.

---

## 47. SMART PICKS 3-Pillar Scoring

### 47.1 Formula with 3 Pillars

The `smart_score` score determines the SMART PICKS ranking and the auto-inject in HFT targets. The previous version relied almost exclusively on backtest metrics. The new formula combines 3 pillars:

| pylon | Weight | What measure |
|-------|---------|------------|
| **BACKTEST** | 50% | Historical performance: Win%, Sharpe, number of trades, Profit Factor, Expectancy |
| **MOMENTUM** | 35% | Current Strength: CHG% (price change today), RVOL (relative volume), MarketRegime (Oracle) |
| **QUALITY** | 15% | Intrinsic quality: liquidity (float), presence of catalyst, Oracle-price directional alignment |

**BACKTEST PILLAR (50%):**
- Base: `0.30 * Win% + 0.25 * Sharpe_norm + 0.20 * Trades_norm + 0.15 * PF_norm + 0.10 * Expect_norm`
- Penalty multipliers: Win% < 30% = 0.25x, Trades < 5 = 0.35x, Sharpe < 0.3 = 0.45x, PF < 0.8 = 0.55x, Expectancy < 0 = 0.40x
- Consistency bonus: Win% >= 50% + Sharpe >= 1.0 + Trades >= 10 = 1.15x
- If the data is insufficient (trades < 3, sharpe <= 0, win_rate < 15%, expectancy < -0.01), the pillar receives only 10% of the Fusion score

**PILLAR MOMENTUM (35%):**
- CHG% score (40%): >10% = 100, >5% = 80, >2% = 60, >0% = 40, >-2% = 20, >-5% = 10, otherwise = 0
- RVOL score (30%): >5x = 100, >3x = 90, >2x = 75, >1x = 50, otherwise = 20
- Regime score (30%): Trending = 100, MeanReverting = 60, Unknown = 40, Chaotic = 20

**QUALITY PILLAR (15%):**
- Liquidity: HIGH = 1.0x, MED = 0.92x, LOW + price < $3 = 0.50x, LOW otherwise = 0.75x
- Catalyst: present catalyst = 1.20x bonus (applies to ALL modes, not just Warrior)
- Directional alignment: Oracle and price in the same direction = 1.10x, opposite directions = 0.75x

### 47.2 Quality Filters

Before being displayed in SMART PICKS, candidates must pass the filters:

| Filter | Value | Scope |
|--------|---------|------|
| `bt_trades` | >= 5 | Sufficient statistical data |
| `bt_sharpe` | > 0.0 | Positive Risk Adjusted Return |
| `bt_win_rate` | > 20% | Minimum viability |
| `smart_score` | > 25 | Global quality (was > 20) |
| `change_percent` | > -5% | Excludes stocks in free fall |
| `price` | >= $1.50 | Excludes extreme penny stocks |

### 47.3 New Columns in the SMART PICKS Table

The SMART PICKS table now displays 3 new columns with color coding:

- **CHG%** — The percentage change of the price compared to the opening. Deep green (>5%), light green (>0%), gray (0 to -2%), red (<-2%).
- **RVOL** — Volume relative to the daily average. Green (>= 5x), yellow (>= 2x), gray (< 2x). Format: `2.3x`.
- **REGIME** — The market regime detected by Oracle. Colored badge: TREND (green), MR (blue, Mean Reverting), CHAOS (red), UNK (grey, Unknown).
- **Catalyst Badge** — If the symbol has a catalyst detected (news), a news icon appears next to the symbol.

### 47.4 Catalyst Detection in All Modes

Previously, the catalyst check (`has_catalyst`) only applies when Warrior Pro mode was active. Now:

- Catalyst check is executed in ALL scan modes (Standard)
- The 1.20x bonus in Quality Pillar applies universally
- Log: `[CATALYST] {symbol} has active catalyst (score={score})` appears regardless of the active mode
- Impact: symbols with news of interest (FDA approval, earnings beat, contract win) are promoted in ranking

---

## 48. MARKET DISCOVERY — Advanced Parameters and Auto-Scan

### 48.1 ScanParams — Configurable Parameters

The market scanner now uses the `ScanParams` structure from `scanner.rs`, with all configurable fields in the UI:

| Parameter | Default | Description |
|-----------|---------|-----------|
| `price_min` | $1.00 | Minimum price per share |
| `price_max` | $20.00 | Maximum price per share action |
| `volume_min` | 500,000 | Minimum daily volume |
| `change_min` | 5.0% | Minimum percentage change |
| `rvol_min` | 2.0x | Minimum relative volume (compared to average) |
| `gap_min` | 3.0% | Minimum gap to previous close |
| `require_catalyst` | false | Mandatory catalytic filter (recent news) |
| `max_candidates` | 100 | Maximum number of candidates processed |
| `scan_interval_minutes` | 30 | Auto-scan interval (5-60 minutes) |

### 48.2 Configuration from UI

In the **Market Discovery** section of sidebar:
- **RVOL Min** — slider or numerical input for the minimum relative volume
- **Gap Min** — slider or numerical input for the minimum gap
- **Require Catalyst** — toggle ON/OFF, when active filters only symbols with recent news
- **Max Candidates** — slider 10-300, controls how many symbols are processed in each scan
- **Auto-Scan Interval** — slider 5-60 minutes (replaces the hardcoded interval of 30 min from `ai_scan_loop`)

All parameters are automatically saved in `titan_settings.json` through `scan_params` field from `AppSettings`.

### 48.3 Refactored Functions

- **`quant_prefilter()`** — accept `&ScanParams` instead of hardcoded values. Apply price, volume, change, RVOL, gap, and catalyst filters.
- **`momentum_prefilter()`** — (formerly `warrior_prefilter`) Ross Cameron strict filter: price $1-$20, RVOL >= 5x, change >= 10%, volume >= 1M. Available as an advanced subfilter.

### 48.4 Watchlist Management

- **DELETE ALL** — deletes all symbols from the watchlist, EXCEPT those that have open live positions. Useful for quick cleaning after a scan session.
- **INJECT ALL -> WATCHLIST** — imports all the results from the last scan into the watchlist, without deleting the existing ones.
- **AUTO INJECT** — toggle that activates the automatic injection of the results of each scan into the watchlist.

### 48.5 Watchlist — Chart interaction

When you click on a symbol from the watchlist, it is displayed on the chart (the main graph changes to the selected symbol).

---

## 49. MANUAL TRADE — USD Manual Transaction

### 49.1 Manual Trade panel

The **Manual Trade** section in the sidebar (collapsible element `<details>`) allows direct manual trading:

| Element | Function |
|---------|---------|
| **Symbol Input** | Text field for the asset symbol (ex: AAPL, TSLA) |
| **LONG / SHORT** | Direction selector: LONG (buy) or SHORT (short sell) |
| **USD Amount** | The amount in USD you want to trade |
| **EXECUTE** | Place the order |
| **FROM WL** | Inject the symbol selected from the Watchlist into the Symbol field |

### 49.2 Backend — `submit_manual_order`

Tauri command `submit_manual_order` processes the order:

1. **Receives:** `ManualOrder { symbol, side, usd_amount }`
2. **Calculate qty:** Fetch current price from cache (or fetch live from Alpaca), divide by `usd_amount / price`, round to whole
3. **Place order:** Call `send_order_internal()` with type `market`
4. **Returns:** Confirmation or error

### 49.3 Remarks

- Manual orders are **MARKET** type (execution at the current price)
- There is no automatic SL/TP protection on manual orders — the user must manage the position
- Manually traded assets appear in the list of live positions with the possibility to set the AUTO or MANUAL mode (see Chap. 51)

---

## 50. FORCE MANUAL OVERRIDE — Total Control

### 50.1 What It Does

The **FORCE MANUAL RES** toggle in the sidebar activates total user control over ALL positions. When active:

- **`autopilot_loop` is completely blocked** — on each iteration, if `force_manual_tf == true`, the loop does `continue` immediately, without evaluating or handling any positions
- **NO automatic entry** — the bot does not open new positions
- **NO automatic exit** — the bot does not close, does not adjust SL/TP, does not do trailing stop
- **User takes full control** — all trading decisions are manual

### 50.2 Visual Indicator

When `force_manual_tf` is active, a pulsating red banner appears in the main UI area:

**"MANUAL OVERRIDE ACTIVE"** — with animation `pulse-border` (semi-transparent red background with pulsating red border)

The banner is synchronized with the toggle state via `refreshSidebarState()`.

### 50.3 Difference from Per-Asset Manual Mode

| Feature | Force Manual Override | Per-Asset Manual (Chap. 51) |
|----------------|----------------------|----------------------------|
| **Purpose** | Lock ALL the bot | Lock the bot on ONE symbol |
| **Positions affected** | ALL | Only the symbol set to "manual" |
| **New entries** | Completely blocked | Allowed on other symbols |
| **When to use** | Emergency situations, temporary total control | Individual management of specific positions |

### 50.4 Interaction with Aggressive Fashion

`force_manual_tf` overrides Aggressive Mode — even if Aggressive Mode is active, if Force Manual Override is ON, the bot does nothing. The `force_manual_tf` check comes BEFORE the aggressive parameters in `autopilot_loop`.

---

## 51. LIVE POSITIONS — Auto/Manual mode per Asset

### 51.1 The MODE column in the Table of Positions

The live position table now contains a **MODE** column with a toggle button per symbol:

| modeller | icona | Color | Effect |
|------|------|---------|-------|
| **CAR** | Robot | Verdant | The bot manages the normal position (SL, TP, trailing) |
| **MANUAL** | Hand | Orange | The bot does NOT interact with the position in any way |

### 51.2 Orders Bulls

- **`set_position_mode(symbol, mode)`** — sets the mode for a symbol: `"auto"` or `"manual"`
- **`get_position_modes()`** — returns `HashMap<String, String>` with all modes active

### 51.3 Storage and Persistence

Mods are stored in `position_trade_modes: HashMap<String, String>` of `AppSettings`/`AppState`. It is automatically saved in `titan_settings.json` through state persistence.

### 51.4 Bot Behavior

In `autopilot_loop`, for each symbol processed:
1. Check if the symbol has mode set in `position_trade_modes`
2. If `mode == "manual"` → the position is completely skipped (skip)
3. If `mode == "auto"` or has no mode set → the position is managed normally

### 51.5 Use Cases

- **Long term position:** Set MANUAL on a symbol you want to keep without bot interference
- **Volatile asset:** Set MANUAL when the market is unpredictable and you want direct control
- **Manual transaction:** After `submit_manual_order`, set MANUAL to prevent auto-exit

---

## 52. DRAWING TOOLS V2 — All Tools

### 52.1 General Presentation

52.2 The 11 Instruments

### 52.2 The 11 Instruments

| # | Instrument | Description |
|---|-----------|-----------|
| 1 | **Crosshair** | Disables drawing mode, restores normal interaction with the chart (zoom, pan) |
| 2 | **Trend** | Trend line between 2 points (click-drag) |
| 3 | **Ray** | Infinite radius — from point 1 through point 2, continuing to the edge of the chart |
| 4 | **HLine** | Horizontal price line — extends across the entire width of the chart at the selected price level |
| 5 | **VLine** | Vertical timeline — from top to bottom at the selected time |
| 6 | **Fib** | Fibonacci retracements — automatic levels: 0%, 23.6%, 38.2%, 50%, 61.8%, 78.6%, 100% |
| 7 | **Text** | Text annotation — click on the chart, enter text in the prompt, place it at the chosen coordinates |
| 8 | **Measure** | Distance measurement — displays the difference in price, percentage, and time between 2 points |
| 9 | **Long** | Long box position — visual areas with entry, target, and stop |
| 10 | **Shorts** | Short box position — visual areas with entry, target, and stop |
| 11 | **Delete** | Delete ALL the drawings on the chart |

### 52.3 Toolbar Execution

Under the chart there is an execution toolbar (`exec-toolbar`):

- **BET $** — input field for the USD amount of the transaction
- **EXEC** — execution button (places order based on drawn Long/Short instrument)
- **AI** — AI assist button
- **DEL** — deletes the last drawing

### 52.4 Persistence

Current drawings are serialized with `serialize()` and saved in `localStorage` with key `drawings_{symbol}`
1. Current drawings are serialized with `serialize()` and saved in `localStorage` with key `drawings_{symbol}`
2. The new symbol designs are restored with `setDrawings()` from `localStorage`

### 52.5 Solution Symbol

Drawing Engine V2 uses `window.currentSymbol` to identify the current symbol, instead of "BTC/USD" hardcoded (fixed from the previous version).

### 52.6 Interaction with the Chart

- When a drawing tool is active (not crosshair), pointer events are routed to the drawing engine
- When the crosshair is selected, the chart returns to normal behavior (zoom, pan, hover info)
- The cursor changes automatically: crosshair for drawing, default for chart interaction

---

## 53. AGGRESSIVE MODE — Full Audit

### 53.1 All Code Paths

Aggressive Mode modifies the behavior of the bot in 4 locations:

| File | Location | Normal | Aggressive |
|--------|---------|--------|---------|
| `trading_engine.rs` | L39-54 | SL: -2.5%, tight trailing | SL: -8%, trailing wider |
| `strategy.rs` | L173-198 | Standard entry thresholds | Lower thresholds (more permissive inputs) |
| `main.rs` (autopilot) | Stop-Loss | 1.0x | 0.52x (stop much further) |
| `main.rs` (autopilot) | take profit | 1.0x | 1.68x (more ambitious target) |
| `main.rs` (autopilot) | 1.0x | 1.0x | 0.67x (tighter trailing) |
| `main.rs` (entry) | `stop_dist` | 0.8x vol_factor | 53.2 Visual Indicator |

### 53.2 Visual Indicator

When Aggressive Mode is active:
- The associated sidebar section receives a red pulsation effect
- Text "AGGRESSIVE ACTIVE" is visibly displayed

### 53.3 Override Sliders

Under the Aggressive Mode toggle there are 3 adjustment sliders:
- **SL %** — stop-loss override percentage
- **TP %** — take-profit override percentage
- **TRAIL %** — trailing stop override percentage

### 53.4 Interaction with Force Manual

**Force Manual Override overrides Aggressive Mode.** If both are active, `force_manual_tf` completely blocks the autopilot — aggressive parameters become irrelevant.

Check order in `autopilot_loop`:
1. `force_manual_tf` → if true, `continue` (skip all)
2. Per-asset manual mode → if manual, skip that symbol
3. Aggressive parameters → apply modified multipliers

---

## 54. EXTENDED HOURS — Full Audit

### 54.1 What It Does

When the **Extended Hours** toggle is active, orders sent to Alpaca include the `extended_hours: true` flag. This flag allows the execution of orders outside normal market hours:

- **Pre-market:** 04:00 - 09:30 ET
- **After-hours:** 4:00 PM - 8:00 PM ET

### 54.2 Backend Implementation

In function `send_order_internal()` of `main.rs`:
1. Check if `enable_extended` is ON in settings
2. It is checked if the asset is NOT crypto (crypto trades 24/7, does not require a flag)
3. If both conditions are met: `"extended_hours": true` is added to the JSON body of the order

### 54.3 Visual Badge

When Extended Hours is active, a purple **"EXTENDED"** badge appears in the application header, next to the market status information. The badge is synchronized with the toggle state via `refreshSidebarState()`.

### 54.4 Observations

- Extended Hours only cooks **entry** (entry) — exit (close) orders are not affected
- Alpaca Paper Trading supports extended hours for testing
- Liquidity in extended hours is significantly lower — spreads are higher, prices more volatile
- Not all actions have sufficient volume in extended hours — market orders may receive weak fills

---

## 55. ELIMINATIONS FROM SESSION 7

### 55.1 Warrior 3rd Bot — REMOVED

**What it was:** A third mode of strategy based on the Ross Cameron criteria (day trading of low-price stocks with momentum). It included a `WARRIOR` toggle in the sidebar, separate cash block, dedicated scoring strategy, `scan_warrior()`, and `warrior_prefilter()` functions.

**What was removed:**
- **Frontend:** checkbox `chk-warrior`, block cash `warrior-cash-block`, toggle `WARRIOR PRO` from the AI ​​Scanner tab, footer warrior section, wiring wallet warrior, option `WARRIOR` from the strategy dropdown Market Discovery
- **Backend:** `warrior_mode` from `AppSettings`, `AppState`, `update_settings`, `settings_from_state`, `apply_settings`; `warrior_cash` from `Wallet` and `WalletSnapshot`; branching warrior from `run_ai_scan`, `ai_scan_loop`, `score_candidate`; function `scan_warrior()` completely deleted
- **JS:** toggle wiring from `sidebar_wiring.js` and `main.js`

**What has been preserved:** The function `warrior_prefilter()` has been renamed to **`momentum_prefilter()`** and is available as an advanced subfilter in the scanner. Ross Cameron's criteria (price $1-$20, RVOL >= 5x, change >= 10%, volume >= 1M) are kept in this function.

**Why:** Warrior functionality was redundant — Ross Cameron criteria are now integrated into the configurable `ScanParams`, and `momentum_prefilter()` remains available as a strictly optional filter.

### 55.2 Kelly Criterion — ELIMINATED

**What it was:** A `KELLY` toggle that enabled Kelly Criterion position sizing — a mathematical formula for sizing positions based on the strategy's historical edge.

**What was removed:**
- **Frontend:** checkbox `chk-kelly`, binding `kelly_active`
- **Backend:** field `kelly_active` from `AppSettings`, `AppState`, `update_settings`, `settings_from_state`, `apply_settings`

**Why:** Kelly Criterion position sizing was not fully integrated into the bot execution pipeline. Sizing position hardcaps (Chap. 44) offer more robust protection.

### 55.3 Sniper List — REMOVED

**What it was:** A dedicated section in the sidebar (`LISTA LUNETIST`) with a table of monitored symbols, own budget controls, and performance statistics. It had dedicated CSS classes and separate management logic.

**What was removed:**
- **Frontend:** the entire section `<details>` of LIST SNIPER, including budget row, budget val, budget ctrl, statistics table
- **CSS:** classes `.lunetist-budget-row`, `.lunetist-budget-val`, `.lunetist-budget-ctrl`, `.lunetist-stats`
- **Button renamed:** "DEEP FOCUS (SNIPER)" → "DEEP FOCUS"

**Why:** The Sniper List functionality was partial and redundant with Watchlist + per-asset modes. The improved watchlist (Ch. 48) and per-asset modes (Ch. 51) cover all use cases.

### 55.4 Summary of Files Affected by Removals

| File | What has changed |
|--------|-----------------|
| `index.html` | Deleted: warrior checkbox, cash block, PRO toggle, footer, wallet wiring, sniper section, kelly checkbox |
| `main.rs` | Deleted: warrior_mode, kelly_active from structs/settings/commands, warrior branching in scan/score |
| `app_state.rs` | Deleted: warrior_mode, kelly_active from AppState and default |
| `scanner.rs` | Deleted: scan_warrior(); renamed: warrior_prefilter → momentum_prefilter |
| `sidebar_wiring.js` | Deleted: chk-warrior wiring |
| `main.js` | Deleted: chk-warrior wiring |
| `styles.css` | Deleted: .lunetist-stats CSS block |

---

## Ch. 56 — Central Panel: Watchlist (tab-9) and HFT Targets (tab-10)

### 56.1 Why they moved from the Sidebar

The left sidebar was too narrow (~250px) to display the tables with all the columns in the AI Scanner (SYMBOL, PRICE, CHG%, VOL, RVOL, GAP%, SCORE, BT W/R, BT PnL, DD, CAT). The columns were either truncated or did not fit at all.

**Solution:** Complete migration of Watchlist and HFT Engine from the sidebar to the central panel, where they have ~800-1200px width. Two new tabs have been created next to "REPLAY":

- **tab-9 — WATCHLIST** (button in the tab bar of the central panel)
- **tab-10 — HFT TARGETS** (button in the tab bar)

### 56.2 Tab-9: WATCHLIST

Displays all the assets in the list of favorites (watchlist) with full columns:

| Column | Description |
|---------|-----------|
| SYMBOL | Asset ticker |
| PRICE | Current price (from Alpaca snapshot) |
| CHG% | Daily change percentage, color-coded (green/red) |
| VOL | Daily volume |
| RVOL | Relative Volume (compared to the 20-day average) |
| GAP | Gap% compared to the previous close |
| SCORE | Calculated score (change% × 2 + RVOL × 15 + log(vol) × 3), color-coded |
| CAT | Catalyst badge — see Chap. 59 |
| TRADE | Button "→MT" — inject into Manual Trade |
| HFT | Button "→HFT" — inject into HFT Engine |
| AI | AI backtest button per asset — see Chap. 57 |

**Buttons from header tab-9:**
- **INJECT ALL → HFT** — add all symbols from the watchlist to the HFT Engine
- **BACKTEST ALL** — run AI scoring on all symbols (see Chap. 57)
- **CHECK NEWS** — check news/catalyst on-demand for all symbols

**Backend:**
- `get_watchlist_enriched` — Tauri command that returns `WatchlistEntry` with snapshot (price, change%, vol, RVOL, gap) and `score` calculated

### 56.3 Tab-10: HFT TARGETS

Displays assets from HFT Engine with **all** columns from AI Scanner:

| Column | Description |
|---------|-----------|
| SYMBOL | Asset Ticker |
| PRICE | Current Price |
| CHG% | Daily Change Percentage |
| VOL | Daily Volume |
| RVOL | Relative Volume |
| GAP | Gap% |
| SCORE | Composite AI Score |
| BT W/R | Backtest Win Rate % |
| BT PnL | Backtest Profit & Loss |
| DD | Max Drawdown — see Ch. 58 |
| CAT | Catalyst badge |

**Backend:**
- `get_hft_target_enriched` — Tauri command that runs AI scoring **live** if the symbol does not exist in the cache `last_scan_results`. It uses `score_candidate()` with `PreFilterCandidate` built from the Alpaca snapshot, returning `AiScanCandidate` complete.

**Auto-refresh:** The table is automatically updated on every `ai-scan-update` event from the backend.

### 56.4 Inject All / Selective from Watchlist

- **Inject in Manual Trade (→MT):** Click on the per-row button, add the symbol to the manual list and set the "manual" via `set_position_mode`
- **Inject in HFT (→HFT):** Click on the per-row button call `add_hft_target`
- **INJECT ALL → HFT:** Button in header tab-9 — inject all symbols from the watchlist into HFT Engine simultaneously
- **INJECT ALL → MT:** Button in header tab-9 — inject all symbols in Manual Trade

---

## Ch. 57 — AI Backtest for Watchlist

### 57.1 Purpose

Allows the user to run the complete AI scoring pipeline (identical to AI Scanner) on Watchlist assets, without waiting for an automatic scan. The results appear in a secondary table with **all** columns from AI Scanner.

### 57.2 Cum se utilizeaza

1. Navigheaza la tab-9 (WATCHLIST)
2. Click **"BACKTEST ALL"** in header — ruleaza scoring pe toate simbolurile
   - Sau click butonul **"AI"** pe un singur rand pentru backtest individual
3. Se afiseaza un progress bar: "SCORING 3/12..." during processing
4. The results appear in table `#wl-ai-results` under the main table

### 57.3 AI Results table columns

Identical to AI Scanner: SYMBOL, PRICE, CHG%, VOL, RVOL, GAP%, SCORE, BT W/R, BT PnL, BT Trades, BT Best, BT Worst, DD, CAT.

### 57.4 Backend

Scoring uses exactly the same function `score_candidate()` from `main.rs`, which calls:
- `fetch_snapshots_batch` for price/volume data
- `run_backtest_sync` for backtest on historical data
- `get_sentiment_score` + `fetch_news_feed` for sentiment score and catalyst
- The result is a `AiScanCandidate` completely identical to the one in AI Scanner

---

## Chap. 58 — Coloana DD (Max Drawdown)

### 58.1 Ce este Max Drawdown

**Max Drawdown (DD)** reprezinta cea mai mare scadere procentuala de la un varf local la un minim local pe durata unui backtest. Example: if the equity increases to $1000 and then decreases to $800, DD = -20%.

### 58.2 Why it is important

- A large DD (eg: -40%) indicates a strategy with high risk and high equity volatility
- A small DD (eg: -5%) indicates a more stable and predictable strategy
- The DD helps to evaluate the "comfort" of a strategy — even if a strategy is profitable, too high a DD can cause premature abandonment

### 58.3 Where it is displayed

| Table | Column |
|-------|---------|
| AI Scanner (tab-6) | DD |
| Smart Picks (sidebar) | DD |
| WL AI Results (tab-9) | DD |
| HFT Targets (tab-10) | DD |

### 58.4 Color Coding

| Valoare | Culoare | Semnificatie |
|---------|---------|--------------|
| < -20% | **Red** (#f44) | High risk |
| -20% ... -10% | **Yellow** (#fa0) | Moderate risk |
| ≥ -10% | **Grey** (#999) | Low risk |

### 58.5 Camp Backend

`bt_max_drawdown: f64` in `AiScanCandidate` — calculated by the `run_backtest_sync` function based on the equity series.

---

## Ch. 59 — Catalyst Badges and Source Links

### 59.1 Catalyst Tab (tab-7) — Clickable Links

In the CATALYST tab, every news headline is now clickable. When clicked, the source web page opens in the system's default browser via `window.__TAURI__.shell.open(url)`.

Also, a **"READ ↗"** button appears on each line for quick access to the source.

### 59.2 Catalyst Toast — READ Button

When a catalyst toast notification appears, it includes a **"READ"** button that opens the news source.

### 59.3 News AI Tab (tab-1)

Headline-urile articolelor au acum un link **"READ ↗"** clickabil langa data publicarii.

### 59.4 Catalyst Badges in Tables

In the tables in tab-9 (Watchlist) and tab-10 (HFT Targets), the **CAT** column displays:

| Badge | Semnificatie |
|-------|-------------|
| `★` (stea galbena) | Catalyst recent (< 1h) |
| `●` (cerc portocaliu) | Catalyst din ultimele 24h |
| `—` (grey) | No catalyst detected |

**Click on the badge:** Opens the news source in the browser.

### 59.5 On-Demand News Check

- **🔍** button per row in the table — call `check_catalyst` Tauri command
- **CHECK NEWS** button in header — check news for all symbols in the tab
- The result (`CatalystCheckResult`) contains: symbol, has_catalyst, headline, url, sentiment_score

### 59.6 Global Frontend Catalyst Cache

`window._catalystCache` — JavaScript object populated from:
- Events `catalyst-alert` from the backend (real-time)
- Data `ai-scan-update` (fields `has_catalyst`, `catalyst_headline`, `catalyst_url`)
- Results `check_catalyst` on-demand

The cache allows instant display of badges without additional requests.

### 59.7 Backend Enhancements

- `AiScanCandidate` — new fields: `catalyst_headline: String`, `catalyst_url: String`
- `CatalystCheckResult` — NEW struct: `symbol`, `has_catalyst`, `headline`, `url`, `sentiment_score`
- `check_catalyst` — Tauri command NEW that calls `fetch_news_feed` + `get_sentiment_score`

---

## Cap. 60 — Manual Trade Panel: List of Assets, Controls, Inject

### 60.1 List of Manual Assets

Function `renderManualAssetList()` dynamically populates the section of the Manual Trade panel with all assets set to "manual" mode. Each row displays:

| Element | Description |
|---------|-----------|
| **Symbol** | Asset ticker (bold) |
| **Status** | "OPEN +$12.50" or "NO POS" if there is no open position |
| **CLOSE** | Button that sends a closing order (market sell/cover) |
| **AUTO** | Button that switches the asset back to autotrading mode |
| **✕** | Remove button — removes the asset from the manual list |

### 60.2 Auto-Mark Manual

When executing an order from the Manual Trade panel (`submit_manual_order` or `submit_order`), the symbol is **automatically** set to "manual" by calling `set_position_mode(symbol, "manual")`. This prevents the muzzle from interfering with the manually open position.

### 60.3 Inject from Watchlist

- **"FROM WL" button** — opens a popup with all the symbols in the watchlist; click on a symbol adds it to the manual list
- **"+ ADD" button** — opens a dialog for manually adding a symbol (text input)
- **"ALL → AUTO" button** — switches all assets from the manual list back to autotrading mode

### 60.4 Auto-Refresh

`renderManualAssetList()` is called automatically from:
- `updateAccount()` — at each refresh of the live positions
- `loadPositionModes()` — when loading mods from the backend

---

## Chap. 61 — Drawing Tools: Rendering Fixes and projectTime

### 61.1 Problem: Rectangle Sticking to the Right Edge

When an end of a drawing (ex: risk box) had a timestamp outside the visible viewport of the chart, `timeToCoordinate()` returns `null`. The original code used hardcoded fallbacks (`x = 9999` or `x = -9999`), which made the rectangles extend to the edges of the screen.

### 61.2 Solution: `projectTime(time)`

A new method `projectTime(time)` was added to the `DrawingManager` class. This performs **linear extrapolation** of the x coordinate for off-screen timestamps, using the visible range of the chart (at least 2 reference points):

1. Finds at least the minimum and maximum visible timestamp with their corresponding x coordinates
2. Calculates `pixelsPerMs = (xMax - xMin) / (tMax - tMin)`
3. Extrapolates: `x = xMin + (time - tMin) * pixelsPerMs`

If the chart does not have enough data, returns `null` instead of an erroneous value.

### 61.3 Rendering Audit — 5 Fixes

1. **Null guards risk box:** Explicit check `if (y1 == null || y2 == null || yStop == null) return;` at the beginning of the rendering risk box block — prevents dimensions `NaN`
2. **Skip-draw on null projection:** `if (x1 == null || x2 == null) return;` in `render()` and `continue;` in `hitTest()` — if `projectTime()` cannot calculate, the drawing is omitted instead of being rendered at x=0
3. **Label overlap prevention:** The TARGET and STOP labels are drawn only when the box is at least **16px** high — prevents overlap text
4. **Canvas state isolation:** `ctx.save()` / `ctx.restore()` around each drawing in the `render()` loop — prevents "leaking" of `lineWidth`, `textAlign`, `shadowBlur`, `lineDash` from one drawing to another
5. **Division by zero guard:** Checking `d.p1.price !== 0` in the calculation `pPct` (TARGET percentage) — prevents values ​​`Infinity`

---

## Chap. 62 — Long/Short Settings Tab (tab-11)

### 62.1 General Presentation

The **LONG/SHORT** tab (tab-11) is a centralized panel for configuring all the parameters related to the risk box (Long/Short) drawings on the chart. It is organized in 6 sections (A-F).

### 62.2 A. Default Risk Parameters

Set the default risk parameters applied when **creating** a new risk box drawing on the chart.

| Parameter | Description |
|-----------|-----------|
| **R:R Ratio** (radio) | R:R mode — slider from 0.5 to 5.0, default 2.0. The stop is automatically calculated based on the selected Risk:Reward ratio |
| **Fixed %** (radio) | Fixed percentage mode — input fields SL% and TP% (ex: SL = 2%, TP = 4%) |

**Behavior:** When creating a new drawing, `onMouseUp()` reads `_lsSettings.riskMode`:
- If "rr": `stopPrice = entry - (target - entry) / rr`
- If "fixed": `stopPrice = entry × (1 - SL%)`, `targetPrice = entry × (1 + TP%)`

### 62.3 B. Position Sizing

| Parameter | Description |
|-----------|-----------|
| **Fixed $ Amount** (radio) | Fixed amount in USD (ex: $500), entered in the input |
| **% of Equity** (radio) | Percentage of the account equity, slider 0.5%-25% |
| **Calculated fields** | Auto-calculated: Shares/Qty, Risk $, Max Loss $, Max Profit $ |

**Calculated fields** are updated automatically when a risk box is selected on the chart, based on entry price, stop price and bet amount.

### 62.4 C. Label Display Options

5 checkboxes that control what information is displayed on the risk boxes on the chart:

| Checkbox | Effect on the chart |
|----------|----------------|
| **Show % change** | Shows the percentage from entry to target/stop (ex: "+2.5%", "-1.2%") |
| **Show $ P/L** | Show profit/loss in USD (ex: "+$125.00", "-$60.00") |
| **Show R:R ratio** | Show the Risk:Reward ratio (ex: "R:R 2.1") |
| **Show entry price** | Show "ENTRY: 150.25" on the reference line (dashed) |
| **Show target/stop prices** | Show prices under labels (ex: "@ 153.50", "@ 148.00") |

### 62.5 D. Execution

| Parameter | Description |
|-----------|-----------|
| **Auto-submit TP/SL brackets** | Checkbox — when active, EXEC/AI orders automatically include bracket orders (Take Profit + Stop Loss) calculated from the drawing |
| **Order Type** | Selector: Market (immediate execution) or Limit (specified price) |

### 62.6 E. Visual

| Parameter | Description |
|-----------|-----------|
| **Profit color** | Color picker for the profit area (default: #00e676 green) |
| **Stop color** | Color picker for the stop/loss area (default: #ff1744 red) |
| **Box Opacity** | Slider 0-100% — controls the transparency of the box fill. Default: 6% (subtly transparent) |

### 62.7 F. Drawing Management

| Button | Action |
|-------|---------|
| **CLEAR ALL DRAWINGS** | Deletes **all** drawings on the current chart (any type) |
| **CLEAR LONG/SHORT ONLY** | Deletes only the risk box type drawings (long/short), keeps lines, text, etc. |
| **CLEAR ALL SYMBOLS** | Clear all drawings for **all** symbols from localStorage |
| **Drawing info** | Show number of drawings: "3 drawings (2 L/S)" |

**Drawing count badge:** A red badge with the total number of drawings appears on the 🗑️ button in the left toolbar. It is updated automatically with each save/load.

### 62.8 Exec Toolbar — EXEC and AI Buttons

When a risk box is selected on the chart, a contextual toolbar appears with:

| Button | Action |
|-------|---------|
| **BET $** | Input for transaction amount, synchronized bidirectionally with tab-11 |
| **EXEC** | Send manual order via `submit_order` with qty calculated from bet/price |
| **AI** | Send AI-assisted order (same flow, flag `ai_mode: true`) |

**Bracket support:** If the checkbox "Auto-submit TP/SL" is active, the order includes `take_profit` and `stop_loss` calculated from the drawing prices.

### 62.9 Settings Persistence

All Long/Short settings are saved in `localStorage` under the `titan_ls_settings` key as JSON. When starting the application, `_lsLoad()` restores the settings and `_lsApplyToUI()` applies them to the controls in tab-11.

The global object `window._lsSettings` contains:
```
{
  riskMode: "rr" | "fixed",
  defaultRR: 2.0,
  fixedSL: 0.02,
  fixedTP: 0.04,
  sizingMode: "fixed" | "equity",
  betAmount: 500,
  equityPct: 0.05,
  showPct: true,
  showDollar: true,
  showRR: true,
  showEntry: true,
  showTargetStop: true,
  autoBracket: false,
  orderType: "market",
  colorProfit: "#00e676",
  colorStop: "#ff1744",
  opacity: 0.06
}
```

---

## 63. AI SCANNER PARALLEL SCORING (RAYON)

AI Scanner has been optimized for maximum speed using parallel processing.

- **Technology:** `rayon::pre_iter` integration in the Rust backend.
- **Performance:** Instead of analyzing symbols one by one, Titan uses all available CPU threads (ex: 12 or 16 threads simultaneously).
- **Impact:** Scan time for 200 symbols decreased from ~2 minutes in less than 30 seconds (depends on the response speed of the Alpaca API).

## 64. DEEP AUDIT REASONING — MODAL WINDOW

Every score generated by AI Scanner can now be audited in detail.

- **Activation:** Click on the 🔍 (Deep Audit) icon next to any symbol in AI Scanner or Smart Picks.
- **Modal Window (V3):** A dedicated cyberpunk interface that "floats" above the application.
- **Content:** Displays the components of the score (Fusion Signal, Momentum, Quality Factors) and the reasons why it received that grade.
- **Type:** If the data is missing (old scans), the system informs you that a new `MANUAL SCAN` is required.

## 65. MANUAL TRADE USD VS SHARES TOGGLE

Total control over how to place stakes in the manual trading panel.

- **USD/SHARES button:** Switch instantly between fixed stake in dollars (ex: $3000) and fixed stake in number of shares (ex: 50 Shares).
- **Legibility:** Fonts in the sidebar have been increased (14px) to be visible without effort during intense trading sessions.
- **Cache Fix:** Script `build.sh` automatically clears app-specific WebView cache (`~/.local/share/com.titan.hft.v7reloaded` and `~/.cache/titan-hft-v7-reloaded`) to ensure visibility of UI changes.

---

## 66. SIGNAL STRATEGY — TRADING BASED ON ARROWS

Signal Strategy is a way of trading that is based exclusively on the BUY/SELL/TRAIL arrows on the chart, ignoring all other parameters of the strategy.

### 66.1 Principle

- When a **BUY** arrow appears on the chart → the bot opens a LONG position
- When a **SELL** or **TRAIL** arrow appears → the bot closes the position
- Do not take into account the score, OFI, spread, or other parameters (if the bypasses are assets)

### 66.2 Activation

Click on the **SIGNAL STRATEGY** button in the mode bar (next to Oracle / GPU / Fusion).

### 66.3 Signal Source Selector

After activating the Signal Strategy, a sub-selector appears: **FUSION** | **ORACLES** | **GPU**. Select from which strategy the arrows are visible on the chart:
- **FUSION** (default) — arrows based on Oracle + GPU blend
- **ORACLE** — arrows based on the 7 mathematical AI indicators
- **GPU** — arrows based on the momentum of the 150K GPU agents

### 66.4 Dedicated Assets List

Signal Strategy has its own list of assets (separate from HFT Targets):
- **ADD** — manually add a symbol
- **INJECT FROM** — AI Scanner, Smart Picks, Shark Scanner, Watchlist
- **EXPORT TO HFT TARGETS** — export to the main HFT list
- **INJECT TO MANUAL** — set all assets as "manual" in Manual Trade list
- **DEL ALL** — delete the whole list

### 66.5 Assets Table with Enriched Information

Each asset displays: PRICE, CHG%, RVOL, VOL, ORACLE score, NEXUS %, FUSION score, SIGNAL (BUY/SELL/NEUTRAL), BEST TF, and action buttons.

---

## 67. SIGNAL STRATEGY — PARAMETERS CONFIGURATION TAB (SIGNAL CFG)

### 67.1 Structure

The Signal CFG tab contains 3 safety sections with parameters independent of the main strategy:

**A. ENTRY SAFETY:**
- Max Spread (bps) — 1 to 500
- Cooldown (seconds) — 1 to 300
- Daily Loss Limit ($) — 10 to 100000
- Max Daily Trades — 1 to 200
- Max Open Positions — 1 to 50
- OFI Threshold — 0 to 100000

**B. SAFETY POSITION:**
- Trailing Stop (%) — 0.1 to 20
- Stop Loss (%) — 0.1 to 30
- Take Profit (%) — 0.1 to 50
- Max Hold Time (seconds) — 60 to 86400

**C. EXIT SAFETY:**
- Min Hold Time (seconds) — 0 to 600
- Min Profit to Trail ($) — 0.1 to 10000
- Max Correlation — 0.1 to 1.0

### 67.2 Bypass per Parameter

Each parameter (except Max Open Positions) has a checkbox **BYP** (Bypass). When active, that parameter is completely ignored by Signal Strategy. It allows granular control — for example, you can bypass the cooldown but keep the stop loss active.

### 67.3 Smart Default

The **SMART DEFAULT** button sets all parameters to optimized values for active trading:
- Spread: 100 bps, Cooldown: 15s, Daily Loss: $500, Max Trades: 50, Positions: 5, OFI: 500
- Trail: 1.5%, SL: 3%, TP: 5%, Hold: 3600s
- Min Hold: 10s, Min Profit Trail: $5, Max Corr: 0.85
- Enable bypass on: spread, cooldown, OFI, hold time, min hold, min profit trail, max correlation

### 67.4 Visual Dimming

Parameters with active bypass are visually dimmed (opacity 40%) to clearly indicate that they are inactive.

---

## 68. SIGNAL STRATEGY — ARROW BACKTEST

### 68.1 Principle

The Signal Strategy backtest simulates trading exclusively based on the BUY/SELL/TRAIL arrows (of the selected strategy: Oracle, GPU, or Fusion) on all 9 Alpaca timeframes.

### 68.2 Parameters

- **Slider period:** 1 month to 12 months (displays "1mo", "3mo", "6mo", "12mo")
- **RUN SIGNAL BT** — run backtest on the selected asset (with the button SEL)
- **BT ALL ASSETS** — run backtest on all assets in the list

### 68.3 Asset selection for Backtest

In the "#" column in the table, each asset has a **SEL** (yellow) button. Click on SEL:
- Select the asset for backtest (radio button hidden under the hood)
- Highlight the row and the SEL button
- After the backtest, the following is displayed:

### After the backtest, the following is displayed:

After the backtest, the following is displayed:
- **Summary:** Symbol, Best Timeframe, Best Return%, Best Sharpe, Period
- **Table on 9 TFs:** Trades, Win%, Return%, Max DD, Profit Factor, Sharpe, Expectancy, Rank
- The best TF receives the "BEST" badge
- The "BEST TF" column from the assets table is updated automatically

### 68.5 Auto-Trade on TF Optim

When the Signal Strategy is active, the bot automatically uses the optimal timeframe (from the last backtest) to draw signals from the chart.

---

## 69. EXTENDED HOURS — AUTOMATIC CONVERSION OF LIMIT ORDERS

### 69.1 The problem

Alpaca rejects "market" type orders; in extended hours with error 422: "extended hours order must be DAY or GTC limit orders".

### 69.2 Automatic Solution

When the conditions are met:
- `extended_hours` is activated in Settings
- The asset is NOT crypto
- The order does NOT have a bracket (TP/SL)

...the order is automatically converted:
- **Type:** `market` → `limit`
- **Limit price:** Current price + 0.5% slippage (buy) or - 0.5% slippage (sell)
- **Time in Force:** `"day"` (obligatory for extended hours)

### 69.3 Cooldown of Failed Orders

After any failed command, the symbol receives a **60 second cooldown**. During this period, the bot no longer tries to trade that symbol. Prevents retry spam.

Cooldown clears automatically at:
- Successful order on the same symbol
- Expiration of the 60 seconds

---

## 70. NON-FRACTIONAL ASSETS — QUANTITY MANAGEMENT

### Some shares (eg: PLU) do not support fractional amounts. Alpaca rejects with 403: "asset is not fractionable".

70.2 Solution

### 70.2 Solution

When placing the order:
1. Checking property `fractionable` from the Alpaca API
2. If the asset is not fractionable, the amount is rounded down to a whole number (`floor`)
3. If the rounded quantity < 1, the order is rejected early with clear message

### 70.3 Example

- Asset PLU, bet $1000, price $1.63
- Calculated Qty: 614,857
- Asset is not fractionable → `floor(614.857)` = **614 shares**
- Order sent with qty=614

---

## 71. CLICK-TO-CHART FROM LIVE POSITIONS

### 71.1 Functionality

Click on any row in the **Live Positions** table, load that asset on the chart and automatically switch to the Complex Chart tab.

### 71.2 Conduct

- Click on row → `setSymbol(symbol)` + `switchTab(0)`
- Click on the action buttons (SELL, Force Close, Mode Toggle) → the respective action (do not load on the map)
- Cursor `pointer` and tooltip on each row to indicate interactivity

---

## 72. CLICK-TO-CHART FROM SIGNAL STRATEGY ASSETS

### 72.1 Functionality

Click on any row in the Signal Strategy Assets table to load the asset on the chart in its best timeframe.

### 72.2 Conduct

- Click on the row (outside the buttons) → `setSymbol(symbol)` + `setTF(best_tf)` + `switchTab(0)` + flash "loaded on chart"
- Click on **SEL** → select for backtest (do not load on the map)
- Click on **BT** → run backtest on that asset
- Click on **+HFT** → add to HFT Targets
- Click on **X** → delete from the list

---

## 73. CRASH-SAFE PERSISTENCE & STATE CHART

### 73.1 Settings Persistence (Disk)

All application settings are saved in `titan_settings.json`:
- **Periodic:** Every 30 seconds (`settings_autosave_loop`)
- **Atomic:** Writing to temporary file (`.tmp`) then `rename` → prevents corruption at crash
- **Atomic:** Writing to temporary file (`.tmp`) then `rename` → prevents corruption at crash

Persistent settings include: all toggles, sliders, scalping modes, Signal Strategy bypasses, signal source, signal assets list, scan params, position modes, etc.

### 73.2 State Snapshot (Disk)

The runtime state is saved separately in `titan_state_snapshot.json`:
- Trailing stops, partially taken, last prices
- Session analytics (equity, PnL, drawdown, trade PnLs)
- AI scan results complete
- Scan progress (for self-summaries)

### 73.3 Chart State Preservation

When switching between tabs and returning to the Complex Chart:
- **Zoom and scroll** are kept (not reset)
- **Timeframe** is kept
- **The symbol** is kept

Implementation: `resizeCharts()` instead of `chart.timeScale().fitContent()` when returning to tab 0.

## 74. CATALYST CROSS-TAB INTEGRATION (Deep Audit)

The Catalyst system doesn't work in isolation in Tab 7 — it's integrated into **5 tabs** via global helpers.

### 74.1 `_catalystBadgeHtml(sym, candidate)` — Global Badge

Global JS function that generates the CAT badge on any table:
- **🔻CAT** (red) for negative scores
- **🔻CAT** (red) for negative scores
- Tooltip with headline, score and age ("2m ago")
- Click opens the news URL in external browser
- Use `catalystSeverityColor()` for consistent colors

### 74.2 `_updateCatalystCacheFromResults(results)` — DRY Helper

Centralized update cache function, used in 3 places:
- Watchlist CHECK NEWS
- HFT Targets CHECK NEWS
- `_checkSingleCatalyst` per-symbol check

Complete removal of the 3 blocks of duplicate code.

### 74.3 Integration per Tab

| Tab | Feature Catalyst |
|-----|------------------|
| **Tab 1 (News AI)** | Button 🔥 TEST BULLISH (gradient), article cards with severity badge (EXPLOSIVE/STRONG) |
| **Tab 6 (AI Scanner)** | CAT badge in scan results tables + SMART PICKS, auto-update cache at `ai-scan-update` |
| **Tab 9 (Watchlist)** | CAT badge per symbol + CHECK NEWS button on all symbols (with error feedback) |
| **Tab 10 (HFT Targets)** | CAT badge per target + CHECK NEWS button (with error feedback) |
| **CFG signal** | Toggle REQUIRE CATALYST (scan filter) |

### 74.4 Error Feedback on CHECK NEWS

On failure, the CHECK NEWS (Watchlist + HFT) buttons display:
- Red `ERROR` text for 2 seconds
- Then back to normal text `CHECK NEWS`
- The error is logged to the console

---

## 75. ARCHITECTURE MODERNIZATION — data.rs SPLIT (Phase 9)

### Problem

The file `commands/data.rs` had ~3,800 lines with mixed concerns: market data, order execution, various operations, scoring/backtest. Difficult to navigate, difficult to revise during code review, increased risk of merge conflicts.

### Solution — 4 Domain Modules

| Module | Responsibility | Lines |
|-------|-----------------|-------|
| `commands/market_data.rs` | Candles, snapshots, quotes, price queries | ~300 |
| `commands/order_exec.rs` | Order execution, bracket orders, order management | ~250 |
| `commands/ops.rs` | Health check, state queries, miscellaneous operations | ~200 |
| `commands/scoring.rs` | Backtest scoring, GPU scores, signal markers, AI scan scoring | ~2,415 |

`data.rs` has been reduced to a **re-export shim** (~50 lines) that makes `pub use` out of sub-modules. All existing Tauri commands work identically — zero breaking changes.

### `TitanError` is the application's central error enum, defined in `src-tauri/src/error.rs`:

```
commands/
├── mod.rs              ← Module declarations
├── data.rs             ← Re-export shim
├── market_data.rs      ← Market data queries
├── order_exec.rs       ← Order execution
├── ops.rs              ← Operations & state
├── scoring.rs          ← Scoring & backtest (cel mai mare modul)
├── backtest.rs         ← Backtest sweep
├── account.rs          ← Account management
├── trading.rs          ← Trading commands
├── settings.rs         ← Settings CRUD
├── favorites.rs        ← Watchlist favorites
├── hft_targets.rs      ← HFT targets
├── journal_cmd.rs      ← Trade journal
├── signal.rs           ← Signal strategy
├── scanner_cmd.rs      ← Scanner commands
├── monitoring.rs       ← Telemetry & heartbeat
├── strategy_cmd.rs     ← Strategy scoring
└── misc.rs             ← Replay, heatmap, alerts
```

---

## 76. TITAN ERROR SYSTEM (Phase 8)

### The TitanError structure

`TitanError` is the application's central error enum, defined in `src-tauri/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error, Serialize)]
pub enum TitanError {
    Network(String),     // Erori retea (API calls, WebSocket)
    Broker(String),      // Erori broker (Alpaca rejects, 403/422)
    Validation(String),  // Input invalid (parametri gresiti)
    Internal(String),    // Erori interne (logica, state)
    Io(String),          // Erori I/O (fisiere, disk)
    Gpu(String),         // Erori GPU (WGPU, compute shaders)
    Timeout(String),     // Timeout-uri (API, WebSocket)
    RateLimit(String),   // Rate limiting (API throttle)
}
```

### Automatic Conversions (From impls)

- `From<reqwest::Error>` — detects timeout vs generic network
- `From<serde_json::Error>` — wraps as Internal("JSON: ...")
- `From<std::io::Error>` — wraps like Io
- `From<toml::de::Error>` — wraps as Internal("TOML: ...")
- `From<String>` — wraps as Internal (backward compat)
- `From<TitanError> for String` — Tauri 1.x compatibility (send as string to frontend)

### Macro titan_bail!

```rust
titan_bail!(Network, "API call failed: {}", url);
// Expands to: return Err(TitanError::Network(format!("API call failed: {}", url)))
```

### Impact

- 84+ functions migrated from `Result<T, String>` to `Result<T, TitanError>`
- The `?` operator works naturally on reqwest, serde_json, io, toml
- Zero frontend changes (TitanError is automatically converted to String for Tauri)

---

## 77. DECIMAL MIGRATION — Tier 1 (Phase 7C)

### Fields Migrated to rust_decimal::Decimal

**AppState / AppSettings:**
- `equity` — the total capital of the account
- `bot_trade_size` — standard size per trade
- `circuit_breaker_limit` — circuit breaker limit
- `max_manual_order_usd` — maximum head per manual order

**TradeAction:**
- `price` — the execution price
- `qty` — quantity
- `pnl` — profit/loss

**EntryContext:**
- `equity` — equity at the time of entry

**StateSnapshot (6 fields):**
- `session_starting_equity`, `session_peak_equity`, `session_current_equity`
- `session_total_pnl`, `session_best_trade`, `session_worst_trade`

### Serde Adapters

For compatibility with existing JSON files (which store numbers as floats), custom adapters are used:

```rust
#[serde(serialize_with = "serialize_decimal_as_f64")]
#[serde(deserialize_with = "deserialize_decimal_from_f64")]
pub equity: Decimal,
```

### What remains f64

78. FRONTEND MODULARIZATION (Phase 7E)

---

## 78. FRONTEND MODULARIZATION (Phase 7E)

### From Monolith to 30 Modules

`index.html` was reduced from **15,454 lines** to **2,595 lines** by:

1. **External CSS** — block `<style>` of 2,877 lines extracted in `src/styles.css`
2. **30 JavaScript modules** — all IIFEs and inline scripts extracted into separate files
3. **switchTab fix** — chain of 6 overrides on `window.switchTab` replaced with a single definition in `titan_tabs.js`

### Loading Order

The scripts are loaded in `<head>` (before the DOM) and at the end of `<body>` (after the DOM):

**Head (utilities and core):**
```html
<script src="lightweight-charts.js"></script>
<script src="titan_core.js"></script>
<script src="titan_bus.js"></script>
<!-- ... 16 module infrastructure ... -->
<script src="titan_bootstrap.js"></script>
```

**End of body (feature modules, after DOM):**
```html
<script src="titan_globals.js"></script>
<script src="titan_journal.js"></script>
<!-- ... 8 module feature ... -->
<script src="titan_ls_settings.js"></script>
```

### Shared Utilities (titan_core.js)

All modules have access to utilities from `titan_core.js`:
- `escapeHtml(str)` — XSS protection
- `normalizeSymbol(sym)` — token normalization (BTC → BTC/USD)
- `formatNum(n, d)`, `fmtPct(n)`, `fmtUsd(n)` — numeric formatting
- `formatVolume(val, decimals)` — volume formatting (K/M/B)
- `pnlColor(n)`, `sideColor(side)` — conditional colors
- `$el(id)` — DOM cache helper
- `setTxt(id, val, color)` — set text content (cached)
- `TitanTimers` — centralized interval registry
- `TitanError` — centralized error reporting
- `TitanDebug` — debug logging with timestamp
- `window.invoke(cmd, args)` — Tauri invoke wrapper

---

## 79. XSS HARDENING & escapeHtml (Phase 10)

### The escapeHtml function

```javascript
function escapeHtml(str) {
    if (!str) return '';
    return String(str).replace(/&/g,'&amp;')
        .replace(/</g,'&lt;').replace(/>/g,'&gt;')
        .replace(/"/g,'&quot;').replace(/'/g,'&#039;');
}
```

### Systematic Audit

All `innerHTML` assignments that include dynamic values ​​were audited in 15 JS files. String values ​​(symbols, headlines, timestamps, labels, IDs) are now wrapped with `escapeHtml()`:

```javascript
// Inainte (vulnerabil la XSS):
row.innerHTML = `<td>${t.symbol}</td><td>${t.reason}</td>`;

// Dupa (hardened):
row.innerHTML = `<td>${escapeHtml(t.symbol)}</td><td>${escapeHtml(t.reason)}</td>`;
```

### Intentional Exceptions

Calculated numeric values ​​(formatted with `.toFixed()`, `formatVolume()`, etc.) are NOT escaped — they are safe by construction because they produce only digits and characters `.`, `%`, `$`, `-`.

### Unified Helper formatVolume

`fmtVol` and `fmtK` (two duplicate functions) have been replaced with a single function:

```javascript
function formatVolume(val, decimals = 1) {
    const n = Number(val);
    if (!Number.isFinite(n)) return "--";
    const abs = Math.abs(n);
    if (abs >= 1e9) return (n / 1e9).toFixed(decimals) + "B";
    if (abs >= 1e6) return (n / 1e6).toFixed(decimals) + "M";
    if (abs >= 1e3) return (n / 1e3).toFixed(decimals) + "K";
    return n.toFixed(0);
}
```

---

## 80. CENTRALIZED ERROR HANDLING (Phase 11)

### TitanError.report (Frontend)

Defined in `titan_core.js`, replaces `console.error` in catch blocks:

```javascript
const TitanError = {
    _log: [],
    report(context, error) {
        const entry = { ts: Date.now(), context, error: String(error) };
        this._log.push(entry);
        if (this._log.length > 200) this._log.shift();
        console.error(`[TITAN:${context}]`, error);
    },
    getLog() { return this._log; }
};
```

### TitanDebug.log

Logging with timestamp, partial replace for `console.log`:

```javascript
const TitanDebug = {
    log(tag, ...args) {
        console.log(`[${new Date().toISOString()}][${tag}]`, ...args);
    }
};
```

### TitanTimers

Centralized registry for `setInterval` / `clearInterval`:

```javascript
const TitanTimers = {
    _timers: {},
    set(name, fn, ms) {
        if (this._timers[name]) clearInterval(this._timers[name]);
        this._timers[name] = setInterval(fn, ms);
    },
    clear(name) {
        if (this._timers[name]) { clearInterval(this._timers[name]); delete this._timers[name]; }
    },
    clearAll() { Object.keys(this._timers).forEach(k => this.clear(k)); }
};
```

### Global Error Handlers

```javascript
window.onerror = (msg, src, line, col, err) => {
    TitanError.report('window:onerror', `${msg} at ${src}:${line}:${col}`);
};
window.addEventListener('unhandledrejection', (e) => {
    TitanError.report('unhandledrejection', e.reason);
});
```

### Console.warn Preserved Intentionally

~20 non-critical `console.warn` are preserved (localStorage fallback, CRUD operations, capability detection) — they are not errors but operational information.

---

## 81. DOM CACHE SYSTEM — $el Helper with WeakRef (Phase 3 + RF-D Part 1)

### Implementation (titan_core.js)

```javascript
const _domWeakCache = typeof WeakRef !== 'undefined' ? Object.create(null) : null;

function $el(id) {
    if (_domWeakCache) {
        const ref = _domWeakCache[id];
        if (ref) {
            const el = ref.deref();
            if (el && el.isConnected) return el;   // cache hit
            // stale — re-resolve
        }
        const el = document.getElementById(id);
        if (el) _domWeakCache[id] = new WeakRef(el);
        return el;
    }
    return document.getElementById(id);             // fallback fara WeakRef
}
```

### How It Works

- **WeakRef cache:** Each element found is stored as `WeakRef`. When the node is GC-ed or disconnected from the DOM (`!isConnected`), the reference is automatically invalidated and a new DOM search is made.
- **Null misses:** Non-existent elements (null) are not cached — they are re-tried at each call. Prevents blocking on elements that appear later in the lifecycle.
- **Fallback:** If `WeakRef` does not exist at runtime, `$el()` directly does `document.getElementById()` every time.
- `setTxt(id, val, color)` uses `$el(id)` internally — so all calls to `setTxt` benefit automatically.

### Debug API

`window.TitanDomCacheDebug.snapshot()` returns live counters:

| Counter | What it measures |
|--------|-----------|
| `lookups` | Total calls `$el()` |
| `cacheHits` | Returned from WeakRef without DOM query |
| `staleInvalidations` | Re-resolve after `deref()` null or `!isConnected` |
| `domQueries` | Effective searches `document.getElementById()` |
| `domQueryMisses` | Searches that returned null |
| `fallbackDirectCalls` | Direct calls (WeakRef unavailable) |
| `weakRefEnabled` | `true` if WeakRef is active |

`window.TitanDomCacheDebug.reset()` — reset all counters to 0.

### Migrated Files (RF-D Part 1)

All `document.getElementById` calls in these files have been replaced with `$el()`:
- `titan_perf_stats.js` (~186 instances)
- `titan_ls_settings.js` (~72 instances)
- `titan_manual_trade.js` (~60 instances)

Plus hot-paths converted to original Phase 3:
- `titan_monitor.js` — `updateNyTime` (1s), `applyAiTargetUi` (1.5s), `applyRedisStatus`
- `titan_positions.js` — `updateAccount` (3s), `updateStrategyScore` (4s)
- `titan_bootstrap.js` — `price-update` (200ms), `l2-top` `l2-update` listeners
- `titan_core.js` — `renderOfi`, `renderFlow`
- `titan_indicators_ui.js` — `bindIndicatorToggles`, toggle state
- `titan_chart_settings.js` — `applyIndicatorToggleState`

### Anti-Regression Guard

`guard_no_direct_getelementbyid.sh` — script that fails if `document.getElementById(` reappears in the migrated files. Automatically run by `build.sh` at step 2 (DOM regression guard). As new files are migrated, they are added to the list in the script.

### Extension Scope

There are ~400+ `document.getElementById` calls left in the rest of the modules (titan_hft_panel, titan_bootstrap, titan_sidebar, etc.), planned for incremental conversion in the next sessions (RF-D Part 2+).

---

## 82. PRODUCTION HARDENING (Phase 7H)

### Cargo Profile Release

```toml
[profile.release]
lto = true           # Link-Time Optimization (binar mai mic, mai rapid)
codegen-units = 1    # Optimizare maxima (un singur codegen unit)
panic = "abort"      # Fara stack unwinding in productie
strip = true         # Strip debug symbols
opt-level = 3        # Optimizare maxima pentru viteza
```

Result: binary ~14.7MB (from ~17MB without LTO/strip).

### Feature Flags

```toml
[features]
default = ["gpu", "telegram", "web-dashboard", "options"]
custom-protocol = ["tauri/custom-protocol"]
gpu = []
telegram = []
web-dashboard = []
options = []
```

Optional modules gate-watched with `#[cfg(feature = "...")]`:
- `mod gpu` + GPU Nexus loop + compute shaders
- `mod telegram` + alert integration points
- `mod web_dashboard` + axum server
- `mod options` + options chain fetch + hedge commands

### CI Compliance

- `cargo fmt` — consistent formatting across the codebase
- `cargo clippy --workspace -- -D warnings` — zero warnings (blocks the build)
- `cargo audit` — CVE vulnerability scan
- `cargo deny check` — license audit + advisory scan (deny.toml with whitelist)
- `cargo audit` — CVE vulnerability scan
- GitHub Actions workflows (`.github/workflows/ci.yml` + `release.yml`) — run automatically at push/PR/tag
- Criterion benchmarks available on-demand: `cargo bench` (oracle, indicators, full pipeline)

### Build Script — 13-Step Pipeline with Validation-First

`build.sh` runs 13 sequential steps. All checks (steps 1-6) are executed BEFORE build/deploy — any failure stops the pipeline without affecting the current installation.

| Step | what is he doing | Control lime env |
|-----|---------|-------------------|
| 1 | `node --check` on 33 first-party JS files (excludes `lightweight-charts.js`, `node_modules`, `dist`, `target`) | — |
| 2 | DOM regression guard (`guard_no_direct_getelementbyid.sh`) — check that migrated files do not reintroduce `document.getElementById` | — |
| 3 | Static UI smoke — serves local `src/`, launches Chrome headless + CDP, tests Manual Trade, quick buy/sell, KILL ALL confirm, cyberConfirm. Requires `requests` + `websockets` Python modules. | `TITAN_SKIP_STATIC_UI_SMOKE=1` (skip) |
| 4 | `cargo test --workspace` (259 tests, fail-fast) | — |
| 5 | `cargo clippy --workspace -- -D warnings` (**blocks** at any warning) | — |
| 6 | Benchmarks (optional): focused `bench_check_entry` or full `cargo bench` | `TITAN_RUN_BENCH=1` (focused), `TITAN_RUN_FULL_BENCH=1` (full) |
| 7 | `rsync -a --delete src/ → dist/` | — |
| 8 | `cargo build --release --features custom-protocol` | — |
| 9 | Clear Webview cache (`~/.local/share/com.titan.hft.v7reloaded`, `~/.cache/titan-hft-v7-reloaded`) | — |
| 10 | Stop the instance running | — |
| 11 | Deploy binary to `~/.local/bin/titan-hft-v7-reloaded` + desktop entries + icons (32, 128, 512) | — |
| 12 | Launch with logging in `$HOME/HFT_Bot/logs/titan_YYYYMMDD_HHMMSS.log` | — |
| 13 | Summary: duration, binary size, timestamp | — |

**Wrapper with benchmark:** `build_bench.sh` — executable that runs `TITAN_RUN_BENCH=1 ./build.sh` (shortcut for full validation with focused benchmark).

---

## 83. SIGNAL SCALPING KIT (Session 17) — Realtime Alerts + Exact Journal

This chapter documents the updates for manual scalping in `SIGNAL_STRATEGY`, delivered in session 17.

### 83.1 What's new in UI

- `SIGNAL HEALTH` panel displays live:
  - `SRC` (FUSION / ORACLE / GPU)
  - `TF` active
  - `AGE` last update
  - marker number `BUY / SELL / TRAIL / CHAOS`
- New controls in the panel:
  - `ALERTS ON/OFF`
  - `VOL` (0-100%)

### 83.2 Realtime alerts (audio + visual)

- Triggers for new signals: `BUY`, `SELL`, `TRAIL`, `CHAOS`
- Each type has:
  - dedicated toast with distinctive color
  - distinct audio tone
- When `ALERTS` is `OFF`:
  - no audio is playing
  - no toast appears
  - new signals are internally marked as seen (avoids spam upon reactivation)

### 83.3 Persistence of alert preferences

Preferences are stored locally in browser storage:

- `enabled` (true/false)
- `volume` (0.0-1.0)

Upon restart, the application automatically restores the last configuration.

### 83.4 Journal exactly on source/timeframe (backend)

`TradeRecord` explicitly includes:

- `source` (`ORACLE` / `GPU` / `FUSION` / `UNKNOWN`)
- `timeframe` (`1m`, `3m`, ..., `4h`, `n/a`)

Benefits:

- KPIs on the source/TF become accurate (without inference from the text)
- Backend CSV export includes columns `Source` and `Timeframe`
- Old data remains compatible (`UNKNOWN` / `n/a` default)

### Set `MODE = SIGNAL`, then choose `SRC` (`FUSION`, `ORACLE`, `GPU`).

1. Activate `SIGNALS` on the chart and check the live markers.
2. Activate `SIGNALS` on the chart and check the live markers.
3. In `SIGNAL HEALTH`, leave `ALERTS ON` and set `VOL` comfortably.
4. After the session, analyze the KPI Journal on `source` and `timeframe`.

### 83.6 Operational note

For the next session, the priority backlog remains:

- 7D Strategy Indicators
- DOM Cache Expansion
- Decimal Tier 2

---

## 84. ALL CONFIGURABLE TRADING PARAMETERS (Session 18)

In session 18, **absolutely all** parameters that the bot uses in the trading process were exposed. Previously, these values ​​were hardcoded in the Rust code. Now each is:

- Visible and adjustable through the slider in the **HFT Engine** tab
- Automatically persisted in `titan_settings.json` to SAVE
- Strictly respected by the bot in all trading decisions
- Backward-compatible: the default values ​​are identical to the previous ones

### 84.1 AUTOPILOT CORE (10 parameters)

Controls the fundamental behavior of the autopilot:

| Parameter | Default | field | Description |
|-----------|---------|---------|-----------|
| `min_hold_secs` | 180 | 0-3600 | Minimum holding time before forced exit |
| `nexus_consensus_threshold` | 70 | 30-100 | Nexus consensus threshold for stop/take adjustment |
| `autopilot_loop_ms` | 1500 | 500-10000 | `ofi_stale_ms` |
| `ofi_stale_ms` | 10000 | 1000-60000 | The time after which the OFI is considered expired |
| `min_bars_for_entry` | 20 | 5-200 | No. minimum number of candles required for entry |
| `ofi_entry_threshold` | 5.0 | 0-50 | OFI threshold for entry confirmation |
| `max_qty_cap` | 1000 | 1-100000 | Maximum quantity per order |
| `min_qty_threshold` | 0.001 | Minimum quantity accepted | Minimum quantity accepted |
| `default_equity_fallback` | 10000 | 1000-1000000 | Equity default when the data is missing |
| `max_bypass_risk_pct` | 25 | Maximum risk in bypass mode | Maximum risk in bypass mode |

### 84.2 EXIT SCALING (17 parameters)

Controls how stop-loss, take-profit and trailing stop are calculated:

| Parameter | Default | Description |
|-----------|---------|-----------|
| `vol_factor_min/max` | 0.6 / 2.5 | Volatility factor domain |
| `fallback_sl/tp/trail_pct` | 2.5 / 2.5 / 1.5 | SL/TP/trail fallback percentages |
| `scalping_stop/take/trail_mult` | 0.70 / 0.60 / 0.50 | Scalping multipliers |
| `partial_first_target_frac` | 0.5 | TP fraction for the first partial output |
| `partial_min_qty` | 0.001 | Minimum quantity for partial output |
| `atr_period` | 14 | ATR period |
| 0.5 / 8.0 | 0.5 / 8.0 | ATR stop clamping range |
| `atr_take_clamp_min/max` | 1.0 / 15.0 | ATR take clamping range |
| `atr_trail_clamp_min/max` | 0.3 / 5.0 | ATR trail clamping area |

### 84.3 ORACLE EXIT (6 parameters)

Thresholds for Oracle-based outputs:

| Parameter | Default | Description |
|-----------|---------|-----------|
| Minimum confidence for exit on opposite signal | 0.40 | Minimum confidence for exit on opposite signal |
| `oracle_exit_min_score` | 30.0 | The minimum score for exit on the opposite signal |
| `oracle_flip_score_threshold` | 10.0 | Score Threshold for Oracle Flip Exit |
| `oracle_flip_confidence` | 0.20 | Minimum confidence for Flip Exit |
| `chaos_exit_max_loss_pct` | -0.5 | Maximum loss in Chaotic mode |
| `chaos_exit_min_confidence` | 0.25 | Minimum confidence for Chaos Exit |

### 84.4 CIRCUIT BREAKER (5 parameters)

Daily loss protection thresholds:

| Parameter | Default | Description |
|-----------|---------|-----------|
| `breaker_caution_pct` | -3.0% | Warning threshold |
| `breaker_halt_pct` | -5.0% | Threshold for stopping new entries |
| `breaker_panic_pct` | -8.0% | Panic threshold (kill switch) |
| `max_notional_equity_frac` | 0.25 | The maximum fraction of equity per position |
| `min_stop_distance_frac` | 0.0001 | Minimum stop distance as a fraction of the price |

### 84.5 STRATEGY SCORING (9 parameters)

Parameters of the strategy scoring engine:

| Parameter | Default | Description |
|-----------|---------|-----------|
| `strategy_base_score` | 50 | Base Score (Neutral) |
| `vpin_gate_scalping` | 0.85 | VPIN threshold for scalping |
| `vpin_gate_normal` | 0.75 | VPIN threshold for normal trading |
| `min_rvol_threshold` | 0.5 | The relative minimum volume for entry |
| `rvol_window` | 20 | RVOL calculation window |
| `rvol_relax_threshold` | 3.0 | RVOL threshold for relaxation |
| `rvol_relax_value` | 5.0 | Threshold adjustment at high volume |
| `chaos_confidence_kill` | 0.10 | The confidence threshold below which the score is cancelled |
| `chaos_regime_mult` | 0.55 | Regime multiplier for Oracle contribution |

### 84.6 ENGINE STOPS (6 parameters)

Stops and trails from the trading engine (manage_positions):

| Parameter | Default | Description |
|-----------|---------|-----------|
| `long_stop_normal` | -2.5% | Normal stop loss for long |
| `long_stop_aggressive` | -8.0% | Aggressive stop loss for long |
| `short_stop_pct` | -5.0% | Stop loss for short |
| `long_trail_min_profit` | 1.5% | Minimum profit for trailing long activation |
| `short_trail_gap` | 1.5% | The trailing gap for short |
| `short_trail_min_profit` | 1.0% | Minimum profit for trailing short |

### 84.7 ORACLE FUSION (8 parameters)

Oracle fusion weights and thresholds (7 layers):

| Parameter | Default | Description |
|-----------|---------|-----------|
| `oracle_kalman_weight` | 0.30 | Kalman weighting in fusion |
| `oracle_entropy_weight` | 0.25 | Wavelet Weight/Entropy |
| `oracle_hurst_weight` | 0.20 | Hurst weight |
| `oracle_vpin_weight` | 0.15 | Share of VPIN |
| `oracle_wavelet_weight` | 0.10 | Entropy direction weight |
| `oracle_confidence_floor` | 0.25 | The minimum confidence threshold |
| `entropy_chaos_threshold` | 0.85 | Entropy threshold for chaotic regime |
| `vpin_toxicity_alert` | 0.6 | VPIN threshold for toxicity alert |

### 84.8 PERIOD INDICATOR (12 parameters)

Periods of all classic technical indicators:

| Parameter | Default | Description |
|-----------|---------|-----------|
| `ema_fast_period` | 9 | Fast EMA |
| `ema_mid_period` | 20 | Average EMA |
| `ema_slow_period` | 50 | Slow EMA |
| `ema_trend_period` | 200 | EMA trend |
| `rsi_period` | 14 | RSI period |
| `macd_fast` | 12 | MACD fast line |
| `macd_slow` | 26 | MACD slow line |
| `macd_signal` | 9 | MACD signal |
| `bb_period` | 20 | Bollinger Bands period |
| `bb_std_dev` | 2.0 | Bollinger Bands standard deviation |
| `adx_period` | 14 | ADX period |
| 14 | 14 | Stochastic RSI period |

### 84.9 How it works technically

1. **UI**: Each parameter has a slider in the HFT Engine tab, organized in 8 color-coded sections
2. **Save**: When you click SAVE, all values ​​are sent as a batch to the backend (`batch_update_settings`)
3. **Backend**: `apply_setting_field` parses, clamps and writes to `AppState` + `titan_settings.json`
4. **Bot**: Autopilot, strategy, trading_engine, risk, oracle and indicators read from `settings.*` instead of hardcoded values
5. **Load**: On the first visit to the tab after the restart, `loadHftSettings` hydrates the sliders from the backend snapshot

### 84.10 Recommendations for use

- **Do not modify all parameters at once** — adjust one section at a time and observe the effect
- **The default values ​​are optimized** — they are the ones that were tested and hardcoded initially
- **Circuit Breaker parameters are critical** — don't weaken the protections unless you know exactly what you're doing
- **Indicator Periods affect signals** — their change modifies BUY/SELL signals
- **After changes, run in DRY RUN** before real trading

---

## FINAL NOTE

TITAN HFT (Shark V7 Reloaded) is a complete automatic trading system with **87 chapters** of documentation.

**Sessions 15-16 (March 25-26, 2026)** executed Phase 7 completely (Architecture, Modernization & Production Readiness):
- **7A** Dead code cleanup (10 items removed, 10 structures deduplicated)
- **7B** TitanError migration (84+ functions, thiserror, titan_bail! macro)
- **7C** Decimal migration Tier 1 (currency fields AppState/TradeAction/StateSnapshot)
- **7E** Frontend modularization (30 JS modules, index.html 15K→2.5K lines)
- **7F** Aggressive param cleanup
- **7G** CI compliance (cargo fmt + clippy + test + cargo-deny + cargo-audit + GitHub Actions)
- **7H** Production hardening (LTO, feature flags, strip, opt-level 3)
- **Phase 8** Production Safety Sweep (unwrap elimination, XSS hardening, global error handlers)
- **Phase 9** data.rs split (3,800 lines → 4 domain modules)
- **Phase 10** innerHTML hardening & formatVolume dedup (15 audited files)
- **Phase 11** Console error → TitanError.report migration (33 calls, 15 files)

**Session 17 (March 27, 2026)** added the SIGNAL toolkit for manual scalping:
- realtime BUY/SELL/TRAIL/CHAOS alerts
- controls `ALERTS ON/OFF` + `VOLUME`
- backend log with `source` + `timeframe` explicit for exact KPI

**Session 18 (March 27, 2026)** exposed absolutely all trading parameters (~80+) in the HFT Engine tab:
- 8 new sections: Autopilot Core, Exit Scaling, Oracle Exit, Circuit Breaker, Strategy Scoring, Engine Stops, Oracle Fusion, Indicator Periods
- Each parameter: visible as a slider, persisted in `titan_settings.json`, strictly respected by the bot
- Backend: 10 modified Rust files, extended `SizingParams`, `OracleWeights` struct, `IndicatorPeriods` struct
- Validations: cargo check OK, clippy 0 warnings, 80/80 tests passed

**Session 19 (March 27-28, 2026)** — Deep Audit + 12 Smart Features + Complete Styling:
- 12 new features: Emergency Kill Dashboard, DRY/LIVE Badge, Undo Disk Persist, Settings Diff View, Per-Symbol Overrides, Live Impact Preview, Regime Heatmap, Parameter Profiles, Risk Calculator, Parameter Health, Auto-Suggest, Search Filter
- SAFE/SCALP presets expanded to 90+ sliders (all sections)
- Completely refactored styling: unified cyan sliders, dark inputs, text sizing, purple/orange health badges
- Chart persistence on disk (`titan_chart_settings.json`) — survives WebView cache clear
- updated build.sh with icon install step
- HFT Engine Dedicated Manual: `TITAN_HFT_ENGINE_MANUAL.md`
- Validations: cargo check OK, clippy 0 warnings, 80/80 test passed, 0 warnings

**Session 20 (March 28-29, 2026)** — Cyber-HUD + Ross Cameron Columns + FMP Enrichment:
- Cyber-HUD theme on 3 tables (HFT, Signal, Watchlist): glassmorphism, neon cyan/magenta/gold
- Complete Ross Cameron columns: GAP%, FLOAT, CAP, INS, SCORE on all 3 tables
- FMP enrichment universal (pipeline cache, graceful fallback)
- Cross-module refresh automatically when injecting symbols

**Session 21 (March 29, 2026)** — Phase 9 Final Upgrades:
- **Position FSM** — enum `PositionState` with 5 states, 5 new fields on Position, backward compatible
- **Micro-Latency Metrics** — ring buffers for RTT/GPU/streamer order, p50/p95/p99 percentiles
- **WebSocket Dashboard** — live push in 2s, trade/alert events, JS client with auto-reconnect (removed polling)
- **Session Analytics** — streak tracking, best/worst PnL, average duration, partial exit journaling
- Clippy 0 warnings, 122 tests passed

Backend: 59+ Rust files, 122 tests, 0 clippy warnings, 4 workspace crates (titan-types, titan-risk, titan-genetic, titan-indicators). Frontend: 30 JS modules (at the time of session 21; now 33 modules), DOM cache $el with WeakRef, centralized TitanError.report, TitanTimers registry. Live WebSocket dashboard.

Left for the next session: 7D (Strategy TA Filters), DOM Cache Expansion, Decimal Tier 2, Profile persistence on disk, Latency Metrics UI, Position FSM Status Column.

---

## 85. DEEP AUDIT + 12 SMART FEATURES + STYLING (Session 19)

In session 19, a deep audit of the entire HFT Engine tab was performed, 12 new functionalities ("Smart Features") were added, the styling was completely refactored, and the SAFE/SCALP presets were extended to all sliders (~90+).

### 85.1 Emergency Kill Dashboard

Sticky panel at the top of the HFT Engine tab with:
- **KILL ALL** — red button that calls `emergency_liquidate` on the backend, liquidates ALL positions and cancels ALL active orders. Need confirmation.
- **Breaker Status** — shows the current state of the Circuit Breaker (NORMAL / CAUTION / HALT / PANIC) with corresponding colors
- **Equity** — the current value of the account
- **Daily P&L** — daily profit/loss
- **Positions** — the number of open positions

The panel is updated automatically every 3 seconds.

### 85.2 DRY RUN / LIVE Badge

Badge displayed next to the title "HFT ENGINE CONFIG":
- **DRY RUN** (blue) — the bot simulates, does not execute real orders
- **LIVE** (green, pulsating) — the bot executes real orders on the market

The badge reads the real status from the backend and is synchronized through the `dryRunChanged` event on `TitanBus`.

### 85.3 SAFE DEFAULT / SCALP DEFAULT presets

Two buttons that automatically set ALL the sliders (~90+) in the tab to pre-calculated values:

**SAFE DEFAULT** — conservative parameters:
- Tighter stops, lower trailing stop
- Fewer simultaneous positions
- Lower trade risk
- Suitable for volatile markets or beginners

**SCALP DEFAULT** — aggressive parameters:
- Wider stops, trailing stop adapted to fast scalping
- More simultaneous positions
- Lower take-profit but higher frequency
- Suitable for markets with high liquidity and low volatility

Each preset has a **toggle lock** (switch on/off):
- When it is ON, the preset is "locked" — is automatically applied at startup and prevents manual changes to the sliders
- Only one preset can be locked at a given time
- The state of the lock is saved in `localStorage`

### 85.4 Search Filter

Search field "Search parameters..." which filters all the sliders in the tab in real time:
- Match on parameter name (case-insensitive)
- Hide mismatched sliders
- Hide completely empty sections
- Works on all containers (.mb-8, .hft-grid-2/3/4)

### 85.5 Undo (Persistent on Disk)

The **UNDO** button restores all settings to the state before the last change:
- Snapshot is saved to disk in `titan_settings_undo.json`
- Survives restart/crash
- Automatically created before applying a preset or a save

Rust commands: `save_undo_snapshot`, `load_undo_snapshot`

### 85.6 Settings Diff View

The **SHOW CHANGES** button displays a table with all unsaved changes:
- Columns: Parameter | Saved | Current
- Modified values are highlighted (red/green)
- Can be closed with the CLOSE button

### 85.7 Per-Symbol Overrides

Collapsible section that allows setting custom parameters per symbol:
- **Symbol** — enter the symbol (ex: AAPL, BTC/USD)
- **Trail%** — trailing stop custom
- **Take%** — take profit custom
- **Risk%** — risk on trade custom
- **SET** — apply the override
- **REMOVE** — delete the override

Overrides are persisted in `titan_settings.json` and respected by the autopilot during trading.

Rust commands: `set_symbol_override`, `remove_symbol_override`, `get_symbol_overrides`

### 85.8 Live Impact Preview (Effective Exit Values)

Panel showing the EFFECTIVE values of stop/take/trail AFTER applying the adaptive multipliers:
- **Stop Loss** — effective value vs. basis
- **Take Profit** — effective value vs. basis
- **Trailing** — effective value vs. base

It is automatically updated when the sliders are changed (debounced).

Rust command: `preview_effective_exits`

### 85.9 Regime Heatmap

Three colored boxes showing the multipliers for each market regime:
- **TRENDING** — market with clear trend (up/down)
- **RANGE** — lateral market, no clear direction
- **CHAOTIC** — chaotic market, extreme volatility

Each box shows: S (stop much), T (take much), Tr (trail much). The active mode is highlighted. It is updated every 5 seconds.

### 85.10 Parameter Profiles (RF-C: Disk Persistence)

Named profile system for saving/loading complete configurations. Profiles are stored on disk in `titan_profiles.json` (not in localStorage) -- they survive rebuilds, crashes, and can be exported/imported.

**Available actions:**
- **Save As** — save the current configuration (ALL parameters including TA Gates, Scan Params, Symbol Overrides) under a chosen name
- **Load** — load a saved profile and apply ALL settings (including nested objects)
- **Delete** — delete a profile (with cyber confirmation)
- **Export** — export the selected profile as a JSON file (for backup or sharing)
- **Import** — imports a profile from a JSON file (the name is taken from the file or from the text field)
- Dropdown with all saved profiles, sorted alphabetically

**Storage:** `titan_profiles.json` in the data folder of the application, with atomic writing (temp+rename) and auto-backup `.bak`. Versioning scheme (`schema_v: 1`) for compatibility with future updates.

**Automatic migration:** If there are old profiles in localStorage (from previous versions), they are automatically imported to disk at the first run and the old key is deleted.

### 85.11 Risk Calculator Live

Panel with 4 risk metrics calculated in real time:
- **Max Loss / Trade** — the maximum loss per transaction in dollars
- **Position Size** — the size of the position in dollars
- **Risk/Reward** — the risk/reward ratio
- **Max Daily Loss** — the maximum daily loss in dollars

It is updated automatically when the sliders are changed.

### 85.12 Parameter Health

Diagnostic system with 15 conflict/risk rules:
- Colored badges: **OK** (green), **WARN** (orange), **DANGER** (purple)
- Check: stop > take (ineffective), risk too high, Circuit Breaker too weak, etc.
- Updates in real time with any slider change

### 85.13 Auto-Suggest

The **ANALYZE TRADES** button analyzes the last 50 trades in the log and generates suggestions:
- Detects 7 patterns (too tight stops, ineffective trailing, etc.)
- Each suggestion has an **APPLY** button that automatically applies the adjustment
- Can be closed with the **CLOSE** button

### 85.14 Adaptive Exit Multipliers

Advanced section (collapsible) with 18 sliders + toggle + floor that control the adaptive multipliers of the outputs:

**Toggle: DISABLE ADAPTIVE EXITS** — when ON, all multipliers are ignored and only the raw values from the main sliders are used.

**Min Exit Floor %** — the minimum value to which stop/take/trail can be compressed after cascading.

**Regime Multipliers** (9 sliders) — separate multipliers for Stop/Take/Trail in each regime (Chaotic, Range, Trend).

**Signal Alignment Multipliers** (5 sliders) — multipliers for when the signals are aligned or opposite to the position.

**Timeframe Multipliers** (3 sliders) — multipliers for short timeframes (1m/3m).

### 85.15 Multi-Asset Control

- **Toggle** — enable/disable simultaneous trading on several assets
- **Max Concurrent Assets** — slider (1-100) that limits the number of assets the bot can trade simultaneously

### 85.16 Chart Persistence on Disk

Chart settings (timeframe, indicators, configurations) are now saved to disk:
- **Dual save**: `localStorage` (for quick toggle between tabs) + disk (`titan_chart_settings.json`)
- At startup, the graphic loads from disk with `await invoke('load_chart_settings')`
- Survives WebView cache clear (from build.sh)

Rust commands: `save_chart_settings`, `load_chart_settings`

### 85.17 Completely Refactored Styling

- Cyan unified sliders in all tab-4 (yellow/cyan inconsistency removed)
- Number/text/select inputs with dark background (white background removed)
- Text sizing extended to ALL sections
- Dark scrollbar, consistent borders, hover effects on sections
- Health badges with purple (danger) and yellow-orange colors (warn)
- `.btn-cmd` forced to `color: #ccc` in tab-4 (removed black text on dark background)
- LLM Sentiment Provider dropdown fixed (dark background)
- Duplicated SAVE CONFIG/RESET DEFAULTS buttons removed

### 85.18 HFT Engine dedicated manual

A separate user manual was created, exclusively dedicated HFT Engine tab:
- **File:** `TITAN_HFT_ENGINE_MANUAL.md`
- Detailed explanations for EVERY element in the tab
- Concrete numerical examples
- Default values, domains, recommendations
- Written for beginners

---

## 86. CYBER-HUD + ROSS CAMERON COLUMNS + FMP ENRICHMENT (Session 20)

### 86.1 Cyber-HUD Styling

All 3 tables in the center panel (HFT Targets, Signal CFG, Watchlist) have received the Cyber-HUD theme:
- Font: **Share Tech Mono 13px** (enlarged from 10-11px for readability)
- Action badges: `.tbl-badge` with gold/cyan/magenta/red variants
- Consistent color-coding on columns:
  - `CHG%`: cyan ≥10%, gold ≥0%, red <0%
  - `RVOL`: cyan ≥5x, gold ≥2x
  - `GAP%`: cyan >3%, gold >0%
  - `FLOAT`: violet=LOW, gold=MED
  - `SCORE`: cyan ≥60, gold ≥30
  - `INS`: cyan=BUY, red=SELL
  - `CAP`: auto format (150M / 1.2B)

### 86.2 Ross Cameron Columns Complete

The full set of columns is now available on all 3 tables:

| Column | Source | Description |
|---------|-------|----------|
| GAP% | Alpaca snapshots | Gap from open vs prev close |
| FLOAT | Estimated from volume | LOW (<3M vol), MED (3-10M), HIGH (>10M) |
| CAP | FMP /stable/profile | Market cap in millions (ex: 22M, 1.2B) |
| INS | FMP /stable/insider-trading | Last insider activity: BUY, SELL, or — |
| SCORE | AI scoring pipeline | Smart score or composite AI score |

### 86.3 FMP Enrichment Universal

Before this session, FMP data (market cap, insider signal) was populated **only** by scanning Market Discovery.

Now, FMP enrichment is integrated into **all** data pipelines:
- **HFT Targets → REFRESH** → fetch FMP profile + insider
- **Signal CFG → REFRESH** → fetch FMP profile + insider
- **Watchlist → REFRESH** → fetch FMP profile + insider

FMP data is **cache-forgotten** — it is not re-fetched at each refresh if it is already in the cache.

If the FMP API key is not set (in Settings → API Keys), the CAP and INS columns will show `—` (fallback graceful).

### 86.4 Cross-Module Refresh

When injecting symbols from Market Discovery:
- Signal CFG is updated automatically (`window.loadSignalAssets()`)
- HFT Targets is updated automatically (`refreshHftTargets()`)
- Manual refresh is no longer necessary after injection

### 86.5 Modified Rust files

- `scanner.rs` — `AiScanCandidate` extended struct with `market_cap` and `insider_signal`
- `commands/favorites.rs` — `WatchlistEntry` extended with `market_cap_m`, `insider_signal`, `float_category`
- `commands/hft_targets.rs` — FMP enrichment added
- `commands/signal.rs` — FMP enrichment added
- `commands/scoring.rs` — new fields on construct

---

## 87. PHASE 9 FINAL UPGRADES — POSITION FSM + LATENCY METRICS + WEBSOCKET DASHBOARD + SESSION ANALYTICS (Session 21)

87.1 Position State Machine (FSM)

### 87.1 Position State Machine (FSM)

The positions had an implicit lifecycle — their state was deduced from the context (presence/absence in the HashMap). Information such as the time of opening, hold duration, partial fills and reason for entry were dispersed in separate HashMaps on TitanState.

condition

| condition | Meaning |
|-------|-------------|
| `PendingEntry` | Order sent, waiting for fill |
| `Open` | Active position (default) |
| `PartialExit` | Partially executed |
| `PendingClose` | Closing order sent |
| `Closed` | Completely closed position |

**New fields on `Position` struct:**

| Field | Type | Default | Description |
|------|-----|---------|-----------|
| `state` | `PositionState` | `Open` | Current status of the FSM |
| `opened_at` | `Option<u64>` | `None` | Epoch seconds on opening |
| `closed_at` | `Option<u64>` | `None` | Epoch seconds at closing |
| `partial_taken` | `bool` | `false` | If the take was partially executed |
| `entry_tag` | `Option<String>` | `None` | Reason for entry (ex: "momentum_entry") |

All fields have `#[serde(default)]` — the previously saved state (without these fields) is deserialized correctly.

**FSM transitions:**
- **`autopilot.rs` entry** → `state: Open`, `opened_at: now`, `entry_tag: entry_type`
- **`trade_updates.rs` fill** → `state: Open`, `opened_at: now` (if it was `PendingEntry`)
- **`trade_updates.rs` partial adverse fill** → `state: PartialExit`, `partial_taken: true`
- **`trade_updates.rs` close fill** → `state: Closed`, `closed_at: now`
- **`autopilot.rs` exit** → `state: Closed`, `closed_at: now`
- **`trading.rs` liquidate/force-close** → `state: PendingClose`, `closed_at: now`
- **`position_recon.rs` goes** → preserve `state`, `opened_at`, `entry_tag`, `partial_taken` from local position

### 87.2 Micro-Latency Metrics

API latency (order round-trip, GPU dispatch, streamer tick processing) was only logged with `tracing::info!`. There is no aggregation or percentiles.

**Structure `LatencyMetrics`:**
- 3 ring buffers of 100 entries each (`Mutex<VecDeque<u64>>`):
  - `order_rtt_us` — API Alpaca round-trip (microseconds)
  - `gpu_dispatch_us` — GPU/WGPU dispatch time (microseconds)
  - `streamer_tick_us` — WebSocket SIP message processing (microseconds)

**Methods:**
- `snapshot() -> LatencySnapshot` — calculates percentiles on all 3 buckets
- `snapshot() -> LatencySnapshot` — calculates percentiles on all 3 buckets

**Structure `LatencySnapshot`:**

```rust
pub struct LatencySnapshot {
    pub order: LatencyBucket,
    pub gpu: LatencyBucket,
    pub streamer: LatencyBucket,
}
pub struct LatencyBucket {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub min: u64,
    pub max: u64,
    pub count: usize,
}
```

**Tauri command:** `get_latency_metrics` — returns `LatencySnapshot` as JSON for the frontend.

**Registration points:**
- `commands/order_exec.rs` — after each Alpaca API call
- `loops/nexus_loop.rs` and `commands/misc.rs` — after GPU dispatch
- `loops/trade_updates.rs` — in the WebSocket message processing hot loop

### 87.3 WebSockets Dashboard

The web dashboard (axum, port 7777) was using `<meta http-equiv="refresh" content="5">` for updating — not suitable for real-time monitoring.

**Implementation (updated S02 — Session 45):**
- Endpoint `GET /ws` with `WebSocketUpgrade` (axum 0.7, feature `ws`)
- Authentication via `Authorization: Bearer <token>` on HTTP upgrade request (unified with HTTP API — S02)
- `tokio::sync::broadcast::Sender<String>` on `TitanState` (256 slots)
- Push **status** every 2 seconds: equity, positions with FSM state, circuit breaker status, latency p50/p95
- Push **trade events** from autopilot: entry, exit (with PnL and duration), partial exit
- Push **alert events** from watchdog: loop recovery notifications
- Inline JavaScript client with auto-reconnect (exponential backoff 1s → 30s max)
- Completely removed `<meta http-equiv="refresh">` — 0 polling, 100% push
- Completely removed `<meta http-equiv="refresh">` — 0 polling, 100% push

**WebSocket Message Types (JSON):**

| type | When | Fields |
|------|------|---------|
| `status` | In 2 seconds | equity, cash, positions[], breaker, latency |
| `trade` | At entry/exit/partial | action, symbol, side, pnl, duration_secs |
| `alert` | At watchdog recovery | messages |

### 87.4 Session Analytics Improvements

The journal records the duration of the trades as `duration_secs: 0`. Partial exits were not journalized at all. There were no streak counters.

**New fields on `SessionAnalytics` (crate `titan-risk`):**

| Field | Type | Description |
|------|-----|-----------|
| `longest_win_streak` | `u32` | Longest streak of consecutive wins |
| `longest_lose_streak` | `u32` | The longest streak of consecutive losses |
| `current_streak` | `i32` | Current Streak (+ = wins, - = losses) |
| `avg_duration_secs` | `f64` | Average duration of trades (rolling) |
| `best_trade_pnl` | `Decimal` | The highest profit from a trade |
| `worst_trade_pnl` | `Decimal` | The biggest loss in a trade |

**Method `record_trade_with_duration(pnl, duration_secs)`:**
- Update streak counters with each trade: increment `current_streak` (+1 win / -1 loss), reset when changing direction
- Update `longest_win_streak` / `longest_lose_streak` to the new registration
- Update `best_trade_pnl` / `worst_trade_pnl`
- Calculate `avg_duration_secs` as rolling average

**Partial exit journaling:**
- The autopilot now creates an entry `TradeRecordInput` with `reason: "PARTIAL_TAKE"` for each partial exit
- Fully journaled with symbol, side, qty, pnl, duration

**Exposed in API:**
- `get_risk_status` returns the new fields
- Dashboard WebSocket includes streaks in push status

### 87.5 Post-Implementation Clippy Fixes

3 clippy errors detected by `build.sh` (running `cargo clippy -- -D warnings`):

1. **`derivable_impls`** in `app_state.rs` — `impl Default for PositionState` manual replaced by `#[derive(Default)]` + `#[default]` on the `Open` version
2. **`dead_code`** in `app_state.rs` — `Position::new_open()` marked with `#[allow(dead_code)]` (public API, will be used in the future)
3. **`bind_instead_of_map`** in `state.rs` — `.and_then(|x| Some(y))` replaced by `.map(|x| y)` on the `genetic_champion` calculation

### 87.6 Amended Files — Full Summary

**Backend Rust (12 files):**
- `app_state.rs` — `PositionState` enum, new fields on Position, `new_open()`, derives
- `state.rs` — `LatencyMetrics`, `LatencySnapshot`, `LatencyBucket`, `ws_broadcast`, fixed `.map`
- `trading_engine.rs` — FSM transitions to manage_positions
- `loops/trade_updates.rs` — FSM transitions to fills, latency recording
- `loops/position_recon.rs` — FSM state preservation at reconciliation
- `loops/autopilot.rs` — FSM reads/writes, WS broadcast, journal partial exits, `record_trade_with_duration`
- `loops/watchdog_loop.rs` — WS broadcast alert events
- `commands/trading.rs` — FSM `PendingClose` to liquidate/force-close
- `commands/order_exec.rs` — latency recording
- `commands/misc.rs` — GPU latency recording
- `commands/monitoring.rs` — `get_latency_metrics` order, new fields in `get_risk_status`
- `web_dashboard.rs` — `/ws` endpoint, WS handler, JS client, removed refresh polling
- `Cargo.toml` — `axum = { version = "0.7", features = ["ws"] }`
- `Cargo.toml` — `axum = { version = "0.7", features = ["ws"] }`
- `cargo check`: 0 errors

### 87.7 Validations

- `cargo check`: 0 errors
- `cargo clippy -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 122 tests passed
- Build release: 15M binary
- The application starts and works correctly

---

## Head. 88 — Session 23: FSM Status Column, Latency Panel, Rate Limiting, Watchdog Fix

**Date:** 29 March 2026 (Session 23)

This session added visibility in the UI for position status and latency, secured the web dashboard with CORS and rate limiting, and fixed a critical gap in the watchdog.

### 88.1 Position FSM Status Column (RF-A1)

The table of positions in the "CURRENT PORTFOLIO" section; now has a **STATUS** column between TYPE and MODE that shows the FSM (Finite State Machine) status of each position.

**5 possible states with colored badges:**

| condition | Badge | Color | When it appears |
|-------|-------|---------|------------|
| `PendingEntry` | PENDING | Yellow with pulse animation | Entry order sent, waiting for fill |
| `Open` | OPEN | Cyan | The position is active and open |
| `PartialExit` | PARTIAL | Magenta | A partial take profit was executed |
| `PendingClose` | CLOSING | Red with pulse animation | Closing order sent |
| `Closed` | CLOSED | Gray | Position closed (rarely visible, transient) |

**How ​​the status reaches the frontend:**
- `PositionView` (struct sent by `get_account_snapshot`) has been expanded with field `fsm_state: String`
- At each refresh, the backend does a lookup in `settings.positions` for the FSM internal state of each symbol
- The frontend maps the string to the corresponding CSS badge

**Why it's useful:** Before, you didn't have visibility if an order was being executed or if a partial exit had already been made. Now you can instantly see the real status.

### 88.2 Latency Metrics Panel (RF-A2)

In the rightbar of the chart (the section on the right, under "Time"), a new section **"Latency (us)"** has appeared that shows the real-time performance of the 3 critical channels.

**4x4 grid with 3 channels x 3 percentiles:**

| Channel | What measure | Where is it registered? |
|-------|------------|----------------------|
| **Order** | Round-trip time of Alpaca API orders | `commands/order_exec.rs` |
| **GPU** | Nexus GPU dispatch duration | `loops/nexus_loop.rs` |
| **Stream** | WebSocket streamer message processing | `streamer.rs` |

**Percentiles:** p50 (median), p95 (95th percentile), p99 (99th percentile)

**Automatic color coding:**
- **Cyan** (green) — under 1ms (1000us) — all OK
- **Yellow** — 1ms-5ms — acceptable performance but to be monitored
- **Red** — over 5ms — latency problems, investigate

**Smart format:** Values ​​are displayed as `us` (microseconds), `ms` (milliseconds), or `s` (seconds) depending on the magnitude.

**Polling:** The data is automatically updated every 5 seconds via `invoke('get_latency_metrics')`.

**The backend was already implemented** (session 21 — `LatencyMetrics` with ring buffer of 100 entries per channel). This session only added the frontend.

### 88.3 Rate Limiting + CORS (Phase 1 Security)

Web dashboard and Telegram have received security protections.

**Web Dashboard — CORS (updated S02 — Session 45):**
- `tower_http::cors::CorsLayer` restrictive added on the axum router
- **Allow Origin:** `http://127.0.0.1:<port>` and `http://localhost:<port>` (includes port — fixed S02)
- **Allow Methods:** GET and POST
- **Allow Headers:** Authorization and Content-Type
- A malicious site opened in the browser can no longer fetch at `http://127.0.0.1:7777/api/status`

**Web Dashboard — Rate Limiting (updated S02 — Session 45):**
- Sliding window of 60 requests per minute on all protected routes, including `/ws` upgrade (fix S02)
- Custom middleware with `Arc<Mutex<VecDeque<Instant>>>`
- When exceeded, returns HTTP 429 (Too Many Requests) with JSON error
- WS message-level rate limit: max 10 client messages per tick (2s), disconnect when exceeded (S02)

**Telegram — Rate Limiting:**
- Global sliding window of 20 messages per minute on `send_telegram_alert`
- `LazyLock<Mutex<VecDeque<Instant>>>` — static, shared between all threads
- When exceeded, the message is dropped silently with `tracing::warn`
- Prevents Telegram API flooding in situations of cascading alerts (ex: 10 positions closed simultaneously)

### 88.4 Watchdog Streamer Spawn Fix (Phase 2)

**Problem:** The streamer was registered in watchdog (`heartbeat("streamer", 0)`) but does NOT exist in `spawn_loop_by_name`. If the streamer task panicked or stopped, the watchdog detected it as dead but **couldn't restart it**.

**Fix:** Added branch `"streamer"` in `spawn_loop_by_name`:
- Create new channel (`tokio::sync::mpsc::unbounded_channel::<StreamCmd>()`)
- Replace `stream_tx` in states with the new sender
- Spawn `run_streamer` with the new receiver

**Why it's important:** The streamer is the most critical component — without it, you have no real-time price data. If it fell and was not restarted, the whole bot became "blind".

### 88.5 N1 Cleanup — println! Residuals

The last 2 `println!` of the production code (from `commands/settings.rs`) have been migrated to `tracing::info!`:
- Line 390: `fmp_api_key` update log — now `tracing::info!(component = "settings", ...)`
- Line 433: `batch_update_settings` count — now `tracing::info!(component = "settings", count = count, ...)`

Zero `println!` remain in production code. All logs go through `tracing` (structured logging with components, level filtering).

### 88.6 Modified Files — Full Summary

**Backend Rust (5 files):**
- `commands/settings.rs` — N1 cleanup: println -> tracing::info
- `commands/account.rs` — `PositionView` + `fsm_state` field, lookup from internal FSM
- `web_dashboard.rs` — CORS layer, rate limiting middleware (sliding window 60 req/min)
- `telegram.rs` — send rate limiter (20 msg/min) with LazyLock
- `loops/watchdog_loop.rs` — branch "streamer" in spawn_loop_by_name with channel recreation

**Frontend (4 files):**
- `titan_positions.js` — FSM badge rendering (map state -> label + CSS class)
- `titan_rightbar.js` — latency panel: polling, formatting, color coding
- `index.html` — STATUS column header, latency grid HTML section
- `styles.css` — FSM badge classes (5 states), latency grid styles, color states

### 88.7 Validations

- `cargo check`: 0 errors
- `cargo clippy -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 122 tests passed (37 risk + 30 indicators + 21 genetic + 29 main + 5 integration)
- Linter errors: 0
- FSM Status Column: checked visually — OPEN cyan badges displayed correctly

---

## Chap. 89 — Session 24: Ring Buffer + ArcSwap Lock-Free + Flash Crash Detector

Session 24 implemented 3 major components from BLOCK 2 of the audit plan, focused on **hot-path performance** and **capital protection in extreme events**.

### 89.1 S1: CandleRingBuffer — Market Data In-Memory

**Problem:** The autopilot calls `fetch_bars_internal` (REST HTTP to Alpaca) at each iteration, for each candidate symbol. This means 50-500ms latency per call, multiplied by the number of symbols.

**Solution:** `CandleRingBuffer` — circular buffer in memory per symbol, fed directly from WebSocket.

**New file:** `src-tauri/src/candle_buffer.rs`

**Architecture:**

```rust
pub struct CandleRingBuffer {
    candles: VecDeque<Candle>,   // max 200 candles finalizate
    current: Option<Candle>,     // candle in constructie (nu inca inchisa)
    tf_secs: i64,                // durata timeframe-ului in secunde
}
```

**Data flow:**
1. Streamer receives trade tick from WebSocket (price, size, timestamp)
2. Call `ring_buffer.update_trade(price, size, ts)`:
   - If `current` is `None` or the timestamp exceeds the current window → complete the current candle, move it to `candles`, create a new one
   - Otherwise, update OHLCV in-place (high = max, low = min, close = price, volume += size)
3. When `candles.len() > 200` → evicts the oldest (front pop)

**Autopilot integration — `get_candles()` helper:**

```rust
async fn get_candles(state: &Arc<TitanState>, symbol: &str, timeframe: &str, min_candles: usize)
    -> Result<Vec<Candle>, TitanError>
```

- **Fast path:** Search in `state.candle_buffers` for the key `"SYMBOL|TF"`. If it has >= `min_candles` → return the snapshot directly (~0ms)
- **Fallback:** If the buffer does not exist or has too few candles → call `fetch_bars_internal` (REST) ​​and seed the result in the buffer for the following calls

All 7 calls `fetch_bars_internal` from the autopilot were redirected through `get_candles()`.

**Utility `timeframe_to_secs()`:** Converts strings ("1m", "1Min", "5m", "5Min", "15m", "1h", "1Hour", "1d", "1Day") to seconds.

**Unit tests (7 new):**
- seed + snapshot correctness
- OHLCV update in-place (high/low/close/volume)
- boundary cross (finalization candle + new candle)
- eviction at max capacity (200)
- sort ascending by timestamp
- zero-price trade ignored
- timeframe string conversion

**Performance impact:** Hot path from 50-500ms (REST) ​​to <1ms (memory read).

### 89.2 S3: ArcSwap Lock-Free on OfiState

**Problem:** `ofi_state` was protected by `Arc<Mutex<OfiState>>`. The streamer (write at each tick) and the autopilot (read at each iteration) were fighting on the same lock, creating contention on the most latency-sensitive path.

**Solution:** `arc_swap::ArcSwap<OfiState>` — atomic swap lock-free.

**Before:**
```rust
pub ofi_state: Arc<Mutex<OfiState>>,
// Write: state.ofi_state.lock().unwrap().current_ofi = val;
// Read:  let ofi = state.ofi_state.lock().unwrap().clone();
```

**After:**
```rust
pub ofi_state: arc_swap::ArcSwap<OfiState>,
// Write: let prev = state.ofi_state.load();
//        let mut st = (*prev).clone();
//        st.current_ofi = val;
//        state.ofi_state.store(Arc::new(st));
// Read:  let ofi = state.ofi_state.load();  // Guard<Arc<OfiState>>, no lock
```

**Updated write paths (streamer.rs):**
1. **Book-path OFI** — when receiving L2 deltas from WebSocket, recalculates OFI and makes atomic store
2. **Quote-path OFI** — when it receives NBBO quotes, it updates best_bid/ask/spread and makes atomic store

**Read paths updated (3 locations):**
1. `autopilot.rs` — spread guard + OFI stale check
2. `commands/order_exec.rs` — smart limit pricing (midpoint calculation)
3. `commands/strategy_cmd.rs` — `get_ofi_status` command for frontend

**Conservative approach:** ONLY migrated `ofi_state` because it has a clear single-writer pattern (the streamer). `last_prices`, `trail_*` and others remain `Mutex` (multi-writer patterns).

**Impact:** Zero contention on read path; write is O(1) atomic pointer swap.

### 89.3 S7: Flash Crash Detector with Auto-Protection

**Problem:** The existing circuit breaker only measures your PnL (losses from positions). It does not detect abnormal market movements (flash crashes) that have not yet hit the stop.

**Solution:** `FlashCrashDetector` — rolling window price monitoring per symbol.

```rust
pub struct FlashCrashDetector {
    windows: HashMap<String, VecDeque<(Instant, f64)>>,  // preturi recente per simbol
    blocked_until: HashMap<String, Instant>,               // cooldown per simbol
    window_duration: Duration,   // fereastra de monitorizare (default 30s)
    threshold_pct: f64,          // prag de declansare (default 3.0%)
    cooldown: Duration,          // blocare post-trigger (default 5min)
}
```

**Detection logic:**
1. At each trade tick, the price is added to the symbol window
2. Clearing prices older than `window_duration`
3. `min` and `max` are calculated from the window
4. If `(max - min) / min * 100 > threshold_pct` → FLASH CRASH DETECTED

**5 automatic actions when triggered:**
1. `tracing::warn!(component = "flash_crash", symbol, drop_pct, ...)` — structured log
2. `app.emit_all("flash-crash", { symbol, drop_pct, action })` — event to the frontend
3. Auto-close the position on the affected symbol — spawn `close_position(state, symbol, qty, side, app)`
4. `send_telegram_alert(...)` — Telegram notification (if enabled)
5. Block new entries on the symbol for `cooldown_mins` minutes

**3 configurable parameters from the UI (AppState + AppSettings):**

| Parameter | Default | Range | What it does |
|-----------|---------|-------|---------|
| `flash_crash_window_secs` | 30 | 5-120 | Monitoring window in seconds |
| `flash_crash_threshold_pct` | 3.0 | 0.5-20.0 | Trigger percentage threshold |
| `flash_crash_cooldown_mins` | 5 | 1-60 | How many minutes does the post-trigger symbol block |

All 3 have `#[serde(default)]` for backward compatibility with previously saved settings.

**Autopilot integration:** At the beginning of the evaluation of each candidate symbol, the autopilot checks `state.flash_crash.lock().is_blocked(candidate)` and does `continue` if the symbol is blocked.

**Unit tests (6 new):**
- normal trades under threshold → no event
- crash detection with price drop > threshold
- spike up detection (symmetric — also detects abnormal increases)
- cooldown enforced post-trigger
- window expiry (old prices leave the window)
- clear() resets everything

### 89.4 Modified files — Full Summary

**New file (1):**
- `src-tauri/src/candle_buffer.rs` — CandleRingBuffer + FlashCrashDetector + timeframe_to_secs + 13 unit tests

**Backend Rust (8 files changed):**
- `main.rs` — `mod candle_buffer`, initialize `candle_buffers` and `flash_crash` on TitanState
- `state.rs` — import CandleRingBuffer/FlashCrashDetector, new fields on TitanState, 3 flash crash fields on AppSettings + defaults + sync functions, ofi_state migrated to ArcSwap
- `app_state.rs` — 3 flash crash fields on AppState + defaults
- `streamer.rs` — candle buffer update on trade, flash crash check on trade (auto-close + telegram), ArcSwap writes on both OFI paths
- `loops/autopilot.rs` — `get_candles()` helper, 7 redirects, ArcSwap read, flash crash block gate
- `commands/order_exec.rs` — ArcSwap read on OFI
- `commands/strategy_cmd.rs` — ArcSwap read on OFI
- `commands/settings.rs` — 3 flash crash settings handlers with clamping

### 89.5 Validations

- `cargo check`: 0 errors
- `cargo clippy -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 135 tests passed (37 risk + 30 indicators + 21 genetic + 42 main + 5 integration)
- 13 new tests (7 candle_buffer + 6 flash_crash), 0 failed
- Linter errors: 0

---

## Ch. 90 — Session 25: Drawdown Velocity Detection + Keyboard Command Palette

### 90.1 S8: Intraday Drawdown Velocity Detection

**Problem solved:** The circuit breaker measures the magnitude of the loss (daily PnL < threshold). The Flash Crash Detector measures the movement of the market. But none reacts to the **speed** of the loss. The dangerous scenario: you have -0.8% daily PnL (under the halt threshold of -1%), but in the last 3 minutes you lost 0.6% — cascading losses in progress. The circuit breaker does not trip yet, but the speed shows that something is fundamentally wrong.

**Solution:** Ring buffer of drawdown snapshots with velocity detection.

**How it works:**
1. At each autopilot cycle (~1.5s), a snapshot `(epoch_secs, drawdown_pct)` is recorded in the ring buffer on `SessionAnalytics`
2. The ring buffer retains a maximum of 120 entries (the last ~3 minutes at normal refresh)
3. Calculate `dd_velocity` = drawdown change in %/min on the window configured
4. If the speed exceeds the threshold (default -1.0 %/min), new entries are blocked — **independent** of the circuit breaker
5. Log message periodically in UI: "DD VELOCITY HALT — Drawdown speed X.XX%/min exceeds threshold"

**Configurable settings (HFT Engine tab):**

| Parameter | Default | Range | Description |
|-----------|---------|---------|-----------|
| `dd_velocity_threshold` | -1.0 | -10.0 ... -0.1 | Maximum drawdown speed in %/min before halt |
| `dd_velocity_window_secs` | 300 | 30 ... 600 | Lookback window in seconds |

**Difference to Circuit Breaker:**

| Appearance | Circuit Breaker | DD Velocity |
|--------|----------------|-------------|
| Measure | Magnitude (daily PnL %) | Rate of loss (%/min) |
| Trigger | PnL below absolute threshold | Rate of loss over the threshold |
| When it helps | Large accumulated losses | Rapid losses in the cascade |
| Recovery | Manual reset | Automatic when the speed decreases |

**Concrete example:** Equity $100,000. In 3 minutes you lose 5 consecutive trades, equity drops to $97,000. Drawdown = 3%. Speed ​​= -1.0%/min. The default threshold (-1.0) is reached → blocked inputs BEFORE the circuit breaker activates at -5%.

**Exposed in API:** `get_risk_status` returns the field `dd_velocity` (f64, negative when drawdown deepens).

**Tests:** 7 unit tests — no_data, stable_equity, cascading_loss, safe_threshold, ring_eviction, window_secs, positive_threshold.

### 90.2 S11: Keyboard Shortcut Command Palette

**Problem solved:** In active scalping, every second counts. Navigating with the mouse through the tabs, changing the timeframe or quickly placing orders consumes precious time.

**Solution:** Global keyboard shortcuts module with integrated command palette (Ctrl+K).

**Direct shortcuts (works anytime):**

| Shortcut | Action |
|----------|---------|
| `Ctrl+K` | Open Command Palette |
| `Ctrl+Enter` | Quick Buy (market order on the current symbol) |
| `Ctrl+Shift+Enter` | Quick Sell (market order on the symbol current) |
| `Ctrl+Shift+X` | Flatten All — liquidate all positions (emergency) |
| `Ctrl+P` | Toggle Autopilot on/off |
| `Ctrl+[` | Smaller timeframe (ex: 5m → 3m → 1m) |
| `Ctrl+]` | Larger timeframe (ex: 5m → 10m → 15m) |

**Command Palette (Ctrl+K):**
- Glassmorphism overlay with blur, Share Tech Mono font, neon cyan/green styling
- Search input with **fuzzy matching** on 19 commands
- Navigation with **Arrow Up/Down + Enter** or click
- **Escape** closes the palette
- If the entered text does not match any command, **Enter** it treat as a symbol and switch the chart to that symbol
- Categories: Trade (3), Control (1), Navigate (8), Chart (7)

**Commands available in the palette:**

| Category | Order | Shortcut |
|-----------|---------|----------|
| Trade | Quick Buy (market) | Ctrl+Enter |
| Trade | Quick Sell (market) | Ctrl+Shift+Enter |
| Trade | Flatten All Positions | Ctrl+Shift+X |
| Control | Toggle Autopilot | Ctrl+P |
| Navigate | Go to Chart | 1 |
| Navigate | Go to News AI | 2 |
| Navigate | Go to Journal | 3 |
| Navigate | Go to HFT Engine | 4 |
| Navigate | Go to Performance | 5 |
| Navigate | Go to AI Scanner | 6 |
| Navigate | Go to Watchlist | 9 |
| Navigate | Go to HFT Targets | 0 |
| Chart | Timeframe Higher | Ctrl+] |
| Chart | Timeframe Lower | Ctrl+[ |
| Chart | Timeframe → 1m/5m/15m/1h/1d | — |

**Compatibility:** Ctrl+* shortcuts do not interfere with the chart hotkeys (L=line, R=ray, H=hline, etc.) that work only without Ctrl. Navigating through tables with numbers (1-9, 0) is available ONLY through the command palette, not directly as hotkeys (avoids conflict with `0` = chart reset).

**Input guard:** All shortcuts (except Ctrl+K) are disabled when the focus is on a `<input>`, `<textarea>` or `contentEditable` element. Ctrl+K always works — opens the palette and moves the focus to the palette input.

### 90.3 Changed Files — Full Summary

**New file (1):**
- `src/titan_keyboard.js` — Keyboard shortcuts + Command Palette (230 lines)

**Backend Rust (6 modified files):**
- `crates/titan-risk/src/lib.rs` — dd_velocity_ring, record_dd_snapshot, compute_dd_velocity, check_dd_velocity_halt + 7 tests
- `app_state.rs` — dd_velocity_threshold, dd_velocity_window_secs fields + defaults
- `state.rs` — AppSettings fields, default functions, settings_from_state, apply_settings
- `commands/settings.rs` — handlers dd_velocity_threshold, dd_velocity_window_secs
- `commands/monitoring.rs` — dd_velocity in get_risk_status JSON
- `loops/autopilot.rs` — record_dd_snapshot every cycle, dd_velocity_halt check, entries_allowed gate

**Frontend (2 modified files):**
- `index.html` — `<script src="titan_keyboard.js">`
- `styles.css` — Command Palette CSS (~100 lines)

### 90.4 Validations

- `cargo check`: 0 errors
- `cargo clippy -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 142 tests passed (44 risk + 30 indicators + 21 genetic + 42 main + 5 integration)
- 7 new drawdown velocity tests, 0 failed
- BLOCK 2 of the master plan: complete (S1, S3, S7, S8, S11 + watchdog streamer)

---

## Ch. 91 — Session 26: TA Entry Gates with Soft/Hard Mode (RF-B)

### 91.1 What are TA Entry Gates

TA Entry Gates are **6 technical pre-entry filters** that are applied AFTER the Oracle decides Buy/Short, but BEFORE returning the final trading decision. Their purpose is to filter the suboptimal inputs that Oracle alone misses.

Each gate has 3 modes of operation:

| Mode | Effect |
|-----|-------|
| **Off** | The Gate is completely ignored (as if it does not exist) |
| **Soft** | Entry is allowed, but the score is penalized with N points (default 15). If the score falls below the threshold, the entry is still not executed |
| **Hard** | The entry is completely blocked — `check_entry()` returns `None`, regardless of the Oracle score |

### 91.2 The 6 Gates

| Gate | What it checks | When block/penalize |
|------|------------|---------------------------|
| **RSI** | RSI vs thresholds overbought/oversold | Long: RSI > `rsi_overbought` (default 70). Short: RSI < `rsi_oversold` (default 30) |
| **EMA Alignment** | Order EMA9/20/50 | Long: need EMA9 > EMA20 > EMA50. Short: EMA9 < EMA20 < EMA50. If they are not aligned, the gate is activated |
| **ADX** | Trend strength | ADX < `adx_trend_threshold` (default 25) — trend too weak |
| **BB Squeeze** | Width of Bollinger bands | If `(BB_upper - BB_lower) / BB_mid < 2%` — market consolidating, avoid entries |
| **MACD** | MACD histogram direction | Long: the histogram must > 0. Short: the histogram must be < 0. If it is not aligned with the direction of the trade, the gate is activated |
| **Stoch RSI** | Extreme StochRSI | Long: StochRSI > 80 (already overbought). Short: StochRSI < 20 (already oversold). Avoid entries when the indicator is in the extreme opposite direction |

### 91.3 Presets (Predefined Profiles)

| Preset | RSI | EMA | ADX | BB | MACD | StochRSI | When to use it |
|--------|-----|-----|-----|----|------|----------|-------------------|
| **Strict** | Hard | Hard | Hard | Hard | Hard | Hard | Maximum quality, few inputs. Good for volatile markets |
| **Balanced** | Soft | Off | Soft | Off | Off | Off | Compromise: RSI and ADX filter but not block |
| **Oracle Only** | Off | Off | Off | Off | Off | Off | Default. The Oracle decides by itself, without additional filters |

### 91.4 Soft Penalty (Adjustable Penalty)

When a gate in Soft mode is violated, the score of the trade decreases by `ta_gate_soft_penalty` points (default 15, configurable 1-50 from the UI).

Penalties are cumulative: if 3 Soft gates are violated simultaneously, the score decreases by 45 points (3 x 15). A low enough score can make the trade not pass the entry threshold, even if it is not explicitly blocked.

### 91.5 Fail-Open Design

If an indicator is `None` (not enough data — for example, RSI is None on the first 14 candles), that gate is **completely skipped**. It does not block or penalize.

The reason: at the beginning of the trading session or after a restart, the candle buffer is small. If the gates would block in the absence of data, you would lose the first minutes of the market open — the most volatile and profitable.

### 91.6 Place in the Pipeline

The complete pipeline `check_entry()`:

```
Candle-uri → FVG Detection → Oracle Score → VPIN Gate → RVOL Gate
  → Regime Thresholds → Long/Short Decision
  → TA Gates (NOU) → Classical Bonus → Return TradeDecision
```

### 91.7 Configuration from UI

In the HFT Engine tab, the **"TA ENTRY GATES"** section (with orange background) contains:
- **Preset dropdown** (top right): Custom / Strict / Balanced / Oracle Only
- **6 selectors** Off/Soft/Hard — one per gate
- **Slider Soft Penalty** — the penalty in points per violated Soft gate (1-50)

The changes are applied **live** (when changing each select) and automatically saves to disk.

### 91.8 Validations

- `cargo check`: 0 errors
- `cargo clippy -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 158 tests passed (44 risk + 30 indicators + 21 genetic + 58 main + 5 integration)
- 16 new unit tests TA gates, 0 failed, 0 regressions

---

## Chap. 92 — Session 27: StreamingIndicators — O(1) Incremental Calculation (S2+S4 Part 1)

### 92.1 What is StreamingIndicators

`StreamingIndicators` is an internal module in the `titan-indicators` crate that maintains the state of each technical indicator and can produce a complete snapshot of indicators by processing a single candle (O(1)), instead of recalculates all indicators from zero for 200 candles (O(n)) at each cycle.

This is an internal optimization. **Does not change any visible behavior** — same indicators, same values, same trading decision. It's just that it's calculated more efficiently.

### 92.2 Why it is necessary

The autopilot calls `calculate_all_with_periods()` every cycle (500-1500ms), for each candidate symbol. This function recalculates from scratch:
- 4 EMA series (9, 20, 50, 200)
- RSI (Wilder smoothing)
- MACD (3 internal EMAs)
- Bollinger Bands (sliding window + std dev)
- ADX (4 rolling means)
- VWAP (cumulative)
- StochRSI (ring buffer + 2 SMA)
- Oracle 7-layer (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD)

From all these calculations, only the LAST value matters (the current snapshot). The first 199 candles are recalculated unnecessarily. In addition, in 95%+ of the cycles the candles are identical to the previous cycle (a 1-minute candle changes once a minute, but the autopilot checks 60-120 times).

### 92.3 How it works

`StreamingIndicators` maintains persistent states per pointer:

| Indicator | Internal state | Update complexity |
|-----------|-------------|---------------------|
| EMA (x4) | Previous EMA value + precomputed alpha | O(1) |
| RSI | avg_gain, avg_loss (Wilder smoothing) | O(1) |
| MACD | 3 internal EMAs (fast, slow, signal) | O(1) |
| BB | Sliding window VecDeque (20 values) | O(period) ~20 ops |
| ADX | 4 sliding window SMA (TR, +DM, -DM, DX) | O(period) ~14 ops |
| VWAP | Cumulative amounts (volume, volume*price) | O(1) |
| StochRSI | Ring buffer RSI + 2 SMA(3) | O(period) ~14 ops |

**Important design decision:** BB and ADX use sliding window (`VecDeque`) instead of online algorithms (Welford). The reason: the existing batch uses sample variance `(N-1)` and rolling SMA. Sliding window guarantees identical output at the bit level, without the risk of cumulative floating point errors.

### 92.4 API Public

| Method | What it does |
|--------|---------|
| `StreamingIndicators::new(periods)` | Creates the empty instance, ready for `update()` |
| `StreamingIndicators::cold_start(candles, periods)` | Processes an initial batch of candles, initializes state |
| `streaming.update(candle)` | Process a new candle, return `Indicators` snapshot |
| `streaming.candle_count()` | Total number of candles processed |

### 92.5 Numeric validation

The output `StreamingIndicators` has been validated against batch `calculate_all_with_periods()` with proptest on random sequences of 50-200 candles. All values ​​are identical to epsilon 1e-10 (0.0000000001).

### 92.6 Oracle

The Oracle (7-layer predictive super-indicator) was NOT incrementalized in session 26. `update()` returns `oracle_score = None`. Oracle is called batch only for new candle (once per minute at TF 1m), not every cycle. **Note:** Online Oracle Weight Adaptation (S15) was implemented in Session 38 — see Chap. 103. This adjusts the weights, not the incremental calculation.

### 92.7 What's next (Part 2)

In the next session, `IndicatorCache` is implemented — a cache per symbol+timeframe that:
- At identical candle (95%+ of cycles): return cached, zero calculation
- At new candle: call `streaming.update()` (O(1)) + Oracle batch
- At first call: `cold_start()` from the complete batch

Estimated reduction: ~95-99% of the indicator calculation overhead.

### 92.8 Modified files

One file: `src-tauri/crates/titan-indicators/src/indicators.rs`

Zero changes in the main code (`src-tauri/src/`). Completely isolated implementation in crates.

### 92.9 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 170 tests passed (44 risk + 42 indicators + 21 genetic + 58 main + 5 integration)
- 12 new tests streaming indicators (3 proptest + 9 unit), 0 failed, 0 regressions

---

## Chap. 93 — Session 28: IndicatorCache — Hybrid Cache per Symbol+TF (S2+S4 Part 2)

### 93.1 What is

`IndicatorCache` is the module that integrates `StreamingIndicators` (Chap. 92) into the trading engine. Without it, StreamingIndicators exists isolated in crates with no real effect. With IndicatorCache, the autopilot no longer recalculates all indicators from scratch at each cycle.

### 93.2 Hybrid strategy

The cache uses a hybrid strategy with two components:

- **Classic indicators** (EMA, RSI, MACD, BB, ADX, VWAP, StochRSI): calculate incrementally via StreamingIndicators — O(1) per new candle
- **Oracle** (7 layers: Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD): recalculated batch only at the candle boundary (when the timestamp of the last candle changes)

### 93.3 The three execution paths

| Path | When it is activated | what is he doing | Cost |
|------|-------------------|---------|------|
| **Cache hit** | Same candle timestamp (~95% of cycles) | Returns the instant stored snapshot | Zero |
| **Cache miss** | New candle (different timestamp) | `streaming.update()` O(1) + `oracle::calculate()` batch | O(1) + O(n) oracles |
| **Cold start** | First call per symbol | `StreamingIndicators::cold_start()` + `calculate_all_with_periods()` batch | A(n) full |

At the cache hit, NO computation is done. The result is returned directly from memory.

### 93.4 Structure

```
IndicatorCache
  └─ entries: HashMap<"SIMBOL|TF", CacheEntry>

CacheEntry
  ├─ streaming: StreamingIndicators  (state incremental persistent)
  ├─ last_candle_ts: i64             (timestamp ultimul candle procesat)
  ├─ snapshot: Indicators            (rezultat complet: streaming + oracle)
  └─ last_access: Instant            (pentru eviction)
```

### 93.5 API

| Method | what is he doing |
|--------|---------|
| `IndicatorCache::new()` | Create empty cache |
| `cache.get_or_compute(symbol, tf, candles, periods, oracle_weights)` | Returns Indicators: cache hit, incremental update, or cold start |
| `cache.evict_stale(max_age)` | Delete entries not accessed more than `max_age` |

### In autopilot.rs, the call:

In autopilot.rs, the call:

```rust
let indicators = calculate_all_with_periods(&candles, &periods, &oracle_w);
```

was replaced by:

```rust
let indicators = {
    let mut cache = state.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
    cache.get_or_compute(candidate_symbol, &cand_tf, &candles, &periods, &oracle_w)
};
```

The eviction is done periodically (~1% of cycles, check order) with a maximum duration of 1 hour.

### 93.7 Design decision: cold start uses batch

At the first call per symbol, `calculate_all_with_periods()` is run completely (batch). `StreamingIndicators::cold_start()` only initializes the internal state. The reason: it guarantees BIT-IDENTICAL output with the non-cached path. After cold start, all subsequent calls use incremental streaming.

### 93.8 What NOT to do

- Oracle is NOT incrementalized (it remains batch, but it is called only once per new candle)
- `compute_strategy_score_with_mode()` (scoring dashboard) and entry check from misc.rs are not cached (pure functions without access to state)
- The backtest does not use the cache (it runs in isolation with different data)

### 93.9 Performance impact

| Metra | Before | After |
|---------|---------|------|
| Calculation indicator per cycle | O(200) candles * 14 indicators + Oracle 7 layers | Zero (cache hit 95%+) |
| At the new candle | O(200) full recalculated | O(1) streaming + O(n) Oracle (once) |
| On first call | O(200) full | O(200) full (identical, cold start) |

### 93.10 Files

- **Created:** `src-tauri/src/indicator_cache.rs`
- **Modified:** `main.rs` (mod + init), `state.rs` (field on TitanState), `autopilot.rs` (cache lookup + eviction)

### 93.11 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 errors, 0 warnings
- `cargo test --workspace`: 179 tests passed (44 risk + 42 indicators + 21 genetic + 67 main + 5 integration)
- 9 new indicator_cache tests, 0 failed, 0 regressions
- Test `cold_start_matches_batch`: all 19 Indicators fields identical to batch on 200 candles

---

## 94.1 What it is

### 94.1 What it is

Module `TitanAudio` adds procedural audio alerts, screen-edge glow and flash title on critical trading events. The goal: the user does not have to keep his eyes on the screen non-stop — sounds and visual effects alert him instantly when something important happens.

### 94.2 Problem solved

Before S12:
- Trade fills appeared only as text in the log
- Circuit breaker PANIC/HALT was only visible on the TITAN SAFETY badge
- **Flash crash** was emitted from Rust (`streamer.rs`) but **had NO listener in frontend** — critical safety gap
- Watchdog alerts were only log entries

After S12, all these events have sound distinctive + visual feedback (flash title and/or screen glow).

### 94.3 Procedural sounds (7 types)

All sounds are generated procedurally with the Web Audio API (no .wav/.mp3 files):

| Sound | Wave type | Frequencies | Duration | When played |
|-------|----------|-----------|--------|---------------|
| `fill-entry` | sine | 520→780→1040 Hz (up) | 200ms | At each trade fill |
| `fill-exit-profit` | sine+triangle | 880→1320→1760 Hz (cha-ching) | 300ms | Available programmatically |
| `fill-exit-loss` | sawtooth | 220→180 Hz (buzz) | 350ms | Programmatically available |
| `breaker` | square | 600+750 Hz, 3x pulse | 1s | Breaker CAUTION/HALT/PANIC |
| `flash-crash` | sawtooth+square | 440→330→220 Hz (descending) | 1s | Flash crash detected |
| `signal` | self | 1000+1200 Hz (ping) | 200ms | Programmatically available |
| `watchdog` | triangle | 400→500→400 Hz (triple) | 550ms | Watchdog dead loops |

### 94.4 Visual feedback

**Flash Title:** Alternate document.title at 500ms with descriptive message (ex: "FILL: BUY AAPL", "BREAKER PANIC"). Variable duration (2-6 seconds depending on severity). Useful when the Titan window is not in focus.

**Screen Glow:** Fullscreen overlay with animated `inset box-shadow` (3 pulses). Colors:
- **Red** (`rgba(255, 43, 43, 0.35)`) — PANIC/HALT breaker, 4 seconds
- **Yellow** (`rgba(255, 215, 0, 0.25)`) — CAUTION breaker, 2.5 seconds
- **Magenta** (`rgba(255, 0, 100, 0.4)`) — flash crash, 5 seconds

### 94.5 Toggle AUDIO ON/OFF

Button in the TITAN SAFETY panel header (next to the breaker badge). Persistent state in `localStorage` (key `titan_audio_enabled`). Default: ON.

Toggle synchronizes visually: cyan when ON, gray when OFF. A click on the button changes the status instantly.

### 94.6 Wire-up on Tauri events

| Event Tauri | Sound | Flash Title | Screen Glow |
|-------------|-------|-------------|-------------|
| `trade-fill` | fill-entry | "FILL: SIDE SYMBOL" (2.5s) | — |
| `breaker-triggered` (non-NORMAL) | breaker | "BREAKER LEVEL" (5s) | Red/Yellow |
| `flash-crash` | flash-crash | "FLASH CRASH: SYMBOL" (6s) | Magenta |
| `watchdog-alert` (dead>0) | watchdog | "WATCHDOG: N DEAD" (4s) | — |

Breaker with level=NORMAL (reset) does not emit sound — it is a reset, not an alarm.

### 94.7 Technical Details

- **Shared AudioContext** — a single reused instance (more efficient than the existing pattern of `new AudioContext()` per sound from `_playSignalAlertTone`)
- **Lazy init** — AudioContext is created at the first user gesture (click/keydown), required for Chrome/WebKit autoplay policy
- **Volume** — default 0.35, configurable via `TitanAudio.setVolume(v)` (range 0.05-1.0)
- 94.8 Files

### 94.8 Files

- **Modified:** `src/index.html` (script tag + AUDIO toggle button), `src/titan_bootstrap.js` (wireAudioAlerts step), `src/styles.css` (screen glow + toggle button)
- **Modified:** `src/index.html` (script tag + AUDIO toggle button), `src/titan_bootstrap.js` (wireAudioAlerts step), `src/styles.css` (screen glow + toggle button)

### 94.9 Public API

| Method | what is he doing |
|--------|---------|
| `TitanAudio.play(eventType)` | Play the specified sound |
| `TitanAudio.flashTitle(msg, ms)` | Toggle document.title |
| `TitanAudio.showScreenGlow(color, ms)` | Overlay fullscreen glow |
| `TitanAudio.toggle()` | ON↔OFF |
| `TitanAudio.setEnabled(bool)` | Direct setting |
| `TitanAudio.isEnabled()` | Query status |
| `TitanAudio.setVolume(v)` | Volume (0.05-1.0) |
| `TitanAudio.wireEvents()` | Login to Tauri events |

### 94.10 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 179 tests passed, 0 regressions
- 29 consecutive sessions without regressions

## Head. 95 — Session 30: TitanToasts — Unified Notification System (Phase 4)

### 95.1 What is it?

TitanToasts is a unified non-blocking notification system that completely replaces the browser's native `alert()` and `confirm()`. Instead of blocking the UI with native modal dialogs (which prevent any interaction until closed), notifications appear as slide-in toasts in the upper-right corner and disappear automatically.

### 95.2 Why was it necessary?

65 `alert()` calls completely blocked the application UI. In active scalping, a dialog blocker that appears after an order fill or an error can prevent the operator from reacting to another critical event. Also, 16 `confirm()` calls had the same blocker effect. Plus there were 4 different toast systems (each with inconsistent CSS and behavior).

### 95.3 showCyberToast(message, type, durationMs)

Displays a non-blocking notification in the upper-right corner of the screen.

**Available types:**

| Type | Color | Default duration | icona | When it is used |
|-----|---------|---------------|------|-------------------|
| `success` | Green (#00FF88) | 3 seconds | ✔ | Order executed, config saved, successful action |
| `error` | Red (#FF3344) | 6 seconds | ✖ | Order error, API error, critical failure |
| `warning` | Amber (#FFD700) | 5 seconds | ⚠ | Empty list, invalid input, attention |
| `info` | Cyan (#00BFFF) | 3 seconds | ℹ | Information, results, status |

**Behavior:**
- Slide-in from the right with animation, auto-dismiss after the specified duration
- Click X to manually dismiss
- Stack max 5 visible toasts simultaneously (oldest auto-removed)
- Glassmorphism (backdrop-filter: blur) consistent with cyber-HUD aesthetics

### 95.4 cyberConfirm(message, opts)

Replaces the native `confirm()` with a stylized modal. Returns `Promise<boolean>`.

**Features:**
- Overlay with background blur
- Centered box with dark gradient and cyan border
- CONFIRM (cyan) / CANCEL (gray) buttons
- **Keyboard:** Enter = confirm, Escape = cancel
- Click on the overlay (outside the box) = cancel
- Automatic focus on the CONFIRM button

**Usage example:**
```javascript
if (!(await cyberConfirm('Delete all drawings?'))) return;
// utilizatorul a confirmat, continua actiunea
```

### 95.5 What was replaced

**65 alert() replaced in 11 JS files:**
- titan_bootstrap.js (14), titan_watchlist.js (11), titan_ai_scanner.js (11)
- titan_positions.js (8), titan_scanner_plus.js (5), titan_manual_trade.js (5)
- titan_ls_settings.js (3), titan_sidebar.js (2), titan_replay_alerts.js (2)
- drawing_engine_v2.js (2), titan_rightbar.js (1)

**16 confirm() replaced in 11 JS files:**
- titan_nexus.js (3), titan_ls_settings.js (3), titan_perf_stats.js (2)
- titan_manual_trade.js (2), titan_bootstrap.js (1), titan_positions.js (1)
- titan_scanner_plus.js (1), titan_ai_scanner.js (1), titan_watchlist.js (1)
- titan_hft_panel.js (1), drawing_engine_v2.js (1)

### 95.6 Consolidation of existing toasts

| Old toast | What happened | Reason |
|------------|------------------|-------|
| `hftToast()` (30 uses) | Redirected to showCyberToast (thin wrapper with color map) | Same type of simple notification |
| `showNewMoverToast()` | Simplified to a single showCyberToast call | It was a simple notification with custom CSS |
| `_showSignalAlertToast()` | KEPT as such | Domain-specific: buy/sell/trail/chaos types, bottom-right positioning |
| `showCatalystToast()` | KEPT as such | Complex multi-field layout: symbol, headline, score, action buttons |

### 95.7 Files

- **Created:** `src/titan_toasts.js` (152 lines)
- **Modified:** `src/index.html` (script tag), `src/styles.css` (toast + confirm CSS, dead code cleanup), plus 14 JS files with alert/confirm replacements

### 95.8 Public API

| Function | what is he doing |
|---------|---------|
| `showCyberToast(msg, type, ms)` | Display non-blocking toast |
| `cyberConfirm(msg, opts)` | Async modal, returns Promise |
| `TitanToasts.show(msg, type, ms)` | showCyberToast equivalent (internal API) |
| `TitanToasts.confirm(msg, opts)` | CyberConfirm equivalent (internal API) |

### 95.9 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 179 tests passed, 0 regressions
- 30 consecutive sessions without regressions

---

## 96. PERSISTENCY PROFILES ON DISK (RF-C) — Session 31

### 96.1 What was done

The trading profiles were moved from `localStorage` (ephemeral, lost on rebuild) on disk to `titan_profiles.json`. Loading the profile restores ALL parameters including nested objects (TA Gates, Scan Params, Symbol Overrides) — the fix for an existing bug that filters objects on load.

### 96.2 New bulls orders (backend)

| Command | what is he doing |
|---------|---------|
| `save_profile(name)` | Complete AppSettings snapshot with metadata (schema_v, saved_at) |
| `load_profile(name)` | Apply full profile to AppState via apply_settings() |
| `delete_profile(name)` | Delete profile (graceful if it doesn't exist) |
| `list_profiles()` | Returns the sorted list with metadata (name, saved_at, schema_v) |
| `export_profile(name)` | Returns the complete JSON of the profile |
| `import_profile(name, data)` | Import profile from JSON |

### 96.3 Disk storage

- **File:** `titan_profiles.json` in data folder (`data/`)
- **Format:** `{ "v": 1, "profiles": { "scalping": { "schema_v": 1, "saved_at": "2026-...", "data": {...} } } }`
- **Atomic writing:** write-to-temp + rename (pattern identical to `titan_settings.json`)
- **Auto-backup:** `.bak` created before each save
- **Fallback:** If the main file is corrupt, `.bak` is tried; if that is also corrupt, store empty

### 96.4 Automatic migration from localStorage

On the first run after the update, if the `titan_hft_profiles` key exists in localStorage:
1. Each old profile is imported to disk via `import_profile`
2. The old key is deleted from localStorage
3. The migration is done only once (one-shot)

### 96.5 New buttons in the UI

- **EXPORT** — Export the selected profile as a JSON file (`titan_profile_<name>.json`)
- **IMPORT** — Open file picker for profile import from JSON; the name is taken from the file or from the text field

### 96.6 Bonus: N5 — max_manual_order_usd Enforcement

The setting `max_manual_order_usd` (default $50,000) existed but was not checked server-side. Now `submit_manual_order` in `trading.rs` blocks orders that exceed the limit with a clear error: "Order $X exceeds max_manual_order_usd limit ($Y)".

### 96.7 Bonus: Fixed async DELETE handler

Profile delete handler had `await cyberConfirm()` in a non-async function — real JS bug. Fixed with `async () => { ... }`.

### 96.8 Modified files

- `src-tauri/src/paths.rs` — `profiles_path()` added
- `src-tauri/src/state.rs` — ProfileStore/ProfileEntry/ProfileMeta structs, load/save functions, 8 tests
- `src-tauri/src/commands/settings.rs` — 6 orders New Bulls
- `src-tauri/src/commands/trading.rs` — N5 enforcement + 1 test
- `src-tauri/src/main.rs` — registered orders
- `src/titan_perf_stats.js` — PARAMETER PROFILES completely rewritten
- `src/index.html` — 2 new buttons (EXPORT/IMPORT profiles)

### 96.9 Validations

- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 188 tests passed (179 + 9 new), 0 regressions
- 31 consecutive sessions without regressions

---

## 97. CONCURRENT PRE-FETCH + EXIT PROCESSING (S5+S6) — Session 32

### 97.1 What was done

The autopilot evaluates positions and candidates **sequentially** — with 5 open positions and 10 candidate symbols, the latency was O(n * per_simbol_latency). Now evaluations run **concurrent** via `futures::future::join_all`, and decisions (API calls, state mutations) are applied sequentially.

### 97.2 Architecture: Gather in Parallel, Apply Sequentially

The principle: **reading is done in parallel, writing remains sequential**. No lock is held in the parallel code for more than microseconds.

```
Tick Autopilot:
  1. [PARALEL] join_all(evaluate_position_exit per pozitie)
     → Vec<ExitEvaluation> (candles, ATR, signals, stops, force_exit)
  2. [SECVENTIAL] Aplica decizii: close_position, trail updates, analytics, journal
  3. [SECVENTIAL] Pre-filtreaza candidati (flash crash, holding, pending, cooldown, correlation)
  4a. [SECVENTIAL] SignalStrategy path (neschimbat)
  4b. [PARALEL] join_all(evaluate_candidate_entry per candidat)
     → Vec<EntryEvaluation> (candles, indicators, check_entry)
  5. [SECVENTIAL] Selecteaza best entry (scor maxim), aplica OFI/Nexus, send_order
```

### 97.3 New structures

**ExitEvaluation** — encapsulates the exit decision per position without side effects:
- symbol, qty, is_short, current_price, entry_price, pnl_pct
- stop_loss, take_profit, trail_pct (already calculated with ATR + adaptive multipliers)
- force_exit, force_exit_reason (oracle flip, chaos, signal exit)
- partial_close: Option (partial_qty, partial_side)
- hold_secs, when_tf

**EntryEvaluation** — encapsulates the entry decision per candidate:
- symbol, decision (Option<TradeDecision>), cand_tf, vol_factor

### 97.4 Behavior Change: Best-Score Selection

Previously, the autopilot took the **first candidate** that passed the filters. Now, all the candidates are evaluated in parallel, and the one with the **maximum score** is selected (best-match instead of first-match). Result: the autopilot chooses optimally.

### 97.5 What has NOT changed

- **SignalStrategy** remains sequential (different flow, markers, not check_entry)
- **close_position** remains serialized (one API call to the broker per tick)
- **Pre-filtering** remains sequential (flash crash, pending orders, correlation — they depend on states)

### 97.6 Performance impact

With ring buffer (S1) + cache indicator (S2+S4) already implemented, most `get_candles` are cache hits (<1us). Parallelization helps the most when:
- The buffer is empty and REST fallback is performed (50-500ms per symbol)
- There are many open positions with ATR calculation per symbol
- There are many open positions with ATR calculation per symbol

### 97.7 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 197 tests passed (188 + 9 new), 0 regressions
- 32 consecutive sessions without regressions

---

## 98. PORTFOLIO VaR ENGINE + HHI CONCENTRATION GATE (S9) — Session 33

### 98.1 What is it?

Two complementary risk gates at **portfolio** level that PREVENT entries when the total risk is too high. Unlike CircuitBreaker / FlashCrash / DD Velocity (which are **reactive** — act AFTER the loss), VaR and HHI are **preventive** — block entry BEFORE the risk occurs.

- **VaR (Value at Risk)** — measures the statistical risk of the portfolio using volatility and correlation between assets. The question he answers: "How much can I lose per day with 99% probability?"
- **HHI (Herfindahl-Hirschman Index)** — measures portfolio concentration. The question: "How much capital did I put in a single symbol?"

### 98.2 Why was it necessary?

The existing risk management protects against INDIVIDUAL losses:
- Stop loss per position
- Circuit breaker on total daily PnL
- Flash crash on sudden price movement
- DD Velocity on loss speed

But there is NOTHING that says:
- "Don't enter NVDA because 80% of the portfolio is already NVDA" (concentration)
- "The portfolio has an implied volatility of 5%, a new position would increase it to 7%" (statistical risk)

VaR and HHI cover these two gaps.

### 98.3 VaR — Parametric Value at Risk

**Method:** Parametric (variance-covariance) at 99% confidence:

```
VaR = z_score(0.99) * sqrt(w' * Sigma * w)
```

where:
- `w` = the vector of weights (notional per asset / total notional)
- `Sigma` = the NxN covariance matrix (from log returns over the last 20 periods)
- `z_score(0.99)` = 2.326 (approximation Abramowitz & Stegun)

**CVaR (Expected Shortfall)** — average losses BEYOND VaR:

```
CVaR = VaR * pdf(z) / (z * (1 - confidence))
```

CVaR is always higher than VaR and captures tail risk.

**How the gate works:**

1. Close prices are extracted from the CandleRingBuffer for each symbol in the portfolio
2. Log returns and the covariance matrix are calculated
3. VaR is calculated with the ACTUAL portfolio
4. VaR is calculated with the PROPOSED portfolio (adding the new position)
5. If the proposed VaR > `max_portfolio_var_pct` → entry BLOCKED

### 98.4 HHI — Concentration Risk Gate

**Formula:**

```
HHI = sum(w_i^2)     unde w_i = |notional_i| / total_notional
```

**Range:**
- `HHI = 1.0` — all capital in one symbol (maximum concentration)
- `HHI = 0.25` — 4 equal positions (diversified)
- `HHI = 0.10` — 10 equal positions (well diversified)

**How the gate works:**

1. The notionals of the current positions are extracted
2. The estimated notional of the proposed position is added
3. The proposed HHI is calculated
4. If the proposed HHI > `max_concentration_hhi` → entry BLOCKED

### 98.5 Settings (HFT Engine tab)

In the **PORTFOLIO RISK GATES** card (purple, after TA ENTRY GATES):

| Setting | Default | Range | Description |
|---------|---------|-------|-----------|
| VaR Gate ON/OFF | OFF | toggle | Enable/disable VaR gate |
| Max Portfolio VaR % | 2.0% | 0.1-10.0 | Maximum threshold VaR 99% |
| Lookback Periods | 20 | 5-100 | How many return periods for covariance |
| Concentration Gate ON/OFF | OFF | toggle | Activate/deactivate HHI gate |
| Max HHI | 0.40 | 0.10-1.00 | Maximum concentration threshold |

**Both gates are OFF by default** — zero change in behavior until the user activates them explicitly.

### For HFT scalping with minute positions, 95% VaR is too permissive — it allows too many entries in high volatility conditions. 99% offers a higher safety margin, especially in combination with Circuit Breaker which already acts on absolute losses.

```
check_entry() → decide Buy/Short + scor
   ↓
TA Gates (Soft/Hard) → penalizeaza sau blocheaza
   ↓
OFI Filter → verifica flow direction
   ↓
Nexus Consensus → verifica acord multi-agent
   ↓
HHI Gate → verifica concentrare portofoliu ← NOU
   ↓
VaR Gate → verifica risc statistic portofoliu ← NOU
   ↓
Position Sizing → calculeaza qty
   ↓
send_order_internal() → executa
```

### 98.7 Why VaR at 99% and not 95%?

For HFT scalping with minute positions, 95% VaR is too permissive — it allows too many entries in high volatility conditions. 99% offers a higher safety margin, especially in combination with Circuit Breaker which already acts on absolute losses.

### 98.8 Known limitations

- **Parametric VaR assumes normal distribution** — underestimates tail risk (fat tails). The HHI complements with non-statistical protection.
- **20 lookback periods** are few — the covariance can be unstable. But in scalping, market conditions change quickly, so a long lookback would be counter-productive.
- **CVaR is calculated but not exposed in the UI** — available via API for future analysis.
- **Gates do not apply on SignalStrategy path** — only on non-signal candidates (consistent behavior with OFI/Nexus).

### 98.9 Tests

16 new tests in titan-risk crate (total 60):
- `compute_returns_basic/empty_and_single` — correct log returns + edge cases
- `covariance_matrix_single_asset/symmetric` — valid array
- `single_position_var` — VaR positive, VaR99 > VaR95
- `two_correlated_assets_var` — 2 identical assets 50/50 = same VaR as single
- `uncorrelated_assets_diversification_benefit` — diversified < concentrated
- `empty_portfolio_zero_var` — zero VaR without positions
- `cvar_exceeds_var` — CVaR > VaR (tail risk)
- `var_gate_blocks/allows` — gate works correctly at both extremes
- `hhi_single_position/equal_positions/empty` — HHI mathematically correct
- `hhi_gate_blocks_concentrated/allows_diversified` — HHI gate works

### 98.10 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 213 tests passed (197 + 16 new), 0 regressions
- 33 consecutive sessions without regressions

---

## 99. FEATURE STORE — ENTRY CONTEXT SNAPSHOT (S13) — Session 34

### 99.1 What is

Feature Store is a system for capturing the complete context of indicators and strategy at the exact moment of entering a trade. Each trade in the journal now contains a snapshot with 20 fields that describe the state of the market and the trading decision at the time of entry.

### 99.2 Problem solved

Before the Feature Store, `TradeRecord` in the log only contained execution data: symbol, direction, quantity, prices, PnL, duration, strategy, timeframe. The `notes` field was empty. The information "why the trade was entered" and "what was the state of the indicators" he was constantly losing on every trade.

Consequences:
- The post-trade analysis was limited to "I won/lost X dollars"
- It was not possible to answer the question "why did/didn't the trade work?"
- There is no data for optimization based on indicator-performance correlations
- Success/failure patterns were invisible

### 99.3 What TradeFeatures captures (20 fields)

| Category | Field | Description |
|-----------|------|-----------|
| **Oracle** | `oracle_score` | Oracle composite score (-100 to +100) |
| **Oracle** | `oracle_confidence` | Oracle Trust Level (0.0 to 1.0) |
| **Oracle** | `oracle_regime` | Market regime (Trending, MeanReverting, Chaotic, Unknown) |
| **Indicators** | `rsi` | Relative Strength Index |
| **Indicators** | `adx` | Average Directional Index (trend strength) |
| **Indicators** | `atr_pct` | ATR as a percentage of price (volatility) |
| **Indicators** | `ema9` | 9 period EMA |
| **Indicators** | `ema50` | 50-period EMA |
| **Flow** | `vpin` | Volume-sync Probability of Informed Trading |
| **Flow** | `hurst` | Hurst Exponent (persistence/mean-reversion) |
| **Flow** | `rvol` | Relative Volume (current vs average volume) |
| **Microstructure** | `ofi` | Order Flow Imbalance (bid/ask pressure) |
| **Microstructure** | `spread_bps` | Spread in basis points |
| **Microstructure** | `imbalance_ratio` | Bid/ask imbalance ratio |
| **Strategy** | `strategy_score` | check_entry() |
| **Strategy** | `signal_source` | Signal Source (ORACLE, GPU, FUSION, SIGNAL_STRATEGY) |
| **Strategy** | `fvg_type` | FVG Type (Bullish, Bearish, None) |
| **Strategy** | `entry_type` | Entry type (ex: Trending_LONG, SCALP_MeanRev_SHORT) |
| **TA Gates** | `ta_gates_passed` | Number of TA gates passed |
| **TA Gates** | `ta_gates_total` | Total number of evaluated TA gates |

### 99.4 How it works

```
ENTRY (send_order reusit)           EXIT (close_position)
    │                                    │
    ▼                                    ▼
snapshot_entry_features()          journal.record_trade()
    │                                    │
    ▼                                    ▼
state.entry_features               entry_features.remove(sym)
    .insert(symbol, features)           │
                                        ▼
                                   TradeRecord {
                                     ...,
                                     entry_features: Some(features)
                                   }
                                        │
                                        ▼
                                   journal.json pe disk
```

**At entry:** After `send_order_internal` succeeded, `snapshot_entry_features()` reads:
- Indicators from `IndicatorCache` (read-only, no re-compute)
- OFI state from `ArcSwap` (lock-free)
- Metadata from `TradeDecision` (score, FVG, entry_type)
- RVOL from vol_cache

**At full exit:** `entry_features.remove(&symbol)` -- extract and delete from map.

**At partial exit:** `entry_features.get(&symbol).cloned()` -- read without deleting (the position is still open).

### 99.5 Backward compatibility

Field `entry_features` on `TradeRecord` has `#[serde(default)]`. Old logs (without this field) are deserialized with `entry_features: None`. Zero impact on existing UI or functionality.

### 99.6 Future Use

Data in the Feature Store will be used by:
- **S20 AI Trade Journal Digest** -- the LLM will analyze indicator-performance correlations
- **S15 Oracle Weight Adaptation** -- IMPLEMENTED (Session 38). The Oracle weights adapt automatically per regime, with confidence-weighted learning and continuous decay. See Ch. 103
- **Improved Auto-Suggest** -- suggestions based on indicators, not just PnL
- **Pattern recognition** -- automatic identification of optimal input conditions

### 99.7 Changed files

- `src-tauri/src/journal.rs` -- TradeFeatures struct, TradeRecord/TradeRecordInput extended
- `src-tauri/src/indicator_cache.rs` -- `peek()` read-only lookup
- `src-tauri/src/loops/autopilot.rs` -- snapshot at entry (2 paths), propagation at exit (2 paths)
- `src-tauri/src/state.rs` -- `entry_features` on TitanState
- `src-tauri/src/main.rs` -- initialization `entry_features`
- `src-tauri/src/commands/misc.rs` -- `entry_features: None` on trade_updates

### 99.8 Tests (8 new, 221 total)

- Serde complete roundtrip (20 fields)
- Backward compatibility (load old journal)
- Persist with features / without features
- Mixed age (some with features, others without)
- Default values (zeros and empty strings)

### 99.9 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 221 tests passed (213 + 8 new), 0 regressions
- 34 consecutive sessions without regressions

---

## 100. MICROSTRUCTURE ALPHA SIGNALS — STREAM VPIN, SPREAD REGIME, DUAL VPIN DIVERGENCE (S10-lite) — Session 35

### 100.1 What is

Complete system of microstructure alpha signals that transforms existing streamer data (trade classification, spread, imbalance) from **binary filters** (yes/no) into **soft score modifiers** (bonus/penalty on the entry score). It adds three new components: **Stream VPIN** (real VPIN from trade flow), **Spread Regime** (narrowing detection), and an original signal -- **Dual VPIN Divergence** (stream vs candle VPIN comparison).

### 100.2 Why it matters

Prior to this implementation, microstructure data existed in the system but was only used in binary:
- `spread_is_safe()` -- yes or no (no shades)
- `ofi_confirms_signal()` -- yes or no (no intensity)
- VPIN from oracle -- calculated from BVC candle (weak proxy, not from real tape)

Now, the microstructure influences the **score** of each entry candidate. A candidate with narrowing spread, aligned imbalance and healthy VPIN receives up to **+14 points** bonus. A candidate with hidden toxicity receives a **-18 point** penalty. The difference between entering or not entering a trade.

### 100.3 Stream VPIN -- Real VPIN from trade flow

**What it does:** Calculates VPIN (Volume-Weighted Probability of Informed Trading) directly from trade-by-trade flow, using the existing Lee-Ready classification (`classify_trade`).

**Difference compared to VPIN candle (oracle):**

| Layout | Candle VPIN (oracle) | Stream VPIN (S10-lite) |
|--------|---------------------|----------------------|
| Classification | BVC (close position in range) -- proxy | Lee-Ready quote rule -- exact |
| Granularity | Per candle (1m/5m) | Per trade (~ms) |
| Binning | Time-synced (n candles) | Volume-synced (n shares) |
| Update | At candle boundary | At each classified trade |
| Accuracy on short TFs | Moderate | High |

**How it works:**

1. Each classified trade (buy/sell) adds volume to the current bucket
2. When `buy_vol + sell_vol >= bucket_vol_threshold` (5000 default), the bucket closes
3. VPIN raw = `sum(|buy - sell|) / sum(buy + sell)` over the last 20 buckets
4. EMA smoothing on VPIN raw (`alpha = 0.15`) to filter noise

**Interpretation:**
- VPIN ~ 0: balanced flow (buyers and sellers in balance)
- VPIN > 0.5: unbalanced flow (informants trade aggressively)
- VPIN > 0.85: extreme toxicity (high risk of adverse movement)

### 100.4 Spread Regime -- Narrowing detection

**What it does:** Two EMAs on `spread_bps` (bid-ask spread in basis points) with different speeds. When the fast EMA falls below 80% of the slow EMA, the `spread_narrowing` flag is activated.

**Parameters:**
- EMA fast: `alpha = 0.15` (~13 ticks half-life)
- EMA slow: `alpha = 0.03` (~66 ticks half-life)
- Narrowing threshold: `fast < slow * 0.80`

**Why it matters:**

Market makers tighten the spread when they anticipate imminent directional movement. Spread narrowing after a period of wide spread is a pre-directional signal. The autopilot uses this flag as a +5 bonus on the entry score.

### 100.5 Dual VPIN Divergence (original concept)

**What it does:** Compares Stream VPIN (from tap, precise) with Candle VPIN (from oracle, roughly). When they diverge significantly, it is informative:

| Situation | Divergence | Interpretation | Effect |
|----------|-----------|-------------|-------|
| Stream VPIN high + Candle VPIN low | > +0.15 | **Hidden toxicity** -- toxic flow detected in tape but not yet visible in candles | Penalty **-8** |
| Stream VPIN low + Candle VPIN high | < -0.15 | **False alarm** -- VPIN candle is noise from range compression | Both similar |
| Both similar | [-0.15, +0.15] | Concordant, without divergence | 0 |

**Academic reference:** Inspired by Easley, Lopez de Prado, O'Hara (2012) "Flow Toxicity and Liquidity in a High-Frequency World". Dual-source divergence is not implemented in retail HFT -- original concept.

### 100.6 Microstructure Score Modifier -- Full Table

Function `apply_microstructure_modifier()` is applied in `evaluate_candidate_entry()`, AFTER `check_entry()` calculates the base score, BEFORE the ranking of the candidates:

| Condition | Bonus/Penalty | Cumulative | Note |
|----------|--------------|-----------|------|
| Spread narrowing | **+5** | Yes | Market makers anticipate direction |
| NBBO imbalance aligned (buy + imb > 0.3 or sell + imb < -0.3) | **+4** | Yes | Order flow confirms direction |
| Dual VPIN: false alarm filtered (divergence < -0.15) | **+5** | Yes | Candle VPIN was noise |
| Dual VPIN: hidden toxicity (divergence > +0.15) | **-8** | Yes | Danger hidden in the tape |
| Extreme VPIN Stream (> 0.85) | **-10** | Yes | Very high toxicity |

**Maximum possible:** +14 (narrowing + imbalance + false alarm) to -18 (hidden toxicity + extreme toxicity)

**Important:** All modifications are **soft** (adjusts the score, does not block). A candidate with a high penalty can still be selected if he has a sufficiently high base score. Consistent with the "best candidate by score" philosophy from concurrent entry evaluation (S5+S6).

### 100.7 Integration in the pipeline

```
Trade message --> classify_trade() --> StreamVpin.update()
                                            |
Quote/L2 message --> spread_bps --> SpreadRegime.update()
                                            |
                               OfiState ArcSwap (lock-free)
                              /                            \
                    Autopilot:                          Frontend:
                evaluate_candidate_entry()          ofi-update event
                        |                      (stream_vpin, spread_narrowing)
                check_entry() -> score
                        |
        apply_microstructure_modifier()
                        |
              score +/- modifier
                        |
                best candidate selected
                        |
            snapshot_entry_features()
            (vpin=stream_vpin, vpin_divergence, spread_narrowing)
```

### 100.8 TradeFeatures -- New fields

Two fields added to `TradeFeatures` (journal.rs), both with `#[serde(default)]` for backward compatibility:

| Field | Type | Default | Description |
|------|-----|---------|-----------|
| `vpin_divergence` | f64 | 0.0 | `stream_vpin - candle_vpin` at the time of entry |
| `spread_narrowing` | bool | false | Flag spread regime at the time of entry |

The existing `vpin` field now contains **stream_vpin** (more precisely) instead of VPIN candle.

### 100.9 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 240 tests passed (221 + 19 new), 0 regressions
- 35 consecutive sessions without regressions
- 19 new tests: stream_vpin (4), spread_regime (4), micro_modifier (10), journal backward compat (3)

---

## 101. CONFIGURABLE STRATEGY THRESHOLDS — SCORE MODIFIERS, CLASSICAL BONUSES, ENTRY THRESHOLD MATRIX (N4) — Session 36

### 101.1 What it is

The extraction system of ~50 hardcoded magic numbers from `strategy.rs` in configurable structures from the UI. Before, the critical parameters that controlled how easy/hard Titan enters a trade were fixed in the source code -- now they are editable in real time from the HFT Engine tab, without recompilation.

### 101.2 Why it matters

Before N4:
- If you wanted to make the scoring more aggressive in trending, you had to modify the source code and recompile
- If you wanted to test a bonus FVG of 20 instead of 15, the same story
- You couldn't quickly compare different configurations
- The risk of a typo was real (change a number and break another branch)

After N4:
- All these values ​​are **sliders and inputs** in the UI
- Saved on disk, persistent between restarts
- VALIDATE button that checks logical consistency
- RESET DEFAULTS button that restores the original values
- **Zero behavior change** with identical defaults -- the bot behaves exactly as before if you don't change anything

### 101.3 The 3 groups of parameters

**Group 1: Score Modifiers** (21 parameters)

Fine controls on the score calculated by `calculate_score()`:

| Parameter | Default | What it does |
|-----------|---------|---------|
| Chaos Kill Penalty | -50 | Penalty when confidence is below chaos threshold |
| Chaos Kill Fallback | 50 | Score returned when oracle is not available |
| Chaos Conf x MeanRev | 1.5 | Chaos threshold multiplier per MeanRev regime |
| Chaos Conf x Chaotic | 2.5 | Multiplier per Chaotic regime |
| Chaos Conf x Unknown | 1.5 | Multiplier per regime Unknown |
| Regime x MeanRev | 0.82 | Multiplier oracle contribution per MeanRev |
| Regime x Chaotic | 0.55 | Multiplier per Chaotic |
| Regime x Unknown | 0.73 | Multiplier on Unknown |
| VPIN Penalty High | -20 | Penalty when VPIN > scalping gate |
| VPIN Penalty Medium | -10 | Penalty when VPIN > normal gate |
| FVG Bull Bonus | +15 | Bonus on Fair Value Gap bullish detected |
| FVG Bear Penalty | -15 | Penalty on bearish FVG |
| RVOL Tier1 Thr. | 2.0 | Tier 1 RVOL Threshold |
| RVOL Tier1 Bonus | +8 | Bonus if RVOL > tier 1 |
| RVOL Tier2 Thr. | 4.0 | RVOL tier 2 threshold (institutional) |
| RVOL Tier2 Bonus | +12 | Bonus if RVOL > tier 2 |
| StochRSI OB | 80 | Overbought threshold for TA gate StochRSI |
| StochRSI OS | 20 | Oversold threshold for TA gate StochRSI |
| BB Squeeze Width | 0.02 | Bollinger squeeze threshold (width < this threshold) |
| Strong Label | 80 | Oracle score > this threshold = "STRONG" entry |
| Mild Label | 60 | Oracle score > this threshold = normal entry |

**Group 2: Classical Bonuses** (6 parameters)

Controls on bonuses in `compute_classical_confirmation()` (active only when `use_classical_filters` is ON):

| Parameter | Default | What it does |
|-----------|---------|---------|
| RSI Bonus | 10.0 | Bonus when RSI confirms direction |
| EMA Align Bonus | 15.0 | Bonus when EMA20 > EMA50 (long) or vice versa |
| ADX Bonus | 10.0 | Bonus when ADX > trend threshold |
| BB Squeeze Bonus | 5.0 | Bonus when Bollinger bands are tight |
| VWAP Bonus | 10.0 | Bonus when price is above/below VWAP |
| Cap | 50.0 | Upper limit of the amount of bonuses |

**Group 3: Entry Threshold Matrix** (7 regimes x 3 values x 2 modes = ~42 cells)

The grid that determines the entry thresholds per market regime:

| Regim | Mode | Long | Short | Confidence |
|-------|------|------|-------|------------|
| Trending | Scalp | 22 | -22 | 0.18 |
| Trending Aggro | Scalp | 15 | -15 | 0.12 |
| MeanRev | Scalp | 32 | -32 | 0.28 |
| MeanRev Aggro | Scalp | 25 | -25 | 0.22 |
| Chaotic | Scalp | 50 | -50 | 0.38 |
| Unknown | Scalp | 32 | -32 | 0.25 |
| Unknown Aggro | Scalp | 22 | -22 | 0.18 |
| Trending | Normal | 35 | -35 | 0.25 |
| Trending Aggro | Normal | 25 | -25 | 0.20 |
| MeanRev | Normal | 50 | -50 | 0.35 |
| MeanRev Aggro | Normal | 40 | -40 | 0.30 |
| Chaotic | Normal | 70 | -70 | 0.55 |
| Unknown | Normal | 50 | -50 | 0.30 |
| Unknown Aggro | Normal | 35 | -35 | 0.25 |

Meaning of the columns:
- **Long**: oracle_score must exceed this value for long entry
- **Short**: oracle_score must be below this value for short entry
- 101.4 UI in HFT Engine tab

### 101.4 UI in HFT Engine tab

Three new cards appear between STRATEGY SCORING and ENGINE STOPS:

1. **SCORE MODIFIERS** (orange header) -- 21 sliders in a 3-column grid
2. **CLASSICAL BONUSES** (green header) -- 6 sliders, the card appears semi-transparent when Classical Filters is OFF
3. **ENTRY THRESHOLD MATRIX** (header cyan) -- grid 7 rows x 3 columns with numeric inputs, Scalping/Normal dropdown, VALIDATE and RESET DEFAULTS buttons

### 101.5 VALIDATE button

Call the command `validate_thresholds` and check:

1. All long_threshold > 0 (a negative threshold has no meaning)
2. All short_threshold < 0
3. All conf_threshold in range [0.05, 0.95]
4. Scalping thresholds <= Normal thresholds (scalping is more permissive)
5. Trending thresholds <= Chaotic thresholds (natural gradient)
6. StochRSI overbought > oversold
7. RVOL tier1 < tier2

The results appear as toasts (warnings, not blocking errors).

### 101.6 RESET DEFAULTS button

Restores the values ​​from the table above for the currently selected mode (Scalping or Normal). Useful after experiments.

### 101.7 Technical implementation

Three new structs in `app_state.rs`:
- `EntryThreshold { long_threshold, short_threshold, conf_threshold }`
- `ScoreModifiers` (21 fields with `impl Default`)
- `ClassicalBonuses` (6 fields with `impl Default`)

Fields on `AppState` with `#[serde(default)]` for backward compatibility. Entry thresholds are `HashMap<String, EntryThreshold>` with keys like `trending`, `trending_aggro`, `mean_rev`, etc.

The function `lookup_entry_threshold()` ​​in `strategy.rs` does lookup with fallback safe if a key is missing from the HashMap.

33+ new match arms in `settings.rs` + `apply_entry_threshold_field()` dynamic parser for entry thresholds.

### 101.8 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 248 tests passed (240 + 8 new), 0 regressions
- 36 consecutive sessions without regressions
- 8 new tests: defaults match hardcoded (3), custom values ​​(2), serde roundtrip (1), validation (2)

---

## 102. CI HARDENING + CRITERION BENCHMARKS (S19) — Session 37

### 102.1 What was added

The CI pipeline has been expanded with two new audit tools and a complete system of performance benchmarks.

Configuration in `src-tauri/deny.toml`
- Configuration in `src-tauri/deny.toml`
- Whitelist licenses: MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode, OpenSSL, BSL-1.0, CC0-1.0, MPL-2.0
- 19 advisories ignored with documented rationale (all from Tauri 1.x transitive deps)
- All workspace crates marked `publish = false` (private, not published)
- It runs automatically in CI at every push/PR

**cargo-audit** — scan the dependency tree for known CVEs (RustSec Advisory DB):
- It runs automatically in CI at every push/PR
- Block the build if there are unresolved vulnerabilities

**Criterion Benchmarks** — 6 benchmarks on 3 critical paths:
- Files: `src-tauri/crates/titan-indicators/benches/bench_oracle.rs`, `bench_indicators.rs`, `src-tauri/benches/bench_check_entry.rs`
- It is run manually with `cargo bench` (not automatically in CI — see 102.3)

### 102.2 Available Benchmarks

| Benchmark | What measure | Candles | Baseline result | Target |
|-----------|-----------|---------|-------------------|--------|
| oracle_calculate_200 | Oracle 7-layer + fusion | 200 | 873 us | <5ms |
| oracle_calculate_100 | Oracle 7-layer + fusion | 100 | 208 doors | - |
| oracle_calculate_50 | Oracle 7-layer + fusion | 50 | 51 doors | - |
| indicators_all_200 | EMA + RSI + MACD + BB + ADX + VWAP + StochRSI + Oracle | 200 | 895 USD | <2ms |
| indicators_all_100 | The same complete set | 100 | 218 doors | - |
| full_pipeline_200 | Indicators + Oracle end-to-end (as check_entry) | 200 | 906 us | <10ms |

All under 1ms for 200 candles. The complete pipeline is ~11x below target.

### 102.3 How to use benchmarks

Benchmarks are on-demand (not automatic). It is run when you want to check the performance after a change:

```bash
cargo bench                            # toate 6 benchmarks
cargo bench -p titan-indicators        # doar oracle + indicators (5 benches)
cargo bench --bench bench_check_entry  # doar full pipeline (3 benches)
```

Criterion generates HTML reports in `target/criterion/report/index.html`. On successive runs, it automatically compares with the previous baseline and shows the regression/improvement.

### 102.4 Full CI Pipeline (Post-Session 37)

Files: `.github/workflows/ci.yml` (push/PR) + `.github/workflows/release.yml` (v* tags)

Steps in order:
1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — lint
3. `cargo build --release` — compilation
4. `cargo test` — 248 tests
5. `cargo deny check` — license + advisory audit
6. `cargo audit` — CVE vulnerability scan

### 102.5 deny.toml — Advisory Ignores

19 ignored advisories, all from transitive dependencies that we did not control:
- 10x gtk-rs GTK3 bindings (Tauri 1.x, resolved in Tauri 2.x)
- instant, paste, proc-macro-error, rustls-pemfile, fxhash (Tauri 1.x unmaintained deps)
- dotenv (startup-only, zero security risk)
- tar (transitive, not used in our codepaths)
- bytes BytesMut overflow (reqwest transitive, not exploitable)
- regex DoS (we don't have regex user-facing)

### 102.6 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 248 tests passed, 0 regressions
- `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok
- `cargo bench`: 6 benchmarks OK, all under 1ms
- 37 consecutive sessions without regressions

---

## 103. ONLINE ORACLE WEIGHT ADAPTATION (S15) — Session 38

### 103.1 What it is

Before S15, the Oracle weights were static (Kalman 0.30, Wavelet 0.25, Hurst 0.20, VPIN 0.15, Entropy 0.10), identical regardless of the market regime or the real performance of each layer. A layer that predicts incorrectly for months keeps the same weight as one that predicts correctly. There is no feedback loop between the trade results and the Oracle configuration.

### 103.2 Problem solved

Before S15, the Oracle weights were static (Kalman 0.30, Wavelet 0.25, Hurst 0.20, VPIN 0.15, Entropy 0.10), identical regardless of the market regime or the real performance of each layer. A layer that predicts incorrectly for months keeps the same weight as one that predicts correctly. There is no feedback loop between the trade results and the Oracle configuration.

### 103.3 How it works

**When entering the trade:**
1. Oracle calculates the complete snapshot (7 layers)
2. The 5 directional signals [-1, +1] are extracted (kalman_signal, wavelet_signal, etc.)
3. It is stored together with the regime and confidence at the time of entry

**When exiting the trade:**
1. The signals from the entry are recovered
2. The PnL direction is calculated (+1 profit, -1 loss)
3. The per-layer EMA correlation is updated: `corr = corr * (1 - alpha) + (signal * direction) * alpha`
4. Every 50 trades, the adapted weights are recalculated

**On the next entry:**
1. We are looking for adapted weights for the CURRENT regime of the market
2. Oracle uses adaptive weights instead of static ones

### 103.4 The 3 smart upgrades

**Per-Regime Adaptation:** 4 independent sets of weights, one per MarketRegime (Trending, MeanReverting, Chaotic, Unknown). Kalman can dominate in trend but Hurst in mean-reversion. Each regime optimizes its weights independently.

**Confidence-Weighted Learning:** `alpha_efectiv = alpha_baza * confidence_la_entry`. Trades with high confidence (Oracle was sure) contribute more to learning than those with low confidence. It filters the noise from uncertain decisions.

**Continuous Decay:** `corr *= 0.997` applied to every trade on all regimes. Half-life of ~230 trades. Adaptation "forget" gradually the old information, protecting against overfitting on market conditions that are no longer relevant.

### 103.5 Safety mechanisms

| Mechanism | Effect |
|----------|-------|
| Weight cap 0.5x - 2.0x | A weight cannot increase/decrease more than 2x compared to the base |
| Sum normalization = 1.0 | The summed adjusted weights always make 1.0 |
| Toggle default OFF | `enable_oracle_adaptation` must be activated manually |
| Continuous decay | Prevents overfitting on historical sequences |
| `layer_signals_at_entry` volatile | On restart, only entry snapshots in-flight are lost (not the correlations) |

### 103.6 How to activate

In the **HFT Engine** tab, the **Oracle Fusion** section:
- Toggle **Enable Oracle Adaptation** = ON
- Setting key: `enable_oracle_adaptation`

The adaptation starts accumulating data from the first trade after activation. After the first 50 trades, the weights are recalculated automatically.

### 103.7 Persistence

`OracleAdaptation` (per-regime correlations, adjusted weights, trade_count) is saved in `titan_state_snapshot.json` through `StateSnapshot`. Upon restart, the accumulated data is automatically restored (if the snapshot is less than 12 hours old).

`layer_signals_at_entry` (signals at entry) are NOT persisted — they are volatile per-session data. If a trade is open at the restart, that trade will not contribute to the adaptation.

### 103.8 Interaction with manual weights

The weights in the Oracle Fusion slider (Kalman W, Entropy W, etc.) become the **base weights** when the adaptation is active. Adaptation changes them relative to these values ​​(0.5x - 2.0x). If you change the Kalman W slider from 0.30 to 0.40, the adaptation will oscillate between 0.20 and 0.80 (not 0.15 - 0.60).

When the adaptation is OFF, the weights in the slider are used directly (identical behavior as before S15).

### 103.9 Changed files (8)

- `oracle.rs` — OracleAdaptation struct, LayerEntrySnapshot, extract_layer_signals, 4 methods, 11 tests
- `lib.rs` (titan-indicators) — export OracleAdaptation + extract_layer_signals
- `Cargo.toml` (titan-indicators) — serde_json dev-dependency
- `autopilot.rs` — 2 entry hooks + 2 exit hooks
- `state.rs` — TitanState + StateSnapshot + save/restore + AppSettings
- `app_state.rs` — enable_oracle_adaptation + Default
- `settings.rs` — apply_setting_field match arm
- `main.rs` — TitanState init

### 103.10 Tests (11 new, 259 total)

- adaptation_default_is_neutral
- record_outcome_updates_correct_regime
- weight_direction_follows_correlation
- weight_cap_enforced
- weights_normalized_to_one
- continuous_dec ay_shrinks_correlations
- confidence_weighted_ema
- serde_roundtrip_adaptation
- adapt_weights_no_data_returns_base
- fifty_trade_trigger
- extract_layer_signals_from_snapshot

### 103.11 Validations

- `cargo check`: 0 errors
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 259 tests passed (60 risk + 53 indicators + 21 genetic + 120 main + 5 integration)
- `cargo bench -p titan-indicators`: oracle_calculate_200 = 898us (target <5ms), indicators_all_200 = 919us (target <2ms)
- 0 regressions, 38 consecutive sessions without regressions

---

## 104. EXIT INFLIGHT GUARD — Anti-Loop Protection at Exits (Sessions 39-40)

### 104.1 Problem

The autopilot decides to close a position based on the positions reported by the broker (`fetch_positions` via API Alpaca). After `close_position` sends a sell order, the autopilot loop restarts in ~500ms. The broker may need 1-5+ seconds to reflect the closed position in the API response, so the next `fetch_positions` still returns the position — and the autopilot sends another SELL order.

**Symptoms observed in production:**

```
19:28:07 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
19:28:07 ORDER ACCEPTED: SELL LAR x212.0000 LAR
19:27:52 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
19:27:52 ORDER ACCEPTED: SELL LAR x212.0000 LAR
19:27:42 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
... (12+ ordine identice in 42 secunde)
19:27:48 RECONCILIATION: 2 open positions synced from Alpaca
```

Each duplicate order is either executed (accidentally opening a short position) or generates 429 errors (rate limit).

**What was missing (S39):**
- `evaluate_position_exit` does not check if an exit is already in flight
- `close_position` does not set `PendingClose` or any inflight flag
- `has_pending_orders_for_symbol` was only used for the entry gate, not on the exit
- Entry cooldown (`entry_cooldown_secs`) does not apply exits

**Race conditions discovered after S39 (resolved in S40):**
- The 30s TTL guard was prematurely reset by `trade_updates` (local fill) and `position_recon` (grace/blacklist filtered symbols)
- Optimistic deletion of the position in `settings.positions` caused the creation of a synthetic position at fill
- There is no verification of open orders at the broker before us exits

### 104.2 Solution (V2 — Session 40)

**Two layers of protection:**

1. **Guard inflight strict** — Field on `TitanState`:

```rust
pub exit_inflight: Mutex<HashMap<String, u64>>
```

Maps `simbol -> timestamp_ms`. If a symbol exists in the map, the autopilot does not send another exit — **no timeout**. The guard remains active until the broker confirms that the position is flat (through reconciliation) or a permanent error removes the entry.

2. **Guard broker open-order** — Before `close_position`, the autopilot queries `has_pending_orders_for_symbol()` (REST `GET /v2/orders?status=open&symbols=X`). If the broker already has an open order for the symbol, the new exit is skipped.

### 104.3 Complete flow

```
evaluate_position_exit() (tick autopilot)
    |
    +-- exit_inflight.contains_key(symbol)?
    |       DA -> return None (exit deja in zbor)
    |
    v
should_close = true (decizia de exit)
    |
    +-- has_pending_orders_for_symbol(symbol)?
    |       DA -> skip (broker are deja ordin deschis)
    |
    v
close_position() apelat
    |
    +-- cancel_open_orders_for_symbol (curata ordine vechi)
    +-- seteaza exit_inflight[symbol] = now_ms
    +-- trimite ordinul la broker (send_order_internal)
    |
    v
Autopilot success path:
    +-- marcheaza pozitia PendingClose (nu o sterge)
    +-- seteaza recon_grace (30s)
    +-- journal, analytics, trail cleanup
    |
    v
Fill ajunge pe WebSocket (trade_updates):
    +-- reduce qty pe pozitia existenta
    +-- daca qty ~= 0: sterge pozitia, state = Closed
    +-- exit_inflight NU se sterge (lasat pentru recon)
    +-- daca pozitia nu exista + exit_inflight activ: NU creeaza sintetica
    |
    v
Position reconciliation (la 15s):
    +-- fetch_positions de la broker (sursa de adevar)
    +-- exit_inflight.retain: pastreaza doar simboluri inca prezente la broker
    +-- cand broker-ul nu mai raporteaza simbolul -> exit_inflight se curata
```

### 104.4 Where to set / where to delete

| Location | Action | File |
|---------|---------|--------|
| `close_position()` | **SET** — before sending the order | `commands/market_data.rs` |
| Position reconciliation | **CLEAR** — when the symbol disappears from the broker's raw positions | `loops/position_recon.rs` |
| Autopilot permanent failure | **CLEAR** — for errors 422/invalid/not found | `loops/autopilot.rs` |

**Does not clear in:**
- Autopilot exit success — position is marked `PendingClose`, guard remains active
- Trade updates fill complete — local fill does not reset guard (broker may still report position)

### 104.5 Why strict guard without TTL

- **Initial version (S39)** used a TTL of 30 seconds — after 30s, the guard expires and the autopilot could resend. But 4 mechanisms could reset prematurely: trade_updates fill, position_recon filtered, autopilot success clear, and lack of verification at the broker.
- **Current version (S40)** removes the complete timeout. The guard remains active until **the broker confirms that the position is flat** (through reconciliation at 15s). This is the only reliable source of truth.
- **Safety net:** If something goes very wrong (network partition, API down), reconciliation at 15s will detect the disappearance of the symbol and clear the guard. Permanent errors (422, not found) clear it immediately.
- **Open-order check** adds a second layer: even if exit_inflight is somehow reset, the broker still reports the open order and the autopilot will not send a duplicate.

### 104.6 Modified files

**S39 (basic structure — 5 files):**
- `src-tauri/src/state.rs` — field `exit_inflight: Mutex<HashMap<String, u64>>` on TitanState
- `src-tauri/src/main.rs` — initialization with empty HashMap
- `src-tauri/src/loops/autopilot.rs` — guard inflight in `evaluate_position_exit` + clear at success and permanent failure
- `src-tauri/src/commands/market_data.rs` — set timestamp in `close_position` before de submit
- `src-tauri/src/loops/trade_updates.rs` — clear inflight at full fill
- `src-tauri/src/loops/position_recon.rs` — clear stale inflight when symbols disappear

**S40 (hardening — 4 files, 6 changes):**
- `src-tauri/src/loops/autopilot.rs` — guard broker open-order + guard inflight strict (without TTL) + PendingClose on success
- `src-tauri/src/loops/trade_updates.rs` — removed clear inflight on fill + suppress synthetic position on exit
- `src-tauri/src/loops/position_recon.rs` — `exit_inflight.retain` uses raw positions (not `local_positions` filtered)
- `src-tauri/src/commands/order_exec.rs` — `ORDER FILLED` renamed to `ORDER ACCEPTED`

### 104.7 Validations

- `cargo check`: 0 errors, 0 warnings
- 0 linter errors
- 40 consecutive sessions without regressions

### 104.8 PendingClose Lifecycle

Prior to S40, the autopilot would delete the position from `settings.positions` immediately after `close_position` returned `Ok()`. This causes two problems:

1. When the fill arrived on the WebSocket, `trade_updates` did not find the position and created a new synthetic Sell type (false short position)
2. Reconciliation re-inserts the position as `Open` after the expiration of the grace if the broker still reports it

**Solution:** The autopilot marks the position `PendingClose` (with `closed_at` set) instead of deleting it. This allows:
- `trade_updates` to find the position and process it correctly (reduce qty, delete when flat)
- Reconciliation to keep the `PendingClose` status by merging with the existing data
- The position should disappear naturally when the broker no longer reports it

### 104.9 ORDER ACCEPTED — Log Correct

Previously, `send_order_internal` emitted the message `ORDER FILLED: SELL LAR x212.0000 LAR` on the HTTP 200 response. This was misleading — HTTP 200 means that the broker **accepted** the command, not that it was **executed** (the actual fill comes on the WebSocket as a separate event of type `fill` or `partial_fill`).

**After S40:** The message is `ORDER ACCEPTED: SELL LAR x212.0000 LAR`. Actual fills appear as `FILL: SELL LAR x212.0000 @ $X.XX [fill]` (issued by `trade_updates`).

---

## 105. CONFIGURATION COHERENCE — apply_runtime_config (Session 44)

The first session of the Audit Roadmap. Solves the problem of desynchronization between the saved settings (`AppSettings` / `AppState`) and the runtime fields in `TitanState`.

### 105.1 Problem solved

Before S44, there were **4 different ways** through which the settings reached the runtime (update_settings, batch_update, load_profile, main.rs startup), each with its own synchronization logic. Result: some runtime fields (strategy_mode, dry_run, circuit breaker thresholds, flash crash params, nexus_fusion_alpha) could remain out of sync after certain operations.

Additionally, `batch_update_settings` calls `save_settings_to_disk` **before** error checking — if an individual field fails, the partially corrupted config was already persisted to disk.

### 105.2 Solution: apply_runtime_config()

A single function in `state.rs` that synchronizes **all** runtime fields from AppSettings in TitanState:

- **strategy_mode** — via `StrategyMode::from_name()` (centralized parsing, remove duplicate match from 3 places)
- **nexus_fusion_alpha** — direct copy
- **dry_run** — combine `TITAN_DRY_RUN` env var with `settings.dry_run_enabled`
- **circuit_breaker** — `update_thresholds(caution, halt, panic)` with Decimal conversion
- **flash_crash** — `reconfigure(window_secs, threshold_pct, cooldown_mins)` without losing sliding window history

### 105.3 Transactional batch_update_settings

`batch_update_settings` now uses a snapshot + rollback mechanism:

1. **Snapshot** — full clone of AppState before changes
2. **Apply** — apply each field individually; if either fails, restore the snapshot
3. **Persist** — save on disk and Redis side effects only on success path
4. **Runtime sync** — `apply_runtime_config` called only once at the end

### 105.4 New command: apply_full_settings

Tauri command `apply_full_settings` accepts a `AppSettings` directly as JSON typed. Used by:

- **Import** (from `titan_perf_stats.js`) — send the JSON payload directly, without `String(v)` coercion
- **Undo** — ditto, sends full AppSettings

Include `validate_settings_coherence()` — cross-field validation:
- breaker_caution > breaker_halt > breaker_panic (ordering)
- stop_loss, take_profit > 0
- risk_per_trade between 0-100%
- hold_time_max >= hold_time_exit

### 105.5 FlashCrashDetector::reconfigure()

The new method on `FlashCrashDetector` (in `candle_buffer.rs`) that updates the parameters (window duration, threshold %, cooldown) **without** deleting the price history from the sliding windows. Before, the only option was `clear()` which lost all history.

### 105.6 Import hardened profiles

`import_profile` check now:
- **schema_v** — version of the schema in the profile vs `CURRENT_PROFILE_SCHEMA_V`
- **Round-trip serde** — serializes + deserializes the payload; if it fails, it rejects the import
- **Coherence warnings** — log any warnings from `validate_settings_coherence()`

### 105.7 Regression tests (4)

| Test | What is he checking? |
|------|-------------|
| `settings_round_trip_coherence` | AppSettings ->> JSON --> AppSettings keeps all fields |
| `strategy_mode_from_name_covers_all_variants` | All variants of StrategyMode are covered by from_name |
| `flash_crash_reconfigure_preserves_windows` | reconfigure() does not lose sliding window history |
| `validate_coherence_catches_bad_breakers` | Wrong breaker ordering is detected |

### 105.8 Modified Files

| File | Change |
|--------|-----------|
| `state.rs` | apply_runtime_config(), StrategyMode::from_name(), 4 tests |
| `candle_buffer.rs` | FlashCrashDetector::reconfigure() |
| `commands/settings.rs` | batch transactional, apply_full_settings, validate_coherence, import hardened |
| `commands/strategy_cmd.rs` | from_name() replaces duplicate match |
| `main.rs` | from_name() at startup + register apply_full_settings |
| `titan_perf_stats.js` | Import/undo use apply_full_settings (no String coercion) |
| `.cursor/rules/titan-cyber-hud.mdc` | Design system rule (NEW) |

### 105.9 Cyber-HUD Design System Rule

Bonus: created `.cursor/rules/titan-cyber-hud.mdc` — encodes the existing design system from `styles.css` (37 CSS custom properties, component patterns, animations). Any future frontend changes automatically respect this visual contract. Includes: color tokens, typography, component patterns (`.cyber-panel`, `.titan-btn-*`, `.titan-toast-*`), strict rules (zero inline styles, max 6 glow effects on screen, mandatory fonts).

---

## 106. SMARTTREND AI v3 — FSM + ENGINE + LIVE PANEL (S07b) — Session 55

### 106.1 What is

SmartTrend AI v3 is a trend intelligence system with 3 pillars (Macro, Micro, Momentum) orchestrated by a finite state machine (FSM). In S07a we built the pillars and the BayesianTracker. In S07b I added the complete FSM, the SmartTrendEngine engine, the wiring in settings, the Tauri command, and the UI panel.

### 106.2 State Machine (FSM)

SmartTrend operates through 5 states that shape the cycle of a trade:

| Status | Description | Badge color |
|-------|-----------|---------------|
| **Flat** | No position, no active signal | Dim (gray) |
| **Stalking** | Signal detected, waiting for confirmation | Gold |
| **Entry** | Confirmed, marker issued, position being opened | Cyan (flicker) |
| **InTrade** | Open position, monitoring active | Green |
| **Trail** | Signal weakening, trailing stop active | Magenta (breathe) |

**Transitions:**
- **Flat -> Stalking**: directional strength >= stalking threshold AND non-Neutral direction
- **Stalking -> Entry**: strength >= entry threshold AND confirmation_bars >= confirmation_needed AND the pillars are aligned (no cross-divergence)
- **Stalking -> Flat**: strength drops below threshold (with hysteresis) OR timeout (2x confirmation_needed)
- **Entry -> InTrade**: automatically on next evaluate()
- **InTrade -> Trail**: directional strength drops below entry * 0.8
- **Trail -> Flat**: the direction reverses (exit marker issued + BayesianTracker updated)
- **Any state -> Flat**: changepoint reset (when changepoint_prob >= 0.7)

### 106.3 SmartTrendEngine

`SmartTrendEngine::evaluate()` is the central point. Receives a `SmartTrendInput` and returns `SmartTrendResult`:

**Input:** macro_input, micro_input, momentum_input, current_price, changepoint_prob, dominant_cycle, tf_minutes

**Output:** score (0-100), state, prev_state, direction, markers (vec), pillar_scores, bars_in_state, confirmation_progress

**Internal flow:**
1. Evaluate the 3 pillars via `evaluate_pillars()`
2. Check changepoint reset (force Flat + Exit marker if it was InTrade/Trail)
3. Run FSM transitions based on directional_strength
4. Emit markers at each transition
5. Update BayesianTracker at exits (success/failure)

### 106.4 Dynamic Confirmation Window

The confirmation window (how many bars the score must remain above the entry threshold in Stalking) adapts dynamically:

```
confirmation_needed = max(2, dominant_cycle / tf_minutes)
```

- dominant_cycle comes from wavelet (OracleHistory.dominant_cycle_series)
- tf_minutes is deduced from timeframe (1m=1, 5m=5, etc.)
- Fallback: st_stabilization_window (default 8) if cycle unavailable
- Shortening proportional to strength: Extreme -25%, Strong -15%

### 106.5 Changepoint-Aware Reset

When the changepoint probability exceeds 0.7 (configurable `st_changepoint_reset_threshold`):

- Force transition to Flat, regardless of current state
- If it was in InTrade/Trail, issue Exit marker
- Reset BayesianTracker to uninformative prior
- Emit ChangePointReset marker with details
- Increment stats.changepoint_resets

### 106.6 Bonuses implemented

| Bonus | Description | Config |
|-------|-----------|--------|
| **Trail ATR Auto-Tighten (B1)** | In Trail, stop tightens by 5% per bar | `st_trail_tighten_pct` (0.05) |
| **Pillar Agreement Gate (B2)** | Entry requires 2/3 aligned pillars (without cross-divergence) | cross_divergence from PillarDiagnostics |
| **FSM Stats Tracking (B3)** | Transitions per type, bars per state, hit rates, resets | SmartTrendStats in diagnostics |
| **Hysteresis Anti-Chatter (B4)** | Stalking->Flat requires drop below threshold - margin | `st_hysteresis_margin` (5.0) |

### 106.7 SmartTrend Live Panel (UI)

Cyber-HUD panel in sidebar (under MICROSTRUCTURE), visible only when `st_enabled = true`:

```
+--[ SMARTTREND LIVE ]-------------------+
|  STATE: [FLT] [STK] [ENT] [TRD] [TRL] |
|  DIR: LONG    SCORE: 67.2  STR: Mod    |
|  BARS: 4/8 [========>           ]      |
|  M:62  m:71  P:58   AGREE             |
+----------------------------------------+
```

- **STATE**: 5 badges (one active with glow corresponding to the state)
- **DIR**: LONG (green) / SHORT (red) / -- (dim)
- **SCORE**: cyan value
- **STR**: Weak/Moderate/Strong/Extreme with corresponding color
- **BARS**: progress bar confirmation (only visible in Stalking)
- **M/m/P**: Macro/micro/Momentum pillar scores, with AGREE/DIVERGE indicator
- **KILLED**: flashing red badge when st_killed == true
- Poll: 3 seconds via `get_smarttrend_diagnostics`

### 106.8 Configuration SmartTrend

All SmartTrend settings are persisted in AppSettings and editable via batch update. New fields added in S07b:

| Field | Default | Description |
|------|---------|-----------|
| `st_changepoint_reset_threshold` | 0.7 | Changepoint threshold for force reset to Flat |
| `st_trail_tighten_pct` | 0.05 | Tighten trailing stop per bar in Trail |
| `st_hysteresis_margin` | 5.0 | Anti-chatter edge Stalking->Flat |

Existing camps from S07a: `st_macro_weight`, `st_micro_weight`, `st_momentum_weight`, `st_entry_threshold`, `st_stalking_threshold`, `st_entry_thresholds[4]`, `st_stabilization_window`, `st_bayesian_decay`, `st_bayesian_halflife`, `st_dominant_cycle_filter`, `st_ofi_norm_range`, `st_mp_bps_range`, `st_cvd_scale`, `st_enabled`, `st_killed`.

### 106.9 Modified files

| File | Change |
|--------|-----------|
| `smarttrend.rs` | FSM, Engine, markers, stats, 17 tests, 3 config fields |
| `app_state.rs` | smarttrend: SmartTrendConfig + default |
| `state.rs` | AppSettings, settings_from_state, apply_settings, apply_runtime_config |
| `commands/settings.rs` | apply_smarttrend_field() with 21 fields |
| `commands/market_data.rs` | get_smarttrend_diagnostics command |
| `main.rs` | register command, SmartTrendEngine init |
| `index.html` | SmartTrend Live panel HTML |
| `styles.css` | SmartTrend panel CSS cyber-HUD |
| `titan_bootstrap.js` | initSmartTrendPanel() polling + rendering |

---

## 107. SMARTTREND SCORING + KILL SWITCH (S08) — Session 56

### 107.1 StrategyMode::SmartTrend

New variant added to `enum StrategyMode` in `state.rs`:

| Variant | Display | Parse |
|----------|---------|-------|
| `SmartTrend` | `SMARTTREND` | "SMARTTREND", "SMART_TREND", "ST" |

Toggle cycle: MathOracle -> GpuNexus -> Fusion -> SignalStrategy -> **SmartTrend** -> MathOracle

### 107.2 SmartTrend Pipeline Scoring

In `get_signal_markers_internal()`, when `mode == SmartTrend`:

1. `evaluate()` is run on each history bar (warmup 50 bars)
2. `evaluate()` is run on each history bar (warmup 50 bars)
3. Macro/Micro/Momentum inputs are built from oracle history, GPU scores, and candle data
4. FSM markers are converted to `SignalMarker` format:

| SmartTrend Marker | Chart Shape | dye | Text |
|-------------------|-------------|-------|------|
| Entry (Bullish) | arrowUp | `#00E5FF` | ST BUY |
| Entry (Bearish) | arrowUp | `#00E5FF` | ST SELL |
| exit | arrowDown | `#FFD700` | ST EXIT |
| ferry | about | `#BF00FF` | ST TRAIL |
| Stalking | about | `#FFD700` | ST STALK |
| ChangePointReset | about | `#FF8800` | ST BREAK |

SmartTrend mode skips the default exit signal generation (FSM handles exits).

### 107.3 Regime-Adaptive Entry Colors (B4)

Entry markers change their color intensity proportionally to the market regime:

| diet | dye |
|--------|-------|
| Trending | `#00E5FF` (full) |
| MeanReverting | `rgba(0,229,255,0.6)` |
| Chaotic | `rgba(0,229,255,0.5)` |
| Unknown | `rgba(0,229,255,0.4)` |

### 107.4 Kill Switch

**Backend:**
- `st_killed = true` => SmartTrend does not produce markers, autopilot blocks entries
- Persistent on disk in `SmartTrendConfig.st_killed`
- Re-activation requires confirm token `CONFIRM_ST_REACTIVATE`
- Autopilot guard: when `mode == SmartTrend && st_killed == true`, skip all entries with log warn

**Tauri Commands (smarttrend_cmd.rs):**

| Command | parameters | returns |
|---------|-----------|------------|
| `smarttrend_kill_switch` | `enable: bool, confirm_token: Option<String>` | `String` |
| `get_smarttrend_config` | -- | `SmartTrendConfig` |
| `update_smarttrend_config` | `field: String, value: String` | `String` |
| `get_smarttrend_session_stats` | -- | `SmartTrendStats` |
| `get_smarttrend_state` | -- | `SmartTrendDiagnostics` |

### 107.5 UI Kill Switch + Mode Button

**Kill Switch button** in SmartTrend Live panel:
- Red button "KILL" with confirm dialog (TitanToast.confirm)
- When killed: "REACTIVATE" text, green color, re-enable requires double confirm
- CSS cyber-HUD with hover glow and transitions

**Strategy mode selector** — button "ST" added to nexus-mode-toggle:
- Active: cyan border + glow
- Specific CSS: `nexus-mode-btn[data-mode="SMARTTREND"]`

### 107.6 Score Overlay (B1)

SmartTrend scores (0-100) are included in `SignalMarkersResponse.st_scores`:
- Structure `StScorePoint { time: i64, score: f64, state: String }`
- Frontend `_renderStScoreOverlay()` renders scores on oracle chart overlay
- Score centered at 0 (score - 50) for baseline view

### 107.7 Quick-Stats Banner (B2)

Persistent banner under SmartTrend Live panel:

```
+--[ Quick Stats ]------------------------+
|  WIN: 65%  BARS: 12  TRAIL: 45%  RST: 3 |
+-----------------------------------------+
```

- **WIN**: entries_reached_trail / entries_total
- **BARS**: average bars_per_state / entries_total
- **TRAIL**: same as WIN (trail hit rate)
- **RST**: changepoint_resets count
- Styled: val-green, val-cyan, val-magenta, val-gold

### 107.8 Mode Transition Toast (B3)

When user changes StrategyMode to/from SmartTrend:
- Toast cyber-HUD with status: "SMARTTREND ACTIVATED — Engine: READY/KILLED/DISABLED"
- Audio alert: `TitanAudio.play('mode_change')`
- On deactivation: "SMARTTREND DEACTIVATED — Switched to {mode}"

### 107.9 New tests S08

19 new tests (395 total):
- 6 SmartTrend engine: kill_switch_default, kill_switch_serde, killed_engine_state, strong_signal_markers, marker_kinds, diagnostics_stats
- 4 scoring: exit_marker_smarttrend, exit_marker_rejects_st_entries, smarttrend_scoring_neutral, smarttrend_colors
- 6 SmartTrend engine: kill_switch_default, kill_switch_serde, killed_engine_state, strong_signal_markers, marker_kinds, diagnostics_stats
- 108. SMARTTREND SENTINEL LOOP + ALERTS (S09) — Session 57

---

## 108. SMARTTREND SENTINEL LOOP + ALERTS (S09) — Session 57

### 108.1 Sentinel Loop Backend

New file: `src-tauri/src/loops/smarttrend_sentinel.rs`

The sentinel loop continuously scans all active assets (positions + HFT targets + signal assets), calls `SmartTrendEngine::evaluate()` on each one with real live data, detects FSM transitions and issues alerts.

**Adaptive cadence per symbol:**

| FSM status | Default range | Config field |
|-----------|-----------------|--------------|
| Flat | 30s | `st_sentinel_idle_secs` |
| Stalking | 5s | `st_sentinel_stalking_secs` |
| Entry/InTrade/Trail | 2s | `st_sentinel_hot_secs` |

The loop sleeps after the shortest cadence of all monitored symbols.

**Assembly live data:**

| Input | Source | Lock type |
|-------|-------|-----------|
| Oracle score/confidence/regime | `indicator_cache.last_oracle_snapshot()` | Mutex |
| OFI, microprice, spread, imbalance | `market.ofi_state` | ArcSwap (lock-free) |
| CVD | `market.cvd` | Mutex |
| GPU consensus | `nexus_consensus` | Mutex |
| Current price | `market.last_prices` | Mutex |

Registered in `watchdog.register("smarttrend_sentinel")` at startup
- Registered in `watchdog.register("smarttrend_sentinel")` at startup
- Heartbeat at each iteration with tick duration
- `spawn_loop_by_name("smarttrend_sentinel")` in watchdog_loop.rs
- Generation ID check: the loop exits if the current generation < boot generation

### 108.2 Symbol Priority Queue (B2)

Instead of linear iteration, symbols are evaluated by `BinaryHeap<SymbolPriority>`:

| member | Ranka | Priority |
|-------|------|-----------|
| InTrade | 4 | The tallest |
| Entry/Trail | 3 | |
| Stalking | 2 | |
| Flat | 1 | The lowest |

Within the same rank, symbols with a higher score are evaluated first. Reduces reaction latency on critical symbols.

### 108.3 Micro-Latency Tracking (B1)

Each `evaluate()` is timed with `Instant::now()` / `elapsed()`:
- `last_evaluate_us` stored per-symbol in the tracker
- `tracing::warn!` issued if duration > 5ms (cache miss or lock contention indicator)
- Average propagated through `sentinel-badge` Tauri event

**UI indicator** in Alert Log header:
- `< 1ms` → cyan (fast)
- `1-5ms` → gold (mid)
- `> 5ms` → red (slow)

### 108.4 Alert Severity Classification (B4)

```
AlertSeverity::Info     — Flat->Stalking, Stalking->Flat
AlertSeverity::Warning  — Entry, Trail, Stalking timeout
AlertSeverity::Critical — Exit, ChangePoint Reset, Kill
```

Backend issues severity in payload `smarttrend-alert`. Differentiated cooldown: INFO respects per-symbol cooldown (default 30s), WARNING/CRITICAL always emits.

### 108.5 Audio Alerts (6 new sounds)

| Sound | Eventos | Character |
|-------|-------|---------|
| `st_stalking` | Flat->Stalking | Short rising tone |
| `st_entry` | Entry confirmed | Double beep fast |
| `st_exit` | Exit signal | Descending tone |
| `st_trail` | Trail enabled | Short metallic tone |
| `st_changepoint` | Break regime | Glitch/static burst |
| Kill switch | Kill switch | Short alarm |

All generated via Web Audio API (oscillator + gain envelope), without external files. Cooldown 30s per symbol + event-type in frontend.

### 108.6 Alert Log UI

Container `#st-alert-log` under SmartTrend Live panel:

```
+--[ ALERT LOG ]--[ 245µs ]--[I][W][C]--[×]--+
| TIME     SYM   EVENT  SCORE  DIR            |
| 14:23:05 AAPL  ENTRY  72.3   LONG           |
| 14:22:51 TSLA  STALK  56.8   SHORT          |
| 14:20:12 NVDA  EXIT   45.1   --             |
+--------------------------------------------- +
```

- Max 20 LIFO alerts
- Severity toggles (I/W/C) in header — filters rendering and audio/toast
- Clear button (x) deletes the log
- Row border-left per severity: cyan (INFO), gold (WARNING), red (CRITICAL)
- Marker CSS classes: `mk-stalking`, `mk-entry`, `mk-exit`, `mk-trail`, `mk-changepoint`

### 108.7 Dashboard Badge (B3)

Persistent badge in the header-right area:

```
[ 5 | 2 ACT | TSLA InTrade ]
```

- Total symbols monitored
- Active Symbols (Non-Flat)
- Hottest symbol + status (with `data-flicker` animation)
- Updated via `sentinel-badge` Tauri event at each loop tick

### 108.8 Visual Alerts

- **Toast dedicated to SmartTrend**: `titan-toast-smarttrend` class with border-left per-marker
- **Screen glow**: WARNING = gold, CRITICAL = red (via `showScreenGlow()`)
- **Title flash**: `[ST ENTRY] AAPL` on Entry/Exit (10s)
- **Pulsing badge**: `st-alert-active` class on SmartTrend panel (3s animation)

### 108.9 Config Fields New

| Field | Default | Clamp | Description |
|------|---------|-------|-----------|
| `st_sentinel_idle_secs` | 30 | 5-300 | Cadence Flat |
| `st_sentinel_stalking_secs` | 5 | 1-60 | Stalking cadence |
| `st_sentinel_hot_secs` | 2 | 1-30 | Entry/InTrade/Trail cadence |
| `st_sentinel_alert_cooldown_secs` | 30 | 5-600 | Cooldown per-symbol INFO alerts |

Backward compatible — old settings files without these fields use the default values ​​via `#[serde(default)]`.

### 108.10 Tauri Commands New

| Command | Return | Description |
|---------|--------|-----------|
| `get_sentinel_status` | `SentinelStatus` | Sentinel status: running, symbols, badge |

### 108.11 Files

| File | CHANGE |
|--------|-----------|
| `loops/smarttrend_sentinel.rs` | NEW: sentinel loop + 10 tests |
| `loops/mod.rs` | Registration modules |
| `loops/watchdog_loop.rs` | spawn_loop_by_name branch |
| `smarttrend.rs` | 4 config fields + serde defaults |
| `indicator_cache.rs` | `last_oracle_snapshot()` + `OracleSnapshot` |
| `commands/smarttrend_cmd.rs` | `get_sentinel_status` command |
| `commands/settings.rs` | 4 fields in apply_smarttrend_field |
| `main.rs` | watchdog register + spawn loop + command |
| `titan_audio.js` | 6 new synthetic sounds |
| `titan_bootstrap.js` | Alert listener, log rendering, badge, severity |
| `index.html` | Alert log container, dashboard badge, severity toggles |
| `styles.css` | Alert log CSS, badge, severity, toast SmartTrend |

### 108.12 New tests S09

10 new tests (405 total):
- `sentinel_cadence_idle_30s`, `sentinel_cadence_stalking_5s`, `sentinel_cadence_hot_2s`
- `sentinel_state_transition_detection`
- `sentinel_cooldown_suppresses_repeat`
- `sentinel_killed_skips_evaluate`
- `sentinel_severity_classification`
- `sentinel_priority_queue_ordering`
- `sentinel_badge_empty_trackers`
- `sentinel_alert_severity_values`

---

## 109. SMARTTREND BOT MARKER-DRIVEN (S10) — Session 58

### 109.1 SmartTrendAccount — Virtual Sub-Account

Structure `SmartTrendAccount` in `smarttrend.rs` — separate capital tracking for SmartTrend trades:

| Field | Type | Description |
|------|-----|-----------|
| `allocated_capital` | f64 | Current capital (increase/decrease with PnL + topup/withdraw) |
| f64 | f64 | `realized_pnl_today` |
| PnL realized in the current session | f64 | PnL realized in the current session |
| `total_realized_pnl` | f64 | Cumulative Total PnL |
| `high_water_mark` | f64 | The highest capital achieved |
| `max_drawdown_pct` | f64 | Maximum drawdown as a percentage |
| `trade_count_today` | u32 | Number of trades today |
| `win_count_today` | u32 | Number of wins today |
| `loss_count_today` | u32 | Number of losses today |
| `total_trades` | u64 | Total trades all-time |
| u64 | u64 | All-time win totals |
| `avg_win` | f64 | Earnings average (running) |
| `avg_loss` | f64 | Running average |
| `paused` | bool | Bot paused flag |
| `breaker_triggered` | bool | Circuit breaker fired |

Key methods: `record_fill(pnl)`, `drawdown_pct()`, `hit_rate()`, `kelly_fraction()`, `compute_size_usd()`, `daily_reset()`.

Persistence: `data/titan_st_account.json` with .bak automatic backup.

### 109.2 Config Fields S10

7 new fields in `SmartTrendConfig` (all with `#[serde(default)]` backward-compatible):

| Field | Default | Description |
|------|---------|-----------|
| `st_risk_per_trade` | 0.01 | Fraction of capital per trade (1%) |
| `st_max_dd_pct` | 5.0 | Maximum drawdown before the circuit breaker |
| `st_bot_loop_ms` | 2000 | Bot loop cadence in ms |
| `st_bot_paused` | false | Bot starts paused |
| `st_kelly_enabled` | false | Activate Kelly sizing |
| `st_kelly_min_trades` | 20 | Minimum trades before Kelly |
| `st_max_kelly_fraction` | 0.25 | Cap Kelly fraction |

### 109.3 SmartTrend Bot Loop

NEW File: `src-tauri/src/loops/smarttrend_bot.rs`

**Architecture:** Dedicated loop (NOT autopilot) that reads the state of the SmartTrend engine from the sentinel and executes orders based on the FSM markers.

**Main Stream:**
1. Guard: skip if mode != SmartTrend, killed, paused
2. ST Circuit Breaker: if drawdown > st_max_dd_pct → force exit ALL + pause + emit alert
3. Read engine snapshot (state, direction, score, transition)
4. Entry: on SmartTrendStateKind::Entry with no existing position → compute_size_usd → send_order_internal with tag `ST|AUTO|{symbol}|{ts}`
5. Exit: on Flat transition from Trail/InTrade → close_position
6. Persist account periodically + dedup actions
7. Adaptive cadence: fast (1s) with positions, slow (3s) without

**Safety mechanisms:**
- Generation ID check (stale loop exits)
- Failed order cooldown tracking
- Action after (60s window)
- Circuit breaker with force exit + auto-pause

### 109.4 Trade Updates — ST Fill Detection

In `trade_updates.rs`:
- At each fill, check `entry_tag.starts_with("ST|")`
- If fill is closing (contra-direction): calculate PnL, call `SmartTrendAccount::record_fill(pnl)`, save to disk
- Issue event `smarttrend-fill` with `{ symbol, side, qty, price, tag, is_closing }`
- Structured Log `[ST FILL]`

### 109.5 Tauri Commands S10

5 new orders registered in `invoke_handler`:

| Command | parameters | Guard | Effect |
|---------|-----------|-------|-------|
| `smarttrend_topup` | `amount: f64` | `amount <= buying_power` | Increase allocated_capital |
| `smarttrend_withdraw` | `amount: f64` | `amount <= available - notional_open` | Decrease allocated_capital |
| `smarttrend_pause` | `paused: bool` | - | Toggle st_bot_paused + persist |
| `smarttrend_account_stats` | - | - | Return SmartTrendAccount JSON |
| `smarttrend_force_exit_all` | `confirm_token: String` | `"CONFIRM_ST_LIQUIDATE"` | Force close ALL ST AUTO positions |

### 109.6 UI — SmartTrend Account Panel

Panel `#st-account-panel` (`.cyber-panel`) with:
- **Metrics:** Capital, PnL Today (green/red), Total PnL, Drawdown %, Drawdown visual bar with threshold marker, Hit Rate, Trades Today/Total
- **Status badge:** ACTIVE (cyan) / PAUSED (orange) / BREAKER (pulsating red)
- **Controls:** Input amount + TOPUP + WITHDRAW + PAUSE/RESUME + FORCE EXIT ALL (with confirm dialog)
- **Polling:** 5s interval + immediate update on `smarttrend-fill` event

### 109.7 S10 Bonuses

**B1: Live Micro Inputs** — Sentinel `assemble_macro_input()` now uses real data from oracle snapshot (kalman_gain, price_surprise, changepoint_prob, run_length) instead of fixed placeholders. 4 new fields added in `Indicators` struct and propagated through `merge_streaming_oracle`.

**B2: Daily Reset** — `daily_reset.rs` calls `SmartTrendAccount::daily_reset()` at 09:30 ET. Zero pnl_today, trade_count_today, win_count_today, loss_count_today. Adjust initial_capital to current allocated_capital. Issue `smarttrend-daily-reset` event.

**B3: Sentinel-to-Bot Bridge** — When sentinel detects Entry marker, call `bot_wake().notify_one()`. Bot loop uses `sleep_or_wake()` with `tokio::select!` on sleep + Notify. Reduces entry latency from ~2s (normal cadence) to ~0s.

**B4: ST Positions Badge in Header** — Badge `#st-positions-badge` shows "ST: 3 +$45.20" (green) or "ST: 0 -$12.00" (red). It is updated on every `smarttrend-fill` event + polling 5s.

### 109.8 S10 Tests

20 new tests (274 total):
- SmartTrendAccount: `st_account_new_sets_capital`, `st_account_record_win`, `st_account_record_loss`, `st_account_drawdown_pct`, `st_account_hit_rate`, `st_account_avg_win_loss_ratio`, `st_account_daily_reset`, `st_account_serde_roundtrip`
- Kelly: `st_account_kelly_insufficient_trades`, `st_account_kelly_positive_edge`, `st_account_kelly_capped`
- Sizing: `st_account_compute_size_usd_base`, `st_account_compute_size_usd_kelly_boost`
- Config: `st_config_s10_defaults`, `st_config_s10_serde_roundtrip`, `st_config_s10_backward_compat`, `st_config_s10_validation_warnings`
- Bot: `action_kind_equality`, `engine_snapshot_defaults`, `now_epoch_ms_is_reasonable`

---

## 110. S11: AUTO/MAN Per Asset + Trade Panels

### 110.1 Per-Asset AUTO/MAN Mode

SmartTrend supports per-symbol execution mode: each asset can be individually configured as **AUTO** (automatic execution by bot) or **MAN** (manual execution by trade panel).

**Backend:**
- `st_asset_modes: HashMap<String, String>` in `AppState` / `AppSettings` — values: `"auto"` / `"man"`, default `"man"` (any unset asset is considered MAN)
- Automatic persistence on disk via `save_settings_to_disk()` at every change
- Integrated in `apply_settings()` and `settings_from_state()`

**Tauri Commands S11:**

| Command | Params | Guard | Description |
|---------|--------|-------|-----------|
| `smarttrend_get_asset_modes` | - | - | Returns complete HashMap |
| `smarttrend_set_asset_mode` | `symbol: String, mode: String` | AUTO->MAN blocked if AUTO position live | Set mode per asset |
| `smarttrend_set_all_modes` | `mode: String, confirm_token: Option<String>` | ALL AUTO requires `CONFIRM_ST_ALL_AUTO` | Set ALL assets to one mode |
| `smarttrend_get_trade_panel_data` | `symbol: String` | - | Assemble complete data (price, score, sizing, Kelly, pillars, mode) |

### 110.2 Bot Loop — Filtering on Asset Mode

- Bot entry gate: read `st_asset_modes` for active symbol; **skip entry if `mode == "man"`**
- Distinctive tag: `ST|AUTO|{symbol}|{ts}` vs `ST|MAN|{symbol}|{ts}`
- Circuit breaker: forces exit only on positions `ST|AUTO` (MAN — user manages)
- `force_exit_all_st_positions_filtered(auto_only: bool)` — `true` = AUTO only, `false` = all ST

### 110.3 Sentinel Events S11

- On marker Entry + asset mode MAN: emit `smarttrend-trade-prompt` with full payload (symbol, direction, score, state, price, sizing suggestion, Kelly, pillars, mode)
- On marker Exit/Trail + open MAN position: emit `smarttrend-exit-prompt` with (symbol, marker, direction, score, severity)

### 110.4 UI — Trade Panel SMARTTREND TRADE

Panel `#st-trade-panel` (`.cyber-panel`):
- Appears automatically on `smarttrend-trade-prompt` event (asset MAN)
- Auto-populated: Symbol, Direction badge (LONG green / SHORT red), Score, State FSM, Pillar scores, Price
- 3 sizing modes (toggle): **USD** / **SHARES** / **%CAP**
- Quick amounts: 100 / 250 / 500 / 1K
- Input manual sizing
- TP/SL optional (toggles + inputs)
- **EXECUTE**: invoke `submit_order` with tag `ST|MAN|{symbol}|{ts}`, toast confirmation
- **SKIP**: dismiss panel, log, toast
- Audio `st_entry` at appearance
- Countdown timer 60s -> 0s with auto-SKIP on expiration

### 110.5 UI — Exit Signal Panel

Panel `#st-exit-panel` (`.cyber-panel`):
- Appears on `smarttrend-exit-prompt` event (MAN position with Exit/Trail marker)
- Information: Symbol, Current PnL, Position duration, Exit reason (marker type)
- **CLOSE 100%** / **CLOSE 50%** / **CLOSE 25%** / **HOLD**
- HOLD: dismissed with log, reappears at next marker
- Audio `st_exit` on appearance

### 110.6 UI — ST Assets Table

Panel `#st-assets-panel` (`.cyber-hud-table`):
- Columns: SYMBOL / MODE / STATE / SCORE / DIR / LAST ALERT / PnL
- MODE: interactive badge `[AUTO]` (green) / `[MAN]` (orange)
  - Click AUTO->MAN: instant, without confirmation
  - Click MAN->AUTO: with `TitanToast.confirm()` ("Activate automatic trading on {SYMBOL}?")
  - Guard: AUTO asset with live position cannot switch to MAN
- Header: **ALL AUTO** / **ALL MAN** buttons
- Sort: LIVE positions first, then by FSM status (InTrade > Trail > Entry > Stalking > Flat)
- Poll at 5s
- Tooltip on hover: last 5 trades ST (B3)

### 110.7 Keyboard Shortcuts S11

| Shortcut | Trade Panel | Exit Panel |
|----------|-------------|------------|
| **Enter** | EXECUTE | CLOSE 100% |
| **Escape** | SKIP | HOLD |
| **V** | VIEW CHART | VIEW CHART |
| **1/2/3/4** | Quick sizing presets | - |

 Guard: shortcuts active only when the ST panel is visible (does not interfere with the rest).

### 110.8 Position Markers

SmartTrend positions display distinctive tag:
- `[ST] AUTO` — green badge, green border
- `[ST] MAN` — orange badge, orange border
- CSS: `.st-pos-tag`, `.st-pos-auto`, `.st-pos-man`

### 110.9 Bonuses S11

**B1: Countdown Timer + Urgency Indicator**
- Progress bar on trade panel: 60s -> 0s
- Color: cyan (60-30s) -> gold (30-10s) -> red (10-0s) with pulse animation
- At 0s: auto-SKIP with toast "Trade opportunity expired"

**B2: Smart Sizing Suggestion**
- Recommended sizing: `risk_per_trade * allocated_capital`, adjust Kelly if activated
- Display: "SUGGESTED: $350 (1.4% risk, Kelly 0.12)"
- Pre-select in input, user can modify

**B3: Trade History per Asset Tooltip**
- Hover on row of ST Assets Table
- Last 5 ST trades on that asset: data, direction, PnL, mode (AUTO/MAN)
- Hit rate per asset

**B4: Mode Change Audit Trail**
- `tracing::info!("[ST MODE] {symbol}: {old} -> {new} (user)")` on every change
- Event `smarttrend-alert` with marker `MODE_CHANGE` in Alert Log
- Immediate persistence on disk

### 110.10 Tests S11

16 new tests (441 total):
- Bot: `st_bot_skips_man_asset_logic`
- Tags: `st_tag_auto_vs_man`, `st_force_exit_auto_only_filter`
- Bot: `st_bot_skips_man_asset_logic`
- Sizing: `st_sizing_suggestion_basic`, `st_sizing_suggestion_kelly`
- Date: `st_panel_data_pillar_scores`, `st_trade_prompt_event_fields`, `st_exit_prompt_event_fields`
- Audit: `st_mode_audit_log_format`

---

## 111. S12a: SMARTTREND CONFIG TAB (Tab 13)

Tab 13 **SMARTTREND CFG** is the complete configuration interface for the SmartTrend engine. Displays and allows modification of all ST parameters, with live readout and FSM state visualization.

### 111.1 Access

- Click on the **SMARTTREND CFG** button in the tab bar (after SIGNAL CFG)
- Two-column responsive layout (stacking on screen below 1100px)

### 111.2 Section 1: PILLAR WEIGHTS (cyan border)

Configure the weights of the 3 SmartTrend pillars:

| Slider | Camp Backend | rank | Default |
|--------|-------------|-------|---------|
| Macro | `st_macro_weight` | 0-100 (%) | 40 |
| Micro | `st_micro_weight` | 0-100 (%) | 35 |
| Momentum | `st_momentum_weight` | 0-100 (%) | 25 |

- **Proportion bar**: normalized view with 3 colored segments (cyan/green/orange)
- **Radar Chart** (B2): canvas 120x120px with 3 axes, semi-transparent polygon, live update
- **Presets**: TREND FOLLOW (60/15/25), SCALP (25/45/30), BALANCED (34/33/33)
- **Validation**: the amount must be ~100%; red warning with animation if it is not

### 111.3 Section 2: ENTRY THRESHOLDS (orange border)

| Slider | Camp Backend | rank | Default |
|--------|-------------|-------|---------|
| Global Entry | `st_entry_threshold` | 10-100 | 65 |
| Stalking | `st_stalking_threshold` | 5-100 | 55 |
| HYSTEREZE | `st_hysteresis_margin` | 0-20 | 5.0 |

**Per-Regime Grid**: overrides per market regime:

| Regime | Field | Default |
|-------|------|---------|
| TRENDING | `st_entry_thresholds_0` | 60 |
| MEAN REV | `st_entry_thresholds_1` | 70 |
| CHAOTIC | `st_entry_thresholds_2` | 75 |
| UNKNOWN | `st_entry_thresholds_3` | 65 |

Validation: the stalking threshold must be lower than the entry threshold.

### 111.4 Section 3: MICROSTRUCTURE (green border)

| Control | Camp Backend | Type | Default |
|---------|-------------|-----|---------|
| Dominant Cycle Filter | `st_dominant_cycle_filter` | Toggle | MR |
| OFI Norm Range | `st_ofi_norm_range` | Slider 1-200 | 50 |
| Microprice BPS Range | `st_mp_bps_range` | Slider 1-50 | 10.0 |
| CVD Scale | `st_cvd_scale` | Slider 1-1000 | 100 |

Info box gold: "Micro pillar active only for streamer symbol with live L2 data"

### 111.5 Section 4: EXIT & TRAIL (orange border)

| Slider | Camp Backend | rank | Default |
|--------|-------------|-------|---------|
| Trail Tighten % / bar | `st_trail_tighten_pct` | 0-50% | 5% |
| Changepoint Reset | `st_changepoint_reset_threshold` | 0.10-1.00 | 0.70 |
| Stabilization Window | `st_stabilization_window` | 2-50 bars | 8 |

### 111.6 Section 5: SELF-CORRECTION (purple border)

| Slider | Camp Backend | rank | Default |
|--------|-------------|-------|---------|
| Bayesian Decay | `st_bayesian_decay` | 0.990-1.000 | 0.997 |
| Halflife (trades) | 10-1000 | 10-1000 | 200 |

**Live Readout** (update to 5s via `get_smarttrend_state`):
- Hit Rate (entries/trail ratio)
- Entries total
- Trail Hits
- Changepoint Resets
- Stalking Timeouts
- Last Transition

### 111.7 Section 6: STATE MACHINE LIVE (cyan border)

5 FSM boxes with the active status highlighted:

| member | Active Color | Animation |
|-------|---------------|----------|
| FLAT | Grey | -- |
| STALKING | Goldie | `neon-pulse` |
| ENTRY | Cyan | -- |
| IN TRADE | GREENE | `neon-pulse` |
| ferry | Magenta | -- |

Under the boxes: Direction (LONG/SHORT/--), Bars in State, Entry Price, Best Price, Combined Score, Agreement (AGREE/MIXED/DIVERGE).

Confirmation progress bar visible in Stalking status.

Refresh at 3s via `get_smarttrend_state`.

### 111.8 Action Bar

| Button | Action | Feedback |
|-------|---------|----------|
| SAVE CONFIG | Apply only fields modified in batch | Toast success + audio click |
| SMART DEFAULT | Preset BALANCED (34/33/33) | Toast info |
| RESET DEFAULTS | Restore all defaults | Confirmation + toast + audio confirmation |
| COMPARE (B3) | Current vs saved side-by-side overlay | Gold highlight diffs table |

### 111.9 Bonuses S12a

**B1: Config Diff Indicator**
- Cyan dot next to the label of each modified field (animation `neon-pulse`)
- Counter "N unsaved" in the action bar
- Toast warning gold when leaving tab-13 with unsaved changes

**B2: Pillar Weights Radar Chart**
- Canvas 120x120px with 3 axes (M/MI/MO)
- Concentric grid rings, colored axes
- Semi-transparent cyan data polygon with points on each axis
- Live update with every slider movement

**B3: Config Comparison Mode**
- Full-screen overlay with FIELD / CURRENT / SAVED / DELTA table
- The highlight differences with the gold background
- Dismiss with click overlay, X button, or Escape key

### 111.10 Files Involved

| File | Role |
|--------|-----|
| `src/index.html` | Tab button + tab-13 complete container |
| `src/styles.css` | CSS S12a (~350 lines) |
| `src/titan_bootstrap.js` | `initSmartTrendCfg()` (~380 lines) |
| `src/titan_keyboard.js` | Escape dismiss compare overlay |
| `src/titan_tabs.js` | Unsaved changes warning on tab switch |

### 111.11 Orders Used Bulls

| Command | Scope |
|---------|------|
| `get_smarttrend_config` | Initial load config |
| `update_smarttrend_config` | Save per field to SAVE |
| `get_smarttrend_state` | Live readout + FSM state |

### 111.12 Tests S12a

12 new tests (453 total):
- Serde: `st_config_serde_roundtrip`, `st_config_partial_json_backcompat`
- Validation: `st_config_validation_valid_default`, `st_config_validation_weight_sum_warning`, `st_config_validation_negative_weight_error`, `st_config_validation_threshold_ordering_error`
- Presets: `st_config_preset_trend_follow_values`, `st_config_preset_scalp_values`, `st_config_preset_balanced_values`
- Range: `st_config_per_regime_threshold_range`
- Diagnostics: `st_config_diagnostics_contains_all_fields`, `st_config_stats_default_zeros`

---

## 112. S12b: SMARTTREND UI ACTIONS + CHART OVERLAY

Extension of tab 13 with sections 7-10 (state changes), SmartTrend overlay chart, and EXPORT/IMPORT config.

### 112.1 Section 7: SMARTTREND ACCOUNT (cyan border)

SmartTrend virtual account management panel.

**Information Grid:**

| Field | Source | Refresh |
|------|-------|---------|
| ALLOCATED | `allocated_capital` | 5s |
| INITIAL | 5s | 5s |
| DAILY PnL | `realized_pnl_today` | 5s (green/red) |
| TOTAL PnL | `total_realized_pnl` | 5s (green/red) |
| HWM | `high_water_mark` | 5s |
| MAX DD% | `max_drawdown_pct` | 5s (red) |
| BETRAYED | `total_trades` | 5s |
| W/L | `total_wins / losses` | 5s |

**Actions:**

| Button | Order Taurus | Confirmation |
|-------|--------------|------------|
| TOP-UP | `smarttrend_topup` | No (validate amount > 0) |
| WITHDRAW | `smarttrend_withdraw` | No (validate amount > 0) |
| BREAKS / SUMMARY | `smarttrend_pause` | No (toggle) |
| Liquid | `smarttrend_force_exit_all` | Yes (TitanToast.confirm) |
| KILL SWITCH | `smarttrend_kill_switch` | Yes (double confirmation) |

**B1 Equity Micro-Chart**: Canvas 200x80px with equity curve cyan, gradient fill, HWM dashed gold, drawdown zones red. Fallback text if < 3 trades.

### 112.2 Section 8: SMARTTREND ASSETS (green border)

Interactive table with all SmartTrend symbols.

| Column | Description |
|---------|-----------|
| SYMBOL | The name of the asset |
| modeller | Badge AUTO (cyan) / MAN (gold), click to toggle |
| Member | FSM colored badge (Flat gray, Stalking cyan, Entry cyan pulse, InTrade green, Trail gold) |
| slag | SmartTrend Combined Score |
| DIR | Direction (Bullish/Bearish/Neutral) |
| Barse | Bars in the current state |
| In Play | Green dot if there is a position `ST | open |

**Toolbar**: Input ADD symbol (add with MAN mode), ALL AUTO (with confirmation `CONFIRM_ST_ALL_AUTO`), ALL MAN.

**B2 Asset Heatmap**: Subtle background on each row based on score — red (<30), neutru (30-60), cyan (60-80), green (>80).

### 112.3 Section 9: SMARTTREND BACKTEST (orange border)

Quick backtest on the active symbol.

- **Period slider**: 7-90 days (default 30)
- **RUN BACKTEST**: Invoke `run_signal_backtest` with SmartTrend modes
- **Progress bar**: Animated, event listening `signal-bt-progress`
- **Results grid**: Trades, Win Rate, Return%, Max DD, Sharpe, Profit Factor, Expectancy

**B3 Backtest Sparkline**: Canvas 300x60px with equity curve green, HWM dashed gold, drawdown zones red, hover tooltip with value.

### 112.4 Section 10: SENTINEL ALERTS (purple border)

Sentinel alert configuration and monitoring.

| Control | Camp Backend | Default |
|---------|-------------|---------|
| Idle cadence | `st_sentinel_idle_secs` | 30s |
| Stalking cadence | `st_sentinel_stalking_secs` | 5s |
| Hot cadence | `st_sentinel_hot_secs` | 2s |

**Toggles**: Audio ON/OFF, Visual ON/OFF, Severity filter (Info/Warning/Critical).
**Alert Log**: The last 20 alerts, scrollable, with severity badges (Info cyan, Warning orange, Critical red).
**Status Badge**: RUNNING (green) / STOPPED (red) via `get_sentinel_status`.

**B4 Sentinel Heatmap Badge**: Horizontal strip with 3 proportional segments (Info/Warning/Critical), pulse animation on the dominant segment, tooltip with exact counts.

### 112.5 Chart Overlay SmartTrend

| Elements | Description |
|---------|-----------|
| States Strip | FSM Markers |
| Arrow up cyan (Entry), arrow down red (Exit), circle gold (Trail) on the candlestick at transitions | Arrow up cyan (Entry), arrow down red (Exit), circle gold (Trail) on the candlestick at transitions |
| CVD Toggle | Checkbox in strip for magenta CVD overlay |

The strip is automatically hidden if SmartTrend is not enabled. Refresh at 3s.

### 112.6 EXPORT / IMPORT Config

| Button | Action |
|-------|---------|
| EXPORT | Download full JSON `SmartTrendConfig` with `schema_version: 1` |
| IMPORT | JSON upload, client + server validation (serde + range checks), confirmation, reload |

Format: Full JSON with all `SmartTrendConfig` fields. Unknown fields are ignored during deserialization (backward compatible with `#[serde(default)]`).

### 112.7 Orders New Bulls S12b

| Command | Scope |
|---------|------|
| `smarttrend_export_config` | Returns complete JSON config with `schema_version` |
| `smarttrend_import_config` | Receives JSON string, validates, applies and saves |

### 112.8 Tests S12b

8 new tests (461 total):
- Diagnostics: `diagnostics_includes_entry_best_price`, `diagnostics_entry_price_after_entry`
- Export/Import: `export_config_roundtrip`, `import_config_rejects_invalid_json`, `import_config_validates_negative_weights`, `import_config_validates_threshold_ordering`, `export_config_with_schema_version`, `import_full_config_roundtrip`

---

## 113. S14: EXECUTION RESILIENCE (Session 63)

Complete hardening of the order execution cycle, health observability, and data-stream robustness.

### 113.1 Order Lifecycle Tracking

Each order generates a unique `client_order_id` (UUID v4). The broker's response is parsed for `broker_id` and `status`. Automatic retry 1-2x for transient network/server errors (5xx, timeout). Event `order-lifecycle` is emitted on each attempt.

| Field | Type | Source |
|------|-----|-------|
| UUID v4 | UUID v4 | Locally generated |
| `broker_id` | String | Parsed from Alpaca's answer |
| `retry_count` | 0-2 | 0-2 |
| `status` | String | `new`, `accepted`, `filled` etc. |

### 113.2 Pending Order Watchdog

Orders sent are tracked locally in `TradingState::pending_orders`. The watchdog runs in an autopilot loop and automatically cancels orders:

- **Stale:** older than `pending_order_timeout_secs` (default 120s, configurable)
- **Generation Mismatch:** `generation_id` different from the current generation (after respawn loop)

Cancellation is made via Alpaca DELETE `/v2/orders/{broker_id}`. Orders are removed from the pending list at `fill`, `partial_fill`, `rejected`, or `canceled`.

### 113.3 Equity Sync Health Emitter

| Event | Condition | Payload |
|-----------|----------|---------|
| `equity-sync-degraded` | Moderate consecutive errors | `{ consecutive_failures, msg }` |
| `equity-sync-critical` | Consecutive errors >= threshold | `{ consecutive_failures, msg }` |

Frontend: Badge **EQUITY STALE** (gold with pulse) in the cockpit strip when a degraded/critical event is received.

### 113.4 Trade Updates Resilience

| Metra | Description |
|---------|-----------|
| `reconnect_count` | Number of WS reconnections since the start |
| `trade-updates-health` | Event issued with `connected: bool, reconnect_count, last_disconnect_reason` |

Front end: Badge **WS DISCONN** (red with pulses) in cockpit strip when `connected = false`.

Fills received on WS correlate `order.id` with `pending_orders` to remove them from tracking.

### 113.5 Exit Features

`TradeRecord` was extended with `exit_features: Option<TradeFeatures>` — snapshot of the market conditions at the time of closing the position (price, volatility, volume, spread, RSI, VWAP). Complements `entry_features` for post-trade analysis (entry vs exit delta).

### 113.6 Bounded Streamer Channel

The Herfindahl-Hirschman Index calculation now uses `qty * live_price` (from `state.market.last_prices`) instead of `qty * entry_price`. Structured logging with `hhi_pre`, `hhi_post`, `blocked`.

### 113.7 HHI Notional Fixed

The Herfindahl-Hirschman Index calculation now uses `qty * live_price` (from `state.market.last_prices`) instead of `qty * entry_price`. Structured logging with `hhi_pre`, `hhi_post`, `blocked`.

### 113.8 Cockpit Strip (B1)

The persistent bar under the header with aggregated metrics and health badges:

| Elements | Source | Refresh |
|---------|-------|---------|
| EQUITY | `get_settings` → equity | 5s poll |
| P&L | `get_settings` → Calculated P&L | 5s poll |
| POS | `get_settings` → positions.len | 5s poll |
| modeller | `get_settings` → auto/manual | 5s poll |
| BREAKER | `breaker-triggered` event | Eventos |
| GENDER | `get_settings` → generation | 5s poll |
| EQUITY STABLES | `equity-sync-degraded/critical` | Eventos |
| WS DISCONN | `trade-updates-health` | Eventos |

Styling: `.cockpit-strip` with `border-bottom: 1px solid var(--border-subtle)`, mono font, responsive collapse on mobile.

### Triangulation of fills on the chart via LightweightCharts markers:

Triangulation of fills on the chart via LightweightCharts markers:

| Type | dye | dye | Text |
|-----|-------|-------|------|
| BUY | arrowUp | lime(--neon-green) | `B ${qty}` |
| SELL / sell | arrowDown | lime(--neon-red) | `S ${qty}` |

Markers are persistent in `window._titanFillMarkers[]`. Refresh at 3s via `setInterval`. The timestamp is converted from ISO 8601 to epoch seconds UTC.

### 113.10 Journal Min P&L Filter (B3)

Input `j-filter-minpnl` added to the filter bar of the journal. The value is passed to `get_journal_trades` as parameter `min_pnl`. Reset also clears this field.

### 113.11 Time-in-Trade Column (B4)

**TIME** column added in the positions table (cyber-hud-table):

| term | Color | CSS class |
|--------|---------|-----------|
| 0-5 min | dim | `pos-time-dim` |
| 5-30 min | cyan | `pos-time-cyan` |
| 30 minutes - 2 hours | Goldie | `pos-time-gold` |
| > 2h | orange + flicker | `pos-time-orange` |

Calculation: `Date.now()/1000 - position.opened_at`. Format: `Xh Ym` / `Xm Ys` / `Xs`.

### 113.12 Involved Files S14

| File | modifications |
|--------|-----------|
| `src-tauri/Cargo.toml` | +uuid dependency |
| `src-tauri/src/app_state.rs` | +broker_order_id on Position, +pending_order_timeout_secs |
| `src-tauri/src/state.rs` | Bounded Sender, +pending_order_timeout_secs settings |
| `src-tauri/src/trading_state.rs` | `src-tauri/src/commands/order_exec.rs` |
| `src-tauri/src/commands/order_exec.rs` | +OrderResult, UUID gen, retry loop, broker_id parse |
| `src-tauri/src/commands/trading.rs` | Adapted to OrderResult |
| `src-tauri/src/commands/misc.rs` | +exit_features/broker_order_id on manual record |
| `src-tauri/src/commands/account.rs` | try_send bounded channel |
| `src-tauri/src/commands/settings.rs` | try_send bounded channel |
| `src-tauri/src/loops/autopilot.rs` | Pending watchdog, exit features snapshot, HHI fixed |
| `src-tauri/src/loops/trade_updates.rs` | Health emitter, fill correlation, pending removal |
| `src-tauri/src/loops/equity_sync.rs` | Health emitter events |
| `src-tauri/src/loops/watchdog_loop.rs` | Bounded channel |
| `src-tauri/src/loops/position_recon.rs` | `src-tauri/src/trading_engine.rs` |
| `src-tauri/src/trading_engine.rs` | +broker_order_id |
| `src-tauri/src/streamer.rs` | Bounded Receiver |
| `src-tauri/src/main.rs` | Bounded channel(256) |
| `src-tauri/src/journal.rs` | +exit_features, +broker_order_id |
| `src/index.html` | Cockpit strip, positions cyber-hud-table + TIME col |
| `src/styles.css` | Cockpit strip CSS, time-in-trade CSS |
| `src/titan_bootstrap.js` | Cockpit strip init, health listeners, fill markers |
| `src/titan_chart_layout.js` | applyFillMarkers() |
| `src/titan_journal.js` | minPnl filter wire |
| `src/titan_positions.js` | TIME column rendering + color coding |

---

## 114. S15: TRADING EDGE UPGRADES (Session 64)

Four trading logic improvements + four UI/UX bonuses, all with consistent cyber-HUD style.

### 114.1 Max-Hold Decay Exit (T1)

Progressive exit mechanism that tightens the trail as the position approaches `max_hold_secs`. It prevents dead-weight positions that consume capital without action.

**How ​​it works:**
1. When `elapsed > max_hold_secs * max_hold_decay_start_pct` (default 50% of max hold): the trail starts to tighten proportionally with `progress` (0..1)
2. Formula: `effective_trail = base_trail * (1.0 - progress * decay_trail_tighten * 100)`
3. When `elapsed >= max_hold_secs`: force exit with log `[MAX_HOLD_DECAY] Force exit`

**Settings:**
- `max_hold_decay_enabled` (default: true) — Enables/disables the mechanism
- `max_hold_decay_start_pct` (default: 0.5, range 0.2-0.9) — At what percentage of max_hold does decay begin
- `max_hold_decay_trail_tighten` (default: 0.003, range 0.001-0.02) — Collection rate per tick

### 114.2 MTF Confirmation Gate (T2)

Confirmation gate on higher timeframe before entry. Check if the proposed direction (BUY/SELL) is aligned with the trend on HTF.

**Soft Mode (default):** Penalizes the score with `mtf_gate_penalty` points if HTF does not confirm.
**Hard Mode:** Completely blocks the entry if HTF does not confirm.

**Shared helper:** `is_htf_aligned(candles, oracle_weights, is_buy) -> (aligned, htf_score)` in `strategy.rs`. Automatic LTF->HTF mapping: 1m->5m, 5m->15m, 15m->1h, 1h->4h, 4h->1d.

**Settings:**
- `mtf_gate_enabled` (default: true)
- `mtf_gate_mode` (default: "soft", options: "soft"/"hard")
- `mtf_gate_penalty` (default: 15.0, range 1-50)
- `mtf_gate_htf` (default: "15m") — Manual override of HTF

### 114.3 Tiered Scanner Top-K (T3)

Pre-ranking pipeline in the scanner that reduces expensive scoring (oracle + GPU) on illegal tickers.

**Pipelines:**
1. Pre-rank by dollar volume (close * volume) DESC
2. **Tier 1** (top `tier1_count`): fully scored — always
3. **Tier 2** (following `tier2_count`): scored only if RVOL >= gate OR change% >= 3%
4. Merge results, re-sort by prefilter score, truncate at `max_candidates`

**Settings on ScanParams:**
- `tiered_scan_enabled` (default: true)
- `tier1_count` (default: 50, range 10-500)
- `tier2_count` (default: 100, range 10-500)

### 114.4 Risk Parity Manual Orders (T4)

HHI gates on manual orders in `send_order_internal`. Similar to the existing gate on autopilot, but with a more relaxed limit.

**Calculation:** HHI post-trade = Σ(notional_i / total_notional)². If it exceeds the limit, the order is rejected with a descriptive error.

**Settings:**
- `hhi_gate_manual_orders` (default: true)
- `max_concentration_hhi_manual` (default: 0.35, range 0.1-1.0) — More relaxed than autopilot (0.30)

### 114.5 Regime Band on Chart (B1)

Visual strip under the chart showing the current market regime, updated at 5s.

| Regime | Color | Labelle |
|-------|---------|-------|
| Trending Up | Neon Green glow | TRENDING UP |
| Trending Down | Neon Red glow | TRENDING DOWN |
| Chaotic | Neon Orange glow | CHAOTIC |
| Quiet/Range | Dim border | QUIET/RANGE |

The oracle score is displayed to the right of the strip.

### 114.6 Journal Hour-of-Day PnL Heatmap (B2)

Grid canvas 5 days (Mon-Fri) x 13 30-minute intervals (9:30am-4:00pm ET).

Each cell shows the total PnL on that bucket, with gradient color proportional to the magnitude (green=profit, red=loss). Tooltip on hover: day, interval, PnL, number of trades, win rate.

Positioned in Journal tab as `.cyber-panel` under existing KPIs.

### Mini-panel in Nexus with 3 strategy indicators and agreement meter.

Mini-panel in Nexus with 3 strategy indicators and agreement meter.

**Indicators:** Oracle, GPU, SmartTrend — each with colored badge LONG/SHORT/NEUTRAL.
**Agreement Meter:** Bar 0-100%: 3/3 agree = 100% green, 2/3 = 66% cyan, 1/3 = 33% gold, 0 = red.
**Stale Detection:** If an indicator has not updated > 60s, the badge shows "STALE" with animation `data-flicker`.

Powered by Taurus events `nexus-update` and `smarttrend-state`.

### 114.8 Latency Panel V2 (B4)

Upgrade of the existing latency panel with graphic views.

**Sparklines:** Canvas 120x28 per metric (Order, GPU, Stream), ring buffer of 60 samples. Cyan line with colored dot at the last value (green < 50%, gold 50-80%, red > 80% of max).

**Histogram:** Canvas 160x40 with 5 distribution buckets: <1ms (green), 1-5ms (cyan), 5-20ms (gold), 20-100ms (orange), >100ms (red). Accumulate global counter as new data appears.

### 114.9 Involved Files S15

| File | modifications |
|--------|-----------|
| `src-tauri/src/state.rs` | +12 settings S15, +10 default functions, +settings_from_state sync, +apply_settings sync, +2 tests |
| `src-tauri/src/app_state.rs` | +12 fields S15 on AppState, +defaults |
| `src-tauri/src/commands/settings.rs` | +12 batch update handlers S15 |
| `src-tauri/src/loops/autopilot.rs` | Max-Hold Decay exit logic, MTF gate in evaluate_candidate_entry |
| `src-tauri/src/strategy.rs` | +is_htf_aligned, +ltf_to_htf, +6 tests |
| `src-tauri/src/commands/order_exec.rs` | HHI gate manual orders |
| `src-tauri/src/scanner.rs` | Tiered scan pipeline, +3 ScanParams fields, +3 tests |
| `src-tauri/src/trading_engine.rs` | +fixed broker_order_id on tests |
| `src/index.html` | Regime strip, journal heatmap canvas, strategy agreement panel, latency sparklines+histogram |
| `src/styles.css` | +120 CSS lines: regime strip, heatmap tooltip, strategy agreement, latency sparklines |
| `src/titan_bootstrap.js` | Regime band init, Strategy agreement init |
| `src/titan_journal.js` | Hour-of-day PnL heatmap (canvas bucketing + rendering + tooltips) |
| `src/titan_rightbar.js` | Latency V2 sparklines + histogram (ring buffer + canvas drawing) |

---

## §115 — S16: UX Polish + Cleanup (Session 65 — Final roadmap session)

### 115.1 Cockpit Strip V2

- **GEN Badge** — sync with `session_generation` from `get_risk_status` at 5s poll
- **BREAKER Badge** — initial polling + pulse animation when > Normal
- **W/R Badge** — Win rate from session analytics (>=55% green, >=45% cyan, <45% orange)
- **UPTIME Badge** — Elapsed time HH:MM:SS since session start

### 115.2 Unified Chart Markers Engine

`window._titanMarkerEngine` — unique manager of fills + signals + SmartTrend. Debounce 100ms, FIFO head 50, pruning >24h.

### 115.3 Latency Histogram Rolling Window

Ring buffer 300 samples (5 min). The distribution no longer increases indefinitely.

### 115.4 Fill Quality Columns HFT Targets

3 columns: SLIP (avg slippage bps), T2F (avg duration), TRADES (count). Backend `get_fill_quality_stats`. Polling 30s.

### 115.5 PiP Mini Dashboard (B1)

Floating HUD 260px, draggable, shortcut P, ​​opacity slider, minimize. Reuse cockpit data.

### 115.6 LLM Sentiment Batch + Decay (B2)

Batch scoring N headlines, exp decay halflife on confidence. Settings: `llm_batch_size` (5), `sentiment_halflife_secs` (3600).

### 115.7 Mode Audit Trail (B3)

`Vec<ModeTransition>` chapter 100, timeline panel in Nexus, color coding per mode. Command `get_mode_audit_trail`.

### 115.8 Health Score Badge (B4)

`HP:XX%` in the cockpit. 6 sub-scores: equity sync, WS, breaker, latency, loops, fill rate. Polling 10s.

### 115.9 Module Entrypoint + $el() Rollout + CSS Cleanup

`titan_init.js`, `registerTabInit()`, 464 getElementById → $el(), CSS var() tokens.

---

## 106. Wave 1 MUST-FIX — 9 Critical Fixes + 4 Bonuses (Session 67)

### 106.1 Fill Deduplication (CRIT-11)

Alpaca reports `filled_qty` and `filled_avg_price` **cumulative** per order. The previous code treats them as incremental, leading to inflated positions ~50% on partial fill -> fill sequences.

**Solution:** HashMap `fill_cumulative` per order ID on `TradingState`. At each fill event, `delta_qty = cum_filled_qty - prev_qty` is calculated and ONLY the delta is applied. Fills with delta <= 0 are automatically suppressed and counted in `fills_deduplicated`.

**Cockpit badge:** `DEDUP:N` (cyan, visible only when N > 0) shows in real time how many duplicate fills have been prevented.

### 106.2 Bot Exit Operator Precedence (CRIT-12)

The exit condition in SmartTrend bot had `state == Flat && transition.contains("Trail->Flat") || transition.contains("InTrade->Flat")` -- without brackets, `&&` binds more tightly than `||`, so `InTrade->Flat` triggers exit even if the state was NOT Flat.

**Fix:** `state == Flat && (transition.contains("Trail->Flat") || transition.contains("InTrade->Flat"))`

### 106.3 Circuit Breaker Latch (CRIT-1)

The breaker sets `account.paused = true` and `account.breaker_triggered = true`, but the gate at the beginning of the loop ONLY checks `st_config.st_bot_paused` from the settings. After the first iteration, the bot could continue trading.

**Fix:** Check `account.paused || account.breaker_triggered` at the beginning of the loop + persist `settings.smarttrend.st_bot_paused = true` after the breaker wire.

### 106.4 Position Recon Trail-Stops (CRIT-15)

Reconciliation lost the trail high when the price rose above the old high (copy `existing.highest_price` only if it was higher than `current_price`, but not if `current_price` was the highest).

**Fixed:** `highest_price = existing.highest_price.max(current_price)`, `lowest_price = existing.lowest_price.min(current_price)`. It also preserves `broker_order_id`.

### 106.5 Exit Inflight TTL (CRIT-16)

`close_position` inserts in `exit_inflight` but the remove was incomplete — it was missing on the error path and canceled/rejected events. Symbols could be stuck permanently.

**Fix:** TTL 120s cleanup in trade_updates loop + explicit remove on: complete fill, send_order error, rejected/cancelled events.

### 106.6 Lock Order (CRIT-21)

`smarttrend_kill_switch` take engine -> settings, `smarttrend_pause` take settings -> engine = ABBA deadlock potential.

**Fix:** Standardized order in ALL commands: **settings(1) -> smarttrend_engine(2) -> st_account(3)**. NEVER vice versa.

### 106.7 API Rate Limit (CRIT-13)

`cancel_open_orders_for_symbol` was doing DELETE to Alpaca WITHOUT `acquire_api_permit`, generating 429 rate limit rejections.

**Fix:** Added permit + tracking `record_api_call()`. Cockpit badge `API:N/200` with color coding.

### 106.8 AssetCheck JSON (CRIT-5)

If HTTP answered 200 but JSON parse failed, `if let Ok(json)` was skipped and `is_fractionable` remained `true`. The order could go on ticker delisted.

**Fix:** `match` explicitly on JSON parse. On Err: reject in hard mode, `is_fractionable = false` in soft mode.

### 106.9 Generation ID (HIGH-24)

`generation_id` was read AFTER HTTP POST, so a `bump_generation()` during the request attached the wrong generation to PendingOrder.

**Fix:** Moved `state.current_generation()` to the beginning of the `send_order_internal` function, before any await.

### 106.10 Trail-Stop Integrity Indicator (B4)

New column `TRAIL` in positions table with badge `TRAIL OK` (green) / `TRAIL WARN` (red pulse). Checked at every cockpit poll (5s) by `_titanTrailHealth`. Visually confirm that trail extremes are correct after reconciliation.

---

## 107. Wave 2A SmartTrend Safety — 8 Fixes + 4 Bonuses (Session 68)

Session 68 fixed 8 critical safety bugs in the SmartTrend engine and added 4 cyber-HUD bonuses. Total: 516 tests (+10 new), clippy clean.

### 107.1 Sentinel Generation Check (CRIT-3)

The Sentinel compares `current_generation() < boot_gen`, but the generation is monotonically increasing (increases with each respawn). After a watchdog respawn, the old sentinel did NOT stop — it ran like a zombie alongside the new one.

**Fix:** Replaced `<` with `!=`, identical to `smarttrend_bot.rs`. When the generation changes (in any direction), the sentinel exits with a log warning and gives way to the new instance.

### 107.2 Multi-Symbol ST Exit (CRIT-6)

The bot exit checks `pos.symbol == settings.active_symbol`. SmartTrend positions on other symbols are orphaned — they never got auto-exit when the FSM went Flat.

**Fix:** Deleted the `active_symbol` guard from the exit block. ALL positions with `strategy_mode == "ST|..."` receive exit when the engine is Flat. Entries remain gated on `active_symbol` (correct — only the current symbol in the chart is entered). The guard `recently_acted` per symbol and `freshness_window_ms` remain as an anti-spam safety net.

### 107.3 Kelly Bayesian Loss Floor (CRIT-14)

With zero losses (100% win rate), `avg_loss <= 0` => `b = 0` => `kelly_fraction = 0`. Paradoxically, the bot does not measure positions when performing at its maximum.

**Fix:** Bayesian floor on `avg_loss`: if `avg_loss <= 0`, use `max(avg_win * 0.1, 1e-9)` as conservative prior. Kelly returns a positive fraction (capped at `max_fraction`), allowing proportional sizing even with a perfect track record.

### 107.4 Kill Switch Import Guard (CRIT-20)

`import_config` replaced all of `SmartTrendConfig` in JSON, including `st_killed`. A JSON file with `st_killed: false` disarms the silent kill switch.

**Fix:** After parsing the import, the safety fields are forced from the current runtime: `imported.st_killed = settings.st_killed` and `imported.st_bot_paused = settings.st_bot_paused`. These fields can be modified ONLY through the dedicated commands (`smarttrend_kill_switch`, `smarttrend_pause`).

### 107.5 Entry State Persist Bars (HIGH-11)

The FSM was doing `Entry -> InTrade` immediately at the next `evaluate()`. The bot had only one eval window to see the Entry status. If the loop was desynchronized (async timing), the complete entry was missed.

**Fix:** New config field `st_entry_persist_bars` (default 3). Entry persists until:
- `bars_in_state >= st_entry_persist_bars` (automatic timeout)
- The bot confirms that it has placed the order (sets `entry_acted = true` on engine)

### 107.6 entries_total Double Count (HIGH-13)

`record_transition` increments `entries_total` and `Stalking->Entry` AND on `Entry->InTrade`. A single trade = 2 entries in statistics, corrupting trail_hit_rate.

**Fix:** Counts only transition IN `Entry` (not `InTrade`). One transition per trade.

### 107.7 NaN Weight Firewall (HIGH-15)

If a weight or score was NaN, `sum <= 0.0` si `adj_sum > 0.0` did not detect NaN (`NaN > 0` = false in IEEE 754). NaN propagates in `combined` score, affecting entry/exit decisions.

**Fix:** Guards NaN at 3 levels:
1. `normalize_weights`: `!sum.is_finite()` => fallback to defaults (0.40, 0.35, 0.25)
2. Individual pillar scores: `!score.is_finite()` => sanitized at 50.0 (neutral) + confidence 0
3. Combined final score: `!combined.is_finite()` => 50.0 (neutral)

Counter `nan_sanitized_count` on engine incremented at each sanitization, exposed in `get_risk_status`.

### 107.8 Zombie Loops AbortHandle (HIGH-20)

`main.rs` spawns loops with `tauri::async_runtime::spawn()` without `abort_and_replace`. At the first respawn via watchdog, there are no old handles to abort — the original task continues alongside the new one.

**Fix:** Macro `spawn_with_handle!` records `AbortHandle` for ALL loops at startup through `watchdog.abort_and_replace()`. On respawn, the old handle is correctly aborted.

### 107.9 ST SHIELD Safety Dashboard (B1)

Cyber-HUD panel in the sidebar with 8 safety metrics:

| Metrics | Values | Source |
|---------|--------|-------|
| KILL | OFF (green) / ON (red pulse) | engine.config.st_killed |
| BREAKER | NORMAL / PAUSED / TRIPPED | st_account |
| DEDUP | counter (gold if > 0) | fills_deduplicated |
| ZOMBIE | counter (red if > 0) | watchdog.dead_loops() |
| NaN | counter (red if > 0) | nan_sanitized_count |
| KELLY | fraction 0.00-0.25 | kelly_fraction() |
| GEN | #N | session_generation |
| ORPHANS | counter (red if > 0) | orphan detection |

Shield score composite 0-100%: penalize kill (-50), breaker tripped (-30), zombies (-20), orphans (-15), NaN (-10).

### 107.10 Generation Timeline Strip (B2)

Visual strip in the cockpit showing the history of generations. Each generation displays the ID and duration. The last generation pulses with `neon-pulse`. Ring buffer of max 10 entries.

### 107.11 NaN Firewall Monitor Badge (B3)

Badge `NaN:0` (green) / `NaN:N` (red pulse) in cockpit strip. It shows in real time how many NaN values ​​were intercepted and sanitized by the firewall in FIX-7.

### 107.12 Orphan Position Detector (B4)

New column `ORPHAN` in positions table, visible only for SmartTrend positions. Badge `OK` (green) confirms that the position has a valid exit path after FIX-2 (multi-symbol exit). Badge `ORPHAN` (red pulse) indicates positions without an exit path.

---

## 108. Wave 2B SmartTrend Correctness — 9 Fixes + 5 Bonuses (Session 69)

Session 69 fixed 9 correctness bugs in the SmartTrend engine and added 5 cyber-HUD bonuses. Total: 526 tests (+10 new), clippy clean.

### 108.1 Hysteresis Validation (HIGH-12)

`st_hysteresis_margin` could be configured high enough for the FSM to remain stuck in Stalking (hysteresis >= stalking_strength/2, `drop_strength` became 0, exit condition was impossible). Fix: semantic validation in `validate_smarttrend_config` + runtime clamp in `max_safe_margin`. `drop_strength` minimum 0.1.

### 108.2 Sentinel Per-Symbol Oracle Inputs (HIGH-14)

The Sentinel used `last_oracle_snapshot()` which returned the last accessed entry from the (global) cache. With multi-symbol evaluation, all symbols received the oracle data of the last symbol evaluated. Fixed: new `oracle_snapshot_for(symbol)` with per-symbol lookup on prefix `symbol:` in cache keys. Fallback to global when there is no per-symbol data. `nexus_consensus` remains global (documented architectural limitation).

### 108.3 Circuit Breaker Reset Race (HIGH-16)

`reset_circuit_breaker` only resets `risk.circuit_breaker` but NOT `st_account.breaker_triggered`, `st_account.paused`, `settings.st_bot_paused`. `daily_reset` resets `breaker_triggered` but not `paused`. Fixed: both functions now fully reset the ST state. `daily_reset` sets `paused = false` only when `breaker_triggered` was active (preserves intentional manual pauses).

### 108.4 Exit Floor Dead Code (HIGH-21)

Line `local_stop = local_stop.min(-floor).max(local_stop)` was mathematically a no-op (`min(x, -floor)` followed by `max(result, x)` = `x`). Fixed: deleted the no-op, kept only the functional checks `if abs < floor`. Adaugat guard `exit_floor_pct > 0.0`.

### 108.5 Max-Hold Decay 100x (HIGH-22)

The `tighten_factor = 1.0 - progress * trail_tighten * 100.0` formula made the `0.003` parameter act like `0.3` per progress unit (100x more aggressive than configured). Fixed: deleted `* 100.0`, default changed from `0.003` to `0.3`, clamp range adjusted `[0.05, 0.8]`. The actual behavior is identical, but the parameter now correctly reflects its scale.

### 108.6 Lyapunov Live vs History (HIGH-25)

Live `calculate()` calculates Lyapunov on the entire series of closes (global). History `calculate_history()` used window rolling of 200 bars. Divergence: live could see "chaos" on 500 bars, history see "predictable" on the last 200. Fixed: live now uses `closes[len-200..]` trailing slice, identical to history.

### 108.7 Kalman price_vs_predicted Mismatch (CRIT-4)

Three issues fixed: (a) `indicator_cache` applies `.abs()` to `price_surprise`, deleting the sign used by macro pillar for velocity alignment; (b) history `price_vs_predicted` era raw delta (`close - pred`), live era ratio (`(close - state.x[0]) / close`); (c) `surprise_series` per-bar in `kalman_filter` used `.abs()`. Fixed: deleted `.abs()` from the cache and surprise_series, normalized history to ratio `(close - pred) / close`.

### 108.8 Duplicate Changepoint Markers (MED-5)

Chart markers were duplicated at each eval cycle because `_merge()` concatenates without dedup. Fix: dedup by composite key `time|position|shape|text` via Set in `_titanMarkerEngine._merge()`.

### 108.9 stalking_timeouts Incremented Wrong (MED-8)

`stalking_timeouts` is incremented both on hysteresis drop (Stalking->Flat from low score) and on actual timeout. Fixed: the increment now exists only on the path `stalking_timeout` (when `bars_in_state >= 2 * confirmation_needed`).

### 108.10 ST CORRECTNESS Dashboard (B1)

Cyber-HUD panel with 4 correctness metrics, displayed in the sidebar under ST SHIELD:

| Metric | Values | Meaning |
|---------|--------|-------------|
| HYS | OK (green) / STUCK:N (red pulse) | Detect FSM stuck in Stalking > 50 consecutive evaluations |
| DECAY | Current score (gold) | Last value `last_score` from engine |
| LYAP | -- (placeholder) | Current Lyapunov exponent |
| KAL | +0.3% (green) / -2.1% (red) | Kalman price surprise signed ratio |

### 108.11 FSM Transition Heatmap Badge (B2)

Badge compact `FSM:S12/E3/T8` in cockpit strip. Ring buffer of 100 evaluations on `SmartTrendEngine`. Counter per-state (Stalking/Entry/InTrade+Trail). Dominant color: cyan (Stalking), green (Entry), gold (InTrade).

### 108.12 Breaker Sync Indicator (B3)

Badge `BRK:SYNC` (green) / `BRK:DESYNC` (red pulse) in ST SHIELD panel. Compare `risk.circuit_breaker.level() != Normal` with `st_account.breaker_triggered`. Visually confirm that the FIX-3 is working correctly.

### 108.13 Kalman Alignment Sparkline (B4)

Mini CSS bar chart (8 bars) in cockpit strip. Last 8 values ​​`price_surprise` from cache pointer. Cyan bars for positive surprise, magenta for negative. Height proportional to maximum. Flat = Kalman aligned with the price, spike = divergence.

### 108.14 Max-Hold Timer HUD (B5)

New column `HOLD` in positions table. Progress bar `15/60m` with gradient stops: cyan (normal), gold + glow-breathe (>70% of max_hold), red + pulse-red-ring (>90%). Visually shows how close the forced exit position is from max-hold decay.

---

## 109. Wave 2C — SmartTrend UI+Chart (Session 71)

### 109.1 Visible Chart Markers (CRIT-8)

`window.candleSeries` is now exposed as a global property after creating the series in `titan_bootstrap.js`. The marker engine `_titanMarkerEngine._merge()` can set markers on the chart. All 3 sources (signals, fills, SmartTrend) are visible.

### 109.2 ST CFG Flicker Eliminated (CRIT-9)

All SmartTrend timers in `titan_bootstrap.js` use `TitanTimers.register` with `{ tab: 13 }`:
- `st:diagnostics` (3s) -- SmartTrend diagnostics panel sidebar
- `st:panel-poll` (5s) -- SmartTrend account panel sidebar
- `st:s12b-account` (5s) -- S12b account section tab-13
- `st:s12b-assets` (5.5s) -- S12b assets table tab-13
- `st:s12b-sentinel` (6s) -- S12b sentinel status tab-13

Timers with tab guard automatically stop when another tab is visible, eliminating unnecessary polls and DOM flicker.

### 109.3 Duplicate HTML IDs Resolved (HIGH-6)

Tab-13 (S12b sections) used the same IDs as the SmartTrend sidebar: `st-acct-capital`, `st-acct-dd`, `st-acct-trades`, `st-acct-amount`, `st-assets-tbody`. Now tab-13 uses prefix `s12b-` (ex: `s12b-acct-capital`, `s12b-assets-tbody`). Both sections are updated independently.

### 109.4 titan_init.js Loaded (HIGH-4)

`<script src="titan_init.js" defer></script>` added to `<head>`. INIT_ORDER is executed at DOMContentLoaded, orchestrating the initialization of the modules in the correct order.

### 109.5 Health Badge Real Data (MED-3)

Health badge `HP:N%` now uses real data:
- **latencyQuality:** `100 - floor(avg_latency_ms / 10)`. Latency comes from `LatencyMetrics.snapshot().order_rtt_us.p50`.
- **fillRate:** `fills_received / orders_submitted * 100`. Atomic counters on `TradingState`.
- Backend: `get_risk_status` exposes `avg_latency_ms`, `orders_submitted`, `fills_received`.

### 109.6 Marker Engine Cleanup

- Removed fallback `candleSeries.setMarkers()` from `titan_indicators_ui.js` -- use exclusively `_titanMarkerEngine.update('signals', ...)`
- Removed `engine._stMarkers` hack from `titan_data_load.js` -- uses `engine.getSource('smarttrend')` (public API)
- Added `stats()` and `getSource()` to marker engine for introspection

### 109.7 Timer Consolidation Badge (B1)

Badge `TMR:N` in the cockpit strip shows the total number of active timers. Tooltip with breakdown: name, interval, tab guard, status. `TitanTimers.getAll()` displays the full list.

### 109.8 Marker Engine Health Badge (B2)

Badge `MRK:S12/F3/T8` in cockpit strip shows count per source of markers. Color coding: green (all sources > 0), gold (partial), red (no marker). Visually confirm that FIX-1 and FIX-6 are working.

### 109.9 Duplicate Poll Detector (B3)

Guard dev-only enabled via `localStorage.setItem('titanDebug', '1')`. Wrapper on `invoke` with ring buffer 50 entries. If the same Tauri command appears 2+ times in 1 second, log `[TITAN] Duplicate poll detected:` in the console.

### 109.10 Chart Marker Stats Overlay (B4)

Micro-overlay on chart (top left corner): `SIG:12 FILL:3 ST:8 TOT:23`. Semi-transparent (opacity 0.5), 9px cyber-HUD font. Automatically updated every `_merge()`. CSS: `.chart-marker-stats`.

### 109.11 Incremental SessionAnalytics (B5)

`recalculate_stats()` from `titan-risk` no longer does O(n) full-scan. Running totals `running_gross_wins` and `running_gross_losses` are maintained incrementally in `record_trade_with_duration` (O(1)). `restore_trade_pnls` does a single O(n) pass at startup.

### 109.12 IPC Optimizations

- **get_risk_status:** Health badge consumes `window._lastRiskPayload` cached by cockpit update (5s). -1 IPC round-trip / 10s.
- **get_settings:** Regime strip consumes `window._lastSettingsPayload` cached by cockpit update. -1 IPC round-trip / 5s.
- **Cache indicator:** Removed redundant clone in `update_candle()` -- move semantics on `merged`, single `.clone()` on return.

---

## 110. Wave 3A — Persistence+Defaults (Session 72)

### 110.1 Settings Error Propagation (CRIT-7)

All 8 commands that save settings to disk (`update_settings`, `batch_update_settings`, `set_active_symbol`, `set_position_mode`, `set_dry_run`, `set_symbol_override`, `remove_symbol_override`, `apply_full_settings`) propagated the error with `let _ =` -- any write failure was silently ignored.

**Fix:** Each command returns `Err(e)` to the frontend with `tracing::error!`. Issue event `settings:save-failed` visible in Save Event Timeline (B3).

### 110.2 Journal Atomic Save (CRIT-17)

`TradeJournal::save()` used `fs::write()` directly. A crash in the middle of writing could corrupt the JSON file.

**Fix:** Write `tmp` + `fs::rename()`. The pattern is atomic on POSIX -- the final file is either completely old or completely new, never partial. Emits event `journal:saved` for timeline UI.

### 110.3 Profile Load Validation (CRIT-18)

110.4 Dry-Run Default True (CRIT-19)

110.4 Dry-Run Default True (CRIT-19)

### 110.4 Dry-Run Default True (CRIT-19)

Fresh install had `dry_run_enabled: false` in `AppState::default()`. A new user starting the bot for the first time was automatically running with real money.

**Fixed:**
- `dry_run_enabled: true` default -- the bot always starts in simulation
- `tracing::warn!("LIVE TRADING ENABLED -- dry_run is OFF")` logged in every `apply_runtime_config` when dry_run is disabled
- Test: `test_default_dry_run_is_true`

### 110.5 WS Auth via Query Parameter (CRIT-2)

The WebSocket `/ws` from the web dashboard asked for the token only in the `Authorization: Bearer` header. The native browser WebSocket API (used by `new WebSocket(url)`) does not support custom headers.

**Fix:** Handler `/ws` accepts token and via `?token=...` query parameter. The frontend transmits `ws=new WebSocket(proto+'//'+location.host+'/ws?token='+encodeURIComponent(token))`. The `/ws` route is separated in `ws_router` with its own auth middleware.

### 110.6 Trade PnLs Ring Buffer + Welford Sharpe (HIGH-17)

`trade_pnls: Vec<Decimal>` was growing indefinitely -- after 10,000+ trades, the vector was consuming significant memory and `sharpe_ratio()` was doing O(n) full scan on each call.

**Fixed:**
- `VecDeque<Decimal>` with a head of **2000** entries (ring buffer). Old entries are automatically evicted.
- **Welford online algorithm** for incremental mean/variance:
  - `sharpe_count`, `sharpe_mean`, `sharpe_m2` -- updated O(1) per trade
  - `sharpe_ratio()` calculates from online statistics, no longer scans the vector
- `restore_trade_pnls` recomputes Welford in a single O(n) pass at startup
- 3 tests: `trade_pnls_capped_at_2000`, `sharpe_incremental_matches_full_scan`, `restore_trade_pnls_caps_to_limit`

### 110.7 HashMap TTL Eviction (HIGH-18)

4 HashMaps in `TradingState` grew indefinitely in long sessions:
- `fill_cumulative` — the number of fills per symbol
- `trail_highs` / `trail_lows` — trailing stop per symbol
- `trail_highs` / `trail_lows` — trailing stop per symbol

**Fix:** `evict_stale_entries()` with different TTL policies:
| Map | TTL | Condition |
|-----|-----|----------|
| `fill_cumulative` | 24h | Timestamp > 24h without activity |
| `failed_order_cooldown` | expired | Timestamp value < now |
| `trail_highs` / `trail_lows` | orphan | No active position on the symbol |

- Called every **100** trade updates via `eviction_tick` counter
- Atomic counter `evicted_entries` exposed in `get_risk_status` for monitoring
- `fill_cumulative_ts: Mutex<HashMap<String, u64>>` — timestamp per entry
- 3 tests: `fill_cumulative_eviction_after_24h`, `cooldown_eviction`, `map_entries_total_counts_all`

### 110.8 WS Reconnect Staleness Flag (HIGH-19)

After a disconnect+reconnect on the market data stream, the first 30 seconds of data may have gaps or stagnant values. The bot had no detection mechanism.

**Fixed:**
- `stream_stale: AtomicBool` set to `true` at disconnect
- `stream_reconnected_at: AtomicU64` — reconnection timestamp
- `mark_stream_reconnected()` — set to auth+subscribe succeeded
- `mark_stream_data_received()` — clear stale flag after 30s warmup
- `is_stream_stale()` — returns `true` if stalled or in warmup
- Exhibited in `get_risk_status` → badge `WS:STALE` in cockpit strip

### 110.9 Persistence Health Dashboard (B1)

Cyber-HUD panel with 2x2 grid in the monitoring area:

| Cell | Metra | colors |
|--------|---------|--------|
| **DISK** | Settings save status | green (ok) / red (fail) |
| **JRNL** | Journal save status | green (saved) / gold (dirty) |
| **PROF** | Profile loaded name | green (loaded) |
| **DRY** | green (ON) / red (OFF with neon pulses) | green (ON) / red (OFF with neon pulses) |

CSS: `.persist-health-grid`, `.persist-cell`, `.persist-val`. Neon colors with pulsation on critical states.

### 110.10 Memory Pressure Gauge (B2)

Badge `MEM:N` in cockpit strip. `N` = total number of entries from the 4 critical HashMaps in `TradingState`.

| Entries | Color | Meaning |
|---------|---------|--------------|
| < 500 | GREENE | Normal |
| 500-2000 | Goldie | Raised |
| > 2000 | ed | High pressure |

Backend: `map_entries_total` added in `get_risk_status` JSON.

### 110.11 Save Event Timeline (B3)

Chronological timeline in Persistence Health panel with chips per save event:

- **Events:** `settings:saved`, `settings:save-failed`, `journal:saved`, `snapshot:saved`
- **Ring buffer:** 12 entries (most recent)
- **Format:** `HH:MM:SS LABEL` (ex: `14:32:07 SETTINGS OK`)
- **Colours:** `.save-ok` (green), `.save-fail` (pulsating red)
- Implemented via Tauri event listeners + `_pushSaveEvent` ring buffer

### 110.12 Dry-Run Guard Overlay (B4)

When `dry_run=false` (live trading):
- **Border:** `document.body` receives the `.live-trading-border` class — 3px solid red border with neon box-shadow
- **Banner:** `div#live-trading-banner` with text `LIVE TRADING — DRY RUN OFF` — positioned fixed top, maximum z-index, animation `neon-pulse`
- Automatically hidden when `dry_run=true`

### 110.13 Eviction Stats Counter (B5)

Badge `EVT:N` in cockpit strip. `N` = total entries evicted by `evict_stale_entries()` since startup.

| Evictions | Color |
|-----------|---------|
| GREENE | GREENE |
| 100-1000 | Goldie |
| > 1000 | ed |

Backend: `evicted_entries_total` exposed in `get_risk_status` from `AtomicU64`.

### 110.14 Persistence Optimizations

**OPT-1: Batch Settings Save Debounce**
`batch_update_settings` execute a single `save_settings_to_disk` at the end, instead of write per field. Reduce N disk writes to 1 per batch.

**OPT-2: Journal Save Debounce**
`TradeJournal` with `dirty: bool` and `last_save_epoch: u64`. Save effectively only if:
1. `dirty == true` (there is unsaved data)
2. >= 30s have passed since the last save

`flush()` forced to `graceful_shutdown` so as not to lose data. Reduces N disk writes to ~2/minute during active periods.

**OPT-3: Snapshot Delta Compression**
`save_state_snapshot` compare the SHA-256 hash of the serialized snapshot with the previous hash (read from the `.sha256` sidecar file). If they are identical, skip write completely. Reduces disk writes to 0 when the state does not change (idle periods).

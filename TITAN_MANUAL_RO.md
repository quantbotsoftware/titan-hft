# TITAN HFT (SHARK V7 RELOADED) — Manual de Utilizare Complet

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


**Versiune:** V7 Reloaded  
**Motorul:** Rust (Tauri) + WGPU + HTML/JS Frontend  
**Platforma:** Alpaca Markets (Paper + Live)  
**Asset-uri suportate:** Crypto (BTC/USD, ETH/USD, etc.) + US Stocks  
**Data:** Aprilie 2026 (actualizat 3 Aprilie — Sesiunea 69: Wave 2B SmartTrend Correctness — 108 Capitole)

---

## CUPRINS

1. [Pornire si Configurare Initiala](#1-pornire-si-configurare-initiala)
2. [Arhitectura Sistemului](#2-arhitectura-sistemului)
3. [Interfata Utilizator (UI)](#3-interfata-utilizator-ui)
4. [Sidebar-ul Restructurat (Full Audit)](#4-sidebar-ul-restructurat-full-audit)
5. [Sistemul de Strategie (3 Moduri)](#5-sistemul-de-strategie-3-moduri)
6. [Cum Alege Botul Asset-urile](#6-cum-alege-botul-asset-urile)
7. [TITAN Oracle — 7 Indicatori AI](#7-titan-oracle--7-indicatori-ai)
8. [GPU Nexus — 150,000 Agenti](#8-gpu-nexus--150000-agenti)
9. [L2 Orderbook si OFI (Order Flow Imbalance)](#9-l2-orderbook-si-ofi-order-flow-imbalance)
10. [Autopilot — Motorul de Trading Automat](#10-autopilot--motorul-de-trading-automat)
11. [Filtrele de Siguranta (Safety Gates)](#11-filtrele-de-siguranta-safety-gates)
12. [Genetic Optimizer — Evolutie Automata a Parametrilor](#12-genetic-optimizer--evolutie-automata-a-parametrilor)
13. [Circuit Breaker — Sistemul de Siguranta](#13-circuit-breaker--sistemul-de-siguranta)
14. [Dry Run Mode — Simulare fara Risc](#14-dry-run-mode--simulare-fara-risc)
15. [Watchdog si Telemetrie](#15-watchdog-si-telemetrie)
16. [Logging si Rapoarte](#16-logging-si-rapoarte)
17. [Backtesting](#17-backtesting)
18. [Graficul Profesional](#18-graficul-profesional)
19. [Tab-uri Noi (News AI, Jurnal, HFT Engine, Performance)](#19-tab-uri-noi)
20. [AI Scanner — FUSION Asset Discovery](#20-ai-scanner--fusion-asset-discovery)
21. [Managementul Pozitiilor si Fondurilor](#21-managementul-pozitiilor-si-fondurilor)
22. [Toate Loop-urile din Background](#22-toate-loop-urile-din-background)
23. [Toate Comenzile Disponibile](#23-toate-comenzile-disponibile)
24. [Structura Fisierelor](#24-structura-fisierelor)
25. [Probleme Frecvente si Solutii](#25-probleme-frecvente-si-solutii)
26. [Catalyst Watchdog — Sentient News Feed](#26-catalyst-watchdog--sentient-news-feed)
27. [Semnale și Săgeți pe Grafic](#27-semnale-si-săgeți-pe-grafic)
28. [Telegram Alerts — Notificari Mobile](#28-telegram-alerts--notificari-mobile)
29. [Risk Correlation Matrix](#29-risk-correlation-matrix)
30. [Portfolio Performance & Rebalancing](#30-portfolio-performance--rebalancing)
31. [ATR Dynamic Stop-Loss & Partial Profit Taking](#31-atr-dynamic-stop-loss--partial-profit-taking)
32. [Web Dashboard — Acces Remote Mobile](#32-web-dashboard--acces-remote-mobile)
33. [State Persistence — Crash Recovery](#33-state-persistence--crash-recovery)
34. [Smart Score V4 — Ranking Institutional](#34-smart-score-v4--ranking-institutional)
35. [Auto Entry — Control Intrari Automate](#35-auto-entry--control-intrari-automate)
36. [Signal Alignment — FUSION-Aware Autopilot & Causal Chart](#36-signal-alignment--fusion-aware-autopilot--causal-chart)
37. [Auto-Resume Scan — Continuare de la Punctul de Intrerupere](#37-auto-resume-scan--continuare-de-la-punctul-de-intrerupere)
38. [Multi-Symbol Entry Fix & DEL ALL](#38-multi-symbol-entry-fix--del-all)
39. [Entry Gate Scoring Pipeline — Debottlenecking](#39-entry-gate-scoring-pipeline--debottlenecking)
40. [Scalping Mode — Toggle-uri per Strategie](#40-scalping-mode--toggle-uri-per-strategie)
41. [Regime-Adaptive Trading Signals](#41-regime-adaptive-trading-signals)
42. [Autopilot Alignment — check_entry + Oracle Exits](#42-autopilot-alignment--check_entry--oracle-exits)
43. [WARRIOR PRO — Ross Cameron Scanner](#43-warrior-pro--ross-cameron-scanner)
44. [Position Sizing Safety — Hardcaps & Duplicate Prevention](#44-position-sizing-safety--hardcaps--duplicate-prevention)
45. [Scalping Position Marker](#45-scalping-position-marker)
46. [AI Scanner Enhancements — Cancellation, Shared Scoring, Rate Limiting](#46-ai-scanner-enhancements)
47. [SMART PICKS 3-Pillar Scoring](#47-smart-picks-3-pillar-scoring)
48. [Market Discovery — Parametri Avansati si Auto-Scan](#48-market-discovery--parametri-avansati-si-auto-scan)
49. [Manual Trade — Tranzactionare Manuala USD](#49-manual-trade--tranzactionare-manuala-usd)
50. [Force Manual Override — Control Total](#50-force-manual-override--control-total)
51. [Live Positions — Mod Auto/Manual per Asset](#51-live-positions--mod-automanual-per-asset)
52. [Drawing Tools V2 — Toate Instrumentele](#52-drawing-tools-v2--toate-instrumentele)
53. [Aggressive Mode — Audit Complet](#53-aggressive-mode--audit-complet)
54. [Extended Hours — Audit Complet](#54-extended-hours--audit-complet)
55. [Eliminari din Sesiunea 7](#55-eliminari-din-sesiunea-7)
56. [AI Scanner Parallel Scoring (Rayon)](#56-ai-scanner-parallel-scoring-rayon)
57. [Deep Audit Reasoning — Fereastra Modal](#57-deep-audit-reasoning--fereastra-modal)
58. [Manual Trade USD vs Shares Toggle](#58-manual-trade-usd-vs-shares-toggle)
59. [SIGNAL Scalping Kit — Alerte si Journal Exact](#83-signal-scalping-kit-sesiunea-17--alerte-realtime--journal-exact)
60. [Toti Parametrii de Trading Configurabili (Sesiunea 18)](#84-toti-parametrii-de-trading-configurabili-sesiunea-18)
61. [Deep Audit + 12 Smart Features + Styling (Sesiunea 19)](#85-deep-audit--12-smart-features--styling-sesiunea-19)
62. [Sesiunea 26: TA Entry Gates cu Soft/Hard Mode (RF-B)](#cap-91--sesiunea-26-ta-entry-gates-cu-softhard-mode-rf-b)
63. [Portfolio VaR Engine + HHI Concentration Gate (S9)](#98-portfolio-var-engine--hhi-concentration-gate-s9--sesiunea-33)
64. [Feature Store — Entry Context Snapshot (S13)](#99-feature-store--entry-context-snapshot-s13--sesiunea-34)
65. [Microstructure Alpha Signals — Stream VPIN, Spread Regime, Dual VPIN Divergence (S10-lite)](#100-microstructure-alpha-signals)
66. [N4 Strategy Thresholds Configurabile (Sesiunea 35-36)](#101-n4-strategy-thresholds-configurabile)
67. [CI Hardening + Criterion Benchmarks (S19, Sesiunea 37)](#102-ci-hardening--criterion-benchmarks-s19--sesiunea-37)
68. [Online Oracle Weight Adaptation (S15, Sesiunea 38)](#103-online-oracle-weight-adaptation-s15--sesiunea-38)
69. [Exit Inflight Guard — Protectie Anti-Loop (Sesiunea 39)](#104-exit-inflight-guard--protectie-anti-loop-la-iesiri-sesiunea-39)
70. [Configuration Coherence — apply_runtime_config (Sesiunea 44)](#105-configuration-coherence--apply_runtime_config-sesiunea-44)
71. [Wave 1 MUST-FIX — 9 Fix-uri Critice + 4 Bonusuri (Sesiunea 67)](#106-wave-1-must-fix--9-fix-uri-critice--4-bonusuri-sesiunea-67)
72. [Wave 2A SmartTrend Safety — 8 Fix-uri + 4 Bonusuri (Sesiunea 68)](#107-wave-2a-smarttrend-safety--8-fix-uri--4-bonusuri-sesiunea-68)
73. [Wave 2B SmartTrend Correctness — 9 Fix-uri + 5 Bonusuri (Sesiunea 69)](#108-wave-2b-smarttrend-correctness--9-fix-uri--5-bonusuri-sesiunea-69)

---

## 1. PORNIRE SI CONFIGURARE INITIALA

### 1.1 Cerinte de sistem

- **OS:** Linux (Pop!_OS, Ubuntu, Fedora)
- **GPU:** NVIDIA cu suport Vulkan (RTX 3060+)
- **RAM:** 8 GB minim, 16 GB recomandat
- **Rust:** Instalat cu rustup
- **Redis:** Optional, dar recomandat (`redis-server` pe port 6379)
- **Alpaca Account:** Paper sau Live cu Algo Trader Plus ($99/mo pentru SIP data)

### 1.2 Pornirea aplicatiei

```bash
${TITAN_HFT_ROOT:-.}/src-tauri/target/release/titan_hft_rust
```

### 1.3 La pornire, Titan face automat

1. **Initializeaza logging** — scrie in `$HOME/HFT_Bot/logs/titan.log.YYYY-MM-DD`
2. **Incarca cheile API** din `$HOME/HFT_Bot/titan_keys.json`
3. **Health Check complet** — verifica:
   - API Keys (incarcate si non-empty)
   - Alpaca API (conexiune, equity, status cont)
   - Redis (conexiune)
   - GPU WGPU (adapter Vulkan gasit)
   - Settings (valori in limite sigure)
   - Log directory (writable)
4. **Porneste toate loop-urile** (9 loop-uri + watchdog supervisor)
5. **Sincronizeaza pozitiile** de pe Alpaca (Reconciliation)
6. **Afiseaza in Activity Log** rezultatul Health Check-ului

### 1.4 Configurare API Keys

In interfata, in sidebar-ul stang, expandeaza sectiunea **API & WALLET**:

1. Introdu **API KEY ID** (26 caractere)
2. Introdu **API SECRET** (44 caractere)
3. Click **SAVE KEYS**

Cheile se salveaza in `$HOME/HFT_Bot/titan_keys.json` si persista intre sesiuni.

---

## 2. ARHITECTURA SISTEMULUI

### 2.1 Module Rust (Backend)

| Fisier | Responsabilitate |
|--------|-----------------|
| `main.rs` | Orchestrator central, state management, ~70 comenzi Tauri, 14 loop-uri background |
| `titan_oracle.rs` | 7 indicatori AI avansati (Kalman, Hurst, Entropy, Wavelet, VPIN, Lyapunov, BOCPD) |
| `gpu.rs` | WGPU compute shaders, 150K agenti GPU Nexus, EMA/RSI pe GPU |
| `risk.rs` | Circuit Breaker (4 nivele), SessionAnalytics, OFI, pozitionare bazata pe risc |
| `genetic.rs` | Optimizator genetic cu walk-forward backtest (70/30 IS/OOS) |
| `l2_heatmap.rs` | L2 orderbook live, OFI din full depth, crypto v1beta3 + SIP stocks |
| `watchdog.rs` | Heartbeat monitoring, telemetrie per loop, session reports |
| `titan_log.rs` | Logging structurat cu tracing, rotatie zilnica |
| `indicators.rs` | Indicatori tehnici clasici (EMA, RSI, MACD, ADX, BB, VWAP) |
| `strategy.rs` | Logica de strategie si semnale |
| `streamer.rs` | Price streaming si heatmap |
| `scanner.rs` | Market scanner (quant_prefilter + warrior_prefilter) |
| `monitor.rs` | Hardware monitoring (CPU, GPU, RAM, Network) |
| `sentiment.rs` | Analiza sentiment + Catalyst check |
| `app_state.rs` | Definire stari aplicatie |
| `correlation.rs` | Pearson correlation, portfolio performance, rebalancing |
| `telegram.rs` | Telegram Bot alerts (fill, breaker, catalyst, daily) |
| `journal.rs` | Trade journal persistent (JSON, stats, CSV export) |

### 2.2 GPU Shaders (WGSL)

| Shader | Ce face |
|--------|---------|
| `nexus_kernel.wgsl` | 150K agenti: fiecare calculeaza intent (buy/sell), voteaza, capital se ajusteaza |
| `gpu_indicators.wgsl` | EMA fast, EMA slow, RSI pentru o singura serie |
| `gpu_indicators_batch.wgsl` | Acelasi lucru dar pentru mai multe serii simultan |
| `gpu_dot.wgsl` | Produs scalar paralel (reducere GPU) |

### 2.3 Frontend

- **index.html** — Aplicatie single-page cu TradingView LightweightCharts
- **Comunicare:** Tauri `invoke()` pentru comenzi, `listen()` pentru evenimente push

---

## 3. INTERFATA UTILIZATOR (UI)

### 3.1 Header

- **Titlul:** "TITAN V4.37 (SHARK V7 RELOADED)"
- **Status indicator:** CLOSED / LIVE
- **NY Time:** Ora New York actualizata la fiecare secunda

### 3.2 Zona Centrala — Tab-uri

| Tab | Continut |
|-----|----------|
| **Grafic Complex** (tab-0) | Chart profesional OHLC cu EMA, BB, VWAP, RSI, MACD, ADX, Oracle |
| **News AI** (tab-1) | Flux de stiri cu sentiment analysis (gauge, timeline, cards) |
| **Backtest** (tab-2) | Motor de backtesting cu sweep multi-timeframe si multi-symbol, equity curve |
| **Jurnal** (tab-3) | Istoric tranzactii persistent cu filtre, export CSV, grafice equity/PnL |
| **HFT Engine** (tab-4) | Setari avansate motor HFT: entry/exit params, Discord alerts, safety limits |

### 3.3 Panoul Central Superior

**GPU Nexus — 150K Agents:**

- Bara de consens: verde (BUY) vs rosu (SELL)
- Butoane de mod: ORACLE / GPU / FUSION
- Slider Fusion Alpha (0.0 = full Oracle, 1.0 = full GPU)
- Afiseaza: numar agenti, latenta GPU (microsecunde), mod curent

### 3.4 Panoul TITAN SAFETY

Afiseaza in timp real:

- **Circuit Breaker Level:** NORMAL (verde) / CAUTION (galben) / HALT (portocaliu) / PANIC (rosu)
- **Daily PnL:** Profitul/pierderea din ziua curenta
- **Max DD:** Maximum drawdown (procent)
- **Win Rate:** Rata de castig
- **Trades:** Numar total de tranzactii
- **Sharpe:** Sharpe ratio al sesiunii
- **Profit Factor:** Raportul profit/pierdere
- **Equity:** Valoarea totala a contului

### 3.5 Panoul EVOLUTION

- **Gen:** Generatia curenta a optimizatorului genetic
- **Fitness:** Scorul de fitness al campionului
- **Sharpe:** Sharpe ratio al campionului
- **TP/SL:** Take Profit / Stop Loss ale parametrilor activi
- **FREEZE / UNFREEZE:** Opreste/porneste evolutia genetica
- **BACKTEST NOW:** Ruleaza un backtest instant cu parametrii actuali

### 3.6 Panoul OFI LIVE

- **SPR:** Spread-ul live in basis points (bps)
- **OFI bar:** Vizualizare grafica a Order Flow Imbalance
- **BID depth / ASK depth:** Adancimea orderbook-ului
- **IMB:** Raportul de dezechilibru (imbalance ratio)

### 3.7 Panoul ACTIVITY LOG

Log in timp real cu color-coding:

- **Verde:** Mesaje de succes (BOOT, OK, PASS)
- **Rosu:** Erori si avertismente
- **Galben:** Warnings
- **Cyan:** Informatii trade

### 3.8 Panoul LOOP HEARTBEATS

- **Status global:** "ALL ALIVE" (verde) sau "X DEAD LOOPS" (rosu)
- **Per loop:** Nume, timp de la ultimul heartbeat, numar de tick-uri
- Actualizat la fiecare 10 secunde

### 3.9 Butoane de Control

| Buton | Functie |
|-------|---------|
| **DRY RUN: OFF/ON** | Activeaza/dezactiveaza simularea (nu trimite ordine la Alpaca) |
| **SHUTDOWN** | Oprire gratiata — salveaza raportul, opreste toate loop-urile |
| **LIQUIDATE ALL** | Inchide TOATE pozitiile deschise si anuleaza ordinele |

### 3.10 Sidebar Stang (Sumar)

Sidebar-ul a fost complet restructurat in 8 sectiuni logice (vezi [Sectiunea 4](#4-sidebar-ul-restructurat-full-audit) pentru detalii complete).

---

## 4. SIDEBAR-UL RESTRUCTURAT (FULL AUDIT)

Sidebar-ul stang a fost complet reorganizat in 8 sectiuni logice, fiecare cu header vizibil si continut consistent.

### 4.1 Sectiunea 1 — SYMBOL & FAVORITES

- **Input simbol:** Casuta unde se introduce simbolul activ (ex: BTC/USD, AAPL)
- **Butoane simbol:** BTC/USD, ETH/USD, SPY, AAPL, TSLA (schimbare rapida)
- **Butoane de actiune:** BUY / SELL manual
- **Lista FAVORITES:** Simboluri salvate persistent (se pastreaza intre sesiuni)
  - Buton **+** pentru adaugare, buton **x** pentru stergere
  - Click pe favorit → devine simbolul activ
  - Favorites sunt stocate in `titan_settings.json` (backend), nu in localStorage

### 4.2 Sectiunea 2 — SYMBOL TABS (WATCHLIST / HFT ENGINE)

Doua sub-tab-uri cu butoane interactive:

| Tab | Continut |
|-----|----------|
| **WATCHLIST** | Lista de simboluri favorite cu selectie rapida |
| **HFT ENGINE** | Lista de simboluri tinta pentru multi-symbol autopilot |

HFT ENGINE tab afiseaza:

- Lista de simboluri tinta configurate
- Butoane pentru adaugare/stergere din lista HFT
- Status per simbol (activ/inactiv)

### 4.3 Sectiunea 3 — HARDWARE MONITOR (Colapsabila)

Sectiune `<details>` deschisa implicit. Afiseaza:

**GPU:**

- Nume model (ex: NVIDIA GeForce RTX 3060)
- Temperatura, utilizare %, memorie folosita/totala
- Putere consumata, viteza ventilator

**CPU:**

- Utilizare %, temperatura, frecventa
- Viteza ventilator, RAM utilizat/total

**Network:**

- Interfata activa, download/upload speed
- Total transfer date

### 4.4 Sectiunea 4 — BANK & ACCOUNTS

Afiseaza fondurile in timp real:

| Camp | Descriere |
|------|-----------|
| **Cash (Power)** | Puterea de cumparare disponibila |
| **Equity (Total)** | Valoarea totala a contului |
| **Bot Alloc** | Fonduri alocate botului |
| **User Deposit** | Depozitul virtual al utilizatorului |
| **PnL** | Profit/pierdere zilnica |
| **Bot Trade Size** | Marimea ordinelor (persistent in backend) |
| **Circuit Breaker Limit** | Limita circuit breaker (persistent in backend) |

Butoane de transfer:

- **TOP UP BOT** — Aloca fonduri de la user la bot
- **WITHDRAW** — Retrage fonduri din bot la user
- **TOPUP USER** — Creste depozitul virtual

### 4.5 Sectiunea 5 — API & WALLET (Colapsabila)

Sectiune `<details>` deschisa implicit.

- **API KEY ID** si **API SECRET** — campuri editabile
- **SAVE KEYS** — Salveaza in `titan_keys.json`
- **Wallet info:** Afisare stare portofel

### 4.6 Sectiunea 6 — RISK & PARAMS (Colapsabila)

Sectiune `<details>` deschisa implicit. Contine:

**Toggle-uri (checkboxe):**

| Toggle | Efect |
|--------|-------|
| **Scan Active** | Activeaza scannerul de piata |
| **Kelly** | Pozitionare Kelly Criterion |
| **Extended Hours** | Trading in afara orelor normale |
| **Manual Result** | Rezultat manual per trade |
| **Force Manual** | Forteaza modul manual (dezactiveaza autopilot) |
| **Warrior** | Mod agresiv |
| **Auto-Pilot** | Activare/dezactivare autopilot |
| **Aggressive** | Parametri agresivi |
| **Short Selling** | Permite short selling |
| **Multi-Symbol** | Activare trading simultan pe mai multe simboluri |

**Slidere:**

- **Risk per trade** (0-100%) — slider galben stralucitor
- Toate slider-urile au track galben (#FFD700 → #FFEA00) cu glow si thumb auriu

### 4.7 Sectiunea 7 — MARKET DISCOVERY (Colapsabila)

Sectiune `<details>` deschisa implicit.

- **Market Scanner:** Cauta simboluri noi bazat pe criterii (volum, volatilitate)
- **Discovery results:** Lista de simboluri gasite

### 4.8 Sectiunea 8 — CONTROL BUTTONS

Butoane de control la baza sidebar-ului:

| Buton | Culoare | Efect |
|-------|---------|-------|
| **DRY RUN: OFF/ON** | Galben | Activeaza/dezactiveaza simularea |
| **SHUTDOWN** | Gri | Oprire gratiata cu salvare raport |
| **LIQUIDATE ALL** | Rosu | Kill switch — inchide totul |

### 4.9 Persistenta Setarilor

Dupa Full Audit, urmatoarele setari sunt stocate persistent in `titan_settings.json` (backend Rust), NU in localStorage:

| Setare | Descriere |
|--------|-----------|
| `dry_run_enabled` | Starea Dry Run (true/false) |
| `strategy_mode_name` | Modul de strategie (ORACLE/GPU/FUSION) |
| `bot_trade_size` | Marimea ordinelor |
| `circuit_breaker_limit` | Limita circuit breaker |
| `favorites` | Lista de simboluri favorite |

Setarile se incarca automat la pornire si se salveaza instant la modificare.

### 4.10 Stilizare

- **Inputuri/Selecturi:** Fundal negru (`#000`), text auriu (`#FFD700`), border gri (`#333`), font JetBrains Mono
- **Slidere:** Track galben stralucitor cu glow, thumb auriu cu border alb
- **Butoane:** Clase uniforme — `.btn-primary` (accent), `.btn-danger` (rosu), `.btn-secondary` (gri)
- **Toggle-uri:** Hover effect galben pe `.chk-row:hover`
- **Headere sectiuni:** `.sb-header` cu font uppercase, border-bottom galben

---

## 5. SISTEMUL DE STRATEGIE (3 MODURI)

Titan opereaza in 3 moduri de strategie, selectabile din UI:

### 5.1 ORACLE Mode

Foloseste exclusiv **TITAN Oracle** (7 indicatori AI):

- Semnalul provine din scorul Oracle (-100 la +100)
- Ideal pentru trend following si regime detection
- Procesare 100% pe CPU (Rust nativ)

### 5.2 GPU NEXUS Mode

Foloseste exclusiv **GPU Nexus** (150K agenti pe RTX):

- 150,000 de agenti autonomi voteaza simultan pe GPU
- Consensus derivat din vot majoritar ponderat
- Latenta sub 1ms (tipic ~500-700 microsecunde)
- Ideal pentru reactie rapida la piata

### 5.3 FUSION Mode (Recomandat)

Combina Oracle + GPU Nexus cu un blend ajustabil:

- **Fusion Alpha slider:** 0.0 = 100% Oracle, 1.0 = 100% GPU
- **Default absolut:** Sistemul porneste intotdeauna in modul FUSION prin default (setare persistenta UI + Backend). Blend initial: 0.50.
- Combina analiza profunda (Oracle) cu reactie rapida (GPU)
- Parametrii de intrare sunt controlati de optimizatorul genetic

### Schimbare mod

- Din UI: click pe butoanele **ORACLE**, **GPU**, **FUSION**
- Slider-ul alpha e vizibil doar in modul FUSION

---

## 6. CUM ALEGE BOTUL ASSET-URILE

### 6.1 Doua Moduri de Selectie

Titan nu scaneaza singur piata pentru a descoperi ce sa tranzactioneze. Asset-urile sunt definite de utilizator:

**Modul 1 — Simbol Unic (Default):**

- Botul tranzactioneaza DOAR simbolul activ din sidebar (ex: BTC/USD)
- Schimbi simbolul → botul se concentreaza pe noul simbol
- Toate semnalele (Oracle, GPU Nexus) sunt calculate pe acel simbol

**Modul 2 — Multi-Symbol (HFT Engine):**

- Activat prin toggle-ul **Multi-Symbol** din sidebar
- Botul tranzactioneaza simultan pe lista de simboluri din tab-ul HFT ENGINE
- Fiecare simbol primeste propria analiza si semnale independente
- Autopilot-ul ruleaza ciclul de entry/exit pentru TOATE simbolurile din lista

### 6.2 Fluxul Decizional (per simbol)

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

### 6.3 Relatia Strategie — Filtre

Strategia (Oracle/GPU/Fusion) genereaza **semnalul** si **scorul**. Dar semnalul NU garanteaza un trade. Filtrele de siguranta sunt **independente** si pot bloca intrarea chiar daca scorul este mare:

- Strategie spune "BUY cu scor 85" → Spread Guard blocheaza (spread prea mare)
- Strategie spune "SELL cu scor 70" → Circuit Breaker e pe HALT → blocat
- Strategie spune "BUY cu scor 90" → OFI e negativ → semnalul nu e confirmat

Strategia e **creatorul semnalului**. Filtrele sunt **portarii** care protejeaza capitalul.

---

## 7. TITAN ORACLE — 7 INDICATORI AI

Oracle-ul combina 7 indicatori avansati intr-un scor final (-100 la +100):

### 7.1 Kalman Adaptive Predictor (KAP) — Pondere 30%

- **Ce face:** Filtru Kalman cu model de viteza constanta
- **Cum functioneaza:** Estimeaza pretul si viteza (trendul). Matricea Q se scaleaza cu volatilitatea.
- **Semnal:** Daca pretul prezis > pretul curent → bullish. Invers → bearish.
- **Utilitate:** Cel mai stabil predictor pe piete trending

### 7.2 Haar Wavelet Multi-Resolution — Pondere 25%

- **Ce face:** Descompune pretul in trend + zgomot
- **Cum functioneaza:** Aplica transformata Haar wavelet, pastreaza doar frecventele joase
- **Semnal:** Trendul denoised indica directia reala
- **Utilitate:** Filtreaza noise-ul de piata

### 7.3 Fractal Hurst Momentum (FHM) — Pondere 20%

- **Ce face:** Analiza R/S (Range/Scale) pentru detectia regimului
- **Cum functioneaza:** Calculeaza exponentul Hurst pe fereastra de pret
- **Interpretare:**
  - H > 0.5 → Piata trending (momentum works)
  - H < 0.5 → Piata mean-reverting (contrarian works)
  - H = 0.5 → Random walk
- **Utilitate:** Decide daca strategia momentum sau contrarian e mai potrivita

### 7.4 VPIN (Volume-Weighted Probability of Informed Trading) — Pondere 15%

- **Ce face:** Masoara probabilitatea de "informed trading"
- **Cum functioneaza:** Impartire in volume buckets, calcul buy/sell imbalance
- **Interpretare:** VPIN ridicat → traderi informati activi, posibil miscare mare
- **Utilitate:** Early warning pentru volatility spikes

### 7.5 Shannon Entropy Flow Regime Detector (EFRD) — Pondere 10%

- **Ce face:** Masoara entropia returnurilor
- **Cum functioneaza:** Retururile sunt binned in 20 categorii, se calculeaza entropia Shannon
- **Interpretare:**
  - Entropie scazuta → Regim clar, directional
  - Entropie ridicata → Piata haotica, fara directie
- **Utilitate:** Meta-indicator de claritate a pietei

### 7.6 Lyapunov Chaos Detector

- **Ce face:** Detecteaza chaos in seria de pret
- **Cum functioneaza:** Analiza nearest-neighbor divergence in spatiul fazei
- **Interpretare:**
  - Lambda pozitiv → Chaos (dificil de prezis)
  - Lambda negativ → Predictabil
- **Utilitate:** Ajusteaza meta-confidence — daca piata e haotica, reduce semnalul

### 7.7 Bayesian Online Changepoint Detector (BOCPD)

- **Ce face:** Detecteaza schimbari de regim in timp real
- **Cum functioneaza:** Algoritmul Adams & MacKay (2007), probabilitate bayesiana de changepoint
- **Interpretare:** Probabilitate changepoint > 0.3 → Reduce semnalul cu 70%
- **Utilitate:** Previne intrarea in tranzactii chiar la schimbarea regimului

### 7.8 Formula de Fuziune Finala

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

**Range final:** -100 (extrem bearish) la +100 (extrem bullish)

---

## 8. GPU NEXUS — 150,000 AGENTI

### 8.1 Cum functioneaza

- **150,000 de agenti** ruleaza simultan pe GPU-ul NVIDIA (RTX 3060)
- Fiecare agent are:
  - **Tip:** Momentum (40%), Mean-Reversion (35%), Breakout (25%)
  - **Capital virtual:** Creste cu +0.2% la predictie corecta, scade cu -0.2% la gresita
  - **Intent:** BUY sau SELL, ponderat de capitalul agentului
- **Latenta:** ~500-700 microsecunde per dispatch (sub 1ms)

### 8.2 Consens

La fiecare tick (500ms):

1. Se trimite `MarketState` (pret, volume, indicatori) la GPU
2. Toti agentii calculeaza simultan intentul
3. Se sumarizeaza `buy_weight` si `sell_weight`
4. Se calculeaza procentele si consensul

| Consens | Conditie |
|---------|----------|
| **STRONG BUY** | buy_pct > 70% |
| **BUY** | buy_pct > 55% |
| **NEUTRAL** | Niciuna din cele de mai sus |
| **SELL** | sell_pct > 55% |
| **STRONG SELL** | sell_pct > 70% |

### 8.3 Evolutia Agentilor

Agentii nu au parametri statici — capitalul lor se ajusteaza automat:

- Agenti cu predictii bune → cresc in influenta
- Agenti cu predictii gresite → scad
- Acest mecanism creeaza un "survival of the fittest" natural

---

## 9. L2 ORDERBOOK SI OFI (ORDER FLOW IMBALANCE)

### 9.1 Sursa datelor

- **Crypto (BTC/USD etc.):** `wss://stream.data.alpaca.markets/v1beta3/crypto/us`
  - Subscriptie: **orderbooks** (L2 full depth) + quotes + trades
  - Orderbook real cu bids/asks pe multiple nivele
- **Stocks:** `wss://stream.data.alpaca.markets/v2/sip` (necesita Algo Trader Plus)
  - Subscriptie: quotes + trades
  - OFI calculat din quote size changes

### 9.2 Detectie automata crypto vs stocks

Simbolul e detectat automat:

- Contine `/` (ex: BTC/USD) → Crypto
- Incepe cu BTC, ETH, LTC, DOGE, SOL, AVAX, UNI → Crypto
- Altfel → Stocks (SIP)

### 9.3 Orderbook Management (Crypto)

- **BTreeMap** ordonat dupa pret pentru bids si asks
- Mesaj `"o"` cu `r=true` → Reset complet (snapshot)
- Mesaj `"o"` cu `r=false` → Delta (add/update/remove)
  - `s == 0` → Sterge nivelul de pret
  - `s > 0` → Adauga sau actualizeaza
- Top 20 nivele extrase pentru OFI si heatmap

### 9.4 OFI (Order Flow Imbalance)

**Formula:**

```
OFI = Σ(bid_size_deltas) - Σ(ask_size_deltas)  pe top 10 nivele
```

- **OFI pozitiv** → Net buying pressure (bullish)
- **OFI negativ** → Net selling pressure (bearish)
- **Smoothing:** EMA cu alpha = 0.3

**OFI ca filtru de intrare:**

- Buy signal: OFI > threshold (confirma presiunea de cumparare)
- Sell signal: OFI < -threshold (confirma presiunea de vanzare)
- Daca OFI nu confirma semnalul, intrarea e blocata

### 9.5 Spread Safety

- **Formula:** `spread_pct = (ask - bid) / mid * 100`
- Daca spread-ul depaseste limita configurata, intrarile sunt blocate
- Previne tranzactiile in conditii de lichiditate scazuta

### 9.6 Consolidarea Stream-ului L2

Datele L2 sunt procesate printr-un **singur** WebSocket in `streamer.rs`. Initial existau doua conexiuni separate (`streamer.rs` si `l2_heatmap.rs`), ceea ce cauza eroarea "connection limit exceeded (code 406)". Dupa consolidare:

- `streamer.rs` deschide o singura conexiune WebSocket
- Proceseaza mesaje de tip `"o"` (orderbook) si `"b"` (bar)
- Calculeaza OFI si emite `l2-update` si `ofi-update` catre frontend
- `l2_heatmap.rs` ramane disponibil ca modul dar nu mai deschide conexiuni proprii

### 9.7 Ce vezi in UI

- **SPR:** Spread-ul actual in basis points
- **Bara OFI:** Verde = buying pressure, Rosu = selling pressure
- **BID/ASK depth:** Volumul total pe fiecare parte
- **IMB:** Imbalance ratio (-1 la +1)

---

## 10. AUTOPILOT — MOTORUL DE TRADING AUTOMAT

### 10.1 Ciclul de trading (la fiecare 1.5 secunde)

1. **Verifica Circuit Breaker** — Daca nu e NORMAL, nu intra in pozitii noi
2. **Fetch bars** (OHLCV) de pe Alpaca
3. **Calculeaza Oracle Score** si/sau **GPU Nexus Consensus**
4. **Verifica OFI** — Semnalul trebuie confirmat de order flow
5. **Verifica Spread** — Spread-ul trebuie sa fie in limite sigure
6. **Verifica Cooldown** — Timp minim intre intrari
7. **Calculeaza pozitia** — Risk-based sizing
8. **Trimite ordinul** sau **simuleaza** (daca Dry Run e ON)

### 10.2 Intrari (Entry)

Conditii cumulative (TOATE trebuie indeplinite):

- Circuit breaker = NORMAL si entries_blocked = false
- Scor >= fusion_threshold (din genetic optimizer)
- Confidence >= min_confidence (din genetic optimizer)
- OFI confirma directia (daca ofi_filter_enabled)
- Spread este in limite
- Cooldown elapsed (timp minim de la ultima intrare)
- Pret valid (> 0)
- Max open positions nu e atins

### 10.3 Iesiri (Exit)

Verificari pe pozitiile existente (la fiecare tick):

- **Take Profit (TP):** Pretul a atins target-ul de profit
- **Stop Loss (SL):** Pretul a atins limita de pierdere
- **Trailing Stop:** Trail gap dinamic — urmareste pretul favorabil si iese daca retrage cu trail_gap_pct
- **Circuit Breaker exits:** Daca se activeaza HALT sau PANIC

### 10.4 Position Sizing (Risk-Based)

**Formula:**

```
risk_amount = equity * (risk_per_trade / 100) * breaker_multiplier
qty = risk_amount / |entry_price - stop_loss_price|
```

**Safety checks:**

- Notionalul maxim = 95% din equity
- Qty minim = 0.0001
- Daca pretul e 0 → Ordin respins
- Daca keys sunt goale → Ordin respins

### 10.5 Trailing Stop

- Se mentine `trail_high` (pentru long) sau `trail_low` (pentru short) per simbol
- La fiecare tick, se actualizeaza trail-ul daca pretul merge favorabil
- Daca pretul retrage cu > `trail_gap_pct` de la trail → EXIT
- La inchiderea pozitiei, trail data se sterge automat

---

### 10.6 Crypto vs Stocks — Detectie Automata

Cand trimite un ordin, botul detecteaza automat tipul de asset:

**Detectie crypto** (oricare din):

- Simbolul contine `USD` sau `/`
- Simbolul incepe cu: BTC, ETH, LTC, DOGE, SOL, AVAX, UNI, AAVE, LINK, DOT

**Diferenta cheie — Time In Force:**

- **Crypto:** `time_in_force: "gtc"` (Good Till Cancelled)
- **Stocks/ETF:** `time_in_force: "day"`

Trimiterea unui ordin crypto cu `"day"` cauzeaza eroare `422 Unprocessable Entity: invalid crypto time_in_force` pe Alpaca. Titan gestioneaza automat aceasta diferenta.

### 10.7 Gestionarea Erorilor Permanente la Exit (Fix 403 Forbidden)

O problemă critică a fost eroarea `403 Forbidden: insufficient qty` când botul încerca să închidă o poziție (Market Sell) în timp ce acțiunile erau blocate de un ordin Limit (Take Profit) deschis anterior.

**Soluția Titan V7 (10 Martie 2026):**
La fiecare semnal de Exit (Stop Loss, Take Profit sau Manual Sell), botul execută acum un **"Blind Cancel"** proactiv:

1. Trimite o cerere `DELETE` la Alpaca pentru a anula toate ordinele deschise pe simbolul respectiv.
2. Așteaptă **500 milisecunde** (pentru a asigura sincronizarea serverelor Alpaca).
3. Trimite ordinul Market Sell de lichidare.

Această logică eliberează cantitatea de acțiuni blocată și este proiectată să facă lichidarea să reușească în mod fiabil, eliminând eroarea 403.

### 10.8 Seeding Price Cache la Exit

Autopilot-ul foloseste `state.last_prices` pentru risk checks. La iesirea dintr-o pozitie, pretul curent poate sa nu fie in cache (daca simbolul nu mai streamuieste activ). Solutia:

Inainte de a trimite ordinul de inchidere, autopilot-ul copiaza `current_price` din datele pozitiei Alpaca in `last_prices`. Aceasta previne eroarea `"NO PRICE DATA for [SYMBOL]"`.

---

## 11. FILTRELE DE SIGURANTA (SAFETY GATES)

Filtrele de siguranta sunt **independente** de strategie si ruleaza inainte de fiecare intrare potentiala. Chiar daca strategia genereaza un semnal puternic, daca un filtru blocheaza, intrarea NU se face.

### 11.1 Lista Completa a Filtrelor

| # | Filtru | Cand Blocheaza | Mesaj in Log |
|---|--------|---------------|--------------|
| 1 | **Circuit Breaker** | Nivel != NORMAL | `CIRCUIT BREAKER: [level]` |
| 2 | **Spread Guard** | Spread > limita configurata | `SPREAD GUARD: spread [x] > max [y]` |
| 3 | **OFI Filter** | OFI nu confirma directia semnalului | `OFI FILTER: signal blocked` |
| 4 | **Cooldown** | Timp insuficient de la ultima intrare | `COOLDOWN: [x]s remaining` |
| 5 | **Max Positions** | Numar maxim de pozitii atins | `MAX POSITIONS: [n] reached` |
| 6 | **Sentiment Filter** | Sentiment prea negativ pentru buy (sau prea pozitiv pentru sell) | `SENTIMENT FILTER: blocked` |
| 7 | **Price Validity** | Pretul e 0 sau invalid | `NO PRICE DATA` |

### 11.2 Spread Guard — Detalii

**Ce e Spread-ul?** Diferenta intre cel mai bun bid si cel mai bun ask din orderbook.

**Formula:** `spread_pct = (best_ask - best_bid) / midpoint * 100`

**Prag configurabil:** Pragul SPREAD GUARD este controlat de `max_spread_bps` din **HFT Engine tab** (sectiunea Safety). Default: **50 bps** (0.50%). Interval configurabil: 1 — 500 bps. Anterior era hardcodat la 15 bps; acum foloseste direct valoarea din settings.

**Conversie:** `max_spread_bps / 100 = procent maxim spread acceptat`. Exemplu: 50 bps = 0.50%, 200 bps = 2.00%.

**De ce e important:** Un spread mare inseamna ca platesti un "cost de intrare" ridicat. Pe piete lichide (large-cap, BTC/USD), spread-ul e de 5-20 bps. Pe micro-caps cu volum mic, poate fi 100-300 bps. Ajustati `max_spread_bps` in functie de tipul de asseturi tranzactionate.

**Mesajele SPREAD GUARD in log:**

- `SPREAD GUARD: 152.7 bps — too wide, skipping entries` → Spread-ul actual depaseste limita configurata
- Acestea sunt **mesaje informative**, nu erori. Inseamna ca botul protejeaza capitalul asteptand conditii mai bune.

**Recomandari per tip de asset:**

- Large-cap (AAPL, MSFT, NVDA): 20-50 bps
- Mid-cap: 50-100 bps
- Small/micro-cap (SOC, QBTS): 150-250 bps
- Crypto (BTC/USD): 10-30 bps

**Exceptie Overnight/No-Data:** Daca spread-ul raportat este 0.0 bps (fara date in afara orelor de program), botul devine automat "safe" permitand tranzactiilor sa se pozitioneze in loc sa fie blocate artificial de lipsa conexiunii de adancime.

### 11.3 Safety Gates Avansate (S04.1) 🆕

Sesiunea 48 a adaugat 4 noi safety gates configurabile din panoul **⬡ SAFETY GATES** in HFT Engine tab:

| # | Gate | Setting | Default | Efect |
|---|------|---------|---------|-------|
| 8 | **VaR Fail-Closed** | `var_fail_closed` | ON | Blocheaza entry daca nu exista date de returns (fail-safe) |
| 9 | **Asset Check Hard** | `asset_check_hard_gate` | ON | Respinge ordine cand API-ul de assets e indisponibil |
| 10 | **Blackout First** | `blackout_first_mins` | 5 | Blocheaza entry in primele N minute dupa open (9:30 ET) |
| 11 | **Blackout Last** | `blackout_last_mins` | 5 | Blocheaza entry in ultimele N minute inainte de close (16:00 ET) |

**Blackout Exempt Symbols:** Simboluri care bypasseaza blackout-ul (ex: `SPY, QQQ`). Configurabil in UI.

**Min Dollar Volume:** Filtru scanner — elimina tickers cu `price × volume < $5M` (default). Previne trading pe instrumente illiquid.

**Validation Layer:** Warnings automate daca:
- Total blackout >= 195 min (jumatate din ziua de trading)
- Dollar volume < $100K (risc illiquid)
- VaR gate ON dar fail-closed OFF (risc gap)

### 11.4 Cum sa Interpretezi Log-ul

Cand vezi multe mesaje SPREAD GUARD sau OFI FILTER, nu inseamna ca botul e stricat. Inseamna:

- Piata nu ofera conditii optime de intrare in acel moment
- Filtrele functioneaza corect protejand capitalul
- Cand conditiile se imbunatatesc, botul va intra automat

---

## 12. GENETIC OPTIMIZER — EVOLUTIE AUTOMATA A PARAMETRILOR

### 12.1 Ce optimizeaza

| Parametru | Default | Range |
|-----------|---------|-------|
| `fusion_threshold` | 50.0 | ±15% din safety rails |
| `tp_percent` (Take Profit) | 2.5% | 0.5% — rails*1.15 |
| `sl_percent` (Stop Loss) | 2.5% | 0.5% — rails*1.15 |
| `trail_gap_pct` | 1.5% | 0.3% — rails*1.15 |
| `min_confidence` | 0.30 | 0.10 — 0.90 |
| `ofi_filter_enabled` | true | 90% pastreaza, 10% flip |
| `nexus_buy_threshold_pct` | 60% | 50 — 90 |

### 12.2 Cum functioneaza

**Populatie:** 20 indivizi (seturi de parametri)

**Ciclul de evolutie (la fiecare 60 secunde):**

1. **Evaluare fitness** pe datele in-sample (primele 70% din candles)
2. **Sortare** dupa fitness
3. **Selectie:** Top 5 = elites (supravietuiesc intact)
4. **Reproducere:** 15 noi offspring din combinatii de elites
   - 70% sansa: crossover (2 parinti) + mutatie
   - 30% sansa: mutatie directa (1 parinte)
5. **Mutatie:** Fiecare parametru ±15% cu probabilitate aleatoare
6. **Validare OOS:** Campionul e testat pe datele out-of-sample (ultimele 30%)
7. **Promovare:** Campionul e promovat DOAR daca:
   - Fitness IS > fitness campion curent
   - Sharpe IS > 0
   - Sharpe OOS > 0 si PnL OOS > 0

### 12.3 Walk-Forward Backtest

- **In-Sample (70%):** Evaluarea fitness-ului
- **Out-of-Sample (30%):** Validarea anti-overfitting
- **Logica:** SMA(5) vs SMA(20) crossover + volatilitate + threshold
- **Fitness formula:** `sharpe * min(profit_factor, 5) + trade_bonus - dd_penalty`

### 12.4 Safety Rails

Toti parametrii sunt clamped intre limite stricte:

- TP/SL nu pot fi sub 0.5%
- Trail gap nu poate fi sub 0.3%
- Confidence intre 0.10 si 0.90
- Threshold intre 50 si 90

### 12.5 Ce vezi in UI

- **Gen:** Generatia curenta (creste la fiecare evolutie)
- **Fitness:** Scorul cel mai bun din populatie
- **Sharpe:** Sharpe-ul campionului
- **TP/SL:** Parametrii activi de Take Profit / Stop Loss
- **FREEZE:** Opreste evolutia (pastreaza parametrii actuali)
- **BACKTEST NOW:** Ruleaza un backtest manual cu parametrii curenti

---

## 13. CIRCUIT BREAKER — SISTEMUL DE SIGURANTA

### 13.1 Cele 4 Nivele

| Nivel | Trigger | Efect | Culoare UI |
|-------|---------|-------|------------|
| **NORMAL** | PnL zilnic > -2% si DD > -5% | Trading normal, full size | Verde |
| **CAUTION** | PnL zilnic ≤ -2% | Pozitii injumatatite (50%) | Galben |
| **HALT** | PnL zilnic ≤ -3% | Intrari noi BLOCATE | Portocaliu |
| **PANIC** | DD ≤ -5% SAU PnL zilnic ≤ -5% | Kill switch — LICHIDEAZA TOT | Rosu |

### 13.2 Cum functioneaza

- **Evaluare continua** la fiecare tick al autopilot-ului
- **Threshold-uri configurabile** din sidebar Settings
- **Kill Switch:** In modul PANIC, se activeaza automat si inchide TOATE pozitiile
- **Size reduction:** In CAUTION, ordinele sunt la 50% din dimensiunea normala

### 13.3 Reset

- **Automat:** La daily reset (09:30 ET)
- **Manual:** Butonul RESET disponibil in UI
- **Din cod:** Comanda `reset_circuit_breaker`

### 13.4 Emergency Liquidate

Butonul **LIQUIDATE ALL** din UI:

1. Activeaza kill switch
2. Blocheaza intrarile
3. Anuleaza TOATE ordinele deschise
4. Inchide TOATE pozitiile
5. Emite eveniment "breaker-triggered" cu nivel PANIC

---

## 14. DRY RUN MODE — SIMULARE FARA RISC

### 14.1 Ce face

Cand Dry Run e activat:

- Ordinele **NU** se trimit la Alpaca
- In schimb, se logheaza si se afiseaza in Activity Log ca "DRY RUN ORDER"
- Toate celelalte functii (OFI, Oracle, Circuit Breaker) functioneaza normal
- Ideal pentru testare in conditii reale de piata

### 14.2 Activare/Dezactivare

- **Din UI:** Click pe butonul **DRY RUN: OFF** → devine **DRY RUN: ON**
- **Din environment:** Setare `TITAN_DRY_RUN=1` inainte de pornire
- **Din cod:** Comanda `set_dry_run(true/false)`

**Persistenta:** Starea Dry Run se salveaza in `titan_settings.json`. La repornire, botul isi aminteste daca era in Dry Run sau nu.

**Default (S72):** Fresh install porneste cu `dry_run_enabled: true`. Live trading trebuie activat explicit.

**Guard vizual (S72):** Cand dry_run e OFF (live trading), UI-ul afiseaza border rosu pulsant pe intregul ecran + banner `LIVE TRADING — DRY RUN OFF` pentru a preveni trading-ul accidental.

### 14.3 Ce se intampla in Dry Run

- `send_order_internal` → Logheaza ordinul, emite `dry-run-order`, return OK
- `liquidate_all_internal` → Logheaza, return fara actiune
- Activity Log arata: "DRY RUN: BUY 0.5 BTC/USD (simulated)"

---

## 15. WATCHDOG SI TELEMETRIE

### 15.1 Watchdog — Monitorizare Loop-uri

Fiecare loop din background trimite un **heartbeat** la fiecare iteratie. Watchdog-ul urmareste:

- **tick_count:** Cate iteratii a facut loop-ul
- **last_heartbeat:** Cand a fost ultimul heartbeat
- **last_tick_ms:** Durata ultimei iteratii
- **error_count:** Cate erori au aparut

**Loop-uri supravegheate (13):** autopilot, nexus, volatility, equity_sync, position_recon, trade_updates, ai_scanner, daily_reset, genetic_evo, catalyst_watchdog, streamer, news, telegram.

### 15.2 Dead Loop Detection

- **Threshold:** 120 secunde fara heartbeat = loop considerat DEAD
- **Watchdog Supervisor:** Verifica la fiecare 30 secunde
- **Alert:** Emite `watchdog-alert` catre UI cu lista loop-urilor moarte
- **UI:** Panoul LOOP HEARTBEATS arata "ALL ALIVE" (verde) sau "X DEAD LOOPS" (rosu)

### 15.3 Cancel/Join Contract pe Respawn (S03)

Watchdog-ul stocheaza un `AbortHandle` per loop. La respawn:

1. **Abort** -- task-ul vechi este abortat explicit (nu mai ruleaza in paralel)
2. **Spawn** -- task nou pornit si handle-ul stocat
3. **Log** -- "Aborted old task handle before respawn"

Aceasta elimina riscul de **split-brain** cand un loop "mort" inca ruleaza si un nou spawn il duplica.

### 15.4 Generation ID — Invalidare Automata (S03)

**`session_generation`** este un contor `AtomicU64` pe `TitanState`, incrementat la respawn-ul loop-urilor critice: **streamer**, **autopilot**, **trade_updates**.

Fiecare loop critic captureaza generatia la start. La fiecare tick, verifica `state.current_generation() == my_generation`. Daca nu coincide, loop-ul iese automat cu log:

```
"Generation mismatch — exiting stale autopilot instance"
```

**Efect:** operatiuni din generatia anterioara sunt invalidate automat (fail-closed). Nu exista fereastra in care doua instante ale aceluiasi loop proceseaza ordine simultan.

### 15.5 Ce vezi in UI (LOOP HEARTBEATS)

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

### 15.6 Telemetrie Detaliata

Comanda `get_telemetry` returneaza per loop:

- `name`, `tick_count`, `error_count`
- `last_tick_ms`, `avg_tick_ms`
- `last_heartbeat_secs_ago`
- `is_alive` (true/false)
- `uptime_secs` — cate secunde ruleaza loop-ul fara restart (resetat la respawn)

### 15.7 Watchdog Health Event (S03b)

La fiecare tick de 30 secunde, watchdog supervisor-ul emite un event **`watchdog-health`** catre frontend cu snapshot-ul complet al tuturor loop-urilor, generatia curenta si numarul de generation mismatch-uri:

```json
{
  "loops": [ { "name": "autopilot", "tick_count": 214, "uptime_secs": 3600.5, ... } ],
  "generation": 3,
  "generation_mismatches": 1
}
```

Aceasta da vizibilitate continua pe starea tuturor loop-urilor, nu doar pe cele moarte.

### 15.8 Generation Mismatch Counter (S03b)

**`generation_mismatch_count`** este un `AtomicU64` pe `TitanState`. Incrementat automat in cele 3 loop-uri critice (autopilot, trade_updates, streamer) cand detecteaza un generation mismatch si ies. Valoarea apare in event-ul `watchdog-health` sub cheia `generation_mismatches`.

### 15.9 Snapshot Integrity Hash (S03b)

La fiecare `save_state_snapshot`, dupa serializarea JSON, se calculeaza SHA-256 pe payload si se scrie intr-un fisier `.sha256` alaturi de snapshot. La `load_state_snapshot`, daca fisierul `.sha256` exista, hash-ul e verificat. Mismatch = log error + snapshot respins (nu se restaureaza date corupte).

---

## 16. LOGGING SI RAPOARTE

### 16.1 Structured Logging

- **Librarie:** `tracing` + `tracing-subscriber` + `tracing-appender`
- **Director:** `$HOME/HFT_Bot/logs/`
- **Format:** `titan.log.YYYY-MM-DD` (rotatie zilnica automata)
- **Continut:** Timestamps UTC RFC-3339, thread IDs, target module, level (DEBUG/INFO/WARN/ERROR)
- **Doua iesiri simultane:**
  - Fisier: Full detail cu target si thread
  - Stdout: Compact (vizibil in terminal)

### 16.2 Session Reports

La **shutdown** si la **daily reset** (09:30 ET), se salveaza un raport JSON:

**Locatie:** `$HOME/HFT_Bot/reports/session_YYYY-MM-DD.json`

**Continut:**

- Data sesiunii
- Total trades, winning, losing
- Daily PnL, Max Drawdown
- Win Rate, Profit Factor, Sharpe
- Peak Equity, Final Equity
- Champion params (din genetic optimizer)
- Loop telemetry (toate loop-urile)
- Breaker events count
- Dry run status

### 16.3 Activity Log (UI)

Toate evenimentele importante apar in panoul ACTIVITY LOG:

- Boot checks
- Trade fills
- Circuit breaker events
- WebSocket connections/disconnections
- OFI updates
- Erori

---

## 17. BACKTESTING

### 17.1 Tipuri de Backtest

| Tip | Descriere | Comanda |
|-----|-----------|---------|
| **Single Backtest** | Un simbol, un timeframe | `run_backtest` |
| **TF Sweep** | Un simbol, toate timeframe-urile | `run_backtest_sweep` |
| **Multi Sweep** | Mai multe simboluri, toate TF-urile | `run_backtest_sweep_multi` |
| **Quick Backtest** | Cu parametrii activi si simbolul curent | `run_backtest_now` |

### 17.2 Timeframe-uri suportate

1m, 5m, 15m, 30m, 1h, 4h, 1d

### 17.3 Export

- **CSV:** Date tabulare pentru Excel/Google Sheets
- **JSON:** Date structurate pentru analiza programatica
- **HTML:** Raport vizual cu grafice
- **Multi-Sweep CSV:** Comparatie intre simboluri si timeframe-uri

### 17.4 Auto-Optimize

Butonul **AUTO TUNE TF** ruleaza `optimize_timeframe`:

- Testeaza toate timeframe-urile pe ultimele 30 de zile
- Selecteaza timeframe-ul cu cel mai bun Sharpe ratio
- Actualizeaza automat timeframe-ul activ

---

## 18. GRAFICUL PROFESIONAL

### 18.1 Librarie

**TradingView LightweightCharts** — chart profesional cu:

- Candlestick OHLC
- Volume histogram
- Multi-pane layout (chart, volume, RSI, MACD, ADX)

### 18.2 Indicatori disponibili

| Indicator | Pane | Configurabil |
|-----------|------|-------------|
| **EMA 9** | Chart principal | Perioada |
| **EMA 50** | Chart principal | Perioada |
| **EMA 200** | Chart principal | Perioada |
| **VWAP** | Chart principal | - |
| **Bollinger Bands** | Chart principal | Perioada, StdDev |
| **RSI** | Sub-chart | Perioada (linii 30/50/70) |
| **MACD** | Sub-chart | Fast/Slow/Signal |
| **ADX** | Sub-chart | Perioada (linii 20/25) |
| **Oracle Score** | Sub-chart | - |

### 18.3 Drawing Tools

- Crosshair, Trendline, Ray, Horizontal Line, Vertical Line
- Fibonacci Retracement
- Text annotation
- Measure tool
- Long/Short position markers
- Delete All, Reset

### 18.4 Functii Speciale

- **Replay Mode:** Simuleaza miscarea pretului in timp real (viteza ajustabila)
- **Magnet:** Snap la OHLC
- **Screenshot:** Captura chart-ului
- **Fullscreen:** Chart pe tot ecranul
- **Resize:** Pane-urile sunt redimensionabile (P+/P-, H+/H-)
- **Timeframes:** 1m, 5m, 15m, 30m, 1h, 4h, 1d

### 18.5 TITAN Oracle Fullscreen

Histograma Oracle poate fi vizualizata fullscreen:

- **Buton FS** (coltul dreapta-sus al panoului Oracle)
- Fullscreen acopera tot viewport-ul (position: fixed)
- Graficul se redimensioneaza automat la viewport
- Click din nou pe FS pentru a reveni la dimensiunea normala

### 18.6 Live Feed

Sub chart se afiseaza:

- **BID / ASK** live (din L2 stream)
- **SPR** (Spread)
- **BIDS / ASKS** lists (orderbook top levels)
- **Orderbook Heatmap** (vizualizare depth)

---

## 19. TAB-URI NOI (NEWS AI, JURNAL, HFT ENGINE, PERFORMANCE)

### 19.1 News AI (tab-1)

Tab dedicat monitorizarii stirilor cu analiza de sentiment.

**Componente:**

- **Sentiment Gauge:** Indicator vizual circular (-100 la +100) cu culori gradient
- **News Timeline:** Flux cronologic de stiri cu timestamps
- **News Cards:** Fiecare stire afiseaza:
  - Titlu si sursa
  - Scor sentiment (pozitiv/negativ/neutru)
  - Culoare: verde (bullish), rosu (bearish), gri (neutru)
- **Statistici:** Media sentiment pe ultimele X ore

**Sentiment Analysis:**

- Keyword-based scoring cu lexicon extins
- Cuvinte bullish: "rally", "surge", "breakthrough", "uptick", "bullish", etc.
- Cuvinte bearish: "crash", "plunge", "selloff", "bearish", "decline", etc.
- Scorul se calculeaza per headline/summary si se medieaza

### 19.2 Jurnal (tab-3)

Istoric persistent al tranzactiilor.

**Functionalitati:**

- **Tabel de tranzactii:** Data, simbol, directie, cantitate, pret, PnL
- **Filtre:** Dupa simbol, data, directie (buy/sell), rezultat (win/loss)
- **Grafice:** Equity curve, PnL per trade, distributie rezultate
- **Export:** Buton export CSV pentru analiza externa
- **Persistenta:** Datele sunt stocate in `journal.rs` (backend Rust), nu in browser

### 19.3 HFT Engine (tab-4)

Setari avansate pentru motorul de trading:

**Entry Parameters:**

- Fusion threshold, min confidence
- OFI filter toggle
- Cooldown interval

**Exit Parameters:**

- Take Profit %, Stop Loss %
- Trail gap %
- Max holding time

**Safety:**

- Circuit breaker thresholds (editabile)
- Max open positions
- Risk per trade (slider scurt, 15% din latime)

**Alerting:**

- Discord webhook URL
- Selectie evenimente: fills, breaker events, daily summary
- Test alert button

**Slidere in HFT Engine:** Au latime redusa (15% din normal) pentru a ocupa mai putin spatiu si a arata compact.

**SPREAD GUARD — Conexiune la Settings:**
Parametrul `max_spread_bps` din HFT Engine controleaza direct pragul SPREAD GUARD din autopilot. Anterior era hardcodat la 15 bps (0.15%), acum foloseste valoarea din settings (default 50 bps = 0.50%). Aceasta valoare determina spread-ul maxim bid-ask acceptat pentru a intra intr-un trade. Asseturile cu lichiditate scazuta (micro-caps) au spread-uri de 100-300 bps si necesita cresterea acestei valori.

### 19.4 Performance Dashboard

Integrated in panoul TITAN SAFETY si in tab-urile de backtest:

**KPI Cards:**

- Daily PnL cu trend arrow
- Win Rate cu progress bar
- Sharpe Ratio
- Profit Factor
- Max Drawdown
- Total Trades

---

## 20. AI SCANNER — FUSION ASSET DISCOVERY

### 20.1 Ce este AI Scanner

AI Scanner este un motor inteligent de descoperire a asseturilor profitabile. Combina filtre cantitative (quant) cu scoring FUSION (Oracle + GPU Nexus) si backtest automat pe toate timeframe-urile pentru a gasi si clasifica cele mai promitente actiuni US.

Accesibil din tab-ul **AI Scanner** (tab-6) in bara de taburi din panoul central.

### 20.2 Pipeline-ul de Scanare

Scanarea se desfasoara in 4 faze:

1. **Fetch Assets** — Descarca lista completa de actiuni active de pe Alpaca (~8000+)
2. **Snapshots** — Incarca datele zilnice (pret, volum, previous day) in batch-uri de 1000
3. **Pre-filter Quant** — Aplica filtre cantitative si sorteaza dupa scor compozit
4. **FUSION Scoring + Sweep Backtest** — Pentru fiecare candidat:
   - Descarca candle-uri 5m
   - Ruleaza Oracle 7-layer (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, Changepoint)
   - Dispatches GPU Nexus (150,000 agenti voteaza)
   - Calculeaza FUSION score (Oracle *(1-alpha) + Nexus* alpha)
   - Ruleaza backtest sweep pe **8 timeframe-uri**: 1m, 5m, 10m, 15m, 30m, 1h, 4h, 1D
   - Pastreaza cel mai bun timeframe (dupa Sharpe + Return combinat)
   - Calculeaza AI Score compozit

### 20.3 Filtre Cantitative (Pre-filter)

| Filtru | Conditie |
|--------|----------|
| Pret | $1 - $500 |
| Volum minim | 500,000 |
| Schimbare minima | > 1% (absolut) |
| RVOL | volume_today / volume_yesterday |
| Gap % | (open - prev_close) / prev_close * 100 |
| Volume spike | volume > 2x prev_volume |

**Scoring pre-filter:**

- Change% absolut (max 20%) * 2
- RVOL > 2x: (RVOL-1) * 5 (max 50 puncte)
- Gap > 1%: gap * 3 (max 30 puncte)
- Volume spike bonus: +15 puncte
- High volume bonus: +5 (>2M) sau +10 (>5M)

### 20.4 AI Score — Formula Compozita

```
AI Score = 0.40 * FUSION_normalized +
           0.25 * Sharpe_normalized +
           0.20 * Win_Rate +
           0.15 * ProfitFactor_normalized
```

Conditii: minim 3 trades in backtest si Oracle confidence > 0.15. Altfel, AI Score = FUSION * 0.5.

### 20.5 Sweep Backtest Multi-Timeframe

Fiecare asset scanat trece prin backtest pe toate cele 8 timeframe-uri. Se pastreaza timeframe-ul cu cel mai bun scor combinat (Sharpe + Return * 0.01). Coloana **BEST TF** din tabel arata timeframe-ul optim.

Profunzimea backtestului depinde de datele disponibile de la Alpaca (pana la 365 zile, limita 1000 candle-uri per timeframe).

### 20.6 Coloanele Tabelului AI Scanner

| Coloana | Descriere |
|---------|-----------|
| # | Rank dupa AI Score |
| SYMBOL | Simbolul actiunii |
| PRICE | Pretul curent |
| CHG% | Schimbare procentuala fata de ziua anterioara |
| RVOL | Relative Volume (today / yesterday) |
| GAP% | Gap la deschidere fata de previous close |
| VOL | Volumul zilei |
| FLOAT | Categorie float estimat: LOW / MED / HIGH |
| ORACLE | Scorul Oracle (-100 la +100) |
| REGIME | Regimul pietei: Trending / MeanReverting / Chaotic |
| NEXUS | GPU Nexus buy % (150K agenti) |
| FUSION | Scorul FUSION combinat (-100 la +100) |
| SIGNAL | Semnalul: LONG / SHORT / NEUTRAL |
| BEST TF | Timeframe-ul optim din sweep backtest |
| TRADES | Numar tranzactii din backtest |
| WIN% | Win rate din backtest |
| SHARPE | Sharpe ratio din backtest |
| RET% | Return % din backtest |
| EXPECT | Expectancy per trade din backtest |
| AI SCORE | Scorul compozit final (0-100) |

### 20.7 Functionalitati UI

- **AI AUTO-SCAN (switch ON/OFF):** In header-ul tab-ului AI Scanner, switch-ul "AI AUTO-SCAN" activeaza/dezactiveaza scanarea automata in background (la fiecare 30 minute). Cand e OFF, doar SCAN NOW manual ruleaza. Starea se persista in `titan_settings.json` (`ai_scan_active`).
- **Candidate switch (50/100/200):** Controleaza numarul de asseturi trecute prin pre-filter
- **SCAN NOW:** Porneste scanul manual (dureaza 2-10 minute in functie de numarul de candidati)
- **Progress bar:** Afiseaza faza curenta si simbolul procesat in timp real
- **Filtre client-side:** Symbol search, Signal (LONG/SHORT/NEUTRAL), Regime
- **Sortare:** Click pe orice header de coloana pentru sortare asc/desc
- **Color coding:** Randuri verzi (LONG), rosii (SHORT), neutre (gri)
- **Checkbox per row:** Selecteaza asseturile dorite
- **INJECT SELECTED → HFT:** Butonul verde adauga asseturile selectate direct in HFT Engine targets si activeaza multi-symbol mode

### 20.8 SMART PICKS (Asset-uri Premium Filtrate)

Panoul **⚡ SMART PICKS** este o sub-sectiune a AI Scanner care curata zgomotul si prezinta doar "crema" pietei.
Filtrul actioneaza POST-scanare si retine exclusiv candidatele de momentum (high-reward):

- **Win Rate:** > 20% (Prinde breakout-urile masive de momentum ce pierd des dar castiga mult cand prind trendul)
- **Trades:** Minim 5 trade-uri in backtest (Elimina rezultatele accidentale din doar 2-3 trade-uri)
- **Sharpe Ratio:** > 0.0 (Indică profitabilitate ajustată la risc pozitivă pe backtest — rezultat simulat, nu garanție)

**Functionalitati Smart Picks:**

- Include coloana **BEST TF** indicand exact intervalul orar unde s-a demonstrat profitabilitatea in istoric.
- **Top N Selector:** Permite izolarea primelor 3, 5, 10 sau 20 de asset-uri (Top 20 ofera marja de eroare solida contra riscului macro).
- **MANUAL Mode:** Utilizatorul poate revizui manual lista Premium si injecta folosind **INJECT ALL -> HFT** (butoanele indicand instant vizual `✓ ACTIVE`).
- **AUTO Mode:** Injecteaza automat in HFT Engine numarul configurat (Top N) de monede direct de pe scanner la finalizarea acestuia (usurinta totala in fundal).

### 20.9 Auto-Scan (Background)

- `ai_scan_loop` ruleaza automat la fiecare **30 minute** in orele de piata (09:00-17:00 ET) — **doar daca switch-ul AI AUTO-SCAN este ON**
- Switch-ul din header permite oprirea auto-scanului fara a opri aplicatia (economiseste API calls si CPU)
- Scaneaza top 100 candidati
- Emite `ai-scan-update` event — tabelul se actualizeaza automat
- Heartbeat la watchdog la fiecare 60 secunde
- Nu scaneaza in afara orelor de piata

### 20.10 Injectare in HFT Engine

Dupa scanare, selectati asseturile dorite cu checkbox si apasati **INJECT SELECTED → HFT**. Acest lucru:

1. Adauga simbolurile in lista `hft_targets` din `titan_settings.json`
2. Activeaza `multi_symbol_enabled = true`
3. Autopilotul va incepe sa evalueze si aceste simboluri la urmatoarea iteratie

---

## 21. MANAGEMENTUL POZITIILOR SI FONDURILOR

### 21.1 Pozitii

- **Sincronizare automata:** La fiecare 15 secunde, pozitiile sunt sincronizate de pe Alpaca
- **Afisare:** In panoul "Portofoliu Activ" cu simbol, pret, TF, tip, investit, P/L
- **Actiuni per pozitie:** Buton SELL pentru inchidere manuala

### 21.2 Equity Sync

- La fiecare 30 secunde, equity-ul este sincronizat de pe Alpaca
- Afisare in panoul TITAN SAFETY si in sidebar

### 21.3 Trade Updates (Fill Notifications)

- WebSocket conectat la Alpaca Trade Updates stream
- Primeste notificari in timp real pentru:
  - Ordine executate (fills)
  - Ordine partial executate
  - Ordine anulate
- Afiseaza in Activity Log: simbol, cantitate, pret mediu

### 21.4 Fonduri

| Actiune | Buton | Ce face |
|---------|-------|---------|
| **Top Up User** | TOPUP USER | Adauga la depozitul virtual al utilizatorului |
| **Top Up Bot** | TOP UP | Aloca fonduri catre bot din depozitul user |
| **Withdraw** | RETRAGE | Retrage fonduri din bot catre user |

---

## 22. TOATE LOOP-URILE DIN BACKGROUND

| # | Loop | Interval | Ce face |
|---|------|----------|---------|
| 1 | **autopilot_loop** | 1.5s | Trading automat: analiza, intrari, iesiri, trailing stop |
| 2 | **nexus_loop** | 500ms | GPU Nexus: 150K agenti voteaza, consens calculat |
| 3 | **update_volatility_loop** | 30s | Actualizeaza cache-ul de volatilitate |
| 4 | **equity_sync_loop** | 30s | Sincronizeaza equity de pe Alpaca |
| 5 | **position_reconciliation_loop** | 15s | Sincronizeaza pozitiile deschise de pe Alpaca |
| 6 | **trade_updates_stream** | Continuu | WebSocket: notificari fill-uri in timp real |
| 7 | **l2_stream_manager** | Continuu | L2 orderbook stream (crypto v1beta3 / stocks SIP) |
| 8 | **daily_reset_loop** | 60s (check) | Reset zilnic la 09:30 ET |
| 9 | **genetic_evolution_loop** | 60s | Evolutie genetica a parametrilor de trading |
| 10 | **watchdog_supervisor** | 30s | Detecteaza loop-uri moarte |
| 11 | **startup_health_check** | O data (boot) | Verifica toate dependentele |
| 12 | **streamer::run_streamer** | Event-driven | Price streaming si heatmap |
| 13 | **ai_scan_loop** | 30min (heartbeat 60s) | AI Scanner: pre-filter + FUSION + sweep backtest |
| 14 | **state_snapshot_loop** | 30s | Snapshot state pe disk (crash recovery) |

---

## 23. TOATE COMENZILE DISPONIBILE

### 23.1 Trading

| Comanda | Ce face |
|---------|---------|
| `submit_order` | Trimite ordin (prin safety checks + risk sizing) |
| `liquidate_all_positions` | Inchide toate pozitiile |
| `cancel_all_orders` | Anuleaza toate ordinele deschise |
| `emergency_liquidate` | Kill switch + liquidate |
| `set_dry_run` | Activeaza/dezactiveaza simularea |

### 23.2 Strategie

| Comanda | Ce face |
|---------|---------|
| `set_strategy_mode` | Seteaza Oracle / GPU / Fusion |
| `toggle_strategy_mode` | Cicleaza intre moduri |
| `set_nexus_alpha` | Ajusteaza blend-ul Fusion (0-1) |
| `get_nexus_status` | Status GPU Nexus |

### 23.3 Genetic Optimizer

| Comanda | Ce face |
|---------|---------|
| `freeze_evolution` | Opreste evolutia Fusion |
| `resume_evolution` | Porneste evolutia Fusion |
| `get_evolution_stats` | Statistici optimizer Fusion |
| `get_active_trading_params` | Parametrii activi Fusion |
| `run_backtest_now` | Backtest rapid |

#### SmartTrend Genetic (S13)

| Comanda | Ce face |
|---------|---------|
| `get_st_evolution_stats` | Statistici optimizer SmartTrend (gen, fitness, sharpe, diversity, champion params) |
| `get_st_fitness_history` | Istoric fitness IS/OOS per generatie (pentru convergence chart) |
| `get_st_champion_params` | Drill-down champion: fiecare param cu value, default, delta%, min, max |
| `run_comparison_backtest_cmd` | Ruleaza Fusion + SmartTrend pe aceleasi candle, returneaza metrics side-by-side |
| `freeze_st_evolution` | Opreste evolutia SmartTrend independent |
| `resume_st_evolution` | Porneste evolutia SmartTrend |

**Comportament conditional:** Cand `StrategyMode::SmartTrend` este activ, `genetic_evolution_loop` foloseste automat `SmartTrendParams` si `smarttrend_walk_forward_backtest` (FSM real: Flat/Stalking/Entry/InTrade/Trail). In orice alt mod, ruleaza optimizatorul Fusion clasic cu `TradingParams`. Cele doua optimizatoare sunt complet independente.

**SmartTrendParams evolvabile (12):** macro_weight, micro_weight, momentum_weight, entry_threshold, stalking_threshold, changepoint_reset, trail_tighten_pct, hysteresis_margin, bayesian_halflife, stabilization_window, dominant_cycle_filter, risk_per_trade. Fiecare are safety rails (min/max) hardcoded.

**Fitness SmartTrend:** `sharpe * min(PF, 5) + trade_count_bonus + win_rate_bonus + stability_bonus - dd_penalty`. Win rate bonus activ peste 55%. Stability bonus cand avg_bars_in_trade intre 3-30. OOS validation: champion trebuie sharpe > 0 si total_pnl > 0 pe 30% out-of-sample.

**UI Evolution Panel (S13):**
- **Convergence Chart:** Canvas in panoul EVOLUTION — linie IS fitness (cyan) + OOS (gold dashed) pe generatii, green dots pe champion promovat, diversity bar la baza
- **PARAMS button:** Click -> modal drill-down cu toate champion params, delta% vs default, heat-color bar proportional cu magnitude
- **DIV badge:** Badge "DIV: N%" in header panel — rosu cu pulse sub 15% (premature convergence), cyan 15-40%, verde peste 40%
- **Comparison Panel:** Sectiune noua in tab-13 SMARTTREND CFG — RUN COMPARISON ruleaza ambele strategii pe aceleasi date, grid cu winner highlighting, verdict banner

### 23.4 Risk & Safety

| Comanda | Ce face |
|---------|---------|
| `get_risk_status` | Circuit breaker + session analytics |
| `reset_circuit_breaker` | Resetare manuala breaker |
| `reset_daily_analytics` | Reset analytics zilnice |
| `get_ofi_status` | Status OFI live |

### 23.5 Cont & Fonduri

| Comanda | Ce face |
|---------|---------|
| `connect_api` / `update_api_keys` | Conectare/update API keys |
| `get_account_snapshot` | Snapshot cont Alpaca |
| `topup_user` / `topup_bot` / `withdraw_bot` | Gestiune fonduri |
| `get_wallet_snapshot` | Status portofel |

### 23.6 Backtesting

| Comanda | Ce face |
|---------|---------|
| `run_backtest` | Backtest singur |
| `run_backtest_sweep` | Sweep pe toate TF-urile |
| `run_backtest_sweep_multi` | Multi-symbol sweep |
| `optimize_timeframe` | Auto-optimizare TF |
| `export_backtest_csv/json/html` | Export rezultate |

### 23.7 Monitorizare

| Comanda | Ce face |
|---------|---------|
| `get_telemetry` | Telemetrie loop-uri |
| `get_monitor_snapshot` | Hardware (CPU, GPU, RAM, NET) |
| `graceful_shutdown` | Oprire gratiata |

### 23.8 Settings & Persistenta

| Comanda | Ce face |
|---------|---------|
| `update_settings` | Actualizeaza orice setare (field + value) — include `bot_trade_size`, `circuit_breaker_limit`, `dry_run_enabled`, `strategy_mode_name` |
| `get_favorites` | Returneaza lista de simboluri favorite |
| `add_favorite` | Adauga simbol la favorite (persistent) |
| `remove_favorite` | Sterge simbol din favorite (persistent) |

### 23.9 AI Scanner

| Comanda | Ce face |
|---------|---------|
| `run_ai_scan` | Porneste scan AI cu numar configurabil de candidati (50/100/200) |
| `activate_scan_targets` | Injecteaza simbolurile selectate in HFT Engine targets |

---

## 24. STRUCTURA FISIERELOR

```
$HOME/HFT_Bot/
├── titan_keys.json              ← API keys (CONFIDENTIAL)
├── titan_settings.json          ← Setari persistente (dry_run, strategy_mode, favorites, etc.)
├── titan_wallet.json            ← Stare portofel (user deposit, bot alloc)
├── titan_state_snapshot.json    ← State snapshot (crash recovery, auto-resume)
├── titan_position_blacklist.json ← Blacklist pozitii force-closed
├── TITAN_MANUAL_RO.md           ← Acest manual
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

## 25. PROBLEME FRECVENTE SI SOLUTII

### 25.1 L2 Stream nu primeste date

**Simptom:** OFI LIVE arata 0.0, BID/ASK = 0

**Cauze posibile:**

- **Crypto:** Endpoint-ul trebuie sa fie `v1beta3/crypto/us` (nu `v2/sip`)
- **Stocks:** Necesita plan **Algo Trader Plus** ($99/mo) pentru SIP
- **API Keys:** Verificati ca sunt corecte si fara spatii

**Verificare:** In terminal, cautati:

```
L2 SUB RESPONSE: [{"T":"subscription","orderbooks":["BTC/USD"],...}]
```

### 25.2 Trade Updates nu se autentifica

**Simptom:** "Auth failed" in Activity Log

**Solutie:** Titan incearca automat doua formate de auth:

1. `authenticate` cu `key_id`/`secret_key` (format v2)
2. `auth` cu `key`/`secret` (format legacy)

Verificati Activity Log pentru mesajul exact de la server.

### 25.3 Circuit Breaker se activeaza

**Simptom:** TITAN SAFETY arata CAUTION/HALT/PANIC

**Solutie:**

- **CAUTION:** Normal — reduce pozitiile automat. Asteptati recuperare.
- **HALT:** Intrarile sunt blocate. Revizuiti strategia. Reset manual daca e nevoie.
- **PANIC:** Lichidare automata. Investigati cauza. Reset dupa analiza.

### 25.4 Loop mort (DEAD)

**Simptom:** LOOP HEARTBEATS arata un loop cu `> 120s`

**Solutie:** Restartati aplicatia. Daca persista, verificati logurile pentru erori specifice loop-ului respectiv.

### 25.5 GPU nu e detectat

**Simptom:** Health Check arata "GPU (WGPU): FAILED"

**Solutie:**

- Verificati driverele NVIDIA: `nvidia-smi`
- Verificati suportul Vulkan: `vulkaninfo`
- Setati `WEBKIT_DISABLE_COMPOSITING_MODE=1` (se face automat)

### 25.6 EGL spam in terminal

**Simptom:** Mesaje repetitive EGL/WebKit in terminal

**Solutie:** Titan seteaza automat `WEBKIT_DISABLE_COMPOSITING_MODE=1` la pornire. Daca persista, adaugati in `.bashrc`:

```bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
```

### 25.7 "Connection limit exceeded (code 406)"

**Simptom:** In Activity Log apare repetat:

```
L2 STREAM AUTH ERROR: [{"T":"error","code":406,"msg":"connection limit exceeded"}]
L2 STREAM: Quick disconnect — failure #N, backoff Xs
```

**Cauza:** Alpaca permite un numar limitat de conexiuni WebSocket simultane pe cont. Daca doua componente deschid conexiuni la acelasi stream, limita e atinsa.

**Solutie:** Aceasta problema a fost rezolvata prin consolidarea tuturor conexiunilor L2 in `streamer.rs`. Daca reapar:

1. Verificati ca nu exista instante multiple ale aplicatiei
2. Restartati aplicatia (kill → re-launch)
3. Asteptati ~60 secunde pentru ca Alpaca sa elibereze conexiunile vechi

### 25.8 "AUTOPILOT EXIT FAILED: NO PRICE DATA"

**Simptom:** Autopilot-ul incearca repetat sa inchida o pozitie dar esueaza.

**Cauza (rezolvata):** Cache-ul intern `last_prices` nu avea pretul simbolului daca acesta nu era activ streaming.

**Solutie:** Autopilot-ul acum copiaza automat `current_price` din datele pozitiei Alpaca in cache inainte de a trimite ordinul de exit. Daca apare totusi:

1. Verificati ca L2 stream-ul e conectat
2. Verificati ca simbolul e valid pe Alpaca

### 25.9 "422 Unprocessable Entity: invalid crypto time_in_force"

**Simptom:** Ordinele pe crypto (BTC/USD, ETH/USD) esueaza cu eroare 422.

**Cauza (rezolvata):** Alpaca necesita `time_in_force: "gtc"` pentru crypto, nu `"day"`.

**Solutie:** Titan detecteaza automat crypto vs stocks si seteaza TIF corect. Aceasta problema nu ar trebui sa mai apara.

### 25.10 Aplicatia ruleaza versiune veche dupa rebuild

**Simptom:** Dupa modificari in `index.html`, aplicatia afiseaza UI-ul vechi.

**Cauze posibile:**

1. **Build incremental Tauri** — Nu re-embeds frontend-ul daca doar HTML s-a schimbat
2. **WebKit cache** — Browser-ul intern cache-uieste stilurile vechi

**Solutie:**

```bash
# 1. Sterge build cache-ul Tauri
rm -rf ${TITAN_HFT_ROOT:-.}/src-tauri/target/release/build/titan_hft_rust-*

# 2. Sterge WebKit cache
rm -rf ~/.cache/titan_hft_rust/WebKitCache

# 3. Rebuild
cd ${TITAN_HFT_ROOT:-.}/src-tauri && cargo build --release

# 4. Relanseaza aplicatia
```

### 25.11 Butoane/Tab-uri nu functioneaza (CSP)

**Simptom:** Click pe un buton (ex: HFT ENGINE tab) nu face nimic.

**Cauza:** Tauri WebView blocheaza `onclick="..."` inline din motive de Content Security Policy.

**Solutie:** Folositi `data-` attributes si `addEventListener` in loc de `onclick`:

```html
<!-- Corect: -->
<button data-symtab="hft">HFT ENGINE</button>

<!-- Gresit (blocat de CSP): -->
<button onclick="switchSymbolTab('hft')">HFT ENGINE</button>
```

JavaScript-ul atasaza event listener-ii la `DOMContentLoaded`.

### 25.12 Text invizibil in inputuri/selecturi

**Simptom:** Nu se vede textul in casutele de input din sidebar.

**Cauza:** WebKit aplica stiluri native (fundal alb, text negru) care suprascriu tema dark a aplicatiei.

**Solutie:** Aplicata cu `color-scheme: dark !important` pe toate elementele input/select din sidebar:

```css
#sidebar input, #sidebar select {
    background: #000 !important;
    color: #FFD700 !important;
    color-scheme: dark !important;
}
```

Daca se intampla din nou dupa rebuild, stergeti WebKit cache-ul (vezi 25.10).

### 25.13 SPREAD GUARD blocheaza toate intrarile pe un asset

**Simptom:** Log-ul arata constant `SPREAD GUARD: X bps — too wide, skipping entries` si botul nu intra niciodata.

**Cauza:** Asset-ul are spread natural mai mare decat limita `max_spread_bps` configurata. Micro-caps (SOC, penny stocks) au frecvent spread-uri de 100-300 bps.

**Solutie:** Cresteti `max_spread_bps` din HFT Engine tab. Recomandari: 50 bps pentru large-caps, 100-200 bps pentru small/micro-caps. Alternativ, alegeti asseturi cu lichiditate mai buna (volum > 2M, spread < 50 bps).

### 25.14 WATCHDOG: Dead loops — ai_scanner sau autopilot

**Simptom:** Mesaje rosii `WATCHDOG: Dead loops: ai_scanner` sau `autopilot` in Activity Log.

**Cauza ai_scanner (rezolvata):** Anterior, `ai_scan_loop` trimitea heartbeat doar la fiecare 30 minute, depasind threshold-ul watchdog de 120 secunde.

**Solutie ai_scanner:** Loop-ul trimite acum heartbeat la fiecare 60 secunde (sub threshold-ul de 120s), independent de intervalul de scan de 30 minute.

**Cauza autopilot:** Autopilotul poate parea "mort" daca SPREAD GUARD blocheaza constant si loop-ul petrece tot timpul in `continue` fara sa ajunga la executia efectiva. Heartbeat-ul e trimis la inceputul fiecarei iteratii (1.5s), deci nu ar trebui sa apara in mod normal. Daca apare, restartati aplicatia.

### 25.15 AI Scanner — scanul dureaza foarte mult

**Simptom:** Dupa click SCAN NOW, progress bar avanseaza foarte lent (mai ales cu 200 candidati).

**Cauza:** Fiecare candidat necesita ~9 cereri API Alpaca (1 pentru candle-uri 5m + 8 pentru sweep backtest pe toate timeframe-urile). Cu 200 candidati = ~1800 cereri.

**Solutie:** Alpaca rate limit e ~200 req/min. Scanul cu 200 candidati poate dura 10-15 minute. Folositi 50 candidati pentru rezultate rapide (~3 minute).

---

## NOTA FINALA

Titan HFT este un sistem complex cu multiple straturi de siguranta. Recomandarile principale:

1. **Porniti intotdeauna in DRY RUN** pana cand sunteti confortabil cu comportamentul
2. **Monitorizati Activity Log** pentru anomalii
3. **Nu modificati manual threshold-urile** Circuit Breaker fara sa intelegeti implicatiile
4. **Lasati Genetic Optimizer-ul** sa ruleze cel putin cateva ore inainte de trading live
5. **Verificati LOOP HEARTBEATS** periodic — toate loop-urile trebuie sa fie ALIVE
6. **Session Reports** din `$HOME/HFT_Bot/reports/` sunt sursa de adevar pentru performanta
7. **Mesajele SPREAD GUARD** sunt normale — inseamna ca filtrele functioneaza si protejeaza capitalul
8. **Dupa modificari frontend**, stergeti WebKit cache si faceti clean build (vezi sectiunea 25.10)
9. **Setarile persista automat** in `titan_settings.json` — dry_run, strategy_mode, favorites, trade size
10. **Strategia genereaza semnalul, filtrele il valideaza** — vezi sectiunea 6 si 11 pentru detalii

---

## 26. CATALYST WATCHDOG — SENTIENT NEWS FEED

Sistem avansat de monitorizare a știrilor în timp real, conceput pentru a identifica evenimente "Black Swan" sau momentum-uri majore pe piața de **acțiuni (Stocks)** înainte ca acestea să fie complet digerate de piață.

### 26.1 Cum funcționează

- **Polling Loop:** Rulează în background (`sentiment.rs`) și verifică fluxul Alpaca News la fiecare 60 de secunde.
- **OnceLock Lexicon:** Lexiconul (~100 termeni + regex) se compilează o singură dată via `std::sync::OnceLock`, eliminând overhead-ul la fiecare iterație.
- **Parallel Check:** `check_catalyst` procesează simboluri în paralel (`futures_util::join_all`), chunking la 5 simboluri cu timeout 10s.
- **Crypto Filter:** Ignoră complet știrile crypto.
- **Sentiment Engine:** Lexicon bursier avansat cu analiză de negație "lookback" pentru **Confidence Score** (0.00 - 1.00).
- **Bearish Support:** Alertează atât bullish (score ≥ 0.50) cât și bearish (score ≤ -0.50).
- **Per-Symbol Cooldown:** 5 minute cooldown per simbol contra alert spam.

### 26.2 Severity Levels

| Score | Severity | Culoare |
|-------|----------|---------|
| ≥ 0.85 sau ≤ -0.85 | **EXPLOSIVE** | Verde / Rosu intens |
| ≥ 0.50 sau ≤ -0.50 | **STRONG** | Verde / Rosu |
| ≥ 0.35 | **MODERATE** | Galben |
| < 0.35 | **MILD** | Gri |

### 26.3 Tab-ul 7 — Catalyst Watchdog Dashboard

**Header:**
- Titlu magenta cu badge `alerts` count
- Butoane pe rând separat: 🔥 TEST BULLISH (gradient magenta) | 🔻 TEST BEARISH (gradient rosu) | CLEAR LOG
- Butoanele au `white-space:nowrap` — nu se trunchiaza

**Stats Bar (5 carduri responsive):**
- **Total Catalysts** — contor total alerte
- **Bullish** — contor alerte pozitive (verde)
- **Bearish** — contor alerte negative (rosu)
- **Last Alert** — ultimul simbol + oră
- **Watchdog Status** — LIVE (verde pulse) / NO_KEYS / DOWN

**Tabel Log (7 coloane):**

| Coloana | Descriere |
|---------|-----------|
| TIME | Ora alertei (HH:MM AM/PM) |
| SYMBOL | Ticker (click deschide pe chart) |
| HEADLINE | Titlul stirii (link clickabil) |
| SEVERITY | Badge colorat (EXPLOSIVE/STRONG/MODERATE/MILD) |
| SCORE | Scorul sentiment (colorat) |
| SENTIMENT | BULLISH (verde) / BEARISH (rosu) |
| ACTION | Buton INJECT (adauga in HFT Targets) |

**Filtre:** Simbol, Scor minim, Sentiment (All/Bullish/Bearish)
**Max rows:** 100 (auto-eviction cele mai vechi)

### 26.4 Toast Notifications V3

- **Bullish toast:** Verde/cyan, sunet 2-tone
- **Bearish toast:** Rosu/magenta, sunet deep bass
- **Stacking:** Counter badge pentru alerte multiple (+2, +3 etc.)
- **Auto-dismiss:** 8 secunde cu bara de progres
- **Butoane:** VIEW (deschide Tab 7), INJECT (adauga in HFT), X (inchide)

### 26.5 Backend Commands

| Command | Descriere |
|---------|-----------|
| `test_catalyst_alert` | Emite alertă mock bullish (NVDA, +0.98) |
| `test_catalyst_bearish` | Emite alertă mock bearish (TSLA, -0.92) |
| `check_catalyst` | Verifică știri per simbol, returnează `CatalystCheckResult` |
| `get_news_feed` | Returnează `SentimentSnapshot` cu `label_from_score()` |

---

## 27. SEMNALE ȘI SĂGEȚI PE GRAFIC

Sistemul de semnalizare vizuală este strategy-specific — săgețile reflectă modul de strategie activ:

### 27.1 Mod Oracle — Săgeți Verzi/Roșii (Threshold ±8.0)

- **BUY (verde, ▲ sub bar):** Scorul Oracle traversează +8.0 de jos în sus.
- **SELL (roșu, ▼ deasupra bar):** Scorul Oracle traversează -8.0 de sus în jos.
- Recalibrat de la ±15.0 (prea mare pentru volatilitatea curentă) la ±8.0.

### 27.2 Mod GPU Nexus — Săgeți Cyan/Roz (Momentum ±0.02)

- **GPU BUY (cyan `#00BFFF`, ▲ sub bar):** GPU momentum traversează +0.02.
- **GPU SELL (roz `#FF6EC7`, ▼ deasupra bar):** GPU momentum traversează -0.02.
- Momentum-ul e calculat per-candle din dot product ponderat pe returnuri (ferestre 64/128/256 candle).

### 27.3 Mod Fusion — Săgeți Aurii/Portocalii (Threshold Adaptiv)

- **FUSION BUY (auriu `#FFD700`, ▲ sub bar):** Scorul Fusion traversează threshold-ul adaptiv.
- **FUSION SELL (portocaliu `#FF4500`, ▼ deasupra bar):** Scorul Fusion traversează -threshold.
- **Formula Fusion:** `oracle_score × (1 - alpha) + gpu_momentum × 100 × alpha`
- **Threshold adaptiv:** `8.0 × (1 - alpha) + 2.0 × alpha` — se ajustează automat cu slider-ul Fusion Alpha.

### 27.4 Changepoint Markers (Toate Modurile)

- **Puncte portocalii (`#FF8800`, ● deasupra bar, text "BREAK"):** Indică regime breaks detectate de BOCPD.
- **Trigger:** Probabilitate changepoint > 0.7.
- Active în toate cele 3 moduri de strategie.

### 27.5 Toggle Vizibilitate

- Checkbox-ul **"Signals"** din legenda graficului controlează vizibilitatea săgeților.
- Starea toggle-ului persistă între schimbările de simbol/timeframe.

### 27.6 Culori per Mod (Referință Rapidă)

| Mod | BUY | SELL | Threshold |
|-----|-----|------|-----------|
| **Oracle** | Verde `#00FF41` | Roșu `#FF2B2B` | ±8.0 |
| **GPU Nexus** | Cyan `#00BFFF` | Roz `#FF6EC7` | ±0.02 momentum |
| **Fusion** | Auriu `#FFD700` | Portocaliu `#FF4500` | Adaptiv (8.0×(1-α) + 2.0×α) |
| **Changepoint** | — | — | BOCPD prob > 0.7 |

---

## 28. TELEGRAM ALERTS — NOTIFICARI MOBILE

Sistem de alertare in timp real pe telefonul mobil prin Telegram Bot API.

### 28.1 Setup Initial (BotFather)

1. Deschide Telegram si cauta **@BotFather**.
2. Trimite `/newbot` si urmeaza pasii pentru a crea un bot nou.
3. Copiaza **Bot Token** (format: `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`).
4. Trimite un mesaj botului tau nou creat (pentru a initializa conversatia).
5. Acceseaza `https://api.telegram.org/bot<TOKEN>/getUpdates` in browser.
6. Copiaza **chat_id** din raspunsul JSON (numarul din `"chat":{"id":123456789}`).

### 28.2 Configurare in Titan

In tab-ul **HFT Engine (tab-4)**, sectiunea **TELEGRAM ALERTS**:

- **Bot Token:** Introdu token-ul BotFather.
- **Chat ID:** Introdu chat_id-ul tau.
- **Enable:** Checkbox pentru activare/dezactivare.
- **TEST:** Buton pentru a trimite un mesaj de test ("Titan HFT Connected!").

### 28.3 Tipuri de Alerte

| Eveniment | Cand se trimite | Informatii incluse |
|-----------|----------------|--------------------|
| **Trade Fill** | La executarea unui ordin | Simbol, side (BUY/SELL), cantitate, pret, PnL |
| **Circuit Breaker** | La activarea PANIC | Nivelul breaker, motivul |
| **Catalyst** | La detectarea unei stiri majore | Simbol, scor, headline |
| **Daily Summary** | La reset zilnic (09:30 ET) | Total trades, win rate, PnL, profit factor |

### 28.4 Securitate

- Mesajele sunt trimise prin HTTPS catre API-ul Telegram.
- Token-ul e stocat local in `titan_settings.json`, nu se transmite altundeva.
- Alertele se trimit doar daca `telegram_enabled = true` si token/chat_id sunt non-empty.

---

## 29. RISK CORRELATION MATRIX

Analiza corelatiei intre simbolurile tranzactionate simultan in modul Multi-Symbol, pentru a evita expunerea redundanta.

### 29.1 Ce este Corelatia

Corelatia Pearson masoara cat de similar se misca doua active:

- **+1.0** — Se misca identic (expunere dubla la acelasi risc).
- **0.0** — Nicio legatura (diversificare perfecta).
- **-1.0** — Se misca opus (hedge natural).

### 29.2 Cum Functioneaza in Titan

La fiecare tentativa de entry in modul Multi-Symbol:

1. Titan descarca candle-uri recente pentru simbol candidat si simbolurile deja detinute.
2. Calculeaza returnurile procentuale din preturi close.
3. Aplica `pearson_correlation` intre candidat si fiecare pozitie existenta.
4. Daca corelatia depaseste `max_correlation` (default 0.70), entry-ul e **blocat**.

**Exemplu:** Daca detii AAPL si incerci sa intri pe MSFT (corelatie ~0.85), Titan blocheaza entry-ul deoarece ambele sunt Large-Cap Tech si se misca similar.

### 29.3 Configurare

In tab-ul **HFT Engine (tab-4)**:

- **Slider "Max Corr":** Seteaza pragul maxim de corelatie acceptata (0.30 - 0.95, default 0.70).
- **Correlation Window:** Numarul de candle-uri folosite pentru calcul (default 60).

### 29.4 Vizualizare in UI

Sectiunea **RISK CORRELATION MATRIX** din tab-4 afiseaza:

- **Tabel matrice:** Perechi de simboluri cu corelatia calculata (rosu = risc, verde = safe).
- **Buton REFRESH:** Recalculeaza matricea pe baza datelor curente.

---

## 30. PORTFOLIO PERFORMANCE & REBALANCING

Analiza si optimizare a portofoliului pe baza performantei istorice din Trade Journal.

### 30.1 Metrici Per Simbol

Din datele din jurnal, Titan calculeaza pentru fiecare simbol:

| Metrica | Descriere |
|---------|-----------|
| **Trades** | Numarul total de tranzactii |
| **Win Rate** | Procentul de tranzactii profitabile |
| **Total PnL** | Profitul/pierderea cumulata |
| **Avg PnL** | Profitul mediu per tranzactie |
| **Performance Score** | Scor compozit: `win_rate * 40 + pnl_magnitude * 30 + trade_bonus * 30` |
| **Tag** | Clasificare: STAR / GOOD / AVERAGE / UNDERPERFORMER |

### 30.2 Risk Multiplier

Performanta istorica influenteaza direct dimensiunea pozitiilor viitoare:

| Tag | Risk Multiplier | Efect |
|-----|----------------|-------|
| **STAR** | 1.5x | Pozitii mai mari pe simboluri performante |
| **GOOD** | 1.2x | Pozitii usor mai mari |
| **AVERAGE** | 1.0x | Dimensiune standard |
| **UNDERPERFORMER** | 0.5x | Pozitii injumatatite pe simboluri slabe |

### 30.3 Rebalancing

Butonul **REBALANCE** din tab-4 analizeaza portofoliul si sugereaza:

- **Remove:** Simboluri cu pierderi consistente (tag UNDERPERFORMER, PnL < 0, >3 trades).
- **Demote:** Simboluri cu performanta sub medie dar nu dezastruoasa.
- **Promote:** Simboluri STAR care nu sunt inca in HFT targets.

### 30.4 Vizualizare in UI

Sectiunea **PORTFOLIO PERFORMANCE** din tab-4:

- **Tabel:** Simbol, Trades, Win Rate, PnL, Tag (colorat), Risk Multiplier.
- **Buton REFRESH:** Recalculeaza metrici din jurnal.
- **Buton REBALANCE:** Afiseaza actiunile sugerate.

---

## 31. ATR DYNAMIC STOP-LOSS & PARTIAL PROFIT TAKING

Sistem de iesire adaptiv la volatilitate, inlocuind stop-loss/take-profit fixe cu valori dinamice bazate pe ATR.

### 31.1 Ce este ATR (Average True Range)

ATR masoara volatilitatea medie a unui instrument pe o perioada data. True Range = max(High-Low, |High-PrevClose|, |Low-PrevClose|). ATR = media True Range pe N perioade (default 14).

### 31.2 Cum Functioneaza Stop-Loss ATR

In loc de un stop-loss fix (ex: -2.5%), Titan calculeaza:

```
stop_loss_price = entry_price - ATR * atr_stop_multiplier    (pentru LONG)
stop_loss_price = entry_price + ATR * atr_stop_multiplier    (pentru SHORT)

take_profit_price = entry_price + ATR * atr_take_multiplier  (pentru LONG)
take_profit_price = entry_price - ATR * atr_take_multiplier  (pentru SHORT)
```

**Avantaj:** Pe piete volatile, stop-ul se largeste automat (evita stop-out prematur). Pe piete calme, stop-ul se strange (protejeaza profitul).

### 31.3 Configurare

In tab-ul **HFT Engine (tab-4)**, sectiunea **ATR DYNAMIC STOPS**:

| Parametru | Default | Range | Descriere |
|-----------|---------|-------|-----------|
| **Stop-Loss (x ATR)** | 2.0 | 0.5 - 5.0 | Multiplicator ATR pentru stop-loss |
| **Take-Profit (x ATR)** | 3.0 | 1.0 - 10.0 | Multiplicator ATR pentru take-profit |
| **Partial Take %** | 50% | 25% - 75% | Procentul din pozitie vandut la partial |
| **Partial Profit Enable** | OFF | ON/OFF | Activeaza/dezactiveaza partial profit |

### 31.4 Partial Profit Taking

Cand profitul atinge 50% din target-ul take-profit:

1. Titan vinde automat `partial_take_pct` (ex: 50%) din pozitie.
2. Restul pozitiei continua cu trailing stop activ.
3. Se logheaza "PARTIAL TAKE: sold X% of SYMBOL" in Activity Log.
4. Se trimite alerta Telegram (daca activat).

**Exemplu:** Entry AAPL $180, TP = $186 (ATR * 3.0 = $6). La $183 (50% din $6 = $3 profit), se vinde 50% din pozitie. Restul ramane cu trailing stop.

### 31.5 Trailing Stop cu ATR

Trailing stop-ul e calculat pe baza ATR:

```
trail_pct = ATR / entry_price * atr_stop_multiplier * 0.5
```

Se actualizeaza `trail_high` (LONG) sau `trail_low` (SHORT) la fiecare tick. Exit daca pretul retrage cu > trail_pct de la peak.

---

## 32. WEB DASHBOARD — ACCES LOCAL

Dashboard web servit de un server HTTP embedded (axum) direct din Titan, accesibil pe localhost.

### 32.1 Acces

- **URL:** `http://127.0.0.1:7777` (localhost only — nu este expus pe LAN)
- **Compatibilitate:** Orice browser (Chrome, Safari, Firefox).
- **Autentificare:** Toate endpoint-urile (inclusiv `/` si `/ws`) sunt protejate cu Bearer token.

### 32.2 Ce Afiseaza

- **Cards:** Equity, Daily PnL, Win Rate, Pozitii deschise, Autopilot, Strategy, Breaker, Trades.
- **Tabel Pozitii:** Simbol, side, cantitate, entry price, current price, PnL (colorat verde/rosu cu neon glow).
- **Tabel Trades:** Ultimele 10 tranzactii cu timestamp si PnL.
- **WebSocket Live:** Status push la 2 secunde, trade/alert events, auto-reconnect (zero polling).
- **Design:** Cyber-HUD dark theme (Share Tech Mono, neon cyan/gold/green palette, scan-line animation), mobile-responsive.

### 32.3 API Endpoints

| Endpoint | Metoda | Auth | Descriere |
|----------|--------|------|-----------|
| `/api/health` | GET | Nu | Health check: status, uptime, version |
| `/api/build-info` | GET | Nu | Build info: version, git hash, build date, uptime (S04) |
| `get_state_health` | Tauri | -- | State decomposition health: lock contention per sub-state (S04.5) |
| `/` | GET | Bearer | Dashboard HTML cyber-HUD complet |
| `/api/status` | GET | Bearer | Equity, autopilot, strategy, breaker, dry_run |
| `/api/positions` | GET | Bearer | Lista pozitii cu PnL |
| `/api/trades` | GET | Bearer | Ultimele 20 trades |
| `/api/stats` | GET | Bearer | Win rate, PnL, profit factor |
| `/api/autopilot` | POST | Bearer | Toggle autopilot |
| `/ws` | GET | Bearer | WebSocket live push (status, trade, alert) |

### 32.4 Securitate

- **Token:** Toate endpoint-urile (cu exceptia `/api/health`) sunt protejate cu `Authorization: Bearer <token>`. Token-ul este `web_dashboard_token` (generat automat la prima rulare, 64 hex chars).
- **Token Rotation:** Comanda Tauri `rotate_dashboard_token` regenereaza token-ul instant si persista pe disk.
- **Acces:** Serverul asculta exclusiv pe `127.0.0.1` (localhost). Nu este accesibil din LAN.
- **CORS:** Permite doar originile `http://127.0.0.1:<port>` si `http://localhost:<port>`.
- **Rate Limit:** 60 cereri/minut sliding window pe toate rutele protejate (inclusiv WS upgrade).
- **WS Rate Limit:** Maximum 10 mesaje client per tick (2s). Depasirea deconecteaza clientul.
- **Auth Logging:** Esecurile de autentificare sunt logate; la fiecare 5 esecuri consecutive se emite alert de tip brute-force.

### 32.5 Configurare

In tab-ul **HFT Engine (tab-4)**, sectiunea **WEB DASHBOARD**:

- **Enable:** Toggle activare/dezactivare server.
- **Port:** Port HTTP (default 7777, range 1024-65535).
- **Token:** Afisat in UI, buton COPY URL. Poate fi rotit cu `rotate_dashboard_token`.

---

## 32.6 State Architecture (S04.5)

Incepand cu V4.0, `TitanState` este descompus in 4 sub-state-uri clare, fiecare cu lock ordering intern documentat:

| Sub-state | Modul | Campuri | Scop |
|-----------|-------|---------|------|
| **TradingState** | `trading_state.rs` | 10 Mutex | Trail tracking, entry metadata, exit inflight, blacklist, cooldowns |
| **MarketDataState** | `market_state.rs` | 5 Mutex + ArcSwap | Preturi, candle buffers, indicator cache, vol/bars cache, OFI |
| **RiskState** | `risk_state.rs` | 3 Mutex + atomics | Circuit breaker, session analytics, drawdown, flash crash |
| **ScanState** | `scan_state.rs` | 5 Mutex | AI scan pipeline: best-TF, results, progress, prefilter, cancel |

**Lock ordering global:** 40 slots in 9 grupuri, complet documentat in `state.rs`. Orice achizitie de Mutex trebuie sa respecte ordinea.

**State Health Dots:** In sidebar STATUS, 4 mini-dots (TRD/MKT/RSK/SCN) arata sanatatea sub-state-urilor in timp real:
- Verde: toate Mutex-urile libere
- Galben: 1 Mutex contended (lock contention normala)
- Rosu: 2+ Mutex-uri contended (potential bottleneck)

Hover pe fiecare dot afiseaza detalii (locks free / contended, generatie sesiune).

**Tauri command:** `get_state_health` returneaza per sub-state: name, fields, locks_ok, locks_contended, plus session_generation si uptime.

---

## 33. STATE PERSISTENCE — CRASH RECOVERY

Sistem complet de snapshot/restore care asigura ca botul supravietuieste crash-urilor si restart-urilor fara a pierde context.

### 33.1 Ce se Salveaza

La fiecare **30 secunde**, Titan scrie un snapshot complet in `$HOME/HFT_Bot/titan_state_snapshot.json`:

| Data | Descriere |
|------|-----------|
| **trail_highs / trail_lows** | Trailing stop tracking per simbol |
| **partial_taken** | Flag-uri partial profit per simbol |
| **last_prices** | Cache preturi per simbol |
| **symbol_best_tf** | Timeframe optim per simbol din AI scan |
| **nexus_fusion_alpha** | Valoarea curenta fusion alpha |
| **session_analytics** | Equity, trades, PnL, drawdown, trade_pnls complet |
| **dd_velocity_ring** | Ring buffer drawdown velocity (timestamp, dd_pct) — restaurat la restart (S03) |
| **session_generation** | Contor generatie sesiune — monotonic, invalidare ordine stale (S03) |
| **last_scan_results** | Toata tabela AI scan (toate randurile) |
| **scan_meta** | Era un scan in progres? Cate procesate? Total cerut? |

### 33.2 Auto-Restore la Startup

La pornire, Titan cauta snapshot-ul si il restaureaza automat:

1. Citeste `$HOME/HFT_Bot/titan_state_snapshot.json`
2. Verifica varsta — ignora snapshots mai vechi de **12 ore** (datele ar fi prea stale)
3. Populeaza trail stops, prices, analytics, scan results in memorie
4. Emite `ai-scan-update` la frontend — tabela de scan apare **instant** fara re-scanare
5. Afiseaza in Activity Log: "SNAPSHOT RESTORE: X scan results loaded, Y trail stops, Z cached prices"

### 33.3 Auto-Resume Scan (Continuare de la Punctul de Intrerupere)

Daca snapshot-ul indica ca un scan era in progres cand aplicatia s-a oprit:

1. Rezultatele partiale apar **instant** in UI (ex: 158 simboluri deja procesate)
2. Asteapta 5 secunde (pentru ca API-urile sa se conecteze)
3. Continua scanul **de unde a ramas** (ex: de la 158/200), procesand doar simbolurile ramase
4. Progress bar-ul porneste de la punctul de intrerupere (ex: 158/200)
5. La final, merge rezultatele vechi + noi, re-sorteaza, si afiseaza lista completa
6. Afiseaza: "AUTO-RESUME: Continuing scan from 158/200 (42 remaining)"

**Cum functioneaza tehnic:** Pre-filter queue-ul (lista completa de simboluri de procesat) se salveaza in snapshot. La resume, se construieste un HashSet din simbolurile deja procesate si se proceseaza doar cele ramase. Rezultatele se merge incremental (salvate pe parcurs in state pentru safety).

### 33.4 Graceful Shutdown

La apasarea butonului **SHUTDOWN**, Titan salveaza un ultim snapshot inainte de oprire, asigurand ca absolut tot state-ul e pe disk.

### 33.5 Scriere Atomica

Snapshot-ul se scrie atomic (write to `.tmp` file, then rename) pentru a preveni coruptia la crash in timpul scrierii. Journal-ul (`TradeJournal::save()`) foloseste acelasi pattern (S72).

### 33.5b Delta Compression (S72)

`save_state_snapshot` compara SHA-256 hash-ul noului snapshot cu hash-ul anterior (stocat in `.sha256` sidecar file). Daca sunt identice, scrierea pe disk este omisa. Reduce semnificativ I/O in perioadele idle.

### 33.6 Ce NU se Salveaza (Intentionat)

| Data | Motiv |
|------|-------|
| `bars_cache` | Candle-uri stale, re-fetch la cerere |
| `gpu_score_cache` | Recalculat per tick |
| `ofi_state` | Date L2 live, fara sens dupa restart |
| `nexus_consensus` | Recalculat de GPU agents |
| `news_cache` | Re-fetch de background loop |
| `circuit_breaker` | Mai sigur sa resetezi dupa crash |

---

## 34. SMART SCORE V4 — RANKING INSTITUTIONAL

Formula de ranking a asseturilor din AI Scanner, optimizata pentru a favoriza calitatea si consistenta.

### 34.1 Formula de Baza

```
base = 0.35 * Win% + 0.22 * Sharpe_norm + 0.15 * Trades_norm
     + 0.10 * PF_norm + 0.10 * Expect_norm + 0.08 * Fusion_norm
```

### 34.2 Multiplier Win% (Cel Mai Important)

| Win Rate | Multiplier | Efect |
|----------|-----------|-------|
| < 30% | 0.25x | Eliminare agresiva |
| 30-38% | 0.45x | Penalizare severa |
| 38-45% | 0.60x | Penalizare moderata |
| 45-50% | 0.82x | Sub-par dar acceptabil |
| 50-60% | 1.00x | Standard |
| 60-70% | 1.12x | Bonus |
| 70%+ | 1.20x | Bonus maxim |

### 34.3 Alte Multipliers

| Factor | Conditie | Multiplier |
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

### 34.4 Consistency Bonus (NOU in V4)

Simbolurile care exceleaza pe **toate** axele primesc bonus suplimentar:

| Conditie | Bonus |
|----------|-------|
| Win >= 50% AND Sharpe >= 1.0 AND Trades >= 10 | 1.15x |
| Win >= 55% AND Sharpe >= 1.0 | 1.08x |

### 34.5 Hard Kill

Daca oricare din conditiile urmatoare e adevarata, SMART score se reduce la `fusion_normalized * 0.10`:

- Trades < 3
- Sharpe <= 0
- Win Rate < 15%
- Expectancy < -0.01

### 34.6 Rezultat

Formula produce un ranking unde top 20 SMART PICKS sunt predominant simboluri cu Win% > 50%, Sharpe > 1.0, si suficiente trades pentru semnificanta statistica. Simbolurile cu Win% sub 40% nu mai pot compensa prin numar mare de trades.

---

## 35. AUTO ENTRY — CONTROL INTRARI AUTOMATE

### 35.1 Ce Este

Toggle-ul **AUTO ENTRY** din sidebar (sectiunea TRADING CONTROLS) controleaza daca botul deschide pozitii noi automat.

### 35.2 Distinctia AUTOPILOT vs AUTO ENTRY

| Toggle | Ce Face | Default |
|--------|---------|---------|
| **AUTOPILOT** | Gestioneaza exit-urile: stop-loss, take-profit, trailing stop pe pozitii existente | ON |
| **AUTO ENTRY** | Deschide pozitii noi automat pe baza semnalelor FUSION/Nexus | OFF |

**Important:** Ambele trebuie sa fie ON pentru trading complet automat. AUTOPILOT singur = botul inchide pozitii dar nu deschide altele noi.

### 35.3 Safety Gates pentru Entry

Cand AUTO ENTRY e ON, botul verifica urmatoarele inainte de a deschide o pozitie:

1. Circuit Breaker = NORMAL
2. Spread Guard < limita
3. Cooldown elapsed
4. Max positions nu e atins
5. Scor strategie >= threshold
6. OFI confirma directia
7. Nexus consensus >= threshold
8. Correlation check (daca Multi-Symbol)

### 35.4 Recomandare

Activati AUTO ENTRY **doar dupa** ce:
- Ati verificat ca semnalele FUSION apar corect pe chart
- Ati rulat cel putin un scan si verificat SMART PICKS
- Ati configurat risk_per_trade si max_open_positions
- Ati testat in DRY RUN

---

## 36. SIGNAL ALIGNMENT — FUSION-AWARE AUTOPILOT & CAUSAL CHART

### 36.1 Problema Anterioara

Doua probleme distincte afectau calitatea semnalelor:

1. **Autopilot ignora GPU/Fusion:** Functia `compute_strategy_score` genera semnalul LONG/SHORT/NEUTRAL folosind DOAR Oracle cu threshold +/-30. In modul FUSION, GPU Nexus era complet ignorat. Pe simboluri cu Oracle score mic (~0.03), semnalul era mereu NEUTRAL — botul nu intra niciodata.

2. **Chart signals cu hindsight:** `adaptive_thresholds` calcula percentile 80/20 pe TOATE scorurile din fereastra vizibila (retrospectiv). Un semnal SELL la $17.60 aparea doar pentru ca algoritmul "vedea" deja caderea de la $18.80 — bias de hindsight.

### 36.2 Solutia: compute_strategy_score_with_mode

Functia `compute_strategy_score` a fost refactorizata in `compute_strategy_score_with_mode(candles, mode, alpha, gpu_momentum)`:

| Mod | Sursa Semnal | Threshold |
|-----|-------------|-----------|
| **ORACLE** | Oracle score | +/-8.0 (redus de la +/-15), confidence > 0.10 |
| **GPU** | GPU momentum | +/-0.003 (redus de la +/-0.005) |
| **FUSION** | `oracle_norm * (1-alpha) + gpu_signal * alpha` | Adaptiv, max 6.0 |

Toti callerii (`autopilot_loop`, `get_strategy_score`) paseaza acum `strategy_mode` si `nexus_fusion_alpha` din state.

### 36.3 Solutia: Causal Rolling Lookback (Chart)

Inlocuit `adaptive_thresholds` (retrospectiv) cu **rolling lookback** pe ultimele 40 bare:

- La fiecare bar `i`, threshold-urile se calculeaza din `scores[max(0, i-40)..i]`
- Doar bare **precedente** — nicio informatie din viitor
- Semnalele sunt oneste si reflecta conditiile reale din momentul generarii
- Eliminat complet bias-ul de hindsight din chart markers

### 36.4 Impact

- Autopilotul genereaza acum semnale corecte in modul FUSION (cel mai folosit)
- Chart-ul arata semnale realiste, nu "perfect hindsight"
- Semnalele din chart si din autopilot sunt acum aliniate (aceeasi logica)

---

## 37. AUTO-RESUME SCAN — CONTINUARE DE LA PUNCTUL DE INTRERUPERE

### 37.1 Problema

La restart dupa un scan intrerupt (ex: 158/200), auto-resume pornea de la 0/200. Pre-filter pipeline-ul (fetch assets -> snapshots -> quant_prefilter) este non-deterministic (depinde de date live), deci nu se putea garanta aceeasi lista de simboluri la restart.

### 37.2 Solutia

Salvarea pre-filter queue-ului (lista completa de simboluri de procesat) in StateSnapshot:

1. **`PreFilterCandidate`** primeste `Serialize, Deserialize` derives
2. **`scan_prefilter_queue`** adaugat in `TitanState` si `StateSnapshot`
3. Queue-ul se salveaza dupa `quant_prefilter` si se sterge la completare
4. **`auto_resume_scan`** rescris complet:
   - Accepta `saved_queue` + `partial_results`
   - Skip simboluri deja procesate (HashSet lookup)
   - Progress de la `done_count` (ex: 158/200)
   - Merge incremental pe parcurs
   - Final: merge + re-sort + re-rank + emit

### 37.3 Comportament la Restart

| Pas | Ce se intampla |
|-----|---------------|
| 1 | Rezultatele partiale (158) apar instant in UI |
| 2 | Log: "AUTO-RESUME: Continuing scan from 158/200 (42 remaining)" |
| 3 | Progress bar porneste de la 158/200 |
| 4 | Proceseaza doar cele 42 simboluri ramase |
| 5 | Lista completa de 200 simboluri ranked apare in UI |

### 37.4 Safety

- Rezultatele noi se salveaza incremental in state (snapshot-ul periodic le captureaza)
- Daca aplicatia crasheaza din nou in timpul resume-ului, la urmatorul restart va continua de unde a ramas (ex: 178/200)
- `#[serde(default)]` pe `scan_prefilter_queue` asigura backward compatibility cu snapshot-uri vechi

---

## 38. MULTI-SYMBOL ENTRY FIX & DEL ALL

### 38.1 Multi-Symbol Entry Fix

**Problema:** Cu 21 simboluri in HFT Engine si Multi-Symbol activat, nicio pozitie deschisa in 4 ore.

**Cauze identificate:**

1. `max_open_positions = 1` — doar 1 pozitie simultana permisa
2. OFI filter calculat pe `active_symbol` dar aplicat pe TOATE simbolurile
3. Nexus consensus filter — la fel, doar pe active_symbol dar aplicat global
4. `compute_strategy_score` ignora GPU/Fusion (vezi Cap. 36)

**Solutii aplicate:**

- `max_open_positions` crescut la 5
- OFI filter: in multi-symbol mode, se aplica DOAR pe `active_symbol`; celelalte simboluri trec fara OFI gate (datele OFI sunt irelevante pentru ele)
- Nexus consensus filter: la fel, doar pe `active_symbol`
- Diagnostic logging: `ENTRY BLOCKED: auto_entry=OFF / breaker blocking / spread too wide` (0.8% sampling rate)

### 38.2 DEL ALL Button

Buton rapid pentru stergerea tuturor simbolurilor din HFT Engine list:

- **Locatie:** In header-ul listei HFT targets, langa sageata toggle
- **Stil:** Rosu cyberpunk pe fundal inchis (`#1a0800`)
- **Comportament:** Sterge doar simbolurile IDLE — pozitiile active sunt **protejate** automat
- **Confirm dialog:** "Delete all IDLE targets? Active positions will be kept."
- **Backend:** `clear_all_hft_targets` — `hft_targets.retain(|s| active_set.contains(s))`

---

## 39. ENTRY GATE SCORING PIPELINE — DEBOTTLENECKING

### 39.1 Problema (14 Martie 2026)

Entry gate-ul nu a permis nicio intrare. Analiza a 42 de loguri (sampled la 1%, reprezentand ~4200 evaluari reale) a aratat scor=50.0 si signal=NEUTRAL pe toate simbolurile, pe o fereastra de 8 minute (06:23-06:31). Scorul era blocat la baseline-ul neutru de 50.0 din cauza unui lant de 5 bottleneck-uri multiplicative.

### 39.2 Cele 5 Bottleneck-uri (Inainte de Fix)

Pipeline-ul de scoring avea urmatorii pasi, fiecare reducand semnalul:

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

### 39.3 Fix-urile Aplicate

**Fix 1 — meta_confidence: produs → media geometrica**

| Inainte | Dupa |
|---------|------|
| `predictability * clarity * stability` | `(predictability * clarity * stability).cbrt()` |
| Exemplu: 0.4 * 0.5 * 0.3 = **0.06** | Exemplu: (0.4 * 0.5 * 0.3)^(1/3) = **0.39** |

Media geometrica pastreaza intent-ul (toate 3 trebuie rezonabile) dar nu colapseaza la near-zero.

**Fix 2 — noise_ratio: penalty injumatatit cu floor**

| Inainte | Dupa |
|---------|------|
| `meta_conf * (1 - noise_ratio).max(0)` | `meta_conf * (1 - noise_ratio * 0.5).max(0.25)` |
| noise_ratio=0.7 → factor 0.3 | noise_ratio=0.7 → factor 0.65 |

Floor de 0.25 previne moartea totala a semnalului in conditii de zgomot ridicat (pre-market).

**Fix 3 — score formula: eliminat multiplicator redundant 0.5**

| Mod | Inainte | Dupa |
|-----|---------|------|
| MathOracle | `50 + oracle_val * 0.5 * oracle_conf` | `50 + oracle_val * oracle_conf` |
| GpuNexus | `50 + gpu_scaled * 0.5` | `50 + gpu_scaled` |
| Fusion | `50 + fusion_val * 0.5 * fusion_conf` | `50 + fusion_val * fusion_conf` |

**Fix 4 — signal thresholds reduse**

| Mod | Inainte | Dupa |
|-----|---------|------|
| MathOracle | oracle_val > 15.0, conf > 0.15 | oracle_val > 8.0, conf > 0.10 |
| GpuNexus | gpu_momentum > 0.005 | gpu_momentum > 0.003 |
| Fusion | fusion_thresh fara cap | fusion_thresh cu .min(6.0) |

**Fix 5 — log message corect**

| Situatie | Inainte | Dupa |
|----------|---------|------|
| Signal NEUTRAL | `score 50.0 < threshold 40.0` (misleading) | `signal NEUTRAL (score=50.0, no directional conviction)` |
| Score sub threshold | Aceeasi ca mai sus | `score X < threshold Y (signal=LONG)` |

### 39.4 Fluxul Corectat

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

### 39.5 Impact Numeric

Cu valori tipice (predictability=0.4, clarity=0.5, stability=0.3, noise_ratio=0.7, raw_signal=0.5):

| Metrica | Inainte | Dupa |
|---------|---------|------|
| meta_confidence | 0.06 | 0.39 |
| confidence (final) | 0.018 | 0.25 |
| oracle_score | 2.4 | 15.6 |
| entry score | 50.02 | 53.9 |
| signal | NEUTRAL (blocat) | LONG (trece) |

### 39.6 Fisiere Modificate

| Fisier | Locatie | Modificare |
|--------|---------|-----------|
| `titan_oracle.rs` | linia 1171 | meta_confidence: `.cbrt()` |
| `titan_oracle.rs` | linia 1202 | confidence: `* 0.5`, `.max(0.25)` |
| `titan_oracle.rs` | linia 1431 | series path: `.cbrt()` (consistent cu fuse_oracle) |
| `main.rs` | liniile 2690-2716 | score formula fara 0.5, thresholds reduse |
| `main.rs` | liniile 4905-4916 | log message diferentiat NEUTRAL vs score<threshold |

---

## 40. ORACLE HISTORY FIXES (S05)

### 40.1 Ce s-a corectat

Cinci buguri critice in `calculate_history()` (folosit pentru chart overlay, backtest, SmartTrend) au fost corectate:

| Bug | Efect inainte | Fix |
|-----|---------------|-----|
| **Entropy double-normalization** | clarity ~0.88-1.0 constant (gresit) | clarity = `1 - normalized_entropy` (corect [0,1]) |
| **noise_ratio hardcodat 0.3** | Confidenta identica indiferent de zgomot | Foloseste noise_ratio real din wavelet decomposition |
| **VPIN buy_pressure = VPIN score** | buy_pressure era toxicity, nu BVC fraction | BVC buy fraction: `(close-low)/(high-low)` per-bara |
| **Changepoint threshold 0.5 vs 0.3** | History mai permisiva cu stabilitatea | Ambele paths folosesc 0.3 (constanta partajata) |
| **Backtest exit exact match** | Exituri prefixate ("O EXIT") nedetectate | `contains("EXIT")` in loc de `== "EXIT"` |

### 40.2 Impact

- **Scorurile oracle din istorie** sunt acum consistente cu scorurile live
- **Backtest-ul** detecteaza corect toate exit markers (inclusiv cele din SignalStrategy cu prefix O/G/F)
- **SmartTrend** (viitor) va primi date corecte din `OracleHistory`

### 40.3 Serii noi disponibile

`OracleHistory` contine acum 5 serii aditionale:

| Serie | Sursa | Utilitate |
|-------|-------|-----------|
| `kalman_gain_series` | Kalman K[0] per-bara | Cat de mult filtrul se ajusteaza la date noi |
| `price_surprise_series` | innovation/predicted | Miscarile de pret neasteptate de Kalman |
| `run_length_series` | BOCPD MAP run length | Cate bare de la ultimul changepoint |
| `dominant_cycle_series` | Wavelet max-energy level | Ciclul dominant (2^n bare) |
| `noise_ratio_series` | Wavelet detail/total energy | Raportul zgomot/semnal |

---

## NOTA FINALA

Titan HFT este un sistem complex cu multiple straturi de siguranta. Recomandarile principale:

1. **Porniti intotdeauna in DRY RUN** pana cand sunteti confortabil cu comportamentul
2. **Monitorizati Activity Log** pentru anomalii
3. **Nu modificati manual threshold-urile** Circuit Breaker fara sa intelegeti implicatiile
4. **Lasati Genetic Optimizer-ul** sa ruleze cel putin cateva ore inainte de trading live
5. **Verificati LOOP HEARTBEATS** periodic — toate loop-urile trebuie sa fie ALIVE
6. **Session Reports** din `$HOME/HFT_Bot/reports/` sunt sursa de adevar pentru performanta
7. **Mesajele SPREAD GUARD** sunt normale — inseamna ca filtrele functioneaza si protejeaza capitalul
8. **Dupa modificari frontend**, stergeti WebKit cache si faceti clean build (vezi sectiunea 25.10)
9. **Setarile persista automat** in `titan_settings.json` — dry_run, strategy_mode, favorites, trade size
10. **Strategia genereaza semnalul, filtrele il valideaza** — vezi sectiunea 6 si 11 pentru detalii
11. **Telegram Alerts** — configurati BotFather si activati din tab-4 pentru alerte pe mobil
12. **Correlation Matrix** — ajustati Max Corr din tab-4 pentru diversificare optima in Multi-Symbol
13. **ATR Dynamic Stops** — lasati multiplicatorii default (2.0x SL, 3.0x TP) pana cand intelegeti comportamentul pe asseturile voastre
14. **State Persistence** — La crash/restart, totul se restaureaza automat din snapshot (trail stops, analytics, scan results)
15. **AUTO ENTRY** — Trebuie activat explicit din sidebar pentru ca botul sa deschida pozitii noi
16. **Signal Alignment** — Semnalele din chart si autopilot sunt acum aliniate si FUSION-aware (Cap. 36)
17. **Auto-Resume Scan** — La restart, scanul continua de unde a ramas, nu de la 0 (Cap. 37)
18. **DEL ALL** — Sterge doar simbolurile IDLE din HFT Engine; pozitiile active sunt protejate (Cap. 38)
19. **Entry Gate Scoring** — Pipeline-ul de scoring a fost deblocat: meta_confidence cu cbrt, noise floor, thresholds reduse (Cap. 39)
20. **Scalping Mode** — Activati scalping per strategie pentru semnale mai frecvente pe 1m-5m (Cap. 40)
21. **WARRIOR PRO** — Criteriile Ross Cameron filtreaza strict: RVOL>=5x, change>=10%, $1-$20, cu catalyst check (Cap. 43)
22. **Position Sizing Safety** — Max 25% equity per trade, 1000 share cap, duplicate order prevention (Cap. 44)

---

## 40. SCALPING MODE — TOGGLE-URI PER STRATEGIE

### 40.1 Ce Este

Fiecare din cele 3 strategii (Oracle, GPU Nexus, Fusion) poate fi comutata independent in mod scalping. Scalping mode genereaza semnale mai frecvente, optimizate pentru timeframe-uri de 1m si 5m cu holding times de secunde pana la 30 minute.

### 40.2 Toggle-uri in UI

In panoul GPU NEXUS din interfata:

| Toggle | Setting | Default |
|--------|---------|---------|
| **SCALP ORACLE** | `scalping_oracle` | ON |
| **SCALP GPU** | `scalping_gpu` | ON |
| **SCALP FUSION** | `scalping_fusion` | ON |

### 40.3 Diferente Scalping ON vs OFF

| Parametru | Scalping ON | Scalping OFF |
|-----------|------------|-------------|
| Lookback | 15 bare | 30 bare |
| Cooldown | 3 bare | 8 bare |
| Score threshold | 0.4 | 0.55 |
| Confidence gate | 0.08 | 0.15 |
| Exit SL multiplier | 0.70x | 1.0x |
| Exit TP multiplier | 0.60x | 1.0x |
| Exit Trail multiplier | 0.50x | 1.0x |

### 40.4 Impact pe Chart

Cand scalping e activ, chart-ul arata semnificativ mai multe markere BUY/SELL/EXIT. Cand e dezactivat, semnalele sunt mai rare si mai conservative.

---

## 41. REGIME-ADAPTIVE TRADING SIGNALS

### 41.1 Ce Este

Semnalele de trading se adapteaza automat la regimul pietei detectat de Oracle (Trending, MeanReverting, Chaotic, Unknown).

### 41.2 Praguri per Regim

| Regim | Score Threshold | Confidence Min | Comportament |
|-------|----------------|---------------|-------------|
| **Trending** | 0.35 (scalp) / 0.50 (swing) | 0.06 / 0.12 | Semnale mai sensibile, urmareste trendul |
| **MeanReverting** | 0.50 / 0.60 | 0.10 / 0.18 | Praguri moderate, confirmare mai stricta |
| **Chaotic** | 0.65 / 0.75 | 0.15 / 0.25 | Foarte selectiv, evita noise |
| **Unknown** | 0.45 / 0.55 | 0.08 / 0.15 | Default echilibrat |

### 41.3 Exit Signals Explicite

Pe chart apar markeri speciali de iesire:

| Marker | Culoare | Conditie |
|--------|---------|----------|
| **TRAIL** | Galben #FFD700 | Trailing stop atins |
| **EXIT** | Alb #FFFFFF | SL/TP sau semnal opus |
| **CHAOS** | Portocaliu #FF6600 | Regim chaotic cu pierdere |

### 41.4 Multi-Timeframe Confirmation

Semnalele sunt validate contra directiei Oracle pe higher timeframe. Un BUY pe 1m e mai puternic daca Oracle pe 5m e de asemenea bullish.

---

## 42. AUTOPILOT ALIGNMENT — CHECK_ENTRY + ORACLE EXITS

### 42.1 Problema Anterioara

Optimizarile de pe chart (regime-adaptive, scalping, VPIN gate) nu afectau autopilotul live. Chart-ul si autopilotul aveau logici diferite.

### 42.2 Solutia: check_entry()

Autopilotul foloseste acum aceeasi functie `check_entry()` din `strategy.rs` ca si chart-ul. Aceasta include:

- VPIN toxicity gate
- RVOL institutional volume check
- Regime-adaptive entry thresholds
- Scalping-specific parameters

### 42.3 Exit-uri Suplimentare

| Exit | Conditie | Descriere |
|------|----------|-----------|
| **Oracle Flip** | Oracle score < -10 (long) sau > 10 (short), confidence > 0.20 | Oracle s-a inversat puternic |
| **Chaos Exit** | Regim Chaotic + PnL negativ + confidence > 0.25 | Nu mentine pozitie in haos |
| **Scalping Exit** | Scalping activ | SL * 0.70, TP * 0.60, Trail * 0.50 |

---

## 43. WARRIOR PRO — ROSS CAMERON SCANNER

### 43.1 Ce Este

WARRIOR PRO este un mod de scanare bazat pe criteriile Ross Cameron (Warrior Trading), optimizat pentru day trading pe low-float stocks.

### 43.2 Criterii de Pre-filter (warrior_prefilter)

| Criteriu | Valoare | Explicatie |
|----------|---------|-----------|
| **Pret** | $1 - $20 | Low-price stocks cu potential de miscare mare |
| **Change%** | >= 10% | Doar actiuni in miscare semnificativa |
| **RVOL** | >= 5x | Volum relativ extraordinar fata de ziua anterioara |
| **Volum** | > 1M | Lichiditate minima |

### 43.3 Catalyst Check

Cand WARRIOR PRO e activ, fiecare candidat este verificat pentru stiri recente via Alpaca News API:

| Stare | Efect pe Score |
|-------|---------------|
| **Cu catalyst (stiri detectate)** | +25% bonus la ai_score si smart_score |
| **Fara catalyst** | -40% penalizare |

Bonus-uri suplimentare:
- RVOL >= 10x: +15%
- Change >= 20%: +10%
- Pret <= $10: +10%

### 43.4 Activare

Doua moduri de activare (sincronizate):
1. **Toggle `WARRIOR PRO`** in AI Scanner panel (tab-6) — buton rosu
2. **Toggle `Warrior`** in sidebar Risk & Params

Ambele controleaza acelasi `warrior_mode` in backend.

### 43.5 Badge Catalyst

In tabelul AI Scanner, simbolurile cu stiri recente detectate au iconita de ziar langa nume.

### 43.6 Warrior Scanner Sidebar

In Market Discovery (sidebar), scannerul WARRIOR are acum:
- Criteriile Ross Cameron (pret $1-$20, change >= 10%, RVOL >= 5x)
- Header de coloane: SYMBOL | PRICE | CHG% | RVOL | VOL
- Color coding: verde pentru CHG% >= 10% si RVOL >= 5x

---

## 44. POSITION SIZING SAFETY — HARDCAPS & DUPLICATE PREVENTION

### 44.1 Problema

Autopilotul plasa ordine repetate de 8000+ actiuni pe acelasi simbol (95% din equity per trade), fara a verifica ordinele pending sau orele de piata.

### 44.2 Solutii

| Fix | Detalii |
|-----|---------|
| **Max notional** | `equity * 0.25` (era `equity * 0.95`) |
| **Share cap** | Maximum 1000 actiuni per ordin |
| **Pending order check** | `has_pending_orders_for_symbol()` — query Alpaca `/v2/orders?status=open` |
| **Market hours check** | `is_market_open()` — query Alpaca `/v2/clock` |
| **Entry cooldown** | 60 secunde (era 30) |

### 44.3 Comportament

- Daca un simbol are deja ordine pending → nu se mai plaseaza
- Daca piata e inchisa si `enable_extended` e OFF → skip
- Daca qty calculat > 1000 → se limiteaza la 1000
- Daca notional > 25% equity → se reduce proportional

---

## 45. SCALPING POSITION MARKER

### 45.1 Ce Este

Pozitiile deschise in regim de scalping au un marker vizual distinct in tabelul de pozitii.

### 45.2 Cum Arata

- **Badge galben "SCALP"** — langa simbolul pozitiei
- **Regim badge** — ex: "Trending_LONG", "MeanReverting_SHORT" — langa timeframe

### 45.3 Implementare Tehnica

- `entry_tags: Mutex<HashMap<String, String>>` in `TitanState`
- La entry autopilot: `SCALP|Trending_LONG` sau `MeanReverting_SHORT`
- La close: tag-ul se sterge automat
- Frontend parseaza tag-ul si afiseaza badge-uri colorate

---

## 46. AI SCANNER ENHANCEMENTS

### 46.1 Scan Cancellation

Butonul **CANCEL** in progress bar-ul AI Scanner permite oprirea unui scan in curs. Backend: `CancellationToken` verificat la fiecare candidat.

### 46.2 Shared score_candidate()

Functia `score_candidate()` centralizeaza logica de scoring (Oracle + GPU Nexus + Backtest Sweep + AI Score + Smart Score + Catalyst Check). Folosita de `run_ai_scan`, `ai_scan_loop`, si `auto_resume_scan`.

### 46.3 HFT Targets Cleanup

La auto-inject, simbolurile cu `smart_score < 10.0` sunt eliminate automat din `hft_targets`. Cap maxim: 20 simboluri.

### 46.4 Timezone Fix

Market hours calculate corect cu DST: EDT (UTC-4) Martie-Octombrie, EST (UTC-5) Noiembrie-Februarie.

### 46.5 Rate Limiting

500ms delay intre batch-uri de snapshots (1000 simboluri/batch) pentru a respecta Alpaca rate limits.

---

## 47. SMART PICKS 3-Pillar Scoring

### 47.1 Formula cu 3 Piloni

Scorul `smart_score` determina clasamentul SMART PICKS si auto-inject-ul in HFT targets. Versiunea anterioara se baza aproape exclusiv pe metrici de backtest. Noua formula combina 3 piloni:

| Pilon | Pondere | Ce masoara |
|-------|---------|------------|
| **BACKTEST** | 50% | Performanta istorica: Win%, Sharpe, numar de trades, Profit Factor, Expectancy |
| **MOMENTUM** | 35% | Forta curenta: CHG% (schimbare pret azi), RVOL (volum relativ), MarketRegime (Oracle) |
| **QUALITY** | 15% | Calitate intrinseca: lichiditate (float), prezenta catalizator, aliniere directionala Oracle-pret |

**BACKTEST PILLAR (50%):**
- Baza: `0.30 * Win% + 0.25 * Sharpe_norm + 0.20 * Trades_norm + 0.15 * PF_norm + 0.10 * Expect_norm`
- Multiplier-uri penalizatoare: Win% < 30% = 0.25x, Trades < 5 = 0.35x, Sharpe < 0.3 = 0.45x, PF < 0.8 = 0.55x, Expectancy < 0 = 0.40x
- Consistency bonus: Win% >= 50% + Sharpe >= 1.0 + Trades >= 10 = 1.15x
- Daca datele sunt insuficiente (trades < 3, sharpe <= 0, win_rate < 15%, expectancy < -0.01), pilonul primeste doar 10% din Fusion score

**MOMENTUM PILLAR (35%):**
- CHG% score (40%): >10% = 100, >5% = 80, >2% = 60, >0% = 40, >-2% = 20, >-5% = 10, altfel = 0
- RVOL score (30%): >5x = 100, >3x = 90, >2x = 75, >1x = 50, altfel = 20
- Regime score (30%): Trending = 100, MeanReverting = 60, Unknown = 40, Chaotic = 20

**QUALITY PILLAR (15%):**
- Lichiditate: HIGH = 1.0x, MED = 0.92x, LOW + price < $3 = 0.50x, LOW altfel = 0.75x
- Catalyst: prezenta catalizator = 1.20x bonus (se aplica in TOATE modurile, nu doar Warrior)
- Aliniere directionala: Oracle si pret pe aceeasi directie = 1.10x, directii opuse = 0.75x

### 47.2 Filtre de Calitate

Inainte de afisare in SMART PICKS, candidatii trebuie sa treaca filtrele:

| Filtru | Valoare | Scop |
|--------|---------|------|
| `bt_trades` | >= 5 | Suficiente date statistice |
| `bt_sharpe` | > 0.0 | Randament ajustat la risc pozitiv |
| `bt_win_rate` | > 20% | Minimum viabilitate |
| `smart_score` | > 25 | Calitate globala (era > 20) |
| `change_percent` | > -5% | Exclude actiuni in cadere libera |
| `price` | >= $1.50 | Exclude extreme penny stocks |

### 47.3 Coloane Noi in Tabela SMART PICKS

Tabela SMART PICKS afiseaza acum 3 coloane noi cu codare cromatica:

- **CHG%** — Schimbarea procentuala a pretului fata de deschidere. Verde intens (>5%), verde deschis (>0%), gri (0 pana la -2%), rosu (<-2%).
- **RVOL** — Volumul relativ fata de media zilnica. Verde (>= 5x), galben (>= 2x), gri (< 2x). Format: `2.3x`.
- **REGIME** — Regimul de piata detectat de Oracle. Badge colorat: TREND (verde), MR (albastru, Mean Reverting), CHAOS (rosu), UNK (gri, Unknown).
- **Catalyst Badge** — Daca simbolul are un catalizator detectat (stire), apare un icon de stire langa simbol.

### 47.4 Catalyst Detection in Toate Modurile

Anterior, verificarea de catalizatori (`has_catalyst`) se aplica doar cand modul Warrior Pro era activ. Acum:

- Catalyst check se executa in TOATE modurile de scanare (Standard)
- Bonusul de 1.20x in Quality Pillar se aplica universal
- Logare: `[CATALYST] {symbol} has active catalyst (score={score})` apare indiferent de modul activ
- Impact: simboluri cu stiri de interes (FDA approval, earnings beat, contract win) sunt promovate in clasament

---

## 48. MARKET DISCOVERY — Parametri Avansati si Auto-Scan

### 48.1 ScanParams — Parametri Configurabili

Scannerul de piata foloseste acum structura `ScanParams` din `scanner.rs`, cu toate campurile configurabile din UI:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `price_min` | $1.00 | Pret minim per actiune |
| `price_max` | $20.00 | Pret maxim per actiune |
| `volume_min` | 500,000 | Volum minim zilnic |
| `change_min` | 5.0% | Schimbare procentuala minima |
| `rvol_min` | 2.0x | Volum relativ minim (fata de media) |
| `gap_min` | 3.0% | Gap minim fata de inchidere anterioara |
| `require_catalyst` | false | Filtru catalizator obligatoriu (stiri recente) |
| `max_candidates` | 100 | Numar maxim de candidati procesati |
| `scan_interval_minutes` | 30 | Interval auto-scan (5-60 minute) |

### 48.2 Configurare din UI

In sectiunea **Market Discovery** din sidebar:
- **RVOL Min** — slider sau input numeric pentru volumul relativ minim
- **Gap Min** — slider sau input numeric pentru gap-ul minim
- **Require Catalyst** — toggle ON/OFF, cand activ filtreaza doar simboluri cu stiri recente
- **Max Candidates** — slider 10-300, controleaza cate simboluri sunt procesate in fiecare scan
- **Auto-Scan Interval** — slider 5-60 minute (inlocuieste intervalul hardcoded de 30 min din `ai_scan_loop`)

Toti parametrii se salveaza automat in `titan_settings.json` prin `scan_params` field din `AppSettings`.

### 48.3 Functii Refactorizate

- **`quant_prefilter()`** — accepta `&ScanParams` in loc de valori hardcoded. Aplica filtrele de pret, volum, schimbare, RVOL, gap, si catalyst.
- **`momentum_prefilter()`** — (fost `warrior_prefilter`) filtru strict Ross Cameron: pret $1-$20, RVOL >= 5x, change >= 10%, volum >= 1M. Disponibil ca subfiltru avansat.

### 48.4 Watchlist Management

- **DELETE ALL** — sterge toate simbolurile din watchlist, cu EXCEPTIA celor care au pozitii live deschise. Util pentru curatare rapida dupa o sesiune de scan.
- **INJECT ALL -> WATCHLIST** — importa toate rezultatele din ultimul scan in watchlist, fara a sterge existentele.
- **AUTO INJECT** — toggle care activeaza injectarea automata a rezultatelor fiecarui scan in watchlist.

### 48.5 Interactiune Watchlist — Chart

Cand dai click pe un simbol din watchlist, acesta este afisat pe chart (graficul principal se schimba la simbolul selectat).

---

## 49. MANUAL TRADE — Tranzactionare Manuala USD

### 49.1 Panelul Manual Trade

Sectiunea **Manual Trade** din sidebar (element colapsabil `<details>`) permite tranzactionare manuala directa:

| Element | Functie |
|---------|---------|
| **Symbol Input** | Campul de text pentru simbolul asset-ului (ex: AAPL, TSLA) |
| **LONG / SHORT** | Selector directie: LONG (cumparare) sau SHORT (vanzare in lipsa) |
| **USD Amount** | Suma in USD pe care doresti sa o tranzactionezi |
| **EXECUTE** | Plaseaza ordinul |
| **FROM WL** | Injecteaza simbolul selectat din Watchlist in campul Symbol |

### 49.2 Backend — `submit_manual_order`

Comanda Tauri `submit_manual_order` proceseaza ordinul:

1. **Primeste:** `ManualOrder { symbol, side, usd_amount }`
2. **Calculeaza qty:** Preia pretul curent din cache (sau fetch live de la Alpaca), imparte `usd_amount / price`, rotunjeste la intreg
3. **Plaseaza ordin:** Apeleaza `send_order_internal()` cu tipul `market`
4. **Returneaza:** Confirmare sau eroare

### 49.3 Observatii

- Ordinele manuale sunt de tip **MARKET** (executie la pretul curent)
- Nu exista protectie automata SL/TP pe ordinele manuale — utilizatorul trebuie sa gestioneze pozitia
- Asset-urile tranzactionate manual apar in lista de pozitii live cu posibilitate de a seta modul AUTO sau MANUAL (vezi Cap. 51)

---

## 50. FORCE MANUAL OVERRIDE — Control Total

### 50.1 Ce Face

Toggle-ul **FORCE MANUAL RES** din sidebar activeaza controlul total al utilizatorului asupra TUTUROR pozitiilor. Cand este activ:

- **`autopilot_loop` este blocat complet** — la fiecare iteratie, daca `force_manual_tf == true`, loop-ul face `continue` imediat, fara a evalua sau gestiona vreo pozitie
- **NICIO intrare automata** — botul nu deschide pozitii noi
- **NICIO iesire automata** — botul nu inchide, nu ajusteaza SL/TP, nu face trailing stop
- **Utilizatorul preia controlul complet** — toate deciziile de trading sunt manuale

### 50.2 Indicator Vizual

Cand `force_manual_tf` este activ, un banner rosu pulsant apare in zona principala a UI:

**"MANUAL OVERRIDE ACTIVE"** — cu animatie `pulse-border` (fundal rosu semi-transparent cu border rosu pulsant)

Banner-ul este sincronizat cu starea toggle-ului prin `refreshSidebarState()`.

### 50.3 Diferenta fata de Per-Asset Manual Mode

| Caracteristica | Force Manual Override | Per-Asset Manual (Cap. 51) |
|----------------|----------------------|----------------------------|
| **Scop** | Blocheaza TOT botul | Blocheaza botul pe UN singur simbol |
| **Pozitii afectate** | TOATE | Doar simbolul setat pe "manual" |
| **Intrari noi** | Blocate complet | Permise pe alte simboluri |
| **Cand se foloseste** | Situatii de urgenta, control total temporar | Gestiune individuala a unor pozitii specifice |

### 50.4 Interactiune cu Aggressive Mode

`force_manual_tf` suprascrie Aggressive Mode — chiar daca Aggressive Mode este activ, daca Force Manual Override este ON, botul nu face nimic. Verificarea `force_manual_tf` vine INAINTE de parametrii agresivi in `autopilot_loop`.

---

## 51. LIVE POSITIONS — Mod Auto/Manual per Asset

### 51.1 Coloana MODE in Tabelul Pozitiilor

Tabelul de pozitii live contine acum o coloana **MODE** cu un buton toggle per simbol:

| Mode | Icon | Culoare | Efect |
|------|------|---------|-------|
| **AUTO** | Robot | Verde | Botul gestioneaza pozitia normal (SL, TP, trailing) |
| **MANUAL** | Mana | Portocaliu | Botul NU interactioneaza cu pozitia sub nicio forma |

### 51.2 Comenzi Tauri

- **`set_position_mode(symbol, mode)`** — seteaza modul pentru un simbol: `"auto"` sau `"manual"`
- **`get_position_modes()`** — returneaza `HashMap<String, String>` cu toate modurile active

### 51.3 Stocare si Persistenta

Modurile sunt stocate in `position_trade_modes: HashMap<String, String>` din `AppSettings`/`AppState`. Se salveaza automat in `titan_settings.json` prin state persistence.

### 51.4 Comportament Bot

In `autopilot_loop`, pentru fiecare simbol procesat:
1. Se verifica daca simbolul are mode setat in `position_trade_modes`
2. Daca `mode == "manual"` → pozitia este complet sarita (skip)
3. Daca `mode == "auto"` sau nu are mode setat → pozitia este gestionata normal

### 51.5 Cazuri de Utilizare

- **Pozitie pe termen lung:** Setezi MANUAL pe un simbol pe care vrei sa-l tii fara interferenta botului
- **Asset volatil:** Setezi MANUAL cand piata e imprevizibila si vrei control direct
- **Tranzactie manuala:** Dupa `submit_manual_order`, setezi MANUAL pentru a preveni auto-exit

---

## 52. DRAWING TOOLS V2 — Toate Instrumentele

### 52.1 Prezentare Generala

Drawing Engine V2 (`drawing_engine_v2.js`) inlocuieste complet versiunea anterioara cu 11 instrumente profesionale de desen pe chart. Fiecare instrument se selecteaza din toolbar-ul de sus.

### 52.2 Cele 11 Instrumente

| # | Instrument | Descriere |
|---|-----------|-----------|
| 1 | **Crosshair** | Dezactiveaza modul desen, restaureaza interactiunea normala cu chart-ul (zoom, pan) |
| 2 | **Trend** | Linie de trend intre 2 puncte (click-drag) |
| 3 | **Ray** | Raza infinita — de la punctul 1 prin punctul 2, continuand pana la marginea chart-ului |
| 4 | **HLine** | Linie orizontala de pret — se extinde pe toata latimea chart-ului la nivelul de pret selectat |
| 5 | **VLine** | Linie verticala de timp — de sus pana jos la momentul selectat |
| 6 | **Fib** | Retrageri Fibonacci — niveluri automate: 0%, 23.6%, 38.2%, 50%, 61.8%, 78.6%, 100% |
| 7 | **Text** | Adnotare text — click pe chart, introducere text in prompt, plasare la coordonatele alese |
| 8 | **Measure** | Masurare distanta — afiseaza diferenta de pret, procent, si timp intre 2 puncte |
| 9 | **Long** | Pozitie long box — zone vizuale cu entry, target, si stop |
| 10 | **Short** | Pozitie short box — zone vizuale cu entry, target, si stop |
| 11 | **Delete** | Sterge TOATE desenele de pe chart |

### 52.3 Toolbar Executie

Sub chart exista un toolbar de executie (`exec-toolbar`):

- **BET $** — camp input pentru suma in USD a tranzactiei
- **EXEC** — buton de executie (plaseaza ordin bazat pe instrumentul Long/Short desenat)
- **AI** — buton AI assist
- **DEL** — sterge ultimul desen

### 52.4 Persistenta

La schimbarea simbolului pe chart:
1. Desenele curente sunt serializate cu `serialize()` si salvate in `localStorage` cu cheia `drawings_{symbol}`
2. Desenele noului simbol sunt restaurate cu `setDrawings()` din `localStorage`

### 52.5 Rezolvare Simbol

Drawing Engine V2 foloseste `window.currentSymbol` pentru a identifica simbolul curent, in loc de "BTC/USD" hardcoded (fix din versiunea anterioara).

### 52.6 Interactiune cu Chart-ul

- Cand un instrument de desen este activ (nu crosshair), pointer events sunt rutate catre drawing engine
- Cand crosshair este selectat, chart-ul revine la comportamentul normal (zoom, pan, hover info)
- Cursorul se schimba automat: crosshair pentru desen, default pentru interactiune chart

---

## 53. AGGRESSIVE MODE — Audit Complet

### 53.1 Toate Caile de Cod

Aggressive Mode modifica comportamentul botului in 4 locatii:

| Fisier | Locatie | Normal | Agresiv |
|--------|---------|--------|---------|
| `trading_engine.rs` | L39-54 | SL: -2.5%, trailing strins | SL: -8%, trailing mai larg |
| `strategy.rs` | L173-198 | Praguri de intrare standard | Praguri mai joase (intrari mai permisive) |
| `main.rs` (autopilot) | Stop-Loss | 1.0x | 0.52x (stop mult mai departe) |
| `main.rs` (autopilot) | Take-Profit | 1.0x | 1.68x (target mai ambitios) |
| `main.rs` (autopilot) | Trailing | 1.0x | 0.67x (trailing mai strans) |
| `main.rs` (entry) | `stop_dist` | 0.8x vol_factor | 1.3x vol_factor (stop mai departe la intrare) |

### 53.2 Indicator Vizual

Cand Aggressive Mode este activ:
- Sectiunea sidebar asociata primeste efect de pulsare rosie
- Text "AGGRESSIVE ACTIVE" este afisat vizibil

### 53.3 Override Sliders

Sub toggle-ul Aggressive Mode exista 3 slidere de ajustare:
- **SL %** — procentul stop-loss override
- **TP %** — procentul take-profit override
- **TRAIL %** — procentul trailing stop override

### 53.4 Interactiune cu Force Manual

**Force Manual Override suprascrie Aggressive Mode.** Daca ambele sunt active, `force_manual_tf` blocheaza complet autopilot-ul — parametrii agresivi devin irelevanti.

Ordinea de verificare in `autopilot_loop`:
1. `force_manual_tf` → daca true, `continue` (skip tot)
2. Per-asset manual mode → daca manual, skip acel simbol
3. Aggressive parametrii → aplica multiplicatori modificati

---

## 54. EXTENDED HOURS — Audit Complet

### 54.1 Ce Face

Cand toggle-ul **Extended Hours** este activ, ordinele trimise la Alpaca includ flag-ul `extended_hours: true`. Acest flag permite executia ordinelor in afara orelor normale de piata:

- **Pre-market:** 04:00 - 09:30 ET
- **After-hours:** 16:00 - 20:00 ET

### 54.2 Implementare Backend

In functia `send_order_internal()` din `main.rs`:
1. Se verifica daca `enable_extended` este ON in settings
2. Se verifica daca asset-ul NU este crypto (crypto tradeaza 24/7, nu necesita flag)
3. Daca ambele conditii sunt indeplinite: `"extended_hours": true` se adauga in body-ul JSON al ordinului

### 54.3 Badge Vizual

Cand Extended Hours este activ, un badge violet **"EXTENDED"** apare in header-ul aplicatiei, langa informatiile de market status. Badge-ul este sincronizat cu starea toggle-ului prin `refreshSidebarState()`.

### 54.4 Observatii

- Extended Hours gateaza doar **intrarea** (entry) — ordinele de exit (close) nu sunt afectate
- Alpaca Paper Trading suporta extended hours pentru testare
- Lichiditatea in extended hours este semnificativ mai mica — spreadurile sunt mai mari, preturile mai volatile
- Nu toate actiunile au volum suficient in extended hours — ordinele market pot primi fill-uri slabe

---

## 55. ELIMINARI DIN SESIUNEA 7

### 55.1 Warrior 3rd Bot — ELIMINAT

**Ce era:** Un al treilea mod de strategie care se baza pe criteriile Ross Cameron (day trading de actiuni low-price cu momentum). Includea un toggle `WARRIOR` in sidebar, cash block separat, strategie de scoring dedicata, functie `scan_warrior()`, si `warrior_prefilter()`.

**Ce s-a eliminat:**
- **Frontend:** checkbox `chk-warrior`, bloc cash `warrior-cash-block`, toggle `WARRIOR PRO` din AI Scanner tab, sectiune footer warrior, wiring wallet warrior, optiunea `WARRIOR` din strategy dropdown Market Discovery
- **Backend:** `warrior_mode` din `AppSettings`, `AppState`, `update_settings`, `settings_from_state`, `apply_settings`; `warrior_cash` din `Wallet` si `WalletSnapshot`; branching warrior din `run_ai_scan`, `ai_scan_loop`, `score_candidate`; functia `scan_warrior()` stearsa complet
- **JS:** toggle wiring din `sidebar_wiring.js` si `main.js`

**Ce s-a pastrat:** Functia `warrior_prefilter()` a fost redenumita in **`momentum_prefilter()`** si este disponibila ca subfiltru avansat in scanner. Criteriile Ross Cameron (pret $1-$20, RVOL >= 5x, change >= 10%, volum >= 1M) sunt pastrate in aceasta functie.

**De ce:** Functionalitatea Warrior era redundanta — criteriile Ross Cameron sunt acum integrate in `ScanParams` configurabil, iar `momentum_prefilter()` ramane disponibil ca filtru strict optional.

### 55.2 Kelly Criterion — ELIMINAT

**Ce era:** Un toggle `KELLY` care activa Kelly Criterion position sizing — o formula matematica pentru dimensionarea pozitiilor bazata pe edge-ul istoric al strategiei.

**Ce s-a eliminat:**
- **Frontend:** checkbox `chk-kelly`, binding `kelly_active`
- **Backend:** camp `kelly_active` din `AppSettings`, `AppState`, `update_settings`, `settings_from_state`, `apply_settings`

**De ce:** Kelly Criterion position sizing nu era integrat complet in pipeline-ul de executie al botului. Hardcap-urile de pozitie sizing (Cap. 44) ofera protectie mai robusta.

### 55.3 Lista Lunetist — ELIMINATA

**Ce era:** O sectiune dedicata in sidebar (`LISTA LUNETIST`) cu un tabel de simboluri monitorizate, budget controls proprii, si statistici de performanta. Avea CSS classes dedicate si logica de gestionare separata.

**Ce s-a eliminat:**
- **Frontend:** intreaga sectiune `<details>` a LISTA LUNETIST, inclusiv budget row, budget val, budget ctrl, tabel statistici
- **CSS:** clasele `.lunetist-budget-row`, `.lunetist-budget-val`, `.lunetist-budget-ctrl`, `.lunetist-stats`
- **Buton redenumit:** "DEEP FOCUS (LUNETIST)" → "DEEP FOCUS"

**De ce:** Functionalitatea Lista Lunetist era partiala si redundanta cu Watchlist + per-asset modes. Watchlist-ul imbunatatit (Cap. 48) si modurile per-asset (Cap. 51) acopera toate cazurile de utilizare.

### 55.4 Rezumat Fisiere Afectate de Eliminari

| Fisier | Ce s-a schimbat |
|--------|-----------------|
| `index.html` | Sterse: warrior checkbox, cash block, PRO toggle, footer, wallet wiring, lunetist section, kelly checkbox |
| `main.rs` | Sterse: warrior_mode, kelly_active din structs/settings/commands, warrior branching in scan/score |
| `app_state.rs` | Sterse: warrior_mode, kelly_active din AppState si default |
| `scanner.rs` | Sterse: scan_warrior(); redenumit: warrior_prefilter → momentum_prefilter |
| `sidebar_wiring.js` | Sters: chk-warrior wiring |
| `main.js` | Sters: chk-warrior wiring |
| `styles.css` | Sterse: .lunetist-stats CSS block |

---

## Cap. 56 — Central Panel: Watchlist (tab-9) si HFT Targets (tab-10)

### 56.1 De ce s-au mutat din Sidebar

Sidebar-ul din stanga era prea ingust (~250px) pentru a afisa tabelele cu toate coloanele din AI Scanner (SYMBOL, PRICE, CHG%, VOL, RVOL, GAP%, SCORE, BT W/R, BT PnL, DD, CAT). Coloanele fie se trunchieau, fie nu incapeau deloc.

**Solutia:** Migrarea completa a Watchlist si HFT Engine din sidebar in panoul central, unde dispun de ~800-1200px latime. Doua tab-uri noi au fost create langa "REPLAY":

- **tab-9 — WATCHLIST** (buton in bara de tab-uri a panoului central)
- **tab-10 — HFT TARGETS** (buton in bara de tab-uri)

### 56.2 Tab-9: WATCHLIST

Afiseaza toate asset-urile din lista de favorite (watchlist) cu coloane complete:

| Coloana | Descriere |
|---------|-----------|
| SYMBOL | Ticker-ul asset-ului |
| PRICE | Pret curent (din snapshot Alpaca) |
| CHG% | Procentaj schimbare zilnica, color-coded (verde/rosu) |
| VOL | Volumul zilnic |
| RVOL | Relative Volume (fata de media 20 zile) |
| GAP | Gap% fata de inchiderea anterioara |
| SCORE | Scor calculat (change% × 2 + RVOL × 15 + log(vol) × 3), color-coded |
| CAT | Catalyst badge — vezi Cap. 59 |
| TRADE | Buton "→MT" — injecteaza in Manual Trade |
| HFT | Buton "→HFT" — injecteaza in HFT Engine |
| AI | Buton AI backtest per asset — vezi Cap. 57 |

**Butoane din header tab-9:**
- **INJECT ALL → HFT** — adauga toate simbolurile din watchlist in HFT Engine
- **BACKTEST ALL** — ruleaza AI scoring pe toate simbolurile (vezi Cap. 57)
- **CHECK NEWS** — verifica stiri/catalyst on-demand pentru toate simbolurile

**Backend:**
- `get_watchlist_enriched` — Tauri command care returneaza `WatchlistEntry` cu snapshot (price, change%, vol, RVOL, gap) si `score` calculat

### 56.3 Tab-10: HFT TARGETS

Afiseaza asset-urile din HFT Engine cu **toate** coloanele din AI Scanner:

| Coloana | Descriere |
|---------|-----------|
| SYMBOL | Ticker-ul asset-ului |
| PRICE | Pret curent |
| CHG% | Procentaj schimbare zilnica |
| VOL | Volum zilnic |
| RVOL | Relative Volume |
| GAP | Gap% |
| SCORE | Scor AI compozit |
| BT W/R | Backtest Win Rate % |
| BT PnL | Backtest Profit & Loss |
| DD | Max Drawdown — vezi Cap. 58 |
| CAT | Catalyst badge |

**Backend:**
- `get_hft_target_enriched` — Tauri command care ruleaza AI scoring **live** daca simbolul nu exista in cache `last_scan_results`. Foloseste `score_candidate()` cu `PreFilterCandidate` construit din snapshot Alpaca, returnand `AiScanCandidate` complet.

**Auto-refresh:** Tabelul se actualizeaza automat la fiecare eveniment `ai-scan-update` de la backend.

### 56.4 Inject All / Selective din Watchlist

- **Inject in Manual Trade (→MT):** Click pe butonul per-row adauga simbolul in lista manuala si seteaza modul "manual" via `set_position_mode`
- **Inject in HFT (→HFT):** Click pe butonul per-row apeleaza `add_hft_target`
- **INJECT ALL → HFT:** Buton in header tab-9 — injecteaza toate simbolurile din watchlist in HFT Engine simultan
- **INJECT ALL → MT:** Buton in header tab-9 — injecteaza toate simbolurile in Manual Trade

---

## Cap. 57 — AI Backtest pentru Watchlist

### 57.1 Scop

Permite utilizatorului sa ruleze pipeline-ul complet de AI scoring (identic cu AI Scanner) pe asset-urile din Watchlist, fara a astepta un scan automat. Rezultatele apar intr-un tabel secundar cu **toate** coloanele din AI Scanner.

### 57.2 Cum se utilizeaza

1. Navigheaza la tab-9 (WATCHLIST)
2. Click **"BACKTEST ALL"** in header — ruleaza scoring pe toate simbolurile
   - Sau click butonul **"AI"** pe un singur rand pentru backtest individual
3. Se afiseaza un progress bar: "SCORING 3/12..." pe durata procesarii
4. Rezultatele apar in tabelul `#wl-ai-results` sub tabelul principal

### 57.3 Coloanele tabelului AI Results

Identice cu AI Scanner: SYMBOL, PRICE, CHG%, VOL, RVOL, GAP%, SCORE, BT W/R, BT PnL, BT Trades, BT Best, BT Worst, DD, CAT.

### 57.4 Backend

Scoring-ul foloseste exact aceeasi functie `score_candidate()` din `main.rs`, care apeleaza:
- `fetch_snapshots_batch` pentru date pret/volum
- `run_backtest_sync` pentru backtest pe date istorice
- `get_sentiment_score` + `fetch_news_feed` pentru scor sentiment si catalyst
- Rezultatul este un `AiScanCandidate` complet identic cu cel din AI Scanner

---

## Cap. 58 — Coloana DD (Max Drawdown)

### 58.1 Ce este Max Drawdown

**Max Drawdown (DD)** reprezinta cea mai mare scadere procentuala de la un varf local la un minim local pe durata unui backtest. Exemplu: daca equity-ul creste la $1000 si apoi scade la $800, DD = -20%.

### 58.2 De ce este important

- Un DD mare (ex: -40%) indica o strategie cu risc ridicat si volatilitate mare a equity-ului
- Un DD mic (ex: -5%) indica o strategie mai stabila si previzibila
- DD-ul ajuta la evaluarea "confortului" unei strategii — chiar daca o strategie este profitabila, un DD prea mare poate determina abandonarea prematura

### 58.3 Unde este afisat

| Tabel | Coloana |
|-------|---------|
| AI Scanner (tab-6) | DD |
| Smart Picks (sidebar) | DD |
| WL AI Results (tab-9) | DD |
| HFT Targets (tab-10) | DD |

### 58.4 Color Coding

| Valoare | Culoare | Semnificatie |
|---------|---------|--------------|
| < -20% | **Rosu** (#f44) | Risc ridicat |
| -20% ... -10% | **Galben** (#fa0) | Risc moderat |
| ≥ -10% | **Gri** (#999) | Risc scazut |

### 58.5 Camp Backend

`bt_max_drawdown: f64` in `AiScanCandidate` — calculat de functia `run_backtest_sync` pe baza seriei de equity.

---

## Cap. 59 — Catalyst Badges si Link-uri Sursa

### 59.1 Catalyst Tab (tab-7) — Link-uri Clickabile

In tabul CATALYST, fiecare headline de stire este acum clickabil. La click se deschide pagina web sursa in browser-ul default al sistemului via `window.__TAURI__.shell.open(url)`.

Deasemenea, un buton **"READ ↗"** apare pe fiecare rand pentru acces rapid la sursa.

### 59.2 Catalyst Toast — Buton READ

Cand apare o notificare toast de catalyst, aceasta include un buton **"READ"** care deschide sursa stirii.

### 59.3 News AI Tab (tab-1)

Headline-urile articolelor au acum un link **"READ ↗"** clickabil langa data publicarii.

### 59.4 Catalyst Badges in Tabele

In tabelele din tab-9 (Watchlist) si tab-10 (HFT Targets), coloana **CAT** afiseaza:

| Badge | Semnificatie |
|-------|-------------|
| `★` (stea galbena) | Catalyst recent (< 1h) |
| `●` (cerc portocaliu) | Catalyst din ultimele 24h |
| `—` (gri) | Fara catalyst detectat |

**Click pe badge:** Deschide sursa stirii in browser.

### 59.5 On-Demand News Check

- Buton **🔍** per rand in tabel — apeleaza `check_catalyst` Tauri command
- **CHECK NEWS** buton in header — verifica stiri pentru toate simbolurile din tab
- Rezultatul (`CatalystCheckResult`) contine: symbol, has_catalyst, headline, url, sentiment_score

### 59.6 Global Frontend Catalyst Cache

`window._catalystCache` — obiect JavaScript populat din:
- Evenimente `catalyst-alert` de la backend (real-time)
- Date `ai-scan-update` (campurile `has_catalyst`, `catalyst_headline`, `catalyst_url`)
- Rezultate `check_catalyst` on-demand

Cache-ul permite afisarea instantanee a badge-urilor fara request-uri suplimentare.

### 59.7 Backend Enhancements

- `AiScanCandidate` — campuri noi: `catalyst_headline: String`, `catalyst_url: String`
- `CatalystCheckResult` — struct NOU: `symbol`, `has_catalyst`, `headline`, `url`, `sentiment_score`
- `check_catalyst` — Tauri command NOU care apeleaza `fetch_news_feed` + `get_sentiment_score`

---

## Cap. 60 — Manual Trade Panel: Lista Assets, Controls, Inject

### 60.1 Lista de Assets Manuale

Functia `renderManualAssetList()` populeaza dinamic sectiunea din Manual Trade panel cu toate asset-urile setate pe mod "manual". Fiecare rand afiseaza:

| Element | Descriere |
|---------|-----------|
| **Simbol** | Ticker-ul asset-ului (bold) |
| **Status** | "OPEN +$12.50" sau "NO POS" daca nu exista pozitie deschisa |
| **CLOSE** | Buton care trimite ordin de inchidere (market sell/cover) |
| **AUTO** | Buton care comuta asset-ul inapoi pe mod autotrading |
| **✕** | Buton remove — scoate asset-ul din lista manuala |

### 60.2 Auto-Mark Manual

La executia unui ordin din panelul Manual Trade (`submit_manual_order` sau `submit_order`), simbolul este **automat** setat pe mod "manual" prin apelul `set_position_mode(symbol, "manual")`. Aceasta previne botul sa interfereze cu pozitia deschisa manual.

### 60.3 Inject din Watchlist

- **"FROM WL" buton** — deschide un popup cu toate simbolurile din watchlist; click pe un simbol il adauga in lista manuala
- **"+ ADD" buton** — deschide un dialog pentru adaugarea manuala a unui simbol (input text)
- **"ALL → AUTO" buton** — comuta toate asset-urile din lista manuala inapoi pe mod autotrading

### 60.4 Auto-Refresh

`renderManualAssetList()` este apelat automat din:
- `updateAccount()` — la fiecare refresh al pozitiilor live
- `loadPositionModes()` — la incarcarea modurilor din backend

---

## Cap. 61 — Drawing Tools: Rendering Fixes si projectTime

### 61.1 Problema: Dreptunghi Lipit de Marginea Dreapta

Cand un capat al unui desen (ex: risk box) avea un timestamp in afara viewport-ului vizibil al chart-ului, `timeToCoordinate()` returna `null`. Codul original folosea fallback-uri hardcoded (`x = 9999` sau `x = -9999`), ceea ce facea dreptunghiurile sa se extinda pana la marginile ecranului.

### 61.2 Solutia: `projectTime(time)`

O noua metoda `projectTime(time)` a fost adaugata in clasa `DrawingManager`. Aceasta realizeaza **extrapolare liniara** a coordonatei x pentru timestamp-uri off-screen, folosind range-ul vizibil al chart-ului (cel putin 2 puncte de referinta):

1. Gaseste barem minimul si maximul timestamp vizibil cu coordonatele lor x corespunzatoare
2. Calculeaza `pixelsPerMs = (xMax - xMin) / (tMax - tMin)`
3. Extrapoleaza: `x = xMin + (time - tMin) * pixelsPerMs`

Daca chart-ul nu are date suficiente, returneaza `null` in loc de o valoare eronata.

### 61.3 Rendering Audit — 5 Fixuri

1. **Null guards risk box:** Verificare explicita `if (y1 == null || y2 == null || yStop == null) return;` la inceputul blocului de rendering risk box — previne dimensiuni `NaN`
2. **Skip-draw on null projection:** `if (x1 == null || x2 == null) return;` in `render()` si `continue;` in `hitTest()` — daca `projectTime()` nu poate calcula, desenul este omis in loc sa fie randuit la x=0
3. **Label overlap prevention:** Labels-urile TARGET si STOP sunt desenate doar cand box-ul are minim **16px** inaltime — previne suprapunerea textului
4. **Canvas state isolation:** `ctx.save()` / `ctx.restore()` in jurul fiecarui desen din bucla `render()` — previne "leaking" al `lineWidth`, `textAlign`, `shadowBlur`, `lineDash` de la un desen la altul
5. **Division by zero guard:** Verificare `d.p1.price !== 0` in calculul `pPct` (procentaj TARGET) — previne valori `Infinity`

---

## Cap. 62 — Long/Short Settings Tab (tab-11)

### 62.1 Prezentare Generala

Tab-ul **LONG/SHORT** (tab-11) este un panou centralizat pentru configurarea tuturor parametrilor legati de desenele de tip risk box (Long/Short) de pe chart. Este organizat in 6 sectiuni (A-F).

### 62.2 A. Default Risk Parameters

Seteaza parametrii de risc impliciti aplicati la **crearea** unui nou desen risk box pe chart.

| Parametru | Descriere |
|-----------|-----------|
| **R:R Ratio** (radio) | Mod R:R — slider de la 0.5 la 5.0, default 2.0. Stop-ul este calculat automat bazat pe raportul Risk:Reward selectat |
| **Fixed %** (radio) | Mod procentaj fix — campuri input SL% si TP% (ex: SL = 2%, TP = 4%) |

**Comportament:** La crearea unui nou desen, `onMouseUp()` citeste `_lsSettings.riskMode`:
- Daca "rr": `stopPrice = entry - (target - entry) / rr`
- Daca "fixed": `stopPrice = entry × (1 - SL%)`, `targetPrice = entry × (1 + TP%)`

### 62.3 B. Position Sizing

| Parametru | Descriere |
|-----------|-----------|
| **Fixed $ Amount** (radio) | Suma fixa in USD (ex: $500), introdusa in input |
| **% of Equity** (radio) | Procent din equity-ul contului, slider 0.5%-25% |
| **Calculated fields** | Auto-calculate: Shares/Qty, Risk $, Max Loss $, Max Profit $ |

**Campurile calculate** se actualizeaza automat cand un risk box este selectat pe chart, bazat pe entry price, stop price si bet amount.

### 62.4 C. Label Display Options

5 checkbox-uri care controleaza ce informatii sunt afisate pe risk box-urile de pe chart:

| Checkbox | Efect pe chart |
|----------|----------------|
| **Show % change** | Afiseaza procentajul de la entry la target/stop (ex: "+2.5%", "-1.2%") |
| **Show $ P/L** | Afiseaza profit/pierdere in USD (ex: "+$125.00", "-$60.00") |
| **Show R:R ratio** | Afiseaza raportul Risk:Reward (ex: "R:R 2.1") |
| **Show entry price** | Afiseaza "ENTRY: 150.25" pe linia de referinta (dashed) |
| **Show target/stop prices** | Afiseaza preturile sub labels (ex: "@ 153.50", "@ 148.00") |

### 62.5 D. Execution

| Parametru | Descriere |
|-----------|-----------|
| **Auto-submit TP/SL brackets** | Checkbox — cand activ, ordinele EXEC/AI includ automat bracket orders (Take Profit + Stop Loss) calculate din desen |
| **Order Type** | Selector: Market (executie imediata) sau Limit (pret specificat) |

### 62.6 E. Visual

| Parametru | Descriere |
|-----------|-----------|
| **Profit color** | Color picker pentru zona de profit (default: #00e676 verde) |
| **Stop color** | Color picker pentru zona de stop/loss (default: #ff1744 rosu) |
| **Box Opacity** | Slider 0-100% — controleaza transparenta fill-ului box-urilor. Default: 6% (subtil transparent) |

### 62.7 F. Drawing Management

| Buton | Actiune |
|-------|---------|
| **CLEAR ALL DRAWINGS** | Sterge **toate** desenele de pe chart-ul curent (orice tip) |
| **CLEAR LONG/SHORT ONLY** | Sterge doar desenele de tip risk box (long/short), pastreaza linii, text, etc. |
| **CLEAR ALL SYMBOLS** | Sterge toate desenele pentru **toate** simbolurile din localStorage |
| **Drawing info** | Afiseaza numar desene: "3 drawings (2 L/S)" |

**Drawing count badge:** Pe butonul 🗑️ din toolbar-ul din stanga apare un badge rosu cu numarul total de desene. Se actualizeaza automat la fiecare save/load.

### 62.8 Exec Toolbar — EXEC si AI Buttons

Cand un risk box este selectat pe chart, apare un toolbar contextual cu:

| Buton | Actiune |
|-------|---------|
| **BET $** | Input pentru suma tranzactiei, sincronizat bidirectional cu tab-11 |
| **EXEC** | Trimite ordin manual via `submit_order` cu qty calculat din bet/price |
| **AI** | Trimite ordin AI-asisted (acelasi flux, flag `ai_mode: true`) |

**Bracket support:** Daca checkbox-ul "Auto-submit TP/SL" este activ, ordinul include `take_profit` si `stop_loss` calculate din preturile desenului.

### 62.9 Persistenta Setarilor

Toate setarile Long/Short sunt salvate in `localStorage` sub cheia `titan_ls_settings` ca JSON. La pornirea aplicatiei, `_lsLoad()` restaureaza setarile si `_lsApplyToUI()` le aplica pe controalele din tab-11.

Obiectul global `window._lsSettings` contine:
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

AI Scanner-ul a fost optimizat pentru viteza maxima folosind procesarea paralela.

- **Tehnologie:** Integrare `rayon::pre_iter` in backend-ul Rust.
- **Performanta:** In loc sa analizeze simbolurile unul cate unul, Titan foloseste toate thread-urile disponibile ale CPU-ului (ex: 12 sau 16 thread-uri simultan).
- **Impact:** Timpul de scanare pentru 200 de simboluri a scazut de la ~2 minute la sub 30 de secunde (depinde de viteza de raspuns a API-ului Alpaca).

## 64. DEEP AUDIT REASONING — FEREASTRA MODAL

Fiecare scor generat de AI Scanner poate fi acum auditat in detaliu.

- **Activare:** Click pe iconita 🔍 (Deep Audit) de langa orice simbol in AI Scanner sau Smart Picks.
- **Fereastra Modal (V3):** O interfata dedicata cyberpunk care "pluteste" deasupra aplicatiei.
- **Continut:** Afiseaza componentele scorului (Fusion Signal, Momentum, Quality Factors) si motivele pentru care a primit acea nota.
- **Tip:** Daca datele lipsesc (scanari vechi), sistemul te anunta ca este necesar un nou `MANUAL SCAN`.

## 65. MANUAL TRADE USD VS SHARES TOGGLE

Control total asupra modului de plasare a mizelor in panelul de tranzactionare manuala.

- **Buton USD/SHARES:** Comuta instantaneu intre miza fixa in dolari (ex: $3000) si miza fixa in numar de actiuni (ex: 50 Shares).
- **Legibilitate:** Fonturile din sidebar au fost crescute (14px) pentru a fi vizibile fara efort in timpul sesiunilor intense de trading.
- **Cache Fix:** Scriptul `build.sh` sterge automat cache-ul WebView app-specific (`~/.local/share/com.titan.hft.v7reloaded` si `~/.cache/titan-hft-v7-reloaded`) pentru a asigura vizibilitatea modificarilor UI.

---

## 66. SIGNAL STRATEGY — TRADING PE BAZA SAGETILOR

Signal Strategy este un mod de tranzactionare care se bazeaza exclusiv pe sagetile BUY/SELL/TRAIL de pe chart, ignorand toti ceilalti parametri ai strategiei.

### 66.1 Principiu

- Cand apare o sageata **BUY** pe chart → botul deschide pozitie LONG
- Cand apare o sageata **SELL** sau **TRAIL** → botul inchide pozitia
- Nu tine cont de scor, OFI, spread, sau alti parametri (daca bypass-urile sunt active)

### 66.2 Activare

Click pe butonul **SIGNAL STRATEGY** din bara de mod (langa Oracle / GPU / Fusion).

### 66.3 Sursa Semnale (Signal Source Selector)

Dupa activarea Signal Strategy, apare un sub-selector: **FUSION** | **ORACLE** | **GPU**. Selecteaza de la care strategie se vad sagetile pe chart:
- **FUSION** (default) — sageti bazate pe blend Oracle + GPU
- **ORACLE** — sageti bazate pe cele 7 indicatoare AI matematice
- **GPU** — sageti bazate pe momentum-ul celor 150K agenti GPU

### 66.4 Lista Assets Dedicata

Signal Strategy are propria lista de assets (separata de HFT Targets):
- **ADD** — adauga manual un simbol
- **INJECT FROM** — AI Scanner, Smart Picks, Shark Scanner, Watchlist
- **EXPORT TO HFT TARGETS** — exporta in lista principala HFT
- **INJECT TO MANUAL** — seteaza toate asset-urile ca "manual" in Manual Trade list
- **DEL ALL** — sterge toata lista

### 66.5 Tabel Assets cu Informatii Enriched

Fiecare asset afiseaza: PRICE, CHG%, RVOL, VOL, ORACLE score, NEXUS %, FUSION score, SIGNAL (BUY/SELL/NEUTRAL), BEST TF, si butoane actiune.

---

## 67. SIGNAL STRATEGY — TAB CONFIGURARE PARAMETRI (SIGNAL CFG)

### 67.1 Structura

Tab-ul Signal CFG contine 3 sectiuni de safety cu parametri independenti de strategia principala:

**A. ENTRY SAFETY:**
- Max Spread (bps) — 1 la 500
- Cooldown (secunde) — 1 la 300
- Daily Loss Limit ($) — 10 la 100000
- Max Daily Trades — 1 la 200
- Max Open Positions — 1 la 50
- OFI Threshold — 0 la 100000

**B. POSITION SAFETY:**
- Trailing Stop (%) — 0.1 la 20
- Stop Loss (%) — 0.1 la 30
- Take Profit (%) — 0.1 la 50
- Max Hold Time (secunde) — 60 la 86400

**C. EXIT SAFETY:**
- Min Hold Time (secunde) — 0 la 600
- Min Profit to Trail ($) — 0.1 la 10000
- Max Correlation — 0.1 la 1.0

### 67.2 Bypass per Parametru

Fiecare parametru (cu exceptia Max Open Positions) are un checkbox **BYP** (Bypass). Cand activ, acel parametru este complet ignorat de Signal Strategy. Permite control granular — de exemplu, poti bypasa cooldown-ul dar pastra stop loss-ul activ.

### 67.3 Smart Default

Butonul **SMART DEFAULT** seteaza toti parametrii la valori optimizate pentru trading activ:
- Spread: 100 bps, Cooldown: 15s, Daily Loss: $500, Max Trades: 50, Positions: 5, OFI: 500
- Trail: 1.5%, SL: 3%, TP: 5%, Hold: 3600s
- Min Hold: 10s, Min Profit Trail: $5, Max Corr: 0.85
- Activeaza bypass pe: spread, cooldown, OFI, hold time, min hold, min profit trail, max correlation

### 67.4 Dimming Vizual

Parametrii cu bypass activ sunt vizual estompati (opacity 40%) pentru a indica clar ca sunt inactivi.

---

## 68. SIGNAL STRATEGY — BACKTEST PE SAGETI

### 68.1 Principiu

Backtest-ul Signal Strategy simuleaza tranzactionarea exclusiv pe baza sagetilor BUY/SELL/TRAIL (ale strategiei selectate: Oracle, GPU, sau Fusion) pe toate cele 9 timeframe-uri Alpaca.

### 68.2 Parametri

- **Slider perioada:** 1 luna pana la 12 luni (afiseaza "1mo", "3mo", "6mo", "12mo")
- **RUN SIGNAL BT** — ruleaza backtest pe asset-ul selectat (cu butonul SEL)
- **BT ALL ASSETS** — ruleaza backtest pe toate asset-urile din lista

### 68.3 Selectare Asset pentru Backtest

In coloana "#" din tabel, fiecare asset are un buton **SEL** (galben). Click pe SEL:
- Selecteaza asset-ul pentru backtest (radio button hidden sub capota)
- Highlight-eaza row-ul si butonul SEL
- Afiseaza toast flash de confirmare

### 68.4 Rezultate

Dupa backtest, se afiseaza:
- **Summary:** Symbol, Best Timeframe, Best Return%, Best Sharpe, Period
- **Tabel pe 9 TF-uri:** Trades, Win%, Return%, Max DD, Profit Factor, Sharpe, Expectancy, Rank
- Cel mai bun TF primeste badge "BEST"
- Coloana "BEST TF" din tabelul assets se actualizeaza automat

### 68.5 Auto-Trade pe TF Optim

Cand Signal Strategy este activ, botul foloseste automat timeframe-ul optim (din ultimul backtest) pentru a trage semnalele de pe chart.

---

## 69. EXTENDED HOURS — CONVERSIE AUTOMATA LIMIT ORDERS

### 69.1 Problema

Alpaca respinge ordinele de tip "market" in extended hours cu eroare 422: "extended hours order must be DAY or GTC limit orders".

### 69.2 Solutie Automata

Cand sunt indeplinite conditiile:
- `extended_hours` este activat in Settings
- Asset-ul NU este crypto
- Ordinul NU are bracket (TP/SL)

...ordinul se converteste automat:
- **Tip:** `market` → `limit`
- **Pret limit:** Pretul curent + 0.5% slippage (buy) sau - 0.5% slippage (sell)
- **Time in Force:** `"day"` (obligatoriu pentru extended hours)

### 69.3 Cooldown Ordine Esuate

Dupa orice ordin esuat, simbolul primeste un **cooldown de 60 secunde**. In aceasta perioada, botul nu mai incearca sa tranzactioneze acel simbol. Previne spamul de retry-uri.

Cooldown-ul se sterge automat la:
- Ordin reusit pe acelasi simbol
- Expirarea celor 60 de secunde

---

## 70. NON-FRACTIONABLE ASSETS — GESTIONARE CANTITATI

### 70.1 Problema

Unele actiuni (ex: PLU) nu suporta cantitati fractionare. Alpaca respinge cu 403: "asset is not fractionable".

### 70.2 Solutie

In momentul plasarii ordinului:
1. Se verifica proprietatea `fractionable` din API-ul Alpaca
2. Daca asset-ul nu este fractionable, cantitatea se rotunjeste in jos la numar intreg (`floor`)
3. Daca cantitatea rotunjita < 1, ordinul este respins devreme cu mesaj clar

### 70.3 Exemplu

- Asset PLU, bet $1000, pret $1.63
- Qty calculat: 614.857
- Asset nu e fractionable → `floor(614.857)` = **614 shares**
- Ordin trimis cu qty=614

---

## 71. CLICK-TO-CHART DIN LIVE POSITIONS

### 71.1 Functionalitate

Click pe orice row din tabelul **Live Positions** incarca acel asset pe chart si comuta automat la tab-ul Grafic Complex.

### 71.2 Comportament

- Click pe row → `setSymbol(symbol)` + `switchTab(0)`
- Click pe butoanele de actiune (SELL, Force Close, Mode Toggle) → actiunea respectiva (nu incarca pe harta)
- Cursor `pointer` si tooltip pe fiecare row pentru a indica interactivitatea

---

## 72. CLICK-TO-CHART DIN SIGNAL STRATEGY ASSETS

### 72.1 Functionalitate

Click pe orice row din tabelul Signal Strategy Assets incarca asset-ul pe chart in best timeframe-ul sau.

### 72.2 Comportament

- Click pe row (in afara butoanelor) → `setSymbol(symbol)` + `setTF(best_tf)` + `switchTab(0)` + flash "loaded on chart"
- Click pe **SEL** → selecteaza pentru backtest (nu incarca pe harta)
- Click pe **BT** → ruleaza backtest pe acel asset
- Click pe **+HFT** → adauga in HFT Targets
- Click pe **X** → sterge din lista

---

## 73. CRASH-SAFE PERSISTENCE & CHART STATE

### 73.1 Settings Persistence (Disk)

Toate setarile aplicatiei sunt salvate in `titan_settings.json`:
- **Periodic:** La fiecare 30 secunde (`settings_autosave_loop`)
- **La schimbare:** La fiecare update de settings din UI
- **Atomic:** Scriere in fisier temporar (`.tmp`) apoi `rename` → previne coruperea la crash

Setari persistate includ: toate toggle-urile, slidere-le, scalping modes, bypass-urile Signal Strategy, signal source, signal assets list, scan params, position modes, etc.

### 73.2 State Snapshot (Disk)

State-ul runtime se salveaza separat in `titan_state_snapshot.json`:
- Trailing stops, partial taken, last prices
- Session analytics (equity, PnL, drawdown, trade PnLs)
- AI scan results complet
- Scan progress (pentru auto-resume)

### 73.3 Chart State Preservation

La switch intre tab-uri si revenirea pe Grafic Complex:
- **Zoom si scroll** se pastreaza (nu se reseteaza)
- **Timeframe** se pastreaza
- **Simbolul** se pastreaza

Implementare: `resizeCharts()` in loc de `chart.timeScale().fitContent()` la revenirea pe tab 0.

## 74. CATALYST CROSS-TAB INTEGRATION (Deep Audit)

Sistemul Catalyst nu funcționează izolat în Tab 7 — este integrat în **5 tab-uri** prin helper-uri globale.

### 74.1 `_catalystBadgeHtml(sym, candidate)` — Badge Global

Funcție JS globală care generează badge-ul CAT pe orice tabel:
- **🔥CAT** (verde/galben) pentru scoruri pozitive
- **🔻CAT** (rosu) pentru scoruri negative
- Tooltip cu headline, scor și vârstă ("2m ago")
- Click deschide URL-ul știrii în browser extern
- Folosește `catalystSeverityColor()` pentru culori consistente

### 74.2 `_updateCatalystCacheFromResults(results)` — DRY Helper

Funcție centralizată de cache update, folosită în 3 locuri:
- Watchlist CHECK NEWS
- HFT Targets CHECK NEWS
- `_checkSingleCatalyst` per-symbol check

Eliminare completă a celor 3 blocuri de cod duplicat.

### 74.3 Integrare per Tab

| Tab | Feature Catalyst |
|-----|------------------|
| **Tab 1 (News AI)** | Buton 🔥 TEST BULLISH (gradient), article cards cu severity badge (EXPLOSIVE/STRONG) |
| **Tab 6 (AI Scanner)** | Badge CAT în tabele scan results + SMART PICKS, cache auto-update la `ai-scan-update` |
| **Tab 9 (Watchlist)** | Badge CAT per simbol + buton CHECK NEWS pe toate simbolurile (cu error feedback) |
| **Tab 10 (HFT Targets)** | Badge CAT per target + buton CHECK NEWS (cu error feedback) |
| **Signal CFG** | Toggle REQUIRE CATALYST (filtru scan) |

### 74.4 Error Feedback pe CHECK NEWS

La eșec, butoanele CHECK NEWS (Watchlist + HFT) afișează:
- Text `ERROR` rosu timp de 2 secunde
- Apoi revin la textul normal `CHECK NEWS`
- Eroarea se loghează în consolă

---

## 75. ARCHITECTURE MODERNIZATION — data.rs SPLIT (Phase 9)

### Problema

Fisierul `commands/data.rs` avea ~3,800 linii cu concern-uri mixte: market data, order execution, operatiuni diverse, scoring/backtest. Dificil de navigat, dificil de revizuit la code review, risc crescut de merge conflicts.

### Solutia — 4 Module de Domeniu

| Modul | Responsabilitate | Linii |
|-------|-----------------|-------|
| `commands/market_data.rs` | Candles, snapshots, quotes, price queries | ~300 |
| `commands/order_exec.rs` | Executie ordine, bracket orders, order management | ~250 |
| `commands/ops.rs` | Health check, state queries, operatiuni miscelanee | ~200 |
| `commands/scoring.rs` | Backtest scoring, GPU scores, signal markers, AI scan scoring | ~2,415 |

`data.rs` a fost redus la un **shim de re-export** (~50 linii) care face `pub use` din sub-module. Toate comenzile Tauri existente functioneaza identic — zero breaking changes.

### Structura commands/

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

### Structura TitanError

`TitanError` este enum-ul central de erori al aplicatiei, definit in `src-tauri/src/error.rs`:

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

### Conversii Automate (From impls)

- `From<reqwest::Error>` — detecteaza timeout vs network generic
- `From<serde_json::Error>` — wraps ca Internal("JSON: ...")
- `From<std::io::Error>` — wraps ca Io
- `From<toml::de::Error>` — wraps ca Internal("TOML: ...")
- `From<String>` — wraps ca Internal (backward compat)
- `From<TitanError> for String` — Tauri 1.x compatibility (trimite ca string la frontend)

### Macro titan_bail!

```rust
titan_bail!(Network, "API call failed: {}", url);
// Expands to: return Err(TitanError::Network(format!("API call failed: {}", url)))
```

### Impact

- 84+ functii migrate de la `Result<T, String>` la `Result<T, TitanError>`
- Operatorul `?` functioneaza natural pe reqwest, serde_json, io, toml
- Zero modificari frontend (TitanError se converteste automat la String pentru Tauri)

---

## 77. DECIMAL MIGRATION — Tier 1 (Phase 7C)

### Campuri Migrate la rust_decimal::Decimal

**AppState / AppSettings:**
- `equity` — capitalul total al contului
- `bot_trade_size` — marimea standard per trade
- `circuit_breaker_limit` — limita circuit breaker
- `max_manual_order_usd` — cap maxim per ordin manual

**TradeAction:**
- `price` — pretul de executie
- `qty` — cantitatea
- `pnl` — profit/loss

**EntryContext:**
- `equity` — equity la momentul intrarii

**StateSnapshot (6 campuri):**
- `session_starting_equity`, `session_peak_equity`, `session_current_equity`
- `session_total_pnl`, `session_best_trade`, `session_worst_trade`

### Serde Adaptorii

Pentru compatibilitate cu fisierele JSON existente (care stocheaza numere ca float-uri), se folosesc adaptori custom:

```rust
#[serde(serialize_with = "serialize_decimal_as_f64")]
#[serde(deserialize_with = "deserialize_decimal_from_f64")]
pub equity: Decimal,
```

### Ce ramane f64

Procentele, ratiile, scorurile, multiplicatorii, threshold-urile, statisticile hardware, alpha blends, multiplicatorii ATR — toate raman f64 intentionat (nu necesita precizie financiara).

---

## 78. FRONTEND MODULARIZATION (Phase 7E)

### De la Monolit la 30 Module

`index.html` a fost redus de la **15,454 linii** la **2,595 linii** prin:

1. **CSS extern** — bloc `<style>` de 2,877 linii extras in `src/styles.css`
2. **30 module JavaScript** — toate IIFE-urile si scripturile inline extrase in fisiere separate
3. **switchTab fix** — lantul de 6 overrides pe `window.switchTab` inlocuit cu o singura definitie in `titan_tabs.js`

### Ordinea de Incarcare

Scripturile se incarca in `<head>` (inainte de DOM) si la sfarsitul `<body>` (dupa DOM):

**Head (utilities si core):**
```html
<script src="lightweight-charts.js"></script>
<script src="titan_core.js"></script>
<script src="titan_bus.js"></script>
<!-- ... 16 module infrastructure ... -->
<script src="titan_bootstrap.js"></script>
```

**End of body (feature modules, dupa DOM):**
```html
<script src="titan_globals.js"></script>
<script src="titan_journal.js"></script>
<!-- ... 8 module feature ... -->
<script src="titan_ls_settings.js"></script>
```

### Shared Utilities (titan_core.js)

Toate modulele au acces la utilitatile din `titan_core.js`:
- `escapeHtml(str)` — XSS protection
- `normalizeSymbol(sym)` — normalizare simboluri (BTC → BTC/USD)
- `formatNum(n, d)`, `fmtPct(n)`, `fmtUsd(n)` — formatare numerica
- `formatVolume(val, decimals)` — formatare volume (K/M/B)
- `pnlColor(n)`, `sideColor(side)` — culori conditionale
- `$el(id)` — DOM cache helper
- `setTxt(id, val, color)` — set text content (cached)
- `TitanTimers` — registry centralizat intervale
- `TitanError` — error reporting centralizat
- `TitanDebug` — debug logging cu timestamp
- `window.invoke(cmd, args)` — wrapper Tauri invoke

---

## 79. XSS HARDENING & escapeHtml (Phase 10)

### Functia escapeHtml

```javascript
function escapeHtml(str) {
    if (!str) return '';
    return String(str).replace(/&/g,'&amp;')
        .replace(/</g,'&lt;').replace(/>/g,'&gt;')
        .replace(/"/g,'&quot;').replace(/'/g,'&#039;');
}
```

### Audit Sistematic

Toate asignarile `innerHTML` care includ valori dinamice au fost auditate in 15 fisiere JS. Valorile de tip string (simboluri, headline-uri, timestamp-uri, labels, ID-uri) sunt acum wrappate cu `escapeHtml()`:

```javascript
// Inainte (vulnerabil la XSS):
row.innerHTML = `<td>${t.symbol}</td><td>${t.reason}</td>`;

// Dupa (hardened):
row.innerHTML = `<td>${escapeHtml(t.symbol)}</td><td>${escapeHtml(t.reason)}</td>`;
```

### Exceptii Intentionate

Valorile numerice calculate (formatate cu `.toFixed()`, `formatVolume()`, etc.) NU sunt escaped — sunt sigure by construction deoarece produc doar cifre si caractere `.`, `%`, `$`, `-`.

### Helper Unificat formatVolume

`fmtVol` si `fmtK` (doua functii duplicat) au fost inlocuite cu o singura functie:

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

Definit in `titan_core.js`, inlocuieste `console.error` in catch blocks:

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

Logging cu timestamp, partial replace pentru `console.log`:

```javascript
const TitanDebug = {
    log(tag, ...args) {
        console.log(`[${new Date().toISOString()}][${tag}]`, ...args);
    }
};
```

### TitanTimers

Registry centralizat pentru `setInterval` / `clearInterval`:

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

### Console.warn Pastrate Intentionat

~20 `console.warn` non-critice sunt pastrate (localStorage fallback, CRUD operations, capability detection) — nu sunt erori ci informatii operationale.

---

## 81. DOM CACHE SYSTEM — $el Helper cu WeakRef (Phase 3 + RF-D Part 1)

### Implementare (titan_core.js)

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

### Cum Functioneaza

- **WeakRef cache:** Fiecare element gasit e stocat ca `WeakRef`. Cand nodul e GC-ed sau deconectat din DOM (`!isConnected`), referinta e invalidata automat si se face o noua cautare DOM.
- **Null misses:** Elementele inexistente (null) nu sunt cached — se re-incearca la fiecare apel. Previne blocarea pe elemente care apar mai tarziu in lifecycle.
- **Fallback:** Daca `WeakRef` nu exista in runtime, `$el()` face direct `document.getElementById()` de fiecare data.
- `setTxt(id, val, color)` foloseste `$el(id)` intern — deci toate apelurile `setTxt` beneficiaza automat.

### Debug API

`window.TitanDomCacheDebug.snapshot()` returneaza contoare live:

| Contor | Ce masoara |
|--------|-----------|
| `lookups` | Total apeluri `$el()` |
| `cacheHits` | Returnat din WeakRef fara DOM query |
| `staleInvalidations` | Re-resolve dupa `deref()` null sau `!isConnected` |
| `domQueries` | Cautari efective `document.getElementById()` |
| `domQueryMisses` | Cautari care au returnat null |
| `fallbackDirectCalls` | Apeluri directe (WeakRef indisponibil) |
| `weakRefEnabled` | `true` daca WeakRef e activ |

`window.TitanDomCacheDebug.reset()` — reseteaza toate contoarele la 0.

### Fisiere Migrate (RF-D Part 1)

Toate apelurile `document.getElementById` din aceste fisiere au fost inlocuite cu `$el()`:
- `titan_perf_stats.js` (~186 instante)
- `titan_ls_settings.js` (~72 instante)
- `titan_manual_trade.js` (~60 instante)

Plus hot-paths convertite in Phase 3 originala:
- `titan_monitor.js` — `updateNyTime` (1s), `applyAiTargetUi` (1.5s), `applyRedisStatus`
- `titan_positions.js` — `updateAccount` (3s), `updateStrategyScore` (4s)
- `titan_bootstrap.js` — `price-update` (200ms), `l2-top`, `l2-update` listeners
- `titan_core.js` — `renderOfi`, `renderFlow`
- `titan_indicators_ui.js` — `bindIndicatorToggles`, toggle state
- `titan_chart_settings.js` — `applyIndicatorToggleState`

### Anti-Regression Guard

`guard_no_direct_getelementbyid.sh` — script care esueaza daca `document.getElementById(` reapare in fisierele migrate. Rulat automat de `build.sh` la pasul 2 (DOM regression guard). Pe masura ce se migreaza noi fisiere, se adauga in lista din script.

### Scope de Extindere

Raman ~400+ apeluri `document.getElementById` in restul modulelor (titan_hft_panel, titan_bootstrap, titan_sidebar, etc.), planificate pentru conversie incrementala in sesiunile urmatoare (RF-D Part 2+).

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

Rezultat: binar ~14.7MB (de la ~17MB fara LTO/strip).

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

Module optionale gate-uite cu `#[cfg(feature = "...")]`:
- `mod gpu` + GPU Nexus loop + compute shaders
- `mod telegram` + alert integration points
- `mod web_dashboard` + axum server
- `mod options` + options chain fetch + hedge commands

### CI Compliance

- `cargo fmt` — formatare consistenta pe tot codebase-ul
- `cargo clippy --workspace -- -D warnings` — zero warnings (blocheaza build-ul)
- `cargo test --workspace` — 259 teste, toate passed
- `cargo deny check` — license audit + advisory scan (deny.toml cu whitelist)
- `cargo audit` — CVE vulnerability scan
- GitHub Actions workflows (`.github/workflows/ci.yml` + `release.yml`) — ruleaza automat la push/PR/tag
- Criterion benchmarks disponibile on-demand: `cargo bench` (oracle, indicators, full pipeline)

### Build Script — Pipeline 13-Step cu Validare-First

`build.sh` ruleaza 13 pasi secventiali. Toate verificarile (pasi 1-6) se executa INAINTE de build/deploy — orice esec opreste pipeline-ul fara a afecta instalarea curenta.

| Pas | Ce face | Env var de control |
|-----|---------|-------------------|
| 1 | `node --check` pe 33 fisiere JS first-party (exclude `lightweight-charts.js`, `node_modules`, `dist`, `target`) | — |
| 2 | DOM regression guard (`guard_no_direct_getelementbyid.sh`) — verifica ca fisierele migrate nu reintroduc `document.getElementById` | — |
| 3 | Static UI smoke — serveste `src/` local, lanseaza Chrome headless + CDP, testeaza Manual Trade, quick buy/sell, KILL ALL confirm, cyberConfirm. Necesita `requests` + `websockets` Python modules. | `TITAN_SKIP_STATIC_UI_SMOKE=1` (skip) |
| 4 | `cargo test --workspace` (259 teste, fail-fast) | — |
| 5 | `cargo clippy --workspace -- -D warnings` (**blocheaza** la orice warning) | — |
| 6 | Benchmarks (optional): focused `bench_check_entry` sau full `cargo bench` | `TITAN_RUN_BENCH=1` (focused), `TITAN_RUN_FULL_BENCH=1` (full) |
| 7 | `rsync -a --delete src/ → dist/` | — |
| 8 | `cargo build --release --features custom-protocol` | — |
| 9 | Clear Webview cache (`~/.local/share/com.titan.hft.v7reloaded`, `~/.cache/titan-hft-v7-reloaded`) | — |
| 10 | Stop instanta running | — |
| 11 | Deploy binary la `~/.local/bin/titan-hft-v7-reloaded` + desktop entries + icons (32, 128, 512) | — |
| 12 | Launch cu logging in `$HOME/HFT_Bot/logs/titan_YYYYMMDD_HHMMSS.log` | — |
| 13 | Summary: durata, binary size, timestamp | — |

**Wrapper cu benchmark:** `build_bench.sh` — executabil care ruleaza `TITAN_RUN_BENCH=1 ./build.sh` (shortcut pentru validare completa cu benchmark focalizat).

---

## 83. SIGNAL SCALPING KIT (Sesiunea 17) — Alerte Realtime + Journal Exact

Acest capitol documenteaza update-urile pentru scalping manual in `SIGNAL_STRATEGY`, livrate in sesiunea 17.

### 83.1 Ce este nou in UI

- `SIGNAL HEALTH` panel afiseaza live:
  - `SRC` (FUSION / ORACLE / GPU)
  - `TF` activ
  - `AGE` ultimului update
  - numar markeri `BUY / SELL / TRAIL / CHAOS`
- Controale noi in panel:
  - `ALERTS ON/OFF`
  - `VOL` (0-100%)

### 83.2 Alerte realtime (audio + vizual)

- Se declanseaza pentru semnale noi: `BUY`, `SELL`, `TRAIL`, `CHAOS`
- Fiecare tip are:
  - toast dedicat cu culoare distincta
  - tonalitate audio distincta
- Cand `ALERTS` este `OFF`:
  - nu se reda audio
  - nu apare toast
  - semnalele noi sunt marcate intern ca vazute (evita spam la reactivare)

### 83.3 Persistenta preferintelor alerte

Preferintele sunt stocate local in browser storage:

- `enabled` (true/false)
- `volume` (0.0-1.0)

La restart aplicatia restaureaza automat ultima configuratie.

### 83.4 Journal exact pe source/timeframe (backend)

`TradeRecord` include explicit:

- `source` (`ORACLE` / `GPU` / `FUSION` / `UNKNOWN`)
- `timeframe` (`1m`, `3m`, ..., `4h`, `n/a`)

Beneficii:

- KPI pe sursa/TF devin exacte (fara inferenta din text)
- Export CSV backend include coloanele `Source` si `Timeframe`
- Datele vechi raman compatibile (`UNKNOWN` / `n/a` default)

### 83.5 Ghid rapid de utilizare (scalping manual)

1. Seteaza `MODE = SIGNAL`, apoi alege `SRC` (`FUSION`, `ORACLE`, `GPU`).
2. Activeaza `SIGNALS` pe chart si verifica marker-ele live.
3. In `SIGNAL HEALTH`, lasa `ALERTS ON` si seteaza `VOL` confortabil.
4. Dupa sesiune, analizeaza KPI Journal pe `source` si `timeframe`.

### 83.6 Nota operationala

Pentru sesiunea urmatoare, backlog-ul prioritar ramane:

- 7D Strategy Indicators
- DOM Cache Expansion
- Decimal Tier 2

---

## 84. TOTI PARAMETRII DE TRADING CONFIGURABILI (Sesiunea 18)

In sesiunea 18 au fost expusi **absolut toti** parametrii pe care botul ii foloseste in procesul de trading. Anterior, aceste valori erau hardcodate in cod Rust. Acum fiecare este:

- Vizibil si ajustabil prin slider in tab-ul **HFT Engine**
- Persistat automat in `titan_settings.json` la SAVE
- Respectat strict de bot in toate deciziile de trading
- Backward-compatible: valorile default sunt identice cu cele anterioare

### 84.1 AUTOPILOT CORE (10 parametri)

Controleaza comportamentul fundamental al autopilot-ului:

| Parametru | Default | Domeniu | Descriere |
|-----------|---------|---------|-----------|
| `min_hold_secs` | 180 | 0-3600 | Timp minim de detinere inainte de exit fortat |
| `nexus_consensus_threshold` | 70 | 30-100 | Pragul de consens Nexus pentru ajustarea stop/take |
| `autopilot_loop_ms` | 1500 | 500-10000 | Intervalul buclei autopilot (ms) |
| `ofi_stale_ms` | 10000 | 1000-60000 | Timpul dupa care OFI este considerat expirat |
| `min_bars_for_entry` | 20 | 5-200 | Nr. minim de lumanari necesare pentru intrare |
| `ofi_entry_threshold` | 5.0 | 0-50 | Pragul OFI pentru confirmarea intrarii |
| `max_qty_cap` | 1000 | 1-100000 | Cantitatea maxima per ordin |
| `min_qty_threshold` | 0.001 | 0.0001-1.0 | Cantitatea minima acceptata |
| `default_equity_fallback` | 10000 | 1000-1000000 | Equity default cand datele lipsesc |
| `max_bypass_risk_pct` | 25 | 1-100 | Riscul maxim in mod bypass |

### 84.2 EXIT SCALING (17 parametri)

Controleaza modul in care se calculeaza stop-loss, take-profit si trailing stop:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `vol_factor_min/max` | 0.6 / 2.5 | Domeniul factorului de volatilitate |
| `fallback_sl/tp/trail_pct` | 2.5 / 2.5 / 1.5 | Procentaje fallback SL/TP/trail |
| `scalping_stop/take/trail_mult` | 0.70 / 0.60 / 0.50 | Multiplicatori scalping |
| `partial_first_target_frac` | 0.5 | Fractiunea TP pentru prima iesire partiala |
| `partial_min_qty` | 0.001 | Cantitatea minima pentru iesire partiala |
| `atr_period` | 14 | Perioada ATR |
| `atr_stop_clamp_min/max` | 0.5 / 8.0 | Domeniul clampare ATR stop |
| `atr_take_clamp_min/max` | 1.0 / 15.0 | Domeniul clampare ATR take |
| `atr_trail_clamp_min/max` | 0.3 / 5.0 | Domeniul clampare ATR trail |

### 84.3 ORACLE EXIT (6 parametri)

Praguri pentru iesirile bazate pe Oracle:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `oracle_exit_min_confidence` | 0.40 | Confidenta minima pentru exit pe semnal opus |
| `oracle_exit_min_score` | 30.0 | Scorul minim pentru exit pe semnal opus |
| `oracle_flip_score_threshold` | 10.0 | Pragul de scor pentru Oracle Flip Exit |
| `oracle_flip_confidence` | 0.20 | Confidenta minima pentru Flip Exit |
| `chaos_exit_max_loss_pct` | -0.5 | Pierderea maxima in regim Chaotic |
| `chaos_exit_min_confidence` | 0.25 | Confidenta minima pentru Chaos Exit |

### 84.4 CIRCUIT BREAKER (5 parametri)

Pragurile de protectie la pierderi zilnice:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `breaker_caution_pct` | -3.0% | Pragul de atentionare |
| `breaker_halt_pct` | -5.0% | Pragul de oprire intrari noi |
| `breaker_panic_pct` | -8.0% | Pragul de panic (kill switch) |
| `max_notional_equity_frac` | 0.25 | Fractiunea maxima din equity per pozitie |
| `min_stop_distance_frac` | 0.0001 | Distanta minima stop ca fractiune din pret |

### 84.5 STRATEGY SCORING (9 parametri)

Parametrii motorului de scoring al strategiei:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `strategy_base_score` | 50 | Scorul de baza (neutral) |
| `vpin_gate_scalping` | 0.85 | Pragul VPIN pentru scalping |
| `vpin_gate_normal` | 0.75 | Pragul VPIN pentru trading normal |
| `min_rvol_threshold` | 0.5 | Volumul relativ minim pentru intrare |
| `rvol_window` | 20 | Fereastra de calcul RVOL |
| `rvol_relax_threshold` | 3.0 | Pragul RVOL pentru relaxare |
| `rvol_relax_value` | 5.0 | Ajustarea pragului la volum mare |
| `chaos_confidence_kill` | 0.10 | Pragul de confidenta sub care scorul se anuleaza |
| `chaos_regime_mult` | 0.55 | Multiplicatorul regimului pentru contributia Oracle |

### 84.6 ENGINE STOPS (6 parametri)

Stop-uri si trail-uri din trading engine (manage_positions):

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `long_stop_normal` | -2.5% | Stop loss normal pentru long |
| `long_stop_aggressive` | -8.0% | Stop loss agresiv pentru long |
| `short_stop_pct` | -5.0% | Stop loss pentru short |
| `long_trail_min_profit` | 1.5% | Profitul minim pentru activare trailing long |
| `short_trail_gap` | 1.5% | Gap-ul trailing pentru short |
| `short_trail_min_profit` | 1.0% | Profitul minim pentru trailing short |

### 84.7 ORACLE FUSION (8 parametri)

Ponderile si pragurile din fuziunea Oracle (7 layere):

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `oracle_kalman_weight` | 0.30 | Ponderea Kalman in fuziune |
| `oracle_entropy_weight` | 0.25 | Ponderea Wavelet/Entropy |
| `oracle_hurst_weight` | 0.20 | Ponderea Hurst |
| `oracle_vpin_weight` | 0.15 | Ponderea VPIN |
| `oracle_wavelet_weight` | 0.10 | Ponderea directiei Entropy |
| `oracle_confidence_floor` | 0.25 | Pragul minim de confidenta |
| `entropy_chaos_threshold` | 0.85 | Pragul entropiei pentru regim chaotic |
| `vpin_toxicity_alert` | 0.6 | Pragul VPIN pentru alerta de toxicitate |

### 84.8 INDICATOR PERIODS (12 parametri)

Perioadele tuturor indicatorilor tehnici clasici:

| Parametru | Default | Descriere |
|-----------|---------|-----------|
| `ema_fast_period` | 9 | EMA rapida |
| `ema_mid_period` | 20 | EMA medie |
| `ema_slow_period` | 50 | EMA lenta |
| `ema_trend_period` | 200 | EMA trend |
| `rsi_period` | 14 | Perioada RSI |
| `macd_fast` | 12 | MACD linia rapida |
| `macd_slow` | 26 | MACD linia lenta |
| `macd_signal` | 9 | MACD semnal |
| `bb_period` | 20 | Bollinger Bands perioada |
| `bb_std_dev` | 2.0 | Bollinger Bands deviatia standard |
| `adx_period` | 14 | ADX perioada |
| `stoch_rsi_period` | 14 | Stochastic RSI perioada |

### 84.9 Cum functioneaza tehnic

1. **UI**: Fiecare parametru are un slider in tab-ul HFT Engine, organizat in 8 sectiuni color-coded
2. **Save**: La click pe SAVE, toate valorile se trimit ca batch la backend (`batch_update_settings`)
3. **Backend**: `apply_setting_field` parseaza, clampeaza si scrie in `AppState` + `titan_settings.json`
4. **Bot**: Autopilot, strategy, trading_engine, risk, oracle si indicators citesc din `settings.*` in loc de valori hardcodate
5. **Load**: La prima vizita pe tab dupa restart, `loadHftSettings` hidrateaza slider-ele din snapshot-ul backend

### 84.10 Recomandari de utilizare

- **Nu modifica toti parametrii deodata** — ajusteaza cate o sectiune si observa efectul
- **Valorile default sunt optimizate** — sunt cele care au fost testate si hardcodate initial
- **Parametrii Circuit Breaker sunt critici** — nu slabi protectiile decat daca stii exact ce faci
- **Indicator Periods afecteaza semnalele** — schimbarea lor modifica BUY/SELL signals
- **Dupa modificari, ruleaza in DRY RUN** inainte de trading real

---

## NOTA FINALA

TITAN HFT (Shark V7 Reloaded) este un sistem complet de trading automat cu **87 de capitole** de documentatie.

**Sesiunile 15-16 (25-26 Martie 2026)** au executat Phase 7 complet (Architecture, Modernization & Production Readiness):
- **7A** Dead code cleanup (10 itemi eliminati, 10 struct-uri deduplicate)
- **7B** TitanError migration (84+ functii, thiserror, titan_bail! macro)
- **7C** Decimal migration Tier 1 (campuri monetare AppState/TradeAction/StateSnapshot)
- **7E** Frontend modularizare (30 module JS, index.html 15K→2.5K linii)
- **7F** Aggressive param cleanup
- **7G** CI compliance (cargo fmt + clippy + test + cargo-deny + cargo-audit + GitHub Actions)
- **7H** Production hardening (LTO, feature flags, strip, opt-level 3)
- **Phase 8** Production Safety Sweep (unwrap elimination, XSS hardening, global error handlers)
- **Phase 9** data.rs split (3,800 linii → 4 module de domeniu)
- **Phase 10** innerHTML hardening & formatVolume dedup (15 fisiere auditate)
- **Phase 11** Console error → TitanError.report migration (33 apeluri, 15 fisiere)

**Sesiunea 17 (27 Martie 2026)** a adaugat toolkit-ul SIGNAL pentru scalping manual:
- alerte realtime BUY/SELL/TRAIL/CHAOS
- controale `ALERTS ON/OFF` + `VOLUME`
- jurnal backend cu `source` + `timeframe` explicit pentru KPI exact

**Sesiunea 18 (27 Martie 2026)** a expus absolut toti parametrii de trading (~80+) in tab-ul HFT Engine:
- 8 sectiuni noi: Autopilot Core, Exit Scaling, Oracle Exit, Circuit Breaker, Strategy Scoring, Engine Stops, Oracle Fusion, Indicator Periods
- Fiecare parametru: vizibil ca slider, persistat in `titan_settings.json`, respectat strict de bot
- Backend: 10 fisiere Rust modificate, `SizingParams` extins, `OracleWeights` struct, `IndicatorPeriods` struct
- Validari: cargo check OK, clippy 0 warnings, 80/80 teste passed

**Sesiunea 19 (27-28 Martie 2026)** — Deep Audit + 12 Smart Features + Styling complet:
- 12 features noi: Emergency Kill Dashboard, DRY/LIVE Badge, Undo Disk Persist, Settings Diff View, Per-Symbol Overrides, Live Impact Preview, Regime Heatmap, Parameter Profiles, Risk Calculator, Parameter Health, Auto-Suggest, Search Filter
- Preset-uri SAFE/SCALP extinse la 90+ slidere (toate sectiunile)
- Styling complet refactorizat: cyan sliders unificate, dark inputs, text sizing, health badges violet/orange
- Chart persistence pe disk (`titan_chart_settings.json`) — supravietuieste WebView cache clear
- build.sh actualizat cu icon install step
- Manual dedicat HFT Engine: `TITAN_HFT_ENGINE_MANUAL.md`
- Validari: cargo check OK, clippy 0 warnings, 80/80 teste passed, 0 warnings

**Sesiunea 20 (28-29 Martie 2026)** — Cyber-HUD + Coloane Ross Cameron + FMP Enrichment:
- Tema Cyber-HUD pe 3 tabele (HFT, Signal, Watchlist): glassmorphism, neon cyan/magenta/gold
- Coloane Ross Cameron complete: GAP%, FLOAT, CAP, INS, SCORE pe toate 3 tabelele
- FMP enrichment universal (pipeline cache, fallback graceful)
- Cross-module refresh automat la inject simboluri

**Sesiunea 21 (29 Martie 2026)** — Phase 9 Final Upgrades:
- **Position FSM** — enum `PositionState` cu 5 stari, 5 campuri noi pe Position, backward compatible
- **Micro-Latency Metrics** — ring buffers pentru order RTT/GPU/streamer, percentile p50/p95/p99
- **WebSocket Dashboard** — live push la 2s, trade/alert events, client JS cu auto-reconnect (eliminat polling)
- **Session Analytics** — streak tracking, best/worst PnL, average duration, partial exit journaling
- Clippy 0 warnings, 122 teste passed

Backend: 59+ fisiere Rust, 122 teste, 0 clippy warnings, 4 crate-uri workspace (titan-types, titan-risk, titan-genetic, titan-indicators). Frontend: 30 module JS (la momentul sesiunii 21; acum 33 module), DOM cache $el cu WeakRef, TitanError.report centralizat, TitanTimers registry. WebSocket dashboard live.

Ramas pentru urmatoarea sesiune: 7D (Strategy TA Filters), DOM Cache Expansion, Decimal Tier 2, Profile persistence pe disk, Latency Metrics UI, Position FSM Status Column.

---

## 85. DEEP AUDIT + 12 SMART FEATURES + STYLING (Sesiunea 19)

In sesiunea 19 s-a realizat un deep audit al intregului tab HFT Engine, s-au adaugat 12 functionalitati noi ("Smart Features"), s-a refactorizat complet styling-ul, si s-au extins preset-urile SAFE/SCALP la toate slider-ele (~90+).

### 85.1 Emergency Kill Dashboard

Panel sticky in partea de sus a tab-ului HFT Engine cu:
- **KILL ALL** — buton rosu care apeleaza `emergency_liquidate` pe backend, lichideaza TOATE pozitiile si anuleaza TOATE ordinele active. Necesita confirmare.
- **Breaker Status** — arata starea curenta a Circuit Breaker-ului (NORMAL / CAUTION / HALT / PANIC) cu culori corespunzatoare
- **Equity** — valoarea curenta a contului
- **Daily P&L** — profitul/pierderea zilnica
- **Positions** — numarul de pozitii deschise

Panelul se actualizeaza automat la fiecare 3 secunde.

### 85.2 DRY RUN / LIVE Badge

Badge afisat langa titlul "HFT ENGINE CONFIG":
- **DRY RUN** (albastru) — botul simuleaza, nu executa ordine reale
- **LIVE** (verde, pulsand) — botul executa ordine reale pe piata

Badge-ul citeste starea reala din backend si se sincronizeaza prin evenimentul `dryRunChanged` pe `TitanBus`.

### 85.3 Preset-uri SAFE DEFAULT / SCALP DEFAULT

Doua butoane care seteaza automat TOATE slider-ele (~90+) din tab la valori pre-calculate:

**SAFE DEFAULT** — parametri conservatori:
- Stop-uri mai stranse, trailing stop mai mic
- Mai putine pozitii simultane
- Riscul pe trade mai mic
- Potrivit pentru piete volatile sau incepatori

**SCALP DEFAULT** — parametri agresivi:
- Stop-uri mai largi, trailing stop adaptat la scalping rapid
- Mai multe pozitii simultane
- Take-profit mai mic dar frecventa mai mare
- Potrivit pentru piete cu lichiditate mare si volatilitate mica

Fiecare preset are un **toggle lock** (switch on/off):
- Cand este ON, preset-ul este "locked" — se aplica automat la startup si previne modificari manuale ale slider-elor
- Doar un preset poate fi locked la un moment dat
- Starea lock-ului se salveaza in `localStorage`

### 85.4 Search Filter

Camp de cautare "Search parameters..." care filtreaza in timp real toate slider-ele din tab:
- Se potriveste pe numele parametrului (case-insensitive)
- Ascunde slider-ele care nu corespund
- Ascunde sectiunile complet goale
- Functineaza pe toate containerele (.mb-8, .hft-grid-2/3/4)

### 85.5 Undo (Persistent pe Disk)

Butonul **UNDO** restaureaza toate setarile la starea de dinainte de ultima modificare:
- Snapshot-ul se salveaza pe disk in `titan_settings_undo.json`
- Supravietuieste restart/crash
- Se creeaza automat inainte de aplicarea unui preset sau a unei salvari

Comenzi Rust: `save_undo_snapshot`, `load_undo_snapshot`

### 85.6 Settings Diff View

Butonul **SHOW CHANGES** afiseaza un tabel cu toate modificarile nesalvate:
- Coloane: Parametru | Salvat | Curent
- Valorile modificate sunt highlight-ate (rosu/verde)
- Se poate inchide cu butonul CLOSE

### 85.7 Per-Symbol Overrides

Sectiune colapsabila care permite setarea de parametri custom per simbol:
- **Symbol** — se introduce simbolul (ex: AAPL, BTC/USD)
- **Trail%** — trailing stop custom
- **Take%** — take profit custom
- **Risk%** — risc pe trade custom
- **SET** — aplica override-ul
- **REMOVE** — sterge override-ul

Override-urile sunt persistate in `titan_settings.json` si respectate de autopilot in timpul trading-ului.

Comenzi Rust: `set_symbol_override`, `remove_symbol_override`, `get_symbol_overrides`

### 85.8 Live Impact Preview (Effective Exit Values)

Panel care arata valorile EFECTIVE ale stop/take/trail DUPA aplicarea multiplicatorilor adaptivi:
- **Stop Loss** — valoarea efectiva vs. baza
- **Take Profit** — valoarea efectiva vs. baza
- **Trailing** — valoarea efectiva vs. baza

Se actualizeaza automat cand se modifica slider-ele (debounced).

Comanda Rust: `preview_effective_exits`

### 85.9 Regime Heatmap

Trei box-uri colorate care arata multiplicatorii pentru fiecare regim de piata:
- **TRENDING** — piata cu trend clar (sus/jos)
- **RANGE** — piata laterala, fara directie clara
- **CHAOTIC** — piata haotica, volatilitate extrema

Fiecare box arata: S (stop mult), T (take mult), Tr (trail mult). Regimul activ este highlight-at. Se actualizeaza la fiecare 5 secunde.

### 85.10 Parameter Profiles (RF-C: Disk Persistence)

Sistem de profile numite pentru salvarea/incarcarea configuratiilor complete. Profilele sunt stocate pe disk in `titan_profiles.json` (nu in localStorage) -- supravietuiesc rebuild-uri, crash-uri, si pot fi exportate/importate.

**Actiuni disponibile:**
- **Save As** — salveaza configuratia curenta (TOTI parametrii inclusiv TA Gates, Scan Params, Symbol Overrides) sub un nume ales
- **Load** — incarca un profil salvat si aplica TOATE setarile (inclusiv obiecte nested)
- **Delete** — sterge un profil (cu confirmare cyber)
- **Export** — exporta profilul selectat ca fisier JSON (pentru backup sau sharing)
- **Import** — importa un profil dintr-un fisier JSON (numele se preia din fisier sau din campul de text)
- Dropdown cu toate profilele salvate, sortat alfabetic

**Storage:** `titan_profiles.json` in folderul de date al aplicatiei, cu scriere atomica (temp+rename) si auto-backup `.bak`. Schema versioning (`schema_v: 1`) pentru compatibilitate cu viitoarele update-uri.

**Migrare automata:** Daca exista profile vechi in localStorage (din versiunile anterioare), sunt importate automat pe disk la prima rulare si cheia veche este stearsa.

### 85.11 Risk Calculator Live

Panel cu 4 metrici de risc calculate in timp real:
- **Max Loss / Trade** — pierderea maxima per tranzactie in dolari
- **Position Size** — marimea pozitiei in dolari
- **Risk/Reward** — raportul risc/recompensa
- **Max Daily Loss** — pierderea maxima zilnica in dolari

Se actualizeaza automat cand se modifica slider-ele.

### 85.12 Parameter Health

Sistem de diagnostic cu 15 reguli de conflict/risc:
- Badge-uri colorate: **OK** (verde), **WARN** (portocaliu), **DANGER** (violet)
- Verifica: stop > take (ineficient), risc prea mare, Circuit Breaker prea slab, etc.
- Se actualizeaza in timp real la orice modificare de slider

### 85.13 Auto-Suggest

Butonul **ANALYZE TRADES** analizeaza ultimele 50 de trade-uri din jurnal si genereaza sugestii:
- Detecteaza 7 pattern-uri (stop-uri prea stranse, trailing ineficient, etc.)
- Fiecare sugestie are buton **APPLY** care aplica automat ajustarea
- Se poate inchide cu butonul **CLOSE**

### 85.14 Adaptive Exit Multipliers

Sectiune avansata (colapsabila) cu 18 slidere + toggle + floor care controleaza multiplicatorii adaptivi ai iesirilor:

**Toggle: DISABLE ADAPTIVE EXITS** — cand ON, toti multiplicatorii sunt ignorati si se folosesc doar valorile brute din slider-ele principale.

**Min Exit Floor %** — valoarea minima la care pot fi comprimate stop/take/trail dupa cascadare.

**Regime Multipliers** (9 slidere) — multiplicatori separati pt Stop/Take/Trail in fiecare regim (Chaotic, Range, Trend).

**Signal Alignment Multipliers** (5 slidere) — multiplicatori pt cand semnalele sunt aliniate sau opuse pozitiei.

**Timeframe Multipliers** (3 slidere) — multiplicatori pentru timeframe-uri scurte (1m/3m).

### 85.15 Multi-Asset Control

- **Toggle** — activeaza/dezactiveaza trading-ul simultan pe mai multe asset-uri
- **Max Concurrent Assets** — slider (1-100) care limiteaza numarul de asset-uri pe care botul le poate tranzactiona simultan

### 85.16 Chart Persistence pe Disk

Setarile graficului (timeframe, indicatori, configuratii) se salveaza acum pe disk:
- **Dual save**: `localStorage` (pt toggle rapid intre taburi) + disk (`titan_chart_settings.json`)
- La startup, graficul incarca de pe disk cu `await invoke('load_chart_settings')`
- Supravietuieste WebView cache clear (din build.sh)

Comenzi Rust: `save_chart_settings`, `load_chart_settings`

### 85.17 Styling Complet Refactorizat

- Slidere unificate cyan in tot tab-4 (eliminat inconsistenta galben/cyan)
- Input-uri number/text/select cu fundal dark (eliminat fundalul alb)
- Text sizing extins la TOATE sectiunile
- Dark scrollbar, consistent borders, hover effects pe sectiuni
- Health badges cu culori violet (danger) si galben-portocaliu (warn)
- `.btn-cmd` fortat la `color: #ccc` in tab-4 (eliminat text negru pe fundal dark)
- LLM Sentiment Provider dropdown fixat (fundal dark)
- Butoane SAVE CONFIG/RESET DEFAULTS duplicate eliminate

### 85.18 Manual dedicat HFT Engine

A fost creat un manual de utilizare separat, dedicat exclusiv tab-ului HFT Engine:
- **Fisier:** `TITAN_HFT_ENGINE_MANUAL.md`
- Explicatii detaliate pentru FIECARE element din tab
- Exemple numerice concrete
- Valori default, domenii, recomandari
- Scris pentru incepatori

---

## 86. CYBER-HUD + COLOANE ROSS CAMERON + FMP ENRICHMENT (Sesiunea 20)

### 86.1 Cyber-HUD Styling

Toate cele 3 tabele din panoul central (HFT Targets, Signal CFG, Watchlist) au primit tema Cyber-HUD:
- Font: **Share Tech Mono 13px** (marit de la 10-11px pentru lizibilitate)
- Badge-uri de actiuni: `.tbl-badge` cu variante gold/cyan/magenta/red
- Color-coding consistent pe coloane:
  - `CHG%`: cyan ≥10%, gold ≥0%, rosu <0%
  - `RVOL`: cyan ≥5x, gold ≥2x
  - `GAP%`: cyan >3%, gold >0%
  - `FLOAT`: violet=LOW, gold=MED
  - `SCORE`: cyan ≥60, gold ≥30
  - `INS`: cyan=BUY, rosu=SELL
  - `CAP`: format automat (150M / 1.2B)

### 86.2 Coloane Ross Cameron Complete

Setul complet de coloane este acum disponibil pe toate cele 3 tabele:

| Coloana | Sursa | Descriere |
|---------|-------|----------|
| GAP% | Alpaca snapshots | Gap-ul de la open vs prev close |
| FLOAT | Estimat din volum | LOW (<3M vol), MED (3-10M), HIGH (>10M) |
| CAP | FMP /stable/profile | Market cap in milioane (ex: 22M, 1.2B) |
| INS | FMP /stable/insider-trading | Ultima activitate insider: BUY, SELL, sau — |
| SCORE | AI scoring pipeline | Smart score sau AI score compozit |

### 86.3 FMP Enrichment Universal

Inainte de aceasta sesiune, datele FMP (market cap, insider signal) se populau **doar** prin scanarea Market Discovery.

Acum, FMP enrichment este integrat in **toate** pipeline-urile de date:
- **HFT Targets → REFRESH** → fetch FMP profile + insider
- **Signal CFG → REFRESH** → fetch FMP profile + insider
- **Watchlist → REFRESH** → fetch FMP profile + insider

Datele FMP sunt **cache-uite** — nu se re-fetch la fiecare refresh daca sunt deja in cache.

Daca FMP API key nu este setat (in Settings → API Keys), coloanele CAP si INS vor arata `—` (fallback graceful).

### 86.4 Cross-Module Refresh

La injectarea simbolurilor din Market Discovery:
- Signal CFG se actualizeaza automat (`window.loadSignalAssets()`)
- HFT Targets se actualizeaza automat (`refreshHftTargets()`)
- Nu mai e necesar refresh manual dupa inject

### 86.5 Fisiere Rust Modificate

- `scanner.rs` — `AiScanCandidate` struct extins cu `market_cap` si `insider_signal`
- `commands/favorites.rs` — `WatchlistEntry` extins cu `market_cap_m`, `insider_signal`, `float_category`
- `commands/hft_targets.rs` — FMP enrichment adaugat
- `commands/signal.rs` — FMP enrichment adaugat
- `commands/scoring.rs` — campuri noi pe construct

---

## 87. PHASE 9 FINAL UPGRADES — POSITION FSM + LATENCY METRICS + WEBSOCKET DASHBOARD + SESSION ANALYTICS (Sesiunea 21)

In sesiunea 21 s-au implementat cele 4 sub-faze ale Phase 9 (reordonate din planul original): Position State Machine, Micro-Latency Metrics, WebSocket Dashboard si Session Analytics improvements. S-au remediat si 3 erori clippy post-implementare.

### 87.1 Position State Machine (FSM)

Pozitiile aveau un lifecycle implicit — starea lor era dedusa din context (prezenta/absenta in HashMap). Informatii precum momentul deschiderii, durata hold, partial fills si motivul intrarii erau dispersate in HashMap-uri separate pe TitanState.

**Enum `PositionState`:**

| Stare | Semnificatie |
|-------|-------------|
| `PendingEntry` | Ordin trimis, asteptam fill |
| `Open` | Pozitie activa (default) |
| `PartialExit` | Partial take executat |
| `PendingClose` | Ordin de inchidere trimis |
| `Closed` | Pozitie inchisa complet |

**Campuri noi pe `Position` struct:**

| Camp | Tip | Default | Descriere |
|------|-----|---------|-----------|
| `state` | `PositionState` | `Open` | Starea curenta FSM |
| `opened_at` | `Option<u64>` | `None` | Epoch seconds la deschidere |
| `closed_at` | `Option<u64>` | `None` | Epoch seconds la inchidere |
| `partial_taken` | `bool` | `false` | Daca s-a executat partial take |
| `entry_tag` | `Option<String>` | `None` | Motivul intrarii (ex: "momentum_entry") |

Toate campurile au `#[serde(default)]` — starea salvata anterior (fara aceste campuri) se deserializeaza corect.

**Tranzitii FSM:**
- **`autopilot.rs` entry** → `state: Open`, `opened_at: now`, `entry_tag: entry_type`
- **`trade_updates.rs` fill** → `state: Open`, `opened_at: now` (daca era `PendingEntry`)
- **`trade_updates.rs` partial adverse fill** → `state: PartialExit`, `partial_taken: true`
- **`trade_updates.rs` close fill** → `state: Closed`, `closed_at: now`
- **`autopilot.rs` exit** → `state: Closed`, `closed_at: now`
- **`trading.rs` liquidate/force-close** → `state: PendingClose`, `closed_at: now`
- **`position_recon.rs` merge** → preserva `state`, `opened_at`, `entry_tag`, `partial_taken` din pozitia locala

### 87.2 Micro-Latency Metrics

Latenta API-urilor (order round-trip, GPU dispatch, streamer tick processing) era doar logata cu `tracing::info!`. Nu exista agregare sau percentile.

**Structura `LatencyMetrics`:**
- 3 ring buffers de 100 intrari fiecare (`Mutex<VecDeque<u64>>`):
  - `order_rtt_us` — round-trip API Alpaca (microsecunde)
  - `gpu_dispatch_us` — timp dispatch GPU/WGPU (microsecunde)
  - `streamer_tick_us` — procesare mesaj WebSocket SIP (microsecunde)

**Metode:**
- `record_order(us)`, `record_gpu(us)`, `record_streamer(us)` — push in ring buffer (drop cel mai vechi daca >100)
- `snapshot() -> LatencySnapshot` — calculeaza percentile pe toate 3 bucket-urile

**Structura `LatencySnapshot`:**

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

**Comanda Tauri:** `get_latency_metrics` — returneaza `LatencySnapshot` ca JSON pentru frontend.

**Puncte de inregistrare:**
- `commands/order_exec.rs` — dupa fiecare API call Alpaca
- `loops/nexus_loop.rs` si `commands/misc.rs` — dupa GPU dispatch
- `loops/trade_updates.rs` — in hot loop-ul de procesare mesaje WebSocket

### 87.3 WebSocket Dashboard

Dashboard-ul web (axum, port 7777) folosea `<meta http-equiv="refresh" content="5">` pentru actualizare — nepotrivit pentru monitorizare in timp real.

**Implementare (actualizat S02 — Sesiunea 45):**
- Endpoint `GET /ws` cu `WebSocketUpgrade` (axum 0.7, feature `ws`)
- Autentificare prin `Authorization: Bearer <token>` pe HTTP upgrade request (unificat cu API-ul HTTP — S02)
- `tokio::sync::broadcast::Sender<String>` pe `TitanState` (256 slots)
- Push **status** la fiecare 2 secunde: equity, positions cu FSM state, circuit breaker status, latency p50/p95
- Push **trade events** de la autopilot: entry, exit (cu PnL si duration), partial exit
- Push **alert events** de la watchdog: loop recovery notifications
- Client JavaScript inline cu auto-reconnect (exponential backoff 1s → 30s max)
- WS message rate limit: max 10 mesaje client per tick, disconnect la depasire (S02)
- Eliminat complet `<meta http-equiv="refresh">` — 0 polling, 100% push

**Tipuri de mesaje WebSocket (JSON):**

| type | Cand | Campuri |
|------|------|---------|
| `status` | La 2 secunde | equity, cash, positions[], breaker, latency |
| `trade` | La entry/exit/partial | action, symbol, side, pnl, duration_secs |
| `alert` | La watchdog recovery | message |

### 87.4 Session Analytics Improvements

Jurnalul inregistra durata trade-urilor ca `duration_secs: 0`. Partial exits nu erau jurnalizate deloc. Nu existau streak counters.

**Campuri noi pe `SessionAnalytics` (crate `titan-risk`):**

| Camp | Tip | Descriere |
|------|-----|-----------|
| `longest_win_streak` | `u32` | Cel mai lung sir de wins consecutivi |
| `longest_lose_streak` | `u32` | Cel mai lung sir de losses consecutive |
| `current_streak` | `i32` | Streak curent (+ = wins, - = losses) |
| `avg_duration_secs` | `f64` | Durata medie a trade-urilor (rolling) |
| `best_trade_pnl` | `Decimal` | Cel mai mare profit dintr-un trade |
| `worst_trade_pnl` | `Decimal` | Cea mai mare pierdere dintr-un trade |

**Metoda `record_trade_with_duration(pnl, duration_secs)`:**
- Actualizeaza streak counters la fiecare trade: incrementeaza `current_streak` (+1 win / -1 loss), reseteaza la schimbare directie
- Actualizeaza `longest_win_streak` / `longest_lose_streak` la noua inregistrare
- Actualizeaza `best_trade_pnl` / `worst_trade_pnl`
- Calculeaza `avg_duration_secs` ca rolling average

**Partial exit journaling:**
- Autopilot-ul creeaza acum o intrare `TradeRecordInput` cu `reason: "PARTIAL_TAKE"` pentru fiecare partial exit
- Jurnalizata complet cu symbol, side, qty, pnl, duration

**Expus in API:**
- `get_risk_status` returneaza campurile noi
- Dashboard WebSocket include streak-urile in status push

### 87.5 Clippy Fixes Post-Implementare

3 erori clippy detectate de `build.sh` (care ruleaza `cargo clippy -- -D warnings`):

1. **`derivable_impls`** in `app_state.rs` — `impl Default for PositionState` manual inlocuit cu `#[derive(Default)]` + `#[default]` pe varianta `Open`
2. **`dead_code`** in `app_state.rs` — `Position::new_open()` marcat cu `#[allow(dead_code)]` (API public, va fi utilizat in viitor)
3. **`bind_instead_of_map`** in `state.rs` — `.and_then(|x| Some(y))` inlocuit cu `.map(|x| y)` pe calculul `genetic_champion`

### 87.6 Fisiere Modificate — Sumar Complet

**Backend Rust (12 fisiere):**
- `app_state.rs` — `PositionState` enum, campuri noi pe Position, `new_open()`, derives
- `state.rs` — `LatencyMetrics`, `LatencySnapshot`, `LatencyBucket`, `ws_broadcast`, fix `.map`
- `trading_engine.rs` — FSM transitions la manage_positions
- `loops/trade_updates.rs` — FSM transitions la fills, latency recording
- `loops/position_recon.rs` — FSM state preservation la reconciliere
- `loops/autopilot.rs` — FSM reads/writes, WS broadcast, journal partial exits, `record_trade_with_duration`
- `loops/watchdog_loop.rs` — WS broadcast alert events
- `commands/trading.rs` — FSM `PendingClose` la liquidate/force-close
- `commands/order_exec.rs` — latency recording
- `commands/misc.rs` — GPU latency recording
- `commands/monitoring.rs` — `get_latency_metrics` comanda, campuri noi in `get_risk_status`
- `web_dashboard.rs` — `/ws` endpoint, WS handler, JS client, eliminat refresh polling
- `main.rs` — initializare `LatencyMetrics`, `ws_broadcast`, comanda noua registered
- `Cargo.toml` — `axum = { version = "0.7", features = ["ws"] }`
- `crates/titan-risk/src/lib.rs` — campuri noi SessionAnalytics, `record_trade_with_duration()`

### 87.7 Validari

- `cargo check`: 0 erori
- `cargo clippy -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 122 teste passed
- Build release: 15M binary
- Aplicatia porneste si functioneaza corect

---

## Cap. 88 — Sesiunea 23: FSM Status Column, Latency Panel, Rate Limiting, Watchdog Fix

**Data:** 29 Martie 2026 (Sesiunea 23)

Aceasta sesiune a adaugat vizibilitate in UI pentru starea pozitiilor si latenta, a securizat web dashboard-ul cu CORS si rate limiting, si a fixat un gap critic in watchdog.

### 88.1 Position FSM Status Column (RF-A1)

Tabelul de pozitii din sectiunea "PORTOFOLIU ACTUAL" are acum o coloana **STATUS** intre TIP si MODE care arata starea FSM (Finite State Machine) a fiecarei pozitii.

**5 stari posibile cu badge-uri colorate:**

| Stare | Badge | Culoare | Cand apare |
|-------|-------|---------|------------|
| `PendingEntry` | PENDING | Galben cu pulse animation | Ordinul de intrare trimis, asteptam fill |
| `Open` | OPEN | Cyan | Pozitia este activa si deschisa |
| `PartialExit` | PARTIAL | Magenta | S-a executat un partial take profit |
| `PendingClose` | CLOSING | Rosu cu pulse animation | Ordinul de inchidere trimis |
| `Closed` | CLOSED | Gri | Pozitia s-a inchis (rar vizibil, tranzient) |

**Cum ajunge starea in frontend:**
- `PositionView` (structura trimisa de `get_account_snapshot`) a fost extins cu campul `fsm_state: String`
- La fiecare refresh, backend-ul face lookup in `settings.positions` pentru FSM state-ul intern al fiecarui simbol
- Frontend-ul mapeaza string-ul la badge CSS corespunzator

**De ce e util:** Inainte nu aveai vizibilitate daca un ordin era in curs de executie sau daca s-a facut deja un partial exit. Acum vezi instant starea reala.

### 88.2 Latency Metrics Panel (RF-A2)

In rightbar-ul chartului (sectiunea din dreapta, sub "Time"), a aparut o sectiune noua **"Latency (us)"** care arata performanta in timp real a celor 3 canale critice.

**Grid 4x4 cu 3 canale x 3 percentile:**

| Canal | Ce masoara | Unde se inregistreaza |
|-------|------------|----------------------|
| **Order** | Round-trip time al ordinelor Alpaca API | `commands/order_exec.rs` |
| **GPU** | Durata dispatch GPU Nexus | `loops/nexus_loop.rs` |
| **Stream** | Procesare mesaje WebSocket streamer | `streamer.rs` |

**Percentile:** p50 (median), p95 (95th percentile), p99 (99th percentile)

**Color coding automat:**
- **Cyan** (verde) — sub 1ms (1000us) — totul OK
- **Galben** — 1ms-5ms — performanta acceptabila dar de monitorizat
- **Rosu** — peste 5ms — probleme de latenta, investigheaza

**Format inteligent:** Valorile se afiseaza ca `us` (microsecunde), `ms` (milisecunde), sau `s` (secunde) in functie de magnitudine.

**Polling:** Datele se actualizeaza automat la fiecare 5 secunde via `invoke('get_latency_metrics')`.

**Backend-ul era deja implementat** (sesiunea 21 — `LatencyMetrics` cu ring buffer de 100 intrari pe canal). Aceasta sesiune a adaugat doar frontend-ul.

### 88.3 Rate Limiting + CORS (Phase 1 Security)

Web dashboard-ul si Telegram au primit protectii de securitate.

**Web Dashboard — CORS (actualizat S02 — Sesiunea 45):**
- `tower_http::cors::CorsLayer` restrictiv adaugat pe router-ul axum
- **Allow Origin:** `http://127.0.0.1:<port>` si `http://localhost:<port>` (include portul — fix S02)
- **Allow Methods:** GET si POST
- **Allow Headers:** Authorization si Content-Type
- Un site malitios deschis in browser nu mai poate face fetch la `http://127.0.0.1:7777/api/status`

**Web Dashboard — Rate Limiting (actualizat S02 — Sesiunea 45):**
- Sliding window de 60 cereri per minut pe toate rutele protejate, inclusiv `/ws` upgrade (fix S02)
- Middleware custom cu `Arc<Mutex<VecDeque<Instant>>>`
- La depasire, returneaza HTTP 429 (Too Many Requests) cu JSON error
- WS message-level rate limit: max 10 mesaje client per tick (2s), disconnect la depasire (S02)

**Telegram — Rate Limiting:**
- Global sliding window de 20 mesaje per minut pe `send_telegram_alert`
- `LazyLock<Mutex<VecDeque<Instant>>>` — static, shared intre toate thread-urile
- La depasire, mesajul e dropped silentios cu `tracing::warn`
- Previne flood-area Telegram API in situatii de cascading alerts (ex: 10 pozitii inchise simultan)

### 88.4 Watchdog Streamer Spawn Fix (Phase 2)

**Problema:** Streamerul era registered in watchdog (`heartbeat("streamer", 0)`) dar NU exista in `spawn_loop_by_name`. Daca task-ul de streamer facea panic sau se oprea, watchdog-ul il detecta ca mort dar **nu-l putea reporni**.

**Fix:** Adaugat branch `"streamer"` in `spawn_loop_by_name`:
- Creeaza channel nou (`tokio::sync::mpsc::unbounded_channel::<StreamCmd>()`)
- Inlocuieste `stream_tx` in state cu noul sender
- Spawneaza `run_streamer` cu noul receiver

**De ce e important:** Streamerul e componenta cea mai critica — fara el, nu ai date de pret in timp real. Daca cadea si nu era restartat, tot botul devenea "orb".

### 88.5 N1 Cleanup — println! Reziduale

Ultimele 2 `println!` din production code (din `commands/settings.rs`) au fost migrate la `tracing::info!`:
- Linia 390: `fmp_api_key` update log — acum `tracing::info!(component = "settings", ...)`
- Linia 433: `batch_update_settings` count — acum `tracing::info!(component = "settings", count = count, ...)`

Zero `println!` raman in production code. Toate log-urile trec prin `tracing` (structured logging cu componente, filtrare pe nivel).

### 88.6 Fisiere Modificate — Sumar Complet

**Backend Rust (5 fisiere):**
- `commands/settings.rs` — N1 cleanup: println -> tracing::info
- `commands/account.rs` — `PositionView` + `fsm_state` field, lookup din FSM intern
- `web_dashboard.rs` — CORS layer, rate limiting middleware (sliding window 60 req/min)
- `telegram.rs` — send rate limiter (20 msg/min) cu LazyLock
- `loops/watchdog_loop.rs` — branch "streamer" in spawn_loop_by_name cu channel recreation

**Frontend (4 fisiere):**
- `titan_positions.js` — FSM badge rendering (map state -> label + CSS class)
- `titan_rightbar.js` — latency panel: polling, formatting, color coding
- `index.html` — STATUS column header, latency grid HTML section
- `styles.css` — FSM badge classes (5 stari), latency grid styles, color states

### 88.7 Validari

- `cargo check`: 0 erori
- `cargo clippy -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 122 teste passed (37 risk + 30 indicators + 21 genetic + 29 main + 5 integration)
- Linter errors: 0
- FSM Status Column: verificat vizual — badge-uri OPEN cyan afisate corect

---

## Cap. 89 — Sesiunea 24: Ring Buffer + ArcSwap Lock-Free + Flash Crash Detector

Sesiunea 24 a implementat 3 componente majore din BLOC 2 al planului de audit, focusate pe **performanta hot-path** si **protectie capital in evenimente extreme**.

### 89.1 S1: CandleRingBuffer — Market Data In-Memory

**Problema:** Autopilotul apela `fetch_bars_internal` (REST HTTP catre Alpaca) la fiecare iteratie, pentru fiecare simbol candidat. Aceasta insemna 50-500ms latenta per apel, multiplicat cu numarul de simboluri.

**Solutia:** `CandleRingBuffer` — buffer circular in memorie per simbol, alimentat direct din WebSocket.

**Fisier nou:** `src-tauri/src/candle_buffer.rs`

**Arhitectura:**

```rust
pub struct CandleRingBuffer {
    candles: VecDeque<Candle>,   // max 200 candles finalizate
    current: Option<Candle>,     // candle in constructie (nu inca inchisa)
    tf_secs: i64,                // durata timeframe-ului in secunde
}
```

**Flux date:**
1. Streamerul primeste trade tick din WebSocket (pret, size, timestamp)
2. Apeleaza `ring_buffer.update_trade(price, size, ts)`:
   - Daca `current` este `None` sau timestamp-ul depaseste fereastra curenta → finalizaza candle-ul curent, il muta in `candles`, creaza unul nou
   - Altfel actualizeaza OHLCV in-place (high = max, low = min, close = price, volume += size)
3. Cand `candles.len() > 200` → evict-uieste cea mai veche (front pop)

**Integrare autopilot — `get_candles()` helper:**

```rust
async fn get_candles(state: &Arc<TitanState>, symbol: &str, timeframe: &str, min_candles: usize)
    -> Result<Vec<Candle>, TitanError>
```

- **Fast path:** Cauta in `state.candle_buffers` cheia `"SYMBOL|TF"`. Daca are >= `min_candles` → returneaza snapshot-ul direct (~0ms)
- **Fallback:** Daca bufferul nu exista sau are prea putine candles → apeleaza `fetch_bars_internal` (REST) si seed-uieste rezultatul in buffer pentru apelurile urmatoare

Toate cele 7 apeluri `fetch_bars_internal` din autopilot au fost redirectate prin `get_candles()`.

**Utilitar `timeframe_to_secs()`:** Converteste stringuri ("1m", "1Min", "5m", "5Min", "15m", "1h", "1Hour", "1d", "1Day") in secunde.

**Teste unitare (7 noi):**
- seed + snapshot correctness
- OHLCV update in-place (high/low/close/volume)
- boundary cross (candle finalizare + new candle)
- eviction la max capacity (200)
- sort ascending dupa timestamp
- zero-price trade ignorat
- timeframe string conversion

**Impact performanta:** Hot path de la 50-500ms (REST) la <1ms (memory read).

### 89.2 S3: ArcSwap Lock-Free pe OfiState

**Problema:** `ofi_state` era protejat de `Arc<Mutex<OfiState>>`. Streamerul (write la fiecare tick) si autopilotul (read la fiecare iteratie) se bateau pe acelasi lock, creand contention pe cel mai latency-sensitive path.

**Solutia:** `arc_swap::ArcSwap<OfiState>` — atomic swap lock-free.

**Inainte:**
```rust
pub ofi_state: Arc<Mutex<OfiState>>,
// Write: state.ofi_state.lock().unwrap().current_ofi = val;
// Read:  let ofi = state.ofi_state.lock().unwrap().clone();
```

**Dupa:**
```rust
pub ofi_state: arc_swap::ArcSwap<OfiState>,
// Write: let prev = state.ofi_state.load();
//        let mut st = (*prev).clone();
//        st.current_ofi = val;
//        state.ofi_state.store(Arc::new(st));
// Read:  let ofi = state.ofi_state.load();  // Guard<Arc<OfiState>>, no lock
```

**Write paths actualizate (streamer.rs):**
1. **Book-path OFI** — cand primeste L2 deltas din WebSocket, recalculeaza OFI si face atomic store
2. **Quote-path OFI** — cand primeste NBBO quotes, actualizeaza best_bid/ask/spread si face atomic store

**Read paths actualizate (3 locatii):**
1. `autopilot.rs` — spread guard + OFI stale check
2. `commands/order_exec.rs` — smart limit pricing (midpoint calculation)
3. `commands/strategy_cmd.rs` — `get_ofi_status` command pt frontend

**Abordare conservatoare:** Migrat DOAR `ofi_state` deoarece are pattern clar single-writer (streamerul). `last_prices`, `trail_*` si altele raman `Mutex` (multi-writer patterns).

**Impact:** Zero contention pe read path; write-ul este O(1) atomic pointer swap.

### 89.3 S7: Flash Crash Detector cu Auto-Protection

**Problema:** Circuit breaker-ul existent masoara doar PnL-ul tau (pierderi din pozitii). Nu detecteaza miscari anormale ale pietei (flash crash-uri) care nu au lovit inca stop-ul.

**Solutia:** `FlashCrashDetector` — monitorizare rolling window a preturilor per simbol.

```rust
pub struct FlashCrashDetector {
    windows: HashMap<String, VecDeque<(Instant, f64)>>,  // preturi recente per simbol
    blocked_until: HashMap<String, Instant>,               // cooldown per simbol
    window_duration: Duration,   // fereastra de monitorizare (default 30s)
    threshold_pct: f64,          // prag de declansare (default 3.0%)
    cooldown: Duration,          // blocare post-trigger (default 5min)
}
```

**Logica detectie:**
1. La fiecare trade tick, pretul este adaugat in fereastra simbolului
2. Se curata preturile mai vechi de `window_duration`
3. Se calculeaza `min` si `max` din fereastra
4. Daca `(max - min) / min * 100 > threshold_pct` → FLASH CRASH DETECTAT

**5 actiuni automate la declansare:**
1. `tracing::warn!(component = "flash_crash", symbol, drop_pct, ...)` — log structured
2. `app.emit_all("flash-crash", { symbol, drop_pct, action })` — event catre frontend
3. Auto-close pozitia pe simbolul afectat — spawn `close_position(state, symbol, qty, side, app)`
4. `send_telegram_alert(...)` — notificare Telegram (daca enabled)
5. Block intrari noi pe simbol timp de `cooldown_mins` minute

**3 parametri configurabili din UI (AppState + AppSettings):**

| Parametru | Default | Range | Ce face |
|-----------|---------|-------|---------|
| `flash_crash_window_secs` | 30 | 5-120 | Fereastra de monitorizare in secunde |
| `flash_crash_threshold_pct` | 3.0 | 0.5-20.0 | Pragul procentual de declansare |
| `flash_crash_cooldown_mins` | 5 | 1-60 | Cate minute blocheaza simbolul post-trigger |

Toti 3 au `#[serde(default)]` pentru backward compatibility cu settings salvate anterior.

**Integrare autopilot:** La inceputul evaluarii fiecarui simbol candidat, autopilotul verifica `state.flash_crash.lock().is_blocked(candidate)` si face `continue` daca simbolul este blocat.

**Teste unitare (6 noi):**
- normal trades sub threshold → no event
- crash detection cu price drop > threshold
- spike up detection (symmetric — detecteaza si cresteri anormale)
- cooldown enforced post-trigger
- window expiry (preturile vechi ies din fereastra)
- clear() reseteaza tot

### 89.4 Fisiere Modificate — Sumar Complet

**Fisier nou (1):**
- `src-tauri/src/candle_buffer.rs` — CandleRingBuffer + FlashCrashDetector + timeframe_to_secs + 13 teste unitare

**Backend Rust (8 fisiere modificate):**
- `main.rs` — `mod candle_buffer`, initializare `candle_buffers` si `flash_crash` pe TitanState
- `state.rs` — import CandleRingBuffer/FlashCrashDetector, campuri noi pe TitanState, 3 flash crash fields pe AppSettings + defaults + sync functions, ofi_state migrat la ArcSwap
- `app_state.rs` — 3 flash crash fields pe AppState + defaults
- `streamer.rs` — candle buffer update pe trade, flash crash check pe trade (auto-close + telegram), ArcSwap writes pe ambele OFI paths
- `loops/autopilot.rs` — `get_candles()` helper, 7 redirectari, ArcSwap read, flash crash block gate
- `commands/order_exec.rs` — ArcSwap read pe OFI
- `commands/strategy_cmd.rs` — ArcSwap read pe OFI
- `commands/settings.rs` — 3 flash crash settings handlers cu clamping

### 89.5 Validari

- `cargo check`: 0 erori
- `cargo clippy -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 135 teste passed (37 risk + 30 indicators + 21 genetic + 42 main + 5 integration)
- 13 teste noi (7 candle_buffer + 6 flash_crash), 0 failed
- Linter errors: 0

---

## Cap. 90 — Sesiunea 25: Drawdown Velocity Detection + Keyboard Command Palette

### 90.1 S8: Intraday Drawdown Velocity Detection

**Problema rezolvata:** Circuit breaker-ul masoara magnitudinea pierderii (daily PnL < threshold). Flash Crash Detector-ul masoara miscarea pietei. Dar niciuna nu reactioneaza la **viteza** pierderii. Scenariul periculos: ai -0.8% daily PnL (sub halt threshold de -1%), dar in ultimele 3 minute ai pierdut 0.6% — cascading losses in desfasurare. Circuit breaker-ul nu se declanseaza inca, dar viteza arata ca ceva e gresit fundamental.

**Solutia:** Ring buffer de snapshot-uri drawdown cu detectie de velocitate.

**Cum functioneaza:**
1. La fiecare ciclu autopilot (~1.5s), se inregistreaza un snapshot `(epoch_secs, drawdown_pct)` in ring buffer-ul de pe `SessionAnalytics`
2. Ring buffer-ul retine maxim 120 de intrari (ultimele ~3 minute la refresh normal)
3. Se calculeaza `dd_velocity` = schimbarea drawdown-ului in %/min pe fereastra configurata
4. Daca velocitatea depaseste threshold-ul (default -1.0 %/min), se blocheaza intrarile noi — **independent** de circuit breaker
5. Log message periodic in UI: "DD VELOCITY HALT — Drawdown speed X.XX%/min exceeds threshold"

**Setari configurabile (HFT Engine tab):**

| Parametru | Default | Domeniu | Descriere |
|-----------|---------|---------|-----------|
| `dd_velocity_threshold` | -1.0 | -10.0 ... -0.1 | Viteza maxima de drawdown in %/min inainte de halt |
| `dd_velocity_window_secs` | 300 | 30 ... 600 | Fereastra de lookback in secunde |

**Diferenta fata de Circuit Breaker:**

| Aspect | Circuit Breaker | DD Velocity |
|--------|----------------|-------------|
| Masoara | Magnitudinea (daily PnL %) | Viteza de pierdere (%/min) |
| Trigger | PnL sub prag absolut | Rata de pierdere peste prag |
| Cand ajuta | Pierderi cumulate mari | Pierderi rapide in cascada |
| Recovery | Manual reset | Automat cand viteza scade |

**Exemplu concret:** Equity $100,000. In 3 minute pierzi 5 trade-uri consecutive, equity scade la $97,000. Drawdown = 3%. Velocitate = -1.0%/min. Threshold default (-1.0) e atins → intrari blocate INAINTE ca circuit breaker-ul sa se activeze la -5%.

**Expus in API:** `get_risk_status` returneaza campul `dd_velocity` (f64, negativ cand drawdown se adanceste).

**Teste:** 7 teste unitare — no_data, stable_equity, cascading_loss, safe_threshold, ring_eviction, window_secs, positive_threshold.

### 90.2 S11: Keyboard Shortcut Command Palette

**Problema rezolvata:** In scalping activ, fiecare secunda conteaza. Navigarea cu mouse-ul prin taburi, schimbarea timeframe-ului sau plasarea rapida a ordinelor consuma timp pretios.

**Solutia:** Modul de keyboard shortcuts global cu command palette integrat (Ctrl+K).

**Shortcut-uri directe (functioneaza oricand):**

| Shortcut | Actiune |
|----------|---------|
| `Ctrl+K` | Deschide Command Palette |
| `Ctrl+Enter` | Quick Buy (market order pe simbolul curent) |
| `Ctrl+Shift+Enter` | Quick Sell (market order pe simbolul curent) |
| `Ctrl+Shift+X` | Flatten All — lichideaza toate pozitiile (emergency) |
| `Ctrl+P` | Toggle Autopilot on/off |
| `Ctrl+[` | Timeframe mai mic (ex: 5m → 3m → 1m) |
| `Ctrl+]` | Timeframe mai mare (ex: 5m → 10m → 15m) |

**Command Palette (Ctrl+K):**
- Overlay glassmorphism cu blur, font Share Tech Mono, styling neon cyan/green
- Input de cautare cu **fuzzy matching** pe 19 comenzi
- Navigare cu **Arrow Up/Down + Enter** sau click
- **Escape** inchide paleta
- Daca textul introdus nu corespunde niciunei comenzi, **Enter** il trateaza ca simbol si comuta chart-ul pe acel simbol
- Categorii: Trade (3), Control (1), Navigate (8), Chart (7)

**Comenzi disponibile in paleta:**

| Categorie | Comanda | Shortcut |
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

**Compatibilitate:** Shortcut-urile Ctrl+* nu interfereaza cu hotkey-urile de chart (L=linie, R=ray, H=hline etc.) care functioneaza doar fara Ctrl. Navigarea prin taburi cu cifre (1-9, 0) e disponibila DOAR prin command palette, nu direct ca hotkeys (evita conflict cu `0` = chart reset).

**Guard de input:** Toate shortcut-urile (exceptie Ctrl+K) se dezactiveaza cand focus-ul e pe un `<input>`, `<textarea>` sau element `contentEditable`. Ctrl+K functioneaza mereu — deschide paleta si muta focus-ul pe input-ul paletei.

### 90.3 Fisiere Modificate — Sumar Complet

**Fisier nou (1):**
- `src/titan_keyboard.js` — Keyboard shortcuts + Command Palette (230 linii)

**Backend Rust (6 fisiere modificate):**
- `crates/titan-risk/src/lib.rs` — dd_velocity_ring, record_dd_snapshot, compute_dd_velocity, check_dd_velocity_halt + 7 teste
- `app_state.rs` — dd_velocity_threshold, dd_velocity_window_secs fields + defaults
- `state.rs` — AppSettings fields, default functions, settings_from_state, apply_settings
- `commands/settings.rs` — handler-uri dd_velocity_threshold, dd_velocity_window_secs
- `commands/monitoring.rs` — dd_velocity in get_risk_status JSON
- `loops/autopilot.rs` — record_dd_snapshot la fiecare ciclu, dd_velocity_halt check, entries_allowed gate

**Frontend (2 fisiere modificate):**
- `index.html` — `<script src="titan_keyboard.js">`
- `styles.css` — Command Palette CSS (~100 linii)

### 90.4 Validari

- `cargo check`: 0 erori
- `cargo clippy -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 142 teste passed (44 risk + 30 indicators + 21 genetic + 42 main + 5 integration)
- 7 teste noi drawdown velocity, 0 failed
- BLOC 2 din plan master: complet (S1, S3, S7, S8, S11 + watchdog streamer)

---

## Cap. 91 — Sesiunea 26: TA Entry Gates cu Soft/Hard Mode (RF-B)

### 91.1 Ce sunt TA Entry Gates

TA Entry Gates sunt **6 filtre tehnice pre-entry** care se aplica DUPA ce Oracle-ul decide Buy/Short, dar INAINTE de a returna decizia finala de trading. Scopul lor este sa filtreze intrarile suboptime pe care Oracle-ul singur le scapa.

Fiecare gate are 3 moduri de functionare:

| Mod | Efect |
|-----|-------|
| **Off** | Gate-ul e ignorat complet (ca si cum nu exista) |
| **Soft** | Entry-ul e permis, dar scorul e penalizat cu N puncte (default 15). Daca scorul scade sub threshold, entry-ul tot nu se executa |
| **Hard** | Entry-ul e blocat complet — `check_entry()` returneaza `None`, indiferent de scorul Oracle |

### 91.2 Cele 6 Gate-uri

| Gate | Ce verifica | Cand blocheaza/penalizeaza |
|------|------------|---------------------------|
| **RSI** | RSI vs thresholds overbought/oversold | Long: RSI > `rsi_overbought` (default 70). Short: RSI < `rsi_oversold` (default 30) |
| **EMA Alignment** | Ordinea EMA9/20/50 | Long: trebuie EMA9 > EMA20 > EMA50. Short: EMA9 < EMA20 < EMA50. Daca nu sunt aliniate, gate-ul se activeaza |
| **ADX** | Forta trendului | ADX < `adx_trend_threshold` (default 25) — trend prea slab |
| **BB Squeeze** | Latimea benzilor Bollinger | Daca `(BB_upper - BB_lower) / BB_mid < 2%` — piata in consolidare, evita intrari |
| **MACD** | Directia histogramei MACD | Long: histograma trebuie > 0. Short: histograma trebuie < 0. Daca nu e aliniata cu directia trade-ului, gate-ul se activeaza |
| **Stoch RSI** | Extreme StochRSI | Long: StochRSI > 80 (deja supraachizitonat). Short: StochRSI < 20 (deja supravandut). Evita intrari cand indicatorul e in extrema opusa directiei |

### 91.3 Presets (Profile Predefinite)

| Preset | RSI | EMA | ADX | BB | MACD | StochRSI | Cand il folosesti |
|--------|-----|-----|-----|----|------|----------|-------------------|
| **Strict** | Hard | Hard | Hard | Hard | Hard | Hard | Calitate maxima, putine intrari. Bun pentru piete volatile |
| **Balanced** | Soft | Off | Soft | Off | Off | Off | Compromise: RSI si ADX filtreaza, dar nu blocheaza |
| **Oracle Only** | Off | Off | Off | Off | Off | Off | Default. Oracle-ul decide singur, fara filtre suplimentare |

### 91.4 Soft Penalty (Penalizarea Ajustabila)

Cand un gate in mod Soft e incalcat, scorul trade-ului scade cu `ta_gate_soft_penalty` puncte (default 15, configurabil 1-50 din UI).

Penalizarile se cumuleaza: daca 3 gate-uri Soft sunt incalcate simultan, scorul scade cu 45 puncte (3 x 15). Un scor suficient de mic poate face ca trade-ul sa nu treaca de threshold-ul de intrare, chiar daca nu e blocat explicit.

### 91.5 Fail-Open Design

Daca un indicator e `None` (nu sunt destule date — de exemplu, RSI e None la primele 14 candle-uri), gate-ul respectiv este **sarit complet**. Nu blocheaza si nu penalizeaza.

Motivul: la inceputul sesiunii de trading sau dupa un restart, bufferul de candle-uri e mic. Daca gate-urile ar bloca in lipsa datelor, ai pierde primele minute de market open — cele mai volatile si profitabile.

### 91.6 Locul in Pipeline

Pipeline-ul complet `check_entry()`:

```
Candle-uri → FVG Detection → Oracle Score → VPIN Gate → RVOL Gate
  → Regime Thresholds → Long/Short Decision
  → TA Gates (NOU) → Classical Bonus → Return TradeDecision
```

### 91.7 Configurare din UI

In tabul HFT Engine, sectiunea **"TA ENTRY GATES"** (cu fundal portocaliu) contine:
- **Preset dropdown** (dreapta sus): Custom / Strict / Balanced / Oracle Only
- **6 selectoare** Off/Soft/Hard — cate unul per gate
- **Slider Soft Penalty** — penalizarea in puncte per gate Soft incalcat (1-50)

Modificarile se aplica **live** (la schimbarea fiecarui select) si se salveaza automat pe disk.

### 91.8 Validari

- `cargo check`: 0 erori
- `cargo clippy -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 158 teste passed (44 risk + 30 indicators + 21 genetic + 58 main + 5 integration)
- 16 teste unitare noi TA gates, 0 failed, 0 regressions

---

## Cap. 92 — Sesiunea 27: StreamingIndicators — Calcul Incremental O(1) (S2+S4 Partea 1)

### 92.1 Ce este StreamingIndicators

`StreamingIndicators` este un modul intern in crate-ul `titan-indicators` care mentine starea fiecarui indicator tehnic si poate produce un snapshot complet de indicatori procesand un singur candle (O(1)), in loc de a recalcula toti indicatorii de la zero pe 200 candles (O(n)) la fiecare ciclu.

Aceasta este o optimizare interna. **Nu schimba niciun comportament vizibil** — aceiasi indicatori, aceleasi valori, aceeasi decizie de trading. Doar ca se calculeaza mai eficient.

### 92.2 De ce este necesar

Autopilotul apeleaza `calculate_all_with_periods()` la fiecare ciclu (500-1500ms), pentru fiecare simbol candidat. Aceasta functie recalculeaza de la zero:
- 4 serii EMA (9, 20, 50, 200)
- RSI (Wilder smoothing)
- MACD (3 EMA-uri interne)
- Bollinger Bands (sliding window + std dev)
- ADX (4 rolling means)
- VWAP (cumulativ)
- StochRSI (ring buffer + 2 SMA)
- Oracle 7-layer (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD)

Din toate aceste calcule, doar ULTIMA valoare conteaza (snapshot-ul curent). Primele 199 candle-uri sunt recalculate inutil. In plus, in 95%+ din cicluri candle-urile sunt identice cu ciclul anterior (un candle de 1 minut se schimba o data pe minut, dar autopilotul verifica de 60-120 ori).

### 92.3 Cum functioneaza

`StreamingIndicators` mentine state persistent per indicator:

| Indicator | State intern | Complexitate update |
|-----------|-------------|---------------------|
| EMA (x4) | Valoarea EMA anterioara + alpha precomputed | O(1) |
| RSI | avg_gain, avg_loss (Wilder smoothing) | O(1) |
| MACD | 3 EMA-uri interne (fast, slow, signal) | O(1) |
| BB | Sliding window VecDeque (20 valori) | O(period) ~20 ops |
| ADX | 4 sliding window SMA (TR, +DM, -DM, DX) | O(period) ~14 ops |
| VWAP | Sume cumulative (volum, volum*pret) | O(1) |
| StochRSI | Ring buffer RSI + 2 SMA(3) | O(period) ~14 ops |

**Decizie de design importanta:** BB si ADX folosesc sliding window (`VecDeque`) in loc de algoritmi online (Welford). Motivul: batch-ul existent foloseste sample variance `(N-1)` si rolling SMA. Sliding window garanteaza output identic la nivel de bit, fara riscul de erori de floating point cumulative.

### 92.4 API Public

| Metoda | Ce face |
|--------|---------|
| `StreamingIndicators::new(periods)` | Creeaza instanta goala, pregatita pentru `update()` |
| `StreamingIndicators::cold_start(candles, periods)` | Proceseaza un batch initial de candle-uri, initializeaza state-ul |
| `streaming.update(candle)` | Proceseaza un candle nou, returneaza `Indicators` snapshot |
| `streaming.candle_count()` | Numarul total de candle-uri procesate |

### 92.5 Validare numerica

Output-ul `StreamingIndicators` a fost validat contra batch-ului `calculate_all_with_periods()` cu proptest pe secvente random de 50-200 candle-uri. Toate valorile sunt identice la epsilon 1e-10 (0.0000000001).

### 92.6 Oracle

Oracle-ul (7-layer predictive super-indicator) NU a fost incrementalizat in sesiunea 26. `update()` returneaza `oracle_score = None`. Oracle se apeleaza batch doar la candle nou (o data per minut la TF 1m), nu la fiecare ciclu. **Nota:** Online Oracle Weight Adaptation (S15) a fost implementat in Sesiunea 38 — vezi Cap. 103. Aceasta adapteaza ponderile, nu calcul-ul incremental.

### 92.7 Ce urmeaza (Partea 2)

In sesiunea urmatoare se implementeaza `IndicatorCache` — un cache per simbol+timeframe care:
- La candle identic (95%+ din cicluri): returneaza cached, zero calcul
- La candle nou: apeleaza `streaming.update()` (O(1)) + Oracle batch
- La prima apelare: `cold_start()` din batch-ul complet

Reducerea estimata: ~95-99% din overhead-ul de calcul al indicatorilor.

### 92.8 Fisiere modificate

Un singur fisier: `src-tauri/crates/titan-indicators/src/indicators.rs`

Zero modificari in codul principal (`src-tauri/src/`). Implementare complet izolata in crate.

### 92.9 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 170 teste passed (44 risk + 42 indicators + 21 genetic + 58 main + 5 integration)
- 12 teste noi streaming indicators (3 proptest + 9 unit), 0 failed, 0 regressions

---

## Cap. 93 — Sesiunea 28: IndicatorCache — Cache Hybrid per Simbol+TF (S2+S4 Partea 2)

### 93.1 Ce este

`IndicatorCache` este modulul care integreaza `StreamingIndicators` (Cap. 92) in engine-ul de trading. Fara el, StreamingIndicators exista izolat in crate fara efect real. Cu IndicatorCache, autopilotul nu mai recalculeaza toti indicatorii de la zero la fiecare ciclu.

### 93.2 Strategie hibrida

Cache-ul foloseste o strategie hibrida cu doua componente:

- **Indicatori clasici** (EMA, RSI, MACD, BB, ADX, VWAP, StochRSI): calculati incremental via StreamingIndicators — O(1) per candle nou
- **Oracle** (7 layere: Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD): recalculat batch doar la candle boundary (cand timestamp-ul ultimului candle se schimba)

### 93.3 Cele trei path-uri de executie

| Path | Cand se activeaza | Ce face | Cost |
|------|-------------------|---------|------|
| **Cache hit** | Acelasi candle timestamp (~95% din cicluri) | Returneaza snapshot-ul stocat instant | Zero |
| **Cache miss** | Candle nou (timestamp diferit) | `streaming.update()` O(1) + `oracle::calculate()` batch | O(1) + O(n) oracle |
| **Cold start** | Prima apelare per simbol | `StreamingIndicators::cold_start()` + `calculate_all_with_periods()` batch | O(n) full |

La cache hit, nu se face NICIO computatie. Rezultatul e returnat direct din memorie.

### 93.4 Structura

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

| Metoda | Ce face |
|--------|---------|
| `IndicatorCache::new()` | Creeaza cache gol |
| `cache.get_or_compute(symbol, tf, candles, periods, oracle_weights)` | Returneaza Indicators: cache hit, update incremental, sau cold start |
| `cache.evict_stale(max_age)` | Sterge entries neaccesate mai mult de `max_age` |

### 93.6 Integrare in autopilot

In autopilot.rs, apelul:

```rust
let indicators = calculate_all_with_periods(&candles, &periods, &oracle_w);
```

a fost inlocuit cu:

```rust
let indicators = {
    let mut cache = state.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
    cache.get_or_compute(candidate_symbol, &cand_tf, &candles, &periods, &oracle_w)
};
```

Eviction-ul se face periodic (~1% din cicluri, rand check) cu durata maxima de 1 ora.

### 93.7 Decizie de design: cold start foloseste batch

La prima apelare per simbol, se ruleaza `calculate_all_with_periods()` complet (batch). `StreamingIndicators::cold_start()` initializeaza doar state-ul intern. Motivul: garanteaza output BIT-IDENTIC cu path-ul non-cached. Dupa cold start, toate apelurile ulterioare folosesc streaming incremental.

### 93.8 Ce NU se face

- Oracle NU e incrementalizat (ramane batch, dar se apeleaza doar o data per candle nou)
- `compute_strategy_score_with_mode()` (scoring dashboard) si entry check din misc.rs nu sunt cached (pure functions fara acces la state)
- Backtest-ul nu foloseste cache-ul (ruleaza izolat cu date diferite)

### 93.9 Impact performanta

| Metrica | Inainte | Dupa |
|---------|---------|------|
| Indicator computation per ciclu | O(200) candles * 14 indicatori + Oracle 7 layere | Zero (cache hit 95%+) |
| La candle nou | O(200) full recalculate | O(1) streaming + O(n) Oracle (o data) |
| La prima apelare | O(200) full | O(200) full (identic, cold start) |

### 93.10 Fisiere

- **Creat:** `src-tauri/src/indicator_cache.rs`
- **Modificate:** `main.rs` (mod + init), `state.rs` (field pe TitanState), `autopilot.rs` (cache lookup + eviction)

### 93.11 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 erori, 0 warnings
- `cargo test --workspace`: 179 teste passed (44 risk + 42 indicators + 21 genetic + 67 main + 5 integration)
- 9 teste noi indicator_cache, 0 failed, 0 regressions
- Test `cold_start_matches_batch`: toate cele 19 campuri Indicators identice cu batch pe 200 candles

---

## Cap. 94 — Sesiunea 29: TitanAudio — Audio/Visual Alert System (S12)

### 94.1 Ce este

Modul `TitanAudio` adauga alerte audio procedurale, screen-edge glow si flash title pe evenimentele critice de trading. Scopul: utilizatorul nu trebuie sa stea cu ochii pe ecran non-stop — sunetele si efectele vizuale il alerteaza instant cand se intampla ceva important.

### 94.2 Problema rezolvata

Inainte de S12:
- Trade fills apareau doar ca text in log
- Circuit breaker PANIC/HALT se vedea doar pe badge-ul TITAN SAFETY
- **Flash crash** era emis din Rust (`streamer.rs`) dar **nu avea NICIUN listener in frontend** — gap critic de safety
- Watchdog alerts erau doar log entries

Dupa S12, toate aceste evenimente au sunet distinctiv + feedback vizual (flash title si/sau screen glow).

### 94.3 Sunete procedurale (7 tipuri)

Toate sunetele sunt generate procedural cu Web Audio API (fara fisiere .wav/.mp3):

| Sunet | Tip undă | Frecvente | Durata | Cand se joaca |
|-------|----------|-----------|--------|---------------|
| `fill-entry` | sine | 520→780→1040 Hz (ascendent) | 200ms | La fiecare trade fill |
| `fill-exit-profit` | sine+triangle | 880→1320→1760 Hz (cha-ching) | 300ms | Disponibil programatic |
| `fill-exit-loss` | sawtooth | 220→180 Hz (buzz) | 350ms | Disponibil programatic |
| `breaker` | square | 600+750 Hz, 3x pulse | 1s | Breaker CAUTION/HALT/PANIC |
| `flash-crash` | sawtooth+square | 440→330→220 Hz (descendent) | 1s | Flash crash detectat |
| `signal` | sine | 1000+1200 Hz (ping) | 200ms | Disponibil programatic |
| `watchdog` | triangle | 400→500→400 Hz (triple) | 550ms | Watchdog dead loops |

### 94.4 Feedback vizual

**Flash Title:** Alternare document.title la 500ms cu mesaj descriptiv (ex: "FILL: BUY AAPL", "BREAKER PANIC"). Durata variabila (2-6 secunde dupa severitate). Util cand fereastra Titan nu e in focus.

**Screen Glow:** Overlay fullscreen cu `inset box-shadow` animat (3 pulsuri). Culori:
- **Rosu** (`rgba(255, 43, 43, 0.35)`) — breaker PANIC/HALT, 4 secunde
- **Galben** (`rgba(255, 215, 0, 0.25)`) — breaker CAUTION, 2.5 secunde
- **Magenta** (`rgba(255, 0, 100, 0.4)`) — flash crash, 5 secunde

### 94.5 Toggle AUDIO ON/OFF

Buton in header-ul TITAN SAFETY panel (langa breaker badge). Stare persistata in `localStorage` (key `titan_audio_enabled`). Default: ON.

Toggle se sincronizeaza vizual: cyan cand ON, gri cand OFF. Un click pe buton schimba starea instant.

### 94.6 Wire-up pe Tauri events

| Event Tauri | Sunet | Flash Title | Screen Glow |
|-------------|-------|-------------|-------------|
| `trade-fill` | fill-entry | "FILL: SIDE SYMBOL" (2.5s) | — |
| `breaker-triggered` (non-NORMAL) | breaker | "BREAKER LEVEL" (5s) | Rosu/Galben |
| `flash-crash` | flash-crash | "FLASH CRASH: SYMBOL" (6s) | Magenta |
| `watchdog-alert` (dead>0) | watchdog | "WATCHDOG: N DEAD" (4s) | — |

Breaker cu level=NORMAL (reset) nu emite sunet — e un reset, nu un alarm.

### 94.7 Detalii tehnice

- **AudioContext partajat** — o singura instanta reutilizata (mai eficient decat pattern-ul existent de `new AudioContext()` per sunet din `_playSignalAlertTone`)
- **Lazy init** — AudioContext se creeaza la primul user gesture (click/keydown), necesar pentru Chrome/WebKit autoplay policy
- **Volume** — default 0.35, configurabil via `TitanAudio.setVolume(v)` (range 0.05-1.0)
- **Tauri Notification API** — nu e folosit (nu e in `tauri.conf.json` allowlist); flash title + screen glow au impact vizual superior

### 94.8 Fisiere

- **Creat:** `src/titan_audio.js` (234 linii)
- **Modificate:** `src/index.html` (script tag + buton AUDIO toggle), `src/titan_bootstrap.js` (wireAudioAlerts step), `src/styles.css` (screen glow + toggle button)

### 94.9 API public

| Metoda | Ce face |
|--------|---------|
| `TitanAudio.play(eventType)` | Joaca sunetul specificat |
| `TitanAudio.flashTitle(msg, ms)` | Alternare document.title |
| `TitanAudio.showScreenGlow(color, ms)` | Overlay fullscreen glow |
| `TitanAudio.toggle()` | ON↔OFF |
| `TitanAudio.setEnabled(bool)` | Setare directa |
| `TitanAudio.isEnabled()` | Query stare |
| `TitanAudio.setVolume(v)` | Volum (0.05-1.0) |
| `TitanAudio.wireEvents()` | Conectare la Tauri events |

### 94.10 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 179 teste passed, 0 regressions
- 29 sesiuni consecutive fara regressions

## Cap. 95 — Sesiunea 30: TitanToasts — Sistem Unificat de Notificari (Phase 4)

### 95.1 Ce este?

TitanToasts este un sistem unificat de notificari non-blocking care inlocuieste complet `alert()` si `confirm()` nativ ale browser-ului. In loc sa blocheze UI-ul cu dialoguri modale native (care impiedica orice interactiune pana la inchidere), notificarile apar ca toast-uri slide-in in coltul dreapta-sus si dispar automat.

### 95.2 De ce era necesar?

65 de `alert()` calls blocau complet UI-ul aplicatiei. In scalping activ, un dialog blocker care apare dupa un order fill sau o eroare poate impiedica operatorul sa reactioneze la un alt eveniment critic. De asemenea, 16 `confirm()` calls aveau acelasi efect blocker. Plus existau 4 sisteme de toast diferite (fiecare cu CSS si comportament inconsistent).

### 95.3 showCyberToast(message, type, durationMs)

Afiseaza o notificare non-blocking in coltul dreapta-sus al ecranului.

**Tipuri disponibile:**

| Tip | Culoare | Durata default | Icon | Cand se foloseste |
|-----|---------|---------------|------|-------------------|
| `success` | Verde (#00FF88) | 3 secunde | ✔ | Order executat, config salvat, actiune reusita |
| `error` | Rosu (#FF3344) | 6 secunde | ✖ | Eroare order, eroare API, esec critic |
| `warning` | Amber (#FFD700) | 5 secunde | ⚠ | Lista goala, input invalid, atentie |
| `info` | Cyan (#00BFFF) | 3 secunde | ℹ | Informatii, rezultate, status |

**Comportament:**
- Slide-in din dreapta cu animatie, auto-dismiss dupa durata specificata
- Click pe X pentru dismiss manual
- Stack max 5 toast-uri vizibile simultan (oldest auto-removed)
- Glassmorphism (backdrop-filter: blur) consistent cu estetica cyber-HUD

### 95.4 cyberConfirm(message, opts)

Inlocuieste `confirm()` nativ cu un modal stilizat. Returneaza `Promise<boolean>`.

**Caracteristici:**
- Overlay cu backdrop blur
- Box centrat cu gradient dark si border cyan
- Butoane CONFIRM (cyan) / CANCEL (gri)
- **Keyboard:** Enter = confirm, Escape = cancel
- Click pe overlay (in afara box-ului) = cancel
- Focus automat pe butonul CONFIRM

**Exemplu de utilizare:**
```javascript
if (!(await cyberConfirm('Delete all drawings?'))) return;
// utilizatorul a confirmat, continua actiunea
```

### 95.5 Ce s-a inlocuit

**65 alert() inlocuite in 11 fisiere JS:**
- titan_bootstrap.js (14), titan_watchlist.js (11), titan_ai_scanner.js (11)
- titan_positions.js (8), titan_scanner_plus.js (5), titan_manual_trade.js (5)
- titan_ls_settings.js (3), titan_sidebar.js (2), titan_replay_alerts.js (2)
- drawing_engine_v2.js (2), titan_rightbar.js (1)

**16 confirm() inlocuite in 11 fisiere JS:**
- titan_nexus.js (3), titan_ls_settings.js (3), titan_perf_stats.js (2)
- titan_manual_trade.js (2), titan_bootstrap.js (1), titan_positions.js (1)
- titan_scanner_plus.js (1), titan_ai_scanner.js (1), titan_watchlist.js (1)
- titan_hft_panel.js (1), drawing_engine_v2.js (1)

### 95.6 Consolidare toast-uri existente

| Toast vechi | Ce s-a intamplat | Motiv |
|------------|------------------|-------|
| `hftToast()` (30 utilizari) | Redirected la showCyberToast (thin wrapper cu color map) | Acelasi tip de notificare simpla |
| `showNewMoverToast()` | Simplificat la un singur showCyberToast call | Era o notificare simpla cu CSS custom |
| `_showSignalAlertToast()` | PASTRAT ca atare | Domain-specific: tipuri buy/sell/trail/chaos, pozitionare bottom-right |
| `showCatalystToast()` | PASTRAT ca atare | Layout complex multi-field: symbol, headline, score, action buttons |

### 95.7 Fisiere

- **Creat:** `src/titan_toasts.js` (152 linii)
- **Modificate:** `src/index.html` (script tag), `src/styles.css` (toast + confirm CSS, dead code cleanup), plus 14 fisiere JS cu inlocuiri alert/confirm

### 95.8 API public

| Functie | Ce face |
|---------|---------|
| `showCyberToast(msg, type, ms)` | Afiseaza toast non-blocking |
| `cyberConfirm(msg, opts)` | Modal async, returneaza Promise |
| `TitanToasts.show(msg, type, ms)` | Echivalent showCyberToast (API interna) |
| `TitanToasts.confirm(msg, opts)` | Echivalent cyberConfirm (API interna) |

### 95.9 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 179 teste passed, 0 regressions
- 30 sesiuni consecutive fara regressions

---

## 96. PROFILE PERSISTENCE PE DISK (RF-C) — Sesiunea 31

### 96.1 Ce s-a facut

Profilele de trading au fost mutate de la `localStorage` (efemer, se pierde la rebuild) pe disk in `titan_profiles.json`. Incarcarea profildului restaureaza TOTI parametrii inclusiv obiecte nested (TA Gates, Scan Params, Symbol Overrides) — fix-ul unui bug existent care filtra obiectele la load.

### 96.2 Comenzi Tauri noi (backend)

| Comanda | Ce face |
|---------|---------|
| `save_profile(name)` | Snapshot complet AppSettings cu metadata (schema_v, saved_at) |
| `load_profile(name)` | Aplica profil complet pe AppState via apply_settings() |
| `delete_profile(name)` | Sterge profil (graceful daca nu exista) |
| `list_profiles()` | Returneaza lista sortata cu metadata (name, saved_at, schema_v) |
| `export_profile(name)` | Returneaza JSON complet al profilului |
| `import_profile(name, data)` | Importa profil din JSON |

### 96.3 Storage pe disk

- **Fisier:** `titan_profiles.json` in folderul de date (`data/`)
- **Format:** `{ "v": 1, "profiles": { "scalping": { "schema_v": 1, "saved_at": "2026-...", "data": {...} } } }`
- **Scriere atomica:** write-to-temp + rename (pattern identic cu `titan_settings.json`)
- **Auto-backup:** `.bak` creat inainte de fiecare save
- **Fallback:** Daca fisierul principal e corrupt, se incearca `.bak`; daca si acela e corrupt, store gol

### 96.4 Migrare automata din localStorage

La prima rulare dupa update, daca exista cheia `titan_hft_profiles` in localStorage:
1. Fiecare profil vechi este importat pe disk via `import_profile`
2. Cheia veche este stearsa din localStorage
3. Migrarea se face o singura data (one-shot)

### 96.5 Butoane noi in UI

- **EXPORT** — Exporta profilul selectat ca fisier JSON (`titan_profile_<name>.json`)
- **IMPORT** — Deschide file picker pentru import profil din JSON; numele se preia din fisier sau din campul de text

### 96.6 Bonus: N5 — max_manual_order_usd Enforcement

Setting-ul `max_manual_order_usd` (default $50,000) exista dar nu era verificat server-side. Acum `submit_manual_order` in `trading.rs` blocheaza ordine care depasesc limita cu eroare clara: "Order $X exceeds max_manual_order_usd limit ($Y)".

### 96.7 Bonus: Fix async DELETE handler

Handler-ul de delete profil avea `await cyberConfirm()` intr-o functie non-async — bug JS real. Fix-at cu `async () => { ... }`.

### 96.8 Fisiere modificate

- `src-tauri/src/paths.rs` — `profiles_path()` adaugat
- `src-tauri/src/state.rs` — ProfileStore/ProfileEntry/ProfileMeta structs, load/save functions, 8 teste
- `src-tauri/src/commands/settings.rs` — 6 comenzi Tauri noi
- `src-tauri/src/commands/trading.rs` — N5 enforcement + 1 test
- `src-tauri/src/main.rs` — comenzi inregistrate
- `src/titan_perf_stats.js` — PARAMETER PROFILES rescris complet
- `src/index.html` — 2 butoane noi (EXPORT/IMPORT profile)

### 96.9 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 188 teste passed (179 + 9 noi), 0 regressions
- 31 sesiuni consecutive fara regressions

---

## 97. CONCURRENT PRE-FETCH + EXIT PROCESSING (S5+S6) — Sesiunea 32

### 97.1 Ce s-a facut

Autopilot-ul evalua pozitiile si candidatii **secvential** — cu 5 pozitii deschise si 10 simboluri candidate, latenta era O(n * per_simbol_latency). Acum evaluarile ruleaza **concurent** via `futures::future::join_all`, iar deciziile (API calls, state mutations) se aplica secvential.

### 97.2 Arhitectura: Gather in Parallel, Apply Sequentially

Principiul: **citirile se fac in paralel, scrierile raman secventiale**. Niciun lock nu e tinut in codul paralel mai mult de microsecunde.

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

### 97.3 Structuri noi

**ExitEvaluation** — encapsuleaza decizia de exit per pozitie fara side effects:
- symbol, qty, is_short, current_price, entry_price, pnl_pct
- stop_loss, take_profit, trail_pct (deja calculate cu ATR + adaptive multipliers)
- force_exit, force_exit_reason (oracle flip, chaos, signal exit)
- partial_close: Option (partial_qty, partial_side)
- hold_secs, cand_tf

**EntryEvaluation** — encapsuleaza decizia de entry per candidat:
- symbol, decision (Option<TradeDecision>), cand_tf, vol_factor

### 97.4 Schimbare de comportament: Best-Score Selection

Anterior, autopilotul lua **primul candidat** care trecea filtrele. Acum, toti candidatii sunt evaluati in paralel, si se selecteaza cel cu **scorul maxim** (best-match in loc de first-match). Rezultat: autopilotul alege optim.

### 97.5 Ce NU s-a schimbat

- **SignalStrategy** ramane secvential (flow diferit, markers, nu check_entry)
- **close_position** ramane serializat (un singur API call la broker per tick)
- **Pre-filtrarea** ramane secventiala (flash crash, pending orders, correlation — au dependinte pe state)

### 97.6 Impact performanta

Cu ring buffer (S1) + indicator cache (S2+S4) deja implementate, majoritatea `get_candles` sunt cache hits (<1us). Paralelizarea ajuta cel mai mult cand:
- Buffer-ul e gol si se face REST fallback (50-500ms per simbol)
- `get_signal_markers_internal` trebuie calculat (compute-heavy)
- Sunt multe pozitii deschise cu ATR calculation per simbol

### 97.7 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 197 teste passed (188 + 9 noi), 0 regressions
- 32 sesiuni consecutive fara regressions

---

## 98. PORTFOLIO VaR ENGINE + HHI CONCENTRATION GATE (S9) — Sesiunea 33

### 98.1 Ce este?

Doua gate-uri complementare de risc la nivel de **portofoliu** care PREVIN intrari cand riscul total e prea mare. Spre deosebire de CircuitBreaker / FlashCrash / DD Velocity (care sunt **reactive** — actioneaza DUPA pierdere), VaR si HHI sunt **preventive** — blocheaza intrarea INAINTE sa apara riscul.

- **VaR (Value at Risk)** — masoara riscul statistic al portofoliului folosind volatilitatea si corelatia intre assets. Intrebarea la care raspunde: "Cat pot pierde pe zi cu 99% probabilitate?"
- **HHI (Herfindahl-Hirschman Index)** — masoara concentrarea portofoliului. Intrebarea: "Cat de mult capital am pus intr-un singur simbol?"

### 98.2 De ce era necesar?

Risk management-ul existent proteja contra pierderilor INDIVIDUALE:
- Stop loss per pozitie
- Circuit breaker pe PnL zilnic total
- Flash crash pe miscare brusca de pret
- DD Velocity pe viteza pierderii

Dar NU exista nimic care sa spuna:
- "Nu intra in NVDA pentru ca 80% din portofoliu e deja NVDA" (concentrare)
- "Portofoliul are o volatilitate implicita de 5%, o pozitie noua ar creste-o la 7%" (risc statistic)

VaR si HHI acopera aceste doua gap-uri.

### 98.3 VaR — Parametric Value at Risk

**Metoda:** Parametric (variance-covariance) la 99% confidence:

```
VaR = z_score(0.99) * sqrt(w' * Sigma * w)
```

unde:
- `w` = vectorul de ponderi (notional per asset / total notional)
- `Sigma` = matricea de covarianta NxN (din returns log pe ultimele 20 perioade)
- `z_score(0.99)` = 2.326 (approximatie Abramowitz & Stegun)

**CVaR (Expected Shortfall)** — media pierderilor DINCOLO de VaR:

```
CVaR = VaR * pdf(z) / (z * (1 - confidence))
```

CVaR e intotdeauna mai mare decat VaR si captureaza tail risk.

**Cum functioneaza gate-ul:**

1. Se extrag close prices din CandleRingBuffer pentru fiecare simbol din portofoliu
2. Se computa log returns si matricea de covarianta
3. Se calculeaza VaR cu portofoliul ACTUAL
4. Se calculeaza VaR cu portofoliul PROPUS (adaugand noua pozitie)
5. Daca VaR propus > `max_portfolio_var_pct` → entry BLOCAT

### 98.4 HHI — Concentration Risk Gate

**Formula:**

```
HHI = sum(w_i^2)     unde w_i = |notional_i| / total_notional
```

**Range:**
- `HHI = 1.0` — tot capitalul intr-un singur simbol (concentrare maxima)
- `HHI = 0.25` — 4 pozitii egale (diversificat)
- `HHI = 0.10` — 10 pozitii egale (bine diversificat)

**Cum functioneaza gate-ul:**

1. Se extrag notional-urile pozitiilor curente
2. Se adauga notional-ul estimat al pozitiei propuse
3. Se calculeaza HHI propus
4. Daca HHI propus > `max_concentration_hhi` → entry BLOCAT

### 98.5 Setari (tab HFT Engine)

In cardul **PORTFOLIO RISK GATES** (violet, dupa TA ENTRY GATES):

| Setting | Default | Range | Descriere |
|---------|---------|-------|-----------|
| VaR Gate ON/OFF | OFF | toggle | Activeaza/dezactiveaza VaR gate |
| Max Portfolio VaR % | 2.0% | 0.1-10.0 | Threshold maxim VaR 99% |
| Lookback Periods | 20 | 5-100 | Cate perioade de returns pentru covarianta |
| Concentration Gate ON/OFF | OFF | toggle | Activeaza/dezactiveaza HHI gate |
| Max HHI | 0.40 | 0.10-1.00 | Threshold maxim concentrare |

**Ambele gate-uri sunt default OFF** — zero schimbare de comportament pana cand utilizatorul le activeaza explicit.

### 98.6 Pozitionare in pipeline-ul de entry

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

### 98.7 De ce VaR la 99% si nu 95%?

Pentru scalping HFT cu pozitii de minute, 95% VaR e prea permisiv — permite prea multe intrari in conditii de volatilitate ridicata. 99% ofera o marja de siguranta mai mare, mai ales in combinatie cu Circuit Breaker care deja actioneaza pe pierderi absolute.

### 98.8 Limitari cunoscute

- **Parametric VaR presupune distributie normala** — subestimeaza tail risk (fat tails). HHI-ul complementeaza prin protectie non-statistica.
- **20 perioade de lookback** sunt putine — covarianta poate fi instabila. Dar in scalping, conditiile de piata se schimba rapid, deci un lookback lung ar fi contra-productiv.
- **CVaR e calculat dar nu e expus in UI** — disponibil prin API pentru analiza viitoare.
- **Gate-urile nu se aplica pe SignalStrategy path** — doar pe non-signal candidates (comportament consistent cu OFI/Nexus).

### 98.9 Teste

16 teste noi in titan-risk crate (total 60):
- `compute_returns_basic/empty_and_single` — log returns corecte + edge cases
- `covariance_matrix_single_asset/symmetric` — matrice valida
- `single_position_var` — VaR pozitiv, VaR99 > VaR95
- `two_correlated_assets_var` — 2 assets identice 50/50 = acelasi VaR ca single
- `uncorrelated_assets_diversification_benefit` — diversificat < concentrat
- `empty_portfolio_zero_var` — zero VaR fara pozitii
- `cvar_exceeds_var` — CVaR > VaR (tail risk)
- `var_gate_blocks/allows` — gate functioneaza corect la ambele extreme
- `hhi_single_position/equal_positions/empty` — HHI corect matematic
- `hhi_gate_blocks_concentrated/allows_diversified` — gate HHI functioneaza

### 98.10 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 213 teste passed (197 + 16 noi), 0 regressions
- 33 sesiuni consecutive fara regressions

---

## 99. FEATURE STORE — ENTRY CONTEXT SNAPSHOT (S13) — Sesiunea 34

### 99.1 Ce este

Feature Store este un sistem de capturare a contextului complet al indicatorilor si strategiei la momentul exact al intrarii intr-un trade. Fiecare trade din jurnal contine acum un snapshot cu 20 de campuri care descriu starea pietei si decizia de trading la momentul entry-ului.

### 99.2 Problema rezolvata

Inainte de Feature Store, `TradeRecord` in jurnal continea doar date de executie: simbol, directie, cantitate, preturi, PnL, durata, strategie, timeframe. Campul `notes` era gol. Informatia "de ce s-a intrat in trade" si "care era starea indicatorilor" se pierdea permanent la fiecare trade.

Consecinte:
- Analiza post-trade era limitata la "am castigat/pierdut X dolari"
- Nu se putea raspunde la "de ce a mers/nu a mers trade-ul?"
- Nu exista date pentru optimizare bazata pe corelatii indicator-performanta
- Pattern-urile de succes/esec erau invizibile

### 99.3 Ce captureaza TradeFeatures (20 campuri)

| Categorie | Camp | Descriere |
|-----------|------|-----------|
| **Oracle** | `oracle_score` | Scorul Oracle composite (-100 la +100) |
| **Oracle** | `oracle_confidence` | Nivel de incredere Oracle (0.0 la 1.0) |
| **Oracle** | `oracle_regime` | Regimul pietei (Trending, MeanReverting, Chaotic, Unknown) |
| **Indicatori** | `rsi` | Relative Strength Index |
| **Indicatori** | `adx` | Average Directional Index (forta trendului) |
| **Indicatori** | `atr_pct` | ATR ca procent din pret (volatilitate) |
| **Indicatori** | `ema9` | EMA pe 9 perioade |
| **Indicatori** | `ema50` | EMA pe 50 perioade |
| **Flow** | `vpin` | Volume-sync Probability of Informed Trading |
| **Flow** | `hurst` | Hurst Exponent (persistenta/mean-reversion) |
| **Flow** | `rvol` | Relative Volume (volum curent vs medie) |
| **Microstructure** | `ofi` | Order Flow Imbalance (presiune bid/ask) |
| **Microstructure** | `spread_bps` | Spread in basis points |
| **Microstructure** | `imbalance_ratio` | Ratio dezechilibru bid/ask |
| **Strategy** | `strategy_score` | Scorul total din check_entry() |
| **Strategy** | `signal_source` | Sursa semnalului (ORACLE, GPU, FUSION, SIGNAL_STRATEGY) |
| **Strategy** | `fvg_type` | Tip FVG (Bullish, Bearish, None) |
| **Strategy** | `entry_type` | Tipul intrarii (ex: Trending_LONG, SCALP_MeanRev_SHORT) |
| **TA Gates** | `ta_gates_passed` | Numarul de gate-uri TA trecute |
| **TA Gates** | `ta_gates_total` | Numarul total de gate-uri TA evaluate |

### 99.4 Cum functioneaza

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

**La entry:** Dupa `send_order_internal` reusit, `snapshot_entry_features()` citeste:
- Indicatori din `IndicatorCache` (read-only, fara re-compute)
- OFI state din `ArcSwap` (lock-free)
- Metadata din `TradeDecision` (score, FVG, entry_type)
- RVOL din vol_cache

**La full exit:** `entry_features.remove(&symbol)` -- extrage si sterge din map.

**La partial exit:** `entry_features.get(&symbol).cloned()` -- citeste fara a sterge (pozitia e inca deschisa).

### 99.5 Backward compatibility

Campul `entry_features` pe `TradeRecord` are `#[serde(default)]`. Jurnal-uri vechi (fara acest camp) se deserializeaza cu `entry_features: None`. Zero impact pe UI sau functionalitate existenta.

### 99.6 Utilizare viitoare

Datele din Feature Store vor fi folosite de:
- **S20 AI Trade Journal Digest** -- LLM-ul va analiza corelatii indicator-performanta
- **S15 Oracle Weight Adaptation** -- IMPLEMENTAT (Sesiunea 38). Ponderile Oracle se adapteaza automat per-regime, cu confidence-weighted learning si continuous decay. Vezi Cap. 103
- **Auto-Suggest imbunatatit** -- sugestii bazate pe indicatori, nu doar PnL
- **Pattern recognition** -- identificare automata a conditiilor optime de intrare

### 99.7 Fisiere modificate

- `src-tauri/src/journal.rs` -- TradeFeatures struct, TradeRecord/TradeRecordInput extinse
- `src-tauri/src/indicator_cache.rs` -- `peek()` read-only lookup
- `src-tauri/src/loops/autopilot.rs` -- snapshot la entry (2 paths), propagare la exit (2 paths)
- `src-tauri/src/state.rs` -- `entry_features` pe TitanState
- `src-tauri/src/main.rs` -- initializare `entry_features`
- `src-tauri/src/commands/misc.rs` -- `entry_features: None` pe trade_updates

### 99.8 Teste (8 noi, 221 total)

- Serde roundtrip complet (20 campuri)
- Backward compatibility (load journal vechi)
- Persist cu features / fara features
- Vec mixt (unele cu features, altele fara)
- Default values (zerouri si stringuri goale)

### 99.9 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 221 teste passed (213 + 8 noi), 0 regressions
- 34 sesiuni consecutive fara regressions

---

## 100. MICROSTRUCTURE ALPHA SIGNALS — STREAM VPIN, SPREAD REGIME, DUAL VPIN DIVERGENCE (S10-lite) — Sesiunea 35

### 100.1 Ce este

Sistem complet de microstructure alpha signals care transforma datele existente din streamer (trade classification, spread, imbalance) din **filtre binare** (da/nu) in **score modifiers soft** (bonus/penalty pe scorul de entry). Adauga trei componente noi: **Stream VPIN** (VPIN real din trade flow), **Spread Regime** (detectie narrowing), si un semnal original -- **Dual VPIN Divergence** (comparatie stream vs candle VPIN).

### 100.2 De ce conteaza

Inainte de aceasta implementare, datele de microstructura existau in sistem dar erau folosite doar binar:
- `spread_is_safe()` -- da sau nu (fara nuante)
- `ofi_confirms_signal()` -- da sau nu (fara intensitate)
- VPIN din oracle -- calculat din candle BVC (proxy slab, nu din tape real)

Acum, microstructura influenteaza **scorul** fiecarui candidat de entry. Un candidat cu spread narrowing, imbalance aliniat si VPIN sanatos primeste pana la **+14 puncte** bonus. Un candidat cu toxicitate ascunsa primeste **-18 puncte** penalty. Diferenta intre a intra sau nu intr-un trade.

### 100.3 Stream VPIN -- VPIN real din trade flow

**Ce face:** Calculeaza VPIN (Volume-Weighted Probability of Informed Trading) direct din trade-by-trade flow, folosind clasificarea Lee-Ready existenta (`classify_trade`).

**Diferenta fata de candle VPIN (oracle):**

| Aspect | Candle VPIN (oracle) | Stream VPIN (S10-lite) |
|--------|---------------------|----------------------|
| Clasificare | BVC (close position in range) -- proxy | Lee-Ready quote rule -- exact |
| Granularitate | Per candle (1m/5m) | Per trade (~ms) |
| Binning | Time-synced (n candles) | Volume-synced (n shares) |
| Actualizare | La candle boundary | La fiecare trade clasificat |
| Precizie pe TF scurte | Moderata | Inalta |

**Cum functioneaza:**

1. Fiecare trade clasificat (buy/sell) adauga volum la bucket-ul curent
2. Cand `buy_vol + sell_vol >= bucket_vol_threshold` (5000 default), bucket-ul se inchide
3. VPIN raw = `sum(|buy - sell|) / sum(buy + sell)` peste ultimele 20 buckets
4. EMA smoothing pe VPIN raw (`alpha = 0.15`) pentru a filtra noise

**Interpretare:**
- VPIN ~ 0: flow balansat (cumparatori si vanzatori in echilibru)
- VPIN > 0.5: flow dezechilibrat (informati tranzactioneaza agresiv)
- VPIN > 0.85: toxicitate extrema (risc ridicat de miscare adversa)

### 100.4 Spread Regime -- Detectie narrowing

**Ce face:** Doua EMA-uri pe `spread_bps` (bid-ask spread in basis points) cu viteze diferite. Cand EMA rapida scade sub 80% din EMA lenta, flagul `spread_narrowing` se activeaza.

**Parametri:**
- EMA fast: `alpha = 0.15` (~13 ticks half-life)
- EMA slow: `alpha = 0.03` (~66 ticks half-life)
- Narrowing threshold: `fast < slow * 0.80`

**De ce conteaza:**

Market makers strang spread-ul cand anticipeaza miscare directionala iminenta. Spread narrowing dupa o perioada de spread larg este un semnal pre-directional. Autopilotul foloseste acest flag ca bonus de +5 pe scorul entry-ului.

### 100.5 Dual VPIN Divergence (concept original)

**Ce face:** Compara Stream VPIN (din tape, precis) cu Candle VPIN (din oracle, aproximativ). Cand divergeaza semnificativ, este informativ:

| Situatie | Divergenta | Interpretare | Efect |
|----------|-----------|-------------|-------|
| Stream VPIN ridicat + Candle VPIN scazut | > +0.15 | **Hidden toxicity** -- flow toxic detectat in tape dar inca nu vizibil in candle-uri | Penalty **-8** |
| Stream VPIN scazut + Candle VPIN ridicat | < -0.15 | **False alarm** -- candle VPIN e noise din range compression | Bonus **+5** |
| Ambele similare | [-0.15, +0.15] | Concordanta, fara divergenta | 0 |

**Referinta academica:** Inspirat de Easley, Lopez de Prado, O'Hara (2012) "Flow Toxicity and Liquidity in a High-Frequency World". Dual-source divergence nu e implementat in retail HFT -- concept original.

### 100.6 Microstructure Score Modifier -- Tabel complet

Functia `apply_microstructure_modifier()` se aplica in `evaluate_candidate_entry()`, DUPA ce `check_entry()` calculeaza scorul base, INAINTE de ranking-ul candidatilor:

| Conditie | Bonus/Penalty | Cumulativ | Nota |
|----------|--------------|-----------|------|
| Spread narrowing | **+5** | Da | Market makers anticipeaza directie |
| NBBO imbalance aliniat (buy + imb > 0.3 sau sell + imb < -0.3) | **+4** | Da | Order flow confirma directia |
| Dual VPIN: false alarm filtrata (divergence < -0.15) | **+5** | Da | Candle VPIN era noise |
| Dual VPIN: hidden toxicity (divergence > +0.15) | **-8** | Da | Pericol ascuns in tape |
| Stream VPIN extreme (> 0.85) | **-10** | Da | Toxicitate foarte ridicata |

**Maxim posibil:** +14 (narrowing + imbalance + false alarm) la -18 (hidden toxicity + extreme toxicity)

**Important:** Toti modifierii sunt **soft** (ajusteaza scorul, nu blocheaza). Un candidat cu penalty mare poate inca fi selectat daca are scor base suficient de mare. Consistent cu filozofia "best candidate by score" din concurrent entry evaluation (S5+S6).

### 100.7 Integrare in pipeline

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

### 100.8 TradeFeatures -- Campuri noi

Doua campuri adaugate la `TradeFeatures` (journal.rs), ambele cu `#[serde(default)]` pentru backward compatibility:

| Camp | Tip | Default | Descriere |
|------|-----|---------|-----------|
| `vpin_divergence` | f64 | 0.0 | `stream_vpin - candle_vpin` la momentul entry |
| `spread_narrowing` | bool | false | Flag spread regime la momentul entry |

Campul `vpin` existent acum contine **stream_vpin** (mai precis) in loc de candle VPIN.

### 100.9 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 240 teste passed (221 + 19 noi), 0 regressions
- 35 sesiuni consecutive fara regressions
- 19 teste noi: stream_vpin (4), spread_regime (4), micro_modifier (10), journal backward compat (3)

---

## 101. STRATEGY THRESHOLDS CONFIGURABILE — SCORE MODIFIERS, CLASSICAL BONUSES, ENTRY THRESHOLD MATRIX (N4) — Sesiunea 36

### 101.1 Ce este

Sistemul de extractie a ~50 magic numbers hardcoded din `strategy.rs` in structuri configurabile din UI. Inainte, parametrii critici care controlau cat de usor/greu intra Titan intr-un trade erau fixati in cod sursa -- acum sunt editabili in timp real din tab-ul HFT Engine, fara recompilare.

### 101.2 De ce conteaza

Inainte de N4:
- Daca voiai sa faci scoring-ul mai agresiv in trending, trebuia sa modifici codul sursa si sa recompilezi
- Daca voiai sa testezi un FVG bonus de 20 in loc de 15, aceeasi poveste
- Nu puteai compara rapid configuratii diferite
- Riscul de typo era real (schimbi un numar si spargi alt branch)

Dupa N4:
- Toate aceste valori sunt **slidere si inputuri** in UI
- Salvate pe disk, persistente intre restartari
- Buton VALIDATE care verifica consistenta logica
- Buton RESET DEFAULTS care restaureaza valorile originale
- **Zero behavior change** cu defaults identice -- botul se comporta exact ca inainte daca nu schimbi nimic

### 101.3 Cele 3 grupe de parametri

**Grup 1: Score Modifiers** (21 parametri)

Controluri fine asupra scorului calculat de `calculate_score()`:

| Parametru | Default | Ce face |
|-----------|---------|---------|
| Chaos Kill Penalty | -50 | Penalty cand confidence e sub chaos threshold |
| Chaos Kill Fallback | 50 | Scor returnat cand oracle nu e disponibil |
| Chaos Conf x MeanRev | 1.5 | Multiplicator chaos threshold pe regim MeanRev |
| Chaos Conf x Chaotic | 2.5 | Multiplicator pe regim Chaotic |
| Chaos Conf x Unknown | 1.5 | Multiplicator pe regim Unknown |
| Regime x MeanRev | 0.82 | Multiplicator oracle contribution pe MeanRev |
| Regime x Chaotic | 0.55 | Multiplicator pe Chaotic |
| Regime x Unknown | 0.73 | Multiplicator pe Unknown |
| VPIN Penalty High | -20 | Penalty cand VPIN > scalping gate |
| VPIN Penalty Medium | -10 | Penalty cand VPIN > normal gate |
| FVG Bull Bonus | +15 | Bonus pe Fair Value Gap bullish detectat |
| FVG Bear Penalty | -15 | Penalty pe FVG bearish |
| RVOL Tier1 Thr. | 2.0 | Prag RVOL tier 1 |
| RVOL Tier1 Bonus | +8 | Bonus daca RVOL > tier 1 |
| RVOL Tier2 Thr. | 4.0 | Prag RVOL tier 2 (institutional) |
| RVOL Tier2 Bonus | +12 | Bonus daca RVOL > tier 2 |
| StochRSI OB | 80 | Prag overbought pentru TA gate StochRSI |
| StochRSI OS | 20 | Prag oversold pentru TA gate StochRSI |
| BB Squeeze Width | 0.02 | Prag Bollinger squeeze (width < acest prag) |
| Strong Label | 80 | Oracle score > acest prag = "STRONG" entry |
| Mild Label | 60 | Oracle score > acest prag = entry normal |

**Grup 2: Classical Bonuses** (6 parametri)

Controluri asupra bonusurilor din `compute_classical_confirmation()` (active doar cand `use_classical_filters` e ON):

| Parametru | Default | Ce face |
|-----------|---------|---------|
| RSI Bonus | 10.0 | Bonus cand RSI confirma directia |
| EMA Align Bonus | 15.0 | Bonus cand EMA20 > EMA50 (long) sau invers |
| ADX Bonus | 10.0 | Bonus cand ADX > trend threshold |
| BB Squeeze Bonus | 5.0 | Bonus cand Bollinger bands sunt stranse |
| VWAP Bonus | 10.0 | Bonus cand pret e deasupra/sub VWAP |
| Cap | 50.0 | Limita superioara a sumei bonusurilor |

**Grup 3: Entry Threshold Matrix** (7 regimuri x 3 valori x 2 moduri = ~42 celule)

Grila care determina pragurile de entry per regim de piata:

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

Semnificatia coloanelor:
- **Long**: oracle_score trebuie sa depaseasca aceasta valoare pentru entry long
- **Short**: oracle_score trebuie sa fie sub aceasta valoare pentru entry short
- **Confidence**: oracle_confidence trebuie sa depaseasca aceasta valoare

### 101.4 UI in HFT Engine tab

Trei card-uri noi apar intre STRATEGY SCORING si ENGINE STOPS:

1. **SCORE MODIFIERS** (header portocaliu) -- 21 slidere in grid 3 coloane
2. **CLASSICAL BONUSES** (header verde) -- 6 slidere, card-ul apare semi-transparent cand Classical Filters e OFF
3. **ENTRY THRESHOLD MATRIX** (header cyan) -- grid 7 randuri x 3 coloane cu inputuri numerice, dropdown Scalping/Normal, butoane VALIDATE si RESET DEFAULTS

### 101.5 Butonul VALIDATE

Apeleaza comanda `validate_thresholds` si verifica:

1. Toate long_threshold > 0 (un threshold negativ nu are sens)
2. Toate short_threshold < 0
3. Toate conf_threshold in range [0.05, 0.95]
4. Scalping thresholds <= Normal thresholds (scalping e mai permisiv)
5. Trending thresholds <= Chaotic thresholds (gradient natural)
6. StochRSI overbought > oversold
7. RVOL tier1 < tier2

Rezultatele apar ca toasts (warnings, nu erori blocante).

### 101.6 Butonul RESET DEFAULTS

Restaureaza valorile din tabelul de mai sus pentru modul curent selectat (Scalping sau Normal). Util dupa experimente.

### 101.7 Implementare tehnica

Trei struct-uri noi in `app_state.rs`:
- `EntryThreshold { long_threshold, short_threshold, conf_threshold }`
- `ScoreModifiers` (21 campuri cu `impl Default`)
- `ClassicalBonuses` (6 campuri cu `impl Default`)

Campuri pe `AppState` cu `#[serde(default)]` pentru backward compatibility. Entry thresholds sunt `HashMap<String, EntryThreshold>` cu chei precum `trending`, `trending_aggro`, `mean_rev`, etc.

Functia `lookup_entry_threshold()` in `strategy.rs` face lookup cu fallback safe daca un key lipseste din HashMap.

33+ match arms noi in `settings.rs` + `apply_entry_threshold_field()` parser dinamic pentru entry thresholds.

### 101.8 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 248 teste passed (240 + 8 noi), 0 regressions
- 36 sesiuni consecutive fara regressions
- 8 teste noi: defaults match hardcoded (3), custom values (2), serde roundtrip (1), validation (2)

---

## 102. CI HARDENING + CRITERION BENCHMARKS (S19) — Sesiunea 37

### 102.1 Ce s-a adaugat

Pipeline-ul CI a fost extins cu doua instrumente noi de audit si un sistem complet de benchmarks de performanta.

**cargo-deny** — auditeaza licentele si advisory-urile tuturor dependintelor:
- Configuratie in `src-tauri/deny.toml`
- Whitelist licente: MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode, OpenSSL, BSL-1.0, CC0-1.0, MPL-2.0
- 19 advisory-uri ignorate cu rationale documentata (toate din Tauri 1.x transitive deps)
- Toate crate-urile workspace marcate `publish = false` (private, nu se publica)
- Ruleaza automat in CI la fiecare push/PR

**cargo-audit** — scaneaza dependency tree pentru CVE-uri cunoscute (RustSec Advisory DB):
- Ruleaza automat in CI la fiecare push/PR
- Blocheaza build-ul daca exista vulnerabilitati nerezolvate

**Criterion Benchmarks** — 6 benchmarks pe 3 path-uri critice:
- Fisiere: `src-tauri/crates/titan-indicators/benches/bench_oracle.rs`, `bench_indicators.rs`, `src-tauri/benches/bench_check_entry.rs`
- Se ruleaza manual cu `cargo bench` (nu automat in CI — vezi 102.3)

### 102.2 Benchmark-uri disponibile

| Benchmark | Ce masoara | Candles | Rezultat baseline | Target |
|-----------|-----------|---------|-------------------|--------|
| oracle_calculate_200 | Oracle 7-layer + fusion | 200 | 873 us | <5ms |
| oracle_calculate_100 | Oracle 7-layer + fusion | 100 | 208 us | - |
| oracle_calculate_50 | Oracle 7-layer + fusion | 50 | 51 us | - |
| indicators_all_200 | EMA + RSI + MACD + BB + ADX + VWAP + StochRSI + Oracle | 200 | 895 us | <2ms |
| indicators_all_100 | Acelasi set complet | 100 | 218 us | - |
| full_pipeline_200 | Indicators + Oracle end-to-end (ca check_entry) | 200 | 906 us | <10ms |

Toate sub 1ms pentru 200 candles. Pipeline-ul complet e de ~11x sub target.

### 102.3 Cum se folosesc benchmark-urile

Benchmark-urile sunt on-demand (nu automate). Se ruleaza cand vrei sa verifici performanta dupa o schimbare:

```bash
cargo bench                            # toate 6 benchmarks
cargo bench -p titan-indicators        # doar oracle + indicators (5 benches)
cargo bench --bench bench_check_entry  # doar full pipeline (3 benches)
```

Criterion genereaza rapoarte HTML in `target/criterion/report/index.html`. La rulari succesive, compara automat cu baseline-ul anterior si arata regresia/imbunatatirea.

### 102.4 CI Pipeline complet (post-sesiunea 37)

Fisiere: `.github/workflows/ci.yml` (push/PR) + `.github/workflows/release.yml` (tags v*)

Steps in ordine:
1. `cargo fmt --check` — formatare
2. `cargo clippy -- -D warnings` — lint
3. `cargo build --release` — compilare
4. `cargo test` — 248 teste
5. `cargo deny check` — license + advisory audit
6. `cargo audit` — CVE vulnerability scan

### 102.5 deny.toml — Advisory Ignorari

19 advisory-uri ignorate, toate din dependinte tranzitive pe care nu le controlam:
- 10x gtk-rs GTK3 bindings (Tauri 1.x, se rezolva la Tauri 2.x)
- instant, paste, proc-macro-error, rustls-pemfile, fxhash (Tauri 1.x unmaintained deps)
- dotenv (startup-only, zero risc securitate)
- tar (transitive, nu folosit in codepaths noastre)
- bytes BytesMut overflow (reqwest transitive, nu exploatabil)
- regex DoS (nu avem regex user-facing)

### 102.6 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 248 teste passed, 0 regressions
- `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok
- `cargo bench`: 6 benchmarks OK, toate sub 1ms
- 37 sesiuni consecutive fara regressions

---

## 103. ONLINE ORACLE WEIGHT ADAPTATION (S15) — Sesiunea 38

### 103.1 Ce este

Oracle Weight Adaptation e un sistem de auto-tuning care ajusteaza automat cele 5 ponderi directionale ale Oracle-ului (Kalman, Wavelet, Hurst, VPIN, Entropy) pe baza performantei reale a trade-urilor. Oracle-ul invata din propriile greseli si se auto-optimizeaza.

### 103.2 Problema rezolvata

Inainte de S15, ponderile Oracle erau statice (Kalman 0.30, Wavelet 0.25, Hurst 0.20, VPIN 0.15, Entropy 0.10), identice indiferent de regimul pietei sau de performanta reala a fiecarui layer. Un layer care prezice gresit luni de zile pastra aceeasi pondere ca unul care prezice corect. Nu exista feedback loop intre rezultatele trade-urilor si configurarea Oracle-ului.

### 103.3 Cum functioneaza

**La intrare in trade:**
1. Oracle calculeaza snapshot-ul complet (7 layere)
2. Se extrag cele 5 semnale directionale [-1, +1] (kalman_signal, wavelet_signal, etc.)
3. Se stocheaza impreuna cu regimul si confidence-ul la momentul intrarii

**La iesire din trade:**
1. Se recupereaza semnalele de la entry
2. Se calculeaza directia PnL (+1 profit, -1 loss)
3. Se actualizeaza corelatia EMA per-layer: `corr = corr * (1 - alpha) + (signal * direction) * alpha`
4. La fiecare 50 de trades, se recalculeaza ponderile adaptate

**La urmatorul entry:**
1. Se cauta ponderile adaptate pentru regimul CURENT al pietei
2. Oracle foloseste ponderile adaptate in loc de cele statice

### 103.4 Cele 3 upgrades smart

**Per-Regime Adaptation:** 4 seturi independente de ponderi, cate unul per MarketRegime (Trending, MeanReverting, Chaotic, Unknown). Kalman poate domina in trend dar Hurst in mean-reversion. Fiecare regim isi optimizeaza ponderile independent.

**Confidence-Weighted Learning:** `alpha_efectiv = alpha_baza * confidence_la_entry`. Trade-uri cu confidence mare (Oracle era sigur) contribuie mai mult la learning decat cele cu confidence mic. Filtreaza zgomotul din decizii nesigure.

**Continuous Decay:** `corr *= 0.997` aplicat la fiecare trade pe toate regimurile. Half-life de ~230 trades. Adaptarea "uita" treptat informatia veche, protejand impotriva overfitting-ului pe conditii de piata care nu mai sunt relevante.

### 103.5 Safety mechanisms

| Mecanism | Efect |
|----------|-------|
| Weight cap 0.5x - 2.0x | O pondere nu poate creste/scadea mai mult de 2x fata de baza |
| Normalizare sum = 1.0 | Ponderile adaptate insumate fac intotdeauna 1.0 |
| Toggle default OFF | `enable_oracle_adaptation` trebuie activat manual |
| Continuous decay | Previne overfitting pe secvente istorice |
| `layer_signals_at_entry` volatil | La restart, doar entry snapshots in-flight se pierd (nu corelatiile) |

### 103.6 Cum se activeaza

In tab-ul **HFT Engine**, sectiunea **Oracle Fusion**:
- Toggle **Enable Oracle Adaptation** = ON
- Setting key: `enable_oracle_adaptation`

Adaptarea incepe sa acumuleze date din primul trade dupa activare. Dupa primele 50 de trade-uri, ponderile se recalculeaza automat.

### 103.7 Persistenta

`OracleAdaptation` (corelatii per-regime, ponderi adaptate, trade_count) se salveaza in `titan_state_snapshot.json` prin `StateSnapshot`. La restart, datele acumulate se restaureaza automat (daca snapshot-ul are mai putin de 12 ore).

`layer_signals_at_entry` (semnalele la entry) NU se persista — sunt date volatile per-sesiune. Daca un trade e deschis la restart, acel trade nu va contribui la adaptare.

### 103.8 Interactiunea cu ponderile manuale

Ponderile din sliderul Oracle Fusion (Kalman W, Entropy W, etc.) devin **ponderile de baza** cand adaptarea e activa. Adaptarea le modifica relativ la aceste valori (0.5x - 2.0x). Daca schimbi slider-ul Kalman W de la 0.30 la 0.40, adaptarea va oscila intre 0.20 si 0.80 (nu 0.15 - 0.60).

Cand adaptarea e OFF, ponderile din slider se folosesc direct (comportament identic cu cel dinainte de S15).

### 103.9 Fisiere modificate (8)

- `oracle.rs` — OracleAdaptation struct, LayerEntrySnapshot, extract_layer_signals, 4 metode, 11 teste
- `lib.rs` (titan-indicators) — export OracleAdaptation + extract_layer_signals
- `Cargo.toml` (titan-indicators) — serde_json dev-dependency
- `autopilot.rs` — 2 entry hooks + 2 exit hooks
- `state.rs` — TitanState + StateSnapshot + save/restore + AppSettings
- `app_state.rs` — enable_oracle_adaptation + Default
- `settings.rs` — apply_setting_field match arm
- `main.rs` — TitanState init

### 103.10 Teste (11 noi, 259 total)

- adaptation_default_is_neutral
- record_outcome_updates_correct_regime
- weight_direction_follows_correlation
- weight_cap_enforced
- weights_normalized_to_one
- continuous_decay_shrinks_correlations
- confidence_weighted_ema
- serde_roundtrip_adaptation
- adapt_weights_no_data_returns_base
- fifty_trade_trigger
- extract_layer_signals_from_snapshot

### 103.11 Validari

- `cargo check`: 0 erori
- `cargo clippy --workspace -- -D warnings`: 0 warnings
- `cargo test --workspace`: 259 teste passed (60 risk + 53 indicators + 21 genetic + 120 main + 5 integration)
- `cargo bench -p titan-indicators`: oracle_calculate_200 = 898us (target <5ms), indicators_all_200 = 919us (target <2ms)
- 0 regressions, 38 sesiuni consecutive fara regressions

---

## 104. EXIT INFLIGHT GUARD — Protectie Anti-Loop la Iesiri (Sesiunile 39-40)

### 104.1 Problema

Autopilotul decide sa inchida o pozitie pe baza pozitiilor raportate de broker (`fetch_positions` via API Alpaca). Dupa ce `close_position` trimite un ordin de vanzare, loop-ul autopilot reporneste in ~500ms. Broker-ul poate avea nevoie de 1-5+ secunde sa reflecte pozitia inchisa in raspunsul API-ului, asa ca urmatorul `fetch_positions` inca returneaza pozitia — si autopilotul trimite un alt ordin SELL.

**Simptome observate in productie:**

```
19:28:07 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
19:28:07 ORDER ACCEPTED: SELL LAR x212.0000 LAR
19:27:52 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
19:27:52 ORDER ACCEPTED: SELL LAR x212.0000 LAR
19:27:42 AUTOPILOT EXIT: SELL LAR — PnL: $-21.20
... (12+ ordine identice in 42 secunde)
19:27:48 RECONCILIATION: 2 open positions synced from Alpaca
```

Fiecare ordin duplicate fie se executa (deschizand accidental o pozitie short), fie genereaza erori 429 (rate limit).

**Ce lipsea (S39):**
- `evaluate_position_exit` nu verifica daca un exit e deja in zbor
- `close_position` nu seteaza `PendingClose` sau vreun flag inflight
- `has_pending_orders_for_symbol` era folosit doar pentru gate-ul de intrare, nu si pe exit
- Cooldown-ul de intrare (`entry_cooldown_secs`) nu se aplica iesirilor

**Race conditions descoperite dupa S39 (rezolvate in S40):**
- Guard-ul TTL de 30s era resetat prematur de `trade_updates` (fill local) si `position_recon` (simboluri filtrate de grace/blacklist)
- Stergerea optimista a pozitiei din `settings.positions` cauza crearea unei pozitii sintetice la fill
- Nu exista verificare a ordinelor deschise la broker inainte de noi exit-uri

### 104.2 Solutia (V2 — Sesiunea 40)

**Doua straturi de protectie:**

1. **Guard inflight strict** — Camp pe `TitanState`:

```rust
pub exit_inflight: Mutex<HashMap<String, u64>>
```

Mapeaza `simbol -> timestamp_ms`. Daca un simbol exista in map, autopilotul nu trimite alt exit — **fara timeout**. Guard-ul ramane activ pana cand broker-ul confirma ca pozitia e flat (prin reconciliation) sau o eroare permanenta elimina intrarea.

2. **Guard broker open-order** — Inainte de `close_position`, autopilotul interogheaza `has_pending_orders_for_symbol()` (REST `GET /v2/orders?status=open&symbols=X`). Daca broker-ul are deja un ordin deschis pentru simbol, noul exit e sarit.

### 104.3 Flow-ul complet

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

### 104.4 Unde se seteaza / unde se sterge

| Locatie | Actiune | Fisier |
|---------|---------|--------|
| `close_position()` | **SET** — inainte de trimiterea ordinului | `commands/market_data.rs` |
| Position reconciliation | **CLEAR** — cand simbolul dispare din pozitiile brute ale broker-ului | `loops/position_recon.rs` |
| Autopilot permanent failure | **CLEAR** — la erori 422/invalid/not found | `loops/autopilot.rs` |

**Nu se curata in:**
- Autopilot exit success — pozitia e marcata `PendingClose`, guard-ul ramane activ
- Trade updates fill complet — fill-ul local nu reseteaza guard-ul (broker-ul poate inca raporta pozitia)

### 104.5 De ce guard strict fara TTL

- **Versiunea initiala (S39)** folosea un TTL de 30 secunde — dupa 30s, guard-ul expira si autopilotul putea retrimite. Dar 4 mecanisme puteau reseta prematur: trade_updates fill, position_recon filtrat, autopilot success clear, si lipsa verificarii la broker.
- **Versiunea actuala (S40)** elimina timeout-ul complet. Guard-ul ramane activ pana cand **broker-ul confirma ca pozitia e flat** (prin reconciliation la 15s). Aceasta este singura sursa de adevar fiabila.
- **Safety net:** Daca ceva merge foarte gresit (partitie de retea, API down), reconciliation la 15s va detecta disparitia simbolului si va curata guard-ul. Erorile permanente (422, not found) il curata imediat.
- **Open-order check** adauga un al doilea strat: chiar daca exit_inflight ar fi cumva resetat, broker-ul inca raporteaza ordinul deschis si autopilotul nu va trimite un duplicat.

### 104.6 Fisiere modificate

**S39 (structura de baza — 5 fisiere):**
- `src-tauri/src/state.rs` — camp `exit_inflight: Mutex<HashMap<String, u64>>` pe TitanState
- `src-tauri/src/main.rs` — initializare cu HashMap gol
- `src-tauri/src/loops/autopilot.rs` — guard inflight in `evaluate_position_exit` + clear la success si permanent failure
- `src-tauri/src/commands/market_data.rs` — set timestamp in `close_position` inainte de submit
- `src-tauri/src/loops/trade_updates.rs` — clear inflight la fill complet
- `src-tauri/src/loops/position_recon.rs` — clear stale inflight cand simbolurile dispar

**S40 (hardening — 4 fisiere, 6 schimbari):**
- `src-tauri/src/loops/autopilot.rs` — guard broker open-order + guard inflight strict (fara TTL) + PendingClose la success
- `src-tauri/src/loops/trade_updates.rs` — eliminat clear inflight la fill + suprimare pozitie sintetica pe exit
- `src-tauri/src/loops/position_recon.rs` — `exit_inflight.retain` foloseste pozitiile brute (nu `local_positions` filtrate)
- `src-tauri/src/commands/order_exec.rs` — `ORDER FILLED` redenumit in `ORDER ACCEPTED`

### 104.7 Validari

- `cargo check`: 0 erori, 0 warnings
- 0 linter errors
- 40 sesiuni consecutive fara regressions

### 104.8 PendingClose Lifecycle

Inainte de S40, autopilotul stergea pozitia din `settings.positions` imediat dupa ce `close_position` returna `Ok()`. Asta cauza doua probleme:

1. Cand fill-ul ajungea pe WebSocket, `trade_updates` nu gasea pozitia si crea una noua sintetica de tip Sell (falsa pozitie short)
2. Reconciliation re-insera pozitia ca `Open` dupa expirarea grace-ului daca broker-ul inca o raporta

**Solutia:** Autopilotul marcheaza pozitia `PendingClose` (cu `closed_at` setat) in loc sa o stearga. Asta permite:
- `trade_updates` sa gaseasca pozitia si sa o proceseze corect (reduce qty, sterge cand flat)
- Reconciliation sa pastreze starea `PendingClose` prin merge-ul cu datele existente
- Pozitia sa dispara natural cand broker-ul nu o mai raporteaza

### 104.9 ORDER ACCEPTED — Log Corect

Anterior, `send_order_internal` emitea mesajul `ORDER FILLED: SELL LAR x212.0000 LAR` pe raspunsul HTTP 200. Aceasta era inselator — HTTP 200 inseamna ca broker-ul a **acceptat** ordinul, nu ca s-a **executat** (fill-ul real vine pe WebSocket ca eveniment separat de tip `fill` sau `partial_fill`).

**Dupa S40:** Mesajul este `ORDER ACCEPTED: SELL LAR x212.0000 LAR`. Fill-urile reale apar ca `FILL: SELL LAR x212.0000 @ $X.XX [fill]` (emise de `trade_updates`).

---

## 105. CONFIGURATION COHERENCE — apply_runtime_config (Sesiunea 44)

Prima sesiune din Audit Roadmap. Rezolva problema desincronizarii intre setarile salvate (`AppSettings` / `AppState`) si campurile runtime din `TitanState`.

### 105.1 Problema rezolvata

Inainte de S44, existau **4 cai diferite** prin care setarile ajungeau in runtime (update_settings, batch_update, load_profile, main.rs startup), fiecare cu propria logica de sincronizare. Rezultat: unele campuri runtime (strategy_mode, dry_run, circuit breaker thresholds, flash crash params, nexus_fusion_alpha) puteau ramane desincronizate dupa anumite operatii.

In plus, `batch_update_settings` apela `save_settings_to_disk` **inainte** de verificarea erorilor — daca un camp individual esua, config partial corupta era deja persistata pe disk.

### 105.2 Solutia: apply_runtime_config()

O singura functie in `state.rs` care sincronizeaza **toate** campurile runtime din AppSettings in TitanState:

- **strategy_mode** — prin `StrategyMode::from_name()` (parsare centralizata, elimina match duplicat din 3 locuri)
- **nexus_fusion_alpha** — direct copy
- **dry_run** — combina `TITAN_DRY_RUN` env var cu `settings.dry_run_enabled`
- **circuit_breaker** — `update_thresholds(caution, halt, panic)` cu Decimal conversion
- **flash_crash** — `reconfigure(window_secs, threshold_pct, cooldown_mins)` fara pierderea sliding window history

### 105.3 Transactional batch_update_settings

`batch_update_settings` foloseste acum un mecanism de snapshot + rollback:

1. **Snapshot** — clone complet al AppState inainte de modificari
2. **Apply** — aplica fiecare camp individual; daca oricare esueaza, restaureaza snapshot-ul
3. **Persist** — save pe disk si Redis side effects doar pe success path
4. **Runtime sync** — `apply_runtime_config` apelat o singura data la final

### 105.4 Comanda noua: apply_full_settings

Comanda Tauri `apply_full_settings` accepta un `AppSettings` direct ca JSON typed. Folosita de:

- **Import** (din `titan_perf_stats.js`) — trimite payload-ul JSON direct, fara `String(v)` coercion
- **Undo** — la fel, trimite AppSettings complet

Include `validate_settings_coherence()` — validare cross-field:
- breaker_caution > breaker_halt > breaker_panic (ordering)
- stop_loss, take_profit > 0
- risk_per_trade intre 0-100%
- hold_time_max >= hold_time_exit

### 105.5 FlashCrashDetector::reconfigure()

Metoda noua pe `FlashCrashDetector` (in `candle_buffer.rs`) care actualizeaza parametrii (window duration, threshold %, cooldown) **fara** a sterge istoricul de preturi din sliding windows. Inainte, singura optiune era `clear()` care pierdea toata istoria.

### 105.6 Import profile hardened

`import_profile` verifica acum:
- **schema_v** — versiunea schemei din profil vs `CURRENT_PROFILE_SCHEMA_V`
- **Round-trip serde** — serializeaza + deserializeaza payload-ul; daca esueaza, respinge importul
- **Coherence warnings** — logheaza orice avertizari din `validate_settings_coherence()`

### 105.7 Teste de regresie (4)

| Test | Ce verifica |
|------|-------------|
| `settings_round_trip_coherence` | AppSettings -> JSON -> AppSettings pastreaza toate campurile |
| `strategy_mode_from_name_covers_all_variants` | Toate variantele StrategyMode sunt acoperite de from_name |
| `flash_crash_reconfigure_preserves_windows` | reconfigure() nu pierde sliding window history |
| `validate_coherence_catches_bad_breakers` | Breaker ordering gresit e detectat |

### 105.8 Fisiere modificate

| Fisier | Schimbare |
|--------|-----------|
| `state.rs` | apply_runtime_config(), StrategyMode::from_name(), 4 teste |
| `candle_buffer.rs` | FlashCrashDetector::reconfigure() |
| `commands/settings.rs` | batch transactional, apply_full_settings, validate_coherence, import hardened |
| `commands/strategy_cmd.rs` | from_name() inlocuieste match duplicat |
| `main.rs` | from_name() la startup + register apply_full_settings |
| `titan_perf_stats.js` | Import/undo folosesc apply_full_settings (fara String coercion) |
| `.cursor/rules/titan-cyber-hud.mdc` | Design system rule (NOU) |

### 105.9 Cyber-HUD Design System Rule

Bonus: creat `.cursor/rules/titan-cyber-hud.mdc` — codifica design system-ul existent din `styles.css` (37 CSS custom properties, component patterns, animatii). Orice modificare frontend viitoare respecta automat acest contract vizual. Include: color tokens, typography, component patterns (`.cyber-panel`, `.titan-btn-*`, `.titan-toast-*`), reguli stricte (zero inline styles, max 6 glow effects pe ecran, font-uri obligatorii).

---

## 106. SMARTTREND AI v3 — FSM + ENGINE + LIVE PANEL (S07b) — Sesiunea 55

### 106.1 Ce este

SmartTrend AI v3 este un sistem de trend intelligence cu 3 piloni (Macro, Micro, Momentum) orchestrat de o masina de stari finita (FSM). In S07a am construit pilonii si BayesianTracker. In S07b am adaugat FSM-ul complet, motorul SmartTrendEngine, wiring-ul in settings, comanda Tauri, si panelul UI.

### 106.2 Masina de Stari (FSM)

SmartTrend opereaza prin 5 stari care modeleaza ciclul unui trade:

| Stare | Descriere | Culoare badge |
|-------|-----------|---------------|
| **Flat** | Fara pozitie, fara semnal activ | Dim (gri) |
| **Stalking** | Semnal detectat, asteptam confirmare | Gold |
| **Entry** | Confirmat, marker emis, pozitie in curs de deschidere | Cyan (flicker) |
| **InTrade** | Pozitie deschisa, monitorizare activa | Green |
| **Trail** | Semnal slabeste, trailing stop activ | Magenta (breathe) |

**Tranzitii:**
- **Flat -> Stalking**: directional strength >= stalking threshold SI directie non-Neutral
- **Stalking -> Entry**: strength >= entry threshold SI confirmation_bars >= confirmation_needed SI pilonii sunt aliniati (fara cross-divergence)
- **Stalking -> Flat**: strength scade sub threshold (cu hysteresis) SAU timeout (2x confirmation_needed)
- **Entry -> InTrade**: automat pe urmatorul evaluate()
- **InTrade -> Trail**: directional strength scade sub entry * 0.8
- **Trail -> Flat**: directia se inverseaza (exit marker emis + BayesianTracker updated)
- **Orice stare -> Flat**: changepoint reset (cand changepoint_prob >= 0.7)

### 106.3 Motorul SmartTrendEngine

`SmartTrendEngine::evaluate()` este punctul central. Primeste un `SmartTrendInput` si returneaza `SmartTrendResult`:

**Input:** macro_input, micro_input, momentum_input, current_price, changepoint_prob, dominant_cycle, tf_minutes

**Output:** score (0-100), state, prev_state, direction, markers (vec), pillar_scores, bars_in_state, confirmation_progress

**Flow intern:**
1. Evalueaza cei 3 piloni via `evaluate_pillars()`
2. Verifica changepoint reset (force Flat + Exit marker daca era InTrade/Trail)
3. Ruleaza tranzitiile FSM bazate pe directional_strength
4. Emite markers la fiecare tranzitie
5. Actualizeaza BayesianTracker la exit-uri (success/failure)

### 106.4 Dynamic Confirmation Window

Fereastra de confirmare (cate bare trebuie sa ramana scorul deasupra entry threshold in Stalking) se adapteaza dinamic:

```
confirmation_needed = max(2, dominant_cycle / tf_minutes)
```

- dominant_cycle vine din wavelet (OracleHistory.dominant_cycle_series)
- tf_minutes se deduce din timeframe (1m=1, 5m=5, etc.)
- Fallback: st_stabilization_window (default 8) daca cycle indisponibil
- Scurtare proportionala cu strength: Extreme -25%, Strong -15%

### 106.5 Changepoint-Aware Reset

Cand probabilitatea de changepoint depaseste 0.7 (configurable `st_changepoint_reset_threshold`):

- Force transition la Flat, indiferent de starea curenta
- Daca era in InTrade/Trail, emite Exit marker
- Reset BayesianTracker la prior uninformative
- Emite ChangePointReset marker cu detalii
- Incrementeaza stats.changepoint_resets

### 106.6 Bonusuri implementate

| Bonus | Descriere | Config |
|-------|-----------|--------|
| **Trail ATR Auto-Tighten (B1)** | In Trail, stop se strange cu 5% per bara | `st_trail_tighten_pct` (0.05) |
| **Pillar Agreement Gate (B2)** | Entry necesita 2/3 piloni aliniati (fara cross-divergence) | cross_divergence din PillarDiagnostics |
| **FSM Stats Tracking (B3)** | Tranzitii per tip, bare per stare, hit rate, resets | SmartTrendStats in diagnostics |
| **Hysteresis Anti-Chatter (B4)** | Stalking->Flat necesita drop sub threshold - margin | `st_hysteresis_margin` (5.0) |

### 106.7 SmartTrend Live Panel (UI)

Panou cyber-HUD in sidebar (sub MICROSTRUCTURE), vizibil doar cand `st_enabled = true`:

```
+--[ SMARTTREND LIVE ]-------------------+
|  STATE: [FLT] [STK] [ENT] [TRD] [TRL] |
|  DIR: LONG    SCORE: 67.2  STR: Mod    |
|  BARS: 4/8 [========>           ]      |
|  M:62  m:71  P:58   AGREE             |
+----------------------------------------+
```

- **STATE**: 5 badge-uri (una activa cu glow corespunzator starii)
- **DIR**: LONG (green) / SHORT (red) / -- (dim)
- **SCORE**: valoare cyan
- **STR**: Weak/Moderate/Strong/Extreme cu culoare corespunzatoare
- **BARS**: progress bar confirmation (vizibil doar in Stalking)
- **M/m/P**: scoruri piloni Macro/micro/Momentum, cu indicator AGREE/DIVERGE
- **KILLED**: badge rosu pulsant cand st_killed == true
- Poll: 3 secunde via `get_smarttrend_diagnostics`

### 106.8 Configuratie SmartTrend

Toate setarile SmartTrend sunt persistate in AppSettings si editabile via batch update. Campuri noi adaugate in S07b:

| Camp | Default | Descriere |
|------|---------|-----------|
| `st_changepoint_reset_threshold` | 0.7 | Prag changepoint pentru force reset la Flat |
| `st_trail_tighten_pct` | 0.05 | Tighten trailing stop per bara in Trail |
| `st_hysteresis_margin` | 5.0 | Margine anti-chatter Stalking->Flat |

Campuri existente din S07a: `st_macro_weight`, `st_micro_weight`, `st_momentum_weight`, `st_entry_threshold`, `st_stalking_threshold`, `st_entry_thresholds[4]`, `st_stabilization_window`, `st_bayesian_decay`, `st_bayesian_halflife`, `st_dominant_cycle_filter`, `st_ofi_norm_range`, `st_mp_bps_range`, `st_cvd_scale`, `st_enabled`, `st_killed`.

### 106.9 Fisiere modificate

| Fisier | Schimbare |
|--------|-----------|
| `smarttrend.rs` | FSM, Engine, markers, stats, 17 teste, 3 config fields |
| `app_state.rs` | smarttrend: SmartTrendConfig + default |
| `state.rs` | AppSettings, settings_from_state, apply_settings, apply_runtime_config |
| `commands/settings.rs` | apply_smarttrend_field() cu 21 campuri |
| `commands/market_data.rs` | get_smarttrend_diagnostics command |
| `main.rs` | register command, SmartTrendEngine init |
| `index.html` | SmartTrend Live panel HTML |
| `styles.css` | SmartTrend panel CSS cyber-HUD |
| `titan_bootstrap.js` | initSmartTrendPanel() polling + rendering |

---

## 107. SMARTTREND SCORING + KILL SWITCH (S08) — Sesiunea 56

### 107.1 StrategyMode::SmartTrend

Varianta noua adaugata la `enum StrategyMode` in `state.rs`:

| Varianta | Display | Parse |
|----------|---------|-------|
| `SmartTrend` | `SMARTTREND` | "SMARTTREND", "SMART_TREND", "ST" |

Toggle cycle: MathOracle -> GpuNexus -> Fusion -> SignalStrategy -> **SmartTrend** -> MathOracle

### 107.2 Scoring Pipeline SmartTrend

In `get_signal_markers_internal()`, cand `mode == SmartTrend`:

1. Se instantiaza `SmartTrendEngine` temporar cu config din settings
2. Se ruleaza `evaluate()` pe fiecare bara history (warmup 50 bare)
3. Se construiesc inputuri Macro/Micro/Momentum din oracle history, GPU scores, si candle data
4. Markers FSM sunt convertite in `SignalMarker` format:

| SmartTrend Marker | Chart Shape | Color | Text |
|-------------------|-------------|-------|------|
| Entry (Bullish) | arrowUp | `#00E5FF` | ST BUY |
| Entry (Bearish) | arrowUp | `#00E5FF` | ST SELL |
| Exit | arrowDown | `#FFD700` | ST EXIT |
| Trail | circle | `#BF00FF` | ST TRAIL |
| Stalking | circle | `#FFD700` | ST STALK |
| ChangePointReset | circle | `#FF8800` | ST BREAK |

SmartTrend mode skips the default exit signal generation (FSM handles exits).

### 107.3 Regime-Adaptive Entry Colors (B4)

Entry markers isi schimba intensitatea culorii proportional cu regime-ul piata:

| Regime | Color |
|--------|-------|
| Trending | `#00E5FF` (full) |
| MeanReverting | `rgba(0,229,255,0.6)` |
| Chaotic | `rgba(0,229,255,0.5)` |
| Unknown | `rgba(0,229,255,0.4)` |

### 107.4 Kill Switch

**Backend:**
- `st_killed = true` => SmartTrend nu produce markers, autopilot blocheaza entries
- Persistent pe disk in `SmartTrendConfig.st_killed`
- Re-activare necesita confirm token `CONFIRM_ST_REACTIVATE`
- Autopilot guard: cand `mode == SmartTrend && st_killed == true`, skip all entries cu log warn

**Tauri Commands (smarttrend_cmd.rs):**

| Command | Parametri | Returneaza |
|---------|-----------|------------|
| `smarttrend_kill_switch` | `enable: bool, confirm_token: Option<String>` | `String` |
| `get_smarttrend_config` | -- | `SmartTrendConfig` |
| `update_smarttrend_config` | `field: String, value: String` | `String` |
| `get_smarttrend_session_stats` | -- | `SmartTrendStats` |
| `get_smarttrend_state` | -- | `SmartTrendDiagnostics` |

### 107.5 UI Kill Switch + Mode Button

**Kill Switch button** in SmartTrend Live panel:
- Buton rosu "KILL" cu confirm dialog (TitanToast.confirm)
- Cand killed: text "REACTIVATE", culoare verde, re-enable necesita double confirm
- CSS cyber-HUD cu hover glow si transitions

**Strategy mode selector** — buton "ST" adaugat in nexus-mode-toggle:
- Activ: cyan border + glow
- CSS specific: `nexus-mode-btn[data-mode="SMARTTREND"]`

### 107.6 Score Overlay (B1)

SmartTrend scores (0-100) sunt incluse in `SignalMarkersResponse.st_scores`:
- Struct `StScorePoint { time: i64, score: f64, state: String }`
- Frontend `_renderStScoreOverlay()` renders scores pe oracle chart overlay
- Score centrat la 0 (score - 50) pentru vizualizare baseline

### 107.7 Quick-Stats Banner (B2)

Banner persistent sub SmartTrend Live panel:

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

Cand user schimba StrategyMode spre/din SmartTrend:
- Toast cyber-HUD cu status: "SMARTTREND ACTIVATED — Engine: READY/KILLED/DISABLED"
- Audio alert: `TitanAudio.play('mode_change')`
- La deactivare: "SMARTTREND DEACTIVATED — Switched to {mode}"

### 107.9 Teste noi S08

19 teste noi (395 total):
- 5 StrategyMode: parse, display, roundtrip, toggle_cycle, serde_roundtrip
- 4 scoring: exit_marker_smarttrend, exit_marker_rejects_st_entries, smarttrend_scoring_neutral, smarttrend_colors
- 6 SmartTrend engine: kill_switch_default, kill_switch_serde, killed_engine_state, strong_signal_markers, marker_kinds, diagnostics_stats
- 4 commands: kill_switch_token, config_defaults, config_toggle, stats_empty

---

## 108. SMARTTREND SENTINEL LOOP + ALERTE (S09) — Sesiunea 57

### 108.1 Sentinel Loop Backend

Fisier nou: `src-tauri/src/loops/smarttrend_sentinel.rs`

Loop-ul sentinel scaneaza continuu toate asset-urile active (pozitii + HFT targets + signal assets), apeleaza `SmartTrendEngine::evaluate()` pe fiecare cu date live reale, detecteaza tranzitii FSM si emite alerte.

**Cadenta adaptiva per-symbol:**

| Stare FSM | Interval default | Config field |
|-----------|-----------------|--------------|
| Flat | 30s | `st_sentinel_idle_secs` |
| Stalking | 5s | `st_sentinel_stalking_secs` |
| Entry/InTrade/Trail | 2s | `st_sentinel_hot_secs` |

Loop-ul doarme dupa cea mai scurta cadenta din toti simbolii monitorizati.

**Date live assemblate:**

| Input | Sursa | Lock type |
|-------|-------|-----------|
| Oracle score/confidence/regime | `indicator_cache.last_oracle_snapshot()` | Mutex |
| OFI, microprice, spread, imbalance | `market.ofi_state` | ArcSwap (lock-free) |
| CVD | `market.cvd` | Mutex |
| GPU consensus | `nexus_consensus` | Mutex |
| Pret curent | `market.last_prices` | Mutex |

**Integrare watchdog:**
- Registered in `watchdog.register("smarttrend_sentinel")` la startup
- Heartbeat la fiecare iteratie cu tick duration
- `spawn_loop_by_name("smarttrend_sentinel")` in watchdog_loop.rs
- Generation ID check: loop-ul iese daca generatia curenta < boot generation

### 108.2 Symbol Priority Queue (B2)

In loc de iteratie liniara, simbolurile sunt evaluate prin `BinaryHeap<SymbolPriority>`:

| State | Rank | Prioritate |
|-------|------|-----------|
| InTrade | 4 | Cel mai inalt |
| Entry/Trail | 3 | |
| Stalking | 2 | |
| Flat | 1 | Cel mai scazut |

In cadrul aceluiasi rank, simbolurile cu scor mai mare sunt evaluate primele. Reduce latenta de reactie pe simbolurile critice.

### 108.3 Micro-Latency Tracking (B1)

Fiecare `evaluate()` este cronometrat cu `Instant::now()` / `elapsed()`:
- `last_evaluate_us` stocat per-symbol in tracker
- `tracing::warn!` emis daca durata > 5ms (indicator cache miss sau lock contention)
- Media propagata prin `sentinel-badge` Tauri event

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

Backend emite severity in payload-ul `smarttrend-alert`. Cooldown diferentiat: INFO respecta per-symbol cooldown (default 30s), WARNING/CRITICAL emit mereu.

### 108.5 Alerte Audio (6 sunete noi)

| Sunet | Event | Caracter |
|-------|-------|---------|
| `st_stalking` | Flat->Stalking | Ton ascendent scurt |
| `st_entry` | Entry confirmed | Dublu beep rapid |
| `st_exit` | Exit signal | Ton descendent |
| `st_trail` | Trail activat | Ton metalic scurt |
| `st_changepoint` | Regime break | Glitch/static burst |
| `st_kill` | Kill switch | Alarm scurt |

Toate generate via Web Audio API (oscillator + gain envelope), fara fisiere externe. Cooldown 30s per-symbol + event-type in frontend.

### 108.6 Alert Log UI

Container `#st-alert-log` sub SmartTrend Live panel:

```
+--[ ALERT LOG ]--[ 245µs ]--[I][W][C]--[×]--+
| TIME     SYM   EVENT  SCORE  DIR            |
| 14:23:05 AAPL  ENTRY  72.3   LONG           |
| 14:22:51 TSLA  STALK  56.8   SHORT          |
| 14:20:12 NVDA  EXIT   45.1   --             |
+--------------------------------------------- +
```

- Max 20 alerte LIFO
- Severity toggles (I/W/C) in header — filtreaza rendering si audio/toast
- Clear button (x) sterge log-ul
- Row border-left per severity: cyan (INFO), gold (WARNING), red (CRITICAL)
- Marker CSS classes: `mk-stalking`, `mk-entry`, `mk-exit`, `mk-trail`, `mk-changepoint`

### 108.7 Dashboard Badge (B3)

Badge persistent in header-right area:

```
[ 5 | 2 ACT | TSLA InTrade ]
```

- Total simboluri monitorizate
- Simboluri active (non-Flat)
- Cel mai fierbinte simbol + stare (cu `data-flicker` animation)
- Updated via `sentinel-badge` Tauri event la fiecare loop tick

### 108.8 Alerte Vizuale

- **Toast dedicat SmartTrend**: `titan-toast-smarttrend` class cu border-left per-marker
- **Screen glow**: WARNING = gold, CRITICAL = red (via `showScreenGlow()`)
- **Title flash**: `[ST ENTRY] AAPL` pe Entry/Exit (10s)
- **Badge pulsant**: `st-alert-active` class pe SmartTrend panel (3s animation)

### 108.9 Config Fields Noi

| Camp | Default | Clamp | Descriere |
|------|---------|-------|-----------|
| `st_sentinel_idle_secs` | 30 | 5-300 | Cadenta Flat |
| `st_sentinel_stalking_secs` | 5 | 1-60 | Cadenta Stalking |
| `st_sentinel_hot_secs` | 2 | 1-30 | Cadenta Entry/InTrade/Trail |
| `st_sentinel_alert_cooldown_secs` | 30 | 5-600 | Cooldown per-symbol INFO alerts |

Backward compatible — fisiere settings vechi fara aceste campuri folosesc valorile default via `#[serde(default)]`.

### 108.10 Tauri Commands Noi

| Command | Return | Descriere |
|---------|--------|-----------|
| `get_sentinel_status` | `SentinelStatus` | Starea sentinel: running, symbols, badge |

### 108.11 Fisiere

| Fisier | Modificare |
|--------|-----------|
| `loops/smarttrend_sentinel.rs` | NOU: sentinel loop + 10 teste |
| `loops/mod.rs` | Module registration |
| `loops/watchdog_loop.rs` | spawn_loop_by_name branch |
| `smarttrend.rs` | 4 config fields + serde defaults |
| `indicator_cache.rs` | `last_oracle_snapshot()` + `OracleSnapshot` |
| `commands/smarttrend_cmd.rs` | `get_sentinel_status` command |
| `commands/settings.rs` | 4 campuri in apply_smarttrend_field |
| `main.rs` | watchdog register + spawn loop + command |
| `titan_audio.js` | 6 sunete sintetice noi |
| `titan_bootstrap.js` | Alert listener, log rendering, badge, severity |
| `index.html` | Alert log container, dashboard badge, severity toggles |
| `styles.css` | Alert log CSS, badge, severity, toast SmartTrend |

### 108.12 Teste noi S09

10 teste noi (405 total):
- `sentinel_cadence_idle_30s`, `sentinel_cadence_stalking_5s`, `sentinel_cadence_hot_2s`
- `sentinel_state_transition_detection`
- `sentinel_cooldown_suppresses_repeat`
- `sentinel_killed_skips_evaluate`
- `sentinel_severity_classification`
- `sentinel_priority_queue_ordering`
- `sentinel_badge_empty_trackers`
- `sentinel_alert_severity_values`

---

## 109. SMARTTREND BOT MARKER-DRIVEN (S10) — Sesiunea 58

### 109.1 SmartTrendAccount — Virtual Sub-Account

Struct `SmartTrendAccount` in `smarttrend.rs` — capital tracking separat pentru SmartTrend trades:

| Camp | Tip | Descriere |
|------|-----|-----------|
| `allocated_capital` | f64 | Capital curent (creste/scade cu PnL + topup/withdraw) |
| `initial_capital` | f64 | Capital la inceputul zilei (reset daily) |
| `realized_pnl_today` | f64 | PnL realizat in sesiunea curenta |
| `total_realized_pnl` | f64 | PnL total cumulat |
| `high_water_mark` | f64 | Cel mai mare capital atins |
| `max_drawdown_pct` | f64 | Drawdown maxim ca procent |
| `trade_count_today` | u32 | Numar trade-uri azi |
| `win_count_today` | u32 | Numar win-uri azi |
| `loss_count_today` | u32 | Numar loss-uri azi |
| `total_trades` | u64 | Total trade-uri all-time |
| `total_wins` | u64 | Total win-uri all-time |
| `avg_win` | f64 | Media castigurilor (running) |
| `avg_loss` | f64 | Media pierderilor (running) |
| `paused` | bool | Bot paused flag |
| `breaker_triggered` | bool | Circuit breaker fired |

Metode cheie: `record_fill(pnl)`, `drawdown_pct()`, `hit_rate()`, `kelly_fraction()`, `compute_size_usd()`, `daily_reset()`.

Persistenta: `data/titan_st_account.json` cu .bak backup automat.

### 109.2 Config Fields S10

7 campuri noi in `SmartTrendConfig` (toate cu `#[serde(default)]` backward-compatible):

| Camp | Default | Descriere |
|------|---------|-----------|
| `st_risk_per_trade` | 0.01 | Fractiune din capital per trade (1%) |
| `st_max_dd_pct` | 5.0 | Drawdown maxim inainte de circuit breaker |
| `st_bot_loop_ms` | 2000 | Cadenta bot loop in ms |
| `st_bot_paused` | true | Bot porneste paused |
| `st_kelly_enabled` | false | Activeaza Kelly sizing |
| `st_kelly_min_trades` | 20 | Minimum trades inainte de Kelly |
| `st_max_kelly_fraction` | 0.25 | Cap Kelly fraction |

### 109.3 SmartTrend Bot Loop

Fisier NOU: `src-tauri/src/loops/smarttrend_bot.rs`

**Arhitectura:** Loop dedicat (NU autopilot) care citeste starea engine-ului SmartTrend din sentinel si executa ordine pe baza marker-elor FSM.

**Fluxul principal:**
1. Guard: skip daca mode != SmartTrend, killed, paused
2. ST Circuit Breaker: daca drawdown > st_max_dd_pct → force exit ALL + pause + emit alert
3. Read engine snapshot (state, direction, score, transition)
4. Entry: pe SmartTrendStateKind::Entry cu no existing position → compute_size_usd → send_order_internal cu tag `ST|AUTO|{symbol}|{ts}`
5. Exit: pe Flat transition din Trail/InTrade → close_position
6. Persist account periodic + dedup actions
7. Adaptive cadence: fast (1s) cu pozitii, slow (3s) fara

**Safety mechanisms:**
- Generation ID check (stale loop exits)
- Failed order cooldown tracking
- Action dedup (60s window)
- Circuit breaker cu force exit + auto-pause

### 109.4 Trade Updates — ST Fill Detection

In `trade_updates.rs`:
- La fiecare fill, verifica `entry_tag.starts_with("ST|")`
- Daca fill e closing (contra-direction): calculeaza PnL, apeleaza `SmartTrendAccount::record_fill(pnl)`, salveaza pe disk
- Emite event `smarttrend-fill` cu `{ symbol, side, qty, price, tag, is_closing }`
- Log structurat `[ST FILL]`

### 109.5 Tauri Commands S10

5 comenzi noi registered in `invoke_handler`:

| Comanda | Parametri | Guard | Efect |
|---------|-----------|-------|-------|
| `smarttrend_topup` | `amount: f64` | `amount <= buying_power` | Creste allocated_capital |
| `smarttrend_withdraw` | `amount: f64` | `amount <= available - notional_open` | Scade allocated_capital |
| `smarttrend_pause` | `paused: bool` | - | Toggle st_bot_paused + persist |
| `smarttrend_account_stats` | - | - | Returneaza SmartTrendAccount JSON |
| `smarttrend_force_exit_all` | `confirm_token: String` | `"CONFIRM_ST_LIQUIDATE"` | Force close ALL ST AUTO positions |

### 109.6 UI — SmartTrend Account Panel

Panel `#st-account-panel` (`.cyber-panel`) cu:
- **Metrics:** Capital, PnL Today (verde/rosu), Total PnL, Drawdown %, Drawdown bar vizuala cu threshold marker, Hit Rate, Trades Today/Total
- **Status badge:** ACTIVE (cyan) / PAUSED (orange) / BREAKER (red pulsant)
- **Controale:** Input amount + TOPUP + WITHDRAW + PAUSE/RESUME + FORCE EXIT ALL (cu confirm dialog)
- **Polling:** 5s interval + update imediat pe `smarttrend-fill` event

### 109.7 Bonusuri S10

**B1: Live Micro Inputs** — Sentinel `assemble_macro_input()` foloseste acum date reale din oracle snapshot (kalman_gain, price_surprise, changepoint_prob, run_length) in loc de placeholders fixe. 4 campuri noi adaugate in `Indicators` struct si propagate prin `merge_streaming_oracle`.

**B2: Daily Reset** — `daily_reset.rs` apeleaza `SmartTrendAccount::daily_reset()` la 09:30 ET. Zero pnl_today, trade_count_today, win_count_today, loss_count_today. Ajusteaza initial_capital la allocated_capital curent. Emite `smarttrend-daily-reset` event.

**B3: Sentinel-to-Bot Bridge** — Cand sentinel detecteaza marker Entry, apeleaza `bot_wake().notify_one()`. Bot loop foloseste `sleep_or_wake()` cu `tokio::select!` pe sleep + Notify. Reduce latenta entry de la ~2s (cadenta normala) la ~0s.

**B4: ST Positions Badge in Header** — Badge `#st-positions-badge` arata "ST: 3 +$45.20" (verde) sau "ST: 0 -$12.00" (rosu). Se actualizeaza pe fiecare `smarttrend-fill` event + polling 5s.

### 109.8 Teste S10

20 teste noi (274 total):
- SmartTrendAccount: `st_account_new_sets_capital`, `st_account_record_win`, `st_account_record_loss`, `st_account_drawdown_pct`, `st_account_hit_rate`, `st_account_avg_win_loss_ratio`, `st_account_daily_reset`, `st_account_serde_roundtrip`
- Kelly: `st_account_kelly_insufficient_trades`, `st_account_kelly_positive_edge`, `st_account_kelly_capped`
- Sizing: `st_account_compute_size_usd_base`, `st_account_compute_size_usd_kelly_boost`
- Config: `st_config_s10_defaults`, `st_config_s10_serde_roundtrip`, `st_config_s10_backward_compat`, `st_config_s10_validation_warnings`
- Bot: `action_kind_equality`, `engine_snapshot_defaults`, `now_epoch_ms_is_reasonable`

---

## 110. S11: AUTO/MAN Per Asset + Trade Panels

### 110.1 Per-Asset AUTO/MAN Mode

SmartTrend suporta modul de executie per-simbol: fiecare asset poate fi configurat individual ca **AUTO** (executie automata de catre bot) sau **MAN** (executie manuala prin trade panel).

**Backend:**
- `st_asset_modes: HashMap<String, String>` in `AppState` / `AppSettings` — valori: `"auto"` / `"man"`, default `"man"` (orice asset nesetat se considera MAN)
- Persistenta automata pe disk via `save_settings_to_disk()` la fiecare schimbare
- Integrat in `apply_settings()` si `settings_from_state()`

**Tauri Commands S11:**

| Command | Params | Guard | Descriere |
|---------|--------|-------|-----------|
| `smarttrend_get_asset_modes` | - | - | Returneaza HashMap complet |
| `smarttrend_set_asset_mode` | `symbol: String, mode: String` | AUTO->MAN blocked daca pozitie AUTO live | Set mode per asset |
| `smarttrend_set_all_modes` | `mode: String, confirm_token: Option<String>` | ALL AUTO cere `CONFIRM_ST_ALL_AUTO` | Set ALL assets la un mod |
| `smarttrend_get_trade_panel_data` | `symbol: String` | - | Assembleaza date complete (price, score, sizing, Kelly, pillars, mode) |

### 110.2 Bot Loop — Filtrare pe Asset Mode

- Bot entry gate: citeste `st_asset_modes` pentru simbolul activ; **skip entry daca `mode == "man"`**
- Tag distinctiv: `ST|AUTO|{symbol}|{ts}` vs `ST|MAN|{symbol}|{ts}`
- Circuit breaker: forteaza exit doar pe pozitii `ST|AUTO` (MAN — user gestioneaza)
- `force_exit_all_st_positions_filtered(auto_only: bool)` — `true` = doar AUTO, `false` = toate ST

### 110.3 Sentinel Events S11

- Pe marker Entry + asset mode MAN: emite `smarttrend-trade-prompt` cu payload complet (symbol, direction, score, state, price, sizing suggestion, Kelly, pillars, mode)
- Pe marker Exit/Trail + pozitie MAN deschisa: emite `smarttrend-exit-prompt` cu (symbol, marker, direction, score, severity)

### 110.4 UI — Trade Panel SMARTTREND TRADE

Panel `#st-trade-panel` (`.cyber-panel`):
- Apare automat pe `smarttrend-trade-prompt` event (asset MAN)
- Auto-populate: Symbol, Direction badge (LONG verde / SHORT rosu), Score, State FSM, Pillar scores, Price
- 3 moduri sizing (toggle): **USD** / **SHARES** / **%CAP**
- Quick amounts: 100 / 250 / 500 / 1K
- Input manual sizing
- TP/SL optional (toggles + inputs)
- **EXECUTE**: invoca `submit_order` cu tag `ST|MAN|{symbol}|{ts}`, toast confirmare
- **SKIP**: dismiss panel, log, toast
- Audio `st_entry` la aparitie
- Countdown timer 60s -> 0s cu auto-SKIP la expirare

### 110.5 UI — Exit Signal Panel

Panel `#st-exit-panel` (`.cyber-panel`):
- Apare pe `smarttrend-exit-prompt` event (pozitie MAN cu marker Exit/Trail)
- Informatii: Symbol, PnL curent, Durata pozitie, Motivul exit (marker type)
- **CLOSE 100%** / **CLOSE 50%** / **CLOSE 25%** / **HOLD**
- HOLD: dismiss cu log, reapare la urmatorul marker
- Audio `st_exit` la aparitie

### 110.6 UI — ST Assets Table

Panel `#st-assets-panel` (`.cyber-hud-table`):
- Coloane: SYMBOL / MODE / STATE / SCORE / DIR / LAST ALERT / PnL
- MODE: badge interactiv `[AUTO]` (verde) / `[MAN]` (portocaliu)
  - Click AUTO->MAN: instant, fara confirmare
  - Click MAN->AUTO: cu `TitanToast.confirm()` ("Activezi trading automat pe {SYMBOL}?")
  - Guard: asset AUTO cu pozitie live nu poate trece la MAN
- Header: **ALL AUTO** / **ALL MAN** buttons
- Sort: pozitii LIVE primele, apoi dupa stare FSM (InTrade > Trail > Entry > Stalking > Flat)
- Poll la 5s
- Tooltip pe hover: ultimele 5 trade-uri ST (B3)

### 110.7 Keyboard Shortcuts S11

| Shortcut | Trade Panel | Exit Panel |
|----------|-------------|------------|
| **Enter** | EXECUTE | CLOSE 100% |
| **Escape** | SKIP | HOLD |
| **V** | VIEW CHART | VIEW CHART |
| **1/2/3/4** | Quick sizing presets | - |

Guard: shortcuts active doar cand panel ST vizibil (nu interfereaza cu restul).

### 110.8 Position Markers

Pozitii SmartTrend afiseaza tag distinctiv:
- `[ST] AUTO` — badge verde, border verde
- `[ST] MAN` — badge portocaliu, border portocaliu
- CSS: `.st-pos-tag`, `.st-pos-auto`, `.st-pos-man`

### 110.9 Bonusuri S11

**B1: Countdown Timer + Urgency Indicator**
- Barra progres pe trade panel: 60s -> 0s
- Culoare: cyan (60-30s) -> gold (30-10s) -> red (10-0s) cu pulse animation
- La 0s: auto-SKIP cu toast "Trade opportunity expired"

**B2: Smart Sizing Suggestion**
- Sizing recomandat: `risk_per_trade * allocated_capital`, adjust Kelly daca activat
- Display: "SUGGESTED: $350 (1.4% risk, Kelly 0.12)"
- Pre-select in input, user poate modifica

**B3: Trade History per Asset Tooltip**
- Hover pe row din ST Assets Table
- Ultimele 5 trade-uri ST pe acel asset: date, direction, PnL, mode (AUTO/MAN)
- Hit rate per asset

**B4: Mode Change Audit Trail**
- `tracing::info!("[ST MODE] {symbol}: {old} -> {new} (user)")` pe fiecare schimbare
- Event `smarttrend-alert` cu marker `MODE_CHANGE` in Alert Log
- Persistenta imediata pe disk

### 110.10 Teste S11

16 teste noi (441 total):
- Mode: `st_asset_mode_default_man`, `st_asset_mode_set_auto`, `st_asset_mode_serde_roundtrip`, `st_asset_mode_empty_default`, `st_asset_mode_guard_live_position`, `st_all_modes_toggle`, `st_mode_change_persists_in_hashmap`
- Tags: `st_tag_auto_vs_man`, `st_force_exit_auto_only_filter`
- Bot: `st_bot_skips_man_asset_logic`
- Sizing: `st_sizing_suggestion_basic`, `st_sizing_suggestion_kelly`
- Data: `st_panel_data_pillar_scores`, `st_trade_prompt_event_fields`, `st_exit_prompt_event_fields`
- Audit: `st_mode_audit_log_format`

---

## 111. S12a: SMARTTREND CONFIG TAB (Tab 13)

Tab 13 **SMARTTREND CFG** este interfata de configuratie completa pentru motorul SmartTrend. Afiseaza si permite modificarea tuturor parametrilor ST, cu live readout si FSM state vizualizare.

### 111.1 Acces

- Click pe butonul **SMARTTREND CFG** din bara de tab-uri (dupa SIGNAL CFG)
- Layout two-column responsive (stivuire pe ecran sub 1100px)

### 111.2 Section 1: PILLAR WEIGHTS (cyan border)

Configureaza ponderile celor 3 piloni SmartTrend:

| Slider | Camp Backend | Range | Default |
|--------|-------------|-------|---------|
| Macro | `st_macro_weight` | 0-100 (%) | 40 |
| Micro | `st_micro_weight` | 0-100 (%) | 35 |
| Momentum | `st_momentum_weight` | 0-100 (%) | 25 |

- **Bara de proportii**: vizualizare normalizata cu 3 segmente colorate (cyan/verde/portocaliu)
- **Radar Chart** (B2): canvas 120x120px cu 3 axe, poligon semitransparent, actualizare live
- **Preseturi**: TREND FOLLOW (60/15/25), SCALP (25/45/30), BALANCED (34/33/33)
- **Validare**: suma trebuie sa fie ~100%; warning rosu cu animatie daca nu este

### 111.3 Section 2: ENTRY THRESHOLDS (orange border)

| Slider | Camp Backend | Range | Default |
|--------|-------------|-------|---------|
| Global Entry | `st_entry_threshold` | 10-100 | 65 |
| Stalking | `st_stalking_threshold` | 5-100 | 55 |
| Hysteresis | `st_hysteresis_margin` | 0-20 | 5.0 |

**Per-Regime Grid**: override-uri per regim de piata:

| Regim | Camp | Default |
|-------|------|---------|
| TRENDING | `st_entry_thresholds_0` | 60 |
| MEAN REV | `st_entry_thresholds_1` | 70 |
| CHAOTIC | `st_entry_thresholds_2` | 75 |
| UNKNOWN | `st_entry_thresholds_3` | 65 |

Validare: stalking threshold trebuie sa fie mai mic decat entry threshold.

### 111.4 Section 3: MICROSTRUCTURE (green border)

| Control | Camp Backend | Tip | Default |
|---------|-------------|-----|---------|
| Dominant Cycle Filter | `st_dominant_cycle_filter` | Toggle | ON |
| OFI Norm Range | `st_ofi_norm_range` | Slider 1-200 | 50 |
| Microprice BPS Range | `st_mp_bps_range` | Slider 1-50 | 10.0 |
| CVD Scale | `st_cvd_scale` | Slider 1-1000 | 100 |

Info box gold: "Micro pillar active only for streamer symbol with live L2 data"

### 111.5 Section 4: EXIT & TRAIL (orange border)

| Slider | Camp Backend | Range | Default |
|--------|-------------|-------|---------|
| Trail Tighten % / bar | `st_trail_tighten_pct` | 0-50% | 5% |
| Changepoint Reset | `st_changepoint_reset_threshold` | 0.10-1.00 | 0.70 |
| Stabilization Window | `st_stabilization_window` | 2-50 bars | 8 |

### 111.6 Section 5: SELF-CORRECTION (purple border)

| Slider | Camp Backend | Range | Default |
|--------|-------------|-------|---------|
| Bayesian Decay | `st_bayesian_decay` | 0.990-1.000 | 0.997 |
| Halflife (trades) | `st_bayesian_halflife` | 10-1000 | 200 |

**Live Readout** (actualizare la 5s via `get_smarttrend_state`):
- Hit Rate (entries/trail ratio)
- Entries total
- Trail Hits
- Changepoint Resets
- Stalking Timeouts
- Last Transition

### 111.7 Section 6: STATE MACHINE LIVE (cyan border)

5 casute FSM cu starea activa highlightata:

| State | Culoare Activa | Animatie |
|-------|---------------|----------|
| FLAT | Gray | -- |
| STALKING | Gold | `neon-pulse` |
| ENTRY | Cyan | -- |
| IN TRADE | Green | `neon-pulse` |
| TRAIL | Magenta | -- |

Sub casute: Direction (LONG/SHORT/--), Bars in State, Entry Price, Best Price, Combined Score, Agreement (AGREE/MIXED/DIVERGE).

Confirmation progress bar vizibila in starea Stalking.

Refresh la 3s via `get_smarttrend_state`.

### 111.8 Action Bar

| Buton | Actiune | Feedback |
|-------|---------|----------|
| SAVE CONFIG | Aplica doar campuri modificate in batch | Toast success + audio click |
| SMART DEFAULT | Preset BALANCED (34/33/33) | Toast info |
| RESET DEFAULTS | Restaureaza toate defaults | Confirmare + toast + audio confirm |
| COMPARE (B3) | Overlay side-by-side curent vs salvat | Tabel cu diffs gold highlight |

### 111.9 Bonusuri S12a

**B1: Config Diff Indicator**
- Dot cyan langa label-ul fiecarui camp modificat (animatie `neon-pulse`)
- Counter "N unsaved" in action bar
- Toast warning gold la parasirea tab-13 cu modificari nesalvate

**B2: Pillar Weights Radar Chart**
- Canvas 120x120px cu 3 axe (M/MI/MO)
- Grid rings concentrice, axe colorate
- Poligon date cyan semitransparent cu puncte pe fiecare axa
- Actualizare live la fiecare miscare de slider

**B3: Config Comparison Mode**
- Overlay full-screen cu tabel FIELD / CURRENT / SAVED / DELTA
- Diferentele highlight cu background gold
- Dismiss cu click overlay, buton X, sau tasta Escape

### 111.10 Fisiere Implicate

| Fisier | Rol |
|--------|-----|
| `src/index.html` | Tab button + tab-13 container complet |
| `src/styles.css` | CSS S12a (~350 linii) |
| `src/titan_bootstrap.js` | `initSmartTrendCfg()` (~380 linii) |
| `src/titan_keyboard.js` | Escape dismiss compare overlay |
| `src/titan_tabs.js` | Unsaved changes warning on tab switch |

### 111.11 Comenzi Tauri Folosite

| Comanda | Scop |
|---------|------|
| `get_smarttrend_config` | Incarcare initiala config |
| `update_smarttrend_config` | Salvare per-field la SAVE |
| `get_smarttrend_state` | Live readout + FSM state |

### 111.12 Teste S12a

12 teste noi (453 total):
- Serde: `st_config_serde_roundtrip`, `st_config_partial_json_backcompat`
- Validation: `st_config_validation_valid_default`, `st_config_validation_weight_sum_warning`, `st_config_validation_negative_weight_error`, `st_config_validation_threshold_ordering_error`
- Presets: `st_config_preset_trend_follow_values`, `st_config_preset_scalp_values`, `st_config_preset_balanced_values`
- Range: `st_config_per_regime_threshold_range`
- Diagnostics: `st_config_diagnostics_contains_all_fields`, `st_config_stats_default_zeros`

---

## 112. S12b: SMARTTREND UI ACTIONS + CHART OVERLAY

Extinderea tab-ului 13 cu sectiunile 7-10 (mutatii de stare), chart overlay SmartTrend, si EXPORT/IMPORT config.

### 112.1 Section 7: SMARTTREND ACCOUNT (cyan border)

Panel de management al contului virtual SmartTrend.

**Grid Informatii:**

| Camp | Sursa | Refresh |
|------|-------|---------|
| ALLOCATED | `allocated_capital` | 5s |
| INITIAL | `initial_capital` | 5s |
| DAILY PnL | `realized_pnl_today` | 5s (green/red) |
| TOTAL PnL | `total_realized_pnl` | 5s (green/red) |
| HWM | `high_water_mark` | 5s |
| MAX DD% | `max_drawdown_pct` | 5s (red) |
| TRADES | `total_trades` | 5s |
| W / L | `total_wins / losses` | 5s |

**Actiuni:**

| Buton | Comanda Tauri | Confirmare |
|-------|--------------|------------|
| TOP-UP | `smarttrend_topup` | Nu (validare amount > 0) |
| WITHDRAW | `smarttrend_withdraw` | Nu (validare amount > 0) |
| PAUSE / RESUME | `smarttrend_pause` | Nu (toggle) |
| LIQUIDATE | `smarttrend_force_exit_all` | Da (TitanToast.confirm) |
| KILL SWITCH | `smarttrend_kill_switch` | Da (dubla confirmare) |

**B1 Equity Micro-Chart**: Canvas 200x80px cu equity curve cyan, gradient fill, HWM dashed gold, drawdown zones red. Fallback text daca < 3 trades.

### 112.2 Section 8: SMARTTREND ASSETS (green border)

Tabel interactiv cu toate simbolurile SmartTrend.

| Coloana | Descriere |
|---------|-----------|
| SYMBOL | Numele activului |
| MODE | Badge AUTO (cyan) / MAN (gold), click pt toggle |
| STATE | FSM badge colorat (Flat gray, Stalking cyan, Entry cyan pulse, InTrade green, Trail gold) |
| SCORE | Scor combinat SmartTrend |
| DIR | Directia (Bullish/Bearish/Neutral) |
| BARS | Bare in starea curenta |
| LIVE | Dot verde daca exista pozitie `ST|` deschisa |

**Toolbar**: Input ADD symbol (adauga cu mode MAN), ALL AUTO (cu confirmare `CONFIRM_ST_ALL_AUTO`), ALL MAN.

**B2 Asset Heatmap**: Background subtil pe fiecare row bazat pe scor — rosu (<30), neutru (30-60), cyan (60-80), green (>80).

### 112.3 Section 9: SMARTTREND BACKTEST (orange border)

Backtest rapid pe simbolul activ.

- **Period slider**: 7-90 zile (default 30)
- **RUN BACKTEST**: Invoke `run_signal_backtest` cu mode SmartTrend
- **Progress bar**: Animata, ascultare event `signal-bt-progress`
- **Results grid**: Trades, Win Rate, Return%, Max DD, Sharpe, Profit Factor, Expectancy

**B3 Backtest Sparkline**: Canvas 300x60px cu equity curve green, HWM dashed gold, drawdown zones red, hover tooltip cu valoare.

### 112.4 Section 10: SENTINEL ALERTS (purple border)

Configuratie si monitorizare alerte sentinel.

| Control | Camp Backend | Default |
|---------|-------------|---------|
| Idle cadence | `st_sentinel_idle_secs` | 30s |
| Stalking cadence | `st_sentinel_stalking_secs` | 5s |
| Hot cadence | `st_sentinel_hot_secs` | 2s |

**Toggles**: Audio ON/OFF, Visual ON/OFF, Severity filter (Info/Warning/Critical).
**Alert Log**: Ultimele 20 alerte, scrollable, cu severity badges (Info cyan, Warning orange, Critical red).
**Status Badge**: RUNNING (verde) / STOPPED (rosu) via `get_sentinel_status`.

**B4 Sentinel Heatmap Badge**: Strip orizontal cu 3 segmente proportionale (Info/Warning/Critical), animatie pulse pe segmentul dominant, tooltip cu counts exacte.

### 112.5 Chart Overlay SmartTrend

| Element | Descriere |
|---------|-----------|
| State Strip | Bara 22px sub chart cu 5 segmente (FLAT/STALK/ENTRY/TRADE/TRAIL), glow pe starea activa |
| FSM Markers | Arrow up cyan (Entry), arrow down red (Exit), circle gold (Trail) pe candlestick la tranzitii |
| CVD Toggle | Checkbox in strip pentru overlay CVD magenta |

Strip-ul se ascunde automat daca SmartTrend nu este enabled. Refresh la 3s.

### 112.6 EXPORT / IMPORT Config

| Buton | Actiune |
|-------|---------|
| EXPORT | Descarca JSON complet `SmartTrendConfig` cu `schema_version: 1` |
| IMPORT | Upload JSON, validare client + server (serde + range checks), confirmare, reload |

Format: JSON complet cu toate campurile `SmartTrendConfig`. Campuri necunoscute sunt ignorate la deserializare (backward compatible cu `#[serde(default)]`).

### 112.7 Comenzi Tauri Noi S12b

| Comanda | Scop |
|---------|------|
| `smarttrend_export_config` | Returneaza JSON complet config cu `schema_version` |
| `smarttrend_import_config` | Primeste JSON string, valideaza, aplica si salveaza |

### 112.8 Teste S12b

8 teste noi (461 total):
- Diagnostics: `diagnostics_includes_entry_best_price`, `diagnostics_entry_price_after_entry`
- Export/Import: `export_config_roundtrip`, `import_config_rejects_invalid_json`, `import_config_validates_negative_weights`, `import_config_validates_threshold_ordering`, `export_config_with_schema_version`, `import_full_config_roundtrip`

---

## 113. S14: EXECUTION RESILIENCE (Sesiunea 63)

Hardening complet al ciclului de executie ordine, health observability, si data-stream robustness.

### 113.1 Order Lifecycle Tracking

Fiecare ordin genereaza un `client_order_id` (UUID v4) unic. Raspunsul broker-ului este parsat pentru `broker_id` si `status`. Retry automat 1-2x pentru erori tranziente de retea/server (5xx, timeout). Evenimentul `order-lifecycle` este emis la fiecare tentativa.

| Camp | Tip | Sursa |
|------|-----|-------|
| `client_order_id` | UUID v4 | Generat local |
| `broker_id` | String | Parsat din raspunsul Alpaca |
| `retry_count` | u8 | 0-2 |
| `status` | String | `new`, `accepted`, `filled` etc. |

### 113.2 Pending Order Watchdog

Ordinele trimise sunt tracked local in `TradingState::pending_orders`. Watchdog-ul ruleaza in bucla autopilot si canceleaza automat ordinele:

- **Stale:** mai vechi de `pending_order_timeout_secs` (default 120s, configurabil)
- **Generation Mismatch:** `generation_id` diferit de generatia curenta (dupa respawn loop)

Cancelarea se face via Alpaca DELETE `/v2/orders/{broker_id}`. Ordinele sunt scoase din pending list la `fill`, `partial_fill`, `rejected`, sau `canceled`.

### 113.3 Equity Sync Health Emitter

| Eveniment | Conditie | Payload |
|-----------|----------|---------|
| `equity-sync-degraded` | Erori consecutive moderate | `{ consecutive_failures, msg }` |
| `equity-sync-critical` | Erori consecutive >= threshold | `{ consecutive_failures, msg }` |

Frontend: Badge **EQUITY STALE** (gold cu pulse) in cockpit strip cand se primeste eveniment degraded/critical.

### 113.4 Trade Updates Resilience

| Metrica | Descriere |
|---------|-----------|
| `reconnect_count` | Numar reconectari WS de la start |
| `trade-updates-health` | Event emis cu `connected: bool, reconnect_count, last_disconnect_reason` |

Frontend: Badge **WS DISCONN** (rosu cu pulse) in cockpit strip cand `connected = false`.

Fill-urile primite pe WS corelaza `order.id` cu `pending_orders` pentru a le elimina din tracking.

### 113.5 Exit Features

`TradeRecord` a fost extins cu `exit_features: Option<TradeFeatures>` — snapshot al conditiilor de piata la momentul inchiderii pozitiei (pret, volatilitate, volum, spread, RSI, VWAP). Complementeaza `entry_features` pentru analiza post-trade (entry vs exit delta).

### 113.6 Bounded Streamer Channel

`tokio::sync::mpsc::channel(256)` inlocuieste `unbounded_channel` pentru stream commands. `try_send` cu warning log la overflow previne memory exhaustion. Toate `sender.send()` convertite la `sender.try_send()`.

### 113.7 HHI Notional Fix

Calculul Herfindahl-Hirschman Index foloseste acum `qty * live_price` (din `state.market.last_prices`) in loc de `qty * entry_price`. Logging structurat cu `hhi_pre`, `hhi_post`, `blocked`.

### 113.8 Cockpit Strip (B1)

Bara persistenta sub header cu metrici agregate si health badges:

| Element | Sursa | Refresh |
|---------|-------|---------|
| EQUITY | `get_settings` → equity | 5s poll |
| P&L | `get_settings` → P&L calculat | 5s poll |
| POS | `get_settings` → positions.len | 5s poll |
| MODE | `get_settings` → auto/manual | 5s poll |
| BREAKER | `breaker-triggered` event | Event |
| GEN | `get_settings` → generation | 5s poll |
| EQUITY STALE | `equity-sync-degraded/critical` | Event |
| WS DISCONN | `trade-updates-health` | Event |

Stilizare: `.cockpit-strip` cu `border-bottom: 1px solid var(--border-subtle)`, font mono, responsive collapse pe mobile.

### 113.9 Live Fill Markers (B2)

Triangulare fill-uri pe chart via LightweightCharts markers:

| Tip | Shape | Color | Text |
|-----|-------|-------|------|
| BUY / buy | arrowUp | var(--neon-green) | `B ${qty}` |
| SELL / sell | arrowDown | var(--neon-red) | `S ${qty}` |

Marker-urile sunt persistente in `window._titanFillMarkers[]`. Refresh la 3s via `setInterval`. Timestamp-ul este convertit din ISO 8601 la epoch seconds UTC.

### 113.10 Journal Min P&L Filter (B3)

Input `j-filter-minpnl` adaugat in barul de filtre al jurnalului. Valoarea este transmisa la `get_journal_trades` ca parametru `min_pnl`. Reset curata si acest camp.

### 113.11 Time-in-Trade Column (B4)

Coloana **TIME** adaugata in tabelul positions (cyber-hud-table):

| Durata | Culoare | Clasa CSS |
|--------|---------|-----------|
| 0-5 min | dim | `pos-time-dim` |
| 5-30 min | cyan | `pos-time-cyan` |
| 30 min - 2h | gold | `pos-time-gold` |
| > 2h | orange + flicker | `pos-time-orange` |

Calculul: `Date.now()/1000 - position.opened_at`. Format: `Xh Ym` / `Xm Ys` / `Xs`.

### 113.12 Fisiere Implicate S14

| Fisier | Modificari |
|--------|-----------|
| `src-tauri/Cargo.toml` | +uuid dependency |
| `src-tauri/src/app_state.rs` | +broker_order_id pe Position, +pending_order_timeout_secs |
| `src-tauri/src/state.rs` | Bounded Sender, +pending_order_timeout_secs settings |
| `src-tauri/src/trading_state.rs` | +PendingOrder struct, +pending_orders vec |
| `src-tauri/src/commands/order_exec.rs` | +OrderResult, UUID gen, retry loop, broker_id parse |
| `src-tauri/src/commands/trading.rs` | Adaptat la OrderResult |
| `src-tauri/src/commands/misc.rs` | +exit_features/broker_order_id pe manual record |
| `src-tauri/src/commands/account.rs` | try_send bounded channel |
| `src-tauri/src/commands/settings.rs` | try_send bounded channel |
| `src-tauri/src/loops/autopilot.rs` | Pending watchdog, exit features snapshot, HHI fix |
| `src-tauri/src/loops/trade_updates.rs` | Health emitter, fill correlation, pending removal |
| `src-tauri/src/loops/equity_sync.rs` | Health emitter events |
| `src-tauri/src/loops/watchdog_loop.rs` | Bounded channel |
| `src-tauri/src/loops/position_recon.rs` | +broker_order_id |
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

## 114. S15: TRADING EDGE UPGRADES (Sesiunea 64)

Patru imbunatatiri de trading logic + patru bonusuri UI/UX, toate cu style cyber-HUD consistent.

### 114.1 Max-Hold Decay Exit (T1)

Mecanism de exit progresiv care strange trail-ul pe masura ce pozitia se apropie de `max_hold_secs`. Previne dead-weight positions care consuma capital fara actiune.

**Cum functioneaza:**
1. Cand `elapsed > max_hold_secs * max_hold_decay_start_pct` (default 50% din max hold): trail-ul incepe sa se stranga proportional cu `progress` (0..1)
2. Formula: `effective_trail = base_trail * (1.0 - progress * decay_trail_tighten * 100)`
3. Cand `elapsed >= max_hold_secs`: force exit cu log `[MAX_HOLD_DECAY] Force exit`

**Settings:**
- `max_hold_decay_enabled` (default: true) — Activeaza/dezactiveaza mecanismul
- `max_hold_decay_start_pct` (default: 0.5, range 0.2-0.9) — La ce procent din max_hold incepe decay
- `max_hold_decay_trail_tighten` (default: 0.003, range 0.001-0.02) — Rata de strangere per tick

### 114.2 MTF Confirmation Gate (T2)

Gate de confirmare pe timeframe superior inainte de entry. Verifica daca directia propusa (BUY/SELL) este aliniata cu trendul pe HTF.

**Mod Soft (default):** Penalizeaza scorul cu `mtf_gate_penalty` puncte daca HTF nu confirma.
**Mod Hard:** Blocheaza complet intrarea daca HTF nu confirma.

**Helper partajat:** `is_htf_aligned(candles, oracle_weights, is_buy) -> (aligned, htf_score)` in `strategy.rs`. Mapping automat LTF->HTF: 1m->5m, 5m->15m, 15m->1h, 1h->4h, 4h->1d.

**Settings:**
- `mtf_gate_enabled` (default: true)
- `mtf_gate_mode` (default: "soft", optiuni: "soft"/"hard")
- `mtf_gate_penalty` (default: 15.0, range 1-50)
- `mtf_gate_htf` (default: "15m") — Override manual al HTF

### 114.3 Tiered Scanner Top-K (T3)

Pipeline de pre-ranking in scanner care reduce scorarea costisitoare (oracle + GPU) pe tickers ilichizi.

**Pipeline:**
1. Pre-rank by dollar volume (close * volume) DESC
2. **Tier 1** (top `tier1_count`): scorate complet — mereu
3. **Tier 2** (urmatoarele `tier2_count`): scorate doar daca RVOL >= gate SAU change% >= 3%
4. Merge results, re-sort by prefilter score, truncate la `max_candidates`

**Settings pe ScanParams:**
- `tiered_scan_enabled` (default: true)
- `tier1_count` (default: 50, range 10-500)
- `tier2_count` (default: 100, range 10-500)

### 114.4 Risk Parity Manual Orders (T4)

Gate HHI pe ordine manuale in `send_order_internal`. Similar cu gate-ul existent pe autopilot, dar cu limita mai relaxata.

**Calcul:** HHI post-trade = Σ(notional_i / total_notional)². Daca depaseste limita, ordinul este rejectat cu eroare descriptiva.

**Settings:**
- `hhi_gate_manual_orders` (default: true)
- `max_concentration_hhi_manual` (default: 0.35, range 0.1-1.0) — Mai relaxat decat autopilot (0.30)

### 114.5 Regime Band pe Chart (B1)

Strip vizual sub chart care arata regimul de piata curent, actualizat la 5s.

| Regim | Culoare | Label |
|-------|---------|-------|
| Trending Up | Neon Green glow | TRENDING UP |
| Trending Down | Neon Red glow | TRENDING DOWN |
| Chaotic | Neon Orange glow | CHAOTIC |
| Quiet/Range | Dim border | QUIET/RANGE |

Score-ul oracle este afisat in dreapta strip-ului.

### 114.6 Journal Hour-of-Day PnL Heatmap (B2)

Grid canvas 5 zile (L-V) x 13 intervale de 30 minute (9:30-16:00 ET).

Fiecare celula arata PnL-ul total pe acel bucket, cu culoare gradient proportionala cu magnitudinea (green=profit, red=loss). Tooltip pe hover: ziua, intervalul, PnL, numar trades, win rate.

Pozitionat in Journal tab ca `.cyber-panel` sub KPI-urile existente.

### 114.7 Strategy Agreement Dashboard (B3)

Mini-panel in Nexus cu 3 indicatori de strategie si metru de acord.

**Indicatori:** Oracle, GPU, SmartTrend — fiecare cu badge colorat LONG/SHORT/NEUTRAL.
**Agreement Meter:** Barra 0-100%: 3/3 agree = 100% green, 2/3 = 66% cyan, 1/3 = 33% gold, 0 = red.
**Stale Detection:** Daca un indicator nu s-a actualizat > 60s, badge-ul arata "STALE" cu animatie `data-flicker`.

Alimentat de evenimentele Tauri `nexus-update` si `smarttrend-state`.

### 114.8 Latency Panel V2 (B4)

Upgrade al panelului de latenta existent cu vizualizari grafice.

**Sparklines:** Canvas 120x28 per metric (Order, GPU, Stream), ring buffer de 60 sample-uri. Linie cyan cu punct colorat la ultima valoare (green < 50%, gold 50-80%, red > 80% din max).

**Histogram:** Canvas 160x40 cu 5 buckets de distributie: <1ms (green), 1-5ms (cyan), 5-20ms (gold), 20-100ms (orange), >100ms (red). Acumuleaza contor global pe masura ce apar date noi.

### 114.9 Fisiere Implicate S15

| Fisier | Modificari |
|--------|-----------|
| `src-tauri/src/state.rs` | +12 settings S15, +10 default functions, +settings_from_state sync, +apply_settings sync, +2 teste |
| `src-tauri/src/app_state.rs` | +12 fields S15 pe AppState, +defaults |
| `src-tauri/src/commands/settings.rs` | +12 batch update handlers S15 |
| `src-tauri/src/loops/autopilot.rs` | Max-Hold Decay exit logic, MTF gate in evaluate_candidate_entry |
| `src-tauri/src/strategy.rs` | +is_htf_aligned, +ltf_to_htf, +6 teste |
| `src-tauri/src/commands/order_exec.rs` | HHI gate manual orders |
| `src-tauri/src/scanner.rs` | Tiered scan pipeline, +3 ScanParams fields, +3 teste |
| `src-tauri/src/trading_engine.rs` | +broker_order_id fix pe teste |
| `src/index.html` | Regime strip, journal heatmap canvas, strategy agreement panel, latency sparklines+histogram |
| `src/styles.css` | +120 linii CSS: regime strip, heatmap tooltip, strategy agreement, latency sparklines |
| `src/titan_bootstrap.js` | Regime band init, Strategy agreement init |
| `src/titan_journal.js` | Hour-of-day PnL heatmap (canvas bucketing + rendering + tooltips) |
| `src/titan_rightbar.js` | Latency V2 sparklines + histogram (ring buffer + canvas drawing) |

---

## §115 — S16: UX Polish + Cleanup (Sesiunea 65 — Sesiunea finala roadmap)

### 115.1 Cockpit Strip V2

- **GEN Badge** — sync cu `session_generation` din `get_risk_status` la 5s poll
- **BREAKER Badge** — polling initial + pulse animation cand > Normal
- **W/R Badge** — Win rate din session analytics (>=55% green, >=45% cyan, <45% orange)
- **UPTIME Badge** — Elapsed time HH:MM:SS de la session start

### 115.2 Unified Chart Markers Engine

`window._titanMarkerEngine` — manager unic fills + signals + SmartTrend. Debounce 100ms, FIFO cap 50, pruning >24h.

### 115.3 Latency Histogram Rolling Window

Ring buffer 300 samples (5 min). Distributia nu mai creste indefinit.

### 115.4 Fill Quality Columns HFT Targets

3 coloane: SLIP (avg slippage bps), T2F (avg duration), TRADES (count). Backend `get_fill_quality_stats`. Polling 30s.

### 115.5 PiP Mini Dashboard (B1)

Floating HUD 260px, draggable, shortcut P, opacity slider, minimize. Refoloseste date cockpit.

### 115.6 LLM Sentiment Batch + Decay (B2)

Batch scoring N headlines, exp decay halflife pe confidence. Settings: `llm_batch_size` (5), `sentiment_halflife_secs` (3600).

### 115.7 Mode Audit Trail (B3)

`Vec<ModeTransition>` cap 100, timeline panel in Nexus, color coding per mode. Command `get_mode_audit_trail`.

### 115.8 Health Score Badge (B4)

`HP:XX%` in cockpit. 6 sub-scores: equity sync, WS, breaker, latency, loops, fill rate. Polling 10s.

### 115.9 Module Entrypoint + $el() Rollout + CSS Cleanup

`titan_init.js`, `registerTabInit()`, 464 getElementById → $el(), CSS var() tokens.

---

## 106. Wave 1 MUST-FIX — 9 Fix-uri Critice + 4 Bonusuri (Sesiunea 67)

### 106.1 Fill Deduplication (CRIT-11)

Alpaca raporteaza `filled_qty` si `filled_avg_price` **cumulativ** per order. Codul anterior le trata ca incrementale, ducand la pozitii inflate ~50% pe partial fill -> fill sequences.

**Solutia:** HashMap `fill_cumulative` per order ID pe `TradingState`. La fiecare fill event, se calculeaza `delta_qty = cum_filled_qty - prev_qty` si se aplica DOAR delta-ul. Fills cu delta <= 0 sunt suprimate automat si contorizate in `fills_deduplicated`.

**Badge cockpit:** `DEDUP:N` (cyan, vizibil doar cand N > 0) arata in timp real cate fills duplicate au fost prevenite.

### 106.2 Bot Exit Operator Precedence (CRIT-12)

Conditia de exit din SmartTrend bot avea `state == Flat && transition.contains("Trail->Flat") || transition.contains("InTrade->Flat")` -- fara paranteze, `&&` leaga mai strans decat `||`, deci `InTrade->Flat` triggera exit chiar daca starea NU era Flat.

**Fix:** `state == Flat && (transition.contains("Trail->Flat") || transition.contains("InTrade->Flat"))`

### 106.3 Circuit Breaker Latch (CRIT-1)

Breaker-ul seta `account.paused = true` si `account.breaker_triggered = true`, dar gate-ul de la inceputul loop-ului verifica DOAR `st_config.st_bot_paused` din settings. Dupa prima iteratie, bot-ul putea continua sa tradeeze.

**Fix:** Check `account.paused || account.breaker_triggered` la inceputul loop-ului + persist `settings.smarttrend.st_bot_paused = true` dupa breaker fire.

### 106.4 Position Recon Trail-Stops (CRIT-15)

Reconcilierea pierdea trail high cand pretul crestea peste old high (copia `existing.highest_price` doar daca era mai mare decat `current_price`, dar nu si daca `current_price` era cel mai mare).

**Fix:** `highest_price = existing.highest_price.max(current_price)`, `lowest_price = existing.lowest_price.min(current_price)`. Preserva si `broker_order_id`.

### 106.5 Exit Inflight TTL (CRIT-16)

`close_position` insera in `exit_inflight` dar remove-ul era incomplet — lipsea pe error path si pe canceled/rejected events. Simboluri puteau ramane blocate permanent.

**Fix:** TTL 120s cleanup in trade_updates loop + explicit remove pe: fill complet, send_order error, rejected/canceled events.

### 106.6 Lock Order (CRIT-21)

`smarttrend_kill_switch` lua engine -> settings, `smarttrend_pause` lua settings -> engine = ABBA deadlock potential.

**Fix:** Standardizat ordinea in TOATE comenzile: **settings(1) -> smarttrend_engine(2) -> st_account(3)**. NICIODATA invers.

### 106.7 API Rate Limit (CRIT-13)

`cancel_open_orders_for_symbol` facea DELETE la Alpaca FARA `acquire_api_permit`, putand genera 429 rate limit rejections.

**Fix:** Adaugat permit + tracking `record_api_call()`. Badge cockpit `API:N/200` cu color coding.

### 106.8 AssetCheck JSON (CRIT-5)

Daca HTTP raspundea 200 dar JSON parse esua, `if let Ok(json)` era skipped si `is_fractionable` ramanea `true`. Ordinul putea trece pe ticker delisted.

**Fix:** `match` explicit pe JSON parse. Pe Err: reject in hard mode, `is_fractionable = false` in soft mode.

### 106.9 Generation ID (HIGH-24)

`generation_id` era citit DUPA HTTP POST, deci un `bump_generation()` in timpul request-ului atasa generation gresita la PendingOrder.

**Fix:** Mutat `state.current_generation()` la inceputul functiei `send_order_internal`, inainte de orice await.

### 106.10 Trail-Stop Integrity Indicator (B4)

Coloana noua `TRAIL` in positions table cu badge `TRAIL OK` (green) / `TRAIL WARN` (red pulse). Verificat la fiecare cockpit poll (5s) prin `_titanTrailHealth`. Confirma vizual ca trail extremes sunt corecte dupa reconciliation.

---

## 107. Wave 2A SmartTrend Safety — 8 Fix-uri + 4 Bonusuri (Sesiunea 68)

Sesiunea 68 a fixat 8 bug-uri critice de safety in engine-ul SmartTrend si a adaugat 4 bonusuri cyber-HUD. Total: 516 teste (+10 noi), clippy clean.

### 107.1 Sentinel Generation Check (CRIT-3)

Sentinel-ul compara `current_generation() < boot_gen`, dar generatia e monoton crescatoare (creste la fiecare respawn). Dupa un respawn watchdog, sentinel-ul vechi NU se oprea — rula ca zombie alaturi de cel nou.

**Fix:** Inlocuit `<` cu `!=`, identic cu `smarttrend_bot.rs`. Cand generatia se schimba (in orice directie), sentinel-ul iese cu log warning si lasa locul instansei noi.

### 107.2 Multi-Symbol ST Exit (CRIT-6)

Exit-ul din bot verifica `pos.symbol == settings.active_symbol`. Pozitii SmartTrend pe alte simboluri raman orfane — nu primeau niciodata auto-exit cand FSM-ul mergea Flat.

**Fix:** Sters guard-ul `active_symbol` din blocul de exit. TOATE pozitiile cu `strategy_mode == "ST|..."` primesc exit cand engine-ul e Flat. Entry-urile raman gated pe `active_symbol` (corect — doar pe simbolul curent din chart se intra). Guard-ul `recently_acted` per symbol si `freshness_window_ms` raman ca safety net anti-spam.

### 107.3 Kelly Bayesian Loss Floor (CRIT-14)

Cu zero losses (100% win rate), `avg_loss <= 0` => `b = 0` => `kelly_fraction = 0`. Paradoxal, botul nu dimensiona pozitii cand performa la maxim.

**Fix:** Bayesian floor pe `avg_loss`: daca `avg_loss <= 0`, foloseste `max(avg_win * 0.1, 1e-9)` ca prior conservativ. Kelly returneaza o fractie pozitiva (capped la `max_fraction`), permitand sizing proportional chiar si cu track record perfect.

### 107.4 Kill Switch Import Guard (CRIT-20)

`import_config` inlocuia intregul `SmartTrendConfig` din JSON, inclusiv `st_killed`. Un fisier JSON cu `st_killed: false` dezarma kill switch-ul silentios.

**Fix:** Dupa parsarea importului, campurile safety sunt fortat din runtime-ul curent: `imported.st_killed = settings.st_killed` si `imported.st_bot_paused = settings.st_bot_paused`. Aceste campuri pot fi modificate DOAR prin comenzile dedicate (`smarttrend_kill_switch`, `smarttrend_pause`).

### 107.5 Entry State Persist Bars (HIGH-11)

FSM-ul facea `Entry -> InTrade` imediat la urmatorul `evaluate()`. Botul avea o singura fereastra de eval sa vada starea Entry. Daca loop-ul era desincronizat (async timing), ratea entry-ul complet.

**Fix:** Nou camp config `st_entry_persist_bars` (default 3). Entry persista pana la:
- `bars_in_state >= st_entry_persist_bars` (timeout automat)
- Botul confirma ca a plasat order-ul (seteaza `entry_acted = true` pe engine)

### 107.6 entries_total Double Count (HIGH-13)

`record_transition` incrementa `entries_total` si pe `Stalking->Entry` SI pe `Entry->InTrade`. Un singur trade = 2 entries in statistici, corupand trail_hit_rate.

**Fix:** Contorizeaza doar tranzitia IN `Entry` (nu si `InTrade`). O tranzitie per trade.

### 107.7 NaN Weight Firewall (HIGH-15)

Daca un weight sau score era NaN, `sum <= 0.0` si `adj_sum > 0.0` nu detectau NaN (`NaN > 0` = false in IEEE 754). NaN se propaga in `combined` score, afectand decizii entry/exit.

**Fix:** Guards NaN la 3 nivele:
1. `normalize_weights`: `!sum.is_finite()` => fallback la defaults (0.40, 0.35, 0.25)
2. Pillar scores individuale: `!score.is_finite()` => sanitizat la 50.0 (neutral) + confidence 0
3. Combined score final: `!combined.is_finite()` => 50.0 (neutral)

Counter `nan_sanitized_count` pe engine incrementat la fiecare sanitizare, expus in `get_risk_status`.

### 107.8 Zombie Loops AbortHandle (HIGH-20)

`main.rs` spawna loop-uri cu `tauri::async_runtime::spawn()` fara `abort_and_replace`. La primul respawn prin watchdog, nu exista handle vechi de abortat — task-ul original continua alaturi de cel nou.

**Fix:** Macro `spawn_with_handle!` inregistreaza `AbortHandle` pentru TOATE loop-urile la startup prin `watchdog.abort_and_replace()`. La respawn, handle-ul vechi e abortat corect.

### 107.9 ST SHIELD Safety Dashboard (B1)

Panel cyber-HUD in sidebar cu 8 metrici de safety:

| Metrica | Valori | Sursa |
|---------|--------|-------|
| KILL | OFF (green) / ON (red pulse) | engine.config.st_killed |
| BREAKER | NORMAL / PAUSED / TRIPPED | st_account |
| DEDUP | counter (gold daca > 0) | fills_deduplicated |
| ZOMBIE | counter (red daca > 0) | watchdog.dead_loops() |
| NaN | counter (red daca > 0) | nan_sanitized_count |
| KELLY | fractie 0.00-0.25 | kelly_fraction() |
| GEN | #N | session_generation |
| ORPHANS | counter (red daca > 0) | orphan detection |

Shield score composite 0-100%: penalizeaza kill (-50), breaker tripped (-30), zombies (-20), orphans (-15), NaN (-10).

### 107.10 Generation Timeline Strip (B2)

Banda vizuala in cockpit care arata istoria generatiilor. Fiecare generatie afiseaza ID-ul si durata. Ultima generatie pulseaza cu `neon-pulse`. Ring buffer de max 10 entries.

### 107.11 NaN Firewall Monitor Badge (B3)

Badge `NaN:0` (green) / `NaN:N` (red pulse) in cockpit strip. Arata in timp real cate valori NaN au fost interceptate si sanitizate de firewall-ul din FIX-7.

### 107.12 Orphan Position Detector (B4)

Coloana noua `ORPHAN` in positions table, vizibila doar pentru pozitii SmartTrend. Badge `OK` (green) confirma ca pozitia are exit path valid dupa FIX-2 (multi-symbol exit). Badge `ORPHAN` (red pulse) semnaleaza pozitii fara exit path.

---

## 108. Wave 2B SmartTrend Correctness — 9 Fix-uri + 5 Bonusuri (Sesiunea 69)

Sesiunea 69 a fixat 9 bug-uri de corectitudine in engine-ul SmartTrend si a adaugat 5 bonusuri cyber-HUD. Total: 526 teste (+10 noi), clippy clean.

### 108.1 Hysteresis Validation (HIGH-12)

`st_hysteresis_margin` putea fi configurat suficient de mare incat FSM-ul sa ramana blocat in Stalking (hysteresis >= stalking_strength/2, `drop_strength` devenea 0, conditia de exit era imposibila). Fix: validare semantica in `validate_smarttrend_config` + runtime clamp la `max_safe_margin`. `drop_strength` minim 0.1.

### 108.2 Sentinel Per-Symbol Oracle Inputs (HIGH-14)

Sentinel-ul folosea `last_oracle_snapshot()` care returna ultimul entry accesat din cache (global). Cu multi-symbol evaluation, toate simbolurile primeau datele oracle ale ultimului simbol evaluat. Fix: nou `oracle_snapshot_for(symbol)` cu lookup per-simbol pe prefix `symbol:` in cache keys. Fallback la global cand nu exista date per-simbol. `nexus_consensus` ramane global (limitare arhitecturala documentata).

### 108.3 Circuit Breaker Reset Race (HIGH-16)

`reset_circuit_breaker` reseta doar `risk.circuit_breaker` dar NU `st_account.breaker_triggered`, `st_account.paused`, `settings.st_bot_paused`. `daily_reset` reseta `breaker_triggered` dar nu `paused`. Fix: ambele functii reseteza acum complet starea ST. `daily_reset` seteaza `paused = false` doar cand `breaker_triggered` era activ (preserva pause manual intentionat).

### 108.4 Exit Floor Dead Code (HIGH-21)

Linia `local_stop = local_stop.min(-floor).max(local_stop)` era matematic un no-op (`min(x, -floor)` urmat de `max(result, x)` = `x`). Fix: sters no-op-ul, pastrat doar verificarile functionale `if abs < floor`. Adaugat guard `exit_floor_pct > 0.0`.

### 108.5 Max-Hold Decay 100x (HIGH-22)

Formula `tighten_factor = 1.0 - progress * trail_tighten * 100.0` facea parametrul `0.003` sa actioneze ca `0.3` per progress unit (100x mai agresiv decat configurat). Fix: sters `* 100.0`, default schimbat de la `0.003` la `0.3`, clamp range ajustat `[0.05, 0.8]`. Comportamentul efectiv este identic, dar parametrul reflecta acum corect scala sa.

### 108.6 Lyapunov Live vs History (HIGH-25)

Live `calculate()` calcula Lyapunov pe intreaga serie de closes (globala). History `calculate_history()` folosea window rolling de 200 bare. Divergenta: live putea vedea "chaos" pe 500 bare, history vedea "predictabil" pe ultimele 200. Fix: live acum foloseste `closes[len-200..]` trailing slice, identic cu history.

### 108.7 Kalman price_vs_predicted Mismatch (CRIT-4)

Trei probleme fixate: (a) `indicator_cache` aplica `.abs()` pe `price_surprise`, stergand semnul folosit de macro pillar pentru alignment cu velocity; (b) history `price_vs_predicted` era raw delta (`close - pred`), live era ratio (`(close - state.x[0]) / close`); (c) `surprise_series` per-bar in `kalman_filter` folosea `.abs()`. Fix: sters `.abs()` din cache merge si surprise_series, normalizat history la ratio `(close - pred) / close`.

### 108.8 Duplicate Changepoint Markers (MED-5)

Chart markers se duplicau la fiecare eval cycle deoarece `_merge()` concatena fara dedup. Fix: dedup by composite key `time|position|shape|text` via Set in `_titanMarkerEngine._merge()`.

### 108.9 stalking_timeouts Incrementat Gresit (MED-8)

`stalking_timeouts` se incrementa atat pe hysteresis drop (Stalking->Flat din scor scazut) cat si pe timeout real. Fix: incrementul exista acum doar pe path-ul `stalking_timeout` (cand `bars_in_state >= 2 * confirmation_needed`).

### 108.10 ST CORRECTNESS Dashboard (B1)

Panel cyber-HUD cu 4 metrici de corectitudine, afisate in sidebar sub ST SHIELD:

| Metrica | Valori | Semnificatie |
|---------|--------|-------------|
| HYS | OK (green) / STUCK:N (red pulse) | Detecteaza FSM blocat in Stalking > 50 evaluari consecutive |
| DECAY | Score curent (gold) | Ultima valoare `last_score` din engine |
| LYAP | -- (placeholder) | Lyapunov exponent curent |
| KAL | +0.3% (green) / -2.1% (red) | Kalman price surprise signed ratio |

### 108.11 FSM Transition Heatmap Badge (B2)

Badge compact `FSM:S12/E3/T8` in cockpit strip. Ring buffer de 100 evaluari pe `SmartTrendEngine`. Counter per-state (Stalking/Entry/InTrade+Trail). Culoare dominanta: cyan (Stalking), green (Entry), gold (InTrade).

### 108.12 Breaker Sync Indicator (B3)

Badge `BRK:SYNC` (green) / `BRK:DESYNC` (red pulse) in ST SHIELD panel. Compara `risk.circuit_breaker.level() != Normal` cu `st_account.breaker_triggered`. Confirma vizual ca fix-ul FIX-3 functioneaza corect.

### 108.13 Kalman Alignment Sparkline (B4)

Mini bar chart CSS (8 bare) in cockpit strip. Ultimele 8 valori `price_surprise` din indicator cache. Bare cyan pentru surprise pozitiv, magenta pentru negativ. Height proportional cu maxim. Flat = Kalman aliniat cu pretul, spike = divergenta.

### 108.14 Max-Hold Timer HUD (B5)

Coloana noua `HOLD` in positions table. Progress bar `15/60m` cu gradient stops: cyan (normal), gold + glow-breathe (>70% din max_hold), red + pulse-red-ring (>90%). Arata vizual cat de aproape este pozitia de forced exit din max-hold decay.

---

## 109. Wave 2C — SmartTrend UI+Chart (Sesiunea 71)

### 109.1 Chart Markers Vizibile (CRIT-8)

`window.candleSeries` este acum expus ca proprietate globala dupa crearea seriei in `titan_bootstrap.js`. Marker engine-ul `_titanMarkerEngine._merge()` poate seta markere pe chart. Toate cele 3 surse (signals, fills, SmartTrend) sunt vizibile.

### 109.2 ST CFG Flicker Eliminat (CRIT-9)

Toate timer-ele SmartTrend din `titan_bootstrap.js` folosesc `TitanTimers.register` cu `{ tab: 13 }`:
- `st:diagnostics` (3s) -- SmartTrend diagnostics panel sidebar
- `st:panel-poll` (5s) -- SmartTrend account panel sidebar
- `st:s12b-account` (5s) -- S12b account section tab-13
- `st:s12b-assets` (5.5s) -- S12b assets table tab-13
- `st:s12b-sentinel` (6s) -- S12b sentinel status tab-13

Timer-ele cu tab guard se opresc automat cand alt tab este vizibil, eliminand poll-uri inutile si flicker DOM.

### 109.3 Duplicate HTML IDs Rezolvate (HIGH-6)

Tab-13 (S12b sections) folosea aceleasi ID-uri ca sidebar-ul SmartTrend: `st-acct-capital`, `st-acct-dd`, `st-acct-trades`, `st-acct-amount`, `st-assets-tbody`. Acum tab-13 foloseste prefix `s12b-` (ex: `s12b-acct-capital`, `s12b-assets-tbody`). Ambele sectiuni se actualizeaza independent.

### 109.4 titan_init.js Incarcat (HIGH-4)

`<script src="titan_init.js" defer></script>` adaugat in `<head>`. INIT_ORDER se executa la DOMContentLoaded, orchestrand initializarea modulelor in ordinea corecta.

### 109.5 Health Badge Date Reale (MED-3)

Health badge `HP:N%` foloseste acum date reale:
- **latencyQuality:** `100 - floor(avg_latency_ms / 10)`. Latenta vine din `LatencyMetrics.snapshot().order_rtt_us.p50`.
- **fillRate:** `fills_received / orders_submitted * 100`. Counters atomici pe `TradingState`.
- Backend: `get_risk_status` expune `avg_latency_ms`, `orders_submitted`, `fills_received`.

### 109.6 Marker Engine Cleanup

- Eliminat fallback `candleSeries.setMarkers()` din `titan_indicators_ui.js` -- foloseste exclusiv `_titanMarkerEngine.update('signals', ...)`
- Eliminat `engine._stMarkers` hack din `titan_data_load.js` -- foloseste `engine.getSource('smarttrend')` (API publica)
- Adaugat `stats()` si `getSource()` pe marker engine pentru introspectie

### 109.7 Timer Consolidation Badge (B1)

Badge `TMR:N` in cockpit strip arata numarul total de timere active. Tooltip cu breakdown: name, interval, tab guard, status. `TitanTimers.getAll()` expune lista completa.

### 109.8 Marker Engine Health Badge (B2)

Badge `MRK:S12/F3/T8` in cockpit strip arata count per sursa de markere. Color coding: green (toate sursele > 0), gold (partial), red (niciun marker). Confirma vizual ca FIX-1 si FIX-6 functioneaza.

### 109.9 Duplicate Poll Detector (B3)

Guard dev-only activat via `localStorage.setItem('titanDebug', '1')`. Wrapper pe `invoke` cu ring buffer 50 entries. Daca aceeasi comanda Tauri apare de 2+ ori in 1 secunda, loggheaza `[TITAN] Duplicate poll detected:` in consola.

### 109.10 Chart Marker Stats Overlay (B4)

Micro-overlay pe chart (colt stanga-sus): `SIG:12 FILL:3 ST:8 TOT:23`. Semi-transparent (opacity 0.5), font 9px cyber-HUD. Actualizat automat la fiecare `_merge()`. CSS: `.chart-marker-stats`.

### 109.11 Incremental SessionAnalytics (B5)

`recalculate_stats()` din `titan-risk` nu mai face O(n) full-scan. Running totals `running_gross_wins` si `running_gross_losses` sunt mentenute incremental in `record_trade_with_duration` (O(1)). `restore_trade_pnls` face un singur pass O(n) la startup.

### 109.12 Optimizari IPC

- **get_risk_status:** Health badge consuma `window._lastRiskPayload` cached de cockpit update (5s). -1 IPC round-trip / 10s.
- **get_settings:** Regime strip consuma `window._lastSettingsPayload` cached de cockpit update. -1 IPC round-trip / 5s.
- **Indicator cache:** Eliminat clone redundant in `update_candle()` -- move semantics pe `merged`, un singur `.clone()` pe return.

---

## 110. Wave 3A — Persistence+Defaults (Sesiunea 72)

### 110.1 Settings Error Propagation (CRIT-7)

Toate cele 8 comenzi care salveaza settings pe disk (`update_settings`, `batch_update_settings`, `set_active_symbol`, `set_position_mode`, `set_dry_run`, `set_symbol_override`, `remove_symbol_override`, `apply_full_settings`) propagau eroarea cu `let _ =` -- orice esec de scriere era ignorat silentios.

**Fix:** Fiecare comanda returneaza `Err(e)` la frontend cu `tracing::error!`. Emite eveniment `settings:save-failed` vizibil in Save Event Timeline (B3).

### 110.2 Journal Atomic Save (CRIT-17)

`TradeJournal::save()` folosea `fs::write()` direct. Un crash la jumatatea scrierii putea corupe fisierul JSON.

**Fix:** Write `tmp` + `fs::rename()`. Pattern-ul este atomic pe POSIX -- fisierul final e fie complet vechi, fie complet nou, niciodata partial. Emite eveniment `journal:saved` pentru UI timeline.

### 110.3 Profile Load Validation (CRIT-18)

`load_profile` incarca un fisier JSON arbitrar ca profil de settings, fara nicio verificare de coerenta sau siguranta. Un profil exportat cu `max_loss_pct: 150` sau `trail_stop: -5` era acceptat fara eroare.

**Fix:** Dupa deserializare, apeleaza `validate_settings_coherence` + `validate_safety_settings`. Problemele gasite sunt returnate ca warnings in Result fara a bloca load-ul, permitand utilizatorului sa vada si sa corecteze.

### 110.4 Dry-Run Default True (CRIT-19)

Fresh install avea `dry_run_enabled: false` in `AppState::default()`. Un utilizator nou care pornea botul prima data rula automat cu bani reali.

**Fix:**
- `dry_run_enabled: true` default -- botul porneste intotdeauna in simulare
- `tracing::warn!("LIVE TRADING ENABLED -- dry_run is OFF")` logat la fiecare `apply_runtime_config` cand dry_run e dezactivat
- Test: `test_default_dry_run_is_true`

### 110.5 WS Auth via Query Parameter (CRIT-2)

WebSocket-ul `/ws` din web dashboard cerea token doar in header `Authorization: Bearer`. Browser-ul nativ WebSocket API (folosit de `new WebSocket(url)`) nu suporta custom headers.

**Fix:** Handler-ul `/ws` accepta token si via `?token=...` query parameter. Frontend-ul transmite `ws=new WebSocket(proto+'//'+location.host+'/ws?token='+encodeURIComponent(token))`. Route-ul `/ws` e separat in `ws_router` cu auth middleware propriu.

### 110.6 Trade PnLs Ring Buffer + Welford Sharpe (HIGH-17)

`trade_pnls: Vec<Decimal>` crestea nelimitat -- dupa 10,000+ trades, vectorul consuma memorie semnificativa si `sharpe_ratio()` facea O(n) full scan la fiecare apel.

**Fix:**
- `VecDeque<Decimal>` cu cap la **2000** entries (ring buffer). Entries vechi sunt evictate automat.
- **Welford online algorithm** pentru mean/variance incremental:
  - `sharpe_count`, `sharpe_mean`, `sharpe_m2` -- actualizate O(1) per trade
  - `sharpe_ratio()` calculeaza din statistici online, nu mai scaneaza vectorul
- `restore_trade_pnls` recomputa Welford dintr-un singur pass O(n) la startup
- 3 teste: `trade_pnls_capped_at_2000`, `sharpe_incremental_matches_full_scan`, `restore_trade_pnls_caps_to_limit`

### 110.7 HashMap TTL Eviction (HIGH-18)

4 HashMaps in `TradingState` cresteau nelimitat in sesiuni lungi:
- `fill_cumulative` — cumul fill-uri per simbol
- `failed_order_cooldown` — cooldown dupa erori ordine
- `trail_highs` / `trail_lows` — trailing stop per simbol

**Fix:** `evict_stale_entries()` cu politici TTL diferentiate:
| Map | TTL | Conditie |
|-----|-----|----------|
| `fill_cumulative` | 24h | Timestamp > 24h fara activitate |
| `failed_order_cooldown` | expired | Valoare timestamp < now |
| `trail_highs` / `trail_lows` | orphan | Fara pozitie activa pe simbol |

- Apelat la fiecare **100** trade updates via `eviction_tick` counter
- Counter atomic `evicted_entries` expus in `get_risk_status` pentru monitoring
- `fill_cumulative_ts: Mutex<HashMap<String, u64>>` — timestamp per entry
- 3 teste: `fill_cumulative_eviction_after_24h`, `cooldown_eviction`, `map_entries_total_counts_all`

### 110.8 WS Reconnect Staleness Flag (HIGH-19)

Dupa un disconnect+reconnect pe market data stream, primele 30 secunde de date pot avea gap-uri sau valori stale. Botul nu avea niciun mecanism de detectie.

**Fix:**
- `stream_stale: AtomicBool` setat la `true` la disconnect
- `stream_reconnected_at: AtomicU64` — timestamp reconectare
- `mark_stream_reconnected()` — setat la auth+subscribe reusit
- `mark_stream_data_received()` — clear stale flag dupa 30s warmup
- `is_stream_stale()` — returneaza `true` daca stale sau in warmup
- Expus in `get_risk_status` → badge `WS:STALE` in cockpit strip

### 110.9 Persistence Health Dashboard (B1)

Panou cyber-HUD cu grid 2x2 in zona de monitoring:

| Celula | Metrica | Culori |
|--------|---------|--------|
| **DISK** | Settings save status | green (ok) / red (fail) |
| **JRNL** | Journal save status | green (saved) / gold (dirty) |
| **PROF** | Profile loaded name | green (loaded) |
| **DRY** | Dry-run on/off | green (ON) / red (OFF cu neon-pulse) |

CSS: `.persist-health-grid`, `.persist-cell`, `.persist-val`. Culori neon cu pulsare pe stari critice.

### 110.10 Memory Pressure Gauge (B2)

Badge `MEM:N` in cockpit strip. `N` = numar total entries din cele 4 HashMaps critice din `TradingState`.

| Entries | Culoare | Semnificatie |
|---------|---------|--------------|
| < 500 | green | Normal |
| 500-2000 | gold | Crescut |
| > 2000 | red | Presiune mare |

Backend: `map_entries_total` adaugat in `get_risk_status` JSON.

### 110.11 Save Event Timeline (B3)

Timeline cronologica in Persistence Health panel cu chip-uri per eveniment save:

- **Events:** `settings:saved`, `settings:save-failed`, `journal:saved`, `snapshot:saved`
- **Ring buffer:** 12 entries (cele mai recente)
- **Format:** `HH:MM:SS LABEL` (ex: `14:32:07 SETTINGS OK`)
- **Culori:** `.save-ok` (green), `.save-fail` (red pulsant)
- Implementat via Tauri event listeners + `_pushSaveEvent` ring buffer

### 110.12 Dry-Run Guard Overlay (B4)

Cand `dry_run=false` (live trading):
- **Border:** `document.body` primeste clasa `.live-trading-border` — border rosu 3px solid cu box-shadow neon
- **Banner:** `div#live-trading-banner` cu text `LIVE TRADING — DRY RUN OFF` — pozitionat fixed top, z-index maxim, animatie `neon-pulse`
- Automat ascuns cand `dry_run=true`

### 110.13 Eviction Stats Counter (B5)

Badge `EVT:N` in cockpit strip. `N` = total entries evicted de `evict_stale_entries()` de la pornire.

| Evictions | Culoare |
|-----------|---------|
| < 100 | green |
| 100-1000 | gold |
| > 1000 | red |

Backend: `evicted_entries_total` expus in `get_risk_status` din `AtomicU64`.

### 110.14 Optimizari Persistence

**OPT-1: Batch Settings Save Debounce**
`batch_update_settings` executa un singur `save_settings_to_disk` la sfarsit, in loc de write per field. Reduce N disk writes la 1 per batch.

**OPT-2: Journal Save Debounce**
`TradeJournal` cu `dirty: bool` si `last_save_epoch: u64`. Save efectiv doar daca:
1. `dirty == true` (exista date nesalvate)
2. Au trecut >= 30s de la ultimul save

`flush()` fortat la `graceful_shutdown` pentru a nu pierde date. Reduce N disk writes la ~2/minut in perioadele active.

**OPT-3: Snapshot Delta Compression**
`save_state_snapshot` compara SHA-256 hash al snapshot-ului serializat cu hash-ul anterior (citit din `.sha256` sidecar file). Daca sunt identice, skip write complet. Reduce disk writes la 0 cand starea nu se schimba (idle periods).

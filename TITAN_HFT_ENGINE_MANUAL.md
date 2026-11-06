# TITAN HFT ENGINE — Detailed User Manual

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


**Version:** V7 Reloaded V5.9
**Tab:** HFT Engine (tab-4)
**Date:** March 28, 2026 (Session 19). Updated: April 3, 2026 (Session 69 — Wave 2B SmartTrend Correctness)
**Language:** Romanian
**Level:** Beginner → Advanced

---

## content

1. [Introduction](#1-introducere)
2. [Emergency Kill Dashboard](#2-emergency-kill-dashboard)
3. [Entry Parameters](#3-entry-parameters)
4. [Exit Parameters](#4-exit-parameters)
5. [Position Sizing](#5-position-sizing)
6. [Discord Alerts](#6-discord-alerts)
7. [Telegram Alerts](#7-telegram-alerts)
8. [Safety Limits](#8-safety-limits)
9. [Correlation Matrix](#9-correlation-matrix)
10. [Portfolio Performance](#10-portfolio-performance)
11. [ATR Dynamic Stops](#11-atr-dynamic-stops)
12. [LLM Sentiment](#12-llm-sentiment)
13. [Options Hedging](#13-options-hedging)
14. [Web Dashboard](#14-web-dashboard)
15. [Autopilot Core](#15-autopilot-core)
16. [Exit Scaling](#16-exit-scaling)
17. [Oracle Exit](#17-oracle-exit)
18. [Circuit Breaker](#18-circuit-breaker)
19. [Strategy Scoring](#19-strategy-scoring)
20. [Engine Stops](#20-engine-stops)
21. [Oracle Fusion](#21-oracle-fusion)
22. [Indicator Periods](#22-indicator-periods)
23. [Adaptive Exit Multipliers](#23-adaptive-exit-multipliers)
24. [Multi-Asset Control](#24-multi-asset-control)
25. [Effective Exit Values ​​(Live Impact Preview)](#25-effective-exit-values)
26. [Heatmap mode](#26-regime-heatmap)
27. [Parameter Profiles](#27-parameter-profiles)
28. [Risk Calculator Live](#28-risk-calculator-live)
29. [Parameter Health](#29-parameter-health)
30. [Auto-Suggest](#30-auto-suggest)
31. [Per-Symbol Overrides](#31-per-symbol-overrides)
32. [Preset Bar — Global Buttons and Controls](#32-preset-bar)
33. [Settings Diff Panel](#33-settings-diff-panel)
34. [DRY RUN vs LIVE](#34-dry-run-vs-live)
35. [Persistence of Settings](#35-persistenta-setarilor)
36. [Recommendations and Tips](#36-recomandari-si-tips)
37. [Glossary of Terms](#37-glosar-de-termeni)
38. [Keyboard Shortcuts and Command Palette](#38-keyboard-shortcuts-si-command-palette)
39. [TA Entry Gates — Pre-Entry Filters with Soft/Hard Mode](#39-ta-entry-gates)
40. [IndicatorCache — Hybrid Streaming + Oracle Cache](#40-indicatorcache--hybrid-streaming--oracle-cache)
41. [Audio/Visual Alerts — Titan Audio System](#41-audiovisual-alerts--titan-audio-system)
42. [Concurrent Autopilot Architecture (S5+S6)](#42-concurrent-autopilot-architecture-s5s6)
43. [Portfolio Risk Gates — VaR + HHI Concentration (S9)](#43-portfolio-risk-gates--var--hhi-concentration-s9)
44. [Feature Store — Entry Context Snapshot (S13)](#44-feature-store--entry-context-snapshot-s13)
45. [Microstructure Score Modifier — Stream VPIN + Spread Regime + Dual VPIN Divergence (S10-lite)](#45-microstructure-score-modifier)
49. [Safety Gates — VaR Fail-Closed, Asset Check, Intraday Blackout (S04.1)](#49-safety-gates--var-fail-closed-asset-check-intraday-blackout-s041)
50. [State Health Dots — Sub-State Decomposition Monitor (S04.5)](#50-state-health-dots--sub-state-decomposition-monitor-s045)
46. [Score Modifiers + Classical Bonuses + Entry Threshold Matrix (N4)](#46-score-modifiers--classical-bonuses--entry-threshold-matrix-n4)
47. [Oracle Weight Adaptation (S15)](#47-oracle-weight-adaptation-s15)
48. [Exit Inflight Guard (S39-S40)](#48-exit-inflight-guard-s39-s40)

---

## 1. INTRODUCTION

### What is the HFT Engine tab?

The **HFT Engine** tab (the 5th tab in the TITAN application, internally identified as `tab-4`) is the **command center** of the trading bot. Here you configure absolutely all the parameters that control the way the bot:

- Decide when to ENTER a transaction
- Decide when to EXIT a transaction
- How much to risk per transaction
- How to protect against big losses
- How to adapt to different market conditions

### How is it accessed?

Click on the **HFT Engine** tab in the top navigation bar of the application. It is the 5th tab (after Chart, Scanner, News, Journal).

### Basic principle

Each parameter in this tab:
1. Adjusts with a **slider** (horizontal bar that you drag left/right)
2. Displays the current value next to the slider
3. It is **saved on disk** by clicking on SAVE CONFIG
4. It is **strictly respected** by the bot during trading
5. **Survives** the restart of the application

### Conventions in this manual

- **Default** = the value that the bot comes with from the factory
- **Range** = the minimum-maximum range of the slider
- **Example** = concrete scenarios with real numbers
- **Impact** = what happens in the bot when you change parameter

---

## 2. EMERGENCY KILL DASHBOARD

### What is it?

A red, sticky panel (remains visible at the top) that provides instant access to emergency actions and critical account status information.

### Elements

| Element | What it shows | When it is important |
|---------|----------|-----------------|
| **KILL ALL** | Red emergency button | Press ONLY in case of extreme emergency |
| **BREAKER** | Circuit status Breaker | Constantly monitor |
| **EQUITY** | Total account value | Permanent reference |
| **DAILY P&L** | Profit/loss today | Track daily performance |
| **POSITIONS** | No. open positions | Check current exposure |

### KILL ALL button

**What it does:** After confirmation (`cyberConfirm`), liquidates all open positions and cancels all active orders. It is the equivalent of a "stop everything NOW".

**When to use it:**
- The market is moving extremely fast against you
- The Circuit Breaker has reached PANIC and you want additional manual intervention
- You detect an abnormal behavior of the bot
- You want to completely stop trading immediately

**Attention:** The action is IRREVERSIBLE after confirmation. Once confirmed, all positions are closed at the current market price (market orders). There is no undo.

### Circuit Breaker States

| Status | Color | What it means |
|-------|---------|-------------|
| **NORMAL** | Green | Everything is OK, the bot trades normally |
| **CAUTION** | Yellow | Moderate losses today, bot becomes more conservative |
| **HALT** | Orange | Significant losses, bot stops new entries |
| **PANIC** | Red (flash) | Serious losses, bot closes automatically |

**Example:** You have a $10,000 account. If you lose $300 today (3%), the breaker switches to CAUTION. If you lose $500 (5%), go into HALT. If you lose $800 (8%), PANIC and close everything.

---

## 3. ENTRY PARAMETERS

These parameters control **WHEN** the bot opens a new position (enters a trade).

### 3.1 Signal Strength Min

| Property | Value |
|-------------|---------|
| **Default** | 40 |
| **Range** | 10 — 95 |
| **What it does** | Minimum strength of input signal |

**Explanation for beginners:** The bot analyzes the market and generates a "score" of signal (from 0 to 100). This slider sets the minimum threshold: signals weaker than this value are ignored.

**Example:**
- Signal Strength Min = 40 → The bot comes in when the signal is 40 or higher
- Signal Strength Min = 70 → The bot comes in ONLY when the signal is very strong (70+)
- Signal Strength Min = 20 → The bot comes in even with weak signals (more inputs, but lower quality)

**Recommendations:**
- Beginners: 50-60 (fewer entries, but safer)
- Scalping: 30-40 (more entries)
- Conservative: 70-80 (very few, but quality)

### 3.2 OFI Threshold

| Ownership | Value |
|-------------|---------|
| **Default** | 5.0 |
| **Range** | 0 — 100 (displayed /10) |
| **What it does** | Minimum Threshold for Order Flow Imbalance |

**Explanation for beginners:** OFI (Order Flow Imbalance) measures the difference between buy and sell orders in the orderbook. A large positive OFI means that there are many more buyers than sellers.

**Example:**
- OFI Threshold = 5.0 → Bot asks for moderate imbalance before entering
- OFI Threshold = 20.0 → Bot asks for strong imbalance (fewer entries, but confirmed by real flow)
- OFI Threshold = 0 → Bot completely ignores order flow (not recommended)

### 3.3 Max Spread (bps)

| Property | Value |
|-------------|---------|
| **Default** | 50 bps |
| **Range** | 1 — 500 |
| **What it does** | Maximum spread accepted for input |

**Explanation for beginners:** The spread is the difference between the purchase price (ask) and the sale price (bid). A large spread means that you lose money from the start when you enter. 1 bps = 0.01%.

**Example:** If BTC is trading at $50,000:
- 50 bps = $25 spread → The bot enters if the spread is below $25
- 10 bps = $5 spread → The bot enters ONLY with a small spread (high liquidity)
- 200 bps = $100 spread → The bot accepts large spreads (not recommended for scalping)

**Recommendations:**
- Crypto: 30-80 bps
- Liquid stocks (AAPL, TSLA): 10-30 bps
- Less liquid stocks: 50-150 bps

### 3.4 Entry Cooldown (s)

| Property | Value |
|-------------|---------|
| **Default** | 30 seconds |
| **Range** | 5 — 3600 (1 hour) |
| **What it does** | Minimum time between two consecutive entries |

**Explanation for beginners:** After the bot makes an entry, it must wait at least this number of seconds before doing another. Prevents "over-trading" (too many transactions in a short time).

**Example:**
- Cooldown = 30s → The bot can enter maximum 2 times per minute
- Cooldown = 300s (5 min) → The bot waits 5 minutes between entries
- Cooldown = 5s → The bot can enter almost continuously (aggressive)

---

## 4. EXIT PARAMETERS

These parameters control **WHEN** the bot closes an existing position.

### 4.1 Trailing Stop %

| Property | Value |
|-------------|---------|
| **Default** | 1.5% |
| **Range** | 0.1% — 20.0% |
| **What it does** | The distance at which the trailing stop follows the price |

**Explanation for beginners:** The trailing stop moves UP with the price but does NOT move down. When the price drops by X% compared to the maximum reached, the position is closed automatically.

**Example with BTC at $50,000 and Trailing Stop = 1.5%:**
1. Buy at $50,000
2. BTC rises to $51,000 → Trailing stop moves to $51,000 - 1.5% = $50,235
3. BTC rises to $52,000 → Trailing stop moves to $52,000 - 1.5% = $51,220
4. BTC drops to $51,220 → ACTIVATES, the position is closed with a profit of ~$1,220

**Recommendations:**
- Scalping: 0.5% — 1.0%
- Day trading: 1.0% — 2.5%
- Swing trading: 3.0% — 5.0%

### 4.2 Take Profit %

| Property | Value |
|-------------|---------|
| **Default** | 2.5% |
| **Range** | 0.1% — 50.0% |
| **What it does** | The profit percentage at which the position is automatically closed |

**Explanation for beginners:** When the profit on a position reaches this percentage, the bot close the position and collect the profit. It's a "profit target".

**Example with AAPL at $200 and Take Profit = 2.5%:**
- Buy at $200
- Target: $200 + 2.5% = $205
- When AAPL reaches $205 → Position is automatically closed with profit of $5/share

**Attention:** Trailing stop and take profit work TOGETHER. The one who activates first "wins".

### 4.3 Max Hold Time

| Property | Value |
|-------------|---------|
| **Default** | 60 minutes (3600s) |
| **Domain** | 30s — 86,400s (24 hours) |
| **What does** | The maximum time the muzzle holds an open position |

**Explanation for beginners:** Even if neither the stop nor the take profit has been activated, after this time the position is closed automatically. Prevents situations in which a position remains "blocked" whole hours without movement.

**Example:**
- Max Hold = 60m → After 1 hour, the position is closed regardless of profit/loss
- Max Hold = 5m (300s) → Suitable for ultra-fast scalping
- Max Hold = 24h (86400s) → Allows overnight positions

---

## 5. POSITION SIZING

These parameters control **HOW** the bot risks per transaction.

### 5.1 Risk Per Trade %

| Property | Value |
|-------------|---------|
| **Default** | 2.0% |
| **Domain** | 0.1% — 25.0% |
| **What does** | The percentage of equity that he risks on a single trade |

**Explanation for beginners:** This is the maximum loss you accept on a single transaction, expressed as a percentage of the total value of the account.

**Example with $10,000 account and Risk Per Trade = 2%:**
- Maximum risk per trade: $10,000 x 2% = $200
- If the stop loss is 1%, the bot will calculate the size of the position so that the maximum loss is $200
- Position size: $200 / 1% = $20,000 notional

**The classic rule:** Most professional traders risk between 1-2% per trade. More than 5% is considered very risky.

### 5.2 Max Open Positions

| Property | Value |
|-------------|---------|
| **Default** | 1 |
| **Domain** | 1 — 20 |
| **What does** | The maximum number of positions open simultaneously |

**Example:**
- Max Open = 1 → The bot holds only one active trade (the safest)
- Max Open = 5 → The bot can have 5 trades open simultaneously
- Max Open = 10 → High risk, 10 active positions (requires high equity)

**Attention:** With Max Open = 5 and Risk = 2%, the total risk can be 5 x 2% = 10% of the account simultaneously.

### 5.3 Max Daily Trades

| Property | Value |
|-------------|---------|
| **Default** | 50 |
| **Domain** | 1 — 500 |
| **What does** | The maximum number of transactions per day |

**Example:**
- Max Daily = 50 → The bot makes a maximum of 50 trades per day (typical for HFT)
- Max Daily = 10 → Limited, suitable for conservative day trading
- Max Daily = 200 → Very aggressive, intense scalping

---

## 6. DISCOURSE ALERTS

### what is it

The Discord Alerts section sends notifications to a Discord server when the bot does important actions.

### How to configure (step by step)

1. **Create a Discord server** (or use an existing one)
2. **Create a Webhook:**
   - On the server → Settings → Integrations → Webhooks → New Webhook
   - Copy the webhook URL
3. **Paste the URL** in the "Webhook URL" field from HFT Engine
4. **Tick** the desired alerts:
   - **Alert on Trade Fill** — notification for each executed transaction
   - **Alert on Circuit Breaker** — notification when the breaker changes its state
   - **Alert Daily Summary** — summary at the end of the day
5. **Click TEST DISCORD** — check that the webhook is working

---

## 7. TELEGRAM ALERTS

### what is it

Similar to Discord, but on Telegram. You receive notifications on your mobile phone.

### How to configure (step by step)

1. **Create a Telegram Bot:**
   - Open Telegram, search for `@BotFather`
   - Send `/newbot`, follow instructions
   - You receive a **Bot Token** (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)
2. **Find out your Chat ID:**
   - Send a message to your bot
   - Access: `https://api.telegram.org/bot<TOKEN>/getUpdates`
   - Find `chat.id` in the answer
3. **Enter Bot Token** in the corresponding field
4. **Enter Chat ID** in the corresponding field
5. **Check "Enable Telegram Alerts"**
6. **Click TEST TELEGRAM** — check that you receive the message

---

## 8. SAFETY LIMITS

### 8.1 Daily Loss Limit %

| Property | Value |
|-------------|---------|
| **Default** | 3.0% |
| **Domain** | 0.5% — 20.0% |
| **What does** | Maximum daily loss accepted |

**Explanation for beginners:** When the total loss for the current day reaches this percentage of equity, the bot completely STOPS trading for the rest of the day.

**Example with $10,000 account and Daily Loss = 3%:**
- Daily limit: $10,000 x 3% = $300
- If you lose $300 in total today → The bot stops until tomorrow
- It protects you against "dark" days. which can destroy the account

**Recommendations:**
- Conservative: 2-3%
- Moderate: 3-5%
- Aggressive: 5-10% (not recommended for beginners)

---

## 9. CORRELATION MATRIX

### what is it

The correlation matrix shows how much the assets you trade move together.

### elements

**Max Corr (slider)**
| Property | Value |
|-------------|---------|
| **Default** | 0.70 |
| **Domain** | 0.00 — 1.00 |
| **What does** | The maximum correlation allowed between the assets held |

**Explanation:** If two assets have a correlation of 0.95, they move almost identically. Holding both is like having double the position on one. The slider limits this correlation.

**Example:**
- AAPL and MSFT have a correlation of ~0.85
- With Max Corr = 0.70, the bot will NOT hold simultaneous positions on AAPL and MSFT
- With Max Corr = 0.90, the bot allows them

**REFRESH button:** Recalculates the correlation matrix with the current data.

---

## 10. PERFORMANCE PORTFOLIO

### what is it

The Portfolio Performance panel shows the performance of each asset in the portfolio.

### elements

- **REFRESH** — Reload the performance data
- **REBALANCE** — Rebalances the portfolio according to the target weights

**What the table shows:**
- The asset (symbol)
- The percentage of the portfolio
- Performance (profit/loss)
- Volatility

---

## 11. ATR DYNAMIC STOPS

ATR (Average True Range) measures the real volatility of an asset. These parameters set stops and take-profits based on volatility, not on fixed percentages.

### 11.1 Stop-Loss (x ATR)

| Property | Value |
|-------------|---------|
| **Default** | 1.5x |
| **Domain** | 0.5x — 5.0x |
| **What does** | Stop loss as a multiple of ATR |

**Example:** If AAPL's 14 period ATR is $3:
- Stop = 1.5 x $3 = $4.50 below entry price
- Entry at $200 → Stop at $195.50

### 11.2 Take-Profit (x ATR)

| Property | Value |
|-------------|---------|
| **Default** | 3.0x |
| **Domain** | 1.0x — 10.0x |
| **What does** | Take profit as a multiple of ATR |

**Example:** With ATR = $3:
- Take = 3.0 x $3 = $9 above entry price
- Entry at $200 → Take profit at $209

### 11.3 Partial Take %

| Property | Value |
|-------------|---------|
| **Default** | 50% |
| **Domain** | 25% — 75% |
| **What does** | How much of the position is closed at the first profit target |

**Explanation:** Instead of closing the entire position at take profit, you close only a part (ex: 50%) and let the rest flow with a trailing stop.

**Example:** You have 100 AAPL shares. With Partial = 50%:
1. At the first target: sell 50 shares, collect partial profit
2. The other 50 continue with trailing stop
3. If the price continues to rise, you also win the second half

### 11.4 Partial Profit Taking (checkbox)

Enables/disables partial profit taking. When it is disabled, the entire position is closed at take profit.

---

## 12. LLM SENTIMENT

### what is it

Sentiment analysis based on AI (Large Language Models) that processes news and generates a sentiment score for traded assets.

### elements

| Elements | what is he doing |
|---------|---------|
| **API Key** | API key from OpenAI or Anthropic |
| **Provider** | OpenAI (GPT-4o-mini) or Anthropic (Claude Haiku) |
| **Enable** | Enable/disable sentiment analysis |

**How ​​it works:** The bot sends recent news about the asset to LLM and receives a sentiment score (bullish/bearish/neutral) which it includes in the trading decision.

**Cost:** Using LLMs involves costs per API call. GPT-4o-mini is the cheapest.

---

## 13. HEDGING OPTIONS

### what is it

Hedging with options is a protection strategy: you buy put options (which increase in value when the price decreases) to protect long positions.

### elements

### 13.1 OTM %

| Property | Value |
|-------------|---------|
| **Default** | 5% |
| **Domain** | 1% — 20% |
| **What does** | As for "out of the money" to be the purchased options |

**Example:** You have a long position on AAPL at $200. With OTM = 5%:
- The bot buys put with strike $200 - 5% = $190
- If AAPL falls below $190, the put protects you

### 13.2 Max Premium %

| Property | Value |
|-------------|---------|
| **Default** | 1.0% |
| **Domain** | 0.1% — 5.0% |
| **What does** | How much of the position value is spent on the option premium |

### 13.3 Auto-Hedge (checkbox)

Activate automatic hedging. When ON, the bot automatically buys put options for long positions.

### HEDGE ALL (button)

Apply hedging immediately on ALL open positions.

---

## 14. WEB DASHBOARD

### what is it

A local web server that allows you to monitor the bot from your browser. Listen exclusively on `127.0.0.1` (localhost). It includes the HTML cyber-HUD page with live status, API endpoints JSON, and WebSocket for real-time push updates.

### elements

| Elements | what is he doing | Default |
|---------|---------|---------|
| **Enable** | Starts/stops the web server | OFF |
| **Port** | The port it runs on (1024-65535) | 7777 |
| **URL** | Access address | http://127.0.0.1:7777 |
| **COPY URL** | Copy the URL to the clipboard | — |
| **Token** | Authentication token (automatically generated) | Random 64-char hex |
| **Rotate Token** | Regenerate the instant token (command `rotate_dashboard_token`) | — |

**How ​​to use:**
1. Tick ​​"Enable"
2. Open browser on `http://127.0.0.1:7777`
3. The page is automatically authenticated with the embedded token

### Security

**Authentication (unified S02, extended S72):** All HTTP endpoints use the `Authorization: Bearer <token>` header. The WebSocket endpoint `/ws` accepts tokens both in the header and via query parameter `?token=...` (necessary because the WebSocket API browser does not support custom headers). The token can be found in settings. Without a valid token, you get 401 Unauthorized. The only exception: `/api/health` does not require authentication.

**CORS (Cross-Origin Resource Security):**
- Only allows `http://127.0.0.1:<port>` and `http://localhost:<port>` (includes port)
- A malicious site opened in the browser cannot make requests to the dashboard
- Allowed methods: GET, POST
- Allowed headers: Authorization, Content-Type

**Rate Limiting:**
- Maximum 60 requests per minute on all protected routes (including WS upgrade)
- When exceeded: HTTP 429 (Too Many Requests)
- WS message rate limit: max 10 client messages per tick (2s), disconnect when exceeded
- Prevents accidental or intentional abuse

**Auth Failure Logging:**
- Every authentication failure is logged (warn)
- Every 5 consecutive failures: log error with brute-force alert
- The counter is reset upon successful authentication

**Token Rotation:**
- The Tauri `rotate_dashboard_token` command generates a new random token
- Save to disk automatically
- Useful if the token is compromised — no restart required

### WebSocket (Real-Time Push)

Endpoint: `/ws` (Bearer authentication on HTTP upgrade request)

| Message type | Frequency | What does it contain? |
|-----------|-----------|------------|
| `status` | In 2 seconds | equity, positions[], breaker, latency p50/p95 |
| `trade` | At every trade | action, symbol, side, pnl, duration_secs |
| `alert` | At watchdog recovery | messages |

The JavaScript client has auto-reconnect with exponential backoff (1s → 30s).

### API Endpoints

| Endpoint | The method | Auth | What does it return? |
|----------|--------|------|---------------|
| `/api/health` | GET | Not | `{"status":"ok","uptime_secs":N,"version":"..."}` |
| `/api/build-info` | GET | Not | `{"version":"...","git_hash":"...","build_date":"...","uptime_secs":N}` (S04) |
| `/` | GET | Bearer | HTML dashboard page (cyber-HUD) with build tags |
| `/api/status` | GET | Bearer | Equity, autopilot, breaker, strategy |
| `/api/positions` | GET | Bearer | List of open positions |
| `/api/trades` | GET | Bearer | Last 20 trades |
| `/api/stats` | GET | Bearer | Win rate, total PnL, profit factor |
| `/api/autopilot` | POST | Bearer | Toggle autopilot ON/OFF |
| `/ws` | GET | Bearer | WebSocket live push |

---

## 15. CORE AUTOPILOT

These 10 parameters control the fundamental engine of the autopilot — the main trading loop.

### 15.1 Min Hold (s)

| Property | Value |
|-------------|---------|
| **Default** | 180s (3 min) |
| **Domain** | 0 — 3600s |
| **What does** | The minimum time a position must be held |

**Explanation:** After entering, the bot will not leave its position for at least this number of seconds, even if it receives an exit signal. Prevents premature noise exits.

**Example:** Min Hold = 180s → After you buy, the bot waits at least 3 minutes before considering the exit.

### 15.2 Consensus %

| Property | Value |
|-------------|---------|
| **Default** | 70% |
| **Range** | 30% — 100% |
| **What it does** | Minimum percentage of GPU Nexus agents that must agree |

**Explanation:** GPU Nexus is running 150,000 agents of simulation. Consensus % sets how many must agree that a trade is good. 70% means that at least 105,000 agents must confirm.

### 15.3 Loop (ms)

| Property | Value |
|-------------|---------|
| **Default** | 1500ms |
| **Range** | 500 — 10,000ms |
| **What it does** | How often does the autopilot loop run |

**Example:** Loop = 1500ms → The bot checks the market 0.67 times per second. Loop = 500ms → Check 2 times per second (faster, but consumes more resources).

### 15.4 OFI Stale (ms)

| Property | Value |
|-------------|---------|
| **Default** | 10,000ms (10s) |
| **Range** | 1,000 — 60,000ms |
| **What it does** | After how many milliseconds the OFI data is considered "old" |

**Explanation:** If the last OFI was received more than X ms ago, the bot ignores it and does not enter based on it.

### 15.5 Min Bars

| Property | Value |
|-------------|---------|
| **Default** | 20 |
| **Range** | 5 — 200 |
| **What it does** | Minimum number of candles (bars) required before the first entry |

**Explanation:** The bot waits to have at least 20 price bars before making the first entry on an asset. Technical indicators (EMA, RSI, MACD) need enough data to calculate correctly.

### 15.6 OFI Entry Thr.

| Property | Value |
|-------------|---------|
| **Default** | 5.0 |
| **Range** | 0 — 50 (displayed /10) |
| **What it does** | Specific OFI Threshold for the input signal |

**Difference from OFI Threshold (Entry Parameters):** This is the internal threshold of the autopilot, not the one in the entry module. Both must be satisfied.

### 15.7 Max Qty Cap

| Property | Value |
|-------------|---------|
| **Default** | 1,000 |
| **Range** | 1 — 100,000 |
| **What it does** | Maximum quantity of units (shares/coins) per order |

**Example:** Max Qty = 1000 → The bot will not buy more than 1000 shares in a single order, regardless of how much the position sizing indicates.

### 15.8 Min Qty Thr.

| Property | Value |
|-------------|---------|
| **Default** | 0.001 |
| **Range** | 0.001 — varies |
| **What it does** | Minimum quantity per order (below this, the order is not sent) |

**Explanation:** Some assets (especially crypto) have very small lots. If position sizing calculates a quantity below the minimum threshold, the order is not sent.

### 15.9 Equity Fallback

| Property | Value |
|-------------|---------|
| **Default** | $10,000 |
| **Range** | $1,000 — $1,000,000 |
| **What do** | The equity value used if the Alpaca API does not respond |

**Explanation:** If the bot cannot read the real equity from the account (network timeout, API error), it uses this value as a temporary reference. Prevent orders with abnormal sizes.

### 15.10 Max Bypass Risk %

| Property | Value |
|-------------|---------|
| **Default** | 25% |
| **Range** | 1% — 100% |
| **What it does** | Maximum risk allowed in bypass mode (manual override) |

### 15.11 Generation-Aware Loop (S03)

Autopilot captures `session_generation` at startup. At each tick, check if the current generation matches. If not (eg: after a watchdog respawn), the old loop exits automatically:

```
"Generation mismatch — exiting stale autopilot instance"
```

This prevents operational **split-brain**: two instances of autopilot trading simultaneously after a respawn. The same protection applies to `trade_updates` and `streamer`.

**Generation-critical loops:** streamer, autopilot, trade_updates. The generation is automatically incremented by `spawn_loop_by_name` only for these three.

---

## 16. EXIT SCALING

These 17 parameters control how stop-loss, take-profit and trailing stop are calculated depending on volatility and trading mode.

### 16.1 Vol Factor Min / Max

| Parameter | Default | Range |
|-----------|---------|---------|
| Vol Factor Min | 0.6 | 0.1 — 2.0 |
| Vol Factor Max | 2.5 | 1.0 — 10.0 |

**What they do:** Limits the volatility factor that adjusts the stops. If the volatility is very low, the factor does not fall below Min; if it is very high, it does not exceed Max.

**Example:** With Vol Min = 0.6 and trail of 1.5%:
- Low volatility: adjusted trailing = 1.5% x 0.6 = 0.9% (not tighter than that)
- Normal volatility: trailing = 1.5% x 1.0 = 1.5%
- High volatility: adjusted trailing = 1.5% x 2.5 = 3.75% (not wider than that)

### 16.2 Fallback SL / TP / Trail %

| Parameter | Default |
|-----------|---------|
| Fallback SL % | 2.5% |
| Fallback TP % | 2.5% |
| Fallback Trail % | 1.5% |

**What I do:** Backup values used when the dynamic calculation (based on ATR/volatility) fails or returns invalid values.

### 16.3 Scalp Stop / Take / Trail Mult

| Parameter | Default |
|-----------|---------|
| Scalp Stop Mult | 0.70 |
| Scalp Take Mult | 0.60 |
| Scalp Trail Mult | 0.50 |

**What I do:** In scalping mode, stop/take/trail are MULTIPLYED with these multipliers, making them tighter.

**Example:** With trail of 1.5% and Scalp Trail Mult = 0.50:
- Normal mode: trail = 1.5%
- Scalping mode: trail = 1.5% x 0.50 = 0.75%

### 16.4 Partial 1st Target / Min Qty

| Parameter | Default | What it does |
|-----------|---------|---------|
| Partial 1st Target | 0.5 (50%) | Fraction of position closed at the first target |
| Partial Min Qty | 0.001 | Minimum quantity remaining after partial close |

### 16.5 ATR Period

| Property | Value |
|-------------|---------|
| **Default** | 14 |
| **Range** | 2 — 100 |
| **What it does** | Number of periods for ATR calculation |

**Explanation:** An ATR with 14 periods it uses the last 14 candles to calculate average volatility. Bigger = smoother, smaller = more reactive.

### 16.6 ATR Stop/Take/Trail Clamp Min/Max

| Parameter | Default Min | Default Max |
|-----------|-------------|-------------|
| ATR Stop Clamp | 0.5 | 8.0 |
| ATR Take Clamp | 1.0 | 15.0 |
| ATR Trail Clamp | 0.3 | 5.0 |

**What I do:** Limits ATR-based stop/take/trail values to a reasonable range. It prevents extreme values ​​when volatility explodes.

---

## 17. ORACLE EXIT

These 6 parameters control the exits based on Oracle AI (the bot's artificial intelligence system).

### 17.1 Exit Min Conf.

| Property | Value |
|-------------|---------|
| **Default** | 0.40 |
| **Range** | 0 — 1.00 |
| **What it does** | Oracle's minimum confidence to trigger exit |

**Example:** The Oracle analyzes the market and says "I am 60% sure that the price will decrease". With Exit Min Conf = 0.40, this confidence (0.60) exceeds the threshold, so the bot exits.

### 17.2 Exit Min Score

| Property | Value |
|-------------|---------|
| **Default** | 30 |
| **Domain** | 0 — 100 |
| **What does** | The minimum score of the Oracle output signal |

### 17.3 Flip Score Thr.

| Property | Value |
|-------------|---------|
| **Default** | 10 |
| **Domain** | 0 — 100 |
| **What does** | The score at which the Oracle considers the market to have "turned" (flip) |

**Example:** You are long (you bought). The Oracle detects a sell score = 15, with Flip Thr = 10. The score exceeds the threshold → The bot considers that the market has turned and exits.

### 17.4 Flip Confidence

| Property | Value |
|-------------|---------|
| **Default** | 0.20 |
| **Domain** | 0 — 1.00 |
| **What does** | Minimum confidence to confirm the flip |

### 17.5 Chaos Max Loss %

| Property | Value |
|-------------|---------|
| **Default** | -0.5% |
| **Domain** | -10.0% — 0% |
| **What does** | The maximum loss accepted in chaotic mode |

**Explanation:** When the Oracle detects that the market is in CHAOTIC mode, it applies a much tighter stop. If the loss reaches this percentage, it comes out immediately.

### 17.6 Chaos Min Conf.

| Property | Value |
|-------------|---------|
| **Default** | 0.25 |
| **Domain** | 0 — 1.00 |
| **What does** | Oracle's minimum confidence to declare chaotic regime |

---

## 18. CIRCUIT BREAKER

Account protection system. **The most important safety mechanism.**

### 18.1 Deposit %

| Property | Value |
|-------------|---------|
| **Default** | -3.0% |
| **Domain** | -20% — 0% |
| **What does** | Daily loss threshold for CAUTION mode |

**Example with $10,000 account:** When you lose $300 (3%) in one day → CAUTION is activated. The bot becomes more conservative (does not enter weak signals).

### 18.2 Halt %

| Property | Value |
|-------------|---------|
| **Default** | -5.0% |
| **Domain** | -30% — 0% |
| **What does** | Daily loss threshold for HALT mode |

**Example:** Lose $500 (5%) → HALT. The bot completely STOPS new entries. Existing positions continue to be managed (stop/trail).

### 18.3 Panic %

| Property | Value |
|-------------|---------|
| **Default** | -8.0% |
| **Domain** | -50% — 0% |
| **What does** | Daily loss threshold for PANIC mode |

**Example:** You lose $800 (8%) → PANIC. The bot CLOSES ALL positions and stops trading completely for the current day.

### 18.4 Max Notional Frac

| Property | Value |
|-------------|---------|
| **Default** | 0.25 (25%) |
| **Domain** | 0.01 — 1.00 |
| **What does** | The maximum fraction of equity that a single position can represent |

**Example with $10,000 account:** Max Notional = 0.25 → A single position cannot exceed $2,500 notional value.

### 18.5 Min Stop Frac

| Property | Value |
|-------------|---------|
| **Default** | 0.0001 (0.01%) |
| **Domain** | 0.0001 — 0.01 |
| **What does** | The minimum stop distance as a fraction of the price |

**Explanation:** Prevents too tight stops that would activate on noise. With 0.0001 on AAPL at $200, the minimum stop is at $200 x 0.0001 = $0.02 distance.

### 18.6 Flash Crash Detector

The Flash Crash Detector system monitors the price movement in real time and acts automatically when it detects abnormal market dislocations.

**Difference from Circuit Breaker:** Circuit Breaker measures your PnL (losses from positions). The Flash Crash Detector measures market price movement — it can detect a crash before your stop is hit.

#### 18.6.1 Flash Crash Window (seconds)

| Property | Value |
|-------------|---------|
| **Default** | 30 seconds |
| **Range** | 5 — 120 |
| **What it does** | Time window in which monitors price movement |

**Explanation:** The bot keeps track of all prices received in the last N seconds. If the difference between the maximum and minimum price in this window exceeds the threshold, a flash crash is declared.

**Example:** Window = 30s, AAPL trades at $200. In the last 30 seconds, the price was between $195 and $200 → variation = (200-195)/195 × 100 = 2.56%. If the threshold is 3%, it is not activated.

#### 18.6.2 Flash Crash Threshold (%)

| Property | Value |
|-------------|---------|
| **Default** | 3.0% |
| **Range** | 0.5 — 20.0 |
| **What it does** | The percentage of price movement that triggers the protection |

**Explanation:** If the price moves by more than X% in the monitoring window (in any direction — up or down), the bot considers it an abnormal dislocation and activates the protection.

**Recommendations:**
- **Large-cap stocks** (AAPL, MSFT, NVDA): 2-3% (very rarely moves this much in 30s)
- **Mid-cap volatile stocks**: 4-5%
- **Penny stocks / crypto**: 8-15% (higher normal volatility)

#### 18.6.3 Flash Crash Cooldown (minutes)

| Property | Value |
|-------------|---------|
| **Default** | 5 minutes |
| **Range** | 1 — 60 |
| **What it does** | How long it blocks new entries on the affected symbol after a flash crash |

**Explanation:** After detecting a flash crash on a symbol, the bot blocks any new entry on that symbol for N minutes. It prevents re-entry into an unstable market.

#### What happens when Flash Crash Detector is activated?

5 automatic actions, in order:

1. **Log** — detailed log recording with component="flash_crash"
2. **Frontend event** — emit "flash-crash" to UI (toast/visual alert)
3. **Auto-close** — automatically closes the open position on the affected symbol
4. **Telegram alert** — sends notification on Telegram (if enabled)
5. **Block** — blocks new entries on that symbol for `cooldown` minutes

### 18.7 Drawdown Velocity Detection

The Drawdown Velocity Detection system monitors the **speed** with which the drawdown increases and can block the inputs BEFORE the Circuit Breaker activates.

**Difference to Circuit Breaker:** Circuit Breaker measures **how much** you lost (magnitude). Drawdown Velocity measures **how fast** you lose (velocity). It can detect cascading losses in progress minutes before the absolute threshold is reached.

**Difference from Flash Crash Detector:** Flash Crash measures the price movement of the **market**. Drawdown Velocity measures the speed of **your** loss (of your equity).

#### 18.7.1 DD Velocity Threshold (%/min)

| Property | Value |
|-------------|---------|
| **Default** | -1.0 %/min |
| **Range** | -10.0 ... -0.1 |
| **What does** | Maximum tolerated loss speed, in percentage of peak equity per minute |

**Explanation:** If the drawdown increases by more than X% per minute (on the lookback window), the bot blocks all new entries. The value is negative by convention (the loss is negative).

**Examples:**
- **-1.0** (default): Block if you lose 1% of peak equity per minute or more
- **-0.5** (aggressive/protective): Block at 0.5%/min — more sensitive, fewer entries in volatile periods
- **-2.0** (relaxed): Allows faster losses before halt — suitable for strategies with more natural drawdown big

**Recommendations:**
- **Conservative scalping**: -0.5 (quick reaction to losses)
- **Day trading normal**: -1.0 (default, balance between protection and flexibility)
- **Swing/aggressive**: -2.0 (allows higher drawdown before intervention)

#### 18.7.2 DD Velocity Window (seconds)

| Property | Value |
|-------------|---------|
| **Default** | 300 seconds (5 minutes) |
| **Range** | 30 — 600 |
| **What it does** | The time window over which the drawdown speed is calculated |

**Explanation:** The speed is calculated by comparing the current drawdown with the one from N seconds ago. Shorter window = faster reaction but more sensitive to short spikes. Longer window = more stable but slower reaction.

**Recommendations:**
- **60-120s**: Very fast reaction, suitable for scalping on 1m
- **300s** (default): Good balance, filters short noise
- **600s**: Slow reaction, only sustained cascading losses activate the halt

#### What happens when DD Velocity Halt activate?

1. **Block entries** — all new entries are blocked (independent of the Circuit Breaker)
2. **Log** — periodic message in the UI: "DD VELOCITY HALT — Drawdown speed X.XX%/min exceeds threshold"
3. **Automatic recovery** — when the speed drops below the threshold, the entries are automatically re-activated (does not require a manual reset as in Circuit Breaker)
4. **Existing positions** are not affected — only applies to new entries

**Visible in API:** `get_risk_status` returns field `dd_velocity` (f64). Negative value = increasing drawdown. Value close to 0 = stable.

### 18.8 Global Breaker Synchronization with SmartTrend (S69)

From Session 69, `reset_circuit_breaker` also resets the SmartTrend status: `st_account.breaker_triggered`, `st_account.paused`, `settings.st_bot_paused`. Daily reset does the same when the breaker was active. Badge `BRK:SYNC` in ST SHIELD visually confirms that the two systems are synchronized.

---

## 19. STRATEGY SCORING

These 9 parameters control the strategy scoring engine — how the bot evaluates the quality of each input signal.

### 19.1 Base Score

| Property | Value |
|-------------|---------|
| **Default** | 50 |
| **Range** | 0 — 100 |
| **What it does** | Base Score at which the evaluation starts |

**Explanation:** The strategy starts with this score and adds/subtracts points based on the indicators. A high final score (eg: 75+) indicates a good input signal.

### 19.2 VPIN Gate Scalp / Normal

| Parameter | Default |
|-----------|---------|
| VPIN Gate Scalp | 0.85 |
| VPIN Gate Normal | 0.75 |

**What I do:** VPIN (Volume-Synchronized PIN) measures flow toxicity. If the VPIN exceeds the threshold, the bot DOES NOT enter — the market is too "toxic" (probably insider trading or flow opponent).

### 19.3 Min RVOL

| Property | Value |
|-------------|---------|
| **Default** | 0.5 |
| **Range** | 0 — 5.0 |
| **What it does** | Relative volume minimum required |

**Explanation:** RVOL = current volume / historical average. RVOL = 0.5 means that the volume must be at least 50% of the average. RVOL = 2.0 would require twice the average volume.

### 19.4 RVOL Window

| Property | Value |
|-------------|---------|
| **Default** | 20 |
| **Range** | 5 — 200 |
| **What it does** | How many periods are used for volume averaging |

### 19.5 RVOL Relax Thr. / Val.

| Parameter | Default | What it does |
|-----------|---------|---------|
| RVOL Relax Thr. | 3.0 | If RVOL exceeds this value, it "relaxes" filter |
| RVOL Relax Val. | 5.0 | The replacement value when relaxing |

**Explanation:** When the volume is VERY high (3x the average), it is probably a major event and the normal RVOL filter would be too restrictive.

### 19.6 Chaos Conf Kill / Chaos Regime Mult

| Parameter | Default | What it does |
|-----------|---------|---------|
| Chaos Conf Kill | 0.10 | Confidence under which the score cancels in chaos |
| Chaos Regime Mult | 0.55 | The multiplier applied to the Oracle contribution in chaotic mode |

---

## 20. ENGINE STOPS

These 6 parameters control the stops and trailing in the trading engine (function `manage_positions`).

### 20.1 Long Stop Normal %

| Property | Value |
|-------------|---------|
| **Default** | -2.5% |
| **Range** | -20% — 0% |
| **What it does** | Fixed stop loss for long positions in normal mode |

### 20.2 Long Stop Aggr %

| Property | Value |
|-------------|---------|
| **Default** | -8.0% |
| **Range** | -30% — 0% |
| **What do** | Fixed stop loss for long positions in aggressive mode |

**Difference:** Aggressive mode allows wider stops to give the position room to breathe.

### 20.3 Short Stop %

| Property | Value |
|-------------|---------|
| **Default** | -5.0% |
| **Range** | -20% — 0% |
| **What it does** | Stop fixed loss for short positions |

### 20.4 Long Trail Min Profit %

| Property | Value |
|-------------|---------|
| **Default** | 1.5% |
| **Range** | 0.1% — 10% |
| **What it does** | Minimum profit required before activating trailing stop on long |

**Example:** With Min Profit = 1.5% and long position on AAPL at $200:
- Trail activates ONLY after AAPL reaches $203 (+1.5%)
- Below $203, normal fixed stop applies

### 20.5 Short Trail Gap %

| Property | Value |
|-------------|---------|
| **Default** | 1.5% |
| **Domain** | 0.1% — 10% |
| **What it does** | Trailing stop distance on short positions |

### 20.6 Short Trail Min Profit %

| Property | Value |
|-------------|---------|
| **Default** | 1.0% |
| **Range** | 0.1% — 10% |
| **What it does** | Minimum profit required to activate trailing stop on short |

---

## 21. ORACLE FUSION

These 8 parameters control the weights and thresholds in the 7-layer Oracle AI system.

### 21.1 Kalman W / Entropy W / Hurst W / VPIN W / Wavelet W

| Parameter | Default | What measure |
|-----------|---------|------------|
| Kalman W | 0.30 | Kalman filter — price direction estimation |
| Entropy W | 0.25 | Entropy — degree of market disorder |
| Hurst W | 0.20 | Hurst exponent — trend persistence |
| VPIN W | 0.15 | VPIN — flow toxicity |
| Wavelet W | 0.10 | Wavelet — price frequency decomposition |

**Explanation:** Oracle combines 5 metrics into a final score. Weights (W) determine how much each metric matters.

**Example:** With Kalman W = 0.30 and Entropy W = 0.25, price direction (Kalman) matters more than market disorder (Entropy).

**Recommendation:** The sum of the weights should be close to 1.00 (not mandatory, but logical). Default: 0.30 + 0.25 + 0.20 + 0.15 + 0.10 = 1.00.

### 21.2 Conf Floor

| Property | Value |
|-------------|---------|
| **Default** | 0.25 |
| **Range** | 0 — 1.00 |
| **What it does** | Oracle's minimum confidence below which the signal is ignored |

### 21.3 Entropy Chaos Thr.

| Property | Value |
|-------------|---------|
| **Default** | 0.85 |
| **Range** | 0 — 1.00 |
| **What it does** | The entropy threshold above which the market is declared CHAOTIC |

### 21.4 VPIN Toxicity Alert

| Property | Value |
|-------------|---------|
| **Default** | 0.60 |
| **Range** | 0 — 1.00 |
| **What it does** | VPIN threshold above which the toxicity alert is issued |

---

## 22. INDICATOR PERIODS

These 12 parameters set the periods of classic technical indicators. Their modification DIRECTLY affects the BUY/SELL signals on the chart.

### 22.1 EMA Fast / Mid / Slow / Trend

| Parameter | Default | What it does |
|-----------|---------|---------|
| EMA Fast | 9 | Fast exponential moving average (fast reaction to price) |
| EMA Mid | 20 | Average moving average |
| EMA Slow | 50 | Slow moving average (medium term trend) |
| EMA Trend | 200 | Trend moving average (long term trend) |

**How the crossover works:**
- Fast EMA goes ABOVE Slow EMA = BUY signal
- Fast EMA goes UNDER Slow EMA = SELL signal

**Example:** With EMA Fast = 9 and EMA Slow = 50:
- More frequent signals but with more noise
- With EMA Fast = 20 and EMA Slow = 100:
- Rare but more reliable signals

### 22.2 RSI Period

| Property | Value |
|-------------|---------|
| **Default** | 14 |
| **Range** | 2 — 100 |
| **What it does** | Period Relative Strength Index |

**Explanation:** RSI measures the speed and magnitude of price movements. RSI < 30 = oversold (potential cumparare), RSI > 70 = overbought (potential sale).

### 22.3 MACD Fast / Slow / Signal

| Parameter | Default |
|-----------|---------|
| MACD Fast | 12 |
| MACD Slow | 26 |
| MACD Signal | 9 |

**Explanation:** MACD = difference between two EMAs. Signal line = EMA of MACD. MACD crossover above Signal = bullish, below = bearish.

### 22.4 BB Period / BB Std Dev

| Parameter | Default | What it does |
|-----------|---------|---------|
| BB Period | 20 | Bollinger Bands Period |
| BB Std Dev | 2.0 | Number of standard deviations |

**Explanation:** Bollinger Bands = moving average +/- N standard deviations. The price below the lower band = potentially oversold, above the upper band = potentially overbought.

### 22.5 ADX Period

| Property | Value |
|-------------|---------|
| **Default** | 14 |
| **Range** | 2 — 100 |
| **What it does** | Period Average Directional Index |

**Explanation:** ADX measures the strength of the trend (not the direction). ADX > 25 = strong trend, ADX < 20 = side square.

### 22.6 StochRSI Period

| Property | Value |
|-------------|---------|
| **Default** | 14 |
| **Range** | 2 — 100 |
| **What it does** | Stochastic RSI Period |

**Explanation:** StochRSI applies the Stochastic formula to the RSI, generating a more sensitive oscillator (0-1). Below 0.2 = oversold, above 0.8 = overbought.

---

## 23. ADAPTIVE EXIT MULTIPLIERS

This is the most advanced section of the HFT Engine. Control how the bot **automatically adjusts** stop/take/trail depending on the market regime, signal alignment and timeframe.

### How the cascade works

The final stop/take/trail values are calculated as follows:

```
Valoare efectiva = Valoare baza x Regime Mult x Signal Mult x TF Mult
```

**Full example:**
- Trailing Stop (base) = 1.5%
- Regime: CHAOTIC, Trail Mult = 0.50x
- Signal: ALIGNED, Trail Mult = 1.10x
- Timeframe: 1m, Trail Mult = 0.90x
- **Effective = 1.5% x 0.50 x 1.10 x 0.90 = 0.743%**

Trailing-ul s-a comprimat de la 1.5% la 0.743% pentru ca piata e haotica, dar semanalele sunt aliniate.

### 23.1 DISABLE ADAPTIVE EXITS (toggle)

When ON, ALL multipliers are ignored. The bot only uses the raw values ​​from the Entry/Exit sliders. Useful if you want total control without automatic adjustments.

### 23.2 Min Exit Floor %

| Property | Value |
|-------------|---------|
| **Default** | 0.30% |
| **Range** | 0.05% — 5.00% |
| **What it does** | The minimum value to which stop/take/trail can be compressed |

**Explanation:** Even if cascading multipliers would produce a value of 0.01%, Exit Floor prevents this — stop/take/trail will never be lower than 0.30%.

### 23.3 Regime Multipliers (9 sliders)

Each market regime has separate multipliers for Stop, Take and Trail:

| Mode | Stop (default) | Take (default) | Trail (default) |
|-------|---------------|----------------|-----------------|
| **Chaotic** | 0.70x | 0.60x | 0.50x |
| **Range** | 0.85x | 0.75x | 0.85x |
| **Trend** | 1.05x | 1.30x | 1.10x |

**Logic:**
- **Chaotic** (chaos): Stop/Take/Trail TIGHTER (< 1.0x) = exit quickly, protect capital
- **Range** (lateral): Moderately tighter
- **Trend** (directional): WIDER (> 1.0x) = let profit flow

**Example CHAOTIC:** Base Trail = 1.5%, Chaotic Trail = 0.50x → Effective Trail = 0.75%. The muzzle comes out faster in chaos.

**TREND Example:** Base Take = 2.5%, Trend Take = 1.30x → Effective Take = 3.25%. The bot lets the profit grow in the trend.

### 23.4 Signal Alignment Multipliers (5 sliders)

| Parameter | Default | When applied |
|-----------|---------|---------------|
| Aligned Stop x | 1.10x | When signals confirm the position |
| Aligned Take x | 1.25x | When signals confirm position |
| Aligned Trail x | 1.10x | When signals confirm position |
| Oppose Stop x | 0.85x | When signals contradict position |
| Oppose Take x | 0.80x | When the signals contradict the position |

**Logic:**
- **Aligned** (signals confirm): Mult > 1.0 = more space (signals support the position)
- **Opposite** (signals contradict): Much < 1.0 = tighter (signals warn)

### 23.5 Timeframe Multipliers (3 sliders)

| Parameter | Default |
|-----------|---------|
| Short TF Stop x | 0.90x |
| Short TF Take x | 0.90x |
| Short TF Trail x | 0.90x |

**What I do:** On short timeframes (1m, 3m), stop/take/trail are tighter (x 0.90). On long timeframes, a neutral multiplier (1.0x) is applied.

---

## 24. MULTI-ASSET CONTROL

### Toggle MULTI-ASSET TRADING

| Status | What it means |
|-------|-------------|
| **OFF** | The bot trades only one asset at a time |
| **ON** | The bot can trade multiple assets simultaneously |

### Max Concurrent Assets

| Property | Value |
|-------------|---------|
| **Default** | 5 |
| **Range** | 1 — 100 |
| **What it does** | Maximum number of assets traded simultaneously |

**Example:** With Max Concurrent = 5, the bot can have open positions simultaneously on AAPL, TSLA, BTC/USD, ETH/USD and MSFT. The 6th asset is ignored until one of the positions is closed.

**Caution:** More assets at once = more exposure. Make sure the equity supports it.

---

## 25. EFFECTIVE EXIT VALUES

### What is it?

The panel "EFFECTIVE EXIT VALUES" shows the REAL values ​​the bot uses for stop/take/trail AFTER applying all the adaptive multipliers.

### How to read

| Element | What it looks like | Color |
|---------|----------|---------|
| **STOP LOSS** | Effective stop loss percentage | Red |
| **TAKE PROFIT** | Effective take percentage profit | Green |
| **TRAILING** | The effective trailing stop percentage | Gold |

Under each effective value, **base:** — the gross value from the slider, without multipliers, also appears.

**Example:**
- Stop Loss: **1.05%** (base: 1.50%) → Multipliers compressed the stop
- Take Profit: **3.25%** (base: 2.50%) → Multipliers extended the take (probably in trend)
- Trailing: **0.75%** (base: 1.50%) → Trailing compressed (probably in chaos)

It updates automatically when you change any slider.

---

## 26. REGIME HEATMAP

### What is it?

Three colored boxes that visually show the active multipliers for each market regime and highlight the CURRENT regime.

### How to read

Each box shows:
- **S:** = Stop multiplier
- **T:** = Take multiplier
- **Tr:** = Trail multiplier

The active mode box has a shiny outline and a brighter background.

**Example:**
```
[TRENDING]        [RANGE]           [CHAOTIC]
 S:1.05x          S:0.85x           S:0.70x    ← activ (highlight)
 T:1.30x          T:0.75x           T:0.60x
 Tr:1.10x         Tr:0.85x          Tr:0.50x
```

If the CHAOTIC box is highlighted, it means that the Oracle has detected chaos on the market and the CHAOTIC multipliers are active.

It updates automatically every 5 seconds.

---

## 27. PARAMETER PROFILES (Disk Persistence)

### What is it?

A system of named profiles that allows saving and loading complete configurations. Profiles are stored on disk in `titan_profiles.json` -- they survive rebuilds, crashes, and can be exported/imported between instances.

### How to create a profile

1. Configure all the sliders as you want
2. Click SAVE CONFIG (save to disk)
3. Write a name in the "Profile name..." field. (ex: "Crypto Scalp Night")
4. Click **SAVE AS**
5. The profile appears in the dropdown

### How to load a profile

1. Select the profile from the dropdown
2. Click **LOAD**
3. **ALL** parameters are restored -- including TA Gates, Scan Params, Symbol Overrides (nested objects)
4. An undo snapshot is automatically created before load

### How to delete a profile

1. Select the profile from the dropdown
2. Click **DELETE**
3. Confirm the deletion in the cyber dialogue
4. The profile is permanently deleted from the disk

### How to export/import a profile

1. **EXPORT:** Select the profile, click **EXPORT** -- se download a file `titan_profile_<name>.json`
2. **IMPORT:** Click **IMPORT**, select the JSON file -- the profile is imported to disk (the name is taken from the file or from the text field)

**Import validations (from Session 44):**
- **Schema version check** — if the version of the scheme in the file does not correspond to the current one, the import is rejected with a clear message
- **Round-trip serde** — the JSON payload is serialized and deserialized back; if it loses data, the import fails
- **Coherence warnings** — after a successful import, warnings are logged if the values ​​are not coherent (eg: breaker thresholds in the wrong order, negative stop loss)

### Automatic migration

If you have old profiles from localStorage (previous versions), they are automatically imported to disk on the first run and the old key is deleted.

**Practical use:**
- Create a "Crypto BTC" profile with settings optimized for Bitcoin
- Create a "Stocks Conservator" with safe settings for actions
- Export your optimal profile and import it on another car
- Quickly switch between profiles according to market conditions

---

## 28. RISK CALCULATOR LIVE

### What is it?

Panel with 4 risk metrics calculated in real time from the current settings.

### Metrics

| Metric | What it shows | Color |
|---------|----------|---------|
| **MAX LOSS / TRADE** | Maximum possible loss on a single trade, in dollars | Red |
| **POSITION SIZE** | Size of the position in dollars | Blue |
| **RISK/REWARD** | The ratio between the assumed risk and the target profit | Green |
| **MAX DAILY LOSS** | The maximum daily loss in dollars | Gold |

**Example with $10,000 account, Risk = 2%, Stop = 1.5%, Take = 2.5%:**
- Max Loss / Trade: $10,000 x 2% = **$200**
- Position Size: $200 / 1.5% = **$13,333**
- Risk/Reward: 1.5% : 2.5% = **1:1.67**
- Max Daily Loss: $10,000 x 3% (daily limit) = **$300**

It is updated automatically when you change the sliders.

---

## 29. PARAMETER HEALTH

### What is it?

An automatic diagnostic system that checks 15 conflict/risk rules and displays warnings.

### Types of badges

| Badge | Color | What it means |
|-------|---------|-------------|
| **OK** | Green | Everything is fine |
| **WARN** | Orange | Caution — possibly suboptimal |
| **DANGER** | Purple | Danger — risky setting |

### Examples of checked rules

- **Stop > Take** → DANGER: "Stop loss is bigger than take profit — you will lose more than you win"
- **Risk > 5%** → WARN: "Risk per trade too high for a beginner"
- **Circuit Breaker disabled** → DANGER: "No daily loss protection system"
- **Max Open Positions > 5** → WARN: "Many simultaneous positions may increase exposure"
- **Trailing > Take** → WARN: "Trailing wider than take profit — take profit will always activate first"

It is updated in real time with any slider change.

---

## 30. AUTO-SUGGEST

### What is it?

Function that analyzes the last 50 trades in the log and generates automatic suggestions for adjusting the parameters.

### How to use it

1. Click **ANALYZE TRADES**
2. The bot analyzes the results from the log
3. Cards with suggestions are displayed
4. Each suggestion has an **APPLY** button that automatically applies the adjustment
5. Click **CLOSE** when you are done

### Examples of suggestions

- "High stop-out rate (>60%). Suggestion: Increase trailing stop by 0.5%." → [APPLY]
- "Average profit per trade is under $10. Suggestion: Increase take profit by 1%." → [APPLY]
- "Many exits on max hold time. Suggestion: Increase Max Hold from 60m to 120m." → [APPLY]

### 7 patterns detected

1. High stop-out rate
2. Average profit too low
3. Frequent exits on max hold
4. Ratio suboptimal risk/reward
5. Too many daily trades
6. Losses concentrated on one certain asset
7. Trailing stop ineffective (never activates)

---

## 31. PER-SYMBOL OVERRIDES

### What is it?

Allows the setting of CUSTOM parameters per symbol, which override the global settings for that asset only.

### How to set an override

1. Open the section (click on "PER-SYMBOL OVERRIDES")
2. Write the symbol in the "Symbol" field. (ex: AAPL)
3. Fill in the desired fields:
   - **Trail%** — trailing stop custom (leave blank for global value)
   - **Take%** — take profit custom
   - **Risk%** — risk per trade custom
4. Click **SET**

### How to delete a override

1. Write the symbol in the "Symbol" field
2. Click **REMOVE**

### Practical example

Global settings: Trail = 1.5%, Take = 2.5%, Risk = 2.0%

Override for BTC/USD: Trail = 3.0%, Take = 5.0% (crypto is more volatile, so wider stops)
Override for AAPL: Risk = 1.0% (stable stock, lower risk)

The list of active overrides is displayed below the buttons.

---

## 32. PRESET BAR

Bar of presets and global controls at the bottom of the tab.

### Elements (from left to right)

| Element | What it does |
|---------|---------|
| **Search** | Filter sliders by name |
| **SAFE DEFAULT** | Apply conservative preset to ALL sliders |
| **Lock SAFE** | Switch on/off — lock the SAFE preset |
| **SCALP DEFAULT** | Apply aggressive scalping preset |
| **Lock SCALP** | Switch on/off — lock the SCALP preset |
| **SAVE CONFIG** | Save the settings current on disk |
| **EXPORT** | Export the configuration in a JSON file |
| **IMPORT** | Import the configuration from a JSON file |
| **UNDO** | Restore the settings before the last modification |
| **SHOW CHANGES** | Displays a table with unsaved changes |

### Search (Parameter search)

Write in the search field and only the sliders that contain the text remain visible. Completely empty sections are hidden.

**Example:** Write "trail" → Only sliders with "trail" appear. in names: Trailing Stop, Scalp Trail Mult, ATR Trail Min, etc.

### Preset locks

When a lock is ON:
- The preset is automatically applied every time the application is started
- The sliders CANNOT be modified manually (they are locked)
- Only one preset can be locked at a time
- The state of the lock is saved in localStorage

### EXPORT / IMPORT

- **EXPORT:** Download a `.json` file with all current settings. Useful as a backup or to transfer settings to another computer.
- **IMPORT:** Load a previously exported `.json` file and apply all settings from it. From Session 44, the import sends the JSON payload directly (typed) through the `apply_full_settings` command, eliminating the `String(v)` coercion that could corrupt nested objects (TA Gates, Symbol Overrides).

### UNDO

It restores the settings to the state before the last important change (applied preset, import, etc.). It is saved on disk in `titan_settings_undo.json`, so it survives restart. From Session 44, UNDO uses the same `apply_full_settings` command as import — it sends the complete snapshot as JSON typed, ensuring faithful restoration including for complex fields.

### Batch Update Transactional (Session 44)

When applying a preset or changing several parameters simultaneously, `batch_update_settings` uses a **snapshot + rollback** mechanism:

1. A clone of the current state is made
2. Each parameter is applied individually
3. If any one fails — **all** state returns to the snapshot (full rollback)
4. Save to disk and runtime synchronization only after complete success

Result: there is no more risk of partially saved configuration on disk after an error.

---

## 33. SETTINGS DIFF PANEL

### what is it

Displayable panel through the **SHOW CHANGES** button that shows a table with ALL unsaved changes.

### How to read

| Column | What does it look like? |
|---------|----------|
| **Parameter** | The name of the changed setting |
| **Saved** | Value that is on disk (last save) |
| **Current** | The current value in the slider |

Different values ​​are highlighted in color:
- Red = old (saved) value
- Green = new value (current)

**Example:**
```
Parametru          | Salvat | Curent
trailing_stop_pct  |  1.5%  |  2.0%   ← modificat
take_profit_pct    |  2.5%  |  3.0%   ← modificat
risk_per_trade     |  2.0%  |  2.0%   (nemodificat — nu apare)
```

Only MODIFIED parameters appear in the table. If there are no changes, "No unsaved changes" is displayed.

---

## 34. DRY RUN vs LIVE

### The DRY/LIVE badge

The badge in the right corner of the title shows the bot's current mode:

| Mode | Badge | What does the bot do? |
|-----|-------|---------------|
| **DRY RUN** | Blue | SIMULATE trading — do not send real orders |
| **LIVE** | Green (flashing) | EXECUTE real orders on the market |

### How it changes

The badge is changed from the main sidebar (not from the HFT Engine). In HFT Engine it is only a readonly indicator.

**Attention:** In LIVE mode, every decision of the bot translates into REAL orders with REAL money. Make sure all settings are correct before going LIVE.

**Default (S72):** Fresh install starts with `dry_run_enabled: true`. Live trading must be activated explicitly. When live trading is active, the UI displays a pulsating red border + banner `LIVE TRADING — DRY RUN OFF`.

---

## 35. PERSISTENCE OF SETTINGS

### Where is it saved?

| Set | File on disk | Location |
|--------|---------------|---------|
| All HFT Engine parameters | `titan_settings.json` | The config folder of the application |
| Undo snapshot | `titan_settings_undo.json` | Next to titan_settings.json |
| Chart settings | `titan_chart_settings.json` | Next to titan_settings.json |
| profile sites | `titan_profiles.json` | Next to titan_settings.json (+ auto-backup `.bak`) |
| Preset locks | `localStorage` | Internal Browser/WebView |

### What survives the restart

| Event | HFT settings | Profiles | Locks |
|-----------|-----------|----------|-------|
| Toggle between tabs | YES | YES | YES |
| Close + reopen the application | YES | YES | YES |
| Application crash | YES | YES | Possible |
| Rebuild with build.sh | YES | YES | NOT* |

(*) `build.sh` can clear the WebView cache (`localStorage`). Settings from `titan_settings.json` and profiles from `titan_profiles.json` always survive -- they're on disk, not in the browser.

### How is it saved?

1. Change the sliders as you wish
2. Click **SAVE CONFIG**
3. Toast "Settings saved" confirm
4. The settings are on the disk and will be restored at the next start

**Attention:** If you change the sliders but do NOT SAVE CONFIG, the changes are lost on restart.

### Persistence Protections (S72)

- **Error propagation:** All 8 save commands propagate the error to the frontend (they are no longer silently ignored). Event `settings:save-failed` visible in Save Event Timeline.
- **Journal atomic save:** `TradeJournal::save()` uses `tmp` + `rename` pattern -- the file cannot be corrupted on crash.
- **Profile validation:** `load_profile` validates the consistency and security of the settings after loading. Warnings returned in Result.
- **Snapshot delta compression:** The snapshot is written only if it differs from the previous one (SHA-256 comparison).
- **Journal debounce:** Effectively save only if there is new data and 30s have passed. Forced flush at shutdown.

### Runtime Sync (from Session 44)

At each save / load profile / batch update, the `apply_runtime_config()` function automatically synchronizes the runtime fields in TitanState with the saved values. Synchronized fields: strategy_mode, nexus_fusion_alpha, dry_run, circuit_breaker thresholds, flash_crash params, smarttrend config. No manual action is needed to ensure consistency between settings and runtime.

**S08 Update:** `StrategyMode` now includes 5 variants: MathOracle, GpuNexus, Fusion, SignalStrategy, **SmartTrend**. SmartTrend mode activates the dedicated scoring pipeline (SmartTrendEngine with FSM, markers ST BUY/SELL/EXIT/TRAIL/STALK/BREAK, kill switch with confirm token). When `st_killed == true`, autopilot blocks all SmartTrend entries.

---

## 36. RECOMMENDATIONS AND TIPS

### For beginners

1. **Start with DRY RUN** — test a few days in simulation before LIVE
2. **Use SAFE DEFAULT** — conservative values ​​are the safest to begin with
3. **Do not change all parameters at once** — change one parameter at a time and observe the effect
4. **Monitor Parameter Health** — if you see DANGER, correct it before going LIVE
5. **Risk Per Trade below 2%** — the golden rule of risk management
6. **Export configuration** — before experimenting, export the settings as a backup

### For advanced

1. **Create profiles per market** — different settings for crypto vs stocks
2. **Use Per-Symbol Overrides** — fine-tune on individual assets
3. **Adjusts Adaptive Multipliers** — customizes the response to regimes
4. **Monitor Regime Heatmap** — understand which regime is active and how it affects trades
5. **Run Auto-Suggest** — after 50+ trades, analyze the patterns

### Frequent mistakes to avoid

1. **Stop > Take** — you lose more than you win on each trade
2. **Circuit Breaker too weak** — you can lose the whole account in one day
3. **Max Positions too high with high Risk** — total exposure can exceed 50% of the account
4. **Adaptive Exits disabled without understanding** — you lose protection in the chaotic market
5. **Ignoring UNDO** — after a wrong change, UNDO save

---

## 37. GLOSSARY

| Term | Explanation |
|--------|-----------|
| **ATR** | Average True Range — measures average volatility over N periods |
| **Bollinger Bands** | Bands around the moving average, based on standard deviation |
| **Circuit Breaker** | Safety system that stops trading at large losses |
| **Crossover** | When one line crosses over/under another (trading signal) |
| **DRY RUN** | Simulation — the bot calculates but does not send real orders |
| **EMA** | Exponential Moving Average — weighted average with emphasis on recent data |
| **Entropy** | Measure of disorder/chaos in price movements |
| **Flash Crash** | Extreme and rapid price movement; the automatic detector closes positions and blocks entries (see §18.6) |
| **Equity** | Total account value (cash + open positions) |
| **Hurst Exponent** | Persistent trend measure (>0.5 = trending, <0.5 = mean-reverting) |
| **HFT** | High-Frequency Trading — high-frequency trading |
| **Kalman Filter** | Estimation algorithm that predicts price direction |
| **LIVE** | Real mode — the bot sends real orders with real money |
| **MACD** | Moving Average Convergence/Divergence — trend and momentum indicator |
| **Notional** | Total dollar value of a position |
| **OFI** | Order Flow Imbalance — imbalance between buy and sell orders |
| **Oracle** | AI system of bot with 7 layers of analysis |
| **Overbought** | Price too high compared to the average, potential for downward correction |
| **Oversold** | Price too low compared to the average, potential for an upward return |
| **Regime** | Current state of the market: Trending, Range or Chaotic |
| **RSI** | Relative Strength Index — momentum oscillator (0-100) |
| **RVOL** | Relative Volume — current volume compared to the historical average |
| **Scalping** | Trading strategy with very short positions (seconds-minutes) |
| **Spread** | The difference between the purchase price (ask) and the sale price (bid) |
| **StochRSI** | Stochastic RSI — RSI applied to RSI, more sensitive |
| **Stop Loss** | The order that closes the position at the set maximum loss |
| **Take Profit** | The order that closes the position at the target profit |
| **Trailing Stop** | Stop that moves with the price in the direction of profit |
| **VPIN** | Volume-Synchronized PIN — measures the toxicity of the flow |
| **Wavelet** | Frequency decomposition of the price |
| **DD Velocity** | The speed with which the drawdown increases, measured in %/min — see §18.7 |
| **Command Palette** | Quick command search overlay, activated with Ctrl+K — see §38 |

---

## 38. KEYBOARD SHORTCUTS AND COMMAND PALETTE

Titan HFT offers a set of global shortcuts for quick actions in scalping and a palette of commands with fuzzy search.

### 38.1 Direct Shortcuts

They work at any time (regardless of the active tab), except when the focus is on an input field or texting.

| Short cut | Action | Observations |
|----------|---------|------------|
| **Ctrl+K** | Open the Command Palette | It always works, including from the input |
| **Ctrl+Enter** | Quick Buy | Place a market buy order on the chart symbol |
| **Ctrl+Shift+Enter** | Quick Sell (market) | Place a market sell order on the chart symbol |
| **Ctrl+Shift+X** | Flatten All | Liquidate ALL positions (emergency liquidated) |
| **Ctrl+P** | Toggle Autopilot | Turns the autopilot on/off |
| **Ctrl+]** | Longer timeframe | Cycle: 1m → 3m → 5m → 10m → 15m → 30m → 1h → 4h → 1d |
| **Ctrl+[** | Shorter timeframe | It cycles in the opposite direction: 1d → 4h → 1h → ... → 1m |

### 38.2 Command Palette (Ctrl+K)

Press **Ctrl+K** from anywhere in the application to open the command palette.

**How ​​it works:**
1. An overlay appears with a search field and list of orders
2. Type to filter commands (fuzzy search)
3. Navigate with **Arrow Up/Down** and confirm with **Enter**
4. Or click on the desired order
5. **Escape** closes the palette

**19 orders available:**

| Category | Command | Shortcut (if any) |
|-----------|---------|------------------------|
| **Trade** | Quick Buy | Ctrl+Enter |
| **Trade** | Quick Sell (market) | Ctrl+Shift+Enter |
| **Trade** | Flatten All Positions | Ctrl+Shift+X |
| **Control** | Toggle Autopilot | Ctrl+P |
| **Navigate** | Go to Chart | — |
| **Navigate** | Go to AI News | — |
| **Navigate** | Go to Journal | — |
| **Navigate** | Go to HFT Engine | — |
| **Navigate** | Go to Performance | — |
| **Navigate** | Go to AI Scanner | — |
| **Navigate** | Go to Watchlist | — |
| **Navigate** | Go to HFT Targets | — |
| **Chart** | Timeframe Higher | Ctrl+] |
| **Chart** | Timeframe Lower | Ctrl+[ |
| **Chart** | Timeframe → 1m | — |
| **Chart** | Timeframe → 5m | — |
| **Chart** | Timeframe → 15m | — |
| **Chart** | Timeframe → 1h | — |
| **Chart** | Timeframe → 1d | — |

**Symbol switch:** If the text entered in the palette does not correspond to any command, pressing Enter treats it as a trading symbol and switches the chart to that symbol. Example: Ctrl+K → "TSLA" → Enter = the chart changes to TSLA.

### 38.3 Compatibility with Chart Hotkeys

Existing chart shortcuts (L=line, R=ray, H=hline, V=vline, F=fib, T=text, M=measure, C=crosshair, A=alert, P=replay) still work normally. They are activated only without Ctrl, so they do not interfere with the Ctrl+* shortcuts above.

---

## 39. TA ENTRY GATES — PRE-ENTRY FILTERS WITH SOFT/HARD MODE

The **"TA ENTRY GATES"** section (with an orange title) appears in the HFT Engine tab, just above Strategy Scoring. It contains 6 technical filters that are applied after the Oracle decides Buy/Short, but before executing the entry.

### 39.1 How it works

Each gate checks a technical indicator and can either **penalize** the score (Soft) or **block** the entry completely (Hard).

| Mode | Effect | When you use it |
|-----|-------|-------------------|
| **Off** | The gate does nothing | You don't want this filter |
| **Soft** | The trade score is reduced by Soft Penalty points | You want feedback, but you don't want to block |
| **Hard** | The entry is completely blocked (the order is not placed) | You want a guarantee that you do not enter in bad conditions |

### 39.2 The 6 Gates

| Gate | Indicator | When it is activated |
|------|-----------|-------------------|
| **RSI** | Relative Strength Index | Long when RSI > overbought (70). Short when RSI < oversold (30) |
| **EMA Align** | EMA 9/20/50 | Long when EMAs are not aligned bullish. Short when they are not decreasing |
| **ADX** | Average Directional Index | When ADX < threshold (25) — trend too weak to enter |
| **BB Squeeze** | Bollinger Bands width | When the bands are too narrow (< 2% of average price) — consolidation |
| **MACD** | MACD Histogram | Long cand histogram < 0. Short cand histogram > 0 |
| **Stoch RSI** | Stochastic RSI | Long when Stoch > 80. Short when Stoch < 20 |

**RSI** Thresholds** (`rsi_overbought`, `rsi_oversold`) and **ADX** (`adx_trend_threshold`) are the ones already existing in the Classical Filters section. They do not need to be configured separately.

### 39.3 Presets

The dropdown to the right of the title allows quick selection of a predefined profile:

| Preset | What it sets |
|--------|-----------|
| **Strict** | All 6 gates on Hard — maximum quality, few inputs |
| **Balanced** | RSI + ADX on Soft, the rest Off — moderate filter |
| **Oracle Only** | All Off (default) — Oracle decides by itself |

After selecting a preset, you can manually adjust the individual gates. The dropdown returns to "Custom" to any manual modification.

### 39.4 Soft Penalty

The slider under the gate grid controls **how many points** are deducted from the score per Soft gate violated.

| Property | Value |
|-------------|---------|
| **Default** | 15 |
| **Domain** | 1 — 50 |
| **What it does** | Penalty per gate Infringed software |

 Penalties are accumulate. If 3 Soft gates are violated simultaneously and penalty = 15, the score decreases by 45 points. A low enough score prevents natural entry (it falls below the regime threshold).

### 39.5 Fail-Open on Lack of Data

If an indicator is `None` (there are not enough candles for calculation — RSI requires 14, ADX 14, EMA50 requires 50), the respective gate is **skipped**. It does not block or penalize.

### 39.6 JSON Settings

Fields in `titan_settings.json`:

```json
{
  "ta_gates": {
    "rsi_gate": "Off",
    "ema_alignment_gate": "Off",
    "adx_gate": "Off",
    "bb_squeeze_gate": "Off",
    "macd_gate": "Off",
    "stoch_rsi_gate": "Off"
  },
  "ta_gate_soft_penalty": 15.0
}
```

Possible values per gate: `"Off"`, `"Soft"`, `"Hard"`.

### 39.7 Visual Pipeline

```
Oracle decide Buy/Short
    ↓
TA Gates (RSI → EMA → ADX → BB → MACD → StochRSI)
    ↓ Hard gate failed?  → return None (BLOCAT)
    ↓ Soft gates failed?  → score -= penalty * nr_failed
    ↓
Classical Bonus (daca activ)
    ↓
Return TradeDecision cu scorul final
```

---

## 40. INDICATOR CACHE — HYBRID STREAMING + ORACLE WITH CACHE PER SYMBOL

### 40.1 What is

IndicatorCache eliminates repeated recalculation of technical indicators in autopilot. Instead of recalculating all indicators (EMA, RSI, MACD, BB, ADX, VWAP, StochRSI, Oracle) from scratch on 200 candles every cycle, the cache stores the result and returns it instantly when the candle has not changed.

### 40.2 How it works

The autopilot evaluates each candidate symbol at each cycle (~1 second). Between candles (eg: at TF 5m, 300 cycles per candle), the indicators do not change. The cache exploits this property:

| Situation | What does the cache do | Latency |
|----------|-------------------|---------|
| Same candle (95%+ of cycles) | Returns stored result | ~0 |
| New candle | StreamingIndicators O(1) + Oracle batch | < 5ms |
| First time per symbol | Full batch calculation | 10-50ms |

### 40.3 Hybrid strategy

- **Classic indicators** (EMA 9/20/50/200, RSI, MACD, BB, ADX, VWAP, StochRSI): calculate incrementally by `StreamingIndicators` — only the last candle is processed, the rest of the state is persistent
- **Oracle** (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD): recalculated batch on all candles, but only once per new candle — not every cycle

The two results are combined in one complete `Indicators` snapshot.

### 40.4 Eviction

The cache automatically deletes entries not accessed for more than 1 hour. This prevents memory growth for symbols that have been evaluated only once and are no longer active.

Eviction runs at ~1% of autopilot cycles (probabilistic check, not dedicated timer).

### 40.5 Guarantee of correctness

At cold start (first call per symbol), the cache runs `calculate_all_with_periods()` completely — exactly the same function as the path without cache. This guarantees that the result is BIT-IDENTICAL. After the cold start, the incremental updates were validated with proptest at 1e-10 accuracy.

### 40.6 What is NOT cached

- **Dashboard score** (the Strategy Score display in the header) — uses `compute_strategy_score_with_mode()` which is a separate pure function
- **Backtest** — runs in isolation with different data, does not use the live cache
- **ATR for stop-loss** — calculated separately by `calculate_atr()`, not by IndicatorCache

### 40.7 Updated Pipeline

```
Autopilot ciclu
    ↓
get_candles(simbol, TF)  ← Ring Buffer sau REST fallback
    ↓
IndicatorCache.get_or_compute(simbol, TF, candles)
    ├─ Cache hit?  → return instant (zero calcul)
    ├─ Candle nou? → StreamingIndicators.update() + Oracle batch → merge → store
    └─ Cold start? → batch complet → store
    ↓
check_entry(simbol, candles, indicators)
    ↓
TA Gates (RSI → EMA → ADX → BB → MACD → StochRSI)
    ↓
Trade Decision
```

---

## 41. AUDIO/VISUAL ALERTS — TITAN AUDIO SYSTEM

### 41.1 What is

TitanAudio adds sound and visual alerts on critical trading events. Sounds are generated procedurally with the Web Audio API (no external audio files). The goal: not to miss fills, breakers or flash crashes when you are not actively looking at the screen.

### AUDIO ON/OFF button

In the TITAN SAFETY panel header, next to the circuit breaker badge, there is the `AUDIO: ON` / `AUDIO: OFF` button.

- **Click** — toggle instant ON↔OFF
- **Persistence** — the state is saved in localStorage and restored on restart
- **Visual** — cyan when ON, gray when OFF

### 41.2 Monitored events

| Event | Sound | Effects visuals |
|-----------|-------|----------------|
| Trade fill (buy/sell) | Blip ascending 3 tones (520→780→1040 Hz) | Flash title: "FILL: SIDE SYMBOL" (2.5s) |
| Breaker CAUTION | Pulsating alarm 3x (600+750 Hz) | Flash title + yellow screen glow |
| Breaker HALT/PANIC | Pulsating alarm 3x (600+750 Hz) | Flash title + red screen glow |
| Flash crash detected | Aggressive descending buzz (440→330→220 Hz) | Flash title + screen glow magenta |
| Watchdog dead loops | Triple triangle ping (400→500→400 Hz) | Flash title: "WATCHDOG: N DEAD" |
| Breaker NORMAL (reset) | — (no sound) | — |

### 41.3 Screen glow

Semi-transparent fullscreen overlay with animated border glow (3 pulses). It deactivates automatically after 2-5 seconds. Colors indicate severity:

- **Red** — PANIC/HALT breaker (most severe)
- **Yellow** — CAUTION breaker
- **Magenta** — flash crash detected

### 41.4 Flash title

Alternating document.title with the alert message at 500ms. Useful when the Titan window is minimized or on another monitor — the title flashes in the taskbar.

### 41.5 Technical notes

- AudioContext is created on the first click/keypress (Chrome autoplay policy)
- Default volume: 0.35 (configurable via the console: `TitanAudio.setVolume(0.5)`)
- Sounds also exist for `fill-exit-profit`, `fill-exit-loss`, `signal` — available programmatically through `TitanAudio.play('fill-exit-profit')` but not connected automatically (the trade-fill payload in Rust does not yet contain PnL)

---

---

## 42. CONCURRENT AUTOPILOT ARCHITECTURE (S5+S6)

### 42.1 What it is

Autopilot uses `futures::future::join_all` to evaluate positions and candidates in parallel. Decisions (close_position, send_order, state mutations) are applied sequentially after gather.

### 42.2 Flow on a tick

1. **Parallel Exit Gather** — for each open position, `evaluate_position_exit()` is run concurrently: get_candles, calculate_atr, signal markers, adaptive stops, force exit checks. Returns `ExitEvaluation` (read-only struct, zero side effects).

2. **Sequential Exit Apply** — iterate `Vec<ExitEvaluation>`:
   - Partial take → `close_position` + update partial_taken
   - Full exit → `close_position` + journal + trail cleanup
   - After any exit: `continue 'autopilot` (re-fetch positions)

3. **Sequential Pre-filter** — for each candidate: flash crash check, holding check, pending orders (API call), cooldown, position limit, correlation guard

4. **Parallel Entry Gather** (non-SignalStrategy) — `evaluate_candidate_entry()` competitor: get_candles, indicator_cache.get_or_compute, check_entry. Returns `EntryEvaluation`.

5. **Best-Score Selection** — from all evaluations with Signal::Buy or Signal::Short, the candidate with the highest score is selected. OFI + Nexus filters applied to the selected candidate. Then send_order.

### 42.3 Best-Score vs First-Match

Previously, the autopilot executed the first candidate that passed the filters (the order was arbitrary from the list). Now, all the candidates are evaluated, and the one with the **highest score** is chosen. Advantage: higher quality entries when there are several candidates at the same time.

### 42.4 Locking Safety

All locks in parallel code are short-lived reads (<1us):
- `candle_buffers` (Mutex) — snapshot()
- `indicator_cache` (Mutex) — get_or_compute()
- `vol_cache`, `symbol_best_tf` (Mutex) — read
- `ofi_state` (ArcSwap) — load(), lock-free

Mutations and API calls remain on the main task: `trail_highs/lows`, `session_analytics`, `close_position`, `send_order_internal`.

### 42.5 When it helps the most

- Many open positions (5+) with ATR calculation per symbol
- Ring buffer empty (REST fallback 50-500ms per symbol — all done in parallel)
- SignalStrategy with many assets (pre-filtering is sequential, but after it the evaluation is parallel)

---

## 43. PORTFOLIO RISK GATES — VaR + HHI Concentration (S9)

### 43.1 What are

Two complementary gates at **portfolio** level that block entries when the total risk is too high. It is applied AFTER OFI and Nexus filters, BEFORE position sizing.

- **VaR (Value at Risk)** — statistical risk based on volatility and inter-asset correlation
- **HHI (Herfindahl-Hirschman Index)** — measure of capital concentration

### 43.2 Location in UI

**PORTFOLIO RISK GATES** card (purple, `#E040FB`) in the HFT Engine tab, after TA Entry Gates. Two sub-panels: VaR (left) and HHI (right).

### 43.3 Parameters

| Parameter | Element ID | Settings Backend | Default | rank |
|-----------|-----------|-----------------|---------|-------|
| VaR Gate toggle | `hft-var-gate` | `enable_var_gate` | OFF | bool |
| Max Portfolio VaR % | `hft-var-max` | `max_portfolio_var_pct` | 2.0 | 0.1-10.0 |
| Lookback Periods | `hft-var-lookback` | `var_lookback_periods` | 20 | 5-100 |
| Concentration Gate toggle | `hft-hhi-gate` | `enable_concentration_gate` | OFF | bool |
| Max HHI | `hft-hhi-max` | `max_concentration_hhi` | 0.40 | 0.10-1.00 |

The VaR % slider has div=10 (slider 1-100, display 0.1-10.0%). The HHI slider has div=100 (slider 10-100, display 0.10-1.00).

### 43.4 How VaR Gate works

1. Extract close prices from `CandleRingBuffer` per symbol in the portfolio
2. Compute log returns: `r_t = ln(close_t / close_{t-1})` over the last `var_lookback_periods` periods
3. Constructs the NxN covariance matrix (sample covariance, N-1 denominator)
4. Calculate parametric VaR: `VaR = z_score(0.99) * sqrt(w' * Sigma * w)` where w = position weights
5. If the proposed VaR (with the new position included) > `max_portfolio_var_pct` → entry BLOCKED

**z_score(0.99) = 2.326** — we use 99% confidence (not 95%) for more aggressive protection in scalping.

**CVaR (Expected Shortfall)** is calculated internally but not used as gate — available for future analysis.

### 43.5 How HHI Gate works

```
HHI = sum(w_i^2)     unde w_i = |notional_i| / total_notional
```

- HHI = 1.0 → 100% in a single symbol
- HHI = 0.25 → 4 equal positions (diversified)
- HHI = 0.10 → 10 equal positions

If adding the proposed position would take HHI over `max_concentration_hhi`, the entry is blocked.

### 43.6 Pipeline position

```
check_entry() → Signal Buy/Short + scor
  → TA Gates (Soft/Hard penalties)
    → OFI Filter (flow direction)
      → Nexus Consensus (multi-agent)
        → HHI Gate (concentrare) ← S9
          → VaR Gate (risc statistic) ← S9
            → Position Sizing
              → send_order_internal()
```

HHI is evaluated before VaR because it is O(1) (pure arithmetic), while VaR requires the construction of the covariance matrix.

### 43.7 Recommendations for use

- **Start with both OFF** — observe the existing behavior for a few sessions
- **Activate HHI first** — it's easier to understand and doesn't depend on historical data
- **HHI 0.40** is a good compromise — it allows max 2-3 concentrated positions, but blocks all-in on a single symbol
- **VaR 2.0%** is conservative — increases to 3-5% if your strategy generates multi-symbol entries frequently
- **Lookback 20** is the default — increase to 50 if you want to capture volatility over a longer horizon

### 43.8 Difference with Circuit Breaker

| appearance | Circuit Breaker (§18) | VaR/HHI Gate (§43) |
|--------|----------------------|-------------------|
| When it acts | AFTER loss | BEFORE entry |
| What measure | Daily PnL, drawdown | Volatility, correlation, concentration |
| Level | Total portfolio (PnL) | Proposed per-entry |
| Action | Reduce positions / stop trading | Blocks the specific entry |
| Reset | Manual | Automatic (conditions change) |

### 43.9 Technical implementation

**Backend:** `titan-risk` crate — `compute_returns()`, `covariance_matrix()`, `parametric_var()`, `conditional_var()`, `var_gate_blocks_entry()`, `concentration_hhi()`, `hhi_gate_blocks_entry()`

**Integration:** `src-tauri/src/loops/autopilot.rs` — block of ~45 lines after OFI+Nexus checks

**Frontend:** `src/index.html` (HTML card) + `src/titan_perf_stats.js` (event handlers + restore + batch save)

---

## 44. FEATURE STORE — ENTRY CONTEXT SNAPSHOT (S13)

### 44.1 What is

Feature Store captures the complete context of indicators and strategy at the time of entering a trade. The snapshot (20 fields) is temporarily stored in memory and attached to `TradeRecord` when the trade is closed (full or partial exit).

### 44.2 Why it matters

Without the Feature Store, the log only contains PnL and timestamp. With Feature Store, each trade responds to:
- What did the Oracle say? (score, confidence, regimen)
- What were the indicators? (RSI, ADX, EMA, VPIN, Hurst)
- How does the microstructure look? (OFI, spread, imbalance)
- What score did the strategy have? (strategy_score, entry_type, FVG)

### 44.3 TradeFeatures fields

**Oracle:** oracle_score, oracle_confidence, oracle_regime

**Indicators:** rsi, adx, atr_pct, ema9, ema50, vpin, hurst, rvol

**Microstructure:** ofi, spread_bps, imbalance_ratio

**Strategy:** strategy_score, signal_source, fvg_type, entry_type, ta_gates_passed, ta_gates_total

### 44.4 Technical flow

1. **Entry successful** → `snapshot_entry_features()` reads from IndicatorCache (`peek()`) + OfiState (`ArcSwap.load()`) + TradeDecision → stores in `state.entry_features` HashMap
2. **Full exit** → `entry_features.remove(symbol)` → passes to `TradeRecordInput` → journal on disk
3. **Partial exit** → `entry_features.get(symbol).cloned()` (DO NOT delete) → journal on disk

### 44.5 Backward compatibility

`#[serde(default)]` on field `entry_features: Option<TradeFeatures>`. Old journals are loaded with `None`. Zero breaking changes.

### 44.6 Component `peek()` on IndicatorCache

Read-only method that returns a clone of the cached snapshot without triggering re-computation or updating the access time. Used exclusively by the Feature Store.

### 44.7 Impact on performance

Zero measurable overhead:
- `peek()` = HashMap lookup + clone (~1us)
- `OfiState.load()` = ArcSwap atomic load (~1ns)
- `entry_features.insert()` = HashMap insert under Mutex (~1us)
- Total: <5 microseconds per entry, executed only once per trade

---

## 45. MICROSTRUCTURE SCORE MODIFIER — STREAM VPIN + SPREAD REGIME + DUAL VPIN DIVERGENCE (S10-lite)

### 45.1 What it does

Applies **soft score modifiers** to entries based on microstructure signals in real time. Inserted in `evaluate_candidate_entry()` after `check_entry()` and before the ranking of the candidates.

### 45.2 Stream VPIN

VPIN calculated from trade-by-trade flow (Lee-Ready classification), not from candle BVC:

- **Volume-synced bins**: bucket closes at `bucket_vol_threshold` (5000 default)
- **20 buckets** rolling, EMA smoothing `alpha = 0.15`
- Stored in `OfiState.stream_vpin` (ArcSwap, lock-free)
- **More accurate** than candle VPIN on short timeframes (1m, 5m)

### 45.3 Spread Regime

Dual EMA on `spread_bps` with narrowing detection:

- **EMA fast**: `alpha = 0.15` (~13 ticks half-life)
- **EMA slow**: `alpha = 0.03` (~66 ticks half-life)
- **Narrowing**: `ema_fast < ema_slow * 0.80`
- Stored in `OfiState.spread_narrowing` (ArcSwap, lock-free)
- **Meaning**: market makers tighten the spread when anticipating movement

### 45.4 Dual VPIN Divergence

Compare stream VPIN (tape) vs candle VPIN (oracle). Divergence = information:

| Divergence | Interpretation | Score |
|-----------|-------------|-------|
| > +0.15 | Hidden toxicity (tape sees what the candles don't show) | **-8** |
| < -0.15 | Filtered false alarm (candle VPIN is noise) | **+5** |
| [-0.15, +0.15] | Concordance | **0** |

### 45.5 Complete table score modifiers

| Condition | Score | Cumulative |
|----------|-------|-----------|
| Spread narrowing | **+5** | Yes |
| NBBO imbalance aligned ( | Sawmill | > 0.3) | **+4** | Yes |
| Dual VPIN false alarm (div < -0.15) | **+5** | Yes |
| Dual VPIN hidden toxicity (div > +0.15) | **-8** | Yes |
| Extreme VPIN Stream (> 0.85) | **-10** | Yes |

**Possible Range:** +14 (all bonuses) to -18 (all penalties)

### 45.6 Position in the pipeline

```
check_entry() -> TradeDecision (score base)
        |
apply_microstructure_modifier() -> score ajustat
        |
best candidate selected (max score)
        |
OFI filter / Nexus filter (binary)
        |
VaR / HHI gates (binary)
        |
position sizing + send_order
```

Microstructure modifier is applied AFTER scoring but BEFORE ranking. It affects which candidate is selected, not only if he enters.

### 45.7 Extended TradeFeatures

Two new fields on `TradeFeatures` (backward compatible with `#[serde(default)]`):

- `vpin_divergence: f64` — stream_vpin minus candle_vpin at entry
- `spread_narrowing: bool` — flag spread regime at entry

The existing `vpin` field now contains stream_vpin (more precisely than candle VPIN).

### 45.8 OfiState new fields

4 fields added to `OfiState` (l2_heatmap.rs):

- `stream_vpin: f64` — VPIN from trade flow
- `spread_ema_fast: f64` — EMA fast spread (diagnostic)
- `spread_ema_slow: f64` — EMA slow spread (diagnostic)
- `spread_narrowing: bool` — derived flag

All with `Default = 0/false`. ArcSwap lock-free, zero contention added.

### 45.9 Extended event frontend

`ofi-update` event now includes `stream_vpin` and `spread_narrowing` for UI display.

### 45.10 Impact on performance

Zero measurable overhead:
- `StreamVpin.update()` = VecDeque push + arithmetic (~100ns per trade)
- `SpreadRegime.update()` = 2 EMA updates (~10ns per quote)
- `apply_microstructure_modifier()` = ArcSwap load + 5 comparisons (~50ns per candidate)
- `OfiState` remains below 128 bytes, ArcSwap atomic store unchanged

---

---

## 46. ​​SCORE MODIFIERS + CLASSICAL BONUSES + ENTRY THRESHOLD MATRIX (N4)

### 46.1 What it does

It exposes ~50 numerical values ​​(previously hardcoded in `strategy.rs`) as configurable parameters in the UI. Organized in 3 new cards between STRATEGY SCORING and ENGINE STOPS.

### 46.2 Card: SCORE MODIFIERS (orange header)

21 sliders that directly control how `calculate_score()` and `check_ta_gates()` calculate the entry score:

| Slider | Backend Field | Default | Effect |
|--------|--------------|---------|-------|
| Chaos Kill Penalty | `sm_chaos_kill_penalty` | -50 | Penalty when confidence < chaos threshold |
| Chaos Kill Fallback | `sm_chaos_kill_fallback` | 50 | Score returned when oracle is missing |
| Chaos Conf x MeanRev | `sm_chaos_conf_mult_mean_rev` | 1.5 | Chaos threshold multiplier on MeanRev |
| Chaos Conf x Chaotic | `sm_chaos_conf_mult_chaotic` | 2.5 | Multiplier on Chaotic |
| Chaos Conf x Unknown | `sm_chaos_conf_mult_unknown` | 1.5 | Multiplier on Unknown |
| Regime x MeanRev | `sm_regime_mult_mean_rev` | 0.82 | Oracle contribution weight on MeanRev |
| Regime x Chaotic | `sm_regime_mult_chaotic` | 0.55 | Weight on Chaotic |
| Regime x Unknown | `sm_regime_mult_unknown` | 0.73 | Weight on Unknown |
| VPIN Penalty High | `sm_vpin_penalty_high` | -20 | When VPIN > scalping gates |
| VPIN Penalty Med | `sm_vpin_penalty_medium` | -10 | When VPIN > normally gate |
| FVG Bull Bonus | `sm_fvg_bullish_bonus` | +15 | Bullish FVG detected |
| FVG Bear Penalty | `sm_fvg_bearish_penalty` | -15 | Bearish FVG detected |
| RVOL Tier1 Thr. | `sm_rvol_tier1_threshold` | 2.0 | Tier 1 volume threshold |
| RVOL Tier1 Bonus | `sm_rvol_tier1_bonus` | +8 | Bonus RVOL > 2x |
| RVOL Tier2 Thr. | `sm_rvol_tier2_threshold` | 4.0 | Tier 2 volume threshold |
| RVOL Tier2 Bonus | `sm_rvol_tier2_bonus` | +12 | Bonus RVOL > 4x |
| StochRSI OB | `sm_stoch_rsi_overbought` | 80 | Overbought threshold |
| StochRSI OS | `sm_stoch_rsi_oversold` | 20 | Oversold threshold |
| BB Squeeze Width | `sm_bb_squeeze_width` | 0.02 | Squeeze detection threshold |
| Strong Label | `sm_entry_label_strong` | 80 | Score > threshold = "STRONG" |
| Mild Label | `sm_entry_label_mild` | 60 | Score > threshold = normal entry |

### 46.3 Card: CLASSICAL BONUSES (green header)

6 sliders that control the bonuses in `compute_classical_confirmation()`. The card appears semi-transparent (opacity 0.5) when Classical Filters is OFF.

| Slider | Backend Field | Default | Effect |
|--------|--------------|---------|-------|
| RSI Bonus | `cb_rsi_bonus` | 10.0 | RSI confirmation bonus |
| EMA Align Bonus | `cb_ema_alignment_bonus` | 15.0 | Bonus EMA20/50 alignment |
| ADX Bonus | `cb_adx_bonus` | 10.0 | Bonus trend strength |
| BB Squeeze Bonus | `cb_bb_squeeze_bonus` | 5.0 | Bonus Bollinger squeeze |
| VWAP Bonus | `cb_vwap_bonus` | 10.0 | Bonus price vs VWAP |
| Head | `cb_cap` | 50.0 | Maximum bonus limit |

### 46.4 Card: ENTRY THRESHOLD MATRIX (header cyan)

Grid 7 regimes x 3 values ​​(Long, Short, Confidence) with direct numerical inputs. The Scalping/Normal dropdown switches between the two sets.

**Regimes:** Trending, Trending Aggro, MeanRev, MeanRev Aggro, Chaotic, Unknown, Unknown Aggro.

Interpretation:
- **Long threshold**: oracle_score must be > this value for BUY
- **Short threshold**: oracle_score must be < this value for SHORT
- **Confidence threshold**: oracle_confidence must be > this value

Chaotic Regime does not have the Aggro variant (it is already maximally restrictive).

### 46.5 VALIDATE button

Call `invoke('validate_thresholds')` and check:
- long > 0, short < 0, conf in [0.05, 0.95]
- Scalp thresholds <= Normal (scalping = more permissive)
- Trending <= Chaotic (natural gradient of restrictiveness)
- StochRSI OB > OS, RVOL tier1 < tier2

Warnings displayed as toasts (does not block the save).

### 46.6 RESET DEFAULTS button

Restores all values ​​to defaults for the currently selected mode (Scalp or Normal). Save automatically on the backend.

### 46.7 Persistence

All values ​​are saved in `AppSettings` / `titan_settings.json`. Entry thresholds as `HashMap<String, EntryThreshold>`, score modifiers and classical bonuses as structs with `#[serde(default)]`. Old settings are loaded correctly (defaults inserted automatically).

### 46.8 Relationship with other sections

- **§19 Strategy Scoring**: Base Score, VPIN Gates, RVOL, Chaos Kill -- these parameters control `calculate_score()` along with Score Modifiers (§46.2)
- **§39 TA Entry Gates**: StochRSI OB/OS and BB Squeeze Width from Score Modifiers also affect TA gates
- **§45 Microstructure**: Microstructure modifiers (spread narrowing, VPIN divergence) are applied AFTER the scoring controlled by §46

---

## 47. ORACLE WEIGHT ADAPTATION (S15)

### 47.1 Toggle

| Property | Value |
|-------------|---------|
| **Settings** | `enable_oracle_adaptation` |
| **Default** | OFF |
| **What does** | Enables the auto-tuning of Oracle weights based on the PnL of the trades |

When it is OFF, the weights from §21 are used directly (identical behavior to the previous one). When it is ON, the weights are automatically adapted per regime.

### 47.2 How adaptation works

The Oracle has 5 directional weights (§21.1): Kalman, Entropy, Hurst, VPIN, Wavelet. With active adaptation:

1. At each trade entry, the 5 directional signals of the Oracle and the market regime are captured
2. At each output, the signals are correlated with the PnL direction (profit = +1, loss = -1)
3. Every 50 trades, the weights are recalculated proportionally to the absolute correlation per layer

### 47.3 Per-Regime Adaptation

The adaptation maintains **4 separate** sets of weights, one per `MarketRegime`:

| Regime | Description | Typical dominant weight |
|-------|-----------|--------------------------|
| Trending | Directional market | Kalman (directional predictor) |
| MeanReverting | Oscillating market | Hurst (persistence signal) |
| Chaotic | The messy market | The weights tend to be uniform (no layer is reliable) |
| Unknown | Indeterminate regime | The weights remain close to the bases |

At entry, the weights adapted for the CURRENT regime of the market (from the last cached cycle) are used.

### 47.4 Confidence-Weighted Learning

The learning rate (alpha) varies with the confidence of Oracle at entry:

| Confidence | Effectively Alpha | Effect |
|------------|--------------|-------|
| 0.95 | 0.095 | It contributes maximally to learning |
| 0.50 | 0.050 | Contributes medium |
| 0.10 | 0.010 | Contribute minimally (filtered) |

Trades where Oracle was unreliable do not pollute the adaptation.

### 47.5 Safety: Caps and Decay

| Mechanism | Value | Effect |
|----------|---------|-------|
| Lower head | 0.5x base | A weight does not fall below half of the slider |
| Upper head | 2.0x base | A weight does not increase more than twice compared to the slider |
| Normalization | sum = 1.0 | After each recalculation, the sum = 1.0 |
| Continuous decay | 0.997/trade | Half-life ~230 trades, prevents overfitting |

### 47.6 Interaction with the Oracle Fusion slider (§21)

The slider sets the **basic weights**. Adaptation changes them relatively:
- Kalman Slider W = 0.30 + adaptation -> effective range [0.15, 0.60]
- Changing the slider also changes the field of adaptation
- With adaptation OFF, the slider controls directly (identical pre-S15)

### 47.7 Persistence

The adaptation data (correlations, adapted weights, trade count) are saved in `titan_state_snapshot.json`. On restart, it is restored automatically. In-flight signals (trades open at restart) do not contribute to adaptation.

### 47.8 Relationship with other sections

- **§21 Oracle Fusion**: Weights in §21 become "base weights" when the adaptation is active
- **§19 Strategy Scoring**: Strategy Scoring uses Oracle with adapted weights (if active)
- **§44 Feature Store**: `oracle_score` and `oracle_confidence` in the Feature Store reflect the Oracle with adapted weights
- **§46 Entry Thresholds**: Thresholds remain fixed; only the Oracle weights change

---

## 48. EXIT INFLIGHT GUARD (S39-S40)

### 48.1 What is

Per-symbol deduplication guard on the exit path of the autopilot that prevents repeated sending of sell orders for the same position. Solve a critical "runaway exit loop" bug; — when the broker does not yet reflect the closing of the position, the autopilot sent duplicate SELL orders every tick (~2-3 seconds), each one either opening an accidental short position or generating 429 Too Many Requests errors.

Strengthened in S40 with strict guard (no TTL), checking broker open-orders, PendingClose lifecycle, and suppression of synthetic positions.

### 48.2 How it works

Camp on `TitanState`:

```rust
pub exit_inflight: Mutex<HashMap<String, u64>>
```

Two layers of protection:

| Layer | Moment | Action |
|-------|--------|---------|
| 1. Strict inflight | `evaluate_position_exit()` on every tick | If `exit_inflight.contains_key(symbol)` → return `None` (complete skip, no TTL) |
| 2. Broker open-order | Forward by `close_position()` | `has_pending_orders_for_symbol()` asks the broker (`GET /v2/orders?status=open`). If there is an open order → skip |
| SET | `close_position()` called | Set `exit_inflight[symbol] = timestamp_ms` before sending the order |

 The guard remains active until **the broker confirms that the position is flat** (through reconciliation) or a permanent error removes the entry. There is no timeout — the source of truth is the broker.

### 48.3 Where to clean

The guard is cleaned (remove from HashMap) in **2** locations:

1. **Position reconciliation** — every 15s, `exit_inflight.retain` keeps only symbols still present in the broker's **raw** positions (before grace/blacklist filtering). When the broker no longer reports the symbol, the entry disappears automatically.
2. **Autopilot permanent failure** — for errors 422, "invalid", "does not exist", "not found" — wipe to avoid permanent blockage.

**Do not clean in:**
- Autopilot exit success — the position is marked `PendingClose`, the guard remains active
- Trade updates fill complete — the local fill does not reset the guard (the REST broker may still report the position a few seconds after the fill)

### 48.4 Relationship with other sections

- **§15 Autopilot Core**: The Guard acts inside of the autopilot loop, on the exit path (between `fetch_positions` and `close_position`). It does not affect the parameters in §15 (min hold, consensus, loop ms, etc.)
- **§42 Concurrent Autopilot Architecture**: The Guard is checked in Phase 1 (`evaluate_position_exit`, read-only). Clearing is done in Phase 2 (sequential apply). It does not introduce additional lock contention — `exit_inflight` is a separate Mutex, independent of `settings` lock.
- **§16 Exit Scaling / §17 Oracle Exit**: These mechanisms decide IF to close. The inflight guard decides whether the closing decision should be ACTIVATED or temporarily suppressed. The exit logic remains unchanged.

### 48.5 PendingClose on Exit Success

On `close_position` success, the autopilot marks the `PendingClose` position with `closed_at` set, instead of deleting it from `settings.positions`. This prevents two problems:

1. **False synthetic position** — without PendingClose, when the fill reaches the WebSocket, `trade_updates` did not find the position and created a new Sell type (false short position)
2. **Re-insertion by recon** — reconciliation could re-insert the position as `Open` after the grace period, ignoring the fact that the autopilot closed

The state `PendingClose` is preserved by the merge in `position_recon` (line `new_pos.state = existing.state.clone()`). The position disappears naturally when the broker no longer reports it.

### 48.6 Synthetic Position Suppression

When a fill arrives on the WebSocket for a symbol that does NOT exist in `settings.positions`, `trade_updates` checks `exit_inflight`:

- If `exit_inflight.contains_key(symbol)` → the fill is **closing** (not opening), **no new position is created**
- If the symbol is NOT in `exit_inflight` → the fill is a legitimate **opening**, the normal position is created

This logic prevents the appearance of a Sell-type phantom position when the closing fill arrives after the autopilot marked the PendingClose position or after it was deleted by recon.

### 48.7 ORDER ACCEPTED — Correct Log

`send_order_internal` now issues `ORDER ACCEPTED` (not `ORDER FILLED`) on the HTTP 200 response. HTTP 200 means that the broker accepted the order, not that it was executed. Actual fills appear as `FILL: SELL LAR x212.0000 @ $X.XX [fill]` (emitted by `trade_updates` on WebSocket).

---

## 49. Safety Gates — VaR Fail-Closed, Asset Check, Intraday Blackout (S04.1)

Panel `⬡ SAFETY GATES` in the HFT Engine tab, under Portfolio Risk Gates.

### 49.1 Parameters

| Parameter | Element ID | Settings Backend | Default | rank |
|-----------|-----------|-----------------|---------|-------|
| VaR Fail-Closed | `sg-var-fail-closed` | `var_fail_closed` | MR | bool |
| Asset Check Hard Gate | `sg-asset-check-hard` | `asset_check_hard_gate` | MR | bool |
| Blackout First (min) | `sg-blackout-first` | `blackout_first_mins` | 5 | 0-60 |
| Blackout Last (min) | `sg-blackout-last` | `blackout_last_mins` | 5 | 0-60 |
| Blackout Exempt Symbols | `sg-blackout-exempt` | `blackout_exempt_symbols` | empty | comma-sep |
| Min Dollar Volume | `sg-min-dvol` | `scan_min_dollar_volume` | $5M | 0-$100M |

### 49.2 VaR Fail-Closed

When activated (default), if there is not enough return data (no symbol with >= 2 periods), `var_gate_blocks_entry()` returns `true` (blocks entry). It prevents entries in conditions of risk uncertainty. When disabled, lack of data allows entry (legacy fail-open behavior).

### 49.3 Asset Check Hard Gate

When activated (default), if the HTTP request to `/v2/assets/{symbol}` fails (timeout, network error), the order is **rejected** with an error. When disabled, the HTTP failure generates only a warning and the command continues (soft fail).

### 49.4 Intraday Blackout

It prevents new entries in the first `blackout_first_mins` minutes (after 9:30 ET) and the last `blackout_last_mins` minutes (before 4:00 PM ET) of the session. It does not affect the exits. DST-aware via `is_us_dst()`.

**Blackout Exempt Symbols:** Comma-separated list of symbols (ex: `SPY, QQQ`) that bypass the blackout. Case-insensitive.

### 49.5 Blackout Status Badge

Neon-orange badge with animation `glow-breathe` displayed in 2 locations:
- HFT Engine header (next to DRY/LIVE badge)
- STATUS sidebar

Displays the countdown of the remaining minutes. It hides automatically when the blackout is not active. Refresh: 15 seconds, calculated client-side.

### 49.6 VaR Gate Event + Toast

When VaR gate blocks an entry, it emits event Tauri `var-gate-blocked` with:
```json
{ "symbol": "AAPL", "reason": "portfolio VaR would exceed threshold", "max_var_pct": 2.0, "fail_closed": true }
```
Frontend: `TitanToast.show()` warning type with symbol and motif (4s display).

### 49.7 Settings Validation

`validate_safety_settings()` check:
- `blackout_first_mins + blackout_last_mins >= 195` → warning (>= half of the trading day)
- `min_dollar_volume < $100K` → warning (illiquid tickers risk)
- `enable_var_gate=true` + `var_fail_closed=false` → warning (risk gap: entries without data)

Warnings are logged and included in the `batch_update_settings` response.

### 49.8 Gate Chain Complete (Updated)

```
Entry Signal (Oracle/GPU/Signal)
  → Score Modifiers (N4)
    → TA Gates (Soft/Hard penalties)
      → BLACKOUT Gate (first/last mins, exempt symbols) ← S04.1
        → OFI Filter (flow direction)
          → Nexus Consensus (multi-agent)
            → HHI Gate (concentrare) ← S9
              → VaR Gate (risc statistic, fail-closed) ← S9+S04.1
                → Position Sizing
                  → Asset Check (hard gate) ← S04.1
                    → send_order_internal()
```

---

## 50. State Health Dots — Sub-State Decomposition Monitor (S04.5)

In the STATUS sidebar, 4 cyber-HUD mini-dots show the health of the sub-states in real time:

| Assoc | Sub-states | Monitored fields |
|-----|-----------|---------------------|
| **TRD** | TradingState | 10 Mutex (trail, entry metadata, exit, blacklist) |
| **MKT** | MarketDataState | 5 Mutex (prices, candles, indicators, vol, bars) |
| **RSK** | RiskState | 3 Mutex (analytics, drawdown, flash crash) |
| **SCN** | ScanState | 5 Mutex (best-TF, results, progress, prefilter, cancel) |

**Colors:**
- Green = all mutexes free (normal)
- Yellow = 1 Mutex contended (normal lock contention under load)
- Red = 2+ Mutexes contended (potential bottleneck)

**Hover:** Tooltip with details (locks free / contended, session generation).

**Backend:** Tauri command `get_state_health` -- try_lock sampling on all 23 Mutexes. Poll every 10 seconds.

---

## 51. SMARTTREND GENETIC OPTIMIZER (S13)

SmartTrend has a **dedicated genetic optimizer** (`StGeneticOptimizer`), completely independent of the classic Fusion optimizer. When `StrategyMode::SmartTrend` is active, `genetic_evolution_loop` automatically switches to SmartTrend evolution.

### Evolving Parameters (12)

| Parameter | Min | MAX | What controls |
|-----------|-----|-----|---------------|
| macro_weight | 0.05 | 0.60 | MACRO pillar share |
| micro_weight | 0.05 | 0.60 | MICRO pillar share |
| momentum_weight | 0.05 | 0.60 | The share of the MOMENTUM pillar |
| entry_threshold | 0.50 | 0.95 | Minimum score for ENTRY |
| stalking_threshold | 0.30 | 0.80 | Minimum score for STALKING |
| changepoint_reset | 1.0 | 10.0 | % volatility for FSM reset |
| trail_tighten_pct | 0.10 | 0.80 | How tight the trail stop becomes |
| hysteresis_margin | 0.01 | 0.20 | Dead zones between FSM states |
| bayesian_halflife | 3 | 30 | Bayesian half-life tracker |
| stabilization_window | 5 | 50 | Stabilization window after regime shift |
| dominant_cycle_filter | 5 | 50 | Dominant cycle filter |
| risk_per_trade | 0.002 | 0.030 | Risk per trade (fraction) |

The weights (macro/micro/momentum) are auto-normalized to sum = 1.0 after mutation/crossover.

### Walk-Forward Backtest SmartTrend

Run the real FSM (Flat → Stalking → Entry → InTrade → Trail) on historical data with IS/OOS split 70/30. Calculate PnL, Sharpe, Profit Factor, Max Drawdown, Win Rate, Trade Count, Avg Bars In Trade.

### Fitness Formula

```
fitness = sharpe * min(PF, 5.0) + trade_count_bonus + win_rate_bonus + stability_bonus - dd_penalty
```

- **trade_count_bonus**: +0.5 when > 5 trades
- **win_rate_bonus**: +(win_rate - 0.55) * 2 when > 55%
- **stability_bonus**: +0.3 when avg_bars_in_trade between 3-30
- **dd_penalty**: max_dd * 5 when > 20%
- **OOS validation**: champion MUST have sharpe > 0 and total_pnl > 0 on 30% out-of-sample

### UI Components

| Elements | Location | What does it look like? |
|---------|---------|----------|
| Convergence Chart | EVOLUTION panel canvas | IS fitness (cyan) + OOS fitness (gold dashed) by generations, green dots at champion promotion, diversity bar |
| PARAMS button | EVOLUTION panel header | Modal drill-down with all 12 params, delta% vs defaults, heat-color bars |
| DIV badge | EVOLUTION panel header | Diversity % population. Red+pulses < 15%, cyan 15-40%, verde > 40% |
| Comparison Panel | tab-13 SmartTrend CFG | RUN COMPARISON: run Fusion + SmartTrend on the same candle, grid with winner highlight |

### Taurus orders

| Command | returns |
|---------|-----------|
| `get_st_evolution_stats` | Gender, IS/OOS fitness, sharpe, diversity, champion params |
| `get_st_fitness_history` | Vec<{gen, is_fitness, oos_fitness, diversity, promoted}> |
| `get_st_champion_params` | Params with value, default, delta%, min, max |
| `run_comparison_backtest_cmd` | ComparisonResult: {fusion: BacktestResult, smarttrend: StBacktestResult, verdict} |
| `freeze_st_evolution` | It stops the evolution of SmartTrend |
| `resume_st_evolution` | SmartTrend evolution starts |

---

---

## 52. EXECUTION RESILIENCE (S14, Session 63)

### Order Lifecycle

- **client_order_id**: UUID v4 generated locally per order
- **broker_id**: Parsed from the Alpaca response (`id` field)
- **Retry**: 1-2x automatic for transient errors (5xx, timeout)
- **Event**: `order-lifecycle` issued on submit with client_order_id, broker_id, status, retry_count

### Pending Order Watchdog

- Tracked locally in `TradingState::pending_orders` (Vec<PendingOrder>)
- Auto-cancel after `pending_order_timeout_secs` (default 120s) or `generation_id` mismatch
- Removal from tracking at fill/partial_fill/rejected/cancelled

### Health Observability

| Eventos | Trigger | Badge UI |
|-------|---------|----------|
| `equity-sync-degraded` | Consecutive equity sync errors | EQUITY STALE (gold+pulse) |
| `equity-sync-critical` | Critical errors equity sync | EQUITY STALE (gold+pulse) |
| `trade-updates-health` | WS connect/disconnect | WS DISCONN (red+pulses) |

### Cockpit Strip

Persistent bar under the main header with: EQUITY, P&L, POS count, MODE (auto/manual), BREAKER status, GEN (generation id), health badges. Poll 5s + event-driven badges. New badges (S71): TMR:N (active timers), MRK:S/F/T (chart markers per source), HP:N% (real health with latency + fill rate).

### Bounded Channels

`mpsc::channel(256)` replaces `unbounded_channel` on the streamer. `try_send` with overflow warning.

### HHI Notional Fixed

`qty * live_price` instead of `qty * entry_price`. Structured log: `hhi_pre`, `hhi_post`, `blocked`.

### Exit Features

`exit_features: Option<TradeFeatures>` on TradeRecord — snapshot market conditions at close trade.

### UI Bonuses

| Features | Description |
|---------|-----------|
| Live Fill Markers | BUY/SELL triangles on the chart, persistent, 3s refresh |
| Journal Min P&L | Input filter in journal bar |
| Time-in-Trade | TIME column by positions (dim/cyan/gold/orange) |

### §53: S16 UX Polish — HFT-Relevant Changes

**Fill Quality Columns:** 3 new columns in HFT Targets table — SLIP (avg slippage bps, color-coded), T2F (avg time-to-fill), TRADES (count per symbol). Backend `get_fill_quality_stats`. Polling 30s.

**Unified Chart Markers:** `_titanMarkerEngine` goes fills+signals+SmartTrend, debounce 100ms, cap 50 fills, pruning 24h. Public API: `update(source, markers)`, `clear(source)`, `stats()`, `getSource(name)`. Overlay on chart: `SIG:N FILL:N ST:N TOT:N`.

**Latency Histogram Rolling:** Ring buffer 300 samples (5 min window) instead of cumulative.

**Health Score Badge:** `HP:XX%` in the cockpit — 6 composite sub-scores. Polling 10s.

**Cockpit Strip V2:** GEN sync real, BREAKER initial poll, W/R badge, UPTIME badge.

*Manual created: March 28, 2026 — Session 19. Updated: April 3, 2026 — Session 71 (cockpit strip badges TMR/MRK/HP, unified marker engine API, chart marker overlay). Full coverage: ~155+ configurable parameters, 28 smart features, 53 sections.*

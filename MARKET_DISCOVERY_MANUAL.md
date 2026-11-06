# 🔍 MARKET DISCOVERY — Complete User Manual

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


**Version:** V8 FMP-Enriched (Discovery V8 Engine)
**Last Updated:** April 2, 2026 (Session 64 — S15: Tiered Scanner Top-K)
**UI Location:** Left Sidebar → Section 8 (collapsible)
**Data sources:** Alpaca Markets (prices, volumes) + Financial Modeling Prep (float, market cap, insider trading, earnings, price targets)

---

## CONTENTS

1. [What is Market Discovery?](#1-ce-este-market-discovery)
2. [Where do I find Market Discovery in the interface?](#2-unde-gasesc-market-discovery-in-interfata)
3. [Filter Parameters (Input Fields)](#3-parametrii-de-filtrare)
4. [The buttons Action](#4-butoanele-de-actiune)
5. [Presets (Quick Presets)](#5-presetari-quick-presets)
6. [Score Mode — Scoring Modes](#6-score-mode)
7. [Scan Results](#7-rezultatele-scanarii)
8. [Column Δ (Delta) — DIFF ALERT](#8-coloana-delta-diff-alert)
9. [NEW MOVER — New Symbol Alerts](#9-new-mover-alerte)
10. [Auto Scan](#10-auto-scan)
11. [Ross Cameron Preset](#11-ross-cameron-preset)
12. [Inject into Watchlist / Signal CFG](#12-inject-in-watchlist)
13. [How the Backend Scanner Works](#13-cum-functioneaza-backend)
14. [FMP Data Integration — FREE Basic Data](#14-fmp-data-integration)
15. [Practical Usage Scenarios](#15-scenarii-practice)
16. [Expert Tips for Beginners](#16-sfaturi-expert)
17. [Frequent Problems and Solutions](#17-probleme-frecvente)
18. [Glossary of Terms](#18-glosar)

---

## 1. WHAT IS MARKET DISCOVERY?

Market Discovery is **market scanner** integrated into Titan HFT. It scans **all stocks listed on US exchanges** (NYSE, NASDAQ, AMEX, ARCA) and filters them according to your criteria, returning only the ones that move the most and have the highest trading potential.

### What does it actually do?

Imagine you have **10,000+ stocks** listed on the US market. You can't track them all manually. Market Discovery does this for you:

1. **Download** the list of all active stocks from Alpaca Markets
2. **Filter** only those on US exchanges (NYSE, NASDAQ, AMEX, ARCA)
3. **Get** current price snapshot (price, volume, change %) for all
4. **Apply** your filters (price, volume, change %, RVOL, gap)
5. **Score** each candidate on a scale of 0-100
6. **Sort** the results by score (best first)
7. **Display** the list in the sidebar

### What does it help?

- **Find the hottest "hottest" stocks of the day** — the ones that move the most
- **Identify trading opportunities** that you would not have seen otherwise
- **Save time** — instead of 2 hours of manual research, the scanner does everything in 30 seconds

---

## 2. WHERE DO I FIND MARKET DISCOVERY IN THE INTERFACE?

Market Discovery is located in the **left sidebar** of the app as a collapsible section with the header:

```
MARKET DISCOVERY ▸
```

- **Click** on the header to open/close the section
- The header is **cyan neon** with glow, cyber-HUD style
- When the section is open, you see **▾** (rotated 90°)
- When there is a new scan with new symbols, a **red badge** (ex: 🔴3) appears on header

---

## 3. FILTER PARAMETERS

These are the fields where you set **WHAT YOU ARE LOOKING FOR**. Each parameter removes actions that do not match your criteria.

### 3.1 Min Price ($) — `md-price-min`

**What it does:** Sets the lower price limit. Any stock priced UNDER this value is removed.

| Field | Default Value | Range |
|------|----------------|----------|
| Min Price | $1 | $0.01 - $99,999 |

**Examples:**
- `Preț Min = 1` → Eliminate penny stocks below $1 (protection)
- `Preț Min = 10` → Only look for stocks above $10
- `Preț Min = 0.10` → Include micro-caps (very high risk!)

**Tip:** Beginners should use at least $1 — stocks under $1 are extremely volatile and manipulated.

---

### 3.2 Max Price ($) — `md-price-max`

**What it does:** Sets the upper price limit. Any stock priced ABOVE this value is removed.

| Field | Default value | Interval |
|------|----------------|----------|
| Price Max | $500 | $1 - $99,999 |

**Examples:**
- `Preț Max = 20` → Only cheap stocks (small-caps), ideal for aggressive day trading
- `Preț Max = 100` → Excludes super-expensive stocks (AMZN, etc.)
- `Preț Max = 500` → Includes almost everything but BRK.A ($600K+)

**Tip:** For day trading, the $1-$50 range is the most used. Expensive stocks ($200+) have large spreads and require large capital.

---

### 3.3 Min Volume (M) — `md-vol-min`

**What it does:** Sets the minimum daily volume in **millions**. Stocks with volume below this value are removed.

| Field | Default value | Interval |
|------|----------------|----------|
| Volume Min | 0.5M (500,000) | 0.1M - 100M |

**What does volume mean?** The total number of shares traded that day. High volume = many traders = good liquidity.

**Examples:**
- `0.5M` (500K) → Standard, removes stocks with low liquidity
- `1M` → Only stocks with medium-high liquidity
- `5M` → Only very liquid stocks (large-cap, popular ETFs)
- `0.1M` (100K) → Also includes micro-caps (risk: large spreads, slippage)

**Why does volume matter?**
- Low volume = hard to buy/sell quickly → **slippage** (sell more than you want)
- High volume = you can enter/exit instantly without impact on the price
- Golden Rule: **never trade a stock with volume below 200K**

---

### 3.4 Min % Change — `md-change-min` (Slider)

**What it does:** Sets the minimum percentage change in price from the previous day's close. Stocks that have moved less are eliminated.

| Field | Default value | Interval | Control |
|------|----------------|----------|---------|
| Min % Change | 1% | 0% - 50% | Slider (step 0.5%) |

**What does the change look like?** If a stock closed at $10 yesterday and is now at $11, the change is +10%.

**Examples:**
- `1%` → Includes almost anything that moves (conservative)
- `5%` → Only actions with significant movement
- `10%` → Only "hot" — gap-ups, news, earnings
- `20%` → Extremely aggressive — only explosive moves (penny stocks, squeezes)

**Special Behavior:**
- If **Short Selling is disabled** in settings: scanner ONLY looks for positive changes (+5% yes, -5% no)
- If **Short Selling is enabled**: the scanner looks for changes in both directions (|change| >= min), so even -15% passes the 10% filter

**Tip:** The value read from the slider is displayed in yellow (#FFD700) to the right of the slider.

---

### 3.5 Minimum RVOL (x) — `md-rvol-min`

**What it does:** Sets the volume to relative minimum. **RVOL** (Relative Volume) compares today's volume with the previous day's volume.

| Field | Default value | Interval |
|------|----------------|----------|
| RVOL Min | 1x | 0 - 50x |

**Formula:** `RVOL = Volumul de azi / Volumul de ieri`

**What does this mean?**

| RVOL | Interpretation |
|------|-------------|
| 0.5x | Half the volume of yesterday — quiet day |
| 1.0x | Normal volume — like yesterday |
| 2.0x | Double from yesterday — interesting |
| 5.0x | 5 times more than yesterday — **something is happening!** |
| 10x+ | Explosive — probably big news, FDA, earnings |

**Practical examples:**
- `RVOL = 1` → Accepts anything (no real filter)
- `RVOL = 2` → Only stocks with above-normal activity
- `RVOL = 5` → Only stocks with explosive activity (Ross Cameron recommended)
- `RVOL = 10` → Ultra-selective — a few stocks per day

**Why is RVOL important?**

RVOL is **the most important indicator for a day trader**. A high RVOL action means:
- Institutional traders are active
- Important news or catalyst
- Price movement has "fuel" real (not just noise)
- Lower spreads (better liquidity)

**Golden rule:** NEVER trade a stock with RVOL below 2. Those are "dead" — no energy, no opportunity.

---

### 3.6 Gap Min (%) — `md-gap-min`

**What it does:** Sets the minimum gap percentage. The **gap** is the difference between today's opening price and yesterday's closing price.

| Field | Default Value | Range |
|------|----------------|----------|
| Gap Min | 0% | 0% - 50% |

**Formula:** `Gap% = (Open azi - Close ieri) / Close ieri × 100`

**What is a Gap?**

A gap occurs when a stock **opens** at a significantly different price than it **closed** yesterday:

```
Ieri: Acțiunea X a închis la $10.00
Azi:  Acțiunea X a deschis la $12.00 → Gap-up de +20%
```

**Types of gaps:**
- **Gap-up** (+) → Price jumped UP overnight (good news, good earnings)
- **Gap-down** (-) → Price fell DOWN overnight (bad news, poor earnings)

**Examples:**
- `Gap = 0%` → No filter (includes everything, it's the default)
- `Gap = 2%` → Only stocks that opened with at least 2% difference
- `Gap = 5%` → Significant gap — usually news or earnings
- `Gap = 10%` → Major gap — stocks that will be in the center of attention all day

**Special behavior:** As with Change, if Short Selling is disabled, only positive gaps pass the filter.

---

### 3.7 Float Max (M) — `md-float-max` 🆕

**What it does:** Sets the maximum number of shares in free circulation (float) in **millions**. Stocks with higher float are removed.

| Field | Default Value | Range | Data Source |
|------|----------------|----------|------------|
| Float Max | 0 (disabled) | 0 - 10,000M | FMP API |

**What is the Float?** The float is the number of shares publicly available for trading. **NOT** includes shares held by insiders, locked-up funds or restricted shares.

**Why does Float matter?**

| Float | Classification | Effect on Price |
|-------|------------|---------------|
| < 5M | **Ultra-low float** | EXTREME movements (±50-100%/day). Maximum risk. |
| 5-20M | **Low float** | Large movements (±10-30%/day). Ross Cameron target. |
| 20-100M | **Medium** | Moderate movements. Liquidity OK. |
| > 100M | **Large** | Small movements. Excellent liquidity. |

**Examples:**
- `Float Max = 0` → Filter is disabled (include everything)
- `Float Max = 20` → **Ross Cameron style** — only low float stocks
- `Float Max = 50` → Also includes medium float
- `Float Max = 100` → Excludes only massive large-caps

**How it works technically:**
- Data comes from **Financial Modeling Prep (FMP)** API
- Cache 24h (float rarely changes — only on dilution/offerings)
- If FMP has no data for a symbol, it is **NOT removed** (passes the filter)
- Requires FMP API Key (see section API & WALLET)

**Colors in table:**
- 🟣 **Purple** — Float ≤ 20M (low float, Ross Cameron target)
- ⚪ **Grey** — Float > 20M

**Tip:** Small float + high RVOL = **explosive cocktail**. When many buyers are fighting over the few available shares, the price EXPLODES. But it can fall just as quickly!

---

### 3.8 Market Cap Max (B) — `md-cap-max` 🆕

**What it does:** Sets the maximum market cap in **billions**. Larger companies are eliminated.

| Field | Default value | Interval | Data source |
|------|----------------|----------|------------|
| Head Max | 0 (disabled) | 0 - 10,000B | FMP API |

**What is Market Cap?** `Market Cap = Preț × Total Acțiuni`. It measures the "size" the company.

| Market Cap | Classification | Color in table |
|-----------|------------|------------------|
| < 300M | **Micro Cap** | 🟣 Purple |
| 300M - 2B | **Small Cap** | 🟡 Yellow |
| 2B - 10B | **Mid Cap** | ⚪ Gray |
| > 10B | **Large Cap** | ⚫ Dark gray |

**Examples:**
- `Cap Max = 0` → Filter disabled
- `Cap Max = 2` → **Ross Cameron style** — only micro + small cap
- `Cap Max = 10` → Exclude large-caps (AAPL, MSFT, GOOGL)

**Why does it matter?** Small companies move more percentage wise. AAPL needs to grow by $300B in cap for +10%. A $50M company only needs $5M — that's why small-caps have explosive moves.

**Tooltip:** Hover over the CAP column shows the **sector** of the company (Technology, Healthcare, Energy, etc.)

---

### 3.9 REQUIRE CATALYST — Toggle (Checkbox)

**What it does:** When enabled, the scanner will flag stocks that have a catalyst (news, earnings).

| Field | Default value | Control |
|------|----------------|---------|
| Requires Catalyst | OFF | Checkbox toggle |

**What is a catalyst?**

A catalyst is a **news or event** that causes price movement:
- 📰 Earnings report (quarterly financial report)
- 💊 FDA approval (for pharma companies)
- 🤝 Partnership or acquisition announced
- 📊 Upgrade/downgrade from an analyst
- 🏛️ Legislation affecting the sector
- 🟢 **Insider buying** (CEO/CFO buys shares — now automatically detected via FMP!)

**Why does it matter?** An action that moves WITHOUT a catalyst is often just "noise" or manipulation. A catalyst action has a fundamental reason for movement—and movement tends to continue.

---

### 3.10 EXCLUDE EARNINGS ±3d — Toggle (Checkbox) 🆕

**What it does:** When enabled, removes stocks that have an earnings report in the next ±3 days.

| Field | Default value | Control | Data source |
|------|----------------|---------|------------|
| Excludes Earnings | OFF | Checkbox toggle | FMP API |

**Why would you exclude earnings?**
- Stocks move **±20-50%** to earnings — unpredictable!
- If you are a conservative day trader, earnings = **the risk is not worth it**
- Spreads increase dramatically ahead of earnings
- Implied volatility is inflated (expensive options)

**When to activate it:**
- ✅ If you are a beginner and don't want extreme risks
- ✅ If you trade with capital that you cannot lose

**When to turn it off:**
- ❌ If you are explicitly looking for earnings plays (gap-and-go post-earnings strategies)
- ❌ If you want to see all the opportunities

**How ​​it works:** FMP returns the calendar with ONE API call (super efficient). Cache 12h.

---

### 3.8 Max Candidates — Slider

**What it does:** Limits the maximum number of results returned by the scanner.

| Field | Default value | Interval | Control |
|------|----------------|----------|---------|
| Max Candidates | 100 | 10 - 300 | Slider (step 10) |

**Examples:**
- `50` → Top 50 best opportunities (fast, focused)
- `100` → Standard — good balance between visibility and speed
- `200` → Wide view — includes weaker candidates
- `300` → Maximum — everything that passes the filters

**Tip:** More candidates = slower scan (more snapshots to retrieve). For fast scans, use 50.

---

### 3.9 Auto-Scan Interval (min) — Slider

**What it does:** Sets the interval in minutes for auto-scanning (when Auto Scan is enabled).

| Field | Default value | Interval | Control |
|------|----------------|----------|---------|
| Auto-Scan Interval | 30 min | 5 - 60 min | Slider (step 5) |

**Examples:**
- `5 min` → Very frequent scanning (ideal for active day trading, but consumes more API)
- `15 min` → Good balance
- `30 min` → Standard — recommended
- `60 min` → Infrequent scanning — for swing trading or passive monitoring

---

## 4. ACTION BUTTONS

### 4.1 ⚡ SCAN (Cyan Blue, Large)

**What it does:** Starts a manual market scan NOW using all parameters set by you.

**How it works:**
1. Click the button → The text changes to "⚡ SCANNING..."
2. The button deactivates (prevents double-click)
3. The backend downloads all active actions (~10,000)
4. Filters them according to your parameters
5. Scores them with the scoring formula selected
6. Returns sorted results
7. Button returns to "⚡ SCAN"

**Duration:** 20-60 seconds (depends on internet connection and number of actions)

**When to scan:** The best time is between **9:30 - 10:30 AM New York** (first 60 minutes of trading). Here are the most opportunities.

---

### 4.2 ✕ CLEAR (Red)

**What it does:** Clears all results displayed in the results panel.

**Does not affect:** Filter parameters, presets or settings. Just clear the results list.

---

### 4.3 AUTO SCAN: OFF / ON (Cyan)

**What it does:** Enables/disables periodic auto scan.

**When ON:**
- The text changes to "AUTO SCAN: ON" with green color
- The scanner runs automatically at the set interval (default 30 min)
- Scanning is done only **Monday-Friday, between 9:00 - 17:00 New York time**
- DO NOT scan on weekends (saves API calls on Friday stale dates)

**When it is OFF:**
- The text shows "AUTO SCAN: OFF"
- You have to manually press "⚡ SCAN"

---

### 4.4 ⚔️ ROSS CAMERON (Yellow/Gold)

**What it does:** Applies professional preset Ross Cameron (founder of Warrior Trading). See [section 11](#11-ross-cameron-preset) for full details.

---

### 4.5 INJECT → WATCHLIST (Cyan)

**What it does:** Adds ALL actions from scan results to watchlist/favorites.

**Example:** If the scan returned 15 actions, all 15 are added to your watchlist with one click.

---

### 4.6 AUTO INJECT

**What it does:** Enable/disable automatic injection of top results into the watchlist after each scan.

---

## 5. PRESETS (QUICK PRESETS)

Presets are **predefined configurations** that automatically set all filter parameters with a single click. They are displayed in a 2×2 grid, each with a distinct color.

### 5.1 🚀 PENNY ROCKETS (Orange) — `btn-md-penny-rockets`

**Strategy:** Very cheap stocks with huge percentage moves.

| Parameter | Value | Why? |
|-----------|---------|--------|
| Min Price | $0.10 | Includes penny stocks |
| Max Price | $5.00 | Only ultra-cheap stocks |
| Min Volume | 2M | High volume (penny liquidity required) |
| RVOL Min | 5x | Explosive activity |
| Gap Min | 0% | No gap filtering |
| Change Min | 20% | Massive moves only |
| Max Candidates | 50 | Focused on the top |
| Catalyst | OFF | Does not require |
| Score Mode | MOMENTUM | Prioritize price change |

**When to use:** When searching stocks under $5 that jump 20-100% in a day. Risk profile: **VERY HIGH**. Only with money you can afford to lose.

**Real Example:** A stock at $0.50 that jumps to $1.50 in a day = +200%. But it can just as easily fall to $0.20 (-60%).

---

### 5.2 📈 SWING SETUP (Mov/Purple) — `btn-md-swing-setup`

**Strategy:** Mid-cap stocks with significant gaps, ideal for swing trading (1-5 day holdings).

| Parameter | Value | Why? |
|-----------|---------|--------|
| Min Price | $10 | Exclude penny stocks |
| Max Price | $100 | Mid-cap range |
| Min Volume | 1M | Good liquidity |
| RVOL Min | 3x | Activity Above Normal |
| Gap Min | 2% | Significant Gap |
| Change Min | 3% | Moderate Movement |
| Max Candidates | 100 | Wide view |
| Catalyst | OFF | Not required |
| Score Mode | GAP SCANNER | Prioritize the gap |

**When to use:** Evening (after hours) or early morning, to identify stocks that opened with a gap and could continue the movement.

**Example:** A stock at $30 opens at $33 (gap +10%) on good earnings. You enter in the first hour, set your stop-loss at $31.50, and target $36 in 2-3 days.

---

### 5.3 🏛️ LARGE CAP (Blue) — `btn-md-large-cap`

**Strategy:** Large, liquid, stable stocks — ideal for conservative traders.

| Parameter | Value | Why? |
|-----------|---------|--------|
| Min Price | $50 | Large Companies Only |
| Max Price | $500 | Includes AAPL, GOOGL, MSFT |
| Min Volume | 5M | Ultra Liquid |
| RVOL Min | 1x | Normal volume is OK |
| Gap Min | 0% | Do not filter gap |
| Change Min | 3% | Significant movement for large-cap |
| Max Candidates | 50 | Focused |
| Catalyst | OFF | Does not require |
| Score Mode | VOLUME HUNTER | Prioritize volume |

**When to you use:** Anytime. Large-caps move less percentage-wise, but have small spreads and minimal slippage. Perfect for beginners.

**Example:** AAPL drops 4% on supply chain news → enter long with tight stop, waiting for rebound.

---

### 5.4 🔥 HOT MOMENTUM (Red) — `btn-md-hot-momentum`

**Strategy:** The hottest "hot" market share — combination of high change and high RVOL.

| Parameter | Value | Why? |
|-----------|---------|--------|
| Min Price | $1 | Includes small-caps |
| Max Price | $50 | Excludes expensive stocks |
| Min Volume | 1M | Decent liquidity |
| RVOL Min | 3x | Minimum 3x normal volume |
| Gap Min | 0% | Include all |
| Change Min | 10% | Strong moves only |
| Max Candidates | 100 | Wide view |
| Catalyst | OFF | Does not require |
| Score Mode | MOMENTUM | Prioritizes price change |

**When to use:** During the first 2 hours of market open (9:30-11:30 ET). This is the "momentum" — actions that move tend to continue.

---

### 5.5 What happens when you press a preset?

1. **All parameters** are instantly updated in the UI (the colors of the sliders change)
2. **The values are saved** in the backend (persistent between sessions)
3. **Score Mode** automatically changes to the mode according to the preset
4. **The button** lights up cyan briefly (0.8 seconds) as confirmation
5. **Status message:** "Penny Rockets preset applied" (at the bottom of the section)

**⚠️ Important:** Clicking on a preset does NOT start autoscan. You have to press "⚡ SCAN" separated.

---

## 6. SCORE MODE — SCORING MODES

Score Mode controls the **scoring formula** — that is, how the score (0-100) is calculated for each candidate action. Different modes emphasize different metrics.

### 6.1 Components of the score

The total score has **4 components**, each with a weight. Sum of weights = 100:

| Component | What it measures | Good value example |
|-----------|-----------|---------------------|
| **CHG** (Change) | How much the percentage price moved | +15% |
| **RVOL** (Relative Volume) | How active is the face of normal | 8x |
| **GAP** (Gap) | How much it opened different from yesterday | +5% |
| **VOL** (Absolute Volume) | Gross volume in shares | 12M |

### 6.2 The 4 ways available

#### ⚖️ BALANCED — 25 / 25 / 25 / 25

**All components are of equal importance.** The most neutral mode. It doesn't favor anything.

**When:** When you don't know what you're looking for. Good for general exploration.

**Example scoring:**
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

**Price change matters 2 TIMES more.** Ideal for aggressive day trading.

**When:** Look for stocks with the biggest percentage moves.

**Example:** A stock with +25% change but modest volume will be scored higher than one with +5% but huge volume.

#### 📊 VOLUME HUNTER — 15 / 20 / 10 / 55

**Absolute volume counts DOUBLE.** Ideal for traders who prioritize liquidity.

**When:** Looking for stocks with the highest traded volume. Good if you trade with large amounts.

**Example:** Apple (AAPL) with 50M volume and +3% change will be scored higher than a penny with 500K volume and +30% change.

#### 📉 GAP SCANNER — 10 / 15 / 55 / 20

**The gap counts TWICE as much.** Ideal for gap trading.

**When:** Looking for stocks that opened with a significant gap. Strategy: buy the gap and wait for continuation or fill.

**Example:** A stock that opened +8% since yesterday's close will be scored very high, even if the current change is only +5% (it has withdrawn from the gap).

### 6.3 How do I change Score Mode?

1. Open the "**◈ SCORE MODE**" dropdown
2. Select the desired mode
3. It is automatically saved in the backend
4. The status message shows: "Score mode: MOMENTUM" (or whatever you chose)
5. **The effect is applied to the next scan** — does not change the results already displayed

### 6.4 Legend "CHG / RVOL / GAP / VOL"

A small gray legend appears below the dropdown:

```
CHG / RVOL / GAP / VOL
```

This shows you the order of the components in the displayed figures. For example, "50 / 20 / 10 / 20" means:
- CHG (Change) = 50 points maximum
- RVOL = 20 points maximum
- GAP = 10 points maximum
- VOL (Volume) = 20 points maximum

---

## 7. SCAN RESULTS

After pressing "⚡ SCAN", the results appear in a table with the following columns:

### 7.1 Table columns

| Column | What does it look like? | colors | Source |
|---------|---------|--------|-------|
| **SYMBOL** | Name of the action (ex: AAPL) + 📊 badge earnings | Click → change active symbol | Alpaca |
| **PRICES** | Current price | White/grey | Alpaca |
| **CHG%** | % change from yesterday | 🔵Cyan(≥10%), 🟡Yellow(≥0%), 🔴Red(<0%) | Alpaca |
| **RVOL** | Relative volume (ex: 5.2x) | 🔵Cyan(≥5x), ⚪Gray(<5x) | Alpaca |
| **THEFT** | Absolute volume (short format) | Gray | Alpaca |
| **$VOL** 🆕 | Dollar Volume (price × volumes) | 🟡Gold(≥$10M), ⚪Dim(<$10M) | Calcutta |
| **FLOAT** | Float shares in millions | 🟣Purple(≤20M), ⚪Grey(>20M) | FMP |
| **HEAD** 🆕 | Market cap (hover=sector) | 🟣Purple(<300M), 🟡Yellow(<2B), ⚪Grey(<10B), ⚫(<10B+) | FMP |
| **INS** 🆕 | Insider trading signal | 🟢(BUY), 🔴(SELL), —(none). Hover=details. | FMP |
| **SCORE** | Calculated score (0-100) | 🔵Cyan(≥60), 🟡Yellow(≥30), ⚪Grey(<30) | Calcutta |
| **Δ** | Delta from previous scan | See section 8 | Calcutta |
| ***+** | Add to watchlist button | 🟡Yellow → 🔵Cyan(✓) upon confirmation | — |

### 7.2 FMP Columns (only appear with FMP API Key)

The FLOAT, CAP and INS columns are **visible only if FMP API Key is configured** and data is available. Without the key, the scanner works normally but without this additional data.

**Earnings Badge (📊):**
If a share has an earnings report in the next ±3 days, a yellow badge 📊 appears next to the symbol name. Hover over the badge shows "Earnings ±3 days".

**Insider Signal (INS):**
- 🟢 = Recent Insider **BUY** (CEO, CFO, director bought shares)
- 🔴 = Recent Insider **SELL**
- — = No insider data
- **Hover** on emoji shows details: *"Tim Cook BUY 50000sh @$182.50"*

### 7.2 Volume Formatting

The volume is displayed in short format for readability:

| Actual Volume | Displayed As |
|------------|-----------|
| 500,000 | 500K |
| 1,200,000 | 1.2M |
| 15,700,000 | 15.7M |
| 250,000,000 | 250M |

### 7.3 Row Interactions

- **Click on symbol** → Set action as active symbol (chart changes, analysis refocuses)
- **Click on +** → Add the action to watchlist/favorites

---

## 8. COLUMN Δ (DELTA) — DIFF ALERT

### What is Delta?

Delta (Δ) is the **score difference** between the current scan and the previous scan for the same action.

### How does it work?

1. **First scan:** Column Δ does not appear (has no previous reference)
2. **Second scan (and after):** Every action shows the difference:
   - If XYZ had score 45 30 min ago and now has 72 → Δ = **+27**
   - If ABC had a score of 60 and now has a score of 50 → Δ = **-10**
3. **New actions** (which were not in the previous scan) → Showing "—" (dash)

### Delta colors

| Delta | Color | It means |
|-------|---------|----------|
| **> +5** | 🟢 Green (#00FF88) | Strong acceleration — CAUTION! |
| **> 0** | 🔵 Cyan (#00BFFF) | Moderate improvement |
| **= 0** | ⚪ Gray (#555) | Stable |
| **< 0** | 🟠 Orange (#FF8800) | Deceleration |
| **< -5** | 🔴 Red (#FF4444) | Significant decrease |

### Why is Delta important?

**Delta is the strongest intraday signal.** Why?

- An absolute score of 70 can be a "always hot" (ex: GME, AMC)
- But a delta of **+25** (score jumped from 45 to 70 in 30 min) means: **SOMETHING HAS CHANGED NOW**
- This indicates **acceleration** — the stock is gaining momentum RIGHT NOW
- This is exactly when professional traders come in

### Practical example

```
Scanare 1 (10:00 AM):
  MARA  $22.50  +5%   2.1x  3.2M  +2%   42     —
  RIOT  $15.30  +3%   1.8x  2.5M  +1%   35     —

Scanare 2 (10:30 AM):
  MARA  $24.10  +12%  4.5x  8.1M  +2%   67    +25 🟢  ← ACCELERARE!
  RIOT  $15.80  +6%   2.3x  3.1M  +1%   41    +6  🟢
  NVDA  $850    +4%   3.2x  25M   +1%   55     — (NEW)
```

MARA had a delta of +25 → Jumped from score 42 to 67 in 30 minutes. Something is happening — possibly Bitcoin news, sector move, short squeeze.

---

## 9. NEW MOVER — ALERT FOR NEW SYMBOLS

### What is a New Mover?

A "New Mover" is an action that **appears in results FOR THE FIRST TIME** — it was not in the previous scan but now passes all filters.

### How are they displayed?

#### 9.1 Badge "NEW" in turn

New symbols have a pulsating green badge **NEW** before the name:

```
🟢NEW NVDA  $850.00  +4%  3.2x  25M  +1%  55
```

- Badge has **pulse** animation (pulsating green glow)
- The row has a **green left border** and **green subtle background**

#### 9.2 Toast Notification

When new symbols appear, a **toast** (notification) slide-in appears in the **upper right-hand corner** of the screen:

```
✔ 3 NEW MOVERS: NVDA, MARA, RIVN
```

- Use the TitanToasts unified system (`showCyberToast`, type `success`)
- Glassmorphism with green gradient and neon border
- Auto-closes after **5 seconds**, click X to dismiss manually
- Display maximum 4 symbols + "+X" for the rest

#### 9.3 Badge on Header

The "MARKET DISCOVERY" header receive a **red badge** with the number of new movers:

```
MARKET DISCOVERY 🔴3
```

- The badge has a "pop" at emergence (scales 0 → 1.3 → 1)
- Glow red (box-shadow)
- Resets on next scan (or becomes 0)

### Why do New Movers matter?

**A New Mover is the most dangerous/opportune intraday event:**

- An action that was NOT in the scanner 30 min ago but NOW passes all filters means:
  - It just had a volume explosion
  - A price/change threshold has been passed that has "activated"
  - Probably a catalyst started the NOW movement
- These are the candidates with the greatest potential **SHORT TERM** (15-60 min)

---

## 10. AUTO SCAN — AUTO SCAN

### How do I enable Auto Scan?

1. Click on the **"AUTO SCAN: OFF"** button
2. It changes to **"AUTO SCAN: ON"** with green color
3. The scanner runs automatically every `Auto-Scan Interval` minutes

### Operating rules

- **Monday-Friday:** Only scan between 9:00am - 5:00pm New York time
- **Weekend (Saturday/Sunday):** DO NOT scan (stock exchanges are closed)
- **Outside hours:** DOES NOT scan (data does not change)
- **Parameters used:** The same parameters you see in the UI

### Auto-Scan flow

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

### Who is Ross Cameron?

Ross Cameron is the founder of **Warrior Trading** and one of the most respected day traders in the world. His strategy focuses on cheap actions with explosive moves. His 5 criteria are: **Low Price, Relatively High Volume, Catalyst, Low Float, High Change.**

### Ross Cameron Specs (V8 — Complete with FMP)

| # | Parameter | Value | Ross Cameron Criterion | Source |
|---|-----------|---------|----------------------|-------|
| 1 | Price Min | $1 | ✅ Penny stocks included | Alpaca |
| 2 | Price Max | $20 | ✅ Small-cap focus | Alpaca |
| 3 | Volume Min | 1M | ✅ Minimum liquidity | Alpaca |
| 4 | RVOL Min | 5x | ✅ **"Stocks that are already UP"** | Alpaca |
| 5 | Gap Min | 0% | Does not filter gap (Change is enough) | Alpaca |
| 6 | Change Min | 10% | ✅ **"Stocks already up 10%"** | Alpaca |
| 7 | **Float Max** | **20M** | ✅ **"Low float stocks"** | 🆕 FMP |
| 8 | **Cap Max** | **2B** | ✅ **Small cap only** | 🆕 FMP |
| 9 | Catalyst | MR | ✅ **"There must be a reason"** | Alpaca |
| 10 | Excludes Earnings | OFF | Does not exclude (earnings = valid catalyst) | 🆕 FMP |

### Ross Cameron's 5 criteria vs Titan implementation:

| Criterion | Description | Titan V7 | Titan V8 (FMP) |
|----------|-----------|:--------:|:--------------:|
| 1. RVOL | Relative volume > 5x | ✅ | ✅ |
| 2. Change | Already UP 10%+ | ✅ | ✅ |
| 3. Catalyst | Reason for movement | ✅ | ✅+ (insider) |
| 4. Float | Small float (< 20M) | ❌ was missing | ✅ **FMP** |
| 5. Price | $1 - $20 | ✅ | ✅ |
| 6. Market Cap | Small chap | ❌ was missing | ✅ **FMP** |

**Titan V8 implements ALL Ross Cameron criteria — 100% complete.**

### Why does it work?

The combination **small price + explosive volume + small float + small cap** identifies stocks that:
- They have a catalyst (news, earnings, insider buying)
- They are the focus of traders' attention
- They have low float = limited supply + high demand = **price explosion**
- They have the potential of +20-50% intraday
- But also -20-30% risk if the momentum reverses

### ⚠️ Attention

**The Ross Cameron strategy is AGGRESSIVE.** Not suitable for beginners with small capital. The risk of loss is as great as the potential for gain.

---

## 12. INJECT IN WATCHLIST / SIGNAL CFG

### INJECT → WATCHLIST (Cyan)

Clicking this button adds **ALL** actions from the current results to your watchlist (favorites).

**What happens:**
1. Each symbol in the results is sent to the backend with `add_favorite`
2. The watchlist is updated instantly
3. The symbols become visible in the WATCHLIST tab in the sidebar

### INJECT → SIGNAL CFG (Magenta/Purple) 🆕

Clicking this button injects the results directly into the **Signal CFG** — the asset list for the automated trading strategy.

**What happens:**
1. Symbols are sent to the backend with `inject_to_signal_assets`
2. Signal CFG receives the new assets for monitoring
3. The button flashes magenta as confirmation

**Difference:**
- **WATCHLIST** = your personal watchlist (passive)
- **SIGNAL CFG** = the active list that the bot uses for automatic trading

### AUTO INJECT

When enabled, the result list is automatically injected after each scan (manual or auto).

### The button "+" per row

Each row in the results has a **"+"** button to the right that adds ONLY that symbol to the watchlist:
- Click "+" → It changes to "✓" (cyan) for 1.5 seconds
- The symbol appears instantly in the watchlist

---

## 13. HOW THE SCANNER WORKS INSIDE (BACKEND)

### The complete pipeline (from click to results)

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

### Detailed scoring formula

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

## 14. FMP DATA INTEGRATION — FREE BASIC DATA

### 14.1 What is FMP (Financial Modeling Prep)?

FMP is an API service that provides **fundamental business data** — data that Alpaca ($99/mo) **DOES NOT provide**. The free plan includes **250 API calls/day**, enough to use Titan.

### 14.2 Setup FMP API Key

1. Create a free account at [financialmodelingprep.com](https://financialmodelingprep.com)
2. Copy the API Key from the dashboard
3. In Titan, open the **API & WALLET** from the sidebar
4. Paste the key in the **FMP API KEY** field
5. Click **SAVE FMP KEY** → The button turns green "✓ SAVED"
6. The key is saved persistently in `titan_keys.json`

### 14.3 The 5 Integrated FMP Modules

| # | Module | What it offers | Cache | API Calls |
|---|-------|---------|-------|----------|
| 1 | **Float** | No. shares in float (millions) | 24h | 1/symbol |
| 2 | **Company Profile** | Market head + sector | 24h | 1/symbol |
| 3 | **Insider Trading** | Buying/selling CEO, CFO, etc. | 24h | 1/symbol |
| 4 | **Earnings Calendar** | Earnings ±3 days | 12h | 1 call TOTAL |
| 5 | **Price Targets** | Analyst consensus (upside %) | 7 days | 1/symbol |

### 14.4 Budget API Calls (250/day free)

With aggressive cache, typical consumption:

| Scenario | First scan | Subsequent scans | Total/day |
|----------|:------------:|:------------------:|:--------:|
| 1 manual scan | ~150 | — | ~150 ✅ |
| Auto-Scan 30 min | ~150 | ~20-30 each | ~200 ✅ |
| Auto-Scan 15 min | ~150 | ~20-30 each | ~230 ✅ |
| Auto-Scan 5 min | ~150 | ~20-30 each | ~250 ⚠️ |

**Why is it effective?** The cache makes the second scan only consume calls for NEW symbols (that were not in the previous scan). Float, profiles and targets remain cached.

### 14.5 Insider Trading — The Strongest Signal

**What are insider trades?** When the CEO, CFO or executives of a company **buy or sell stock with their personal money**. These transactions are compulsorily reported to the SEC (Form 4).

**Why does it matter?**
- **Insider BUY** = The insider believes the stock will GO UP
- It is considered **the most credible bullish signal** in the market
- Studies show that insider buys beat the market by 6-10% per year
- **Insider SELL** = Less significant (can sell for various reasons)

**Example in table:**
```
SYMBOL  PRICE  CHG%  RVOL  VOL   FLOAT  CAP    INS   SCORE
MULN    $2.35  +45%  12x   5.2M  8.2M   120M   🟢    87.3
                                                 ^
                                          Hover: "John Smith BUY 50000sh @$1.85"
```

### 14.6 Earnings Calendar — Protection or Opportunity?

The 📊 badge automatically appears next to the stock symbol with earnings ±3 days:

```
MULN📊  $2.35  +45%  ...
```

**Two approaches:**
1. **Conservative:** Enable "EXCLUDE EARNINGS" → Avoid risk
2. **Aggressive:** Explicitly look for stocks with 📊 → Earnings gap-and-go

### 14.7 Price Targets — Upside Potential

The `analyst_upside_pct` score is automatically calculated:
```
Upside% = (Consensus Target - Preț Actual) / Preț Actual × 100
```

Example: If analysts target $50 and the stock is at $10 → Upside = +400%.

---

## 15. PRACTICAL USE SCENARIOS

### Scenario 1: "I'm a beginner, what do I do?"

1. Open **Market Discovery**
2. Press **🏛️ LARGE CAP**
3. Press **⚡ SCAN**
4. Look at the first 5 results — all large, liquid companies
5. Click on the symbol → Chart loads
6. Analyze the chart and decide

### Scenario 2: "I want shares at the open (9:30 AM)"

1. At 9:35 AM ET, click **🔥 HOT MOMENTUM**
2. Press **⚡ SCAN**
3. Mentally sort by RVOL (those with 5x+ are the most interesting)
4. Click on top 3 → Analyze chart quickly

### Scenario 3: "I want explosive penny stocks"

1. Press **🚀 PENNY ROCKETS**
2. Press **⚡ SCAN**
3. Search for actions with RVOL > 10x and Change > 30%
4. **⚠️ EXTREME RISK** — set stop-loss TIGHT

### Scenario 4: "I want continuous scanning" (passive monitoring)

1. Set the desired parameters (or press a preset)
2. Set **Auto-Scan Interval** to 15 min
3. Press **AUTO SCAN: OFF** → becomes **ON**
4. Let it scan itself
5. Watch the 🔴 badge and toasts for NEW MOVERS
6. When you see something interesting → Click on the symbol

### Scenario 5: "I want to compare two scans"

1. Press **⚡ SCAN** at 10:00 AM → First results
2. Press **⚡ SCAN** at 10:30 AM → Results with column Δ
3. Mentally sort by Δ:
   - **Δ = +20** → Action Accelerates NOW → Investigate!
   - **Δ = -15** → Momentum Declines → Avoid
   - **NEW** → Brand New Action → Check Now

---

### Scenario 6: "I want Ross Cameron COMPLETE" (V8 with FMP) 🆕

1. Set FMP API Key (one time)
2. Press **⚔️ ROSS CAMERON** → Automatically set: Float ≤20M, Cap ≤2B, RVOL ≥5x, Change ≥10%
3. Press **⚡ SCAN**
4. FLOAT (purple), CAP (yellow), INS columns (🟢/🔴) appear automatically
5. Look for stocks with: **Score >60, Float <10M, Cap <500M, INS=🟢**
6. These are the PERFECT candidates Ross Cameron — low float, small cap, insider confidence

### Scenario 7: "I want to avoid earnings" 🆕

1. Enable **EXCLUDE EARNINGS ±3d**
2. Scan normally
3. All actions with 📊 are automatically removed
4. Trade without earnings surprises

---

## 16. EXPERT TIPS FOR BEGINNERS

### ✅ DO (What to do)

1. **Scan the first 30 min** (9:30-10:00 AM ET) — this is where the most volume is
2. **Use RVOL ≥ 3** — stocks with RVOL below 2 are "dead"
3. **Check chart** before trading — scanner finds candidates, NOT signals
4. **Set stop-loss** BEFORE entering — decide how much you are willing to lose
5. **Use Score Mode correctly** — if you are a day trader, use MOMENTUM; if you are a swing trader, GAP SCANNER
6. **Scan 2+ times** — Δ column only appears from the second scan

### ❌ DON'T (What NOT to do)

1. **DO NOT trade penny stocks** without experience — they are extremely volatile
2. **DO NOT ignore volume** — low volume stocks have spreads high
3. **DO NOT enter based on score alone** — score indicates potential, NOT guarantee
4. **DO NOT scan on weekend** — data is from Friday, not relevant
5. **DO NOT leave Auto-Scan on 5 min** if you don't need to — consumes API rate limits
6. **DO NOT ignore negative Δ** — a delta of -20 means momentum disappear

### 📊 Practice Rules

| Rule | Explanation |
|--------|-----------|
| **Score > 60** | Strong candidate — worth looking into |
| **Score 30-60** | Average candidate — only check if you have time |
| **Score < 30** | Weak — probably not worth attention |
| **RVOL > 5** | Explosive activity — something is happening |
| **Gap > 5%** | Significant gap — real continuation potential |
| **Δ > +10** | Acceleration — NOW is the time |
| **NEW badge** | Fresh action — check now |

| **FLOAT < 10M** | Ultra low float — movements of ±30-50% | |
| **CAP < 300M** | Micro cap — Ross Cameron area |
| **INS = 🟢** | Insider buying — the strongest bullish signal |
| **📊 badge** | Has earnings ±3 days — BEWARE of risk! |

---

## 17. FREQUENT PROBLEMS AND SOLUTIONS

### P: "I pressed SCAN but I get no results"

**Possible solutions:**
1. **Filters are too restrictive** — Set Change Min to 1%, RVOL to 1, Gap to 0%
2. **Market is closed** — Scanner works best 9:30am-4:00pm ET Mon-Fri
3. **Missing API keys** — Check the API & WALLET from sidebar
4. **Weekend** — Friday's data will not have the same movements

### P: "The Δ column does not appear"

**Explanation:** Δ only appears from the **SECOND scan** in the current session. At the first scan it has nothing to compare it with. Scan again and it will appear.

### P: "I got toast with NEW MOVERS but I don't see the row"

**Explanation:** Scroll down in the results panel. New Movers may be lower on the list (their score may be lower than Old Movers).

### P: "Scanning is taking a long time"

**Possible causes:**
1. **Max Candidates = 300** → Set to 50-100 for speed
2. **Slow internet connection** → Snapshots require data from Alpaca
3. **API rate limiting** → Alpaca has request limit per minute. If you have multiple scans at once, they slow each other down.

### P: "Morning vs evening results are different"

**Normal!** Mornings (9:30-10:30AM ET) have the most high-moving action. In the evening (3:00pm+), the volume drops and less action passes the filters.

### P: "The score of an action has changed dramatically"

**Normal!** The score is recalculated on every scan with the LIVE data. If an action continued to increase (change +5% → +15%), the score will increase proportionally.

### P: "FLOAT, CAP, INS columns do not appear" 🆕

**Explanation:** These columns require **FMP API Key**. If you have not set the key, the scanner works normally but without FMP data. Set the key to the API & WALLET → FMP API KEY → SAVE FMP KEY.

### P: "I see 'FMP limit reached' in log" 🆕

**Explanation:** You have exceeded 250 API calls/day. Solutions:
1. Increase Auto-Scan interval to 30+ min
2. Cached data remains valid — scan continues with cache
3. Tomorrow the counter resets
4. Optional: upgrade to Starter FMP plan ($19/mo) for 300 calls/minute

---

## 18. GLOSSARY OF TERMS

| Term | Definition |
|--------|----------|
| **Ask** | The lowest price at which someone sells the stock |
| **Bid** | The highest price at which someone buys the stock |
| **Catalyst** | Event or news that causes price movement |
| **Change %** | Percentage difference between current price and yesterday's close |
| **Day Trading** | Buying and selling on the same day |
| **Delta (Δ)** | Score difference between two consecutive scans |
| **Gap** | The difference between today's open and yesterday's close |
| **Gap-up** | The stock opened higher than it closed yesterday |
| **Gap-down** | The stock opened lower than it closed yesterday |
| **Large Cap** | Company with market cap over $10 billion |
| **Liquidity** | How easily you can buy/sell without affecting the price |
| **Mid Cap** | Company with capitalization between $2-10 billion |
| **Momentum** | The price trend to continue in the same direction |
| **New Mover** | Action that first appears in scan results |
| **Penny Stocks** | Action priced under $5 |
| **Preset** | Predefined configuration that sets all parameters with one click |
| **RVOL** | Relative Volume — today's volume divided by yesterday's volume |
| **Score** | Calculated score (0-100) based on change, rvol, gap and volume |
| **Score Mode** | The weighting mode of the score components |
| **Short Selling** | Selling a stock you don't own (profits on the decline) |
| **Slippage** | The difference between the desired price and the actual strike price |
| **Small Cap** | Company with capitalization below $2 billion |
| **Snapshot** | Instant capture of price, volume and other data |
| **Spread** | Difference between Bid and Ask |
| **Swing Trading** | Transactions held 1-5 days |
| **Toast** | Temporary notification that appears and disappears automatically |
| **Volumes** | Total number of shares traded |
| **Float** 🆕 | The number of shares publicly available for trading |
| **Market Cap** 🆕 | Market Cap = Price × Total Shares |
| **Insider Trading** 🆕 | Transactions made by company insiders (CEO, CFO, etc.) |
| **Insider BUY** 🆕 | CEO/CFO/director buys stock with own money — bullish signal |
| **Insider SELL** 🆕 | CEO/CFO/director sells stock — may be normal (diversification) |
| **Earnings** 🆕 | The company's quarterly financial report |
| **FMP** 🆕 | Financial Modeling Prep — free API service for fundamental data |
| **Micro Head** 🆕 | Company with market cap under $300M |
| **Small Cap** 🆕 | Company with market cap $300M - $2B |
| **SEC Form 4** 🆕 | SEC form reporting insider trades |
| **Price Target** 🆕 | Price target estimated by Wall Street analysts |
| **Upside** 🆕 | The percentage difference between the current price and the price target |

---

## TECHNICAL REFERENCES

### Relevant files

| File | What it contains |
|--------|-----------|
| `src-tauri/src/scanner.rs` | Scanning, filtering and scoring engine |
| `src-tauri/src/commands/scanner_cmd.rs` | Tauri order `run_market_scan` |
| `src-tauri/src/commands/settings.rs` | Saving scan parameters |
| `src-tauri/src/loops/ai_scan_loop.rs` | Auto-scan loop with weekend guard |
| `src/titan_watchlist.js` | Frontend: SCAN button, presets, DIFF ALERT, NEW MOVER |
| `src/titan_manual_trade.js` | Restore startup settings |
| `src/index.html` | HTML for the Market Discovery panel |
| `src/styles.css` | Cyber-HUD styles |

### Data Structures (Rust)

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

### FMP Caches (Backend)

```rust
FLOAT_CACHE    → HashMap<Symbol, (float_m, Instant)>    // TTL: 24h
PROFILE_CACHE  → HashMap<Symbol, (ProfileData, Instant)> // TTL: 24h  
INSIDER_CACHE  → HashMap<Symbol, (InsiderSignal, Instant)> // TTL: 24h
EARNINGS_CACHE → Option<(Instant, HashMap<Symbol, bool>)> // TTL: 12h (1 call!)
TARGETS_CACHE  → HashMap<Symbol, (upside_pct, Instant)>  // TTL: 7 zile
```
```

---

*Titan HFT System — Market Discovery Manual, March 2026*  
*Version: V8 FMP-Enriched*  
*Data sources: Alpaca Markets + Financial Modeling Prep (FMP)*

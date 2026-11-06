# TITAN SIGNAL SCALPING ADVANCED PLAYBOOK (1m-4h)

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


This document is the advanced version for manual trading in `SIGNAL`, without bot dependency.
It is intended for disciplined execution on all Alpaca timeframes between `1m` and `4h`.

Main objective:

- increasing the quality of inputs
- reducing drawdown
- clear protocol for outputs at `TRAIL` and `CHAOS`
- fast but standardized decisions

---

## 1) Correct mental model: what the signals mean

Signals are not certain predictions. They are "probabilistic trigger + management".

- `BUY` / `SELL` = input trigger
- `TRAIL` / `EXIT` / `CHAOS` = management / output trigger

Prefixes on the chart:

- `O` = Oracle
- `G` = GPU
- `F` = Fusion

Critical rule:

- entry is taken on `BUY/SELL`
- exit is executed on `TRAIL/EXIT/CHAOS`
- do not automatically reverse the position at the first exit signal

---

## 2) MTF stack: 3-level professional method

Do not trade "single timeframe". It uses 3 levels:

- `Anchor TF` (direction context)
- `Trigger TF` (signal)
- `Execution TF` (effective input)

Recommended preset:

- for 1m execution: anchor=15m, trigger=5m
- for 3m execution: anchor=30m, trigger=10m
- for 5m execution: anchor=30m, trigger=15m
- for 10m execution: anchor=1h, trigger=15m
- for 15m execution: anchor=1h, trigger=30m
- for 30m execution: anchor=4h, trigger=1h
- for 1h execution: anchor=4h, trigger=1h
- for 4h execution: anchor=1d (context), trigger=4h

Rule:

- if the anchor is clearly opposed to the setup, skip the trade
- if the trigger and execution are aligned, the trade has priority

---

## 3) Source selection (O/G/F) depending on the regime

### 3.1 Clean trending

- primary: `FUSION`
- secondary: `ORACLE`
- `GPU` only for fine input timing

### 3.2 Mean reverting

- primary: `ORACLE`
- secondary: `FUSION`
- `GPU` with size reduced

### 3.3 Chaotic / high noise

- primary: none for aggressiveness
- fallback: `ORACLE` with minimal risk
- avoid overtrading on `GPU`

Practically:

- if you see many `CHAOS` in series, reduce the frequency and size

---

## 4) Entry Quality Score (0-100) - do not fall below the threshold

Quickly calculate an internal score before clicking:

- fresh signal on closed bar: 0-20
- alignment with Anchor TF: 0-20
- price structure (HH/HL or LH/LL): 0-15
- normal volatility (no extreme spike): 0-10
- no `CHAOS` in last 2 bars: 0-15
- reasonable stopping distance (ATR): 0-10
- minimum risk/reward 1.8: 0-10

Execution threshold:

- `>= 70`: normal execution
- `60-69`: reduced size (50%)
- `< 60`: no trade

---

## 5) Advanced position sizing (risk-first)

Simple formula:

`qty = (equity * risk_pct) / abs(entry - stop)`

Recommended settings:

- 1m-5m: `risk_pct` 0.20% - 0.40%
- 10m-30m: `risk_pct` 0.35% - 0.60%
- 1h-4h: `risk_pct` 0.50% - 0.80%

Hard limits:

- max daily loss: 2.0% equity
- max 3 consecutive losses -> pause 30-60 min
- do not double size after loss

---

## 6) Exit Decision Tree (obligatory)

Priority order:

1. hard invalidation (technical stop) -> immediate exit
2. `CHAOS` -> fast defensive output
3. `TRAIL` -> lock profit
4. opposite signal confirmed -> closing rest

Partial exits model (recommended):

- TP1: 40%
- TP2: 30%
- rest 30% managed with `TRAIL`

When `CHAOS` appears after entry:

- on 1m/3m/5m: closing 70-100%
- on 10m+: minimum discount 50%

---

## 7) Execution rules on sessions

### US Open (first 20-30 min)

- high volatility, variable spread
- use reduced size for the first 2 entries

### Mid-session

- usually the best area for clean runs
- strictly follows the MTF protocol

### The last hour

- avoid new entries on marginal signals
- prefer management of existing positions

---

## 8) Anti-patterns that destroy performance

- change `SRC` in the middle of the trade
- you enter the unclosed bar from FOMO
- treat each `SELL` as a mandatory short
- ignore `CHAOS` for "may return"
- overtrade after 2-3 losses

---

## 9) Minimum KPI for real performance control

Monitor separately on `O/G/F` and each TF:

- win rate
- expectation (R/trade)
- average win / average loss
- max adverse excursion (MAE)
- max favorable excursion (MFE)
- average hold time
- output percentage on `TRAIL` vs `CHAOS`

What are you looking for:

- the source that produces constant positive expectancy
- TFs where `CHAOS` occurs too often (high noise)

---

## 10) Weekly tuning protocol (no improvisation)

Once a week:

1. extract the last 50-100 trades
2. segment by source (`O/G/F`) and TF
3. mark trades with entry score <70 (should be removed)
4. adjust only 1-2 variables, not the whole system at once
5. the same version runs for 1 week

Rule:

- don't trade based on a single trade

---

## 11) New ideas (practical roadmap)

Below are concrete ideas that are worth implementing:

### 11.1 Signal Health Panel in UI

Show live:

- active source (`O/G/F`)
- `BUY count`, `SELL count`, `TRAIL count`, `CHAOS count` on the last N bars
- feed status: `LIVE`, `STALE`, `0 markers`

### 11.2 Confidence ribbon on the chart

Discreet overlay with "quality zone":

- green: good setup
- yellow: medium setup
- red: avoid entry

### 11.3 Optional Mode Consensus

Entry only if 2/3 sources confirm the direction in the short time window.
It can reduce frequency, but increase robustness.

### 11.4 Auto cooldown after CHAOS

After `CHAOS`, block new entries X bars on the same symbol/TF.

### 11.5 News risk guard

Protection window around major events (e.g. +/- 5 min):

- reduce size automatically
- or block new entries

### 11.6 Slippage-aware execution hints

In UI:

- "spread high -> reduce size"
- "execution quality low -> skip"

### 11.7 Session templates

Preset profiles:

- Open Aggressive
- Mid-session Balanced
- Late-session Defensive

---

## 12) Ultra-fast checklist (10 seconds)

Before entering:

- [ ] MODE = SIGNAL
- [ ] SRC fixed (I don't change it in trade)
- [ ] signal on closed bar
- [ ] Entry Score >= 70
- [ ] Anchor TF is not clearly opposed
- [ ] risk/trade within limits
- [ ] I know exactly the output trigger (`TRAIL` / `CHAOS`)

If a point is missing: `NO TRADE`.

---

## 13) Operational script for decision (mental)

Question 1: "Do I have a valid input signal?"

- no -> expect
- yes -> question 2

Question 2: "Is the setup aligned MTF and has a score >= 70?"

- no -> skip
- yes -> question 3

Question 3: "Controlled risk + clear exit plan?"

- no -> skip
- yes -> execution

---

## 14) Conclusion

The real edge comes from:

- better entry selection
- disciplined exit execution
- strict risk control

Not from "multiple signals".

Use signals as an execution tool, not as a promise.


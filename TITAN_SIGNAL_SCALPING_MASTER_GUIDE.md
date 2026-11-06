# TITAN SIGNAL SCALPING MASTER GUIDE (1m-4h)

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


This document is a complete guide for manual trading in TITAN HFT using `SIGNAL` mode on timeframes between `1m` and `4h`.

The goal is to have a clear, repeatable protocol, with simple rules for entries/exits, reducing losses and correct interpretation of markers.

---

## 1) What do the signals in the chart represent

In `SIGNAL` mode, the markers are displayed for the active source (`SRC`):

- `O ...` = Oracle
- `G ...` = GPU
- `F ...` = Fusion

Signal types:

- `BUY` = long entry signal
- `SELL` = short entry signal / long exit
- `TRAIL` = trailing exit signal (profit protection)
- `EXIT` = closing signal (flip logic)
- `CHAOS` = defensive signal (unstable regime, risk increased)

Important:

- `BUY/SELL` = entry signals
- `TRAIL/EXIT/CHAOS` = management and exit signals
- Do not interpret `TRAIL`/`CHAOS` as automatic reversal entry.

---

## 2) Correct interpretation on blue/red candles

Many traders make this confusion:

- The color of the candle only tells what the current bar did.
- The marker tells what you are doing with your position.

That's why:

- you can have `G CHAOS` on a blue candle (it's normal)
- you can have `G TRAIL` on a red candle (it's normal)

Explanation:

- `CHAOS` does not necessarily mean the immediate opposite direction, it means "the market is unstable, reduce risk"
- `TRAIL` means "we had favorable movement, now protect the profit"

---

## 3) Differences between sources (O/G/F)

### Oracle (`O`)

- More filtered, more conservative.
- Fewer signals, usually cleaner.

### GPU (`G`)

- More sensitive, faster, reacts quickly.
- Can generate more signals in volatile areas.

### Fusion (`F`)

- Mix between Oracle stability and GPU speed.
- Usually good as the "balanced" variant.

Recommendation:

- Do not change the source in the middle of the same trade.
- Choose the source before entering and keep the same "source context" until exit.

---

## 4) Standard workflow for each session

1. Run `build.sh` and start the application.
2. Select the desired symbol and timeframe.
3. Set `MODE = SIGNAL`.
4. Choose `SRC` (`ORACLE` / `GPU` / `FUSION`).
5. Check toggle `Signals` active in UI.
6. Wait for input signal (`BUY`/`SELL`) on bar closed.
7. Handle output with `TRAIL`/`EXIT`/`CHAOS`.

Don't skip step 6. Input on bar not closed significantly increases the rate of false entries.

---

## 5) Playbook on timeframes (1m -> 4h)

## 5.1) Fast TF: 1m / 3m / 5m

- Enter only on fresh signal.
- After `TRAIL`: fast exit (partial minimum).
- After `CHAOS`: aggressive exit, without hesitation.
- After exit: manual cooldown 1-3 bars.

## 5.2) Average TF: 10m / 15m / 30m

- Enter `BUY/SELL` confirmed (closed bar).
- `TRAIL` = gradual exit (eg: 30-60%).
- `CHAOS` = greatly reduce exposure.

## 5.3) Slow TF: 1h / 4h

- Rare signals, usually more robust.
- Use `TRAIL` to conserve profit, not panic.
- `CHAOS` remains a risk signal raised -> defensive.

---

## 6) Concrete entry/exit rules

### LONG entry rules

- signal `BUY` on the closed bar
- without `CHAOS` immediately in the next 1-2 bars
- ideal: the price structure is not already "extended up" on extreme spike

### SHORT entry rules

- signal `SELL` on closed bar
- without `CHAOS` immediately in next 1-2 bars
- avoid short immediately after strong flush if absorption occurs

### Exit rules

- `TRAIL`: mark profit (partial or full, depending on TF)
- `EXIT`: technical shutdown (flip logic)
- `CHAOS`: defensive output (usually full on small TFs)
- clear opposite signal: close what is left

---

## 7) Risk management (mandatory)

These rules are more important than any signal:

- small and fixed risk per trade
- do not increase size after a loss
- daily loss limit (hard stop)
- consecutive loss limit (ex: 3 trades in a row -> break)

Practical example:

- max risk/trade: 0.25% - 0.75% of equity
- max daily loss: 1.5% - 2.5%
- after 3 consecutive losses: break and reevaluation

---

## 8) When NOT to enter

Do not enter if:

- the signal appears in the middle of a news spike without confirmation
- you just received `CHAOS` and the cooldown didn't pass
- change the source in the middle of the decision (inconsistent context)
- you enter "for fear of losing the movement"

---

## 9) Why can you see many SELL and few BUY (or vice versa)

It is normal if:

- the current regime is directional (dominant trend)
- the confidence/HTF filters remove part of the entries
- volatility state favors a direction

It's not an automatic bug. A bug is when:

- one of the sources remains constantly blocked in `0 markers` under all conditions
- the source does not produce anything for long periods, while the others produce normally

---

## 10) Fast troubleshooting

### Problem: No markers appear at all

Check:

- `MODE = SIGNAL`
- `SRC` correctly selected
- toggle `Signals` active
- valid timeframe (1m, 3m, 5m, 10m, 15m, 30m, 1h, 4h)

### Problem: only one source does not show signals

- switch to the other sources for comparison
- check the backend logs
- if only `gpu` drops to zero, check the GPU/thresholds pipeline

### Problem: the SHARK icon started the old build

- `build.sh` must update:
  - the local binary
  - system binary
  - local launcher
  - system launcher
  - desktop/icon cache

---

## 11) Checklist "before clicking BUY/SELL"

Before any entry:

- [ ] MODE is `SIGNAL`
- [ ] SRC is fixed (O/G/F) and I don't change it in trade
- [ ] Selected TF is clear (1m -> 4h)
- [ ] signal on closed bar
- [ ] calculated risk/trade
- [ ] I know exactly where I exit if `TRAIL` or `CHAOS` appears

If a point is missing, do not execute the trade.

---

## 12) Checklist "after entry"

- [ ] monitor the following 1-3 bars
- [ ] at `TRAIL` reduce / close according to plan
- [ ] at `CHAOS` go into defensive mode
- [ ] I don't move the decision on emotion
- [ ] note the result in the diary

---

## 13) Minimum trading log (recommended)

For each trade write down:

- date/time
- symbol
- TF
- source (`O/G/F`)
- signal input (`BUY/SELL`)
- output signal (`TRAIL/EXIT/CHAOS/opus`)
- result (% and multiple R)
- observations (why it worked / why it didn't work)

After 30-50 trades you have real data for tuning.

---

## 14) Final principles

1. The signal is not certainty, it is probability.
2. Fixed risk > "perfect prediction".
3. Discipline beats the indicator.
4. `CHAOS` is your friend in terms of protection.
5. The best missed trade is better than a poorly executed trade.

---

## 15) Note of responsibility

This system cannot guarantee "no loss".
It can only provide clearer and faster signals.
The final result depends on execution, risk, discipline and market context.


# TITAN Daily Scalping Checklist (1 Page)

> **Disclaimer:** Educational and research material only — **not financial or investment advice**. Trading involves substantial risk of loss; past or simulated results do not indicate future results. Titan HFT is provided AS-IS, with no liability and no support. No API keys are included.


Use this checklist at the start of every session and before every trade.  
Goal: consistent execution, low emotional drift, controlled risk.

---

## A) Pre-Market (5-10 min)

- [ ] App rebuilt/restarted recently (`build.sh`) and UI is live.
- [ ] `MODE = SIGNAL`.
- [ ] Source chosen (`ORACLE` / `GPU` / `FUSION`) and fixed for first setup.
- [ ] Signals toggle is ON.
- [ ] Symbol list prepared (max 3-5 symbols).
- [ ] Session context identified (trend / range / chaotic).
- [ ] Daily max loss set (hard stop).
- [ ] Risk per trade set (fixed % of equity).

---

## B) Before Any Entry (10-second gate)

- [ ] Signal is on a CLOSED candle (`BUY` or `SELL`).
- [ ] MTF alignment is acceptable (anchor not strongly opposite).
- [ ] No immediate `CHAOS` conflict in last 1-2 candles.
- [ ] Stop location is clear and valid.
- [ ] Risk/Reward is acceptable (target >= 1.8R preferred).
- [ ] Position size matches risk rule (not emotional size).

If any box is unchecked -> NO TRADE.

---

## C) Entry Rules

- [ ] Long only on `... BUY`.
- [ ] Short only on `... SELL`.
- [ ] No source switching mid-position.
- [ ] No adding size if entry thesis is unclear.

---

## D) Position Management

- [ ] On `TRAIL`: secure profits (partial/full by TF).
- [ ] On `CHAOS`: reduce risk aggressively (fast on low TF).
- [ ] On opposite confirmed signal: close remainder.
- [ ] Do not "hope hold" after invalidation.

---

## E) Risk Controls (Non-negotiable)

- [ ] Max daily loss not exceeded.
- [ ] Max consecutive losses not exceeded.
- [ ] If two poor executions in a row -> pause and reset.
- [ ] No revenge trade after loss.

---

## F) Post-Session (5 min)

- [ ] Export/log all trades.
- [ ] Tag each trade by source (`O/G/F`) and TF.
- [ ] Note key mistakes (max 3 bullet points).
- [ ] Note one improvement for next session.
- [ ] Stop trading when checklist quality drops.

---

## G) Personal Protocol

- [ ] I trade only my plan.
- [ ] I protect capital first.
- [ ] I skip low-quality setups.
- [ ] I accept missed trades.
- [ ] I end the day disciplined, not exhausted.


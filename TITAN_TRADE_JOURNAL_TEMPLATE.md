# TITAN Trade Journal Template (How To Use)

Use together with `TITAN_TRADE_JOURNAL_TEMPLATE.csv`.

## 1) Minimum logging fields per trade

- `symbol`, `timeframe`, `source`
- `setup_side` (LONG/SHORT)
- `entry_signal` and `exit_signal`
- `entry_price`, `stop_price`, `position_size`
- `risk_pct`, `r_multiple_pl`, `pnl_usd`
- `market_regime`, `entry_score`, `mtf_alignment`
- `mistake_tag`, `execution_quality`, `emotion_state`

If these fields are missing, review quality drops quickly.

## 2) Mistake tags (standardize)

- `None`
- `EarlyEntry`
- `LateExit`
- `IgnoredChaos`
- `Oversized`
- `NoMTFAlignment`
- `RevengeTrade`
- `SourceSwitchMidTrade`

Use one primary tag per trade.

## 3) Execution quality scale

- `Excellent`
- `Good`
- `Average`
- `Poor`

Rule:

- `Poor` + loss = mandatory note with one corrective action.

## 4) Weekly review protocol

Every weekend:

1. Group by `source` (`O/G/F`).
2. Group by `timeframe`.
3. Compute:
   - win rate
   - average R
   - total PnL
   - top 3 mistake tags
4. Keep only setups with positive expectancy.
5. Disable low-expectancy patterns for next week.

## 5) Monthly scorecard (quick)

- Best source:
- Worst source:
- Best timeframe:
- Worst timeframe:
- Most costly mistake tag:
- One process upgrade for next month:


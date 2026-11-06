# Titan HFT — Paper-Trading Track Record

> **Status: no meaningful track record yet.** Generated 2026-11-06.

Titan HFT has been used primarily as an **execution, visualisation and GPU-simulation engine** (60 FPS chart, order-book heatmap, 150K-agent GPU Nexus), not as an autonomous paper-trading bot. Its journal contains **1 real autonomous trade** (net $55.48) plus one synthetic smoke-test entry that is excluded here. That is far too little to say anything about edge, and we will not pretend otherwise.

| Date | Symbol | Side | Qty | Entry | Exit | P&L | Mode | Reason |
|---|---|---|---|---|---|---|---|---|
| 2026-11-06 | SPY | LONG | 68 | 733.3542 | 734.1700 | $55.48 | SignalStrategy | PARTIAL_TAKE |

**Backtests shipped with the app** (e.g. the built-in QBTS 1-day backtest: 5 trades, +13.1%, 64% max drawdown, Sharpe 0.60) are single-symbol, in-sample demonstrations of the backtester UI — *not* evidence of a strategy edge.

**What *is* demonstrable today:** the engine's latency/throughput, the WGPU simulation, the risk guards and the broker integration — see the engine manual and the CI test suite. If an autonomous paper-trading campaign is run in the future, its full journal will be published here in the same unedited form as SentinelTrend's.

---
*MIT · AS-IS · zero warranty · not financial advice.*

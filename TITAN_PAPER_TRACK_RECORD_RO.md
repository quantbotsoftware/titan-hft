# Titan HFT — Istoric de paper-trading

> **Stare: încă nu există un istoric semnificativ.** Generat la 2026-11-06.

Titan HFT a fost folosit în principal ca **motor de execuție, vizualizare și simulare GPU** (grafic 60 FPS, heatmap order-book, GPU Nexus cu 150K agenți), nu ca bot autonom de paper-trading. Jurnalul său conține **1 tranzacție autonomă reală** (net $55.48) plus o intrare sintetică de test, exclusă aici. Este mult prea puțin pentru a spune ceva despre avantaj statistic și nu vom pretinde contrariul.

| Dată | Simbol | Direcție | Cant. | Intrare | Ieșire | P&L | Mod | Motiv |
|---|---|---|---|---|---|---|---|---|
| 2026-11-06 | SPY | LONG | 68 | 733.3542 | 734.1700 | $55.48 | SignalStrategy | PARTIAL_TAKE |

**Backtest-urile livrate cu aplicația** (ex. backtest-ul QBTS 1 zi inclus: 5 tranzacții, +13,1%, drawdown maxim 64%, Sharpe 0,60) sunt demonstrații pe un singur simbol, in-sample, ale interfeței de backtesting — *nu* dovezi ale unui avantaj statistic.

**Ce se poate demonstra azi:** latența/debitul motorului, simularea WGPU, gărzile de risc și integrarea cu brokerul — vezi manualul motorului și suita de teste din CI. Dacă în viitor se va rula o campanie autonomă de paper-trading, jurnalul ei complet va fi publicat aici în aceeași formă needitată ca al SentinelTrend.

---
*MIT · AS-IS · fără garanții · nu este sfat financiar.*

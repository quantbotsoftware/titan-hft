# TITAN HFT ENGINE — Manual de Utilizare Detaliat

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


**Versiune:** V7 Reloaded V5.9
**Tab:** HFT Engine (tab-4)
**Data:** 28 Martie 2026 (Sesiunea 19). Actualizat: 3 Aprilie 2026 (Sesiunea 69 — Wave 2B SmartTrend Correctness)
**Limba:** Romana
**Nivel:** Incepator → Avansat

---

## CUPRINS

1. [Introducere](#1-introducere)
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
25. [Effective Exit Values (Live Impact Preview)](#25-effective-exit-values)
26. [Regime Heatmap](#26-regime-heatmap)
27. [Parameter Profiles](#27-parameter-profiles)
28. [Risk Calculator Live](#28-risk-calculator-live)
29. [Parameter Health](#29-parameter-health)
30. [Auto-Suggest](#30-auto-suggest)
31. [Per-Symbol Overrides](#31-per-symbol-overrides)
32. [Preset Bar — Butoane si Controale Globale](#32-preset-bar)
33. [Settings Diff Panel](#33-settings-diff-panel)
34. [DRY RUN vs LIVE](#34-dry-run-vs-live)
35. [Persistenta Setarilor](#35-persistenta-setarilor)
36. [Recomandari si Tips](#36-recomandari-si-tips)
37. [Glosar de Termeni](#37-glosar-de-termeni)
38. [Keyboard Shortcuts si Command Palette](#38-keyboard-shortcuts-si-command-palette)
39. [TA Entry Gates — Filtre Pre-Entry cu Soft/Hard Mode](#39-ta-entry-gates)
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

## 1. INTRODUCERE

### Ce este tab-ul HFT Engine?

Tab-ul **HFT Engine** (al 5-lea tab din aplicatia TITAN, identificat intern ca `tab-4`) este **centrul de comanda** al botului de trading. Aici se configureaza absolut toti parametrii care controleaza modul in care botul:

- Decide cand sa INTRE intr-o tranzactie
- Decide cand sa IASA dintr-o tranzactie
- Cat de mult sa riste per tranzactie
- Cum sa se protejeze impotriva pierderilor mari
- Cum sa se adapteze la diferite conditii de piata

### Cum se acceseaza?

Click pe tab-ul **HFT Engine** din bara de navigare de sus a aplicatiei. Este al 5-lea tab (dupa Chart, Scanner, News, Journal).

### Principiu de baza

Fiecare parametru din acest tab:
1. Se ajusteaza cu un **slider** (barat orizontala pe care o tragi stanga/dreapta)
2. Afiseaza valoarea curenta langa slider
3. Se **salveaza pe disk** la click pe SAVE CONFIG
4. Este **respectat strict** de bot in timpul trading-ului
5. **Supravietuieste** restartul aplicatiei

### Conventii in acest manual

- **Default** = valoarea cu care vine botul din fabrica
- **Domeniu** = intervalul minim-maxim al slider-ului
- **Exemplu** = scenarii concrete cu numere reale
- **Impact** = ce se intampla in bot cand modifici parametrul

---

## 2. EMERGENCY KILL DASHBOARD

### Ce este?

Un panel rosu, sticky (ramane vizibil in partea de sus) care ofera acces instant la actiuni de urgenta si informatii critice despre starea contului.

### Elemente

| Element | Ce arata | Cand e important |
|---------|----------|-----------------|
| **KILL ALL** | Buton rosu de urgenta | Apasa DOAR in caz de urgenta extrema |
| **BREAKER** | Starea Circuit Breaker-ului | Monitorizeaza constant |
| **EQUITY** | Valoarea totala a contului | Referinta permanenta |
| **DAILY P&L** | Profit/pierdere azi | Urmareste performanta zilnica |
| **POSITIONS** | Nr. pozitii deschise | Verifica expunerea curenta |

### Butonul KILL ALL

**Ce face:** Dupa confirmare (`cyberConfirm`), lichideaza toate pozitiile deschise si anuleaza toate ordinele active. Este echivalentul unui "opreste totul ACUM".

**Cand sa-l folosesti:**
- Piata se misca extrem de rapid impotriva ta
- Circuit Breaker-ul a ajuns in PANIC si vrei interventie manuala suplimentara
- Detectezi un comportament anormal al botului
- Vrei sa opresti complet trading-ul imediat

**Atentie:** Actiunea este IREVERSIBILA dupa confirmare. Odata confirmat, toate pozitiile se inchid la pretul curent de piata (market orders). Nu exista undo.

### Stari Circuit Breaker

| Stare | Culoare | Ce inseamna |
|-------|---------|-------------|
| **NORMAL** | Verde | Totul e OK, botul tradeaza normal |
| **CAUTION** | Galben | Pierderi moderate azi, botul devine mai conservator |
| **HALT** | Portocaliu | Pierderi semnificative, botul opreste intrarile noi |
| **PANIC** | Rosu (flash) | Pierderi grave, botul inchide tot automat |

**Exemplu:** Ai un cont de $10,000. Daca pierzi $300 azi (3%), breaker-ul trece in CAUTION. Daca pierzi $500 (5%), trece in HALT. Daca pierzi $800 (8%), trece in PANIC si inchide totul.

---

## 3. ENTRY PARAMETERS

Acesti parametri controleaza **CAND** botul deschide o pozitie noua (intra intr-un trade).

### 3.1 Signal Strength Min

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 40 |
| **Domeniu** | 10 — 95 |
| **Ce face** | Puterea minima a semnalului de intrare |

**Explicatie pentru incepatori:** Botul analizeaza piata si genereaza un "scor" de semnal (de la 0 la 100). Acest slider seteaza pragul minim: semnalele mai slabe decat aceasta valoare sunt ignorate.

**Exemplu:**
- Signal Strength Min = 40 → Botul intra cand semnalul e 40 sau mai mare
- Signal Strength Min = 70 → Botul intra DOAR cand semnalul e foarte puternic (70+)
- Signal Strength Min = 20 → Botul intra si la semnale slabe (mai multe intrari, dar calitate mai scazuta)

**Recomandari:**
- Incepatori: 50-60 (mai putine intrari, dar mai sigure)
- Scalping: 30-40 (mai multe intrari)
- Conservator: 70-80 (foarte putine, dar de calitate)

### 3.2 OFI Threshold

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 5.0 |
| **Domeniu** | 0 — 100 (afisat /10) |
| **Ce face** | Pragul minim pentru Order Flow Imbalance |

**Explicatie pentru incepatori:** OFI (Order Flow Imbalance) masoara diferenta intre ordinele de cumparare si cele de vanzare din orderbook. Un OFI pozitiv mare inseamna ca sunt mult mai multi cumparatori decat vanzatori.

**Exemplu:**
- OFI Threshold = 5.0 → Botul cere un dezechilibru moderat inainte de a intra
- OFI Threshold = 20.0 → Botul cere un dezechilibru puternic (mai putine intrari, dar confirmate de flow-ul real)
- OFI Threshold = 0 → Botul ignora complet flow-ul de ordine (nu recomandat)

### 3.3 Max Spread (bps)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 50 bps |
| **Domeniu** | 1 — 500 |
| **Ce face** | Spread-ul maxim acceptat pentru intrare |

**Explicatie pentru incepatori:** Spread-ul este diferenta intre pretul de cumparare (ask) si cel de vanzare (bid). Un spread mare inseamna ca pierzi bani din start cand intri. 1 bps = 0.01%.

**Exemplu:** Daca BTC se tranzactioneaza la $50,000:
- 50 bps = $25 spread → Botul intra daca spread-ul e sub $25
- 10 bps = $5 spread → Botul intra DOAR cu spread mic (lichiditate mare)
- 200 bps = $100 spread → Botul accepta spread-uri mari (nu recomandat pt scalping)

**Recomandari:**
- Crypto: 30-80 bps
- Stocks lichide (AAPL, TSLA): 10-30 bps
- Stocks mai putin lichide: 50-150 bps

### 3.4 Entry Cooldown (s)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 30 secunde |
| **Domeniu** | 5 — 3600 (1 ora) |
| **Ce face** | Timpul minim intre doua intrari consecutive |

**Explicatie pentru incepatori:** Dupa ce botul face o intrare, trebuie sa astepte cel putin acest numar de secunde inainte de a face alta. Previne "over-trading" (prea multe tranzactii intr-un timp scurt).

**Exemplu:**
- Cooldown = 30s → Botul poate intra maxim de 2 ori pe minut
- Cooldown = 300s (5 min) → Botul asteapta 5 minute intre intrari
- Cooldown = 5s → Botul poate intra aproape continuu (agresiv)

---

## 4. EXIT PARAMETERS

Acesti parametri controleaza **CAND** botul inchide o pozitie existenta.

### 4.1 Trailing Stop %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.5% |
| **Domeniu** | 0.1% — 20.0% |
| **Ce face** | Distanta la care trailing stop-ul urmareste pretul |

**Explicatie pentru incepatori:** Trailing stop-ul se misca IN SUS cu pretul dar NU se misca in jos. Cand pretul scade cu X% fata de maximul atins, pozitia se inchide automat.

**Exemplu cu BTC la $50,000 si Trailing Stop = 1.5%:**
1. Cumperi la $50,000
2. BTC urca la $51,000 → Trailing stop se muta la $51,000 - 1.5% = $50,235
3. BTC urca la $52,000 → Trailing stop se muta la $52,000 - 1.5% = $51,220
4. BTC scade la $51,220 → SE ACTIVEAZA, pozitia se inchide cu profit de ~$1,220

**Recomandari:**
- Scalping: 0.5% — 1.0%
- Day trading: 1.0% — 2.5%
- Swing trading: 3.0% — 5.0%

### 4.2 Take Profit %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 2.5% |
| **Domeniu** | 0.1% — 50.0% |
| **Ce face** | Procentul de profit la care pozitia se inchide automat |

**Explicatie pentru incepatori:** Cand profitul pe o pozitie atinge acest procent, botul inchide pozitia si incaseaza profitul. E un "tinta de profit".

**Exemplu cu AAPL la $200 si Take Profit = 2.5%:**
- Cumperi la $200
- Tinta: $200 + 2.5% = $205
- Cand AAPL atinge $205 → Pozitia se inchide automat cu profit de $5/actiune

**Atentie:** Trailing stop-ul si take profit-ul lucreaza IMPREUNA. Cel care se activeaza primul "castiga".

### 4.3 Max Hold Time

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 60 minute (3600s) |
| **Domeniu** | 30s — 86,400s (24 ore) |
| **Ce face** | Timpul maxim cat botul tine o pozitie deschisa |

**Explicatie pentru incepatori:** Chiar daca nici stop-ul nici take profit-ul nu s-au activat, dupa acest timp pozitia se inchide automat. Previne situatii in care o pozitie ramane "blocata" ore intregi fara miscare.

**Exemplu:**
- Max Hold = 60m → Dupa 1 ora, pozitia se inchide indiferent de profit/pierdere
- Max Hold = 5m (300s) → Potrivit pentru scalping ultra-rapid
- Max Hold = 24h (86400s) → Permite pozitii overnight

---

## 5. POSITION SIZING

Acesti parametri controleaza **CAT** risca botul per tranzactie.

### 5.1 Risk Per Trade %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 2.0% |
| **Domeniu** | 0.1% — 25.0% |
| **Ce face** | Procentul din equity pe care il risca pe un singur trade |

**Explicatie pentru incepatori:** Aceasta este pierderea maxima pe care o accepti pe o singura tranzactie, exprimata ca procent din valoarea totala a contului.

**Exemplu cu cont de $10,000 si Risk Per Trade = 2%:**
- Risc maxim per trade: $10,000 x 2% = $200
- Daca stop loss-ul este 1%, botul va calcula marimea pozitiei astfel incat pierderea maxima sa fie $200
- Marimea pozitiei: $200 / 1% = $20,000 notional

**Regula clasica:** Majoritatea traderilor profesionisti risca intre 1-2% per trade. Peste 5% este considerat foarte riscant.

### 5.2 Max Open Positions

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1 |
| **Domeniu** | 1 — 20 |
| **Ce face** | Numarul maxim de pozitii deschise simultan |

**Exemplu:**
- Max Open = 1 → Botul tine un singur trade activ (cel mai sigur)
- Max Open = 5 → Botul poate avea 5 trade-uri deschise simultan
- Max Open = 10 → Risc mare, 10 pozitii active (necesita equity mare)

**Atentie:** Cu Max Open = 5 si Risk = 2%, riscul total poate fi 5 x 2% = 10% din cont simultan.

### 5.3 Max Daily Trades

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 50 |
| **Domeniu** | 1 — 500 |
| **Ce face** | Numarul maxim de tranzactii pe zi |

**Exemplu:**
- Max Daily = 50 → Botul face maxim 50 trade-uri pe zi (tipic pt HFT)
- Max Daily = 10 → Limitat, potrivit pt day trading conservator
- Max Daily = 200 → Foarte agresiv, scalping intens

---

## 6. DISCORD ALERTS

### Ce este?

Sectiunea Discord Alerts trimite notificari pe un server Discord cand botul face actiuni importante.

### Cum se configureaza (pas cu pas)

1. **Creeaza un server Discord** (sau foloseste unul existent)
2. **Creeaza un Webhook:**
   - Pe server → Settings → Integrations → Webhooks → New Webhook
   - Copiaza URL-ul webhook-ului
3. **Lipeste URL-ul** in campul "Webhook URL" din HFT Engine
4. **Bifeaza** alertele dorite:
   - **Alert on Trade Fill** — notificare la fiecare tranzactie executata
   - **Alert on Circuit Breaker** — notificare cand breaker-ul isi schimba starea
   - **Alert Daily Summary** — rezumat la sfarsitul zilei
5. **Click TEST DISCORD** — verifica ca webhook-ul functioneaza

---

## 7. TELEGRAM ALERTS

### Ce este?

Similar cu Discord, dar pe Telegram. Primesti notificari pe telefonul mobil.

### Cum se configureaza (pas cu pas)

1. **Creeaza un Bot Telegram:**
   - Deschide Telegram, cauta `@BotFather`
   - Trimite `/newbot`, urmeaza instructiunile
   - Primesti un **Bot Token** (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)
2. **Afla Chat ID-ul tau:**
   - Trimite un mesaj botului tau
   - Acceseaza: `https://api.telegram.org/bot<TOKEN>/getUpdates`
   - Gaseste `chat.id` in raspuns
3. **Introdu Bot Token** in campul corespunzator
4. **Introdu Chat ID** in campul corespunzator
5. **Bifeaza "Enable Telegram Alerts"**
6. **Click TEST TELEGRAM** — verifica ca primesti mesaj

---

## 8. SAFETY LIMITS

### 8.1 Daily Loss Limit %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 3.0% |
| **Domeniu** | 0.5% — 20.0% |
| **Ce face** | Pierderea maxima zilnica acceptata |

**Explicatie pentru incepatori:** Cand pierderea totala pe ziua curenta atinge acest procent din equity, botul OPRESTE complet trading-ul pentru restul zilei.

**Exemplu cu cont de $10,000 si Daily Loss = 3%:**
- Limita zilnica: $10,000 x 3% = $300
- Daca pierzi $300 in total astazi → Botul se opreste pana maine
- Te protejeaza impotriva zilelor "negre" care pot distruge contul

**Recomandari:**
- Conservator: 2-3%
- Moderat: 3-5%
- Agresiv: 5-10% (nu recomandat pentru incepatori)

---

## 9. CORRELATION MATRIX

### Ce este?

Matricea de corelatie arata cat de mult se misca impreuna asset-urile pe care le tranzactionezi.

### Elemente

**Max Corr (slider)**
| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.70 |
| **Domeniu** | 0.00 — 1.00 |
| **Ce face** | Corelatia maxima permisa intre asset-urile detinute |

**Explicatie:** Daca doua asset-uri au corelatie 0.95, se misca aproape identic. A le tine pe ambele este ca si cum ai dubla pozitia pe unul singur. Slider-ul limiteaza aceasta corelatie.

**Exemplu:**
- AAPL si MSFT au corelatie ~0.85
- Cu Max Corr = 0.70, botul NU va tine pozitii simultane pe AAPL si MSFT
- Cu Max Corr = 0.90, botul le permite

**Butonul REFRESH:** Recalculeaza matricea de corelatie cu datele actuale.

---

## 10. PORTFOLIO PERFORMANCE

### Ce este?

Panelul Portfolio Performance arata performanta fiecarui asset din portofoliu.

### Elemente

- **REFRESH** — Reincarca datele de performanta
- **REBALANCE** — Rebalanseaza portofoliul conform ponderilor tinta

**Ce arata tabelul:**
- Asset-ul (simbol)
- Procentul din portofoliu
- Performanta (profit/pierdere)
- Volatilitatea

---

## 11. ATR DYNAMIC STOPS

ATR (Average True Range) masoara volatilitatea reala a unui asset. Acesti parametri seteaza stop-uri si take-profit-uri bazate pe volatilitate, nu pe procente fixe.

### 11.1 Stop-Loss (x ATR)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.5x |
| **Domeniu** | 0.5x — 5.0x |
| **Ce face** | Stop loss-ul ca multiplu al ATR |

**Exemplu:** Daca ATR-ul lui AAPL pe 14 perioade este $3:
- Stop = 1.5 x $3 = $4.50 sub pretul de intrare
- Intrare la $200 → Stop la $195.50

### 11.2 Take-Profit (x ATR)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 3.0x |
| **Domeniu** | 1.0x — 10.0x |
| **Ce face** | Take profit ca multiplu al ATR |

**Exemplu:** Cu ATR = $3:
- Take = 3.0 x $3 = $9 deasupra pretului de intrare
- Intrare la $200 → Take profit la $209

### 11.3 Partial Take %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 50% |
| **Domeniu** | 25% — 75% |
| **Ce face** | Cat din pozitie se inchide la primul target de profit |

**Explicatie:** In loc sa inchizi toata pozitia la take profit, inchizi doar o parte (ex: 50%) si lasi restul sa curga cu trailing stop.

**Exemplu:** Ai 100 actiuni AAPL. Cu Partial = 50%:
1. La primul target: vinzi 50 actiuni, incasezi profit partial
2. Celelalte 50 continua cu trailing stop
3. Daca pretul continua sa urce, castigi si pe a doua jumatate

### 11.4 Partial Profit Taking (checkbox)

Activeaza/dezactiveaza luarea partiala de profit. Cand e dezactivat, toata pozitia se inchide la take profit.

---

## 12. LLM SENTIMENT

### Ce este?

Analiza de sentiment bazata pe AI (Large Language Models) care proceseaza stiri si genereaza un scor de sentiment pentru asset-urile tranzactionate.

### Elemente

| Element | Ce face |
|---------|---------|
| **API Key** | Cheia API de la OpenAI sau Anthropic |
| **Provider** | OpenAI (GPT-4o-mini) sau Anthropic (Claude Haiku) |
| **Enable** | Activeaza/dezactiveaza analiza de sentiment |

**Cum functioneaza:** Botul trimite stiri recente despre asset la LLM si primeste un scor de sentiment (bullish/bearish/neutral) pe care il include in decizia de trading.

**Cost:** Folosirea LLM-urilor implica costuri pe API call. GPT-4o-mini este cel mai ieftin.

---

## 13. OPTIONS HEDGING

### Ce este?

Hedging-ul cu optiuni este o strategie de protectie: cumperi optiuni put (care cresc in valoare cand pretul scade) pentru a proteja pozitiile long.

### Elemente

### 13.1 OTM %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 5% |
| **Domeniu** | 1% — 20% |
| **Ce face** | Cat de "out of the money" sa fie optiunile cumparate |

**Exemplu:** Ai o pozitie long pe AAPL la $200. Cu OTM = 5%:
- Botul cumpara put cu strike $200 - 5% = $190
- Daca AAPL scade sub $190, put-ul te protejeaza

### 13.2 Max Premium %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.0% |
| **Domeniu** | 0.1% — 5.0% |
| **Ce face** | Cat din valoarea pozitiei se cheltuie pe prima optiunii |

### 13.3 Auto-Hedge (checkbox)

Activeaza hedging-ul automat. Cand este ON, botul cumpara automat optiuni put pentru pozitiile long.

### HEDGE ALL (buton)

Aplica hedging imediat pe TOATE pozitiile deschise.

---

## 14. WEB DASHBOARD

### Ce este?

Un server web local care iti permite sa monitorizezi botul din browser. Asculta exclusiv pe `127.0.0.1` (localhost). Include pagina HTML cyber-HUD cu status live, API endpoints JSON, si WebSocket pentru push updates in timp real.

### Elemente

| Element | Ce face | Default |
|---------|---------|---------|
| **Enable** | Porneste/opreste serverul web | OFF |
| **Port** | Portul pe care ruleaza (1024-65535) | 7777 |
| **URL** | Adresa de acces | http://127.0.0.1:7777 |
| **COPY URL** | Copiaza URL-ul in clipboard | — |
| **Token** | Token de autentificare (generat automat) | Random 64-char hex |
| **Rotate Token** | Regenereaza token-ul instant (comanda `rotate_dashboard_token`) | — |

**Cum se foloseste:**
1. Bifeaza "Enable"
2. Deschide browser pe `http://127.0.0.1:7777`
3. Pagina se autentifica automat cu token-ul embedded

### Securitate

**Autentificare (unificata S02, extins S72):** Toate endpoint-urile HTTP folosesc header `Authorization: Bearer <token>`. WebSocket endpoint-ul `/ws` accepta token atat in header cat si via query parameter `?token=...` (necesar deoarece browser WebSocket API nu suporta custom headers). Token-ul se gaseste in settings. Fara token valid, primesti 401 Unauthorized. Singura exceptie: `/api/health` nu necesita autentificare.

**CORS (Cross-Origin Resource Security):**
- Permite doar `http://127.0.0.1:<port>` si `http://localhost:<port>` (include portul)
- Un site malitios deschis in browser nu poate face cereri la dashboard
- Methods permise: GET, POST
- Headers permise: Authorization, Content-Type

**Rate Limiting:**
- Maximum 60 cereri per minut pe toate rutele protejate (inclusiv WS upgrade)
- La depasire: HTTP 429 (Too Many Requests)
- WS message rate limit: max 10 mesaje client per tick (2s), disconnect la depasire
- Previne abuz accidental sau intentionat

**Auth Failure Logging:**
- Fiecare esec de autentificare este logat (warn)
- La fiecare 5 esecuri consecutive: log error cu alerta brute-force
- Contorul se reseteaza la autentificare reusita

**Token Rotation:**
- Comanda Tauri `rotate_dashboard_token` genereaza un token nou random
- Salveaza pe disk automat
- Util daca token-ul e compromis — fara restart necesar

### WebSocket (Real-Time Push)

Endpoint: `/ws` (autentificare Bearer pe HTTP upgrade request)

| Tip mesaj | Frecventa | Ce contine |
|-----------|-----------|------------|
| `status` | La 2 secunde | equity, positions[], breaker, latency p50/p95 |
| `trade` | La fiecare trade | action, symbol, side, pnl, duration_secs |
| `alert` | La watchdog recovery | message |

Client-ul JavaScript are auto-reconnect cu exponential backoff (1s → 30s).

### API Endpoints

| Endpoint | Metoda | Auth | Ce returneaza |
|----------|--------|------|---------------|
| `/api/health` | GET | Nu | `{"status":"ok","uptime_secs":N,"version":"..."}` |
| `/api/build-info` | GET | Nu | `{"version":"...","git_hash":"...","build_date":"...","uptime_secs":N}` (S04) |
| `/` | GET | Bearer | Pagina HTML dashboard (cyber-HUD) cu build tags |
| `/api/status` | GET | Bearer | Equity, autopilot, breaker, strategy |
| `/api/positions` | GET | Bearer | Lista pozitii deschise |
| `/api/trades` | GET | Bearer | Ultimele 20 trade-uri |
| `/api/stats` | GET | Bearer | Win rate, PnL total, profit factor |
| `/api/autopilot` | POST | Bearer | Toggle autopilot ON/OFF |
| `/ws` | GET | Bearer | WebSocket live push |

---

## 15. AUTOPILOT CORE

Acesti 10 parametri controleaza motorul fundamental al autopilot-ului — bucla principala de trading.

### 15.1 Min Hold (s)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 180s (3 min) |
| **Domeniu** | 0 — 3600s |
| **Ce face** | Timpul minim cat trebuie tinuta o pozitie |

**Explicatie:** Dupa intrare, botul nu va iesi din pozitie cel putin acest numar de secunde, chiar daca primeste semnal de iesire. Previne iesiri premature pe zgomot.

**Exemplu:** Min Hold = 180s → Dupa ce cumperi, botul asteapta cel putin 3 minute inainte de a lua in considerare iesirea.

### 15.2 Consensus %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 70% |
| **Domeniu** | 30% — 100% |
| **Ce face** | Procentul minim de agenti GPU Nexus care trebuie sa fie de acord |

**Explicatie:** GPU Nexus ruleaza 150,000 de agenti de simulare. Consensus % seteaza cat de multi trebuie sa fie de acord ca un trade e bun. 70% inseamna ca cel putin 105,000 agenti trebuie sa confirme.

### 15.3 Loop (ms)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1500ms |
| **Domeniu** | 500 — 10,000ms |
| **Ce face** | Cat de des ruleaza bucla de autopilot |

**Exemplu:** Loop = 1500ms → Botul verifica piata de 0.67 ori pe secunda. Loop = 500ms → Verifica de 2 ori pe secunda (mai rapid, dar consuma mai multe resurse).

### 15.4 OFI Stale (ms)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 10,000ms (10s) |
| **Domeniu** | 1,000 — 60,000ms |
| **Ce face** | Dupa cate milisecunde datele OFI sunt considerate "vechi" |

**Explicatie:** Daca ultimul OFI a fost primit acum mai mult de X ms, botul il ignora si nu intra pe baza lui.

### 15.5 Min Bars

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 20 |
| **Domeniu** | 5 — 200 |
| **Ce face** | Numarul minim de candele (bare) necesare inainte de prima intrare |

**Explicatie:** Botul asteapta sa aiba cel putin 20 de bare de pret inainte de a face prima intrare pe un asset. Indicatorii tehnici (EMA, RSI, MACD) au nevoie de date suficiente pentru a calcula corect.

### 15.6 OFI Entry Thr.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 5.0 |
| **Domeniu** | 0 — 50 (afisat /10) |
| **Ce face** | Pragul OFI specific pentru semnalul de intrare |

**Diferenta fata de OFI Threshold (Entry Parameters):** Acesta este pragul intern al autopilot-ului, nu cel din modulul de intrare. Ambele trebuie satisfacute.

### 15.7 Max Qty Cap

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1,000 |
| **Domeniu** | 1 — 100,000 |
| **Ce face** | Cantitatea maxima de unitati (shares/coins) per order |

**Exemplu:** Max Qty = 1000 → Botul nu va cumpara mai mult de 1000 actiuni intr-un singur ordin, indiferent de cat indica position sizing.

### 15.8 Min Qty Thr.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.001 |
| **Domeniu** | 0.001 — varies |
| **Ce face** | Cantitatea minima per order (sub aceasta, ordinul nu se trimite) |

**Explicatie:** Unele asset-uri (mai ales crypto) au loturi foarte mici. Daca position sizing calculeaza o cantitate sub pragul minim, ordinul nu se trimite.

### 15.9 Equity Fallback

| Proprietate | Valoare |
|-------------|---------|
| **Default** | $10,000 |
| **Domeniu** | $1,000 — $1,000,000 |
| **Ce face** | Valoarea equity folosita daca API-ul Alpaca nu raspunde |

**Explicatie:** Daca botul nu poate citi equity-ul real din cont (timeout de retea, eroare API), foloseste aceasta valoare ca referinta temporara. Previne ordine cu dimensiuni aberante.

### 15.10 Max Bypass Risk %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 25% |
| **Domeniu** | 1% — 100% |
| **Ce face** | Riscul maxim permis in modul bypass (manual override) |

### 15.11 Generation-Aware Loop (S03)

Autopilot captureaza `session_generation` la pornire. La fiecare tick, verifica daca generatia curenta coincide. Daca nu (ex: dupa un respawn de watchdog), loop-ul vechi iese automat:

```
"Generation mismatch — exiting stale autopilot instance"
```

Aceasta previne **split-brain** operational: doua instante de autopilot care tradeaza simultan dupa un respawn. Aceeasi protectie se aplica si pe `trade_updates` si `streamer`.

**Generation-critical loops:** streamer, autopilot, trade_updates. Generatia se incrementeaza automat de `spawn_loop_by_name` doar pentru aceste trei.

---

## 16. EXIT SCALING

Acesti 17 parametri controleaza modul in care se calculeaza stop-loss, take-profit si trailing stop in functie de volatilitate si modul de trading.

### 16.1 Vol Factor Min / Max

| Parametru | Default | Domeniu |
|-----------|---------|---------|
| Vol Factor Min | 0.6 | 0.1 — 2.0 |
| Vol Factor Max | 2.5 | 1.0 — 10.0 |

**Ce fac:** Limiteaza factorul de volatilitate care ajusteaza stop-urile. Daca volatilitatea e foarte mica, factorul nu scade sub Min; daca e foarte mare, nu depaseste Max.

**Exemplu:** Cu Vol Min = 0.6 si trail de 1.5%:
- Volatilitate scazuta: trailing ajustat = 1.5% x 0.6 = 0.9% (nu mai stransa de atat)
- Volatilitate normala: trailing = 1.5% x 1.0 = 1.5%
- Volatilitate mare: trailing ajustat = 1.5% x 2.5 = 3.75% (nu mai larga de atat)

### 16.2 Fallback SL / TP / Trail %

| Parametru | Default |
|-----------|---------|
| Fallback SL % | 2.5% |
| Fallback TP % | 2.5% |
| Fallback Trail % | 1.5% |

**Ce fac:** Valori de rezerva folosite cand calculul dinamic (bazat pe ATR/volatilitate) esueaza sau returneaza valori invalide.

### 16.3 Scalp Stop / Take / Trail Mult

| Parametru | Default |
|-----------|---------|
| Scalp Stop Mult | 0.70 |
| Scalp Take Mult | 0.60 |
| Scalp Trail Mult | 0.50 |

**Ce fac:** In modul scalping, stop/take/trail sunt INMULTITE cu acesti multiplicatori, facandu-le mai stranse.

**Exemplu:** Cu trail de 1.5% si Scalp Trail Mult = 0.50:
- Mod normal: trail = 1.5%
- Mod scalping: trail = 1.5% x 0.50 = 0.75%

### 16.4 Partial 1st Target / Min Qty

| Parametru | Default | Ce face |
|-----------|---------|---------|
| Partial 1st Target | 0.5 (50%) | Fractiunea de pozitie inchisa la primul target |
| Partial Min Qty | 0.001 | Cantitatea minima ramasa dupa partial close |

### 16.5 ATR Period

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 14 |
| **Domeniu** | 2 — 100 |
| **Ce face** | Numarul de perioade pentru calculul ATR |

**Explicatie:** Un ATR cu 14 perioade foloseste ultimele 14 candele pentru a calcula volatilitatea medie. Mai mare = mai smooth, mai mic = mai reactiv.

### 16.6 ATR Stop/Take/Trail Clamp Min/Max

| Parametru | Default Min | Default Max |
|-----------|-------------|-------------|
| ATR Stop Clamp | 0.5 | 8.0 |
| ATR Take Clamp | 1.0 | 15.0 |
| ATR Trail Clamp | 0.3 | 5.0 |

**Ce fac:** Limiteaza valorile ATR-based stop/take/trail la un interval rezonabil. Previne valori extreme cand volatilitatea explodeaza.

---

## 17. ORACLE EXIT

Acesti 6 parametri controleaza iesirile bazate pe Oracle AI (sistemul de inteligenta artificiala al botului).

### 17.1 Exit Min Conf.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.40 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Confidenta minima a Oracle-ului pentru a declansa iesire |

**Exemplu:** Oracle-ul analizeaza piata si zice "sunt 60% sigur ca pretul va scadea". Cu Exit Min Conf = 0.40, aceasta confidenta (0.60) depaseste pragul, deci botul iese.

### 17.2 Exit Min Score

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 30 |
| **Domeniu** | 0 — 100 |
| **Ce face** | Scorul minim al semnalului de iesire Oracle |

### 17.3 Flip Score Thr.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 10 |
| **Domeniu** | 0 — 100 |
| **Ce face** | Scorul la care Oracle-ul considera ca piata s-a "intors" (flip) |

**Exemplu:** Esti long (ai cumparat). Oracle-ul detecteaza un scor de sell = 15, cu Flip Thr = 10. Scorul depaseste pragul → Botul considera ca piata s-a intors si iese.

### 17.4 Flip Confidence

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.20 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Confidenta minima pentru a confirma flip-ul |

### 17.5 Chaos Max Loss %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -0.5% |
| **Domeniu** | -10.0% — 0% |
| **Ce face** | Pierderea maxima acceptata in regim haotic |

**Explicatie:** Cand Oracle-ul detecteaza ca piata este in regim CHAOTIC (haos), aplica un stop mult mai strans. Daca pierderea atinge acest procent, iese imediat.

### 17.6 Chaos Min Conf.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.25 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Confidenta minima a Oracle-ului pentru a declara regim haotic |

---

## 18. CIRCUIT BREAKER

Sistemul de protectie al contului. **Cel mai important mecanism de siguranta.**

### 18.1 Caution %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -3.0% |
| **Domeniu** | -20% — 0% |
| **Ce face** | Pragul de pierdere zilnica pentru modul CAUTION |

**Exemplu cu cont $10,000:** Cand pierzi $300 (3%) intr-o zi → CAUTION se activeaza. Botul devine mai conservator (nu intra pe semnale slabe).

### 18.2 Halt %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -5.0% |
| **Domeniu** | -30% — 0% |
| **Ce face** | Pragul de pierdere zilnica pentru modul HALT |

**Exemplu:** Pierzi $500 (5%) → HALT. Botul OPRESTE complet intrarile noi. Pozitiile existente continua sa fie gestionate (stop/trail).

### 18.3 Panic %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -8.0% |
| **Domeniu** | -50% — 0% |
| **Ce face** | Pragul de pierdere zilnica pentru modul PANIC |

**Exemplu:** Pierzi $800 (8%) → PANIC. Botul INCHIDE TOATE pozitiile si opreste trading-ul complet pentru ziua curenta.

### 18.4 Max Notional Frac

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.25 (25%) |
| **Domeniu** | 0.01 — 1.00 |
| **Ce face** | Fractiunea maxima din equity pe care o pozitie singura o poate reprezenta |

**Exemplu cu cont $10,000:** Max Notional = 0.25 → O singura pozitie nu poate depasi $2,500 valoare notionala.

### 18.5 Min Stop Frac

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.0001 (0.01%) |
| **Domeniu** | 0.0001 — 0.01 |
| **Ce face** | Distanta minima a stop-ului ca fractiune din pret |

**Explicatie:** Previne stop-uri prea stranse care s-ar activa pe zgomot. Cu 0.0001 pe AAPL la $200, stop-ul minim e la $200 x 0.0001 = $0.02 distanta.

### 18.6 Flash Crash Detector

Sistemul Flash Crash Detector monitorizeaza miscarea pretului in timp real si actioneaza automat cand detecteaza dislocatii anormale ale pietei.

**Diferenta fata de Circuit Breaker:** Circuit Breaker-ul masoara PnL-ul tau (pierderi din pozitii). Flash Crash Detector-ul masoara miscarea pretului pietei — poate detecta un crash inainte ca stop-ul tau sa fie lovit.

#### 18.6.1 Flash Crash Window (secunde)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 30 secunde |
| **Domeniu** | 5 — 120 |
| **Ce face** | Fereastra de timp in care se monitorizeaza miscarea pretului |

**Explicatie:** Botul tine evidenta tuturor preturilor primite in ultimele N secunde. Daca diferenta intre pretul maxim si minim din aceasta fereastra depaseste threshold-ul, se declara flash crash.

**Exemplu:** Window = 30s, AAPL tradeaza la $200. In ultimele 30 secunde, pretul a fost intre $195 si $200 → variatie = (200-195)/195 × 100 = 2.56%. Daca threshold-ul e 3%, nu se activeaza.

#### 18.6.2 Flash Crash Threshold (%)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 3.0% |
| **Domeniu** | 0.5 — 20.0 |
| **Ce face** | Procentul de miscare a pretului care declanseaza protectia |

**Explicatie:** Daca pretul se misca cu mai mult de X% in fereastra de monitorizare (in orice directie — sus sau jos), botul considera ca e o dislocatie anormala si activeaza protectia.

**Recomandari:**
- **Actiuni large-cap** (AAPL, MSFT, NVDA): 2-3% (foarte rar se misca atat in 30s)
- **Actiuni mid-cap volatile**: 4-5%
- **Penny stocks / crypto**: 8-15% (volatilitate normala mai mare)

#### 18.6.3 Flash Crash Cooldown (minute)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 5 minute |
| **Domeniu** | 1 — 60 |
| **Ce face** | Cat timp blocheaza intrari noi pe simbolul afectat dupa un flash crash |

**Explicatie:** Dupa ce detecteaza un flash crash pe un simbol, botul blocheaza orice intrare noua pe acel simbol timp de N minute. Previne reintrarea intr-o piata instabila.

#### Ce se intampla cand Flash Crash Detector se activeaza?

5 actiuni automate, in ordine:

1. **Log** — inregistrare detaliata in log cu component="flash_crash"
2. **Frontend event** — emit "flash-crash" catre UI (toast/alerta vizuala)
3. **Auto-close** — inchide automat pozitia deschisa pe simbolul afectat
4. **Telegram alert** — trimite notificare pe Telegram (daca este activat)
5. **Block** — blocheaza intrari noi pe acel simbol timp de `cooldown` minute

### 18.7 Drawdown Velocity Detection

Sistemul Drawdown Velocity Detection monitorizeaza **viteza** cu care drawdown-ul creste si poate bloca intrarile INAINTE ca Circuit Breaker-ul sa se activeze.

**Diferenta fata de Circuit Breaker:** Circuit Breaker-ul masoara **cat de mult** ai pierdut (magnitudine). Drawdown Velocity masoara **cat de repede** pierzi (viteza). Poate detecta cascading losses in desfasurare cu minute inainte ca threshold-ul absolut sa fie atins.

**Diferenta fata de Flash Crash Detector:** Flash Crash-ul masoara miscarea pretului **pietei**. Drawdown Velocity masoara viteza pierderii **tale** (a equity-ului tau).

#### 18.7.1 DD Velocity Threshold (%/min)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -1.0 %/min |
| **Domeniu** | -10.0 ... -0.1 |
| **Ce face** | Viteza maxima de pierdere tolerata, in procent din peak equity pe minut |

**Explicatie:** Daca drawdown-ul creste cu mai mult de X% pe minut (pe fereastra de lookback), botul blocheaza toate intrarile noi. Valoarea e negativa prin conventie (pierderea e negativa).

**Exemple:**
- **-1.0** (default): Blocheaza daca pierzi 1% din peak equity pe minut sau mai mult
- **-0.5** (agresiv/protectiv): Blocheaza la 0.5%/min — mai sensibil, mai putine intrari in perioadele volatile
- **-2.0** (relaxat): Permite pierderi mai rapide inainte de halt — potrivit pentru strategii cu drawdown natural mai mare

**Recomandari:**
- **Scalping conservator**: -0.5 (reactie rapida la pierderi)
- **Day trading normal**: -1.0 (default, balans intre protectie si flexibilitate)
- **Swing/agresiv**: -2.0 (permite drawdown mai mare inainte de interventie)

#### 18.7.2 DD Velocity Window (secunde)

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 300 secunde (5 minute) |
| **Domeniu** | 30 — 600 |
| **Ce face** | Fereastra de timp pe care se calculeaza viteza drawdown-ului |

**Explicatie:** Velocitatea se calculeaza comparand drawdown-ul curent cu cel de acum N secunde. Fereastra mai scurta = reactie mai rapida dar mai sensibila la spike-uri scurte. Fereastra mai lunga = mai stabila dar reactie mai lenta.

**Recomandari:**
- **60-120s**: Reactie foarte rapida, potrivita pentru scalping pe 1m
- **300s** (default): Balans bun, filtreaza noise-ul scurt
- **600s**: Reactie lenta, doar cascading losses sustinute activeaza halt-ul

#### Ce se intampla cand DD Velocity Halt se activeaza?

1. **Block entries** — toate intrarile noi sunt blocate (independent de Circuit Breaker)
2. **Log** — mesaj periodic in UI: "DD VELOCITY HALT — Drawdown speed X.XX%/min exceeds threshold"
3. **Recovery automata** — cand viteza scade sub threshold, intrarile sunt re-activate automat (nu necesita reset manual ca la Circuit Breaker)
4. **Pozitiile existente** nu sunt afectate — se aplica doar intrarilor noi

**Vizibil in API:** `get_risk_status` returneaza campul `dd_velocity` (f64). Valoare negativa = drawdown in crestere. Valoare aproape de 0 = stabil.

### 18.8 Sincronizare Breaker Global cu SmartTrend (S69)

De la Sesiunea 69, `reset_circuit_breaker` reseteaza si starea SmartTrend: `st_account.breaker_triggered`, `st_account.paused`, `settings.st_bot_paused`. Daily reset face la fel cand breaker-ul era activ. Badge `BRK:SYNC` in ST SHIELD confirma vizual ca cele doua sisteme sunt sincronizate.

---

## 19. STRATEGY SCORING

Acesti 9 parametri controleaza motorul de scoring al strategiei — cum botul evalueaza calitatea fiecarui semnal de intrare.

### 19.1 Base Score

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 50 |
| **Domeniu** | 0 — 100 |
| **Ce face** | Scorul de baza de la care incepe evaluarea |

**Explicatie:** Strategia incepe cu acest scor si adauga/scade puncte pe baza indicatorilor. Un scor final mare (ex: 75+) indica un semnal bun de intrare.

### 19.2 VPIN Gate Scalp / Normal

| Parametru | Default |
|-----------|---------|
| VPIN Gate Scalp | 0.85 |
| VPIN Gate Normal | 0.75 |

**Ce fac:** VPIN (Volume-Synchronized PIN) masoara toxicitatea flow-ului. Daca VPIN depaseste pragul, botul NU intra — piata e prea "toxica" (probabil insider trading sau flow adversar).

### 19.3 Min RVOL

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.5 |
| **Domeniu** | 0 — 5.0 |
| **Ce face** | Volumul relativ minim necesar |

**Explicatie:** RVOL = volumul actual / media istorica. RVOL = 0.5 inseamna ca volumul trebuie sa fie cel putin 50% din medie. RVOL = 2.0 ar cere volum dublu fata de medie.

### 19.4 RVOL Window

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 20 |
| **Domeniu** | 5 — 200 |
| **Ce face** | Cate perioade se folosesc pt media volumului |

### 19.5 RVOL Relax Thr. / Val.

| Parametru | Default | Ce face |
|-----------|---------|---------|
| RVOL Relax Thr. | 3.0 | Daca RVOL depaseste aceasta valoare, se "relaxeaza" filtrul |
| RVOL Relax Val. | 5.0 | Valoarea de inlocuire cand se relaxeaza |

**Explicatie:** Cand volumul e FOARTE mare (de 3x fata de medie), e probabil un eveniment major si filtrul RVOL normal ar fi prea restrictiv.

### 19.6 Chaos Conf Kill / Chaos Regime Mult

| Parametru | Default | Ce face |
|-----------|---------|---------|
| Chaos Conf Kill | 0.10 | Confidenta sub care scorul se anuleaza in chaos |
| Chaos Regime Mult | 0.55 | Multiplicatorul aplicat contributiei Oracle in regim chaotic |

---

## 20. ENGINE STOPS

Acesti 6 parametri controleaza stop-urile si trailing-ul din motorul de trading (functia `manage_positions`).

### 20.1 Long Stop Normal %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -2.5% |
| **Domeniu** | -20% — 0% |
| **Ce face** | Stop loss fix pentru pozitii long in modul normal |

### 20.2 Long Stop Aggr %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -8.0% |
| **Domeniu** | -30% — 0% |
| **Ce face** | Stop loss fix pentru pozitii long in modul agresiv |

**Diferenta:** Modul agresiv permite stop-uri mai largi pentru a da spatiu pozitiei sa respire.

### 20.3 Short Stop %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | -5.0% |
| **Domeniu** | -20% — 0% |
| **Ce face** | Stop loss fix pentru pozitii short |

### 20.4 Long Trail Min Profit %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.5% |
| **Domeniu** | 0.1% — 10% |
| **Ce face** | Profitul minim necesar inainte de activarea trailing stop pe long |

**Exemplu:** Cu Min Profit = 1.5% si pozitie long pe AAPL la $200:
- Trail-ul se activeaza DOAR dupa ce AAPL atinge $203 (+1.5%)
- Sub $203, se aplica stop-ul fix normal

### 20.5 Short Trail Gap %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.5% |
| **Domeniu** | 0.1% — 10% |
| **Ce face** | Distanta trailing stop pe pozitii short |

### 20.6 Short Trail Min Profit %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 1.0% |
| **Domeniu** | 0.1% — 10% |
| **Ce face** | Profitul minim necesar pt activarea trailing stop pe short |

---

## 21. ORACLE FUSION

Acesti 8 parametri controleaza ponderile si pragurile din sistemul Oracle AI cu 7 layere.

### 21.1 Kalman W / Entropy W / Hurst W / VPIN W / Wavelet W

| Parametru | Default | Ce masoara |
|-----------|---------|------------|
| Kalman W | 0.30 | Filtrul Kalman — estimarea directiei pretului |
| Entropy W | 0.25 | Entropia — gradul de dezordine al pietei |
| Hurst W | 0.20 | Exponentul Hurst — persistenta trendului |
| VPIN W | 0.15 | VPIN — toxicitatea flow-ului |
| Wavelet W | 0.10 | Wavelet — descompunerea frecventelor pretului |

**Explicatie:** Oracle-ul combina 5 metrici in un scor final. Ponderile (W) determina cat de mult conteaza fiecare metrica.

**Exemplu:** Cu Kalman W = 0.30 si Entropy W = 0.25, directia pretului (Kalman) conteaza mai mult decat dezordinea pietei (Entropy).

**Recomandare:** Suma ponderilor ar trebui sa fie aproape de 1.00 (nu obligatoriu, dar logic). Default: 0.30 + 0.25 + 0.20 + 0.15 + 0.10 = 1.00.

### 21.2 Conf Floor

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.25 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Confidenta minima a Oracle-ului sub care semnalul e ignorat |

### 21.3 Entropy Chaos Thr.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.85 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Pragul de entropie peste care piata e declarata CHAOTIC |

### 21.4 VPIN Toxicity Alert

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.60 |
| **Domeniu** | 0 — 1.00 |
| **Ce face** | Pragul VPIN peste care se emite alerta de toxicitate |

---

## 22. INDICATOR PERIODS

Acesti 12 parametri seteaza perioadele indicatorilor tehnici clasici. Modificarea lor afecteaza DIRECT semnalele BUY/SELL de pe grafic.

### 22.1 EMA Fast / Mid / Slow / Trend

| Parametru | Default | Ce face |
|-----------|---------|---------|
| EMA Fast | 9 | Media mobila exponentiala rapida (reactie rapida la pret) |
| EMA Mid | 20 | Media mobila medie |
| EMA Slow | 50 | Media mobila lenta (trend pe termen mediu) |
| EMA Trend | 200 | Media mobila de trend (trend pe termen lung) |

**Cum functioneaza crossover-ul:**
- EMA Fast trece PESTE EMA Slow = semnal BUY
- EMA Fast trece SUB EMA Slow = semnal SELL

**Exemplu:** Cu EMA Fast = 9 si EMA Slow = 50:
- Semnale mai frecvente dar cu mai mult zgomot
- Cu EMA Fast = 20 si EMA Slow = 100:
- Semnale mai rare dar mai fiabile

### 22.2 RSI Period

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 14 |
| **Domeniu** | 2 — 100 |
| **Ce face** | Perioada Relative Strength Index |

**Explicatie:** RSI masoara viteza si magnitudinea miscarilor de pret. RSI < 30 = oversold (potential cumparare), RSI > 70 = overbought (potential vanzare).

### 22.3 MACD Fast / Slow / Signal

| Parametru | Default |
|-----------|---------|
| MACD Fast | 12 |
| MACD Slow | 26 |
| MACD Signal | 9 |

**Explicatie:** MACD = diferenta intre doua EMA. Signal line = EMA a MACD. Crossover MACD peste Signal = bullish, sub = bearish.

### 22.4 BB Period / BB Std Dev

| Parametru | Default | Ce face |
|-----------|---------|---------|
| BB Period | 20 | Perioada Bollinger Bands |
| BB Std Dev | 2.0 | Numarul de deviatii standard |

**Explicatie:** Bollinger Bands = media mobila +/- N deviatii standard. Pretul sub banda inferioara = potential oversold, peste banda superioara = potential overbought.

### 22.5 ADX Period

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 14 |
| **Domeniu** | 2 — 100 |
| **Ce face** | Perioada Average Directional Index |

**Explicatie:** ADX masoara puterea trendului (nu directia). ADX > 25 = trend puternic, ADX < 20 = piata laterala.

### 22.6 StochRSI Period

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 14 |
| **Domeniu** | 2 — 100 |
| **Ce face** | Perioada Stochastic RSI |

**Explicatie:** StochRSI aplica formula Stochastic pe RSI, generand un oscilator mai sensibil (0-1). Sub 0.2 = oversold, peste 0.8 = overbought.

---

## 23. ADAPTIVE EXIT MULTIPLIERS

Aceasta este sectiunea cea mai avansata din HFT Engine. Controleaza cum botul **ajusteaza automat** stop/take/trail in functie de regimul de piata, alinierea semnalelor si timeframe-ul.

### Cum functioneaza cascadarea

Valorile finale ale stop/take/trail se calculeaza astfel:

```
Valoare efectiva = Valoare baza x Regime Mult x Signal Mult x TF Mult
```

**Exemplu complet:**
- Trailing Stop (baza) = 1.5%
- Regim: CHAOTIC, Trail Mult = 0.50x
- Signal: ALIGNED, Trail Mult = 1.10x
- Timeframe: 1m, Trail Mult = 0.90x
- **Efectiv = 1.5% x 0.50 x 1.10 x 0.90 = 0.743%**

Trailing-ul s-a comprimat de la 1.5% la 0.743% pentru ca piata e haotica, dar semanalele sunt aliniate.

### 23.1 DISABLE ADAPTIVE EXITS (toggle)

Cand este ON, TOTI multiplicatorii sunt ignorati. Botul foloseste doar valorile brute din slider-ele Entry/Exit. Util daca vrei control total fara ajustari automate.

### 23.2 Min Exit Floor %

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 0.30% |
| **Domeniu** | 0.05% — 5.00% |
| **Ce face** | Valoarea minima la care pot fi comprimate stop/take/trail |

**Explicatie:** Chiar daca cascadarea multiplicatorilor ar produce o valoare de 0.01%, Exit Floor previne asta — stop/take/trail nu vor fi niciodata mai mici de 0.30%.

### 23.3 Regime Multipliers (9 slidere)

Fiecare regim de piata are multiplicatori separati pentru Stop, Take si Trail:

| Regim | Stop (default) | Take (default) | Trail (default) |
|-------|---------------|----------------|-----------------|
| **Chaotic** | 0.70x | 0.60x | 0.50x |
| **Range** | 0.85x | 0.75x | 0.85x |
| **Trend** | 1.05x | 1.30x | 1.10x |

**Logica:**
- **Chaotic** (haos): Stop/Take/Trail MAI STRANSE (< 1.0x) = iesi rapid, protejeaza capital
- **Range** (lateral): Moderat mai stranse
- **Trend** (directional): MAI LARGI (> 1.0x) = lasa profitul sa curga

**Exemplu CHAOTIC:** Trail baza = 1.5%, Chaotic Trail = 0.50x → Trail efectiv = 0.75%. Botul iese mai repede in haos.

**Exemplu TREND:** Take baza = 2.5%, Trend Take = 1.30x → Take efectiv = 3.25%. Botul lasa profitul sa creasca in trend.

### 23.4 Signal Alignment Multipliers (5 slidere)

| Parametru | Default | Cand se aplica |
|-----------|---------|---------------|
| Aligned Stop x | 1.10x | Cand semnalele confirma pozitia |
| Aligned Take x | 1.25x | Cand semnalele confirma pozitia |
| Aligned Trail x | 1.10x | Cand semnalele confirma pozitia |
| Oppose Stop x | 0.85x | Cand semnalele contrazic pozitia |
| Oppose Take x | 0.80x | Cand semnalele contrazic pozitia |

**Logica:**
- **Aligned** (semnalele confirma): Mult > 1.0 = mai mult spatiu (semnalele sustin pozitia)
- **Oppose** (semnalele contrazic): Mult < 1.0 = mai strans (semnalele avertizeaza)

### 23.5 Timeframe Multipliers (3 slidere)

| Parametru | Default |
|-----------|---------|
| Short TF Stop x | 0.90x |
| Short TF Take x | 0.90x |
| Short TF Trail x | 0.90x |

**Ce fac:** Pe timeframe-uri scurte (1m, 3m), stop/take/trail sunt mai stranse (x 0.90). Pe timeframe-uri lungi se aplica multiplicator neutru (1.0x).

---

## 24. MULTI-ASSET CONTROL

### Toggle MULTI-ASSET TRADING

| Stare | Ce inseamna |
|-------|-------------|
| **OFF** | Botul tradeaza un singur asset la un moment dat |
| **ON** | Botul poate tranzactiona mai multe asset-uri simultan |

### Max Concurrent Assets

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 5 |
| **Domeniu** | 1 — 100 |
| **Ce face** | Numarul maxim de asset-uri tranzactionate simultan |

**Exemplu:** Cu Max Concurrent = 5, botul poate avea pozitii deschise simultan pe AAPL, TSLA, BTC/USD, ETH/USD si MSFT. Al 6-lea asset este ignorat pana cand una din pozitii se inchide.

**Atentie:** Mai multe asset-uri simultan = mai multa expunere. Asigura-te ca equity-ul suporta.

---

## 25. EFFECTIVE EXIT VALUES

### Ce este?

Panelul "EFFECTIVE EXIT VALUES" arata valorile REALE pe care botul le foloseste pentru stop/take/trail DUPA aplicarea tuturor multiplicatorilor adaptivi.

### Cum se citeste

| Element | Ce arata | Culoare |
|---------|----------|---------|
| **STOP LOSS** | Procentul efectiv de stop loss | Rosu |
| **TAKE PROFIT** | Procentul efectiv de take profit | Verde |
| **TRAILING** | Procentul efectiv de trailing stop | Auriu |

Sub fiecare valoare efectiva, apare si **base:** — valoarea bruta din slider, fara multiplicatori.

**Exemplu:**
- Stop Loss: **1.05%** (base: 1.50%) → Multiplicatorii au comprimat stop-ul
- Take Profit: **3.25%** (base: 2.50%) → Multiplicatorii au extins take-ul (probabil in trend)
- Trailing: **0.75%** (base: 1.50%) → Trailing comprimat (probabil in chaos)

Se actualizeaza automat cand modifici orice slider.

---

## 26. REGIME HEATMAP

### Ce este?

Trei box-uri colorate care arata vizual multiplicatorii activi pentru fiecare regim de piata si evidentiaza regimul CURENT.

### Cum se citeste

Fiecare box arata:
- **S:** = multiplicatorul Stop
- **T:** = multiplicatorul Take
- **Tr:** = multiplicatorul Trail

Box-ul regimului activ are contur stralucitor si fundal mai luminos.

**Exemplu:**
```
[TRENDING]        [RANGE]           [CHAOTIC]
 S:1.05x          S:0.85x           S:0.70x    ← activ (highlight)
 T:1.30x          T:0.75x           T:0.60x
 Tr:1.10x         Tr:0.85x          Tr:0.50x
```

Daca box-ul CHAOTIC este highlight, inseamna ca Oracle-ul a detectat haos pe piata si multiplicatorii din CHAOTIC sunt activi.

Se actualizeaza automat la fiecare 5 secunde.

---

## 27. PARAMETER PROFILES (Disk Persistence)

### Ce este?

Un sistem de profile numite care permite salvarea si incarcarea configuratiilor complete. Profilele sunt stocate pe disk in `titan_profiles.json` -- supravietuiesc rebuild-uri, crash-uri, si pot fi exportate/importate intre instante.

### Cum se creeaza un profil

1. Configureaza toti slider-ii cum doresti
2. Click SAVE CONFIG (salveaza pe disk)
3. Scrie un nume in campul "Profile name..." (ex: "Crypto Scalp Noapte")
4. Click **SAVE AS**
5. Profilul apare in dropdown

### Cum se incarca un profil

1. Selecteaza profilul din dropdown
2. Click **LOAD**
3. **TOTI** parametrii se restaureaza -- inclusiv TA Gates, Scan Params, Symbol Overrides (obiecte nested)
4. Se creeaza automat un undo snapshot inainte de load

### Cum se sterge un profil

1. Selecteaza profilul din dropdown
2. Click **DELETE**
3. Confirma stergerea in dialogul cyber
4. Profilul este sters permanent de pe disk

### Cum se exporta/importa un profil

1. **EXPORT:** Selecteaza profilul, click **EXPORT** -- se descarca un fisier `titan_profile_<name>.json`
2. **IMPORT:** Click **IMPORT**, selecteaza fisierul JSON -- profilul este importat pe disk (numele se preia din fisier sau din campul de text)

**Validari la import (din Sesiunea 44):**
- **Schema version check** — daca versiunea schemei din fisier nu corespunde cu cea curenta, importul este respins cu mesaj clar
- **Round-trip serde** — payload-ul JSON este serializat si deserializat inapoi; daca pierde date, importul esueaza
- **Coherence warnings** — dupa import reusit, se logheaza avertizari daca valorile nu sunt coerente (ex: breaker thresholds in ordine gresita, stop loss negativ)

### Migrare automata

Daca ai profile vechi din localStorage (versiuni anterioare), ele sunt importate automat pe disk la prima rulare si cheia veche este stearsa.

**Utilizare practica:**
- Creeaza un profil "Crypto BTC" cu setari optimizate pentru Bitcoin
- Creeaza un profil "Stocks Conservator" cu setari sigure pentru actiuni
- Exporta profilul tau optim si importa-l pe alta masina
- Schimba rapid intre profile in functie de conditiile de piata

---

## 28. RISK CALCULATOR LIVE

### Ce este?

Panel cu 4 metrici de risc calculate in timp real din setarile curente.

### Metrici

| Metrica | Ce arata | Culoare |
|---------|----------|---------|
| **MAX LOSS / TRADE** | Pierderea maxima posibila pe un singur trade, in dolari | Rosu |
| **POSITION SIZE** | Marimea pozitiei in dolari | Albastru |
| **RISK/REWARD** | Raportul intre riscul asumat si profitul tinta | Verde |
| **MAX DAILY LOSS** | Pierderea maxima zilnica in dolari | Auriu |

**Exemplu cu cont $10,000, Risk = 2%, Stop = 1.5%, Take = 2.5%:**
- Max Loss / Trade: $10,000 x 2% = **$200**
- Position Size: $200 / 1.5% = **$13,333**
- Risk/Reward: 1.5% : 2.5% = **1:1.67**
- Max Daily Loss: $10,000 x 3% (daily limit) = **$300**

Se actualizeaza automat cand modifici slider-ele.

---

## 29. PARAMETER HEALTH

### Ce este?

Un sistem de diagnostic automat care verifica 15 reguli de conflict/risc si afiseaza avertizari.

### Tipuri de badge-uri

| Badge | Culoare | Ce inseamna |
|-------|---------|-------------|
| **OK** | Verde | Totul e in regula |
| **WARN** | Portocaliu | Atentie — posibil suboptim |
| **DANGER** | Violet | Pericol — setare riscanta |

### Exemple de reguli verificate

- **Stop > Take** → DANGER: "Stop loss este mai mare decat take profit — vei pierde mai mult decat castigi"
- **Risk > 5%** → WARN: "Risc per trade prea mare pentru un incepator"
- **Circuit Breaker dezactivat** → DANGER: "Nici un sistem de protectie la pierderi zilnice"
- **Max Open Positions > 5** → WARN: "Multe pozitii simultane pot creste expunerea"
- **Trailing > Take** → WARN: "Trailing mai larg decat take profit — take profitul se va activa intotdeauna primul"

Se actualizeaza in timp real la orice modificare de slider.

---

## 30. AUTO-SUGGEST

### Ce este?

Functie care analizeaza ultimele 50 de trade-uri din jurnal si genereaza sugestii automate de ajustare a parametrilor.

### Cum se foloseste

1. Click **ANALYZE TRADES**
2. Botul analizeaza rezultatele din jurnal
3. Se afiseaza carduri cu sugestii
4. Fiecare sugestie are buton **APPLY** care aplica automat ajustarea
5. Click **CLOSE** cand ai terminat

### Exemple de sugestii

- "Rata mare de stop-out (>60%). Sugestie: Mareste trailing stop cu 0.5%." → [APPLY]
- "Profitul mediu pe trade este sub $10. Sugestie: Mareste take profit cu 1%." → [APPLY]
- "Multe exit-uri pe max hold time. Sugestie: Mareste Max Hold de la 60m la 120m." → [APPLY]

### 7 pattern-uri detectate

1. Rata mare de stop-out
2. Profit mediu prea mic
3. Exit-uri frecvente pe max hold
4. Ratio risk/reward suboptim
5. Prea multe trade-uri zilnice
6. Pierderi concentrate pe un anumit asset
7. Trailing stop ineficient (nu se activeaza niciodata)

---

## 31. PER-SYMBOL OVERRIDES

### Ce este?

Permite setarea de parametri CUSTOM per simbol, care inlocuiesc setarile globale doar pentru acel asset.

### Cum se seteaza un override

1. Deschide sectiunea (click pe "PER-SYMBOL OVERRIDES")
2. Scrie simbolul in campul "Symbol" (ex: AAPL)
3. Completeaza campurile dorite:
   - **Trail%** — trailing stop custom (lasa gol pt valoarea globala)
   - **Take%** — take profit custom
   - **Risk%** — risc per trade custom
4. Click **SET**

### Cum se sterge un override

1. Scrie simbolul in campul "Symbol"
2. Click **REMOVE**

### Exemplu practic

Setari globale: Trail = 1.5%, Take = 2.5%, Risk = 2.0%

Override pentru BTC/USD: Trail = 3.0%, Take = 5.0% (crypto e mai volatil, deci stop-uri mai largi)
Override pentru AAPL: Risk = 1.0% (actiune stabila, risc mai mic)

Lista override-urilor active se afiseaza sub butoane.

---

## 32. PRESET BAR

Bara de preset-uri si controale globale din partea de jos a tab-ului.

### Elemente (de la stanga la dreapta)

| Element | Ce face |
|---------|---------|
| **Search** | Filtreaza slider-ele dupa nume |
| **SAFE DEFAULT** | Aplica preset conservator pe TOATE slider-ele |
| **Lock SAFE** | Switch on/off — blocheaza preset-ul SAFE |
| **SCALP DEFAULT** | Aplica preset agresiv scalping |
| **Lock SCALP** | Switch on/off — blocheaza preset-ul SCALP |
| **SAVE CONFIG** | Salveaza setarile curente pe disk |
| **EXPORT** | Exporta configuratia intr-un fisier JSON |
| **IMPORT** | Importa configuratie dintr-un fisier JSON |
| **UNDO** | Restaureaza setarile de dinainte de ultima modificare |
| **SHOW CHANGES** | Afiseaza un tabel cu modificarile nesalvate |

### Search (Cautare parametri)

Scrie in campul de cautare si doar slider-ele care contin textul raman vizibile. Sectiunile complet goale se ascund.

**Exemplu:** Scrie "trail" → Apar doar slider-ele cu "trail" in nume: Trailing Stop, Scalp Trail Mult, ATR Trail Min, etc.

### Lock-uri preset

Cand un lock este ON:
- Preset-ul se aplica automat la fiecare pornire a aplicatiei
- Slider-ele NU pot fi modificate manual (sunt blocate)
- Doar un singur preset poate fi blocat la un moment dat
- Starea lock-ului se salveaza in localStorage

### EXPORT / IMPORT

- **EXPORT:** Descarca un fisier `.json` cu toate setarile curente. Util ca backup sau pentru a transfera setari pe alt calculator.
- **IMPORT:** Incarca un fisier `.json` exportat anterior si aplica toate setarile din el. Din Sesiunea 44, import-ul trimite payload-ul JSON direct (typed) prin comanda `apply_full_settings`, eliminand coercion-ul `String(v)` care putea corupe obiecte nested (TA Gates, Symbol Overrides).

### UNDO

Restaureaza setarile la starea de dinaintea ultimei modificari importante (preset aplicat, import, etc.). Se salveaza pe disk in `titan_settings_undo.json`, deci supravietuieste restart. Din Sesiunea 44, UNDO foloseste aceeasi comanda `apply_full_settings` ca si import-ul — trimite snapshot-ul complet ca JSON typed, asigurand restaurare fidela inclusiv pentru campuri complexe.

### Batch Update Transactional (Sesiunea 44)

Cand se aplica un preset sau se modifica mai multi parametri simultan, `batch_update_settings` foloseste un mecanism de **snapshot + rollback**:

1. Se face un clone al starii curente
2. Se aplica fiecare parametru individual
3. Daca oricare esueaza — **toata** starea revine la snapshot (rollback complet)
4. Save pe disk si sincronizare runtime doar dupa succes complet

Rezultat: nu mai exista risc de configuratie partial salvata pe disk dupa o eroare.

---

## 33. SETTINGS DIFF PANEL

### Ce este?

Panel afisabil prin butonul **SHOW CHANGES** care arata un tabel cu TOATE modificarile nesalvate.

### Cum se citeste

| Coloana | Ce arata |
|---------|----------|
| **Parametru** | Numele setarii modificate |
| **Salvat** | Valoarea care este pe disk (ultima salvare) |
| **Curent** | Valoarea actuala din slider |

Valorile diferite sunt highlight-ate color:
- Rosu = valoare veche (salvata)
- Verde = valoare noua (curenta)

**Exemplu:**
```
Parametru          | Salvat | Curent
trailing_stop_pct  |  1.5%  |  2.0%   ← modificat
take_profit_pct    |  2.5%  |  3.0%   ← modificat
risk_per_trade     |  2.0%  |  2.0%   (nemodificat — nu apare)
```

Doar parametrii MODIFICATI apar in tabel. Daca nu exista modificari, se afiseaza "No unsaved changes".

---

## 34. DRY RUN vs LIVE

### Badge-ul DRY/LIVE

Badge-ul din coltul din dreapta titlului arata modul curent al botului:

| Mod | Badge | Ce face botul |
|-----|-------|---------------|
| **DRY RUN** | Albastru | SIMULEAZA trading — nu trimite ordine reale |
| **LIVE** | Verde (pulsand) | EXECUTA ordine reale pe piata |

### Cum se schimba

Badge-ul se schimba din sidebar-ul principal (nu din HFT Engine). In HFT Engine este doar un indicator readonly.

**Atentie:** In modul LIVE, fiecare decizie a botului se traduce in ordine REALE cu bani REALI. Asigura-te ca toate setarile sunt corecte inainte de a trece in LIVE.

**Default (S72):** Fresh install porneste cu `dry_run_enabled: true`. Live trading trebuie activat explicit. Cand live trading e activ, UI-ul afiseaza border rosu pulsant + banner `LIVE TRADING — DRY RUN OFF`.

---

## 35. PERSISTENTA SETARILOR

### Unde se salveaza

| Setare | Fisier pe disk | Locatie |
|--------|---------------|---------|
| Toti parametrii HFT Engine | `titan_settings.json` | Folderul config al aplicatiei |
| Undo snapshot | `titan_settings_undo.json` | Langa titan_settings.json |
| Chart settings | `titan_chart_settings.json` | Langa titan_settings.json |
| Profile-uri | `titan_profiles.json` | Langa titan_settings.json (+ auto-backup `.bak`) |
| Lock-uri preset | `localStorage` | Browser/WebView intern |

### Ce supravietuieste restart

| Eveniment | Setari HFT | Profiles | Locks |
|-----------|-----------|----------|-------|
| Toggle intre taburi | DA | DA | DA |
| Inchidere + redeschidere aplicatie | DA | DA | DA |
| Crash aplicatie | DA | DA | Posibil |
| Rebuild cu build.sh | DA | DA | NU* |

(*) `build.sh` poate sterge cache-ul WebView (`localStorage`). Setarile din `titan_settings.json` si profilele din `titan_profiles.json` supravietuiesc intotdeauna -- sunt pe disk, nu in browser.

### Cum se salveaza

1. Modifica slider-ele cum doresti
2. Click **SAVE CONFIG**
3. Toast "Settings saved" confirma
4. Setarile sunt pe disk si vor fi restaurate la urmatorul start

**Atentie:** Daca modifici slider-ele dar NU dai SAVE CONFIG, modificarile se pierd la restart.

### Protectii Persistence (S72)

- **Error propagation:** Toate cele 8 comenzi de save propaga eroarea la frontend (nu mai sunt ignorate silentios). Eveniment `settings:save-failed` vizibil in Save Event Timeline.
- **Journal atomic save:** `TradeJournal::save()` foloseste `tmp` + `rename` pattern -- fisierul nu se poate corupe la crash.
- **Profile validation:** `load_profile` valideaza coerenta si siguranta setarilor dupa incarcare. Warnings returnate in Result.
- **Snapshot delta compression:** Snapshot-ul se scrie doar daca difera de cel anterior (comparatie SHA-256).
- **Journal debounce:** Save efectiv doar daca exista date noi si au trecut 30s. Flush fortat la shutdown.

### Runtime Sync (din Sesiunea 44)

La fiecare save / load profile / batch update, functia `apply_runtime_config()` sincronizeaza automat campurile runtime din TitanState cu valorile salvate. Campurile sincronizate: strategy_mode, nexus_fusion_alpha, dry_run, circuit_breaker thresholds, flash_crash params, smarttrend config. Nu mai este necesara nicio actiune manuala pentru a asigura coerenta intre settings si runtime.

**S08 Update:** `StrategyMode` include acum 5 variante: MathOracle, GpuNexus, Fusion, SignalStrategy, **SmartTrend**. SmartTrend mode activeaza scoring pipeline-ul dedicat (SmartTrendEngine cu FSM, markers ST BUY/SELL/EXIT/TRAIL/STALK/BREAK, kill switch cu confirm token). Cand `st_killed == true`, autopilot blocheaza toate entries SmartTrend.

---

## 36. RECOMANDARI SI TIPS

### Pentru incepatori

1. **Incepe cu DRY RUN** — testeaza cateva zile in simulare inainte de LIVE
2. **Foloseste SAFE DEFAULT** — valorile conservative sunt cele mai sigure la inceput
3. **Nu modifica toti parametrii deodata** — schimba cate un parametru si observa efectul
4. **Monitorizeaza Parameter Health** — daca vezi DANGER, corecteaza inainte de a trece LIVE
5. **Risk Per Trade sub 2%** — regula de aur a managementului de risc
6. **Exporta configuratia** — inainte de a experimenta, exporta setarile ca backup

### Pentru avansati

1. **Creeaza profile per piata** — setari diferite pt crypto vs stocks
2. **Foloseste Per-Symbol Overrides** — fine-tune pe asset-uri individuale
3. **Ajusteaza Adaptive Multipliers** — personalizeaza raspunsul la regimuri
4. **Monitorizeaza Regime Heatmap** — intelege ce regim e activ si cum afecteaza trade-urile
5. **Ruleaza Auto-Suggest** — dupa 50+ trade-uri, analizeaza pattern-urile

### Greseli frecvente de evitat

1. **Stop > Take** — pierzi mai mult decat castigi pe fiecare trade
2. **Circuit Breaker prea slab** — poti pierde tot contul intr-o zi
3. **Max Positions prea mare cu Risk mare** — expunere totala poate depasi 50% din cont
4. **Adaptive Exits dezactivat fara intelegere** — pierzi protectia in piata haotica
5. **Ignorarea UNDO** — dupa o modificare gresita, UNDO te salveaza

---

## 37. GLOSAR DE TERMENI

| Termen | Explicatie |
|--------|-----------|
| **ATR** | Average True Range — masoara volatilitatea medie pe N perioade |
| **Bollinger Bands** | Benzi in jurul mediei mobile, bazate pe deviatie standard |
| **Circuit Breaker** | Sistem de siguranta care opreste trading-ul la pierderi mari |
| **Crossover** | Cand o linie trece peste/sub alta (semnal de trading) |
| **DRY RUN** | Simulare — botul calculeaza dar nu trimite ordine reale |
| **EMA** | Exponential Moving Average — medie ponderata cu accent pe date recente |
| **Entropy** | Masura a dezordinii/haosului in miscarile de pret |
| **Flash Crash** | Miscare extrema si rapida a pretului; detectorul automat inchide pozitii si blocheaza intrari (vezi §18.6) |
| **Equity** | Valoarea totala a contului (cash + pozitii deschise) |
| **Hurst Exponent** | Masoara persistenta trendului (>0.5 = trending, <0.5 = mean-reverting) |
| **HFT** | High-Frequency Trading — trading cu frecventa mare |
| **Kalman Filter** | Algoritm de estimare care prezice directia pretului |
| **LIVE** | Modul real — botul trimite ordine reale cu bani reali |
| **MACD** | Moving Average Convergence/Divergence — indicator de trend si momentum |
| **Notional** | Valoarea totala in dolari a unei pozitii |
| **OFI** | Order Flow Imbalance — dezechilibrul intre ordine de cumparare si vanzare |
| **Oracle** | Sistemul AI al botului cu 7 layere de analiza |
| **Overbought** | Pret prea ridicat fata de medie, potential de corectie in jos |
| **Oversold** | Pret prea scazut fata de medie, potential de revenire in sus |
| **Regime** | Starea curenta a pietei: Trending, Range sau Chaotic |
| **RSI** | Relative Strength Index — oscilator de momentum (0-100) |
| **RVOL** | Relative Volume — volumul curent fata de media istorica |
| **Scalping** | Strategie de trading cu pozitii foarte scurte (secunde-minute) |
| **Spread** | Diferenta intre pretul de cumparare (ask) si cel de vanzare (bid) |
| **StochRSI** | Stochastic RSI — RSI aplicat pe RSI, mai sensibil |
| **Stop Loss** | Ordinul care inchide pozitia la pierdere maxima setata |
| **Take Profit** | Ordinul care inchide pozitia la profitul tinta |
| **Trailing Stop** | Stop care se misca cu pretul in directia profitului |
| **VPIN** | Volume-Synchronized PIN — masoara toxicitatea flow-ului |
| **Wavelet** | Descompunere frecventiala a pretului |
| **DD Velocity** | Viteza cu care drawdown-ul creste, masurata in %/min — vezi §18.7 |
| **Command Palette** | Overlay de cautare rapida a comenzilor, activat cu Ctrl+K — vezi §38 |

---

## 38. KEYBOARD SHORTCUTS SI COMMAND PALETTE

Titan HFT ofera un set de shortcut-uri globale pentru actiuni rapide in scalping si o paleta de comenzi cu cautare fuzzy.

### 38.1 Shortcut-uri Directe

Acestea functioneaza oricand (indiferent de tab-ul activ), cu exceptia cazului cand focus-ul e pe un camp de input sau textarea.

| Shortcut | Actiune | Observatii |
|----------|---------|------------|
| **Ctrl+K** | Deschide Command Palette | Functioneaza mereu, inclusiv din input |
| **Ctrl+Enter** | Quick Buy (market) | Plaseaza ordin market buy pe simbolul din chart |
| **Ctrl+Shift+Enter** | Quick Sell (market) | Plaseaza ordin market sell pe simbolul din chart |
| **Ctrl+Shift+X** | Flatten All | Lichideaza TOATE pozitiile (emergency liquidate) |
| **Ctrl+P** | Toggle Autopilot | Porneste/opreste autopilotul |
| **Ctrl+]** | Timeframe mai mare | Cicleaza: 1m → 3m → 5m → 10m → 15m → 30m → 1h → 4h → 1d |
| **Ctrl+[** | Timeframe mai mic | Cicleaza in sens invers: 1d → 4h → 1h → ... → 1m |

### 38.2 Command Palette (Ctrl+K)

Apasa **Ctrl+K** din orice loc in aplicatie pentru a deschide paleta de comenzi.

**Cum functioneaza:**
1. Apare un overlay cu un camp de cautare si lista de comenzi
2. Tasteaza pentru a filtra comenzile (cautare fuzzy)
3. Navigheaza cu **Arrow Up/Down** si confirma cu **Enter**
4. Sau click pe comanda dorita
5. **Escape** inchide paleta

**19 comenzi disponibile:**

| Categorie | Comanda | Shortcut (daca exista) |
|-----------|---------|------------------------|
| **Trade** | Quick Buy (market) | Ctrl+Enter |
| **Trade** | Quick Sell (market) | Ctrl+Shift+Enter |
| **Trade** | Flatten All Positions | Ctrl+Shift+X |
| **Control** | Toggle Autopilot | Ctrl+P |
| **Navigate** | Go to Chart | — |
| **Navigate** | Go to News AI | — |
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

**Symbol switch:** Daca textul introdus in paleta nu corespunde niciunei comenzi, apasarea Enter il trateaza ca simbol de trading si comuta chart-ul pe acel simbol. Exemplu: Ctrl+K → "TSLA" → Enter = chart-ul se schimba pe TSLA.

### 38.3 Compatibilitate cu Chart Hotkeys

Shortcut-urile de chart existente (L=linie, R=ray, H=hline, V=vline, F=fib, T=text, M=measure, C=crosshair, A=alert, P=replay) functioneaza in continuare normal. Ele se activeaza doar fara Ctrl, deci nu interfereaza cu shortcut-urile Ctrl+* de mai sus.

---

## 39. TA ENTRY GATES — FILTRE PRE-ENTRY CU SOFT/HARD MODE

Sectiunea **"TA ENTRY GATES"** (cu titlu portocaliu) apare in tab-ul HFT Engine, chiar deasupra Strategy Scoring. Contine 6 filtre tehnice care se aplica dupa ce Oracle-ul decide Buy/Short, dar inainte de a executa entry-ul.

### 39.1 Cum functioneaza

Fiecare gate verifica un indicator tehnic si poate fie **penaliza** scorul (Soft), fie **bloca** complet entry-ul (Hard).

| Mod | Efect | Cand il folosesti |
|-----|-------|-------------------|
| **Off** | Gate-ul nu face nimic | Nu vrei acest filtru |
| **Soft** | Scorul trade-ului scade cu Soft Penalty puncte | Vrei feedback, dar nu vrei sa blochezi |
| **Hard** | Entry-ul e blocat complet (nu se plaseaza ordinul) | Vrei garantia ca nu intri in conditii proaste |

### 39.2 Cele 6 Gate-uri

| Gate | Indicator | Cand se activeaza |
|------|-----------|-------------------|
| **RSI** | Relative Strength Index | Long cand RSI > overbought (70). Short cand RSI < oversold (30) |
| **EMA Align** | EMA 9/20/50 | Long cand EMA-urile nu sunt aliniate crescator. Short cand nu sunt descrescatoare |
| **ADX** | Average Directional Index | Cand ADX < threshold (25) — trend prea slab pt a intra |
| **BB Squeeze** | Bollinger Bands width | Cand benzile sunt prea inguste (< 2% din pret mediu) — consolidare |
| **MACD** | MACD Histogram | Long cand histogram < 0. Short cand histogram > 0 |
| **Stoch RSI** | Stochastic RSI | Long cand Stoch > 80. Short cand Stoch < 20 |

**Threshold-urile RSI** (`rsi_overbought`, `rsi_oversold`) si **ADX** (`adx_trend_threshold`) sunt cele deja existente din sectiunea Classical Filters. Nu trebuie configurate separat.

### 39.3 Presets

Dropdown-ul din dreapta titlului permite selectarea rapida a unui profil predefinit:

| Preset | Ce seteaza |
|--------|-----------|
| **Strict** | Toate 6 gate-uri pe Hard — calitate maxima, putine intrari |
| **Balanced** | RSI + ADX pe Soft, restul Off — filtru moderat |
| **Oracle Only** | Toate Off (default) — Oracle-ul decide singur |

Dupa selectarea unui preset, poti ajusta manual gate-urile individuale. Dropdown-ul revine la "Custom" la orice modificare manuala.

### 39.4 Soft Penalty

Slider-ul de sub grid-ul de gate-uri controleaza **cate puncte** se scad din scor per gate Soft incalcat.

| Proprietate | Valoare |
|-------------|---------|
| **Default** | 15 |
| **Domeniu** | 1 — 50 |
| **Ce face** | Penalizare per gate Soft incalcat |

Penalizarile se cumuleaza. Daca 3 gate-uri Soft sunt incalcate simultan si penalty = 15, scorul scade cu 45 puncte. Un scor suficient de mic impiedica entry-ul natural (cade sub threshold-ul de regim).

### 39.5 Fail-Open pe Lipsa Date

Daca un indicator e `None` (nu sunt destule candle-uri pt calcul — RSI necesita 14, ADX 14, EMA50 necesita 50), gate-ul respectiv este **sarit**. Nu blocheaza si nu penalizeaza.

### 39.6 Setari JSON

Campurile din `titan_settings.json`:

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

Valorile posibile per gate: `"Off"`, `"Soft"`, `"Hard"`.

### 39.7 Pipeline Visual

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

## 40. INDICATOR CACHE — HYBRID STREAMING + ORACLE CU CACHE PER SIMBOL

### 40.1 Ce este

IndicatorCache elimina recalcularea repetata a indicatorilor tehnici in autopilot. In loc sa recalculeze toti indicatorii (EMA, RSI, MACD, BB, ADX, VWAP, StochRSI, Oracle) de la zero pe 200 candle-uri la fiecare ciclu, cache-ul stocheaza rezultatul si il returneaza instant cand candle-ul nu s-a schimbat.

### 40.2 Cum functioneaza

Autopilotul evalueaza fiecare simbol candidat la fiecare ciclu (~1 secunda). Intre candle-uri (ex: la TF 5m, 300 cicluri per candle), indicatorii nu se schimba. Cache-ul exploateaza aceasta proprietate:

| Situatie | Ce face cache-ul | Latenta |
|----------|-------------------|---------|
| Acelasi candle (95%+ din cicluri) | Returneaza rezultat stocat | ~0 |
| Candle nou | StreamingIndicators O(1) + Oracle batch | < 5ms |
| Prima data per simbol | Calcul complet batch | 10-50ms |

### 40.3 Strategie hibrida

- **Indicatori clasici** (EMA 9/20/50/200, RSI, MACD, BB, ADX, VWAP, StochRSI): calculati incremental prin `StreamingIndicators` — doar ultimul candle e procesat, restul state-ului e persistent
- **Oracle** (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD): recalculat batch pe toate candle-urile, dar doar o data per candle nou — nu la fiecare ciclu

Cele doua rezultate sunt combinate intr-un singur `Indicators` snapshot complet.

### 40.4 Eviction

Cache-ul sterge automat entries neaccesate mai mult de 1 ora. Asta previne cresterea memoriei pentru simboluri care au fost evaluate o singura data si nu mai sunt active.

Eviction-ul ruleaza la ~1% din ciclurile autopilot (verificare probabilistica, nu timer dedicat).

### 40.5 Garantie de corectitudine

La cold start (prima apelare per simbol), cache-ul ruleaza `calculate_all_with_periods()` complet — exact aceeasi functie ca path-ul fara cache. Asta garanteaza ca rezultatul este BIT-IDENTIC. Dupa cold start, update-urile incrementale au fost validate cu proptest la precizie 1e-10.

### 40.6 Ce NU este cached

- **Dashboard score** (afisajul Strategy Score din header) — foloseste `compute_strategy_score_with_mode()` care e o functie pura separata
- **Backtest** — ruleaza izolat cu date diferite, nu foloseste cache-ul live
- **ATR pentru stop-loss** — calculat separat prin `calculate_atr()`, nu prin IndicatorCache

### 40.7 Pipeline actualizat

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

### 41.1 Ce este

TitanAudio adauga alerte sonore si vizuale pe evenimentele critice de trading. Sunetele sunt generate procedural cu Web Audio API (fara fisiere audio externe). Scopul: sa nu ratezi fill-uri, breaker-e sau flash crash-uri cand nu te uiti activ la ecran.

### Butonul AUDIO ON/OFF

In header-ul panelului TITAN SAFETY, langa badge-ul circuit breaker, exista butonul `AUDIO: ON` / `AUDIO: OFF`.

- **Click** — toggle instant ON↔OFF
- **Persistenta** — starea se salveaza in localStorage si se restaureaza la restart
- **Visual** — cyan cand ON, gri cand OFF

### 41.2 Evenimente monitorizate

| Eveniment | Sunet | Efecte vizuale |
|-----------|-------|----------------|
| Trade fill (buy/sell) | Blip ascendent 3 tonuri (520→780→1040 Hz) | Flash title: "FILL: SIDE SYMBOL" (2.5s) |
| Breaker CAUTION | Alarm pulsatorie 3x (600+750 Hz) | Flash title + screen glow galben |
| Breaker HALT/PANIC | Alarm pulsatorie 3x (600+750 Hz) | Flash title + screen glow rosu |
| Flash crash detectat | Buzz descendent agresiv (440→330→220 Hz) | Flash title + screen glow magenta |
| Watchdog dead loops | Triple triangle ping (400→500→400 Hz) | Flash title: "WATCHDOG: N DEAD" |
| Breaker NORMAL (reset) | — (fara sunet) | — |

### 41.3 Screen glow

Overlay fullscreen semi-transparent cu border glow animat (3 pulsuri). Se dezactiveaza automat dupa 2-5 secunde. Culorile indica severitatea:

- **Rosu** — breaker PANIC/HALT (cel mai sever)
- **Galben** — breaker CAUTION
- **Magenta** — flash crash detectat

### 41.4 Flash title

Alternare document.title cu mesajul alertei la 500ms. Util cand fereastra Titan e minimizata sau pe alt monitor — titlul clipeste in taskbar.

### 41.5 Note tehnice

- AudioContext se creeaza la primul click/keypress (Chrome autoplay policy)
- Volume default: 0.35 (configurabil via consola: `TitanAudio.setVolume(0.5)`)
- Sunetele exista si pentru `fill-exit-profit`, `fill-exit-loss`, `signal` — disponibile programatic prin `TitanAudio.play('fill-exit-profit')` dar neconectate automat (payload-ul trade-fill din Rust nu contine inca PnL)

---

---

## 42. CONCURRENT AUTOPILOT ARCHITECTURE (S5+S6)

### 42.1 Ce este

Autopilotul foloseste `futures::future::join_all` pentru a evalua pozitiile si candidatii in paralel. Deciziile (close_position, send_order, state mutations) se aplica secvential dupa gather.

### 42.2 Flow pe un tick

1. **Parallel Exit Gather** — pentru fiecare pozitie deschisa, `evaluate_position_exit()` se ruleaza concurent: get_candles, calculate_atr, signal markers, adaptive stops, force exit checks. Returneaza `ExitEvaluation` (read-only struct, zero side effects).

2. **Sequential Exit Apply** — itereaza `Vec<ExitEvaluation>`:
   - Partial take → `close_position` + update partial_taken
   - Full exit → `close_position` + journal + trail cleanup
   - Dupa orice exit: `continue 'autopilot` (re-fetch positions)

3. **Sequential Pre-filter** — pentru fiecare candidat: flash crash check, holding check, pending orders (API call), cooldown, position limit, correlation guard

4. **Parallel Entry Gather** (non-SignalStrategy) — `evaluate_candidate_entry()` concurent: get_candles, indicator_cache.get_or_compute, check_entry. Returneaza `EntryEvaluation`.

5. **Best-Score Selection** — din toate evaluarile cu Signal::Buy sau Signal::Short, se selecteaza candidatul cu scorul maxim. OFI + Nexus filters aplicate pe candidatul selectat. Apoi send_order.

### 42.3 Best-Score vs First-Match

Anterior, autopilotul executa primul candidat care trecea filtrele (ordinea era arbitrara din lista). Acum, toti candidatii sunt evaluati, si se alege cel cu **scorul cel mai mare**. Avantaj: intrari de calitate mai mare cand exista mai multi candidati simultan.

### 42.4 Locking Safety

Toate lock-urile din codul paralel sunt short-lived reads (<1us):
- `candle_buffers` (Mutex) — snapshot()
- `indicator_cache` (Mutex) — get_or_compute()
- `vol_cache`, `symbol_best_tf` (Mutex) — read
- `ofi_state` (ArcSwap) — load(), lock-free

Mutations si API calls raman pe main task: `trail_highs/lows`, `session_analytics`, `close_position`, `send_order_internal`.

### 42.5 Cand ajuta cel mai mult

- Multe pozitii deschise (5+) cu ATR calculation per simbol
- Ring buffer gol (REST fallback 50-500ms per simbol — se fac toate in paralel)
- SignalStrategy cu multe assets (pre-filtrarea e secventiala, dar dupa ea evaluarea e paralela)

---

## 43. PORTFOLIO RISK GATES — VaR + HHI Concentration (S9)

### 43.1 Ce sunt

Doua gate-uri complementare la nivel de **portofoliu** care blocheaza intrari cand riscul total e prea mare. Se aplica DUPA OFI si Nexus filters, INAINTE de position sizing.

- **VaR (Value at Risk)** — risc statistic bazat pe volatilitate si corelatie inter-assets
- **HHI (Herfindahl-Hirschman Index)** — masura de concentrare a capitalului

### 43.2 Locatie in UI

Card **PORTFOLIO RISK GATES** (violet, `#E040FB`) in tab-ul HFT Engine, dupa TA Entry Gates. Doua sub-panouri: VaR (stanga) si HHI (dreapta).

### 43.3 Parametri

| Parametru | ID Element | Setting Backend | Default | Range |
|-----------|-----------|-----------------|---------|-------|
| VaR Gate toggle | `hft-var-gate` | `enable_var_gate` | OFF | bool |
| Max Portfolio VaR % | `hft-var-max` | `max_portfolio_var_pct` | 2.0 | 0.1-10.0 |
| Lookback Periods | `hft-var-lookback` | `var_lookback_periods` | 20 | 5-100 |
| Concentration Gate toggle | `hft-hhi-gate` | `enable_concentration_gate` | OFF | bool |
| Max HHI | `hft-hhi-max` | `max_concentration_hhi` | 0.40 | 0.10-1.00 |

Slider-ul VaR % are div=10 (slider 1-100, display 0.1-10.0%). Slider-ul HHI are div=100 (slider 10-100, display 0.10-1.00).

### 43.4 Cum functioneaza VaR Gate

1. Extrage close prices din `CandleRingBuffer` per simbol din portofoliu
2. Computa log returns: `r_t = ln(close_t / close_{t-1})` pe ultimele `var_lookback_periods` perioade
3. Construieste matricea de covarianta NxN (sample covariance, N-1 denominator)
4. Calculeaza VaR parametric: `VaR = z_score(0.99) * sqrt(w' * Sigma * w)` unde w = ponderi pozitii
5. Daca VaR propus (cu noua pozitie inclusa) > `max_portfolio_var_pct` → entry BLOCAT

**z_score(0.99) = 2.326** — folosim 99% confidence (nu 95%) pentru protectie mai agresiva in scalping.

**CVaR (Expected Shortfall)** este calculat intern dar nu e folosit ca gate — disponibil pentru analiza viitoare.

### 43.5 Cum functioneaza HHI Gate

```
HHI = sum(w_i^2)     unde w_i = |notional_i| / total_notional
```

- HHI = 1.0 → 100% intr-un singur simbol
- HHI = 0.25 → 4 pozitii egale (diversificat)
- HHI = 0.10 → 10 pozitii egale

Daca adaugarea pozitiei propuse ar duce HHI peste `max_concentration_hhi`, entry-ul e blocat.

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

HHI se evalueaza inaintea VaR deoarece e O(1) (pur aritmetic), in timp ce VaR necesita constructia matricei de covarianta.

### 43.7 Recomandari de utilizare

- **Incepe cu ambele OFF** — observa comportamentul existent cateva sesiuni
- **Activeaza HHI primul** — e mai usor de inteles si nu depinde de date istorice
- **HHI 0.40** e un bun compromis — permite max 2-3 pozitii concentrate, dar blocheaza all-in pe un singur simbol
- **VaR 2.0%** e conservator — creste la 3-5% daca strategia ta genereaza multi-symbol entries frecvent
- **Lookback 20** e default — creste la 50 daca vrei sa capturezi volatilitate pe un orizont mai lung

### 43.8 Diferenta fata de Circuit Breaker

| Aspect | Circuit Breaker (§18) | VaR/HHI Gate (§43) |
|--------|----------------------|-------------------|
| Cand actioneaza | DUPA pierdere | INAINTE de entry |
| Ce masoara | PnL zilnic, drawdown | Volatilitate, corelatie, concentrare |
| Nivel | Portofoliu total (PnL) | Per-entry propus |
| Actiune | Reduce pozitii / stop trading | Blocheaza intrarea specifica |
| Reset | Manual | Automat (conditii se schimba) |

### 43.9 Implementare tehnica

**Backend:** `titan-risk` crate — `compute_returns()`, `covariance_matrix()`, `parametric_var()`, `conditional_var()`, `var_gate_blocks_entry()`, `concentration_hhi()`, `hhi_gate_blocks_entry()`

**Integrare:** `src-tauri/src/loops/autopilot.rs` — bloc de ~45 linii dupa OFI+Nexus checks

**Frontend:** `src/index.html` (card HTML) + `src/titan_perf_stats.js` (event handlers + restore + batch save)

---

## 44. FEATURE STORE — ENTRY CONTEXT SNAPSHOT (S13)

### 44.1 Ce este

Feature Store captureaza contextul complet al indicatorilor si strategiei la momentul intrarii intr-un trade. Snapshot-ul (20 campuri) se stocheaza temporar in memorie si se ataseaza la `TradeRecord` cand trade-ul se inchide (full sau partial exit).

### 44.2 De ce conteaza

Fara Feature Store, jurnalul contine doar PnL si timestamp. Cu Feature Store, fiecare trade raspunde la:
- Ce spunea Oracle-ul? (score, confidence, regime)
- Care erau indicatorii? (RSI, ADX, EMA, VPIN, Hurst)
- Cum arata microstructura? (OFI, spread, imbalance)
- Ce scor a avut strategia? (strategy_score, entry_type, FVG)

### 44.3 Campurile TradeFeatures

**Oracle:** oracle_score, oracle_confidence, oracle_regime

**Indicatori:** rsi, adx, atr_pct, ema9, ema50, vpin, hurst, rvol

**Microstructure:** ofi, spread_bps, imbalance_ratio

**Strategy:** strategy_score, signal_source, fvg_type, entry_type, ta_gates_passed, ta_gates_total

### 44.4 Flow tehnic

1. **Entry reusit** → `snapshot_entry_features()` citeste din IndicatorCache (`peek()`) + OfiState (`ArcSwap.load()`) + TradeDecision → stocheaza in `state.entry_features` HashMap
2. **Full exit** → `entry_features.remove(symbol)` → paseaza in `TradeRecordInput` → journal pe disk
3. **Partial exit** → `entry_features.get(symbol).cloned()` (NU sterge) → journal pe disk

### 44.5 Backward compatibility

`#[serde(default)]` pe campul `entry_features: Option<TradeFeatures>`. Jurnal-uri vechi se incarca cu `None`. Zero breaking changes.

### 44.6 Componenta `peek()` pe IndicatorCache

Metoda read-only care returneaza un clone al snapshot-ului cached fara a triggera re-computation sau a actualiza access time. Folosita exclusiv de Feature Store.

### 44.7 Impactul pe performanta

Zero overhead masurabil:
- `peek()` = HashMap lookup + clone (~1us)
- `OfiState.load()` = ArcSwap atomic load (~1ns)
- `entry_features.insert()` = HashMap insert sub Mutex (~1us)
- Total: <5 microsecunde per entry, executat o singura data per trade

---

## 45. MICROSTRUCTURE SCORE MODIFIER — STREAM VPIN + SPREAD REGIME + DUAL VPIN DIVERGENCE (S10-lite)

### 45.1 Ce face

Aplica **score modifiers soft** pe entry-uri bazate pe semnale de microstructura in timp real. Inserat in `evaluate_candidate_entry()` dupa `check_entry()` si inainte de ranking-ul candidatilor.

### 45.2 Stream VPIN

VPIN calculat din trade-by-trade flow (Lee-Ready classification), nu din candle BVC:

- **Volume-synced bins**: bucket se inchide la `bucket_vol_threshold` (5000 default)
- **20 buckets** rolling, EMA smoothing `alpha = 0.15`
- Stocat in `OfiState.stream_vpin` (ArcSwap, lock-free)
- **Mai precis** decat candle VPIN pe timeframe-uri scurte (1m, 5m)

### 45.3 Spread Regime

Dual EMA pe `spread_bps` cu detectie narrowing:

- **EMA fast**: `alpha = 0.15` (~13 ticks half-life)
- **EMA slow**: `alpha = 0.03` (~66 ticks half-life)
- **Narrowing**: `ema_fast < ema_slow * 0.80`
- Stocat in `OfiState.spread_narrowing` (ArcSwap, lock-free)
- **Semnificatie**: market makers strang spread-ul cand anticipeaza miscare

### 45.4 Dual VPIN Divergence

Compara stream VPIN (tape) vs candle VPIN (oracle). Divergenta = informatie:

| Divergenta | Interpretare | Score |
|-----------|-------------|-------|
| > +0.15 | Hidden toxicity (tape vede ce candle-urile nu arata) | **-8** |
| < -0.15 | False alarm filtrata (candle VPIN e noise) | **+5** |
| [-0.15, +0.15] | Concordanta | **0** |

### 45.5 Tabel complet score modifiers

| Conditie | Score | Cumulativ |
|----------|-------|-----------|
| Spread narrowing | **+5** | Da |
| NBBO imbalance aliniat (|imb| > 0.3) | **+4** | Da |
| Dual VPIN false alarm (div < -0.15) | **+5** | Da |
| Dual VPIN hidden toxicity (div > +0.15) | **-8** | Da |
| Stream VPIN extreme (> 0.85) | **-10** | Da |

**Range posibil:** +14 (toate bonus-urile) la -18 (toate penalty-urile)

### 45.6 Pozitia in pipeline

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

Microstructure modifier se aplica DUPA scoring dar INAINTE de ranking. Afecteaza care candidat e selectat, nu doar daca intra.

### 45.7 TradeFeatures extinse

Doua campuri noi pe `TradeFeatures` (backward compat cu `#[serde(default)]`):

- `vpin_divergence: f64` — stream_vpin minus candle_vpin la entry
- `spread_narrowing: bool` — flag spread regime la entry

Campul `vpin` existent acum contine stream_vpin (mai precis decat candle VPIN).

### 45.8 OfiState campuri noi

4 campuri adaugate la `OfiState` (l2_heatmap.rs):

- `stream_vpin: f64` — VPIN din trade flow
- `spread_ema_fast: f64` — EMA rapida spread (diagnostic)
- `spread_ema_slow: f64` — EMA lenta spread (diagnostic)
- `spread_narrowing: bool` — flag derivat

Toate cu `Default = 0/false`. ArcSwap lock-free, zero contention adaugata.

### 45.9 Frontend event extins

`ofi-update` event include acum `stream_vpin` si `spread_narrowing` pentru afisare in UI.

### 45.10 Impactul pe performanta

Zero overhead masurabil:
- `StreamVpin.update()` = VecDeque push + arithmetic (~100ns per trade)
- `SpreadRegime.update()` = 2 EMA updates (~10ns per quote)
- `apply_microstructure_modifier()` = ArcSwap load + 5 comparatii (~50ns per candidat)
- `OfiState` ramane sub 128 bytes, ArcSwap atomic store neschimbat

---

---

## 46. SCORE MODIFIERS + CLASSICAL BONUSES + ENTRY THRESHOLD MATRIX (N4)

### 46.1 Ce face

Expune ~50 de valori numerice (anterior hardcoded in `strategy.rs`) ca parametri configurabili in UI. Organizate in 3 card-uri noi intre STRATEGY SCORING si ENGINE STOPS.

### 46.2 Card: SCORE MODIFIERS (header portocaliu)

21 slidere care controleaza direct cum `calculate_score()` si `check_ta_gates()` computa scorul de entry:

| Slider | Backend Field | Default | Efect |
|--------|--------------|---------|-------|
| Chaos Kill Penalty | `sm_chaos_kill_penalty` | -50 | Penalty cand confidence < chaos threshold |
| Chaos Kill Fallback | `sm_chaos_kill_fallback` | 50 | Scor returnat cand oracle lipsa |
| Chaos Conf x MeanRev | `sm_chaos_conf_mult_mean_rev` | 1.5 | Multiplicator chaos threshold pe MeanRev |
| Chaos Conf x Chaotic | `sm_chaos_conf_mult_chaotic` | 2.5 | Multiplicator pe Chaotic |
| Chaos Conf x Unknown | `sm_chaos_conf_mult_unknown` | 1.5 | Multiplicator pe Unknown |
| Regime x MeanRev | `sm_regime_mult_mean_rev` | 0.82 | Oracle contribution weight pe MeanRev |
| Regime x Chaotic | `sm_regime_mult_chaotic` | 0.55 | Weight pe Chaotic |
| Regime x Unknown | `sm_regime_mult_unknown` | 0.73 | Weight pe Unknown |
| VPIN Penalty High | `sm_vpin_penalty_high` | -20 | Cand VPIN > scalping gate |
| VPIN Penalty Med | `sm_vpin_penalty_medium` | -10 | Cand VPIN > normal gate |
| FVG Bull Bonus | `sm_fvg_bullish_bonus` | +15 | FVG bullish detectat |
| FVG Bear Penalty | `sm_fvg_bearish_penalty` | -15 | FVG bearish detectat |
| RVOL Tier1 Thr. | `sm_rvol_tier1_threshold` | 2.0 | Prag volume tier 1 |
| RVOL Tier1 Bonus | `sm_rvol_tier1_bonus` | +8 | Bonus RVOL > 2x |
| RVOL Tier2 Thr. | `sm_rvol_tier2_threshold` | 4.0 | Prag volume tier 2 |
| RVOL Tier2 Bonus | `sm_rvol_tier2_bonus` | +12 | Bonus RVOL > 4x |
| StochRSI OB | `sm_stoch_rsi_overbought` | 80 | Prag overbought |
| StochRSI OS | `sm_stoch_rsi_oversold` | 20 | Prag oversold |
| BB Squeeze Width | `sm_bb_squeeze_width` | 0.02 | Prag squeeze detection |
| Strong Label | `sm_entry_label_strong` | 80 | Score > prag = "STRONG" |
| Mild Label | `sm_entry_label_mild` | 60 | Score > prag = entry normal |

### 46.3 Card: CLASSICAL BONUSES (header verde)

6 slidere care controleaza bonusurile din `compute_classical_confirmation()`. Card-ul apare semi-transparent (opacity 0.5) cand Classical Filters e OFF.

| Slider | Backend Field | Default | Efect |
|--------|--------------|---------|-------|
| RSI Bonus | `cb_rsi_bonus` | 10.0 | Bonus confirmare RSI |
| EMA Align Bonus | `cb_ema_alignment_bonus` | 15.0 | Bonus EMA20/50 alignment |
| ADX Bonus | `cb_adx_bonus` | 10.0 | Bonus trend strength |
| BB Squeeze Bonus | `cb_bb_squeeze_bonus` | 5.0 | Bonus Bollinger squeeze |
| VWAP Bonus | `cb_vwap_bonus` | 10.0 | Bonus pret vs VWAP |
| Cap | `cb_cap` | 50.0 | Limita maxima bonusuri |

### 46.4 Card: ENTRY THRESHOLD MATRIX (header cyan)

Grid 7 regimuri x 3 valori (Long, Short, Confidence) cu inputuri numerice directe. Dropdown Scalping/Normal comuta intre cele doua seturi.

**Regimuri:** Trending, Trending Aggro, MeanRev, MeanRev Aggro, Chaotic, Unknown, Unknown Aggro.

Interpretare:
- **Long threshold**: oracle_score trebuie sa fie > aceasta valoare pentru BUY
- **Short threshold**: oracle_score trebuie sa fie < aceasta valoare pentru SHORT
- **Confidence threshold**: oracle_confidence trebuie sa fie > aceasta valoare

Regime Chaotic nu are varianta Aggro (e deja maxim restrictiv).

### 46.5 VALIDATE button

Apeleaza `invoke('validate_thresholds')` si verifica:
- long > 0, short < 0, conf in [0.05, 0.95]
- Scalp thresholds <= Normal (scalping = mai permisiv)
- Trending <= Chaotic (gradient natural de restrictivitate)
- StochRSI OB > OS, RVOL tier1 < tier2

Warnings afisate ca toasts (nu blocheaza save-ul).

### 46.6 RESET DEFAULTS button

Restaureaza toate valorile la defaults pentru modul curent selectat (Scalp sau Normal). Salveaza automat pe backend.

### 46.7 Persistenta

Toate valorile sunt salvate in `AppSettings` / `titan_settings.json`. Entry thresholds ca `HashMap<String, EntryThreshold>`, score modifiers si classical bonuses ca struct-uri cu `#[serde(default)]`. Settings-uri vechi se incarca corect (defaults inserate automat).

### 46.8 Relatie cu alte sectiuni

- **§19 Strategy Scoring**: Base Score, VPIN Gates, RVOL, Chaos Kill -- acesti parametri controleaza `calculate_score()` impreuna cu Score Modifiers (§46.2)
- **§39 TA Entry Gates**: StochRSI OB/OS si BB Squeeze Width din Score Modifiers afecteaza si TA gates
- **§45 Microstructure**: Microstructure modifiers (spread narrowing, VPIN divergence) se aplica DUPA scoring-ul controlat de §46

---

## 47. ORACLE WEIGHT ADAPTATION (S15)

### 47.1 Toggle

| Proprietate | Valoare |
|-------------|---------|
| **Setting** | `enable_oracle_adaptation` |
| **Default** | OFF |
| **Ce face** | Activeaza auto-tuning-ul ponderilor Oracle pe baza PnL-ului trade-urilor |

Cand e OFF, ponderile din §21 se folosesc direct (comportament identic cu cel anterior). Cand e ON, ponderile se adapteaza automat per-regime.

### 47.2 Cum functioneaza adaptarea

Oracle-ul are 5 ponderi directionale (§21.1): Kalman, Entropy, Hurst, VPIN, Wavelet. Cu adaptarea activa:

1. La fiecare intrare in trade, se captureaza cele 5 semnale directionale ale Oracle-ului si regimul pietei
2. La fiecare iesire, se coreleaza semnalele cu directia PnL (profit = +1, loss = -1)
3. La fiecare 50 trades, ponderile se recalculeaza proportional cu corelatia absoluta per layer

### 47.3 Per-Regime Adaptation

Adaptarea mentine **4 seturi separate** de ponderi, cate unul per `MarketRegime`:

| Regim | Descriere | Pondere dominanta tipica |
|-------|-----------|--------------------------|
| Trending | Piata directionala | Kalman (predictor directional) |
| MeanReverting | Piata oscilanta | Hurst (persistence signal) |
| Chaotic | Piata dezordonata | Ponderile tind spre uniform (nici un layer nu e fiabil) |
| Unknown | Regim nedeterminat | Ponderile raman aproape de base |

La entry, se folosesc ponderile adaptate pentru regimul CURENT al pietei (din ultimul ciclu cached).

### 47.4 Confidence-Weighted Learning

Learning rate-ul (alpha) variaza cu confidence-ul Oracle la entry:

| Confidence | Alpha efectiv | Efect |
|------------|--------------|-------|
| 0.95 | 0.095 | Contribuie maxim la learning |
| 0.50 | 0.050 | Contribuie mediu |
| 0.10 | 0.010 | Contribuie minim (filtrat) |

Trade-uri unde Oracle a fost nesigur nu polueaza adaptarea.

### 47.5 Safety: Caps si Decay

| Mecanism | Valoare | Efect |
|----------|---------|-------|
| Cap inferior | 0.5x baza | O pondere nu scade sub jumatate din slider |
| Cap superior | 2.0x baza | O pondere nu creste peste dublu fata de slider |
| Normalizare | sum = 1.0 | Dupa fiecare recalc, suma = 1.0 |
| Continuous decay | 0.997/trade | Half-life ~230 trades, previne overfitting |

### 47.6 Interactiunea cu sliderul Oracle Fusion (§21)

Sliderul seteaza **ponderile de baza**. Adaptarea le modifica relativ:
- Slider Kalman W = 0.30 + adaptare -> domeniu efectiv [0.15, 0.60]
- Schimbarea slider-ului schimba si domeniul adaptarii
- Cu adaptarea OFF, sliderul controleaza direct (identic pre-S15)

### 47.7 Persistenta

Datele de adaptare (corelatii, ponderi adaptate, trade count) se salveaza in `titan_state_snapshot.json`. La restart, se restaureaza automat. Semnalele in-flight (trade-uri deschise la restart) nu contribuie la adaptare.

### 47.8 Relatie cu alte sectiuni

- **§21 Oracle Fusion**: Ponderile din §21 devin "base weights" cand adaptarea e activa
- **§19 Strategy Scoring**: Scorul de strategie foloseste Oracle cu ponderi adaptate (daca activ)
- **§44 Feature Store**: `oracle_score` si `oracle_confidence` din Feature Store reflecta Oracle-ul cu ponderi adaptate
- **§46 Entry Thresholds**: Threshold-urile raman fixe; doar ponderile Oracle se schimba

---

## 48. EXIT INFLIGHT GUARD (S39-S40)

### 48.1 Ce este

Guard de deduplicare per-simbol pe calea de exit a autopilotului care previne trimiterea repetata de ordine de vanzare pentru aceeasi pozitie. Rezolva un bug critic de tip "runaway exit loop" — cand broker-ul nu reflecteaza inca inchiderea pozitiei, autopilotul trimitea ordine duplicate de SELL la fiecare tick (~2-3 secunde), fiecare fie deschizand o pozitie short accidentala, fie generand erori 429 Too Many Requests.

Intarit in S40 cu guard strict (fara TTL), verificare broker open-orders, PendingClose lifecycle, si suprimare pozitii sintetice.

### 48.2 Cum functioneaza

Camp pe `TitanState`:

```rust
pub exit_inflight: Mutex<HashMap<String, u64>>
```

Doua straturi de protectie:

| Strat | Moment | Actiune |
|-------|--------|---------|
| 1. Inflight strict | `evaluate_position_exit()` pe fiecare tick | Daca `exit_inflight.contains_key(symbol)` → returneaza `None` (skip complet, fara TTL) |
| 2. Broker open-order | Inainte de `close_position()` | `has_pending_orders_for_symbol()` intreaba broker-ul (`GET /v2/orders?status=open`). Daca exista ordin deschis → skip |
| SET | `close_position()` apelat | Seteaza `exit_inflight[symbol] = timestamp_ms` inainte de a trimite ordinul |

Guard-ul ramane activ pana cand **broker-ul confirma ca pozitia e flat** (prin reconciliation) sau o eroare permanenta elimina intrarea. Nu exista timeout — sursa de adevar este broker-ul.

### 48.3 Unde se curata

Guard-ul se curata (remove din HashMap) in **2** locatii:

1. **Position reconciliation** — la fiecare 15s, `exit_inflight.retain` pastreaza doar simboluri inca prezente in pozitiile **brute** ale broker-ului (inainte de filtrare grace/blacklist). Cand broker-ul nu mai raporteaza simbolul, intrarea dispare automat.
2. **Autopilot permanent failure** — la erori 422, "invalid", "does not exist", "not found" — se sterge pentru a nu bloca permanent.

**Nu se curata in:**
- Autopilot exit success — pozitia e marcata `PendingClose`, guard-ul ramane activ
- Trade updates fill complet — fill-ul local nu reseteaza guard-ul (broker REST poate inca raporta pozitia cateva secunde dupa fill)

### 48.4 Relatie cu alte sectiuni

- **§15 Autopilot Core**: Guard-ul actioneaza in interiorul loop-ului autopilot, pe calea de exit (intre `fetch_positions` si `close_position`). Nu afecteaza parametrii din §15 (min hold, consensus, loop ms, etc.)
- **§42 Concurrent Autopilot Architecture**: Guard-ul se verifica in Phase 1 (`evaluate_position_exit`, read-only). Clear-ul se face in Phase 2 (sequential apply). Nu introduce lock contention suplimentar — `exit_inflight` e un Mutex separat, independent de `settings` lock.
- **§16 Exit Scaling / §17 Oracle Exit**: Aceste mecanisme decid DACA sa inchida. Guard-ul inflight decide daca decizia de inchidere sa fie ACTIONATA sau suprimata temporar. Logica de exit ramane neschimbata.

### 48.5 PendingClose la Exit Success

La success-ul `close_position`, autopilotul marcheaza pozitia `PendingClose` cu `closed_at` setat, in loc sa o stearga din `settings.positions`. Asta previne doua probleme:

1. **Pozitie sintetica falsa** — fara PendingClose, cand fill-ul ajunge pe WebSocket, `trade_updates` nu gasea pozitia si crea una noua de tip Sell (falsa pozitie short)
2. **Re-inserare de catre recon** — reconciliation putea re-insera pozitia ca `Open` dupa expirarea grace-ului, ignorand faptul ca autopilotul o inchisese

Starea `PendingClose` este pastrata prin merge-ul din `position_recon` (linia `new_pos.state = existing.state.clone()`). Pozitia dispare natural cand broker-ul nu o mai raporteaza.

### 48.6 Suprimare Pozitie Sintetica

Cand un fill ajunge pe WebSocket pentru un simbol care NU exista in `settings.positions`, `trade_updates` verifica `exit_inflight`:

- Daca `exit_inflight.contains_key(symbol)` → fill-ul este de **inchidere** (nu de deschidere), **nu se creeaza pozitie noua**
- Daca simbolul NU e in `exit_inflight` → fill-ul este de **deschidere** legitima, se creeaza pozitia normal

Aceasta logica previne aparitia unei pozitii-fantoma de tip Sell cand fill-ul de closing ajunge dupa ce autopilotul a marcat pozitia PendingClose sau dupa ce a fost stearsa de recon.

### 48.7 ORDER ACCEPTED — Log Corect

`send_order_internal` emite acum `ORDER ACCEPTED` (nu `ORDER FILLED`) pe raspunsul HTTP 200. HTTP 200 inseamna ca broker-ul a acceptat ordinul, nu ca s-a executat. Fill-urile reale apar ca `FILL: SELL LAR x212.0000 @ $X.XX [fill]` (emise de `trade_updates` pe WebSocket).

---

## 49. Safety Gates — VaR Fail-Closed, Asset Check, Intraday Blackout (S04.1)

Panel `⬡ SAFETY GATES` in HFT Engine tab, sub Portfolio Risk Gates.

### 49.1 Parametri

| Parametru | ID Element | Setting Backend | Default | Range |
|-----------|-----------|-----------------|---------|-------|
| VaR Fail-Closed | `sg-var-fail-closed` | `var_fail_closed` | ON | bool |
| Asset Check Hard Gate | `sg-asset-check-hard` | `asset_check_hard_gate` | ON | bool |
| Blackout First (min) | `sg-blackout-first` | `blackout_first_mins` | 5 | 0-60 |
| Blackout Last (min) | `sg-blackout-last` | `blackout_last_mins` | 5 | 0-60 |
| Blackout Exempt Symbols | `sg-blackout-exempt` | `blackout_exempt_symbols` | empty | comma-sep |
| Min Dollar Volume | `sg-min-dvol` | `scan_min_dollar_volume` | $5M | 0-$100M |

### 49.2 VaR Fail-Closed

Cand activat (default), daca nu exista date suficiente de returns (niciun simbol cu >= 2 perioade), `var_gate_blocks_entry()` returneaza `true` (blocheaza entry). Previne intrari in conditii de incertitudine a riscului. Cand dezactivat, lipsa datelor permite entry (comportament legacy fail-open).

### 49.3 Asset Check Hard Gate

Cand activat (default), daca HTTP request-ul catre `/v2/assets/{symbol}` esueaza (timeout, network error), ordinul este **respins** cu eroare. Cand dezactivat, esecul HTTP genereaza doar un warning si ordinul continua (soft fail).

### 49.4 Intraday Blackout

Previne intrari noi in primele `blackout_first_mins` minute (dupa 9:30 ET) si ultimele `blackout_last_mins` minute (inainte de 16:00 ET) ale sesiunii. Nu afecteaza exit-urile. DST-aware via `is_us_dst()`.

**Blackout Exempt Symbols:** Lista comma-separated de simboluri (ex: `SPY, QQQ`) care bypasseaza blackout-ul. Case-insensitive.

### 49.5 Blackout Status Badge

Badge neon-orange cu animatie `glow-breathe` afisat in 2 locatii:
- Header-ul HFT Engine (langa DRY/LIVE badge)
- Sidebar STATUS

Afiseaza countdown-ul minutelor ramase. Se ascunde automat cand blackout-ul nu este activ. Refresh: 15 secunde, calculat client-side.

### 49.6 VaR Gate Event + Toast

Cand VaR gate blocheaza o entry, emite event Tauri `var-gate-blocked` cu:
```json
{ "symbol": "AAPL", "reason": "portfolio VaR would exceed threshold", "max_var_pct": 2.0, "fail_closed": true }
```
Frontend: `TitanToast.show()` de tip warning cu simbol si motiv (4s display).

### 49.7 Settings Validation

`validate_safety_settings()` verifica:
- `blackout_first_mins + blackout_last_mins >= 195` → warning (>= jumatate din ziua de trading)
- `min_dollar_volume < $100K` → warning (risc illiquid tickers)
- `enable_var_gate=true` + `var_fail_closed=false` → warning (risc gap: entries fara date)

Warnings sunt logate si incluse in response-ul `batch_update_settings`.

### 49.8 Gate Chain complet (actualizat)

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

In sidebar STATUS, 4 mini-dots cyber-HUD arata sanatatea sub-state-urilor in timp real:

| Dot | Sub-state | Campuri monitorizate |
|-----|-----------|---------------------|
| **TRD** | TradingState | 10 Mutex (trail, entry metadata, exit, blacklist) |
| **MKT** | MarketDataState | 5 Mutex (prices, candles, indicators, vol, bars) |
| **RSK** | RiskState | 3 Mutex (analytics, drawdown, flash crash) |
| **SCN** | ScanState | 5 Mutex (best-TF, results, progress, prefilter, cancel) |

**Culori:**
- Verde = toate Mutex-urile libere (normal)
- Galben = 1 Mutex contended (lock contention normala sub load)
- Rosu = 2+ Mutex-uri contended (potential bottleneck)

**Hover:** Tooltip cu detalii (locks free / contended, session generation).

**Backend:** Comanda Tauri `get_state_health` -- try_lock sampling pe toate 23 Mutex-urile. Poll la 10 secunde.

---

## 51. SMARTTREND GENETIC OPTIMIZER (S13)

SmartTrend dispune de un **optimizer genetic dedicat** (`StGeneticOptimizer`), complet independent de optimizatorul Fusion clasic. Cand `StrategyMode::SmartTrend` este activ, `genetic_evolution_loop` comuta automat pe SmartTrend evolution.

### Parametri Evolvabili (12)

| Parametru | Min | Max | Ce controleaza |
|-----------|-----|-----|---------------|
| macro_weight | 0.05 | 0.60 | Ponderea pilon MACRO |
| micro_weight | 0.05 | 0.60 | Ponderea pilon MICRO |
| momentum_weight | 0.05 | 0.60 | Ponderea pilon MOMENTUM |
| entry_threshold | 0.50 | 0.95 | Scor minim pentru ENTRY |
| stalking_threshold | 0.30 | 0.80 | Scor minim pentru STALKING |
| changepoint_reset | 1.0 | 10.0 | % volatilitate pt reset FSM |
| trail_tighten_pct | 0.10 | 0.80 | Cat de tight devine trail stop |
| hysteresis_margin | 0.01 | 0.20 | Dead zone intre stari FSM |
| bayesian_halflife | 3 | 30 | Halflife tracker bayesian |
| stabilization_window | 5 | 50 | Ferestra stabilizare dupa regime shift |
| dominant_cycle_filter | 5 | 50 | Filtru ciclu dominant |
| risk_per_trade | 0.002 | 0.030 | Risk per trade (fractie) |

Ponderile (macro/micro/momentum) sunt auto-normalizate la suma = 1.0 dupa mutatie/crossover.

### Walk-Forward Backtest SmartTrend

Ruleaza FSM-ul real (Flat → Stalking → Entry → InTrade → Trail) pe date istorice cu IS/OOS split 70/30. Calculeaza PnL, Sharpe, Profit Factor, Max Drawdown, Win Rate, Trade Count, Avg Bars In Trade.

### Fitness Formula

```
fitness = sharpe * min(PF, 5.0) + trade_count_bonus + win_rate_bonus + stability_bonus - dd_penalty
```

- **trade_count_bonus**: +0.5 cand > 5 trades
- **win_rate_bonus**: +(win_rate - 0.55) * 2 cand > 55%
- **stability_bonus**: +0.3 cand avg_bars_in_trade intre 3-30
- **dd_penalty**: max_dd * 5 cand > 20%
- **OOS validation**: champion TREBUIE sa aiba sharpe > 0 si total_pnl > 0 pe 30% out-of-sample

### UI Components

| Element | Locatie | Ce arata |
|---------|---------|----------|
| Convergence Chart | EVOLUTION panel canvas | IS fitness (cyan) + OOS fitness (gold dashed) pe generatii, dots verzi la champion promotion, diversity bar |
| PARAMS button | EVOLUTION panel header | Modal drill-down cu toate 12 params, delta% vs defaults, heat-color bars |
| DIV badge | EVOLUTION panel header | Diversity % populatie. Rosu+pulse < 15%, cyan 15-40%, verde > 40% |
| Comparison Panel | tab-13 SmartTrend CFG | RUN COMPARISON: ruleaza Fusion + SmartTrend pe aceleasi candle, grid cu winner highlight |

### Comenzi Tauri

| Comanda | Returneaza |
|---------|-----------|
| `get_st_evolution_stats` | Gen, fitness IS/OOS, sharpe, diversity, champion params |
| `get_st_fitness_history` | Vec<{gen, is_fitness, oos_fitness, diversity, promoted}> |
| `get_st_champion_params` | Params cu value, default, delta%, min, max |
| `run_comparison_backtest_cmd` | ComparisonResult: {fusion: BacktestResult, smarttrend: StBacktestResult, verdict} |
| `freeze_st_evolution` | Opreste evolutia SmartTrend |
| `resume_st_evolution` | Porneste evolutia SmartTrend |

---

---

## 52. EXECUTION RESILIENCE (S14, Sesiunea 63)

### Order Lifecycle

- **client_order_id**: UUID v4 generat local per ordin
- **broker_id**: Parsat din raspunsul Alpaca (`id` field)
- **Retry**: 1-2x automat pentru erori tranziente (5xx, timeout)
- **Event**: `order-lifecycle` emis la submit cu client_order_id, broker_id, status, retry_count

### Pending Order Watchdog

- Tracked local in `TradingState::pending_orders` (Vec<PendingOrder>)
- Auto-cancel dupa `pending_order_timeout_secs` (default 120s) sau `generation_id` mismatch
- Eliminare din tracking la fill/partial_fill/rejected/canceled

### Health Observability

| Event | Trigger | Badge UI |
|-------|---------|----------|
| `equity-sync-degraded` | Erori consecutive equity sync | EQUITY STALE (gold+pulse) |
| `equity-sync-critical` | Erori critice equity sync | EQUITY STALE (gold+pulse) |
| `trade-updates-health` | WS connect/disconnect | WS DISCONN (rosu+pulse) |

### Cockpit Strip

Bara persistenta sub header principal cu: EQUITY, P&L, POS count, MODE (auto/manual), BREAKER status, GEN (generation id), health badges. Poll 5s + event-driven badges. Badges noi (S71): TMR:N (timere active), MRK:S/F/T (chart markers per sursa), HP:N% (health real cu latenta + fill rate).

### Bounded Channels

`mpsc::channel(256)` inlocuieste `unbounded_channel` pe streamer. `try_send` cu overflow warning.

### HHI Notional Fix

`qty * live_price` in loc de `qty * entry_price`. Log structurat: `hhi_pre`, `hhi_post`, `blocked`.

### Exit Features

`exit_features: Option<TradeFeatures>` pe TradeRecord — snapshot conditii piata la close trade.

### UI Bonuses

| Feature | Descriere |
|---------|-----------|
| Live Fill Markers | BUY/SELL triangles pe chart, persistent, 3s refresh |
| Journal Min P&L | Input filter in journal bar |
| Time-in-Trade | Coloana TIME pe positions (dim/cyan/gold/orange) |

### §53: S16 UX Polish — HFT-Relevant Changes

**Fill Quality Columns:** 3 coloane noi in HFT Targets table — SLIP (avg slippage bps, color-coded), T2F (avg time-to-fill), TRADES (count per symbol). Backend `get_fill_quality_stats`. Polling 30s.

**Unified Chart Markers:** `_titanMarkerEngine` merge-uieste fills+signals+SmartTrend, debounce 100ms, cap 50 fills, pruning 24h. API publica: `update(source, markers)`, `clear(source)`, `stats()`, `getSource(name)`. Overlay pe chart: `SIG:N FILL:N ST:N TOT:N`.

**Latency Histogram Rolling:** Ring buffer 300 samples (5 min window) in loc de cumulative.

**Health Score Badge:** `HP:XX%` in cockpit — 6 sub-scores composite. Polling 10s.

**Cockpit Strip V2:** GEN sync real, BREAKER initial poll, W/R badge, UPTIME badge.

*Manual creat: 28 Martie 2026 — Sesiunea 19. Actualizat: 3 Aprilie 2026 — Sesiunea 71 (cockpit strip badges TMR/MRK/HP, unified marker engine API, chart marker overlay). Acoperire completa: ~155+ parametri configurabili, 28 smart features, 53 sectiuni.*

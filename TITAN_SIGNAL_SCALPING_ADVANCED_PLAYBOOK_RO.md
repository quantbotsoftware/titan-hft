# TITAN SIGNAL SCALPING ADVANCED PLAYBOOK (1m-4h)

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


Acest document este versiunea avansata pentru trading manual in `SIGNAL`, fara dependenta de bot.
Este gandit pentru executie disciplinata pe toate timeframe-urile Alpaca intre `1m` si `4h`.

Obiectivul principal:

- cresterea calitatii intrarilor
- reducerea drawdown-ului
- protocol clar pentru iesiri la `TRAIL` si `CHAOS`
- decizii rapide, dar standardizate

---

## 1) Model mental corect: ce inseamna semnalele

Semnalele nu sunt predictii certe. Sunt "trigger probabilistic + management".

- `BUY` / `SELL` = trigger de intrare
- `TRAIL` / `EXIT` / `CHAOS` = trigger de management / iesire

Prefixele pe chart:

- `O` = Oracle
- `G` = GPU
- `F` = Fusion

Regula critica:

- intrarea se ia pe `BUY/SELL`
- iesirea se executa pe `TRAIL/EXIT/CHAOS`
- nu inversa automat pozitia la primul semnal de iesire

---

## 2) MTF stack: metoda profesionala pe 3 niveluri

Nu tranzactiona "single timeframe". Foloseste 3 niveluri:

- `Anchor TF` (context directie)
- `Trigger TF` (semnal)
- `Execution TF` (intrare efectiva)

Preset recomandat:

- pentru 1m execution: anchor=15m, trigger=5m
- pentru 3m execution: anchor=30m, trigger=10m
- pentru 5m execution: anchor=30m, trigger=15m
- pentru 10m execution: anchor=1h, trigger=15m
- pentru 15m execution: anchor=1h, trigger=30m
- pentru 30m execution: anchor=4h, trigger=1h
- pentru 1h execution: anchor=4h, trigger=1h
- pentru 4h execution: anchor=1d (context), trigger=4h

Regula:

- daca anchor este clar opus setup-ului, sari trade-ul
- daca trigger si execution sunt aliniate, trade-ul are prioritate

---

## 3) Selectia sursei (O/G/F) in functie de regim

### 3.1 Trending curat

- primar: `FUSION`
- secundar: `ORACLE`
- `GPU` doar pentru timing fin al intrarii

### 3.2 Mean reverting

- primar: `ORACLE`
- secundar: `FUSION`
- `GPU` cu size redus

### 3.3 Chaotic / high noise

- primar: niciuna pentru agresivitate
- fallback: `ORACLE` cu risc minim
- evita overtrading pe `GPU`

Practic:

- daca vezi multe `CHAOS` in serie, redu frecventa si size

---

## 4) Entry Quality Score (0-100) - nu intra sub prag

Calculeaza rapid un scor intern inainte de click:

- semnal proaspat pe bara inchisa: 0-20
- aliniere cu Anchor TF: 0-20
- structura pret (HH/HL sau LH/LL): 0-15
- volatilitate normala (nu spike extrem): 0-10
- fara `CHAOS` in ultimele 2 bare: 0-15
- distanta stop rezonabila (ATR): 0-10
- risc/reward minim 1.8: 0-10

Prag de executie:

- `>= 70`: executie normala
- `60-69`: size redus (50%)
- `< 60`: no trade

---

## 5) Position sizing avansat (risk-first)

Formula simpla:

`qty = (equity * risk_pct) / abs(entry - stop)`

Setari recomandate:

- 1m-5m: `risk_pct` 0.20% - 0.40%
- 10m-30m: `risk_pct` 0.35% - 0.60%
- 1h-4h: `risk_pct` 0.50% - 0.80%

Hard limits:

- max daily loss: 2.0% equity
- max 3 pierderi consecutive -> pauza 30-60 min
- nu dubla size dupa pierdere

---

## 6) Exit Decision Tree (obligatoriu)

Ordine de prioritate:

1. invalidare hard (stop tehnic) -> iesire imediata
2. `CHAOS` -> iesire defensiva rapida
3. `TRAIL` -> lock profit
4. semnal opus confirmat -> inchidere rest

Model partial exits (recomandat):

- TP1: 40%
- TP2: 30%
- rest 30% gestionat cu `TRAIL`

Cand apare `CHAOS` dupa intrare:

- pe 1m/3m/5m: inchidere 70-100%
- pe 10m+: minim reducere 50%

---

## 7) Reguli de executie pe sesiuni

### Open US (primele 20-30 min)

- volatilitate mare, spread variabil
- foloseste size redus la primele 2 intrari

### Mid-session

- de obicei cea mai buna zona pentru executii curate
- urmeaza strict protocolul MTF

### Ultima ora

- evita intrari noi pe semnale marginale
- prefera managementul pozitiilor existente

---

## 8) Anti-patterns care distrug performanta

- schimbi `SRC` in mijlocul trade-ului
- intri pe bara neinchisa din FOMO
- tratezi fiecare `SELL` ca short obligatoriu
- ignori `CHAOS` pentru "poate revine"
- overtrade dupa 2-3 pierderi

---

## 9) KPI minim pentru control real al performantei

Monitorizeaza separat pe `O/G/F` si pe fiecare TF:

- win rate
- expectancy (R/trade)
- average win / average loss
- max adverse excursion (MAE)
- max favorable excursion (MFE)
- hold time mediu
- procent iesiri pe `TRAIL` vs `CHAOS`

Ce cauti:

- sursa care produce expectancy pozitiv constant
- TF-uri unde `CHAOS` apare prea des (zgomot mare)

---

## 10) Protocol saptamanal de tuning (fara improvizatie)

O data pe saptamana:

1. extrage ultimele 50-100 trades
2. segmenteaza pe sursa (`O/G/F`) si TF
3. marcheaza trades cu scor intrare <70 (ar trebui eliminate)
4. ajusteaza doar 1-2 variabile, nu tot sistemul odata
5. ruleaza 1 saptamana aceeasi versiune

Regula:

- nu tunezi pe baza unui singur trade

---

## 11) Idei noi (roadmap practic)

Mai jos sunt idei concrete care merita implementate:

### 11.1 Signal Health Panel in UI

Afiseaza live:

- source activ (`O/G/F`)
- `BUY count`, `SELL count`, `TRAIL count`, `CHAOS count` pe ultimele N bare
- starea feed: `LIVE`, `STALE`, `0 markers`

### 11.2 Confidence ribbon pe chart

Overlay discret cu "quality zone":

- verde: setup bun
- galben: setup mediu
- rosu: evita intrarea

### 11.3 Consensus mode optional

Intrare doar daca 2/3 surse confirma directia in fereastra de timp scurta.
Poate reduce frecventa, dar creste robustetea.

### 11.4 Auto cooldown dupa CHAOS

Dupa `CHAOS`, blocheaza intrari noi X bare pe acelasi simbol/TF.

### 11.5 News risk guard

Fereastra de protectie in jurul evenimentelor majore (de ex. +/- 5 min):

- reduce size automat
- sau blocheaza intrarile noi

### 11.6 Slippage-aware execution hints

In UI:

- "spread high -> reduce size"
- "execution quality low -> skip"

### 11.7 Session templates

Profiluri preset:

- Open Aggressive
- Mid-session Balanced
- Late-session Defensive

---

## 12) Checklist ultra-rapid (10 secunde)

Inainte de intrare:

- [ ] MODE = SIGNAL
- [ ] SRC fixat (nu il schimb in trade)
- [ ] semnal pe bara inchisa
- [ ] Entry Score >= 70
- [ ] Anchor TF nu este clar opus
- [ ] risc/trade in limite
- [ ] stiu exact trigger-ul de iesire (`TRAIL` / `CHAOS`)

Daca lipseste un punct: `NO TRADE`.

---

## 13) Script operational pentru decizie (mental)

Intrebare 1: "Am semnal valid de intrare?"

- nu -> astept
- da -> intrebarea 2

Intrebare 2: "Setup-ul e aliniat MTF si are scor >= 70?"

- nu -> skip
- da -> intrebarea 3

Intrebare 3: "Risc controlat + plan de iesire clar?"

- nu -> skip
- da -> executie

---

## 14) Concluzie

Edge-ul real vine din:

- selectie mai buna a intrarilor
- executie disciplinata a iesirilor
- control strict al riscului

Nu din "mai multe semnale".

Foloseste semnalele ca instrument de executie, nu ca promisiune.


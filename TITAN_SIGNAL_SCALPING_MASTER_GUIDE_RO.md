# TITAN SIGNAL SCALPING MASTER GUIDE (1m-4h)

> **Disclaimer:** Material exclusiv educațional și de cercetare — **nu reprezintă consultanță financiară sau de investiții**. Tranzacționarea implică risc substanțial de pierdere; rezultatele trecute sau simulate nu garantează rezultate viitoare. Titan HFT este oferit AS-IS, fără nicio răspundere și fără asistență. Nu sunt incluse chei API.


Acest document este un ghid complet pentru trading manual in TITAN HFT folosind modul `SIGNAL` pe timeframe-uri intre `1m` si `4h`.

Scopul este sa ai un protocol clar, repetabil, cu reguli simple pentru intrari/iesiri, reducerea pierderilor si interpretarea corecta a markerelor.

---

## 1) Ce reprezinta semnalele din chart

In modul `SIGNAL`, marker-ele sunt afisate pentru sursa activa (`SRC`):

- `O ...` = Oracle
- `G ...` = GPU
- `F ...` = Fusion

Tipuri de semnal:

- `BUY` = semnal de intrare long
- `SELL` = semnal de intrare short / iesire din long
- `TRAIL` = semnal de trailing exit (protectie profit)
- `EXIT` = semnal de inchidere (flip logic)
- `CHAOS` = semnal defensiv (regim instabil, risc crescut)

Important:

- `BUY/SELL` = semnale de intrare
- `TRAIL/EXIT/CHAOS` = semnale de management si iesire
- Nu interpreta `TRAIL`/`CHAOS` ca intrare inversa automata.

---

## 2) Interpretare corecta pe lumanari albastre/rosii

Multi traderi fac confuzia asta:

- Culoarea lumanarii spune doar ce a facut bara curenta.
- Marker-ul spune ce faci cu pozitia ta.

De aceea:

- poti avea `G CHAOS` pe lumanare albastra (este normal)
- poti avea `G TRAIL` pe lumanare rosie (este normal)

Explicatie:

- `CHAOS` nu inseamna neaparat directie opusa imediata, inseamna "piata e instabila, redu risc"
- `TRAIL` inseamna "am avut miscare favorabila, acum protejeaza profitul"

---

## 3) Diferente intre surse (O/G/F)

### Oracle (`O`)

- Mai filtrat, mai conservator.
- Mai putine semnale, de obicei mai curate.

### GPU (`G`)

- Mai sensibil, mai rapid, reactioneaza repede.
- Poate genera mai multe semnale in zone volatile.

### Fusion (`F`)

- Mix intre stabilitatea Oracle si viteza GPU.
- De obicei bun ca varianta "balanced".

Recomandare:

- Nu schimba sursa in mijlocul aceluiasi trade.
- Alege sursa inainte de intrare si tine acelasi "source context" pana la iesire.

---

## 4) Workflow standard pentru fiecare sesiune

1. Ruleaza `build.sh` si porneste aplicatia.
2. Selecteaza simbolul si timeframe-ul dorit.
3. Seteaza `MODE = SIGNAL`.
4. Alege `SRC` (`ORACLE` / `GPU` / `FUSION`).
5. Verifica toggle `Signals` activ in UI.
6. Asteapta semnal de intrare (`BUY`/`SELL`) pe bara inchisa.
7. Gestioneaza iesirea cu `TRAIL`/`EXIT`/`CHAOS`.

Nu sari peste pasul 6. Intrarea pe bara neinchisa creste semnificativ rata de false entries.

---

## 5) Playbook pe timeframe-uri (1m -> 4h)

## 5.1) TF rapide: 1m / 3m / 5m

- Intri doar pe semnal proaspat.
- Dupa `TRAIL`: iesire rapida (minim partial).
- Dupa `CHAOS`: iesire agresiva, fara ezitare.
- Dupa iesire: cooldown manual 1-3 bare.

## 5.2) TF medii: 10m / 15m / 30m

- Intri pe `BUY/SELL` confirmat (bara inchisa).
- `TRAIL` = iesire graduala (de ex: 30-60%).
- `CHAOS` = reduci puternic expunerea.

## 5.3) TF lente: 1h / 4h

- Semnale mai rare, de obicei mai robuste.
- Foloseste `TRAIL` pentru a conserva profitul, nu pentru panica.
- `CHAOS` ramane signal de risc crescut -> defensiv.

---

## 6) Reguli concrete de intrare/iesire

### Reguli intrare LONG

- semnal `BUY` pe bara inchisa
- fara `CHAOS` imediat in 1-2 bare urmatoare
- ideal: structura de pret nu este deja "extins sus" pe spike extrem

### Reguli intrare SHORT

- semnal `SELL` pe bara inchisa
- fara `CHAOS` imediat in 1-2 bare urmatoare
- evita short imediat dupa flush puternic daca apare absorbtie

### Reguli iesire

- `TRAIL`: marchezi profit (partial sau full, in functie de TF)
- `EXIT`: inchidere tehnica (flip logic)
- `CHAOS`: iesire defensiva (de obicei full pe TF mici)
- semnal opus clar: inchizi ce a ramas

---

## 7) Risk management (obligatoriu)

Aceste reguli sunt mai importante decat orice semnal:

- risc per trade mic si fix
- nu cresti size dupa o pierdere
- limita de pierdere pe zi (hard stop)
- limita de pierdere consecutiva (ex: 3 trades la rand -> pauza)

Exemplu practic:

- max risc/trade: 0.25% - 0.75% din equity
- max daily loss: 1.5% - 2.5%
- dupa 3 pierderi consecutive: pauza si reevaluare

---

## 8) Cand sa NU intri

Nu intra daca:

- semnalul apare in mijloc de spike de news fara confirmare
- tocmai ai primit `CHAOS` si nu a trecut cooldown
- schimbi sursa in mijlocul deciziei (context inconsistent)
- intri "din frica sa nu pierzi miscarea"

---

## 9) De ce poti vedea multe SELL si putine BUY (sau invers)

Este normal daca:

- regimul curent este directional (trend dominant)
- filtrele de confidence/HTF elimina o parte din intrari
- volatility state favorizeaza o directie

Nu este bug automat. Bug este cand:

- una din surse ramane blocata constant in `0 markers` pe toate conditiile
- sursa nu produce nimic pe perioade lungi, in timp ce celelalte produc normal

---

## 10) Troubleshooting rapid

### Problema: nu apar markere deloc

Verifica:

- `MODE = SIGNAL`
- `SRC` corect selectat
- toggle `Signals` activ
- timeframe valid (1m, 3m, 5m, 10m, 15m, 30m, 1h, 4h)

### Problema: doar o sursa nu arata semnale

- comuta pe celelalte surse pentru comparatie
- verifica logurile backend
- daca doar `gpu` cade la zero, verifica pipeline-ul GPU/thresholds

### Problema: iconita SHARK pornea build vechi

- `build.sh` trebuie sa actualizeze:
  - binarul local
  - binarul de sistem
  - launcher local
  - launcher de sistem
  - desktop/icon cache

---

## 11) Checklist "before click BUY/SELL"

Inainte de orice intrare:

- [ ] MODE este `SIGNAL`
- [ ] SRC este fixat (O/G/F) si nu il schimb in trade
- [ ] TF ales este clar (1m -> 4h)
- [ ] semnal pe bara inchisa
- [ ] risc/trade calculat
- [ ] stiu exact unde ies daca apare `TRAIL` sau `CHAOS`

Daca un punct lipseste, nu executa trade.

---

## 12) Checklist "after entry"

- [ ] monitorizez urmatoarele 1-3 bare
- [ ] la `TRAIL` reduc / inchid conform plan
- [ ] la `CHAOS` trec in modul defensiv
- [ ] nu mut decizia pe emotional
- [ ] notez rezultatul in jurnal

---

## 13) Jurnal minim de trading (recomandat)

Pentru fiecare trade noteaza:

- data/ora
- simbol
- TF
- sursa (`O/G/F`)
- semnal intrare (`BUY/SELL`)
- semnal iesire (`TRAIL/EXIT/CHAOS/opus`)
- rezultat (% si R multiple)
- observatii (de ce a mers / de ce nu a mers)

Dupa 30-50 trades ai date reale pentru tuning.

---

## 14) Principii finale

1. Semnalul nu este certitudine, este probabilitate.
2. Risc fix > "predictie perfecta".
3. Disciplina bate indicatorul.
4. `CHAOS` este prietenul tau la capitolul protectie.
5. Cel mai bun trade ratat este mai bun decat un trade prost executat.

---

## 15) Nota de responsabilitate

Acest sistem nu poate garanta "fara pierdere".
Poate doar sa ofere semnale mai clare si mai rapide.
Rezultatul final depinde de executie, risc, disciplina si contextul de piata.


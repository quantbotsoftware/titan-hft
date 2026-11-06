function setChartStatus(msg, color) {
    const el = $el('chart-status');
    if (!el) return;
    if (!msg) {
        el.style.display = 'none';
        return;
    }
    el.innerText = msg;
    el.style.color = color || '#888';
    el.style.display = 'block';
}

function beginLoadUi(sym) {
    const normalized = normalizeSymbol(sym);
    currentSymbol = normalized || currentSymbol;

    const titleEl = document.querySelector('.chart-title');
    if (titleEl) titleEl.innerHTML = `⚡ Pro Chart (60 FPS) | ${escapeHtml(currentSymbol)} | ${escapeHtml(currentTF)} <span style="color:yellow; margin-left:10px;">CONNECTING...</span>`;

    const symEl = $el('analysis-symbol');
    if (symEl) symEl.innerText = currentSymbol;
    const chartSym = $el('chart-symbol');
    if (chartSym) chartSym.innerText = currentSymbol;

    setChartStatus(`CONNECTING ${currentSymbol} ${currentTF}`, '#8aa');
    updateRightBarHeader();
    updateRightBarFeedStatus("CONNECTING", "warning");
    setRightBarText('rb-last-tick', '--');
    setRightBarText('rb-bid', '--');
    setRightBarText('rb-ask', '--');
    setRightBarText('rb-bid-size', '--');
    setRightBarText('rb-ask-size', '--');
    setRightBarText('rb-spread', '--');
    const l2Top = $el('l2-top');
    if (l2Top) l2Top.innerText = 'BID -- | ASK -- | SPR --';
    updateRightBarLatency(null);
}

function clearChartState() {
    if (candleSeries) candleSeries.setData([]);
    if (volSeries) volSeries.setData([]);
    if (seriesOracle) seriesOracle.setData([]);
    if (seriesOracleZero) seriesOracleZero.setData([]);
    lastCandleData = [];
    window.lastCandleData = [];
    lastBar = null;
    window.lastBar = null;
    livePriceLine = null;
    lastVwap = null;
    signalMarkers = [];
    if (window._titanMarkerEngine) window._titanMarkerEngine.clear();
    if (typeof updateSignalMarkers === 'function') updateSignalMarkers();
    setRightBarText('rb-vwap', '--');
    setChartStatus(`NO DATA: ${currentSymbol} ${currentTF}`, '#aaa');
}

async function fetchAndApplyCandles(mySeq) {
    const res = await invoke("fetch_alpaca_history", { symbol: currentSymbol, timeframe: currentTF });
    if (mySeq !== _loadSeq) return false;

    if (!res || !res.candles) {
        TitanError.report('data:invalid-response', 'Invalid response structure');
        clearChartState();
        updateStatus("❌ DATA FORMAT ERROR. See Console.", "red", "error");
        updateRightBarFeedStatus("ERROR", "error");
        return false;
    }

    if (res.candles.length === 0) {
        if (candleSeries) candleSeries.setData([]);
        if (volSeries) volSeries.setData([]);
        lastCandleData = [];
        window.lastCandleData = [];
        lastBar = null;
        window.lastBar = null;
        livePriceLine = null;
        lastVwap = null;
        signalMarkers = [];
        if (window._titanMarkerEngine) window._titanMarkerEngine.clear();
        if (typeof updateSignalMarkers === 'function') updateSignalMarkers();
        setRightBarText('rb-vwap', '--');
        setChartStatus(`NO DATA: ${currentSymbol} ${currentTF}`, '#aaa');
        updateStatus("⚠️ NO DATA (Empty Array) for " + currentSymbol, "yellow", "warning");
        updateRightBarFeedStatus("NO DATA", "warning");
        return false;
    }

    const data = res.candles.map(c => ({ time: c.time, open: c.open, high: c.high, low: c.low, close: c.close, volume: c.volume || 0 }));
    data.sort((a, b) => a.time - b.time);
    lastCandleData = data;
    window.lastCandleData = lastCandleData;
    if (window.initDrawingManager) window.initDrawingManager();
    if (window.loadDrawings) window.loadDrawings();

    if (data.length > 0) {
        candleSeries.setData(data);
        if (volSeries) {
            volSeries.setData(data.map(d => ({
                time: d.time,
                value: d.volume || 0,
                color: d.close >= d.open ? '#00BFFF' : '#FF2B2B'
            })));
        }
        lastBar = data[data.length - 1];
        window.lastBar = lastBar;
        lastRefLinesLen = 0;
        rightBarState.prevClose = data.length > 1 ? data[data.length - 2].close : data[0].open;
        updateRightBarSessionStats(data);
        updateRightBarFromBar(lastBar, { price: lastBar.close, time: lastBar.time });
        lastAlertPrice = lastBar.close;
        loadAlertsFromStorage();
        updateCandleTimer();
        if (!livePriceLine && candleSeries) {
            livePriceLine = candleSeries.createPriceLine({
                price: lastBar.close,
                color: '#00BFFF',
                lineWidth: 1,
                lineStyle: LightweightCharts.LineStyle.Dotted,
                axisLabelVisible: true,
                title: 'LIVE'
            });
        }
        applyIndicators(data);
        fitSubCharts();
        chart.timeScale().fitContent();
    }

    await updateOracleUI();
    if (mySeq !== _loadSeq) return false;

    if (lastBar) {
        const titleEl = document.querySelector('.chart-title');
        if (titleEl) titleEl.innerHTML = `⚡ Pro Chart (60 FPS) | ${escapeHtml(currentSymbol)} <span style="color:#fff; margin-left:10px;">$${lastBar.close.toFixed(2)}</span>`;
        setChartStatus('', '');
        const feedLabel = isCryptoSymbol(currentSymbol) ? "CRYPTO" : "SIP";
        updateStatus(`🟢 LIVE FEED (${feedLabel})`, "#00BFFF", "success");
        updateRightBarFeedStatus("LIVE", "success");
    }
    return true;
}

function _oracleRefreshMs() {
    const fast = new Set(['1m', '3m', '5m']);
    const mid = new Set(['10m', '15m']);
    if (fast.has(currentTF)) return 5000;
    if (mid.has(currentTF)) return 8000;
    return 15000;
}

function scheduleOracleRefresh() {
    TitanTimers.register('data:oracle-refresh', async () => { await updateOracleUI(); }, _oracleRefreshMs());
}

async function loadData(sym) {
    try {
        const mySeq = ++_loadSeq;
        beginLoadUi(sym);
        await invoke("set_active_symbol", { symbol: currentSymbol });
        if (mySeq !== _loadSeq) return;

        const ok = await fetchAndApplyCandles(mySeq);
        if (ok) scheduleOracleRefresh();
    } catch (e) {
        TitanDebug.log('data', 'Fetch Error:', e);
        clearChartState();
        updateStatus("❌ ERROR: " + normalizeError(e), "red", "error");
        updateRightBarFeedStatus("ERROR", "error");
    }
}
window.loadData = loadData;

let _oracleUpdateInFlight = false;

async function updateOracleUI() {
    if (_oracleUpdateInFlight) return;
    _oracleUpdateInFlight = true;
    const sym = currentSymbol;
    const tf = currentTF;
    try {
        try {
            const result = await invoke("get_signal_markers", { symbol: sym, timeframe: tf });
            if (sym !== currentSymbol || tf !== currentTF) return;
            signalMarkers = (result && result.markers) || [];
            window.__signalMarkerReason = (result && result.reason) || null;
            window.__signalLastUpdateMs = Date.now();
            window.__signalLastErrorMs = 0;
            updateSignalMarkers();
            if (result && result.st_scores && seriesOracle) {
                _renderStScoreOverlay(result.st_scores);
            }
        } catch (e) {
            if (sym !== currentSymbol || tf !== currentTF) return;
            window.__signalLastErrorMs = Date.now();
            TitanDebug.log('data', 'Markers periodic error:', e);
            if (typeof updateSignalHealthPanel === 'function') updateSignalHealthPanel();
        }

        try {
            if (seriesOracle) {
                const oracleResult = await invoke('get_oracle_snapshot', { symbol: sym, timeframe: tf });
                if (sym !== currentSymbol || tf !== currentTF) return;
                if (oracleResult && oracleResult.oracle_scores && oracleResult.timestamps) {
                    const oracleData = [];
                    const zeroData = [];
                    let lastTime = -1;

                    for (let i = 0; i < oracleResult.timestamps.length; i++) {
                        const t = Number(oracleResult.timestamps[i]);
                        if (!isNaN(t) && t > lastTime) {
                            const val = oracleResult.oracle_scores[i];
                            const numVal = (val !== null && val !== undefined && !isNaN(Number(val)))
                                ? Number(val)
                                : 0;
                            oracleData.push({ time: t, value: numVal });
                            zeroData.push({ time: t, value: 0 });
                            lastTime = t;
                        }
                    }

                    if (oracleData.length > 0) {
                        seriesOracle.setData(oracleData);
                        if (seriesOracleZero) seriesOracleZero.setData(zeroData);

                        if (window.lastBar && window.lastBar.time > oracleData[oracleData.length - 1].time) {
                            const paddingPoint = { time: window.lastBar.time, value: oracleData[oracleData.length - 1].value };
                            seriesOracle.update(paddingPoint);
                            if (seriesOracleZero) seriesOracleZero.update({ time: window.lastBar.time, value: 0 });
                        }

                        const nonZero = oracleData.filter(d => d.value !== 0).length;
                        const lastCandleStr = window.lastCandleData?.length > 0
                            ? new Date(window.lastCandleData[window.lastCandleData.length - 1].time * 1000).toISOString()
                            : "N/A";
                        const lastOracleStr = new Date(oracleData[oracleData.length - 1].time * 1000).toISOString();
                        const debugEl = $el('chart-debug-text');
                        if (debugEl) {
                            debugEl.innerText = `[Oracle Debug] Candles: ${window.lastCandleData?.length || 0} END=${lastCandleStr} | Oracle: ${oracleData.length} (${nonZero} active) END=${lastOracleStr}`;
                        }
                    }
                }
            }
        } catch (err) {
            TitanDebug.log('data', 'Oracle periodic error:', err);
        }
    } finally {
        _oracleUpdateInFlight = false;
    }
}

function _renderStScoreOverlay(stScores) {
    if (!seriesOracle || !stScores || stScores.length === 0) return;
    const scoreData = [];
    const zeroData = [];
    let lastTime = -1;
    for (const pt of stScores) {
        const t = Number(pt.time);
        if (!isNaN(t) && t > lastTime) {
            scoreData.push({ time: t, value: pt.score - 50 });
            zeroData.push({ time: t, value: 0 });
            lastTime = t;
        }
    }
    if (scoreData.length > 0) {
        seriesOracle.setData(scoreData);
        if (seriesOracleZero) seriesOracleZero.setData(zeroData);
    }
}

// ═══════════════════════════════════════════════════════════════
//  S12b: SmartTrend Chart Strip + FSM Markers
// ═══════════════════════════════════════════════════════════════

let _stStripTimer = null;
let _stLastState = '';

function _updateStChartStrip() {
    if (!window.__TAURI__) return;
    const strip = $el('st-chart-strip');
    if (!strip) return;

    window.__TAURI__.tauri.invoke('get_smarttrend_state').then(d => {
        if (!d || !d.enabled) { strip.hidden = true; return; }
        strip.hidden = false;

        const state = d.state || 'Flat';
        strip.querySelectorAll('.st-strip-seg').forEach(seg => {
            const s = seg.dataset.stState;
            seg.classList.toggle('st-strip-active', s === state);
        });

        if (state !== _stLastState && _stLastState !== '') {
            const engine = window._titanMarkerEngine;
            if (engine) {
                const now = Math.floor(Date.now() / 1000);
                const prev = engine.getSource('smarttrend');
                const stMarkers = [...prev];
                if (state === 'Entry' || state === 'InTrade') {
                    stMarkers.push({
                        time: now, position: 'belowBar', color: '#00BFFF',
                        shape: 'arrowUp', text: 'ST▲',
                    });
                } else if (state === 'Flat' && (_stLastState === 'InTrade' || _stLastState === 'Trail')) {
                    stMarkers.push({
                        time: now, position: 'aboveBar', color: '#FF2B2B',
                        shape: 'arrowDown', text: 'ST▼',
                    });
                } else if (state === 'Trail') {
                    stMarkers.push({
                        time: now, position: 'aboveBar', color: '#FFD700',
                        shape: 'circle', text: 'ST◆',
                    });
                }
                const capped = stMarkers.length > 50 ? stMarkers.slice(stMarkers.length - 50) : stMarkers;
                engine.update('smarttrend', capped);
            }
        }
        _stLastState = state;
    }).catch(() => {});
}

function _initStChartStrip() {
    _stStripTimer = setInterval(_updateStChartStrip, 3000);
    setTimeout(_updateStChartStrip, 1500);
}

window._initStChartStrip = _initStChartStrip;

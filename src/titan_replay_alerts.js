// titan_chart_sync -> titan_chart_sync.js

// tfToSeconds, formatCountdown -> titan_core.js

function updateCandleTimer() {
    const el = document.querySelector('.chart-timer');

    if (replayState && replayState.active) {
        if (el) el.innerText = `⏱️ REPLAY`;
        return;
    }
    if (!lastBar || !lastBar.time) {
        if (el) el.innerText = `⏱️ --:--`;
        return;
    }
    const tfSec = tfToSeconds(currentTF);
    const now = Date.now() / 1000;
    const nextClose = lastBar.time + tfSec;
    // Modulo fix for clock skew (if system time is behind market time)
    let remain = Math.max(0, Math.floor(nextClose - now));
    if (remain > tfSec) {
        remain = remain % tfSec;
    }
    const remainStr = formatCountdown(remain);

    // Update legacy DOM element if it exists
    if (el) el.innerText = `⏱️ ${remainStr}`;

    // Update Price Line Title (TradingView Style)
    if (livePriceLine) {
        livePriceLine.applyOptions({
            title: `LIVE ${remainStr}`
        });
    }
}

window.snapToOhlc = true;
window.snapTolerancePx = 8;

function setMagnetState(enabled) {
    window.snapToOhlc = !!enabled;
    const btn = $el('btn-magnet');
    if (btn) {
        btn.classList.toggle('active', window.snapToOhlc);
        btn.title = window.snapToOhlc ? 'Snap to OHLC (ON)' : 'Snap to OHLC (OFF)';
    }
}

function toggleMagnet() {
    setMagnetState(!window.snapToOhlc);
}

// getDrawingManager -> titan_core.js

function drawingsStorageKey() {
    return `titan_drawings_${currentSymbol}_${currentTF}`;
}

window.persistDrawings = function () {
    const mgr = getDrawingManager();
    if (!mgr) return;
    try {
        localStorage.setItem(drawingsStorageKey(), JSON.stringify(mgr.serialize()));
    } catch (e) {
        console.warn("Persist drawings failed", e);
    }
};

window.loadDrawings = function () {
    const mgr = getDrawingManager();
    if (!mgr) return;
    try {
        const raw = localStorage.getItem(drawingsStorageKey());
        if (!raw) {
            mgr.setDrawings([]);
            return;
        }
        const data = JSON.parse(raw);
        if (Array.isArray(data)) mgr.setDrawings(data);
    } catch (e) {
        console.warn("Load drawings failed", e);
    }
    if (typeof _lsUpdateDrawingCount === 'function') _lsUpdateDrawingCount();
};

window.initDrawingManager = function () {
    const mgr = getDrawingManager();
    if (mgr) return mgr;
    if (typeof DrawingManager !== 'undefined' && chart && candleSeries) {
        drawingManager = new DrawingManager(chart, candleSeries, '');
        window.drawingManager = drawingManager;
        if (window.loadDrawings) window.loadDrawings();
        return drawingManager;
    }
    return null;
};

function setChartFullscreenUi(on) {
    const btn = $el('btn-fullscreen');
    if (btn) btn.innerText = on ? "🧭 EXIT FULL SCREEN" : "💻 FULL SCREEN";
}

async function setNativeFullscreen(on) {
    const appWindow = window.__TAURI__?.window?.appWindow;
    if (appWindow?.setFullscreen) {
        try {
            await appWindow.setFullscreen(on);
            return true;
        } catch (e) {
            console.warn("Native fullscreen failed", e);
        }
    }
    return false;
}

async function enterChartFullscreen() {
    document.body.classList.add('fullscreen-chart');
    setChartFullscreenUi(true);
    const usedNative = await setNativeFullscreen(true);
    if (!usedNative && !document.fullscreenElement && document.documentElement.requestFullscreen) {
        document.documentElement.requestFullscreen().catch(() => { });
    }
    setTimeout(() => resizeCharts(), 50);
}

async function exitChartFullscreen() {
    document.body.classList.remove('fullscreen-chart');
    setChartFullscreenUi(false);
    const usedNative = await setNativeFullscreen(false);
    if (!usedNative && document.fullscreenElement && document.exitFullscreen) {
        document.exitFullscreen().catch(() => { });
    }
    setTimeout(() => resizeCharts(), 50);
}

function toggleChartFullscreen() {
    if (document.body.classList.contains('fullscreen-chart')) {
        exitChartFullscreen();
    } else {
        enterChartFullscreen();
    }
}

function updateReplayPanelVisibility(show) {
    const panel = $el('replay-panel');
    if (!panel) return;
    panel.classList.toggle('is-hidden', !show);
}

function updateReplayUi() {
    const slider = $el('replay-slider');
    const timeEl = $el('replay-time');
    if (slider) slider.value = String(replayState.index || 0);
    const data = replayState.data || [];
    const bar = data[replayState.index] || data[data.length - 1];
    if (timeEl) timeEl.innerText = bar ? formatTimeShort(bar.time) : "--";
    const toggle = $el('replay-toggle');
    if (toggle) toggle.innerText = replayState.playing ? "⏸" : "▶";
}

function applyReplaySlice() {
    const data = replayState.data || [];
    if (!data.length) return;
    const idx = Math.max(0, Math.min(replayState.index, data.length - 1));
    const slice = data.slice(0, idx + 1);
    if (candleSeries) candleSeries.setData(slice);
    if (volSeries) {
        volSeries.setData(slice.map(d => ({
            time: d.time,
            value: d.volume || 0,
            color: d.close >= d.open ? '#00BFFF' : '#FF2B2B'
        })));
    }
    lastBar = slice[slice.length - 1];
    window.lastBar = lastBar;
    if (lastBar) {
        updateRightBarFromBar(lastBar, { price: lastBar.close, time: lastBar.time });
        if (livePriceLine) livePriceLine.applyOptions({ price: lastBar.close });
        applyIndicators(slice);
    }
    if (typeof updateSignalMarkers === 'function') updateSignalMarkers();
    updateReplayUi();
}

function enterReplayMode() {
    if (!lastCandleData || lastCandleData.length < 10) {
        showCyberToast("Replay needs historical candles", 'warning');
        return;
    }
    replayState.active = true;
    replayState.playing = false;
    replayState.data = lastCandleData.slice();
    replayState.index = Math.max(20, Math.min(replayState.data.length - 1, replayState.data.length - 50));
    updateReplayPanelVisibility(true);
    const btn = $el('btn-replay');
    if (btn) btn.classList.add('active');
    updateRightBarFeedStatus("REPLAY", "warning");
    setChartStatus("REPLAY MODE", "#FFCC00");
    const slider = $el('replay-slider');
    if (slider) {
        slider.max = String(replayState.data.length - 1);
        slider.value = String(replayState.index);
    }
    applyReplaySlice();
}

function exitReplayMode() {
    replayState.active = false;
    replayState.playing = false;
    TitanTimers.clear('replay:step');
    replayState.timer = null;
    updateReplayPanelVisibility(false);
    const btn = $el('btn-replay');
    if (btn) btn.classList.remove('active');
    updateRightBarFeedStatus("LIVE", "success");
    setChartStatus("", "");
    if (lastCandleData && lastCandleData.length) {
        candleSeries.setData(lastCandleData);
        if (volSeries) {
            volSeries.setData(lastCandleData.map(d => ({
                time: d.time,
                value: d.volume || 0,
                color: d.close >= d.open ? '#00BFFF' : '#FF2B2B'
            })));
        }
        lastBar = lastCandleData[lastCandleData.length - 1];
        window.lastBar = lastBar;
        if (lastBar) {
            updateRightBarFromBar(lastBar, { price: lastBar.close, time: lastBar.time });
            if (livePriceLine) livePriceLine.applyOptions({ price: lastBar.close });
            applyIndicators(lastCandleData);
            lastAlertPrice = lastBar.close;
        }
        if (typeof updateSignalMarkers === 'function') updateSignalMarkers();
    }
    updateReplayUi();
    updateCandleTimer();
}

function stepReplay(delta) {
    if (!replayState.active) return;
    const data = replayState.data || [];
    if (!data.length) return;
    replayState.index = Math.max(0, Math.min(replayState.index + delta, data.length - 1));
    applyReplaySlice();
    if (replayState.index >= data.length - 1 && replayState.playing) {
        toggleReplay(false);
    }
}

function toggleReplay(forceState) {
    if (!replayState.active) return;
    const next = (forceState !== undefined) ? forceState : !replayState.playing;
    replayState.playing = next;
    TitanTimers.clear('replay:step');
    replayState.timer = null;
    if (replayState.playing) {
        const speed = Math.max(1, Number(replayState.speed || 1));
        replayState.timer = TitanTimers.register('replay:step', () => stepReplay(1), Math.max(50, 500 / speed));
    }
    updateReplayUi();
}

function setReplaySpeed(val) {
    replayState.speed = Number(val) || 1;
    if (replayState.playing) toggleReplay(true);
}

function addPriceAlert(price) {
    if (!candleSeries || !price || !Number.isFinite(price)) return;
    const exists = priceAlerts.some(a => Math.abs(a.price - price) < 0.0001);
    if (exists) return;
    const line = candleSeries.createPriceLine({
        price: price,
        color: '#FFCC00',
        lineWidth: 1,
        lineStyle: LightweightCharts.LineStyle.Dashed,
        axisLabelVisible: true,
        title: 'ALERT'
    });
    priceAlerts.push({ price, line, triggered: false });
    saveAlertsToStorage();
    renderAlertList();
}

function clearPriceAlerts() {
    if (!candleSeries) return;
    priceAlerts.forEach(a => {
        try { candleSeries.removePriceLine(a.line); } catch (e) { }
    });
    priceAlerts = [];
    saveAlertsToStorage();
    renderAlertList();
}

function alertsStorageKey() {
    return `titan_alerts_${currentSymbol}`;
}

function saveAlertsToStorage() {
    try {
        const list = priceAlerts.map(a => a.price);
        localStorage.setItem(alertsStorageKey(), JSON.stringify(list));
    } catch (e) {
        console.warn("Save alerts failed", e);
    }
}

function loadAlertsFromStorage() {
    try {
        const raw = localStorage.getItem(alertsStorageKey());
        if (!raw) {
            clearPriceAlerts();
            return;
        }
        const list = JSON.parse(raw);
        clearPriceAlerts();
        if (Array.isArray(list)) {
            list.forEach(p => addPriceAlert(Number(p)));
        }
        renderAlertList();
    } catch (e) {
        console.warn("Load alerts failed", e);
    }
}

function removePriceAlert(price) {
    const idx = priceAlerts.findIndex(a => Math.abs(a.price - price) < 0.0001);
    if (idx === -1) return;
    const alert = priceAlerts[idx];
    try { candleSeries.removePriceLine(alert.line); } catch (e) { }
    priceAlerts.splice(idx, 1);
    saveAlertsToStorage();
    renderAlertList();
}

function renderAlertList() {
    const el = $el('rb-alerts');
    if (!el) return;
    if (!priceAlerts.length) {
        el.innerText = "No alerts";
        return;
    }
    el.innerHTML = priceAlerts.map(a => {
        const price = Number(a.price);
        const label = Number.isFinite(price) ? price.toFixed(2) : '--';
        return `<div class="rb-alert-row" data-price="${label}">
            <span>${label}</span>
            <span class="rb-alert-remove" data-price="${price}">✕</span>
        </div>`;
    }).join('');
    el.querySelectorAll('.rb-alert-remove').forEach(btn => {
        btn.addEventListener('click', () => {
            const val = Number(btn.getAttribute('data-price'));
            if (Number.isFinite(val)) removePriceAlert(val);
        });
    });
}

function checkPriceAlerts(price) {
    if (!priceAlerts.length) {
        lastAlertPrice = price;
        return;
    }
    const prev = (lastAlertPrice === null || lastAlertPrice === undefined) ? price : lastAlertPrice;
    priceAlerts.forEach(a => {
        if (a.triggered) return;
        const crossedUp = prev < a.price && price >= a.price;
        const crossedDown = prev > a.price && price <= a.price;
        if (crossedUp || crossedDown) {
            a.triggered = true;
            try { candleSeries.removePriceLine(a.line); } catch (e) { }
            showCyberToast(`PRICE ALERT ${currentSymbol} @ ${a.price.toFixed(2)}`, 'info');
        }
    });
    lastAlertPrice = price;
    priceAlerts = priceAlerts.filter(a => !a.triggered);
    saveAlertsToStorage();
    renderAlertList();
}

// --- DRAWING ENGINE V1 ---
// MOVED TO drawing_engine.js


// window.invoke -> titan_core.js

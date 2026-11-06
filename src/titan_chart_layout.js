function bindPaneDrag(resizer, { onStart, onMove, onEnd }) {
    let dragging = false;
    let startY = 0;
    resizer.addEventListener('pointerdown', (e) => {
        e.preventDefault();
        dragging = true;
        startY = e.clientY;
        document.body.classList.add('resizing');
        resizer.setPointerCapture(e.pointerId);
        if (onStart) onStart(e);
    });
    resizer.addEventListener('pointermove', (e) => {
        if (!dragging) return;
        onMove(e.clientY, e.clientY - startY);
    });
    resizer.addEventListener('pointerup', (e) => {
        if (!dragging) return;
        dragging = false;
        try { resizer.releasePointerCapture(e.pointerId); } catch (_) { }
        document.body.classList.remove('resizing');
        if (onEnd) onEnd();
    });
    resizer.addEventListener('pointercancel', () => {
        if (!dragging) return;
        dragging = false;
        document.body.classList.remove('resizing');
        if (onEnd) onEnd();
    });
}

function resizeCharts() {
    const applyResize = () => {
        updatePaneLayout();
        if (chart) {
            const main = $el('chart-div');
            if (main) chart.resize(main.clientWidth, main.clientHeight);
        }
        if (volChart) {
            const volEl = $el('chart-vol');
            if (volEl) volChart.resize(volEl.clientWidth, volEl.clientHeight);
        }
        if (rsiChart) {
            const rsiEl = $el('chart-rsi');
            if (rsiEl) rsiChart.resize(rsiEl.clientWidth, rsiEl.clientHeight);
        }
        if (macdChart) {
            const macdEl = $el('chart-macd');
            if (macdEl) macdChart.resize(macdEl.clientWidth, macdEl.clientHeight);
        }
        if (adxChart) {
            const adxEl = $el('chart-adx');
            if (adxEl) adxChart.resize(adxEl.clientWidth, adxEl.clientHeight);
        }

        if (oracleChart && !oracleFullscreen) {
            const oracleEl = $el('chart-oracle');
            if (oracleEl) oracleChart.resize(oracleEl.clientWidth, oracleEl.clientHeight);
        }
    };
    if (typeof requestAnimationFrame === 'function') {
        requestAnimationFrame(applyResize);
    } else {
        applyResize();
    }
}

function measureChartGeometry() {
    const wrapper = $el('chart-wrapper');
    if (!wrapper) return null;
    const chartArea = wrapper.closest('.chart-area');
    const toolbarTop = chartArea ? chartArea.querySelector('.toolbar-top') : null;
    const topbar = chartArea ? chartArea.querySelector('.chart-topbar') : null;
    const overlay = document.querySelector('.chart-overlay-row');
    const overlayH = overlay ? overlay.offsetHeight : 0;
    const stack = wrapper.querySelector('.chart-stack');
    const mainWrap = $el('chart-main-wrap');
    let available = chartArea ? chartArea.clientHeight : wrapper.clientHeight;
    if (toolbarTop) available -= toolbarTop.offsetHeight;
    if (topbar) available -= topbar.offsetHeight;
    available = Math.max(0, Math.floor(available));
    const mainResizer = wrapper.querySelector('.chart-main-resizer');
    const resizerH = mainResizer ? mainResizer.offsetHeight : 0;
    return { wrapper, stack, mainWrap, available, overlayH, resizerH };
}

function collectVisiblePanes() {
    const paneDefs = [
        { id: 'chart-vol', toggle: 'toggle-vol' },
        { id: 'chart-rsi', toggle: 'toggle-rsi' },
        { id: 'chart-macd', toggle: 'toggle-macd' },
        { id: 'chart-adx', toggle: 'toggle-adx' },
        { id: 'chart-oracle', toggle: 'toggle-oracle' }
    ];
    const panes = paneDefs.map(({ id, toggle }) => {
        const el = $el(id);
        if (!el) return null;
        const toggleEl = $el(toggle);
        const visible = toggleEl ? toggleEl.checked : getComputedStyle(el).display !== 'none';
        if (toggleEl) el.style.display = visible ? 'block' : 'none';
        return { el, visible };
    }).filter(Boolean);
    return panes.filter(p => p.visible).map(p => p.el);
}

function distributeHeights(mainWrap, visiblePanes, total) {
    const minMain = 120;
    const minPane = 50;
    const paneCount = visiblePanes.length;

    const applySize = (el, h) => {
        const px = `${Math.max(minPane, Math.floor(h))}px`;
        el.style.height = px;
        el.style.flex = `0 0 ${px}`;
    };
    const setMain = (h) => {
        const px = `${h}px`;
        mainWrap.style.height = px;
        mainWrap.style.flex = `0 0 ${px}`;
    };

    if (paneCount === 0) {
        setMain(Math.max(120, total));
        return;
    }

    const minTotal = minMain + paneCount * minPane;
    if (total < minTotal) {
        const scale = total / minTotal;
        setMain(Math.max(60, Math.floor(minMain * scale)));
        visiblePanes.forEach(el => applySize(el, Math.max(30, Math.floor(minPane * scale))));
        return;
    }

    const manual = [];
    const auto = [];
    visiblePanes.forEach(el => {
        const saved = paneHeights[el.id];
        if (Number.isFinite(saved)) {
            manual.push({ el, h: saved });
        } else {
            auto.push(el);
        }
    });

    const clampManualTotal = (budget) => {
        let manualTotal = manual.reduce((sum, m) => sum + m.h, 0);
        if (manualTotal > budget && manualTotal > 0) {
            const scale = budget / manualTotal;
            manual.forEach(m => { m.h = Math.max(minPane, Math.floor(m.h * scale)); });
            manualTotal = manual.reduce((sum, m) => sum + m.h, 0);
        }
        return manualTotal;
    };

    if (Number.isFinite(mainChartHeight)) {
        const maxMain = Math.max(minMain, total - paneCount * minPane);
        const mainHeight = Math.min(Math.max(mainChartHeight, minMain), maxMain);
        const availableForPanes = total - mainHeight;
        const maxManualBudget = Math.max(0, availableForPanes - auto.length * minPane);
        const manualTotal = clampManualTotal(maxManualBudget);
        const remainingForAuto = availableForPanes - manualTotal;
        const autoH = auto.length > 0 ? Math.max(minPane, Math.floor(remainingForAuto / auto.length)) : 0;
        setMain(mainHeight);
        manual.forEach(m => applySize(m.el, m.h));
        auto.forEach(el => applySize(el, autoH));
        return;
    }

    const maxManualBudget = Math.max(0, total - minMain - auto.length * minPane);
    const manualTotal = clampManualTotal(maxManualBudget);
    const remaining = total - manualTotal;
    const autoH = auto.length > 0 ? Math.max(minPane, Math.floor((remaining - minMain) / auto.length)) : 0;
    const mainHeight = Math.max(minMain, remaining - autoH * auto.length);
    setMain(mainHeight);
    manual.forEach(m => applySize(m.el, m.h));
    auto.forEach(el => applySize(el, autoH));
}

function updatePaneLayout() {
    const geo = measureChartGeometry();
    if (!geo) return;
    const { wrapper, stack, mainWrap, overlayH, resizerH } = geo;
    let { available } = geo;

    if (available > 0) {
        const wrapperPx = `${available}px`;
        wrapper.style.height = wrapperPx;
        wrapper.style.flex = `0 0 ${wrapperPx}`;
    }
    let stackHeight = Math.max(0, available - overlayH);
    if (stack) {
        const stackPx = `${stackHeight}px`;
        stack.style.height = stackPx;
        stack.style.flex = `0 0 ${stackPx}`;
    }
    if (!mainWrap) return;

    const visiblePanes = collectVisiblePanes();
    const paneCount = visiblePanes.length;
    const minMain = 120;
    const minPane = 50;

    if (paneCount > 0) {
        const requiredWrapper = overlayH + resizerH + minMain + paneCount * minPane;
        if (requiredWrapper > available) {
            available = requiredWrapper;
            const wrapperPx = `${available}px`;
            wrapper.style.height = wrapperPx;
            wrapper.style.flex = `0 0 ${wrapperPx}`;
            stackHeight = Math.max(0, available - overlayH);
            if (stack) {
                const stackPx = `${stackHeight}px`;
                stack.style.height = stackPx;
                stack.style.flex = `0 0 ${stackPx}`;
            }
        }
    }

    const total = Math.max(0, stackHeight - resizerH);
    if (total <= 0) return;
    distributeHeights(mainWrap, visiblePanes, total);
}

function fitSubCharts() {
    if (volChart) volChart.timeScale().fitContent();
    if (rsiChart) rsiChart.timeScale().fitContent();
    if (macdChart) macdChart.timeScale().fitContent();
    if (adxChart) adxChart.timeScale().fitContent();
}

function initChartResizer() {
    const resizer = $el('chart-resizer');
    const workspace = document.querySelector('.chart-workspace');
    const btnMinus = $el('chart-resize-minus');
    const btnPlus = $el('chart-resize-plus');
    if (!resizer || !workspace) return;
    const key = 'chartWorkspaceHeight';
    const saved = localStorage.getItem(key);
    if (saved) {
        const h = parseInt(saved, 10);
        if (Number.isFinite(h) && h > 0) {
            workspace.style.height = `${h}px`;
            workspace.style.minHeight = `${h}px`;
        }
    }
    let startY = 0;
    let startHeight = 0;
    let dragging = false;
    const getBounds = () => {
        const rect = workspace.getBoundingClientRect();
        const minH = 420;
        const maxH = Math.max(minH + 20, window.innerHeight - rect.top - 60);
        return { minH, maxH };
    };
    const applyHeight = (val, persist) => {
        const { minH, maxH } = getBounds();
        const next = Math.min(maxH, Math.max(minH, val));
        const nextPx = `${Math.round(next)}px`;
        workspace.style.height = nextPx;
        workspace.style.minHeight = nextPx;
        resizeCharts();
        if (persist) {
            localStorage.setItem(key, parseInt(nextPx, 10));
        }
    };
    const autoScroll = (clientY) => {
        const edge = 30;
        if (clientY > window.innerHeight - edge) {
            window.scrollBy(0, 12);
        } else if (clientY < edge) {
            window.scrollBy(0, -12);
        }
    };
    const onMove = (clientY) => {
        if (!dragging) return;
        const delta = clientY - startY;
        applyHeight(startHeight + delta, false);
    };
    const stopDrag = () => {
        if (!dragging) return;
        dragging = false;
        document.body.classList.remove('resizing');
        localStorage.setItem(key, parseInt(workspace.style.height, 10));
    };
    resizer.addEventListener('pointerdown', (e) => {
        e.preventDefault();
        dragging = true;
        startY = e.clientY + window.scrollY;
        startHeight = workspace.getBoundingClientRect().height;
        document.body.classList.add('resizing');
        resizer.setPointerCapture(e.pointerId);
    });
    resizer.addEventListener('pointermove', (e) => {
        autoScroll(e.clientY);
        onMove(e.clientY + window.scrollY);
    });
    resizer.addEventListener('pointerup', (e) => {
        try { resizer.releasePointerCapture(e.pointerId); } catch (_) { }
        stopDrag();
    });
    resizer.addEventListener('pointercancel', () => {
        stopDrag();
    });
    resizer.addEventListener('dblclick', () => {
        workspace.style.height = '';
        workspace.style.minHeight = '';
        localStorage.removeItem(key);
        resizeCharts();
    });
    if (btnMinus) {
        btnMinus.addEventListener('click', () => {
            const current = workspace.getBoundingClientRect().height;
            applyHeight(current - 60, true);
        });
    }
    if (btnPlus) {
        btnPlus.addEventListener('click', () => {
            const current = workspace.getBoundingClientRect().height;
            applyHeight(current + 60, true);
        });
    }
}

function paneHeightKey(id) {
    return `paneHeight_${id}`;
}

function histScaleKey(id) {
    return `histScale_${id}`;
}

function loadHistogramPaneSettings() {
    histogramPaneIds.forEach(id => {
        const h = storageGet(paneHeightKey(id), null);
        if (Number.isFinite(h)) {
            paneHeights[id] = h;
        }
        const scale = storageGet(histScaleKey(id), null);
        if (Number.isFinite(scale)) {
            histogramScales[id] = scale;
        }
    });
}

function getHistogramScale(id) {
    const scale = histogramScales[id];
    return Number.isFinite(scale) ? scale : 0.8;
}

function setHistogramScale(id, value) {
    const scale = Math.min(0.95, Math.max(0.25, value));
    histogramScales[id] = scale;
    storageSet(histScaleKey(id), scale);
    applyHistogramScale(id);
}

function applyHistogramScale(id) {
    const scale = getHistogramScale(id);
    if (id === 'chart-vol' && volSeries) {
        const top = Math.max(0.02, 1 - scale);
        volSeries.priceScale().applyOptions({ scaleMargins: { top, bottom: 0 } });
    }
    if (id === 'chart-macd' && macdHistSeries) {
        const side = Math.max(0.02, (1 - scale) / 2);
        macdHistSeries.priceScale().applyOptions({ scaleMargins: { top: side, bottom: side } });
    }
    if (id === 'chart-rsi' && rsiSeries) {
        const side = Math.max(0.02, (1 - scale) / 2);
        rsiSeries.priceScale().applyOptions({ scaleMargins: { top: side, bottom: side } });
    }
    if (id === 'chart-adx' && adxSeries) {
        const side = Math.max(0.02, (1 - scale) / 2);
        adxSeries.priceScale().applyOptions({ scaleMargins: { top: side, bottom: side } });
    }
}

function setPaneHeight(id, height, persist) {
    const pane = $el(id);
    if (!pane) return;
    const next = Math.max(50, Math.floor(height));
    paneHeights[id] = next;
    if (persist) {
        storageSet(paneHeightKey(id), next);
    }
    updatePaneLayout();
    resizeCharts();
}

function adjustPaneHeight(id, delta) {
    const pane = $el(id);
    if (!pane) return;
    const current = paneHeights[id] || pane.getBoundingClientRect().height;
    setPaneHeight(id, current + delta, true);
}

function initHistogramPaneControls() {
    loadHistogramPaneSettings();
    histogramPaneIds.forEach(id => {
        if (Number.isFinite(paneHeights[id])) {
            setPaneHeight(id, paneHeights[id], false);
        }
        applyHistogramScale(id);
    });

    if (histogramControlsBound) return;
    histogramControlsBound = true;

    const buttons = document.querySelectorAll('.pane-btn[data-pane]');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            const paneId = btn.getAttribute('data-pane');
            const action = btn.getAttribute('data-action');
            if (!paneId || !action) return;
            if (action === 'pane-plus') adjustPaneHeight(paneId, 40);
            if (action === 'pane-minus') adjustPaneHeight(paneId, -40);
            if (action === 'hist-plus') setHistogramScale(paneId, getHistogramScale(paneId) + 0.05);
            if (action === 'hist-minus') setHistogramScale(paneId, getHistogramScale(paneId) - 0.05);
        });
    });

    const resizers = document.querySelectorAll('.pane-resizer[data-pane]');
    resizers.forEach(resizer => {
        const paneId = resizer.getAttribute('data-pane');
        const pane = paneId ? $el(paneId) : null;
        if (!pane) return;
        let startY = 0;
        let startHeight = 0;
        bindPaneDrag(resizer, {
            onStart: () => { startHeight = pane.getBoundingClientRect().height; },
            onMove: (_y, delta) => { setPaneHeight(paneId, startHeight + delta, false); },
            onEnd: () => { storageSet(paneHeightKey(paneId), paneHeights[paneId]); }
        });
    });
}

function mainChartHeightKey() {
    return 'chartMainHeight';
}

function setMainChartHeight(height, persist) {
    const next = Math.max(120, Math.floor(height));
    mainChartHeight = next;
    if (persist) {
        storageSet(mainChartHeightKey(), next);
    }
    updatePaneLayout();
    resizeCharts();
}

function clearMainChartHeight() {
    mainChartHeight = null;
    storageSet(mainChartHeightKey(), null);
    updatePaneLayout();
    resizeCharts();
}

function adjustMainChartHeight(delta) {
    const mainWrap = $el('chart-main-wrap');
    if (!mainWrap) return;
    const current = Number.isFinite(mainChartHeight)
        ? mainChartHeight
        : mainWrap.getBoundingClientRect().height;
    setMainChartHeight(current + delta, true);
}

function initMainChartControls() {
    const stored = storageGet(mainChartHeightKey(), null);
    if (Number.isFinite(stored)) {
        mainChartHeight = stored;
        updatePaneLayout();
        resizeCharts();
    }

    if (mainControlsBound) return;
    mainControlsBound = true;

    const buttons = document.querySelectorAll('.pane-btn[data-main]');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            const action = btn.getAttribute('data-action');
            if (action === 'main-plus') adjustMainChartHeight(60);
            if (action === 'main-minus') adjustMainChartHeight(-60);
        });
    });

    const resizers = document.querySelectorAll('.pane-resizer[data-main], .chart-main-resizer[data-main]');
    resizers.forEach(resizer => {
        const mainWrap = $el('chart-main-wrap');
        if (!mainWrap) return;
        let startHeight = 0;
        bindPaneDrag(resizer, {
            onStart: () => { startHeight = mainWrap.getBoundingClientRect().height; },
            onMove: (_y, delta) => { setMainChartHeight(startHeight + delta, false); },
            onEnd: () => { storageSet(mainChartHeightKey(), mainChartHeight); }
        });
        resizer.addEventListener('dblclick', () => clearMainChartHeight());
    });

    const wheelTargets = document.querySelectorAll('.chart-main-resizer[data-main]');
    wheelTargets.forEach(resizer => {
        resizer.addEventListener('wheel', (e) => {
            e.preventDefault();
            e.stopPropagation();
            const delta = Math.abs(e.deltaY);
            const step = Math.min(120, Math.max(20, delta));
            const dir = e.deltaY > 0 ? -1 : 1;
            adjustMainChartHeight(dir * step);
        }, { passive: false });
    });
}

function freePanelHeightKey(id) {
    return `panelHeight_${id}`;
}

function heatmapHeightKey(id) {
    return `heatmapHeight_${id}`;
}

function setFreePanelHeight(id, height, persist) {
    const panel = $el(id);
    if (!panel) return;
    const next = Math.max(80, Math.floor(height));
    freePanelHeights[id] = next;
    panel.style.height = `${next}px`;
    panel.style.minHeight = `${next}px`;
    if (persist) {
        storageSet(freePanelHeightKey(id), next);
    }
}

function adjustFreePanelHeight(id, delta) {
    const panel = $el(id);
    if (!panel) return;
    const current = freePanelHeights[id] || panel.getBoundingClientRect().height;
    setFreePanelHeight(id, current + delta, true);
}

function setHeatmapHeight(canvasId, height, persist) {
    const canvas = $el(canvasId);
    if (!canvas) return;
    const next = Math.min(260, Math.max(40, Math.floor(height)));
    heatmapHeights[canvasId] = next;
    canvas.style.height = `${next}px`;
    canvas.height = Math.floor(next * (window.devicePixelRatio || 1));
    if (persist) {
        storageSet(heatmapHeightKey(canvasId), next);
    }
}

function adjustHeatmapHeight(canvasId, delta) {
    const canvas = $el(canvasId);
    if (!canvas) return;
    const current = heatmapHeights[canvasId] || canvas.getBoundingClientRect().height || canvas.height;
    setHeatmapHeight(canvasId, current + delta, true);
}

function initFreePanelControls() {
    freePanelIds.forEach(id => {
        const saved = storageGet(freePanelHeightKey(id), null);
        if (Number.isFinite(saved)) {
            setFreePanelHeight(id, saved, false);
        }
    });
    Object.values(panelHeatmaps).forEach(canvasId => {
        const saved = storageGet(heatmapHeightKey(canvasId), null);
        if (Number.isFinite(saved)) {
            setHeatmapHeight(canvasId, saved, false);
        }
    });

    if (freePanelControlsBound) {
        resizeCharts();
        return;
    }
    freePanelControlsBound = true;

    const buttons = document.querySelectorAll('.pane-btn[data-panel]');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            const panelId = btn.getAttribute('data-panel');
            const action = btn.getAttribute('data-action');
            if (!panelId || !action) return;
            if (action === 'panel-plus') adjustFreePanelHeight(panelId, 40);
            if (action === 'panel-minus') adjustFreePanelHeight(panelId, -40);
            if (action === 'heatmap-plus') {
                const canvasId = panelHeatmaps[panelId];
                if (canvasId) adjustHeatmapHeight(canvasId, 10);
            }
            if (action === 'heatmap-minus') {
                const canvasId = panelHeatmaps[panelId];
                if (canvasId) adjustHeatmapHeight(canvasId, -10);
            }
        });
    });

    const resizers = document.querySelectorAll('.pane-resizer[data-panel]');
    resizers.forEach(resizer => {
        const panelId = resizer.getAttribute('data-panel');
        const panel = panelId ? $el(panelId) : null;
        if (!panel) return;
        let startHeight = 0;
        bindPaneDrag(resizer, {
            onStart: () => { startHeight = panel.getBoundingClientRect().height; },
            onMove: (_y, delta) => { setFreePanelHeight(panelId, startHeight + delta, false); },
            onEnd: () => { storageSet(freePanelHeightKey(panelId), freePanelHeights[panelId]); }
        });
    });
    resizeCharts();
}

// ═══════════════════════════════════════════════════════════════
//  S16: Unified Chart Marker Engine
//  Merges fills + signals + SmartTrend into a single setMarkers call
// ═══════════════════════════════════════════════════════════════
window._titanFillMarkers = window._titanFillMarkers || [];

window._titanMarkerEngine = (function() {
    const FILL_CAP = 50;
    const FILL_MAX_AGE_S = 86400;
    const DEBOUNCE_MS = 100;
    const _sources = { signals: [], fills: [], smarttrend: [] };
    let _debounceTimer = null;

    function _pruneAndCapFills(fills) {
        const cutoff = Math.floor(Date.now() / 1000) - FILL_MAX_AGE_S;
        const fresh = fills.filter(f => (f.time || 0) > cutoff);
        return fresh.length > FILL_CAP ? fresh.slice(fresh.length - FILL_CAP) : fresh;
    }

    function _merge() {
        if (!window.candleSeries) return;
        const all = [];
        for (const key of Object.keys(_sources)) {
            const arr = _sources[key];
            if (arr && arr.length > 0) all.push(...arr);
        }
        const seen = new Set();
        const deduped = [];
        for (const m of all) {
            const k = `${m.time}|${m.position}|${m.shape}|${m.text || ''}`;
            if (!seen.has(k)) {
                seen.add(k);
                deduped.push(m);
            }
        }
        deduped.sort((a, b) => a.time - b.time);
        try { window.candleSeries.setMarkers(deduped); } catch (_) {}

        const overlay = document.getElementById('chart-marker-stats');
        if (overlay) {
            const s = _sources.signals ? _sources.signals.length : 0;
            const f = _sources.fills ? _sources.fills.length : 0;
            const t = _sources.smarttrend ? _sources.smarttrend.length : 0;
            overlay.textContent = 'SIG:' + s + ' FILL:' + f + ' ST:' + t + ' TOT:' + deduped.length;
        }
    }

    function update(source, markers) {
        if (source === 'fills') {
            _sources.fills = _pruneAndCapFills(markers || []);
        } else {
            _sources[source] = markers || [];
        }
        if (_debounceTimer) clearTimeout(_debounceTimer);
        _debounceTimer = setTimeout(_merge, DEBOUNCE_MS);
    }

    function clear(source) {
        if (source) {
            _sources[source] = [];
        } else {
            _sources.signals = [];
            _sources.fills = [];
            _sources.smarttrend = [];
        }
        _merge();
    }

    function stats() {
        const s = {};
        for (const key of Object.keys(_sources)) {
            s[key] = (_sources[key] || []).length;
        }
        s.total = Object.values(s).reduce((a, b) => a + b, 0);
        return s;
    }

    function getSource(name) {
        return _sources[name] ? [..._sources[name]] : [];
    }

    return { update, clear, stats, getSource };
})();

function applyFillMarkers() {
    const fills = window._titanFillMarkers || [];
    if (fills.length === 0) return;

    const markers = fills.map(f => {
        const isBuy = (f.side || '').toLowerCase() === 'buy';
        const ts = f.timestamp || Math.floor(Date.now() / 1000);
        return {
            time: ts,
            position: isBuy ? 'belowBar' : 'aboveBar',
            color: isBuy ? '#00FF41' : '#FF2B2B',
            shape: isBuy ? 'arrowUp' : 'arrowDown',
            text: (isBuy ? 'B' : 'S') + ' ' + Number(f.filled_qty || 0).toFixed(1),
        };
    });

    window._titanMarkerEngine.update('fills', markers);
}

if (typeof window.__TAURI__ !== 'undefined') {
    setInterval(applyFillMarkers, 3000);
}

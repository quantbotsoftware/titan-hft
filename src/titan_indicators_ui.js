function toSeries(times, values) {
    const out = [];
    for (let i = 0; i < times.length; i++) {
        const v = values[i];
        if (v === null || v === undefined) continue;
        out.push({ time: times[i], value: v });
    }
    return out;
}

function clampInt(val, min, max, fallback) {
    const num = parseInt(val, 10);
    if (Number.isNaN(num)) return fallback;
    return Math.min(max, Math.max(min, num));
}

function clampFloat(val, min, max, fallback) {
    const num = parseFloat(val);
    if (Number.isNaN(num)) return fallback;
    return Math.min(max, Math.max(min, num));
}

function normalizeIndicatorConfig(cfg) {
    const out = {
        ema9: clampInt(cfg.ema9, 2, 300, indicatorDefaults.ema9),
        ema50: clampInt(cfg.ema50, 2, 300, indicatorDefaults.ema50),
        ema200: clampInt(cfg.ema200, 2, 600, indicatorDefaults.ema200),
        bbPeriod: clampInt(cfg.bbPeriod, 5, 200, indicatorDefaults.bbPeriod),
        bbStd: clampFloat(cfg.bbStd, 0.5, 5, indicatorDefaults.bbStd),
        rsiPeriod: clampInt(cfg.rsiPeriod, 2, 100, indicatorDefaults.rsiPeriod),
        macdFast: clampInt(cfg.macdFast, 2, 50, indicatorDefaults.macdFast),
        macdSlow: clampInt(cfg.macdSlow, 5, 120, indicatorDefaults.macdSlow),
        macdSignal: clampInt(cfg.macdSignal, 2, 50, indicatorDefaults.macdSignal),
        adxPeriod: clampInt(cfg.adxPeriod, 5, 100, indicatorDefaults.adxPeriod)
    };
    if (out.macdFast >= out.macdSlow) {
        out.macdSlow = out.macdFast + 1;
    }
    return out;
}

function setIndicatorInputs(cfg) {
    const setVal = (id, val) => {
        const el = $el(id);
        if (el) el.value = val;
    };
    setVal('cfg-ema9', cfg.ema9);
    setVal('cfg-ema50', cfg.ema50);
    setVal('cfg-ema200', cfg.ema200);
    setVal('cfg-bb-period', cfg.bbPeriod);
    setVal('cfg-bb-std', cfg.bbStd);
    setVal('cfg-rsi-period', cfg.rsiPeriod);
    setVal('cfg-macd-fast', cfg.macdFast);
    setVal('cfg-macd-slow', cfg.macdSlow);
    setVal('cfg-macd-signal', cfg.macdSignal);
    setVal('cfg-adx-period', cfg.adxPeriod);
}

function readIndicatorConfigFromInputs() {
    return {
        ema9: $el('cfg-ema9')?.value,
        ema50: $el('cfg-ema50')?.value,
        ema200: $el('cfg-ema200')?.value,
        bbPeriod: $el('cfg-bb-period')?.value,
        bbStd: $el('cfg-bb-std')?.value,
        rsiPeriod: $el('cfg-rsi-period')?.value,
        macdFast: $el('cfg-macd-fast')?.value,
        macdSlow: $el('cfg-macd-slow')?.value,
        macdSignal: $el('cfg-macd-signal')?.value,
        adxPeriod: $el('cfg-adx-period')?.value
    };
}

function applyIndicatorConfig(cfg) {
    indicatorConfig = normalizeIndicatorConfig(cfg);
    setIndicatorInputs(indicatorConfig);
    if (lastCandleData && lastCandleData.length > 0) {
        applyIndicators(lastCandleData);
    }
    saveChartSettings();
}

function toggleIndicatorPanel(force) {
    const panel = $el('indicator-panel');
    if (!panel) return;
    const isHidden = panel.classList.contains('is-hidden');
    const show = typeof force === 'boolean' ? force : isHidden;
    panel.classList.toggle('is-hidden', !show);
}

function bindIndicatorPanel() {
    const btn = $el('btn-indicator-settings');
    const closeBtn = $el('indicator-panel-close');
    const applyBtn = $el('cfg-apply');
    const resetBtn = $el('cfg-reset');

    if (btn) btn.addEventListener('click', () => toggleIndicatorPanel());
    if (closeBtn) closeBtn.addEventListener('click', () => toggleIndicatorPanel(false));
    if (applyBtn) applyBtn.addEventListener('click', () => applyIndicatorConfig(readIndicatorConfigFromInputs()));
    if (resetBtn) resetBtn.addEventListener('click', () => applyIndicatorConfig(indicatorDefaults));

    const inputs = ['cfg-ema9', 'cfg-ema50', 'cfg-ema200', 'cfg-bb-period', 'cfg-bb-std', 'cfg-rsi-period', 'cfg-macd-fast', 'cfg-macd-slow', 'cfg-macd-signal', 'cfg-adx-period'];
    inputs.forEach(id => {
        const el = $el(id);
        if (!el) return;
        el.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') applyIndicatorConfig(readIndicatorConfigFromInputs());
        });
    });

    setIndicatorInputs(indicatorConfig);
}

function applyIndicators(data) {
    if (!data || data.length === 0) return;
    const times = data.map(d => d.time);
    const closes = data.map(d => d.close);

    const cfg = indicatorConfig || indicatorDefaults;
    const ema9 = ema(closes, cfg.ema9);
    const ema50 = ema(closes, cfg.ema50);
    const ema200 = ema(closes, cfg.ema200);
    const bb = bbands(closes, cfg.bbPeriod, cfg.bbStd);
    const rsiVals = rsi(closes, cfg.rsiPeriod);
    const macdVals = macd(closes, cfg.macdFast, cfg.macdSlow, cfg.macdSignal);
    const vwapVals = calcVwapSeries(data);
    const adxVals = adx(data, cfg.adxPeriod);

    if (ema9Series) ema9Series.setData(toSeries(times, ema9));
    if (ema50Series) ema50Series.setData(toSeries(times, ema50));
    if (ema200Series) ema200Series.setData(toSeries(times, ema200));
    if (vwapSeries) vwapSeries.setData(toSeries(times, vwapVals));
    if (bbUpperSeries) bbUpperSeries.setData(toSeries(times, bb.upper));
    if (bbMidSeries) bbMidSeries.setData(toSeries(times, bb.mid));
    if (bbLowerSeries) bbLowerSeries.setData(toSeries(times, bb.lower));
    if (rsiSeries) rsiSeries.setData(toSeries(times, rsiVals));
    if (macdLineSeries) macdLineSeries.setData(toSeries(times, macdVals.macdLine));
    if (macdSignalSeries) macdSignalSeries.setData(toSeries(times, macdVals.signalLine));
    if (macdHistSeries) {
        macdHistSeries.setData(times.map((t, i) => ({
            time: t,
            value: macdVals.hist[i],
            color: pnlColor(macdVals.hist[i])
        })));
    }
    if (adxSeries) adxSeries.setData(toSeries(times, adxVals));
    setRefLines(times);
    if (vwapVals.length) {
        lastVwap = vwapVals[vwapVals.length - 1];
        setRightBarText('rb-vwap', formatNum(lastVwap));
    }
}

function setRefLines(times) {
    if (!times || times.length === 0) return;
    if (times.length === lastRefLinesLen) return;
    lastRefLinesLen = times.length;
    const lineData = (val) => times.map(t => ({ time: t, value: val }));
    if (rsiUpperLine) rsiUpperLine.setData(lineData(70));
    if (rsiLowerLine) rsiLowerLine.setData(lineData(30));
    if (rsiMidLine) rsiMidLine.setData(lineData(50));
    if (macdZeroLine) macdZeroLine.setData(lineData(0));
    if (adxLine20) adxLine20.setData(lineData(20));
    if (adxLine25) adxLine25.setData(lineData(25));
}

function _countSignalMarkers(markers, predicate) {
    if (!Array.isArray(markers) || !markers.length) return 0;
    return markers.reduce((acc, m) => (predicate(m) ? acc + 1 : acc), 0);
}

function updateSignalHealthPanel() {
    const panel = $el('signal-health-panel');
    if (!panel) return;

    const activeModeBtn = document.querySelector('.nexus-mode-btn.active');
    const mode = activeModeBtn?.dataset?.mode || '';
    const isSignalMode = mode === 'SIGNAL_STRATEGY';
    panel.style.display = isSignalMode ? 'block' : 'none';

    const reasonEl = $el('debug-marker-reason');
    if (!isSignalMode) {
        if (reasonEl && window.__signalMarkerReason) {
            reasonEl.innerText = window.__signalMarkerReason;
        }
        return;
    }

    const srcBtn = document.querySelector('.sig-src-btn.active');
    const src = srcBtn?.dataset?.src ? srcBtn.dataset.src.toUpperCase() : 'FUSION';
    const tf = typeof currentTF === 'string' && currentTF ? currentTF : '-';
    const markers = Array.isArray(signalMarkers) ? signalMarkers : [];

    const buyCount = _countSignalMarkers(markers, m => (m?.text || '').endsWith('BUY'));
    const sellCount = _countSignalMarkers(markers, m => (m?.text || '').endsWith('SELL'));
    const trailCount = _countSignalMarkers(markers, m => (m?.text || '').includes('TRAIL'));
    const chaosCount = _countSignalMarkers(markers, m => (m?.text || '').includes('CHAOS'));

    const toggle = $el('toggle-signals');
    const signalsOn = !(toggle && !toggle.checked);
    const lastUpdateMs = Number(window.__signalLastUpdateMs || 0);
    const lastErrorMs = Number(window.__signalLastErrorMs || 0);
    const ageSec = lastUpdateMs > 0 ? Math.max(0, Math.floor((Date.now() - lastUpdateMs) / 1000)) : null;

    let status = 'LIVE';
    if (!signalsOn) {
        status = 'OFF';
    } else if (lastErrorMs > lastUpdateMs && lastErrorMs > 0) {
        status = 'ERROR';
    } else if (markers.length === 0 && window.__signalMarkerReason) {
        status = 'NODATA';
    } else if (markers.length === 0) {
        status = 'EMPTY';
    } else if (ageSec !== null && ageSec > 45) {
        status = 'STALE';
    }

    const setText = (id, value) => {
        const el = $el(id);
        if (el) el.textContent = String(value);
    };

    setText('signal-health-src', src);
    setText('signal-health-tf', tf);
    setText('signal-health-total', markers.length);
    setText('signal-health-buy', buyCount);
    setText('signal-health-sell', sellCount);
    setText('signal-health-trail', trailCount);
    setText('signal-health-chaos', chaosCount);
    setText('signal-health-age', ageSec === null ? '-' : `${ageSec}s`);

    const statusEl = $el('signal-health-status');
    if (statusEl) {
        statusEl.textContent = status;
        statusEl.className = `signal-health-status ${status}`;
    }
}

function _signalTfSeconds(tf) {
    const map = {
        '1m': 60,
        '3m': 180,
        '5m': 300,
        '10m': 600,
        '15m': 900,
        '30m': 1800,
        '1h': 3600,
        '4h': 14400,
        '1d': 86400,
    };
    return map[String(tf || '').toLowerCase()] || 300;
}

function _signalMarkerKey(m) {
    return `${m?.time || ''}|${m?.shape || ''}|${m?.position || ''}|${m?.text || ''}`;
}

const SIGNAL_ALERT_PREFS_KEY = 'titan_signal_alert_prefs_v1';

function _normalizeSignalAlertPrefs(raw) {
    const enabled = raw && raw.enabled === false ? false : true;
    const volRaw = Number(raw && raw.volume);
    const volume = Number.isFinite(volRaw) ? Math.max(0, Math.min(1, volRaw)) : 0.7;
    return { enabled, volume };
}

function _loadSignalAlertPrefs() {
    try {
        const raw = localStorage.getItem(SIGNAL_ALERT_PREFS_KEY);
        if (!raw) return _normalizeSignalAlertPrefs({});
        return _normalizeSignalAlertPrefs(JSON.parse(raw));
    } catch (_) {
        return _normalizeSignalAlertPrefs({});
    }
}

function _saveSignalAlertPrefs(prefs) {
    const normalized = _normalizeSignalAlertPrefs(prefs);
    try {
        localStorage.setItem(SIGNAL_ALERT_PREFS_KEY, JSON.stringify(normalized));
    } catch (_) {
        // ignore storage failures
    }
    return normalized;
}

function _applySignalAlertPrefs(prefs) {
    const cfg = _normalizeSignalAlertPrefs(prefs);
    window.__signalAlertsEnabled = cfg.enabled;
    window.__signalMute = !cfg.enabled;
    window.__signalAlertVolume = cfg.volume;

    const toggle = $el('signal-alerts-enabled');
    if (toggle) toggle.checked = cfg.enabled;
    const label = $el('signal-alerts-label');
    if (label) label.textContent = cfg.enabled ? 'ALERTS ON' : 'ALERTS OFF';

    const slider = $el('signal-alert-volume');
    if (slider) {
        slider.value = String(Math.round(cfg.volume * 100));
        slider.disabled = !cfg.enabled;
    }
    const volLabel = $el('signal-alert-volume-val');
    if (volLabel) volLabel.textContent = `${Math.round(cfg.volume * 100)}%`;
}

function bindSignalAlertControls() {
    const toggle = $el('signal-alerts-enabled');
    const slider = $el('signal-alert-volume');
    if (!toggle || !slider) return;

    _applySignalAlertPrefs(_loadSignalAlertPrefs());
    if (window.__signalAlertControlsBound) return;
    window.__signalAlertControlsBound = true;

    toggle.addEventListener('change', () => {
        const current = _loadSignalAlertPrefs();
        current.enabled = !!toggle.checked;
        const saved = _saveSignalAlertPrefs(current);
        _applySignalAlertPrefs(saved);
    });

    const onVolChange = () => {
        const current = _loadSignalAlertPrefs();
        current.volume = (Number(slider.value) || 0) / 100;
        const saved = _saveSignalAlertPrefs(current);
        _applySignalAlertPrefs(saved);
    };
    slider.addEventListener('input', onVolChange);
    slider.addEventListener('change', onVolChange);
}

function _initSignalAlertPrefs() {
    _applySignalAlertPrefs(_loadSignalAlertPrefs());
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', bindSignalAlertControls, { once: true });
    } else {
        bindSignalAlertControls();
    }
    window.addEventListener('load', bindSignalAlertControls, { once: true });
}
_initSignalAlertPrefs();

function _showSignalAlertToast(kind, text) {
    let el = $el('signal-alert-toast');
    if (!el) {
        el = document.createElement('div');
        el.id = 'signal-alert-toast';
        el.className = 'signal-alert-toast';
        document.body.appendChild(el);
    }
    el.textContent = text;
    el.className = `signal-alert-toast ${kind}`;
    requestAnimationFrame(() => {
        el.classList.add('show');
    });
    clearTimeout(window.__signalToastTimer);
    window.__signalToastTimer = setTimeout(() => {
        el.classList.remove('show');
    }, 1800);
}

function _playSignalAlertTone(kind) {
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (!AudioCtx) return;
    if (window.__signalAlertsEnabled === false || window.__signalMute === true) return;
    const volume = Math.max(0, Math.min(1, Number(window.__signalAlertVolume)));
    if (!Number.isFinite(volume) || volume <= 0) return;
    try {
        const ctx = new AudioCtx();
        const now = ctx.currentTime;
        const tonesByKind = {
            buy: [[740, 0.06, 0.00], [900, 0.07, 0.09]],
            sell: [[900, 0.06, 0.00], [680, 0.08, 0.09]],
            trail: [[660, 0.08, 0.00]],
            chaos: [[880, 0.07, 0.00], [740, 0.08, 0.10]],
        };
        const tones = tonesByKind[kind] || tonesByKind.trail;
        tones.forEach(([freq, dur, delay]) => {
            const osc = ctx.createOscillator();
            const gain = ctx.createGain();
            osc.type = 'sine';
            osc.frequency.value = freq;
            gain.gain.value = 0.0001;
            osc.connect(gain);
            gain.connect(ctx.destination);
            const start = now + delay;
            const peak = 0.008 + (0.05 * volume);
            gain.gain.exponentialRampToValueAtTime(peak, start + 0.01);
            gain.gain.exponentialRampToValueAtTime(0.0001, start + dur);
            osc.start(start);
            osc.stop(start + dur + 0.01);
        });
        setTimeout(() => ctx.close().catch(() => {}), 360);
    } catch (_) {
        // Ignore audio failures (autoplay policy, unavailable backend, etc)
    }
}

function _processSignalRealtimeAlerts(markers) {
    const modeBtn = document.querySelector('.nexus-mode-btn.active');
    if (!modeBtn || modeBtn.dataset.mode !== 'SIGNAL_STRATEGY') return;

    const toggle = $el('toggle-signals');
    if (toggle && !toggle.checked) return;
    if (!Array.isArray(markers) || markers.length === 0) return;

    const srcBtn = document.querySelector('.sig-src-btn.active');
    const src = srcBtn?.dataset?.src ? srcBtn.dataset.src.toUpperCase() : 'FUSION';
    const ctxKey = `${currentSymbol || '-'}|${currentTF || '-'}|${src}`;
    if (window.__signalAlertCtx !== ctxKey) {
        window.__signalAlertCtx = ctxKey;
        window.__signalAlertSeen = new Set(markers.map(_signalMarkerKey));
        return;
    }
    if (!(window.__signalAlertSeen instanceof Set)) {
        window.__signalAlertSeen = new Set(markers.map(_signalMarkerKey));
        return;
    }
    if (window.__signalAlertsEnabled === false || window.__signalMute === true) {
        markers.forEach((m) => window.__signalAlertSeen.add(_signalMarkerKey(m)));
        return;
    }

    const lastBarTime = Array.isArray(lastCandleData) && lastCandleData.length > 0
        ? Number(lastCandleData[lastCandleData.length - 1].time)
        : 0;
    const recentCutoff = lastBarTime > 0 ? lastBarTime - (_signalTfSeconds(currentTF) * 2) : 0;

    const fresh = [];
    markers.forEach((m) => {
        const k = _signalMarkerKey(m);
        if (window.__signalAlertSeen.has(k)) return;
        window.__signalAlertSeen.add(k);
        const txt = String(m?.text || '').toUpperCase();
        const isRealtimeSignal =
            txt.endsWith('BUY') || txt.endsWith('SELL') || txt.includes('TRAIL') || txt.includes('CHAOS');
        if (!isRealtimeSignal) return;
        const mt = Number(m?.time || 0);
        if (recentCutoff > 0 && mt < recentCutoff) return;
        fresh.push(m);
    });
    if (!fresh.length) return;

    fresh.sort((a, b) => Number(a.time || 0) - Number(b.time || 0));
    const m = fresh[fresh.length - 1];
    const txt = String(m?.text || '').toUpperCase();
    const kind = txt.endsWith('BUY')
        ? 'buy'
        : txt.endsWith('SELL')
            ? 'sell'
            : txt.includes('CHAOS')
                ? 'chaos'
                : 'trail';
    const human = kind === 'buy'
        ? 'BUY ENTRY'
        : kind === 'sell'
            ? 'SELL ENTRY'
            : kind === 'chaos'
                ? 'CHAOS EXIT'
                : 'TRAIL EXIT';
    const label = `${human}: ${txt} | ${currentSymbol || '-'} ${currentTF || '-'}`;
    _showSignalAlertToast(kind, label);
    _playSignalAlertTone(kind);
    if (typeof titanLogAdd === 'function') {
        const colors = {
            buy: '#39FF14',
            sell: '#FF8C00',
            trail: '#00D9FF',
            chaos: '#FF4FC3',
        };
        titanLogAdd(`🔔 ${label}`, colors[kind] || '#00D9FF');
    }
}

let _cachedToggleSignals = null;

function updateSignalMarkers() {
    if (!candleSeries || !lastCandleData) return;

    if (!_cachedToggleSignals) {
        _cachedToggleSignals = $el('toggle-signals');
    }
    const toggle = _cachedToggleSignals;
    const engine = window._titanMarkerEngine;

    if (!engine) {
        TitanDebug.log('markers', 'WARNING: _titanMarkerEngine not initialized — signals will not render');
        return;
    }

    if (toggle && !toggle.checked) {
        if (engine) engine.update('signals', []);
    } else {
        const raw = signalMarkers || [];
        let markers = raw;

        if (markers.length > 0 && lastCandleData.length > 0) {
            const visibleData = (replayState && replayState.active && replayState.data)
                ? replayState.data.slice(0, (replayState.index || 0) + 1)
                : lastCandleData;
            const candleTimes = new Set(visibleData.map(c => c.time));
            markers = raw.filter(m => candleTimes.has(m.time));

            if (markers.length === 0 && raw.length > 0) {
                TitanDebug.log('markers',
                    `WARNING: ${raw.length} signal markers exist but NONE match ${visibleData.length} candle times — filtered to 0`);
            } else if (markers.length < raw.length) {
                TitanDebug.log('markers',
                    `Filtered markers: ${markers.length}/${raw.length} matched candle times`);
            }
        }

        if (engine) engine.update('signals', markers);

        const debugEl = $el('debug-marker-count');
        if (debugEl) {
            debugEl.innerText = `Signals: ${markers.length}${markers.length < raw.length ? ` (${raw.length} raw)` : ''}`;
        }

        const reasonEl = $el('debug-marker-reason');
        if (reasonEl) {
            const reason = window.__signalMarkerReason;
            if (markers.length === 0 && raw.length > 0) {
                reasonEl.innerText = 'Markers exist but no candle time match (TF mismatch?)';
            } else if (markers.length === 0 && reason) {
                reasonEl.innerText = reason;
            } else {
                reasonEl.innerText = '';
            }
        }
    }
    _processSignalRealtimeAlerts(signalMarkers || []);
    updateSignalHealthPanel();
}

function setSeriesVisible(series, visible) {
    if (series) series.applyOptions({ visible });
}

function setPaneVisible(id, visible) {
    const el = $el(id);
    if (!el) return;
    el.style.display = visible ? 'block' : 'none';
    resizeCharts();
}

let oracleFullscreen = false;
function toggleOracleFullscreen() {
    const el = $el('chart-oracle');
    const btn = $el('oracle-fs-btn');
    if (!el) return;
    oracleFullscreen = !oracleFullscreen;
    if (oracleFullscreen) {
        el.classList.add('oracle-fullscreen');
        if (btn) { btn.textContent = '✕'; btn.style.background = '#FF2B2B'; btn.style.color = '#FFF'; }
        if (oracleChart) {
            oracleChart.resize(window.innerWidth, window.innerHeight);
            oracleChart.timeScale().fitContent();
        }
    } else {
        el.classList.remove('oracle-fullscreen');
        if (btn) { btn.textContent = 'FS'; btn.style.background = '#FFD700'; btn.style.color = '#0a0a0f'; }
        resizeCharts();
    }
}
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && oracleFullscreen) toggleOracleFullscreen();
});
window.addEventListener('resize', () => {
    if (oracleFullscreen && oracleChart) {
        oracleChart.resize(window.innerWidth, window.innerHeight);
    }
});

function bindIndicatorToggles() {
    const onChange = () => {
        applyIndicatorToggleState();
        saveChartSettings();
    };
    ['toggle-ema9','toggle-ema50','toggle-ema200','toggle-vol','toggle-bb',
     'toggle-vwap','toggle-signals','toggle-rsi','toggle-macd','toggle-adx','toggle-oracle'].forEach(id => {
        const el = $el(id);
        if (el) el.addEventListener('change', onChange);
    });
    applyIndicatorToggleState();
}

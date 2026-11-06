function chartSettingsKey() {
    return 'titan_chart_settings_v1';
}

function setChartToggleChecked(id, value) {
    const el = $el(id);
    if (!el) return;
    el.checked = !!value;
}

function readChartToggleState(id, fallback = true) {
    const el = $el(id);
    if (!el) return fallback;
    return !!el.checked;
}

function applyIndicatorToggleState() {
    const ema9 = $el('toggle-ema9');
    const ema50 = $el('toggle-ema50');
    const ema200 = $el('toggle-ema200');
    const volToggle = $el('toggle-vol');
    const bb = $el('toggle-bb');
    const vwapToggle = $el('toggle-vwap');
    const signals = $el('toggle-signals');
    const rsiToggle = $el('toggle-rsi');
    const macdToggle = $el('toggle-macd');
    const adxToggle = $el('toggle-adx');
    const oracleToggle = $el('toggle-oracle');

    if (ema9) setSeriesVisible(ema9Series, ema9.checked);
    if (ema50) setSeriesVisible(ema50Series, ema50.checked);
    if (ema200) setSeriesVisible(ema200Series, ema200.checked);
    if (volToggle) setPaneVisible('chart-vol', volToggle.checked);
    if (vwapToggle) setSeriesVisible(vwapSeries, vwapToggle.checked);
    if (bb) {
        setSeriesVisible(bbUpperSeries, bb.checked);
        setSeriesVisible(bbMidSeries, bb.checked);
        setSeriesVisible(bbLowerSeries, bb.checked);
    }
    if (signals) updateSignalMarkers();
    if (rsiToggle) setPaneVisible('chart-rsi', rsiToggle.checked);
    if (macdToggle) setPaneVisible('chart-macd', macdToggle.checked);
    if (adxToggle) setPaneVisible('chart-adx', adxToggle.checked);
    if (oracleToggle) setPaneVisible('chart-oracle', oracleToggle.checked);
}

function loadChartSettings() {
    return storageGet(chartSettingsKey(), null);
}

function _buildChartSettingsPayload() {
    return {
        timeframe: currentTF,
        indicatorConfig: indicatorConfig,
        toggles: {
            ema9: readChartToggleState('toggle-ema9', true),
            ema50: readChartToggleState('toggle-ema50', true),
            ema200: readChartToggleState('toggle-ema200', true),
            vol: readChartToggleState('toggle-vol', true),
            bb: readChartToggleState('toggle-bb', true),
            vwap: readChartToggleState('toggle-vwap', true),
            signals: readChartToggleState('toggle-signals', true),
            rsi: readChartToggleState('toggle-rsi', true),
            macd: readChartToggleState('toggle-macd', true),
            adx: readChartToggleState('toggle-adx', true),
            oracle: readChartToggleState('toggle-oracle', true)
        }
    };
}

let _chartSaveDebounce = null;
let _lastSavedChartSnapshot = null;

function saveChartSettings() {
    if (!chartSettingsLoaded) return;

    if (_lastSavedChartSnapshot) {
        const _inv = typeof invoke === 'function' ? invoke : (typeof window.invoke === 'function' ? window.invoke : null);
        if (_inv) _inv('save_chart_undo', { data: _lastSavedChartSnapshot }).catch(() => {});
    }

    const settings = _buildChartSettingsPayload();
    _lastSavedChartSnapshot = JSON.parse(JSON.stringify(settings));
    storageSet(chartSettingsKey(), settings);
    clearTimeout(_chartSaveDebounce);
    _chartSaveDebounce = setTimeout(() => {
        if (typeof invoke === 'function') {
            invoke('save_chart_settings', { data: settings }).catch(() => {});
        } else if (typeof window.invoke === 'function') {
            window.invoke('save_chart_settings', { data: settings }).catch(() => {});
        }
    }, 500);
}

function applyChartSettingsFromStorage() {
    let settings = loadChartSettings();
    if (settings) {
        _applyChartSettingsObj(settings);
        _lastSavedChartSnapshot = JSON.parse(JSON.stringify(settings));
    } else {
        applyIndicatorToggleState();
        _lastSavedChartSnapshot = _buildChartSettingsPayload();
    }
    chartSettingsLoaded = true;
}

function _applyChartSettingsObj(settings) {
    const toggles = settings.toggles || {};
    const forceManual = getToggleState('chk-force-manual');
    if (!forceManual && settings.timeframe) {
        setTF(settings.timeframe, null, { force: true });
    }
    if (settings.indicatorConfig) {
        applyIndicatorConfig(settings.indicatorConfig);
    }
    setChartToggleChecked('toggle-ema9', toggles.ema9 ?? true);
    setChartToggleChecked('toggle-ema50', toggles.ema50 ?? true);
    setChartToggleChecked('toggle-ema200', toggles.ema200 ?? true);
    setChartToggleChecked('toggle-vol', toggles.vol ?? true);
    setChartToggleChecked('toggle-bb', toggles.bb ?? true);
    setChartToggleChecked('toggle-vwap', toggles.vwap ?? true);
    setChartToggleChecked('toggle-signals', toggles.signals ?? true);
    setChartToggleChecked('toggle-rsi', toggles.rsi ?? true);
    setChartToggleChecked('toggle-macd', toggles.macd ?? true);
    setChartToggleChecked('toggle-adx', toggles.adx ?? true);
    setChartToggleChecked('toggle-oracle', toggles.oracle ?? true);
    applyIndicatorToggleState();
}

function captureChartUndo() {
    const snapshot = _buildChartSettingsPayload();
    const _inv = typeof invoke === 'function' ? invoke : (typeof window.invoke === 'function' ? window.invoke : null);
    if (_inv) {
        _inv('save_chart_undo', { data: snapshot }).catch(() => {});
    }
}

async function undoChartSettings() {
    const _inv = typeof invoke === 'function' ? invoke : (typeof window.invoke === 'function' ? window.invoke : null);
    if (!_inv) return;
    try {
        const snapshot = await _inv('load_chart_undo');
        if (snapshot && typeof snapshot === 'object') {
            _applyChartSettingsObj(snapshot);
            saveChartSettings();
            const badge = $el('chart-undo-badge');
            if (badge) {
                badge.textContent = 'RESTORED';
                badge.style.display = 'inline-block';
                setTimeout(() => { badge.style.display = 'none'; }, 2500);
            }
        }
    } catch {
        const badge = $el('chart-undo-badge');
        if (badge) {
            badge.textContent = 'NO SNAPSHOT';
            badge.style.display = 'inline-block';
            setTimeout(() => { badge.style.display = 'none'; }, 2500);
        }
    }
}

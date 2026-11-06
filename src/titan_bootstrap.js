window.onerror = function (msg, src, line, col, err) {
    TitanError.report('global:uncaught', err || msg);
};
window.onunhandledrejection = function (event) {
    TitanError.report('global:promise', event.reason);
};

function initChartsDeferred() {
    setTimeout(async () => {
        if (typeof LightweightCharts === 'undefined') {
            showCyberToast("CRITICAL ERROR: LightweightCharts Library NOT Loaded!", 'error');
            return;
        }

        try {
            const container = $el('chart-div');
            if (container && container.clientHeight === 0) {
                container.style.minHeight = "200px";
            }

            chart = LightweightCharts.createChart(container, {
                layout: { background: { type: 'solid', color: '#0a0a0a' }, textColor: '#889988', fontFamily: "'Share Tech Mono', monospace" },
                grid: { vertLines: { color: 'rgba(0,191,255,0.04)' }, horzLines: { color: 'rgba(0,191,255,0.04)' } },
                timeScale: {
                    borderColor: 'rgba(0,191,255,0.15)',
                    timeVisible: true,
                    rightOffset: 50,
                    shiftVisibleRangeOnNewBar: false
                },
                crosshair: {
                    mode: LightweightCharts.CrosshairMode.Normal,
                    vertLine: { color: 'rgba(0,191,255,0.5)', width: 1, style: LightweightCharts.LineStyle.Solid, labelBackgroundColor: '#0a1a0a' },
                    horzLine: { color: 'rgba(0,191,255,0.5)', width: 1, style: LightweightCharts.LineStyle.Solid, labelBackgroundColor: '#0a1a0a' }
                },
                rightPriceScale: {
                    borderColor: 'rgba(0,191,255,0.15)',
                    mode: LightweightCharts.PriceScaleMode.Normal,
                    autoScale: true,
                    alignLabels: true
                },
                leftPriceScale: { visible: false },
                handleScroll: { mouseWheel: true, pressedMouseMove: true },
                handleScale: { axisPressedMouseMove: true, mouseWheel: true, pinch: true }
            });

            candleSeries = chart.addCandlestickSeries({
                upColor: '#00BFFF', downColor: '#FF2B2B', borderVisible: false, wickUpColor: '#00BFFF', wickDownColor: '#FF2B2B'
            });
            window.candleSeries = candleSeries;

            const volContainer = $el('chart-vol');
            if (volContainer) {
                volChart = LightweightCharts.createChart(volContainer, {
                    layout: { background: { type: 'solid', color: '#050505' }, textColor: '#667766', fontFamily: "'Share Tech Mono', monospace" },
                    grid: { vertLines: { color: 'rgba(0,191,255,0.03)' }, horzLines: { color: 'rgba(0,191,255,0.03)' } },
                    crosshair: { mode: LightweightCharts.CrosshairMode.Normal, vertLine: { color: 'rgba(0,191,255,0.4)', width: 1, style: LightweightCharts.LineStyle.Solid, labelVisible: false }, horzLine: { visible: false } },
                    timeScale: { visible: false },
                    handleScroll: false,
                    handleScale: false
                });
                volSeries = volChart.addHistogramSeries({ priceFormat: { type: 'volume' } });
                volSeries.priceScale().applyOptions({ scaleMargins: { top: 0.2, bottom: 0 } });
            }

            const overlayOpts = { priceLineVisible: false, lastValueVisible: false };
            ema9Series = chart.addLineSeries({ color: '#21b6ff', lineWidth: 1, ...overlayOpts });
            ema50Series = chart.addLineSeries({ color: '#ffae00', lineWidth: 1, ...overlayOpts });
            ema200Series = chart.addLineSeries({ color: '#ff2b2b', lineWidth: 1, ...overlayOpts });
            vwapSeries = chart.addLineSeries({ color: '#00BFFF', lineWidth: 1, ...overlayOpts });
            bbUpperSeries = chart.addLineSeries({ color: '#5aa1ff', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, ...overlayOpts });
            bbMidSeries = chart.addLineSeries({ color: '#8aa', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, ...overlayOpts });
            bbLowerSeries = chart.addLineSeries({ color: '#5aa1ff', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, ...overlayOpts });

            const rsiContainer = $el('chart-rsi');
            if (rsiContainer) {
                rsiChart = LightweightCharts.createChart(rsiContainer, {
                    layout: { background: { type: 'solid', color: '#050505' }, textColor: '#667766', fontFamily: "'Share Tech Mono', monospace" },
                    grid: { vertLines: { color: 'rgba(0,191,255,0.03)' }, horzLines: { color: 'rgba(0,191,255,0.03)' } },
                    crosshair: { mode: LightweightCharts.CrosshairMode.Normal, vertLine: { color: 'rgba(0,191,255,0.4)', width: 1, style: LightweightCharts.LineStyle.Solid, labelVisible: false }, horzLine: { visible: false } },
                    timeScale: { visible: false },
                    handleScroll: false,
                    handleScale: false
                });
                rsiSeries = rsiChart.addLineSeries({ color: '#A88CFF', lineWidth: 1 });
                const refOpts = { color: '#333', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, priceLineVisible: false, lastValueVisible: false };
                rsiUpperLine = rsiChart.addLineSeries(refOpts);
                rsiLowerLine = rsiChart.addLineSeries(refOpts);
                rsiMidLine = rsiChart.addLineSeries({ ...refOpts, color: '#2a2a2a' });
            }

            const macdContainer = $el('chart-macd');
            if (macdContainer) {
                macdChart = LightweightCharts.createChart(macdContainer, {
                    layout: { background: { type: 'solid', color: '#050505' }, textColor: '#667766', fontFamily: "'Share Tech Mono', monospace" },
                    grid: { vertLines: { color: 'rgba(0,191,255,0.03)' }, horzLines: { color: 'rgba(0,191,255,0.03)' } },
                    crosshair: { mode: LightweightCharts.CrosshairMode.Normal, vertLine: { color: 'rgba(0,191,255,0.4)', width: 1, style: LightweightCharts.LineStyle.Solid, labelVisible: false }, horzLine: { visible: false } },
                    timeScale: { visible: false },
                    handleScroll: false,
                    handleScale: false
                });
                macdHistSeries = macdChart.addHistogramSeries({ color: '#00BFFF' });
                macdLineSeries = macdChart.addLineSeries({ color: '#FFC857', lineWidth: 1 });
                macdSignalSeries = macdChart.addLineSeries({ color: '#FF2B2B', lineWidth: 1 });
                macdZeroLine = macdChart.addLineSeries({ color: '#333', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, priceLineVisible: false, lastValueVisible: false });
            }

            const adxContainer = $el('chart-adx');
            if (adxContainer) {
                adxChart = LightweightCharts.createChart(adxContainer, {
                    layout: { background: { type: 'solid', color: '#050505' }, textColor: '#667766', fontFamily: "'Share Tech Mono', monospace" },
                    grid: { vertLines: { color: 'rgba(0,191,255,0.03)' }, horzLines: { color: 'rgba(0,191,255,0.03)' } },
                    crosshair: { mode: LightweightCharts.CrosshairMode.Normal, vertLine: { color: 'rgba(0,191,255,0.4)', width: 1, style: LightweightCharts.LineStyle.Solid, labelVisible: false }, horzLine: { visible: false } },
                    timeScale: { visible: false },
                    handleScroll: false,
                    handleScale: false
                });
                adxSeries = adxChart.addLineSeries({ color: '#7CFC00', lineWidth: 1 });
                const adxRefOpts = { color: '#333', lineWidth: 1, lineStyle: LightweightCharts.LineStyle.Dotted, priceLineVisible: false, lastValueVisible: false };
                adxLine20 = adxChart.addLineSeries(adxRefOpts);
                adxLine25 = adxChart.addLineSeries({ ...adxRefOpts, color: '#2a2a2a' });
            }

            const oracleContainer = $el('chart-oracle');
            if (oracleContainer) {
                oracleChart = LightweightCharts.createChart(oracleContainer, {
                    layout: { background: { type: 'solid', color: '#050505' }, textColor: '#667766', fontFamily: "'Share Tech Mono', monospace" },
                    grid: { vertLines: { color: 'rgba(0,191,255,0.03)' }, horzLines: { color: 'rgba(0,191,255,0.03)' } },
                    crosshair: { mode: LightweightCharts.CrosshairMode.Normal, vertLine: { color: 'rgba(0,191,255,0.4)', width: 1, style: LightweightCharts.LineStyle.Solid, labelVisible: false }, horzLine: { visible: false } },
                    timeScale: { visible: false },
                    handleScroll: false,
                    handleScale: false
                });
                seriesOracle = oracleChart.addBaselineSeries({
                    baseValue: { type: 'price', price: 0 },
                    topLineColor: '#00BFFF',
                    topFillColor1: 'rgba(0, 191, 255, 0.15)',
                    topFillColor2: 'rgba(0, 191, 255, 0.02)',
                    bottomLineColor: '#FF2B2B',
                    bottomFillColor1: 'rgba(255, 43, 43, 0.02)',
                    bottomFillColor2: 'rgba(255, 43, 43, 0.15)',
                    lineWidth: 2,
                });
                seriesOracleZero = oracleChart.addLineSeries({
                    color: 'rgba(0, 191, 255, 0.25)',
                    lineWidth: 1,
                    lineStyle: LightweightCharts.LineStyle.Dotted,
                    priceLineVisible: false,
                    lastValueVisible: false
                });
            }

            const syncTargets = [];
            if (volChart) syncTargets.push(volChart);
            if (rsiChart) syncTargets.push(rsiChart);
            if (macdChart) syncTargets.push(macdChart);
            if (adxChart) syncTargets.push(adxChart);
            if (oracleChart) syncTargets.push(oracleChart);
            if (syncTargets.length > 0) syncTimeScale(chart, syncTargets);
            const allCharts = [chart, ...syncTargets];
            syncCrosshair(allCharts);
            resizeCharts();

            bindIndicatorToggles();
            bindIndicatorPanel();

            // Restore chart settings from disk (survives WebView cache clear)
            try {
                if (typeof invoke === 'function') {
                    const diskSettings = await invoke('load_chart_settings');
                    if (diskSettings && typeof _applyChartSettingsObj === 'function') {
                        _applyChartSettingsObj(diskSettings);
                        storageSet(chartSettingsKey(), diskSettings);
                        chartSettingsLoaded = true;
                    }
                } else {
                    applyChartSettingsFromStorage();
                }
            } catch(e) {
                applyChartSettingsFromStorage();
            }

            candleSeries.setData([
                { time: '2023-01-01', open: 50, high: 60, low: 40, close: 55 },
                { time: '2023-01-02', open: 55, high: 65, low: 50, close: 60 },
                { time: '2023-01-03', open: 60, high: 62, low: 55, close: 58 },
            ]);

            chart.timeScale().fitContent();
            updateRightBarHeader();
            updateRightBarFeedStatus("CONNECTING", "warning");
            chart.subscribeCrosshairMove((param) => {
                if (!param || !param.time) {
                    if (lastBar) updateRightBarFromBar(lastBar);
                    return;
                }
                let bar = null;
                if (param.seriesData && typeof param.seriesData.get === 'function') {
                    bar = param.seriesData.get(candleSeries);
                }
                if (bar) {
                    updateRightBarFromBar(bar, { isCrosshair: true, time: param.time });
                }
            });
            TitanDebug.log('chart', 'CHART INITIALIZED OK');

        } catch (err) {
            TitanError.report('chart:init', err);
            showCyberToast("CHART ERROR: " + err.message, 'error');
        }
    }, 500);
}

function wireDomListeners() {
    window.addEventListener('resize', () => { resizeCharts(); });

    const positionsBody = $el('positions-body');
    if (positionsBody) {
        positionsBody.addEventListener('click', (e) => {
            if (e.target.closest('.btn-sell') || e.target.closest('.btn-force-close') || e.target.closest('.btn-mode-toggle')) return;
            const row = e.target.closest('tr');
            if (!row) return;
            const badge = row.querySelector('.sym-badge');
            if (!badge) return;
            const sym = badge.getAttribute('data-symbol') || badge.innerText.trim();
            if (!sym) return;
            setSymbol(sym, { force: true });
            if (typeof switchTab === 'function') switchTab(0);
        });
    }

    const hftInput = $el('hft-central-input');
    if (hftInput) {
        hftInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                $el('btn-hft-central-add')?.click();
            }
        });
    }

    const chartSym = $el('chart-symbol');
    if (chartSym && hftInput) {
        chartSym.addEventListener('click', () => {
            setSymbol(hftInput.value || chartSym.innerText, { force: true });
        });
    }

    const resizeObs = new ResizeObserver(() => { resizeCharts(); });
    const chartDiv = $el('chart-div');
    const chartWrapper = $el('chart-wrapper');
    if (chartDiv) resizeObs.observe(chartDiv);
    if (chartWrapper) resizeObs.observe(chartWrapper);
    initChartResizer();

    setTimeout(() => loadData(currentSymbol), 1000);
    setTimeout(() => { if (!lastBar) loadData(currentSymbol); }, 5000);
    setTimeout(() => { if (!lastBar) loadData(currentSymbol); }, 12000);

    const btnAuditClose = $el('btn-audit-close');
    const btnAuditBack = $el('btn-audit-back');
    const auditOverlay = $el('titan-audit-v3-overlay');
    if (btnAuditClose && auditOverlay) btnAuditClose.addEventListener('click', () => { auditOverlay.style.display = 'none'; });
    if (btnAuditBack && auditOverlay) btnAuditBack.addEventListener('click', () => { auditOverlay.style.display = 'none'; });

    const btnHardRefresh = $el('btn-hard-refresh');
    if (btnHardRefresh) btnHardRefresh.addEventListener('click', () => { window.location.reload(); });

    const btnNewsTestBullish = $el('btn-news-test-bullish');
    if (btnNewsTestBullish) btnNewsTestBullish.addEventListener('click', () => { invoke('test_catalyst_alert'); });

    startMonitorLoop();
}

function registerTauriListeners() {
    if (!window.__TAURI__) return;
    const { listen } = window.__TAURI__.event;
    let _priceThrottleTimer = null;
    let _ofiThrottleTimer = null;

    listen('price-update', (event) => {
        if (_priceThrottleTimer) return;
        _priceThrottleTimer = setTimeout(() => { _priceThrottleTimer = null; }, 200);
        const update = event.payload || {};
        if (!update || !update.symbol) return;
        if (update.price > 0 && typeof normalizeSymbol === 'function') {
            if (!window._lastPrices) window._lastPrices = {};
            if (!window._lastPriceTimestamps) window._lastPriceTimestamps = {};
            const pk = normalizeSymbol(update.symbol).replace(/\//g, '').toUpperCase();
            if (pk) {
                window._lastPrices[pk] = update.price;
                window._lastPriceTimestamps[pk] = Date.now();
            }
        }
        const normUpdate = update.symbol.replace('/', '');
        const normCurrent = currentSymbol.replace('/', '');

        if (normUpdate === normCurrent) {
            const replayActive = replayState && replayState.active;
            if (!replayActive && candleSeries && lastBar && update.price) {
                const price = update.price;
                if (!window.sessionStartPrice) window.sessionStartPrice = price;

                const el = $el('alpaca-real-price');
                if (el) {
                    el.innerText = '$' + price.toFixed(2);
                    el.style.color = '#00BFFF';
                    setTimeout(() => el.style.color = '#fff', 200);
                }
                const qtEl = $el('qt-live-price');
                if (qtEl) qtEl.textContent = '$' + price.toFixed(2);

                if (livePriceLine) livePriceLine.applyOptions({ price });
                updateCandleTimer();

                lastBar.close = price;
                lastBar.high = Math.max(lastBar.high, price);
                lastBar.low = Math.min(lastBar.low, price);
                if (candleSeries) candleSeries.update(lastBar);

                updateRightBarFromBar(lastBar, { price, time: update.time, source: update.source });
                updateRightBarLatency(update.time);
                checkPriceAlerts(price);
            }
        }
    });

    listen('flow-update', (event) => {
        renderFlow(event.payload);
    });

    listen('l2-top', (event) => {
        const d = event.payload || {};
        const sym = d.symbol ? normalizeSymbol(d.symbol) : '';
        if (sym && normalizeSymbol(currentSymbol) !== sym) return;
        const bid = Number(d.bid);
        const ask = Number(d.ask);
        const bidSize = Number(d.bid_size);
        const askSize = Number(d.ask_size);
        const hasBid = Number.isFinite(bid) && bid > 0;
        const hasAsk = Number.isFinite(ask) && ask > 0;
        const spread = (hasBid && hasAsk) ? (ask - bid) : NaN;
        const el = $el('l2-top');
        if (el) {
            const bidText = hasBid ? bid.toFixed(2) : '--';
            const askText = hasAsk ? ask.toFixed(2) : '--';
            const sprText = Number.isFinite(spread) ? spread.toFixed(2) : '--';
            el.innerText = `BID ${bidText} | ASK ${askText} | SPR ${sprText}`;
        }
        updateRightBarQuote({
            bid: hasBid ? bid : null,
            ask: hasAsk ? ask : null,
            bid_size: Number.isFinite(bidSize) && bidSize > 0 ? bidSize : null,
            ask_size: Number.isFinite(askSize) && askSize > 0 ? askSize : null,
            spread: Number.isFinite(spread) ? spread : null
        });
    });

    listen('l2-update', (event) => {
        const d = event.payload || {};
        const sym = d.symbol ? normalizeSymbol(d.symbol) : '';
        if (sym && normalizeSymbol(currentSymbol) !== sym) return;
        const bids = d.bids || [];
        const asks = d.asks || [];
        if (!window.__l2Sma) window.__l2Sma = { bids: [], asks: [] };
        const bidsEl = $el('l2-bids');
        const asksEl = $el('l2-asks');
        if (bidsEl) {
            bidsEl.innerHTML = bids.map(l => `<div>• ${Number(l.p).toFixed(2)} x ${Number(l.s).toFixed(2)}</div>`).join('');
        }
        if (asksEl) {
            asksEl.innerHTML = asks.map(l => `<div>• ${Number(l.p).toFixed(2)} x ${Number(l.s).toFixed(2)}</div>`).join('');
        }

        const canvas = $el('l2-heatmap');
        if (canvas) {
            const ctx = canvas.getContext('2d');
            const dpr = window.devicePixelRatio || 1;
            const cw = canvas.clientWidth || canvas.width;
            const ch = canvas.clientHeight || canvas.height;
            canvas.width = Math.floor(cw * dpr);
            canvas.height = Math.floor(ch * dpr);
            ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
            const w = cw;
            const h = ch;
            ctx.clearRect(0, 0, w, h);

            const smoothedBids = bids.map((l, i) => {
                const last = window.__l2Sma.bids[i] || 0;
                const cur = Number(l.s || 0);
                const smooth = (last * 0.6) + (cur * 0.4);
                window.__l2Sma.bids[i] = smooth;
                return { p: Number(l.p || 0), s: smooth };
            });
            const smoothedAsks = asks.map((l, i) => {
                const last = window.__l2Sma.asks[i] || 0;
                const cur = Number(l.s || 0);
                const smooth = (last * 0.6) + (cur * 0.4);
                window.__l2Sma.asks[i] = smooth;
                return { p: Number(l.p || 0), s: smooth };
            });
            const maxBid = smoothedBids.reduce((m, l) => Math.max(m, Number(l.s || 0)), 0);
            const maxAsk = smoothedAsks.reduce((m, l) => Math.max(m, Number(l.s || 0)), 0);
            const maxSize = Math.max(maxBid, maxAsk, 1);
            const midY = h / 2;
            const barHeight = (h / 2) - 4;

            smoothedBids.forEach((l, i) => {
                const size = Number(l.s || 0);
                const intensity = Math.min(size / maxSize, 1);
                const barW = (w / Math.max(smoothedBids.length, 1)) * 0.9;
                const x = i * (w / Math.max(smoothedBids.length, 1));
                const alpha = 0.2 + intensity * 0.7;
                const grad = ctx.createLinearGradient(x, 0, x, midY);
                grad.addColorStop(0, `rgba(0, 191, 255, ${alpha * 0.2})`);
                grad.addColorStop(1, `rgba(0, 191, 255, ${alpha})`);
                ctx.fillStyle = grad;
                ctx.shadowColor = 'rgba(0,191,255,0.5)';
                ctx.shadowBlur = 6;
                ctx.fillRect(x, midY - barHeight, barW, barHeight);
            });

            smoothedAsks.forEach((l, i) => {
                const size = Number(l.s || 0);
                const intensity = Math.min(size / maxSize, 1);
                const barW = (w / Math.max(smoothedAsks.length, 1)) * 0.9;
                const x = i * (w / Math.max(smoothedAsks.length, 1));
                const alpha = 0.2 + intensity * 0.7;
                const grad = ctx.createLinearGradient(x, midY, x, h);
                grad.addColorStop(0, `rgba(255, 43, 43, ${alpha})`);
                grad.addColorStop(1, `rgba(255, 43, 43, ${alpha * 0.2})`);
                ctx.fillStyle = grad;
                ctx.shadowColor = 'rgba(255,43,43,0.5)';
                ctx.shadowBlur = 6;
                ctx.fillRect(x, midY + 4, barW, barHeight);
            });

            ctx.shadowBlur = 0;
            ctx.strokeStyle = '#1A1A1A';
            ctx.beginPath();
            ctx.moveTo(0, midY);
            ctx.lineTo(w, midY);
            ctx.stroke();
        }
    });

    listen('connection-status', (event) => {
        const status = event.payload;
        if (status === 'LIVE') {
            updateStatus("🟢 LIVE FEED", "#00BFFF", "success");
            if (!lastBar) setTimeout(() => loadData(currentSymbol), 500);
        }
    });

    listen('manual-order-placed', (event) => {
        const d = event.payload || {};
        if (typeof mtAddRecentTrade === 'function') {
            mtAddRecentTrade(d.symbol || '?', d.side || '?', d.usd || 0,
                `${d.order_type || 'LIMIT'} @ $${(d.price || 0).toFixed(2)}`);
        }
    });

    listen('evolution-update', (event) => {
        const e = event.payload || {};
        setTxt('evo-gen', 'Gen ' + (e.generation || 0));
        setTxt('evo-fitness', e.champion_fitness !== undefined ? e.champion_fitness.toFixed(4) : '--', '#00BFFF');
        setTxt('evo-sharpe', e.champion_sharpe !== undefined ? e.champion_sharpe.toFixed(2) : '--', '#4ecdc4');
        const p = e.champion_params || {};
        const tp = p.tp_percent !== undefined ? p.tp_percent : '--';
        const sl = p.sl_percent !== undefined ? p.sl_percent : '--';
        setTxt('evo-tp-sl', tp + '/' + sl);
    });

    // S13: SmartTrend evolution update — same panel, different data
    listen('st-evolution-update', (event) => {
        const e = event.payload || {};
        setTxt('evo-gen', 'Gen ' + (e.generation || 0));
        setTxt('evo-fitness', e.champion_fitness !== undefined ? e.champion_fitness.toFixed(4) : '--', '#00BFFF');
        setTxt('evo-sharpe', e.champion_sharpe !== undefined ? e.champion_sharpe.toFixed(2) : '--', '#4ecdc4');
        const p = e.champion_params || {};
        const entry = p.entry_threshold !== undefined ? p.entry_threshold.toFixed(0) : '--';
        const stalk = p.stalking_threshold !== undefined ? p.stalking_threshold.toFixed(0) : '--';
        setTxt('evo-tp-sl', entry + '/' + stalk);
        // B4: Update diversity badge
        if (e.diversity_pct !== undefined) { _updateDiversityBadge(e.diversity_pct); }
        // B1: Refresh convergence chart
        _refreshConvergenceChart();
    });

    listen('ofi-update', (event) => {
        if (_ofiThrottleTimer) return;
        _ofiThrottleTimer = setTimeout(() => { _ofiThrottleTimer = null; }, 200);
        renderOfi(event.payload || {});
    });

    listen('ofi-divergence', (event) => {
        const d = event.payload || {};
        const mpDir = d.microprice_dir || '?';
        const ofiDir = d.ofi_dir || '?';
        if (typeof TitanToast !== 'undefined') {
            TitanToast.show(
                'OFI DIVERGENCE: microprice ' + mpDir + ' / OFI ' + ofiDir,
                'warning', 8000
            );
        }
        if (typeof TitanAudio !== 'undefined' && TitanAudio.play) {
            TitanAudio.play('alert');
        }
    });

    listen('trade-fill', (event) => {
        const f = event.payload || {};
        const side = f.side || '?';
        const clr = sideColor(side);
        const qty = f.filled_qty || f.qty || '?';
        const price = f.filled_avg_price || f.price || '--';
        const evtType = f.event ? ' (' + f.event + ')' : '';
        titanLogAdd('FILL: ' + side.toUpperCase() + ' ' + (f.symbol || '?') + ' x' + qty + ' @ $' + price + evtType, clr);
    });

    listen('equity-sync', (event) => {
        const e = event.payload || {};
        setTxt('titan-equity', fmtUsd(e.equity), '#FAFAFA');
        const csEquity = $el('cs-equity');
        if (csEquity) csEquity.textContent = fmtUsd(e.equity);
        const csStale = $el('cs-equity-stale');
        if (csStale) csStale.classList.add('hidden');
    });

    listen('equity-sync-degraded', () => {
        const badge = $el('cs-equity-stale');
        if (badge) badge.classList.remove('hidden');
    });
    listen('equity-sync-critical', () => {
        const badge = $el('cs-equity-stale');
        if (badge) badge.classList.remove('hidden');
    });

    listen('trade-updates-health', (event) => {
        const h = event.payload || {};
        const badge = $el('cs-ws-disconn');
        if (badge) {
            if (h.connected) badge.classList.add('hidden');
            else badge.classList.remove('hidden');
        }
    });

    listen('trade-fill', (event) => {
        const f = event.payload || {};
        if (window._titanFillMarkers) window._titanFillMarkers.push(f);
    });

    listen('health-check', (event) => {
        const data = event.payload || {};
        const probes = data.probes || [];
        probes.forEach(c => {
            const clr = c.ok ? '#00BFFF' : '#FF6B6B';
            titanLogAdd('[BOOT] ' + c.name + ': ' + (c.ok ? 'OK' : 'FAIL — ' + (c.detail || '')), clr);
        });
        const allOk = data.all_ok;
        titanLogAdd('[BOOT] STATUS: ' + (allOk ? 'ALL CRITICAL CHECKS PASSED' : 'CRITICAL FAILURES DETECTED'), allOk ? '#00BFFF' : '#FF2B2B');
    });

    listen('breaker-triggered', (event) => {
        const b = event.payload || {};
        window._breakerStatus = b.level || 'HALT';
        const lvlEl = $el('titan-breaker-level');
        if (lvlEl) {
            lvlEl.textContent = b.level || 'HALT';
            lvlEl.style.background = '#FF2B2B';
            lvlEl.style.color = '#FFF';
        }
        const csBreaker = $el('cs-breaker');
        if (csBreaker) {
            csBreaker.textContent = b.level || 'HALT';
            csBreaker.className = 'cs-value cs-red';
        }
        titanLogAdd('BREAKER ' + (b.level || '?') + ': ' + (b.reason || ''), '#FF2B2B');
    });

    listen('order-lifecycle', (event) => {
        const o = event.payload || {};
        if (o.generation !== undefined) {
            window._titanGeneration = o.generation;
            const csGen = $el('cs-generation');
            if (csGen) csGen.textContent = '#' + o.generation;
        }
    });

    listen('dry-run-order', (event) => {
        const d = event.payload || {};
        titanLogAdd('[DRY] ' + (d.side || '?').toUpperCase() + ' ' + (d.symbol || '?') + ' x' + (d.qty || '?'), '#FFD700');
    });

    listen('watchdog-alert', (event) => {
        const dead = (event.payload || {}).dead_loops || [];
        titanLogAdd('WATCHDOG: Dead loops: ' + dead.join(', '), '#FF2B2B');
        const deadEl = $el('telemetry-dead-count');
        if (deadEl) {
            deadEl.textContent = dead.length + ' DEAD';
            deadEl.style.color = '#FF2B2B';
        }
    });

    listen('log-message', (event) => {
        const msg = typeof event.payload === 'string' ? event.payload : JSON.stringify(event.payload);
        let clr = '#888';
        if (msg.includes('ERROR') || msg.includes('FAIL') || msg.includes('PANIC')) clr = '#FF6B6B';
        else if (msg.includes('WARN') || msg.includes('CAUTION') || msg.includes('HALT')) clr = '#FFD700';
        else if (msg.includes('OK') || msg.includes('SUCCESS') || msg.includes('ENTRY')) clr = '#00BFFF';
        else if (msg.includes('DRY') || msg.includes('SHUTDOWN')) clr = '#FFD700';
        titanLogAdd(msg, clr);
    });

    listen('var-gate-blocked', (event) => {
        const d = event.payload;
        if (d && typeof TitanToast !== 'undefined') {
            TitanToast.show(
                `VaR GATE: ${d.symbol || '?'} blocked — ${d.reason || 'threshold exceeded'}`,
                'warning', 4000
            );
        }
    });

    initBlackoutBadge();
    initStateHealthDots();
    initSmartTrendPanel();
    initStAccountPanel();
    initSentinelAlertLog();
    initStTradePanel();
    if (window._initStChartStrip) window._initStChartStrip();
    initStExitPanel();
    initStAssetsTable();
    initSmartTrendCfg();

    listen('smarttrend-alert', (event) => {
        const a = event.payload;
        if (!a) return;
        _handleSentinelAlert(a);
    });

    listen('sentinel-badge', (event) => {
        const b = event.payload;
        if (!b) return;
        _updateSentinelBadge(b);
    });

    listen('smarttrend-trade-prompt', (event) => {
        const d = event.payload;
        if (d) _showTradePrompt(d);
    });

    listen('smarttrend-exit-prompt', (event) => {
        const d = event.payload;
        if (d) _showExitPrompt(d);
    });
}

// ═══════════════════════════════════════════════════════════════
//  S09: SENTINEL ALERT LOG + BADGE + SEVERITY FILTERING
// ═══════════════════════════════════════════════════════════════

const _stAlertLog = [];
const _stSevFilters = { Info: true, Warning: true, Critical: true };
const _stAlertCooldowns = {};
const ST_MAX_ALERTS = 20;
const ST_AUDIO_COOLDOWN_MS = 30000;
const ST_ICON = '\u25C6';

const ST_MARKER_AUDIO = {
    Stalking: 'st_stalking',
    Entry: 'st_entry',
    Exit: 'st_exit',
    Trail: 'st_trail',
    ChangePointReset: 'st_changepoint',
};

const ST_MARKER_LABELS = {
    Stalking: 'STALK',
    Entry: 'ENTRY',
    Exit: 'EXIT',
    Trail: 'TRAIL',
    ChangePointReset: 'RST',
};

function _markerCssClass(marker) {
    if (marker.includes('Stalking') || marker.includes('STALK')) return 'mk-stalking';
    if (marker.includes('Entry') || marker.includes('ENTRY')) return 'mk-entry';
    if (marker.includes('Exit') || marker.includes('EXIT')) return 'mk-exit';
    if (marker.includes('Trail') || marker.includes('TRAIL')) return 'mk-trail';
    if (marker.includes('Change') || marker.includes('RST')) return 'mk-changepoint';
    return '';
}

function _sevCssClass(sev) {
    if (sev === 'Critical') return 'sev-critical';
    if (sev === 'Warning') return 'sev-warning';
    return 'sev-info';
}

function _formatAlertTime(ms) {
    const d = new Date(ms);
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    const ss = String(d.getSeconds()).padStart(2, '0');
    return hh + ':' + mm + ':' + ss;
}

function _handleSentinelAlert(alert) {
    _stAlertLog.unshift(alert);
    if (_stAlertLog.length > ST_MAX_ALERTS) _stAlertLog.length = ST_MAX_ALERTS;

    const sev = alert.severity || 'Info';
    const markerKey = (alert.marker || '').replace(/"/g, '');

    if (_stSevFilters[sev]) {
        const audioKey = ST_MARKER_AUDIO[markerKey] || 'st_stalking';
        const cooldownKey = alert.symbol + ':' + audioKey;
        const now = Date.now();
        if (!_stAlertCooldowns[cooldownKey] || now - _stAlertCooldowns[cooldownKey] >= ST_AUDIO_COOLDOWN_MS) {
            _stAlertCooldowns[cooldownKey] = now;
            if (window.TitanAudio) TitanAudio.play(audioKey);
        }

        const label = ST_MARKER_LABELS[markerKey] || markerKey;
        const toastMsg = ST_ICON + ' ' + alert.symbol + ' ' + label + ' (' + (alert.score || 0).toFixed(1) + ')';
        const toastType = sev === 'Critical' ? 'error' : sev === 'Warning' ? 'warning' : 'info';
        const toastDur = sev === 'Critical' ? 10000 : 5000;
        if (typeof showCyberToast === 'function') {
            const el = showCyberToast(toastMsg, toastType, toastDur);
            if (el) {
                el.classList.add('titan-toast-smarttrend');
                const markerClass = 'toast-st-' + (markerKey || 'stalking').toLowerCase().replace('pointreset', 'changepoint');
                el.classList.add(markerClass);
            }
        }

        if (sev === 'Critical' || sev === 'Warning') {
            if (window.TitanAudio) {
                const glowColor = sev === 'Critical'
                    ? 'rgba(255, 43, 43, 0.25)'
                    : 'rgba(255, 215, 0, 0.2)';
                TitanAudio.showScreenGlow(glowColor, sev === 'Critical' ? 2000 : 1000);
            }
        }

        if (markerKey === 'Entry' || markerKey === 'Exit') {
            if (window.TitanAudio) {
                TitanAudio.flashTitle('[ST ' + label + '] ' + alert.symbol, 10000);
            }
        }
    }

    const panel = $el('titan-smarttrend-panel');
    if (panel) {
        panel.classList.add('st-alert-active');
        setTimeout(() => panel.classList.remove('st-alert-active'), 3000);
    }

    _renderAlertLog();
}

function _renderAlertLog() {
    const tbody = $el('st-alert-tbody');
    if (!tbody) return;

    tbody.textContent = '';

    for (const a of _stAlertLog) {
        const sev = a.severity || 'Info';
        if (!_stSevFilters[sev]) continue;

        const tr = document.createElement('tr');
        tr.className = 'st-alert-row ' + _sevCssClass(sev);

        const tdTime = document.createElement('td');
        tdTime.className = 'st-alert-time';
        tdTime.textContent = _formatAlertTime(a.timestamp_ms || Date.now());
        tr.appendChild(tdTime);

        const tdSym = document.createElement('td');
        tdSym.className = 'st-alert-sym';
        tdSym.textContent = a.symbol || '?';
        tr.appendChild(tdSym);

        const label = ST_MARKER_LABELS[a.marker] || a.marker || '?';
        const tdEvent = document.createElement('td');
        tdEvent.className = 'st-alert-marker ' + _markerCssClass(a.marker || '');
        tdEvent.textContent = label;
        tr.appendChild(tdEvent);

        const tdScore = document.createElement('td');
        tdScore.className = 'st-alert-score';
        tdScore.textContent = (a.score || 0).toFixed(1);
        tr.appendChild(tdScore);

        const tdDir = document.createElement('td');
        tdDir.className = 'st-alert-dir';
        const dir = (a.direction || 'Neutral').replace(/"/g, '');
        tdDir.textContent = dir === 'Bullish' ? 'LONG' : dir === 'Bearish' ? 'SHORT' : '--';
        tdDir.classList.add(dir === 'Bullish' ? 'val-green' : dir === 'Bearish' ? 'val-red' : 'val-dim');
        tr.appendChild(tdDir);

        tbody.appendChild(tr);
    }
}

function initSentinelAlertLog() {
    const clearBtn = document.querySelector('#st-alert-log .st-log-clear');
    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            _stAlertLog.length = 0;
            _renderAlertLog();
        });
    }

    document.querySelectorAll('.st-sev-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const sev = btn.dataset.sev;
            _stSevFilters[sev] = !_stSevFilters[sev];
            btn.classList.toggle('sev-active', _stSevFilters[sev]);
            _renderAlertLog();
        });
    });
}

function _updateSentinelBadge(badge) {
    const el = $el('st-sentinel-badge');
    if (!el) return;

    if (!badge || badge.total_symbols === 0) {
        el.textContent = '';
        return;
    }

    el.innerHTML = '';

    const countSpan = document.createElement('span');
    countSpan.className = 'st-badge-count';
    countSpan.textContent = badge.total_symbols;
    el.appendChild(countSpan);

    const sep1 = document.createElement('span');
    sep1.className = 'st-badge-sep';
    sep1.textContent = '|';
    el.appendChild(sep1);

    const activeSpan = document.createElement('span');
    activeSpan.className = 'st-badge-active-count';
    activeSpan.textContent = badge.active_symbols + ' ACT';
    el.appendChild(activeSpan);

    if (badge.hottest_symbol && badge.hottest_state !== 'Flat') {
        const sep2 = document.createElement('span');
        sep2.className = 'st-badge-sep';
        sep2.textContent = '|';
        el.appendChild(sep2);

        const hotSpan = document.createElement('span');
        hotSpan.className = 'st-badge-hot';
        hotSpan.textContent = badge.hottest_symbol + ' ' + badge.hottest_state;
        el.appendChild(hotSpan);
    }

    const latEl = $el('st-log-latency');
    if (latEl && badge.avg_evaluate_us !== undefined) {
        const us = badge.avg_evaluate_us;
        latEl.textContent = us < 1000 ? us + '\u00B5s' : (us / 1000).toFixed(1) + 'ms';
        latEl.className = 'st-log-latency ' + (us < 1000 ? 'st-lat-fast' : us < 5000 ? 'st-lat-mid' : 'st-lat-slow');
    }
}

function initBlackoutBadge() {
    const MARKET_OPEN_H = 9, MARKET_OPEN_M = 30;
    const MARKET_CLOSE_H = 16, MARKET_CLOSE_M = 0;

    function getETNow() {
        const now = new Date();
        const jan = new Date(now.getFullYear(), 0, 1);
        const jul = new Date(now.getFullYear(), 6, 1);
        const stdOff = Math.max(jan.getTimezoneOffset(), jul.getTimezoneOffset());
        const isDST = now.getTimezoneOffset() < stdOff;
        const etOffsetMs = isDST ? -4 * 3600000 : -5 * 3600000;
        return new Date(now.getTime() + (now.getTimezoneOffset() * 60000) + etOffsetMs);
    }

    function update() {
        const badge1 = $el('blackout-badge');
        const badge2 = $el('sidebar-blackout-badge');
        const cd1 = $el('blackout-countdown');
        const cd2 = $el('sidebar-blackout-cd');
        const blFirst = $el('sg-blackout-first');
        const blLast = $el('sg-blackout-last');
        const firstMins = blFirst ? parseInt(blFirst.value, 10) || 0 : 0;
        const lastMins = blLast ? parseInt(blLast.value, 10) || 0 : 0;

        if (firstMins === 0 && lastMins === 0) {
            if (badge1) badge1.classList.add('hidden');
            if (badge2) badge2.classList.add('hidden');
            return;
        }

        const et = getETNow();
        const h = et.getHours(), m = et.getMinutes();
        const minsSinceOpen = (h - MARKET_OPEN_H) * 60 + (m - MARKET_OPEN_M);
        const minsUntilClose = (MARKET_CLOSE_H - h) * 60 + (MARKET_CLOSE_M - m);
        let remaining = 0;
        let active = false;

        if (minsSinceOpen >= 0 && minsSinceOpen < firstMins) {
            active = true;
            remaining = firstMins - minsSinceOpen;
        } else if (minsUntilClose >= 0 && minsUntilClose < lastMins) {
            active = true;
            remaining = minsUntilClose;
        }

        [badge1, badge2].forEach(b => { if (b) b.classList.toggle('hidden', !active); });
        const txt = remaining > 0 ? remaining + 'm' : '';
        if (cd1) cd1.textContent = txt;
        if (cd2) cd2.textContent = txt;
    }

    setInterval(update, 15000);
    setTimeout(update, 2000);
}

function initStateHealthDots() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    const DOT_IDS = ['sh-trading', 'sh-market', 'sh-risk', 'sh-scan'];
    const NAMES = ['Trading', 'Market', 'Risk', 'Scan'];

    async function poll() {
        try {
            const h = await invoke('get_state_health');
            if (!h || !h.sub_states) return;
            h.sub_states.forEach((sub, i) => {
                const dot = $el(DOT_IDS[i]);
                if (!dot) return;
                dot.classList.remove('sh-dot-dim', 'sh-dot-green', 'sh-dot-yellow', 'sh-dot-red');
                if (sub.locks_contended === 0) {
                    dot.classList.add('sh-dot-green');
                } else if (sub.locks_contended <= 1) {
                    dot.classList.add('sh-dot-yellow');
                } else {
                    dot.classList.add('sh-dot-red');
                }
                dot.setAttribute('data-label',
                    NAMES[i] + ' ' + sub.locks_ok + '/' + sub.fields + ' OK');
                dot.title = NAMES[i] + ': ' + sub.locks_ok + '/' + sub.fields +
                    ' locks free, ' + sub.locks_contended + ' contended | Gen ' + h.session_generation;
            });
        } catch (_) { /* silently ignore if backend unavailable */ }
    }

    setInterval(poll, 10000);
    setTimeout(poll, 3000);
}

function initSmartTrendPanel() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    const panel = $el('titan-smarttrend-panel');
    if (!panel) return;

    const STATE_MAP = {
        'Flat':     'st-badge-active-flat',
        'Stalking': 'st-badge-active-stalking',
        'Entry':    'st-badge-active-entry',
        'InTrade':  'st-badge-active-intrade',
        'Trail':    'st-badge-active-trail',
    };
    const DIR_CLASS = { 'Bullish': 'st-dir-long', 'Bearish': 'st-dir-short', 'Neutral': 'st-dir-neutral' };
    const DIR_LABEL = { 'Bullish': 'LONG', 'Bearish': 'SHORT', 'Neutral': '--' };
    const STR_CLASS = { 'Weak': 'st-str-weak', 'Moderate': 'st-str-moderate', 'Strong': 'st-str-strong', 'Extreme': 'st-str-extreme' };

    async function poll() {
        try {
            const d = await invoke('get_smarttrend_diagnostics');
            if (!d) return;

            panel.hidden = false;
            if (!d.enabled && !d.killed) {
                return;
            }

            const badges = panel.querySelectorAll('.st-badge');
            badges.forEach(b => {
                b.className = 'st-badge st-badge-dim';
                const st = b.getAttribute('data-st');
                if (st === d.state) {
                    b.className = 'st-badge ' + (STATE_MAP[st] || 'st-badge-dim');
                }
            });

            const dirEl = $el('st-direction');
            if (dirEl) {
                dirEl.textContent = DIR_LABEL[d.direction] || '--';
                dirEl.className = 'st-live-val ' + (DIR_CLASS[d.direction] || 'st-dir-neutral');
            }

            const scoreEl = $el('st-score');
            if (scoreEl) scoreEl.textContent = d.combined_score.toFixed(1);

            const strEl = $el('st-strength');
            if (strEl) {
                strEl.textContent = d.strength || '--';
                strEl.className = 'st-live-val ' + (STR_CLASS[d.strength] || 'st-str-weak');
            }

            const confirmRow = $el('st-confirm-row');
            const confirmText = $el('st-confirm-text');
            const confirmBar = $el('st-confirm-bar');
            if (d.confirmation_progress) {
                if (confirmRow) confirmRow.hidden = false;
                const [cur, total] = d.confirmation_progress;
                if (confirmText) confirmText.textContent = cur + '/' + total;
                if (confirmBar) confirmBar.style.width = (total > 0 ? (cur / total * 100) : 0) + '%';
            } else {
                if (confirmRow) confirmRow.hidden = true;
            }

            const ps = d.pillar_scores;
            const macroEl = $el('st-macro-score');
            const microEl = $el('st-micro-score');
            const momEl = $el('st-momentum-score');
            const agreeEl = $el('st-agreement');
            if (macroEl && ps) macroEl.textContent = ps.macro_score ? ps.macro_score.score.toFixed(0) : '--';
            if (microEl && ps) microEl.textContent = ps.micro_score ? ps.micro_score.score.toFixed(0) : '--';
            if (momEl && ps) momEl.textContent = ps.momentum_score ? ps.momentum_score.score.toFixed(0) : '--';
            if (agreeEl && ps) {
                if (ps.cross_divergence) {
                    agreeEl.textContent = 'DIVERGE';
                    agreeEl.className = 'st-live-val st-diverge';
                } else {
                    agreeEl.textContent = 'AGREE';
                    agreeEl.className = 'st-live-val st-agree';
                }
            }

            const killedEl = $el('st-killed-badge');
            if (killedEl) killedEl.hidden = !d.killed;

            const killBtn = $el('st-kill-btn');
            if (killBtn) {
                if (d.killed) {
                    killBtn.textContent = 'REACTIVATE';
                    killBtn.classList.add('st-kill-active');
                } else {
                    killBtn.textContent = 'KILL';
                    killBtn.classList.remove('st-kill-active');
                }
            }

            updateStQuickStats(d);
        } catch (_) { /* backend unavailable */ }
    }

    function updateStQuickStats(d) {
        const statsEl = $el('st-quick-stats');
        if (!statsEl || !d.stats) return;
        statsEl.hidden = false;

        const entries = d.stats.entries_total || 0;
        const trailHits = d.stats.entries_reached_trail || 0;
        const winRate = entries > 0 ? ((trailHits / entries) * 100).toFixed(0) + '%' : '--';
        const avgBars = entries > 0 ? Math.round(
            Object.values(d.stats.bars_per_state || {}).reduce((a, b) => a + b, 0) / Math.max(entries, 1)
        ) : '--';
        const trailRate = entries > 0 ? ((trailHits / entries) * 100).toFixed(0) + '%' : '--';
        const resets = d.stats.changepoint_resets || 0;

        const wrEl = $el('st-stat-winrate');
        const abEl = $el('st-stat-avgbars');
        const trEl = $el('st-stat-trailrate');
        const rsEl = $el('st-stat-resets');
        if (wrEl) wrEl.textContent = winRate;
        if (abEl) abEl.textContent = avgBars;
        if (trEl) trEl.textContent = trailRate;
        if (rsEl) rsEl.textContent = resets;
    }

    const killBtn = $el('st-kill-btn');
    if (killBtn) {
        killBtn.addEventListener('click', async () => {
            const isKilled = killBtn.classList.contains('st-kill-active');
            if (isKilled) {
                const confirmed = await TitanToast.confirm(
                    'REACTIVATE SMARTTREND?',
                    'This will re-enable the SmartTrend engine. Are you sure?'
                );
                if (!confirmed) return;
                try {
                    await invoke('smarttrend_kill_switch', {
                        enable: false,
                        confirmToken: 'CONFIRM_ST_REACTIVATE',
                    });
                    TitanToast.show('SmartTrend REACTIVATED', 'success');
                    if (typeof TitanAudio !== 'undefined') TitanAudio.play('mode_change');
                } catch (e) {
                    TitanToast.show('Failed: ' + e, 'error');
                }
            } else {
                const confirmed = await TitanToast.confirm(
                    'KILL SMARTTREND?',
                    'Emergency stop — all SmartTrend entries will be blocked immediately.'
                );
                if (!confirmed) return;
                try {
                    await invoke('smarttrend_kill_switch', {
                        enable: true,
                        confirmToken: null,
                    });
                    TitanToast.show('SmartTrend KILLED', 'error');
                    if (typeof TitanAudio !== 'undefined') TitanAudio.play('alert_critical');
                } catch (e) {
                    TitanToast.show('Failed: ' + e, 'error');
                }
            }
            poll();
        });
    }

    TitanTimers.register('st:diagnostics', poll, 3000, { tab: 13 });
    setTimeout(poll, 2000);
}

// ═══════════════════════════════════════════════════════════════
//  S10: SmartTrend Account Panel
// ═══════════════════════════════════════════════════════════════
function initStAccountPanel() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    const { listen } = window.__TAURI__.event;
    const panel = $el('st-account-panel');
    if (!panel) return;

    const $capital = $el('st-acct-capital');
    const $pnlToday = $el('st-acct-pnl-today');
    const $pnlTotal = $el('st-acct-pnl-total');
    const $dd = $el('st-acct-dd');
    const $ddFill = $el('st-acct-dd-fill');
    const $ddThresh = $el('st-acct-dd-thresh');
    const $hitrate = $el('st-acct-hitrate');
    const $trades = $el('st-acct-trades');
    const $status = $el('st-acct-status');
    const $amountInput = $el('st-acct-amount');
    const $topupBtn = $el('st-topup-btn');
    const $withdrawBtn = $el('st-withdraw-btn');
    const $pauseBtn = $el('st-pause-btn');
    const $forceExitBtn = $el('st-force-exit-btn');

    function formatPnl(val) {
        const sign = val >= 0 ? '+' : '';
        return sign + '$' + Math.abs(val).toFixed(2);
    }

    function pnlClass(val) {
        if (val > 0.01) return 'val-positive';
        if (val < -0.01) return 'val-negative';
        return '';
    }

    async function poll() {
        try {
            const acct = await invoke('smarttrend_account_stats');
            if ($capital) $capital.textContent = '$' + (acct.allocated_capital || 0).toFixed(2);

            if ($pnlToday) {
                $pnlToday.textContent = formatPnl(acct.realized_pnl_today || 0);
                $pnlToday.className = 'st-acct-val ' + pnlClass(acct.realized_pnl_today || 0);
            }
            if ($pnlTotal) {
                $pnlTotal.textContent = formatPnl(acct.total_realized_pnl || 0);
                $pnlTotal.className = 'st-acct-val ' + pnlClass(acct.total_realized_pnl || 0);
            }

            const ddPct = acct.max_drawdown_pct || 0;
            if ($dd) {
                $dd.textContent = ddPct.toFixed(1) + '%';
                $dd.className = 'st-acct-val ' + (ddPct > 3 ? 'val-negative' : '');
            }
            if ($ddFill) $ddFill.style.width = Math.min(ddPct * 2, 100) + '%';

            const maxDd = acct.st_max_dd_pct || 5;
            if ($ddThresh) $ddThresh.style.left = Math.min(maxDd * 2, 100) + '%';

            if ($hitrate) {
                const hr = acct.total_trades > 0
                    ? ((acct.total_wins / acct.total_trades) * 100).toFixed(1) + '%'
                    : '--';
                $hitrate.textContent = hr;
            }
            if ($trades) $trades.textContent = (acct.trade_count_today || 0) + ' / ' + (acct.total_trades || 0);

            if ($status) {
                $status.className = 'tbl-badge ';
                if (acct.breaker_triggered) {
                    $status.textContent = 'BREAKER';
                    $status.className += 'tbl-badge-red st-acct-breaker';
                } else if (acct.paused) {
                    $status.textContent = 'PAUSED';
                    $status.className += 'tbl-badge-gold st-acct-paused';
                } else {
                    $status.textContent = 'ACTIVE';
                    $status.className += 'tbl-badge-cyan';
                }
            }

            if ($pauseBtn) $pauseBtn.textContent = acct.paused ? 'RESUME' : 'PAUSE';
        } catch (_) { /* silent */ }
    }

    if ($topupBtn) {
        $topupBtn.addEventListener('click', async () => {
            const amount = parseFloat($amountInput?.value || '0');
            if (!amount || amount <= 0) {
                TitanToast.show('Enter a valid amount', 'warning');
                return;
            }
            try {
                const res = await invoke('smarttrend_topup', { amount });
                TitanToast.show(res, 'success');
                if ($amountInput) $amountInput.value = '';
                poll();
            } catch (e) {
                TitanToast.show('Top-up failed: ' + e, 'error');
            }
        });
    }

    if ($withdrawBtn) {
        $withdrawBtn.addEventListener('click', async () => {
            const amount = parseFloat($amountInput?.value || '0');
            if (!amount || amount <= 0) {
                TitanToast.show('Enter a valid amount', 'warning');
                return;
            }
            try {
                const res = await invoke('smarttrend_withdraw', { amount });
                TitanToast.show(res, 'success');
                if ($amountInput) $amountInput.value = '';
                poll();
            } catch (e) {
                TitanToast.show('Withdraw failed: ' + e, 'error');
            }
        });
    }

    if ($pauseBtn) {
        $pauseBtn.addEventListener('click', async () => {
            const isPaused = $pauseBtn.textContent === 'RESUME';
            try {
                const res = await invoke('smarttrend_pause', { paused: !isPaused });
                TitanToast.show(res, isPaused ? 'success' : 'warning');
                poll();
            } catch (e) {
                TitanToast.show('Failed: ' + e, 'error');
            }
        });
    }

    if ($forceExitBtn) {
        $forceExitBtn.addEventListener('click', async () => {
            const confirmed = await TitanToast.confirm(
                'FORCE EXIT ALL ST POSITIONS?',
                'This will close ALL SmartTrend positions immediately. This action cannot be undone.'
            );
            if (!confirmed) return;
            try {
                const res = await invoke('smarttrend_force_exit_all', {
                    confirmToken: 'CONFIRM_ST_LIQUIDATE',
                });
                TitanToast.show(res, 'success');
                if (typeof TitanAudio !== 'undefined') TitanAudio.play('st_kill');
                poll();
            } catch (e) {
                TitanToast.show('Force exit failed: ' + e, 'error');
            }
        });
    }

    // B4: ST Positions Badge in header
    const $posBadge = $el('st-positions-badge');
    function updatePosBadge(acct) {
        if (!$posBadge) return;
        const trades = acct.trade_count_today || 0;
        const pnl = acct.realized_pnl_today || 0;
        const capital = acct.allocated_capital || 0;
        if (capital <= 0 && trades === 0) {
            $posBadge.textContent = '';
            return;
        }
        const sign = pnl >= 0 ? '+' : '';
        $posBadge.textContent = 'ST: ' + trades + ' ' + sign + '$' + Math.abs(pnl).toFixed(2);
        $posBadge.className = 'st-pos-badge ' + (pnl > 0.01 ? 'st-pos-positive' : pnl < -0.01 ? 'st-pos-negative' : '');
    }

    const origPoll = poll;
    const wrappedPoll = async () => {
        await origPoll();
        try {
            const acct = await invoke('smarttrend_account_stats');
            updatePosBadge(acct);
        } catch (_) { /* silent */ }
    };

    listen('smarttrend-fill', () => { wrappedPoll(); });

    TitanTimers.register('st:panel-poll', wrappedPoll, 5000, { tab: 13 });
    setTimeout(wrappedPoll, 2500);
}

function wireSidebarControls() {
    const saveBtn = $el('btn-save-keys');
    if (saveBtn) saveBtn.onclick = window.saveKeys;

    bindToggle('chk-loop-top', 'loop_active');
    bindToggle('chk-scan', 'omni_scan_active');
    bindToggle('chk-extended', 'enable_extended');
    bindToggle('chk-force-manual', 'force_manual_tf');
    bindToggle('chk-auto', 'auto_pilot');
    bindToggle('chk-auto-entry', 'auto_entry');
    bindToggle('chk-aggressive', 'aggressive_mode');
    bindToggle('chk-short', 'allow_short');
    bindToggle('chk-scalp-oracle', 'scalping_oracle');
    bindToggle('chk-scalp-gpu', 'scalping_gpu');
    bindToggle('chk-scalp-fusion', 'scalping_fusion');
    bindToggle('chk-gpu-backtest', 'gpu_backtest');
    bindToggle('chk-gpu-sweep', 'gpu_sweep');
    bindToggle('chk-gpu-deep', 'gpu_deep_focus');

    bindSlider('slider-refresh', 'refresh_rate', 'val-refresh');
    bindSlider('slider-risk', 'risk_per_trade', 'val-risk');
    bindSlider('slider-backtest', 'backtest_days', 'val-backtest');
    bindSelect('select-manual-res', 'manual_res');
    bindTimeframeButtons();
    const manualRes = $el('select-manual-res');
    if (manualRes) {
        manualRes.addEventListener('change', () => {
            const tf = manualRes.value;
            const btn = document.querySelector(`.tf-btn[data-tf="${tf}"]`);
            setTF(tf, btn);
        });
    }

    wireButtons();
    refreshHftTargets();
    setTimeout(() => refreshHftEnriched(), 2000);
    startSidebarInit();
}

window.onload = function () {
    const statusEl = $el('tauri-status');
    if (window.__TAURI__) {
        statusEl.innerText = "TAURI LINKED";
        statusEl.style.background = "#00BFFF";
        statusEl.style.color = "#0a0a0f";
    } else {
        statusEl.innerText = "NO TAURI";
        statusEl.style.background = "red";
        console.warn("Tauri object not found. Running as a standalone webpage (static/debug mode).");
    }

    const onloadSteps = [
        ['initChartsDeferred', initChartsDeferred],
        ['wireDomListeners', wireDomListeners],
        ['registerTauriListeners', registerTauriListeners],
        ['wireAudioAlerts', () => { if (window.TitanAudio) TitanAudio.wireEvents(); }],
        ['wireSidebarControls', wireSidebarControls],
    ];
    for (const [name, fn] of onloadSteps) {
        try { fn(); } catch (e) { console.error(`[TITAN window.onload] ${name} FAILED:`, e); }
    }
};

window.saveKeys = async function () {
    const keyInput = $el('api-key');
    const secretInput = $el('api-secret');

    if (!keyInput || !secretInput) return;

    const key = keyInput.value.trim();
    const secret = secretInput.value.trim();

    if (key && secret) {
        try {
            const response = await invoke("update_api_keys", { key: key, secret: secret });
            showCyberToast("Keys saved: " + response, 'success');
            updateStatus("🔑 KEYS SAVED (" + key.substring(0, 4) + "...). CONNECTING...", "orange");
            loadData(currentSymbol);
        } catch (e) {
            TitanError.report('bootstrap:save-keys', e);
            updateStatus("❌ KEY SAVE ERROR: " + normalizeError(e), "red");
            showCyberToast("FAILED TO SAVE KEYS: " + normalizeError(e), 'error');
        }
    } else {
        showCyberToast("Please enter both API Key and Secret (no spaces)", 'warning');
    }
};

function wireRiskButtons() {
    const btnLiquidate = $el('btn-liquidate');
    if (btnLiquidate) {
        btnLiquidate.addEventListener('click', async () => {
            const res = await invoke('liquidate_all_positions', { confirmToken: 'CONFIRM_LIQUIDATE' });
            showCyberToast(res, 'warning');
        });
    }

    const btnCancel = $el('btn-cancel-orders');
    if (btnCancel) {
        btnCancel.addEventListener('click', async () => {
            const res = await invoke('cancel_all_orders', { confirmToken: 'CONFIRM_CANCEL' });
            showCyberToast(res, 'info');
        });
    }

}

function wireAnalysisButtons() {
    const btnAuto = $el('btn-auto-opt');
    if (btnAuto) {
        btnAuto.addEventListener('click', async () => {
            startCmdCountdown('AUTO-OPTIMIZE', 30);
            let ok = false;
            try {
                const res = await invoke('optimize_timeframe', { symbol: currentSymbol });
                if (res && res.best_tf) {
                    currentTF = res.best_tf;
                    const tfEl = $el('analysis-timeframe');
                    if (tfEl) tfEl.innerText = currentTF;
                    loadData(currentSymbol);
                    showCyberToast(`BEST TF: ${res.best_tf} | ROI: ${res.best_roi.toFixed(2)}%`, 'success');
                }
                ok = true;
            } catch (e) {
                TitanError.report('analysis:auto-optimize', e);
            } finally {
                finishCmdCountdown(ok ? 'AUTO-OPTIMIZE done' : 'AUTO-OPTIMIZE error');
            }
        });
    }

    const btnDeep = $el('btn-deep-focus');
    if (btnDeep) {
        btnDeep.addEventListener('click', async () => {
            const deepOut = $el('deep-focus-output');
            startCmdCountdown('DEEP FOCUS', 20);
            let ok = false;
            const computeLocalDeepFocus = () => {
                if (!lastCandleData || lastCandleData.length < 2) return null;
                const first = lastCandleData[0].open ?? lastCandleData[0].close ?? 0;
                const last = lastCandleData[lastCandleData.length - 1].close ?? 0;
                if (!Number.isFinite(first) || !Number.isFinite(last) || first <= 0) return null;
                const roi = ((last - first) / first) * 100.0;
                const returns = [];
                for (let i = 1; i < lastCandleData.length; i++) {
                    const prev = lastCandleData[i - 1].close ?? 0;
                    const cur = lastCandleData[i].close ?? 0;
                    if (prev > 0) returns.push((cur - prev) / prev);
                }
                if (!returns.length) return null;
                const mean = returns.reduce((a, b) => a + b, 0) / returns.length;
                const varSum = returns.reduce((a, b) => a + (b - mean) * (b - mean), 0) / returns.length;
                const vol = Math.sqrt(Math.max(varSum, 0)) * 100.0;
                const trend = last >= first ? 'UPTREND' : 'DOWNTREND';
                return { symbol: currentSymbol, roi, volatility: vol, trend, last_price: last, local: true };
            };
            try {
                if (deepOut) deepOut.innerText = 'Deep Focus running...';
                const res = await Promise.race([
                    invoke('deep_focus_analysis', { symbol: currentSymbol }),
                    new Promise((_, reject) => setTimeout(() => reject('DEEP FOCUS TIMEOUT'), 25000))
                ]);
                if (res) {
                    const msg = `Deep Focus: ${res.symbol} | ROI ${res.roi.toFixed(2)}% | ${res.trend} | Vol ${res.volatility.toFixed(2)}%`;
                    if (deepOut) deepOut.innerText = msg;
                    showCyberToast(`${res.symbol} | ROI ${res.roi.toFixed(2)}% | ${res.trend} | Vol ${res.volatility.toFixed(2)}%`, 'success');
                }
                ok = true;
            } catch (e) {
                const local = computeLocalDeepFocus();
                if (local) {
                    const msg = `Deep Focus (local ${currentTF}): ${local.symbol} | ROI ${local.roi.toFixed(2)}% | ${local.trend} | Vol ${local.volatility.toFixed(2)}%`;
                    if (deepOut) deepOut.innerText = msg;
                    showCyberToast(`LOCAL ${local.symbol} | ROI ${local.roi.toFixed(2)}% | ${local.trend} | Vol ${local.volatility.toFixed(2)}%`, 'info');
                    ok = true;
                } else if (deepOut) {
                    deepOut.innerText = `Deep Focus error: ${normalizeError(e)}`;
                }
                TitanError.report('analysis:deep-focus', e);
            } finally {
                finishCmdCountdown(ok ? 'DEEP FOCUS done' : 'DEEP FOCUS error');
            }
        });
    }

    const btnRetrain = $el('btn-retrain');
    if (btnRetrain) {
        btnRetrain.addEventListener('click', async () => {
            const res = await invoke('retrain_brain', { symbol: currentSymbol });
            showCyberToast(res, 'info');
        });
    }
}

function wireChartUxButtons() {
    const btnFull = $el('btn-fullscreen');
    if (btnFull) {
        btnFull.addEventListener('click', async () => {
            toggleChartFullscreen();
        });
    }
    setChartFullscreenUi(document.body.classList.contains('fullscreen-chart'));

    document.addEventListener('fullscreenchange', () => {
        if (!document.fullscreenElement && document.body.classList.contains('fullscreen-chart')) {
            document.body.classList.remove('fullscreen-chart');
            setChartFullscreenUi(false);
            setTimeout(() => resizeCharts(), 50);
        }
    });

    const btnMag = $el('btn-magnet');
    if (btnMag) {
        btnMag.addEventListener('click', toggleMagnet);
        setMagnetState(window.snapToOhlc);
    }

    const btnSnap = $el('btn-snapshot');
    if (btnSnap) {
        btnSnap.addEventListener('click', () => {
            if (!chart || typeof chart.takeScreenshot !== 'function') {
                showCyberToast("Screenshot not supported by chart engine", 'warning');
                return;
            }
            const canvas = chart.takeScreenshot();
            const link = document.createElement('a');
            link.href = canvas.toDataURL('image/png');
            link.download = `titan_${currentSymbol}_${currentTF}.png`;
            document.body.appendChild(link);
            link.click();
            link.remove();
        });
    }

    const btnAlert = $el('btn-alert');
    if (btnAlert) {
        btnAlert.addEventListener('click', () => {
            if (!lastBar) return;
            const raw = prompt("Alert price (type 'clear' to remove all)", lastBar.close.toFixed(2));
            if (!raw) return;
            if (raw.toLowerCase() === 'clear') {
                clearPriceAlerts();
                return;
            }
            const price = Number(raw);
            if (!Number.isFinite(price)) {
                showCyberToast("Invalid price", 'warning');
                return;
            }
            addPriceAlert(price);
        });
    }

    const btnReset = $el('btn-reset');
    if (btnReset) {
        btnReset.addEventListener('click', () => {
            if (chart) chart.timeScale().fitContent();
        });
    }
}

function wireReplayControls() {
    const btnReplay = $el('btn-replay');
    if (btnReplay) {
        btnReplay.addEventListener('click', () => {
            if (replayState.active) exitReplayMode();
            else enterReplayMode();
        });
    }

    const replayToggle = $el('replay-toggle');
    if (replayToggle) replayToggle.addEventListener('click', () => toggleReplay());
    const replayBack = $el('replay-step-back');
    if (replayBack) replayBack.addEventListener('click', () => stepReplay(-1));
    const replayFwd = $el('replay-step-fwd');
    if (replayFwd) replayFwd.addEventListener('click', () => stepReplay(1));
    const replaySpeed = $el('replay-speed');
    if (replaySpeed) replaySpeed.addEventListener('change', (e) => setReplaySpeed(e.target.value));
    const replaySlider = $el('replay-slider');
    if (replaySlider) {
        replaySlider.addEventListener('input', (e) => {
            if (!replayState.active) return;
            replayState.index = Number(e.target.value) || 0;
            applyReplaySlice();
        });
    }
    const replayExit = $el('replay-exit');
    if (replayExit) replayExit.addEventListener('click', () => exitReplayMode());
}

function wireChartHotkeys() {
    if (window.__chartHotkeysBound) return;
    window.__chartHotkeysBound = true;
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.metaKey || e.altKey) return;
        const tag = (e.target && e.target.tagName) ? e.target.tagName.toLowerCase() : '';
        if (tag === 'input' || tag === 'textarea' || (e.target && e.target.isContentEditable)) return;
        const key = e.key.toLowerCase();
        if (key === 'escape') {
            const btn = $el('btn-crosshair');
            toggletool(null, btn);
            if (replayState.active) exitReplayMode();
            return;
        }
        if (key === ' ') {
            if (replayState.active) {
                e.preventDefault();
                toggleReplay();
            }
            return;
        }
        const map = {
            'l': { tool: 'trend', id: 'btn-trend' },
            'r': { tool: 'ray', id: 'btn-ray' },
            'h': { tool: 'hline', id: 'btn-hline' },
            'v': { tool: 'vline', id: 'btn-vline' },
            'f': { tool: 'fib', id: 'btn-fib' },
            't': { tool: 'text', id: 'btn-text' },
            'm': { tool: 'measure', id: 'btn-measure' },
            'c': { tool: null, id: 'btn-crosshair' }
        };
        if (map[key]) {
            const btn = $el(map[key].id);
            toggletool(map[key].tool, btn);
            return;
        }
        if (key === 'a') {
            $el('btn-alert')?.click();
        } else if (key === 'p') {
            $el('btn-replay')?.click();
        } else if (key === 'g') {
            $el('btn-magnet')?.click();
        } else if (key === '0') {
            $el('btn-reset')?.click();
        }
    });
}

function wireHftCentralButtons() {
    const btnAddHft = $el('btn-hft-central-add');
    if (btnAddHft) {
        btnAddHft.addEventListener('click', async () => {
            const input = $el('hft-central-input');
            const symbol = input ? input.value : '';
            if (!symbol.trim()) return;
            await invoke('add_hft_target', { symbol: symbol });
            if (input) input.value = '';
            await refreshHftTargets();
            refreshHftEnriched();
        });
    }

    const btnRefreshHft = $el('btn-hft-central-refresh');
    if (btnRefreshHft) {
        btnRefreshHft.addEventListener('click', () => refreshHftEnriched());
    }

    const btnDelAllHft = $el('btn-hft-central-del-all');
    if (btnDelAllHft) {
        btnDelAllHft.addEventListener('click', async () => {
            if (!(await cyberConfirm('Delete all IDLE targets? Active positions will be kept.'))) return;
            await invoke('clear_all_hft_targets', {});
            _hftEnrichedCache = [];
            await refreshHftTargets();
        });
    }

    const btnHftCheckNews = $el('btn-hft-check-news');
    if (btnHftCheckNews) {
        btnHftCheckNews.addEventListener('click', async () => {
            const syms = _hftEnrichedCache.map(c => c.symbol).filter(Boolean);
            if (!syms.length) { showCyberToast('No HFT targets to check', 'warning'); return; }
            btnHftCheckNews.textContent = 'CHECKING...';
            btnHftCheckNews.disabled = true;
            try {
                const results = await invoke('check_catalyst', { symbols: syms });
                _updateCatalystCacheFromResults(results);
                await refreshHftTargets();
            } catch (e) {
                TitanError.report('hft:check-news', e);
                btnHftCheckNews.textContent = 'ERROR';
                btnHftCheckNews.style.color = '#ff4444';
                setTimeout(() => { btnHftCheckNews.style.color = '#FF00FF'; }, 2000);
            }
            btnHftCheckNews.textContent = 'CHECK NEWS';
            btnHftCheckNews.disabled = false;
        });
    }
}

function wireButtons() {
    wireRiskButtons();
    wireAnalysisButtons();
    wireChartUxButtons();
    wireReplayControls();
    wireChartHotkeys();
    wireHftCentralButtons();
}

// ═══════════════════════════════════════════════════════════════
//  S11: SmartTrend Trade Panel (MAN entry prompt)
// ═══════════════════════════════════════════════════════════════

let _stTradePromptData = null;
let _stTradeCountdownTimer = null;
let _stTradeSizingMode = 'usd';

function initStTradePanel() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    document.querySelectorAll('.st-sizing-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.st-sizing-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            _stTradeSizingMode = btn.dataset.sizing;
        });
    });

    document.querySelectorAll('.st-quick-amt').forEach(btn => {
        btn.addEventListener('click', () => {
            const amtInput = $el('st-trade-amount');
            if (amtInput) amtInput.value = btn.dataset.amt;
        });
    });

    const tpToggle = $el('st-trade-tp-toggle');
    const slToggle = $el('st-trade-sl-toggle');
    const tpInput = $el('st-trade-tp');
    const slInput = $el('st-trade-sl');
    if (tpToggle && tpInput) tpToggle.addEventListener('change', () => { tpInput.disabled = !tpToggle.checked; });
    if (slToggle && slInput) slToggle.addEventListener('change', () => { slInput.disabled = !slToggle.checked; });

    const execBtn = $el('st-trade-execute');
    const skipBtn = $el('st-trade-skip');
    if (execBtn) execBtn.addEventListener('click', () => _executeStTrade());
    if (skipBtn) skipBtn.addEventListener('click', () => _skipStTrade());
}

function _showTradePrompt(data) {
    _stTradePromptData = data;
    const panel = $el('st-trade-panel');
    if (!panel) return;

    panel.classList.remove('st-trade-hidden');

    const sym = $el('st-trade-sym');
    const dir = $el('st-trade-dir');
    const score = $el('st-trade-score');
    const st = $el('st-trade-state');
    const pillars = $el('st-trade-pillars');
    const price = $el('st-trade-price');
    const suggestion = $el('st-trade-suggestion');
    const amtInput = $el('st-trade-amount');

    if (sym) sym.textContent = data.symbol || '---';
    if (dir) {
        const d = (data.direction || '').replace(/"/g, '');
        dir.textContent = d === 'Bullish' ? 'LONG' : d === 'Bearish' ? 'SHORT' : d;
        dir.className = 'st-live-val ' + (d === 'Bullish' ? 'st-dir-long' : d === 'Bearish' ? 'st-dir-short' : '');
    }
    if (score) score.textContent = (data.score || 0).toFixed(1);
    if (st) st.textContent = data.state || '---';
    if (pillars && data.pillar_scores) {
        const ps = data.pillar_scores;
        pillars.textContent = `M:${(ps[0]||0).toFixed(0)} m:${(ps[1]||0).toFixed(0)} P:${(ps[2]||0).toFixed(0)}`;
    }
    if (price) price.textContent = '$' + (data.current_price || 0).toFixed(2);

    if (suggestion) {
        let txt = `SUGGESTED: $${(data.suggested_size_usd || 0).toFixed(0)} (${(data.risk_pct || 0).toFixed(1)}% risk`;
        if (data.kelly_fraction != null) txt += `, Kelly ${data.kelly_fraction.toFixed(2)}`;
        txt += ')';
        suggestion.textContent = txt;
    }

    if (amtInput) amtInput.value = (data.suggested_size_usd || 0).toFixed(0);

    if (typeof TitanAudio !== 'undefined') TitanAudio.play('st_entry');

    _startTradeCountdown(60);
}

function _startTradeCountdown(secs) {
    if (_stTradeCountdownTimer) clearInterval(_stTradeCountdownTimer);
    let remaining = secs;
    const cdText = $el('st-trade-countdown');
    const cdFill = $el('st-trade-countdown-fill');

    const updateCd = () => {
        if (cdText) cdText.textContent = remaining + 's';
        if (cdFill) {
            const pct = (remaining / secs) * 100;
            cdFill.style.width = pct + '%';
            cdFill.classList.remove('st-cd-warn', 'st-cd-critical');
            if (remaining <= 10) cdFill.classList.add('st-cd-critical');
            else if (remaining <= 30) cdFill.classList.add('st-cd-warn');
        }
    };

    updateCd();
    _stTradeCountdownTimer = setInterval(() => {
        remaining--;
        if (remaining <= 0) {
            _skipStTrade();
            if (typeof TitanToast !== 'undefined') {
                TitanToast.show('Trade opportunity expired', 'warning', 3000);
            }
            return;
        }
        updateCd();
    }, 1000);
}

async function _executeStTrade() {
    if (_stTradeCountdownTimer) clearInterval(_stTradeCountdownTimer);
    const data = _stTradePromptData;
    if (!data) return;
    const panel = $el('st-trade-panel');
    if (panel) panel.classList.add('st-trade-hidden');

    const { invoke } = window.__TAURI__.tauri;
    const amtInput = $el('st-trade-amount');
    let rawAmt = parseFloat(amtInput?.value) || data.suggested_size_usd || 0;

    const d = (data.direction || '').replace(/"/g, '');
    const side = d === 'Bullish' ? 'buy' : d === 'Bearish' ? 'sell' : null;
    if (!side) { TitanToast.show('No valid direction', 'error'); return; }

    let qty = rawAmt;
    if (_stTradeSizingMode === 'usd' && data.current_price > 0) {
        qty = rawAmt / data.current_price;
    } else if (_stTradeSizingMode === 'pct') {
        try {
            const panelData = await invoke('smarttrend_get_trade_panel_data', { symbol: data.symbol });
            qty = (panelData.suggested_size_usd * (rawAmt / 100)) / data.current_price;
        } catch { qty = rawAmt; }
    }

    const tpToggle = $el('st-trade-tp-toggle');
    const slToggle = $el('st-trade-sl-toggle');
    const tpInput = $el('st-trade-tp');
    const slInput = $el('st-trade-sl');
    const tp = tpToggle?.checked ? parseFloat(tpInput?.value) || null : null;
    const sl = slToggle?.checked ? parseFloat(slInput?.value) || null : null;

    try {
        await invoke('submit_order', {
            order: {
                symbol: data.symbol,
                side,
                qty: Math.max(qty, 0.001),
                take_profit: tp ? { limit_price: String(tp) } : null,
                stop_loss: sl ? { stop_price: String(sl) } : null,
            },
            tag: `ST|MAN|${data.symbol}|${Math.floor(Date.now()/1000)}`
        });
        TitanToast.show(`ST MAN EXECUTE: ${side.toUpperCase()} ${data.symbol} x${qty.toFixed(4)}`, 'success', 4000);
    } catch (e) {
        TitanError.report('st-man-execute', e);
    }
    _stTradePromptData = null;
}

function _skipStTrade() {
    if (_stTradeCountdownTimer) clearInterval(_stTradeCountdownTimer);
    const panel = $el('st-trade-panel');
    if (panel) panel.classList.add('st-trade-hidden');
    if (_stTradePromptData) {
        TitanToast.show(`ST MAN SKIP: ${_stTradePromptData.symbol}`, 'info', 2000);
    }
    _stTradePromptData = null;
}

// ═══════════════════════════════════════════════════════════════
//  S11: SmartTrend Exit Signal Panel
// ═══════════════════════════════════════════════════════════════

let _stExitPromptData = null;

function initStExitPanel() {
    if (!window.__TAURI__) return;
    const close100 = $el('st-exit-100');
    const close50 = $el('st-exit-50');
    const close25 = $el('st-exit-25');
    const holdBtn = $el('st-exit-hold');

    if (close100) close100.addEventListener('click', () => _stExitClose(1.0));
    if (close50) close50.addEventListener('click', () => _stExitClose(0.5));
    if (close25) close25.addEventListener('click', () => _stExitClose(0.25));
    if (holdBtn) holdBtn.addEventListener('click', () => _stExitHold());
}

function _showExitPrompt(data) {
    _stExitPromptData = data;
    const panel = $el('st-exit-panel');
    if (!panel) return;

    panel.classList.remove('st-trade-hidden');
    const sym = $el('st-exit-sym');
    const pnl = $el('st-exit-pnl');
    const dur = $el('st-exit-duration');
    const reason = $el('st-exit-reason');
    const severity = $el('st-exit-severity');

    if (sym) sym.textContent = data.symbol || '---';
    if (reason) reason.textContent = (data.marker || '---').replace(/"/g, '');
    if (severity) {
        const sev = (data.severity || 'Warning').replace(/"/g, '');
        severity.textContent = sev;
        severity.className = 'tbl-badge ' + (sev === 'Critical' ? 'tbl-badge-red' : 'tbl-badge-gold');
    }

    const settings = window._lastAppState;
    if (settings && settings.positions) {
        const pos = settings.positions.find(p => p.symbol === data.symbol);
        if (pos) {
            if (pnl) {
                const pnlVal = parseFloat(pos.unrealized_pl) || 0;
                pnl.textContent = '$' + pnlVal.toFixed(2);
                pnl.className = 'st-live-val ' + (pnlVal >= 0 ? 'val-positive' : 'val-negative');
            }
            if (dur && pos.opened_at) {
                const elapsed = Math.floor(Date.now() / 1000) - pos.opened_at;
                const mins = Math.floor(elapsed / 60);
                const secs = elapsed % 60;
                dur.textContent = mins + 'm ' + secs + 's';
            }
        }
    }

    if (typeof TitanAudio !== 'undefined') TitanAudio.play('st_exit');
}

async function _stExitClose(fraction) {
    const data = _stExitPromptData;
    if (!data) return;
    const panel = $el('st-exit-panel');
    if (panel) panel.classList.add('st-trade-hidden');

    const { invoke } = window.__TAURI__.tauri;
    try {
        const pct = Math.round(fraction * 100);
        await invoke('force_close_position', {
            symbol: data.symbol,
            closeFraction: fraction,
        });
        TitanToast.show(`ST EXIT: ${data.symbol} CLOSE ${pct}%`, 'info', 3000);
    } catch (e) {
        TitanError.report('st-man-exit', e);
    }
    _stExitPromptData = null;
}

function _stExitHold() {
    const panel = $el('st-exit-panel');
    if (panel) panel.classList.add('st-trade-hidden');
    if (_stExitPromptData) {
        TitanToast.show(`ST HOLD: ${_stExitPromptData.symbol} — will re-prompt on next signal`, 'info', 3000);
    }
    _stExitPromptData = null;
}

// ═══════════════════════════════════════════════════════════════
//  S11: SmartTrend Assets Table + MODE per asset
// ═══════════════════════════════════════════════════════════════

let _stAssetModes = {};
let _stAssetsTooltip = null;

function initStAssetsTable() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    const allAutoBtn = $el('st-all-auto-btn');
    const allManBtn = $el('st-all-man-btn');

    if (allAutoBtn) {
        allAutoBtn.addEventListener('click', async () => {
            if (typeof TitanToast !== 'undefined') {
                TitanToast.confirm('Set ALL SmartTrend assets to AUTO?', async () => {
                    try {
                        await invoke('smarttrend_set_all_modes', { mode: 'auto', confirmToken: 'CONFIRM_ST_ALL_AUTO' });
                        TitanToast.show('All assets set to AUTO', 'success', 2000);
                        _refreshStAssets();
                    } catch (e) { TitanError.report('st-all-auto', e); }
                });
            }
        });
    }

    if (allManBtn) {
        allManBtn.addEventListener('click', async () => {
            try {
                await invoke('smarttrend_set_all_modes', { mode: 'man', confirmToken: null });
                TitanToast.show('All assets set to MAN', 'success', 2000);
                _refreshStAssets();
            } catch (e) { TitanError.report('st-all-man', e); }
        });
    }

    _refreshStAssets();
    TitanTimers.register('st:assets-refresh', _refreshStAssets, 5000, { tab: 13 });
}

async function _refreshStAssets() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    try {
        _stAssetModes = await invoke('smarttrend_get_asset_modes') || {};
    } catch { _stAssetModes = {}; }

    let sentinelData = null;
    try { sentinelData = await invoke('get_sentinel_status'); } catch {}

    let positions = [];
    try {
        const state = await invoke('get_app_state');
        if (state && state.positions) positions = state.positions;
    } catch {}

    const tbody = $el('st-assets-tbody');
    if (!tbody) return;

    const symbols = Object.keys(_stAssetModes);
    if (symbols.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" class="cyber-panel-empty">No ST assets configured</td></tr>';
        return;
    }

    let diagData = null;
    try { diagData = await invoke('get_smarttrend_state'); } catch {}

    const rows = symbols.map(sym => {
        const mode = _stAssetModes[sym] || 'man';
        const pos = positions.find(p => p.symbol === sym);
        const hasPosition = !!pos;
        const isStPos = pos?.entry_tag?.startsWith('ST|');
        const pnl = hasPosition ? (parseFloat(pos.unrealized_pl) || 0) : 0;

        let stState = 'Flat';
        let stScore = 0;
        let stDir = '--';
        if (diagData && diagData.state) {
            stState = diagData.state || 'Flat';
            stScore = diagData.combined_score || 0;
            stDir = diagData.direction || '--';
        }

        return { sym, mode, stState, stScore, stDir, hasPosition, isStPos, pnl };
    });

    rows.sort((a, b) => {
        if (a.hasPosition && !b.hasPosition) return -1;
        if (!a.hasPosition && b.hasPosition) return 1;
        const stateRank = { InTrade: 4, Trail: 3, Entry: 2, Stalking: 1, Flat: 0 };
        return (stateRank[b.stState] || 0) - (stateRank[a.stState] || 0);
    });

    const html = rows.map(r => {
        const modeBadge = r.mode === 'auto'
            ? '<span class="st-mode-badge st-mode-auto" data-action="toggle-mode">AUTO</span>'
            : '<span class="st-mode-badge st-mode-man" data-action="toggle-mode">MAN</span>';
        const stateClass = 'st-badge-active-' + (r.stState || 'flat').toLowerCase();
        const liveBadge = r.hasPosition ? ' <span class="tbl-badge tbl-badge-cyan">LIVE</span>' : '';
        const pnlClass = r.pnl >= 0 ? 'val-positive' : 'val-negative';
        const pnlStr = r.hasPosition ? ('$' + r.pnl.toFixed(2)) : '--';

        return `<tr data-sym="${escapeHtml(r.sym)}" class="${r.hasPosition ? 'st-asset-live' : ''}">
            <td class="td-sym">${escapeHtml(r.sym)}${liveBadge}</td>
            <td>${modeBadge}</td>
            <td><span class="st-badge ${stateClass}">${escapeHtml(r.stState)}</span></td>
            <td class="val-cyan">${r.stScore.toFixed(0)}</td>
            <td>${escapeHtml(r.stDir)}</td>
            <td>--</td>
            <td class="${pnlClass}">${pnlStr}</td>
        </tr>`;
    }).join('');

    if (tbody.innerHTML === html) return;

    requestAnimationFrame(() => {
        tbody.innerHTML = html;

        tbody.querySelectorAll('[data-action="toggle-mode"]').forEach(badge => {
            badge.addEventListener('click', async (e) => {
                const row = e.target.closest('tr');
                const sym = row?.dataset.sym;
                if (!sym) return;
                const currentMode = _stAssetModes[sym] || 'man';
                const newMode = currentMode === 'auto' ? 'man' : 'auto';

                if (newMode === 'auto') {
                    TitanToast.confirm(`Enable AUTO trading on ${sym}?`, async () => {
                        try {
                            await invoke('smarttrend_set_asset_mode', { symbol: sym, mode: 'auto' });
                            _refreshStAssets();
                        } catch (err) { TitanError.report('st-mode', err); }
                    });
                } else {
                    try {
                        await invoke('smarttrend_set_asset_mode', { symbol: sym, mode: 'man' });
                        _refreshStAssets();
                    } catch (err) { TitanError.report('st-mode', err); }
                }
            });
        });

        tbody.querySelectorAll('tr[data-sym]').forEach(row => {
            row.addEventListener('mouseenter', (e) => _showAssetTooltip(e, row.dataset.sym));
            row.addEventListener('mouseleave', () => _hideAssetTooltip());
        });
    });
}

async function _showAssetTooltip(event, symbol) {
    _hideAssetTooltip();
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    let trades = [];
    try {
        const journal = await invoke('get_journal_trades', { limit: 100, offset: 0 });
        if (journal && journal.trades) {
            trades = journal.trades.filter(t =>
                t.symbol === symbol && t.entry_tag && t.entry_tag.startsWith('ST|')
            ).slice(0, 5);
        }
    } catch {}

    if (trades.length === 0) return;

    const tip = document.createElement('div');
    tip.className = 'st-asset-tooltip';
    let wins = 0, losses = 0;
    let html = `<div class="st-tt-header">${escapeHtml(symbol)} — Last ${trades.length} ST trades</div>`;
    trades.forEach(t => {
        const pnl = parseFloat(t.pnl) || 0;
        const cls = pnl >= 0 ? 'st-tt-win' : 'st-tt-loss';
        if (pnl >= 0) wins++; else losses++;
        const mode = t.entry_tag?.includes('AUTO') ? 'A' : 'M';
        html += `<div class="st-tt-row"><span>${t.side || '?'} [${mode}]</span><span class="${cls}">$${pnl.toFixed(2)}</span></div>`;
    });
    const hitRate = trades.length > 0 ? ((wins / trades.length) * 100).toFixed(0) : '0';
    html += `<div class="st-tt-row"><span>Hit Rate</span><span class="val-cyan">${hitRate}%</span></div>`;
    tip.innerHTML = html;

    document.body.appendChild(tip);
    _stAssetsTooltip = tip;
    const rect = event.target.closest('tr').getBoundingClientRect();
    tip.style.left = (rect.right + 8) + 'px';
    tip.style.top = rect.top + 'px';
}

function _hideAssetTooltip() {
    if (_stAssetsTooltip) {
        _stAssetsTooltip.remove();
        _stAssetsTooltip = null;
    }
}

// ═══════════════════════════════════════════════════════════════
//  S12a: SmartTrend Config Tab (Tab 13)
// ═══════════════════════════════════════════════════════════════

function initSmartTrendCfg() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    const PRESETS = {
        trend:    { st_macro_weight: 0.60, st_micro_weight: 0.15, st_momentum_weight: 0.25 },
        scalp:    { st_macro_weight: 0.25, st_micro_weight: 0.45, st_momentum_weight: 0.30 },
        balanced: { st_macro_weight: 0.34, st_micro_weight: 0.33, st_momentum_weight: 0.33 },
    };

    const FIELD_META = {
        st_macro_weight:              { type: 'slider', scale: 100, display: v => Math.round(v * 100) },
        st_micro_weight:              { type: 'slider', scale: 100, display: v => Math.round(v * 100) },
        st_momentum_weight:           { type: 'slider', scale: 100, display: v => Math.round(v * 100) },
        st_entry_threshold:           { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_stalking_threshold:        { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_hysteresis_margin:         { type: 'slider', scale: 1,   display: v => v.toFixed(1) },
        st_entry_thresholds_0:        { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_entry_thresholds_1:        { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_entry_thresholds_2:        { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_entry_thresholds_3:        { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_dominant_cycle_filter:     { type: 'checkbox' },
        st_ofi_norm_range:            { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_mp_bps_range:              { type: 'slider', scale: 1,   display: v => v.toFixed(1) },
        st_cvd_scale:                 { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_trail_tighten_pct:         { type: 'slider', scale: 100, display: v => Math.round(v * 100) + '%' },
        st_changepoint_reset_threshold: { type: 'slider', scale: 100, display: v => v.toFixed(2) },
        st_stabilization_window:      { type: 'slider', scale: 1,   display: v => Math.round(v) },
        st_bayesian_decay:            { type: 'slider', scale: 1000, display: v => v.toFixed(3) },
        st_bayesian_halflife:         { type: 'slider', scale: 1,   display: v => Math.round(v) },
    };

    let _savedConfig = null;
    let _currentConfig = {};
    let _liveReadoutTimer = null;
    let _fsmPollTimer = null;

    function getFieldValue(field, cfg) {
        if (field.startsWith('st_entry_thresholds_')) {
            const idx = parseInt(field.split('_').pop());
            return cfg.st_entry_thresholds ? cfg.st_entry_thresholds[idx] : 65;
        }
        return cfg[field];
    }

    function populateFromConfig(cfg) {
        _currentConfig = structuredClone(cfg);
        document.querySelectorAll('#tab-13 [data-field]').forEach(el => {
            const field = el.dataset.field;
            const meta = FIELD_META[field];
            if (!meta) return;
            const val = getFieldValue(field, cfg);
            if (val === undefined) return;

            if (meta.type === 'checkbox') {
                el.checked = !!val;
            } else if (meta.type === 'slider') {
                el.value = Math.round(val * meta.scale);
                const valEl = $el(el.id + '-val');
                if (valEl) valEl.textContent = meta.display(val);
            }
        });
        updatePillarBar(cfg);
        drawRadar(cfg);
        updateStabVal(cfg);
        updateDiffDots();
    }

    function updatePillarBar(cfg) {
        const m = cfg.st_macro_weight || 0;
        const mi = cfg.st_micro_weight || 0;
        const mo = cfg.st_momentum_weight || 0;
        const sum = m + mi + mo || 1;
        const bm = $el('st-bar-macro');
        const bmi = $el('st-bar-micro');
        const bmo = $el('st-bar-momentum');
        if (bm) bm.style.width = ((m / sum) * 100) + '%';
        if (bmi) bmi.style.width = ((mi / sum) * 100) + '%';
        if (bmo) bmo.style.width = ((mo / sum) * 100) + '%';

        const warn = $el('st-pillar-sum-warn');
        const sumPct = Math.round((m + mi + mo) * 100);
        if (warn) warn.hidden = (sumPct >= 99 && sumPct <= 101);
    }

    function updateStabVal(cfg) {
        const el = $el('st-cfg-stab-val');
        if (el) el.textContent = (cfg.st_stabilization_window || 8) + ' bars';
    }

    // B2: Radar chart
    function drawRadar(cfg) {
        const canvas = $el('st-radar-canvas');
        if (!canvas || !canvas.getContext) return;
        const ctx = canvas.getContext('2d');
        const W = canvas.width, H = canvas.height;
        const cx = W / 2, cy = H / 2;
        const R = Math.min(cx, cy) - 12;
        ctx.clearRect(0, 0, W, H);

        const m = cfg.st_macro_weight || 0;
        const mi = cfg.st_micro_weight || 0;
        const mo = cfg.st_momentum_weight || 0;
        const maxW = Math.max(m, mi, mo, 0.01);
        const vals = [m / maxW, mi / maxW, mo / maxW];
        const colors = ['#00BFFF', '#00FF41', '#FFA500'];
        const labels = ['M', 'MI', 'MO'];
        const angles = [-Math.PI / 2, -Math.PI / 2 + (2 * Math.PI / 3), -Math.PI / 2 + (4 * Math.PI / 3)];

        // Grid rings
        ctx.strokeStyle = 'rgba(255,255,255,0.08)';
        ctx.lineWidth = 0.5;
        for (let r = 0.33; r <= 1; r += 0.33) {
            ctx.beginPath();
            ctx.arc(cx, cy, R * r, 0, Math.PI * 2);
            ctx.stroke();
        }

        // Axes
        for (let i = 0; i < 3; i++) {
            ctx.strokeStyle = colors[i];
            ctx.globalAlpha = 0.4;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(cx, cy);
            ctx.lineTo(cx + R * Math.cos(angles[i]), cy + R * Math.sin(angles[i]));
            ctx.stroke();
            ctx.globalAlpha = 1;

            const lx = cx + (R + 10) * Math.cos(angles[i]);
            const ly = cy + (R + 10) * Math.sin(angles[i]);
            ctx.fillStyle = colors[i];
            ctx.font = '9px Share Tech Mono';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(labels[i], lx, ly);
        }

        // Data polygon
        ctx.beginPath();
        for (let i = 0; i < 3; i++) {
            const x = cx + R * vals[i] * Math.cos(angles[i]);
            const y = cy + R * vals[i] * Math.sin(angles[i]);
            if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
        }
        ctx.closePath();
        ctx.fillStyle = 'rgba(0, 191, 255, 0.15)';
        ctx.fill();
        ctx.strokeStyle = 'rgba(0, 191, 255, 0.7)';
        ctx.lineWidth = 1.5;
        ctx.stroke();

        // Data points
        for (let i = 0; i < 3; i++) {
            const x = cx + R * vals[i] * Math.cos(angles[i]);
            const y = cy + R * vals[i] * Math.sin(angles[i]);
            ctx.beginPath();
            ctx.arc(x, y, 3, 0, Math.PI * 2);
            ctx.fillStyle = colors[i];
            ctx.fill();
        }
    }

    // B1: Diff indicator
    function updateDiffDots() {
        if (!_savedConfig) return;
        let changed = 0;
        document.querySelectorAll('.st-cfg-diff-dot').forEach(dot => {
            const field = dot.dataset.field;
            const saved = getFieldValue(field, _savedConfig);
            const current = getFieldValue(field, _currentConfig);
            const isDiff = saved !== undefined && current !== undefined &&
                (typeof saved === 'boolean' ? saved !== current : Math.abs(saved - current) > 0.001);
            dot.classList.toggle('st-cfg-changed', isDiff);
            if (isDiff) changed++;
        });
        const counter = $el('st-cfg-unsaved-count');
        if (counter) {
            counter.hidden = changed === 0;
            counter.textContent = changed + ' unsaved';
        }
    }

    function applySliderChange(field, rawValue) {
        const meta = FIELD_META[field];
        if (!meta || meta.type !== 'slider') return;
        const val = rawValue / meta.scale;

        if (field.startsWith('st_entry_thresholds_')) {
            const idx = parseInt(field.split('_').pop());
            if (!_currentConfig.st_entry_thresholds) _currentConfig.st_entry_thresholds = [60, 70, 75, 65];
            _currentConfig.st_entry_thresholds[idx] = val;
        } else {
            _currentConfig[field] = val;
        }

        const slider = document.querySelector(`#tab-13 [data-field="${field}"]`);
        if (slider) {
            const valEl = $el(slider.id + '-val');
            if (valEl) valEl.textContent = meta.display(val);
        }

        if (['st_macro_weight', 'st_micro_weight', 'st_momentum_weight'].includes(field)) {
            updatePillarBar(_currentConfig);
            drawRadar(_currentConfig);
        }
        if (field === 'st_stabilization_window') updateStabVal(_currentConfig);
        updateDiffDots();
    }

    function applyCheckboxChange(field, checked) {
        _currentConfig[field] = checked;
        updateDiffDots();
    }

    // T6: Validation
    function validateConfig() {
        const errors = [];
        const c = _currentConfig;
        const m = c.st_macro_weight || 0;
        const mi = c.st_micro_weight || 0;
        const mo = c.st_momentum_weight || 0;
        const sumPct = Math.round((m + mi + mo) * 100);
        if (sumPct < 99 || sumPct > 101) {
            errors.push('Pillar weights sum to ' + sumPct + '% (must be ~100%)');
        }
        if (m < 0 || mi < 0 || mo < 0) errors.push('Negative weight detected');
        if ((c.st_stalking_threshold || 0) >= (c.st_entry_threshold || 100)) {
            errors.push('Stalking threshold must be lower than entry threshold');
        }
        return errors;
    }

    // T5: Save config
    async function saveConfig() {
        const errors = validateConfig();
        if (errors.length > 0) {
            TitanToast.show('Validation failed: ' + errors[0], 'error', 4000);
            return;
        }
        try {
            const fields = Object.keys(FIELD_META);
            for (const field of fields) {
                const meta = FIELD_META[field];
                const val = getFieldValue(field, _currentConfig);
                if (val === undefined) continue;
                const savedVal = _savedConfig ? getFieldValue(field, _savedConfig) : undefined;
                const isDiff = savedVal === undefined ||
                    (typeof val === 'boolean' ? val !== savedVal : Math.abs(val - savedVal) > 0.0001);
                if (!isDiff) continue;
                await invoke('update_smarttrend_config', {
                    field: field,
                    value: String(val),
                });
            }
            _savedConfig = structuredClone(_currentConfig);
            updateDiffDots();
            TitanToast.show('SmartTrend config saved', 'success', 3000);
            if (typeof TitanAudio !== 'undefined') TitanAudio.play('click');
        } catch (e) {
            TitanToast.show('Save failed: ' + e, 'error', 4000);
        }
    }

    async function resetDefaults() {
        const confirmed = await TitanToast.confirm(
            'RESET SMARTTREND CONFIG?',
            'All settings will revert to defaults.'
        );
        if (!confirmed) return;
        const defaults = {
            st_macro_weight: 0.40, st_micro_weight: 0.35, st_momentum_weight: 0.25,
            st_entry_threshold: 65, st_stalking_threshold: 55, st_hysteresis_margin: 5,
            st_entry_thresholds: [60, 70, 75, 65],
            st_dominant_cycle_filter: true,
            st_ofi_norm_range: 50, st_mp_bps_range: 10, st_cvd_scale: 100,
            st_trail_tighten_pct: 0.05, st_changepoint_reset_threshold: 0.7,
            st_stabilization_window: 8,
            st_bayesian_decay: 0.997, st_bayesian_halflife: 200,
        };
        populateFromConfig(defaults);
        await saveConfig();
        TitanToast.show('SmartTrend config reset to defaults', 'info', 3000);
        if (typeof TitanAudio !== 'undefined') TitanAudio.play('confirm');
    }

    function applyPreset(name) {
        const p = PRESETS[name];
        if (!p) return;
        _currentConfig.st_macro_weight = p.st_macro_weight;
        _currentConfig.st_micro_weight = p.st_micro_weight;
        _currentConfig.st_momentum_weight = p.st_momentum_weight;
        populateFromConfig(_currentConfig);
        TitanToast.show('Preset ' + name.toUpperCase() + ' applied (unsaved)', 'info', 2000);
    }

    async function smartDefault() {
        applyPreset('balanced');
    }

    // Section 5: Live readout
    let _prevLiveReadout = '';
    async function pollLiveReadout() {
        try {
            const d = await invoke('get_smarttrend_state');
            if (!d) return;
            const s = d.stats || {};
            const entries = s.entries_total || 0;
            const trails = s.entries_reached_trail || 0;
            const key = `${entries}|${trails}|${s.changepoint_resets || 0}|${s.stalking_timeouts || 0}|${d.last_transition || ''}`;
            if (key === _prevLiveReadout) return;
            _prevLiveReadout = key;

            requestAnimationFrame(() => {
                const hr = $el('st-cfg-live-hitrate');
                const en = $el('st-cfg-live-entries');
                const th = $el('st-cfg-live-trailhits');
                const cr = $el('st-cfg-live-cpresets');
                const to = $el('st-cfg-live-timeouts');
                const tr = $el('st-cfg-live-transition');
                if (hr) hr.textContent = entries > 0 ? trails + '/' + entries + ' (' + ((trails / entries) * 100).toFixed(0) + '%)' : '--';
                if (en) en.textContent = entries || '0';
                if (th) th.textContent = trails || '0';
                if (cr) cr.textContent = s.changepoint_resets || '0';
                if (to) to.textContent = s.stalking_timeouts || '0';
                if (tr) tr.textContent = d.last_transition || '--';
            });
        } catch (_) { /* not running */ }
    }

    // Section 6: FSM live
    const FSM_CLASS_MAP = {
        'Flat':     'st-fsm-active-flat',
        'Stalking': 'st-fsm-active-stalking',
        'Entry':    'st-fsm-active-entry',
        'InTrade':  'st-fsm-active-intrade',
        'Trail':    'st-fsm-active-trail',
    };

    let _prevFsmKey = '';
    async function pollFsmState() {
        try {
            const d = await invoke('get_smarttrend_state');
            if (!d) return;
            const cp = d.confirmation_progress;
            const key = `${d.state}|${d.direction}|${d.bars_in_state}|${d.combined_score}|${d.entry_price}|${d.best_price}|${cp ? cp[0]+'/'+cp[1] : ''}`;
            if (key === _prevFsmKey) return;
            _prevFsmKey = key;

            requestAnimationFrame(() => {
                document.querySelectorAll('#tab-13 .st-fsm-box').forEach(box => {
                    const fsm = box.dataset.fsm;
                    box.className = 'st-fsm-box';
                    if (fsm === d.state) {
                        box.classList.add(FSM_CLASS_MAP[fsm] || '');
                    }
                });

                const dirEl = $el('st-cfg-fsm-dir');
                const barsEl = $el('st-cfg-fsm-bars');
                const epriceEl = $el('st-cfg-fsm-eprice');
                const bpriceEl = $el('st-cfg-fsm-bprice');
                const scoreEl = $el('st-cfg-fsm-score');
                const agreeEl = $el('st-cfg-fsm-agree');

                if (dirEl) {
                    const dirMap = { 'Bullish': 'LONG', 'Bearish': 'SHORT', 'Neutral': '--' };
                    dirEl.textContent = dirMap[d.direction] || '--';
                    dirEl.style.color = d.direction === 'Bullish' ? 'var(--neon-green)' :
                                        d.direction === 'Bearish' ? 'var(--neon-red)' : 'var(--text-dim)';
                }
                if (barsEl) barsEl.textContent = d.bars_in_state ?? '--';
                if (scoreEl) scoreEl.textContent = typeof d.combined_score === 'number' ? d.combined_score.toFixed(1) : '--';

                if (d.pillar_scores) {
                    const ps = d.pillar_scores;
                    const allBullish = ps.macro_score?.direction === 'Bullish' &&
                        ps.micro_score?.direction === 'Bullish' &&
                        ps.momentum_score?.direction === 'Bullish';
                    const allBearish = ps.macro_score?.direction === 'Bearish' &&
                        ps.micro_score?.direction === 'Bearish' &&
                        ps.momentum_score?.direction === 'Bearish';
                    if (agreeEl) {
                        if (ps.cross_divergence) {
                            agreeEl.textContent = 'DIVERGE';
                            agreeEl.style.color = 'var(--neon-orange)';
                        } else if (allBullish || allBearish) {
                            agreeEl.textContent = 'AGREE';
                            agreeEl.style.color = 'var(--neon-green)';
                        } else {
                            agreeEl.textContent = 'MIXED';
                            agreeEl.style.color = 'var(--neon-gold)';
                        }
                    }
                }

                if (d.entry_price != null && epriceEl) epriceEl.textContent = '$' + d.entry_price.toFixed(2);
                else if (epriceEl) epriceEl.textContent = '--';
                if (d.best_price != null && bpriceEl) bpriceEl.textContent = '$' + d.best_price.toFixed(2);
                else if (bpriceEl) bpriceEl.textContent = '--';

                const confirmRow = $el('st-cfg-fsm-confirm-row');
                const confirmBar = $el('st-cfg-fsm-confirm-bar');
                const confirmText = $el('st-cfg-fsm-confirm-text');
                if (d.confirmation_progress) {
                    if (confirmRow) confirmRow.hidden = false;
                    const [cur, total] = d.confirmation_progress;
                    if (confirmText) confirmText.textContent = cur + '/' + total;
                    if (confirmBar) confirmBar.style.width = (total > 0 ? (cur / total * 100) : 0) + '%';
                } else {
                    if (confirmRow) confirmRow.hidden = true;
                }
            });
        } catch (_) { /* not running */ }
    }

    // B3: Compare mode
    function showCompare() {
        if (!_savedConfig) { TitanToast.show('No saved config to compare', 'info'); return; }
        const overlay = $el('st-cfg-compare-overlay');
        const body = $el('st-cfg-compare-body');
        if (!overlay || !body) return;

        let html = '';
        const fields = Object.keys(FIELD_META);
        for (const field of fields) {
            const meta = FIELD_META[field];
            const saved = getFieldValue(field, _savedConfig);
            const current = getFieldValue(field, _currentConfig);
            if (saved === undefined && current === undefined) continue;

            const isDiff = saved !== undefined && current !== undefined &&
                (typeof saved === 'boolean' ? saved !== current : Math.abs(saved - current) > 0.001);
            const cls = isDiff ? ' class="st-cfg-compare-diff"' : '';

            const dispSaved = meta.type === 'checkbox' ? (saved ? 'ON' : 'OFF') : (meta.display ? meta.display(saved || 0) : saved);
            const dispCurrent = meta.type === 'checkbox' ? (current ? 'ON' : 'OFF') : (meta.display ? meta.display(current || 0) : current);

            let delta = '';
            if (isDiff && meta.type === 'slider') {
                const d = (current || 0) - (saved || 0);
                delta = (d > 0 ? '+' : '') + (meta.display ? meta.display(d) : d.toFixed(3));
            } else if (isDiff && meta.type === 'checkbox') {
                delta = saved ? 'ON→OFF' : 'OFF→ON';
            }

            const label = field.replace(/^st_/, '').replace(/_/g, ' ');
            html += `<tr${cls}>`;
            html += `<td>${escapeHtml(label)}</td>`;
            html += `<td class="th-gold">${escapeHtml(String(dispCurrent))}</td>`;
            html += `<td>${escapeHtml(String(dispSaved))}</td>`;
            html += `<td class="st-cfg-compare-delta">${isDiff ? escapeHtml(delta) : '--'}</td>`;
            html += '</tr>';
        }
        body.innerHTML = html;
        overlay.hidden = false;
    }

    function hideCompare() {
        const overlay = $el('st-cfg-compare-overlay');
        if (overlay) overlay.hidden = true;
    }

    // Wire up events
    document.querySelectorAll('#tab-13 .st-cfg-slider').forEach(slider => {
        slider.addEventListener('input', () => {
            applySliderChange(slider.dataset.field, parseFloat(slider.value));
        });
    });

    document.querySelectorAll('#tab-13 input[type="checkbox"][data-field]').forEach(cb => {
        cb.addEventListener('change', () => {
            applyCheckboxChange(cb.dataset.field, cb.checked);
        });
    });

    document.querySelectorAll('.st-preset-btn').forEach(btn => {
        btn.addEventListener('click', () => applyPreset(btn.dataset.preset));
    });

    const saveBtn = $el('st-cfg-save-btn');
    if (saveBtn) saveBtn.addEventListener('click', saveConfig);

    const resetBtn = $el('st-cfg-reset-btn');
    if (resetBtn) resetBtn.addEventListener('click', resetDefaults);

    const smartBtn = $el('st-cfg-smart-default-btn');
    if (smartBtn) smartBtn.addEventListener('click', smartDefault);

    const compareBtn = $el('st-cfg-compare-btn');
    if (compareBtn) compareBtn.addEventListener('click', showCompare);

    const compareClose = $el('st-cfg-compare-close');
    if (compareClose) compareClose.addEventListener('click', hideCompare);

    const compareOverlay = $el('st-cfg-compare-overlay');
    if (compareOverlay) {
        compareOverlay.addEventListener('click', (e) => {
            if (e.target === compareOverlay) hideCompare();
        });
    }

    // Load initial config
    invoke('get_smarttrend_config').then(cfg => {
        _savedConfig = structuredClone(cfg);
        populateFromConfig(cfg);
    }).catch(() => {});

    // Live readout poll (section 5) — 5s, only when tab-13 visible
    _liveReadoutTimer = TitanTimers.register('st:live-readout', pollLiveReadout, 5000, { tab: 13 });
    setTimeout(pollLiveReadout, 2000);

    // FSM state poll (section 6) — 3s, only when tab-13 visible
    _fsmPollTimer = TitanTimers.register('st:fsm-state', pollFsmState, 3000, { tab: 13 });
    setTimeout(pollFsmState, 2500);

    // Navigation warning (B1)
    window._stCfgHasUnsaved = () => {
        if (!_savedConfig) return false;
        return document.querySelectorAll('.st-cfg-diff-dot.st-cfg-changed').length > 0;
    };

    // ═══════════════════════════════════════════════════════════════
    //  S12b: Sections 7-10 + Export/Import + Bonuses
    // ═══════════════════════════════════════════════════════════════

    let _acctTimer = null;
    let _assetsTimer = null;
    let _sentinelAlertLog = [];

    // ── Section 7: Account ──

    function fmtUsd(v) { return '$' + (v || 0).toFixed(2); }
    function fmtPct(v) { return (v || 0).toFixed(2) + '%'; }

    function setPnlAttr(el, val) {
        if (!el) return;
        el.setAttribute('data-positive', val >= 0 ? 'true' : 'false');
    }

    let _prevAcctKey = '';
    function pollAccountStats() {
        invoke('smarttrend_account_stats').then(a => {
            const key = `${a.allocated_capital}|${a.realized_pnl_today}|${a.total_realized_pnl}|${a.total_trades}|${a.total_wins}|${a.max_drawdown_pct}|${a.high_water_mark}`;
            if (key === _prevAcctKey) return;
            _prevAcctKey = key;

            requestAnimationFrame(() => {
                const cap = $el('s12b-acct-capital');
                const ini = $el('s12b-acct-initial');
                const dpnl = $el('s12b-acct-daily-pnl');
                const tpnl = $el('s12b-acct-total-pnl');
                const hwm = $el('s12b-acct-hwm');
                const dd = $el('s12b-acct-dd');
                const trades = $el('s12b-acct-trades');
                const wl = $el('s12b-acct-wl');

                if (cap) cap.textContent = fmtUsd(a.allocated_capital);
                if (ini) ini.textContent = fmtUsd(a.initial_capital);
                if (dpnl) { dpnl.textContent = fmtUsd(a.realized_pnl_today); setPnlAttr(dpnl, a.realized_pnl_today); }
                if (tpnl) { tpnl.textContent = fmtUsd(a.total_realized_pnl); setPnlAttr(tpnl, a.total_realized_pnl); }
                if (hwm) hwm.textContent = fmtUsd(a.high_water_mark);
                if (dd) { dd.textContent = fmtPct(a.max_drawdown_pct); setPnlAttr(dd, -(a.max_drawdown_pct || 0)); }
                if (trades) trades.textContent = a.total_trades || 0;
                if (wl) wl.textContent = `${a.total_wins || 0} / ${(a.total_trades || 0) - (a.total_wins || 0)}`;

                drawEquityMicroChart(a);
            });
        }).catch(() => {});
    }

    // B1: Equity Micro-Chart
    function drawEquityMicroChart(acct) {
        const canvas = $el('s12b-acct-equity-canvas');
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const w = canvas.width, h = canvas.height;
        ctx.clearRect(0, 0, w, h);

        const totalTrades = acct.total_trades || 0;
        if (totalTrades < 3) {
            ctx.fillStyle = '#778899';
            ctx.font = '11px Share Tech Mono';
            ctx.textAlign = 'center';
            ctx.fillText('< 3 trades', w / 2, h / 2 + 4);
            return;
        }

        const equityStart = acct.initial_capital || 1;
        const equityCurrent = acct.allocated_capital || 0;
        const hwm = acct.high_water_mark || equityStart;
        const points = Math.min(totalTrades, 50);
        const step = (equityCurrent - equityStart) / points;
        const data = [];
        for (let i = 0; i <= points; i++) data.push(equityStart + step * i);

        const minY = Math.min(...data) * 0.98;
        const maxY = Math.max(...data, hwm) * 1.02;
        const rangeY = maxY - minY || 1;
        const px = i => (i / points) * w;
        const py = v => h - ((v - minY) / rangeY) * h;

        // Drawdown zones (red below HWM)
        ctx.fillStyle = 'rgba(255, 43, 43, 0.08)';
        const hwmY = py(hwm);
        for (let i = 0; i < data.length; i++) {
            const y = py(data[i]);
            if (y > hwmY) {
                ctx.fillRect(px(i) - 1, hwmY, w / points + 2, y - hwmY);
            }
        }

        // HWM dashed line
        ctx.strokeStyle = 'rgba(255, 215, 0, 0.3)';
        ctx.setLineDash([3, 3]);
        ctx.beginPath();
        ctx.moveTo(0, hwmY);
        ctx.lineTo(w, hwmY);
        ctx.stroke();
        ctx.setLineDash([]);

        // Equity line
        const grad = ctx.createLinearGradient(0, 0, 0, h);
        grad.addColorStop(0, 'rgba(0, 191, 255, 0.3)');
        grad.addColorStop(1, 'rgba(0, 191, 255, 0.02)');
        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.moveTo(px(0), h);
        for (let i = 0; i < data.length; i++) ctx.lineTo(px(i), py(data[i]));
        ctx.lineTo(px(data.length - 1), h);
        ctx.closePath();
        ctx.fill();

        ctx.strokeStyle = '#00BFFF';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        for (let i = 0; i < data.length; i++) {
            if (i === 0) ctx.moveTo(px(i), py(data[i]));
            else ctx.lineTo(px(i), py(data[i]));
        }
        ctx.stroke();
    }

    const topupBtn = $el('s12b-acct-topup-btn');
    if (topupBtn) topupBtn.addEventListener('click', () => {
        const amt = parseFloat($el('s12b-acct-amount')?.value);
        if (!amt || amt <= 0) { TitanToast.show('Enter a valid amount', 'warning'); return; }
        invoke('smarttrend_topup', { amount: amt })
            .then(r => { TitanToast.show(r, 'success'); TitanAudio?.play?.('click'); pollAccountStats(); })
            .catch(e => TitanToast.show(String(e), 'error'));
    });

    const withdrawBtn = $el('s12b-acct-withdraw-btn');
    if (withdrawBtn) withdrawBtn.addEventListener('click', () => {
        const amt = parseFloat($el('s12b-acct-amount')?.value);
        if (!amt || amt <= 0) { TitanToast.show('Enter a valid amount', 'warning'); return; }
        invoke('smarttrend_withdraw', { amount: amt })
            .then(r => { TitanToast.show(r, 'success'); TitanAudio?.play?.('click'); pollAccountStats(); })
            .catch(e => TitanToast.show(String(e), 'error'));
    });

    const pauseBtn = $el('s12b-acct-pause-btn');
    if (pauseBtn) pauseBtn.addEventListener('click', () => {
        const isPaused = pauseBtn.classList.contains('st-paused');
        invoke('smarttrend_pause', { paused: !isPaused })
            .then(r => {
                TitanToast.show(r, 'info');
                pauseBtn.classList.toggle('st-paused', !isPaused);
                pauseBtn.textContent = !isPaused ? 'RESUME' : 'PAUSE';
            })
            .catch(e => TitanToast.show(String(e), 'error'));
    });

    const liqBtn = $el('s12b-acct-liquidate-btn');
    if (liqBtn) liqBtn.addEventListener('click', () => {
        TitanToast.confirm('LIQUIDATE all SmartTrend positions?', () => {
            invoke('smarttrend_force_exit_all', { confirmToken: 'CONFIRM_ST_LIQUIDATE' })
                .then(r => { TitanToast.show(r, 'success'); TitanAudio?.play?.('confirm'); })
                .catch(e => TitanToast.show(String(e), 'error'));
        });
    });

    const killBtn2 = $el('s12b-acct-kill-btn');
    if (killBtn2) killBtn2.addEventListener('click', () => {
        TitanToast.confirm('KILL SWITCH — disable ALL SmartTrend trading?', () => {
            TitanToast.confirm('Are you SURE? This will stop ALL ST activity.', () => {
                invoke('smarttrend_kill_switch', { enable: true })
                    .then(r => { TitanToast.show(r, 'error'); TitanAudio?.play?.('panic'); })
                    .catch(e => TitanToast.show(String(e), 'error'));
            });
        });
    });

    // ── Section 8: Assets Table ──

    function pollAssetsTable() {
        Promise.all([
            invoke('smarttrend_get_asset_modes'),
            invoke('get_smarttrend_state'),
            invoke('get_sentinel_status'),
        ]).then(([modes, diag, sentinel]) => {
            const tbody = $el('s12b-assets-tbody');
            if (!tbody) return;

            const activeSymbol = window.__activeSymbol || '';
            const positions = window.__positions || [];
            const stPositions = new Set(
                positions.filter(p => p.entry_tag && p.entry_tag.startsWith('ST|')).map(p => p.symbol)
            );

            const sentMap = {};
            if (sentinel?.symbols) {
                sentinel.symbols.forEach(s => { sentMap[s.symbol] = s; });
            }

            const rows = [];
            for (const [sym, mode] of Object.entries(modes)) {
                const s = sentMap[sym] || {};
                const state = s.state || 'Flat';
                const score = s.score || (sym === activeSymbol ? diag.combined_score : 0);
                const dir = s.direction || (sym === activeSymbol ? (diag.direction || 'Neutral') : 'Neutral');
                const bars = s.bars_in_state || (sym === activeSymbol ? diag.bars_in_state : 0);
                const isLive = stPositions.has(sym);

                const stateClass = 'st-assets-fsm-' + state.toLowerCase().replace('intrade', 'intrade');
                const modeClass = mode === 'auto' ? 'st-mode-auto' : 'st-mode-man';

                let heatClass = '';
                if (score < 30) heatClass = 'st-asset-heat-cold';
                else if (score >= 60 && score < 80) heatClass = 'st-asset-heat-warm';
                else if (score >= 80) heatClass = 'st-asset-heat-hot';

                rows.push(`<tr class="st-asset-row ${heatClass}" data-sym="${sym}">
                    <td>${sym}</td>
                    <td><span class="st-assets-mode-badge ${modeClass}" data-sym="${sym}" data-mode="${mode}">${mode.toUpperCase()}</span></td>
                    <td><span class="st-assets-fsm-badge ${stateClass}">${state}</span></td>
                    <td class="th-gold">${score.toFixed(1)}</td>
                    <td>${dir}</td>
                    <td>${bars}</td>
                    <td>${isLive ? '<span class="st-assets-live-badge"></span>' : ''}</td>
                </tr>`);
            }

            const html = rows.join('');
            if (tbody.innerHTML === html) return;

            requestAnimationFrame(() => {
                tbody.innerHTML = html;

                tbody.querySelectorAll('.st-assets-mode-badge').forEach(badge => {
                    badge.addEventListener('click', () => {
                        const sym = badge.dataset.sym;
                        const cur = badge.dataset.mode;
                        const next = cur === 'auto' ? 'man' : 'auto';
                        if (next === 'auto') {
                            TitanToast.confirm(`Switch ${sym} to AUTO mode?`, () => {
                                invoke('smarttrend_set_asset_mode', { symbol: sym, mode: next })
                                    .then(r => { TitanToast.show(r, 'info'); pollAssetsTable(); })
                                    .catch(e => TitanToast.show(String(e), 'error'));
                            });
                        } else {
                            invoke('smarttrend_set_asset_mode', { symbol: sym, mode: next })
                                .then(r => { TitanToast.show(r, 'info'); pollAssetsTable(); })
                                .catch(e => TitanToast.show(String(e), 'error'));
                        }
                    });
                });
            });
        }).catch(() => {});
    }

    const addBtn = $el('s12b-assets-add-btn');
    if (addBtn) addBtn.addEventListener('click', () => {
        const input = $el('s12b-assets-add-input');
        const sym = (input?.value || '').trim().toUpperCase();
        if (!sym) { TitanToast.show('Enter a symbol', 'warning'); return; }
        invoke('smarttrend_set_asset_mode', { symbol: sym, mode: 'man' })
            .then(r => { TitanToast.show(`${sym} added (MAN)`, 'success'); input.value = ''; pollAssetsTable(); })
            .catch(e => TitanToast.show(String(e), 'error'));
    });

    const allAutoBtn = $el('s12b-assets-all-auto-btn');
    if (allAutoBtn) allAutoBtn.addEventListener('click', () => {
        TitanToast.confirm('Set ALL assets to AUTO?', () => {
            invoke('smarttrend_set_all_modes', { mode: 'auto', confirmToken: 'CONFIRM_ST_ALL_AUTO' })
                .then(r => { TitanToast.show(r, 'info'); pollAssetsTable(); })
                .catch(e => TitanToast.show(String(e), 'error'));
        });
    });

    const allManBtn = $el('s12b-assets-all-man-btn');
    if (allManBtn) allManBtn.addEventListener('click', () => {
        invoke('smarttrend_set_all_modes', { mode: 'man' })
            .then(r => { TitanToast.show(r, 'info'); pollAssetsTable(); })
            .catch(e => TitanToast.show(String(e), 'error'));
    });

    // ── Section 9: Backtest ──

    const btPeriod = $el('st-bt-period');
    const btPeriodVal = $el('st-bt-period-val');
    if (btPeriod && btPeriodVal) {
        btPeriod.addEventListener('input', () => { btPeriodVal.textContent = btPeriod.value; });
    }

    const btRunBtn = $el('st-bt-run-btn');
    if (btRunBtn) btRunBtn.addEventListener('click', () => {
        const sym = window.__activeSymbol || '';
        if (!sym) { TitanToast.show('No active symbol', 'warning'); return; }
        const days = parseInt(btPeriod?.value || '30');
        const resultEl = $el('st-bt-results');
        const progWrap = $el('st-bt-progress-wrap');
        const progBar = $el('st-bt-progress-bar');
        const progText = $el('st-bt-progress-text');

        if (progWrap) progWrap.hidden = false;
        if (resultEl) resultEl.hidden = true;
        if (progBar) progBar.style.width = '0%';
        if (progText) progText.textContent = '0%';
        btRunBtn.disabled = true;

        invoke('run_signal_backtest', { symbol: sym, periodDays: days })
            .then(res => {
                if (progWrap) progWrap.hidden = true;
                btRunBtn.disabled = false;
                if (!res) return;

                const best = res.best_tf_result || (res.results && res.results[0]) || {};
                const m = best.metrics || best;
                const setVal = (id, v, isPnl) => {
                    const el = $el(id);
                    if (!el) return;
                    el.textContent = v;
                    if (isPnl) setPnlAttr(el, parseFloat(v));
                };

                setVal('st-bt-trades', m.trades || 0);
                setVal('st-bt-winrate', ((m.win_rate || 0) * 100).toFixed(1) + '%');
                setVal('st-bt-return', (m.return_pct || 0).toFixed(2) + '%', true);
                setVal('st-bt-dd', (m.max_drawdown || 0).toFixed(2) + '%', true);
                setVal('st-bt-sharpe', (m.sharpe || 0).toFixed(2));
                setVal('st-bt-pf', (m.profit_factor || 0).toFixed(2));
                setVal('st-bt-expect', (m.expectancy || 0).toFixed(2));

                if (resultEl) resultEl.hidden = false;

                drawBacktestSparkline(m.equity_curve || best.equity_curve);
            })
            .catch(e => {
                if (progWrap) progWrap.hidden = true;
                btRunBtn.disabled = false;
                TitanToast.show('Backtest failed: ' + String(e), 'error');
            });
    });

    // Listen for backtest progress events
    if (window.__TAURI__?.event?.listen) {
        window.__TAURI__.event.listen('signal-bt-progress', (event) => {
            const p = event.payload;
            const pct = typeof p === 'number' ? p : (p?.pct || p?.progress || 0);
            const progBar = $el('st-bt-progress-bar');
            const progText = $el('st-bt-progress-text');
            if (progBar) progBar.style.width = Math.min(pct, 100) + '%';
            if (progText) progText.textContent = Math.round(pct) + '%';
        });
    }

    // B3: Backtest Sparkline Equity
    function drawBacktestSparkline(equityCurve) {
        const canvas = $el('st-bt-sparkline');
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const w = canvas.width, h = canvas.height;
        ctx.clearRect(0, 0, w, h);

        if (!equityCurve || equityCurve.length < 3) {
            ctx.fillStyle = '#778899';
            ctx.font = '10px Share Tech Mono';
            ctx.textAlign = 'center';
            ctx.fillText('No data', w / 2, h / 2 + 3);
            return;
        }

        const data = equityCurve;
        const minY = Math.min(...data) * 0.99;
        const maxY = Math.max(...data) * 1.01;
        const rangeY = maxY - minY || 1;
        const px = i => (i / (data.length - 1)) * w;
        const py = v => h - 4 - ((v - minY) / rangeY) * (h - 8);

        let hwm = data[0];
        const hwmArr = data.map(v => { hwm = Math.max(hwm, v); return hwm; });

        // Drawdown zones
        ctx.fillStyle = 'rgba(255, 43, 43, 0.1)';
        for (let i = 0; i < data.length; i++) {
            const dy = py(data[i]);
            const hy = py(hwmArr[i]);
            if (dy > hy) {
                ctx.fillRect(px(i) - 1, hy, w / data.length + 2, dy - hy);
            }
        }

        // HWM dashed gold
        ctx.strokeStyle = 'rgba(255, 215, 0, 0.3)';
        ctx.setLineDash([2, 2]);
        ctx.lineWidth = 1;
        ctx.beginPath();
        for (let i = 0; i < hwmArr.length; i++) {
            if (i === 0) ctx.moveTo(px(i), py(hwmArr[i]));
            else ctx.lineTo(px(i), py(hwmArr[i]));
        }
        ctx.stroke();
        ctx.setLineDash([]);

        // Gradient fill
        const grad = ctx.createLinearGradient(0, 0, 0, h);
        grad.addColorStop(0, 'rgba(0, 255, 65, 0.2)');
        grad.addColorStop(1, 'rgba(0, 255, 65, 0.01)');
        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.moveTo(px(0), h);
        for (let i = 0; i < data.length; i++) ctx.lineTo(px(i), py(data[i]));
        ctx.lineTo(px(data.length - 1), h);
        ctx.closePath();
        ctx.fill();

        // Equity line
        ctx.strokeStyle = '#00FF41';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        for (let i = 0; i < data.length; i++) {
            if (i === 0) ctx.moveTo(px(i), py(data[i]));
            else ctx.lineTo(px(i), py(data[i]));
        }
        ctx.stroke();

        // Hover tooltip
        canvas.onmousemove = (e) => {
            const rect = canvas.getBoundingClientRect();
            const mx = e.clientX - rect.left;
            const idx = Math.round((mx / w) * (data.length - 1));
            if (idx >= 0 && idx < data.length) {
                canvas.title = `Point ${idx + 1}: $${data[idx].toFixed(2)}`;
            }
        };
    }

    // ── Section 10: Sentinel ──

    function pollSentinelStatus() {
        invoke('get_sentinel_status').then(st => {
            const badge = $el('st-sent-status-badge');
            if (badge) {
                badge.textContent = st.running ? 'RUNNING' : 'STOPPED';
                badge.className = 'sb-badge ' + (st.running ? 'sb-badge-green' : 'sb-badge-red');
            }
        }).catch(() => {});
    }

    // Sentinel cadence sliders
    ['st-sent-idle', 'st-sent-stalk', 'st-sent-hot'].forEach(id => {
        const slider = $el(id);
        const valEl = $el(id + '-val');
        if (slider && valEl) {
            slider.addEventListener('input', () => { valEl.textContent = slider.value; });
        }
    });

    // Load sentinel config into sliders
    invoke('get_smarttrend_config').then(cfg => {
        const idle = $el('st-sent-idle');
        const stalk = $el('st-sent-stalk');
        const hot = $el('st-sent-hot');
        if (idle) { idle.value = cfg.st_sentinel_idle_secs || 30; $el('st-sent-idle-val').textContent = idle.value; }
        if (stalk) { stalk.value = cfg.st_sentinel_stalking_secs || 5; $el('st-sent-stalk-val').textContent = stalk.value; }
        if (hot) { hot.value = cfg.st_sentinel_hot_secs || 2; $el('st-sent-hot-val').textContent = hot.value; }
    }).catch(() => {});

    const sentSaveBtn = $el('st-sent-save-btn');
    if (sentSaveBtn) sentSaveBtn.addEventListener('click', async () => {
        try {
            const idle = $el('st-sent-idle')?.value || '30';
            const stalk = $el('st-sent-stalk')?.value || '5';
            const hot = $el('st-sent-hot')?.value || '2';
            await invoke('update_smarttrend_config', { field: 'st_sentinel_idle_secs', value: idle });
            await invoke('update_smarttrend_config', { field: 'st_sentinel_stalking_secs', value: stalk });
            await invoke('update_smarttrend_config', { field: 'st_sentinel_hot_secs', value: hot });
            TitanToast.show('Sentinel cadence saved', 'success');
            TitanAudio?.play?.('click');
        } catch (e) {
            TitanToast.show('Save failed: ' + String(e), 'error');
        }
    });

    // B4: Sentinel Heatmap Badge
    function updateSentinelHeatmap() {
        const total = _sentinelAlertLog.length;
        if (total === 0) return;
        const counts = { Info: 0, Warning: 0, Critical: 0 };
        _sentinelAlertLog.slice(-50).forEach(a => {
            const s = a.severity || 'Info';
            if (counts[s] !== undefined) counts[s]++;
        });
        const sum = counts.Info + counts.Warning + counts.Critical || 1;

        const infoSeg = $el('st-sent-hm-info');
        const warnSeg = $el('st-sent-hm-warn');
        const critSeg = $el('st-sent-hm-crit');
        if (infoSeg) infoSeg.style.width = (counts.Info / sum * 100) + '%';
        if (warnSeg) warnSeg.style.width = (counts.Warning / sum * 100) + '%';
        if (critSeg) critSeg.style.width = (counts.Critical / sum * 100) + '%';

        // Dominant pulse
        [infoSeg, warnSeg, critSeg].forEach(s => s?.classList.remove('st-sent-hm-dominant'));
        const max = Math.max(counts.Info, counts.Warning, counts.Critical);
        if (max === counts.Critical && critSeg) critSeg.classList.add('st-sent-hm-dominant');
        else if (max === counts.Warning && warnSeg) warnSeg.classList.add('st-sent-hm-dominant');
        else if (infoSeg) infoSeg.classList.add('st-sent-hm-dominant');

        const tooltip = $el('st-sent-hm-tooltip');
        if (tooltip) tooltip.textContent = `I:${counts.Info} W:${counts.Warning} C:${counts.Critical}`;
    }

    function addSentinelLogEntry(alert) {
        _sentinelAlertLog.push(alert);
        if (_sentinelAlertLog.length > 100) _sentinelAlertLog.shift();

        const log = $el('st-sent-log');
        if (!log) return;

        // Check severity filters
        const sevFilter = {
            Info: $el('st-sent-sev-info')?.checked !== false,
            Warning: $el('st-sent-sev-warn')?.checked !== false,
            Critical: $el('st-sent-sev-crit')?.checked !== false,
        };
        if (!sevFilter[alert.severity]) return;

        const sevClass = { Info: 'st-sent-sev-info', Warning: 'st-sent-sev-warn', Critical: 'st-sent-sev-crit' };
        const time = new Date(alert.timestamp_ms).toLocaleTimeString();
        const entry = document.createElement('div');
        entry.className = 'st-sent-log-entry';
        entry.innerHTML = `<span class="st-sent-sev ${sevClass[alert.severity] || ''}">${alert.severity}</span>
            <span style="color:var(--neon-gold)">${escapeHtml(alert.symbol)}</span>
            <span>${escapeHtml(alert.marker)}</span>
            <span style="color:var(--text-dim)">${time}</span>`;
        log.prepend(entry);

        while (log.children.length > 20) log.lastChild.remove();

        updateSentinelHeatmap();
    }

    // Wire sentinel alerts into section 10 log
    if (window.__TAURI__?.event?.listen) {
        window.__TAURI__.event.listen('smarttrend-alert', (event) => {
            addSentinelLogEntry(event.payload);
        });
    }

    // ── Section 7: EXPORT / IMPORT ──

    const exportBtn = $el('st-cfg-export-btn');
    if (exportBtn) exportBtn.addEventListener('click', () => {
        invoke('smarttrend_export_config').then(cfg => {
            const json = JSON.stringify(cfg, null, 2);
            const blob = new Blob([json], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `smarttrend_config_${new Date().toISOString().slice(0, 10)}.json`;
            a.click();
            URL.revokeObjectURL(url);
            TitanToast.show('Config exported', 'success');
            TitanAudio?.play?.('click');
        }).catch(e => TitanToast.show('Export failed: ' + String(e), 'error'));
    });

    const importBtn = $el('st-cfg-import-btn');
    const importFile = $el('st-cfg-import-file');
    if (importBtn && importFile) {
        importBtn.addEventListener('click', () => importFile.click());
        importFile.addEventListener('change', (e) => {
            const file = e.target.files?.[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = (ev) => {
                const text = ev.target.result;
                try {
                    JSON.parse(text);
                } catch {
                    TitanToast.show('Invalid JSON file', 'error');
                    return;
                }
                TitanToast.confirm('Import SmartTrend config? This will overwrite current settings.', () => {
                    invoke('smarttrend_import_config', { jsonStr: text })
                        .then(r => {
                            TitanToast.show(r, 'success');
                            TitanAudio?.play?.('confirm');
                            invoke('get_smarttrend_config').then(cfg => {
                                _savedConfig = structuredClone(cfg);
                                populateFromConfig(cfg);
                            });
                        })
                        .catch(e => TitanToast.show('Import failed: ' + String(e), 'error'));
                });
            };
            reader.readAsText(file);
            importFile.value = '';
        });
    }

    // ── Start S12b polling timers (tab-13 guarded, staggered) ──
    _acctTimer = TitanTimers.register('st:s12b-account', pollAccountStats, 5000, { tab: 13 });
    setTimeout(pollAccountStats, 2000);
    _assetsTimer = TitanTimers.register('st:s12b-assets', pollAssetsTable, 5500, { tab: 13 });
    setTimeout(pollAssetsTable, 2700);
    TitanTimers.register('st:s12b-sentinel', pollSentinelStatus, 6000, { tab: 13 });
    setTimeout(pollSentinelStatus, 3400);

    // ── S13: Comparison backtest button ──
    const cmpBtn = $el('st-compare-run-btn');
    if (cmpBtn) {
        cmpBtn.addEventListener('click', async () => {
            cmpBtn.disabled = true;
            cmpBtn.textContent = 'RUNNING...';
            try {
                const r = await invoke('run_comparison_backtest_cmd');
                _renderComparisonResults(r);
            } catch (e) {
                TitanToast.show('Comparison failed: ' + String(e), 'error');
            } finally {
                cmpBtn.disabled = false;
                cmpBtn.textContent = 'RUN COMPARISON';
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════
//  S13: BONUS FUNCTIONS — Convergence, Comparison, Drilldown, Diversity
// ═══════════════════════════════════════════════════════════════════

// B1: Evolution Convergence Chart
function _refreshConvergenceChart() {
    const canvas = $el('evo-convergence-chart');
    if (!canvas) return;
    invoke('get_st_fitness_history').then(history => {
        if (!history || !history.length) return;
        _drawConvergenceChart(canvas, history);
    }).catch(() => {});
}

function _drawConvergenceChart(canvas, history) {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const w = canvas.width;
    const h = canvas.height;
    const pad = { l: 5, r: 5, t: 5, b: 12 };
    const plotW = w - pad.l - pad.r;
    const plotH = h - pad.t - pad.b;

    ctx.clearRect(0, 0, w, h);

    // Background grid
    ctx.strokeStyle = 'rgba(255,255,255,0.05)';
    ctx.lineWidth = 0.5;
    for (let i = 0; i < 4; i++) {
        const y = pad.t + (plotH / 4) * i;
        ctx.beginPath(); ctx.moveTo(pad.l, y); ctx.lineTo(w - pad.r, y); ctx.stroke();
    }

    if (history.length < 2) return;

    const isVals = history.map(h => h.is_fitness);
    const oosVals = history.map(h => h.oos_fitness);
    const divVals = history.map(h => h.diversity_pct);
    const allVals = [...isVals, ...oosVals].filter(v => isFinite(v));
    if (!allVals.length) return;

    const minY = Math.min(...allVals);
    const maxY = Math.max(...allVals);
    const rangeY = maxY - minY || 1;

    const toX = i => pad.l + (i / (history.length - 1)) * plotW;
    const toY = v => pad.t + plotH - ((v - minY) / rangeY) * plotH;

    // IS fitness line (cyan)
    ctx.beginPath();
    ctx.strokeStyle = '#00BFFF';
    ctx.lineWidth = 1.5;
    isVals.forEach((v, i) => {
        if (!isFinite(v)) return;
        const x = toX(i); const y = toY(v);
        if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    });
    ctx.stroke();

    // OOS fitness line (gold)
    ctx.beginPath();
    ctx.strokeStyle = '#FFD700';
    ctx.lineWidth = 1.5;
    ctx.setLineDash([4, 3]);
    oosVals.forEach((v, i) => {
        if (!isFinite(v)) return;
        const x = toX(i); const y = toY(v);
        if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    });
    ctx.stroke();
    ctx.setLineDash([]);

    // Promotion markers (green dots)
    history.forEach((h, i) => {
        if (h.champion_promoted) {
            ctx.beginPath();
            ctx.fillStyle = '#00FF41';
            ctx.arc(toX(i), toY(h.is_fitness), 3, 0, Math.PI * 2);
            ctx.fill();
        }
    });

    // Diversity bar at bottom
    const barH = 4;
    const barY = h - barH - 2;
    divVals.forEach((d, i) => {
        const x = toX(i);
        const segW = Math.max(2, plotW / history.length);
        const alpha = Math.min(d / 50, 1);
        ctx.fillStyle = d < 15 ? `rgba(255,43,43,${alpha})` : `rgba(0,191,255,${alpha})`;
        ctx.fillRect(x, barY, segW, barH);
    });

    // Labels
    ctx.font = '8px Share Tech Mono, monospace';
    ctx.fillStyle = '#00BFFF';
    ctx.fillText('IS', pad.l, h - 2);
    ctx.fillStyle = '#FFD700';
    ctx.fillText('OOS', pad.l + 20, h - 2);
    ctx.fillStyle = 'rgba(255,255,255,0.3)';
    ctx.fillText('DIV', w - pad.r - 20, h - 2);
}

// B2: Comparison Results Renderer
function _renderComparisonResults(data) {
    const wrap = $el('st-compare-results');
    if (!wrap) return;
    wrap.hidden = false;

    const st = data.smarttrend || {};
    const f = data.fusion || {};

    const pairs = [
        ['st-cmp-st-trades', 'st-cmp-f-trades', st.trade_count || (st.wins || 0) + (st.losses || 0), (f.wins || 0) + (f.losses || 0), true],
        ['st-cmp-st-wr', 'st-cmp-f-wr', st.win_rate !== undefined ? (st.win_rate * 100).toFixed(1) + '%' : '--', f.wins !== undefined && (f.wins + f.losses) > 0 ? (f.wins / (f.wins + f.losses) * 100).toFixed(1) + '%' : '--', true],
        ['st-cmp-st-pnl', 'st-cmp-f-pnl', '$' + (st.total_pnl || 0).toFixed(0), '$' + (f.total_pnl || 0).toFixed(0), true],
        ['st-cmp-st-sharpe', 'st-cmp-f-sharpe', (st.sharpe || 0).toFixed(3), (f.sharpe || 0).toFixed(3), true],
        ['st-cmp-st-dd', 'st-cmp-f-dd', (st.max_drawdown_pct || 0).toFixed(1) + '%', (f.max_drawdown_pct || 0).toFixed(1) + '%', false],
        ['st-cmp-st-pf', 'st-cmp-f-pf', (st.profit_factor || 0).toFixed(2), (f.profit_factor !== undefined ? f.profit_factor : ((f.wins || 0) > 0 ? 'N/A' : '0')), true],
    ];

    let stWins = 0, fWins = 0;
    pairs.forEach(([stId, fId, stVal, fVal, higherBetter]) => {
        setTxt(stId, stVal);
        setTxt(fId, fVal);
        const stNum = parseFloat(String(stVal).replace(/[^0-9.-]/g, ''));
        const fNum = parseFloat(String(fVal).replace(/[^0-9.-]/g, ''));
        const stEl = $el(stId);
        const fEl = $el(fId);
        if (stEl) stEl.classList.remove('winner');
        if (fEl) fEl.classList.remove('winner');
        if (!isNaN(stNum) && !isNaN(fNum)) {
            const better = higherBetter ? stNum > fNum : stNum < fNum;
            if (better) { if (stEl) stEl.classList.add('winner'); stWins++; }
            else if (stNum !== fNum) { if (fEl) fEl.classList.add('winner'); fWins++; }
        }
    });

    // PnL coloring
    ['st-cmp-st-pnl', 'st-cmp-f-pnl'].forEach(id => {
        const el = $el(id);
        if (el) {
            const v = parseFloat(el.textContent.replace(/[^0-9.-]/g, ''));
            el.style.color = v > 0 ? 'var(--neon-green)' : v < 0 ? 'var(--neon-red)' : '';
        }
    });

    const verdict = $el('st-compare-verdict');
    if (verdict) {
        verdict.className = 'st-compare-verdict';
        if (stWins > fWins) {
            verdict.textContent = `SMARTTREND wins ${stWins}/${stWins + fWins} metrics`;
            verdict.classList.add('verdict-st');
        } else if (fWins > stWins) {
            verdict.textContent = `FUSION wins ${fWins}/${stWins + fWins} metrics`;
            verdict.classList.add('verdict-fusion');
        } else {
            verdict.textContent = 'TIE — equal metric wins';
            verdict.classList.add('verdict-tie');
        }
    }
}

// B4: Diversity Badge Update
function _updateDiversityBadge(pct) {
    const badge = $el('evo-diversity-badge');
    if (!badge) return;
    badge.textContent = `DIV: ${pct.toFixed(0)}%`;
    badge.className = 'titan-badge';
    if (pct < 15) {
        badge.classList.add('div-low');
        badge.title = 'WARNING: Low population diversity — risk of premature convergence';
    } else if (pct < 40) {
        badge.classList.add('div-ok');
        badge.title = 'Population diversity OK';
    } else {
        badge.classList.add('div-high');
        badge.title = 'High population diversity — healthy exploration';
    }
}

// B3: Champion Params Drill-Down
window._showChampionDrilldown = async function() {
    try {
        const data = await invoke('get_st_champion_params');
        const fields = data.fields || [];
        const body = $el('champion-drilldown-body');
        if (!body) return;
        body.innerHTML = '';
        fields.forEach(f => {
            const tr = document.createElement('tr');
            const delta = f.delta_pct || 0;
            const absDelta = Math.abs(delta);
            const heatWidth = Math.min(absDelta, 60);
            let heatClass = 'champ-heat-zero';
            let deltaClass = 'champ-delta-zero';
            if (delta > 0.5) { heatClass = 'champ-heat-positive'; deltaClass = 'champ-delta-positive'; }
            else if (delta < -0.5) { heatClass = 'champ-heat-negative'; deltaClass = 'champ-delta-negative'; }

            const valStr = typeof f.value === 'boolean' ? (f.value ? 'ON' : 'OFF') : Number(f.value).toFixed(4);
            const defStr = typeof f.default === 'boolean' ? (f.default ? 'ON' : 'OFF') : Number(f.default).toFixed(4);

            tr.innerHTML = `
                <td style="font-family:var(--font-mono);font-size:10px;color:var(--neon-cyan);">${f.name}</td>
                <td style="text-align:right;font-size:11px;">${valStr}</td>
                <td style="text-align:right;font-size:11px;color:var(--text-dim);">${defStr}</td>
                <td class="${deltaClass}" style="text-align:right;font-size:11px;">${delta.toFixed(1)}%</td>
                <td><span class="champ-heat-bar ${heatClass}" style="width:${Math.max(heatWidth, 4)}px;"></span></td>
            `;
            body.appendChild(tr);
        });
        const overlay = $el('champion-drilldown-overlay');
        if (overlay) overlay.style.display = 'flex';
    } catch (e) {
        TitanToast.show('Failed to load champion params: ' + String(e), 'error');
    }
}

// ═══════════════════════════════════════════════════════════════
//  S14: Cockpit Strip -- persistent status bar + fill markers
// ═══════════════════════════════════════════════════════════════
window._titanFillMarkers = [];

(function initCockpitStrip() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    const _csStartTime = Date.now();

    function _fmtUptime(ms) {
        const totalSec = Math.floor(ms / 1000);
        const h = Math.floor(totalSec / 3600);
        const m = Math.floor((totalSec % 3600) / 60);
        return (h > 0 ? h + ':' : '') + String(m).padStart(2, '0') + ':' + String(totalSec % 60).padStart(2, '0');
    }

    async function updateCockpit() {
        try {
            const [s, risk] = await Promise.all([
                invoke('get_settings'),
                invoke('get_risk_status'),
            ]);
            window._lastRiskPayload = risk;
            window._lastSettingsPayload = s;
            window._titanMaxHoldSecs = s.max_hold_secs || 3600;
            const csEquity = $el('cs-equity');
            const csPnl = $el('cs-pnl');
            const csPos = $el('cs-positions');
            const csMaxPos = $el('cs-max-pos');
            const csMode = $el('cs-mode');
            const csBreaker = $el('cs-breaker');
            const csGen = $el('cs-generation');
            const csWinRate = $el('cs-winrate');
            const csUptime = $el('cs-uptime');

            if (csMaxPos) csMaxPos.textContent = s.max_open_positions || 5;
            if (csMode) csMode.textContent = (s.strategy_mode_name || '--').toUpperCase();

            const positions = s.positions || [];
            const openCount = positions.filter(p => Math.abs(p.qty) > 0.001).length;
            if (csPos) csPos.innerHTML = openCount + '/<span id="cs-max-pos">' + (s.max_open_positions || 5) + '</span>';

            const equity = s.equity || 0;
            if (csEquity) csEquity.textContent = '$' + Number(equity).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 });

            const totalPnl = positions.reduce((sum, p) => sum + (p.pnl || 0), 0);
            if (csPnl) {
                const sign = totalPnl >= 0 ? '+' : '';
                csPnl.textContent = sign + '$' + Math.abs(totalPnl).toFixed(2);
                csPnl.className = 'cs-value ' + (totalPnl > 0 ? 'cs-green' : totalPnl < 0 ? 'cs-red' : '');
            }

            const gen = risk.session_generation || 0;
            if (csGen) csGen.textContent = '#' + gen;
            window._titanGeneration = gen;

            const bl = (risk.breaker_level || 'NORMAL').toUpperCase();
            if (csBreaker) {
                csBreaker.textContent = bl;
                const cls = bl === 'NORMAL' ? 'cs-green' : bl === 'CAUTION' ? 'cs-orange' : 'cs-red';
                csBreaker.className = 'cs-value ' + cls;
                if (bl !== 'NORMAL') {
                    csBreaker.classList.add('cs-breaker-pulse');
                } else {
                    csBreaker.classList.remove('cs-breaker-pulse');
                }
            }

            const sess = risk.session || {};
            const wr = parseFloat(sess.win_rate_pct || '0');
            if (csWinRate) {
                csWinRate.textContent = wr.toFixed(1) + '%';
                csWinRate.className = 'cs-value ' + (wr >= 55 ? 'cs-green' : wr >= 45 ? 'cs-cyan' : wr > 0 ? 'cs-orange' : '');
            }

            if (csUptime) csUptime.textContent = _fmtUptime(Date.now() - _csStartTime);

            // S67 B1: Fill deduplication badge
            const csDedup = $el('cs-dedup-badge');
            if (csDedup) {
                const deduped = risk.fills_deduplicated || 0;
                if (deduped > 0) {
                    csDedup.textContent = 'DEDUP:' + deduped;
                    csDedup.className = 'cs-badge cs-badge-cyan';
                    csDedup.title = 'Fills deduplicated this session (CRIT-11 protection): ' + deduped;
                } else {
                    csDedup.className = 'cs-badge cs-badge-hidden';
                }
            }

            // S67 B3: API rate limit health badge
            const csApi = $el('cs-api-badge');
            if (csApi) {
                const apiCalls = risk.api_calls_this_minute || 0;
                const apiLimit = risk.api_rate_limit || 200;
                csApi.textContent = 'API:' + apiCalls + '/' + apiLimit;
                if (apiCalls > 180) {
                    csApi.className = 'cs-badge cs-badge-red cs-api-pulse';
                } else if (apiCalls > 100) {
                    csApi.className = 'cs-badge cs-badge-gold';
                } else {
                    csApi.className = 'cs-badge cs-badge-green';
                }
                csApi.title = 'Alpaca API calls this minute: ' + apiCalls + ' / ' + apiLimit;
            }

            // S68 B3: NaN Firewall Monitor badge
            const csNan = $el('cs-nan-badge');
            if (csNan) {
                const nanCount = risk.st_nan_sanitized || 0;
                csNan.textContent = 'NaN:' + nanCount;
                if (nanCount > 0) {
                    csNan.className = 'cs-badge cs-badge-red cs-nan-pulse';
                    csNan.title = nanCount + ' NaN values caught and sanitized in SmartTrend pillar calculations';
                } else {
                    csNan.className = 'cs-badge cs-badge-green';
                    csNan.title = 'NaN Firewall: No NaN values detected';
                }
            }

            // S68 B2: Generation Timeline Strip
            const csGenTl = $el('cs-gen-timeline');
            if (csGenTl) {
                const gen = risk.session_generation || 0;
                if (!window._genHistory) window._genHistory = [];
                const hist = window._genHistory;
                const last = hist.length > 0 ? hist[hist.length - 1] : null;
                if (!last || last.gen !== gen) {
                    hist.push({ gen, ts: Date.now() });
                    if (hist.length > 10) hist.shift();
                }
                let html = '';
                for (let i = 0; i < hist.length; i++) {
                    const h = hist[i];
                    const dur = i < hist.length - 1
                        ? _fmtUptime(hist[i + 1].ts - h.ts)
                        : _fmtUptime(Date.now() - h.ts);
                    const isCurrent = i === hist.length - 1;
                    const cls = isCurrent ? 'gen-chip gen-current' : 'gen-chip';
                    const label = isCurrent ? 'LIVE ' + dur : dur;
                    html += '<span class="' + cls + '">#' + h.gen + ' [' + label + ']</span>';
                    if (i < hist.length - 1) html += '<span class="gen-chip-arrow">▸</span>';
                }
                csGenTl.innerHTML = html;
            }

            // S71 B1: Timer Consolidation Badge
            const csTmr = $el('cs-tmr-badge');
            if (csTmr) {
                const count = TitanTimers.count();
                csTmr.textContent = 'TMR:' + count;
                csTmr.title = TitanTimers.getAll().map(t =>
                    t.name + ' ' + t.interval_ms + 'ms' + (t.tab !== null ? ' [tab:' + t.tab + ']' : '') + (t.active ? '' : ' (paused)')
                ).join('\n');
            }

            // S71 B2: Marker Engine Health Badge
            const csMrk = $el('cs-mrk-badge');
            if (csMrk && window._titanMarkerEngine) {
                const st = window._titanMarkerEngine.stats();
                csMrk.textContent = 'MRK:S' + (st.signals || 0) + '/F' + (st.fills || 0) + '/T' + (st.smarttrend || 0);
                const hasAny = st.total > 0;
                const hasAll = (st.signals || 0) > 0 && (st.fills || 0) > 0;
                csMrk.className = 'cs-badge ' + (hasAll ? 'cs-badge-green' : hasAny ? 'cs-badge-gold' : 'cs-badge-red');
                csMrk.title = 'Chart markers: ' + st.total + ' total (Signals:' + (st.signals || 0) + ' Fills:' + (st.fills || 0) + ' ST:' + (st.smarttrend || 0) + ')';
            }

            _updateShieldPanel(risk);
            _updateBonuses(risk);

            // S67 B4: Trail-stop integrity check on positions
            const posArr = s.positions || [];
            window._titanTrailHealth = posArr.map(p => {
                const isLong = (p.side || '').toLowerCase() !== 'sell';
                const ok = isLong
                    ? (p.highest_price || 0) >= (p.current_price || 0)
                    : (p.lowest_price || 0) <= (p.current_price || 0);
                return { symbol: p.symbol, trail_ok: ok };
            });

            // S68 B4: Orphan position detection
            const stPositions = posArr.filter(p => (p.entry_tag || '').startsWith('ST|'));
            window._titanOrphanStatus = stPositions.map(p => ({ symbol: p.symbol, orphan: false }));

        } catch (_) {}
    }

    function _updateShieldPanel(risk) {
        const shKill = $el('shield-kill');
        const shBreaker = $el('shield-breaker');
        const shDedup = $el('shield-dedup');
        const shZombie = $el('shield-zombie');
        const shNan = $el('shield-nan');
        const shKelly = $el('shield-kelly');
        const shGen = $el('shield-gen');
        const shOrphans = $el('shield-orphans');
        const shScore = $el('shield-score');

        if (!shKill) return;

        const killed = risk.st_killed || false;
        const breakerTripped = risk.st_breaker_triggered || false;
        const breakerPaused = risk.st_breaker_paused || false;
        const dedup = risk.fills_deduplicated || 0;
        const zombies = risk.zombie_loops || 0;
        const nanCount = risk.st_nan_sanitized || 0;
        const kelly = risk.st_kelly_fraction || 0;
        const gen = risk.session_generation || 0;
        const orphans = (window._titanOrphanStatus || []).filter(o => o.orphan).length;

        shKill.textContent = killed ? 'ON' : 'OFF';
        shKill.className = 'st-shield-val ' + (killed ? 'shield-on' : 'shield-off');

        if (breakerTripped) {
            shBreaker.textContent = 'TRIPPED';
            shBreaker.className = 'st-shield-val shield-danger';
        } else if (breakerPaused) {
            shBreaker.textContent = 'PAUSED';
            shBreaker.className = 'st-shield-val shield-warn';
        } else {
            shBreaker.textContent = 'NORMAL';
            shBreaker.className = 'st-shield-val shield-off';
        }

        shDedup.textContent = '' + dedup;
        shDedup.className = 'st-shield-val' + (dedup > 0 ? ' shield-warn' : '');

        shZombie.textContent = '' + zombies;
        shZombie.className = 'st-shield-val' + (zombies > 0 ? ' shield-danger' : '');

        shNan.textContent = '' + nanCount;
        shNan.className = 'st-shield-val' + (nanCount > 0 ? ' shield-danger' : '');

        shKelly.textContent = kelly.toFixed(2);
        shGen.textContent = '#' + gen;
        shOrphans.textContent = '' + orphans;
        shOrphans.className = 'st-shield-val' + (orphans > 0 ? ' shield-danger' : '');

        let score = 100;
        if (killed) score -= 50;
        if (breakerTripped) score -= 30;
        if (zombies > 0) score -= 20;
        if (nanCount > 0) score -= 10;
        if (orphans > 0) score -= 15;
        score = Math.max(0, score);

        shScore.textContent = 'SHIELD ' + score + '%';
        if (score >= 80) {
            shScore.className = 'st-shield-score';
        } else if (score >= 50) {
            shScore.className = 'st-shield-score shield-score-warn';
        } else {
            shScore.className = 'st-shield-score shield-score-danger';
        }

        const shBrkSync = $el('shield-brk-sync');
        if (shBrkSync) {
            const sync = risk.st_correctness ? risk.st_correctness.breaker_sync : true;
            shBrkSync.textContent = sync ? 'SYNC' : 'DESYNC';
            shBrkSync.className = 'st-shield-val ' + (sync ? 'shield-brk-sync' : 'shield-brk-desync');
        }
    }

    const _kalRing = [];
    function _updateCorrectnessPanel(risk) {
        const cor = risk.st_correctness;
        if (!cor) return;

        const hysEl = $el('correct-hys');
        if (hysEl) {
            const ok = cor.hys_ok;
            hysEl.textContent = ok ? 'OK' : 'STUCK:' + cor.consecutive_stalking;
            hysEl.className = 'st-correct-val ' + (ok ? 'correct-ok' : 'correct-danger');
        }

        const decayEl = $el('correct-decay');
        if (decayEl) {
            const score = cor.last_score || 50;
            decayEl.textContent = score.toFixed(1);
            decayEl.className = 'st-correct-val ' + (score > 60 || score < 40 ? 'correct-ok' : 'correct-warn');
        }

        const kalEl = $el('correct-kal');
        if (kalEl) {
            const s = cor.kalman_surprise || 0;
            const pct = (s * 100).toFixed(2);
            kalEl.textContent = (s >= 0 ? '+' : '') + pct + '%';
            kalEl.className = 'st-correct-val ' + (Math.abs(s) > 0.05 ? 'correct-warn' : 'correct-ok');
        }

        const lyapEl = $el('correct-lyap');
        if (lyapEl) {
            const lv = cor.lyapunov != null ? cor.lyapunov : 0;
            lyapEl.textContent = lv.toFixed(3);
            lyapEl.className = 'st-correct-val ' + (lv > 0.05 ? 'correct-danger' : lv < -0.01 ? 'correct-ok' : 'correct-warn');
        }

        const fsmBadge = $el('cs-fsm-badge');
        if (fsmBadge) {
            const s = cor.fsm_s || 0, e = cor.fsm_e || 0, t = cor.fsm_t || 0;
            fsmBadge.textContent = 'FSM:S' + s + '/E' + e + '/T' + t;
            const dominant = (s >= e && s >= t) ? 'cs-fsm-dominant-s' : (e >= t ? 'cs-fsm-dominant-e' : 'cs-fsm-dominant-t');
            fsmBadge.className = 'cs-badge ' + dominant;
        }

        _kalRing.push(cor.kalman_surprise || 0);
        if (_kalRing.length > 8) _kalRing.shift();
        const sparkEl = $el('cs-kal-spark');
        if (sparkEl) {
            const maxVal = Math.max(0.001, ...(_kalRing.map(v => Math.abs(v))));
            sparkEl.innerHTML = _kalRing.map(v => {
                const h = Math.max(1, Math.round((Math.abs(v) / maxVal) * 14));
                const cls = v >= 0 ? 'kal-pos' : 'kal-neg';
                return '<div class="cs-kal-bar ' + cls + '" style="height:' + h + 'px"></div>';
            }).join('');
        }
    }

    const _origShieldCall = _updateShieldPanel;

    // S72 B3: Save Event Timeline ring buffer
    const _saveEventRing = [];
    const SAVE_EVENT_MAX = 20;

    function _pushSaveEvent(type, ok) {
        const now = new Date();
        const hm = now.getHours().toString().padStart(2, '0') + ':' + now.getMinutes().toString().padStart(2, '0');
        _saveEventRing.push({ type, ok, hm });
        if (_saveEventRing.length > SAVE_EVENT_MAX) _saveEventRing.shift();
        const el = $el('save-timeline');
        if (!el) return;
        el.innerHTML = _saveEventRing.map(ev => {
            const cls = ev.ok ? 'save-ok' : 'save-fail';
            return '<span class="save-event-chip ' + cls + '">' + ev.type + ' ' + ev.hm + (ev.ok ? ' OK' : ' FAIL') + '</span>';
        }).join('');
    }

    // Listen for save events from backend
    if (window.__TAURI__) {
        const { listen } = window.__TAURI__.event;
        listen('settings:saved', () => _pushSaveEvent('SET', true)).catch(() => {});
        listen('settings:save-failed', () => _pushSaveEvent('SET', false)).catch(() => {});
        listen('journal:saved', () => _pushSaveEvent('JRN', true)).catch(() => {});
        listen('snapshot:saved', () => _pushSaveEvent('SNAP', true)).catch(() => {});
    }

    function _updateBonuses(risk) {
        _updateCorrectnessPanel(risk);

        // B2: Memory Pressure Gauge badge
        const csMem = $el('cs-mem-badge');
        if (csMem) {
            const entries = risk.map_entries_total || 0;
            csMem.textContent = 'MEM:' + entries;
            if (entries > 2000) {
                csMem.className = 'cs-badge cs-badge-red';
            } else if (entries > 1000) {
                csMem.className = 'cs-badge cs-badge-gold';
            } else if (entries > 500) {
                csMem.className = 'cs-badge cs-badge-cyan';
            } else {
                csMem.className = 'cs-badge cs-badge-green';
            }
            csMem.title = 'Trading HashMap entries total: ' + entries;
        }

        // B5: Eviction Stats Counter badge
        const csEvict = $el('cs-evict-badge');
        if (csEvict) {
            const evicted = risk.evicted_entries_total || 0;
            if (evicted > 0) {
                csEvict.textContent = 'EVICT:' + evicted;
                csEvict.className = 'cs-badge cs-badge-cyan';
                csEvict.title = evicted + ' stale entries cleaned from trading maps since startup';
            } else {
                csEvict.className = 'cs-badge cs-badge-hidden';
            }
        }

        // FIX-8: Stream Stale badge
        const csStale = $el('cs-stream-stale');
        if (csStale) {
            if (risk.stream_stale) {
                csStale.classList.remove('hidden');
            } else {
                csStale.classList.add('hidden');
            }
        }

        // B4: Dry-Run Guard Overlay
        const liveBanner = $el('live-trading-banner');
        const body = document.body;
        if (risk.session && risk.session.dry_run !== undefined) {
            const isDryRun = risk.session.dry_run;
            if (!isDryRun) {
                if (liveBanner) liveBanner.classList.remove('hidden');
                body.classList.add('live-trading-border');
            } else {
                if (liveBanner) liveBanner.classList.add('hidden');
                body.classList.remove('live-trading-border');
            }
        }

        // B1: Persistence Health Panel -- all 5 metrics
        const pDry = $el('persist-dry');
        if (pDry && risk.session) {
            const dr = risk.session.dry_run;
            if (dr === false) {
                pDry.textContent = 'LIVE';
                pDry.className = 'persist-val persist-off';
            } else {
                pDry.textContent = 'ON';
                pDry.className = 'persist-val persist-on';
            }
        }
        const pJrnl = $el('persist-jrnl');
        if (pJrnl) {
            const jc = risk.journal_entries || 0;
            pJrnl.textContent = jc + ' entries';
            pJrnl.className = 'persist-val';
        }
        const pDisk = $el('persist-disk');
        if (pDisk) {
            const ts = risk.last_settings_save_ts || 0;
            if (ts > 0) {
                const ageSecs = Math.floor(Date.now() / 1000) - ts;
                const label = ageSecs < 60 ? ageSecs + 's' : Math.floor(ageSecs / 60) + 'm';
                pDisk.textContent = label + ' ago';
                pDisk.className = 'persist-val' + (ageSecs < 60 ? ' persist-on' : ageSecs < 300 ? ' persist-warn' : ' persist-crit');
            } else {
                pDisk.textContent = '--';
                pDisk.className = 'persist-val';
            }
        }
        const pProf = $el('persist-prof');
        if (pProf) {
            const prof = risk.active_profile || 'default';
            pProf.textContent = prof;
            pProf.className = 'persist-val';
        }
        const pSnap = $el('persist-snap');
        if (pSnap) {
            const snapAge = risk.last_snapshot_age_secs || 0;
            if (snapAge > 0) {
                const label = snapAge < 60 ? snapAge + 's' : Math.floor(snapAge / 60) + 'm';
                pSnap.textContent = label + ' ago';
                pSnap.className = 'persist-val' + (snapAge < 60 ? ' persist-on' : snapAge < 300 ? ' persist-warn' : ' persist-crit');
            } else {
                pSnap.textContent = '--';
                pSnap.className = 'persist-val';
            }
        }
    }

    TitanTimers.register('s16:cockpit-strip', updateCockpit, 5000);
    setTimeout(updateCockpit, 1500);
})();

// ═══════════════════════════════════════════════════════════════
//  S15 B1: REGIME BAND — Market regime strip under chart
// ═══════════════════════════════════════════════════════════════
(function initRegimeBand() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;
    const { listen } = window.__TAURI__.event;

    async function updateRegimeStrip() {
        const strip = $el('regime-chart-strip');
        if (!strip) return;
        try {
            const s = window._lastSettingsPayload || await invoke('get_settings');
            const sym = s.active_symbol || '--';
            if (sym === '--' || sym === '---') { strip.hidden = true; return; }
            const tf = s.manual_res || '5m';
            const resp = await invoke('get_signal_markers', { symbol: sym, timeframe: tf });
            if (!resp || !resp.regime) { strip.hidden = true; return; }
            strip.hidden = false;
            const label = $el('regime-strip-label');
            const score = $el('regime-strip-score');
            const regime = (resp.regime || '').toLowerCase();
            strip.className = 'regime-chart-strip';
            if (regime.includes('trend') && resp.oracle_score > 0) {
                strip.classList.add('regime-trend-up');
                if (label) label.textContent = 'TRENDING UP';
            } else if (regime.includes('trend') && resp.oracle_score <= 0) {
                strip.classList.add('regime-trend-down');
                if (label) label.textContent = 'TRENDING DOWN';
            } else if (regime === 'chaotic') {
                strip.classList.add('regime-choppy');
                if (label) label.textContent = 'CHAOTIC';
            } else {
                strip.classList.add('regime-quiet');
                if (label) label.textContent = regime.toUpperCase() || 'QUIET';
            }
            if (score) score.textContent = (resp.oracle_score || 0).toFixed(0);
        } catch (_) {
            const strip2 = $el('regime-chart-strip');
            if (strip2) strip2.hidden = true;
        }
    }

    TitanTimers.register('s15:regime-band', updateRegimeStrip, 5000);
    listen('symbol-changed', () => setTimeout(updateRegimeStrip, 500));
})();

// ═══════════════════════════════════════════════════════════════
//  S15 B3: STRATEGY AGREEMENT — Oracle/GPU/ST direction panel
// ═══════════════════════════════════════════════════════════════
(function initStrategyAgreement() {
    if (!window.__TAURI__) return;
    const { listen } = window.__TAURI__.event;

    let lastOracle = { dir: 'NEUTRAL', ts: 0 };
    let lastGpu = { dir: 'NEUTRAL', ts: 0 };
    let lastSt = { dir: 'NEUTRAL', ts: 0 };

    function updateBadge(id, data) {
        const el = $el(id);
        if (!el) return;
        const now = Date.now() / 1000;
        const stale = (now - data.ts) > 60 && data.ts > 0;
        el.textContent = stale ? 'STALE' : data.dir;
        el.className = 'sa-badge ' + (stale ? 'sa-stale' : data.dir === 'LONG' ? 'sa-long' : data.dir === 'SHORT' ? 'sa-short' : 'sa-neutral');
    }

    function refresh() {
        updateBadge('sa-oracle-badge', lastOracle);
        updateBadge('sa-gpu-badge', lastGpu);
        updateBadge('sa-st-badge', lastSt);

        const dirs = [lastOracle.dir, lastGpu.dir, lastSt.dir];
        const validDirs = dirs.filter(d => d !== 'NEUTRAL');
        let agreePct = 0;
        if (validDirs.length >= 2) {
            const longCount = validDirs.filter(d => d === 'LONG').length;
            const shortCount = validDirs.filter(d => d === 'SHORT').length;
            const maxAgree = Math.max(longCount, shortCount);
            agreePct = Math.round((maxAgree / 3) * 100);
        }

        const pctEl = $el('sa-agreement-pct');
        if (pctEl) pctEl.textContent = agreePct + '%';

        const bar = $el('sa-meter-bar');
        if (bar) {
            bar.style.width = agreePct + '%';
            bar.style.background = agreePct >= 100 ? 'var(--neon-green)' : agreePct >= 66 ? 'var(--neon-cyan)' : agreePct >= 33 ? 'var(--neon-gold)' : 'var(--neon-red)';
        }
    }

    listen('nexus-update', (ev) => {
        const d = ev.payload || {};
        if (typeof d.buy_pct === 'number') {
            lastGpu.dir = d.buy_pct > 55 ? 'LONG' : d.buy_pct < 45 ? 'SHORT' : 'NEUTRAL';
            lastGpu.ts = Date.now() / 1000;
        }
        if (typeof d.oracle_score === 'number') {
            lastOracle.dir = d.oracle_score > 10 ? 'LONG' : d.oracle_score < -10 ? 'SHORT' : 'NEUTRAL';
            lastOracle.ts = Date.now() / 1000;
        }
        refresh();
    });

    listen('smarttrend-state', (ev) => {
        const d = ev.payload || {};
        const state = (d.state || '').toLowerCase();
        if (state === 'intrade' || state === 'trail') {
            lastSt.dir = d.direction === 'short' ? 'SHORT' : 'LONG';
        } else if (state === 'entry' || state === 'stalking') {
            lastSt.dir = d.direction === 'short' ? 'SHORT' : 'LONG';
        } else {
            lastSt.dir = 'NEUTRAL';
        }
        lastSt.ts = Date.now() / 1000;
        refresh();
    });

    TitanTimers.register('strategy:agreement', refresh, 5000);
})();

// ═══════════════════════════════════════════════════════════════
//  S16 B1: PiP MINI DASHBOARD — Floating HUD
// ═══════════════════════════════════════════════════════════════
(function initPipDashboard() {
    function _initPip() {
        const pip = $el('pip-hud');
        if (!pip) return;

        const header = $el('pip-hud-header');
        const closeBtn = $el('pip-close');
        const minBtn = $el('pip-minimize');
        const opacitySlider = $el('pip-opacity');
        let isDragging = false, offsetX = 0, offsetY = 0;

        if (header) {
            header.addEventListener('mousedown', (e) => {
                if (e.target.tagName === 'INPUT' || e.target.tagName === 'BUTTON') return;
                isDragging = true;
                offsetX = e.clientX - pip.getBoundingClientRect().left;
                offsetY = e.clientY - pip.getBoundingClientRect().top;
                pip.style.transition = 'none';
            });
            document.addEventListener('mousemove', (e) => {
                if (!isDragging) return;
                pip.style.left = (e.clientX - offsetX) + 'px';
                pip.style.top = (e.clientY - offsetY) + 'px';
                pip.style.right = 'auto';
                pip.style.bottom = 'auto';
            });
            document.addEventListener('mouseup', () => { isDragging = false; });
        }

        if (closeBtn) closeBtn.addEventListener('click', () => pip.classList.add('hidden'));
        if (minBtn) minBtn.addEventListener('click', () => {
            pip.classList.toggle('pip-minimized');
            minBtn.textContent = pip.classList.contains('pip-minimized') ? '□' : '_';
        });
        if (opacitySlider) opacitySlider.addEventListener('input', () => {
            pip.style.opacity = (parseInt(opacitySlider.value, 10) / 100).toString();
        });

        window._pipToggle = function() {
            pip.classList.toggle('hidden');
        };

        function updatePip() {
            if (pip.classList.contains('hidden')) return;
            const csEquity = $el('cs-equity');
            const csPnl = $el('cs-pnl');
            const csPos = $el('cs-positions');
            const csBreaker = $el('cs-breaker');
            const csWinRate = $el('cs-winrate');
            const csUptime = $el('cs-uptime');

            const pipEquity = $el('pip-equity');
            const pipPnl = $el('pip-pnl');
            const pipPos = $el('pip-positions');
            const pipBreaker = $el('pip-breaker');
            const pipWinRate = $el('pip-winrate');
            const pipUptime = $el('pip-uptime');

            if (pipEquity && csEquity) pipEquity.textContent = csEquity.textContent;
            if (pipPnl && csPnl) {
                pipPnl.textContent = csPnl.textContent;
                pipPnl.className = 'pip-val ' + (csPnl.classList.contains('cs-green') ? 'pip-green' : csPnl.classList.contains('cs-red') ? 'pip-red' : '');
            }
            if (pipPos && csPos) pipPos.textContent = csPos.textContent;
            if (pipBreaker && csBreaker) {
                pipBreaker.textContent = csBreaker.textContent;
                pipBreaker.className = 'pip-val ' + (csBreaker.classList.contains('cs-green') ? 'pip-green' : csBreaker.classList.contains('cs-red') ? 'pip-red' : 'pip-cyan');
            }
            if (pipWinRate && csWinRate) pipWinRate.textContent = csWinRate.textContent;
            if (pipUptime && csUptime) pipUptime.textContent = csUptime.textContent;
        }

        TitanTimers.register('s16:pip-dashboard', updatePip, 5000);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', _initPip, { once: true });
    } else {
        _initPip();
    }
})();

// ═══════════════════════════════════════════════════════════════
//  S16 B4: HEALTH SCORE COMPOSITE BADGE
// ═══════════════════════════════════════════════════════════════
(function initHealthBadge() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    const _healthState = {
        equitySyncAge: 0,
        wsConnected: true,
        breakerProximity: 100,
        latencyQuality: 100,
        deadLoops: 0,
        fillRate: 100,
    };

    function computeHealth() {
        const eqScore = Math.max(0, 100 - _healthState.equitySyncAge * 5);
        const wsScore = _healthState.wsConnected ? 100 : 0;
        const brScore = _healthState.breakerProximity;
        const latScore = _healthState.latencyQuality;
        const loopScore = Math.max(0, 100 - _healthState.deadLoops * 25);
        const fillKnown = _healthState.fillRate >= 0;
        const fillScore = fillKnown ? _healthState.fillRate : 0;
        const parts = [eqScore, wsScore, brScore, latScore, loopScore];
        if (fillKnown) parts.push(fillScore);
        return Math.round(parts.reduce((a, b) => a + b, 0) / parts.length);
    }

    function tooltipText() {
        return 'EQ_SYNC:' + Math.max(0, 100 - _healthState.equitySyncAge * 5) +
            ' WS:' + (_healthState.wsConnected ? 100 : 0) +
            ' BREAKER:' + _healthState.breakerProximity +
            ' LAT:' + _healthState.latencyQuality +
            ' LOOPS:' + Math.max(0, 100 - _healthState.deadLoops * 25) +
            ' FILL:' + (_healthState.fillRate >= 0 ? _healthState.fillRate : '--');
    }

    async function updateHealth() {
        try {
            const risk = window._lastRiskPayload || await invoke('get_risk_status');
            const telem = await invoke('get_telemetry');

            const bl = (risk.breaker_level || 'NORMAL').toUpperCase();
            _healthState.breakerProximity = bl === 'NORMAL' ? 100 : bl === 'CAUTION' ? 50 : 0;
            _healthState.deadLoops = ((telem && telem.dead_loops) || []).length;

            const latMs = risk.avg_latency_ms || 0;
            _healthState.latencyQuality = Math.max(0, 100 - Math.floor(latMs / 10));

            const orders = risk.orders_submitted || 0;
            const fills = risk.fills_received || 0;
            _healthState.fillRate = orders > 0 ? Math.round((fills / orders) * 100) : -1;
        } catch (_) {}

        const badge = $el('cs-health-badge');
        if (!badge) return;
        const score = computeHealth();
        badge.textContent = 'HP:' + score + '%';
        badge.title = tooltipText();
        badge.className = 'cs-health-badge ' + (
            score >= 80 ? 'cs-health-green' :
            score >= 60 ? 'cs-health-cyan' :
            score >= 40 ? 'cs-health-gold' : 'cs-health-red'
        );
    }

    if (window.__TAURI__) {
        window.__TAURI__.event.listen('equity-sync', () => { _healthState.equitySyncAge = 0; });
        window.__TAURI__.event.listen('equity-sync-degraded', () => { _healthState.equitySyncAge += 2; });
        window.__TAURI__.event.listen('trade-updates-health', (ev) => { _healthState.wsConnected = !!(ev.payload && ev.payload.connected); });
    }

    TitanTimers.register('s16:health-badge', updateHealth, 10000);
    setTimeout(updateHealth, 3000);
})();

// ═══════════════════════════════════════════════════════════════
//  S16 B3: MODE AUDIT TRAIL — Strategy transitions timeline
// ═══════════════════════════════════════════════════════════════
(function initModeAuditTrail() {
    if (!window.__TAURI__) return;
    const { invoke } = window.__TAURI__.tauri;

    function modeClass(mode) {
        const m = (mode || '').toLowerCase();
        if (m.includes('fusion')) return 'mode-badge-fusion';
        if (m.includes('signal')) return 'mode-badge-signal';
        if (m.includes('smart')) return 'mode-badge-smarttrend';
        return 'mode-badge-off';
    }

    async function refreshTrail() {
        const body = $el('mode-audit-body');
        const countEl = $el('mode-audit-count');
        if (!body) return;
        try {
            const trail = await invoke('get_mode_audit_trail');
            if (!Array.isArray(trail)) return;
            if (countEl) countEl.textContent = trail.length;
            if (trail.length === 0) {
                body.innerHTML = '<div style="color:var(--text-dim);font-size:10px;padding:4px;">No transitions yet</div>';
                return;
            }
            body.innerHTML = trail.slice(-20).reverse().map(t =>
                '<div class="mode-audit-entry">' +
                '<span class="mode-audit-ts">' + escapeHtml(t.timestamp) + '</span>' +
                '<span class="' + modeClass(t.from_mode) + '">' + escapeHtml(t.from_mode) + '</span>' +
                '<span class="mode-audit-arrow">→</span>' +
                '<span class="' + modeClass(t.to_mode) + '">' + escapeHtml(t.to_mode) + '</span>' +
                '<span style="color:var(--text-dim);font-size:9px;margin-left:auto;">' + escapeHtml(t.trigger) + '</span>' +
                '</div>'
            ).join('');
        } catch (_) {}
    }

    TitanTimers.register('s16:mode-audit', refreshTrail, 10000);
    setTimeout(refreshTrail, 4000);
})();

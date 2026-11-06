// UI HANDLERS
// normalizeSymbol, isCryptoSymbol, formatNum, formatVolume -> titan_core.js

function computeVwap(data) {
    if (!data || data.length === 0) return null;
    let pv = 0;
    let v = 0;
    let typicalSum = 0;
    let count = 0;
    data.forEach(c => {
        const vol = Number(c.volume || 0);
        const typical = (Number(c.high) + Number(c.low) + Number(c.close)) / 3;
        if (!Number.isFinite(typical)) return;
        typicalSum += typical;
        count += 1;
        if (!Number.isFinite(vol) || vol <= 0) return;
        pv += typical * vol;
        v += vol;
    });
    if (v <= 0) {
        if (count <= 0) return null;
        return typicalSum / count;
    }
    return pv / v;
}

function calcVwapSeries(data) {
    if (!data || data.length === 0) return [];
    let cumPV = 0;
    let cumV = 0;
    let cumTypical = 0;
    let count = 0;
    return data.map(c => {
        const vol = Number(c.volume || 0);
        let typical = (Number(c.high) + Number(c.low) + Number(c.close)) / 3;
        if (!Number.isFinite(typical)) typical = Number(c.close || 0);
        if (!Number.isFinite(typical)) typical = 0;
        cumTypical += typical;
        count += 1;
        if (Number.isFinite(vol) && vol > 0) {
            cumPV += typical * vol;
            cumV += vol;
        }
        if (cumV > 0) return cumPV / cumV;
        return cumTypical / Math.max(1, count);
    });
}

function formatTimeShort(ts) {
    if (!ts) return "--";
    const d = new Date(ts * 1000);
    if (Number.isNaN(d.getTime())) return "--";
    return d.toISOString().slice(11, 19);
}

function setRightBarText(id, text) {
    const el = $el(id);
    if (el) el.innerText = text;
}

function updateRightBarHeader() {
    setRightBarText('rb-symbol', currentSymbol);
    setRightBarText('rb-tf', currentTF);
    const feed = isCryptoSymbol(currentSymbol) ? 'CRYPTO' : 'SIP';
    const feedEl = $el('rb-feed');
    if (feedEl) {
        feedEl.innerText = feed;
        feedEl.className = `rb-tag ${feed === 'SIP' ? 'rb-sip' : 'rb-crypto'}`;
    }
}

function updateRightBarFeedStatus(text, type = "warning") {
    const el = $el('rb-feed-status');
    if (!el) return;
    const feedClass = isCryptoSymbol(currentSymbol) ? 'rb-crypto' : 'rb-sip';
    let cls = 'rb-tag ';
    if (type === "success") cls += feedClass;
    else if (type === "error") cls += 'rb-error';
    else cls += 'rb-warn';
    el.className = cls;
    el.innerText = text;
}

function updateRightBarSessionStats(data) {
    if (!data || data.length === 0) {
        rightBarState.sessionHigh = null;
        rightBarState.sessionLow = null;
        setRightBarText('rb-session-high', '--');
        setRightBarText('rb-session-low', '--');
        setRightBarText('rb-range', '--');
        return;
    }
    let hi = data[0].high;
    let lo = data[0].low;
    data.forEach(d => {
        if (typeof d.high === 'number') hi = Math.max(hi, d.high);
        if (typeof d.low === 'number') lo = Math.min(lo, d.low);
    });
    rightBarState.sessionHigh = hi;
    rightBarState.sessionLow = lo;
    setRightBarText('rb-session-high', formatNum(hi));
    setRightBarText('rb-session-low', formatNum(lo));
    setRightBarText('rb-range', formatNum(hi - lo));
}

function updateRightBarQuote(q) {
    if (!q) return;
    if (q.bid === null) setRightBarText('rb-bid', '--');
    else if (typeof q.bid === 'number' && q.bid > 0) setRightBarText('rb-bid', formatNum(q.bid));
    if (q.ask === null) setRightBarText('rb-ask', '--');
    else if (typeof q.ask === 'number' && q.ask > 0) setRightBarText('rb-ask', formatNum(q.ask));
    if (q.bid_size === null) setRightBarText('rb-bid-size', '--');
    else if (typeof q.bid_size === 'number' && q.bid_size > 0) setRightBarText('rb-bid-size', formatVolume(q.bid_size));
    if (q.ask_size === null) setRightBarText('rb-ask-size', '--');
    else if (typeof q.ask_size === 'number' && q.ask_size > 0) setRightBarText('rb-ask-size', formatVolume(q.ask_size));
    if (q.spread === null) setRightBarText('rb-spread', '--');
    else if (typeof q.spread === 'number' && q.spread > 0) setRightBarText('rb-spread', formatNum(q.spread));
}

function updateRightBarLatency(ts) {
    const el = $el('rb-latency');
    if (!el) return;
    if (!ts) {
        el.innerText = '-- ms';
        el.className = 'rb-muted';
        return;
    }
    const ageMs = Math.max(0, Date.now() - (ts * 1000));
    el.innerText = `${Math.round(ageMs)} ms`;
    if (ageMs > 300000) {
        el.className = 'rb-warn';
    } else {
        el.className = 'rb-muted';
    }
}

function updateChartOhlc(bar) {
    const el = $el('chart-ohlc');
    if (!el || !bar) return;
    const open = bar.open ?? bar.close;
    const high = bar.high ?? bar.close;
    const low = bar.low ?? bar.close;
    const close = bar.close ?? open;
    const diff = close - open;
    const pct = open ? (diff / open) * 100 : 0;
    const timeStr = bar.time ? ` ${formatTimeShort(bar.time)}` : '';
    el.innerText = `O ${formatNum(open)} H ${formatNum(high)} L ${formatNum(low)} C ${formatNum(close)} ${diff >= 0 ? '+' : ''}${diff.toFixed(2)} (${pct.toFixed(2)}%)${timeStr}`;
    el.style.color = pnlColor(diff);
}

function updateRightBarFromBar(bar, opts = {}) {
    if (!bar) return;
    const price = (typeof opts.price === 'number') ? opts.price : bar.close;
    const open = bar.open ?? price;
    const high = bar.high ?? price;
    const low = bar.low ?? price;
    const close = price;

    setRightBarText('rb-open', formatNum(open));
    setRightBarText('rb-high', formatNum(high));
    setRightBarText('rb-low', formatNum(low));
    setRightBarText('rb-close', formatNum(close));
    setRightBarText('rb-prev-close', formatNum(rightBarState.prevClose));
    if (lastVwap === null || lastVwap === undefined) {
        lastVwap = computeVwap(window.lastCandleData || []);
    }
    setRightBarText('rb-vwap', formatNum(lastVwap));
    setRightBarText('rb-volume', formatVolume(bar.volume || 0));
    setRightBarText('rb-last-tick', formatTimeShort(opts.time || bar.time));

    const priceEl = $el('rb-last-price');
    if (priceEl) priceEl.innerText = "$" + formatNum(close);

    const base = opts.isCrosshair ? open : (rightBarState.prevClose ?? open);
    const diff = close - base;
    const pct = base ? (diff / base) * 100 : 0;
    const changeEl = $el('rb-change');
    if (changeEl) {
        changeEl.innerText = `${diff >= 0 ? '+' : ''}${diff.toFixed(2)} (${pct.toFixed(2)}%)`;
        changeEl.classList.remove('rb-pos', 'rb-neg', 'rb-muted');
        changeEl.classList.add(diff >= 0 ? 'rb-pos' : 'rb-neg');
    }

    if (rightBarState.sessionHigh !== null && rightBarState.sessionLow !== null) {
        setRightBarText('rb-session-high', formatNum(rightBarState.sessionHigh));
        setRightBarText('rb-session-low', formatNum(rightBarState.sessionLow));
        setRightBarText('rb-range', formatNum(rightBarState.sessionHigh - rightBarState.sessionLow));
    }

    updateChartOhlc({ open, high, low, close, time: opts.time || bar.time });
}

function setSymbol(sym, opts = {}) {
    const next = normalizeSymbol(sym);
    if (!next) return;
    if (next === currentSymbol && opts.force !== true) return;
    const prev = currentSymbol;
    currentSymbol = next;
    const chartSym = $el('chart-symbol');
    if (chartSym) chartSym.innerText = next;
    const qtPrEl = $el('qt-live-price');
    if (qtPrEl) qtPrEl.textContent = '$--';
    updateRightBarHeader();
    if (replayState && replayState.active) exitReplayMode();
    if (typeof loadData === "function") {
        loadData(currentSymbol);
    }
    TitanBus.emit('symbolChanged', { symbol: next, prev });
}

function setTF(tf, el, opts = {}) {
    // DEBUG ALERT
    // alert("DEBUG: SWITCHING TO " + tf); 
    if (!tf) return;
    const prevTF = currentTF;
    TitanDebug.log('tf', 'Switching TF to:', tf);
    currentTF = tf;
    window.currentTimeframe = currentTF;
    updateRightBarHeader();
    if (replayState && replayState.active) exitReplayMode();

    // UI Update
    document.querySelectorAll('.tf-btn[data-tf]').forEach(b => b.classList.remove('active'));
    if (el && el.classList) {
        el.classList.add('active');
    } else {
        const match = document.querySelector(`.tf-btn[data-tf="${tf}"]`);
        if (match) match.classList.add('active');
    }
    const tfEl = $el('analysis-timeframe');
    if (tfEl) tfEl.innerText = currentTF;
    const manualSel = $el('select-manual-res');
    if (manualSel) manualSel.value = currentTF;

    const shouldReload = opts.force === true || prevTF !== currentTF;
    if (shouldReload) {
        if (typeof loadData === "function") {
            loadData(currentSymbol);
        } else {
            showCyberToast("CRITICAL: loadData function not found!", 'error');
        }
    }
    saveChartSettings();
    if (prevTF !== currentTF) {
        TitanBus.emit('tfChanged', { tf: currentTF, prev: prevTF });
    }
}
window.setTF = setTF;
window.setSymbol = setSymbol;

function bindTimeframeButtons() {
    const buttons = document.querySelectorAll('.tf-btn[data-tf]');
    buttons.forEach(btn => {
        btn.removeAttribute('onclick');
        btn.addEventListener('click', () => {
            const tf = btn.getAttribute('data-tf');
            if (tf) setTF(tf, btn);
        });
    });
}

function _latencyColor(el, us) {
    el.classList.remove('lat-warn', 'lat-danger');
    if (us > 5000) el.classList.add('lat-danger');
    else if (us > 1000) el.classList.add('lat-warn');
}

function _formatLatency(us) {
    if (!us && us !== 0) return '--';
    if (us >= 1000000) return (us / 1000000).toFixed(1) + 's';
    if (us >= 1000) return (us / 1000).toFixed(1) + 'ms';
    return us + 'μs';
}

async function updateLatencyPanel() {
    try {
        const m = await invoke('get_latency_metrics');
        if (!m) return;
        const ids = [
            ['lat-order-p50', m.order_rtt_us?.p50],
            ['lat-order-p95', m.order_rtt_us?.p95],
            ['lat-order-p99', m.order_rtt_us?.p99],
            ['lat-gpu-p50', m.gpu_dispatch_us?.p50],
            ['lat-gpu-p95', m.gpu_dispatch_us?.p95],
            ['lat-gpu-p99', m.gpu_dispatch_us?.p99],
            ['lat-stream-p50', m.streamer_tick_us?.p50],
            ['lat-stream-p95', m.streamer_tick_us?.p95],
            ['lat-stream-p99', m.streamer_tick_us?.p99],
        ];
        for (const [id, val] of ids) {
            const el = $el(id);
            if (!el) continue;
            el.textContent = _formatLatency(val);
            _latencyColor(el, val);
        }
    } catch (_) {}
}

let _latencyInterval = null;
function startLatencyPolling() {
    if (_latencyInterval) return;
    updateLatencyPanel();
    _latencyInterval = TitanTimers.register('rightbar:latency', updateLatencyPanel, 5500);
}

// ═══ S15 B4 + S16: Latency Sparklines + Rolling Histogram ═══
const LAT_RING_SIZE = 60;
const LAT_HIST_RING_SIZE = 300;
const _latRings = { order: [], gpu: [], stream: [] };
const _latHistRing = [];

function _pushRing(ring, val) {
    ring.push(val);
    if (ring.length > LAT_RING_SIZE) ring.shift();
}

function _drawSparkline(canvasId, ring) {
    const canvas = $el(canvasId);
    if (!canvas || ring.length < 2) return;
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const w = 120, h = 28;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    ctx.scale(dpr, dpr);
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';

    ctx.clearRect(0, 0, w, h);
    const max = Math.max(...ring, 1);
    const step = w / (ring.length - 1);

    ctx.beginPath();
    ctx.strokeStyle = '#00BFFF';
    ctx.lineWidth = 1.2;
    for (let i = 0; i < ring.length; i++) {
        const x = i * step;
        const y = h - (ring[i] / max) * (h - 4) - 2;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }
    ctx.stroke();

    const last = ring[ring.length - 1];
    const lx = (ring.length - 1) * step;
    const ly = h - (last / max) * (h - 4) - 2;
    ctx.fillStyle = last > max * 0.8 ? '#FF2B2B' : last > max * 0.5 ? '#FFD700' : '#00FF41';
    ctx.beginPath();
    ctx.arc(lx, ly, 2, 0, Math.PI * 2);
    ctx.fill();
}

function _computeHistFromRing() {
    const b = [0, 0, 0, 0, 0];
    for (let i = 0; i < _latHistRing.length; i++) b[_latHistRing[i]]++;
    return b;
}

function _drawHistogram() {
    const canvas = $el('lat-histogram');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const w = 160, h = 40;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    ctx.scale(dpr, dpr);
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';

    ctx.fillStyle = '#080812';
    ctx.fillRect(0, 0, w, h);

    const hist = _computeHistFromRing();
    const max = Math.max(...hist, 1);
    const barW = w / 5 - 4;
    const colors = ['#00FF41', '#00BFFF', '#FFD700', '#FFA500', '#FF2B2B'];
    const labels = ['<1ms', '1-5', '5-20', '20-100', '>100'];

    for (let i = 0; i < 5; i++) {
        const barH = (hist[i] / max) * (h - 14);
        const x = i * (barW + 4) + 2;
        const y = h - 12 - barH;
        ctx.fillStyle = colors[i];
        ctx.fillRect(x, y, barW, barH);
        ctx.fillStyle = '#778899';
        ctx.font = '7px "Share Tech Mono"';
        ctx.textAlign = 'center';
        ctx.fillText(labels[i], x + barW / 2, h - 2);
    }

    ctx.fillStyle = 'rgba(119,136,153,0.5)';
    ctx.font = '7px "Share Tech Mono"';
    ctx.textAlign = 'right';
    ctx.fillText('5m', w - 2, 8);
}

function _classifyLatency(us) {
    const ms = us / 1000;
    if (ms < 1) return 0;
    if (ms < 5) return 1;
    if (ms < 20) return 2;
    if (ms < 100) return 3;
    return 4;
}

function _pushHistSample(bucket) {
    _latHistRing.push(bucket);
    if (_latHistRing.length > LAT_HIST_RING_SIZE) _latHistRing.shift();
}

const _origUpdateLatency = updateLatencyPanel;
updateLatencyPanel = async function() {
    await _origUpdateLatency();
    try {
        const m = await invoke('get_latency_metrics');
        if (!m) return;
        const orderP50 = m.order_rtt_us?.p50 || 0;
        const gpuP50 = m.gpu_dispatch_us?.p50 || 0;
        const streamP50 = m.streamer_tick_us?.p50 || 0;
        _pushRing(_latRings.order, orderP50);
        _pushRing(_latRings.gpu, gpuP50);
        _pushRing(_latRings.stream, streamP50);
        _drawSparkline('lat-spark-order', _latRings.order);
        _drawSparkline('lat-spark-gpu', _latRings.gpu);
        _drawSparkline('lat-spark-stream', _latRings.stream);

        _pushHistSample(_classifyLatency(orderP50));
        _pushHistSample(_classifyLatency(gpuP50));
        _pushHistSample(_classifyLatency(streamP50));
        _drawHistogram();
    } catch (_) {}
};

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', startLatencyPolling);
} else {
    startLatencyPolling();
}

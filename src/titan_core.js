// ═══════════════════════════════════════════════════════════════
//  TITAN CORE — Shared utilities used across all modules
// ═══════════════════════════════════════════════════════════════

const _cryptoBases = new Set(["BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "AVAX", "DOT", "LINK", "LTC", "BCH"]);

function escapeHtml(str) {
    if (!str) return '';
    return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#039;');
}

function normalizeSymbol(sym) {
    if (!sym) return '';
    let s = sym.toString().trim().toUpperCase();
    if (!s) return s;
    if (s.includes('/')) return s;
    if (s.endsWith('USD') && s.length > 3) {
        if (_cryptoBases.has(s.slice(0, -3))) return `${s.slice(0, -3)}/USD`;
    }
    return s;
}

function isCryptoSymbol(sym) {
    if (!sym) return false;
    const s = sym.toString().trim().toUpperCase();
    if (s.includes('/')) return true;
    if (s.endsWith('USD') && s.length > 3) return _cryptoBases.has(s.slice(0, -3));
    return false;
}

function formatNum(val, decimals = 2) {
    if (val === null || val === undefined || Number.isNaN(val)) return "--";
    return Number(val).toFixed(decimals);
}

// --- Shared formatting helpers (deduplicated) ---

function fmtPct(v, decimals = 2) {
    if (v == null || v === 0) return '-';
    return (v >= 0 ? '+' : '') + Number(v).toFixed(decimals) + '%';
}

function formatVolume(val, decimals = 1) {
    const n = Number(val);
    if (!Number.isFinite(n)) return "--";
    const abs = Math.abs(n);
    if (abs >= 1e9) return (n / 1e9).toFixed(decimals) + "B";
    if (abs >= 1e6) return (n / 1e6).toFixed(decimals) + "M";
    if (abs >= 1e3) return (n / 1e3).toFixed(decimals) + "K";
    return n.toFixed(0);
}

function formatDollarVolume(val) {
    const n = Number(val);
    if (!Number.isFinite(n) || n <= 0) return '-';
    if (n >= 1e9) return '$' + (n / 1e9).toFixed(1) + 'B';
    if (n >= 1e6) return '$' + (n / 1e6).toFixed(1) + 'M';
    if (n >= 1e3) return '$' + (n / 1e3).toFixed(0) + 'K';
    return '$' + n.toFixed(0);
}

function fmtUsd(n) { return '$' + (Number(n) || 0).toFixed(2); }
const formatMoney = fmtUsd;

function fmtScore(v) {
    return v != null && v !== 0 ? Number(v).toFixed(1) : '-';
}

// --- DOM helpers ---
// WeakRef cache: entries drop when nodes are GC'd; disconnected nodes are re-resolved. Null misses are never cached.
const _domWeakCache = typeof WeakRef !== 'undefined' ? Object.create(null) : null;

const _domDbg = {
    lookups: 0,
    cacheHits: 0,
    staleInvalidations: 0,
    domQueries: 0,
    domQueryMisses: 0,
    fallbackDirectCalls: 0,
};

const TitanDomCacheDebug = {
    snapshot() {
        return {
            lookups: _domDbg.lookups,
            cacheHits: _domDbg.cacheHits,
            staleInvalidations: _domDbg.staleInvalidations,
            domQueries: _domDbg.domQueries,
            domQueryMisses: _domDbg.domQueryMisses,
            fallbackDirectCalls: _domDbg.fallbackDirectCalls,
            weakRefEnabled: _domWeakCache != null,
        };
    },
    reset() {
        _domDbg.lookups = 0;
        _domDbg.cacheHits = 0;
        _domDbg.staleInvalidations = 0;
        _domDbg.domQueries = 0;
        _domDbg.domQueryMisses = 0;
        _domDbg.fallbackDirectCalls = 0;
    },
};
window.TitanDomCacheDebug = TitanDomCacheDebug;

function $el(id) {
    _domDbg.lookups++;
    if (_domWeakCache) {
        const wr = _domWeakCache[id];
        if (wr !== undefined) {
            const cached = wr.deref();
            if (cached != null && cached.isConnected) {
                _domDbg.cacheHits++;
                return cached;
            }
            delete _domWeakCache[id];
            _domDbg.staleInvalidations++;
        }
        _domDbg.domQueries++;
        const el = document.getElementById(id);
        if (!el) {
            _domDbg.domQueryMisses++;
        } else {
            _domWeakCache[id] = new WeakRef(el);
        }
        return el;
    }
    _domDbg.fallbackDirectCalls++;
    const el = document.getElementById(id);
    if (!el) {
        _domDbg.domQueryMisses++;
    }
    return el;
}

function setTxt(id, val, color) {
    const el = $el(id);
    if (!el) return;
    el.textContent = val;
    if (color) el.style.color = color;
}

// --- Color helpers ---

function pnlColor(v) {
    return v > 0 ? '#00BFFF' : v < 0 ? '#FF2B2B' : '#888';
}

function sideColor(side) {
    const s = (side || '').toUpperCase();
    return (s === 'BUY' || s === 'LONG') ? '#00BFFF' : '#FF2B2B';
}

function tfToSeconds(tf) {
    switch (tf) {
        case '1m': return 60;
        case '3m': return 180;
        case '5m': return 300;
        case '10m': return 600;
        case '15m': return 900;
        case '30m': return 1800;
        case '1h': return 3600;
        case '4h': return 14400;
        case '1d': return 86400;
        default: return 300;
    }
}

function formatCountdown(seconds) {
    const sec = Math.max(0, Math.floor(seconds));
    const hh = Math.floor(sec / 3600);
    const mm = Math.floor((sec % 3600) / 60);
    const ss = sec % 60;
    if (hh > 0) return `${hh}h ${mm}m ${ss}s`;
    if (mm > 0) return `${mm}m ${ss}s`;
    return `${ss}s`;
}

function getDrawingManager() {
    if (typeof drawingManager !== 'undefined' && drawingManager) return drawingManager;
    if (window.drawingManager) return window.drawingManager;
    return null;
}

// --- Shared renderers (OFI + Flow dedup) ---

function renderOfi(o) {
    if (!o) return;
    setTxt('ofi-value', (o.current_ofi || 0).toFixed(2));
    setTxt('ofi-spread', 'SPR: ' + (o.spread_bps || 0).toFixed(1) + ' bps');
    setTxt('ofi-bid-depth', formatVolume(o.bid_depth_total));
    setTxt('ofi-ask-depth', formatVolume(o.ask_depth_total));
    setTxt('ofi-imbalance', (o.imbalance_ratio || 0).toFixed(2));
    const bar = $el('ofi-bar');
    if (bar) {
        const ofi = o.current_ofi || 0;
        const pct = Math.min(Math.abs(ofi) * 50, 50);
        bar.style.width = pct + '%';
        bar.style.left = ofi >= 0 ? '50%' : (50 - pct) + '%';
        bar.style.background = pnlColor(ofi);
    }

    const mpEl = $el('micro-microprice');
    if (mpEl && o.microprice !== undefined && o.microprice > 0) {
        mpEl.innerText = '$' + formatNum(o.microprice);
    }

    const vpinEl = $el('micro-vpin');
    if (vpinEl && o.stream_vpin !== undefined) {
        const vpin = o.stream_vpin;
        vpinEl.innerText = (vpin * 100).toFixed(1) + '%';
        vpinEl.className = 'mp-val ' + (vpin > 0.6 ? 'val-red' : vpin > 0.35 ? 'val-gold' : 'val-green');
    }
}

function renderFlow(f) {
    if (!f) return;
    const buy = f.buy_notional ?? f.flow_buy ?? 0;
    const sell = f.sell_notional ?? f.flow_sell ?? 0;
    const fmtCash = (v) => '$' + v.toLocaleString('en-US', { maximumFractionDigits: 0 });
    const buyEl = $el('flow-buy');
    const sellEl = $el('flow-sell');
    const stateEl = $el('flow-state');
    const flowBox = $el('whale-flow-box');
    if (buyEl && buy > 0) { buyEl.innerText = fmtCash(buy); buyEl.className = 'wb-val green'; }
    if (sellEl && sell > 0) { sellEl.innerText = fmtCash(sell); sellEl.className = 'wb-val red'; }
    if (stateEl) {
        let state = f.flow_state || '';
        if (!state) {
            const bc = f.buy_count || 0;
            const sc = f.sell_count || 0;
            state = bc > sc ? 'BUY' : sc > bc ? 'SELL' : 'BALANCED';
        }
        const up = state.toUpperCase();
        if (up.includes('BUY')) {
            stateEl.innerText = 'WHALE BUY';
            stateEl.style.color = '#00BFFF';
            if (flowBox) { flowBox.classList.remove('red-border', 'neutral-border'); flowBox.classList.add('green-border'); }
        } else if (up.includes('SELL')) {
            stateEl.innerText = 'WHALE SELL';
            stateEl.style.color = '#FF2B2B';
            if (flowBox) { flowBox.classList.remove('green-border', 'neutral-border'); flowBox.classList.add('red-border'); }
        } else {
            stateEl.innerText = 'BALANCED';
            stateEl.style.color = '#aaa';
            if (flowBox) { flowBox.classList.remove('green-border', 'red-border'); flowBox.classList.add('neutral-border'); }
        }
    }
    _renderMicrostructureFlow(f);
}

function _renderMicrostructureFlow(f) {
    const cvdEl = $el('micro-cvd');
    if (cvdEl && f.cvd !== undefined) {
        const cvd = f.cvd;
        const absCvd = Math.abs(cvd);
        let formatted;
        if (absCvd >= 1e6) formatted = (cvd / 1e6).toFixed(1) + 'M';
        else if (absCvd >= 1e3) formatted = (cvd / 1e3).toFixed(1) + 'K';
        else formatted = cvd.toFixed(0);
        const arrow = cvd > 0 ? '\u25B2 ' : cvd < 0 ? '\u25BC ' : '';
        cvdEl.innerText = arrow + formatted;
        cvdEl.className = 'mp-val ' + (cvd > 0 ? 'val-green' : cvd < 0 ? 'val-red' : 'val-dim');
    }

    const badgeEl = $el('micro-class-badge');
    if (badgeEl && f.classified_pct !== undefined) {
        const pct = f.classified_pct;
        badgeEl.innerText = 'CLASS ' + pct.toFixed(0) + '%';
        if (pct >= 90) {
            badgeEl.className = 'class-badge class-badge-green';
        } else if (pct >= 70) {
            badgeEl.className = 'class-badge class-badge-gold';
        } else {
            badgeEl.className = 'class-badge class-badge-red';
        }
    }

    const barEl = $el('micro-intensity-bar');
    if (barEl && f.trade_intensity !== undefined) {
        const intensity = f.trade_intensity;
        const segs = barEl.querySelectorAll('.intensity-bar-seg');
        const thresholds = [0.5, 2, 5, 15, 40];
        segs.forEach((seg, i) => {
            seg.classList.remove('seg-active', 'seg-high', 'seg-extreme');
            if (intensity >= thresholds[i]) {
                if (i >= 4) seg.classList.add('seg-extreme');
                else if (i >= 3) seg.classList.add('seg-high');
                else seg.classList.add('seg-active');
            }
        });
    }
}

// --- Timer registry ---

const TitanTimers = {
    _timers: {},
    _tabStagger: {},
    register(name, fn, ms, opts) {
        if (this._timers[name]) this.clear(name);
        const tab = (opts && opts.tab !== undefined) ? opts.tab : null;
        const entry = { rawFn: fn, ms, tab, id: null };
        this._timers[name] = entry;
        if (tab === null) {
            entry.id = setInterval(fn, ms);
        } else {
            if (window._stCfgCurrentTab === tab) this._startTabTimer(name, entry);
        }
        return name;
    },
    _startTabTimer(name, entry) {
        if (entry.id) return;
        const stagger = this._tabStagger[entry.tab] || 0;
        this._tabStagger[entry.tab] = stagger + 300;
        setTimeout(() => {
            if (!this._timers[name]) return;
            entry.rawFn();
            entry.id = setInterval(entry.rawFn, entry.ms);
        }, stagger);
    },
    _stopTabTimer(entry) {
        if (entry.id) { clearInterval(entry.id); entry.id = null; }
    },
    onTabSwitch(newTab) {
        this._tabStagger = {};
        for (const [name, entry] of Object.entries(this._timers)) {
            if (entry.tab === null) continue;
            if (entry.tab === newTab) {
                this._startTabTimer(name, entry);
            } else {
                this._stopTabTimer(entry);
            }
        }
    },
    clear(name) {
        const e = this._timers[name];
        if (e) {
            if (e.id) clearInterval(e.id);
            delete this._timers[name];
        }
    },
    clearAll() {
        Object.keys(this._timers).forEach(k => this.clear(k));
    },
    list() { return Object.keys(this._timers); },
    getAll() {
        const result = [];
        for (const [name, entry] of Object.entries(this._timers)) {
            result.push({
                name,
                interval_ms: entry.ms,
                tab: entry.tab,
                active: entry.id !== null,
            });
        }
        return result;
    },
    count() { return Object.keys(this._timers).length; },
};
window.TitanTimers = TitanTimers;

// --- Duplicate Poll Detector (dev-only) ---
(function initDupPollDetector() {
    if (typeof window.__TAURI__ === 'undefined') return;
    if (!localStorage.getItem('titanDebug')) return;

    const _ring = [];
    const RING_CAP = 50;
    const DUP_WINDOW_MS = 1000;
    const _origInvoke = window.__TAURI__.tauri.invoke;

    window.__TAURI__.tauri.invoke = function(cmd) {
        const now = Date.now();
        _ring.push({ cmd, ts: now });
        if (_ring.length > RING_CAP) _ring.shift();

        const dups = _ring.filter(e => e.cmd === cmd && (now - e.ts) < DUP_WINDOW_MS);
        if (dups.length > 1) {
            console.warn('[TITAN] Duplicate poll detected:', cmd, 'x' + dups.length + ' in ' + DUP_WINDOW_MS + 'ms');
        }
        return _origInvoke.apply(this, arguments);
    };
})();

// --- Error boundary ---

function normalizeError(e) {
    if (typeof e === 'string') return e;
    if (e instanceof Error) return e.message;
    if (e && typeof e === 'object') {
        const vals = Object.values(e);
        if (vals.length === 1 && typeof vals[0] === 'string') return vals[0];
        try { return JSON.stringify(e); } catch { return '[unknown error]'; }
    }
    return String(e);
}
window.normalizeError = normalizeError;

function normalizeScanError(e) {
    const msg = normalizeError(e);
    if (/\[TIMEOUT\].*fetch_assets|fetch_assets_body: operation timed out|fetch_assets: operation timed out/i.test(msg)) {
        return 'Asset list request timed out. Retry in a few seconds while the app warms the scanner cache.';
    }
    if (/fetch_assets/i.test(msg)) {
        return 'Could not load Alpaca asset list. Check API keys, network, and Alpaca availability.';
    }
    return msg;
}
window.normalizeScanError = normalizeScanError;

const TitanError = {
    _log: [],
    _MAX: 50,
    report(ctx, err) {
        const entry = {
            ts: Date.now(),
            ctx,
            msg: normalizeError(err),
            stack: err instanceof Error ? err.stack : undefined,
        };
        this._log.push(entry);
        if (this._log.length > this._MAX) this._log.shift();
        console.error(`[TITAN ${ctx}]`, err);
        TitanBus.emit('titan:error', entry);
    },
    getLog() { return this._log; },
};
window.TitanError = TitanError;

const TitanDebug = {
    _enabled: false,
    log(tag, ...args) {
        if (this._enabled) console.log(`[TITAN:${tag}]`, ...args);
    },
};
window.TitanDebug = TitanDebug;

const _mockResponses = {
    get_settings_snapshot: {},
    get_replay_symbols: [],
    get_hft_targets: { targets: [], active_symbols: [] },
    get_hft_target_enriched: [],
    get_favorites: [],
    get_risk_status: {},
    get_monitor_snapshot: {},
    get_ofi_status: {},
    get_telemetry: {},
};

window.invoke = async (cmd, args) => {
    if (typeof cmd !== 'string') return;
    if (window.__TAURI__) {
        try {
            return await window.__TAURI__.tauri.invoke(cmd, args);
        } catch (e) {
            TitanError.report(`ipc:${cmd}`, e);
            throw e;
        }
    } else {
        console.warn(`INVOKE FALLBACK (No Tauri): ${cmd}`, args);
        if (cmd in _mockResponses) return _mockResponses[cmd];
        return "MOCK_RESPONSE";
    }
};

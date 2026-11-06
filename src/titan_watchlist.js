let _cachedFavorites = [];
let _wlSnapshotCache = [];

async function loadFavoritesFromBackend() {
    try {
        const result = await invoke('get_favorites');
        _cachedFavorites = Array.isArray(result) ? result : [];
    } catch (e) {
        _cachedFavorites = [];
    }
    renderWatchlistTable(_cachedFavorites);
}

function renderFavorites(list) {
    renderWatchlistTable(list);
}

function renderWatchlistTable(list) {
    if (!Array.isArray(list)) {
        list = Array.isArray(_cachedFavorites) ? _cachedFavorites : [];
    }
    _cachedFavorites = list;

    const body = $el('wl-central-body');
    const emptyEl = $el('wl-central-empty');
    const countEl = $el('wl-central-count');
    const sbCount = $el('sb-wl-count');
    if (sbCount) sbCount.textContent = `(${list.length})`;
    if (countEl) countEl.innerText = `Symbols: ${list.length}`;

    if (!body) return;
    body.innerHTML = '';

    if (!list.length) {
        if (emptyEl) emptyEl.style.display = 'block';
        return;
    }
    if (emptyEl) emptyEl.style.display = 'none';

    const cacheMap = {};
    (_wlSnapshotCache || []).forEach(e => { cacheMap[e.symbol] = e; });
    const hasData = _wlSnapshotCache && _wlSnapshotCache.length > 0;

    // fmtPct, pnlColor -> titan_core.js

    body.innerHTML = list.map((sym, i) => {
        const d = cacheMap[sym] || {};
        const price = d.price || 0;
        const chg = d.change_percent || 0;
        const rvol = d.rvol || 0;
        const vol = d.volume || 0;
        const gap = d.gap_percent || 0;
        const score = d.score || 0;
        const floatCat = d.float_category || '';

        if (!hasData) {
            const es = escapeHtml(sym);
            return `<tr><td class="td-sym-gold" data-wl-sym="${es}">${es}</td><td colspan="9" style="text-align:center;color:#555;">Click REFRESH to load data</td><td style="text-align:center;">${_catalystBadgeHtml(sym)}</td><td style="text-align:center;"><span data-wl-trade="${es}" class="tbl-badge tbl-badge-gold">TRADE</span></td><td style="text-align:center;"><span data-wl-hft="${es}" class="tbl-badge tbl-badge-cyan">HFT</span></td><td style="text-align:center;"><span data-wl-ai="${es}" class="tbl-badge tbl-badge-cyan">AI</span></td><td style="text-align:center;"><span class="tbl-del" data-wl-del="${es}">×</span></td></tr>`;
        }

        const chgC = chg >= 10 ? '#00BFFF' : chg >= 0 ? '#FFD700' : '#ff4444';
        const rvC = rvol >= 5 ? '#00BFFF' : rvol >= 2 ? '#FFD700' : '#888';
        const gapC = gap > 3 ? '#00BFFF' : gap > 0 ? '#FFD700' : '#888';
        const scoreC = score >= 60 ? '#00BFFF' : score >= 30 ? '#FFD700' : '#888';
        const floatClr = floatCat === 'LOW' ? '#A855F7' : floatCat === 'MED' ? '#FFD700' : '#888';

        return `<tr>
            <td class="td-sym-gold" data-wl-sym="${escapeHtml(sym)}">${escapeHtml(sym)}</td>
            <td style="text-align:right;color:#ccc;">$${formatNum(price)}</td>
            <td style="text-align:right;color:${chgC};">${fmtPct(chg)}</td>
            <td style="text-align:right;color:${rvC};">${rvol ? rvol.toFixed(1) + 'x' : '-'}</td>
            <td style="text-align:right;color:#888;">${formatVolume(vol)}</td>
            <td style="text-align:right;color:${gapC};">${fmtPct(gap)}</td>
            <td style="text-align:center;color:${floatClr};">${floatCat || '—'}</td>
            <td style="text-align:right;color:#888;">${(d.market_cap_m || 0) > 0 ? ((d.market_cap_m >= 1000 ? (d.market_cap_m/1000).toFixed(1) + 'B' : d.market_cap_m.toFixed(0) + 'M')) : '—'}</td>
            <td style="text-align:center;color:${d.insider_signal === 'BUY' ? '#00BFFF' : d.insider_signal === 'SELL' ? '#ff4444' : '#555'};">${d.insider_signal || '—'}</td>
            <td style="text-align:right;color:${scoreC};">${score ? formatNum(score) : '-'}</td>
            <td style="text-align:center;">${_catalystBadgeHtml(sym)}</td>
            <td style="text-align:center;"><span data-wl-trade="${escapeHtml(sym)}" class="tbl-badge tbl-badge-gold">TRADE</span></td>
            <td style="text-align:center;"><span data-wl-hft="${escapeHtml(sym)}" class="tbl-badge tbl-badge-cyan">HFT</span></td>
            <td style="text-align:center;"><span data-wl-ai="${escapeHtml(sym)}" class="tbl-badge tbl-badge-cyan">AI</span></td>
            <td style="text-align:center;"><span class="tbl-del" data-wl-del="${escapeHtml(sym)}">×</span></td>
        </tr>`;
    }).join('');

    body.querySelectorAll('[data-wl-sym]').forEach(el => {
        el.addEventListener('click', () => {
            const sym = el.getAttribute('data-wl-sym');
            setSymbol(sym, { force: true });
            invoke('set_active_symbol', { symbol: sym }).catch(e => TitanError.report('wl:set-symbol', e));
        });
    });
    body.querySelectorAll('[data-wl-del]').forEach(el => {
        el.addEventListener('click', async () => {
            const sym = el.getAttribute('data-wl-del');
            try {
                const updated = await invoke('remove_favorite', { symbol: sym });
                _wlSnapshotCache = _wlSnapshotCache.filter(x => x.symbol !== sym);
                renderWatchlistTable(updated);
            } catch (e) { TitanError.report('wl:remove-fav', e); }
        });
    });
    body.querySelectorAll('[data-wl-trade]').forEach(el => {
        el.addEventListener('click', (e) => {
            e.stopPropagation();
            const sym = el.getAttribute('data-wl-trade');
            _injectSymbolToManualTrade(sym);
        });
    });
    body.querySelectorAll('[data-wl-hft]').forEach(el => {
        el.addEventListener('click', async (e) => {
            e.stopPropagation();
            const sym = el.getAttribute('data-wl-hft');
            el.textContent = '✓';
            el.style.color = '#00BFFF';
            try {
                await invoke('add_hft_target', { symbol: sym });
                refreshHftTargets();
                if (typeof refreshHftEnriched === 'function') refreshHftEnriched();
            } catch (err) {
                el.textContent = '!';
                el.style.color = '#ff4444';
            }
        });
    });
    body.querySelectorAll('[data-wl-ai]').forEach(el => {
        el.addEventListener('click', async (e) => {
            e.stopPropagation();
            const sym = el.getAttribute('data-wl-ai');
            el.textContent = '...';
            el.style.color = '#00BFFF';
            try {
                await refreshWlEnriched([sym]);
                el.textContent = '✓';
            } catch (err) {
                el.textContent = '!';
                el.style.color = '#ff4444';
            }
            setTimeout(() => { el.textContent = 'AI'; el.style.color = '#00BFFF'; }, 2000);
        });
    });
}

function _injectSymbolToManualTrade(sym) {
    const mtInput = $el('mt-symbol');
    if (mtInput) mtInput.value = sym;
    const panel = $el('manual-trade-panel');
    if (panel) {
        panel.open = true;
        panel.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
}

let _wlEnrichedCache = [];
let _wlEnrichRunning = false;

function renderWlAiTable(data) {
    const body = $el('wl-ai-body');
    const container = $el('wl-ai-results');
    const statusEl = $el('wl-ai-status');
    if (!body) return;

    if (!data || data.length === 0) {
        body.innerHTML = '<tr><td colspan="25" style="text-align:center;color:#555;padding:20px;">No AI results yet. Click BACKTEST ALL or per-row AI.</td></tr>';
        return;
    }
    if (container) container.style.display = 'block';
    if (statusEl) statusEl.textContent = `${data.length} results`;

    // fmtPct, fmtScore -> titan_core.js

    body.innerHTML = data.map((c, i) => {
        const bg = i % 2 === 0 ? '#0a0a1a' : '#06060f';
        const chgC = c.change_percent >= 10 ? '#00BFFF' : c.change_percent >= 0 ? '#FFD700' : '#ff4444';
        const rvC = c.rvol >= 5 ? '#00BFFF' : c.rvol >= 2 ? '#FFD700' : '#888';
        const gapC = c.gap_percent > 3 ? '#00BFFF' : c.gap_percent > 0 ? '#FFD700' : '#888';
        const floatC = c.float_category === 'LOW' ? '#FF69B4' : c.float_category === 'MED' ? '#FFD700' : '#888';
        const oracleC = c.oracle_score >= 70 ? '#00BFFF' : c.oracle_score >= 40 ? '#FFD700' : '#888';
        const regimeC = c.oracle_regime === 'BULL' || c.oracle_regime === 'Bull' ? '#00BFFF' : c.oracle_regime === 'BEAR' || c.oracle_regime === 'Bear' ? '#ff4444' : '#FFD700';
        const nexusC = c.nexus_buy_pct >= 60 ? '#FF69B4' : '#888';
        const fusionC = c.fusion_score >= 7 ? '#FFD700' : '#888';
        const sigC = c.fusion_signal === 'LONG' || c.fusion_signal === 'BUY' ? '#00BFFF' : c.fusion_signal === 'SHORT' || c.fusion_signal === 'SELL' ? '#ff4444' : '#FFD700';
        const winC = c.bt_win_rate >= 60 ? '#00BFFF' : c.bt_win_rate >= 45 ? '#FFD700' : '#ff4444';
        const retC = pnlColor(c.bt_return_pct);
        const aiC = c.ai_score >= 70 ? '#FFD700' : '#888';
        const smartC = c.smart_score >= 70 ? '#00BFFF' : '#888';

        return `<tr style="background:${bg}; border-bottom:1px solid #0a0a2a;">
            <td style="padding:4px;text-align:center;color:#444;">${c.rank || i + 1}</td>
            <td style="padding:4px;color:#00BFFF;cursor:pointer;font-weight:700;" data-wlai-sym="${escapeHtml(c.symbol)}">${escapeHtml(c.symbol)}</td>
            <td style="padding:4px;text-align:center;">${_catalystBadgeHtml(c.symbol, c)}</td>
            <td style="padding:4px;text-align:right;color:#ccc;">${c.price ? '$' + formatNum(c.price) : '-'}</td>
            <td style="padding:4px;text-align:right;color:${chgC};">${c.change_percent ? fmtPct(c.change_percent) : '-'}</td>
            <td style="padding:4px;text-align:right;color:${rvC};">${c.rvol ? c.rvol.toFixed(1) + 'x' : '-'}</td>
            <td style="padding:4px;text-align:right;color:${gapC};">${c.gap_percent ? fmtPct(c.gap_percent) : '-'}</td>
            <td style="padding:4px;text-align:right;color:#888;">${c.volume ? formatVolume(c.volume) : '-'}</td>
            <td style="padding:4px;text-align:center;color:${floatC};">${escapeHtml(c.float_category || '-')}</td>
            <td style="padding:4px;text-align:right;color:${oracleC};">${fmtScore(c.oracle_score)}</td>
            <td style="padding:4px;text-align:center;color:${regimeC};font-weight:700;">${escapeHtml(c.oracle_regime || '-')}</td>
            <td style="padding:4px;text-align:right;color:${nexusC};">${fmtScore(c.nexus_buy_pct)}</td>
            <td style="padding:4px;text-align:right;color:${fusionC};font-weight:700;">${fmtScore(c.fusion_score)}</td>
            <td style="padding:4px;text-align:center;color:${sigC};font-weight:700;">${escapeHtml(c.fusion_signal || '-')}</td>
            <td style="padding:4px;text-align:center;color:#aaa;">${escapeHtml(c.bt_best_tf || '-')}</td>
            <td style="padding:4px;text-align:right;color:#aaa;">${c.bt_trades || '-'}</td>
            <td style="padding:4px;text-align:right;color:${winC};">${c.bt_win_rate ? fmtPct(c.bt_win_rate) : '-'}</td>
            <td style="padding:4px;text-align:right;color:#aaa;">${fmtScore(c.bt_sharpe)}</td>
            <td style="padding:4px;text-align:right;color:${retC};">${c.bt_return_pct ? fmtPct(c.bt_return_pct) : '-'}</td>
            <td style="padding:4px;text-align:right;color:${(c.bt_max_drawdown || 0) < -20 ? '#ff4444' : (c.bt_max_drawdown || 0) < -10 ? '#FFD700' : '#888'};">${c.bt_max_drawdown ? c.bt_max_drawdown.toFixed(1) + '%' : '-'}</td>
            <td style="padding:4px;text-align:right;color:#aaa;">${fmtScore(c.bt_expectancy)}</td>
            <td style="padding:4px;text-align:right;color:${aiC};font-weight:700;">${fmtScore(c.ai_score)}</td>
            <td style="padding:4px;text-align:right;color:${smartC};font-weight:700;">${fmtScore(c.smart_score)}</td>
            <td style="padding:4px;text-align:center;"><span data-wlai-trade="${escapeHtml(c.symbol)}" style="cursor:pointer;color:#FFD700;font-size:9px;font-weight:700;border:1px solid #FFD700;padding:1px 5px;border-radius:3px;">TRADE</span></td>
            <td style="padding:4px;text-align:center;"><span data-wlai-hft="${escapeHtml(c.symbol)}" style="cursor:pointer;color:#00BFFF;font-size:9px;font-weight:700;border:1px solid #00BFFF;padding:1px 5px;border-radius:3px;">HFT</span></td>
        </tr>`;
    }).join('');

    body.querySelectorAll('[data-wlai-sym]').forEach(el => {
        el.addEventListener('click', () => {
            const sym = el.getAttribute('data-wlai-sym');
            if (sym && typeof setSymbol === 'function') setSymbol(sym, { force: true });
            invoke('set_active_symbol', { symbol: sym }).catch(e => TitanError.report('wl:set-symbol-ai', e));
        });
    });
    body.querySelectorAll('[data-wlai-trade]').forEach(el => {
        el.addEventListener('click', (e) => {
            e.stopPropagation();
            _injectSymbolToManualTrade(el.getAttribute('data-wlai-trade'));
        });
    });
    body.querySelectorAll('[data-wlai-hft]').forEach(el => {
        el.addEventListener('click', async (e) => {
            e.stopPropagation();
            const sym = el.getAttribute('data-wlai-hft');
            el.textContent = '✓';
            el.style.color = '#00BFFF';
            try {
                await invoke('add_hft_target', { symbol: sym });
                refreshHftTargets();
                if (typeof refreshHftEnriched === 'function') refreshHftEnriched();
            } catch (e) {
                TitanError.report('wl:add-hft-target', e);
                el.textContent = '!';
                el.style.color = '#ff4444';
            }
        });
    });
}

async function refreshWlEnriched(symbols) {
    if (_wlEnrichRunning) return;
    _wlEnrichRunning = true;
    const btn = $el('btn-wl-backtest-all');
    const origText = btn ? btn.textContent : '';
    if (btn) { btn.textContent = 'SCORING...'; btn.disabled = true; }

    const container = $el('wl-ai-results');
    if (container) container.style.display = 'block';

    try {
        const res = await invoke('get_watchlist_enriched', { symbols });
        if (res && res.length) {
            const existingMap = {};
            _wlEnrichedCache.forEach(c => { existingMap[c.symbol] = c; });
            res.forEach(c => { existingMap[c.symbol] = c; });
            _wlEnrichedCache = Object.values(existingMap);
        }
        renderWlAiTable(_wlEnrichedCache);
    } catch (e) {
        TitanError.report('wl:ai-enrich', e);
        const statusEl = $el('wl-ai-status');
        if (statusEl) statusEl.textContent = 'Error: ' + normalizeError(e);
    }
    if (btn) { btn.textContent = origText; btn.disabled = false; }
    _wlEnrichRunning = false;
}

async function refreshWatchlistSnapshots() {
    try {
        _wlSnapshotCache = await invoke('get_watchlist_snapshots');
        renderWatchlistTable(_cachedFavorites);
    } catch (e) {
        console.warn('Watchlist snapshot fetch failed:', e);
    }
}

function switchSymbolTab(tab) {
    if (tab === 'watchlist') {
        switchTab(9);
    } else if (tab === 'hft') {
        switchTab(10);
    }
}

function wireFavorites() {
    loadFavoritesFromBackend();

    async function addFavFrom(inputEl) {
        const raw = (inputEl?.value || '').trim().toUpperCase().replace(/\s+/g, '');
        if (!raw) return;
        try {
            _cachedFavorites = await invoke('add_favorite', { symbol: raw });
            renderFavorites(_cachedFavorites);
        } catch (e) { TitanError.report('wl:add-fav-input', e); }
        if (inputEl) inputEl.value = '';
    }

    const sidebarInput = $el('fav-input');
    const sidebarBtn = $el('btn-add-fav');
    if (sidebarBtn) sidebarBtn.addEventListener('click', () => addFavFrom(sidebarInput));
    if (sidebarInput) sidebarInput.addEventListener('keydown', (e) => { if (e.key === 'Enter') addFavFrom(sidebarInput); });

    const centralInput = $el('wl-central-input');
    const centralBtn = $el('btn-wl-central-add');
    if (centralBtn) centralBtn.addEventListener('click', () => addFavFrom(centralInput));
    if (centralInput) centralInput.addEventListener('keydown', (e) => { if (e.key === 'Enter') addFavFrom(centralInput); });

    const btnRefresh = $el('btn-wl-central-refresh');
    if (btnRefresh) {
        btnRefresh.addEventListener('click', async () => {
            btnRefresh.innerText = '...';
            await refreshWatchlistSnapshots();
            btnRefresh.innerText = 'REFRESH';
        });
    }

    const btnDeleteAll = $el('btn-wl-central-delete-all');
    if (btnDeleteAll) {
        btnDeleteAll.addEventListener('click', async () => {
            if (!(await cyberConfirm('Delete all watchlist entries? (Live position assets are protected)'))) return;
            try {
                const snap = await invoke('get_account_snapshot', {});
                const liveSymbols = new Set((snap?.positions || []).map(p => (p.symbol || '').toUpperCase()));
                const favs = await invoke('get_favorites', {});
                const toRemove = (favs || []).filter(s => !liveSymbols.has(s.toUpperCase()));
                for (const sym of toRemove) {
                    try { await invoke('remove_favorite', { symbol: sym }); } catch (e) { TitanError.report('wl:bulk-remove', e); }
                }
                const updated = await invoke('get_favorites', {});
                _wlSnapshotCache = _wlSnapshotCache.filter(x => updated.includes(x.symbol));
                renderWatchlistTable(updated);
                const notice = $el('wl-central-live-notice');
                if (notice && liveSymbols.size > 0) { notice.style.display = 'block'; setTimeout(() => notice.style.display = 'none', 5000); }
            } catch (e) { showCyberToast('Error: ' + normalizeError(e), 'error'); }
        });
    }

    const btnInjectScan = $el('btn-wl-central-inject-scan');
    if (btnInjectScan) {
        btnInjectScan.addEventListener('click', async () => {
            if (!_lastScanResults.length) { showCyberToast('Run a scan first', 'warning'); return; }
            for (const r of _lastScanResults) {
                try { await invoke('add_favorite', { symbol: r.symbol }); } catch (e) { TitanError.report('wl:bulk-add', e); }
            }
            const favs = await invoke('get_favorites', {});
            renderWatchlistTable(favs);
        });
    }

    const btnTrade = $el('btn-wl-central-trade');
    if (btnTrade) {
        btnTrade.addEventListener('click', () => {
            if (!_cachedFavorites.length) { showCyberToast('Watchlist is empty. Add symbols first', 'warning'); return; }
            const choice = prompt('Enter symbol to trade from watchlist:\n' + _cachedFavorites.join(', '));
            if (choice) _injectSymbolToManualTrade(choice.trim().toUpperCase());
        });
    }

    const btnInjectHft = $el('btn-wl-inject-hft');
    if (btnInjectHft) {
        btnInjectHft.addEventListener('click', async () => {
            if (!_cachedFavorites.length) { showCyberToast('Watchlist is empty. Add symbols first', 'warning'); return; }
            btnInjectHft.disabled = true;
            btnInjectHft.textContent = 'INJECTING...';
            let count = 0;
            for (const sym of _cachedFavorites) {
                try {
                    await invoke('add_hft_target', { symbol: sym });
                    count++;
                } catch (e) { TitanError.report('wl:inject-hft', e); }
            }
            btnInjectHft.textContent = `DONE (${count})`;
            btnInjectHft.disabled = false;
            refreshHftTargets();
            if (typeof refreshHftEnriched === 'function') refreshHftEnriched();
            setTimeout(() => { btnInjectHft.textContent = 'INJECT ALL → HFT'; }, 2000);
        });
    }

    const btnBacktestAll = $el('btn-wl-backtest-all');
    if (btnBacktestAll) {
        btnBacktestAll.addEventListener('click', () => {
            if (!_cachedFavorites.length) { showCyberToast('Watchlist is empty. Add symbols first', 'warning'); return; }
            refreshWlEnriched([..._cachedFavorites]);
        });
    }

    const btnWlCheckNews = $el('btn-wl-check-news');
    if (btnWlCheckNews) {
        btnWlCheckNews.addEventListener('click', async () => {
            if (!_cachedFavorites.length) { showCyberToast('Watchlist is empty', 'warning'); return; }
            btnWlCheckNews.textContent = 'CHECKING...';
            btnWlCheckNews.disabled = true;
            try {
                const results = await invoke('check_catalyst', { symbols: [..._cachedFavorites] });
                _updateCatalystCacheFromResults(results);
                renderWatchlistTable(_cachedFavorites);
            } catch (e) {
                TitanError.report('wl:check-news', e);
                btnWlCheckNews.textContent = 'ERROR';
                btnWlCheckNews.style.color = '#ff4444';
                setTimeout(() => { btnWlCheckNews.style.color = '#FF00FF'; }, 2000);
            }
            btnWlCheckNews.textContent = 'CHECK NEWS';
            btnWlCheckNews.disabled = false;
        });
    }

    const btnAiClose = $el('btn-wl-ai-close');
    if (btnAiClose) {
        btnAiClose.addEventListener('click', () => {
            const container = $el('wl-ai-results');
            if (container) container.style.display = 'none';
        });
    }
}

// ═══ DIFF ALERT & NEW MOVER TRACKING ═══
let _prevScanScores = new Map();   // symbol → previous score
let _prevScanSymbols = new Set();  // symbols from last scan
let _scanCount = 0;                // total scans this session

function computeScanDiffs(results) {
    const newMovers = [];
    const enriched = results.map(item => {
        const sym = (item.symbol || '').toUpperCase();
        const prevScore = _prevScanScores.get(sym);
        const delta = (prevScore !== undefined) ? (item.score - prevScore) : null;
        const isNew = _scanCount > 0 && !_prevScanSymbols.has(sym);
        if (isNew) newMovers.push(sym);
        return { ...item, delta, isNew };
    });

    // Update caches for next scan
    _prevScanScores.clear();
    _prevScanSymbols.clear();
    results.forEach(item => {
        const sym = (item.symbol || '').toUpperCase();
        _prevScanScores.set(sym, item.score || 0);
        _prevScanSymbols.add(sym);
    });
    _scanCount++;

    return { enriched, newMovers };
}

function showNewMoverToast(newMovers) {
    if (!newMovers.length) return;
    const count = newMovers.length;
    const syms = newMovers.slice(0, 4).join(', ') + (count > 4 ? ` +${count - 4}` : '');
    showCyberToast(`${count} NEW MOVER${count > 1 ? 'S' : ''}: ${syms}`, 'success', 5000);
}

function updateMdBadge(newCount) {
    const summary = document.querySelector('#market-discovery > summary');
    if (!summary) return;
    let badge = summary.querySelector('.md-new-badge');
    if (newCount > 0) {
        if (!badge) {
            badge = document.createElement('span');
            badge.className = 'md-new-badge';
            summary.appendChild(badge);
        }
        badge.textContent = newCount;
        badge.style.display = 'inline-flex';
    } else if (badge) {
        badge.style.display = 'none';
    }
}

async function runMarketScan(strategy) {
    return await invoke('run_market_scan', { strategy });
}

function renderScanResults(list, containerId) {
    const container = $el(containerId);
    if (!container) return;
    container.style.display = 'block';
    container.innerHTML = '';
    if (!list || !list.length) {
        container.innerHTML = '<div class="sidebar-hint">Fără rezultate.</div>';
        return;
    }
    const showDelta = list.some(item => item.delta !== undefined && item.delta !== null);
    const showNew = list.some(item => item.isNew);
    // Always show FMP columns — show dashes if data is missing
    const showFloat = true;
    const showCap = true;
    const showIns = true;
    const hdr = document.createElement('div');
    hdr.className = 'md-results-header';
    let hdrHtml = `<span class="flex-1">SYMBOL</span><span style="width:50px;text-align:right;">PRICE</span><span style="width:40px;text-align:right;">CHG%</span><span style="width:36px;text-align:right;">RVOL</span><span style="width:32px;text-align:right;">GAP%</span><span style="width:45px;text-align:right;">VOL</span><span style="width:48px;text-align:right;">$VOL</span>`;
    hdrHtml += `<span style="width:38px;text-align:right;">FLOAT</span>`;
    hdrHtml += `<span style="width:38px;text-align:right;">CAP</span>`;
    hdrHtml += `<span style="width:24px;text-align:center;">INS</span>`;
    hdrHtml += `<span style="width:36px;text-align:right;">SCORE</span>`;
    if (showDelta) hdrHtml += `<span style="width:36px;text-align:right;">Δ</span>`;
    hdrHtml += `<span style="width:24px;"></span>`;
    hdr.innerHTML = hdrHtml;
    container.appendChild(hdr);
    list.forEach(item => {
        const sym = normalizeSymbol(item.symbol || '');
        const rvol = item.rvol ? item.rvol.toFixed(1) + 'x' : '-';
        const chgColor = (item.change_percent || 0) >= 10 ? '#00BFFF' : (item.change_percent || 0) >= 0 ? '#FFD700' : '#ff4444';
        const rvolColor = (item.rvol || 0) >= 5 ? '#00BFFF' : '#888';
        const gapVal = item.gap_percent || 0;
        const gapColor = gapVal >= 10 ? '#00BFFF' : gapVal >= 4 ? '#FFD700' : gapVal > 0 ? '#888' : '#ff4444';
        const scoreVal = item.score || 0;
        const scoreColor = scoreVal >= 60 ? '#00BFFF' : scoreVal >= 30 ? '#FFD700' : '#888';
        const row = document.createElement('div');
        row.className = 'scan-result' + (item.isNew ? ' md-new-mover-row' : '');
        row.style.cssText = 'display:flex; justify-content:space-between; align-items:center;';

        let deltaHtml = '';
        if (showDelta && item.delta !== null && item.delta !== undefined) {
            const dVal = item.delta;
            const dColor = dVal > 5 ? '#00FF88' : dVal > 0 ? '#00BFFF' : dVal < -5 ? '#FF4444' : dVal < 0 ? '#FF8800' : '#555';
            const dPrefix = dVal > 0 ? '+' : '';
            deltaHtml = `<span style="width:36px;text-align:right;color:${dColor};font-weight:700;">${dPrefix}${formatNum(dVal)}</span>`;
        } else if (showDelta) {
            deltaHtml = `<span style="width:36px;text-align:right;color:#555;">—</span>`;
        }

        const newBadge = item.isNew ? '<span class="md-new-tag">NEW</span>' : '';
        const earnBadge = item.has_earnings_soon ? ' <span style="color:#FFD700;font-size:9px;" title="Earnings ±3 days">📊</span>' : '';

        // Float column — always shown
        const fv = item.float_shares || 0;
        const floatColor = fv > 0 && fv <= 20 ? '#A855F7' : fv > 20 ? '#888' : '#333';
        const floatText = fv > 0 ? (fv >= 1000 ? `${(fv/1000).toFixed(1)}B` : `${fv.toFixed(1)}M`) : '—';
        const floatHtml = `<span style="width:38px;text-align:right;color:${floatColor};">${floatText}</span>`;

        // Cap column — always shown
        const cm = item.market_cap || 0;
        const capColor = cm > 0 && cm <= 300 ? '#A855F7' : cm <= 2000 ? '#FFD700' : cm <= 10000 ? '#888' : '#555';
        const capText = cm > 0 ? (cm >= 1000 ? `${(cm/1000).toFixed(1)}B` : `${cm.toFixed(0)}M`) : '—';
        const capHtml = `<span style="width:38px;text-align:right;color:${capColor};" title="${item.sector || ''}">${capText}</span>`;

        // Insider column — always shown
        const sig = item.insider_signal || '';
        const insIcon = sig === 'BUY' ? '🟢' : sig === 'SELL' ? '🔴' : '—';
        const insHtml = `<span style="width:24px;text-align:center;font-size:10px;" title="${escapeHtml(item.insider_detail || '')}">${insIcon}</span>`;

        row.innerHTML = `
            <span class="sym" data-symbol="${escapeHtml(sym)}" style="flex:1;cursor:pointer;">${newBadge}${escapeHtml(sym)}${earnBadge}</span>
            <span style="width:50px;text-align:right;color:#ccc;">$${formatNum(item.price)}</span>
            <span style="width:40px;text-align:right;color:${chgColor};">${formatNum(item.change_percent)}%</span>
            <span style="width:36px;text-align:right;color:${rvolColor};">${rvol}</span>
            <span style="width:32px;text-align:right;color:${gapColor};">${gapVal > 0 ? formatNum(gapVal) + '%' : '—'}</span>
            <span style="width:45px;text-align:right;color:#888;">${formatVolume(item.volume)}</span>
            <span style="width:48px;text-align:right;" class="${(item.dollar_volume || item.price * item.volume) >= 10000000 ? 'dvol-gold' : 'dvol-dim'}">${formatDollarVolume(item.dollar_volume || item.price * item.volume)}</span>
            ${floatHtml}
            ${capHtml}
            ${insHtml}
            <span style="width:36px;text-align:right;color:${scoreColor};">${formatNum(scoreVal)}</span>
            ${deltaHtml}
        `;
        const addBtn = document.createElement('span');
        addBtn.style.cssText = 'width:24px;text-align:center;cursor:pointer;color:#FFD700;font-size:11px;font-weight:700;';
        addBtn.innerText = '+';
        addBtn.title = 'Add to Watchlist';
        addBtn.addEventListener('click', async (e) => {
            e.stopPropagation();
            try {
                await invoke('add_favorite', { symbol: sym });
                addBtn.innerText = '✓';
                addBtn.style.color = '#00BFFF';
                setTimeout(() => { addBtn.innerText = '+'; addBtn.style.color = '#FFD700'; }, 1500);
                const favs = await invoke('get_favorites', {});
                if (typeof renderWatchlistTable === 'function') renderWatchlistTable(favs);
                else if (typeof renderFavorites === 'function') renderFavorites(favs);
            } catch (e) { TitanError.report('wl:add-fav', e); }
        });
        row.appendChild(addBtn);
        const symEl = row.querySelector('.sym');
        if (symEl) {
            symEl.addEventListener('click', () => {
                setSymbol(sym, { force: true });
                invoke('set_active_symbol', { symbol: sym }).catch(e => TitanError.report('wl:set-symbol-enriched', e));
            });
        }
        container.appendChild(row);
    });
}

function filterScanResults(list, filters) {
    return (list || []).filter(item => {
        const price = Number(item.price || 0);
        const vol = Number(item.volume || 0);
        const change = Number(item.change_percent || 0);
        if (price < filters.priceMin || price > filters.priceMax) return false;
        if (vol < filters.volMin) return false;
        if (change < filters.changeMin) return false;
        return true;
    });
}

function wireScanButton() {
    const btn = $el('btn-launch-scan');
    const status = $el('scan-results');
    if (!btn) return;
    btn.addEventListener('click', async () => {
        if (status) {
            status.style.display = 'block';
            status.innerHTML = '<div class="sidebar-hint">Scanning SHARK...</div>';
        }
        try {
            const res = await runMarketScan('SHARK');
            renderScanResults(res || [], 'scan-results');
        } catch (e) {
            const errMsg = typeof normalizeScanError === 'function' ? normalizeScanError(e) : normalizeError(e);
            if (status) status.innerHTML = `<div class="sidebar-hint">Scan error: ${escapeHtml(errMsg)}</div>`;
            if (typeof showCyberToast === 'function') showCyberToast(errMsg, 'error');
        }
    });
}

let _lastScanResults = [];

function saveScanParam(field, value) {
    invoke('update_settings', { field, value: String(value) }).catch(e => TitanError.report('wl:save-scan-param', e));
}

function _isAutoScanActive() { return !!TitanTimers._timers['wl:auto-scan']; }

function wireSliderDisplay(sliderId, displayId, settingsField) {
    const slider = $el(sliderId);
    const display = $el(displayId);
    if (slider && display) {
        const update = () => { display.innerText = slider.value; };
        slider.addEventListener('input', update);
        slider.addEventListener('change', () => { if (settingsField) saveScanParam(settingsField, slider.value); });
        update();
    }
}

function wireScanInputs() {
    wireSliderDisplay('md-change-min', 'md-change-val', 'scan_change_min');
    wireSliderDisplay('md-max-candidates', 'md-max-candidates-val', 'scan_max_candidates');
    wireSliderDisplay('md-scan-interval', 'md-scan-interval-val', 'scan_interval_minutes');

    ['md-price-min:scan_price_min', 'md-price-max:scan_price_max', 'md-vol-min:scan_volume_min_m', 'md-rvol-min:scan_rvol_min', 'md-gap-min:scan_gap_min', 'md-float-max:scan_float_max', 'md-cap-max:scan_cap_max'].forEach(pair => {
        const [id, field] = pair.split(':');
        const el = $el(id);
        if (el) el.addEventListener('change', () => {
            let val = Number(el.value);
            if (field === 'scan_volume_min_m') { saveScanParam('scan_volume_min', String(val * 1_000_000)); }
            else { saveScanParam(field, String(val)); }
        });
    });

    bindToggle('chk-require-catalyst', 'scan_require_catalyst');
    bindToggle('chk-exclude-earnings', 'scan_exclude_earnings');

    // Wire FMP API Key save button
    const btnSaveFmp = $el('btn-save-fmp-key');
    if (btnSaveFmp) {
        btnSaveFmp.addEventListener('click', async () => {
            const fmpInput = $el('fmp-api-key');
            if (!fmpInput || !fmpInput.value.trim()) { showCyberToast('Enter FMP API Key', 'warning'); return; }
            try {
                await invoke('update_settings', { field: 'fmp_api_key', value: fmpInput.value.trim() });
                btnSaveFmp.textContent = '✓ SAVED';
                btnSaveFmp.style.borderColor = '#00FF88';
                btnSaveFmp.style.color = '#00FF88';
                setTimeout(() => { btnSaveFmp.textContent = 'SAVE FMP KEY'; btnSaveFmp.style.borderColor = '#A855F7'; btnSaveFmp.style.color = '#A855F7'; }, 2000);
            } catch (e) {
                showCyberToast('Error saving FMP key: ' + normalizeError(e), 'error');
            }
        });
    }
}

function wireScanActions() {
    const btnScan = $el('btn-md-scan');
    const btnClear = $el('btn-md-clear');
    const status = $el('md-status');

    if (btnScan) {
        btnScan.addEventListener('click', async () => {
            if (status) status.innerText = 'Scanare in curs...';
            btnScan.disabled = true;
            btnScan.textContent = '⚡ SCANNING...';
            try {
                const res = await runMarketScan('DISCOVERY');
                const { enriched, newMovers } = computeScanDiffs(res || []);
                _lastScanResults = enriched;
                renderScanResults(enriched, 'md-results');
                if (status) {
                    let msg = `Rezultate: ${enriched.length}`;
                    if (newMovers.length > 0) msg += ` | 🆕 ${newMovers.length} new`;
                    status.innerText = msg;
                }
                showNewMoverToast(newMovers);
                updateMdBadge(newMovers.length);
            } catch (e) {
                const errMsg = typeof normalizeScanError === 'function' ? normalizeScanError(e) : normalizeError(e);
                if (status) status.innerText = `Scan error: ${errMsg}`;
                if (typeof showCyberToast === 'function') showCyberToast(errMsg, 'error');
            }
            btnScan.disabled = false;
            btnScan.textContent = '⚡ SCAN';
        });
    }
    if (btnClear) {
        btnClear.addEventListener('click', () => {
            const results = $el('md-results');
            if (results) results.innerHTML = '';
            if (status) status.innerText = '';
            _lastScanResults = [];
        });
    }

    const btnInjectWl = $el('btn-md-inject-wl');
    if (btnInjectWl) {
        btnInjectWl.addEventListener('click', async () => {
            if (!_lastScanResults.length) { showCyberToast('Run a scan first', 'warning'); return; }
            for (const r of _lastScanResults) {
                try { await invoke('add_favorite', { symbol: r.symbol }); } catch (e) { TitanError.report('wl:bulk-add-md', e); }
            }
            const favs = await invoke('get_favorites', {});
            renderWatchlistTable(favs);
            if (status) status.innerText = `Injected ${_lastScanResults.length} -> Watchlist`;
        });
    }

    const btnAutoInject = $el('btn-md-autoinject');
    if (btnAutoInject) {
        btnAutoInject.addEventListener('click', () => {
            const current = getToggleState('chk-auto-inject-hft') || false;
            invoke('update_settings', { field: 'ai_auto_inject_hft', value: String(!current) });
            setToggleChecked('chk-auto-inject-hft', !current);
            if (status) status.innerText = !current ? 'Auto-inject ON' : 'Auto-inject OFF';
        });
    }

    const btnInjectSignal = $el('btn-md-inject-signal');
    if (btnInjectSignal) {
        btnInjectSignal.addEventListener('click', async () => {
            if (!_lastScanResults.length) { showCyberToast('Run a scan first', 'warning'); return; }
            const symbols = _lastScanResults.map(r => r.symbol).filter(Boolean);
            try {
                const list = await invoke('inject_to_signal_assets', { symbols });
                if (status) status.innerText = `Injected ${symbols.length} → Signal CFG (total: ${list.length})`;
                btnInjectSignal.style.background = 'rgba(168,85,247,0.3)';
                setTimeout(() => { btnInjectSignal.style.background = ''; }, 1200);
                // Refresh Signal CFG table
                if (typeof window.loadSignalAssets === 'function') window.loadSignalAssets();
            } catch (e) {
                if (status) status.innerText = `Signal inject error: ${normalizeError(e)}`;
                TitanError.report('wl:inject-signal', e);
            }
        });
    }

    // INJECT → HFT TARGETS
    const btnInjectHft = $el('btn-md-inject-hft');
    if (btnInjectHft) {
        btnInjectHft.addEventListener('click', async () => {
            if (!_lastScanResults.length) { showCyberToast('Run a scan first', 'warning'); return; }
            const symbols = _lastScanResults.map(r => r.symbol).filter(Boolean);
            let ok = 0;
            for (const sym of symbols) {
                try {
                    await invoke('add_hft_target', { symbol: sym });
                    ok++;
                } catch (_) { /* skip duplicates */ }
            }
            if (status) status.innerText = `Injected ${ok}/${symbols.length} → HFT TARGETS`;
            btnInjectHft.style.background = 'rgba(0,191,255,0.25)';
            setTimeout(() => { btnInjectHft.style.background = ''; }, 1200);
            // Refresh HFT Targets table
            if (typeof refreshHftTargets === 'function') refreshHftTargets();
        });
    }
}

function wireAutoScan() {
    const btnScan = $el('btn-md-scan');
    const btnAutoScan = $el('btn-md-auto-scan');
    if (!btnAutoScan) return;

    function doAutoScan() {
        if (btnScan) btnScan.click();
    }
    function startAutoScan() {
        const mins = Number($el('md-scan-interval')?.value || 30);
        TitanTimers.register('wl:auto-scan', doAutoScan, mins * 60 * 1000);
        btnAutoScan.innerText = 'AUTO SCAN: ON';
        btnAutoScan.style.background = 'rgba(0,191,255,0.25)';
        btnAutoScan.style.borderColor = '#00BFFF';
        btnAutoScan.style.color = '#00BFFF';
        invoke('update_settings', { field: 'md_auto_scan_active', value: 'true' }).catch(e => TitanError.report('wl:auto-scan-toggle', e));
    }
    function stopAutoScan() {
        TitanTimers.clear('wl:auto-scan');
        btnAutoScan.innerText = 'AUTO SCAN: OFF';
        btnAutoScan.style.background = '';
        btnAutoScan.style.borderColor = '';
        btnAutoScan.style.color = '';
        invoke('update_settings', { field: 'md_auto_scan_active', value: 'false' }).catch(e => TitanError.report('wl:auto-scan-toggle', e));
    }
    btnAutoScan.addEventListener('click', () => {
        if (_isAutoScanActive()) stopAutoScan();
        else startAutoScan();
    });
    const intervalSlider = $el('md-scan-interval');
    if (intervalSlider) {
        intervalSlider.addEventListener('change', () => {
            if (_isAutoScanActive()) { stopAutoScan(); startAutoScan(); }
        });
    }
}

function wireWarriorDefaults() {
    const status = $el('md-status');
    const btnWarriorDefaults = $el('btn-md-warrior-defaults');
    if (!btnWarriorDefaults) return;

    btnWarriorDefaults.addEventListener('click', () => {
        const presets = {
            'md-price-min': { val: 1, field: 'scan_price_min' },
            'md-price-max': { val: 20, field: 'scan_price_max' },
            'md-vol-min': { val: 1, field: 'scan_volume_min', mult: 1_000_000 },
            'md-rvol-min': { val: 5, field: 'scan_rvol_min' },
            'md-gap-min': { val: 0, field: 'scan_gap_min' },
            'md-float-max': { val: 20, field: 'scan_float_max' },
            'md-cap-max': { val: 2, field: 'scan_cap_max' },
        };
        for (const [id, cfg] of Object.entries(presets)) {
            const el = $el(id);
            if (el) {
                el.value = cfg.val;
                const sv = cfg.mult ? String(cfg.val * cfg.mult) : String(cfg.val);
                saveScanParam(cfg.field, sv);
            }
        }
        const chgSlider = $el('md-change-min');
        if (chgSlider) { chgSlider.value = 10; const d = $el('md-change-val'); if (d) d.innerText = '10'; saveScanParam('scan_change_min', '10'); }
        const maxSlider = $el('md-max-candidates');
        if (maxSlider) { maxSlider.value = 100; const d = $el('md-max-candidates-val'); if (d) d.innerText = '100'; saveScanParam('scan_max_candidates', '100'); }
        setToggleChecked('chk-require-catalyst', true);
        saveScanParam('scan_require_catalyst', 'true');
        btnWarriorDefaults.style.background = 'rgba(255,215,0,0.3)';
        setTimeout(() => { btnWarriorDefaults.style.background = ''; }, 800);
        if (status) status.innerText = 'Ross Cameron defaults applied';
    });
}

function wireMarketDiscovery() {
    wireScanInputs();
    wireScanActions();
    wireAutoScan();
    wireWarriorDefaults();
    wireExtraPresets();
    wireScorePresetDropdown();
}

function wireExtraPresets() {
    const status = $el('md-status');

    function applyPreset(name, overrides) {
        for (const [id, cfg] of Object.entries(overrides.inputs || {})) {
            const el = $el(id);
            if (el) {
                el.value = cfg.val;
                const sv = cfg.mult ? String(cfg.val * cfg.mult) : String(cfg.val);
                saveScanParam(cfg.field, sv);
            }
        }
        if (overrides.change !== undefined) {
            const chgSlider = $el('md-change-min');
            if (chgSlider) { chgSlider.value = overrides.change; const d = $el('md-change-val'); if (d) d.innerText = String(overrides.change); saveScanParam('scan_change_min', String(overrides.change)); }
        }
        if (overrides.maxCandidates !== undefined) {
            const maxSlider = $el('md-max-candidates');
            if (maxSlider) { maxSlider.value = overrides.maxCandidates; const d = $el('md-max-candidates-val'); if (d) d.innerText = String(overrides.maxCandidates); saveScanParam('scan_max_candidates', String(overrides.maxCandidates)); }
        }
        if (overrides.catalyst !== undefined) {
            setToggleChecked('chk-require-catalyst', overrides.catalyst);
            saveScanParam('scan_require_catalyst', String(overrides.catalyst));
        }
        if (overrides.scorePreset) {
            const sel = $el('md-score-preset');
            if (sel) sel.value = overrides.scorePreset;
            saveScanParam('scan_score_preset', overrides.scorePreset);
        }
        if (status) status.innerText = `${name} preset applied`;
    }

    const presetConfigs = {
        'btn-md-penny-rockets': {
            name: 'Penny Rockets',
            overrides: {
                inputs: {
                    'md-price-min': { val: 0.1, field: 'scan_price_min' },
                    'md-price-max': { val: 5, field: 'scan_price_max' },
                    'md-vol-min': { val: 2, field: 'scan_volume_min', mult: 1_000_000 },
                    'md-rvol-min': { val: 5, field: 'scan_rvol_min' },
                    'md-gap-min': { val: 0, field: 'scan_gap_min' },
                },
                change: 20,
                maxCandidates: 50,
                catalyst: false,
                scorePreset: 'momentum',
            },
        },
        'btn-md-swing-setup': {
            name: 'Swing Setup',
            overrides: {
                inputs: {
                    'md-price-min': { val: 10, field: 'scan_price_min' },
                    'md-price-max': { val: 100, field: 'scan_price_max' },
                    'md-vol-min': { val: 1, field: 'scan_volume_min', mult: 1_000_000 },
                    'md-rvol-min': { val: 3, field: 'scan_rvol_min' },
                    'md-gap-min': { val: 2, field: 'scan_gap_min' },
                },
                change: 3,
                maxCandidates: 100,
                catalyst: false,
                scorePreset: 'gap_scanner',
            },
        },
        'btn-md-large-cap': {
            name: 'Large Cap',
            overrides: {
                inputs: {
                    'md-price-min': { val: 50, field: 'scan_price_min' },
                    'md-price-max': { val: 500, field: 'scan_price_max' },
                    'md-vol-min': { val: 5, field: 'scan_volume_min', mult: 1_000_000 },
                    'md-rvol-min': { val: 1, field: 'scan_rvol_min' },
                    'md-gap-min': { val: 0, field: 'scan_gap_min' },
                },
                change: 3,
                maxCandidates: 50,
                catalyst: false,
                scorePreset: 'volume_hunter',
            },
        },
        'btn-md-hot-momentum': {
            name: 'Hot Momentum',
            overrides: {
                inputs: {
                    'md-price-min': { val: 1, field: 'scan_price_min' },
                    'md-price-max': { val: 50, field: 'scan_price_max' },
                    'md-vol-min': { val: 1, field: 'scan_volume_min', mult: 1_000_000 },
                    'md-rvol-min': { val: 3, field: 'scan_rvol_min' },
                    'md-gap-min': { val: 0, field: 'scan_gap_min' },
                },
                change: 10,
                maxCandidates: 100,
                catalyst: false,
                scorePreset: 'momentum',
            },
        },
    };

    for (const [btnId, cfg] of Object.entries(presetConfigs)) {
        const btn = $el(btnId);
        if (btn) {
            btn.addEventListener('click', () => {
                applyPreset(cfg.name, cfg.overrides);
                btn.style.background = 'rgba(0,191,255,0.3)';
                setTimeout(() => { btn.style.background = ''; }, 800);
            });
        }
    }
}

function wireScorePresetDropdown() {
    const sel = $el('md-score-preset');
    if (sel) {
        sel.addEventListener('change', () => {
            saveScanParam('scan_score_preset', sel.value);
            const status = $el('md-status');
            if (status) status.innerText = `Score mode: ${sel.value.replace('_', ' ').toUpperCase()}`;
        });
    }
}

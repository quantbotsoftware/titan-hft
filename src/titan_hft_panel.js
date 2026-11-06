let _hftEnrichedCache = [];
let _hftEnrichRunning = false;
let _hftFillQualityMap = {};

function renderHftTable(data, activeSet) {
    const body = $el('hft-central-body');
    const emptyEl = $el('hft-central-empty');
    if (!body) return;

    if (!data || data.length === 0) {
        body.innerHTML = '';
        if (emptyEl) emptyEl.style.display = 'block';
        return;
    }
    if (emptyEl) emptyEl.style.display = 'none';

    // fmtPct, fmtScore -> titan_core.js

    body.innerHTML = data.map((c, i) => {
        const isActive = activeSet && activeSet.has(c.symbol);
        const liveTag = isActive ? '<span class="cyber-live-tag">LIVE</span>' : '';

        const chgC = c.change_percent >= 10 ? '#00BFFF' : c.change_percent >= 0 ? '#FFD700' : '#ff4444';
        const rvC = c.rvol >= 5 ? '#00BFFF' : c.rvol >= 2 ? '#FFD700' : '#888';
        const gapC = c.gap_percent > 3 ? '#00BFFF' : c.gap_percent > 0 ? '#FFD700' : '#888';
        const floatC = c.float_category === 'LOW' ? '#A855F7' : c.float_category === 'MED' ? '#FFD700' : '#888';
        const oracleC = c.oracle_score >= 70 ? '#00BFFF' : c.oracle_score >= 40 ? '#FFD700' : '#888';
        const regimeC = c.oracle_regime === 'BULL' || c.oracle_regime === 'Bull' ? '#00BFFF' : c.oracle_regime === 'BEAR' || c.oracle_regime === 'Bear' ? '#ff4444' : '#FFD700';
        const nexusC = c.nexus_buy_pct >= 60 ? '#FF69B4' : '#888';
        const fusionC = c.fusion_score >= 7 ? '#FFD700' : '#888';
        const sigC = c.fusion_signal === 'LONG' || c.fusion_signal === 'BUY' ? '#00BFFF' : c.fusion_signal === 'SHORT' || c.fusion_signal === 'SELL' ? '#ff4444' : '#FFD700';
        const winC = c.bt_win_rate >= 60 ? '#00BFFF' : c.bt_win_rate >= 45 ? '#FFD700' : '#ff4444';
        const retC = pnlColor(c.bt_return_pct);
        const aiC = c.ai_score >= 70 ? '#FFD700' : '#888';
        const smartC = c.smart_score >= 70 ? '#00BFFF' : '#888';

        return `<tr>
            <td style="text-align:center;color:#555;">${c.rank || i + 1}</td>
            <td class="td-sym" data-hft-sym="${escapeHtml(c.symbol)}">${escapeHtml(c.symbol)}${liveTag}</td>
            <td style="text-align:center;">${_catalystBadgeHtml(c.symbol, c)}</td>
            <td style="text-align:right;color:#ccc;">${c.price ? '$' + formatNum(c.price) : '-'}</td>
            <td style="text-align:right;color:${chgC};">${c.change_percent ? fmtPct(c.change_percent) : '-'}</td>
            <td style="text-align:right;color:${rvC};">${c.rvol ? c.rvol.toFixed(1) + 'x' : '-'}</td>
            <td style="text-align:right;color:${gapC};">${c.gap_percent ? fmtPct(c.gap_percent) : '-'}</td>
            <td style="text-align:right;color:#888;">${c.volume ? formatVolume(c.volume) : '-'}</td>
            <td style="text-align:center;color:${floatC};">${escapeHtml(c.float_category || '-')}</td>
            <td style="text-align:right;color:#888;">${c.market_cap > 0 ? (c.market_cap >= 1000 ? (c.market_cap/1000).toFixed(1) + 'B' : c.market_cap.toFixed(0) + 'M') : '—'}</td>
            <td style="text-align:center;color:${c.insider_signal === 'BUY' ? '#00BFFF' : c.insider_signal === 'SELL' ? '#ff4444' : '#555'};">${c.insider_signal || '—'}</td>
            <td style="text-align:right;color:${oracleC};">${fmtScore(c.oracle_score)}</td>
            <td style="text-align:center;color:${regimeC};font-weight:700;">${escapeHtml(c.oracle_regime || '-')}</td>
            <td style="text-align:right;color:${nexusC};">${fmtScore(c.nexus_buy_pct)}</td>
            <td style="text-align:right;color:${fusionC};font-weight:700;">${fmtScore(c.fusion_score)}</td>
            <td style="text-align:center;color:${sigC};font-weight:700;">${escapeHtml(c.fusion_signal || '-')}</td>
            <td style="text-align:center;color:#aaa;">${escapeHtml(c.bt_best_tf || '-')}</td>
            <td style="text-align:right;color:#aaa;">${c.bt_trades || '-'}</td>
            <td style="text-align:right;color:${winC};">${c.bt_win_rate ? fmtPct(c.bt_win_rate) : '-'}</td>
            <td style="text-align:right;color:#aaa;">${fmtScore(c.bt_sharpe)}</td>
            <td style="text-align:right;color:${retC};">${c.bt_return_pct ? fmtPct(c.bt_return_pct) : '-'}</td>
            <td style="text-align:right;color:${(c.bt_max_drawdown || 0) < -20 ? '#ff4444' : (c.bt_max_drawdown || 0) < -10 ? '#FFD700' : '#888'};">${c.bt_max_drawdown ? c.bt_max_drawdown.toFixed(1) + '%' : '-'}</td>
            <td style="text-align:right;color:#aaa;">${fmtScore(c.bt_expectancy)}</td>
            <td style="text-align:right;color:${aiC};font-weight:700;">${fmtScore(c.ai_score)}</td>
            <td style="text-align:right;color:${smartC};font-weight:700;">${fmtScore(c.smart_score)}</td>
            ${_fqCols(c.symbol)}
            <td style="text-align:center;"><span class="tbl-del" data-hft-del="${escapeHtml(c.symbol)}">×</span></td>
        </tr>`;
    }).join('');

    body.querySelectorAll('[data-hft-sym]').forEach(el => {
        el.addEventListener('click', () => {
            const sym = el.getAttribute('data-hft-sym');
            if (sym && typeof setSymbol === 'function') setSymbol(sym, { force: true });
        });
    });
    body.querySelectorAll('[data-hft-del]').forEach(el => {
        el.addEventListener('click', async () => {
            const sym = el.getAttribute('data-hft-del');
            if (activeSet && activeSet.has(sym)) {
                if (!(await cyberConfirm(`${sym} has an active position. Remove from HFT targets anyway?`))) return;
            }
            await invoke('remove_hft_target', { symbol: sym });
            _hftEnrichedCache = _hftEnrichedCache.filter(c => c.symbol !== sym);
            await refreshHftTargets();
        });
    });
}

async function refreshHftTargets() {
    try {
        const res = await invoke('get_hft_targets', {});
        const list = res && res.targets ? res.targets : [];
        const activeSet = new Set(res && res.active_symbols ? res.active_symbols : []);

        const sbCount = $el('sb-hft-count');
        if (sbCount) sbCount.textContent = `(${list.length})`;
        const cntEl = $el('hft-central-count');
        if (cntEl) cntEl.innerText = `Targets: ${list.length}`;

        const cache = Array.isArray(_hftEnrichedCache) ? _hftEnrichedCache : [];
        const filtered = cache.filter(c => list.includes(c.symbol));
        renderHftTable(filtered, activeSet);
    } catch (e) {
        TitanError.report('hft:list', e);
    }
}

async function refreshHftEnriched() {
    if (_hftEnrichRunning) return;
    _hftEnrichRunning = true;
    const btnRefresh = $el('btn-hft-central-refresh');
    if (btnRefresh) { btnRefresh.innerText = 'SCORING...'; btnRefresh.disabled = true; }
    try {
        const results = await invoke('get_hft_target_enriched', {});
        _hftEnrichedCache = Array.isArray(results) ? results : [];
        await refreshHftTargets();
    } catch (e) {
        TitanError.report('hft:enrich', e);
    } finally {
        _hftEnrichRunning = false;
        if (btnRefresh) { btnRefresh.innerText = 'REFRESH'; btnRefresh.disabled = false; }
    }
}

if (window.__TAURI__) {
    window.__TAURI__.event.listen('hft-enrich-progress', (ev) => {
        const d = ev.payload;
        const btn = $el('btn-hft-central-refresh');
        if (!btn) return;
        if (d.phase === 'scoring' && d.symbol) {
            btn.innerText = `AI: ${d.symbol} (${d.done}/${d.total})`;
        } else if (d.phase === 'snapshots') {
            btn.innerText = 'LOADING...';
        } else if (d.phase === 'done') {
            btn.innerText = 'REFRESH';
            btn.disabled = false;
        }
    });
    window.__TAURI__.event.listen('wl-enrich-progress', (ev) => {
        const d = ev.payload;
        const btn = $el('btn-wl-backtest-all');
        if (!btn) return;
        if (d.phase === 'scoring' && d.symbol) {
            btn.innerText = `AI: ${d.symbol} (${d.done}/${d.total})`;
        } else if (d.phase === 'snapshots') {
            btn.innerText = 'LOADING...';
        } else if (d.phase === 'done') {
            btn.innerText = 'BACKTEST ALL';
            btn.disabled = false;
        }
    });

    async function _pollFillQuality() {
        try {
            const stats = await window.__TAURI__.tauri.invoke('get_fill_quality_stats');
            if (Array.isArray(stats)) {
                _hftFillQualityMap = {};
                for (const s of stats) _hftFillQualityMap[s.symbol] = s;
            }
        } catch (_) {}
    }
    setInterval(_pollFillQuality, 30000);
    setTimeout(_pollFillQuality, 4000);
}

function _fqCols(sym) {
    const fq = _hftFillQualityMap[sym];
    if (!fq) return '<td style="text-align:right;color:var(--text-dim);">-</td><td style="text-align:right;color:var(--text-dim);">-</td><td style="text-align:right;color:var(--text-dim);">-</td>';
    const slip = fq.avg_slippage_bps || 0;
    const sc = slip < 2 ? 'var(--neon-green)' : slip < 5 ? 'var(--neon-cyan)' : slip < 10 ? 'var(--neon-gold)' : 'var(--neon-red)';
    const dur = fq.avg_duration_secs || 0;
    const durStr = dur < 60 ? dur.toFixed(0) + 's' : (dur / 60).toFixed(1) + 'm';
    return `<td style="text-align:right;color:${sc};font-size:10px;">${slip.toFixed(1)}</td><td style="text-align:right;color:var(--text-dim);font-size:10px;">${durStr}</td><td style="text-align:right;color:var(--text-dim);font-size:10px;">${fq.trade_count}</td>`;
}

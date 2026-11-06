async function quickTradeExecute(side) {
    const sym = currentSymbol;
    if (!sym) { showCyberToast('No symbol selected', 'warning'); return; }
    const qtyVal = parseFloat($el('qt-qty')?.value || 0);
    if (!qtyVal || qtyVal <= 0) { showCyberToast('Invalid quantity', 'warning'); return; }
    const unit = $el('qt-unit')?.value || 'usd';

    if (unit === 'shares') {
        await executeOrder(sym, side, qtyVal);
    } else {
        try {
            const orderPayload = { symbol: sym, side, usd_amount: qtyVal };
            const res = await invoke('submit_manual_order', { order: orderPayload });
            showCyberToast('QT ORDER SUCCESS: ' + res, 'success');
            await updateAccount();
        } catch (e) {
            showCyberToast('QT ORDER FAILED: ' + normalizeError(e), 'error');
        }
    }
}

async function executeOrder(symbol, side, qty) {
    const statusEl = $el('connection-status');

    try {
        if (statusEl) statusEl.innerText = `⏳ ORDERING ${side.toUpperCase()}...`;

        const res = await invoke('submit_order', {
            order: { symbol: symbol, side: side, qty: parseFloat(qty) }
        });

        showCyberToast('ORDER SUCCESS: ' + res, 'success');
        if (statusEl) statusEl.innerText = "🟢 LIVE";

    } catch (e) {
        showCyberToast('ORDER FAILED: ' + normalizeError(e), 'error');
        if (statusEl) statusEl.innerText = "❌ ERROR";
    }
}

function updateStatus(msg, color, type = "default") {
    const el = $el('connection-status');
    const dot = $el('status-dot');

    if (el) {
        el.innerText = msg;
        el.style.color = color;
    }
    if (dot) {
        dot.className = "status-dot"; // reset
        if (type === "success") dot.classList.add("green");
        else if (type === "warning") dot.classList.add("yellow");
        else if (type === "error") dot.classList.add("red");
    }
}

let apiInitStarted = false;

async function initApiConnection() {
    if (apiInitStarted || !window.__TAURI__) return;
    apiInitStarted = true;
    updateStatus("🔌 CONNECTING API...", "#FFCC00", "warning");
    try {
        await invoke("init_api_connection", {});
        updateStatus("🟢 API CONNECTED", "#00BFFF", "success");
        setTimeout(() => loadData(currentSymbol), 300);
    } catch (e) {
        updateStatus(`⚠️ API NOT CONNECTED: ${normalizeError(e)}`, "#FFCC00", "warning");
    }
}

// → titan_monitor.js

let _positionModes = {};

async function loadPositionModes() {
    try { _positionModes = await invoke('get_position_modes', {}) || {}; } catch (_) { _positionModes = {}; }
    if (typeof renderManualAssetList === 'function') renderManualAssetList();
}

function renderPositions(positions) {
    const body = $el('positions-body');
    if (!body) return;
    body.innerHTML = '';

    if (!positions || positions.length === 0) {
        const row = document.createElement('tr');
        row.innerHTML = `<td colspan="14" class="pos-empty">No open positions</td>`;
        body.appendChild(row);
        return;
    }

    positions.forEach(p => {
        const qty = p.qty || 0;
        const price = qty !== 0 ? Math.abs(p.market_value / qty) : 0;
        const pnl = p.unrealized_pl || 0;
        const pnlPct = (p.unrealized_plpc || 0) * 100.0;
        const side = (p.side || '').toUpperCase();
        const closeSide = side === 'SHORT' ? 'buy' : 'sell';
        const label = side === 'SHORT' ? '💀 BUY' : '💀 SELL';
        const pnlClr = pnlColor(pnl);
        const tag = p.entry_tag || '';
        const isScalp = tag.startsWith('SCALP|');
        const isST = tag.startsWith('ST|');
        const isStAuto = tag.startsWith('ST|AUTO');
        const isStMan = tag.startsWith('ST|MAN');
        const regime = isScalp ? tag.replace('SCALP|','') : (isST ? '' : tag);
        const scalpBadge = isScalp
            ? '<span style="background:#FFD700;color:#000;font-size:8px;font-weight:700;padding:1px 4px;border-radius:3px;margin-left:4px;letter-spacing:0.5px;">SCALP</span>'
            : '';
        const stBadge = isST
            ? `<span class="st-pos-tag ${isStAuto ? 'st-pos-auto' : 'st-pos-man'}" title="SmartTrend ${isStAuto ? 'AUTO' : 'MAN'}">[ST] ${isStAuto ? 'AUTO' : 'MAN'}</span>`
            : '';
        const regimeBadge = regime
            ? `<span style="color:#00BFFF;font-size:8px;margin-left:3px;">${escapeHtml(regime)}</span>`
            : '';

        const fsm = p.fsm_state || 'Open';
        const fsmMap = {
            PendingEntry:  { label: 'PENDING',  cls: 'fsm-pending-entry' },
            Open:          { label: 'OPEN',     cls: 'fsm-open' },
            PartialExit:   { label: 'PARTIAL',  cls: 'fsm-partial' },
            PendingClose:  { label: 'CLOSING',  cls: 'fsm-pending-close' },
            Closed:        { label: 'CLOSED',   cls: 'fsm-closed' },
        };
        const fsmInfo = fsmMap[fsm] || { label: fsm, cls: 'fsm-open' };

        const isManual = _positionModes[p.symbol] === 'manual';
        const modeIcon = isManual ? '🤚' : '🤖';
        const modeColor = isManual ? '#FFA500' : '#00BFFF';
        const modeLabel = isManual ? 'MAN' : 'AUTO';
        const nextMode = isManual ? 'auto' : 'manual';

        const nowSecs = Math.floor(Date.now() / 1000);
        const openedAt = p.opened_at || 0;
        const holdSecs = openedAt > 0 ? nowSecs - openedAt : 0;
        let timeStr = '--';
        let timeClass = 'pos-time-dim';
        if (holdSecs > 0) {
            const hrs = Math.floor(holdSecs / 3600);
            const mins = Math.floor((holdSecs % 3600) / 60);
            const secs = holdSecs % 60;
            if (hrs > 0) timeStr = hrs + 'h ' + mins + 'm';
            else if (mins > 0) timeStr = mins + 'm ' + secs + 's';
            else timeStr = secs + 's';
            if (holdSecs > 7200) timeClass = 'pos-time-orange';
            else if (holdSecs > 1800) timeClass = 'pos-time-gold';
            else if (holdSecs > 300) timeClass = 'pos-time-cyan';
            else timeClass = 'pos-time-dim';
        }

        // S67 B4: Trail-stop integrity indicator
        const trailInfo = (window._titanTrailHealth || []).find(t => t.symbol === p.symbol);
        const trailOk = trailInfo ? trailInfo.trail_ok : true;
        const trailBadge = trailOk
            ? '<span class="trail-ok">TRAIL OK</span>'
            : '<span class="trail-warn">TRAIL WARN</span>';

        const isStPos = (p.entry_tag || '').startsWith('ST|');
        const orphanInfo = (window._titanOrphanStatus || []).find(o => o.symbol === p.symbol);
        const isOrphan = orphanInfo ? orphanInfo.orphan : false;
        const orphanBadge = isStPos
            ? (isOrphan ? '<span class="orphan-warn">ORPHAN</span>' : '<span class="orphan-ok">OK</span>')
            : '';

        const maxHold = window._titanMaxHoldSecs || 3600;
        const holdPct = maxHold > 0 ? Math.min(1, holdSecs / maxHold) : 0;
        const holdCls = holdPct > 0.9 ? 'hold-critical' : holdPct > 0.7 ? 'hold-warn' : '';
        const holdMin = Math.floor(holdSecs / 60);
        const maxMin = Math.floor(maxHold / 60);
        const holdLabel = holdMin + '/' + maxMin + 'm';
        const holdBar = '<div class="hold-timer-cell"><div class="hold-timer-bar"><div class="hold-timer-fill ' + holdCls + '" style="width:' + (holdPct * 100).toFixed(0) + '%"></div></div><span class="hold-timer-text">' + holdLabel + '</span></div>';

        const row = document.createElement('tr');
        row.style.cursor = 'pointer';
        row.title = 'Click to load ' + p.symbol + ' on chart';
        const eSym = escapeHtml(p.symbol);
        const eSide = escapeHtml(side);
        const eTF = escapeHtml(currentTF);
        row.innerHTML = `
            <td><div class="sym-badge sym-click" data-symbol="${eSym}"><span>🟣</span> ${eSym}${scalpBadge}${stBadge}</div></td>
            <td>$${price.toFixed(2)}</td>
            <td><div class="tf-badge">${eTF}</div>${regimeBadge}</td>
            <td>${eSide}</td>
            <td><span class="fsm-badge ${fsmInfo.cls}">${fsmInfo.label}</span></td>
            <td><button class="btn-mode-toggle" data-mode-sym="${eSym}" data-mode-next="${escapeHtml(nextMode)}" style="background:none; border:1px solid ${modeColor}; color:${modeColor}; font-size:9px; padding:2px 6px; border-radius:3px; cursor:pointer; font-family:monospace; font-weight:700;" title="Toggle auto/manual">${modeIcon} ${escapeHtml(modeLabel)}</button></td>
            <td>$${Math.abs(p.market_value).toFixed(2)}</td>
            <td style="color:${pnlClr}">${pnl >= 0 ? '' : '-'}$${Math.abs(pnl).toFixed(2)}</td>
            <td style="color:${pnlClr}">${pnlPct.toFixed(2)}%</td>
            <td>${trailBadge}</td>
            <td>${orphanBadge}</td>
            <td>${holdBar}</td>
            <td class="${timeClass}">${timeStr}</td>
            <td><button class="btn-sell" data-close-symbol="${eSym}" data-close-side="${escapeHtml(closeSide)}" data-close-qty="${Math.abs(qty)}">${escapeHtml(label)}</button> <button class="btn-force-close" data-fc-sym="${eSym}" title="Force close / remove stuck position">✕</button></td>
        `;
        body.appendChild(row);
    });

    body.querySelectorAll('.btn-mode-toggle').forEach(btn => {
        btn.addEventListener('click', async () => {
            const sym = btn.getAttribute('data-mode-sym');
            const next = btn.getAttribute('data-mode-next');
            try {
                await invoke('set_position_mode', { symbol: sym, mode: next });
                _positionModes[sym] = next;
                await updateAccount();
            } catch (e) { TitanError.report('positions:mode-toggle', e); }
        });
    });

    body.querySelectorAll('.btn-sell[data-close-symbol]').forEach(btn => {
        btn.addEventListener('click', () => {
            const sym = btn.getAttribute('data-close-symbol');
            const side = btn.getAttribute('data-close-side');
            const qty = parseFloat(btn.getAttribute('data-close-qty'));
            if (sym && side && qty) executeOrder(sym, side, qty);
        });
    });
    body.querySelectorAll('.btn-force-close[data-fc-sym]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const sym = btn.getAttribute('data-fc-sym');
            if (!(await cyberConfirm(`Force close/remove position ${sym}? This will attempt API close, then remove from local state.`))) return;
            try {
                const res = await invoke('force_close_position', { symbol: sym });
                showCyberToast(res, 'success');
                await updateAccount();
            } catch (e) { showCyberToast('Error: ' + normalizeError(e), 'error'); }
        });
    });
}

function applyBadgeColor(el, value) {
    if (!el) return;
    el.classList.remove('badge-positive', 'badge-negative', 'badge-neutral');
    if (value > 0) el.classList.add('badge-positive');
    else if (value < 0) el.classList.add('badge-negative');
    else el.classList.add('badge-neutral');
}

async function updateAccount() {
    try {
        const snap = await invoke("get_account_snapshot", {});
        if (!snap) return;

        const investedEl = $el('metric-invested');
        const availEl = $el('metric-available');
        const profitEl = $el('metric-profit');
        const profitBadge = $el('metric-profit-badge');
        const roiEl = $el('metric-roi');
        const roiBadge = $el('metric-roi-badge');

        const equity = snap.equity || 0;
        window._lastEquity = equity;
        window._lastEquityTime = Date.now();
        const cash = snap.cash || 0;
        const portfolio = snap.portfolio_value || 0;
        const pnl = snap.pnl || 0;
        const roi = equity > 0 ? (pnl / equity) * 100.0 : 0;

        if (investedEl) investedEl.innerText = formatMoney(portfolio);
        if (availEl) availEl.innerText = formatMoney(cash);
        if (profitEl) {
            profitEl.innerText = formatMoney(pnl);
            profitEl.style.color = pnlColor(pnl);
        }
        if (profitBadge) {
            const arrow = pnl > 0 ? '↑' : pnl < 0 ? '↓' : '—';
            profitBadge.innerText = `${arrow} ${formatMoney(pnl)}`;
            applyBadgeColor(profitBadge, pnl);
        }
        if (roiEl) {
            roiEl.innerText = `${roi.toFixed(2)}%`;
            roiEl.style.color = pnlColor(roi);
        }
        if (roiBadge) {
            const arrow = roi > 0 ? '↑' : roi < 0 ? '↓' : '—';
            roiBadge.innerText = `${arrow} ${roi.toFixed(2)}%`;
            applyBadgeColor(roiBadge, roi);
        }

        const acctCash = $el('acct-cash');
        const acctEquity = $el('acct-equity');
        const acctPnl = $el('acct-pnl');
        if (acctCash) acctCash.innerText = formatMoney(cash);
        if (acctEquity) acctEquity.innerText = formatMoney(equity);
        if (acctPnl) acctPnl.innerText = formatMoney(pnl);
        updateWalletUI();

        window._lastPositions = snap.positions || [];
        renderPositions(window._lastPositions);
        if (typeof renderManualAssetList === 'function') renderManualAssetList();
    } catch (e) {
        // Silent fail if Alpaca unavailable
    }
}

async function updateStrategyScore() {
    try {
        const res = await invoke('get_strategy_score', { symbol: currentSymbol, timeframe: currentTF });
        if (!res) return;
        const scoreEl = $el('strategy-score');
        const sigEl = $el('strategy-signal');
        const trendEl = $el('strategy-trend');
        const rsiEl = $el('rsi-val');
        const badgeEl = $el('strategy-badge');
        if (scoreEl) {
            scoreEl.innerText = res.score.toFixed(1);
            scoreEl.style.color = pnlColor(res.score);
        }
        if (sigEl) {
            sigEl.innerText = res.signal;
            sigEl.classList.remove('sub-positive', 'sub-negative', 'sub-neutral');
            if (res.signal === 'LONG') sigEl.classList.add('sub-positive');
            else if (res.signal === 'SHORT') sigEl.classList.add('sub-negative');
            else sigEl.classList.add('sub-neutral');
        }
        if (trendEl) {
            trendEl.innerText = `${res.trend} | ADX ${res.adx.toFixed(1)} | GPU ${res.gpu_boost.toFixed(1)}`;
        }
        if (rsiEl) {
            rsiEl.innerText = res.rsi.toFixed(1);
            if (res.rsi >= 70) rsiEl.style.color = '#FF2B2B';
            else if (res.rsi <= 30) rsiEl.style.color = '#00BFFF';
            else rsiEl.style.color = '';
        }
        if (badgeEl) {
            badgeEl.innerText = `${res.signal} ${res.score.toFixed(0)}`;
            if (res.signal === 'LONG') {
                badgeEl.style.color = '#00BFFF';
                badgeEl.style.borderColor = '#00BFFF';
            } else if (res.signal === 'SHORT') {
                badgeEl.style.color = '#FF2B2B';
                badgeEl.style.borderColor = '#FF2B2B';
            } else {
                badgeEl.style.color = '#888';
                badgeEl.style.borderColor = '#333';
            }
        }
    } catch (e) {
        // silent
    }
}

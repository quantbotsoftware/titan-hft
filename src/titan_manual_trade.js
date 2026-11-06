function wireClearCacheButton() {
    const btn = $el('btn-clear-cache');
    if (!btn) return;
    btn.addEventListener('click', async () => {
        try {
            await invoke('clear_cache', {});
            showCyberToast('Cache cleared', 'info');
        } catch (e) {
            showCyberToast('Clear cache error: ' + normalizeError(e), 'error');
        }
    });
}

function setToggleChecked(id, value) {
    const el = $el(id);
    if (!el) return;
    const box = el.querySelector('.chk-box');
    if (!box) return;
    if (value) box.classList.add('checked');
    else box.classList.remove('checked');
}

function setSliderValue(id, displayId, value, suffix = "") {
    const el = $el(id);
    const disp = $el(displayId);
    if (!el) return;
    el.value = value;
    if (disp) disp.innerText = value + suffix;
    if (id === 'slider-risk') {
        const riskTarget = $el('risk-target');
        if (riskTarget) riskTarget.innerText = value;
    }
}

function setSelectValue(id, value) {
    const el = $el(id);
    if (!el || !value) return;
    el.value = value;
}

async function loadSettingsFromBackend() {
    try {
        const s = await invoke('get_settings_snapshot', {});
        if (!s) return;

        const clamp = (val, min, max) => Math.min(max, Math.max(min, val));
        const refreshVal = clamp(Number(s.refresh_rate || 60), 5, 60);
        const riskVal = clamp(Number(s.risk_per_trade || 10), 10, 100);
        const backtestVal = clamp(Number(s.backtest_days || 30), 5, 90);

        setToggleChecked('chk-loop-top', s.loop_active);
        setToggleChecked('chk-scan', s.omni_scan_active);
        setToggleChecked('chk-auto', s.auto_pilot);
        setToggleChecked('chk-auto-entry', s.auto_entry);
        setToggleChecked('chk-aggressive', s.aggressive_mode);
        setToggleChecked('chk-short', s.allow_short);
        setToggleChecked('chk-extended', s.enable_extended);
        setToggleChecked('chk-force-manual', s.force_manual_tf);
        setToggleChecked('chk-gpu-backtest', s.gpu_backtest);
        setToggleChecked('chk-gpu-sweep', s.gpu_sweep);
        setToggleChecked('chk-gpu-deep', s.gpu_deep_focus);
        setToggleChecked('chk-ai-scan-active', s.ai_scan_active);

        setSliderValue('slider-refresh', 'val-refresh', refreshVal);
        setSliderValue('slider-risk', 'val-risk', riskVal);
        setSliderValue('slider-backtest', 'val-backtest', backtestVal);

        setSelectValue('select-manual-res', s.manual_res);

        window._maxManualOrderUsd = Number(s.max_manual_order_usd || 50000);

        if (s.force_manual_tf && s.manual_res) {
            setTF(s.manual_res, null, { force: true });
        }

        if (s.active_symbol && typeof s.active_symbol === 'string') {
            const norm = normalizeSymbol(s.active_symbol);
            if (norm) {
                setSymbol(norm, { force: true });
            }
        }

        const tradeEl = $el('bot-trade-size');
        if (tradeEl && s.bot_trade_size) tradeEl.value = s.bot_trade_size;
        const breakerEl = $el('circuit-breaker');
        const breakerDescEl = $el('circuit-breaker-desc');
        if (breakerEl && s.circuit_breaker_limit) {
            breakerEl.value = s.circuit_breaker_limit;
            if (breakerDescEl) breakerDescEl.innerText = `Stop la -$${Number(s.circuit_breaker_limit).toLocaleString()} pierdere zilnică.`;
        }

        if (s.scan_params) {
            const sp = s.scan_params;
            const setVal = (id, v) => { const el = $el(id); if (el) el.value = v; };
            setVal('md-price-min', sp.price_min || 1);
            setVal('md-price-max', sp.price_max || 500);
            setVal('md-vol-min', (sp.volume_min || 500000) / 1_000_000);
            setVal('md-rvol-min', sp.rvol_min || 1);
            setVal('md-gap-min', sp.gap_min || 0);
            setSliderValue('md-change-min', 'md-change-val', sp.change_min || 1);
            setSliderValue('md-max-candidates', 'md-max-candidates-val', sp.max_candidates || 100);
            setSliderValue('md-scan-interval', 'md-scan-interval-val', sp.scan_interval_minutes || 30);
            setToggleChecked('chk-require-catalyst', sp.require_catalyst || false);
            const scoreSel = $el('md-score-preset');
            if (scoreSel && sp.score_preset) scoreSel.value = sp.score_preset;
        }

        if (s.md_auto_scan_active) {
            const autoBtn = $el('btn-md-auto-scan');
            if (autoBtn && !_isAutoScanActive()) autoBtn.click();
        }

        refreshSidebarState();
    } catch (e) {
        // ignore settings load errors
    }
}

function wireSymbolTabs() {
    const btnWl = $el('btn-sb-open-watchlist');
    const btnHft = $el('btn-sb-open-hft');
    if (btnWl) btnWl.addEventListener('click', () => switchTab(9));
    if (btnHft) btnHft.addEventListener('click', () => switchTab(10));
}

function renderManualAssetList() {
    const container = $el('mt-asset-list');
    const emptyMsg = $el('mt-empty-msg');
    const countEl = $el('mt-count');
    if (!container) return;

    const manualSymbols = Object.entries(_positionModes)
        .filter(([_, mode]) => mode === 'manual')
        .map(([sym]) => sym);

    if (countEl) countEl.textContent = `(${manualSymbols.length})`;

    if (manualSymbols.length === 0) {
        container.innerHTML = '';
        if (emptyMsg) emptyMsg.style.display = 'block';
        return;
    }
    if (emptyMsg) emptyMsg.style.display = 'none';

    container.innerHTML = manualSymbols.map(sym => {
        const posData = (window._lastPositions || []).find(p => p.symbol === sym);
        const hasPos = !!posData;
        const pnl = posData ? (posData.unrealized_pl || 0) : 0;
        const pnlPct = posData ? ((posData.unrealized_plpc || 0) * 100) : 0;
        const pnlClass = pnl > 0 ? 'asset-positive' : pnl < 0 ? 'asset-negative' : 'asset-neutral';
        const side = posData ? (posData.side || '').toUpperCase() : '';
        const sideClass = side === 'SHORT' ? 'asset-short' : side === 'LONG' ? 'asset-long' : 'asset-idle';
        const closeSide = side === 'SHORT' ? 'buy' : 'sell';
        const qty = posData ? Math.abs(posData.qty || 0) : 0;

        const cardClass = side === 'LONG' ? 'card-long' : side === 'SHORT' ? 'card-short' : 'card-idle';

        return `<div class="mt-asset-card ${cardClass}">
            <span class="asset-symbol" data-mt-chart="${escapeHtml(sym)}">${escapeHtml(sym)}</span>
            ${hasPos ? `<span class="${sideClass}" style="font-size:14px;font-weight:800;margin-right:15px;">${escapeHtml(side)}</span>` : '<span class="asset-idle" style="font-size:14px;font-weight:800;margin-right:15px;">IDLE</span>'}
            ${hasPos ? `<span class="${pnlClass}" style="font-size:15px;min-width:100px;text-align:right;font-weight:bold;">${pnl >= 0 ? '+' : ''}$${pnl.toFixed(2)} (${pnlPct.toFixed(1)}%)</span>` : ''}
            <span class="flex-1"></span>
            ${hasPos ? `
                <select data-mt-partial="${escapeHtml(sym)}" data-mt-partial-side="${escapeHtml(closeSide)}" data-mt-partial-qty="${qty}" class="mt-partial-sel">
                    <option value="100" class="cyber-opt">CLOSE 100%</option>
                    <option value="75" class="cyber-opt">CLOSE 75%</option>
                    <option value="50" class="cyber-opt">CLOSE 50%</option>
                    <option value="25" class="cyber-opt">CLOSE 25%</option>
                </select>
                <button data-mt-close="${escapeHtml(sym)}" data-mt-close-side="${escapeHtml(closeSide)}" data-mt-close-qty="${qty}" class="mt-btn-close">CLOSE</button>
            ` : ''}
            <button data-mt-to-auto="${escapeHtml(sym)}" class="mt-btn-auto">AUTO</button>
            <button data-mt-remove="${escapeHtml(sym)}" class="mt-btn-remove">X</button>
        </div>`;
    }).join('');

    container.querySelectorAll('[data-mt-chart]').forEach(el => {
        el.addEventListener('click', () => {
            const sym = el.getAttribute('data-mt-chart');
            if (sym && typeof setSymbol === 'function') setSymbol(sym, { force: true });
            invoke('set_active_symbol', { symbol: sym }).catch(e => TitanError.report('trade:set-symbol', e));
        });
    });

    container.querySelectorAll('[data-mt-close]').forEach(el => {
        el.addEventListener('click', async () => {
            const sym = el.getAttribute('data-mt-close');
            const side = el.getAttribute('data-mt-close-side');
            const fullQty = parseFloat(el.getAttribute('data-mt-close-qty'));
            if (!sym || !side || !fullQty) return;
            const partialSel = container.querySelector(`[data-mt-partial="${sym}"]`);
            const pct = partialSel ? parseInt(partialSel.value) : 100;
            const qty = Math.max(0.0001, (fullQty * pct / 100 * 1000000) / 1000000);
            const label = pct < 100 ? `PARTIAL CLOSE ${pct}% of ${sym}? (${side} ${qty.toFixed(4)})` : `Close ${sym}? (${side} ${fullQty})`;
            if (!(await cyberConfirm(label))) return;
            try {
                await invoke('submit_manual_order', { order: { symbol: sym, side, share_qty: qty } });
                el.textContent = 'DONE';
                el.style.color = 'var(--neon-cyan, #00f3ff)';
                await updateAccount();
                setTimeout(renderManualAssetList, 1000);
            } catch (e) { showCyberToast('Close failed: ' + normalizeError(e), 'error'); }
        });
    });

    container.querySelectorAll('[data-mt-to-auto]').forEach(el => {
        el.addEventListener('click', async () => {
            const sym = el.getAttribute('data-mt-to-auto');
            await invoke('set_position_mode', { symbol: sym, mode: 'auto' });
            _positionModes[sym] = 'auto';
            renderManualAssetList();
        });
    });

    container.querySelectorAll('[data-mt-remove]').forEach(el => {
        el.addEventListener('click', async () => {
            const sym = el.getAttribute('data-mt-remove');
            delete _positionModes[sym];
            await invoke('set_position_mode', { symbol: sym, mode: 'auto' });
            renderManualAssetList();
        });
    });
}

const _mtRecentTrades = [];
let _mtCachedPrice = 0;
let _mtPriceDebounce = null;

function mtUpdatePricePreview() {
    clearTimeout(_mtPriceDebounce);
    _mtPriceDebounce = setTimeout(async () => {
        const sym = ($el('mt-symbol')?.value || '').trim().toUpperCase();
        const preview = $el('mt-price-preview');
        if (!sym || sym.length < 1) { if (preview) preview.style.display = 'none'; _mtCachedPrice = 0; return; }
        try {
            const prices = window._lastPrices || {};
            const tsMap = window._lastPriceTimestamps || {};
            const priceKey = typeof normalizeSymbol === 'function' ? normalizeSymbol(sym).replace(/\//g, '').toUpperCase() : sym.replace(/\//g, '').toUpperCase();
            let price = prices[priceKey] ?? prices[sym] ?? 0;
            let usedStreamCache = price > 0;
            if (price <= 0) {
                const snap = await invoke('get_account_snapshot', {});
                if (snap && snap.positions) {
                    const pos = snap.positions.find(p => p.symbol === sym);
                    if (pos) price = pos.current_price || 0;
                }
            }
            _mtCachedPrice = price;
            const streamTs = tsMap[priceKey] ?? tsMap[sym];
            const priceStale = usedStreamCache && (streamTs == null || (Date.now() - streamTs > 30000));
            if (price > 0 && preview) {
                const pc = priceStale ? '#FFA500' : '#00BFFF';
                const suf = priceStale ? ' <span style="color:#FFA500;font-weight:700;">(STALE)</span>' : '';
                preview.innerHTML = `<span style="color:#888;">PRICE:</span> <span style="color:${pc};">$${price.toFixed(2)}</span>${suf}`;
                preview.style.display = 'block';
            } else if (preview) {
                preview.innerHTML = `<span style="color:#888;">PRICE:</span> <span style="color:#666;">fetching...</span>`;
                preview.style.display = 'block';
                setTimeout(() => {
                    if (_mtCachedPrice <= 0 && preview.style.display !== 'none') {
                        preview.innerHTML = `<span style="color:#888;">PRICE:</span> <span style="color:#FF2B2B;">N/A</span>`;
                    }
                }, 5000);
            }
            mtUpdateQtyPreview();
        } catch (e) { TitanError.report('trade:price-stream', e); }
    }, 300);
}

function mtUpdateQtyPreview() {
    const val = Number($el('mt-usd')?.value || 0);
    const qtyPrev = $el('mt-qty-preview');
    const isShares = $el('mt-mode-shares')?.checked;

    if (val > 0 && _mtCachedPrice > 0 && qtyPrev) {
        if (isShares) {
            const totalUsd = (val * _mtCachedPrice);
            qtyPrev.innerHTML = `<span style="color:#888;">Total:</span> <span style="color:#FFA500;">$${totalUsd.toFixed(2)}</span> @ <span style="color:#00BFFF;">$${_mtCachedPrice.toFixed(2)}</span>`;
        } else {
            const shares = (val / _mtCachedPrice);
            qtyPrev.innerHTML = `<span style="color:#888;">${shares.toFixed(4)} shares</span> @ <span style="color:#00BFFF;">$${_mtCachedPrice.toFixed(2)}</span>`;
        }
        qtyPrev.style.display = 'block';
    } else if (qtyPrev) { qtyPrev.style.display = 'none'; }
}

function mtAddRecentTrade(symbol, side, usd, result) {
    _mtRecentTrades.unshift({ symbol, side, usd, result, time: new Date().toLocaleTimeString() });
    if (_mtRecentTrades.length > 5) _mtRecentTrades.pop();
    const el = $el('mt-recent-trades');
    if (!el) return;
    el.innerHTML = _mtRecentTrades.map(t => {
        const sideClr = sideColor(t.side);
        const ok = t.result.includes('SENT') || t.result.includes('OK');
        return `<div style="font-size:9px;padding:2px 0;border-bottom:1px solid #111;display:flex;gap:4px;">
            <span style="color:#555;">${escapeHtml(t.time)}</span>
            <span style="color:${sideClr};font-weight:700;">${escapeHtml(t.side.toUpperCase())}</span>
            <span style="color:#FFD700;">${escapeHtml(t.symbol)}</span>
            <span style="color:#888;">$${t.usd}</span>
            <span style="color:${ok ? '#00BFFF' : '#FF2B2B'};">${ok ? 'OK' : 'FAIL'}</span>
        </div>`;
    }).join('');
}

let _mtExecCooldown = false;

function mtShowConfirmModal(details) {
    return new Promise((resolve) => {
        const overlay = document.createElement('div');
        overlay.className = 'mt-confirm-overlay';
        const modal = document.createElement('div');
        modal.className = 'mt-confirm-modal';
        const sideColor = details.side === 'buy' ? 'val' : 'val-red';
        const sideLabel = details.side === 'buy' ? 'LONG' : 'SHORT';
        const orderTypeLabel = (details.orderType || 'market').toUpperCase();

        const equity = window._lastEquity || 0;
        const totalNum = parseFloat(details.total) || 0;
        const equityPct = equity > 0 ? ((totalNum / equity) * 100).toFixed(2) : '--';
        const slNum = details.sl ? parseFloat(details.sl) : 0;
        const priceNum = _mtCachedPrice || parseFloat(details.price?.replace('$','')) || 0;
        let rrRatio = '--';
        if (details.tp && slNum > 0 && priceNum > 0) {
            const tpNum = parseFloat(details.tp);
            const reward = Math.abs(tpNum - priceNum);
            const risk = Math.abs(priceNum - slNum);
            rrRatio = risk > 0 ? (reward / risk).toFixed(2) : '--';
        }
        const breakerStatus = window._breakerStatus || 'NORMAL';
        const breakerColor = breakerStatus === 'NORMAL' ? '#00BFFF' : '#FF2B2B';
        
        const h = escapeHtml;
        modal.innerHTML = `
            <div class="mt-confirm-title">⚡ CONFIRM TRADE</div>
            <div class="mt-confirm-row"><span class="lbl">Symbol</span><span class="val-gold">${h(details.symbol)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Side</span><span class="${sideColor}">${h(sideLabel)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Order Type</span><span class="val">${h(orderTypeLabel)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Shares</span><span class="val">${h(details.shares)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Price</span><span class="val">${h(details.price)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Total</span><span class="val-gold">$${h(details.total)}</span></div>
            <div class="mt-confirm-row"><span class="lbl">Equity %</span><span class="val">${h(equityPct)}%</span></div>
            ${rrRatio !== '--' ? `<div class="mt-confirm-row"><span class="lbl">Risk/Reward</span><span class="val">1:${h(rrRatio)}</span></div>` : ''}
            <div class="mt-confirm-row"><span class="lbl">Circuit Breaker</span><span style="color:${breakerColor};font-weight:700;">${h(breakerStatus)}</span></div>
            ${details.tp ? `<div class="mt-confirm-row"><span class="lbl">Take Profit</span><span class="val">$${h(details.tp)}</span></div>` : ''}
            ${details.sl ? `<div class="mt-confirm-row"><span class="lbl">Stop Loss</span><span class="val-red">$${h(details.sl)}</span></div>` : ''}
            <div class="mt-confirm-btns">
                <button class="mt-confirm-ok">CONFIRM</button>
                <button class="mt-confirm-cancel">CANCEL</button>
            </div>
        `;
        document.body.appendChild(overlay);
        document.body.appendChild(modal);

        const cleanup = (result) => {
            overlay.remove();
            modal.remove();
            document.removeEventListener('keydown', escHandler);
            resolve(result);
        };
        const escHandler = (e) => { if (e.key === 'Escape') cleanup(false); };
        document.addEventListener('keydown', escHandler);
        overlay.addEventListener('click', () => cleanup(false));
        modal.querySelector('.mt-confirm-ok').addEventListener('click', () => cleanup(true));
        modal.querySelector('.mt-confirm-cancel').addEventListener('click', () => cleanup(false));
    });
}

function mtUpdateSideBanner() {
    const side = $el('mt-side')?.value || 'buy';
    const banner = $el('mt-side-banner');
    const sideSelect = $el('mt-side');
    if (banner) {
        if (side === 'sell') {
            banner.className = 'mt-side-banner side-short';
            banner.textContent = '⬇ SHORT POSITION';
        } else {
            banner.className = 'mt-side-banner side-long';
            banner.textContent = '⬆ LONG POSITION';
        }
        banner.style.display = 'block';
    }
    if (sideSelect) {
        sideSelect.classList.toggle('side-sell-active', side === 'sell');
    }
}

function wireTradeForm() {
    const symInput = $el('mt-symbol');
    const usdInput = $el('mt-usd');
    const orderTypeSelect = $el('mt-order-type');

    if (symInput) symInput.addEventListener('input', mtUpdatePricePreview);
    if (usdInput) usdInput.addEventListener('input', mtUpdateQtyPreview);

    const sideSelect = $el('mt-side');
    if (sideSelect) {
        sideSelect.addEventListener('change', mtUpdateSideBanner);
        mtUpdateSideBanner();
    }

    document.querySelectorAll('input[name="mt-amount-mode"]').forEach(radio => {
        radio.addEventListener('change', () => {
            const isShares = $el('mt-mode-shares')?.checked;
            if (usdInput) {
                usdInput.placeholder = isShares ? "Number of Shares" : "USD Amount";
                usdInput.value = "";
            }
            mtUpdateQtyPreview();
        });
    });

    if (orderTypeSelect) {
        orderTypeSelect.addEventListener('change', () => {
            const limitRow = $el('mt-limit-row');
            if (limitRow) limitRow.style.display = orderTypeSelect.value !== 'market' ? 'flex' : 'none';
        });
        const limitRow = $el('mt-limit-row');
        if (limitRow) limitRow.style.display = orderTypeSelect.value !== 'market' ? 'flex' : 'none';
    }
}

function wireRiskCalc() {
    const symInput = $el('mt-symbol');
    const usdInput = $el('mt-usd');
    const riskCalcBtn = $el('btn-mt-riskcalc');
    if (!riskCalcBtn) return;

    riskCalcBtn.addEventListener('click', async () => {
        const sym = (symInput?.value || '').trim().toUpperCase();
        const sl = Number($el('mt-sl')?.value || 0);
        const equity = window._lastEquity || 0;
        const price = _mtCachedPrice || 0;
        const resultEl = $el('mt-riskcalc-result');
        if (!sym || price <= 0 || sl <= 0 || equity <= 0) {
            if (resultEl) resultEl.textContent = 'Need symbol, SL, and equity';
            return;
        }
        try {
            const s = await invoke('get_settings_snapshot', {});
            const riskPct = s?.risk_per_trade || 2;
            const res = await invoke('calc_position_size', {
                equity, riskPct, entryPrice: price, stopLossPrice: sl
            });
            if (resultEl) {
                resultEl.innerHTML = `<span style="color:#FFD700;">${res.qty.toFixed(2)} shares</span> <span style="color:#888;">($${res.notional.toFixed(0)}, risk $${res.risk_amount.toFixed(0)})</span>`;
            }
            const isShares = $el('mt-mode-shares')?.checked;
            if (isShares && usdInput) {
                usdInput.value = Math.floor(res.qty);
            } else if (usdInput) {
                usdInput.value = Math.floor(res.notional);
            }
            mtUpdateQtyPreview();
        } catch (e) {
            if (resultEl) resultEl.textContent = 'Calc error: ' + normalizeError(e);
        }
    });
}

function wireQuickAmounts() {
    const usdInput = $el('mt-usd');
    document.querySelectorAll('.mt-quick-btns button[data-amt]').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.mt-quick-btns button[data-amt]').forEach(b => b.classList.remove('mt-amt-active'));
            btn.classList.add('mt-amt-active');

            const amt = btn.getAttribute('data-amt');
            const allinHint = $el('mt-allin-est-hint');
            if (amt === 'allin') {
                const eqStale = !window._lastEquityTime || (Date.now() - window._lastEquityTime > 60000);
                if (allinHint) { allinHint.style.display = eqStale ? 'inline' : 'none'; }
                try {
                    const eq = window._lastEquity || 10000;
                    if (usdInput) usdInput.value = Math.floor(eq * 0.95);
                } catch (_) { if (usdInput) usdInput.value = 10000; }
            } else {
                if (allinHint) allinHint.style.display = 'none';
                if (usdInput) usdInput.value = amt;
            }
            mtUpdateQtyPreview();
        });
    });
}

async function executeTradeFlow() {
    const btnExec = $el('btn-mt-execute');
    const statusEl = $el('mt-status');
    const symInput = $el('mt-symbol');
    const usdInput = $el('mt-usd');
    const orderTypeSelect = $el('mt-order-type');

    if (_mtExecCooldown || btnExec?.disabled) return;

    const symbol = (symInput?.value || '').trim().toUpperCase();
    const side = $el('mt-side')?.value || 'buy';
    const usd = Number(usdInput?.value || 0);
    const tp = Number($el('mt-tp')?.value || 0) || null;
    const sl = Number($el('mt-sl')?.value || 0) || null;
    const orderType = orderTypeSelect?.value || 'market';

    const isSharesMode = $el('mt-mode-shares')?.checked;
    if (!symbol) { if (statusEl) statusEl.innerText = 'Enter a symbol'; return; }
    if (usd <= 0) { if (statusEl) statusEl.innerText = isSharesMode ? 'Enter shares > 0' : 'Enter USD > 0'; return; }
    const _maxOrd = window._maxManualOrderUsd || 50000;
    if (!isSharesMode && usd > _maxOrd) { if (statusEl) statusEl.innerText = `Max $${_maxOrd.toLocaleString()}`; return; }

    const shares = _mtCachedPrice > 0 ? (isSharesMode ? usd.toString() : (usd / _mtCachedPrice).toFixed(4)) : '?';
    const priceStr = _mtCachedPrice > 0 ? `$${_mtCachedPrice.toFixed(2)}` : 'market';

    const confirmed = await mtShowConfirmModal({
        symbol, side,
        shares,
        price: priceStr,
        total: isSharesMode && _mtCachedPrice > 0 ? (usd * _mtCachedPrice).toFixed(2) : usd.toFixed(2),
        tp: tp ? tp.toFixed(2) : null,
        sl: sl ? sl.toFixed(2) : null,
        orderType
    });
    if (!confirmed) return;

    if (statusEl) { statusEl.innerText = 'Executing...'; statusEl.style.color = NEON_CYAN || '#00FFFF'; }
    btnExec.disabled = true;
    btnExec.textContent = 'SENDING...';
    _mtExecCooldown = true;

    try {
        const isShares = $el('mt-mode-shares')?.checked;
        const orderPayload = { symbol, side };
        if (isShares) {
            orderPayload.share_qty = usd;
        } else {
            orderPayload.usd_amount = usd;
        }

        if (tp) orderPayload.take_profit = tp;
        if (sl) orderPayload.stop_loss = sl;
        if (orderType !== 'market') {
            orderPayload.order_type = orderType;
            const lp = Number($el('mt-limit-price')?.value || 0);
            if (lp > 0) orderPayload.limit_price = lp;
        }

        const res = await invoke('submit_manual_order', { order: orderPayload });
        if (statusEl) { statusEl.innerText = res; statusEl.style.color = '#00BFFF'; }
        await invoke('set_position_mode', { symbol: symbol, mode: 'manual' });
        _positionModes[symbol] = 'manual';
        await updateAccount();
        renderManualAssetList();
        mtAddRecentTrade(symbol, side, usd, res);
    } catch (e) {
        const errMsg = 'Error: ' + normalizeError(e);
        if (statusEl) { statusEl.innerText = errMsg; statusEl.style.color = '#FF2B2B'; }
        mtAddRecentTrade(symbol, side, usd, errMsg);
    } finally {
        btnExec.disabled = false;
        setTimeout(() => { btnExec.textContent = 'ENGAGE TRADE'; }, 1200);
        setTimeout(() => { _mtExecCooldown = false; }, 2000);
    }
}

function wireWatchlistModal() {
    const btnInjectWl = $el('btn-mt-inject-wl');
    if (!btnInjectWl) return;

    btnInjectWl.addEventListener('click', async () => {
        try {
            const favs = await invoke('get_favorites', {});
            if (!favs || !favs.length) { showCyberToast('Watchlist is empty', 'warning'); return; }
            const symbolInput = $el('mt-symbol');
            if (favs.length === 1) {
                if (symbolInput) { symbolInput.value = favs[0]; mtUpdatePricePreview(); }
                return;
            }
            const listHtml = favs.map(s => `<div style="display:inline-block;margin:2px;"><button class="wl-pick-btn" data-pick="${escapeHtml(s)}" style="font-size:10px;padding:3px 8px;background:#1a1a1a;border:1px solid #FFD700;color:#FFD700;border-radius:3px;cursor:pointer;font-weight:700;">${escapeHtml(s)}</button></div>`).join('');

            const overlay = document.createElement('div');
            overlay.className = 'mt-wl-overlay';
            overlay.style.cssText = 'position:fixed;inset:0;background:rgba(0,0,0,0.5);z-index:99998;';
            const popup = document.createElement('div');
            popup.style.cssText = 'position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);z-index:99999;background:#0a0a1a;border:2px solid #FFD700;border-radius:8px;padding:16px;max-width:400px;max-height:60vh;overflow-y:auto;';
            popup.innerHTML = `<button type="button" id="wl-pick-x" aria-label="Close" style="position:absolute;top:8px;right:8px;width:28px;height:28px;padding:0;line-height:1;border:1px solid #ff4444;background:#330000;color:#ff4444;border-radius:4px;cursor:pointer;font-size:16px;font-weight:700;">×</button><div style="color:#FFD700;font-size:12px;font-weight:700;margin-bottom:8px;padding-right:32px;">SELECT SYMBOL FROM WATCHLIST</div>${listHtml}<div style="margin-top:8px;text-align:right;"><button id="wl-pick-close" style="font-size:10px;padding:4px 12px;background:#330000;border:1px solid #ff4444;color:#ff4444;border-radius:3px;cursor:pointer;">CLOSE</button></div>`;

            document.body.appendChild(overlay);
            document.body.appendChild(popup);

            const closePopup = () => { overlay.remove(); popup.remove(); document.removeEventListener('keydown', escWl); };
            const escWl = (e) => { if (e.key === 'Escape') closePopup(); };
            document.addEventListener('keydown', escWl);
            overlay.addEventListener('click', (ev) => { if (ev.target === overlay) closePopup(); });

            popup.addEventListener('click', (ev) => ev.stopPropagation());

            popup.querySelectorAll('.wl-pick-btn').forEach(btn => {
                btn.addEventListener('click', () => {
                    if (symbolInput) { symbolInput.value = btn.getAttribute('data-pick'); mtUpdatePricePreview(); }
                    closePopup();
                });
            });
            popup.querySelector('#wl-pick-close')?.addEventListener('click', closePopup);
            popup.querySelector('#wl-pick-x')?.addEventListener('click', closePopup);
        } catch (e) { TitanError.report('trade:favorites-popup', e); }
    });
}

function wireAssetModeButtons() {
    const btnAddManual = $el('btn-mt-add-manual');
    if (btnAddManual) {
        btnAddManual.addEventListener('click', async () => {
            const sym = prompt('Enter symbol to add as MANUAL asset:');
            if (!sym) return;
            const symbol = sym.trim().toUpperCase();
            if (!symbol) return;
            await invoke('set_position_mode', { symbol, mode: 'manual' });
            _positionModes[symbol] = 'manual';
            renderManualAssetList();
        });
    }

    const btnAllAuto = $el('btn-mt-all-auto');
    if (btnAllAuto) {
        btnAllAuto.addEventListener('click', async () => {
            const manualSyms = Object.entries(_positionModes).filter(([_, m]) => m === 'manual').map(([s]) => s);
            if (!manualSyms.length) { showCyberToast('No manual assets to switch', 'warning'); return; }
            if (!(await cyberConfirm(`Switch ${manualSyms.length} asset(s) to AUTO mode?`))) return;
            for (const sym of manualSyms) {
                await invoke('set_position_mode', { symbol: sym, mode: 'auto' });
                _positionModes[sym] = 'auto';
            }
            renderManualAssetList();
        });
    }

    renderManualAssetList();
}

function wireManualTrade() {
    wireTradeForm();
    wireRiskCalc();
    wireQuickAmounts();

    const btnExec = $el('btn-mt-execute');
    if (btnExec) btnExec.addEventListener('click', executeTradeFlow);

    const mtPanel = $el('manual-trade-panel');
    if (mtPanel) {
        mtPanel.addEventListener('keydown', (e) => {
            const tag = e.target.tagName;
            const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
            if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey) {
                if (isInput) return;
                e.preventDefault();
                executeTradeFlow();
                return;
            }
            if (isInput) return;
            if (e.key === 'l' || e.key === 'L') {
                const sel = $el('mt-side');
                if (sel) { sel.value = 'buy'; mtUpdateSideBanner(); }
            } else if (e.key === 's' || e.key === 'S') {
                const sel = $el('mt-side');
                if (sel) { sel.value = 'sell'; mtUpdateSideBanner(); }
            }
        });
    }

    wireWatchlistModal();
    wireAssetModeButtons();
}

function initSidebarExtras() {
    const steps = [
        ['wireWalletControls', wireWalletControls],
        ['wireBotControls', wireBotControls],
        ['wireFavorites', wireFavorites],
        ['wireSymbolTabs', wireSymbolTabs],
        ['wireScanButton', wireScanButton],
        ['wireMarketDiscovery', wireMarketDiscovery],
        ['wireManualTrade', wireManualTrade],
        ['wireClearCacheButton', wireClearCacheButton],
        ['loadPositionModes', loadPositionModes],
        ['loadSettingsFromBackend', loadSettingsFromBackend],
        ['refreshSidebarState', refreshSidebarState],
    ];
    for (const [name, fn] of steps) {
        try { fn(); } catch (e) { console.error(`[TITAN initSidebarExtras] ${name} FAILED:`, e); }
    }
    setTimeout(() => initApiConnection(), 800);
}

let sidebarInitStarted = false;

function startSidebarInit() {
    if (sidebarInitStarted) return;
    sidebarInitStarted = true;
    initSidebarExtras();
}

function wirePanelActions() {
    const actionMap = {
        'toggle-dry-run': () => toggleDryRun(),
        'graceful-shutdown': () => gracefulShutdown(),
        'emergency-liquidate': () => emergencyLiquidate(),
        'reset-full-safety': () => resetFullSafety(),
        'toggle-freeze-evo': () => toggleFreezeEvo(),
        'run-backtest-now': () => runBacktestNow(),
        'show-champion-drilldown': () => { if (window._showChampionDrilldown) window._showChampionDrilldown(); },
        'chart-buy': () => quickTradeExecute('buy'),
        'chart-sell': () => quickTradeExecute('sell'),
        'open-indicators': () => openIndicators(),
        'oracle-fullscreen': () => toggleOracleFullscreen(),
    };
    document.querySelectorAll('[data-action]').forEach(btn => {
        const action = btn.dataset.action;
        if (actionMap[action]) {
            btn.addEventListener('click', (e) => {
                e.preventDefault();
                actionMap[action]();
            });
        }
    });
}

function safeStartApp() {
    const steps = [
        ['startMonitorLoop', startMonitorLoop],
        ['startSidebarInit', startSidebarInit],
        ['initHistogramPaneControls', initHistogramPaneControls],
        ['initMainChartControls', initMainChartControls],
        ['initFreePanelControls', initFreePanelControls],
        ['bindTabs', bindTabs],
        ['wirePanelActions', wirePanelActions],
    ];
    for (const [name, fn] of steps) {
        try { fn(); } catch (e) { console.error(`[TITAN safeStartApp] ${name} FAILED:`, e); }
    }
    setCmdProgress('TITAN SYSTEM ACTIVE', 0);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        safeStartApp();
    });
} else {
    safeStartApp();
}

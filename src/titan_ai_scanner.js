        (function () {
            const sigSliders = [
                { id: 'sig-max-spread', field: 'max_spread_bps', valId: 'sig-max-spread-val', fmt: v => v, div: 1 },
                { id: 'sig-cooldown', field: 'entry_cooldown_secs', valId: 'sig-cooldown-val', fmt: v => v + 's', div: 1 },
                { id: 'sig-max-pos', field: 'max_open_positions', valId: 'sig-max-pos-val', fmt: v => v, div: 1 },
                { id: 'sig-risk', field: 'risk_per_trade', valId: 'sig-risk-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'sig-atr-stop', field: 'atr_stop_multiplier', valId: 'sig-atr-stop-val', fmt: v => (v / 10).toFixed(1) + 'x', div: 10 },
                { id: 'sig-atr-take', field: 'atr_take_multiplier', valId: 'sig-atr-take-val', fmt: v => (v / 10).toFixed(1) + 'x', div: 10 },
                { id: 'sig-partial-pct', field: 'partial_take_pct', valId: 'sig-partial-pct-val', fmt: v => v + '%', div: 1 },
                { id: 'sig-max-corr', field: 'max_correlation', valId: 'sig-max-corr-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'sig-corr-window', field: 'correlation_window', valId: 'sig-corr-window-val', fmt: v => v, div: 1 },
                { id: 'sig-daily-loss', field: 'daily_loss_limit_pct', valId: 'sig-daily-loss-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
            ];

            const sigChecks = [
                { id: 'sig-allow-short', field: 'allow_short' },
                { id: 'sig-extended', field: 'enable_extended' },
                { id: 'sig-aggressive', field: 'aggressive_mode' },
                { id: 'sig-partial-enabled', field: 'partial_take_enabled' },
            ];

            const sigBypassChecks = [
                { id: 'sig-bypass-max-spread', field: 'bypass_max_spread' },
                { id: 'sig-bypass-cooldown', field: 'bypass_cooldown' },
                { id: 'sig-bypass-allow-short', field: 'bypass_allow_short' },
                { id: 'sig-bypass-extended', field: 'bypass_extended' },
                { id: 'sig-bypass-risk', field: 'bypass_risk_per_trade' },
                { id: 'sig-bypass-aggressive', field: 'bypass_aggressive' },
                { id: 'sig-bypass-atr-stop', field: 'bypass_atr_stop' },
                { id: 'sig-bypass-atr-take', field: 'bypass_atr_take' },
                { id: 'sig-bypass-partial-enabled', field: 'bypass_partial_take' },
                { id: 'sig-bypass-partial-pct', field: 'bypass_partial_pct' },
                { id: 'sig-bypass-max-corr', field: 'bypass_max_correlation' },
                { id: 'sig-bypass-corr-window', field: 'bypass_correlation_window' },
                { id: 'sig-bypass-daily-loss', field: 'bypass_daily_loss' },
            ];

            const sigBypassDimMap = [
                { bypass: 'sig-bypass-max-spread', dim: ['sig-max-spread', 'sig-max-spread-val'] },
                { bypass: 'sig-bypass-cooldown', dim: ['sig-cooldown', 'sig-cooldown-val'] },
                { bypass: 'sig-bypass-allow-short', dim: ['sig-allow-short'] },
                { bypass: 'sig-bypass-extended', dim: ['sig-extended'] },
                { bypass: 'sig-bypass-risk', dim: ['sig-risk', 'sig-risk-val'] },
                { bypass: 'sig-bypass-aggressive', dim: ['sig-aggressive'] },
                { bypass: 'sig-bypass-atr-stop', dim: ['sig-atr-stop', 'sig-atr-stop-val'] },
                { bypass: 'sig-bypass-atr-take', dim: ['sig-atr-take', 'sig-atr-take-val'] },
                { bypass: 'sig-bypass-partial-enabled', dim: ['sig-partial-enabled'] },
                { bypass: 'sig-bypass-partial-pct', dim: ['sig-partial-pct', 'sig-partial-pct-val'] },
                { bypass: 'sig-bypass-max-corr', dim: ['sig-max-corr', 'sig-max-corr-val'] },
                { bypass: 'sig-bypass-corr-window', dim: ['sig-corr-window', 'sig-corr-window-val'] },
                { bypass: 'sig-bypass-daily-loss', dim: ['sig-daily-loss', 'sig-daily-loss-val'] },
            ];

            function updateSigBypassDimming() {
                sigBypassDimMap.forEach(({ bypass, dim }) => {
                    const b = $el(bypass);
                    const on = b && b.checked;
                    dim.forEach(id => {
                        const el = $el(id);
                        if (el) el.style.opacity = on ? '0.38' : '1';
                    });
                });
            }

            const sigDefaults = {
                max_spread_bps: 50, entry_cooldown_secs: 30, max_open_positions: 3,
                risk_per_trade: 2.0, aggressive_mode: false, allow_short: false,
                enable_extended: false, atr_stop_multiplier: 1.5, atr_take_multiplier: 3.0,
                partial_take_enabled: false, partial_take_pct: 50, max_correlation: 0.70,
                correlation_window: 60, daily_loss_limit_pct: 3.0
            };

            /** Conservative intraday profile for Signal Strategy (safety rails ON) */
            const sigSmartDefaults = {
                max_spread_bps: 75, entry_cooldown_secs: 45, max_open_positions: 3,
                risk_per_trade: 2.0, aggressive_mode: false, allow_short: false,
                enable_extended: false, atr_stop_multiplier: 1.5, atr_take_multiplier: 3.0,
                partial_take_enabled: true, partial_take_pct: 50, max_correlation: 0.65,
                correlation_window: 60, daily_loss_limit_pct: 3.0
            };

            let _sigSettingsLoaded = false;

            async function persistSignalConfig() {
                const batch = {};
                for (const s of sigSliders) {
                    const slider = $el(s.id);
                    if (!slider) continue;
                    const raw = parseFloat(slider.value);
                    batch[s.field] = String(s.div > 1 ? (raw / s.div) : raw);
                }
                for (const c of sigChecks) {
                    const el = $el(c.id);
                    if (!el) continue;
                    batch[c.field] = String(el.checked);
                }
                for (const b of sigBypassChecks) {
                    const el = $el(b.id);
                    if (!el) continue;
                    batch[b.field] = String(el.checked);
                }
                await invoke('batch_update_settings', { settings: batch });
                _sigSettingsLoaded = false;
            }

            function wireSignalSettings() {
                sigSliders.forEach(s => {
                    const slider = $el(s.id);
                    const valEl = $el(s.valId);
                    if (!slider || !valEl) return;
                    slider.addEventListener('input', () => {
                        valEl.textContent = s.fmt(parseFloat(slider.value));
                    });
                });

                sigBypassChecks.forEach(b => {
                    const el = $el(b.id);
                    if (el) el.addEventListener('change', updateSigBypassDimming);
                });

                const btnSave = $el('btn-sig-save');
                if (btnSave) btnSave.addEventListener('click', async () => {
                    await persistSignalConfig();
                    if (typeof titanLogAdd === 'function') titanLogAdd('Signal Strategy config saved');
                });

                const btnSmart = $el('btn-sig-smart');
                if (btnSmart) btnSmart.addEventListener('click', async () => {
                    for (const s of sigSliders) {
                        const defVal = sigSmartDefaults[s.field];
                        if (defVal === undefined) continue;
                        const slider = $el(s.id);
                        if (!slider) continue;
                        slider.value = defVal * s.div;
                        slider.dispatchEvent(new Event('input'));
                    }
                    for (const c of sigChecks) {
                        const defVal = sigSmartDefaults[c.field];
                        const el = $el(c.id);
                        if (!el) continue;
                        el.checked = !!defVal;
                    }
                    sigBypassChecks.forEach(b => {
                        const el = $el(b.id);
                        if (el) el.checked = false;
                    });
                    updateSigBypassDimming();
                    btnSmart.disabled = true;
                    try {
                        await persistSignalConfig();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal Strategy SMART DEFAULT applied + saved');
                    } finally {
                        btnSmart.disabled = false;
                    }
                });

                const btnReset = $el('btn-sig-reset');
                if (btnReset) btnReset.addEventListener('click', async () => {
                    for (const s of sigSliders) {
                        const defVal = sigDefaults[s.field];
                        if (defVal === undefined) continue;
                        const slider = $el(s.id);
                        if (!slider) continue;
                        slider.value = defVal * s.div;
                        slider.dispatchEvent(new Event('input'));
                    }
                    for (const c of sigChecks) {
                        const defVal = sigDefaults[c.field];
                        const el = $el(c.id);
                        if (!el) continue;
                        el.checked = !!defVal;
                    }
                    sigBypassChecks.forEach(b => {
                        const el = $el(b.id);
                        if (el) el.checked = false;
                    });
                    updateSigBypassDimming();
                    await persistSignalConfig();
                    if (typeof titanLogAdd === 'function') titanLogAdd('Signal Strategy config reset to defaults');
                });

                updateSigBypassDimming();
            }

            async function loadSignalSettings() {
                try {
                    const snap = await invoke('get_settings_snapshot');
                    const sliderMap = {
                        'sig-max-spread': snap.max_spread_bps || 50,
                        'sig-cooldown': snap.entry_cooldown_secs || 30,
                        'sig-max-pos': snap.max_open_positions || 3,
                        'sig-risk': (snap.risk_per_trade || 2.0) * 10,
                        'sig-atr-stop': Math.round((snap.atr_stop_multiplier || 1.5) * 10),
                        'sig-atr-take': Math.round((snap.atr_take_multiplier || 3.0) * 10),
                        'sig-partial-pct': snap.partial_take_pct || 50,
                        'sig-max-corr': Math.round((snap.max_correlation || 0.70) * 100),
                        'sig-corr-window': snap.correlation_window || 60,
                        'sig-daily-loss': Math.round((snap.daily_loss_limit_pct || 3.0) * 10),
                    };
                    Object.entries(sliderMap).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) { el.value = val; el.dispatchEvent(new Event('input')); }
                    });
                    const checkMap = {
                        'sig-allow-short': snap.allow_short === true,
                        'sig-extended': snap.enable_extended === true,
                        'sig-aggressive': snap.aggressive_mode === true,
                        'sig-partial-enabled': snap.partial_take_enabled === true,
                    };
                    Object.entries(checkMap).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) el.checked = val;
                    });
                    const bypassMap = {
                        'sig-bypass-max-spread': snap.bypass_max_spread === true,
                        'sig-bypass-cooldown': snap.bypass_cooldown === true,
                        'sig-bypass-allow-short': snap.bypass_allow_short === true,
                        'sig-bypass-extended': snap.bypass_extended === true,
                        'sig-bypass-risk': snap.bypass_risk_per_trade === true,
                        'sig-bypass-aggressive': snap.bypass_aggressive === true,
                        'sig-bypass-atr-stop': snap.bypass_atr_stop === true,
                        'sig-bypass-atr-take': snap.bypass_atr_take === true,
                        'sig-bypass-partial-enabled': snap.bypass_partial_take === true,
                        'sig-bypass-partial-pct': snap.bypass_partial_pct === true,
                        'sig-bypass-max-corr': snap.bypass_max_correlation === true,
                        'sig-bypass-corr-window': snap.bypass_correlation_window === true,
                        'sig-bypass-daily-loss': snap.bypass_daily_loss === true,
                    };
                    Object.entries(bypassMap).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) el.checked = val;
                    });
                    updateSigBypassDimming();
                } catch (e) {
                    console.warn('loadSignalSettings error:', e);
                }
            }

            TitanBus.on('tabSwitch', function (index) {
                if (index === 12 && !_sigSettingsLoaded) {
                    _sigSettingsLoaded = true;
                    loadSignalSettings();
                    loadSignalAssets();
                }
            });

            document.addEventListener('DOMContentLoaded', wireSignalSettings);

            // ═══════════════════════════════════════════════════════════
            //  SIGNAL STRATEGY ASSET LIST
            // ═══════════════════════════════════════════════════════════
            let _sigAssets = [];
            let _sigEnrichedCache = [];
            let _sigEnrichRunning = false;

            function getSelectedSigAsset() {
                const sel = document.querySelector('input[name="sig-asset-sel"]:checked');
                return sel ? sel.value : '';
            }

            function sigFlash(msg) {
                let el = $el('sig-flash-toast');
                if (!el) {
                    el = document.createElement('div');
                    el.id = 'sig-flash-toast';
                    el.style.cssText = 'position:fixed;top:60px;left:50%;transform:translateX(-50%);z-index:999999;background:#0a2a0a;border:1px solid #00BFFF;color:#00BFFF;padding:8px 22px;border-radius:8px;font-size:13px;font-weight:700;font-family:monospace;pointer-events:none;opacity:0;transition:opacity .25s;white-space:nowrap;';
                    document.body.appendChild(el);
                }
                el.textContent = msg;
                el.style.opacity = '1';
                clearTimeout(el._tid);
                el._tid = setTimeout(() => { el.style.opacity = '0'; }, 2000);
            }

            async function loadSignalAssets() {
                try {
                    _sigAssets = await invoke('get_signal_assets');
                    renderSignalAssetTable();
                } catch (e) { console.warn('loadSignalAssets error:', e); }
            }
            window.loadSignalAssets = loadSignalAssets;

            function renderSignalAssetTable() {
                const body = $el('sig-asset-body');
                const empty = $el('sig-asset-empty');
                const countEl = $el('sig-asset-count');
                if (!body) return;

                if (countEl) countEl.textContent = 'Assets: ' + _sigAssets.length;

                if (_sigAssets.length === 0) {
                    body.innerHTML = '';
                    if (empty) empty.style.display = 'block';
                    return;
                }
                if (empty) empty.style.display = 'none';

                const enrichMap = {};
                _sigEnrichedCache.forEach(c => { enrichMap[c.symbol] = c; });

                // fmtPct, formatVolume -> titan_core.js; pctClr = pnlColor

                body.innerHTML = _sigAssets.map((sym, i) => {
                    const c = enrichMap[sym];
                    const price = c ? '$' + c.price.toFixed(2) : '--';
                    const chg = c ? c.change_percent : 0;
                    const rv = c ? c.rvol : 0;
                    const vol = c ? c.volume : 0;
                    const gap = c ? c.gap_percent : 0;
                    const floatCat = c ? c.float_category : '--';
                    const osc = c ? c.oracle_score.toFixed(1) : '--';
                    const nex = c ? c.nexus_buy_pct.toFixed(0) + '%' : '--';
                    const fus = c ? c.fusion_score.toFixed(1) : '--';
                    const sig = c ? c.fusion_signal : '--';
                    const tf = c ? c.bt_best_tf : '--';
                    const score = c ? (c.smart_score || c.ai_score || 0) : 0;
                    const sigClr = sig === 'BUY' ? '#00BFFF' : sig === 'SELL' ? '#ff4444' : sig === 'TRAIL' ? '#FFD700' : '#888';
                    const rvClr = rv >= 5 ? '#00BFFF' : rv >= 2 ? '#FFD700' : '#888';
                    const gapClr = gap > 3 ? '#00BFFF' : gap > 0 ? '#FFD700' : '#888';
                    const floatClr = floatCat === 'LOW' ? '#A855F7' : floatCat === 'MED' ? '#FFD700' : '#888';
                    const scoreClr = score >= 60 ? '#00BFFF' : score >= 30 ? '#FFD700' : '#888';
                    const safeBtf = tf !== '--' ? escapeHtml(tf) : '';

                    return `<tr class="sig-asset-row" data-symbol="${escapeHtml(sym)}" data-btf="${safeBtf}" style="cursor:pointer;">
                        <td style="text-align:center;"><input type="radio" name="sig-asset-sel" value="${escapeHtml(sym)}" style="display:none;"${i === 0 ? ' checked' : ''}><button class="sig-btn-sel tbl-badge tbl-badge-gold" data-sym="${escapeHtml(sym)}" style="${i === 0 ? 'background:rgba(255,215,0,0.15);' : ''}">SEL</button></td>
                        <td style="text-align:center;color:#FFD700;font-weight:700;">${i + 1}</td>
                        <td class="td-sym-gold">${escapeHtml(sym)}</td>
                        <td style="text-align:right;color:#ccc;">${price}</td>
                        <td style="text-align:right;color:${pnlColor(chg)};">${c ? fmtPct(chg) : '--'}</td>
                        <td style="text-align:right;color:${rvClr};">${c ? rv.toFixed(1) + 'x' : '--'}</td>
                        <td style="text-align:right;color:${gapClr};">${c ? fmtPct(gap) : '--'}</td>
                        <td style="text-align:right;color:#888;">${c ? formatVolume(vol) : '--'}</td>
                        <td style="text-align:center;color:${floatClr};">${escapeHtml(floatCat)}</td>
                        <td style="text-align:right;color:#888;">${c && c.market_cap > 0 ? (c.market_cap >= 1000 ? (c.market_cap/1000).toFixed(1) + 'B' : c.market_cap.toFixed(0) + 'M') : '—'}</td>
                        <td style="text-align:center;color:${c && c.insider_signal === 'BUY' ? '#00BFFF' : c && c.insider_signal === 'SELL' ? '#ff4444' : '#555'};">${c && c.insider_signal ? escapeHtml(c.insider_signal) : '—'}</td>
                        <td style="text-align:right;color:#00BFFF;">${osc}</td>
                        <td style="text-align:right;color:#FF69B4;">${nex}</td>
                        <td style="text-align:right;color:#FFD700;font-weight:700;">${fus}</td>
                        <td style="text-align:center;color:${sigClr};font-weight:700;">${escapeHtml(sig)}</td>
                        <td style="text-align:center;color:#aaa;">${escapeHtml(tf)}</td>
                        <td style="text-align:right;color:${scoreClr};font-weight:700;">${score > 0 ? score.toFixed(1) : '--'}</td>
                        <td style="text-align:center;"><button class="sig-btn-bt tbl-badge tbl-badge-magenta" data-sym="${escapeHtml(sym)}">BT</button></td>
                        <td style="text-align:center;"><button class="sig-btn-hft tbl-badge tbl-badge-cyan" data-sym="${escapeHtml(sym)}">+HFT</button></td>
                        <td style="text-align:center;"><button class="sig-btn-del tbl-badge tbl-badge-red" data-sym="${escapeHtml(sym)}">X</button></td>
                    </tr>`;
                }).join('');

                function highlightSelectedSel() {
                    body.querySelectorAll('.sig-btn-sel').forEach(b => { b.style.background = 'transparent'; });
                    body.querySelectorAll('.sig-asset-row').forEach(r => { r.style.background = 'transparent'; });
                    const checked = body.querySelector('input[name="sig-asset-sel"]:checked');
                    if (checked) {
                        const tr = checked.closest('tr');
                        if (tr) {
                            tr.style.background = 'rgba(255,215,0,0.08)';
                            const btn = tr.querySelector('.sig-btn-sel');
                            if (btn) btn.style.background = '#3a3a00';
                        }
                    }
                }
                highlightSelectedSel();

                body.querySelectorAll('.sig-btn-sel').forEach(btn => {
                    btn.addEventListener('click', function (e) {
                        e.stopPropagation();
                        const sym = btn.dataset.sym;
                        const radio = body.querySelector('input[name="sig-asset-sel"][value="' + sym + '"]');
                        if (radio) radio.checked = true;
                        highlightSelectedSel();
                        sigFlash(sym + ' selected for backtest');
                    });
                });

                body.querySelectorAll('.sig-asset-row').forEach(row => {
                    const sym = row.dataset.symbol;
                    const btf = row.dataset.btf;

                    row.addEventListener('click', function (e) {
                        if (e.target.closest('.sig-btn-sel') || e.target.closest('.sig-btn-bt') || e.target.closest('.sig-btn-hft') || e.target.closest('.sig-btn-del')) return;
                        if (typeof window.setSymbol === 'function') window.setSymbol(sym, { force: true });
                        window.invoke('set_active_symbol', { symbol: sym }).catch(e => TitanError.report('ai-scan:set-symbol', e));
                        if (btf && typeof window.setTF === 'function') window.setTF(btf, null, { force: true });
                        sigFlash(sym + ' loaded on chart' + (btf ? ' [' + btf + ']' : ''));
                        if (typeof window.switchTab === 'function') window.switchTab(0);
                    });
                });

                body.querySelectorAll('.sig-btn-bt').forEach(btn => {
                    btn.addEventListener('click', function (e) { e.stopPropagation(); sigBacktestOne(btn.dataset.sym); });
                });
                body.querySelectorAll('.sig-btn-hft').forEach(btn => {
                    btn.addEventListener('click', function (e) { e.stopPropagation(); sigAddToHft(btn.dataset.sym); });
                });
                body.querySelectorAll('.sig-btn-del').forEach(btn => {
                    btn.addEventListener('click', function (e) { e.stopPropagation(); sigRemoveAsset(btn.dataset.sym); });
                });
            }

            async function sigRemoveAsset(sym) {
                try {
                    _sigAssets = await invoke('remove_signal_asset', { symbol: sym });
                    renderSignalAssetTable();
                } catch (e) { console.warn('sigRemoveAsset error:', e); }
            }

            async function sigAddToHft(sym) {
                try {
                    await invoke('add_hft_target', { symbol: sym });
                    if (typeof titanLogAdd === 'function') titanLogAdd('Added ' + sym + ' to HFT Targets');
                } catch (e) { console.warn(e); }
            }

            async function sigBacktestOne(sym) {
                try {
                    const tf = '5m';
                    if (typeof titanLogAdd === 'function') titanLogAdd('Backtesting ' + sym + ' (' + tf + ')...');
                    const result = await invoke('run_backtest', { symbol: sym, timeframe: tf });
                    const msg = sym + ' BT: ' + result.trades + ' trades, ' +
                        result.win_rate.toFixed(1) + '% WR, ' +
                        result.return_pct.toFixed(2) + '% ret, Sharpe ' +
                        result.sharpe.toFixed(2);
                    if (typeof titanLogAdd === 'function') titanLogAdd(msg);
                } catch (e) {
                    if (typeof titanLogAdd === 'function') titanLogAdd('BT ERROR: ' + normalizeError(e));
                }
            }

            async function sigRefreshEnriched() {
                if (_sigEnrichRunning) return;
                _sigEnrichRunning = true;
                const btn = $el('btn-sig-refresh');
                if (btn) { btn.innerText = 'SCORING...'; btn.disabled = true; }
                try {
                    _sigEnrichedCache = await invoke('get_signal_asset_enriched', {});
                    renderSignalAssetTable();
                } catch (e) {
                    TitanError.report('sig:enrich', e);
                } finally {
                    _sigEnrichRunning = false;
                    if (btn) { btn.innerText = 'REFRESH'; btn.disabled = false; }
                }
            }

            window.sigRemoveAsset = sigRemoveAsset;
            window.sigAddToHft = sigAddToHft;
            window.sigBacktestOne = sigBacktestOne;
            window.sigFlash = sigFlash;

            document.addEventListener('DOMContentLoaded', () => {
                const addBtn = $el('btn-sig-add');
                const addInput = $el('sig-asset-input');
                if (addBtn && addInput) {
                    const doAdd = async () => {
                        const raw = addInput.value.trim().toUpperCase();
                        if (!raw) return;
                        const syms = raw.split(/[,;\s]+/).filter(Boolean);
                        for (const s of syms) {
                            try { _sigAssets = await invoke('add_signal_asset', { symbol: s }); }
                            catch (e) { console.warn(e); }
                        }
                        addInput.value = '';
                        renderSignalAssetTable();
                    };
                    addBtn.addEventListener('click', doAdd);
                    addInput.addEventListener('keydown', e => { if (e.key === 'Enter') doAdd(); });
                }

                const refreshBtn = $el('btn-sig-refresh');
                if (refreshBtn) refreshBtn.addEventListener('click', sigRefreshEnriched);

                const delAllBtn = $el('btn-sig-del-all');
                if (delAllBtn) delAllBtn.addEventListener('click', async () => {
                    if (!(await cyberConfirm('Clear all signal strategy assets?'))) return;
                    try {
                        _sigAssets = await invoke('clear_signal_assets');
                        _sigEnrichedCache = [];
                        renderSignalAssetTable();
                    } catch (e) { console.warn(e); }
                });

                const injectAi = $el('btn-sig-inject-ai');
                if (injectAi) injectAi.addEventListener('click', async () => {
                    const data = window._aiScanData || [];
                    if (!data.length) { showCyberToast('Run AI Scanner first', 'warning'); return; }
                    const syms = data.map(r => r.symbol);
                    try {
                        _sigAssets = await invoke('inject_to_signal_assets', { symbols: syms });
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Injected ' + syms.length + ' AI Scanner results → Signal Assets');
                    } catch (e) { console.warn(e); }
                });

                const injectPicks = $el('btn-sig-inject-picks');
                if (injectPicks) injectPicks.addEventListener('click', async () => {
                    const data = window._aiScanData || [];
                    const picks = data.filter(r => (r.smart_score || 0) > 20 && r.bt_trades >= 2 && r.bt_sharpe > 0)
                        .sort((a, b) => (b.smart_score || 0) - (a.smart_score || 0))
                        .slice(0, 10);
                    if (!picks.length) { showCyberToast('No smart picks available. Run AI Scanner first', 'warning'); return; }
                    const syms = picks.map(r => r.symbol);
                    try {
                        _sigAssets = await invoke('inject_to_signal_assets', { symbols: syms });
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Injected ' + syms.length + ' Smart Picks → Signal Assets');
                    } catch (e) { console.warn(e); }
                });

                const injectShark = $el('btn-sig-inject-shark');
                if (injectShark) injectShark.addEventListener('click', async () => {
                    try {
                        const res = await invoke('run_market_scan', { strategy: 'SHARK' });
                        if (!res || !res.length) { showCyberToast('Shark scan returned 0 results', 'warning'); return; }
                        const syms = res.map(r => r.symbol);
                        _sigAssets = await invoke('inject_to_signal_assets', { symbols: syms });
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Injected ' + syms.length + ' Shark results → Signal Assets');
                    } catch (e) {
                        const errMsg = typeof normalizeScanError === 'function' ? normalizeScanError(e) : normalizeError(e);
                        showCyberToast('Shark scan error: ' + errMsg, 'error');
                    }
                });

                const injectWl = $el('btn-sig-inject-wl');
                if (injectWl) injectWl.addEventListener('click', async () => {
                    try {
                        const favs = await invoke('get_favorites');
                        if (!favs || !favs.length) { showCyberToast('Watchlist is empty', 'warning'); return; }
                        _sigAssets = await invoke('inject_to_signal_assets', { symbols: favs });
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Injected ' + favs.length + ' Watchlist items → Signal Assets');
                    } catch (e) { console.warn(e); }
                });

                const toHft = $el('btn-sig-to-hft');
                if (toHft) toHft.addEventListener('click', async () => {
                    if (!_sigAssets.length) { showCyberToast('Signal asset list is empty', 'warning'); return; }
                    try {
                        await invoke('inject_signal_to_hft');
                        if (typeof titanLogAdd === 'function') titanLogAdd('Exported ' + _sigAssets.length + ' Signal Assets → HFT Targets');
                    } catch (e) { console.warn(e); }
                });

                const toManual = $el('btn-sig-to-manual');
                if (toManual) toManual.addEventListener('click', async () => {
                    if (!_sigAssets.length) { showCyberToast('Signal asset list is empty', 'warning'); return; }
                    toManual.innerText = 'INJECTING...';
                    toManual.disabled = true;
                    try {
                        for (const sym of _sigAssets) {
                            await invoke('set_position_mode', { symbol: sym, mode: 'manual' });
                        }
                        if (typeof renderManualAssetList === 'function') renderManualAssetList();
                        sigFlash(_sigAssets.length + ' assets injected to Manual Trade');
                        if (typeof titanLogAdd === 'function') titanLogAdd('Injected ' + _sigAssets.length + ' Signal Assets → Manual Trade');
                    } catch (e) { console.warn(e); }
                    finally {
                        toManual.innerText = 'INJECT TO MANUAL';
                        toManual.disabled = false;
                    }
                });

                const btAll = $el('btn-sig-backtest-all');
                if (btAll) btAll.addEventListener('click', async () => {
                    if (!_sigAssets.length) { showCyberToast('Signal asset list is empty', 'warning'); return; }
                    const days = parseInt($el('sig-bt-period')?.value || '90');
                    btAll.innerText = 'RUNNING...';
                    btAll.disabled = true;
                    const prog = $el('sig-bt-progress');
                    if (prog) { prog.style.display = 'block'; prog.textContent = 'Signal backtest: 0/' + _sigAssets.length + '...'; }
                    try {
                        const results = await invoke('run_signal_backtest_multi', { symbols: _sigAssets, periodDays: days });
                        renderSignalBtMultiResults(results);
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT multi done: ' + results.length + ' symbols');
                    } catch (e) {
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT error: ' + normalizeError(e));
                    } finally {
                        btAll.innerText = 'BACKTEST ALL';
                        btAll.disabled = false;
                        if (prog) prog.style.display = 'none';
                    }
                });

                if (window.__TAURI__) {
                    window.__TAURI__.event.listen('sig-enrich-progress', (ev) => {
                        const d = ev.payload;
                        const btn = $el('btn-sig-refresh');
                        if (!btn) return;
                        if (d.phase === 'scoring' && d.symbol) {
                            btn.innerText = 'AI: ' + d.symbol + ' (' + d.done + '/' + d.total + ')';
                        } else if (d.phase === 'snapshots') {
                            btn.innerText = 'LOADING...';
                        } else if (d.phase === 'done') {
                            btn.innerText = 'REFRESH';
                            btn.disabled = false;
                        }
                    });
                }

                // ═══════════════════════════════════════════════════════
                //  SIGNAL BACKTEST UI
                // ═══════════════════════════════════════════════════════
                const periodSlider = $el('sig-bt-period');
                const periodVal = $el('sig-bt-period-val');
                if (periodSlider && periodVal) {
                    const fmtPeriod = d => {
                        if (d >= 360) return '1yr';
                        if (d >= 30) return Math.round(d / 30) + 'mo';
                        return d + 'd';
                    };
                    periodSlider.addEventListener('input', () => {
                        periodVal.textContent = fmtPeriod(parseInt(periodSlider.value));
                    });
                    periodVal.textContent = fmtPeriod(parseInt(periodSlider.value));
                }

                const btnSigBtRun = $el('btn-sig-bt-run');
                if (btnSigBtRun) btnSigBtRun.addEventListener('click', async () => {
                    const sym = getSelectedSigAsset();
                    if (!sym) { showCyberToast('Select an asset from the Signal Strategy Assets list first', 'warning'); return; }
                    const days = parseInt($el('sig-bt-period')?.value || '90');
                    btnSigBtRun.innerText = 'RUNNING...';
                    btnSigBtRun.disabled = true;
                    const prog = $el('sig-bt-progress');
                    if (prog) { prog.style.display = 'block'; prog.textContent = 'Starting signal backtest for ' + sym + '...'; }
                    try {
                        const result = await invoke('run_signal_backtest', { symbol: sym, periodDays: days });
                        renderSignalBtResults(result);
                        const cached = _sigEnrichedCache.find(c => c.symbol === sym);
                        if (cached) {
                            cached.bt_best_tf = result.best_tf || '';
                        } else {
                            _sigEnrichedCache.push({ symbol: sym, price: 0, change_percent: 0, rvol: 0, volume: 0, oracle_score: 0, nexus_buy_pct: 0, fusion_score: 0, fusion_signal: '--', bt_best_tf: result.best_tf || '' });
                        }
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT done: ' + sym + ' best=' + result.best_tf + ' (' + result.best_return.toFixed(2) + '%)');
                    } catch (e) {
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT error: ' + normalizeError(e));
                        showCyberToast('Signal BT error: ' + normalizeError(e), 'error');
                    } finally {
                        btnSigBtRun.innerText = 'RUN SIGNAL BT';
                        btnSigBtRun.disabled = false;
                        if (prog) prog.style.display = 'none';
                    }
                });

                const btnSigBtAll = $el('btn-sig-bt-all');
                if (btnSigBtAll) btnSigBtAll.addEventListener('click', async () => {
                    if (!_sigAssets.length) { showCyberToast('Signal asset list is empty', 'warning'); return; }
                    const days = parseInt($el('sig-bt-period')?.value || '90');
                    btnSigBtAll.innerText = 'RUNNING...';
                    btnSigBtAll.disabled = true;
                    const prog = $el('sig-bt-progress');
                    if (prog) { prog.style.display = 'block'; prog.textContent = 'Signal backtest: 0/' + _sigAssets.length + ' symbols...'; }
                    try {
                        const results = await invoke('run_signal_backtest_multi', { symbols: _sigAssets, periodDays: days });
                        renderSignalBtMultiResults(results);
                        results.forEach(r => {
                            const cached = _sigEnrichedCache.find(c => c.symbol === r.symbol);
                            if (cached) { cached.bt_best_tf = r.best_tf || ''; }
                        });
                        renderSignalAssetTable();
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT multi done: ' + results.length + ' symbols');
                    } catch (e) {
                        if (typeof titanLogAdd === 'function') titanLogAdd('Signal BT multi error: ' + normalizeError(e));
                    } finally {
                        btnSigBtAll.innerText = 'BT ALL ASSETS';
                        btnSigBtAll.disabled = false;
                        if (prog) prog.style.display = 'none';
                    }
                });

                function renderSignalBtResults(result) {
                    const container = $el('sig-bt-results');
                    const summary = $el('sig-bt-summary');
                    const body = $el('sig-bt-body');
                    if (!container || !body) return;
                    container.style.display = 'block';

                    if (summary) {
                        const bClr = pnlColor(result.best_return);
                        summary.innerHTML = `
                            <div style="background:#1a1a2e; border:1px solid #FF69B4; border-radius:6px; padding:12px 18px; flex:1; min-width:140px;">
                                <div style="font-size:10px; color:#888; margin-bottom:4px;">SYMBOL</div>
                                <div style="font-size:18px; color:#FFD700; font-weight:700;">${escapeHtml(result.symbol)}</div>
                            </div>
                            <div style="background:#1a1a2e; border:1px solid #FF69B4; border-radius:6px; padding:12px 18px; flex:1; min-width:140px;">
                                <div style="font-size:10px; color:#888; margin-bottom:4px;">BEST TIMEFRAME</div>
                                <div style="font-size:18px; color:#00BFFF; font-weight:700;">${escapeHtml(result.best_tf)}</div>
                            </div>
                            <div style="background:#1a1a2e; border:1px solid #FF69B4; border-radius:6px; padding:12px 18px; flex:1; min-width:140px;">
                                <div style="font-size:10px; color:#888; margin-bottom:4px;">BEST RETURN</div>
                                <div style="font-size:18px; color:${bClr}; font-weight:700;">${result.best_return.toFixed(2)}%</div>
                            </div>
                            <div style="background:#1a1a2e; border:1px solid #FF69B4; border-radius:6px; padding:12px 18px; flex:1; min-width:140px;">
                                <div style="font-size:10px; color:#888; margin-bottom:4px;">BEST SHARPE</div>
                                <div style="font-size:18px; color:#00BFFF; font-weight:700;">${result.best_sharpe.toFixed(2)}</div>
                            </div>
                            <div style="background:#1a1a2e; border:1px solid #FF69B4; border-radius:6px; padding:12px 18px; flex:1; min-width:140px;">
                                <div style="font-size:10px; color:#888; margin-bottom:4px;">PERIOD</div>
                                <div style="font-size:18px; color:#aaa; font-weight:700;">${result.period_days}d</div>
                            </div>`;
                    }

                    const sharpeClr = v => v > 1 ? '#00BFFF' : v > 0 ? '#FFD700' : '#ff4444';

                    body.innerHTML = result.results.map((r, i) => {
                        const bg = r.best ? 'rgba(0,191,255,0.08)' : 'transparent';
                        const border = r.best ? '2px solid #00BFFF' : '1px solid #1a1a1a';
                        const badge = r.best ? '<span style="color:#00BFFF;font-weight:900;">BEST</span>' : (i + 1).toString();
                        return `<tr style="background:${bg}; border-bottom:${border};">
                            <td style="padding:6px 8px;color:${r.best ? '#00BFFF' : '#ccc'};font-weight:700;">${escapeHtml(r.tf)}</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${r.trades}</td>
                            <td style="padding:6px 8px;text-align:right;color:${r.win_rate > 50 ? '#00BFFF' : '#ff4444'};">${r.win_rate.toFixed(1)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:${pnlColor(r.return_pct)};font-weight:700;">${r.return_pct > 0 ? '+' : ''}${r.return_pct.toFixed(2)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:#ff4444;">${r.max_drawdown.toFixed(2)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${r.profit_factor.toFixed(2)}</td>
                            <td style="padding:6px 8px;text-align:right;color:${sharpeClr(r.sharpe)};">${r.sharpe.toFixed(2)}</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${r.expectancy.toFixed(4)}</td>
                            <td style="padding:6px 8px;text-align:center;">${badge}</td>
                        </tr>`;
                    }).join('');
                }

                function renderSignalBtMultiResults(results) {
                    const container = $el('sig-bt-multi-results');
                    const body = $el('sig-bt-multi-body');
                    if (!container || !body) return;
                    container.style.display = 'block';

                    const sharpeClr = v => v > 1 ? '#00BFFF' : v > 0 ? '#FFD700' : '#ff4444';

                    const sorted = [...results].sort((a, b) => b.best_return - a.best_return);

                    body.innerHTML = sorted.map(r => {
                        const best = r.results.find(t => t.best) || {};
                        return `<tr style="border-bottom:1px solid #1a1a0a;">
                            <td style="padding:6px 8px;color:#FFD700;font-weight:700;">${escapeHtml(r.symbol)}</td>
                            <td style="padding:6px 8px;text-align:center;color:#00BFFF;font-weight:700;">${escapeHtml(r.best_tf)}</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${best.trades || 0}</td>
                            <td style="padding:6px 8px;text-align:right;color:${(best.win_rate||0) > 50 ? '#00BFFF' : '#ff4444'};">${(best.win_rate||0).toFixed(1)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:${pnlColor(r.best_return)};font-weight:700;">${r.best_return > 0 ? '+' : ''}${r.best_return.toFixed(2)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:${sharpeClr(r.best_sharpe)};">${r.best_sharpe.toFixed(2)}</td>
                            <td style="padding:6px 8px;text-align:right;color:#ff4444;">${(best.max_drawdown||0).toFixed(2)}%</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${(best.profit_factor||0).toFixed(2)}</td>
                            <td style="padding:6px 8px;text-align:right;color:#ccc;">${(best.expectancy||0).toFixed(4)}</td>
                        </tr>`;
                    }).join('');
                }

                if (window.__TAURI__) {
                    window.__TAURI__.event.listen('signal-bt-progress', (ev) => {
                        const d = ev.payload;
                        const prog = $el('sig-bt-progress');
                        if (prog) {
                            if (d.tf === 'done') {
                                prog.textContent = 'Completed!';
                            } else {
                                prog.textContent = d.symbol + ': testing ' + d.tf + ' (' + d.done + '/' + d.total + ')';
                            }
                        }
                    });

                    window.__TAURI__.event.listen('signal-bt-multi-progress', (ev) => {
                        const d = ev.payload;
                        const prog = $el('sig-bt-progress');
                        if (prog) {
                            prog.textContent = 'Symbol ' + (d.sym_done + 1) + '/' + d.sym_total + ': ' + d.symbol;
                        }
                    });
                }
            });
        })();

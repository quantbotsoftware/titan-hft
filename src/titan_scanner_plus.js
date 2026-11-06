        (function () {
            let aiScanData = [];
            window._aiScanData = aiScanData;
            let aiSortCol = 'ai_score';
            let aiSortAsc = false;
            let aiSelectedCandidateCount = 50;

            function wireAiScanner() {
                document.querySelectorAll('.ai-cand-btn').forEach(btn => {
                    btn.addEventListener('click', () => {
                        document.querySelectorAll('.ai-cand-btn').forEach(b => b.classList.remove('active'));
                        btn.classList.add('active');
                        aiSelectedCandidateCount = parseInt(btn.dataset.candidates, 10);
                    });
                });

                const scanBtn = $el('btn-ai-scan');
                if (scanBtn) scanBtn.addEventListener('click', runAiScan);

                if (typeof bindToggle === 'function') {
                    bindToggle('chk-ai-scan-active', 'ai_scan_active');
                    bindToggle('chk-auto-inject-hft', 'ai_auto_inject_hft');
                }

                const autoInjectChk = $el('chk-auto-inject-hft');
                if (autoInjectChk) autoInjectChk.addEventListener('click', async () => {
                    await new Promise(r => setTimeout(r, 50));
                    const box = autoInjectChk.querySelector('.chk-box');
                    const isOn = box && box.classList.contains('checked');
                    const topNVal = $el('auto-inject-top-n')?.value || '5';
                    if (window.__TAURI__) {
                        await window.invoke('update_settings', {
                            field: 'ai_auto_inject_top',
                            value: isOn ? topNVal : '0'
                        });
                    }
                    spMode = isOn ? 'auto' : 'manual';
                    document.querySelectorAll('.sp-mode-btn').forEach(b => {
                        b.classList.toggle('active', b.dataset.spMode === spMode);
                    });
                    if (typeof renderSmartPicks === 'function') renderSmartPicks();
                });

                const autoInjectTopN = $el('auto-inject-top-n');
                if (autoInjectTopN) autoInjectTopN.addEventListener('change', async () => {
                    const chk = document.querySelector('#chk-auto-inject-hft .chk-box');
                    const isOn = chk && chk.classList.contains('checked');
                    if (isOn && window.__TAURI__) {
                        await window.invoke('update_settings', {
                            field: 'ai_auto_inject_top',
                            value: autoInjectTopN.value
                        });
                    }
                    const spTopN = $el('sp-top-n');
                    if (spTopN) spTopN.value = autoInjectTopN.value;
                    if (typeof renderSmartPicks === 'function') renderSmartPicks();
                });

                const activateBtn = $el('btn-ai-activate');
                if (activateBtn) activateBtn.addEventListener('click', activateSelected);

                const activateAllBtn = $el('btn-ai-activate-all');
                if (activateAllBtn) activateAllBtn.addEventListener('click', async () => {
                    let filtered = [...aiScanData];
                    const symFilter = ($el('ai-filter-sym')?.value || '').toUpperCase();
                    const sigFilter = $el('ai-filter-signal')?.value || '';
                    const regFilter = $el('ai-filter-regime')?.value || '';
                    if (symFilter) filtered = filtered.filter(r => r.symbol.includes(symFilter));
                    if (sigFilter) filtered = filtered.filter(r => r.fusion_signal === sigFilter);
                    if (regFilter) filtered = filtered.filter(r => r.oracle_regime.includes(regFilter));
                    const symbols = filtered.map(r => r.symbol);
                    if (symbols.length === 0) { showCyberToast('No symbols to inject', 'warning'); return; }
                    if (!(await cyberConfirm('Inject ' + symbols.length + ' symbols into HFT Engine?'))) return;
                    try {
                        const res = await invoke('activate_scan_targets', { symbols });
                        titanLogAdd(res);
                    } catch(e) { titanLogAdd('Error: ' + normalizeError(e), '#ff4444'); }
                });

                const cancelBtn = $el('btn-ai-cancel');
                if (cancelBtn) cancelBtn.addEventListener('click', async () => {
                    try {
                        const res = await invoke('cancel_ai_scan');
                        titanLogAdd(res);
                        const prog = $el('ai-scan-progress');
                        if (prog) prog.style.display = 'none';
                    } catch(e) { titanLogAdd('Cancel error: ' + normalizeError(e), '#ff4444'); }
                });

                document.querySelectorAll('.ai-sort').forEach(th => {
                    th.addEventListener('click', () => {
                        const col = th.dataset.col;
                        if (aiSortCol === col) aiSortAsc = !aiSortAsc;
                        else { aiSortCol = col; aiSortAsc = false; }
                        renderAiTable();
                    });
                });

                const checkAll = $el('ai-check-all');
                if (checkAll) checkAll.addEventListener('change', () => {
                    document.querySelectorAll('.ai-row-check').forEach(cb => { cb.checked = checkAll.checked; });
                });

                let _aiFilterDebounce = null;
                const debouncedRenderAi = () => {
                    clearTimeout(_aiFilterDebounce);
                    _aiFilterDebounce = setTimeout(renderAiTable, 200);
                };
                ['ai-filter-sym', 'ai-filter-signal', 'ai-filter-regime'].forEach(id => {
                    const el = $el(id);
                    if (el) el.addEventListener('input', debouncedRenderAi);
                    if (el) el.addEventListener('change', renderAiTable);
                });

                if (window.__TAURI__) {
                    window.__TAURI__.event.listen('ai-scan-progress', (ev) => {
                        const d = ev.payload;
                        const prog = $el('ai-scan-progress');
                        const status = $el('ai-scan-status');
                        const bar = $el('ai-scan-bar');
                        const pctEl = $el('ai-scan-pct');
                        const diag = $el('ai-scan-diag');
                        if (!prog) return;
                        prog.style.display = 'block';
                        const pct = d.total > 0 ? Math.round((d.done / d.total) * 100) : 0;
                        let statusText = '';
                        if (d.phase === 'error') {
                            const errMsg = typeof normalizeScanError === 'function'
                                ? normalizeScanError(d.message || 'unknown')
                                : (d.message || 'unknown');
                            statusText = 'ERROR: ' + errMsg;
                            if (status) { status.textContent = statusText; status.style.color = '#FF4444'; }
                            if (bar) bar.style.width = '0%';
                            if (pctEl) pctEl.textContent = '';
                            setTimeout(() => { if (prog) prog.style.display = 'none'; }, 6000);
                            return;
                        }
                        if (status) status.style.color = '#FFD700';
                        if (d.phase === 'assets') statusText = 'Fetching assets...';
                        else if (d.phase === 'snapshots') statusText = 'Loading snapshots...';
                        else if (d.phase === 'prefilter') {
                            if (d.passed !== undefined) {
                                statusText = `Pre-filter: ${d.passed} passed / ${d.total_snapshots || '?'} snapshots`;
                            } else {
                                statusText = 'Running quant pre-filter...';
                            }
                        } else if (d.phase === 'fusion') {
                            const scored = d.scored !== undefined ? d.scored : '?';
                            const dropped = d.dropped !== undefined ? d.dropped : 0;
                            statusText = `FUSION: ${d.symbol || ''} (${d.done}/${d.total}) — ${scored} scored`;
                            if (dropped > 0) statusText += `, ${dropped} skipped`;
                        } else if (d.phase === 'done') {
                            statusText = `Scan complete! (${aiScanData.length} results)`;
                        }
                        if (status) status.textContent = statusText || d.phase;
                        if (bar && d.total > 0) bar.style.width = pct + '%';
                        if (pctEl) pctEl.textContent = pct + '%';
                        if (d.phase === 'done') {
                            if (bar) bar.style.width = '100%';
                            if (pctEl) pctEl.textContent = '100%';
                            if (diag) {
                                const snaps = d.total_snapshots || '?';
                                const pf = d.prefiltered || 0;
                                const sc = d.scored || 0;
                                const dr = d.dropped || 0;
                                diag.style.display = 'block';
                                diag.innerHTML = `<span style="color:#00BFFF">${snaps}</span> snapshots → <span style="color:#FFD700">${pf}</span> pre-filtered → <span style="color:#00FFFF">${sc}</span> scored` + (dr > 0 ? ` <span style="color:#FF6666">(${dr} dropped)</span>` : '');
                            }
                            setTimeout(() => { if (prog) prog.style.display = 'none'; }, 8000);
                        } else {
                            if (diag) diag.style.display = 'none';
                        }
                    });

                    window.__TAURI__.event.listen('ai-scan-update', (ev) => {
                        aiScanData = ev.payload || [];
                        window._aiScanData = aiScanData;
                        (aiScanData || []).forEach(r => {
                            if (r.has_catalyst && (r.catalyst_url || r.catalyst_headline)) {
                                _catalystCache[r.symbol] = {
                                    url: r.catalyst_url || '',
                                    headline: r.catalyst_headline || '',
                                    score: r.sentiment_score || 0.5,
                                    ts: Date.now()
                                };
                            }
                        });
                        renderAiTable();
                        if (typeof refreshHftEnriched === 'function') refreshHftEnriched();
                    });
                }

                renderAiTable();
            }

            window.showAuditReasoning = function(el) {
                const symbol = el.getAttribute('data-symbol') || 'N/A';
                let reasoning = el.getAttribute('data-reasoning') || 'N/A';
                
                const modal = $el('titan-audit-v3-overlay');
                const title = $el('titan-audit-v3-title');
                const content = $el('titan-audit-v3-content');
                
                if (!modal) {
                    showCyberToast("Eroare Critica: Fereastra de Audit (V3) nu a fost gasita in pagina!", 'error');
                    return;
                }

                if (!reasoning || reasoning === 'N/A' || reasoning === '' || reasoning === 'undefined') {
                    reasoning = `SISTEM: Nu exista date de audit stocate pentru ${symbol}.\n\nCAUZA: Datele de audit apar doar dupa o noua scanare.\n\nSOLUTIE: Apasa 'MANUAL SCAN' acum.`;
                } else {
                    reasoning = reasoning.replace(/ \| /g, '\n');
                }

                if (title) title.textContent = `SHARK AI AUDIT: ${symbol}`;
                if (content) content.innerHTML = `<div style="color:#00BFFF; font-weight:bold; margin-bottom:10px;">> ANALYZING FUSION COMPONENTS...</div>` + escapeHtml(reasoning).replace(/\n/g, '<br>');
                
                modal.style.display = 'flex';
                // Force visibility just in case
                modal.style.setProperty('display', 'flex', 'important');
                TitanDebug.log('scanner', 'Modal opened for', symbol);
            };

            async function runAiScan() {
                if (!window.__TAURI__) return;
                const prog = $el('ai-scan-progress');
                if (prog) prog.style.display = 'block';
                const scanBtn = $el('btn-ai-scan');
                const btnText = $el('btn-ai-scan-text');
                if (scanBtn) scanBtn.disabled = true;
                if (btnText) btnText.textContent = 'SCANNING...';
                try {
                    const results = await window.invoke('run_ai_scan', { maxCandidates: aiSelectedCandidateCount });
                    aiScanData = results || [];
                    window._aiScanData = aiScanData;
                    renderAiTable();
                } catch (e) {
                    TitanError.report('ai-scan:run', e);
                    const errMsg = typeof normalizeScanError === 'function' ? normalizeScanError(e) : normalizeError(e);
                    const isTimeout = /timeout/i.test(errMsg);
                    const status = $el('ai-scan-status');
                    if (status) {
                        status.textContent = 'ERROR: ' + errMsg;
                        status.style.color = '#FF4444';
                    }
                    const toastMsg = isTimeout
                        ? 'Scan timed out — check connection and retry'
                        : 'Scan error: ' + errMsg;
                    if (typeof showCyberToast === 'function') showCyberToast(toastMsg, 'error');
                } finally {
                    if (scanBtn) scanBtn.disabled = false;
                    if (btnText) btnText.textContent = 'MANUAL SCAN';
                    setTimeout(() => {
                        if (prog) prog.style.display = 'none';
                    }, 5000);
                }
            }

            async function activateSelected() {
                if (!window.__TAURI__) return;
                const checked = document.querySelectorAll('.ai-row-check:checked');
                const symbols = Array.from(checked).map(cb => cb.dataset.symbol).filter(Boolean);
                if (symbols.length === 0) { showCyberToast('Select at least one asset', 'warning'); return; }
                try {
                    const msg = await window.invoke('activate_scan_targets', { symbols });
                    if (typeof refreshHftTargets === 'function') refreshHftTargets();
                    if (typeof refreshHftEnriched === 'function') refreshHftEnriched();
                    showCyberToast(msg, 'success');
                } catch (e) {
                    showCyberToast('Error: ' + normalizeError(e), 'error');
                }
            }

            function renderAiTable() {
                const body = $el('ai-scan-body');
                const empty = $el('ai-scan-empty');
                if (!body) return;

                let filtered = [...aiScanData];
                const symFilter = ($el('ai-filter-sym')?.value || '').toUpperCase();
                const sigFilter = $el('ai-filter-signal')?.value || '';
                const regFilter = $el('ai-filter-regime')?.value || '';

                if (symFilter) filtered = filtered.filter(r => r.symbol.includes(symFilter));
                if (sigFilter) filtered = filtered.filter(r => r.fusion_signal === sigFilter);
                if (regFilter) filtered = filtered.filter(r => r.oracle_regime.includes(regFilter));

                filtered.sort((a, b) => {
                    let va = a[aiSortCol], vb = b[aiSortCol];
                    if (typeof va === 'string') { va = va.toLowerCase(); vb = (vb || '').toLowerCase(); }
                    if (va < vb) return aiSortAsc ? -1 : 1;
                    if (va > vb) return aiSortAsc ? 1 : -1;
                    return 0;
                });

                if (filtered.length === 0 && aiScanData.length === 0) {
                    body.innerHTML = '';
                    if (empty) empty.style.display = 'block';
                    return;
                }
                if (empty) empty.style.display = 'none';

                // formatVolume, fmtPct, pnlColor -> titan_core.js
                const signalBadge = (s) => s === 'LONG' ? '<span class="ai-badge-long">LONG</span>' : s === 'SHORT' ? '<span class="ai-badge-short">SHORT</span>' : '<span class="ai-badge-neutral">NEUTRAL</span>';
                const scoreBar = (v) => {
                    const w = Math.round(v).toString();
                    const c = v > 70 ? '#00BFFF' : v > 50 ? '#FFD700' : v > 30 ? '#FFA500' : '#ff4444';
                    return `<div style="display:flex;align-items:center;gap:4px;justify-content:flex-end;"><span style="color:${c};font-weight:700;">${v.toFixed(1)}</span><span class="ai-score-bar" style="width:${w}px;background:${c};"></span></div>`;
                };
                const smartBar = (v) => {
                    const w = Math.round(v).toString();
                    const c = v > 70 ? '#00BFFF' : v > 50 ? '#00BFFF' : v > 35 ? '#FFD700' : '#ff4444';
                    return `<div style="display:flex;align-items:center;gap:4px;justify-content:flex-end;"><span style="color:${c};font-weight:700;">${v.toFixed(1)}</span><span class="ai-score-bar" style="width:${w}px;background:${c};"></span></div>`;
                };

                body.innerHTML = filtered.map(r => {
                    const rowClass = r.fusion_signal === 'LONG' ? 'ai-row-long' : r.fusion_signal === 'SHORT' ? 'ai-row-short' : '';
                    const catData = _catalystCache[r.symbol] || (r.catalyst_url ? { url: r.catalyst_url, headline: r.catalyst_headline } : null);
                    const catUrl = catData && catData.url ? catData.url.replace(/'/g, "\\'") : '';
                    const catClick = catUrl ? `onclick="event.stopPropagation();window.__TAURI__?.shell?.open('${catUrl}')" style="color:#00BFFF;font-size:9px;cursor:pointer;" title="${escapeHtml(catData?.headline || 'NEWS CATALYST')}"` : 'style="color:#00BFFF;font-size:9px;" title="NEWS CATALYST"';
                    const catalystBadge = r.has_catalyst ? ` <span ${catClick}>\uD83D\uDCF0</span>` : '';
                    return `<tr class="${rowClass}">
                        <td style="text-align:center;"><input type="checkbox" class="ai-row-check" data-symbol="${escapeHtml(r.symbol)}" style="accent-color:#FFD700;"></td>
                        <td style="text-align:center;color:#666;">${r.rank}</td>
                        <td style="text-align:left;color:#FFD700;font-weight:700;">
                            <span style="cursor:pointer;" onclick="showAuditReasoning(this)" data-symbol="${escapeHtml(r.symbol)}" data-reasoning="${escapeHtml(r.reasoning || 'N/A')}" title="CLICK FOR DEEP AUDIT REASONING">🔍</span>
                            ${escapeHtml(r.symbol)}${catalystBadge}
                        </td>
                        <td style="text-align:right;color:#ccc;">$${r.price.toFixed(2)}</td>
                        <td style="text-align:right;color:${pnlColor(r.change_percent)};">${fmtPct(r.change_percent)}</td>
                        <td style="text-align:right;color:${r.rvol > 2 ? '#FFD700' : '#888'};">${r.rvol.toFixed(1)}x</td>
                        <td style="text-align:right;color:${pnlColor(r.gap_percent)};">${fmtPct(r.gap_percent)}</td>
                        <td style="text-align:right;color:#888;">${formatVolume(r.volume)}</td>
                        <td style="text-align:right;" class="${(r.dollar_volume || 0) >= 10000000 ? 'dvol-gold' : 'dvol-dim'}">${formatDollarVolume(r.dollar_volume || r.price * r.volume)}</td>
                        <td style="text-align:center;color:${r.float_category === 'LOW' ? '#ff4444' : r.float_category === 'HIGH' ? '#00BFFF' : '#FFD700'};">${escapeHtml(r.float_category)}</td>
                        <td style="text-align:right;color:${pnlColor(r.oracle_score)};">${r.oracle_score.toFixed(1)}</td>
                        <td style="text-align:center;color:#00BFFF;font-size:9px;">${escapeHtml(r.oracle_regime)}</td>
                        <td style="text-align:right;color:${r.nexus_buy_pct > 55 ? '#00BFFF' : r.nexus_buy_pct < 45 ? '#ff4444' : '#888'};">${r.nexus_buy_pct.toFixed(0)}%</td>
                        <td style="text-align:right;color:${pnlColor(r.fusion_score)};font-weight:700;">${r.fusion_score.toFixed(1)}</td>
                        <td style="text-align:center;">${signalBadge(r.fusion_signal)}</td>
                        <td style="text-align:center;color:#FFD700;font-weight:700;font-size:9px;">${escapeHtml(r.bt_best_tf || '5m')}</td>
                        <td style="text-align:right;color:#888;">${r.bt_trades}</td>
                        <td style="text-align:right;color:${r.bt_win_rate > 50 ? '#00BFFF' : '#ff4444'};">${r.bt_win_rate.toFixed(1)}%</td>
                        <td style="text-align:right;color:${r.bt_sharpe > 1 ? '#00BFFF' : '#888'};">${r.bt_sharpe.toFixed(2)}</td>
                        <td style="text-align:right;color:${pnlColor(r.bt_return_pct)};">${fmtPct(r.bt_return_pct)}</td>
                        <td style="text-align:right;color:${(r.bt_max_drawdown || 0) < -20 ? '#ff4444' : (r.bt_max_drawdown || 0) < -10 ? '#FFD700' : '#888'};">${r.bt_max_drawdown ? r.bt_max_drawdown.toFixed(1) + '%' : '-'}</td>
                        <td style="text-align:right;color:${pnlColor(r.bt_expectancy || 0)};">${(r.bt_expectancy || 0).toFixed(3)}</td>
                        <td style="text-align:right;">${scoreBar(r.ai_score)}</td>
                        <td style="text-align:right;">${smartBar(r.smart_score || 0)}</td>
                    </tr>`;
                }).join('');

                // Update Smart Picks panel
                renderSmartPicks();
            }

            let _scannerSynced = false;
            TitanBus.on('tabSwitch', function (index) {
                if (index === 6) {
                    if (aiScanData.length === 0) {
                        const empty = $el('ai-scan-empty');
                        if (empty) empty.style.display = 'block';
                    }
                    if (!_scannerSynced && window.__TAURI__ && typeof setToggleChecked === 'function') {
                        _scannerSynced = true;
                        window.invoke('get_settings_snapshot').then(s => {
                        if (s && s.ai_scan_active !== undefined) setToggleChecked('chk-ai-scan-active', s.ai_scan_active);
                    }).catch(e => TitanError.report('scanner:settings-sync', e));
                    }
                }
            });

            document.addEventListener('DOMContentLoaded', wireAiScanner);

            // ========== SMART PICKS ENGINE ==========
            let spMode = 'manual'; // 'manual' | 'auto'
            let spAutoInjectedSymbols = [];

            function getSmartPicks() {
                const topN = parseInt($el('sp-top-n')?.value || '5', 10);
                const strict = [...aiScanData]
                    .filter(r => r.bt_trades >= 2
                        && r.bt_sharpe > 0
                        && r.bt_win_rate > 20
                        && (r.smart_score || 0) > 20
                        && r.change_percent > -5
                        && r.price >= 1.50)
                    .sort((a, b) => (b.smart_score || 0) - (a.smart_score || 0))
                    .slice(0, topN);
                if (strict.length > 0) {
                    window._spRelaxed = false;
                    return strict;
                }
                window._spRelaxed = true;
                return [...aiScanData]
                    .filter(r => r.price >= 1.00 && (r.smart_score || 0) > 0)
                    .sort((a, b) => (b.smart_score || 0) - (a.smart_score || 0))
                    .slice(0, Math.min(topN, 3));
            }

            function renderSmartPicks() {
                const panel = $el('smart-picks-panel');
                const body = $el('smart-picks-body');
                const emptyEl = $el('sp-empty');
                if (!panel || !body) return;

                const picks = getSmartPicks();
                const relaxedBadge = $el('sp-relaxed-badge');

                if (picks.length === 0) {
                    body.innerHTML = '';
                    if (emptyEl) emptyEl.style.display = 'block';
                    if (relaxedBadge) relaxedBadge.style.display = 'none';
                    return;
                }
                if (emptyEl) emptyEl.style.display = 'none';
                if (relaxedBadge) relaxedBadge.style.display = window._spRelaxed ? 'inline' : 'none';

                // fmtPct, pnlColor -> titan_core.js

                body.innerHTML = picks.map((r, i) => {
                    const isAutoInjected = spAutoInjectedSymbols.includes(r.symbol);
                    const sc = r.smart_score || 0;
                    const barW = Math.round(sc);
                    const barC = sc > 70 ? '#00BFFF' : sc > 50 ? '#00BFFF' : sc > 35 ? '#FFD700' : '#ff4444';
                    const chg = r.change_percent || 0;
                    const chgC = chg > 5 ? '#00BFFF' : chg > 0 ? '#88cc88' : chg > -2 ? '#888' : '#ff4444';
                    const rv = r.rvol || 0;
                    const rvC = rv >= 5 ? '#00BFFF' : rv >= 2 ? '#FFD700' : '#888';
                    const reg = r.oracle_regime || 'Unknown';
                    const regC = reg === 'Trending' ? '#00BFFF' : reg === 'MeanReverting' ? '#00BFFF' : reg === 'Chaotic' ? '#ff4444' : '#888';
                    const spCatData = _catalystCache[r.symbol] || (r.catalyst_url ? { url: r.catalyst_url, headline: r.catalyst_headline } : null);
                    const spCatUrl = spCatData && spCatData.url ? spCatData.url.replace(/'/g, "\\'") : '';
                    const spCatClick = spCatUrl ? ` onclick="event.stopPropagation();window.__TAURI__?.shell?.open('${spCatUrl}')" style="cursor:pointer;" title="${escapeHtml(spCatData?.headline || '')}"` : '';
                    const catBadge = r.has_catalyst ? ` <span${spCatClick}>\uD83D\uDCF0</span>` : '';
                    return `<tr style="border-bottom:1px solid #1a2233;">
                        <td style="text-align:center;color:#00BFFF;font-weight:700;">${i + 1}</td>
                        <td style="text-align:left;color:#FFD700;font-weight:700;">
                            <span style="cursor:pointer;" onclick="showAuditReasoning(this)" data-symbol="${escapeHtml(r.symbol)}" data-reasoning="${escapeHtml(r.reasoning || 'N/A')}" title="CLICK FOR DEEP AUDIT REASONING">🔍</span>
                            ${escapeHtml(r.symbol)}${catBadge}
                        </td>
                        <td style="text-align:right;color:#ccc;">$${r.price.toFixed(2)}</td>
                        <td style="text-align:right;color:${chgC};">${chg >= 0 ? '+' : ''}${chg.toFixed(2)}%</td>
                        <td style="text-align:right;color:${rvC};">${rv.toFixed(1)}x</td>
                        <td style="text-align:center;"><span style="color:${regC};font-size:9px;font-weight:700;padding:1px 4px;border:1px solid ${regC};border-radius:3px;">${reg === 'MeanReverting' ? 'MR' : reg === 'Trending' ? 'TREND' : reg === 'Chaotic' ? 'CHAOS' : 'UNK'}</span></td>
                        <td style="text-align:right;color:${r.bt_sharpe > 1 ? '#00BFFF' : '#888'};">${r.bt_sharpe.toFixed(2)}</td>
                        <td style="text-align:right;color:${r.bt_win_rate > 50 ? '#00BFFF' : '#ff4444'};">${r.bt_win_rate.toFixed(1)}%</td>
                        <td style="text-align:right;color:#888;">${r.bt_trades}</td>
                        <td style="text-align:center;color:#FFD700;font-weight:700;font-size:9px;">${escapeHtml(r.bt_best_tf || '5m')}</td>
                        <td style="text-align:right;color:${pnlColor(r.bt_return_pct)};">${fmtPct(r.bt_return_pct)}</td>
                        <td style="text-align:right;color:${(r.bt_max_drawdown || 0) < -20 ? '#ff4444' : (r.bt_max_drawdown || 0) < -10 ? '#FFD700' : '#888'};">${r.bt_max_drawdown ? r.bt_max_drawdown.toFixed(1) + '%' : '-'}</td>
                        <td style="text-align:right;color:${pnlColor(r.bt_expectancy || 0)};">${(r.bt_expectancy || 0).toFixed(3)}</td>
                        <td style="text-align:right;"><div style="display:flex;align-items:center;gap:4px;justify-content:flex-end;"><span style="color:${barC};font-weight:700;">${sc.toFixed(1)}</span><span style="display:inline-block;height:6px;width:${barW}px;background:${barC};border-radius:2px;"></span></div></td>
                        <td style="text-align:center;">${isAutoInjected
                            ? '<span class="sp-auto-badge">\u26a1 AUTO</span>'
                            : '<button class="sp-inject-btn" data-sp-sym="' + r.symbol + '">INJECT</button>'
                        }</td>
                    </tr>`;
                }).join('');

                // Wire individual inject buttons
                body.querySelectorAll('.sp-inject-btn').forEach(btn => {
                    btn.addEventListener('click', async () => {
                        const sym = btn.dataset.spSym;
                        if (!sym || !window.__TAURI__) return;
                        try {
                            await window.invoke('activate_scan_targets', { symbols: [sym] });
                            btn.outerHTML = '<span class="sp-active-badge">\u2713 ACTIVE</span>';
                            if (typeof titanLogAdd === 'function') titanLogAdd('\u26a1 SMART PICK injected: ' + sym);
                        } catch (e) { TitanError.report('smart-picks:inject', e); }
                    });
                });
            }

            function syncAutoInjectCheckbox(isAuto) {
                const box = document.querySelector('#chk-auto-inject-hft .chk-box');
                if (!box) return;
                if (isAuto) box.classList.add('checked');
                else box.classList.remove('checked');
            }

            function wireSmartPicks() {
                // Mode toggle (SMART PICKS MANUAL/AUTO buttons)
                document.querySelectorAll('.sp-mode-btn').forEach(btn => {
                    btn.addEventListener('click', async () => {
                        document.querySelectorAll('.sp-mode-btn').forEach(b => b.classList.remove('active'));
                        btn.classList.add('active');
                        spMode = btn.dataset.spMode;
                        const topN = parseInt($el('sp-top-n')?.value || '5', 10);
                        if (window.__TAURI__) {
                            await invoke('batch_update_settings', { settings: {
                                ai_auto_inject_top: spMode === 'auto' ? String(topN) : '0',
                                ai_auto_inject_hft: spMode === 'auto' ? 'true' : 'false'
                            }});
                        }
                        syncAutoInjectCheckbox(spMode === 'auto');
                        renderSmartPicks();
                    });
                });

                // Top N selector
                const topNSelect = $el('sp-top-n');
                if (topNSelect) topNSelect.addEventListener('change', async () => {
                    if (spMode === 'auto' && window.__TAURI__) {
                        await window.invoke('update_settings', {
                            field: 'ai_auto_inject_top',
                            value: topNSelect.value
                        });
                    }
                    const autoTopN = $el('auto-inject-top-n');
                    if (autoTopN) autoTopN.value = topNSelect.value;
                    renderSmartPicks();
                });

                // Inject All button
                const injectAllBtn = $el('btn-sp-inject-all');
                if (injectAllBtn) injectAllBtn.addEventListener('click', async () => {
                    const picks = getSmartPicks();
                    if (picks.length === 0) return;
                    const symbols = picks.map(r => r.symbol);
                    if (!window.__TAURI__) return;
                    try {
                        const msg = await window.invoke('activate_scan_targets', { symbols });
                        if (typeof titanLogAdd === 'function') titanLogAdd('\u26a1 SMART PICKS injected: ' + symbols.join(', '));
                        // Give visual feedback by changing all inject buttons to active badges
                        document.querySelectorAll('#smart-picks-body .sp-inject-btn').forEach(btn => {
                            btn.outerHTML = '<span class="sp-active-badge">\u2713 ACTIVE</span>';
                        });
                    } catch (e) {
                        TitanError.report('smart-picks:inject-all', e);
                    }
                });

                // Listen for auto-inject events
                if (window.__TAURI__) {
                    window.__TAURI__.event.listen('smart-picks-injected', (ev) => {
                        const d = ev.payload;
                        if (d && d.symbols) {
                            spAutoInjectedSymbols = d.symbols;
                            renderSmartPicks();
                        }
                    });
                }

                // Load mode from settings
                if (window.__TAURI__) {
                    window.invoke('get_settings_snapshot').then(s => {
                        if (s && (s.ai_auto_inject_hft || s.ai_auto_inject_top > 0)) {
                            spMode = 'auto';
                            document.querySelectorAll('.sp-mode-btn').forEach(b => {
                                b.classList.toggle('active', b.dataset.spMode === 'auto');
                            });
                            syncAutoInjectCheckbox(true);
                            const topVal = s.ai_auto_inject_top > 0 ? String(s.ai_auto_inject_top) : '5';
                            const topNEl = $el('sp-top-n');
                            if (topNEl) topNEl.value = topVal;
                            const autoTopN = $el('auto-inject-top-n');
                            if (autoTopN) autoTopN.value = topVal;
                        }
                    }).catch(e => TitanError.report('scanner:smart-picks-sync', e));
                }
            }

            document.addEventListener('DOMContentLoaded', wireSmartPicks);
        })();

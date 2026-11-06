        // ═══════════════════════════════════════════════════════════════
        //  LONG/SHORT SETTINGS — Persistence, wiring, calculations
        // ═══════════════════════════════════════════════════════════════
        window._lsSettings = {
            riskMode: 'rr', defaultRR: 1.0, fixedSL: 2.5, fixedTP: 5.0,
            sizeMode: 'dollar', betAmount: 100, equityRisk: 2.0,
            showPct: true, showDollar: false, showRR: true, showEntry: true, showPrices: false,
            autoBracket: false, orderType: 'market',
            colorProfit: '#00BFFF', colorStop: '#FF2B2B', opacity: 10
        };

        function _lsLoad() {
            try {
                const raw = localStorage.getItem('titan_ls_settings');
                if (raw) Object.assign(window._lsSettings, JSON.parse(raw));
            } catch (e) { console.warn('LS settings load failed', e); }
        }
        function _lsSave() {
            try { localStorage.setItem('titan_ls_settings', JSON.stringify(window._lsSettings)); } catch (e) { TitanError.report('ls:save', e); }
        }
        function _lsApplyToUI() {
            const s = window._lsSettings;
            const el = (id) => $el(id);
            const radio = (name, val) => { const r = document.querySelector(`input[name="${name}"][value="${val}"]`); if (r) r.checked = true; };

            radio('ls-risk-mode', s.riskMode);
            if (el('ls-default-rr')) { el('ls-default-rr').value = s.defaultRR; }
            if (el('ls-rr-val')) el('ls-rr-val').textContent = s.defaultRR.toFixed(1);
            if (el('ls-fixed-sl')) el('ls-fixed-sl').value = s.fixedSL;
            if (el('ls-fixed-tp')) el('ls-fixed-tp').value = s.fixedTP;
            if (el('ls-rr-group')) el('ls-rr-group').style.display = s.riskMode === 'rr' ? '' : 'none';
            if (el('ls-fixed-group')) el('ls-fixed-group').style.display = s.riskMode === 'fixed' ? '' : 'none';

            radio('ls-size-mode', s.sizeMode);
            if (el('ls-bet-amount')) el('ls-bet-amount').value = s.betAmount;
            if (el('ls-equity-risk')) el('ls-equity-risk').value = s.equityRisk;
            if (el('ls-equity-val')) el('ls-equity-val').textContent = s.equityRisk.toFixed(1) + '%';
            if (el('ls-dollar-group')) el('ls-dollar-group').style.display = s.sizeMode === 'dollar' ? '' : 'none';
            if (el('ls-equity-group')) el('ls-equity-group').style.display = s.sizeMode === 'equity' ? '' : 'none';

            if (el('ls-show-pct')) el('ls-show-pct').checked = s.showPct;
            if (el('ls-show-dollar')) el('ls-show-dollar').checked = s.showDollar;
            if (el('ls-show-rr')) el('ls-show-rr').checked = s.showRR;
            if (el('ls-show-entry')) el('ls-show-entry').checked = s.showEntry;
            if (el('ls-show-prices')) el('ls-show-prices').checked = s.showPrices;

            if (el('ls-auto-bracket')) el('ls-auto-bracket').checked = s.autoBracket;
            if (el('ls-order-type')) el('ls-order-type').value = s.orderType;

            if (el('ls-color-profit')) el('ls-color-profit').value = s.colorProfit;
            if (el('ls-color-profit-hex')) { el('ls-color-profit-hex').textContent = s.colorProfit; el('ls-color-profit-hex').style.color = s.colorProfit; }
            if (el('ls-color-stop')) el('ls-color-stop').value = s.colorStop;
            if (el('ls-color-stop-hex')) { el('ls-color-stop-hex').textContent = s.colorStop; el('ls-color-stop-hex').style.color = s.colorStop; }
            if (el('ls-opacity')) el('ls-opacity').value = s.opacity;
            if (el('ls-opacity-val')) el('ls-opacity-val').textContent = s.opacity + '%';

            const betInput = el('bet-input');
            if (betInput) betInput.value = s.betAmount;
        }

        function _lsGetBetAmount() {
            const s = window._lsSettings;
            if (s.sizeMode === 'dollar') return s.betAmount;
            const eqEl = $el('account-equity');
            const equity = eqEl ? parseFloat(eqEl.textContent.replace(/[$,]/g, '')) || 0 : 0;
            return equity * (s.equityRisk / 100);
        }

        function _lsUpdateCalcFields() {
            const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
            const sel = mgr ? mgr.selectedDrawing : null;
            const bet = _lsGetBetAmount();
            const qtyEl = $el('ls-calc-qty');
            const riskEl = $el('ls-calc-risk');
            const lossEl = $el('ls-calc-loss');
            const profitEl = $el('ls-calc-profit');
            if (!qtyEl) return;

            if (!sel || (sel.type !== 'long' && sel.type !== 'short')) {
                qtyEl.textContent = '--'; riskEl.textContent = '--'; lossEl.textContent = '--'; profitEl.textContent = '--';
                return;
            }
            const entry = sel.p1.price;
            const target = sel.p2.price;
            const stop = sel.stopPrice || entry;
            if (!entry || entry === 0) { qtyEl.textContent = '--'; return; }

            const qty = bet / entry;
            const profitDist = Math.abs(target - entry);
            const lossDist = Math.abs(stop - entry);
            const maxProfit = qty * profitDist;
            const maxLoss = qty * lossDist;

            qtyEl.textContent = qty.toFixed(4);
            riskEl.textContent = '$' + bet.toFixed(2);
            lossEl.textContent = '-$' + maxLoss.toFixed(2);
            profitEl.textContent = '+$' + maxProfit.toFixed(2);
        }

        function _lsWireTab() {
            const el = (id) => $el(id);
            const s = window._lsSettings;

            document.querySelectorAll('input[name="ls-risk-mode"]').forEach(r => {
                r.addEventListener('change', () => {
                    s.riskMode = r.value; _lsSave(); _lsApplyToUI();
                    if (drawingManager) drawingManager.render();
                });
            });
            const rrSlider = el('ls-default-rr');
            if (rrSlider) rrSlider.addEventListener('input', () => {
                s.defaultRR = parseFloat(rrSlider.value); el('ls-rr-val').textContent = s.defaultRR.toFixed(1); _lsSave();
            });
            if (el('ls-fixed-sl')) el('ls-fixed-sl').addEventListener('change', () => { s.fixedSL = parseFloat(el('ls-fixed-sl').value) || 2.5; _lsSave(); });
            if (el('ls-fixed-tp')) el('ls-fixed-tp').addEventListener('change', () => { s.fixedTP = parseFloat(el('ls-fixed-tp').value) || 5.0; _lsSave(); });

            document.querySelectorAll('input[name="ls-size-mode"]').forEach(r => {
                r.addEventListener('change', () => {
                    s.sizeMode = r.value; _lsSave(); _lsApplyToUI(); _lsUpdateCalcFields();
                    if (drawingManager) drawingManager.render();
                });
            });
            if (el('ls-bet-amount')) el('ls-bet-amount').addEventListener('input', () => {
                s.betAmount = parseFloat(el('ls-bet-amount').value) || 100;
                const betInput = el('bet-input');
                if (betInput) betInput.value = s.betAmount;
                _lsSave(); _lsUpdateCalcFields();
                if (drawingManager) drawingManager.render();
            });
            if (el('ls-equity-risk')) el('ls-equity-risk').addEventListener('input', () => {
                s.equityRisk = parseFloat(el('ls-equity-risk').value);
                el('ls-equity-val').textContent = s.equityRisk.toFixed(1) + '%';
                _lsSave(); _lsUpdateCalcFields();
                if (drawingManager) drawingManager.render();
            });

            ['ls-show-pct', 'ls-show-dollar', 'ls-show-rr', 'ls-show-entry', 'ls-show-prices'].forEach(id => {
                const chk = el(id);
                if (!chk) return;
                const key = { 'ls-show-pct': 'showPct', 'ls-show-dollar': 'showDollar', 'ls-show-rr': 'showRR', 'ls-show-entry': 'showEntry', 'ls-show-prices': 'showPrices' }[id];
                chk.addEventListener('change', () => { s[key] = chk.checked; _lsSave(); if (drawingManager) drawingManager.render(); });
            });

            if (el('ls-auto-bracket')) el('ls-auto-bracket').addEventListener('change', () => { s.autoBracket = el('ls-auto-bracket').checked; _lsSave(); });
            if (el('ls-order-type') && !el('ls-order-type').disabled) el('ls-order-type').addEventListener('change', () => { s.orderType = el('ls-order-type').value; _lsSave(); });

            if (el('ls-color-profit')) el('ls-color-profit').addEventListener('input', () => {
                s.colorProfit = el('ls-color-profit').value;
                el('ls-color-profit-hex').textContent = s.colorProfit; el('ls-color-profit-hex').style.color = s.colorProfit;
                _lsSave(); if (drawingManager) drawingManager.render();
            });
            if (el('ls-color-stop')) el('ls-color-stop').addEventListener('input', () => {
                s.colorStop = el('ls-color-stop').value;
                el('ls-color-stop-hex').textContent = s.colorStop; el('ls-color-stop-hex').style.color = s.colorStop;
                _lsSave(); if (drawingManager) drawingManager.render();
            });
            if (el('ls-opacity')) el('ls-opacity').addEventListener('input', () => {
                s.opacity = parseInt(el('ls-opacity').value);
                el('ls-opacity-val').textContent = s.opacity + '%';
                _lsSave(); if (drawingManager) drawingManager.render();
            });

            const betInput = el('bet-input');
            if (betInput) betInput.addEventListener('input', () => {
                s.betAmount = parseFloat(betInput.value) || 100;
                if (el('ls-bet-amount')) el('ls-bet-amount').value = s.betAmount;
                _lsSave(); _lsUpdateCalcFields();
                if (drawingManager) drawingManager.render();
            });
        }

        function _lsShowExecToolbar(d) {
            const tb = $el('exec-toolbar');
            if (!tb) return;
            if (!d || (d.type !== 'long' && d.type !== 'short')) {
                tb.style.display = 'none';
                return;
            }
            const chartWrap = $el('chart-main-wrap');
            if (chartWrap) {
                const rect = chartWrap.getBoundingClientRect();
                tb.style.display = 'flex';
                tb.style.top = (rect.top + 4) + 'px';
                tb.style.left = 'auto';
                tb.style.right = (window.innerWidth - rect.right + 80) + 'px';
            } else {
                tb.style.display = 'flex';
                tb.style.top = '4px';
                tb.style.right = '80px';
                tb.style.left = 'auto';
            }
        }

        window.addEventListener('resize', () => {
            const tb = $el('exec-toolbar');
            if (tb && tb.style.display === 'flex') {
                const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
                const sel = mgr ? mgr.selectedDrawing : null;
                _lsShowExecToolbar(sel);
            }
        });

        async function _lsExecuteDrawing(aiMode) {
            const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
            const d = mgr ? mgr.selectedDrawing : null;
            if (!d || (d.type !== 'long' && d.type !== 'short')) return;

            const s = window._lsSettings;
            const bet = _lsGetBetAmount();
            const price = d.p1.price;
            if (!price || price <= 0) return;
            let qty = bet / price;
            if (qty < 0.0001) qty = 0.0001;

            const side = d.type === 'short' ? 'sell' : 'buy';
            const takeProfit = d.p2.price;
            const stopLoss = d.stopPrice;

            const btnId = aiMode ? 'btn-exec-ai' : 'btn-exec-manual';
            const btn = $el(btnId);
            const origText = btn ? btn.innerText : '';
            if (btn) btn.innerText = '...';

            try {
                const symbol = (typeof currentSymbol !== 'undefined' && currentSymbol) ? currentSymbol : 'SPY';
                const orderPayload = {
                    symbol: symbol,
                    qty: parseFloat(qty.toFixed(6)),
                    side: side
                };
                if (s.autoBracket) {
                    orderPayload.take_profit = parseFloat(takeProfit.toFixed(2));
                    orderPayload.stop_loss = parseFloat(stopLoss.toFixed(2));
                }
                const res = await window.invoke('submit_order', { order: orderPayload });
                const prefix = aiMode ? 'AI: ' : 'EXEC: ';
                if (typeof titanLogAdd === 'function') titanLogAdd(prefix + res);
                else showCyberToast(prefix + res, 'success');
            } catch (e) {
                const msg = 'ORDER ERROR: ' + (e.message || e);
                if (typeof titanLogAdd === 'function') titanLogAdd(msg);
                else showCyberToast(msg, 'error');
            } finally {
                if (btn) btn.innerText = origText;
            }
        }

        function _lsUpdateDrawingCount() {
            const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
            const count = mgr ? mgr.drawings.length : 0;
            const badge = $el('drawing-count');
            if (badge) {
                if (count > 0) {
                    badge.style.display = 'flex';
                    badge.textContent = count;
                } else {
                    badge.style.display = 'none';
                }
            }
            const info = $el('ls-drawing-info');
            if (info) {
                const boxes = mgr ? mgr.drawings.filter(d => d.type === 'long' || d.type === 'short').length : 0;
                const others = count - boxes;
                info.textContent = `${count} drawing${count !== 1 ? 's' : ''} (${boxes} long/short, ${others} other)`;
            }
        }

        const _origPersist = window.persistDrawings;
        window.persistDrawings = function() {
            if (_origPersist) _origPersist();
            _lsUpdateDrawingCount();
        };

        _lsLoad();
        setTimeout(() => {
            _lsApplyToUI();
            _lsWireTab();

            const execManual = $el('btn-exec-manual');
            const execAi = $el('btn-exec-ai');
            const execDel = $el('btn-exec-del');
            if (execManual) execManual.addEventListener('click', () => _lsExecuteDrawing(false));
            if (execAi) execAi.addEventListener('click', () => _lsExecuteDrawing(true));
            if (execDel) execDel.addEventListener('click', () => { if (window.DrawingEngine) window.DrawingEngine.deleteSelected(); });

            const btnClearAll = $el('ls-clear-all');
            if (btnClearAll) btnClearAll.addEventListener('click', async () => {
                const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
                if (!mgr || !mgr.drawings.length) return;
                if (!(await cyberConfirm('Delete ALL ' + mgr.drawings.length + ' drawings on this chart?'))) return;
                mgr.clear();
                _lsUpdateDrawingCount();
            });

            const btnClearBoxes = $el('ls-clear-boxes');
            if (btnClearBoxes) btnClearBoxes.addEventListener('click', async () => {
                const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
                if (!mgr) return;
                const boxes = mgr.drawings.filter(d => d.type === 'long' || d.type === 'short');
                if (!boxes.length) return;
                if (!(await cyberConfirm('Delete ' + boxes.length + ' long/short box(es)? Other drawings are kept.'))) return;
                mgr.drawings = mgr.drawings.filter(d => d.type !== 'long' && d.type !== 'short');
                mgr.selectedDrawing = null;
                mgr.render();
                if (window.persistDrawings) window.persistDrawings();
            });

            const btnClearAllSym = $el('ls-clear-all-symbols');
            if (btnClearAllSym) btnClearAllSym.addEventListener('click', async () => {
                const keys = [];
                for (let i = 0; i < localStorage.length; i++) {
                    const k = localStorage.key(i);
                    if (k && k.startsWith('titan_drawings_')) keys.push(k);
                }
                if (!keys.length) { showCyberToast('No saved drawings found', 'info'); return; }
                if (!(await cyberConfirm('Delete drawings for ALL symbols/timeframes? (' + keys.length + ' saved chart(s))'))) return;
                keys.forEach(k => localStorage.removeItem(k));
                const mgr = typeof getDrawingManager === 'function' ? getDrawingManager() : null;
                if (mgr) { mgr.drawings = []; mgr.selectedDrawing = null; mgr.render(); }
                _lsUpdateDrawingCount();
            });

            _lsUpdateDrawingCount();
        }, 500);

        // --- BIND EVENTS MANUALLY (SAFE MODE) ---
        setTimeout(() => {
            const btnTrend = $el('btn-trend');
            const btnRay = $el('btn-ray');
            const btnHLine = $el('btn-hline');
            const btnVLine = $el('btn-vline');
            const btnFib = $el('btn-fib');
            const btnText = $el('btn-text');
            const btnMeasure = $el('btn-measure');
            const btnLong = $el('btn-long');
            const btnShort = $el('btn-short');
            const btnDel = $el('btn-delete');
            const btnCross = $el('btn-crosshair');

            if (btnTrend) btnTrend.addEventListener('click', (e) => toggletool('trend', e.currentTarget));
            if (btnRay) btnRay.addEventListener('click', (e) => toggletool('ray', e.currentTarget));
            if (btnHLine) btnHLine.addEventListener('click', (e) => toggletool('hline', e.currentTarget));
            if (btnVLine) btnVLine.addEventListener('click', (e) => toggletool('vline', e.currentTarget));
            if (btnFib) btnFib.addEventListener('click', (e) => toggletool('fib', e.currentTarget));
            if (btnText) btnText.addEventListener('click', (e) => toggletool('text', e.currentTarget));
            if (btnMeasure) btnMeasure.addEventListener('click', (e) => toggletool('measure', e.currentTarget));
            if (btnLong) btnLong.addEventListener('click', (e) => toggletool('long', e.currentTarget));
            if (btnShort) btnShort.addEventListener('click', (e) => toggletool('short', e.currentTarget));

            if (btnDel) btnDel.addEventListener('click', clearDrawings);
            if (btnCross) btnCross.addEventListener('click', (e) => toggletool(null, e.currentTarget));

            TitanDebug.log('ui', 'EVENTS BOUND.');
        }, 1000);

        // ═══════════════════════════════════════════════════════════════
        //  CATALYST WATCHDOG (Rebuilt V2 — XSS-safe, severity colors)
        // ═══════════════════════════════════════════════════════════════
        let currentCatalystSymbol = null;
        let currentCatalystScore = 0;
        let currentCatalystUrl = null;

        function _catalystBadgeHtml(sym, candidate) {
            let c = _catalystCache[sym];
            if (!c && candidate && candidate.has_catalyst) {
                c = {
                    url: candidate.catalyst_url || '',
                    headline: candidate.catalyst_headline || 'News catalyst detected',
                    score: candidate.sentiment_score || 0.5,
                    ts: Date.now()
                };
            }
            const eSym = escapeHtml(sym);
            if (!c) return `<span data-cat-check="${eSym}" title="Check news for ${eSym}" style="color:#555;font-size:9px;cursor:pointer;" onclick="event.stopPropagation();_checkSingleCatalyst('${eSym}',this)">\uD83D\uDD0D</span>`;
            const age = Date.now() - (c.ts || 0);
            const ageStr = age < 60000 ? '<1m' : age < 3600000 ? Math.floor(age / 60000) + 'm' : Math.floor(age / 3600000) + 'h';
            const sev = catalystSeverityColor(c.score);
            const isBearish = c.score < 0;
            const emoji = isBearish ? '🔻' : '🔥';
            const safeUrl = c.url ? c.url.replace(/'/g, "\\'") : '';
            const click = c.url ? `onclick="event.stopPropagation();window.__TAURI__?.shell?.open('${safeUrl}')"` : '';
            const cursor = c.url ? 'cursor:pointer;' : '';
            const scoreStr = c.score ? ` (${c.score.toFixed(2)})` : '';
            const title = escapeHtml((c.headline || '') + scoreStr + (c.ts ? ' — ' + ageStr + ' ago' : ''));
            return `<span data-cat-sym="${eSym}" title="${title}" style="color:${sev};font-size:9px;font-weight:700;${cursor}border:1px solid ${sev};padding:1px 4px;border-radius:3px;" ${click}>${emoji}CAT</span>`;
        }

        /** DRY helper: update _catalystCache from check_catalyst results */
        function _updateCatalystCacheFromResults(results) {
            (results || []).forEach(r => {
                if (r.has_catalyst || (r.url && r.headline)) {
                    _catalystCache[r.symbol] = {
                        url: r.url || '',
                        headline: r.headline || '',
                        score: r.sentiment_score || 0,
                        ts: Date.now()
                    };
                }
            });
        }

        async function _checkSingleCatalyst(sym, el) {
            if (el) { el.textContent = '...'; el.style.color = '#FFD700'; }
            try {
                const results = await invoke('check_catalyst', { symbols: [sym] });
                _updateCatalystCacheFromResults(results);
                if (el && el.parentElement) {
                    const c = _catalystCache[sym];
                    if (c && (c.headline || c.url)) {
                        el.outerHTML = _catalystBadgeHtml(sym);
                    } else {
                        el.textContent = '-';
                        el.style.color = '#555';
                        el.title = 'No catalyst found';
                    }
                }
            } catch (e) {
                TitanError.report('catalyst:single-check', e);
                if (el) { el.textContent = '\u2716'; el.style.color = '#ff4444'; }

            }
        }

        function catalystSeverityColor(score) {
            if (score >= 0.85) return '#00E5FF'; // cyan — explosive
            if (score >= 0.70) return '#00BFFF'; // green — standard
            return '#FFD600'; // yellow — marginal
        }

        function createCatalystLogRow(data) {
            const tr = document.createElement('tr');
            tr.style.borderBottom = '1px solid #222';
            tr.style.transition = 'background 0.2s';
            tr.onmouseover = () => tr.style.background = '#1a1a2e';
            tr.onmouseout = () => tr.style.background = 'transparent';

            const isBearish = data.sentiment_score < 0;
            const sentimentLabel = isBearish ? 'BEARISH' : 'BULLISH';
            tr.dataset.sym = data.symbol;
            tr.dataset.score = Math.abs(data.sentiment_score).toFixed(2);
            tr.dataset.sentiment = isBearish ? 'bearish' : 'bullish';

            const d = new Date(data.timestamp);
            const timeStr = isNaN(d.getTime()) ? '--:--' : d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

            // TIME
            const tdTime = document.createElement('td');
            tdTime.style.cssText = 'padding:10px; color:#888;';
            tdTime.textContent = timeStr;

            // SYMBOL
            const tdSym = document.createElement('td');
            const symColor = isBearish ? '#ff4444' : '#00BFFF';
            tdSym.style.cssText = `padding:10px; font-weight:bold; color:${symColor};`;
            tdSym.textContent = data.symbol;

            // HEADLINE (XSS-safe, clickable link to source)
            const tdHead = document.createElement('td');
            tdHead.style.cssText = 'padding:10px; font-weight:600;';
            const headSpan = document.createElement('span');
            headSpan.textContent = data.headline;
            if (data.url) {
                headSpan.style.cssText = 'color:#00BFFF; cursor:pointer; text-decoration:underline;';
                headSpan.title = 'Click to read full article';
                headSpan.addEventListener('click', () => { window.__TAURI__?.shell?.open(data.url); });
            } else {
                headSpan.style.color = '#ddd';
            }
            tdHead.appendChild(headSpan);
            if (data.url) {
                const linkIcon = document.createElement('span');
                linkIcon.textContent = ' \u2197';
                linkIcon.style.cssText = 'color:#00BFFF; font-size:10px; cursor:pointer;';
                linkIcon.title = 'Open source article';
                linkIcon.addEventListener('click', (e) => { e.stopPropagation(); window.__TAURI__?.shell?.open(data.url); });
                tdHead.appendChild(linkIcon);
            }

            // SEVERITY badge
            const tdSev = document.createElement('td');
            tdSev.style.cssText = 'padding:10px;';
            const severity = data.severity || 'MODERATE';
            const sevColors = {
                'EXPLOSIVE': { bg: '#00E5FF', text: '#000' },
                'STRONG': { bg: '#00BFFF', text: '#000' },
                'BEARISH_STRONG': { bg: '#ff4444', text: '#fff' },
                'BEARISH_EXPLOSIVE': { bg: '#ff0066', text: '#fff' },
                'MODERATE': { bg: '#FFD600', text: '#000' },
            };
            const sc = sevColors[severity] || sevColors['MODERATE'];
            const sevBadge = document.createElement('span');
            sevBadge.textContent = severity.replace('BEARISH_', '');
            sevBadge.style.cssText = `font-size:9px; font-weight:700; padding:2px 6px; border-radius:3px; background:${sc.bg}; color:${sc.text};`;
            tdSev.appendChild(sevBadge);

            // SCORE
            const scoreColor = catalystSeverityColor(data.sentiment_score);
            const tdScore = document.createElement('td');
            tdScore.style.cssText = `padding:10px; font-weight:bold; color:${scoreColor};`;
            tdScore.textContent = data.sentiment_score.toFixed(2);

            // SENTIMENT label
            const tdSent = document.createElement('td');
            tdSent.style.cssText = 'padding:10px;';
            const sentBadge = document.createElement('span');
            sentBadge.textContent = sentimentLabel;
            sentBadge.style.cssText = `font-size:9px; font-weight:700; padding:2px 6px; border-radius:3px; border:1px solid ${isBearish ? '#ff4444' : '#00BFFF'}; color:${isBearish ? '#ff4444' : '#00BFFF'};`;
            tdSent.appendChild(sentBadge);

            // ACTION BUTTON
            const tdAction = document.createElement('td');
            tdAction.style.cssText = 'padding:10px;';
            const btn = document.createElement('button');
            btn.textContent = 'INJECT';
            btn.style.cssText = 'background:transparent; border:1px solid #555; color:#aaa; font-size:10px; padding:4px 8px; border-radius:4px; cursor:pointer; font-weight:bold; transition:all 0.2s;';
            btn.addEventListener('click', async function () {
                try {
                    if (typeof window.invoke === 'function') {
                        await window.invoke('activate_scan_targets', { symbols: [data.symbol] });
                        if (typeof refreshHftTargets === 'function') refreshHftTargets();
                    }
                    this.textContent = 'ACTIVE';
                    this.style.color = '#00BFFF';
                    this.style.borderColor = '#00BFFF';
                } catch (e) { TitanError.report('settings:inject', e); }
            });
            tdAction.appendChild(btn);

            tr.appendChild(tdTime);
            tr.appendChild(tdSym);
            tr.appendChild(tdHead);
            tr.appendChild(tdSev);
            tr.appendChild(tdScore);
            tr.appendChild(tdSent);
            tr.appendChild(tdAction);
            return tr;
        }



        if (window.__TAURI__) {
            const { listen } = window.__TAURI__.event;
            let catalystTotalCount = 0;
            let catalystBullishCount = 0;
            let catalystBearishCount = 0;
            let catalystToastTimer = null;
            let catalystToastQueue = [];
            const CATALYST_MAX_ROWS = 100;

            listen('catalyst-alert', (event) => {
                const data = event.payload;
                if (!data || !data.symbol) return;

                const isBearish = data.sentiment_score < 0;
                const emoji = isBearish ? '🔻' : '🔥';
                TitanDebug.log('catalyst', `${emoji} CATALYST TRIGGERED:`, data);
                currentCatalystSymbol = data.symbol;
                currentCatalystScore = data.sentiment_score || 0;
                currentCatalystUrl = data.url || null;

                _catalystCache[data.symbol] = {
                    url: data.url || '',
                    headline: data.headline || '',
                    score: data.sentiment_score || 0,
                    timestamp: data.timestamp || '',
                    ts: Date.now()
                };

                // Update tab-7 stats
                catalystTotalCount++;
                if (isBearish) catalystBearishCount++; else catalystBullishCount++;
                const statTotal = $el('catalyst-stat-total');
                if (statTotal) statTotal.textContent = catalystTotalCount;
                const statCount = $el('catalyst-stats-count');
                if (statCount) statCount.textContent = `${catalystTotalCount} alerts`;
                const statBullish = $el('catalyst-stat-bullish');
                if (statBullish) statBullish.textContent = catalystBullishCount;
                const statBearish = $el('catalyst-stat-bearish');
                if (statBearish) statBearish.textContent = catalystBearishCount;
                const statLast = $el('catalyst-stat-last');
                if (statLast) {
                    const d = new Date(data.timestamp);
                    statLast.textContent = isNaN(d.getTime()) ? data.symbol : `${data.symbol} @ ${d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
                    statLast.style.color = isBearish ? '#ff4444' : '#00BFFF';
                }

                // Add to log table (tab-7) with row limit
                const emptyRow = $el('catalyst-log-empty');
                if (emptyRow) emptyRow.style.display = 'none';
                const tbody = $el('catalyst-log-body');
                if (tbody) {
                    const newRow = createCatalystLogRow(data);
                    tbody.prepend(newRow);
                    // Enforce max rows
                    while (tbody.children.length > CATALYST_MAX_ROWS + 1) {
                        tbody.removeChild(tbody.lastChild);
                    }
                    applyCatalystFilters();
                }
                const rowCount = $el('catalyst-row-count');
                if (rowCount) rowCount.textContent = `${Math.min(catalystTotalCount, CATALYST_MAX_ROWS)} / ${CATALYST_MAX_ROWS} max`;

                // Toast stacking: queue if one is already showing
                const toast = $el('catalyst-toast-v3');
                if (toast && toast.style.display === 'block') {
                    catalystToastQueue.push(data);
                    // Show queue counter
                    let qBadge = $el('toast-queue-badge');
                    if (!qBadge) {
                        qBadge = document.createElement('span');
                        qBadge.id = 'toast-queue-badge';
                        qBadge.style.cssText = 'position:absolute; top:4px; right:4px; background:#FF00FF; color:#fff; font-size:9px; font-weight:700; width:18px; height:18px; border-radius:50%; display:flex; align-items:center; justify-content:center;';
                        toast.style.position = 'fixed';
                        toast.appendChild(qBadge);
                    }
                    qBadge.textContent = catalystToastQueue.length;
                } else {
                    showCatalystToast(data);
                }

                // Play 2-tone alert (different tone for bearish)
                catalystPlayAlert(isBearish);



                // Live-update catalyst badges in visible tables
                document.querySelectorAll(`[data-cat-check="${data.symbol}"], [data-cat-sym="${data.symbol}"]`).forEach(el => {
                    el.outerHTML = _catalystBadgeHtml(data.symbol);
                });
            });

            // Watchdog health stats listener
            listen('catalyst-watchdog-stats', (event) => {
                const s = event.payload;
                if (!s) return;
                const dot = $el('catalyst-wd-dot');
                const label = $el('catalyst-wd-label');
                const uptime = $el('catalyst-wd-uptime');
                if (dot && label) {
                    if (s.status === 'LIVE') {
                        dot.style.background = '#00BFFF';
                        dot.style.boxShadow = '0 0 6px #00BFFF';
                        label.textContent = 'LIVE';
                        label.style.color = '#00BFFF';
                    } else if (s.status === 'NO_KEYS') {
                        dot.style.background = '#FFD600';
                        dot.style.boxShadow = '0 0 4px #FFD600';
                        label.textContent = 'NO KEYS';
                        label.style.color = '#FFD600';
                    } else {
                        dot.style.background = '#ff4444';
                        dot.style.boxShadow = '0 0 4px #ff4444';
                        label.textContent = 'DOWN';
                        label.style.color = '#ff4444';
                    }
                }
                if (uptime && s.uptime_secs != null) {
                    const m = Math.floor(s.uptime_secs / 60);
                    const h = Math.floor(m / 60);
                    uptime.textContent = h > 0 ? `${h}h ${m % 60}m | ${s.articles_scanned} scanned | ${s.last_poll_ms}ms` :
                        `${m}m | ${s.articles_scanned} scanned | ${s.last_poll_ms}ms`;
                }
            });

            function showCatalystToast(data) {
                const toast = $el('catalyst-toast-v3');
                if (!toast) return;

                const isBearish = data.sentiment_score < 0;
                const sevColor = isBearish
                    ? (data.sentiment_score <= -0.85 ? '#ff0066' : '#ff4444')
                    : catalystSeverityColor(data.sentiment_score);

                // Populate toast
                $el('toast-symbol').textContent = data.symbol;
                $el('toast-headline').textContent = data.headline;
                $el('toast-score').textContent = data.sentiment_score.toFixed(2);
                $el('toast-score').style.color = sevColor;

                // Apply dynamic border color to the inner box styling
                const innerBody = toast.querySelector('.catalyst-toast-body');
                if (innerBody) {
                    innerBody.style.borderLeftColor = sevColor;
                }
                // Dynamic border
                toast.style.borderLeftColor = sevColor + ' !important';

                // Remove queue badge
                const qBadge = $el('toast-queue-badge');
                if (qBadge) qBadge.remove();

                // Show
                toast.style.display = 'block';
                toast.style.animation = 'toastSlideIn 0.3s ease-out';

                // Progress bar
                const bar = $el('toast-progress');
                if (bar) {
                    bar.style.background = sevColor;
                    bar.style.transition = 'none';
                    bar.style.width = '100%';
                    requestAnimationFrame(() => {
                        requestAnimationFrame(() => {
                            bar.style.transition = 'width 8s linear';
                            bar.style.width = '0%';
                        });
                    });
                }

                // Auto-dismiss after 8s, then show queued toast if any
                if (catalystToastTimer) clearTimeout(catalystToastTimer);
                catalystToastTimer = setTimeout(() => {
                    toast.style.animation = 'toastSlideOut 0.3s ease-in forwards';
                    setTimeout(() => {
                        toast.style.display = 'none';
                        // Show next queued toast
                        if (catalystToastQueue.length > 0) {
                            const next = catalystToastQueue.shift();
                            setTimeout(() => showCatalystToast(next), 200);
                        }
                    }, 300);
                }, 8000);
            }

            // Toast buttons - persistent listeners (timing-safe via window.load)
            window.addEventListener('load', () => {
                const initButtons = () => {
                    const tRead = $el('toast-btn-read');
                    const tView = $el('toast-btn-view');
                    const tInject = $el('toast-btn-inject');
                    const tClose = $el('toast-btn-close');

                    if (tRead) {
                        tRead.onclick = (e) => {
                            e.stopPropagation();
                            if (currentCatalystUrl) {
                                window.__TAURI__?.shell?.open(currentCatalystUrl);
                            }
                        };
                    }

                    if (tView) {
                        tView.onclick = (e) => {
                            e.stopPropagation();
                            document.querySelectorAll('.stTab').forEach(t => t.classList.remove('active'));
                            document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('visible'));
                            const tab7btn = document.querySelector('.stTab[data-tab="7"]');
                            if (tab7btn) tab7btn.classList.add('active');
                            const tab7 = $el('tab-7');
                            if (tab7) { tab7.classList.add('visible'); tab7.style.display = 'block'; }
                            const toastEl = $el('catalyst-toast-v3');
                            if (toastEl) toastEl.style.display = 'none';
                            if (catalystToastTimer) clearTimeout(catalystToastTimer);
                        };
                    }

                    if (tInject) {
                        tInject.onclick = async (e) => {
                            e.stopPropagation();
                            if (!currentCatalystSymbol) return;
                            try {
                                if (typeof window.invoke === 'function') {
                                    await window.invoke('activate_scan_targets', { symbols: [currentCatalystSymbol] });
                                    if (typeof refreshHftTargets === 'function') await refreshHftTargets();
                                    if (typeof updateAppStatusIcon === 'function') updateAppStatusIcon('ACTIVE');
                                }
                            } catch (err) { TitanError.report('settings:inject', err); }
                            const toastEl = $el('catalyst-toast-v3');
                            if (toastEl) toastEl.style.display = 'none';
                            if (catalystToastTimer) clearTimeout(catalystToastTimer);
                        };
                    }

                    if (tClose) {
                        tClose.onclick = (e) => {
                            e.stopPropagation();
                            const toastEl = $el('catalyst-toast-v3');
                            if (toastEl) toastEl.style.display = 'none';
                            if (catalystToastTimer) clearTimeout(catalystToastTimer);
                        };
                    }

                    const bullBtn = $el('btn-test-bullish');
                    if (bullBtn) bullBtn.addEventListener('click', () => { invoke('test_catalyst_alert'); });

                    const bearBtn = $el('btn-test-bearish');
                    if (bearBtn) {
                        bearBtn.onclick = () => { window.invoke('test_catalyst_bearish'); };
                    }

                    // Clear log button
                    const clearBtn = $el('catalyst-clear-log');
                    if (clearBtn) {
                        clearBtn.onclick = () => {
                            const tbody = $el('catalyst-log-body');
                            if (tbody) tbody.innerHTML = '<tr id="catalyst-log-empty"><td colspan="7" style="padding:30px; text-align:center; color:#555; font-style:italic;">Log cleared.</td></tr>';
                            const rowCount = $el('catalyst-row-count');
                            if (rowCount) rowCount.textContent = '0 / 100 max';
                        };
                    }

                    // Filter logic
                    const filterSym = $el('catalyst-filter-sym');
                    const filterScore = $el('catalyst-filter-score');
                    const filterSent = $el('catalyst-filter-sentiment');
                    [filterSym, filterScore, filterSent].forEach(el => {
                        if (el) el.addEventListener('input', applyCatalystFilters);
                    });
                };

                initButtons();
                setTimeout(initButtons, 2000);
            });
        }

        function applyCatalystFilters() {
            const symFilter = ($el('catalyst-filter-sym')?.value || '').toUpperCase();
            const minScore = parseFloat($el('catalyst-filter-score')?.value || '0') || 0;
            const sentFilter = $el('catalyst-filter-sentiment')?.value || '';
            const tbody = $el('catalyst-log-body');
            if (!tbody) return;
            const rows = tbody.querySelectorAll('tr:not(#catalyst-log-empty)');
            rows.forEach(row => {
                const sym = (row.dataset.sym || '').toUpperCase();
                const score = parseFloat(row.dataset.score || '0');
                const sent = row.dataset.sentiment || '';
                const matchSym = !symFilter || sym.includes(symFilter);
                const matchScore = score >= minScore;
                const matchSent = !sentFilter || sent === sentFilter;
                row.style.display = (matchSym && matchScore && matchSent) ? '' : 'none';
            });
        }

        // 2-tone Web Audio alert (bearish = lower pitch)
        function catalystPlayAlert(isBearish) {
            try {
                const ctx = new (window.AudioContext || window.webkitAudioContext)();
                const now = ctx.currentTime;
                const freq1 = isBearish ? 440 : 880;
                const freq2 = isBearish ? 330 : 1320;
                const osc1 = ctx.createOscillator();
                const gain1 = ctx.createGain();
                osc1.frequency.value = freq1;
                osc1.type = 'sine';
                gain1.gain.setValueAtTime(0.3, now);
                gain1.gain.exponentialRampToValueAtTime(0.01, now + 0.15);
                osc1.connect(gain1).connect(ctx.destination);
                osc1.start(now);
                osc1.stop(now + 0.15);
                const osc2 = ctx.createOscillator();
                const gain2 = ctx.createGain();
                osc2.frequency.value = freq2;
                osc2.type = 'sine';
                gain2.gain.setValueAtTime(0.35, now + 0.25);
                gain2.gain.exponentialRampToValueAtTime(0.01, now + 0.45);
                osc2.connect(gain2).connect(ctx.destination);
                osc2.start(now + 0.25);
                osc2.stop(now + 0.45);
                setTimeout(() => ctx.close(), 600);
            } catch (e) { TitanDebug.log('audio', 'Audio alert unavailable:', e); }
        }

        // ═══════════════════════════════════════════════════════════════

        // GLOBAL HARDWARE LOOP (Bypasses window.onload issues)
        function startGlobalHardwareLoop() {
            if (window.globalMonitorRunning) return;
            window.globalMonitorRunning = true;
            TitanDebug.log('hw', 'STARTING GLOBAL HARDWARE LOOP');

            // setTxt -> titan_core.js
            const fmtTemp = (v) => (v && v > 0 ? `${v.toFixed(0)}°C` : "--°C");
            const fmtHwPct = (v) => (Number.isFinite(v) ? `${v.toFixed(0)}%` : "--%");
            const fmtMb = (v) => (v && v > 0 ? `${v.toFixed(0)} MB` : "-- MB");
            const fmtW = (v) => (v && v > 0 ? `${v.toFixed(0)} W` : "-- W");

            const updateGlobal = async () => {
                try {
                    if (typeof window.invoke !== 'function') {
                        console.error("GLOBAL HW ERROR: window.invoke is missing!");
                        return;
                    }
                    const snap = await window.invoke("get_monitor_snapshot", {});
                    if (!snap || !snap.hardware) return;

                    const hw = snap.hardware;

                    setTxt('gpu-name', hw.gpu_name || "GPU");
                    setTxt('gpu-temp', fmtTemp(hw.gpu_temp));
                    setTxt('gpu-util', fmtHwPct(hw.gpu_util));
                    setTxt('gpu-mem', fmtMb(hw.gpu_mem));
                    setTxt('gpu-power', fmtW(hw.gpu_power));

                    const fanGpu = (Number.isFinite(hw.gpu_fan)) ? `${hw.gpu_fan.toFixed(0)} RPM` : "-- RPM";
                    setTxt('gpu-fan', fanGpu);

                    setTxt('cpu-usage', fmtHwPct(hw.cpu_usage));
                    setTxt('cpu-temp', fmtTemp(hw.cpu_temp));

                    const statusEl = $el('connection-status');
                    if (statusEl && !statusEl.innerText.includes('LIVE')) {
                        statusEl.innerText = "⚡ HW ONLINE";
                        statusEl.style.color = "#00BFFF";
                    }

                    const tauriStatus = $el('tauri-status');
                    if (tauriStatus && snap.redis_online !== undefined) {
                        const redisText = snap.redis_online ? "<span style='color:#00BFFF'>+ REDIS</span>" : "<span style='color:red'>- REDIS</span>";
                        tauriStatus.innerHTML = `TAURI LINKED ${redisText}`;
                    }



                } catch (e) {
                    console.error("GLOBAL HW ERROR:", e);
                }
            };

            updateGlobal();
            TitanTimers.register('global:hardware', updateGlobal, 2000);
        }

        setTimeout(startGlobalHardwareLoop, 3000);

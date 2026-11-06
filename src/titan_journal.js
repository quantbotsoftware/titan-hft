(function () {
    let journalData = [];
    let journalSortKey = 'id';
    let journalSortAsc = false;

    const TF_RE = /\b(1m|3m|5m|10m|15m|30m|1h|4h|1d)\b/i;
    const SOURCE_RE = /\b([OGF])\s+(BUY|SELL|TRAIL|EXIT|CHAOS)\b/i;

    function asNum(v) {
        const n = Number(v);
        return Number.isFinite(n) ? n : 0;
    }

    function inferSource(t) {
        const reason = `${t.reason || ''} ${t.notes || ''}`.toUpperCase();
        const m = reason.match(SOURCE_RE);
        if (m) {
            if (m[1] === 'G') return 'GPU';
            if (m[1] === 'O') return 'ORACLE';
            if (m[1] === 'F') return 'FUSION';
        }
        const mode = String(t.strategy_mode || '').toUpperCase();
        if (mode.includes('GPU')) return 'GPU';
        if (mode.includes('ORACLE') || mode.includes('MATH_ORACLE')) return 'ORACLE';
        if (mode.includes('FUSION')) return 'FUSION';
        if (mode.includes('SIGNAL')) return 'SIGNAL';
        return 'OTHER';
    }

    function inferTimeframe(t) {
        const probe = `${t.reason || ''} ${t.notes || ''} ${t.strategy_mode || ''}`;
        const m = probe.match(TF_RE);
        return m ? m[1].toLowerCase() : 'n/a';
    }

    function enrichTrade(t) {
        const sourceExplicit = String(t.source || '').trim().toUpperCase();
        const tfExplicit = String(t.timeframe || '').trim().toLowerCase();
        const source = (sourceExplicit && sourceExplicit !== 'UNKNOWN')
            ? sourceExplicit
            : inferSource(t);
        const tf = (tfExplicit && tfExplicit !== 'n/a')
            ? tfExplicit
            : inferTimeframe(t);
        return {
            ...t,
            __pnl: asNum(t.pnl),
            __source: source || 'OTHER',
            __tf: tf || 'n/a',
        };
    }

    function summarizeBy(rows, keyFn) {
        const map = new Map();
        rows.forEach((r) => {
            const key = keyFn(r) || 'n/a';
            const prev = map.get(key) || { key, trades: 0, wins: 0, pnl: 0 };
            prev.trades += 1;
            if (asNum(r.__pnl) > 0) prev.wins += 1;
            prev.pnl += asNum(r.__pnl);
            map.set(key, prev);
        });
        return Array.from(map.values())
            .map((x) => ({
                ...x,
                winRate: x.trades > 0 ? (x.wins / x.trades) * 100 : 0,
            }))
            .sort((a, b) => b.trades - a.trades || b.pnl - a.pnl);
    }

    function renderKpiGroup(targetId, summary, emptyLabel) {
        const el = $el(targetId);
        if (!el) return;
        if (!summary.length) {
            el.innerHTML = `<span class="journal-kpi-chip">${escapeHtml(emptyLabel)}</span>`;
            return;
        }
        el.innerHTML = summary.slice(0, 8).map((s) => {
            const pnlClr = pnlColor(s.pnl);
            return `<div class="journal-kpi-chip">
                <span class="k">${escapeHtml(s.key.toUpperCase())}</span>
                <span class="v">N:${s.trades}</span>
                <span class="v">WR:${s.winRate.toFixed(0)}%</span>
                <span class="v" style="color:${pnlClr};">PnL:${s.pnl.toFixed(2)}</span>
            </div>`;
        }).join('');
    }

    function renderJournalAdvancedKpis() {
        const srcSummary = summarizeBy(journalData, (r) => r.__source);
        const tfSummary = summarizeBy(journalData, (r) => r.__tf);
        renderKpiGroup('journal-kpi-source', srcSummary, 'No source stats');
        renderKpiGroup('journal-kpi-tf', tfSummary, 'No timeframe stats');
    }

    async function loadJournalStats() {
        try {
            const stats = await invoke('get_journal_stats');
            $el('j-total').textContent = stats.total_trades;
            const wr = $el('j-winrate');
            wr.textContent = stats.total_trades > 0 ? stats.win_rate.toFixed(1) + '%' : '--';
            wr.style.color = pnlColor(stats.win_rate - 50);
            const pnl = $el('j-pnl');
            pnl.textContent = '$' + stats.total_pnl.toFixed(2);
            pnl.style.color = pnlColor(stats.total_pnl);
            const best = $el('j-best');
            best.textContent = stats.best_trade > -Infinity ? '$' + stats.best_trade.toFixed(2) : '--';
            best.style.color = '#00BFFF';
            const worst = $el('j-worst');
            worst.textContent = stats.worst_trade < Infinity ? '$' + stats.worst_trade.toFixed(2) : '--';
            worst.style.color = '#ff4444';
            $el('j-pf').textContent = stats.profit_factor > 0 ? stats.profit_factor.toFixed(2) : '--';
        } catch (e) { TitanError.report('journal:stats', e); }
    }

    async function loadJournalTrades(filters) {
        try {
            const params = filters || {};
            const trades = await invoke('get_journal_trades', {
                symbol: params.symbol || null,
                side: params.side || null,
                dateFrom: params.dateFrom || null,
                dateTo: params.dateTo || null,
                minPnl: params.minPnl || null,
            });
            journalData = (trades || []).map(enrichTrade);
            renderJournalTable();
            renderJournalAdvancedKpis();
            $el('journal-empty').style.display = journalData.length === 0 ? 'block' : 'none';
        } catch (e) { TitanError.report('journal:trades', e); }
    }

    function renderJournalTable() {
        const tbody = $el('journal-tbody');
        if (!tbody) return;
        const sorted = [...journalData].sort((a, b) => {
            let va = a[journalSortKey], vb = b[journalSortKey];
            if (typeof va === 'string') { va = va.toLowerCase(); vb = (vb || '').toLowerCase(); }
            if (va < vb) return journalSortAsc ? -1 : 1;
            if (va > vb) return journalSortAsc ? 1 : -1;
            return 0;
        });
        tbody.innerHTML = sorted.map(t => {
            const pnlClr = pnlColor(t.__pnl);
            const sideClr = sideColor(t.side);
            const ts = String(t.timestamp || '').replace('T', ' ').replace('Z', '');
            return `<tr style="border-bottom:1px solid #222; transition:background 0.15s;" onmouseover="this.style.background='#1a1a2e'" onmouseout="this.style.background='transparent'">
                <td style="padding:6px; color:#555;">${escapeHtml(String(t.id))}</td>
                <td style="padding:6px; color:#aaa; font-size:11px;">${escapeHtml(ts)}</td>
                <td style="padding:6px; color:#fff; font-weight:600;">${escapeHtml(t.symbol)}</td>
                <td style="padding:6px; color:${sideClr}; font-weight:600;">${escapeHtml(t.side)}</td>
                <td style="padding:6px; text-align:right; color:#ccc;">${asNum(t.qty).toFixed(4)}</td>
                <td style="padding:6px; text-align:right; color:#ccc;">$${asNum(t.entry_price).toFixed(2)}</td>
                <td style="padding:6px; text-align:right; color:#ccc;">$${asNum(t.exit_price).toFixed(2)}</td>
                <td style="padding:6px; text-align:right; color:${pnlClr}; font-weight:700;">$${asNum(t.__pnl).toFixed(2)}</td>
                <td style="padding:6px; text-align:right; color:${pnlClr};">${asNum(t.pnl_pct).toFixed(2)}%</td>
                <td style="padding:6px; color:#888; font-size:11px;">${escapeHtml(t.strategy_mode || '')}</td>
                <td style="padding:6px; color:#888; font-size:11px;">${escapeHtml(t.reason || '')}</td>
            </tr>`;
        }).join('');
    }

    function escapeCsvValue(v) {
        const s = String(v ?? '');
        if (s.includes('"') || s.includes(',') || s.includes('\n')) {
            return `"${s.replace(/"/g, '""')}"`;
        }
        return s;
    }

    function buildJournalCsv(rows) {
        const headers = [
            'ID', 'Timestamp', 'Symbol', 'Side', 'Qty', 'EntryPrice', 'ExitPrice', 'PnL',
            'PnL%', 'Strategy', 'Reason', 'Source', 'Timeframe',
        ];
        const lines = [headers.join(',')];
        rows.forEach((r) => {
            const vals = [
                r.id,
                r.timestamp,
                r.symbol,
                r.side,
                asNum(r.qty).toFixed(6),
                asNum(r.entry_price).toFixed(4),
                asNum(r.exit_price).toFixed(4),
                asNum(r.__pnl).toFixed(4),
                asNum(r.pnl_pct).toFixed(4),
                r.strategy_mode || '',
                r.reason || '',
                r.__source || 'n/a',
                r.__tf || 'n/a',
            ].map(escapeCsvValue);
            lines.push(vals.join(','));
        });
        return lines.join('\n');
    }

    function downloadCsv(filename, content) {
        const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.style.display = 'none';
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
    }

    function exportViewCsv() {
        const stamp = new Date().toISOString().replace(/[:.]/g, '-');
        const csv = buildJournalCsv(journalData);
        downloadCsv(`titan_journal_view_${stamp}.csv`, csv);
        if (typeof titanLogAdd === 'function') {
            titanLogAdd(`Exported filtered journal view (${journalData.length} rows)`, '#00BFFF');
        }
    }

    function wireJournal() {
        document.querySelectorAll('#journal-table thead th[data-sort]').forEach(th => {
            th.addEventListener('click', () => {
                const key = th.dataset.sort;
                if (journalSortKey === key) journalSortAsc = !journalSortAsc;
                else { journalSortKey = key; journalSortAsc = true; }
                renderJournalTable();
            });
        });

        const btnFilter = $el('btn-journal-filter');
        if (btnFilter) btnFilter.addEventListener('click', () => {
            const minPnlStr = $el('j-filter-minpnl')?.value || '';
            const minPnl = minPnlStr ? parseFloat(minPnlStr) : null;
            loadJournalTrades({
                symbol: $el('j-filter-symbol')?.value || '',
                side: $el('j-filter-side')?.value || '',
                dateFrom: $el('j-filter-from')?.value || '',
                dateTo: $el('j-filter-to')?.value || '',
                minPnl: (minPnl !== null && !isNaN(minPnl)) ? minPnl : null,
            });
        });

        const btnReset = $el('btn-journal-reset');
        if (btnReset) btnReset.addEventListener('click', () => {
            ['j-filter-symbol', 'j-filter-side', 'j-filter-from', 'j-filter-to', 'j-filter-minpnl'].forEach(id => {
                const el = $el(id);
                if (el) el.value = '';
            });
            loadJournalTrades();
        });

        const btnExportView = $el('btn-journal-export-view');
        if (btnExportView) btnExportView.addEventListener('click', () => exportViewCsv());

        const btnExport = $el('btn-journal-export');
        if (btnExport) btnExport.addEventListener('click', async () => {
            try {
                const result = await invoke('export_journal_csv');
                if (typeof titanLogAdd === 'function') titanLogAdd(result, '#00D9FF');
            } catch (e) {
                if (typeof titanLogAdd === 'function') titanLogAdd('Backend export failed: ' + normalizeError(e), '#FF2B2B');
            }
            exportViewCsv();
        });
    }

    TitanBus.on('tabSwitch', function (index) {
        if (index === 3) {
            loadJournalStats();
            const sym = $el('j-filter-symbol')?.value || '';
            const side = $el('j-filter-side')?.value || '';
            const from = $el('j-filter-from')?.value || '';
            const to = $el('j-filter-to')?.value || '';
            const minPnlStr = $el('j-filter-minpnl')?.value || '';
            const minPnl = minPnlStr ? parseFloat(minPnlStr) : null;
            const hasFilter = sym || side || from || to || minPnl !== null;
            loadJournalTrades(hasFilter ? { symbol: sym, side, dateFrom: from, dateTo: to, minPnl } : undefined);
        }
    });

    document.addEventListener('DOMContentLoaded', wireJournal);

    // ═══ S15 B2: Hour-of-Day PnL Heatmap ═══
    const DAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI'];
    const HOURS_START = 9.5;
    const BUCKET_SIZE = 0.5;
    const BUCKETS = 13;

    function bucketTrades(trades) {
        const grid = {};
        for (let d = 0; d < 5; d++) {
            for (let b = 0; b < BUCKETS; b++) {
                const key = d + ',' + b;
                grid[key] = { pnl: 0, count: 0, wins: 0 };
            }
        }
        for (const t of trades) {
            const pnl = asNum(t.pnl || t.realized_pnl || 0);
            const ts = t.closed_at || t.timestamp || '';
            if (!ts) continue;
            const dt = new Date(ts);
            let dayIdx = dt.getUTCDay() - 1;
            if (dayIdx < 0 || dayIdx > 4) continue;
            const etOffset = 4;
            let hours = dt.getUTCHours() - etOffset + dt.getUTCMinutes() / 60;
            if (hours < 0) hours += 24;
            const bucketIdx = Math.floor((hours - HOURS_START) / BUCKET_SIZE);
            if (bucketIdx < 0 || bucketIdx >= BUCKETS) continue;
            const key = dayIdx + ',' + bucketIdx;
            if (grid[key]) {
                grid[key].pnl += pnl;
                grid[key].count++;
                if (pnl > 0) grid[key].wins++;
            }
        }
        return grid;
    }

    function pnlColor(val, maxAbs) {
        if (maxAbs <= 0) return 'rgba(17,17,34,0.8)';
        const norm = Math.min(Math.abs(val) / maxAbs, 1);
        const alpha = 0.15 + norm * 0.7;
        if (val > 0) return 'rgba(0,255,65,' + alpha.toFixed(2) + ')';
        if (val < 0) return 'rgba(255,43,43,' + alpha.toFixed(2) + ')';
        return 'rgba(17,17,34,0.8)';
    }

    function renderHeatmap(trades) {
        const canvas = $el('journal-heatmap-canvas');
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        const dpr = window.devicePixelRatio || 1;
        const cw = 520, ch = 180;
        canvas.width = cw * dpr;
        canvas.height = ch * dpr;
        ctx.scale(dpr, dpr);
        canvas.style.width = cw + 'px';
        canvas.style.height = ch + 'px';

        ctx.fillStyle = '#080812';
        ctx.fillRect(0, 0, cw, ch);

        const grid = bucketTrades(trades);
        let maxAbs = 0;
        for (const k in grid) { maxAbs = Math.max(maxAbs, Math.abs(grid[k].pnl)); }

        const labelW = 36, labelH = 18;
        const cellW = (cw - labelW) / BUCKETS;
        const cellH = (ch - labelH) / 5;

        ctx.font = '9px "Share Tech Mono"';
        ctx.fillStyle = '#778899';
        ctx.textAlign = 'center';
        for (let b = 0; b < BUCKETS; b++) {
            const hour = HOURS_START + b * BUCKET_SIZE;
            const h = Math.floor(hour);
            const m = (hour % 1) * 60;
            const label = h + ':' + (m < 10 ? '0' : '') + Math.round(m);
            ctx.fillText(label, labelW + b * cellW + cellW / 2, labelH - 4);
        }

        ctx.textAlign = 'right';
        for (let d = 0; d < 5; d++) {
            ctx.fillStyle = '#778899';
            ctx.fillText(DAYS[d], labelW - 4, labelH + d * cellH + cellH / 2 + 3);
        }

        for (let d = 0; d < 5; d++) {
            for (let b = 0; b < BUCKETS; b++) {
                const data = grid[d + ',' + b];
                const x = labelW + b * cellW;
                const y = labelH + d * cellH;
                ctx.fillStyle = pnlColor(data.pnl, maxAbs);
                ctx.fillRect(x + 1, y + 1, cellW - 2, cellH - 2);
                if (data.count > 0 && cellW > 20) {
                    ctx.fillStyle = data.pnl >= 0 ? '#FAFAFA' : '#FAFAFA';
                    ctx.font = '8px "Share Tech Mono"';
                    ctx.textAlign = 'center';
                    const sign = data.pnl >= 0 ? '+' : '';
                    ctx.fillText(sign + data.pnl.toFixed(0), x + cellW / 2, y + cellH / 2 + 3);
                }
            }
        }

        canvas.onmousemove = function(e) {
            const rect = canvas.getBoundingClientRect();
            const mx = e.clientX - rect.left;
            const my = e.clientY - rect.top;
            const scaleX = cw / rect.width;
            const scaleY = ch / rect.height;
            const sx = mx * scaleX, sy = my * scaleY;
            const b = Math.floor((sx - labelW) / cellW);
            const d = Math.floor((sy - labelH) / cellH);
            const tip = $el('journal-heatmap-tooltip');
            if (!tip) return;
            if (b < 0 || b >= BUCKETS || d < 0 || d >= 5) { tip.hidden = true; return; }
            const data = grid[d + ',' + b];
            const hour = HOURS_START + b * BUCKET_SIZE;
            const h = Math.floor(hour);
            const m = (hour % 1) * 60;
            const nextH = h + (m >= 30 ? 1 : 0);
            const nextM = m >= 30 ? 0 : 30;
            const wr = data.count > 0 ? Math.round(data.wins / data.count * 100) : 0;
            const sign = data.pnl >= 0 ? '+' : '';
            tip.innerHTML = DAYS[d] + ' ' + h + ':' + (m < 10 ? '0' : '') + Math.round(m) + '-' + nextH + ':' + (nextM < 10 ? '0' : '') + nextM + ' | P&L: ' + sign + '$' + Math.abs(data.pnl).toFixed(2) + ' | Trades: ' + data.count + ' | WR: ' + wr + '%';
            tip.style.left = (e.clientX + 10) + 'px';
            tip.style.top = (e.clientY - 20) + 'px';
            tip.hidden = false;
        };
        canvas.onmouseleave = function() {
            const tip = $el('journal-heatmap-tooltip');
            if (tip) tip.hidden = true;
        };
    }

    const origLoadTrades = loadJournalTrades;
    loadJournalTrades = async function(filters) {
        await origLoadTrades(filters);
        if (journalData && journalData.length > 0) {
            renderHeatmap(journalData);
        }
    };
})();

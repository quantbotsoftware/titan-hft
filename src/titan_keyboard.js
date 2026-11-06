// ═══════════════════════════════════════════════════════════════════
//  TITAN KEYBOARD — Global Shortcuts + Command Palette (Ctrl+K)
// ═══════════════════════════════════════════════════════════════════

(function () {
    'use strict';

    const TIMEFRAMES = ['1m', '3m', '5m', '10m', '15m', '30m', '1h', '4h', '1d'];

    const COMMANDS = [
        { id: 'buy', label: 'Quick Buy (market)', shortcut: 'Ctrl+Enter', category: 'Trade', action: quickBuy },
        { id: 'sell', label: 'Quick Sell (market)', shortcut: 'Ctrl+Shift+Enter', category: 'Trade', action: quickSell },
        { id: 'flatten', label: 'Flatten All Positions', shortcut: 'Ctrl+Shift+X', category: 'Trade', action: flattenAll },
        { id: 'autopilot', label: 'Toggle Autopilot', shortcut: 'Ctrl+P', category: 'Control', action: toggleAutopilot },
        { id: 'tab-chart', label: 'Go to Chart', shortcut: '1', category: 'Navigate', action: () => switchTab(0) },
        { id: 'tab-news', label: 'Go to News AI', shortcut: '2', category: 'Navigate', action: () => switchTab(1) },
        { id: 'tab-journal', label: 'Go to Journal', shortcut: '3', category: 'Navigate', action: () => switchTab(3) },
        { id: 'tab-hft', label: 'Go to HFT Engine', shortcut: '4', category: 'Navigate', action: () => switchTab(4) },
        { id: 'tab-perf', label: 'Go to Performance', shortcut: '5', category: 'Navigate', action: () => switchTab(5) },
        { id: 'tab-scanner', label: 'Go to AI Scanner', shortcut: '6', category: 'Navigate', action: () => switchTab(6) },
        { id: 'tab-watchlist', label: 'Go to Watchlist', shortcut: '9', category: 'Navigate', action: () => switchTab(9) },
        { id: 'tab-targets', label: 'Go to HFT Targets', shortcut: '0', category: 'Navigate', action: () => switchTab(10) },
        { id: 'tf-up', label: 'Timeframe Higher', shortcut: 'Ctrl+]', category: 'Chart', action: () => cycleTF(1) },
        { id: 'tf-down', label: 'Timeframe Lower', shortcut: 'Ctrl+[', category: 'Chart', action: () => cycleTF(-1) },
        { id: 'tf-1m', label: 'Timeframe → 1m', category: 'Chart', action: () => setTF('1m') },
        { id: 'tf-5m', label: 'Timeframe → 5m', category: 'Chart', action: () => setTF('5m') },
        { id: 'tf-15m', label: 'Timeframe → 15m', category: 'Chart', action: () => setTF('15m') },
        { id: 'tf-1h', label: 'Timeframe → 1h', category: 'Chart', action: () => setTF('1h') },
        { id: 'tf-1d', label: 'Timeframe → 1d', category: 'Chart', action: () => setTF('1d') },
        { id: 'pip-toggle', label: 'Toggle PiP Dashboard', shortcut: 'P', category: 'View', action: () => { if (typeof window._pipToggle === 'function') window._pipToggle(); } },
    ];

    let paletteOpen = false;
    let paletteEl = null;
    let paletteInput = null;
    let paletteList = null;
    let filteredCommands = [];
    let selectedIdx = 0;

    // ── Actions ──────────────────────────────────────────────────

    function resolveQuickManualOrderAmount() {
        const mtUsd = Number($el('mt-usd')?.value || 0);
        const isShares = $el('mt-mode-shares')?.checked;
        const maxOrd = window._maxManualOrderUsd || 50000;
        if (mtUsd > 0) {
            if (!isShares && mtUsd > maxOrd) {
                return { error: 'Quick trade: max $' + maxOrd.toLocaleString() + ' (manual cap)' };
            }
            return isShares ? { share_qty: mtUsd } : { usd_amount: mtUsd };
        }
        const botUsd = Number($el('bot-trade-size')?.value || 0);
        if (botUsd > 0) {
            if (botUsd > maxOrd) {
                return { error: 'Quick trade: max $' + maxOrd.toLocaleString() + ' (manual cap)' };
            }
            return { usd_amount: botUsd };
        }
        return { error: 'Set Manual Trade amount or bot trade size before quick buy/sell.' };
    }

    async function quickBuy() {
        const sym = typeof currentSymbol !== 'undefined' ? currentSymbol : '';
        if (!sym || sym === '---') return;
        const sizing = resolveQuickManualOrderAmount();
        if (sizing.error) {
            showCyberToast(sizing.error, 'error');
            return;
        }
        const symbol = String(sym).replace(/\//g, '').trim().toUpperCase();
        const order = { symbol, side: 'buy' };
        if (sizing.share_qty != null) order.share_qty = sizing.share_qty;
        else order.usd_amount = sizing.usd_amount;
        try {
            const result = await invoke('submit_manual_order', { order });
            showCyberToast('BUY ' + symbol + ': ' + (result || 'OK'), 'success');
        } catch (e) {
            showCyberToast('BUY FAILED: ' + normalizeError(e), 'error');
        }
    }

    async function quickSell() {
        const sym = typeof currentSymbol !== 'undefined' ? currentSymbol : '';
        if (!sym || sym === '---') return;
        const sizing = resolveQuickManualOrderAmount();
        if (sizing.error) {
            showCyberToast(sizing.error, 'error');
            return;
        }
        const symbol = String(sym).replace(/\//g, '').trim().toUpperCase();
        const order = { symbol, side: 'sell' };
        if (sizing.share_qty != null) order.share_qty = sizing.share_qty;
        else order.usd_amount = sizing.usd_amount;
        try {
            const result = await invoke('submit_manual_order', { order });
            showCyberToast('SELL ' + symbol + ': ' + (result || 'OK'), 'success');
        } catch (e) {
            showCyberToast('SELL FAILED: ' + normalizeError(e), 'error');
        }
    }

    async function flattenAll() {
        try {
            const result = await invoke('emergency_liquidate', { confirmToken: 'CONFIRM_EMERGENCY' });
            showCyberToast(result || 'FLATTEN COMPLETE', 'warning');
        } catch (e) {
            showCyberToast('FLATTEN FAILED: ' + normalizeError(e), 'error');
        }
    }

    function toggleAutopilot() {
        const el = $el('chk-auto');
        if (el) el.click();
    }

    function cycleTF(direction) {
        const cur = typeof currentTF !== 'undefined' ? currentTF : '5m';
        const idx = TIMEFRAMES.indexOf(cur);
        if (idx < 0) return;
        const next = idx + direction;
        if (next >= 0 && next < TIMEFRAMES.length) {
            setTF(TIMEFRAMES[next]);
        }
    }

    // ── Palette UI ───────────────────────────────────────────────

    function buildPalette() {
        if (paletteEl) return;

        const overlay = document.createElement('div');
        overlay.id = 'titan-cmd-overlay';
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) closePalette();
        });

        const container = document.createElement('div');
        container.id = 'titan-cmd-palette';

        const input = document.createElement('input');
        input.id = 'titan-cmd-input';
        input.type = 'text';
        input.placeholder = 'Type a command or symbol...';
        input.spellcheck = false;
        input.autocomplete = 'off';

        const list = document.createElement('div');
        list.id = 'titan-cmd-list';

        container.appendChild(input);
        container.appendChild(list);
        overlay.appendChild(container);
        document.body.appendChild(overlay);

        paletteEl = overlay;
        paletteInput = input;
        paletteList = list;

        input.addEventListener('input', onPaletteInput);
        input.addEventListener('keydown', onPaletteKeydown);
    }

    function openPalette() {
        buildPalette();
        paletteOpen = true;
        paletteEl.classList.add('visible');
        paletteInput.value = '';
        selectedIdx = 0;
        filterAndRender('');
        requestAnimationFrame(() => paletteInput.focus());
    }

    function closePalette() {
        if (!paletteEl) return;
        paletteOpen = false;
        paletteEl.classList.remove('visible');
        paletteInput.blur();
    }

    function onPaletteInput() {
        const q = paletteInput.value.trim().toLowerCase();
        selectedIdx = 0;
        filterAndRender(q);
    }

    function onPaletteKeydown(e) {
        if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            closePalette();
            return;
        }
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            selectedIdx = Math.min(selectedIdx + 1, filteredCommands.length - 1);
            renderList();
            return;
        }
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            selectedIdx = Math.max(selectedIdx - 1, 0);
            renderList();
            return;
        }
        if (e.key === 'Enter') {
            e.preventDefault();
            const cmd = filteredCommands[selectedIdx];
            if (cmd) {
                closePalette();
                cmd.action();
            } else {
                const raw = paletteInput.value.trim().toUpperCase();
                if (raw && typeof setSymbol === 'function') {
                    closePalette();
                    setSymbol(raw, { force: true });
                }
            }
            return;
        }
    }

    function filterAndRender(query) {
        if (!query) {
            filteredCommands = COMMANDS.slice();
        } else {
            filteredCommands = COMMANDS.filter(c => {
                const haystack = (c.label + ' ' + c.category + ' ' + (c.shortcut || '')).toLowerCase();
                return query.split(/\s+/).every(w => haystack.includes(w));
            });
        }
        renderList();
    }

    function renderList() {
        if (!paletteList) return;
        const frag = document.createDocumentFragment();
        const max = Math.min(filteredCommands.length, 15);

        for (let i = 0; i < max; i++) {
            const cmd = filteredCommands[i];
            const row = document.createElement('div');
            row.className = 'titan-cmd-row' + (i === selectedIdx ? ' selected' : '');

            const cat = document.createElement('span');
            cat.className = 'titan-cmd-cat';
            cat.textContent = cmd.category;

            const lbl = document.createElement('span');
            lbl.className = 'titan-cmd-label';
            lbl.textContent = cmd.label;

            const key = document.createElement('span');
            key.className = 'titan-cmd-key';
            key.textContent = cmd.shortcut || '';

            row.appendChild(cat);
            row.appendChild(lbl);
            row.appendChild(key);

            row.addEventListener('click', () => {
                closePalette();
                cmd.action();
            });
            row.addEventListener('mouseenter', () => {
                selectedIdx = i;
                renderList();
            });

            frag.appendChild(row);
        }

        if (filteredCommands.length === 0) {
            const hint = document.createElement('div');
            hint.className = 'titan-cmd-row titan-cmd-hint';
            hint.textContent = 'Press Enter to switch to symbol: ' + paletteInput.value.trim().toUpperCase();
            frag.appendChild(hint);
        }

        paletteList.innerHTML = '';
        paletteList.appendChild(frag);
    }

    // ── Global keydown handler ───────────────────────────────────

    document.addEventListener('keydown', (e) => {
        if (paletteOpen) return;

        // Ctrl+K — open command palette
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            e.stopPropagation();
            openPalette();
            return;
        }

        // Ctrl+Enter — quick buy
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            quickBuy();
            return;
        }

        // Ctrl+Shift+Enter — quick sell
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter' && e.shiftKey) {
            e.preventDefault();
            quickSell();
            return;
        }

        // Ctrl+Shift+X — flatten all
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'x') {
            e.preventDefault();
            flattenAll();
            return;
        }

        // Ctrl+P — toggle autopilot
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {
            e.preventDefault();
            toggleAutopilot();
            return;
        }

        // Ctrl+[ / Ctrl+] — cycle timeframe
        if ((e.ctrlKey || e.metaKey) && e.key === '[') {
            e.preventDefault();
            cycleTF(-1);
            return;
        }
        if ((e.ctrlKey || e.metaKey) && e.key === ']') {
            e.preventDefault();
            cycleTF(1);
            return;
        }

        // S12a: Escape dismisses SmartTrend config compare overlay
        if (e.key === 'Escape') {
            const compareOverlay = $el('st-cfg-compare-overlay');
            if (compareOverlay && !compareOverlay.hidden) {
                e.preventDefault();
                compareOverlay.hidden = true;
                return;
            }
        }

        // S11: SmartTrend trade/exit panel shortcuts (only when panels visible)
        const tradePanel = $el('st-trade-panel');
        const exitPanel = $el('st-exit-panel');
        const tradePanelVisible = tradePanel && !tradePanel.classList.contains('st-trade-hidden');
        const exitPanelVisible = exitPanel && !exitPanel.classList.contains('st-trade-hidden');

        if (tradePanelVisible && !e.ctrlKey && !e.metaKey && !e.altKey) {
            if (e.key === 'Enter') {
                e.preventDefault();
                $el('st-trade-execute')?.click();
                return;
            }
            if (e.key === 'Escape') {
                e.preventDefault();
                $el('st-trade-skip')?.click();
                return;
            }
            if (e.key.toLowerCase() === 'v') {
                e.preventDefault();
                const sym = $el('st-trade-sym')?.textContent;
                if (sym && sym !== '---' && typeof window.setSymbol === 'function') {
                    window.setSymbol(sym);
                }
                return;
            }
            if (['1','2','3','4'].includes(e.key)) {
                e.preventDefault();
                const quickBtns = document.querySelectorAll('.st-quick-amt');
                const idx = parseInt(e.key) - 1;
                if (quickBtns[idx]) quickBtns[idx].click();
                return;
            }
        }

        if (exitPanelVisible && !e.ctrlKey && !e.metaKey && !e.altKey) {
            if (e.key === 'Enter') {
                e.preventDefault();
                $el('st-exit-100')?.click();
                return;
            }
            if (e.key === 'Escape') {
                e.preventDefault();
                $el('st-exit-hold')?.click();
                return;
            }
            if (e.key.toLowerCase() === 'v') {
                e.preventDefault();
                const sym = $el('st-exit-sym')?.textContent;
                if (sym && sym !== '---' && typeof window.setSymbol === 'function') {
                    window.setSymbol(sym);
                }
                return;
            }
        }

    }, true);

})();

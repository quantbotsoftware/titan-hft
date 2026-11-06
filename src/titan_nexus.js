// ═══════════════════════════════════════════════════════
//  GPU NEXUS — Strategy Mode & Consensus JS Logic
// ═══════════════════════════════════════════════════════
let currentNexusMode = 'SMARTTREND';

async function setNexusMode(mode) {
    // Optimistic UI update — respond immediately to click
    const previousMode = currentNexusMode;
    currentNexusMode = mode;
    updateNexusModeUI(mode);

    try {
        if (window.__TAURI__ && window.__TAURI__.invoke) {
            const result = await window.invoke('set_strategy_mode', { mode: mode });
            currentNexusMode = result;
            updateNexusModeUI(result);
            TitanDebug.log('nexus', 'NEXUS MODE →', result);
            if (typeof updateOracleUI === 'function') updateOracleUI();

            if (result === 'SMARTTREND' || previousMode === 'SMARTTREND') {
                _showSmartTrendModeToast(result, previousMode);
            }
        } else {
            console.warn('Tauri not available, UI-only mode switch');
        }
    } catch (e) {
        TitanError.report('nexus:mode-switch', e);
        // Revert on error
        currentNexusMode = previousMode;
        updateNexusModeUI(previousMode);
    }
}

// setTxt, fmtUsd, formatVolume -> titan_core.js
const TITAN_LOG_MAX = 50;
function titanLogAdd(msg, color) {
    const container = $el('titan-log-entries');
    if (!container) return;
    const line = document.createElement('div');
    line.style.color = color || '#888';
    const ts = new Date().toLocaleTimeString('en-US', { hour12: false });
    line.textContent = ts + ' ' + msg;
    container.prepend(line);
    while (container.children.length > TITAN_LOG_MAX) container.removeChild(container.lastChild);
}

let titanDryRun = false;
async function toggleDryRun() {
    titanDryRun = !titanDryRun;
    const btn = $el('btn-dry-run');
    if (btn) {
        btn.textContent = 'DRY RUN: ' + (titanDryRun ? 'ON' : 'OFF');
        btn.style.color = titanDryRun ? '#00BFFF' : '#FFD700';
        btn.style.borderColor = titanDryRun ? '#00BFFF' : '#333';
    }
    try { await window.invoke('set_dry_run', { enabled: titanDryRun }); }
    catch (e) { TitanError.report('nexus:set-dry-run', e); }
    titanLogAdd('Dry run ' + (titanDryRun ? 'ENABLED' : 'DISABLED'), titanDryRun ? '#00BFFF' : '#FFD700');
    if (typeof TitanBus !== 'undefined') TitanBus.emit('dryRunChanged', titanDryRun);
}

async function gracefulShutdown() {
    if (!(await cyberConfirm('Shut down Titan engine? All positions will be preserved.'))) return;
    titanLogAdd('Initiating graceful shutdown...', '#FF6B6B');
    try { await window.invoke('graceful_shutdown'); }
    catch (e) { TitanError.report('nexus:shutdown', e); }
}

async function emergencyLiquidate() {
    if (!(await cyberConfirm('EMERGENCY: Liquidate ALL positions immediately?'))) return;
    titanLogAdd('EMERGENCY LIQUIDATE triggered!', '#FF2B2B');
    try { await window.invoke('liquidate_all_positions', { confirmToken: 'CONFIRM_LIQUIDATE' }); }
    catch (e) { TitanError.report('nexus:liquidate', e); }
}

async function resetFullSafety() {
    if (!(await cyberConfirm('Reset ALL safety metrics? (Max DD, PnL, trades, breaker)'))) return;
    try {
        await window.invoke('reset_full_safety');
        titanLogAdd('TITAN SAFETY FULL RESET — all metrics cleared', '#00d4ff');
    } catch (e) {
        TitanError.report('nexus:reset-safety', e);
        titanLogAdd('Reset failed: ' + normalizeError(e), '#FF2B2B');
    }
}

let evoFrozen = false;
async function toggleFreezeEvo() {
    evoFrozen = !evoFrozen;
    const btn = $el('btn-freeze-evo');
    if (btn) {
        btn.textContent = evoFrozen ? 'UNFREEZE' : 'FREEZE';
        btn.style.color = evoFrozen ? '#FF6B6B' : '#4ecdc4';
        btn.style.borderColor = evoFrozen ? '#FF6B6B' : '#333';
    }
    try { await window.invoke(evoFrozen ? 'freeze_evolution' : 'resume_evolution'); }
    catch (e) { TitanError.report('nexus:freeze-evo', e); }
    titanLogAdd('Evolution ' + (evoFrozen ? 'FROZEN' : 'RESUMED'), evoFrozen ? '#FF6B6B' : '#4ecdc4');
}

async function runBacktestNow() {
    titanLogAdd('Manual backtest triggered...', '#FFD700');
    try {
        const result = await window.invoke('run_backtest_now');
        titanLogAdd('Backtest done: fitness=' + (result.fitness || '--').toString().substr(0, 6), '#00BFFF');
    } catch (e) {
        titanLogAdd('Backtest error: ' + normalizeError(e), '#FF6B6B');
    }
}

/* Periodic risk status polling */
TitanTimers.register('nexus:risk-poll', async () => {
    try {
        const r = await window.invoke('get_risk_status');
        if (r) {
            const lvlEl = $el('titan-breaker-level');
            const lvl = r.breaker_level || 'NORMAL';
            if (lvlEl) {
                lvlEl.textContent = lvl;
                lvlEl.style.background = lvl === 'NORMAL' ? '#00BFFF' : lvl === 'CAUTION' ? '#FFD700' : '#FF2B2B';
                lvlEl.style.color = lvl === 'NORMAL' ? '#0a0a0f' : lvl === 'CAUTION' ? '#0a0a0f' : '#FFF';
            }
            const s = r.session || {};
            const pnl = parseFloat(s.daily_pnl) || 0;
            setTxt('titan-daily-pnl', fmtUsd(pnl), pnlColor(pnl));
            setTxt('titan-max-dd', (parseFloat(s.max_drawdown_pct) || 0).toFixed(1) + '%', '#FF6B6B');
            setTxt('titan-win-rate', s.total_trades > 0 ? (parseFloat(s.win_rate_pct) || 0).toFixed(0) + '%' : '--');
            setTxt('titan-total-trades', s.total_trades || 0);
            setTxt('titan-sharpe', s.sharpe !== undefined ? parseFloat(s.sharpe).toFixed(2) : '--', '#4ecdc4');
            setTxt('titan-pf', s.profit_factor ? parseFloat(s.profit_factor).toFixed(2) : '--', '#4ecdc4');
            setTxt('titan-equity', fmtUsd(parseFloat(s.current_equity) || 0), '#FAFAFA');
        }
    } catch (e) { TitanError.report('nexus:risk-poll', e); }
}, 5000);

/* OFI polling */
TitanTimers.register('nexus:ofi-poll', async () => {
    try {
        const o = await window.invoke('get_ofi_status');
        renderOfi(o);
    } catch (e) { TitanError.report('nexus:ofi-poll', e); }
}, 2000);

/* Telemetry polling — loop heartbeats */
TitanTimers.register('nexus:telemetry-poll', async () => {
    try {
        const t = await window.invoke('get_telemetry');
        if (!t) return;
        const grid = $el('telemetry-grid');
        const deadEl = $el('telemetry-dead-count');
        if (grid && t.loops) {
            grid.innerHTML = t.loops.map(l => {
                const alive = l.is_alive;
                const clr = alive ? '#00BFFF' : '#FF2B2B';
                const ago = l.last_heartbeat_secs_ago.toFixed(0);
                return '<div style="color:' + clr + '">' + escapeHtml(l.name) + ' <span style="color:#555">' + ago + 's/' + l.tick_count + '</span></div>';
            }).join('');
        }
        if (deadEl) {
            const dead = t.dead_loops || [];
            if (dead.length > 0) {
                deadEl.textContent = dead.length + ' DEAD';
                deadEl.style.color = '#FF2B2B';
            } else {
                deadEl.textContent = 'ALL ALIVE';
                deadEl.style.color = '#00BFFF';
            }
        }
    } catch (e) { TitanError.report('nexus:telemetry-poll', e); }
}, 10000);

function updateNexusModeUI(mode) {
    // Update toggle buttons
    document.querySelectorAll('.nexus-mode-btn').forEach(btn => {
        if (btn.dataset.mode === mode) {
            btn.classList.add('active');
        } else {
            btn.classList.remove('active');
        }
    });
    // Update mode display
    const modeDisplay = $el('nexus-mode-display');
    if (modeDisplay) modeDisplay.textContent = mode;
    // Show/hide alpha slider (only in FUSION mode)
    const alphaRow = $el('nexus-alpha-row');
    if (alphaRow) {
        alphaRow.classList.toggle('alpha-visible', mode === 'FUSION');
    }
    // Show/hide signal source sub-selector (only in SIGNAL_STRATEGY mode)
    const sigSrcToggle = $el('sig-source-toggle');
    if (sigSrcToggle) {
        sigSrcToggle.style.display = mode === 'SIGNAL_STRATEGY' ? 'flex' : 'none';
    }
    if (typeof updateSignalHealthPanel === 'function') updateSignalHealthPanel();
}

function updateNexusConsensus(data) {
    const buyPct = data.buy_pct || 50;
    const sellPct = data.sell_pct || 50;
    const consensus = data.consensus || 'NEUTRAL';

    // Update bar widths
    const buyBar = $el('nexus-bar-buy');
    const sellBar = $el('nexus-bar-sell');
    if (buyBar) {
        buyBar.style.width = buyPct.toFixed(1) + '%';
        buyBar.textContent = buyPct.toFixed(1) + '%';
    }
    if (sellBar) {
        sellBar.style.width = sellPct.toFixed(1) + '%';
        sellBar.textContent = sellPct.toFixed(1) + '%';
    }

    // Update consensus label
    const label = $el('nexus-consensus');
    if (label) {
        label.textContent = consensus.replace('_', ' ');
        label.className = 'nexus-consensus-label ' + consensus;
    }

    // Update GPU latency
    const gpuUs = $el('nexus-gpu-us');
    if (gpuUs && data.dispatch_us !== undefined) {
        gpuUs.textContent = data.dispatch_us.toLocaleString();
    }
}

function bindNexusPanel() {
    // Enforce initial state — hide alpha row, set mode to MATH_ORACLE
    updateNexusModeUI(currentNexusMode);

    // Bind mode toggle buttons via addEventListener (not inline onclick)
    document.querySelectorAll('.nexus-mode-btn').forEach(btn => {
        btn.addEventListener('click', function (e) {
            e.preventDefault();
            e.stopPropagation();
            const mode = this.dataset.mode;
            if (mode) {
                TitanDebug.log('nexus', 'NEXUS BUTTON CLICKED:', mode);
                setNexusMode(mode);
            }
        });
    });
    // Alpha slider
    const slider = $el('nexus-alpha-slider');
    const alphaVal = $el('nexus-alpha-val');
    if (slider) {
        slider.addEventListener('input', async (e) => {
            const alpha = parseInt(e.target.value) / 100;
            if (alphaVal) alphaVal.textContent = alpha.toFixed(2);
            try {
                if (window.__TAURI__ && window.__TAURI__.invoke) {
                    await window.invoke('set_nexus_alpha', { alpha });
                }
            } catch (err) {
                TitanError.report('nexus:alpha-set', err);
            }
        });
    }

    // Signal source sub-selector (Oracle / GPU / Fusion) for Signal Strategy
    document.querySelectorAll('.sig-src-btn').forEach(btn => {
        btn.addEventListener('click', async function(e) {
            e.preventDefault();
            e.stopPropagation();
            const src = this.dataset.src;
            if (!src) return;
            document.querySelectorAll('.sig-src-btn').forEach(b => {
                if (b.dataset.src === src) {
                    b.classList.add('active');
                    b.style.borderColor = '#39FF14';
                    b.style.background = 'rgba(57,255,20,0.15)';
                    b.style.color = '#39FF14';
                } else {
                    b.classList.remove('active');
                    b.style.borderColor = '#555';
                    b.style.background = 'transparent';
                    b.style.color = '#888';
                }
            });
            if (typeof updateSignalHealthPanel === 'function') updateSignalHealthPanel();
            try {
                await invoke('update_settings', { field: 'signal_source', value: src });
                if (typeof updateOracleUI === 'function') updateOracleUI();
            } catch (err) {
                TitanError.report('nexus:signal-source', err);
            }
        });
    });

    // Load signal_source from settings on init
    if (window.__TAURI__ && window.__TAURI__.invoke) {
        window.invoke('get_settings_snapshot').then(s => {
            if (s && s.signal_source) {
                document.querySelectorAll('.sig-src-btn').forEach(b => {
                    if (b.dataset.src === s.signal_source) {
                        b.classList.add('active');
                        b.style.borderColor = '#39FF14';
                        b.style.background = 'rgba(57,255,20,0.15)';
                        b.style.color = '#39FF14';
                    } else {
                        b.classList.remove('active');
                        b.style.borderColor = '#555';
                        b.style.background = 'transparent';
                        b.style.color = '#888';
                    }
                });
                if (typeof updateSignalHealthPanel === 'function') updateSignalHealthPanel();
            }
        }).catch(e => TitanError.report('nexus:settings-sync', e));
    }

    // Listen for real-time Nexus updates from Rust
    if (window.__TAURI__ && window.__TAURI__.event) {
        window.__TAURI__.event.listen('nexus-update', (event) => {
            updateNexusConsensus(event.payload);
        });
        window.__TAURI__.event.listen('strategy-mode-changed', (event) => {
            currentNexusMode = event.payload;
            updateNexusModeUI(event.payload);
        });
    }
}

async function _showSmartTrendModeToast(newMode, prevMode) {
    if (newMode === 'SMARTTREND') {
        let status = 'READY';
        try {
            if (window.__TAURI__ && window.__TAURI__.invoke) {
                const d = await window.invoke('get_smarttrend_diagnostics');
                if (d && d.killed) status = 'KILLED';
                else if (d && !d.enabled) status = 'DISABLED';
            }
        } catch (_) {}
        TitanToast.show('SMARTTREND ACTIVATED — Engine: ' + status, status === 'READY' ? 'success' : 'warning');
        if (typeof TitanAudio !== 'undefined') TitanAudio.play('mode_change');
    } else if (prevMode === 'SMARTTREND') {
        TitanToast.show('SMARTTREND DEACTIVATED — Switched to ' + newMode, 'info');
        if (typeof TitanAudio !== 'undefined') TitanAudio.play('mode_change');
    }
}

// Auto-bind when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bindNexusPanel);
} else {
    bindNexusPanel();
}

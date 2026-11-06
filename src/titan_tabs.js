// --- TABS LOGIC ---
const _tabInitCallbacks = [];
window.registerTabInit = function(fn) { _tabInitCallbacks.push(fn); };

function switchTab(index) {
    TitanDebug.log('tabs', 'Switching to tab:', index);
    // B1 S12a: warn on leaving tab-13 with unsaved SmartTrend config
    if (window._stCfgCurrentTab === 13 && index !== 13 && typeof window._stCfgHasUnsaved === 'function' && window._stCfgHasUnsaved()) {
        if (typeof TitanToast !== 'undefined') {
            TitanToast.show('SmartTrend config has unsaved changes', 'warning', 3000);
        }
    }
    window._stCfgCurrentTab = index;
    // Update Buttons
    const buttons = document.querySelectorAll('.stTab');
    buttons.forEach((btn, i) => {
        if (i === index) btn.classList.add('active');
        else btn.classList.remove('active');
    });

    // Update Content — dynamic: finds all tab-N elements automatically
    document.querySelectorAll('[id^="tab-"]').forEach(el => {
        const match = el.id.match(/^tab-(\d+)$/);
        if (!match) return;
        const i = parseInt(match[1], 10);
        if (i === index) el.classList.add('visible');
        else el.classList.remove('visible');
        el.style.display = '';
    });


    if (index !== 0) {
        const etb = $el('exec-toolbar');
        if (etb) etb.style.display = 'none';
    }
    if (index === 12) {
        setTimeout(resizeCharts, 50);
    }
    if (index === 0 && chart) {
        setTimeout(resizeCharts, 50);
    }
    if (index === 10 && _hftEnrichedCache.length === 0) {
        refreshHftEnriched();
    }
    if (index === 9 && _wlSnapshotCache.length === 0 && _cachedFavorites.length > 0) {
        refreshWatchlistSnapshots();
    }

    for (let i = 0; i < _tabInitCallbacks.length; i++) {
        try { _tabInitCallbacks[i](index); } catch (_) {}
    }

    TitanTimers.onTabSwitch(index);
    TitanBus.emit('tabSwitch', index);
}

function setCmdProgress(label, pct) {
    const labelEl = $el('cmd-progress-label');
    const fillEl = $el('cmd-progress-fill');
    if (labelEl) labelEl.innerText = label;
    if (fillEl) {
        const clamped = Math.max(0, Math.min(100, pct));
        fillEl.style.width = `${clamped}%`;
    }
}

function startCmdCountdown(label, seconds) {
    const total = Math.max(5, Math.floor(seconds || 0));
    let remaining = total;
    setCmdProgress(`${label}: ${remaining}s`, 0);
    TitanTimers.register('tabs:cmd-countdown', () => {
        remaining = Math.max(0, remaining - 1);
        const pct = total > 0 ? ((total - remaining) / total) * 100 : 100;
        setCmdProgress(`${label}: ${remaining}s`, pct);
        if (remaining <= 0) {
            TitanTimers.clear('tabs:cmd-countdown');
        }
    }, 1000);
}

function finishCmdCountdown(label) {
    TitanTimers.clear('tabs:cmd-countdown');
    setCmdProgress(label, 100);
    setTimeout(() => setCmdProgress('TITAN SYSTEM ACTIVE', 0), 1200);
}



function bindTabs() {
    if (tabsBound) return;
    tabsBound = true;
    document.querySelectorAll('.stTab[data-tab]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            switchTab(parseInt(btn.dataset.tab, 10));
        });
    });
}

window.switchTab = switchTab;

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => { bindTabs(); });
} else {
    bindTabs();
}

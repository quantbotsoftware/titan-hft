const _SCALP_TOGGLE_FIELDS = new Set(['scalping_oracle', 'scalping_gpu', 'scalping_fusion']);

function bindToggle(id, field) {
    const el = $el(id);
    if (!el) return;
    el.addEventListener('click', () => {
        const box = el.querySelector('.chk-box');
        if (!box) return;
        const isChecked = box.classList.contains('checked');
        const newState = !isChecked;
        if (newState) box.classList.add('checked');
        else box.classList.remove('checked');
        invoke('update_settings', { field: field, value: newState.toString() });
        refreshSidebarState();
        if (_SCALP_TOGGLE_FIELDS.has(field) && typeof updateOracleUI === 'function') {
            setTimeout(() => updateOracleUI(), 300);
        }
    });
}

function bindSlider(id, field, displayId, suffix = "") {
    const el = $el(id);
    const disp = $el(displayId);
    if (!el) return;
    const updateDisplay = () => {
        if (disp) disp.innerText = el.value + suffix;
        if (id === 'slider-risk') {
            const riskTarget = $el('risk-target');
            if (riskTarget) riskTarget.innerText = el.value;
        }
    };
    const update = () => {
        updateDisplay();
        invoke('update_settings', { field: field, value: el.value.toString() });
        if (id === 'slider-refresh') {
            refreshSidebarState();
        }
    };
    el.addEventListener('input', update);
    updateDisplay();
}

function bindSelect(id, field) {
    const el = $el(id);
    if (!el) return;
    el.addEventListener('change', () => {
        invoke('update_settings', { field: field, value: el.value.toString() });
    });
}

function getToggleState(id) {
    const el = $el(id);
    if (!el) return false;
    const box = el.querySelector('.chk-box');
    return !!(box && box.classList.contains('checked'));
}

function refreshSidebarState() {
    const loopOn = getToggleState('chk-loop-top');
    const scanOn = getToggleState('chk-scan');

    const loopStatus = $el('loop-status');
    const scanBtn = $el('btn-launch-scan');
    const refreshEl = $el('slider-refresh');
    if (loopStatus) {
        loopStatus.style.display = loopOn ? 'block' : 'none';
        if (refreshEl) loopStatus.innerText = `⚡ LOOP ACTIV (REFRESH ${refreshEl.value}s)`;
    }
    if (scanBtn) {
        scanBtn.style.display = loopOn ? 'none' : 'block';
    }

    const scanInfo = $el('scan-status-info');
    if (scanInfo) {
        scanInfo.innerText = scanOn ? 'SCANARE SHARK Activat.' : 'SCANARE SHARK Oprit.';
        scanInfo.style.color = scanOn ? '#00BFFF' : '#888';
    }

    const forceManualOn = getToggleState('chk-force-manual');
    const banner = $el('force-manual-banner');
    if (banner) banner.style.display = forceManualOn ? 'block' : 'none';

    const aggressiveOn = getToggleState('chk-aggressive');
    const aggrIndicator = $el('aggressive-indicator');
    if (aggrIndicator) aggrIndicator.style.display = aggressiveOn ? 'block' : 'none';

    const extendedOn = getToggleState('chk-extended');
    const extBadge = $el('extended-hours-badge');
    if (extBadge) extBadge.style.display = extendedOn ? 'inline-block' : 'none';
}

function storageGet(key, fallback) {
    try {
        const raw = localStorage.getItem(key);
        return raw ? JSON.parse(raw) : fallback;
    } catch (e) {
        return fallback;
    }
}

function storageSet(key, value) {
    try {
        localStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
        // ignore storage errors
    }
}

function loadWallet() {
    return storageGet('titan_wallet_v1', { bot_cash: 0, user_deposit: 0 });
}

function saveWallet(wallet) {
    storageSet('titan_wallet_v1', wallet);
}

async function fetchWallet() {
    if (!window.__TAURI__) {
        return loadWallet();
    }
    try {
        return await invoke('get_wallet_snapshot', {});
    } catch (e) {
        return loadWallet();
    }
}

function applyWalletSnapshot(wallet) {
    const botAllocEl = $el('bot-alloc-val');
    if (botAllocEl) botAllocEl.innerText = formatMoney(wallet.bot_cash || 0);
    const acctBotEl = $el('acct-bot-alloc');
    if (acctBotEl) acctBotEl.innerText = formatMoney(wallet.bot_cash || 0);
    const acctUserEl = $el('acct-user-deposit');
    if (acctUserEl) acctUserEl.innerText = formatMoney(wallet.user_deposit || 0);
    const userDepEl = $el('user-deposit-val');
    if (userDepEl) userDepEl.innerText = formatMoney(wallet.user_deposit || 0);
}

async function updateWalletUI() {
    const wallet = await fetchWallet();
    applyWalletSnapshot(wallet || { bot_cash: 0, user_deposit: 0 });
}

function wireWalletControls() {
    updateWalletUI();
    const mainInput = $el('main-transfer-amount');
    const btnMainTop = $el('btn-main-topup');
    const btnMainW = $el('btn-main-withdraw');
    if (btnMainTop) {
        btnMainTop.addEventListener('click', () => {
            const amt = Number(mainInput?.value || 0);
            if (!Number.isFinite(amt) || amt <= 0) return;
            if (window.__TAURI__) {
                invoke('topup_bot', { amount: amt }).then(updateWalletUI).catch(alert);
            } else {
                const wallet = loadWallet();
                wallet.bot_cash = (wallet.bot_cash || 0) + amt;
                saveWallet(wallet);
                updateWalletUI();
            }
        });
    }
    if (btnMainW) {
        btnMainW.addEventListener('click', () => {
            const amt = Number(mainInput?.value || 0);
            if (!Number.isFinite(amt) || amt <= 0) return;
            if (window.__TAURI__) {
                invoke('withdraw_bot', { amount: amt }).then(updateWalletUI).catch(alert);
            } else {
                const wallet = loadWallet();
                if ((wallet.bot_cash || 0) < amt) {
                    showCyberToast('Fonduri insuficiente in Bot!', 'warning');
                    return;
                }
                wallet.bot_cash = (wallet.bot_cash || 0) - amt;
                saveWallet(wallet);
                updateWalletUI();
            }
        });
    }

    const userInput = $el('user-topup-amount');
    const btnUserTop = $el('btn-user-topup');
    if (btnUserTop) {
        btnUserTop.addEventListener('click', () => {
            const amt = Number(userInput?.value || 0);
            if (!Number.isFinite(amt) || amt <= 0) return;
            if (window.__TAURI__) {
                invoke('topup_user', { amount: amt }).then(updateWalletUI).catch(alert);
            } else {
                const wallet = loadWallet();
                wallet.user_deposit = (wallet.user_deposit || 0) + amt;
                saveWallet(wallet);
                updateWalletUI();
            }
        });
    }

    const btnDeposit = $el('btn-deposit-virtual');
    if (btnDeposit) {
        btnDeposit.addEventListener('click', () => {
            if (window.__TAURI__) {
                invoke('topup_user', { amount: 10000 }).then(updateWalletUI).catch(alert);
            } else {
                const wallet = loadWallet();
                wallet.user_deposit = (wallet.user_deposit || 0) + 10000;
                saveWallet(wallet);
                updateWalletUI();
            }
        });
    }

    const btnResetLogs = $el('btn-reset-logs');
    if (btnResetLogs) {
        btnResetLogs.addEventListener('click', () => {
            storageSet('titan_logs_v1', []);
            showCyberToast('Logs resetate', 'info');
        });
    }
}

function wireBotControls() {
    const tradeInput = $el('bot-trade-size');
    if (tradeInput) {
        tradeInput.addEventListener('change', () => {
            const v = Number(tradeInput.value || 0);
            if (Number.isFinite(v) && v > 0) invoke('update_settings', { field: 'bot_trade_size', value: String(v) }).catch(console.error);
        });
    }

    const breakerInput = $el('circuit-breaker');
    const breakerDesc = $el('circuit-breaker-desc');
    if (breakerInput) {
        breakerInput.addEventListener('change', () => {
            const v = Number(breakerInput.value || 0);
            if (Number.isFinite(v) && v > 0) {
                invoke('update_settings', { field: 'circuit_breaker_limit', value: String(v) }).catch(console.error);
                if (breakerDesc) breakerDesc.innerText = `Stop la -$${Number(v).toLocaleString()} pierdere zilnică.`;
            }
        });
    }
}

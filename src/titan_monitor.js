// formatMoney -> titan_core.js (alias of fmtUsd)

let monitorLoopStarted = false;

function updateNyTime() {
    const el = $el('ny-time');
    if (!el) return;
    try {
        const now = new Date();
        const ny = new Intl.DateTimeFormat('en-US', {
            timeZone: 'America/New_York',
            hour: '2-digit',
            minute: '2-digit',
            hour12: false
        }).format(now);
        el.innerText = `NY TIME: ${ny} NY`;

        const badge = $el('market-badge');
        if (badge) {
            const nyParts = new Intl.DateTimeFormat('en-US', {
                timeZone: 'America/New_York',
                hour: 'numeric', minute: 'numeric', hour12: false, weekday: 'short'
            }).formatToParts(now);
            let h = 0, m = 0, wd = '';
            nyParts.forEach(p => {
                if (p.type === 'hour') h = parseInt(p.value);
                if (p.type === 'minute') m = parseInt(p.value);
                if (p.type === 'weekday') wd = p.value;
            });
            const mins = h * 60 + m;
            const isWeekday = !['Sat', 'Sun'].includes(wd);
            // Market sessions (ET): Pre-Market 4:00-9:30, Open 9:30-16:00, After-Hours 16:00-20:00, Overnight 20:00-4:00
            let mktState, mktClass;
            if (!isWeekday) {
                mktState = 'CLOSED'; mktClass = 'market-closed';
            } else if (mins >= 570 && mins < 960) {
                mktState = 'OPEN'; mktClass = 'market-open';
            } else if (mins >= 240 && mins < 570) {
                mktState = 'PRE-MARKET'; mktClass = 'market-premarket';
            } else if (mins >= 960 && mins < 1200) {
                mktState = 'AFTER-HOURS'; mktClass = 'market-afterhours';
            } else {
                mktState = 'OVERNIGHT'; mktClass = 'market-closed';
            }
            badge.textContent = mktState;
            badge.classList.remove('market-open', 'market-closed', 'market-premarket', 'market-afterhours');
            badge.classList.add(mktClass);
        }
    } catch (e) {
        const d = new Date();
        const hh = String(d.getHours()).padStart(2, '0');
        const mm = String(d.getMinutes()).padStart(2, '0');
        el.innerText = `NY TIME: ${hh}:${mm} NY`;
    }
}

function startMonitorLoop() {
    if (monitorLoopStarted) return;
    monitorLoopStarted = true;

    updateNyTime();
    TitanTimers.register('monitor:ny-time', updateNyTime, 1000);
    updateMonitor();
    TitanTimers.register('monitor:snapshot', updateMonitor, 1500);
    updateAccount();
    TitanTimers.register('monitor:account', updateAccount, 3000);
    updateStrategyScore();
    TitanTimers.register('monitor:strategy', updateStrategyScore, 4000);
    updateCandleTimer();
    TitanTimers.register('monitor:candle-timer', updateCandleTimer, 1000);
}

function applyHardwareUi(hw) {
    const fmtTemp = (v) => (v && v > 0 ? `${v.toFixed(0)}°C` : "--°C");
    const fmtHwPct = (v) => (Number.isFinite(v) ? `${v.toFixed(0)}%` : "--%");
    const fmtMb = (v) => (v && v > 0 ? `${v.toFixed(0)} MB` : "-- MB");
    const fmtW = (v) => (v && v > 0 ? `${v.toFixed(0)} W` : "-- W");
    const fmtRate = (mbps) => {
        if (!Number.isFinite(mbps)) return "0.00 MB/s";
        if (mbps >= 1) return `${mbps.toFixed(2)} MB/s`;
        return `${(mbps * 1024).toFixed(0)} KiB/s`;
    };
    const fmtTotal = (gib) => {
        if (!Number.isFinite(gib)) return "0.00 GiB";
        if (gib >= 1) return `${gib.toFixed(2)} GiB`;
        return `${(gib * 1024).toFixed(1)} MiB`;
    };

    setTxt('gpu-name', hw.gpu_name || "GPU");
    setTxt('gpu-temp', fmtTemp(hw.gpu_temp));
    setTxt('gpu-util', fmtHwPct(hw.gpu_util));
    setTxt('gpu-mem', fmtMb(hw.gpu_mem));
    setTxt('gpu-power', fmtW(hw.gpu_power));
    const fanGpu = (Number.isFinite(hw.gpu_fan) && hw.gpu_fan > 0) ? `${hw.gpu_fan.toFixed(0)} RPM` : "N/A";
    setTxt('gpu-fan', fanGpu);

    setTxt('cpu-usage', fmtHwPct(hw.cpu_usage));
    setTxt('cpu-temp', fmtTemp(hw.cpu_temp));
    setTxt('cpu-freq', hw.cpu_freq_mhz && hw.cpu_freq_mhz > 0 ? `${hw.cpu_freq_mhz.toFixed(0)} MHz` : "-- MHz");
    const fanCpu = (Number.isFinite(hw.cpu_fan) && hw.cpu_fan > 0) ? `${hw.cpu_fan.toFixed(0)} RPM` : "N/A";
    setTxt('cpu-fan', fanCpu);
    if (hw.ram_total && hw.ram_total > 0) {
        setTxt('cpu-ram', `${hw.ram_used.toFixed(1)}/${hw.ram_total.toFixed(1)} GiB`);
    } else {
        setTxt('cpu-ram', "--");
    }

    setTxt('net-iface', hw.net_iface ? `(${hw.net_iface})` : "(iface)");
    setTxt('net-down', fmtRate(hw.net_down));
    setTxt('net-up', fmtRate(hw.net_up));
    setTxt('net-total', `↓ ${fmtTotal(hw.net_tot_down)} | ↑ ${fmtTotal(hw.net_tot_up)}`);
}

function applyFlowUi(bot) {
    renderFlow(bot);
}

function applyAiTargetUi(bot) {
    const aiTargetEl = $el('ai-target');
    const aiDeltaEl = $el('ai-target-delta');
    if (aiTargetEl && bot.sentiment !== undefined) {
        aiTargetEl.innerText = bot.sentiment.toFixed(2);

        if (aiDeltaEl) {
            const pct = (bot.sentiment * 100).toFixed(2);
            const arrow = bot.sentiment >= 0 ? '↑' : '↓';
            aiDeltaEl.innerText = `${arrow} ${Math.abs(pct)}%`;
            aiDeltaEl.classList.remove('sub-positive', 'sub-negative', 'sub-neutral');
            if (bot.sentiment > 0) aiDeltaEl.classList.add('sub-positive');
            else if (bot.sentiment < 0) aiDeltaEl.classList.add('sub-negative');
            else aiDeltaEl.classList.add('sub-neutral');
        }
    }

    const symEl = $el('analysis-symbol');
    if (symEl && bot.symbol) symEl.innerText = bot.symbol;
    const tfEl = $el('analysis-timeframe');
    if (tfEl && currentTF) tfEl.innerText = currentTF;

    if (typeof bot.sentiment === "number") {
        const currentPrice = (window.lastBar && window.lastBar.close) ? window.lastBar.close : 0;
        const aiTarget = $el('ai-target');
        const aiDelta = $el('ai-target-delta');
        const targetPct = bot.sentiment * 1.5;
        const targetPrice = (currentPrice > 0) ? currentPrice * (1 + targetPct / 100) : 0;

        if (aiTarget) {
            if (targetPrice > 0) {
                aiTarget.innerText = formatMoney(targetPrice);
                const delta = targetPct;
                if (aiDelta) {
                    const arrow = delta >= 0 ? '↑' : '↓';
                    aiDelta.innerText = `${arrow} ${Math.abs(delta).toFixed(2)}%`;
                    aiDelta.classList.remove('sub-positive', 'sub-negative', 'sub-neutral');
                    if (delta > 0) aiDelta.classList.add('sub-positive');
                    else if (delta < 0) aiDelta.classList.add('sub-negative');
                    else aiDelta.classList.add('sub-neutral');
                }
            } else {
                aiTarget.innerText = bot.sentiment.toFixed(2);
            }
        }

        if (candleSeries && targetPrice > 0) {
            const toggleEl = $el('toggle-ai-target');
            const showAiTarget = toggleEl ? toggleEl.checked : true;
            if (window.aiTargetLine) {
                try { candleSeries.removePriceLine(window.aiTargetLine); } catch (_) { }
                window.aiTargetLine = null;
            }
            if (showAiTarget) {
                window.aiTargetLine = candleSeries.createPriceLine({
                    price: targetPrice,
                    color: pnlColor(targetPct),
                    lineWidth: 2,
                    lineStyle: 0,
                    axisLabelVisible: true,
                    title: 'AI TARGET',
                });
            }
        }
    }
}

function applyRedisStatus(snap) {
    const tauriStatus = $el('tauri-status');
    if (tauriStatus) {
        const redisText = snap.redis_online ? "<span style='color:#00BFFF'>+ REDIS</span>" : "<span style='color:red'>- REDIS</span>";
        tauriStatus.innerHTML = `TAURI LINKED ${redisText}`;
    }
    const statusElMon = $el('connection-status');
    const currentStatusText = statusElMon ? statusElMon.innerText : '';
    if (!currentStatusText.includes('LIVE FEED')) {
        if (snap.redis_online) {
            updateStatus("🟢 REDIS ONLINE", "#00BFFF", "success");
        } else {
            updateStatus("🔴 REDIS OFFLINE", "#FF2B2B", "error");
        }
    }
}

async function updateMonitor() {
    try {
        const snap = await invoke("get_monitor_snapshot", {});
        if (!snap) return;
        const hw = snap.hardware || {};
        const bot = snap.bot || {};
        window.lastBotStatus = bot;
        applyHardwareUi(hw);
        applyFlowUi(bot);
        applyAiTargetUi(bot);
        applyRedisStatus(snap);
    } catch (e) {
        updateStatus("❌ MONITOR ERROR", "red", "error");
    }
}

        (function () {
            let perfEquityChart = null, perfDailyChart = null;
            let perfEquitySeries = null, perfDailySeries = null;

            function createMiniChart(containerId, type) {
                const el = $el(containerId);
                if (!el || !window.LightweightCharts) return null;
                const chart = LightweightCharts.createChart(el, {
                    width: el.clientWidth, height: 200,
                    layout: { background: { color: 'transparent' }, textColor: '#888', fontSize: 9 },
                    grid: { vertLines: { visible: false }, horzLines: { color: '#1a1a2e' } },
                    timeScale: { borderVisible: false }, rightPriceScale: { borderVisible: false },
                    crosshair: { mode: 0 },
                });
                return chart;
            }

            async function loadPerformance() {
                try {
                    const [stats, curve, daily] = await Promise.all([
                        invoke('get_journal_stats'),
                        invoke('get_journal_equity_curve'),
                        invoke('get_journal_daily_pnl'),
                    ]);

                    const pnlEl = $el('perf-pnl');
                    pnlEl.textContent = '$' + stats.total_pnl.toFixed(2);
                    pnlEl.style.color = pnlColor(stats.total_pnl);

                    const wrEl = $el('perf-wr');
                    wrEl.textContent = stats.total_trades > 0 ? stats.win_rate.toFixed(1) + '%' : '--';
                    wrEl.style.color = pnlColor(stats.win_rate - 50);

                    $el('perf-pf').textContent = stats.profit_factor > 0 ? stats.profit_factor.toFixed(2) : '--';

                    // Sharpe estimate from trade PnLs
                    const avgPnl = stats.avg_pnl;
                    const avgW = stats.avg_winner || 0;
                    const avgL = Math.abs(stats.avg_loser) || 1;
                    const sharpeLike = avgW > 0 ? (avgPnl / avgL).toFixed(2) : '--';
                    $el('perf-sharpe').textContent = sharpeLike;

                    // Max drawdown from equity curve
                    let maxDD = 0, peak = 0;
                    for (const [_, v] of curve) {
                        if (v > peak) peak = v;
                        const dd = peak - v;
                        if (dd > maxDD) maxDD = dd;
                    }
                    const ddEl = $el('perf-dd');
                    ddEl.textContent = maxDD > 0 ? '-$' + maxDD.toFixed(2) : '--';

                    const durSecs = stats.avg_duration_secs || 0;
                    $el('perf-dur').textContent = durSecs >= 3600
                        ? (durSecs / 3600).toFixed(1) + 'h'
                        : durSecs >= 60 ? Math.round(durSecs / 60) + 'm' : durSecs + 's';

                    // Equity curve chart
                    if (perfEquityChart) { perfEquityChart.remove(); }
                    perfEquityChart = createMiniChart('perf-equity-chart');
                    if (perfEquityChart && curve.length > 0) {
                        perfEquitySeries = perfEquityChart.addBaselineSeries({
                            baseValue: { type: 'price', price: 0 },
                            topLineColor: '#00BFFF', topFillColor1: 'rgba(0,191,255,0.2)', topFillColor2: 'rgba(0,191,255,0.02)',
                            bottomLineColor: '#ff4444', bottomFillColor1: 'rgba(255,68,68,0.02)', bottomFillColor2: 'rgba(255,68,68,0.2)',
                            lineWidth: 2,
                        });
                        const baseTime = Math.floor(Date.now() / 1000) - curve.length * 60;
                        perfEquitySeries.setData(curve.map((c, i) => ({ time: baseTime + i * 60, value: c[1] })));
                        perfEquityChart.timeScale().fitContent();
                    }

                    // Daily PnL chart
                    if (perfDailyChart) { perfDailyChart.remove(); }
                    perfDailyChart = createMiniChart('perf-daily-chart');
                    if (perfDailyChart && daily.length > 0) {
                        perfDailySeries = perfDailyChart.addHistogramSeries({
                            priceFormat: { type: 'price', precision: 2, minMove: 0.01 },
                        });
                        perfDailySeries.setData(daily.map((d, i) => ({
                            time: d[0],
                            value: d[1],
                            color: pnlColor(d[1]),
                        })));
                        perfDailyChart.timeScale().fitContent();
                    }
                } catch (e) { TitanError.report('perf:load', e); }
            }

            function wirePerformance() {
                document.querySelectorAll('.perf-range').forEach(btn => {
                    btn.addEventListener('click', () => {
                        document.querySelectorAll('.perf-range').forEach(b => b.classList.remove('active'));
                        btn.classList.add('active');
                        loadPerformance();
                    });
                });
            }

            let _perfLoaded = false;
            TitanBus.on('tabSwitch', function (index) {
                if (index === 5 && !_perfLoaded) {
                    _perfLoaded = true;
                    loadPerformance();
                }
            });

            document.addEventListener('DOMContentLoaded', wirePerformance);
        })();

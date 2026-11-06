        (function () {
            let newsTimelineChart = null;
            let newsTimelineSeries = null;

            function updateGauge(score, label) {
                const el = $el('news-gauge-score');
                const lbl = $el('news-gauge-label');
                const bar = $el('news-gauge-bar');
                if (!el) return;
                el.textContent = score.toFixed(2);
                lbl.textContent = label;
                const pct = ((score + 1) / 2 * 100).toFixed(0);
                bar.style.width = pct + '%';
                if (score >= 0.15) {
                    el.style.color = '#00BFFF'; lbl.style.color = '#00BFFF'; bar.style.background = '#00BFFF';
                } else if (score > -0.15) {
                    el.style.color = '#ffaa00'; lbl.style.color = '#ffaa00'; bar.style.background = '#ffaa00';
                } else {
                    el.style.color = '#ff4444'; lbl.style.color = '#ff4444'; bar.style.background = '#ff4444';
                }
            }

            function renderArticles(articles) {
                const container = $el('news-articles');
                const empty = $el('news-empty');
                if (!container) return;
                if (!articles || articles.length === 0) {
                    container.innerHTML = '';
                    if (empty) empty.style.display = 'block';
                    return;
                }
                if (empty) empty.style.display = 'none';
                container.innerHTML = articles.map(a => {
                    let badgeColor = '#888', badgeBg = '#333';
                    if (a.sentiment_score >= 0.15) { badgeColor = '#00BFFF'; badgeBg = '#0a2a0a'; }
                    else if (a.sentiment_score <= -0.15) { badgeColor = '#ff4444'; badgeBg = '#2a0a0a'; }
                    const ts = a.timestamp ? a.timestamp.replace('T', ' ').substring(0, 16) : '';
                    const safeUrl = a.url ? a.url.replace(/'/g, "\\'").replace(/"/g, '&quot;') : '';
                    const headlineStyle = a.url
                        ? 'font-size:13px; font-weight:600; color:#00BFFF; line-height:1.3; cursor:pointer; text-decoration:underline;'
                        : 'font-size:13px; font-weight:600; color:#eee; line-height:1.3;';
                    const headlineClick = a.url ? `onclick="window.__TAURI__?.shell?.open('${safeUrl}')"` : '';
                    const readLink = a.url ? `<span style="font-size:9px; color:#00BFFF; cursor:pointer; margin-left:6px; text-decoration:underline;" onclick="window.__TAURI__?.shell?.open('${safeUrl}')">READ ↗</span>` : '';
                    let sevBadgeHtml = '';
                    if (Math.abs(a.sentiment_score) >= 0.50) {
                        const sevLabel = a.sentiment_score >= 0.85 ? 'EXPLOSIVE' : a.sentiment_score >= 0.50 ? 'STRONG' : a.sentiment_score <= -0.85 ? 'EXPLOSIVE' : 'STRONG';
                        const sevEmoji = a.sentiment_score < 0 ? '🔻' : '🔥';
                        const sevC = a.sentiment_score < 0 ? '#ff4444' : '#00BFFF';
                        sevBadgeHtml = `<span style="display:inline-block;margin-top:3px;font-size:8px;font-weight:700;color:${sevC};border:1px solid ${sevC};padding:1px 4px;border-radius:2px;">${sevEmoji} ${sevLabel}</span>`;
                    }
                    return `<div style="background:#1a1a2e; border:1px solid #333; border-radius:6px; padding:12px; transition:border-color 0.2s;" onmouseover="this.style.borderColor='#555'" onmouseout="this.style.borderColor='#333'">
                        <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:10px;">
                            <div style="flex:1;">
                                <div style="${headlineStyle}" ${headlineClick}>${escapeHtml(a.headline || '')}${readLink}</div>
                                <div style="font-size:11px; color:#777; margin-top:4px;">${a.summary ? escapeHtml(a.summary.substring(0, 150)) + (a.summary.length > 150 ? '...' : '') : ''}</div>
                            </div>
                            <div style="min-width:90px; text-align:right;">
                                <span style="display:inline-block; padding:2px 8px; border-radius:3px; font-size:10px; font-weight:700; color:${badgeColor}; background:${badgeBg};">${escapeHtml(a.sentiment_label || '')}</span>
                                <div style="font-size:18px; font-weight:700; color:${badgeColor}; margin-top:2px;">${a.sentiment_score.toFixed(2)}</div>
                                ${sevBadgeHtml}
                            </div>
                        </div>
                        <div style="display:flex; justify-content:space-between; margin-top:6px; font-size:10px; color:#555;">
                            <span>${escapeHtml(a.source || '')}</span><span>${ts}</span>
                        </div>
                    </div>`;
                }).join('');
            }

            function initTimeline(history) {
                const container = $el('news-timeline');
                if (!container || !window.LightweightCharts) return;
                if (newsTimelineChart) { newsTimelineChart.remove(); newsTimelineChart = null; }
                newsTimelineChart = LightweightCharts.createChart(container, {
                    width: container.clientWidth, height: 100,
                    layout: { background: { color: 'transparent' }, textColor: '#888', fontSize: 9 },
                    grid: { vertLines: { visible: false }, horzLines: { color: '#222' } },
                    timeScale: { visible: false }, rightPriceScale: { visible: true, borderVisible: false },
                    crosshair: { mode: 0 },
                });
                newsTimelineSeries = newsTimelineChart.addBaselineSeries({
                    baseValue: { type: 'price', price: 0 },
                    topLineColor: '#00BFFF', topFillColor1: 'rgba(0,191,255,0.15)', topFillColor2: 'rgba(0,191,255,0.02)',
                    bottomLineColor: '#ff4444', bottomFillColor1: 'rgba(255,68,68,0.02)', bottomFillColor2: 'rgba(255,68,68,0.15)',
                    lineWidth: 2,
                });
                if (history && history.length > 0) {
                    const base = Math.floor(Date.now() / 1000) - history.length * 300;
                    const data = history.map((h, i) => ({ time: base + i * 300, value: h[1] }));
                    newsTimelineSeries.setData(data);
                }
            }

            async function loadNews() {
                try {
                    const data = await invoke('get_news_feed');
                    updateGauge(data.overall_score, data.overall_label);
                    renderArticles(data.articles);
                    initTimeline(data.history);
                } catch (e) { TitanError.report('news:load', e); }
            }

            function wireNews() {
                const btn = $el('btn-news-refresh');
                if (btn) btn.addEventListener('click', async () => {
                    btn.textContent = '...';
                    try { await invoke('refresh_news'); await loadNews(); } catch (e) { TitanError.report('news:refresh', e); }
                    btn.textContent = 'REFRESH';
                });
            }

            let _newsLoaded = false;
            TitanBus.on('tabSwitch', function (index) {
                if (index === 1 && !_newsLoaded) {
                    _newsLoaded = true;
                    loadNews();
                }
            });

            document.addEventListener('DOMContentLoaded', wireNews);

            if (window.__TAURI__) {
                window.__TAURI__.event.listen('sentiment-update', (event) => {
                    const d = event.payload;
                    if (d && typeof d.score === 'number') {
                        updateGauge(d.score, d.label || 'NEUTRAL');
                    }
                });
            }
        })();

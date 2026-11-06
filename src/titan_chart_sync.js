function syncTimeScale(source, targets) {
    let syncing = false;
    source.timeScale().subscribeVisibleLogicalRangeChange(range => {
        if (syncing || !range) return;
        syncing = true;
        targets.forEach(t => t.timeScale().setVisibleLogicalRange(range));
        syncing = false;
    });
}

function syncCrosshair(allCharts) {
    // DOM-based vertical crosshair sync across all chart panels
    // LWC v4.0.1 lacks setCrosshairPosition, so we use a CSS overlay
    const chartCol = document.querySelector('.chart-stack');
    if (!chartCol) return;

    // Create the sync line overlay
    let syncLine = $el('cross-sync-line');
    if (!syncLine) {
        syncLine = document.createElement('div');
        syncLine.id = 'cross-sync-line';
        syncLine.style.cssText = 'position:absolute;top:0;bottom:0;width:1px;background:rgba(0,191,255,0.35);pointer-events:none;z-index:9999;display:none;box-shadow:0 0 4px rgba(0,191,255,0.2);';
        chartCol.style.position = 'relative';
        chartCol.appendChild(syncLine);
    }

    // Track mouse across all chart containers
    const containers = [
        $el('chart-div'),
        $el('chart-vol'),
        $el('chart-rsi'),
        $el('chart-macd'),
        $el('chart-adx'),
        $el('chart-oracle'),
    ].filter(Boolean);

    containers.forEach(container => {
        container.addEventListener('mousemove', (e) => {
            const colRect = chartCol.getBoundingClientRect();
            const x = e.clientX - colRect.left;
            syncLine.style.left = x + 'px';
            syncLine.style.display = 'block';
        });
    });

    chartCol.addEventListener('mouseleave', () => {
        syncLine.style.display = 'none';
    });
}

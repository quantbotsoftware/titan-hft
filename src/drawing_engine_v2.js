// --- DRAWING ENGINE V2 (FULL AUDIT: RETINA, GLOW, UNDO, CONTEXT MENU, SNAP, TOOLTIPS) ---
// escapeHtml -> titan_core.js

let drawingManager = null;

const TWO_POINT_TOOLS = ['trend', 'ray', 'fib', 'measure', 'long', 'short'];
const SINGLE_POINT_TOOLS = ['hline', 'vline', 'text'];

const NEON_GREEN = '#00BFFF';
const NEON_CYAN = '#00FFFF';
const NEON_RED = '#FF2B2B';
const NEON_GOLD = '#FFD700';
const PROFIT_FILL = 'rgba(0,191,255,0.10)';
const LOSS_FILL = 'rgba(255,43,43,0.10)';
const PROFIT_STROKE = '#00BFFF';
const LOSS_STROKE = '#FF2B2B';

class DrawingManager {
    constructor(chart, series, canvasId) {
        this.chart = chart;
        this.series = series;
        window.DrawingEngine = this;
        window.drawingManager = this;

        const container = $el('chart-div');
        if (!container) return;

        container.style.position = 'relative';

        this.canvas = document.createElement('canvas');
        this.canvas.id = 'drawing-canvas-dynamic';
        this.canvas.style.position = 'absolute';
        this.canvas.style.top = '0';
        this.canvas.style.left = '0';
        this.canvas.style.width = '100%';
        this.canvas.style.height = '100%';
        this.canvas.style.zIndex = '100';
        this.canvas.style.pointerEvents = 'none';

        container.appendChild(this.canvas);

        this.ctx = this.canvas.getContext('2d');
        this.drawings = [];
        this.activeTool = null;
        this.currentDrawing = null;
        this._dirty = true;
        this._animFrame = null;
        this._pulsePhase = 0;

        this.selectedDrawing = null;
        this.interactionMode = null;
        this.activeHandle = null;
        this.dragStartPoint = null;
        this.initialState = null;

        this.undoStack = [];
        this.redoStack = [];
        this._hoveredDrawing = null;
        this._tooltipEl = null;
        this.snapToPrice = false;

        this._createTooltip(container);
        this._createContextMenu(container);

        this.resize();
        window.addEventListener('resize', () => this.resize());

        this.chart.timeScale().subscribeVisibleTimeRangeChange(() => this.markDirty());

        this.canvas.addEventListener('mousedown', (e) => this.onMouseDown(e));
        container.addEventListener('mousemove', (e) => this.onMouseMove(e));
        this.canvas.addEventListener('mouseup', (e) => this.onMouseUp(e));
        this.canvas.addEventListener('contextmenu', (e) => this._onContextMenu(e));

        window.addEventListener('keydown', (e) => {
            if ((e.key === 'Delete' || e.key === 'Backspace') && this.selectedDrawing) {
                this._pushUndo();
                this.drawings = this.drawings.filter(d => d !== this.selectedDrawing);
                this.selectedDrawing = null;
                if (typeof _lsShowExecToolbar === 'function') _lsShowExecToolbar(null);
                this.markDirty();
                if (window.persistDrawings) window.persistDrawings();
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault();
                this.undo();
            }
            if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
                e.preventDefault();
                this.redo();
            }
        });

        const betInput = $el('bet-input');
        if (betInput) {
            betInput.addEventListener('input', () => this.markDirty());
        }

        this._startAnimLoop();
    }

    _createTooltip(container) {
        this._tooltipEl = document.createElement('div');
        Object.assign(this._tooltipEl.style, {
            position: 'absolute', display: 'none', pointerEvents: 'none', zIndex: '1001',
            background: 'rgba(0,0,0,0.92)', border: '1px solid #00BFFF', borderRadius: '4px',
            padding: '6px 10px', fontFamily: "'Share Tech Mono', monospace", fontSize: '10px',
            color: '#00BFFF', whiteSpace: 'nowrap', boxShadow: '0 0 10px rgba(0,191,255,0.3)'
        });
        container.appendChild(this._tooltipEl);
    }

    _createContextMenu(container) {
        this._ctxMenu = document.createElement('div');
        this._ctxMenu.className = 'drawing-ctx-menu';
        Object.assign(this._ctxMenu.style, {
            position: 'absolute', display: 'none', zIndex: '1002',
            background: 'rgba(10,10,10,0.96)', border: '1px solid #00BFFF', borderRadius: '4px',
            padding: '4px 0', fontFamily: "'Share Tech Mono', monospace", fontSize: '11px',
            color: '#00BFFF', boxShadow: '0 0 15px rgba(0,191,255,0.25)', minWidth: '140px'
        });
        container.appendChild(this._ctxMenu);

        document.addEventListener('click', () => { this._ctxMenu.style.display = 'none'; });
    }

    _showContextMenu(x, y, drawing) {
        const items = [
            { label: 'Delete', action: () => { this._pushUndo(); this.drawings = this.drawings.filter(d => d !== drawing); this.selectedDrawing = null; this.markDirty(); }},
        ];
        if (drawing.type === 'long' || drawing.type === 'short') {
            items.unshift({ label: 'EXEC Manual', action: () => this.executeDrawing(drawing, false) });
            items.unshift({ label: 'EXEC AI', action: () => this.executeDrawing(drawing, true) });
        }
        items.push({ label: 'Duplicate', action: () => {
            this._pushUndo();
            const copy = JSON.parse(JSON.stringify(drawing));
            copy.p1.price *= 1.005;
            if (copy.p2) copy.p2.price *= 1.005;
            if (copy.stopPrice) copy.stopPrice *= 1.005;
            this.drawings.push(copy);
            this.markDirty();
        }});

        this._ctxMenu.innerHTML = '';
        items.forEach(item => {
            const el = document.createElement('div');
            el.textContent = item.label;
            Object.assign(el.style, {
                padding: '5px 14px', cursor: 'pointer', transition: 'background 0.15s'
            });
            el.addEventListener('mouseenter', () => { el.style.background = 'rgba(0,191,255,0.15)'; });
            el.addEventListener('mouseleave', () => { el.style.background = 'transparent'; });
            el.addEventListener('click', (e) => { e.stopPropagation(); item.action(); this._ctxMenu.style.display = 'none'; });
            this._ctxMenu.appendChild(el);
        });

        this._ctxMenu.style.left = x + 'px';
        this._ctxMenu.style.top = y + 'px';
        this._ctxMenu.style.display = 'block';
    }

    _onContextMenu(e) {
        e.preventDefault();
        const coords = this.getChartCoords(e);
        const hit = this.hitTest(coords.x, coords.y);
        if (hit) {
            this.selectedDrawing = hit.d;
            this.markDirty();
            this._showContextMenu(coords.x, coords.y, hit.d);
        }
    }

    _startAnimLoop() {
        let lastFrame = 0;
        const loop = (ts) => {
            if (this.selectedDrawing) {
                this._pulsePhase = (this._pulsePhase + 0.04) % (Math.PI * 2);
                this._renderInternal();
                this._dirty = false;
            } else if (this._dirty) {
                this._renderInternal();
                this._dirty = false;
            }
            this._animFrame = requestAnimationFrame(loop);
        };
        loop(0);
    }

    markDirty() { this._dirty = true; }

    _pushUndo() {
        this.undoStack.push(JSON.stringify(this.drawings.map(d => this._serializeOne(d))));
        if (this.undoStack.length > 50) this.undoStack.shift();
        this.redoStack = [];
    }

    undo() {
        if (this.undoStack.length === 0) return;
        this.redoStack.push(JSON.stringify(this.drawings.map(d => this._serializeOne(d))));
        const prev = JSON.parse(this.undoStack.pop());
        this.setDrawings(prev);
    }

    redo() {
        if (this.redoStack.length === 0) return;
        this.undoStack.push(JSON.stringify(this.drawings.map(d => this._serializeOne(d))));
        const next = JSON.parse(this.redoStack.pop());
        this.setDrawings(next);
    }

    resize() {
        const parent = this.canvas.parentElement;
        if (!parent) return;
        const dpr = window.devicePixelRatio || 1;
        const w = parent.clientWidth;
        const h = parent.clientHeight;
        this.canvas.width = w * dpr;
        this.canvas.height = h * dpr;
        this.canvas.style.width = w + 'px';
        this.canvas.style.height = h + 'px';
        this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        this._logicalWidth = w;
        this._logicalHeight = h;
        this.markDirty();
    }

    setTool(tool) {
        this.activeTool = tool;
        this.selectedDrawing = null;
        this.currentDrawing = null;
        this.markDirty();

        if (tool) {
            this.canvas.style.pointerEvents = 'all';
            this.canvas.style.zIndex = '999';
            this.canvas.style.cursor = 'crosshair';
        } else {
            this.canvas.style.pointerEvents = 'none';
            this.canvas.style.zIndex = '100';
            this.canvas.style.cursor = 'default';
        }
    }

    getChartCoords(e) {
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        let y = e.clientY - rect.top;
        let price = this.series.coordinateToPrice(y);
        const time = this.ensureTime(x);

        if (window.snapToOhlc) {
            const snapped = this.applySnap(price, y, time);
            price = snapped.price;
            y = snapped.y;
        } else if (this.snapToPrice && price != null) {
            price = this._snapPrice(price, time);
        }

        return { x, y, price, time };
    }

    _snapPrice(price, time) {
        try {
            const data = this.series.data ? this.series.data() : null;
            if (!data || data.length === 0) return price;
            let closest = null;
            let minDist = Infinity;
            for (const bar of data) {
                if (Math.abs(bar.time - time) < 120) {
                    for (const val of [bar.open, bar.high, bar.low, bar.close]) {
                        const dist = Math.abs(val - price);
                        if (dist < minDist) { minDist = dist; closest = val; }
                    }
                }
            }
            const threshold = Math.abs(price) * 0.003;
            if (closest !== null && minDist < threshold) return closest;
        } catch (_) {}
        return price;
    }

    projectTime(time) {
        const visibleRange = this.chart.timeScale().getVisibleRange();
        if (!visibleRange) return this.chart.timeScale().timeToCoordinate(time);

        const t1 = visibleRange.from;
        const t2 = visibleRange.to;
        const x1 = this.chart.timeScale().timeToCoordinate(t1);
        const x2 = this.chart.timeScale().timeToCoordinate(t2);

        let x = this.chart.timeScale().timeToCoordinate(time);

        if (x === null && x1 !== null && x2 !== null && t2 !== t1) {
            const rate = (x2 - x1) / (t2 - t1);
            x = x1 + (time - t1) * rate;
        }

        return x;
    }

    ensureTime(x) {
        const t = this.chart.timeScale().coordinateToTime(x);
        if (t !== null && t !== undefined) return t;

        const visibleRange = this.chart.timeScale().getVisibleRange();
        if (!visibleRange) return Math.floor(Date.now() / 1000);

        const t1 = visibleRange.from;
        const t2 = visibleRange.to;
        const x1 = this.chart.timeScale().timeToCoordinate(t1);
        const x2 = this.chart.timeScale().timeToCoordinate(t2);

        if (x1 !== null && x2 !== null && t2 !== t1) {
            const rate = (x2 - x1) / (t2 - t1);
            return Math.floor(t1 + (x - x1) / rate);
        }

        return Math.floor(Date.now() / 1000);
    }

    normalizeTime(t) {
        if (typeof t === 'number') return t;
        if (typeof t === 'string') {
            const parsed = Date.parse(t);
            if (!Number.isNaN(parsed)) return Math.floor(parsed / 1000);
        }
        if (t && typeof t === 'object' && 'year' in t) {
            const dt = new Date(Date.UTC(t.year, (t.month || 1) - 1, t.day || 1));
            return Math.floor(dt.getTime() / 1000);
        }
        return null;
    }

    getNearestCandle(time) {
        const data = window.lastCandleData || [];
        if (!data || data.length === 0) return null;
        const t = this.normalizeTime(time);
        if (t === null) return null;
        let best = data[0], bestDiff = Math.abs((best.time || 0) - t);
        for (let i = 1; i < data.length; i++) {
            const c = data[i];
            const diff = Math.abs((c.time || 0) - t);
            if (diff < bestDiff) { bestDiff = diff; best = c; }
        }
        return best;
    }

    applySnap(price, y, time) {
        if (!window.snapToOhlc || price === null || price === undefined) return { price, y };
        const candle = this.getNearestCandle(time);
        if (!candle) return { price, y };
        const candidates = [candle.open, candle.high, candle.low, candle.close].filter(v => typeof v === 'number' && Number.isFinite(v));
        if (candidates.length === 0) return { price, y };
        let bestPrice = price, bestY = y, bestPx = Infinity;
        candidates.forEach(p => {
            const cy = this.series.priceToCoordinate(p);
            if (cy === null || cy === undefined) return;
            const d = Math.abs(cy - y);
            if (d < bestPx) { bestPx = d; bestPrice = p; bestY = cy; }
        });
        const tol = window.snapTolerancePx || 8;
        if (bestPx <= tol) return { price: bestPrice, y: bestY };
        return { price, y };
    }

    formatTimeLabel(time) {
        const t = this.normalizeTime(time);
        if (!t) return "--";
        const d = new Date(t * 1000);
        if (Number.isNaN(d.getTime())) return "--";
        return d.toISOString().replace('T', ' ').slice(0, 16);
    }

    hitTest(x, y) {
        const dpr = window.devicePixelRatio || 1;
        const HR = Math.max(12, 12 / dpr * 1.5);
        const priceScaleWidth = 60;
        const chartWidth = this._logicalWidth - priceScaleWidth;

        for (let i = this.drawings.length - 1; i >= 0; i--) {
            const d = this.drawings[i];

            if (d.type === 'hline') {
                const y1 = this.series.priceToCoordinate(d.p1.price);
                if (Math.abs(y - y1) < HR && x >= 0 && x <= chartWidth) return { d, type: 'body' };
                if (Math.abs(y - y1) < HR) return { d, type: 'handle', name: 'p1' };
                continue;
            }
            if (d.type === 'vline') {
                const x1 = this.projectTime(d.p1.time);
                if (x1 === null) continue;
                if (Math.abs(x - x1) < HR && y >= 0 && y <= this._logicalHeight) return { d, type: 'body' };
                if (Math.abs(x - x1) < HR) return { d, type: 'handle', name: 'p1' };
                continue;
            }
            if (d.type === 'text') {
                const x1 = this.projectTime(d.p1.time);
                const y1 = this.series.priceToCoordinate(d.p1.price);
                if (x1 === null) continue;
                const txt = d.text || 'Text';
                this.ctx.font = '12px monospace';
                const m = this.ctx.measureText(txt);
                const tw = m.width + 8;
                const th = 18;
                const tx = x1 - 4;
                const ty = y1 - th + 4;
                if (x >= tx && x <= tx + tw && y >= ty && y <= ty + th) return { d, type: 'body' };
                if (Math.abs(x - x1) < HR && Math.abs(y - y1) < HR) return { d, type: 'handle', name: 'p1' };
                continue;
            }

            let x1 = this.projectTime(d.p1.time);
            let x2 = this.projectTime(d.p2.time);

            const y1 = this.series.priceToCoordinate(d.p1.price);
            const y2 = this.series.priceToCoordinate(d.p2.price);

            if (d.stopPrice === undefined && d.p1 && d.p2 && (d.type === 'long' || d.type === 'short')) {
                const _rr = (window._lsSettings && window._lsSettings.defaultRR > 0) ? window._lsSettings.defaultRR : 1.0;
                const _diff = d.p2.price - d.p1.price;
                d.stopPrice = d.p1.price - (_diff / _rr);
            }
            const yStop = d.stopPrice !== undefined ? this.series.priceToCoordinate(d.stopPrice) : y1;

            if (x1 === null) x1 = -9999;
            if (x2 === null) x2 = -9999;

            if (d.type === 'trend' || d.type === 'ray' || d.type === 'measure') {
                if (Math.abs(x - x2) < HR && Math.abs(y - y2) < HR) return { d, type: 'handle', name: 'p2' };
                if (Math.abs(x - x1) < HR && Math.abs(y - y1) < HR) return { d, type: 'handle', name: 'p1' };
                const dist = this._distToSegment(x, y, x1, y1, x2, y2);
                if (dist < HR) return { d, type: 'body' };
                continue;
            }

            if (d.type === 'fib') {
                if (Math.abs(x - x2) < HR && Math.abs(y - y2) < HR) return { d, type: 'handle', name: 'p2' };
                if (Math.abs(x - x1) < HR && Math.abs(y - y1) < HR) return { d, type: 'handle', name: 'p1' };
                const bx = Math.min(x1, x2);
                const bw = Math.abs(x2 - x1);
                const by = Math.min(y1, y2);
                const bh = Math.abs(y2 - y1);
                if (x >= bx && x <= bx + bw && y >= by && y <= by + bh) return { d, type: 'body' };
                continue;
            }

            const bx = Math.min(x1, x2);
            const bw = Math.abs(x2 - x1);
            const by = Math.min(y1, y2);
            const bh = Math.abs(y2 - y1);

            if (Math.abs(x - x2) < HR && Math.abs(y - y2) < HR) return { d, type: 'handle', name: 'p2' };
            if (Math.abs(x - x1) < HR && Math.abs(y - y1) < HR) return { d, type: 'handle', name: 'p1' };

            const stopX = (x1 + x2) / 2;
            if (Math.abs(x - stopX) < HR && Math.abs(y - yStop) < HR) return { d, type: 'handle', name: 'stop' };

            if (x >= bx && x <= bx + bw && y >= by && y <= by + bh) return { d, type: 'body' };

            const sy = Math.min(y1, yStop);
            const sh = Math.abs(yStop - y1);
            if (x >= bx && x <= bx + bw && y >= sy && y <= sy + sh) return { d, type: 'body' };
        }
        return null;
    }

    _distToSegment(px, py, x1, y1, x2, y2) {
        const A = px - x1;
        const B = py - y1;
        const C = x2 - x1;
        const D = y2 - y1;
        const dot = A * C + B * D;
        const lenSq = C * C + D * D;
        let param = lenSq !== 0 ? dot / lenSq : -1;
        param = Math.max(0, Math.min(1, param));
        const xx = x1 + param * C;
        const yy = y1 + param * D;
        return Math.hypot(px - xx, py - yy);
    }

    onMouseDown(e) {
        if (e.button === 2) return;
        const coords = this.getChartCoords(e);

        if (this.activeTool) {
            if (!this.currentDrawing) {
                this._pushUndo();
                this.currentDrawing = {
                    type: this.activeTool,
                    p1: { price: coords.price, time: coords.time },
                    p2: { price: coords.price, time: coords.time },
                };
                if (this.activeTool === 'text') {
                    const txt = prompt('Enter text:', '');
                    if (txt !== null) {
                        this.currentDrawing.text = txt;
                    }
                }
            }
            return;
        }

        const hit = this.hitTest(coords.x, coords.y);
        if (hit) {
            this.selectedDrawing = hit.d;
            this.interactionMode = (hit.type === 'handle') ? 'resize' : 'move';
            this.activeHandle = hit.name;
            this.dragStartPoint = { x: coords.x, y: coords.y };
            this._pushUndo();

            this.initialState = {
                p1: { ...hit.d.p1 },
                p2: hit.d.p2 ? { ...hit.d.p2 } : null,
                stopPrice: hit.d.stopPrice
            };

            this.markDirty();
        } else {
            this.selectedDrawing = null;
            this.markDirty();
        }
    }

    onMouseMove(e) {
        const coords = this.getChartCoords(e);

        if (this.currentDrawing) {
            this.canvas.style.pointerEvents = 'all';
            if (SINGLE_POINT_TOOLS.includes(this.currentDrawing.type)) {
                this.currentDrawing.p2 = { ...this.currentDrawing.p1 };
            } else {
                this.currentDrawing.p2 = { price: coords.price, time: coords.time };
            }
            this.markDirty();
            return;
        }

        if (this.selectedDrawing && this.interactionMode) {
            this.canvas.style.pointerEvents = 'all';
            const d = this.selectedDrawing;
            if (this.interactionMode === 'resize') {
                if (this.activeHandle === 'p1') {
                    if (d.type !== 'hline' && d.type !== 'vline') {
                        if (coords.time) d.p1.time = coords.time;
                    }
                    if (d.type !== 'vline') d.p1.price = coords.price;
                    if (d.type === 'vline') d.p1.time = coords.time;
                }
                else if (this.activeHandle === 'p2') {
                    if (coords.time) d.p2.time = coords.time;
                    d.p2.price = coords.price;
                }
                else if (this.activeHandle === 'stop') {
                    d.stopPrice = coords.price;
                }
                this.canvas.style.cursor = 'crosshair';
                this.markDirty();
            }
            else if (this.interactionMode === 'move') {
                const dy = coords.y - this.dragStartPoint.y;
                const dx = coords.x - this.dragStartPoint.x;

                if (d.type === 'hline') {
                    const initialY1 = this.series.priceToCoordinate(this.initialState.p1.price);
                    d.p1.price = this.series.coordinateToPrice(initialY1 + dy);
                } else if (d.type === 'vline') {
                    const visibleRange = this.chart.timeScale().getVisibleRange();
                    if (visibleRange) {
                        const t1 = visibleRange.from;
                        const t2 = visibleRange.to;
                        const x1c = this.chart.timeScale().timeToCoordinate(t1);
                        const x2c = this.chart.timeScale().timeToCoordinate(t2);
                        if (x2c !== x1c && x1c !== null && x2c !== null) {
                            const rate = (t2 - t1) / (x2c - x1c);
                            const initialX = this.projectTime(this.initialState.p1.time);
                            d.p1.time = Math.floor(t1 + (initialX + dx - x1c) * rate);
                        }
                    }
                } else if (d.type === 'text') {
                    const initialY1 = this.series.priceToCoordinate(this.initialState.p1.price);
                    d.p1.price = this.series.coordinateToPrice(initialY1 + dy);
                    const visibleRange = this.chart.timeScale().getVisibleRange();
                    if (visibleRange) {
                        const t1 = visibleRange.from;
                        const t2 = visibleRange.to;
                        const x1c = this.chart.timeScale().timeToCoordinate(t1);
                        const x2c = this.chart.timeScale().timeToCoordinate(t2);
                        if (x2c !== x1c && x1c !== null && x2c !== null) {
                            const rate = (t2 - t1) / (x2c - x1c);
                            const initialX = this.projectTime(this.initialState.p1.time);
                            d.p1.time = Math.floor(t1 + (initialX + dx - x1c) * rate);
                        }
                    }
                } else {
                    const initialY1 = this.series.priceToCoordinate(this.initialState.p1.price);
                    d.p1.price = this.series.coordinateToPrice(initialY1 + dy);

                    const initialY2 = this.series.priceToCoordinate(this.initialState.p2.price);
                    d.p2.price = this.series.coordinateToPrice(initialY2 + dy);

                    if (this.initialState.stopPrice !== undefined) {
                        const initialYStop = this.series.priceToCoordinate(this.initialState.stopPrice);
                        d.stopPrice = this.series.coordinateToPrice(initialYStop + dy);
                    }

                    const visibleRange = this.chart.timeScale().getVisibleRange();
                    if (visibleRange) {
                        const t1 = visibleRange.from;
                        const t2 = visibleRange.to;
                        const x1c = this.chart.timeScale().timeToCoordinate(t1);
                        const x2c = this.chart.timeScale().timeToCoordinate(t2);
                        if (x2c !== x1c && x1c !== null && x2c !== null) {
                            const rate = (t2 - t1) / (x2c - x1c);
                            const initialX1 = this.projectTime(this.initialState.p1.time);
                            const initialX2 = this.projectTime(this.initialState.p2.time);
                            d.p1.time = Math.floor(t1 + (initialX1 + dx - x1c) * rate);
                            d.p2.time = Math.floor(t1 + (initialX2 + dx - x1c) * rate);
                        }
                    }
                }

                this.canvas.style.cursor = 'move';
                this.markDirty();
            }
            this._hideTooltip();
            return;
        }

        // Hover logic + tooltip
        if (this.activeTool) {
            this.canvas.style.pointerEvents = 'all';
            this.canvas.style.cursor = 'crosshair';
            this._hideTooltip();
        } else {
            const hit = this.hitTest(coords.x, coords.y);
            if (hit) {
                this.canvas.style.pointerEvents = 'all';
                this.canvas.style.cursor = (hit.type === 'handle') ? 'crosshair' : 'move';
                this._hoveredDrawing = hit.d;
                this._showTooltip(coords.x, coords.y, hit.d);
            } else {
                this.canvas.style.pointerEvents = 'none';
                this.canvas.style.cursor = 'default';
                this._hoveredDrawing = null;
                this._hideTooltip();
            }
        }
    }

    _showTooltip(mx, my, d) {
        if (!this._tooltipEl) return;
        let html = `<b style="color:#FFD700">${escapeHtml(d.type.toUpperCase())}</b>`;
        if (d.type === 'long' || d.type === 'short') {
            const entry = d.p1.price.toFixed(2);
            const target = d.p2 ? d.p2.price.toFixed(2) : '--';
            const stop = d.stopPrice != null ? d.stopPrice.toFixed(2) : '--';
            const pDist = d.p2 ? Math.abs(d.p2.price - d.p1.price) : 0;
            const sDist = d.stopPrice != null ? Math.abs(d.stopPrice - d.p1.price) : 0;
            const rr = sDist > 0 ? (pDist / sDist).toFixed(2) : 'N/A';
            const betInput = $el('bet-input');
            const bet = betInput ? (parseFloat(betInput.innerText || betInput.value) || 0) : 0;
            const pPct = d.p1.price > 0 ? (pDist / d.p1.price * 100).toFixed(2) : '0';
            html += `<br>Entry: <span style="color:#fff">$${entry}</span>`;
            html += `<br>Target: <span style="color:${NEON_GREEN}">$${target}</span> (${pPct}%)`;
            html += `<br>Stop: <span style="color:${NEON_RED}">$${stop}</span>`;
            html += `<br>R:R: <span style="color:${NEON_CYAN}">${rr}</span>`;
            if (bet > 0) {
                const profit = (bet * pDist / d.p1.price).toFixed(2);
                const loss = (bet * sDist / d.p1.price).toFixed(2);
                html += `<br>Profit: <span style="color:${NEON_GREEN}">+$${profit}</span> | Loss: <span style="color:${NEON_RED}">-$${loss}</span>`;
            }
        } else if (d.type === 'fib') {
            html += `<br>High: $${Math.max(d.p1.price, d.p2.price).toFixed(2)}`;
            html += `<br>Low: $${Math.min(d.p1.price, d.p2.price).toFixed(2)}`;
        } else if (d.type === 'hline') {
            html += `<br>Price: $${d.p1.price.toFixed(2)}`;
        } else if (d.type === 'measure' && d.p2) {
            const diff = d.p2.price - d.p1.price;
            const pct = d.p1.price !== 0 ? (diff / d.p1.price * 100).toFixed(2) : '0';
            html += `<br>${diff >= 0 ? '+' : ''}${diff.toFixed(2)} (${pct}%)`;
        }
        this._tooltipEl.innerHTML = html;
        this._tooltipEl.style.left = (mx + 16) + 'px';
        this._tooltipEl.style.top = (my - 10) + 'px';
        this._tooltipEl.style.display = 'block';
    }

    _hideTooltip() {
        if (this._tooltipEl) this._tooltipEl.style.display = 'none';
    }

    onMouseUp(e) {
        if (this.currentDrawing) {
            const d = this.currentDrawing;
            if (d.type === 'text' && !d.text) {
                this.currentDrawing = null;
                this.setTool(null);
                document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
                $el('btn-crosshair')?.classList.add('active');
                return;
            }
            if ((d.type === 'long' || d.type === 'short') && d.stopPrice === undefined) {
                const ls = window._lsSettings || {};
                const diff = d.p2.price - d.p1.price;
                if (ls.riskMode === 'fixed' && d.p1.price > 0) {
                    const slPct = (ls.fixedSL || 2.5) / 100;
                    d.stopPrice = d.type === 'long' ? d.p1.price * (1 - slPct) : d.p1.price * (1 + slPct);
                    const tpPct = (ls.fixedTP || 5.0) / 100;
                    d.p2.price = d.type === 'long' ? d.p1.price * (1 + tpPct) : d.p1.price * (1 - tpPct);
                } else {
                    const rr = ls.defaultRR || 1.0;
                    d.stopPrice = rr > 0 ? d.p1.price - (diff / rr) : d.p1.price - diff;
                }
            }
            this.drawings.push(d);
            this.currentDrawing = null;
            this.selectedDrawing = this.drawings[this.drawings.length - 1];
            this.markDirty();
            this.setTool(null);
            document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
            $el('btn-crosshair')?.classList.add('active');
        }
        this.interactionMode = null;
        this.activeHandle = null;
        if (!this.activeTool) {
            this.canvas.style.cursor = 'default';
        }
        if (window.persistDrawings) window.persistDrawings();
    }

    deleteSelected() {
        if (this.selectedDrawing) {
            this._pushUndo();
            this.drawings = this.drawings.filter(d => d !== this.selectedDrawing);
            this.selectedDrawing = null;
            if (typeof _lsShowExecToolbar === 'function') _lsShowExecToolbar(null);
            this.markDirty();
            if (window.persistDrawings) window.persistDrawings();
        }
    }

    getCurrentSymbol() {
        return window.currentSymbol || '';
    }

    _serializeOne(d) {
        const base = { type: d.type, p1: { price: d.p1.price, time: d.p1.time } };
        if (d.p2) base.p2 = { price: d.p2.price, time: d.p2.time };
        if (d.stopPrice !== undefined) base.stopPrice = d.stopPrice;
        if (d.text) base.text = d.text;
        return base;
    }

    serialize() {
        return this.drawings.map(d => this._serializeOne(d));
    }

    setDrawings(arr) {
        if (!Array.isArray(arr)) return;
        this.drawings = arr.map(item => {
            const d = { type: item.type, p1: { ...item.p1 } };
            if (item.p2) d.p2 = { ...item.p2 };
            else d.p2 = { ...item.p1 };
            if (item.stopPrice !== undefined) d.stopPrice = item.stopPrice;
            if (item.text) d.text = item.text;
            return d;
        });
        this.selectedDrawing = null;
        this.markDirty();
    }

    async executeDrawing(d, aiMode) {
        if (!d || !d.p1 || !d.p2) return;

        const betInput = $el('bet-input');
        const betAmount = betInput ? (parseFloat(betInput.innerText || betInput.value) || 0) : 0;
        const price = d.p1.price;
        let qty = (betAmount > 0) ? (betAmount / price) : 0.0001;
        if (qty < 0.0001) qty = 0.0001;

        let side = 'buy';
        if (d.type === 'short') side = 'sell';

        let takeProfit = d.p2.price;
        let stopLoss = d.stopPrice;

        const btnId = aiMode ? 'btn-exec-ai' : 'btn-exec-manual';
        const btn = $el(btnId);
        const originalText = btn ? btn.innerText : "";
        if (btn) btn.innerText = "...";

        try {
            const symbol = this.getCurrentSymbol() || 'BTC/USD';
            const res = await window.__TAURI__.tauri.invoke('submit_order', {
                order: {
                    symbol: symbol,
                    qty: parseFloat(qty.toFixed(6)),
                    side: side,
                    take_profit: parseFloat(takeProfit.toFixed(2)),
                    stop_loss: parseFloat(stopLoss.toFixed(2)),
                    ai_mode: aiMode
                }
            });
            showCyberToast(((aiMode) ? "TITAN AI: " : "EXEC: ") + res, 'success');
        } catch (e) {
            showCyberToast("ERROR: " + normalizeError(e), 'error');
        } finally {
            if (btn) btn.innerText = originalText;
        }
    }

    render() { this.markDirty(); }

    _drawHandle(x, y, isSelected) {
        const pulse = 0.5 + 0.5 * Math.sin(this._pulsePhase);
        const r = isSelected ? 5 + pulse * 2 : 4;
        this.ctx.beginPath();
        this.ctx.arc(x, y, r, 0, Math.PI * 2);
        this.ctx.fillStyle = isSelected ? NEON_CYAN : '#00F3FF';
        this.ctx.fill();
        if (isSelected) {
            this.ctx.beginPath();
            this.ctx.arc(x, y, r + 3, 0, Math.PI * 2);
            this.ctx.strokeStyle = `rgba(0,255,255,${0.3 + pulse * 0.3})`;
            this.ctx.lineWidth = 1;
            this.ctx.stroke();
        }
    }

    _setGlow(color, blur) {
        this.ctx.shadowColor = color;
        this.ctx.shadowBlur = blur;
    }

    _clearGlow() {
        this.ctx.shadowColor = 'transparent';
        this.ctx.shadowBlur = 0;
    }

    _renderInternal() {
        const logW = this._logicalWidth || this.canvas.width;
        const logH = this._logicalHeight || this.canvas.height;
        this.ctx.clearRect(0, 0, logW, logH);

        const priceScaleWidth = 60;
        const chartWidth = logW - priceScaleWidth;

        const betInput = $el('bet-input');
        const betAmount = betInput ? (parseFloat(betInput.innerText || betInput.value) || 0) : 0;

        this.ctx.save();
        this.ctx.beginPath();
        this.ctx.rect(0, 0, chartWidth, logH);
        this.ctx.clip();

        const all = [...this.drawings];
        if (this.currentDrawing) all.push(this.currentDrawing);

        all.forEach(d => {
            if ((d.type === 'long' || d.type === 'short') && d.stopPrice === undefined && d.p1 && d.p2) {
                const _rr = (window._lsSettings && window._lsSettings.defaultRR > 0) ? window._lsSettings.defaultRR : 1.0;
                const _diff = d.p2.price - d.p1.price;
                d.stopPrice = d.p1.price - (_diff / _rr);
            }

            const y1 = this.series.priceToCoordinate(d.p1.price);
            const y2 = d.p2 ? this.series.priceToCoordinate(d.p2.price) : y1;
            const yStop = d.stopPrice !== undefined ? this.series.priceToCoordinate(d.stopPrice) : y1;

            let x1 = this.projectTime(d.p1.time);
            let x2 = d.p2 ? this.projectTime(d.p2.time) : x1;

            if (x1 === null) x1 = -9999;
            if (x2 === null) x2 = -9999;

            const isSelected = (d === this.selectedDrawing);
            this.ctx.lineWidth = 2;

            // --- HLINE ---
            if (d.type === 'hline') {
                const col = isSelected ? '#FFFFFF' : NEON_GREEN;
                this._setGlow(col, isSelected ? 8 : 4);
                this.ctx.beginPath();
                this.ctx.strokeStyle = col;
                this.ctx.moveTo(0, y1);
                this.ctx.lineTo(chartWidth, y1);
                this.ctx.stroke();
                this._clearGlow();
                this.ctx.font = '10px monospace';
                this.ctx.fillStyle = col;
                this.ctx.textAlign = 'left';
                this.ctx.fillText(d.p1.price.toFixed(2), 5, y1 - 5);
                if (isSelected) this._drawHandle(chartWidth / 2, y1, true);
                return;
            }

            // --- VLINE ---
            if (d.type === 'vline') {
                if (x1 === null || x1 < -9999) return;
                const col = isSelected ? '#FFFFFF' : NEON_GREEN;
                this._setGlow(col, isSelected ? 8 : 4);
                this.ctx.beginPath();
                this.ctx.strokeStyle = col;
                this.ctx.setLineDash([6, 4]);
                this.ctx.moveTo(x1, 0);
                this.ctx.lineTo(x1, logH);
                this.ctx.stroke();
                this.ctx.setLineDash([]);
                this._clearGlow();
                const vLabel = this.formatTimeLabel(d.p1.time);
                this.ctx.font = '10px monospace';
                this.ctx.fillStyle = col;
                this.ctx.textAlign = 'left';
                this.ctx.fillText(vLabel, x1 + 4, 12);
                if (isSelected) this._drawHandle(x1, logH / 2, true);
                return;
            }

            // --- TEXT ---
            if (d.type === 'text') {
                if (x1 === null || x1 < -9999) return;
                const txt = d.text || 'Text';
                this.ctx.font = '12px monospace';
                const m = this.ctx.measureText(txt);
                const tw = m.width + 8;
                const th = 18;
                this.ctx.fillStyle = isSelected ? 'rgba(0,255,255,0.15)' : 'rgba(0,191,255,0.12)';
                this.ctx.fillRect(x1 - 4, y1 - th + 4, tw, th);
                this.ctx.strokeStyle = isSelected ? NEON_CYAN : NEON_GREEN;
                this.ctx.strokeRect(x1 - 4, y1 - th + 4, tw, th);
                this.ctx.fillStyle = isSelected ? '#fff' : NEON_GREEN;
                this.ctx.fillText(txt, x1, y1 + 2);
                if (isSelected) this._drawHandle(x1, y1, true);
                return;
            }

            // --- TREND ---
            if (d.type === 'trend') {
                const col = isSelected ? '#FFFFFF' : NEON_GREEN;
                this._setGlow(col, isSelected ? 8 : 4);
                this.ctx.beginPath();
                this.ctx.strokeStyle = col;
                this.ctx.moveTo(x1, y1);
                this.ctx.lineTo(x2, y2);
                this.ctx.stroke();
                this._clearGlow();
                if (isSelected) {
                    this._drawHandle(x1, y1, true);
                    this._drawHandle(x2, y2, true);
                }
                return;
            }

            // --- RAY ---
            if (d.type === 'ray') {
                const rdx = x2 - x1;
                const rdy = y2 - y1;
                let endX = x2, endY = y2;
                if (Math.abs(rdx) > 0.001 || Math.abs(rdy) > 0.001) {
                    const tRight = (chartWidth - x1) / rdx;
                    const tLeft = (0 - x1) / rdx;
                    const tTop = (0 - y1) / rdy;
                    const tBottom = (logH - y1) / rdy;
                    const candidates = [];
                    if (rdx !== 0) { candidates.push({ t: tRight, x: chartWidth, y: y1 + tRight * rdy }); candidates.push({ t: tLeft, x: 0, y: y1 + tLeft * rdy }); }
                    if (rdy !== 0) { candidates.push({ t: tTop, x: x1 + tTop * rdx, y: 0 }); candidates.push({ t: tBottom, x: x1 + tBottom * rdx, y: logH }); }
                    const valid = candidates.filter(c => c.t >= 0 && c.x >= 0 && c.x <= chartWidth && c.y >= 0 && c.y <= logH);
                    if (valid.length > 0) {
                        const best = valid.reduce((a, b) => a.t < b.t ? a : b);
                        endX = best.x;
                        endY = best.y;
                    }
                }
                const col = isSelected ? '#FFFFFF' : NEON_GREEN;
                this._setGlow(col, isSelected ? 8 : 4);
                this.ctx.beginPath();
                this.ctx.strokeStyle = col;
                this.ctx.moveTo(x1, y1);
                this.ctx.lineTo(endX, endY);
                this.ctx.stroke();
                this._clearGlow();
                if (isSelected) {
                    this._drawHandle(x1, y1, true);
                    this._drawHandle(x2, y2, true);
                }
                return;
            }

            // --- FIB ---
            if (d.type === 'fib') {
                const high = Math.max(d.p1.price, d.p2.price);
                const low = Math.min(d.p1.price, d.p2.price);
                const range = high - low;
                const levels = [0, 0.236, 0.382, 0.5, 0.618, 0.786, 1];
                const colors = ['#FF6B6B', '#FFA94D', '#FFE066', '#69DB7C', '#51CF66', '#37B24D', '#2F9E44'];
                const lx1 = Math.min(x1, x2);
                const lx2 = Math.max(x1, x2);

                levels.forEach((lev, i) => {
                    const price = high - range * lev;
                    const yLev = this.series.priceToCoordinate(price);
                    const nextLev = levels[i + 1];
                    if (nextLev !== undefined) {
                        const nextPrice = high - range * nextLev;
                        const yNext = this.series.priceToCoordinate(nextPrice);
                        this.ctx.fillStyle = colors[i] + '15';
                        this.ctx.fillRect(lx1, Math.min(yLev, yNext), lx2 - lx1, Math.abs(yNext - yLev));
                    }
                    this.ctx.beginPath();
                    this.ctx.strokeStyle = isSelected ? '#FFFFFF' : colors[i];
                    this.ctx.setLineDash([4, 4]);
                    this.ctx.moveTo(lx1, yLev);
                    this.ctx.lineTo(lx2, yLev);
                    this.ctx.stroke();
                    this.ctx.setLineDash([]);
                    this.ctx.font = '9px monospace';
                    this.ctx.fillStyle = colors[i];
                    this.ctx.textAlign = 'left';
                    this.ctx.fillText((lev * 100).toFixed(1) + '% ' + price.toFixed(2), lx1 + 4, yLev - 2);
                });
                this.ctx.strokeStyle = isSelected ? '#FFFFFF' : NEON_GREEN;
                this.ctx.strokeRect(lx1, Math.min(y1, y2), Math.abs(x2 - x1), Math.abs(y2 - y1));
                if (isSelected) {
                    this._drawHandle(x1, y1, true);
                    this._drawHandle(x2, y2, true);
                }
                return;
            }

            // --- MEASURE ---
            if (d.type === 'measure') {
                const col = isSelected ? '#FFFFFF' : NEON_CYAN;
                this._setGlow(col, isSelected ? 6 : 3);
                this.ctx.beginPath();
                this.ctx.strokeStyle = col;
                this.ctx.setLineDash([6, 3]);
                this.ctx.moveTo(x1, y1);
                this.ctx.lineTo(x2, y2);
                this.ctx.stroke();
                this.ctx.setLineDash([]);
                this._clearGlow();

                const priceDiff = d.p2.price - d.p1.price;
                const timeDiff = d.p2.time - d.p1.time;
                const pct = d.p1.price !== 0 ? ((priceDiff / d.p1.price) * 100) : 0;
                const midX = (x1 + x2) / 2;
                const midY = (y1 + y2) / 2;

                this.ctx.fillStyle = 'rgba(0,0,0,0.75)';
                this.ctx.fillRect(midX - 80, midY - 18, 160, 30);
                this.ctx.strokeStyle = NEON_CYAN + '44';
                this.ctx.strokeRect(midX - 80, midY - 18, 160, 30);

                const tf = window.currentTimeframe || window.currentTF || '5m';
                const tfSec = (typeof tfToSeconds === 'function') ? tfToSeconds(tf) : 0;
                const mt1 = this.normalizeTime(d.p1.time);
                const mt2 = this.normalizeTime(d.p2.time);
                const bars = (mt1 && mt2 && tfSec) ? Math.round(Math.abs(mt2 - mt1) / tfSec) : 0;
                const measureLabel = `${priceDiff >= 0 ? '+' : ''}${priceDiff.toFixed(2)} (${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%)` + (bars > 0 ? ` \u2022 ${bars} bars` : '');

                this.ctx.font = '10px monospace';
                this.ctx.fillStyle = col;
                this.ctx.textAlign = 'center';
                this.ctx.fillText(measureLabel, midX, midY + 2);
                if (isSelected) {
                    this._drawHandle(x1, y1, true);
                    this._drawHandle(x2, y2, true);
                }
                return;
            }

            // --- LONG / SHORT ---
            const ls = window._lsSettings || {};
            const cProfit = ls.colorProfit || PROFIT_STROKE;
            const cStop = ls.colorStop || LOSS_STROKE;

            const w = x2 - x1;
            const h = y2 - y1;
            const stopH = yStop - y1;

            const hexToRgba = (hex, a) => {
                const r = parseInt(hex.slice(1,3),16), g = parseInt(hex.slice(3,5),16), b = parseInt(hex.slice(5,7),16);
                return `rgba(${r},${g},${b},${a})`;
            };

            const profitDist = Math.abs(d.p2.price - d.p1.price);
            const lossDist = Math.abs((d.stopPrice || d.p1.price) - d.p1.price);
            const isUp = d.p2.price > d.p1.price;
            const leg2Profit = d.type === 'long' ? isUp : !isUp;
            const leg2Color = leg2Profit ? cProfit : cStop;
            const legStopColor = leg2Profit ? cStop : cProfit;

            this.ctx.shadowBlur = isSelected ? 10 : 0;
            this.ctx.shadowColor = "#00BFFF";

            this.ctx.fillStyle = hexToRgba(leg2Color, 0.06);
            this.ctx.strokeStyle = leg2Color;
            this.ctx.fillRect(x1, y1, w, h);
            this.ctx.strokeRect(x1, y1, w, h);

            this.ctx.fillStyle = hexToRgba(legStopColor, 0.06);
            this.ctx.strokeStyle = legStopColor;
            this.ctx.fillRect(x1, y1, w, stopH);
            this.ctx.strokeRect(x1, y1, w, stopH);

            this.ctx.shadowBlur = 0;

            // Entry line
            this.ctx.beginPath();
            this.ctx.strokeStyle = '#FFFFFF';
            this.ctx.lineWidth = 1;
            this.ctx.setLineDash([4, 2]);
            this.ctx.moveTo(x1, y1);
            this.ctx.lineTo(x1 + w, y1);
            this.ctx.stroke();
            this.ctx.setLineDash([]);
            this.ctx.lineWidth = 2;

            const rr = lossDist > 0 ? (profitDist / lossDist).toFixed(2) : 'N/A';
            const bet = typeof _lsGetBetAmount === 'function' ? _lsGetBetAmount() : (betAmount || 0);
            const qty = (bet > 0 && d.p1.price > 0) ? bet / d.p1.price : 0;

            if (ls.showEntry !== false) {
                this.ctx.font = 'bold 10px monospace';
                this.ctx.fillStyle = '#FFFFFF';
                this.ctx.textAlign = 'left';
                this.ctx.fillText('ENTRY: ' + d.p1.price.toFixed(2), x1 + 5, y1 - 5);
            }

            const drawLegLabel = (role, dist, color, yStart, hVal, priceEnd, isProfit) => {
                if (Math.abs(hVal) < 16) return;
                const parts = [];
                if (ls.showPct !== false) {
                    const pct = d.p1.price !== 0 ? ((dist / d.p1.price) * 100).toFixed(2) : '0.00';
                    parts.push(role + ' ' + pct + '%');
                } else {
                    parts.push(role);
                }
                if (ls.showDollar && bet > 0) {
                    const usd = (qty * dist).toFixed(2);
                    parts.push((isProfit ? '+$' : '-$') + usd);
                }
                if (ls.showRR !== false) parts.push('R:R ' + rr);

                this.ctx.textAlign = 'center';
                let lx = x1 + w / 2;
                if (lx < 0) lx = 10;
                if (lx > chartWidth) lx = chartWidth - 10;

                this.ctx.fillStyle = color;
                this.ctx.font = 'bold 11px sans-serif';
                this._setGlow(color, 4);
                this.ctx.fillText(parts.join('  '), lx, yStart + hVal / 2);
                this._clearGlow();

                if (ls.showPrices) {
                    this.ctx.font = '10px monospace';
                    this.ctx.fillText(priceEnd.toFixed(2), lx, yStart + hVal / 2 + 13);
                }
            };

            drawLegLabel('TARGET', profitDist, leg2Color, y1, h, d.p2.price, leg2Profit);
            drawLegLabel('STOP', lossDist, legStopColor, y1, stopH, d.stopPrice || d.p1.price, !leg2Profit);

            if (isSelected) {
                this._drawHandle(x1, y1, true);
                this._drawHandle(x2, y2, true);
                this._drawHandle((x1 + x2) / 2, yStop, true);
            }

            if (isSelected && typeof _lsShowExecToolbar === 'function') _lsShowExecToolbar(d);
            if (typeof _lsUpdateCalcFields === 'function') _lsUpdateCalcFields();
        });

        // Exec toolbar position
        const toolbar = $el('exec-toolbar');
        if (toolbar) {
            if (this.selectedDrawing && (this.selectedDrawing.type === 'long' || this.selectedDrawing.type === 'short')) {
                const dd = this.selectedDrawing;
                const tx = this.projectTime(dd.p1.time);
                const ty = this.series.priceToCoordinate(dd.p1.price);

                if (tx !== null && ty !== null) {
                    toolbar.style.display = 'flex';
                    toolbar.style.left = (tx + 40) + 'px';
                    toolbar.style.top = (ty - 30) + 'px';
                } else {
                    toolbar.style.display = 'none';
                }
            } else {
                toolbar.style.display = 'none';
            }
        }

        this.ctx.restore();
    }

    clear() {
        this._pushUndo();
        this.drawings = [];
        this.markDirty();
        if (window.persistDrawings) window.persistDrawings();
    }
}

// --- GLOBAL HELPERS ---
function toggletool(tool, btnElement) {
    document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');

    const chartObj = (typeof chart !== 'undefined' && chart) ? chart : window.chart;
    if (!chartObj) return;

    if (!drawingManager) {
        try {
            const seriesObj = (typeof candleSeries !== 'undefined' && candleSeries) ? candleSeries : window.candleSeries;
            if (chartObj && seriesObj) {
                drawingManager = new DrawingManager(chartObj, seriesObj, '');
                window.drawingManager = drawingManager;
                if (window.loadDrawings) window.loadDrawings();
            }
        } catch (e) {
            TitanError.report('drawing:error', e);
            return;
        }
    }

    if (drawingManager) drawingManager.setTool(tool);
}

function openIndicators() { if (typeof toggleIndicatorPanel === 'function') toggleIndicatorPanel(); }
async function clearDrawings() {
    if (await cyberConfirm("Delete all drawings?")) {
        if (drawingManager) drawingManager.clear();
    }
}

function toggleSnapToPrice() {
    if (drawingManager) {
        drawingManager.snapToPrice = !drawingManager.snapToPrice;
        const btn = $el('btn-snap');
        if (btn) {
            btn.classList.toggle('active', drawingManager.snapToPrice);
            btn.title = drawingManager.snapToPrice ? 'Snap ON' : 'Snap OFF';
        }
    }
}

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
    const btnSnap = $el('btn-snap');

    const btnExecMan = $el('btn-exec-manual');
    const btnExecAi = $el('btn-exec-ai');

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
    if (btnSnap) btnSnap.addEventListener('click', toggleSnapToPrice);

    // btn-exec-manual and btn-exec-ai handlers are bound in index.html via _lsExecuteDrawing
    // Do NOT add duplicate handlers here (was causing double order submission MT-01)

    TitanDebug.log('drawing', 'Drawing engine V2 events bound.');
}, 1000);

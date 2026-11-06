let chart, candleSeries;
const _catalystCache = {};

let lastBar = null;
let livePriceLine = null;
let volChart, volSeries;
let ema9Series, ema50Series, ema200Series;
let vwapSeries;
let bbUpperSeries, bbMidSeries, bbLowerSeries;
let rsiChart, rsiSeries, rsiUpperLine, rsiLowerLine, rsiMidLine;
let macdChart, macdLineSeries, macdSignalSeries, macdHistSeries, macdZeroLine;
let adxChart, adxSeries, adxLine20, adxLine25;
let oracleChart, seriesOracle, seriesOracleZero;
let lastRefLinesLen = 0;
let signalMarkers = [];
let lastCandleData = [];
let lastVwap = null;
const rightBarState = {
    prevClose: null,
    sessionHigh: null,
    sessionLow: null,
    lastTickTime: null
};
const indicatorDefaults = {
    ema9: 9,
    ema50: 50,
    ema200: 200,
    bbPeriod: 20,
    bbStd: 2,
    rsiPeriod: 14,
    macdFast: 12,
    macdSlow: 26,
    macdSignal: 9,
    adxPeriod: 14
};
let indicatorConfig = { ...indicatorDefaults };
let chartSettingsLoaded = false;
let mainChartHeight = null;
const histogramPaneIds = ['chart-vol', 'chart-macd', 'chart-rsi', 'chart-adx', 'chart-oracle'];
const paneHeights = {};
const histogramScales = {};
const freePanelIds = ['l2-panel', 'l2-heatmap-panel', 'chart-topbar-panel'];
const panelHeatmaps = { 'l2-heatmap-panel': 'l2-heatmap' };
const freePanelHeights = {};
const heatmapHeights = {};
let histogramControlsBound = false;
let mainControlsBound = false;
let freePanelControlsBound = false;
let tabsBound = false;
let currentSymbol = 'BTC/USD';
let _loadSeq = 0;
let currentTF = '5m';
window.currentTimeframe = currentTF;
let replayState = { active: false, playing: false, index: 0, speed: 1, timer: null, data: [] };
let priceAlerts = [];
let lastAlertPrice = null;

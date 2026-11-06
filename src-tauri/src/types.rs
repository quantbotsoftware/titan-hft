use crate::app_state::Candle;
use crate::indicators;
use crate::scanner;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct WalletSnapshot {
    pub bot_cash: f64,
    pub user_deposit: f64,
}

#[derive(Deserialize)]
pub(crate) struct AlpacaAccount {
    pub equity: String,
    pub cash: String,
    pub portfolio_value: String,
    pub last_equity: String,
    #[serde(default)]
    pub buying_power: String,
}

#[derive(Deserialize)]
pub(crate) struct AlpacaPosition {
    pub symbol: String,
    pub qty: String,
    pub side: String,
    pub current_price: Option<String>,
    pub avg_entry_price: Option<String>,
    pub market_value: String,
    pub unrealized_pl: String,
    pub unrealized_plpc: String,
}

#[derive(Serialize)]
pub(crate) struct OptimizeResult {
    pub best_tf: String,
    pub best_roi: f64,
    pub results: Vec<TimeframeScore>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TimeframeScore {
    pub tf: String,
    pub roi: f64,
}

#[derive(Serialize, Clone, Default)]
pub(crate) struct BacktestMetrics {
    pub trades: usize,
    pub win_rate: f64,
    pub return_pct: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub sharpe: f64,
    pub expectancy: f64,
}

#[derive(Serialize)]
pub(crate) struct BacktestResult {
    #[serde(flatten)]
    pub metrics: BacktestMetrics,
    pub equity_curve: Vec<f64>,
}

impl std::ops::Deref for BacktestResult {
    type Target = BacktestMetrics;
    fn deref(&self) -> &BacktestMetrics {
        &self.metrics
    }
}

#[derive(Serialize)]
pub(crate) struct BacktestSweepResult {
    pub tf: String,
    #[serde(flatten)]
    pub metrics: BacktestMetrics,
}

impl std::ops::Deref for BacktestSweepResult {
    type Target = BacktestMetrics;
    fn deref(&self) -> &BacktestMetrics {
        &self.metrics
    }
}

#[derive(Serialize)]
pub(crate) struct BacktestSweepMultiResult {
    pub symbol: String,
    pub tf: String,
    #[serde(flatten)]
    pub metrics: BacktestMetrics,
}

impl std::ops::Deref for BacktestSweepMultiResult {
    type Target = BacktestMetrics;
    fn deref(&self) -> &BacktestMetrics {
        &self.metrics
    }
}

#[derive(Serialize)]
pub(crate) struct ExportResult {
    pub path: String,
    pub rows: usize,
}

#[derive(Serialize)]
pub(crate) struct ExportJsonResult {
    pub path: String,
    pub bars: usize,
}

#[derive(Serialize)]
pub(crate) struct ExportHtmlResult {
    pub path: String,
    pub bars: usize,
}

#[derive(Serialize)]
pub(crate) struct StrategyScore {
    pub score: f64,
    pub signal: String,
    pub rsi: f64,
    pub ema9: f64,
    pub ema50: f64,
    pub adx: f64,
    pub trend: String,
    pub gpu_boost: f64,
    pub oracle_score: f64,
    pub oracle_confidence: f64,
    pub oracle_regime: String,
}

#[derive(Serialize)]
pub(crate) struct ExportSweepResult {
    pub path: String,
    pub rows: usize,
}

#[derive(Serialize)]
pub(crate) struct ExportReportResult {
    pub path: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct SignalMarker {
    pub time: i64,
    pub position: String,
    pub color: String,
    pub shape: String,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct StScorePoint {
    pub time: i64,
    pub score: f64,
    pub state: String,
}

#[derive(Serialize)]
pub(crate) struct SignalMarkersResponse {
    pub markers: Vec<SignalMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub st_scores: Option<Vec<StScorePoint>>,
}

#[derive(Serialize)]
pub(crate) struct GpuBenchResult {
    pub len: usize,
    pub value: f32,
    pub ms: u128,
}

#[derive(Clone)]
pub(crate) struct BacktestInputs {
    pub ema9: Vec<Option<f64>>,
    pub ema50: Vec<Option<f64>>,
    pub rsi: Vec<Option<f64>>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SignalBacktestTfResult {
    pub tf: String,
    pub trades: usize,
    pub win_rate: f64,
    pub return_pct: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub sharpe: f64,
    pub expectancy: f64,
    pub best: bool,
}

#[derive(Serialize)]
pub(crate) struct SignalBacktestResult {
    pub symbol: String,
    pub period_days: i64,
    pub results: Vec<SignalBacktestTfResult>,
    pub best_tf: String,
    pub best_return: f64,
    pub best_sharpe: f64,
}

#[derive(Serialize)]
pub(crate) struct DeepFocusResult {
    pub symbol: String,
    pub roi: f64,
    pub volatility: f64,
    pub trend: String,
    pub last_price: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LegacyOrder {
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    #[serde(default)]
    pub take_profit: Option<f64>,
    #[serde(default)]
    pub stop_loss: Option<f64>,
}

pub(crate) struct ScanScoringResult {
    pub results: Vec<scanner::AiScanCandidate>,
    pub scored: usize,
    pub dropped: usize,
    pub total: usize,
    pub snap_count: usize,
}

#[derive(Serialize)]
pub(crate) struct ChartDataResponse {
    pub candles: Vec<Candle>,
    pub indicators_history: Option<indicators::IndicatorsHistory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_metrics_default_values() {
        let m = BacktestMetrics::default();
        assert_eq!(m.trades, 0);
        assert_eq!(m.win_rate, 0.0);
        assert_eq!(m.return_pct, 0.0);
        assert_eq!(m.max_drawdown, 0.0);
        assert_eq!(m.profit_factor, 0.0);
        assert_eq!(m.sharpe, 0.0);
        assert_eq!(m.expectancy, 0.0);
    }
}

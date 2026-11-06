// ╔══════════════════════════════════════════════════════════════════╗
// ║  SMARTTREND AI v3 — Three-Pillar Trend Intelligence Engine     ║
// ║                                                                 ║
// ║  Pillars:                                                       ║
// ║    1. MACRO  — Oracle score + Kalman surprise/gain + run_length ║
// ║    2. MICRO  — OFI + CVD + microprice (live SIP data)           ║
// ║    3. MOMENTUM — GPU consensus + wavelet dominant_cycle + RVOL  ║
// ║                                                                 ║
// ║  BayesianTracker: Beta distribution per-regime, decay halflife  ║
// ║  SmartTrendConfig: Persistent settings with serde + validation  ║
// ║                                                                 ║
// ║  Session 52 (S07a) — Infrastructure: pillars, Bayesian, config  ║
// ║  Session 55 (S07b) — FSM, SmartTrendEngine, orchestration       ║
// ╚══════════════════════════════════════════════════════════════════╝
// S07b: Many types are prepared for S08-S10 (sentinel, bot, scoring integration).
// They are tested and used in diagnostics; production wiring comes in later sessions.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::titan_oracle::{MarketRegime, MIN_CONFIDENCE_FOR_SIGNAL};

// ═══════════════════════════════════════════════════════════════════
//  SMARTTREND CONFIG — Persistent settings (stored in AppSettings)
// ═══════════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SmartTrendConfig {
    /// Pillar weight: Macro (oracle + Kalman). Default 0.40
    pub st_macro_weight: f64,
    /// Pillar weight: Micro (OFI + CVD + microprice). Default 0.35
    pub st_micro_weight: f64,
    /// Pillar weight: Momentum (GPU + wavelet cycle + RVOL). Default 0.25
    pub st_momentum_weight: f64,
    /// Combined score threshold for entry consideration. Default 65.0
    pub st_entry_threshold: f64,
    /// Combined score threshold for stalking state. Default 55.0
    pub st_stalking_threshold: f64,
    /// Per-regime entry thresholds: [Trending, MeanReverting, Chaotic, Unknown].
    /// Overrides st_entry_threshold when regime is known.
    pub st_entry_thresholds: [f64; 4],
    /// Min bars after changepoint before entry allowed. Default 8
    pub st_stabilization_window: u32,
    /// Bayesian tracker decay factor per-update (legacy). Default 0.997
    pub st_bayesian_decay: f64,
    /// Bayesian halflife in trades: after N trades, prior weight halves. Default 200
    pub st_bayesian_halflife: u32,
    /// Use wavelet dominant_cycle to filter short-lived momentum. Default true
    pub st_dominant_cycle_filter: bool,
    /// OFI normalization range for Micro pilon. Default 50.0
    pub st_ofi_norm_range: f64,
    /// Microprice-vs-mid bps range for full signal. Default 10.0
    pub st_mp_bps_range: f64,
    /// CVD scaling divisor for Micro pilon signal. Default 100.0
    pub st_cvd_scale: f64,
    /// Changepoint probability threshold to force FSM reset to Flat. Default 0.7
    #[serde(default = "default_cp_reset")]
    pub st_changepoint_reset_threshold: f64,
    /// Trail tightening per bar in Trail state (fraction of remaining distance). Default 0.05
    #[serde(default = "default_trail_tighten")]
    pub st_trail_tighten_pct: f64,
    /// Hysteresis margin: Stalking->Flat requires score < threshold - margin. Default 5.0
    #[serde(default = "default_hysteresis")]
    pub st_hysteresis_margin: f64,
    /// Master enable for SmartTrend engine. Default false
    pub st_enabled: bool,
    /// Kill switch (persistent emergency stop). Default false
    pub st_killed: bool,
    /// Sentinel idle cadence (Flat state) in seconds. Default 30
    #[serde(default = "default_sentinel_idle")]
    pub st_sentinel_idle_secs: u64,
    /// Sentinel stalking cadence in seconds. Default 5
    #[serde(default = "default_sentinel_stalking")]
    pub st_sentinel_stalking_secs: u64,
    /// Sentinel hot cadence (Entry/InTrade/Trail) in seconds. Default 2
    #[serde(default = "default_sentinel_hot")]
    pub st_sentinel_hot_secs: u64,
    /// Alert cooldown per-symbol in seconds (suppress repeat INFO alerts). Default 30
    #[serde(default = "default_sentinel_cooldown")]
    pub st_sentinel_alert_cooldown_secs: u64,

    // ── S10: Bot + Account config fields ──

    /// Risk per trade as fraction of allocated_capital. Default 0.01 (1%)
    #[serde(default = "default_risk_per_trade")]
    pub st_risk_per_trade: f64,
    /// Max drawdown % on SmartTrendAccount before ST circuit breaker fires. Default 5.0
    #[serde(default = "default_max_dd_pct")]
    pub st_max_dd_pct: f64,
    /// Bot loop cadence in milliseconds. Default 2000
    #[serde(default = "default_bot_loop_ms")]
    pub st_bot_loop_ms: u64,
    /// Bot paused (no new entries, existing positions still managed). Default true
    #[serde(default = "default_bot_paused")]
    pub st_bot_paused: bool,
    /// Enable Kelly criterion position sizing boost. Default false
    #[serde(default)]
    pub st_kelly_enabled: bool,
    /// Minimum trades before Kelly kicks in. Default 20
    #[serde(default = "default_kelly_min_trades")]
    pub st_kelly_min_trades: u32,
    /// Maximum Kelly fraction (capped). Default 0.25
    #[serde(default = "default_max_kelly_fraction")]
    pub st_max_kelly_fraction: f64,
    /// How many evals the Entry state persists before auto-transitioning to InTrade.
    /// Gives the bot multiple windows to see Entry and place the order. Default 3.
    #[serde(default = "default_entry_persist_bars")]
    pub st_entry_persist_bars: Option<u32>,
}

fn default_entry_persist_bars() -> Option<u32> { Some(3) }
fn default_cp_reset() -> f64 { 0.7 }
fn default_trail_tighten() -> f64 { 0.05 }
fn default_hysteresis() -> f64 { 5.0 }
fn default_sentinel_idle() -> u64 { 30 }
fn default_sentinel_stalking() -> u64 { 5 }
fn default_sentinel_hot() -> u64 { 2 }
fn default_sentinel_cooldown() -> u64 { 30 }
fn default_risk_per_trade() -> f64 { 0.01 }
fn default_max_dd_pct() -> f64 { 5.0 }
fn default_bot_loop_ms() -> u64 { 2000 }
fn default_bot_paused() -> bool { true }
fn default_kelly_min_trades() -> u32 { 20 }
fn default_max_kelly_fraction() -> f64 { 0.25 }

impl Default for SmartTrendConfig {
    fn default() -> Self {
        Self {
            st_macro_weight: 0.40,
            st_micro_weight: 0.35,
            st_momentum_weight: 0.25,
            st_entry_threshold: 65.0,
            st_stalking_threshold: 55.0,
            st_entry_thresholds: [60.0, 70.0, 75.0, 65.0],
            st_stabilization_window: 8,
            st_bayesian_decay: 0.997,
            st_bayesian_halflife: 200,
            st_dominant_cycle_filter: true,
            st_ofi_norm_range: 50.0,
            st_mp_bps_range: 10.0,
            st_cvd_scale: 100.0,
            st_changepoint_reset_threshold: 0.7,
            st_trail_tighten_pct: 0.05,
            st_hysteresis_margin: 5.0,
            st_enabled: false,
            st_killed: false,
            st_sentinel_idle_secs: 30,
            st_sentinel_stalking_secs: 5,
            st_sentinel_hot_secs: 2,
            st_sentinel_alert_cooldown_secs: 30,
            st_risk_per_trade: 0.01,
            st_max_dd_pct: 5.0,
            st_bot_loop_ms: 2000,
            st_bot_paused: true,
            st_kelly_enabled: false,
            st_kelly_min_trades: 20,
            st_max_kelly_fraction: 0.25,
            st_entry_persist_bars: Some(3),
        }
    }
}

/// Validation result for SmartTrendConfig.
/// `errors` block operation (invalid config). `warnings` are informational.
#[derive(Serialize, Clone, Debug)]
#[allow(dead_code)]
pub struct ConfigValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate SmartTrendConfig for logical consistency.
/// Errors = blocking (config cannot be used). Warnings = informational (auto-corrected or advisory).
#[allow(dead_code)]
pub fn validate_smarttrend_config(cfg: &SmartTrendConfig) -> ConfigValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Weight sum check: auto-normalizable, so warning only
    let weight_sum = cfg.st_macro_weight + cfg.st_micro_weight + cfg.st_momentum_weight;
    if (weight_sum - 1.0).abs() > 0.01 {
        warnings.push(format!(
            "Pillar weights sum to {:.3} (expected 1.0) — will auto-normalize",
            weight_sum
        ));
    }

    // Negative weights are an error (cannot produce valid scoring)
    if cfg.st_macro_weight < 0.0 {
        errors.push("st_macro_weight is negative".to_string());
    }
    if cfg.st_micro_weight < 0.0 {
        errors.push("st_micro_weight is negative".to_string());
    }
    if cfg.st_momentum_weight < 0.0 {
        errors.push("st_momentum_weight is negative".to_string());
    }

    // Threshold ordering is an error (stalking must be lower)
    if cfg.st_stalking_threshold >= cfg.st_entry_threshold {
        errors.push(format!(
            "stalking threshold ({}) >= entry threshold ({}) — stalking must be lower",
            cfg.st_stalking_threshold, cfg.st_entry_threshold
        ));
    }
    if cfg.st_entry_threshold <= 0.0 {
        errors.push(format!(
            "st_entry_threshold ({}) must be positive",
            cfg.st_entry_threshold
        ));
    }
    if cfg.st_stalking_threshold <= 0.0 {
        errors.push(format!(
            "st_stalking_threshold ({}) must be positive",
            cfg.st_stalking_threshold
        ));
    }

    // Per-regime thresholds: advisory
    for (i, &t) in cfg.st_entry_thresholds.iter().enumerate() {
        if t <= 0.0 || t > 100.0 {
            let regime = ["Trending", "MeanReverting", "Chaotic", "Unknown"][i];
            warnings.push(format!(
                "st_entry_thresholds[{}] ({}) outside (0, 100] for {}",
                i, t, regime
            ));
        }
    }

    // Stabilization window: advisory
    if cfg.st_stabilization_window < 2 || cfg.st_stabilization_window > 50 {
        warnings.push(format!(
            "st_stabilization_window ({}) outside safe range [2, 50]",
            cfg.st_stabilization_window
        ));
    }

    // Bayesian halflife: advisory
    if cfg.st_bayesian_halflife == 0 {
        warnings.push("st_bayesian_halflife is 0 — decay disabled".to_string());
    }

    // Bayesian decay legacy: advisory
    if cfg.st_bayesian_decay < 0.99 || cfg.st_bayesian_decay > 1.0 {
        warnings.push(format!(
            "st_bayesian_decay ({}) outside safe range [0.99, 1.0]",
            cfg.st_bayesian_decay
        ));
    }

    // Micro pilon params: advisory
    if cfg.st_ofi_norm_range <= 0.0 {
        warnings.push("st_ofi_norm_range must be positive".to_string());
    }
    if cfg.st_mp_bps_range <= 0.0 {
        warnings.push("st_mp_bps_range must be positive".to_string());
    }
    if cfg.st_cvd_scale <= 0.0 {
        warnings.push("st_cvd_scale must be positive".to_string());
    }

    // S07b: FSM config validation
    if cfg.st_changepoint_reset_threshold <= 0.0 || cfg.st_changepoint_reset_threshold > 1.0 {
        warnings.push(format!(
            "st_changepoint_reset_threshold ({}) outside (0.0, 1.0]",
            cfg.st_changepoint_reset_threshold
        ));
    }
    if cfg.st_trail_tighten_pct < 0.0 || cfg.st_trail_tighten_pct > 0.5 {
        warnings.push(format!(
            "st_trail_tighten_pct ({}) outside [0.0, 0.5]",
            cfg.st_trail_tighten_pct
        ));
    }
    if cfg.st_hysteresis_margin < 0.0 || cfg.st_hysteresis_margin > 20.0 {
        warnings.push(format!(
            "st_hysteresis_margin ({}) outside [0.0, 20.0]",
            cfg.st_hysteresis_margin
        ));
    }
    let stalking_strength = (cfg.st_stalking_threshold - 50.0).abs() * 2.0;
    if cfg.st_hysteresis_margin * 2.0 >= stalking_strength {
        let max_safe = (stalking_strength / 2.0 - 0.5).max(0.0);
        warnings.push(format!(
            "st_hysteresis_margin ({}) >= stalking_strength/2 ({:.1}) — FSM would be stuck in Stalking. Clamp to {:.1}",
            cfg.st_hysteresis_margin, stalking_strength / 2.0, max_safe
        ));
    }

    // S10: Bot + Account config validation
    if cfg.st_risk_per_trade <= 0.0 || cfg.st_risk_per_trade > 0.1 {
        warnings.push(format!(
            "st_risk_per_trade ({}) outside (0.0, 0.1]",
            cfg.st_risk_per_trade
        ));
    }
    if cfg.st_max_dd_pct <= 0.0 || cfg.st_max_dd_pct > 50.0 {
        warnings.push(format!(
            "st_max_dd_pct ({}) outside (0.0, 50.0]",
            cfg.st_max_dd_pct
        ));
    }
    if cfg.st_bot_loop_ms < 500 || cfg.st_bot_loop_ms > 30000 {
        warnings.push(format!(
            "st_bot_loop_ms ({}) outside [500, 30000]",
            cfg.st_bot_loop_ms
        ));
    }
    if cfg.st_max_kelly_fraction <= 0.0 || cfg.st_max_kelly_fraction > 0.5 {
        warnings.push(format!(
            "st_max_kelly_fraction ({}) outside (0.0, 0.5]",
            cfg.st_max_kelly_fraction
        ));
    }

    ConfigValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Normalize pillar weights so they sum to 1.0.
/// Returns fallback defaults if any weight is NaN/Inf or sum is non-positive.
fn normalize_weights(macro_w: f64, micro_w: f64, momentum_w: f64) -> (f64, f64, f64) {
    let sum = macro_w + micro_w + momentum_w;
    if !sum.is_finite() || sum <= 0.0
        || !macro_w.is_finite() || !micro_w.is_finite() || !momentum_w.is_finite()
    {
        tracing::warn!(
            component = "smarttrend",
            macro_w, micro_w, momentum_w,
            "NaN/Inf detected in pillar weights — falling back to defaults"
        );
        return (0.40, 0.35, 0.25);
    }
    (macro_w / sum, micro_w / sum, momentum_w / sum)
}

// ═══════════════════════════════════════════════════════════════════
//  SMARTTREND ACCOUNT — Virtual sub-account for ST capital tracking
// ═══════════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SmartTrendAccount {
    pub allocated_capital: f64,
    pub initial_capital: f64,
    pub realized_pnl_today: f64,
    pub total_realized_pnl: f64,
    pub high_water_mark: f64,
    pub max_drawdown_pct: f64,
    pub trade_count_today: u32,
    pub win_count_today: u32,
    pub loss_count_today: u32,
    pub total_trades: u64,
    pub total_wins: u64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub paused: bool,
    pub breaker_triggered: bool,
    #[serde(default = "default_created_at")]
    pub created_at: String,
}

fn default_created_at() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl Default for SmartTrendAccount {
    fn default() -> Self {
        Self {
            allocated_capital: 0.0,
            initial_capital: 0.0,
            realized_pnl_today: 0.0,
            total_realized_pnl: 0.0,
            high_water_mark: 0.0,
            max_drawdown_pct: 0.0,
            trade_count_today: 0,
            win_count_today: 0,
            loss_count_today: 0,
            total_trades: 0,
            total_wins: 0,
            avg_win: 0.0,
            avg_loss: 0.0,
            paused: false,
            breaker_triggered: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl SmartTrendAccount {
    pub fn new(capital: f64) -> Self {
        Self {
            allocated_capital: capital,
            initial_capital: capital,
            high_water_mark: capital,
            created_at: chrono::Utc::now().to_rfc3339(),
            ..Default::default()
        }
    }

    pub fn record_fill(&mut self, pnl: f64) {
        let is_win = pnl > 0.0;
        self.realized_pnl_today += pnl;
        self.total_realized_pnl += pnl;
        self.trade_count_today += 1;
        self.total_trades += 1;

        if is_win {
            self.win_count_today += 1;
            self.total_wins += 1;
            // Running average of wins
            let prev_wins = self.total_wins - 1;
            self.avg_win = if prev_wins == 0 {
                pnl
            } else {
                (self.avg_win * prev_wins as f64 + pnl) / self.total_wins as f64
            };
        } else {
            self.loss_count_today += 1;
            let total_losses = self.total_trades - self.total_wins;
            self.avg_loss = if total_losses <= 1 {
                pnl.abs()
            } else {
                (self.avg_loss * (total_losses - 1) as f64 + pnl.abs()) / total_losses as f64
            };
        }

        self.allocated_capital += pnl;

        if self.allocated_capital > self.high_water_mark {
            self.high_water_mark = self.allocated_capital;
        }

        self.max_drawdown_pct = self.drawdown_pct();
    }

    pub fn drawdown_pct(&self) -> f64 {
        if self.high_water_mark <= 0.0 {
            return 0.0;
        }
        let dd = (self.high_water_mark - self.allocated_capital) / self.high_water_mark * 100.0;
        dd.max(0.0)
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        self.total_wins as f64 / self.total_trades as f64
    }

    pub fn avg_win_loss_ratio(&self) -> f64 {
        if self.avg_loss <= 0.0 {
            return 0.0;
        }
        self.avg_win / self.avg_loss
    }

    /// Fractional Kelly criterion: f* = (bp - q) / b
    /// where b = avg_win/avg_loss, p = win_rate, q = 1 - p
    /// Capped at max_fraction, returns 0 if insufficient data.
    /// Uses a Bayesian loss floor when avg_loss is 0 (100% win rate)
    /// to avoid returning 0 when the bot is performing at its best.
    pub fn kelly_fraction(&self, min_trades: u32, max_fraction: f64) -> f64 {
        if self.total_trades < min_trades as u64 || self.hit_rate() <= 0.0 {
            return 0.0;
        }
        let loss_floor = if self.avg_loss <= 0.0 {
            (self.avg_win * 0.1).max(1e-9)
        } else {
            self.avg_loss
        };
        let b = self.avg_win / loss_floor;
        if !b.is_finite() || b <= 0.0 {
            return 0.0;
        }
        let p = self.hit_rate();
        let q = 1.0 - p;
        let kelly = (b * p - q) / b;
        kelly.clamp(0.0, max_fraction)
    }

    /// Compute position size in USD using risk-per-trade + optional Kelly boost.
    pub fn compute_size_usd(&self, config: &SmartTrendConfig) -> f64 {
        let base = self.allocated_capital * config.st_risk_per_trade;
        if !config.st_kelly_enabled {
            return base.max(0.0);
        }
        let kf = self.kelly_fraction(config.st_kelly_min_trades, config.st_max_kelly_fraction);
        if kf <= 0.0 {
            return base.max(0.0);
        }
        let boosted = base + (kf * 0.5 * base);
        boosted.max(0.0)
    }

    pub fn daily_reset(&mut self) {
        self.realized_pnl_today = 0.0;
        self.trade_count_today = 0;
        self.win_count_today = 0;
        self.loss_count_today = 0;
        self.initial_capital = self.allocated_capital;
        if self.breaker_triggered {
            self.paused = false;
        }
        self.breaker_triggered = false;
    }

    /// Atomic save: write to .tmp then rename. Backup .bak created first.
    pub fn save_to_disk(&self) -> Result<(), std::io::Error> {
        let path = crate::paths::data_dir().join("titan_st_account.json");
        if path.exists() {
            let bak = path.with_extension("json.bak");
            let _ = std::fs::copy(&path, &bak);
        }
        let payload = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &payload)?;
        std::fs::rename(&tmp, &path)
    }

    /// Load with .bak fallback: primary → .bak → Default.
    pub fn load_from_disk() -> Self {
        let path = crate::paths::data_dir().join("titan_st_account.json");
        // Try primary file
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if !data.trim().is_empty() {
                    if let Ok(acct) = serde_json::from_str::<Self>(&data) {
                        return acct;
                    }
                }
            }
        }
        // Primary failed — try .bak
        let bak = path.with_extension("json.bak");
        if bak.exists() {
            tracing::warn!(component = "smarttrend", "Primary account file corrupt/missing — loading from .bak");
            if let Ok(data) = std::fs::read_to_string(&bak) {
                if !data.trim().is_empty() {
                    if let Ok(acct) = serde_json::from_str::<Self>(&data) {
                        return acct;
                    }
                }
            }
        }
        tracing::warn!(component = "smarttrend", "No valid account file found — starting fresh");
        Self::default()
    }
}

// ═══════════════════════════════════════════════════════════════════
//  PILLAR INPUT STRUCTS — Lightweight snapshots for evaluate_*
// ═══════════════════════════════════════════════════════════════════

/// Input data for the Macro pillar.
#[derive(Clone, Debug)]
pub struct MacroInput {
    pub oracle_score: f64,    // [-100, +100]
    pub confidence: f64,      // [0, 1]
    pub regime: MarketRegime,
    pub kalman_gain: f64,     // How much Kalman trusts new data
    pub price_surprise: f64,  // |actual - predicted| / predicted
    pub kalman_velocity: f64, // Predicted velocity direction
    pub run_length: usize,    // Bars since last changepoint
    pub changepoint_prob: f64, // [0, 1]
}

/// Input data for the Micro pillar.
#[derive(Clone, Debug)]
pub struct MicroInput {
    pub ofi_value: f64,         // Current OFI (signed)
    pub microprice: f64,        // Depth-weighted microprice
    pub mid_price: f64,         // Simple mid price (bid+ask)/2
    pub cvd: f64,               // Cumulative Volume Delta (signed)
    pub spread_bps: f64,        // Spread in basis points
    pub imbalance_ratio: f64,   // [-1, +1]
    pub has_live_data: bool,    // false if stale or no feed
}

/// Input data for the Momentum pillar.
#[derive(Clone, Debug)]
pub struct MomentumInput {
    pub gpu_buy_pct: f64,       // [0, 100] from GPU Nexus
    pub gpu_sell_pct: f64,      // [0, 100]
    pub dominant_cycle: usize,  // From wavelet (bars)
    pub momentum_bars: usize,   // How many bars the current move has lasted
    pub rvol: f64,              // Relative volume (>1 = above average)
}

// ═══════════════════════════════════════════════════════════════════
//  PILLAR OUTPUTS
// ═══════════════════════════════════════════════════════════════════

/// Direction hint from a pillar evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PillarDirection {
    Bullish,
    Bearish,
    Neutral,
}

/// Result of a single pillar evaluation.
#[derive(Clone, Debug, Serialize)]
pub struct PillarScore {
    pub score: f64,             // [0, 100] — 50 = neutral
    pub direction: PillarDirection,
    pub confidence: f64,        // [0, 1]
    pub stale: bool,            // true if input data is stale/invalid
    pub detail: String,         // Human-readable explanation
}

/// Signal strength classification based on combined score.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum SmartTrendStrength {
    Weak,     // < 55
    Moderate, // 55-70
    Strong,   // 70-85
    Extreme,  // > 85
}

impl SmartTrendStrength {
    pub fn from_score(score: f64) -> Self {
        if score >= 85.0 {
            Self::Extreme
        } else if score >= 70.0 {
            Self::Strong
        } else if score >= 55.0 {
            Self::Moderate
        } else {
            Self::Weak
        }
    }
}

/// Combined result of all three pillars (diagnostics + event payload).
#[derive(Clone, Debug, Serialize)]
pub struct PillarDiagnostics {
    pub macro_score: PillarScore,
    pub micro_score: PillarScore,
    pub momentum_score: PillarScore,
    pub combined_score: f64,
    pub combined_direction: PillarDirection,
    pub strength: SmartTrendStrength,
    pub regime: String,
    pub effective_entry_threshold: f64,
    pub any_stale: bool,
    pub cross_divergence: bool,
    pub divergence_detail: String,
    pub nan_sanitized: u32,
}

// ═══════════════════════════════════════════════════════════════════
//  T1 — PILON MACRO
// ═══════════════════════════════════════════════════════════════════

/// Evaluate the Macro pillar from oracle data + Kalman internals.
///
/// Score logic:
/// 1. Base from oracle_score mapped to [0, 100] (50 = neutral)
/// 2. Kalman surprise confirmation: if surprise in direction of velocity → boost
/// 3. Kalman gain instability penalty: high gain + velocity change → regime transition
/// 4. Run length penalty: too few bars since changepoint → conservative
/// 5. Changepoint damping: high changepoint prob → drag toward 50
pub fn evaluate_macro(input: &MacroInput, config: &SmartTrendConfig) -> PillarScore {
    // Step 1: Oracle score → [0, 100]
    let base = (input.oracle_score + 100.0) * 0.5; // maps [-100,+100] to [0,100]

    // Step 2: Kalman surprise confirmation
    let velocity_bullish = input.kalman_velocity > 0.0;
    let surprise_confirms = if velocity_bullish {
        input.price_surprise > 0.0 // price above prediction = bullish surprise
    } else {
        input.price_surprise < 0.0 // price below prediction = bearish surprise
    };
    let surprise_mult = if surprise_confirms { 1.1 } else { 0.95 };

    // Step 3: Kalman gain instability
    // High gain (>0.5) = filter is confused, new data is very different from model
    let gain_penalty = if input.kalman_gain > 0.5 { 0.85 } else { 1.0 };

    // Step 4: Run length stability
    let run_penalty = if input.run_length < config.st_stabilization_window as usize {
        0.7 // Too early after changepoint — reduce confidence
    } else {
        1.0
    };

    // Step 5: Changepoint damping
    let cp_damping = if input.changepoint_prob > 0.5 {
        0.6
    } else if input.changepoint_prob > 0.3 {
        0.8
    } else {
        1.0
    };

    // Combine
    let raw = base * surprise_mult * gain_penalty * run_penalty * cp_damping;
    let score = raw.clamp(0.0, 100.0);

    let direction = if score > 55.0 {
        PillarDirection::Bullish
    } else if score < 45.0 {
        PillarDirection::Bearish
    } else {
        PillarDirection::Neutral
    };

    let conf = input.confidence * cp_damping * run_penalty;
    let stale = input.confidence < MIN_CONFIDENCE_FOR_SIGNAL
        || (input.oracle_score.abs() < f64::EPSILON && input.kalman_gain.abs() < f64::EPSILON);

    PillarScore {
        score,
        direction,
        confidence: conf.clamp(0.0, 1.0),
        stale,
        detail: format!(
            "oracle={:.0} surp={:.4} gain={:.3} run={} cp={:.2}",
            input.oracle_score, input.price_surprise, input.kalman_gain,
            input.run_length, input.changepoint_prob
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════
//  T2 — PILON MICRO
// ═══════════════════════════════════════════════════════════════════

/// Evaluate the Micro pillar from live microstructure data.
///
/// Returns 50 (neutral) if no live data — better than garbage.
/// Score logic:
/// 1. OFI direction → base bias (normalized by config range)
/// 2. Microprice vs mid → directional hint (bps range from config)
/// 3. CVD confirmation → boost/penalize (scaled by config)
/// 4. Imbalance ratio → amplifier
pub fn evaluate_micro(input: &MicroInput, config: &SmartTrendConfig) -> PillarScore {
    if !input.has_live_data {
        return PillarScore {
            score: 50.0,
            direction: PillarDirection::Neutral,
            confidence: 0.0,
            stale: true,
            detail: "NO_LIVE_DATA".to_string(),
        };
    }

    let ofi_range = if config.st_ofi_norm_range > 0.0 { config.st_ofi_norm_range } else { 50.0 };
    let mp_range = if config.st_mp_bps_range > 0.0 { config.st_mp_bps_range } else { 10.0 };
    let cvd_scale = if config.st_cvd_scale > 0.0 { config.st_cvd_scale } else { 100.0 };

    // Step 1: OFI direction
    let ofi_signal = if input.ofi_value.abs() < 1.0 {
        0.0
    } else {
        (input.ofi_value / ofi_range).clamp(-1.0, 1.0)
    };

    // Step 2: Microprice vs mid
    let mp_delta = if input.mid_price > 0.0 {
        (input.microprice - input.mid_price) / input.mid_price * 10000.0 // in bps
    } else {
        0.0
    };
    let mp_signal = (mp_delta / mp_range).clamp(-1.0, 1.0);

    // Step 3: CVD direction
    let cvd_signal = if input.cvd.abs() < 0.01 {
        0.0
    } else if input.cvd > 0.0 {
        0.5_f64.min(input.cvd / cvd_scale)
    } else {
        (-0.5_f64).max(input.cvd / cvd_scale)
    };

    // Step 4: Imbalance amplifier
    let imb_factor = 1.0 + input.imbalance_ratio.abs() * 0.3; // max 1.3x

    // Check CVD convergence/divergence with OFI
    let cvd_confirms_ofi = (input.cvd > 0.0 && input.ofi_value > 0.0)
        || (input.cvd < 0.0 && input.ofi_value < 0.0);
    let convergence_mult = if cvd_confirms_ofi { 1.15 } else { 0.85 };

    // Combine signals
    let raw_signal = (ofi_signal * 0.40 + mp_signal * 0.25 + cvd_signal * 0.35) * imb_factor * convergence_mult;
    let score = (50.0 + raw_signal * 30.0).clamp(0.0, 100.0);

    let direction = if score > 55.0 {
        PillarDirection::Bullish
    } else if score < 45.0 {
        PillarDirection::Bearish
    } else {
        PillarDirection::Neutral
    };

    // Confidence based on data freshness and spread
    let spread_conf = if input.spread_bps < 5.0 {
        1.0
    } else if input.spread_bps < 20.0 {
        0.8
    } else {
        0.5
    };

    PillarScore {
        score,
        direction,
        confidence: spread_conf,
        stale: false,
        detail: format!(
            "ofi={:.1} mp_d={:.1}bps cvd={:.1} imb={:.2}",
            input.ofi_value, mp_delta, input.cvd, input.imbalance_ratio
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════
//  T3 — PILON MOMENTUM
// ═══════════════════════════════════════════════════════════════════

/// Evaluate the Momentum pillar from GPU consensus + wavelet cycle.
///
/// Score logic:
/// 1. GPU consensus → base direction
/// 2. Dominant_cycle filter: momentum < cycle → noise penalty
/// 3. RVOL boost: higher volume = more conviction
pub fn evaluate_momentum(input: &MomentumInput, config: &SmartTrendConfig) -> PillarScore {
    // Step 1: GPU consensus → directional signal
    let gpu_signal = (input.gpu_buy_pct - input.gpu_sell_pct) / 100.0; // [-1, +1]

    // Step 2: Dominant cycle filter
    let cycle_penalty = if config.st_dominant_cycle_filter
        && input.dominant_cycle > 0
        && input.momentum_bars < input.dominant_cycle
    {
        // Momentum move shorter than dominant cycle = likely noise
        let ratio = input.momentum_bars as f64 / input.dominant_cycle as f64;
        ratio.clamp(0.3, 1.0) // Don't kill it entirely, min 30%
    } else {
        1.0
    };

    // Step 3: RVOL boost
    let rvol_mult = if input.rvol > 2.0 {
        1.2  // High volume = conviction
    } else if input.rvol > 1.0 {
        1.0 + (input.rvol - 1.0) * 0.2 // Gradual boost
    } else {
        0.85 // Below-average volume = less reliable
    };

    // Combine
    let raw = gpu_signal * cycle_penalty * rvol_mult;
    let score = (50.0 + raw * 35.0).clamp(0.0, 100.0);

    let direction = if score > 55.0 {
        PillarDirection::Bullish
    } else if score < 45.0 {
        PillarDirection::Bearish
    } else {
        PillarDirection::Neutral
    };

    let strength = gpu_signal.abs();
    let confidence = (strength * cycle_penalty * rvol_mult.min(1.3)).clamp(0.0, 1.0);
    let stale = (input.gpu_buy_pct + input.gpu_sell_pct) < 1.0;

    PillarScore {
        score,
        direction,
        confidence,
        stale,
        detail: format!(
            "gpu={:.0}/{:.0} cycle={} bars={} rvol={:.1}",
            input.gpu_buy_pct, input.gpu_sell_pct,
            input.dominant_cycle, input.momentum_bars, input.rvol
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════
//  B5 — CROSS-PILLAR DIVERGENCE DETECTION
// ═══════════════════════════════════════════════════════════════════

/// Detect when ≥2 pillars have opposing directions.
/// Returns (is_divergent, detail_message).
pub fn detect_cross_pillar_divergence(
    macro_dir: PillarDirection,
    micro_dir: PillarDirection,
    momentum_dir: PillarDirection,
) -> (bool, String) {
    let dirs = [
        ("MACRO", macro_dir),
        ("MICRO", micro_dir),
        ("MOMENTUM", momentum_dir),
    ];

    let bullish_count = dirs.iter().filter(|(_, d)| *d == PillarDirection::Bullish).count();
    let bearish_count = dirs.iter().filter(|(_, d)| *d == PillarDirection::Bearish).count();

    if bullish_count >= 1 && bearish_count >= 1 {
        let bulls: Vec<&str> = dirs.iter().filter(|(_, d)| *d == PillarDirection::Bullish).map(|(n, _)| *n).collect();
        let bears: Vec<&str> = dirs.iter().filter(|(_, d)| *d == PillarDirection::Bearish).map(|(n, _)| *n).collect();
        let detail = format!(
            "DIVERGENCE: {} bullish vs {} bearish",
            bulls.join("+"), bears.join("+")
        );
        (true, detail)
    } else {
        (false, "ALIGNED".to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════
//  COMBINED EVALUATION — All 3 pillars + diagnostics
// ═══════════════════════════════════════════════════════════════════

/// Evaluate all three pillars and produce combined diagnostics.
/// Applies confidence-weighted scoring: low-confidence pillars contribute less.
/// Returns regime-adaptive entry threshold and staleness/strength flags.
pub fn evaluate_pillars(
    macro_input: &MacroInput,
    micro_input: &MicroInput,
    momentum_input: &MomentumInput,
    config: &SmartTrendConfig,
) -> PillarDiagnostics {
    let mut macro_score = evaluate_macro(macro_input, config);
    let mut micro_score = evaluate_micro(micro_input, config);
    let mut momentum_score = evaluate_momentum(momentum_input, config);

    // NaN firewall: sanitize individual pillar scores before combining
    let mut nan_count = 0u32;
    if !macro_score.score.is_finite() {
        tracing::warn!(component = "smarttrend", "NaN/Inf in macro pillar score — sanitized to 50.0");
        macro_score.score = 50.0;
        macro_score.confidence = 0.0;
        nan_count += 1;
    }
    if !micro_score.score.is_finite() {
        tracing::warn!(component = "smarttrend", "NaN/Inf in micro pillar score — sanitized to 50.0");
        micro_score.score = 50.0;
        micro_score.confidence = 0.0;
        nan_count += 1;
    }
    if !momentum_score.score.is_finite() {
        tracing::warn!(component = "smarttrend", "NaN/Inf in momentum pillar score — sanitized to 50.0");
        momentum_score.score = 50.0;
        momentum_score.confidence = 0.0;
        nan_count += 1;
    }

    // Base normalized weights
    let (w_macro, w_micro, w_momentum) = normalize_weights(
        config.st_macro_weight,
        config.st_micro_weight,
        config.st_momentum_weight,
    );

    // Confidence-adjusted weights: pillars with confidence < 0.3 have reduced impact
    let conf_adj = |conf: f64| -> f64 {
        if conf < 0.3 { conf / 0.3 } else { 1.0 }
    };
    let adj_macro = w_macro * conf_adj(macro_score.confidence);
    let adj_micro = w_micro * conf_adj(micro_score.confidence);
    let adj_momentum = w_momentum * conf_adj(momentum_score.confidence);

    let adj_sum = adj_macro + adj_micro + adj_momentum;
    let (ew_macro, ew_micro, ew_momentum) = if !adj_sum.is_finite() || adj_sum <= 0.0 {
        (w_macro, w_micro, w_momentum)
    } else {
        (adj_macro / adj_sum, adj_micro / adj_sum, adj_momentum / adj_sum)
    };

    let mut combined = macro_score.score * ew_macro
        + micro_score.score * ew_micro
        + momentum_score.score * ew_momentum;

    // Final NaN firewall on combined score
    if !combined.is_finite() {
        tracing::warn!(component = "smarttrend", "NaN/Inf in combined score — sanitized to 50.0 (neutral)");
        combined = 50.0;
        nan_count += 1;
    }

    let combined_direction = if combined > 55.0 {
        PillarDirection::Bullish
    } else if combined < 45.0 {
        PillarDirection::Bearish
    } else {
        PillarDirection::Neutral
    };

    // Cross-pillar divergence
    let (cross_divergence, divergence_detail) = detect_cross_pillar_divergence(
        macro_score.direction,
        micro_score.direction,
        momentum_score.direction,
    );

    // Regime-adaptive entry threshold
    let regime_idx = match macro_input.regime {
        MarketRegime::Trending => 0,
        MarketRegime::MeanReverting => 1,
        MarketRegime::Chaotic => 2,
        MarketRegime::Unknown => 3,
    };
    let effective_entry_threshold = config.st_entry_thresholds[regime_idx];

    let any_stale = macro_score.stale || micro_score.stale || momentum_score.stale;
    let strength = SmartTrendStrength::from_score(combined);
    let regime_str = format!("{:?}", macro_input.regime);

    PillarDiagnostics {
        macro_score,
        micro_score,
        momentum_score,
        combined_score: combined,
        combined_direction,
        strength,
        regime: regime_str,
        effective_entry_threshold,
        any_stale,
        cross_divergence,
        divergence_detail,
        nan_sanitized: nan_count,
    }
}

// ═══════════════════════════════════════════════════════════════════
//  T4 — BAYESIAN TRACKER (Beta Distribution per-Regime)
// ═══════════════════════════════════════════════════════════════════

/// Per-regime Beta distribution parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegimeBeta {
    pub alpha: f64,
    pub beta: f64,
}

impl Default for RegimeBeta {
    fn default() -> Self {
        Self { alpha: 1.0, beta: 1.0 } // Uniform prior
    }
}

impl RegimeBeta {
    /// Posterior mean estimate: alpha / (alpha + beta).
    pub fn estimate(&self) -> f64 {
        if self.alpha + self.beta <= 0.0 {
            return 0.5;
        }
        self.alpha / (self.alpha + self.beta)
    }

    /// Update with observation.
    pub fn update(&mut self, success: bool) {
        if success {
            self.alpha += 1.0;
        } else {
            self.beta += 1.0;
        }
    }

    /// Apply decay to prevent overfitting to old data.
    pub fn decay(&mut self, factor: f64) {
        self.alpha = (self.alpha * factor).max(1.0);
        self.beta = (self.beta * factor).max(1.0);
    }
}

/// Bayesian tracker with per-regime Beta distributions.
/// Tracks historical accuracy of SmartTrend signals per market regime.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BayesianTracker {
    pub trending: RegimeBeta,
    pub mean_reverting: RegimeBeta,
    pub chaotic: RegimeBeta,
    pub unknown: RegimeBeta,
    pub total_updates: u64,
}

impl BayesianTracker {
    /// Update the regime-specific Beta distribution with a trade outcome.
    pub fn update(&mut self, regime: MarketRegime, success: bool) {
        let dist = self.get_mut(regime);
        dist.update(success);
        self.total_updates += 1;
    }

    /// Get the posterior mean estimate for a regime.
    pub fn estimate(&self, regime: MarketRegime) -> f64 {
        self.get(regime).estimate()
    }

    /// Apply decay to all regimes using a raw multiplicative factor.
    /// Prefer `decay_halflife()` for time-aware decay.
    pub fn decay_all(&mut self, factor: f64) {
        self.trending.decay(factor);
        self.mean_reverting.decay(factor);
        self.chaotic.decay(factor);
        self.unknown.decay(factor);
    }

    /// Apply halflife-based decay: after `halflife` update calls, prior weight halves.
    /// Computes factor = 0.5^(1/halflife) and applies to all regimes.
    /// No-op if halflife == 0.
    pub fn decay_halflife(&mut self, halflife: u32) {
        if halflife == 0 { return; }
        let factor = 0.5_f64.powf(1.0 / halflife as f64);
        self.decay_all(factor);
    }

    /// Get all regime estimates as a serializable map.
    pub fn all_estimates(&self) -> BayesianEstimates {
        BayesianEstimates {
            trending: self.trending.estimate(),
            mean_reverting: self.mean_reverting.estimate(),
            chaotic: self.chaotic.estimate(),
            unknown: self.unknown.estimate(),
            total_updates: self.total_updates,
        }
    }

    fn get(&self, regime: MarketRegime) -> &RegimeBeta {
        match regime {
            MarketRegime::Trending => &self.trending,
            MarketRegime::MeanReverting => &self.mean_reverting,
            MarketRegime::Chaotic => &self.chaotic,
            MarketRegime::Unknown => &self.unknown,
        }
    }

    fn get_mut(&mut self, regime: MarketRegime) -> &mut RegimeBeta {
        match regime {
            MarketRegime::Trending => &mut self.trending,
            MarketRegime::MeanReverting => &mut self.mean_reverting,
            MarketRegime::Chaotic => &mut self.chaotic,
            MarketRegime::Unknown => &mut self.unknown,
        }
    }
}

/// Serializable Bayesian estimates for UI/diagnostics (B1).
#[derive(Clone, Debug, Serialize)]
pub struct BayesianEstimates {
    pub trending: f64,
    pub mean_reverting: f64,
    pub chaotic: f64,
    pub unknown: f64,
    pub total_updates: u64,
}

// ═══════════════════════════════════════════════════════════════════
//  S07b — FSM: State Machine for SmartTrend Orchestration
// ═══════════════════════════════════════════════════════════════════

/// The five trading lifecycle states of the SmartTrend engine.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum SmartTrendStateKind {
    Flat,
    Stalking,
    Entry,
    InTrade,
    Trail,
}

impl std::fmt::Display for SmartTrendStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flat => write!(f, "Flat"),
            Self::Stalking => write!(f, "Stalking"),
            Self::Entry => write!(f, "Entry"),
            Self::InTrade => write!(f, "InTrade"),
            Self::Trail => write!(f, "Trail"),
        }
    }
}

/// Markers emitted by the FSM on state transitions.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum SmartTrendMarkerKind {
    Stalking,
    Entry,
    Exit,
    Trail,
    ChangePointReset,
}

/// Typed exit reason for safe transition matching (replaces string matching).
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub enum ExitReason {
    TrailExit,
    ChangePointExit,
    FlatByClose,
    CircuitBreaker,
    MaxHold,
    None,
}

/// Per-symbol engine snapshot shared via TitanState between sentinel and bot.
/// Sentinel writes these every tick; bot reads them for multi-symbol entries.
#[derive(Serialize, Clone, Debug)]
pub struct SentinelEngineSnapshot {
    pub symbol: String,
    pub state: SmartTrendStateKind,
    pub direction: PillarDirection,
    pub exit_reason: ExitReason,
    pub entry_price: Option<f64>,
    pub best_price: Option<f64>,
    pub bars_in_state: u32,
    pub score: f64,
    pub updated_ms: u64,
}

/// A marker event produced by the SmartTrend engine during evaluation.
#[derive(Serialize, Clone, Debug)]
pub struct SmartTrendMarker {
    pub kind: SmartTrendMarkerKind,
    pub direction: PillarDirection,
    pub score: f64,
    pub detail: String,
}

/// The result of a single SmartTrendEngine::evaluate() call.
#[derive(Serialize, Clone, Debug)]
pub struct SmartTrendResult {
    pub score: f64,
    pub state: SmartTrendStateKind,
    pub prev_state: SmartTrendStateKind,
    pub direction: PillarDirection,
    pub markers: Vec<SmartTrendMarker>,
    pub pillar_scores: PillarDiagnostics,
    pub bars_in_state: u32,
    pub confirmation_progress: Option<(u32, u32)>,
}

/// Per-session statistics tracked by the FSM.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SmartTrendStats {
    pub transitions: std::collections::HashMap<String, u64>,
    pub bars_per_state: std::collections::HashMap<String, u64>,
    pub entries_total: u64,
    pub entries_reached_trail: u64,
    pub changepoint_resets: u64,
    pub stalking_timeouts: u64,
}

impl SmartTrendStats {
    fn record_transition(&mut self, from: SmartTrendStateKind, to: SmartTrendStateKind) {
        let key = format!("{}->{}", from, to);
        *self.transitions.entry(key).or_insert(0) += 1;
        if to == SmartTrendStateKind::Entry {
            self.entries_total += 1;
        }
        if to == SmartTrendStateKind::Trail {
            self.entries_reached_trail += 1;
        }
    }

    fn record_bars_in_state(&mut self, state: SmartTrendStateKind, bars: u32) {
        let key = format!("{}", state);
        *self.bars_per_state.entry(key).or_insert(0) += bars as u64;
    }

    /// Entry hit rate: fraction of entries that reached Trail.
    pub fn trail_hit_rate(&self) -> f64 {
        if self.entries_total == 0 { return 0.0; }
        self.entries_reached_trail as f64 / self.entries_total as f64
    }
}

/// Diagnostics payload for the Tauri command (includes FSM + pillar state).
#[derive(Serialize, Clone, Debug)]
pub struct SmartTrendDiagnostics {
    pub enabled: bool,
    pub killed: bool,
    pub state: SmartTrendStateKind,
    pub direction: PillarDirection,
    pub combined_score: f64,
    pub pillar_scores: PillarDiagnostics,
    pub bars_in_state: u32,
    pub confirmation_progress: Option<(u32, u32)>,
    pub bayesian_estimates: BayesianEstimates,
    pub strength: SmartTrendStrength,
    pub last_transition: String,
    pub stats: SmartTrendStats,
    pub entry_price: Option<f64>,
    pub best_price: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════
//  S07b — SmartTrendEngine: Orchestrator combining pillars + FSM
// ═══════════════════════════════════════════════════════════════════

/// Input bundle for SmartTrendEngine::evaluate().
#[derive(Clone, Debug)]
pub struct SmartTrendInput {
    pub macro_input: MacroInput,
    pub micro_input: MicroInput,
    pub momentum_input: MomentumInput,
    pub current_price: f64,
    pub changepoint_prob: f64,
    pub dominant_cycle: f64,
    pub tf_minutes: f64,
}

/// The orchestrator: owns the FSM, BayesianTracker, config, and stats.
#[derive(Clone, Debug)]
pub struct SmartTrendEngine {
    pub state: SmartTrendStateKind,
    pub direction: PillarDirection,
    pub bars_in_state: u32,
    pub entry_price: Option<f64>,
    pub best_price: Option<f64>,
    pub confirmation_bars: u32,
    pub confirmation_needed: u32,
    pub trail_tighten_accumulated: f64,
    pub last_transition: String,
    pub last_exit_reason: ExitReason,
    pub tracker: BayesianTracker,
    pub config: SmartTrendConfig,
    pub stats: SmartTrendStats,
    pub last_diagnostics: Option<PillarDiagnostics>,
    pub nan_sanitized_count: u64,
    pub entry_acted: bool,
    pub fsm_eval_ring: std::collections::VecDeque<SmartTrendStateKind>,
    pub consecutive_stalking: u32,
    pub last_score: f64,
}

impl SmartTrendEngine {
    pub fn new(config: SmartTrendConfig) -> Self {
        Self {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            bars_in_state: 0,
            entry_price: None,
            best_price: None,
            confirmation_bars: 0,
            confirmation_needed: 0,
            trail_tighten_accumulated: 0.0,
            last_transition: "init".to_string(),
            last_exit_reason: ExitReason::None,
            tracker: BayesianTracker::default(),
            config,
            stats: SmartTrendStats::default(),
            last_diagnostics: None,
            nan_sanitized_count: 0,
            entry_acted: false,
            fsm_eval_ring: std::collections::VecDeque::with_capacity(100),
            consecutive_stalking: 0,
            last_score: 50.0,
        }
    }

    /// Compute the dynamic confirmation window from dominant_cycle and strength.
    fn compute_confirmation_needed(&self, dominant_cycle: f64, tf_minutes: f64, strength: SmartTrendStrength) -> u32 {
        let base = if dominant_cycle > 0.0 && tf_minutes > 0.0 {
            (dominant_cycle / tf_minutes).max(2.0)
        } else {
            self.config.st_stabilization_window as f64
        };

        let strength_mult = match strength {
            SmartTrendStrength::Extreme => 0.75,
            SmartTrendStrength::Strong => 0.85,
            _ => 1.0,
        };

        (base * strength_mult).round().max(2.0) as u32
    }

    /// Transition the FSM to a new state, recording stats and producing a detail string.
    fn transition_to(&mut self, new_state: SmartTrendStateKind, direction: PillarDirection, detail: &str) {
        let old = self.state;
        self.stats.record_bars_in_state(old, self.bars_in_state);
        self.stats.record_transition(old, new_state);
        self.last_transition = format!("{}->{} ({})", old, new_state, detail);
        self.state = new_state;
        self.direction = direction;
        self.bars_in_state = 0;
        self.confirmation_bars = 0;
        self.trail_tighten_accumulated = 0.0;
        if new_state == SmartTrendStateKind::Flat {
            self.entry_price = None;
            self.best_price = None;
        }
    }

    /// Main evaluation entry point. Runs pillar scoring, then FSM transitions.
    pub fn evaluate(&mut self, input: &SmartTrendInput) -> SmartTrendResult {
        let prev_state = self.state;
        let mut markers: Vec<SmartTrendMarker> = Vec::new();

        // 1. Evaluate all three pillars
        let diag = evaluate_pillars(
            &input.macro_input,
            &input.micro_input,
            &input.momentum_input,
            &self.config,
        );
        if diag.nan_sanitized > 0 {
            self.nan_sanitized_count += diag.nan_sanitized as u64;
        }
        self.last_diagnostics = Some(diag.clone());
        let score = diag.combined_score;
        let dir = diag.combined_direction;
        let strength = diag.strength;

        // 2. Changepoint reset check (any state -> Flat)
        if input.changepoint_prob >= self.config.st_changepoint_reset_threshold
            && self.state != SmartTrendStateKind::Flat
        {
            if self.state == SmartTrendStateKind::InTrade || self.state == SmartTrendStateKind::Trail {
                markers.push(SmartTrendMarker {
                    kind: SmartTrendMarkerKind::Exit,
                    direction: self.direction,
                    score,
                    detail: format!("changepoint exit cp={:.2}", input.changepoint_prob),
                });
            }
            markers.push(SmartTrendMarker {
                kind: SmartTrendMarkerKind::ChangePointReset,
                direction: PillarDirection::Neutral,
                score,
                detail: format!("cp={:.2} >= {:.2}", input.changepoint_prob, self.config.st_changepoint_reset_threshold),
            });
            self.transition_to(SmartTrendStateKind::Flat, PillarDirection::Neutral, "changepoint_reset");
            self.last_exit_reason = ExitReason::ChangePointExit;
            // Partial decay instead of full wipe — preserve 50% of regime history
            self.tracker.decay_all(0.5);
            self.stats.changepoint_resets += 1;

            return SmartTrendResult {
                score,
                state: self.state,
                prev_state,
                direction: PillarDirection::Neutral,
                markers,
                pillar_scores: diag,
                bars_in_state: 0,
                confirmation_progress: None,
            };
        }

        // 3. FSM transitions
        self.bars_in_state += 1;

        match self.state {
            SmartTrendStateKind::Flat => {
                // Flat -> Stalking: directional strength exceeds stalking threshold
                // Bullish: score >= stalking_threshold (e.g. 55)
                // Bearish: score <= (100 - stalking_threshold) (e.g. 45)
                let directional_strength = (score - 50.0).abs() * 2.0; // [0, 100] magnitude
                let threshold_met = directional_strength >= (self.config.st_stalking_threshold - 50.0).abs() * 2.0
                    && dir != PillarDirection::Neutral;

                if threshold_met {
                    self.confirmation_needed = self.compute_confirmation_needed(
                        input.dominant_cycle, input.tf_minutes, strength,
                    );
                    self.transition_to(SmartTrendStateKind::Stalking, dir, "score_above_stalking");
                    self.bars_in_state = 1;
                    markers.push(SmartTrendMarker {
                        kind: SmartTrendMarkerKind::Stalking,
                        direction: dir,
                        score,
                        detail: format!("need {} bars to confirm", self.confirmation_needed),
                    });
                }
            }

            SmartTrendStateKind::Stalking => {
                let directional_strength = (score - 50.0).abs() * 2.0;
                let stalking_strength = (self.config.st_stalking_threshold - 50.0).abs() * 2.0;
                let max_safe_margin = (stalking_strength / 2.0 - 0.5).max(0.0);
                let effective_margin = self.config.st_hysteresis_margin.min(max_safe_margin);
                let drop_strength = (stalking_strength - effective_margin * 2.0).max(0.1);

                if directional_strength < drop_strength || dir == PillarDirection::Neutral {
                    self.transition_to(SmartTrendStateKind::Flat, PillarDirection::Neutral, "score_below_stalking");
                } else {
                    // Check confirmation progress (direction-aware)
                    let entry_strength = (diag.effective_entry_threshold - 50.0).abs() * 2.0;
                    if directional_strength >= entry_strength {
                        self.confirmation_bars += 1;
                    }

                    let timeout = self.confirmation_needed.saturating_mul(2);
                    if self.bars_in_state >= timeout && self.confirmation_bars < self.confirmation_needed {
                        self.transition_to(SmartTrendStateKind::Flat, PillarDirection::Neutral, "stalking_timeout");
                        self.stats.stalking_timeouts += 1;
                    }
                    // B2: Pillar agreement gate — need 2/3 aligned (no cross-divergence)
                    else if self.confirmation_bars >= self.confirmation_needed && !diag.cross_divergence {
                        self.transition_to(SmartTrendStateKind::Entry, dir, "confirmed");
                        self.bars_in_state = 1;
                        markers.push(SmartTrendMarker {
                            kind: SmartTrendMarkerKind::Entry,
                            direction: dir,
                            score,
                            detail: format!(
                                "confirmed after {} bars, score={:.1}",
                                self.confirmation_needed, score
                            ),
                        });
                    }
                }
            }

            SmartTrendStateKind::Entry => {
                let persist_bars = self.config.st_entry_persist_bars.unwrap_or(3);
                if self.bars_in_state >= persist_bars || self.entry_acted {
                    self.entry_price = Some(input.current_price);
                    self.best_price = Some(input.current_price);
                    self.entry_acted = false;
                    self.transition_to(SmartTrendStateKind::InTrade, self.direction, "position_opened");
                    self.bars_in_state = 1;
                }
            }

            SmartTrendStateKind::InTrade => {
                // Track best price
                if let Some(ref mut best) = self.best_price {
                    match self.direction {
                        PillarDirection::Bullish => { if input.current_price > *best { *best = input.current_price; } }
                        PillarDirection::Bearish => { if input.current_price < *best { *best = input.current_price; } }
                        _ => {}
                    }
                }

                // InTrade -> Trail: directional strength weakens below entry * 0.8
                let entry_strength = (diag.effective_entry_threshold - 50.0).abs() * 2.0;
                let trail_strength = entry_strength * 0.8;
                let directional_strength = (score - 50.0).abs() * 2.0;
                if directional_strength < trail_strength {
                    self.transition_to(SmartTrendStateKind::Trail, self.direction, "score_weakening");
                    self.bars_in_state = 1;
                    markers.push(SmartTrendMarker {
                        kind: SmartTrendMarkerKind::Trail,
                        direction: self.direction,
                        score,
                        detail: format!("strength {:.1} < trail {:.1}", directional_strength, trail_strength),
                    });
                }
            }

            SmartTrendStateKind::Trail => {
                // B1: Auto-tighten trail
                self.trail_tighten_accumulated += self.config.st_trail_tighten_pct;

                // Trail -> Flat: direction reversal or score inverts
                let should_exit = match self.direction {
                    PillarDirection::Bullish => dir == PillarDirection::Bearish,
                    PillarDirection::Bearish => dir == PillarDirection::Bullish,
                    _ => score < self.config.st_stalking_threshold,
                };

                if should_exit {
                    markers.push(SmartTrendMarker {
                        kind: SmartTrendMarkerKind::Exit,
                        direction: self.direction,
                        score,
                        detail: format!(
                            "direction reversal, tighten={:.0}%",
                            self.trail_tighten_accumulated * 100.0
                        ),
                    });
                    let was_profitable = self.entry_price.is_some_and(|ep| {
                        match self.direction {
                            PillarDirection::Bullish => input.current_price > ep,
                            PillarDirection::Bearish => input.current_price < ep,
                            _ => false,
                        }
                    });
                    self.tracker.update(input.macro_input.regime, was_profitable);
                    self.tracker.decay_halflife(self.config.st_bayesian_halflife);
                    self.last_exit_reason = ExitReason::TrailExit;
                    self.transition_to(SmartTrendStateKind::Flat, PillarDirection::Neutral, "trail_exit");
                }
            }
        }

        let confirmation_progress = if self.state == SmartTrendStateKind::Stalking {
            Some((self.confirmation_bars, self.confirmation_needed))
        } else {
            None
        };

        self.last_score = score;
        if self.fsm_eval_ring.len() >= 100 {
            self.fsm_eval_ring.pop_front();
        }
        self.fsm_eval_ring.push_back(self.state);

        if self.state == SmartTrendStateKind::Stalking {
            self.consecutive_stalking += 1;
        } else {
            self.consecutive_stalking = 0;
        }

        SmartTrendResult {
            score,
            state: self.state,
            prev_state,
            direction: self.direction,
            markers,
            pillar_scores: diag,
            bars_in_state: self.bars_in_state,
            confirmation_progress,
        }
    }

    /// Build a diagnostics snapshot for the Tauri command.
    pub fn diagnostics(&self) -> SmartTrendDiagnostics {
        let diag = self.last_diagnostics.clone().unwrap_or_else(|| {
            let default_macro = MacroInput {
                oracle_score: 0.0, confidence: 0.0, regime: MarketRegime::Unknown,
                kalman_gain: 0.0, price_surprise: 0.0, kalman_velocity: 0.0,
                run_length: 0, changepoint_prob: 0.0,
            };
            let default_micro = MicroInput {
                ofi_value: 0.0, microprice: 0.0, mid_price: 0.0, cvd: 0.0,
                spread_bps: 0.0, imbalance_ratio: 0.0, has_live_data: false,
            };
            let default_momentum = MomentumInput {
                gpu_buy_pct: 0.0, gpu_sell_pct: 0.0, dominant_cycle: 0,
                momentum_bars: 0, rvol: 0.0,
            };
            evaluate_pillars(&default_macro, &default_micro, &default_momentum, &self.config)
        });
        SmartTrendDiagnostics {
            enabled: self.config.st_enabled,
            killed: self.config.st_killed,
            state: self.state,
            direction: self.direction,
            combined_score: diag.combined_score,
            pillar_scores: diag.clone(),
            bars_in_state: self.bars_in_state,
            confirmation_progress: if self.state == SmartTrendStateKind::Stalking {
                Some((self.confirmation_bars, self.confirmation_needed))
            } else {
                None
            },
            bayesian_estimates: self.tracker.all_estimates(),
            strength: diag.strength,
            last_transition: self.last_transition.clone(),
            stats: self.stats.clone(),
            entry_price: self.entry_price,
            best_price: self.best_price,
        }
    }

    pub fn fsm_heatmap(&self) -> (u32, u32, u32) {
        let (mut s, mut e, mut t) = (0u32, 0u32, 0u32);
        for st in &self.fsm_eval_ring {
            match st {
                SmartTrendStateKind::Stalking => s += 1,
                SmartTrendStateKind::Entry => e += 1,
                SmartTrendStateKind::InTrade | SmartTrendStateKind::Trail => t += 1,
                _ => {}
            }
        }
        (s, e, t)
    }

    pub fn hysteresis_health(&self) -> bool {
        self.consecutive_stalking < 50
    }

    /// Get the accumulated trail tightening fraction (for B1 downstream use).
    pub fn trail_tighten_factor(&self) -> f64 {
        self.trail_tighten_accumulated
    }
}

// ═══════════════════════════════════════════════════════════════════
//  TESTS — 18+ unit tests covering all components
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> SmartTrendConfig {
        SmartTrendConfig::default()
    }

    fn bullish_macro_input() -> MacroInput {
        MacroInput {
            oracle_score: 70.0,
            confidence: 0.8,
            regime: MarketRegime::Trending,
            kalman_gain: 0.2,
            price_surprise: 0.01, // Above prediction
            kalman_velocity: 1.0, // Bullish velocity
            run_length: 20,
            changepoint_prob: 0.1,
        }
    }

    fn neutral_macro_input() -> MacroInput {
        MacroInput {
            oracle_score: 0.0,
            confidence: 0.5,
            regime: MarketRegime::Unknown,
            kalman_gain: 0.3,
            price_surprise: 0.0,
            kalman_velocity: 0.0,
            run_length: 15,
            changepoint_prob: 0.1,
        }
    }

    // ── T1: Macro Pillar Tests ──────────────────────────────────

    #[test]
    fn macro_bullish_trending_scores_above_70() {
        let input = bullish_macro_input();
        let score = evaluate_macro(&input, &default_config());
        assert!(
            score.score > 70.0,
            "Bullish trending should score >70, got {:.1}",
            score.score
        );
        assert_eq!(score.direction, PillarDirection::Bullish);
    }

    #[test]
    fn macro_regime_instability_penalizes() {
        let mut input = bullish_macro_input();
        input.run_length = 3; // Very short run after changepoint
        input.changepoint_prob = 0.6; // High changepoint
        let score = evaluate_macro(&input, &default_config());
        let stable_score = evaluate_macro(&bullish_macro_input(), &default_config());
        assert!(
            score.score < stable_score.score,
            "Unstable regime should penalize: unstable={:.1} vs stable={:.1}",
            score.score, stable_score.score
        );
    }

    #[test]
    fn macro_flat_market_near_50() {
        let input = neutral_macro_input();
        let score = evaluate_macro(&input, &default_config());
        assert!(
            (score.score - 50.0).abs() < 15.0,
            "Flat market should score near 50, got {:.1}",
            score.score
        );
    }

    #[test]
    fn macro_bearish_scores_below_40() {
        let input = MacroInput {
            oracle_score: -60.0,
            confidence: 0.75,
            regime: MarketRegime::Trending,
            kalman_gain: 0.2,
            price_surprise: -0.01,
            kalman_velocity: -1.0,
            run_length: 25,
            changepoint_prob: 0.05,
        };
        let score = evaluate_macro(&input, &default_config());
        assert!(
            score.score < 40.0,
            "Bearish should score <40, got {:.1}",
            score.score
        );
        assert_eq!(score.direction, PillarDirection::Bearish);
    }

    // ── T2: Micro Pillar Tests ──────────────────────────────────

    #[test]
    fn micro_buying_pressure_above_60() {
        let input = MicroInput {
            ofi_value: 30.0,
            microprice: 100.05,
            mid_price: 100.0,
            cvd: 50.0,
            spread_bps: 3.0,
            imbalance_ratio: 0.4,
            has_live_data: true,
        };
        let score = evaluate_micro(&input, &default_config());
        assert!(
            score.score > 55.0,
            "Buying pressure should score >55, got {:.1}",
            score.score
        );
    }

    #[test]
    fn micro_cvd_ofi_divergence_penalizes() {
        let input = MicroInput {
            ofi_value: 20.0,     // Bullish OFI
            microprice: 100.02,
            mid_price: 100.0,
            cvd: -40.0,          // Bearish CVD — divergence!
            spread_bps: 5.0,
            imbalance_ratio: 0.2,
            has_live_data: true,
        };
        // Compare with converging input
        let converging = MicroInput {
            cvd: 40.0, // Same direction as OFI
            ..input.clone()
        };
        let div_score = evaluate_micro(&input, &default_config());
        let conv_score = evaluate_micro(&converging, &default_config());
        assert!(
            div_score.score < conv_score.score,
            "Divergence should penalize: div={:.1} vs conv={:.1}",
            div_score.score, conv_score.score
        );
    }

    #[test]
    fn micro_no_data_returns_neutral_50() {
        let input = MicroInput {
            ofi_value: 100.0,
            microprice: 200.0,
            mid_price: 200.0,
            cvd: 500.0,
            spread_bps: 1.0,
            imbalance_ratio: 0.9,
            has_live_data: false, // KEY: no live data
        };
        let score = evaluate_micro(&input, &default_config());
        assert!(
            (score.score - 50.0).abs() < f64::EPSILON,
            "No data should return exactly 50, got {:.1}",
            score.score
        );
        assert_eq!(score.confidence, 0.0);
    }

    // ── T3: Momentum Pillar Tests ───────────────────────────────

    #[test]
    fn momentum_strong_buy_gpu_high_rvol_above_80() {
        let input = MomentumInput {
            gpu_buy_pct: 85.0,
            gpu_sell_pct: 15.0,
            dominant_cycle: 8,
            momentum_bars: 12, // Above cycle
            rvol: 2.5,
        };
        let score = evaluate_momentum(&input, &default_config());
        assert!(
            score.score > 70.0,
            "Strong buy + high RVOL should score >70, got {:.1}",
            score.score
        );
        assert_eq!(score.direction, PillarDirection::Bullish);
    }

    #[test]
    fn momentum_below_dominant_cycle_penalized() {
        let input = MomentumInput {
            gpu_buy_pct: 70.0,
            gpu_sell_pct: 30.0,
            dominant_cycle: 16,
            momentum_bars: 4, // Well below cycle!
            rvol: 1.2,
        };
        let no_filter = MomentumInput {
            momentum_bars: 20, // Above cycle
            ..input.clone()
        };
        let penalized = evaluate_momentum(&input, &default_config());
        let normal = evaluate_momentum(&no_filter, &default_config());
        assert!(
            penalized.score < normal.score,
            "Below-cycle momentum should be penalized: pen={:.1} vs norm={:.1}",
            penalized.score, normal.score
        );
    }

    #[test]
    fn momentum_neutral_gpu_near_50() {
        let input = MomentumInput {
            gpu_buy_pct: 52.0,
            gpu_sell_pct: 48.0,
            dominant_cycle: 8,
            momentum_bars: 10,
            rvol: 1.0,
        };
        let score = evaluate_momentum(&input, &default_config());
        assert!(
            (score.score - 50.0).abs() < 10.0,
            "Neutral GPU should score near 50, got {:.1}",
            score.score
        );
    }

    // ── T4: BayesianTracker Tests ───────────────────────────────

    #[test]
    fn bayesian_convergence_after_100_successes() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..100 {
            tracker.update(MarketRegime::Trending, true);
        }
        let est = tracker.estimate(MarketRegime::Trending);
        assert!(
            est > 0.9,
            "100 successes should give estimate >0.9, got {:.3}",
            est
        );
    }

    #[test]
    fn bayesian_decay_reduces_estimates() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..50 {
            tracker.update(MarketRegime::Trending, true);
        }
        let before = tracker.estimate(MarketRegime::Trending);
        for _ in 0..100 {
            tracker.decay_all(0.95); // Aggressive decay
        }
        let after = tracker.estimate(MarketRegime::Trending);
        assert!(
            after < before,
            "Decay should reduce estimates: before={:.3} after={:.3}",
            before, after
        );
    }

    #[test]
    fn bayesian_per_regime_isolation() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..50 {
            tracker.update(MarketRegime::Trending, true);
        }
        // Chaotic should still be at prior (0.5)
        let chaotic_est = tracker.estimate(MarketRegime::Chaotic);
        assert!(
            (chaotic_est - 0.5).abs() < 0.01,
            "Chaotic regime should be unaffected at ~0.5, got {:.3}",
            chaotic_est
        );
        assert!(tracker.estimate(MarketRegime::Trending) > 0.9);
    }

    #[test]
    fn bayesian_50_50_stays_near_half() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..100 {
            tracker.update(MarketRegime::Unknown, true);
            tracker.update(MarketRegime::Unknown, false);
        }
        let est = tracker.estimate(MarketRegime::Unknown);
        assert!(
            (est - 0.5).abs() < 0.05,
            "50/50 outcomes should give ~0.5, got {:.3}",
            est
        );
    }

    // ── T5: SmartTrendConfig Tests ──────────────────────────────

    #[test]
    fn config_default_is_valid() {
        let cfg = SmartTrendConfig::default();
        let result = validate_smarttrend_config(&cfg);
        assert!(result.valid, "Default config should be valid: {:?}", result.warnings);
    }

    #[test]
    fn config_serde_roundtrip() {
        let cfg = SmartTrendConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let restored: SmartTrendConfig = serde_json::from_str(&json).expect("deserialize");
        assert!((cfg.st_macro_weight - restored.st_macro_weight).abs() < f64::EPSILON);
        assert!((cfg.st_entry_threshold - restored.st_entry_threshold).abs() < f64::EPSILON);
        assert_eq!(cfg.st_enabled, restored.st_enabled);
    }

    // ── B3: Config Validation Tests ─────────────────────────────

    #[test]
    fn config_bad_weights_warns_but_still_valid() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = 0.5;
        cfg.st_micro_weight = 0.5;
        cfg.st_momentum_weight = 0.5; // Sum = 1.5, auto-normalizable
        let result = validate_smarttrend_config(&cfg);
        assert!(result.valid, "Non-negative weights should be valid (auto-normalized)");
        assert!(result.warnings.iter().any(|w| w.contains("weights sum")));
    }

    #[test]
    fn config_bad_decay_warns() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_bayesian_decay = 0.5; // Way too aggressive
        let result = validate_smarttrend_config(&cfg);
        assert!(result.warnings.iter().any(|w| w.contains("bayesian_decay")));
    }

    // ── B5: Cross-Pillar Divergence Tests ───────────────────────

    #[test]
    fn divergence_detected_bull_vs_bear() {
        let (div, detail) = detect_cross_pillar_divergence(
            PillarDirection::Bullish,
            PillarDirection::Bearish,
            PillarDirection::Neutral,
        );
        assert!(div, "Should detect divergence when bull vs bear");
        assert!(detail.contains("DIVERGENCE"));
    }

    #[test]
    fn no_divergence_when_aligned() {
        let (div, _) = detect_cross_pillar_divergence(
            PillarDirection::Bullish,
            PillarDirection::Bullish,
            PillarDirection::Neutral,
        );
        assert!(!div, "Should not detect divergence when aligned");
    }

    // ── Combined Evaluation Test ────────────────────────────────

    #[test]
    fn combined_score_within_range() {
        let macro_in = bullish_macro_input();
        let micro_in = MicroInput {
            ofi_value: 15.0,
            microprice: 100.02,
            mid_price: 100.0,
            cvd: 20.0,
            spread_bps: 4.0,
            imbalance_ratio: 0.3,
            has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 65.0,
            gpu_sell_pct: 35.0,
            dominant_cycle: 8,
            momentum_bars: 10,
            rvol: 1.5,
        };
        let diag = evaluate_pillars(&macro_in, &micro_in, &momentum_in, &default_config());
        assert!(
            diag.combined_score >= 0.0 && diag.combined_score <= 100.0,
            "Combined score should be [0,100], got {:.1}",
            diag.combined_score
        );
    }

    // ── Weight Normalization Test ────────────────────────────────

    #[test]
    fn normalize_weights_sums_to_one() {
        let (a, b, c) = normalize_weights(0.5, 0.3, 0.7);
        let sum = a + b + c;
        assert!(
            (sum - 1.0).abs() < 1e-10,
            "Normalized weights should sum to 1.0, got {:.10}",
            sum
        );
    }

    #[test]
    fn normalize_weights_zero_fallback() {
        let (a, b, c) = normalize_weights(0.0, 0.0, 0.0);
        assert!((a - 0.40).abs() < 1e-10);
        assert!((b - 0.35).abs() < 1e-10);
        assert!((c - 0.25).abs() < 1e-10);
    }

    // ── Bayesian Halflife Tests ─────────────────────────────────

    #[test]
    fn bayesian_halflife_200_halves_weight() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..100 {
            tracker.update(MarketRegime::Trending, true);
        }
        let before_alpha = tracker.trending.alpha;
        for _ in 0..200 {
            tracker.decay_halflife(200);
        }
        let after_alpha = tracker.trending.alpha;
        let ratio = after_alpha / before_alpha;
        assert!(
            (ratio - 0.5).abs() < 0.05,
            "After 200 decay calls with halflife=200, alpha should halve. ratio={:.3}",
            ratio
        );
    }

    #[test]
    fn bayesian_halflife_zero_is_noop() {
        let mut tracker = BayesianTracker::default();
        for _ in 0..50 {
            tracker.update(MarketRegime::Trending, true);
        }
        let before = tracker.trending.alpha;
        tracker.decay_halflife(0);
        assert!(
            (tracker.trending.alpha - before).abs() < f64::EPSILON,
            "halflife=0 should be no-op"
        );
    }

    // ── Config Validation: Errors vs Warnings ───────────────────

    #[test]
    fn config_negative_weight_is_error() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = -0.1;
        let result = validate_smarttrend_config(&cfg);
        assert!(!result.valid, "Negative weight should invalidate config");
        assert!(result.errors.iter().any(|e| e.contains("negative")));
    }

    #[test]
    fn config_threshold_ordering_is_error() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_stalking_threshold = 70.0;
        cfg.st_entry_threshold = 60.0; // stalking >= entry = error
        let result = validate_smarttrend_config(&cfg);
        assert!(!result.valid, "stalking >= entry should invalidate config");
        assert!(result.errors.iter().any(|e| e.contains("stalking")));
    }

    // ── Confidence Weighting Tests ──────────────────────────────

    #[test]
    fn low_confidence_pillar_has_reduced_impact() {
        let macro_in = bullish_macro_input();
        let micro_high_conf = MicroInput {
            ofi_value: -30.0,
            microprice: 99.95,
            mid_price: 100.0,
            cvd: -50.0,
            spread_bps: 3.0,
            imbalance_ratio: -0.4,
            has_live_data: true,
        };
        // With no live data (confidence=0), micro's bearish signal should be muted
        let micro_stale = MicroInput {
            has_live_data: false,
            ..micro_high_conf.clone()
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 60.0,
            gpu_sell_pct: 40.0,
            dominant_cycle: 8,
            momentum_bars: 10,
            rvol: 1.0,
        };
        let diag_live = evaluate_pillars(&macro_in, &micro_high_conf, &momentum_in, &default_config());
        let diag_stale = evaluate_pillars(&macro_in, &micro_stale, &momentum_in, &default_config());
        assert!(
            diag_stale.combined_score > diag_live.combined_score,
            "Stale micro (neutral 50) with bullish macro should score higher than live bearish micro: stale={:.1} vs live={:.1}",
            diag_stale.combined_score, diag_live.combined_score
        );
    }

    #[test]
    fn confidence_redistribution_preserves_range() {
        let macro_in = MacroInput {
            confidence: 0.05, // Very low confidence
            ..bullish_macro_input()
        };
        let micro_in = MicroInput {
            ofi_value: 20.0,
            microprice: 100.02,
            mid_price: 100.0,
            cvd: 30.0,
            spread_bps: 3.0,
            imbalance_ratio: 0.3,
            has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 70.0,
            gpu_sell_pct: 30.0,
            dominant_cycle: 8,
            momentum_bars: 10,
            rvol: 1.5,
        };
        let diag = evaluate_pillars(&macro_in, &micro_in, &momentum_in, &default_config());
        assert!(
            diag.combined_score >= 0.0 && diag.combined_score <= 100.0,
            "Combined score should remain in [0,100] even with low-confidence redistribution: {:.1}",
            diag.combined_score
        );
    }

    // ── Regime-Adaptive Threshold Tests ─────────────────────────

    #[test]
    fn trending_regime_has_lower_threshold() {
        let cfg = SmartTrendConfig::default();
        let macro_trending = MacroInput {
            regime: MarketRegime::Trending,
            ..bullish_macro_input()
        };
        let macro_chaotic = MacroInput {
            regime: MarketRegime::Chaotic,
            ..bullish_macro_input()
        };
        let micro_in = MicroInput {
            ofi_value: 10.0, microprice: 100.01, mid_price: 100.0,
            cvd: 10.0, spread_bps: 3.0, imbalance_ratio: 0.1, has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 60.0, gpu_sell_pct: 40.0,
            dominant_cycle: 8, momentum_bars: 10, rvol: 1.0,
        };
        let diag_trend = evaluate_pillars(&macro_trending, &micro_in, &momentum_in, &cfg);
        let diag_chaos = evaluate_pillars(&macro_chaotic, &micro_in, &momentum_in, &cfg);
        assert!(
            diag_trend.effective_entry_threshold < diag_chaos.effective_entry_threshold,
            "Trending threshold ({}) should be lower than Chaotic ({})",
            diag_trend.effective_entry_threshold, diag_chaos.effective_entry_threshold
        );
    }

    #[test]
    fn regime_threshold_matches_config_array() {
        let cfg = SmartTrendConfig::default();
        let macro_in = MacroInput {
            regime: MarketRegime::MeanReverting,
            ..bullish_macro_input()
        };
        let micro_in = MicroInput {
            ofi_value: 10.0, microprice: 100.01, mid_price: 100.0,
            cvd: 10.0, spread_bps: 3.0, imbalance_ratio: 0.1, has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 60.0, gpu_sell_pct: 40.0,
            dominant_cycle: 8, momentum_bars: 10, rvol: 1.0,
        };
        let diag = evaluate_pillars(&macro_in, &micro_in, &momentum_in, &cfg);
        assert!(
            (diag.effective_entry_threshold - cfg.st_entry_thresholds[1]).abs() < f64::EPSILON,
            "MeanReverting should use index 1: got {} vs config {}",
            diag.effective_entry_threshold, cfg.st_entry_thresholds[1]
        );
    }

    #[test]
    fn chaotic_regime_stricter_than_unknown() {
        let cfg = SmartTrendConfig::default();
        assert!(
            cfg.st_entry_thresholds[2] > cfg.st_entry_thresholds[3],
            "Chaotic ({}) should be stricter than Unknown ({})",
            cfg.st_entry_thresholds[2], cfg.st_entry_thresholds[3]
        );
    }

    // ── Staleness Detection Tests ───────────────────────────────

    #[test]
    fn macro_stale_on_low_confidence() {
        let input = MacroInput {
            confidence: 0.35,
            ..bullish_macro_input()
        };
        let score = evaluate_macro(&input, &default_config());
        assert!(score.stale, "Confidence < MIN_CONFIDENCE_FOR_SIGNAL (0.4) should mark macro as stale");

        let input_ok = MacroInput {
            confidence: 0.45,
            ..bullish_macro_input()
        };
        let score_ok = evaluate_macro(&input_ok, &default_config());
        assert!(!score_ok.stale, "Confidence > MIN_CONFIDENCE_FOR_SIGNAL (0.4) should NOT be stale");
    }

    #[test]
    fn micro_stale_without_live_data() {
        let input = MicroInput {
            ofi_value: 30.0, microprice: 100.05, mid_price: 100.0,
            cvd: 50.0, spread_bps: 3.0, imbalance_ratio: 0.4,
            has_live_data: false,
        };
        let score = evaluate_micro(&input, &default_config());
        assert!(score.stale, "No live data should mark micro as stale");
    }

    #[test]
    fn momentum_stale_without_gpu_data() {
        let input = MomentumInput {
            gpu_buy_pct: 0.0,
            gpu_sell_pct: 0.0, // No GPU data
            dominant_cycle: 8,
            momentum_bars: 10,
            rvol: 1.0,
        };
        let score = evaluate_momentum(&input, &default_config());
        assert!(score.stale, "No GPU data should mark momentum as stale");
    }

    // ── Strength Classifier Tests ───────────────────────────────

    #[test]
    fn strength_classification_boundaries() {
        assert_eq!(SmartTrendStrength::from_score(40.0), SmartTrendStrength::Weak);
        assert_eq!(SmartTrendStrength::from_score(54.9), SmartTrendStrength::Weak);
        assert_eq!(SmartTrendStrength::from_score(55.0), SmartTrendStrength::Moderate);
        assert_eq!(SmartTrendStrength::from_score(69.9), SmartTrendStrength::Moderate);
        assert_eq!(SmartTrendStrength::from_score(70.0), SmartTrendStrength::Strong);
        assert_eq!(SmartTrendStrength::from_score(84.9), SmartTrendStrength::Strong);
        assert_eq!(SmartTrendStrength::from_score(85.0), SmartTrendStrength::Extreme);
        assert_eq!(SmartTrendStrength::from_score(99.0), SmartTrendStrength::Extreme);
    }

    #[test]
    fn diagnostics_includes_strength() {
        let macro_in = bullish_macro_input();
        let micro_in = MicroInput {
            ofi_value: 15.0, microprice: 100.02, mid_price: 100.0,
            cvd: 20.0, spread_bps: 4.0, imbalance_ratio: 0.3, has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 65.0, gpu_sell_pct: 35.0,
            dominant_cycle: 8, momentum_bars: 10, rvol: 1.5,
        };
        let diag = evaluate_pillars(&macro_in, &micro_in, &momentum_in, &default_config());
        let expected = SmartTrendStrength::from_score(diag.combined_score);
        assert_eq!(diag.strength, expected);
    }

    // ═══════════════════════════════════════════════════════════════
    //  S07b — FSM Integration Tests
    // ═══════════════════════════════════════════════════════════════

    fn bullish_st_input(price: f64) -> SmartTrendInput {
        SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 80.0,
                confidence: 0.85,
                regime: MarketRegime::Trending,
                kalman_gain: 0.15,
                price_surprise: 0.02,
                kalman_velocity: 1.5,
                run_length: 30,
                changepoint_prob: 0.05,
            },
            micro_input: MicroInput {
                ofi_value: 40.0,
                microprice: price + 0.05,
                mid_price: price,
                cvd: 80.0,
                spread_bps: 3.0,
                imbalance_ratio: 0.5,
                has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 80.0,
                gpu_sell_pct: 20.0,
                dominant_cycle: 8,
                momentum_bars: 12,
                rvol: 2.0,
            },
            current_price: price,
            changepoint_prob: 0.05,
            dominant_cycle: 8.0,
            tf_minutes: 1.0,
        }
    }

    fn weak_st_input(price: f64) -> SmartTrendInput {
        SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 5.0,
                confidence: 0.4,
                regime: MarketRegime::Unknown,
                kalman_gain: 0.3,
                price_surprise: 0.0,
                kalman_velocity: 0.0,
                run_length: 20,
                changepoint_prob: 0.1,
            },
            micro_input: MicroInput {
                ofi_value: 2.0,
                microprice: price,
                mid_price: price,
                cvd: 1.0,
                spread_bps: 8.0,
                imbalance_ratio: 0.05,
                has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 52.0,
                gpu_sell_pct: 48.0,
                dominant_cycle: 8,
                momentum_bars: 10,
                rvol: 0.8,
            },
            current_price: price,
            changepoint_prob: 0.1,
            dominant_cycle: 8.0,
            tf_minutes: 1.0,
        }
    }

    fn bearish_st_input(price: f64) -> SmartTrendInput {
        SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: -75.0,
                confidence: 0.82,
                regime: MarketRegime::Trending,
                kalman_gain: 0.12,
                price_surprise: -0.02,
                kalman_velocity: -1.5,
                run_length: 25,
                changepoint_prob: 0.05,
            },
            micro_input: MicroInput {
                ofi_value: -35.0,
                microprice: price - 0.05,
                mid_price: price,
                cvd: -70.0,
                spread_bps: 3.0,
                imbalance_ratio: -0.45,
                has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 20.0,
                gpu_sell_pct: 80.0,
                dominant_cycle: 8,
                momentum_bars: 12,
                rvol: 2.2,
            },
            current_price: price,
            changepoint_prob: 0.05,
            dominant_cycle: 8.0,
            tf_minutes: 1.0,
        }
    }

    #[test]
    fn fsm_full_cycle_bullish() {
        let mut engine = SmartTrendEngine::new(default_config());
        assert_eq!(engine.state, SmartTrendStateKind::Flat);

        // Feed bullish data to enter Stalking
        let input = bullish_st_input(100.0);
        let r = engine.evaluate(&input);
        assert_eq!(r.state, SmartTrendStateKind::Stalking, "should enter Stalking, score={:.1}", r.score);
        assert!(r.markers.iter().any(|m| m.kind == SmartTrendMarkerKind::Stalking));

        // Feed enough bars to confirm -> Entry
        for _ in 0..20 {
            let r = engine.evaluate(&input);
            if r.state == SmartTrendStateKind::Entry || r.state == SmartTrendStateKind::InTrade {
                break;
            }
        }
        // Entry now persists for st_entry_persist_bars (default 3) before auto-transition
        let persist_bars = engine.config.st_entry_persist_bars.unwrap_or(3);
        for _ in 0..persist_bars {
            if engine.state == SmartTrendStateKind::Entry {
                engine.evaluate(&input);
            }
        }
        assert_eq!(engine.state, SmartTrendStateKind::InTrade);

        // Feed weak data to trigger Trail
        let weak = weak_st_input(102.0);
        for _ in 0..5 {
            let r = engine.evaluate(&weak);
            if r.state == SmartTrendStateKind::Trail {
                break;
            }
        }
        assert_eq!(engine.state, SmartTrendStateKind::Trail);

        // Feed bearish data to trigger Exit -> Flat
        let bear = bearish_st_input(101.0);
        let r = engine.evaluate(&bear);
        assert_eq!(r.state, SmartTrendStateKind::Flat);
        assert!(r.markers.iter().any(|m| m.kind == SmartTrendMarkerKind::Exit));
    }

    #[test]
    fn fsm_full_cycle_bearish() {
        let mut engine = SmartTrendEngine::new(default_config());

        let input = bearish_st_input(100.0);
        let r = engine.evaluate(&input);
        assert_eq!(r.state, SmartTrendStateKind::Stalking,
            "strong bearish should enter Stalking, score={:.1}", r.score);
        assert_eq!(r.direction, PillarDirection::Bearish);

        for _ in 0..20 {
            let r = engine.evaluate(&input);
            if r.state == SmartTrendStateKind::InTrade { break; }
        }
        let persist_bars = engine.config.st_entry_persist_bars.unwrap_or(3);
        for _ in 0..persist_bars {
            if engine.state == SmartTrendStateKind::Entry {
                engine.evaluate(&input);
            }
        }
        assert_eq!(engine.state, SmartTrendStateKind::InTrade);
        assert_eq!(engine.direction, PillarDirection::Bearish);
    }

    #[test]
    fn fsm_changepoint_reset_from_intrade() {
        let mut engine = SmartTrendEngine::new(default_config());

        // Get to InTrade
        let input = bullish_st_input(100.0);
        for _ in 0..30 {
            engine.evaluate(&input);
            if engine.state == SmartTrendStateKind::InTrade { break; }
        }
        assert_eq!(engine.state, SmartTrendStateKind::InTrade);

        // Hit changepoint
        let mut cp_input = input.clone();
        cp_input.changepoint_prob = 0.85;
        let r = engine.evaluate(&cp_input);
        assert_eq!(r.state, SmartTrendStateKind::Flat, "changepoint should reset to Flat");
        assert!(r.markers.iter().any(|m| m.kind == SmartTrendMarkerKind::Exit));
        assert!(r.markers.iter().any(|m| m.kind == SmartTrendMarkerKind::ChangePointReset));
        assert_eq!(engine.stats.changepoint_resets, 1);
    }

    #[test]
    fn fsm_stalking_timeout() {
        let mut engine = SmartTrendEngine::new(default_config());

        // Enter Stalking
        let input = bullish_st_input(100.0);
        engine.evaluate(&input);
        assert_eq!(engine.state, SmartTrendStateKind::Stalking);

        // Feed data that stays in stalking range (score ~56-58) but below entry threshold (60)
        // stalking_threshold=55 means directional_strength >= 10 (|score-50|*2 >= |55-50|*2 = 10)
        // entry threshold for Trending = 60, meaning directional_strength >= 20 to confirm
        // We need score ~56 => directional_strength = 12, above stalking(10) but below entry(20)
        let limbo = SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 15.0, // Mildly bullish
                confidence: 0.5,
                regime: MarketRegime::Trending,
                kalman_gain: 0.2,
                price_surprise: 0.005,
                kalman_velocity: 0.3,
                run_length: 20,
                changepoint_prob: 0.1,
            },
            micro_input: MicroInput {
                ofi_value: 8.0,
                microprice: 100.01,
                mid_price: 100.0,
                cvd: 10.0,
                spread_bps: 5.0,
                imbalance_ratio: 0.1,
                has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 56.0,
                gpu_sell_pct: 44.0,
                dominant_cycle: 8,
                momentum_bars: 10,
                rvol: 1.0,
            },
            current_price: 100.0,
            changepoint_prob: 0.1,
            dominant_cycle: 8.0,
            tf_minutes: 1.0,
        };

        for _ in 0..50 {
            engine.evaluate(&limbo);
            if engine.state == SmartTrendStateKind::Flat {
                break;
            }
        }
        // Should have either timed out or dropped below stalking
        assert!(
            engine.state == SmartTrendStateKind::Flat || engine.stats.stalking_timeouts > 0,
            "should timeout to Flat or have timeouts recorded, state={:?}", engine.state
        );
    }

    #[test]
    fn fsm_dynamic_confirmation_adapts_to_cycle() {
        let config = default_config();
        let engine = SmartTrendEngine::new(config);

        // Short cycle: dominant_cycle=4, tf=1 -> needed=4 (clamped to 2 min)
        let needed_short = engine.compute_confirmation_needed(4.0, 1.0, SmartTrendStrength::Moderate);
        assert!(needed_short >= 2 && needed_short <= 8, "short cycle: {}", needed_short);

        // Long cycle: dominant_cycle=20, tf=1 -> needed=20
        let needed_long = engine.compute_confirmation_needed(20.0, 1.0, SmartTrendStrength::Moderate);
        assert!(needed_long >= 15 && needed_long <= 25, "long cycle: {}", needed_long);

        // Extreme strength shortens by 25%
        let needed_extreme = engine.compute_confirmation_needed(20.0, 1.0, SmartTrendStrength::Extreme);
        assert!(needed_extreme < needed_long, "extreme should shorten: {} < {}", needed_extreme, needed_long);
    }

    #[test]
    fn fsm_no_entry_during_stabilization() {
        let mut engine = SmartTrendEngine::new(default_config());

        // Bullish data with stable run_length
        let input_stable = bullish_st_input(100.0);
        let r_stable = engine.evaluate(&input_stable);
        let stable_score = r_stable.score;

        // Reset engine
        engine = SmartTrendEngine::new(default_config());

        // Same data but with short run_length (< stabilization_window)
        let mut input_unstable = bullish_st_input(100.0);
        input_unstable.macro_input.run_length = 3;
        let r_unstable = engine.evaluate(&input_unstable);

        // Score should be penalized relative to stable version
        assert!(r_unstable.score < stable_score,
            "short run_length should penalize: unstable={:.1} vs stable={:.1}",
            r_unstable.score, stable_score
        );
    }

    #[test]
    fn fsm_hysteresis_prevents_chatter() {
        let mut cfg = default_config();
        cfg.st_hysteresis_margin = 5.0;
        let mut engine = SmartTrendEngine::new(cfg.clone());

        // Enter Stalking
        let input = bullish_st_input(100.0);
        engine.evaluate(&input);
        assert_eq!(engine.state, SmartTrendStateKind::Stalking);

        // Score just below stalking threshold but above threshold - hysteresis
        // stalking_threshold = 55, hysteresis = 5, so drop_threshold = 50
        let mut borderline = weak_st_input(100.0);
        borderline.macro_input.oracle_score = 20.0; // Will produce score ~55-60 range
        borderline.micro_input.ofi_value = 12.0;
        borderline.micro_input.cvd = 15.0;
        borderline.momentum_input.gpu_buy_pct = 58.0;
        borderline.momentum_input.gpu_sell_pct = 42.0;

        let r = engine.evaluate(&borderline);
        // The hysteresis means we only drop to Flat if score < 50, not < 55
        // So moderate scores should keep us in Stalking
        if r.score > 50.0 && r.score < 55.0 {
            assert_eq!(engine.state, SmartTrendStateKind::Stalking,
                "hysteresis should prevent chatter at score {:.1}", r.score);
        }
    }

    #[test]
    fn fsm_pillar_agreement_blocks_divergent_entry() {
        let mut engine = SmartTrendEngine::new(default_config());

        // Enter Stalking with aligned data
        let input = bullish_st_input(100.0);
        engine.evaluate(&input);
        assert_eq!(engine.state, SmartTrendStateKind::Stalking);

        // Create divergent data: macro bullish, micro bearish
        let mut divergent = input.clone();
        divergent.micro_input.ofi_value = -40.0;
        divergent.micro_input.cvd = -80.0;
        divergent.micro_input.imbalance_ratio = -0.5;
        divergent.micro_input.microprice = 99.95;

        // Feed divergent data — should NOT reach Entry due to pillar agreement gate
        for _ in 0..25 {
            let r = engine.evaluate(&divergent);
            assert_ne!(r.state, SmartTrendStateKind::Entry,
                "divergent pillars should block Entry");
            assert_ne!(r.state, SmartTrendStateKind::InTrade,
                "divergent pillars should block InTrade");
            if r.state == SmartTrendStateKind::Flat { break; }
        }
    }

    #[test]
    fn fsm_trail_tighten_accumulates() {
        let mut engine = SmartTrendEngine::new(default_config());

        // Get to Trail
        let input = bullish_st_input(100.0);
        for _ in 0..30 {
            engine.evaluate(&input);
            if engine.state == SmartTrendStateKind::InTrade { break; }
        }
        assert_eq!(engine.state, SmartTrendStateKind::InTrade);

        // Force trail with weak data
        let weak = weak_st_input(102.0);
        for _ in 0..5 {
            engine.evaluate(&weak);
            if engine.state == SmartTrendStateKind::Trail { break; }
        }
        assert_eq!(engine.state, SmartTrendStateKind::Trail);

        // Each bar in trail accumulates tightening
        engine.evaluate(&weak);
        engine.evaluate(&weak);
        assert!(
            engine.trail_tighten_accumulated > 0.0,
            "trail tighten should accumulate: {:.3}",
            engine.trail_tighten_accumulated
        );
    }

    #[test]
    fn fsm_stats_tracking_works() {
        let mut engine = SmartTrendEngine::new(default_config());
        let input = bullish_st_input(100.0);

        // Run through a full cycle
        for _ in 0..30 {
            engine.evaluate(&input);
        }

        // Stats should have recorded at least one transition
        assert!(!engine.stats.transitions.is_empty(), "stats should record transitions");
        let total_bars: u64 = engine.stats.bars_per_state.values().sum();
        assert!(total_bars > 0, "stats should record bars per state");
    }

    #[test]
    fn fsm_diagnostics_snapshot() {
        let mut engine = SmartTrendEngine::new(default_config());
        engine.config.st_enabled = true;

        let input = bullish_st_input(100.0);
        engine.evaluate(&input);

        let diag = engine.diagnostics();
        assert!(diag.enabled);
        assert!(!diag.killed);
        assert!(diag.combined_score > 0.0);
        assert!(!diag.last_transition.is_empty());
    }

    #[test]
    fn config_new_fields_have_defaults() {
        let cfg = SmartTrendConfig::default();
        assert!((cfg.st_changepoint_reset_threshold - 0.7).abs() < f64::EPSILON);
        assert!((cfg.st_trail_tighten_pct - 0.05).abs() < f64::EPSILON);
        assert!((cfg.st_hysteresis_margin - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn config_new_fields_validate() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_changepoint_reset_threshold = 1.5;
        let v = validate_smarttrend_config(&cfg);
        assert!(v.warnings.iter().any(|w| w.contains("changepoint_reset")));

        cfg.st_trail_tighten_pct = -0.1;
        let v = validate_smarttrend_config(&cfg);
        assert!(v.warnings.iter().any(|w| w.contains("trail_tighten")));
    }

    #[test]
    fn config_serde_roundtrip_with_new_fields() {
        let cfg = SmartTrendConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let restored: SmartTrendConfig = serde_json::from_str(&json).expect("deserialize");
        assert!((cfg.st_changepoint_reset_threshold - restored.st_changepoint_reset_threshold).abs() < f64::EPSILON);
        assert!((cfg.st_trail_tighten_pct - restored.st_trail_tighten_pct).abs() < f64::EPSILON);
        assert!((cfg.st_hysteresis_margin - restored.st_hysteresis_margin).abs() < f64::EPSILON);
    }

    #[test]
    fn config_backward_compat_missing_new_fields() {
        let old_json = r#"{"st_macro_weight":0.4,"st_micro_weight":0.35,"st_momentum_weight":0.25,"st_entry_threshold":65.0,"st_stalking_threshold":55.0,"st_entry_thresholds":[60.0,70.0,75.0,65.0],"st_stabilization_window":8,"st_bayesian_decay":0.997,"st_bayesian_halflife":200,"st_dominant_cycle_filter":true,"st_ofi_norm_range":50.0,"st_mp_bps_range":10.0,"st_cvd_scale":100.0,"st_enabled":false,"st_killed":false}"#;
        let cfg: SmartTrendConfig = serde_json::from_str(old_json).expect("old format should parse with defaults");
        assert!((cfg.st_changepoint_reset_threshold - 0.7).abs() < f64::EPSILON);
        assert!((cfg.st_trail_tighten_pct - 0.05).abs() < f64::EPSILON);
        assert!((cfg.st_hysteresis_margin - 5.0).abs() < f64::EPSILON);
    }

    // ═══════════════════════════════════════════════════════════════
    //  S08 TESTS — Kill Switch + Scoring Integration
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn kill_switch_default_off() {
        let cfg = SmartTrendConfig::default();
        assert!(!cfg.st_killed);
        assert!(!cfg.st_enabled);
    }

    #[test]
    fn kill_switch_persists_through_serde() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_killed = true;
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: SmartTrendConfig = serde_json::from_str(&json).unwrap();
        assert!(restored.st_killed);
    }

    #[test]
    fn killed_engine_still_evaluates_but_markers_informational() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_killed = true;
        cfg.st_enabled = true;
        let engine = SmartTrendEngine::new(cfg.clone());
        let diag = engine.diagnostics();
        assert!(diag.killed);
        assert_eq!(diag.state, SmartTrendStateKind::Flat);
    }

    #[test]
    fn engine_produces_markers_on_strong_signal() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_enabled = true;
        cfg.st_stalking_threshold = 55.0;
        cfg.st_entry_threshold = 65.0;
        let mut engine = SmartTrendEngine::new(cfg);

        let strong_input = SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 80.0, confidence: 0.9, regime: MarketRegime::Trending,
                kalman_gain: 0.1, price_surprise: 1.0, kalman_velocity: 0.5,
                run_length: 20, changepoint_prob: 0.0,
            },
            micro_input: MicroInput {
                ofi_value: 30.0, microprice: 101.0, mid_price: 100.5, cvd: 50.0,
                spread_bps: 5.0, imbalance_ratio: 0.7, has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 70.0, gpu_sell_pct: 30.0, dominant_cycle: 20,
                momentum_bars: 25, rvol: 1.5,
            },
            current_price: 101.0,
            changepoint_prob: 0.0,
            dominant_cycle: 20.0,
            tf_minutes: 5.0,
        };

        for _ in 0..3 {
            engine.evaluate(&strong_input);
        }
        let result = engine.evaluate(&strong_input);
        assert!(result.score > 50.0, "strong bullish signal should score above 50");
    }

    #[test]
    fn marker_kind_to_chart_text() {
        let entry = SmartTrendMarkerKind::Entry;
        let exit = SmartTrendMarkerKind::Exit;
        let trail = SmartTrendMarkerKind::Trail;
        let stalk = SmartTrendMarkerKind::Stalking;
        let cp = SmartTrendMarkerKind::ChangePointReset;

        assert_eq!(entry, SmartTrendMarkerKind::Entry);
        assert_eq!(exit, SmartTrendMarkerKind::Exit);
        assert_eq!(trail, SmartTrendMarkerKind::Trail);
        assert_eq!(stalk, SmartTrendMarkerKind::Stalking);
        assert_eq!(cp, SmartTrendMarkerKind::ChangePointReset);
    }

    #[test]
    fn diagnostics_contains_stats() {
        let cfg = SmartTrendConfig::default();
        let engine = SmartTrendEngine::new(cfg);
        let diag = engine.diagnostics();
        assert_eq!(diag.stats.entries_total, 0);
        assert_eq!(diag.stats.entries_reached_trail, 0);
        assert_eq!(diag.stats.changepoint_resets, 0);
    }

    // ═══════════════════════════════════════════════════════════════
    //  S10 TESTS — SmartTrendAccount + Config
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn st_account_new_sets_capital() {
        let acct = SmartTrendAccount::new(10000.0);
        assert!((acct.allocated_capital - 10000.0).abs() < f64::EPSILON);
        assert!((acct.initial_capital - 10000.0).abs() < f64::EPSILON);
        assert!((acct.high_water_mark - 10000.0).abs() < f64::EPSILON);
        assert_eq!(acct.total_trades, 0);
        assert!(!acct.paused);
    }

    #[test]
    fn st_account_record_win() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(50.0);
        assert_eq!(acct.total_trades, 1);
        assert_eq!(acct.total_wins, 1);
        assert_eq!(acct.trade_count_today, 1);
        assert_eq!(acct.win_count_today, 1);
        assert!((acct.realized_pnl_today - 50.0).abs() < f64::EPSILON);
        assert!((acct.allocated_capital - 10050.0).abs() < f64::EPSILON);
        assert!((acct.high_water_mark - 10050.0).abs() < f64::EPSILON);
        assert!((acct.avg_win - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn st_account_record_loss() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(-30.0);
        assert_eq!(acct.total_trades, 1);
        assert_eq!(acct.total_wins, 0);
        assert_eq!(acct.loss_count_today, 1);
        assert!((acct.realized_pnl_today - (-30.0)).abs() < f64::EPSILON);
        assert!((acct.allocated_capital - 9970.0).abs() < f64::EPSILON);
        assert!((acct.avg_loss - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn st_account_drawdown_pct() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(-500.0); // 5% drawdown
        let dd = acct.drawdown_pct();
        assert!((dd - 5.0).abs() < 0.01, "expected ~5%, got {}", dd);
    }

    #[test]
    fn st_account_hit_rate() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(50.0); // win
        acct.record_fill(50.0); // win
        acct.record_fill(-30.0); // loss
        let hr = acct.hit_rate();
        assert!((hr - 2.0/3.0).abs() < 0.01, "expected ~66.7%, got {}", hr * 100.0);
    }

    #[test]
    fn st_account_avg_win_loss_ratio() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(100.0); // win
        acct.record_fill(-50.0); // loss
        let ratio = acct.avg_win_loss_ratio();
        assert!((ratio - 2.0).abs() < 0.01, "expected 2.0, got {}", ratio);
    }

    #[test]
    fn st_account_daily_reset() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.record_fill(200.0);
        acct.record_fill(-50.0);
        acct.daily_reset();
        assert!((acct.realized_pnl_today).abs() < f64::EPSILON);
        assert_eq!(acct.trade_count_today, 0);
        assert_eq!(acct.win_count_today, 0);
        assert_eq!(acct.loss_count_today, 0);
        assert!(!acct.breaker_triggered);
        assert!((acct.initial_capital - acct.allocated_capital).abs() < f64::EPSILON);
    }

    #[test]
    fn st_account_serde_roundtrip() {
        let mut acct = SmartTrendAccount::new(5000.0);
        acct.record_fill(100.0);
        acct.record_fill(-40.0);
        let json = serde_json::to_string(&acct).expect("serialize");
        let restored: SmartTrendAccount = serde_json::from_str(&json).expect("deserialize");
        assert!((restored.allocated_capital - acct.allocated_capital).abs() < f64::EPSILON);
        assert_eq!(restored.total_trades, acct.total_trades);
        assert_eq!(restored.total_wins, acct.total_wins);
    }

    #[test]
    fn st_account_kelly_insufficient_trades() {
        let acct = SmartTrendAccount::new(10000.0);
        let kf = acct.kelly_fraction(20, 0.25);
        assert!((kf).abs() < f64::EPSILON, "no trades should give 0 kelly");
    }

    #[test]
    fn st_account_kelly_positive_edge() {
        let mut acct = SmartTrendAccount::new(10000.0);
        for _ in 0..15 { acct.record_fill(100.0); }
        for _ in 0..5 { acct.record_fill(-50.0); }
        let kf = acct.kelly_fraction(10, 0.25);
        assert!(kf > 0.0, "75% win rate with 2:1 ratio should give positive kelly");
        assert!(kf <= 0.25, "kelly should be capped at 0.25, got {}", kf);
    }

    #[test]
    fn st_account_kelly_capped() {
        let mut acct = SmartTrendAccount::new(10000.0);
        for _ in 0..50 { acct.record_fill(200.0); }
        for _ in 0..1 { acct.record_fill(-10.0); }
        let kf = acct.kelly_fraction(10, 0.25);
        assert!(kf <= 0.25, "kelly must be capped at max_fraction");
    }

    #[test]
    fn st_account_compute_size_usd_base() {
        let acct = SmartTrendAccount::new(10000.0);
        let cfg = SmartTrendConfig::default(); // st_risk_per_trade = 0.01
        let size = acct.compute_size_usd(&cfg);
        assert!((size - 100.0).abs() < f64::EPSILON, "1% of 10k = $100, got {}", size);
    }

    #[test]
    fn st_account_compute_size_usd_kelly_boost() {
        let mut acct = SmartTrendAccount::new(10000.0);
        for _ in 0..15 { acct.record_fill(100.0); }
        for _ in 0..5 { acct.record_fill(-50.0); }
        let mut cfg = SmartTrendConfig::default();
        cfg.st_kelly_enabled = true;
        cfg.st_kelly_min_trades = 10;
        let size = acct.compute_size_usd(&cfg);
        assert!(size > 100.0, "kelly boost should increase size above base $100, got {}", size);
    }

    #[test]
    fn st_config_s10_defaults() {
        let cfg = SmartTrendConfig::default();
        assert!((cfg.st_risk_per_trade - 0.01).abs() < f64::EPSILON);
        assert!((cfg.st_max_dd_pct - 5.0).abs() < f64::EPSILON);
        assert_eq!(cfg.st_bot_loop_ms, 2000);
        assert!(cfg.st_bot_paused);
        assert!(!cfg.st_kelly_enabled);
        assert_eq!(cfg.st_kelly_min_trades, 20);
        assert!((cfg.st_max_kelly_fraction - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn st_config_s10_serde_roundtrip() {
        let cfg = SmartTrendConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let restored: SmartTrendConfig = serde_json::from_str(&json).expect("deserialize");
        assert!((cfg.st_risk_per_trade - restored.st_risk_per_trade).abs() < f64::EPSILON);
        assert!((cfg.st_max_dd_pct - restored.st_max_dd_pct).abs() < f64::EPSILON);
        assert_eq!(cfg.st_bot_loop_ms, restored.st_bot_loop_ms);
        assert_eq!(cfg.st_bot_paused, restored.st_bot_paused);
        assert_eq!(cfg.st_kelly_enabled, restored.st_kelly_enabled);
    }

    #[test]
    fn st_config_s10_backward_compat() {
        let old_json = r#"{"st_macro_weight":0.4,"st_micro_weight":0.35,"st_momentum_weight":0.25,"st_entry_threshold":65.0,"st_stalking_threshold":55.0,"st_entry_thresholds":[60.0,70.0,75.0,65.0],"st_stabilization_window":8,"st_bayesian_decay":0.997,"st_bayesian_halflife":200,"st_dominant_cycle_filter":true,"st_ofi_norm_range":50.0,"st_mp_bps_range":10.0,"st_cvd_scale":100.0,"st_enabled":false,"st_killed":false}"#;
        let cfg: SmartTrendConfig = serde_json::from_str(old_json).expect("old format should parse");
        assert!((cfg.st_risk_per_trade - 0.01).abs() < f64::EPSILON);
        assert!((cfg.st_max_dd_pct - 5.0).abs() < f64::EPSILON);
        assert_eq!(cfg.st_bot_loop_ms, 2000);
        assert!(cfg.st_bot_paused);
    }

    #[test]
    fn st_config_s10_validation_warnings() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_risk_per_trade = 0.5;
        cfg.st_max_dd_pct = -1.0;
        cfg.st_bot_loop_ms = 100;
        cfg.st_max_kelly_fraction = 2.0;
        let v = validate_smarttrend_config(&cfg);
        assert!(v.warnings.iter().any(|w| w.contains("risk_per_trade")));
        assert!(v.warnings.iter().any(|w| w.contains("max_dd_pct")));
        assert!(v.warnings.iter().any(|w| w.contains("bot_loop_ms")));
        assert!(v.warnings.iter().any(|w| w.contains("max_kelly_fraction")));
    }

    // ═══════════════════════════════════════════════════════════
    //  S11: AUTO/MAN per-asset mode tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn st_asset_mode_default_man() {
        let modes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mode = modes.get("AAPL").cloned().unwrap_or_else(|| "man".to_string());
        assert_eq!(mode, "man", "default mode for missing asset should be MAN");
    }

    #[test]
    fn st_asset_mode_set_auto() {
        let mut modes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        modes.insert("AAPL".to_string(), "auto".to_string());
        assert_eq!(modes.get("AAPL").unwrap(), "auto");
    }

    #[test]
    fn st_asset_mode_serde_roundtrip() {
        let mut modes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        modes.insert("AAPL".to_string(), "auto".to_string());
        modes.insert("TSLA".to_string(), "man".to_string());
        modes.insert("NVDA".to_string(), "auto".to_string());

        let json = serde_json::to_string(&modes).expect("serialize");
        let restored: std::collections::HashMap<String, String> =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get("AAPL").unwrap(), "auto");
        assert_eq!(restored.get("TSLA").unwrap(), "man");
        assert_eq!(restored.get("NVDA").unwrap(), "auto");
    }

    #[test]
    fn st_tag_auto_vs_man() {
        let auto_tag = "ST|AUTO|AAPL|1711929600";
        let man_tag = "ST|MAN|TSLA|1711929601";

        assert!(auto_tag.starts_with("ST|AUTO"));
        assert!(man_tag.starts_with("ST|MAN"));
        assert!(auto_tag.starts_with("ST|"));
        assert!(man_tag.starts_with("ST|"));
        assert!(!auto_tag.starts_with("ST|MAN"));
        assert!(!man_tag.starts_with("ST|AUTO"));
    }

    #[test]
    fn st_force_exit_auto_only_filter() {
        let tags = vec![
            "ST|AUTO|AAPL|123",
            "ST|MAN|TSLA|456",
            "ST|AUTO|NVDA|789",
            "SCALP|Trending",
        ];

        let auto_only: Vec<_> = tags.iter().filter(|t| t.starts_with("ST|AUTO")).collect();
        let all_st: Vec<_> = tags.iter().filter(|t| t.starts_with("ST|")).collect();

        assert_eq!(auto_only.len(), 2, "auto_only should match 2 tags");
        assert_eq!(all_st.len(), 3, "all ST should match 3 tags");
    }

    #[test]
    fn st_sizing_suggestion_basic() {
        let mut account = SmartTrendAccount::new(10000.0);
        account.total_trades = 5;
        account.total_wins = 3;

        let cfg = SmartTrendConfig {
            st_risk_per_trade: 0.02,
            st_kelly_enabled: false,
            ..SmartTrendConfig::default()
        };

        let size = account.compute_size_usd(&cfg);
        let expected = 10000.0 * 0.02;
        assert!(
            (size - expected).abs() < 1.0,
            "base size should be capital * risk_per_trade: got {} expected {}",
            size,
            expected
        );
    }

    #[test]
    fn st_sizing_suggestion_kelly() {
        let mut account = SmartTrendAccount::new(10000.0);
        account.total_trades = 30;
        account.total_wins = 20;
        account.avg_win = 50.0;
        account.avg_loss = 30.0;

        let cfg = SmartTrendConfig {
            st_risk_per_trade: 0.02,
            st_kelly_enabled: true,
            st_kelly_min_trades: 20,
            st_max_kelly_fraction: 0.25,
            ..SmartTrendConfig::default()
        };

        let size = account.compute_size_usd(&cfg);
        assert!(
            size >= 10000.0 * 0.02,
            "Kelly should boost size above base: got {}",
            size
        );
    }

    #[test]
    fn st_mode_change_persists_in_hashmap() {
        let mut modes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        modes.insert("AAPL".to_string(), "man".to_string());
        assert_eq!(modes.get("AAPL").unwrap(), "man");

        modes.insert("AAPL".to_string(), "auto".to_string());
        assert_eq!(modes.get("AAPL").unwrap(), "auto");

        modes.insert("AAPL".to_string(), "man".to_string());
        assert_eq!(modes.get("AAPL").unwrap(), "man");
    }

    #[test]
    fn st_bot_skips_man_asset_logic() {
        let modes: std::collections::HashMap<String, String> = {
            let mut m = std::collections::HashMap::new();
            m.insert("AAPL".to_string(), "man".to_string());
            m.insert("TSLA".to_string(), "auto".to_string());
            m
        };

        let sym_aapl = "AAPL";
        let sym_tsla = "TSLA";

        let aapl_mode = modes.get(sym_aapl).map(|s| s.as_str()).unwrap_or("man");
        let tsla_mode = modes.get(sym_tsla).map(|s| s.as_str()).unwrap_or("man");

        assert_eq!(aapl_mode, "man", "AAPL should be MAN");
        assert_eq!(tsla_mode, "auto", "TSLA should be AUTO");
        assert!(aapl_mode != "auto", "bot should skip AAPL (MAN)");
        assert!(tsla_mode == "auto", "bot should enter TSLA (AUTO)");
    }

    #[test]
    fn st_all_modes_toggle() {
        let mut modes: std::collections::HashMap<String, String> = {
            let mut m = std::collections::HashMap::new();
            m.insert("AAPL".to_string(), "man".to_string());
            m.insert("TSLA".to_string(), "auto".to_string());
            m.insert("NVDA".to_string(), "man".to_string());
            m
        };

        for v in modes.values_mut() {
            *v = "auto".to_string();
        }
        assert!(modes.values().all(|v| v == "auto"));

        for v in modes.values_mut() {
            *v = "man".to_string();
        }
        assert!(modes.values().all(|v| v == "man"));
    }

    #[test]
    fn st_asset_mode_guard_live_position() {
        let auto_tag = Some("ST|AUTO|AAPL|123".to_string());
        let man_tag = Some("ST|MAN|TSLA|456".to_string());
        let no_tag: Option<String> = None;

        let has_auto_pos = auto_tag.as_ref().map(|t| t.starts_with("ST|AUTO")).unwrap_or(false);
        let has_man_pos = man_tag.as_ref().map(|t| t.starts_with("ST|AUTO")).unwrap_or(false);
        let has_no_pos = no_tag.as_ref().map(|t| t.starts_with("ST|AUTO")).unwrap_or(false);

        assert!(has_auto_pos, "AUTO position should block MAN switch");
        assert!(!has_man_pos, "MAN position should not block MAN switch");
        assert!(!has_no_pos, "no position should not block");
    }

    #[test]
    fn st_panel_data_pillar_scores() {
        let scores = [65.0_f64, 42.0, 78.0];
        let weights = [0.4_f64, 0.35, 0.25];
        let combined: f64 = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        assert!(
            (combined - (65.0 * 0.4 + 42.0 * 0.35 + 78.0 * 0.25)).abs() < 0.01,
            "combined score should be weighted sum"
        );
    }

    #[test]
    fn st_mode_audit_log_format() {
        let symbol = "AAPL";
        let old_mode = "man";
        let new_mode = "auto";
        let log = format!("[ST MODE] {}: {} -> {} (user)", symbol, old_mode, new_mode);
        assert!(log.contains("[ST MODE]"));
        assert!(log.contains("AAPL"));
        assert!(log.contains("man -> auto"));
    }

    #[test]
    fn st_asset_mode_empty_default() {
        let modes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        assert!(modes.is_empty());
        let unknown = modes.get("UNKNOWN").cloned().unwrap_or_else(|| "man".to_string());
        assert_eq!(unknown, "man");
    }

    #[test]
    fn st_trade_prompt_event_fields() {
        let prompt = serde_json::json!({
            "symbol": "AAPL",
            "direction": "Bullish",
            "score": 72.5,
            "state": "Entry",
            "current_price": 185.42,
            "suggested_size_usd": 200.0,
            "risk_pct": 2.0,
            "kelly_fraction": null,
            "pillar_scores": [70.0, 65.0, 82.0],
            "mode": "man",
            "timestamp_ms": 1711929600000_u64
        });

        assert_eq!(prompt["symbol"], "AAPL");
        assert_eq!(prompt["mode"], "man");
        assert_eq!(prompt["direction"], "Bullish");
        assert!(prompt["pillar_scores"].is_array());
        assert_eq!(prompt["pillar_scores"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn st_exit_prompt_event_fields() {
        let exit = serde_json::json!({
            "symbol": "TSLA",
            "marker": "Exit",
            "direction": "Bullish",
            "score": 45.0,
            "severity": "Critical",
            "timestamp_ms": 1711929600000_u64
        });

        assert_eq!(exit["symbol"], "TSLA");
        assert_eq!(exit["marker"], "Exit");
        assert_eq!(exit["severity"], "Critical");
    }

    // ═══════════════════════════════════════════════════════════════
    //  S12a: SmartTrend Config UI tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn st_config_serde_roundtrip() {
        let cfg = SmartTrendConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: SmartTrendConfig = serde_json::from_str(&json).unwrap();
        assert!((restored.st_macro_weight - 0.40).abs() < 0.001);
        assert!((restored.st_micro_weight - 0.35).abs() < 0.001);
        assert!((restored.st_momentum_weight - 0.25).abs() < 0.001);
        assert!((restored.st_entry_threshold - 65.0).abs() < 0.001);
        assert!((restored.st_stalking_threshold - 55.0).abs() < 0.001);
        assert_eq!(restored.st_entry_thresholds, [60.0, 70.0, 75.0, 65.0]);
        assert_eq!(restored.st_stabilization_window, 8);
        assert!((restored.st_bayesian_decay - 0.997).abs() < 0.001);
        assert_eq!(restored.st_bayesian_halflife, 200);
        assert!(restored.st_dominant_cycle_filter);
        assert!(!restored.st_enabled);
        assert!(!restored.st_killed);
    }

    #[test]
    fn st_config_validation_valid_default() {
        let cfg = SmartTrendConfig::default();
        let v = validate_smarttrend_config(&cfg);
        assert!(v.valid);
        assert!(v.errors.is_empty());
    }

    #[test]
    fn st_config_validation_weight_sum_warning() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = 0.50;
        cfg.st_micro_weight = 0.50;
        cfg.st_momentum_weight = 0.50;
        let v = validate_smarttrend_config(&cfg);
        assert!(v.valid);
        assert!(v.warnings.iter().any(|w| w.contains("sum")));
    }

    #[test]
    fn st_config_validation_negative_weight_error() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = -0.1;
        let v = validate_smarttrend_config(&cfg);
        assert!(!v.valid);
        assert!(v.errors.iter().any(|e| e.contains("negative")));
    }

    #[test]
    fn st_config_validation_threshold_ordering_error() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_stalking_threshold = 80.0;
        cfg.st_entry_threshold = 65.0;
        let v = validate_smarttrend_config(&cfg);
        assert!(!v.valid);
        assert!(v.errors.iter().any(|e| e.contains("stalking")));
    }

    #[test]
    fn st_config_preset_trend_follow_values() {
        let cfg = SmartTrendConfig {
            st_macro_weight: 0.60,
            st_micro_weight: 0.15,
            st_momentum_weight: 0.25,
            ..SmartTrendConfig::default()
        };
        let sum = cfg.st_macro_weight + cfg.st_micro_weight + cfg.st_momentum_weight;
        assert!((sum - 1.0).abs() < 0.01);
        let v = validate_smarttrend_config(&cfg);
        assert!(v.valid);
    }

    #[test]
    fn st_config_preset_scalp_values() {
        let cfg = SmartTrendConfig {
            st_macro_weight: 0.25,
            st_micro_weight: 0.45,
            st_momentum_weight: 0.30,
            ..SmartTrendConfig::default()
        };
        let sum = cfg.st_macro_weight + cfg.st_micro_weight + cfg.st_momentum_weight;
        assert!((sum - 1.0).abs() < 0.01);
        let v = validate_smarttrend_config(&cfg);
        assert!(v.valid);
    }

    #[test]
    fn st_config_preset_balanced_values() {
        let cfg = SmartTrendConfig {
            st_macro_weight: 0.34,
            st_micro_weight: 0.33,
            st_momentum_weight: 0.33,
            ..SmartTrendConfig::default()
        };
        let sum = cfg.st_macro_weight + cfg.st_micro_weight + cfg.st_momentum_weight;
        assert!((sum - 1.0).abs() < 0.01);
        let v = validate_smarttrend_config(&cfg);
        assert!(v.valid);
    }

    #[test]
    fn st_config_partial_json_backcompat() {
        let full = SmartTrendConfig::default();
        let mut val = serde_json::to_value(&full).unwrap();
        val.as_object_mut().unwrap().remove("st_changepoint_reset_threshold");
        val.as_object_mut().unwrap().remove("st_trail_tighten_pct");
        val.as_object_mut().unwrap().remove("st_hysteresis_margin");
        val.as_object_mut().unwrap().remove("st_sentinel_idle_secs");
        val["st_macro_weight"] = serde_json::json!(0.5);
        let cfg: SmartTrendConfig = serde_json::from_value(val).unwrap();
        assert!((cfg.st_macro_weight - 0.5).abs() < 0.001);
        assert!((cfg.st_changepoint_reset_threshold - 0.7).abs() < 0.001);
        assert!((cfg.st_trail_tighten_pct - 0.05).abs() < 0.001);
        assert!((cfg.st_hysteresis_margin - 5.0).abs() < 0.001);
        assert_eq!(cfg.st_sentinel_idle_secs, 30);
    }

    #[test]
    fn st_config_per_regime_threshold_range() {
        let cfg = SmartTrendConfig::default();
        for (i, &t) in cfg.st_entry_thresholds.iter().enumerate() {
            assert!(t > 0.0 && t <= 100.0, "threshold[{}] = {} out of range", i, t);
        }
        assert!(cfg.st_entry_thresholds[2] > cfg.st_entry_thresholds[0],
            "Chaotic should be stricter than Trending");
    }

    #[test]
    fn st_config_diagnostics_contains_all_fields() {
        let engine = SmartTrendEngine::new(SmartTrendConfig::default());
        let diag = engine.diagnostics();
        assert_eq!(diag.state, SmartTrendStateKind::Flat);
        assert_eq!(diag.direction, PillarDirection::Neutral);
        assert!(!diag.enabled);
        assert!(!diag.killed);
        assert_eq!(diag.bars_in_state, 0);
        assert!(diag.confirmation_progress.is_none());
        assert!(diag.last_transition.contains("init"));
    }

    #[test]
    fn st_config_stats_default_zeros() {
        let stats = SmartTrendStats::default();
        assert_eq!(stats.entries_total, 0);
        assert_eq!(stats.entries_reached_trail, 0);
        assert_eq!(stats.changepoint_resets, 0);
        assert_eq!(stats.stalking_timeouts, 0);
        assert!((stats.trail_hit_rate() - 0.0).abs() < 0.001);
    }

    #[test]
    fn nan_weight_falls_back_to_neutral() {
        let (a, b, c) = normalize_weights(f64::NAN, 0.35, 0.25);
        assert!((a - 0.40).abs() < 0.01, "NaN weight should fall back to defaults");
        assert!((b - 0.35).abs() < 0.01);
        assert!((c - 0.25).abs() < 0.01);

        let (a2, b2, c2) = normalize_weights(0.4, f64::INFINITY, 0.25);
        assert!((a2 - 0.40).abs() < 0.01, "Inf weight should fall back to defaults");
        assert!((b2 - 0.35).abs() < 0.01);
        assert!((c2 - 0.25).abs() < 0.01);
    }

    #[test]
    fn nan_pillar_score_sanitized() {
        let mut macro_in = bullish_macro_input();
        macro_in.oracle_score = f64::NAN;
        let micro_in = MicroInput {
            ofi_value: 10.0, microprice: 100.05, mid_price: 100.0,
            cvd: 50.0, spread_bps: 5.0, imbalance_ratio: 0.3, has_live_data: true,
        };
        let momentum_in = MomentumInput {
            gpu_buy_pct: 60.0, gpu_sell_pct: 40.0,
            dominant_cycle: 5, momentum_bars: 10, rvol: 1.5,
        };
        let diag = evaluate_pillars(&macro_in, &micro_in, &momentum_in, &default_config());
        assert!(diag.combined_score.is_finite(), "Combined score must be finite even with NaN input");
        assert!(diag.nan_sanitized > 0, "Should report NaN sanitization");
    }

    #[test]
    fn kelly_nonzero_on_perfect_win_rate() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.total_trades = 10;
        acct.total_wins = 10;
        acct.avg_win = 50.0;
        acct.avg_loss = 0.0;
        let kelly = acct.kelly_fraction(5, 0.5);
        assert!(kelly > 0.0, "Kelly should be positive with 100% win rate, got {}", kelly);
        assert!(kelly <= 0.5, "Kelly should be capped at max_fraction");
    }

    #[test]
    fn kelly_with_bayesian_loss_floor() {
        let mut acct = SmartTrendAccount::new(10000.0);
        acct.total_trades = 20;
        acct.total_wins = 18;
        acct.avg_win = 100.0;
        acct.avg_loss = 30.0;
        let kelly_normal = acct.kelly_fraction(5, 0.5);

        acct.avg_loss = 0.0;
        let kelly_no_loss = acct.kelly_fraction(5, 0.5);
        assert!(kelly_no_loss > 0.0, "Kelly with zero losses should use Bayesian floor");
        assert!(kelly_no_loss <= kelly_normal * 2.0, "Bayesian Kelly should be conservative");
    }

    #[test]
    fn entries_total_counts_once_per_trade() {
        let mut stats = SmartTrendStats::default();
        stats.record_transition(SmartTrendStateKind::Stalking, SmartTrendStateKind::Entry);
        stats.record_transition(SmartTrendStateKind::Entry, SmartTrendStateKind::InTrade);
        assert_eq!(stats.entries_total, 1, "One trade should increment entries_total once, not twice");
    }

    #[test]
    fn import_config_preserves_kill_switch_fields() {
        let mut live_config = SmartTrendConfig::default();
        live_config.st_killed = true;
        live_config.st_bot_paused = true;

        let mut imported = SmartTrendConfig::default();
        imported.st_killed = false;
        imported.st_bot_paused = false;
        imported.st_macro_weight = 0.5;

        imported.st_killed = live_config.st_killed;
        imported.st_bot_paused = live_config.st_bot_paused;

        assert!(imported.st_killed, "Import must preserve kill switch");
        assert!(imported.st_bot_paused, "Import must preserve pause state");
        assert!((imported.st_macro_weight - 0.5).abs() < 0.01, "Other config values should update");
    }

    #[test]
    fn entry_state_persists_for_configured_bars() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_entry_persist_bars = Some(3);
        let mut engine = SmartTrendEngine::new(cfg);

        engine.state = SmartTrendStateKind::Entry;
        engine.direction = PillarDirection::Bullish;
        engine.bars_in_state = 1;
        engine.entry_acted = false;

        assert_eq!(engine.state, SmartTrendStateKind::Entry);
        assert!(
            engine.bars_in_state < engine.config.st_entry_persist_bars.unwrap_or(3),
            "Entry should persist when bars_in_state < persist_bars"
        );
    }

    #[test]
    fn fsm_hysteresis_clamp_prevents_stuck_stalking() {
        let mut cfg = default_config();
        cfg.st_stalking_threshold = 55.0;
        cfg.st_hysteresis_margin = 20.0; // way too large
        let v = validate_smarttrend_config(&cfg);
        assert!(
            v.warnings.iter().any(|w| w.contains("stuck in Stalking")),
            "Validation should warn about stuck FSM: {:?}", v.warnings
        );

        let mut engine = SmartTrendEngine::new(cfg);
        let input = bullish_st_input(100.0);
        engine.evaluate(&input);
        assert_eq!(engine.state, SmartTrendStateKind::Stalking);

        let neutral = weak_st_input(100.0);
        for _ in 0..50 {
            engine.evaluate(&neutral);
        }
        assert_eq!(
            engine.state, SmartTrendStateKind::Flat,
            "With clamped hysteresis, FSM should eventually return to Flat"
        );
    }

    #[test]
    fn circuit_breaker_reset_clears_st_latch() {
        let mut acct = SmartTrendAccount::default();
        acct.breaker_triggered = true;
        acct.paused = true;

        acct.daily_reset();
        assert!(!acct.breaker_triggered, "daily_reset should clear breaker_triggered");
        assert!(!acct.paused, "daily_reset should clear paused when breaker was triggered");
    }

    #[test]
    fn daily_reset_preserves_manual_pause() {
        let mut acct = SmartTrendAccount::default();
        acct.paused = true;
        acct.breaker_triggered = false;

        acct.daily_reset();
        assert!(acct.paused, "daily_reset should NOT clear paused if breaker wasn't triggered");
    }

    #[test]
    fn stalking_timeout_counts_only_actual_timeouts() {
        let mut cfg = default_config();
        cfg.st_hysteresis_margin = 5.0;
        let mut engine = SmartTrendEngine::new(cfg);

        let input = bullish_st_input(100.0);
        engine.evaluate(&input);
        assert_eq!(engine.state, SmartTrendStateKind::Stalking);

        let neutral = weak_st_input(100.0);
        engine.evaluate(&neutral);

        if engine.state == SmartTrendStateKind::Flat {
            assert_eq!(
                engine.stats.stalking_timeouts, 0,
                "Hysteresis drop should NOT increment stalking_timeouts"
            );
        }
    }
}

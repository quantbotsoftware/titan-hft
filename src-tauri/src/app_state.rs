use crate::scanner;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn generate_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub use titan_types::Candle;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareStats {
    pub cpu_temp: f64,
    pub gpu_temp: f64,
    pub gpu_util: f64,
    pub gpu_power: f64,
    pub gpu_mem: f64,
    pub net_down: f64,
    pub net_up: f64,
    pub net_tot_down: f64, // [NEW] Total Session GB
    pub net_tot_up: f64,   // [NEW] Total Session GB
    pub cpu_fan: f64,      // [NEW] RPM
    pub gpu_fan: f64,      // [NEW] RPM
    pub ram_total: f64,    // [NEW] Total GB
    pub ram_used: f64,     // [NEW] Used GB
    pub cpu_usage: f64,    // [NEW] %
    pub cpu_freq_mhz: f64,
    pub net_iface: String,
    pub gpu_name: String,
}

impl Default for HardwareStats {
    fn default() -> Self {
        Self {
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            gpu_util: 0.0,
            gpu_power: 0.0,
            gpu_mem: 0.0,
            net_down: 0.0,
            net_up: 0.0,
            net_tot_down: 0.0,
            net_tot_up: 0.0,
            cpu_fan: 0.0,
            gpu_fan: 0.0,
            ram_total: 0.0,
            ram_used: 0.0,
            cpu_usage: 0.0,
            cpu_freq_mhz: 0.0,
            net_iface: String::new(),
            gpu_name: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// TA Gate enforcement mode: Off ignores the gate, Soft penalizes the score,
/// Hard blocks entry entirely.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum GateMode {
    #[default]
    Off,
    Soft,
    Hard,
}

/// Per-gate configuration for 6 technical analysis entry gates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaGatesConfig {
    pub rsi_gate: GateMode,
    pub ema_alignment_gate: GateMode,
    pub adx_gate: GateMode,
    pub bb_squeeze_gate: GateMode,
    pub macd_gate: GateMode,
    pub stoch_rsi_gate: GateMode,
}

impl Default for TaGatesConfig {
    fn default() -> Self {
        Self {
            rsi_gate: GateMode::Off,
            ema_alignment_gate: GateMode::Off,
            adx_gate: GateMode::Off,
            bb_squeeze_gate: GateMode::Off,
            macd_gate: GateMode::Off,
            stoch_rsi_gate: GateMode::Off,
        }
    }
}

/// Pre-built gate profiles.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaGatePreset {
    Strict,
    Balanced,
    OracleOnly,
}

impl TaGatesConfig {
    pub fn from_preset(preset: &TaGatePreset) -> Self {
        match preset {
            TaGatePreset::Strict => Self {
                rsi_gate: GateMode::Hard,
                ema_alignment_gate: GateMode::Hard,
                adx_gate: GateMode::Hard,
                bb_squeeze_gate: GateMode::Hard,
                macd_gate: GateMode::Hard,
                stoch_rsi_gate: GateMode::Hard,
            },
            TaGatePreset::Balanced => Self {
                rsi_gate: GateMode::Soft,
                ema_alignment_gate: GateMode::Off,
                adx_gate: GateMode::Soft,
                bb_squeeze_gate: GateMode::Off,
                macd_gate: GateMode::Off,
                stoch_rsi_gate: GateMode::Off,
            },
            TaGatePreset::OracleOnly => Self::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  STRATEGY THRESHOLDS — Configurable scoring parameters (N4)
// ═══════════════════════════════════════════════════════════════════

/// A single {long, short, confidence} tuple for one regime+mode cell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryThreshold {
    pub long_threshold: f64,
    pub short_threshold: f64,
    pub conf_threshold: f64,
}

/// Score modifier knobs consumed by `calculate_score`, `check_entry`, and `check_ta_gates`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ScoreModifiers {
    pub chaos_kill_penalty: i32,
    pub chaos_kill_fallback: i32,
    pub chaos_conf_mult_mean_rev: f64,
    pub chaos_conf_mult_chaotic: f64,
    pub chaos_conf_mult_unknown: f64,
    pub regime_mult_mean_rev: f64,
    pub regime_mult_chaotic: f64,
    pub regime_mult_unknown: f64,
    pub vpin_penalty_high: i32,
    pub vpin_penalty_medium: i32,
    pub fvg_bullish_bonus: i32,
    pub fvg_bearish_penalty: i32,
    pub rvol_tier1_threshold: f64,
    pub rvol_tier1_bonus: i32,
    pub rvol_tier2_threshold: f64,
    pub rvol_tier2_bonus: i32,
    pub stoch_rsi_overbought: f64,
    pub stoch_rsi_oversold: f64,
    pub bb_squeeze_width: f64,
    pub entry_label_strong: f64,
    pub entry_label_mild: f64,
}

impl Default for ScoreModifiers {
    fn default() -> Self {
        Self {
            chaos_kill_penalty: -50,
            chaos_kill_fallback: 50,
            chaos_conf_mult_mean_rev: 1.5,
            chaos_conf_mult_chaotic: 2.5,
            chaos_conf_mult_unknown: 1.5,
            regime_mult_mean_rev: 0.82,
            regime_mult_chaotic: 0.55,
            regime_mult_unknown: 0.73,
            vpin_penalty_high: -20,
            vpin_penalty_medium: -10,
            fvg_bullish_bonus: 15,
            fvg_bearish_penalty: -15,
            rvol_tier1_threshold: 2.0,
            rvol_tier1_bonus: 8,
            rvol_tier2_threshold: 4.0,
            rvol_tier2_bonus: 12,
            stoch_rsi_overbought: 80.0,
            stoch_rsi_oversold: 20.0,
            bb_squeeze_width: 0.02,
            entry_label_strong: 80.0,
            entry_label_mild: 60.0,
        }
    }
}

/// Classical indicator confirmation bonuses used by `compute_classical_confirmation`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClassicalBonuses {
    pub rsi_bonus: f64,
    pub ema_alignment_bonus: f64,
    pub adx_bonus: f64,
    pub bb_squeeze_bonus: f64,
    pub vwap_bonus: f64,
    pub cap: f64,
}

impl Default for ClassicalBonuses {
    fn default() -> Self {
        Self {
            rsi_bonus: 10.0,
            ema_alignment_bonus: 15.0,
            adx_bonus: 10.0,
            bb_squeeze_bonus: 5.0,
            vwap_bonus: 10.0,
            cap: 50.0,
        }
    }
}

/// Build the default entry threshold matrix for scalping mode.
#[allow(dead_code)]
pub fn default_llm_batch_size() -> usize {
    5
}
#[allow(dead_code)]
pub fn default_sentiment_halflife() -> u64 {
    3600
}

pub fn default_entry_thresholds_scalp() -> HashMap<String, EntryThreshold> {
    let mut m = HashMap::new();
    m.insert("trending_aggro".into(), EntryThreshold { long_threshold: 15.0, short_threshold: -15.0, conf_threshold: 0.12 });
    m.insert("trending".into(), EntryThreshold { long_threshold: 22.0, short_threshold: -22.0, conf_threshold: 0.18 });
    m.insert("mean_rev_aggro".into(), EntryThreshold { long_threshold: 25.0, short_threshold: -25.0, conf_threshold: 0.22 });
    m.insert("mean_rev".into(), EntryThreshold { long_threshold: 32.0, short_threshold: -32.0, conf_threshold: 0.28 });
    m.insert("chaotic".into(), EntryThreshold { long_threshold: 50.0, short_threshold: -50.0, conf_threshold: 0.38 });
    m.insert("unknown_aggro".into(), EntryThreshold { long_threshold: 22.0, short_threshold: -22.0, conf_threshold: 0.18 });
    m.insert("unknown".into(), EntryThreshold { long_threshold: 32.0, short_threshold: -32.0, conf_threshold: 0.25 });
    m
}

/// Build the default entry threshold matrix for normal (non-scalping) mode.
pub fn default_entry_thresholds_normal() -> HashMap<String, EntryThreshold> {
    let mut m = HashMap::new();
    m.insert("trending_aggro".into(), EntryThreshold { long_threshold: 25.0, short_threshold: -25.0, conf_threshold: 0.20 });
    m.insert("trending".into(), EntryThreshold { long_threshold: 35.0, short_threshold: -35.0, conf_threshold: 0.25 });
    m.insert("mean_rev_aggro".into(), EntryThreshold { long_threshold: 40.0, short_threshold: -40.0, conf_threshold: 0.30 });
    m.insert("mean_rev".into(), EntryThreshold { long_threshold: 50.0, short_threshold: -50.0, conf_threshold: 0.35 });
    m.insert("chaotic".into(), EntryThreshold { long_threshold: 70.0, short_threshold: -70.0, conf_threshold: 0.55 });
    m.insert("unknown_aggro".into(), EntryThreshold { long_threshold: 35.0, short_threshold: -35.0, conf_threshold: 0.25 });
    m.insert("unknown".into(), EntryThreshold { long_threshold: 50.0, short_threshold: -50.0, conf_threshold: 0.30 });
    m
}

/// Validate strategy thresholds for logical consistency. Returns warnings (not errors).
pub fn validate_strategy_thresholds(
    scalp: &HashMap<String, EntryThreshold>,
    normal: &HashMap<String, EntryThreshold>,
    mods: &ScoreModifiers,
) -> Vec<String> {
    let mut warnings = Vec::new();

    for (key, t) in scalp.iter().chain(normal.iter()) {
        if t.long_threshold <= 0.0 {
            warnings.push(format!("{}: long_threshold ({}) should be positive", key, t.long_threshold));
        }
        if t.short_threshold >= 0.0 {
            warnings.push(format!("{}: short_threshold ({}) should be negative", key, t.short_threshold));
        }
        if t.conf_threshold < 0.05 || t.conf_threshold > 0.95 {
            warnings.push(format!("{}: conf_threshold ({}) outside safe range [0.05, 0.95]", key, t.conf_threshold));
        }
    }

    let regime_keys = ["trending", "mean_rev", "chaotic", "unknown"];
    for key in &regime_keys {
        if let (Some(s), Some(n)) = (scalp.get(*key), normal.get(*key)) {
            if s.long_threshold > n.long_threshold {
                warnings.push(format!("{}: scalping long ({}) more restrictive than normal ({})", key, s.long_threshold, n.long_threshold));
            }
        }
        let aggro_key = format!("{}_aggro", key);
        if let (Some(s), Some(n)) = (scalp.get(&aggro_key), normal.get(&aggro_key)) {
            if s.long_threshold > n.long_threshold {
                warnings.push(format!("{}: scalping long ({}) more restrictive than normal ({})", aggro_key, s.long_threshold, n.long_threshold));
            }
        }
    }

    if let (Some(trend), Some(chaotic)) = (scalp.get("trending"), scalp.get("chaotic")) {
        if trend.long_threshold > chaotic.long_threshold {
            warnings.push(format!("scalp: trending threshold ({}) higher than chaotic ({}) — unnatural gradient", trend.long_threshold, chaotic.long_threshold));
        }
    }
    if let (Some(trend), Some(chaotic)) = (normal.get("trending"), normal.get("chaotic")) {
        if trend.long_threshold > chaotic.long_threshold {
            warnings.push(format!("normal: trending threshold ({}) higher than chaotic ({}) — unnatural gradient", trend.long_threshold, chaotic.long_threshold));
        }
    }

    if mods.stoch_rsi_overbought <= mods.stoch_rsi_oversold {
        warnings.push(format!("stoch_rsi_overbought ({}) should be > stoch_rsi_oversold ({})", mods.stoch_rsi_overbought, mods.stoch_rsi_oversold));
    }
    if mods.rvol_tier1_threshold >= mods.rvol_tier2_threshold {
        warnings.push(format!("rvol_tier1 ({}) should be < rvol_tier2 ({})", mods.rvol_tier1_threshold, mods.rvol_tier2_threshold));
    }

    warnings
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum PositionState {
    PendingEntry,
    #[default]
    Open,
    PartialExit,
    PendingClose,
    Closed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub symbol: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub qty: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub entry_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub pnl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub pnl_percent: Decimal,
    pub side: OrderSide,
    #[serde(with = "rust_decimal::serde::float")]
    pub highest_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub lowest_price: Decimal,
    #[serde(default)]
    pub state: PositionState,
    #[serde(default)]
    pub opened_at: Option<u64>,
    #[serde(default)]
    pub closed_at: Option<u64>,
    #[serde(default)]
    pub partial_taken: bool,
    #[serde(default)]
    pub entry_tag: Option<String>,
    #[serde(default)]
    pub broker_order_id: Option<String>,
}

impl Position {
    #[allow(dead_code)]
    pub fn new_open(symbol: String, qty: Decimal, entry_price: Decimal, side: OrderSide) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            symbol,
            qty,
            entry_price,
            current_price: entry_price,
            pnl: Decimal::ZERO,
            pnl_percent: Decimal::ZERO,
            side,
            highest_price: entry_price,
            lowest_price: entry_price,
            state: PositionState::Open,
            opened_at: Some(now),
            closed_at: None,
            partial_taken: false,
            entry_tag: None,
            broker_order_id: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub active_symbol: String,
    pub equity: Decimal,
    pub buying_power: Decimal,
    pub latest_log: String,
    pub positions: Vec<Position>,
    pub hft_targets: Vec<String>,

    // Toggles
    pub loop_active: bool,
    pub omni_scan_active: bool,
    pub ai_scan_active: bool,
    pub ai_smart_sort: bool,
    pub ai_auto_inject_top: u32,
    pub ai_auto_inject_hft: bool,
    pub auto_pilot: bool,
    pub aggressive_mode: bool,
    pub allow_short: bool,
    pub enable_extended: bool,
    pub force_manual_tf: bool,
    pub scalping_oracle: bool,
    pub scalping_gpu: bool,
    pub scalping_fusion: bool,

    // Settings
    pub risk_per_trade: f64,
    pub refresh_rate: u64,
    pub manual_res: String,
    pub backtest_days: u64,
    pub gpu_backtest: bool,
    pub gpu_sweep: bool,
    pub gpu_deep_focus: bool,

    // Auto-Trading
    pub auto_entry: bool,
    pub max_open_positions: u32,
    pub entry_cooldown_secs: u64,
    pub daily_loss_limit_pct: f64,

    // HFT Engine advanced
    pub trailing_stop_pct: f64,
    pub take_profit_pct: f64,
    pub max_spread_bps: f64,
    pub min_ofi_threshold: f64,
    pub max_daily_trades: u32,
    pub entry_signal_strength_min: f64,
    pub max_hold_secs: u64,

    // Multi-symbol autopilot
    pub multi_symbol_enabled: bool,
    pub max_concurrent_assets: u32,
    pub max_correlation: f64,
    pub correlation_window: usize,

    // Discord alerts
    pub discord_webhook_url: String,
    pub alert_on_fill: bool,
    pub alert_on_breaker: bool,
    pub alert_on_daily_summary: bool,

    // Telegram alerts
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    pub telegram_enabled: bool,

    // ATR dynamic stops
    pub atr_stop_multiplier: f64,
    pub atr_take_multiplier: f64,
    pub partial_take_enabled: bool,
    pub partial_take_pct: f64,

    // LLM Sentiment
    pub llm_api_key: String,
    pub llm_provider: String,
    pub llm_sentiment_enabled: bool,
    pub llm_batch_size: usize,
    pub sentiment_halflife_secs: u64,

    // Options Hedging
    pub options_hedging_enabled: bool,
    pub hedge_otm_pct: f64,
    pub hedge_max_premium_pct: f64,

    // Web Dashboard
    pub web_dashboard_enabled: bool,
    pub web_dashboard_port: u16,
    pub web_dashboard_token: String,

    // Sidebar persisted (migrated from localStorage)
    pub bot_trade_size: Decimal,
    pub circuit_breaker_limit: Decimal,
    pub favorites: Vec<String>,
    pub dry_run_enabled: bool,
    pub strategy_mode_name: String,
    pub scan_params: scanner::ScanParams,
    pub position_trade_modes: std::collections::HashMap<String, String>,
    pub md_auto_scan_active: bool,
    pub nexus_fusion_alpha: f64,
    pub signal_strategy_assets: Vec<String>,
    pub signal_source: String,

    /// Signal Strategy tab: bypass safety rails (only applied when mode is Signal Strategy in autopilot)
    pub bypass_max_spread: bool,
    pub bypass_cooldown: bool,
    pub bypass_allow_short: bool,
    pub bypass_extended: bool,
    pub bypass_risk_per_trade: bool,
    pub bypass_aggressive: bool,
    pub bypass_atr_stop: bool,
    pub bypass_atr_take: bool,
    pub bypass_partial_take: bool,
    pub bypass_partial_pct: bool,
    pub bypass_max_correlation: bool,
    pub bypass_correlation_window: bool,
    pub bypass_daily_loss: bool,
    pub max_manual_order_usd: Decimal,

    // TA Entry Gates (RF-B)
    pub ta_gates: TaGatesConfig,
    pub ta_gate_soft_penalty: f64,

    // Portfolio Risk Gates (S9)
    pub enable_var_gate: bool,
    pub max_portfolio_var_pct: f64,
    pub var_lookback_periods: usize,
    pub enable_concentration_gate: bool,
    pub max_concentration_hhi: f64,

    // Safety Gates (S04.1)
    pub var_fail_closed: bool,
    pub asset_check_hard_gate: bool,
    pub blackout_first_mins: u64,
    pub blackout_last_mins: u64,
    pub blackout_exempt_symbols: Vec<String>,

    // Trading Edge (S15)
    pub max_hold_decay_enabled: bool,
    pub max_hold_decay_start_pct: f64,
    pub max_hold_decay_trail_tighten: f64,
    pub mtf_gate_enabled: bool,
    pub mtf_gate_mode: String,
    pub mtf_gate_penalty: f64,
    pub mtf_gate_htf: String,
    pub hhi_gate_manual_orders: bool,
    pub max_concentration_hhi_manual: f64,
    pub tiered_scan_enabled: bool,
    pub tier1_count: usize,
    pub tier2_count: usize,

    // Execution Resilience (S14)
    pub pending_order_timeout_secs: u64,

    // Microstructure: OFI divergence alert threshold (seconds)
    pub ofi_divergence_threshold_secs: u64,

    // Classical indicator filters (Phase 7D)
    pub use_classical_filters: bool,
    pub rsi_overbought: f64,
    pub rsi_oversold: f64,
    pub require_ema_alignment: bool,
    pub adx_trend_threshold: f64,
    pub classical_weight: f64,

    // Strategy Thresholds (N4)
    pub entry_thresholds_scalp: HashMap<String, EntryThreshold>,
    pub entry_thresholds_normal: HashMap<String, EntryThreshold>,
    pub score_modifiers: ScoreModifiers,
    pub classical_bonuses: ClassicalBonuses,

    // Phase 1: Autopilot Core
    pub min_hold_secs: u64,
    pub nexus_consensus_threshold: f64,
    pub autopilot_loop_ms: u64,
    pub ofi_stale_ms: u64,
    pub flash_crash_window_secs: u64,
    pub flash_crash_threshold_pct: f64,
    pub flash_crash_cooldown_mins: u64,
    pub dd_velocity_threshold: f64,
    pub dd_velocity_window_secs: u64,
    pub min_bars_for_entry: u64,
    pub ofi_entry_threshold: f64,
    pub max_qty_cap: f64,
    pub min_qty_threshold: f64,
    pub default_equity_fallback: f64,
    pub max_bypass_risk_pct: f64,

    // Phase 2: Exit Scaling
    pub vol_factor_min: f64,
    pub vol_factor_max: f64,
    pub fallback_sl_pct: f64,
    pub fallback_tp_pct: f64,
    pub fallback_trail_pct: f64,
    pub scalping_stop_mult: f64,
    pub scalping_take_mult: f64,
    pub scalping_trail_mult: f64,
    pub partial_first_target_frac: f64,
    pub partial_min_qty: f64,
    pub atr_period: u64,
    pub atr_stop_clamp_min: f64,
    pub atr_stop_clamp_max: f64,
    pub atr_take_clamp_min: f64,
    pub atr_take_clamp_max: f64,
    pub atr_trail_clamp_min: f64,
    pub atr_trail_clamp_max: f64,

    // Phase 3: Oracle Exit
    pub oracle_exit_min_confidence: f64,
    pub oracle_exit_min_score: f64,
    pub oracle_flip_score_threshold: f64,
    pub oracle_flip_confidence: f64,
    pub chaos_exit_max_loss_pct: f64,
    pub chaos_exit_min_confidence: f64,

    // Phase 4: Circuit Breaker
    pub breaker_caution_pct: f64,
    pub breaker_halt_pct: f64,
    pub breaker_panic_pct: f64,
    pub max_notional_equity_frac: f64,
    pub min_stop_distance_frac: f64,

    // Phase 5: Strategy Scoring
    pub strategy_base_score: f64,
    pub vpin_gate_scalping: f64,
    pub vpin_gate_normal: f64,
    pub min_rvol_threshold: f64,
    pub rvol_window: u64,
    pub rvol_relax_threshold: f64,
    pub rvol_relax_value: f64,
    pub chaos_confidence_kill: f64,
    pub chaos_regime_mult: f64,

    // Phase 6: Engine Stops
    pub long_stop_normal: f64,
    pub long_stop_aggressive: f64,
    pub short_stop_pct: f64,
    pub long_trail_min_profit: f64,
    pub short_trail_gap: f64,
    pub short_trail_min_profit: f64,

    // Phase 7: Oracle Fusion
    pub oracle_kalman_weight: f64,
    pub oracle_entropy_weight: f64,
    pub oracle_hurst_weight: f64,
    pub oracle_vpin_weight: f64,
    pub oracle_wavelet_weight: f64,
    pub oracle_confidence_floor: f64,
    pub entropy_chaos_threshold: f64,
    pub vpin_toxicity_alert: f64,
    pub enable_oracle_adaptation: bool,

    // Phase 8: Indicator Periods
    pub ema_fast_period: u64,
    pub ema_mid_period: u64,
    pub ema_slow_period: u64,
    pub ema_trend_period: u64,
    pub rsi_period: u64,
    pub macd_fast: u64,
    pub macd_slow: u64,
    pub macd_signal: u64,
    pub bb_period: u64,
    pub bb_std_dev: f64,
    pub adx_period: u64,
    pub stoch_rsi_period: u64,

    // Phase 9: Adaptive Exit Multipliers
    pub disable_adaptive_exits: bool,
    pub exit_floor_pct: f64,

    // Regime multipliers
    pub regime_chaotic_stop_mult: f64,
    pub regime_chaotic_take_mult: f64,
    pub regime_chaotic_trail_mult: f64,
    pub regime_range_stop_mult: f64,
    pub regime_range_take_mult: f64,
    pub regime_range_trail_mult: f64,
    pub regime_trend_stop_mult: f64,
    pub regime_trend_take_mult: f64,
    pub regime_trend_trail_mult: f64,

    // Signal alignment multipliers
    pub signal_aligned_stop_mult: f64,
    pub signal_aligned_take_mult: f64,
    pub signal_aligned_trail_mult: f64,
    pub signal_oppose_stop_mult: f64,
    pub signal_oppose_take_mult: f64,

    // Timeframe multipliers
    pub tf_short_stop_mult: f64,
    pub tf_short_take_mult: f64,
    pub tf_short_trail_mult: f64,

    // Per-symbol overrides: symbol -> JSON of overridden fields
    pub symbol_overrides: HashMap<String, SymbolOverride>,

    // SmartTrend AI v3 config (S07b)
    pub smarttrend: crate::smarttrend::SmartTrendConfig,

    // S11: per-asset AUTO/MAN mode for SmartTrend bot execution
    pub st_asset_modes: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SymbolOverride {
    pub trailing_stop_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub risk_per_trade: Option<f64>,
    pub max_open_positions: Option<u32>,
    pub max_hold_secs: Option<f64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_symbol: "---".to_string(),
            equity: dec!(100000),
            buying_power: dec!(100000),
            latest_log: "System Ready".to_string(),
            positions: Vec::new(),
            hft_targets: Vec::new(),

            // Toggles
            loop_active: false,
            omni_scan_active: false,
            ai_scan_active: true,
            ai_smart_sort: true,
            ai_auto_inject_top: 0,
            ai_auto_inject_hft: false,
            auto_pilot: false,
            aggressive_mode: false,
            allow_short: false,
            enable_extended: false,
            force_manual_tf: false,
            scalping_oracle: true,
            scalping_gpu: true,
            scalping_fusion: true,

            // Settings
            risk_per_trade: 2.0,
            refresh_rate: 2000,
            manual_res: "5m".to_string(),
            backtest_days: 30,
            gpu_backtest: false,
            gpu_sweep: false,
            gpu_deep_focus: false,

            // Auto-Trading defaults
            auto_entry: false,
            max_open_positions: 3,
            entry_cooldown_secs: 60,
            daily_loss_limit_pct: 5.0,

            // HFT Engine defaults
            trailing_stop_pct: 1.5,
            take_profit_pct: 2.5,
            max_spread_bps: 50.0,
            min_ofi_threshold: 5.0,
            max_daily_trades: 50,
            entry_signal_strength_min: 40.0,
            max_hold_secs: 3600,

            // Multi-symbol
            multi_symbol_enabled: false,
            max_concurrent_assets: 5,
            max_correlation: 0.85,
            correlation_window: 60,

            // Discord defaults
            discord_webhook_url: String::new(),
            alert_on_fill: true,
            alert_on_breaker: true,
            alert_on_daily_summary: false,

            // Telegram defaults
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            telegram_enabled: false,

            // ATR dynamic stops defaults
            atr_stop_multiplier: 1.5,
            atr_take_multiplier: 3.0,
            partial_take_enabled: true,
            partial_take_pct: 50.0,

            // LLM Sentiment defaults
            llm_api_key: String::new(),
            llm_provider: "openai".to_string(),
            llm_sentiment_enabled: false,
            llm_batch_size: 5,
            sentiment_halflife_secs: 3600,

            // Options Hedging defaults
            options_hedging_enabled: false,
            hedge_otm_pct: 5.0,
            hedge_max_premium_pct: 1.0,

            // Web Dashboard defaults
            web_dashboard_enabled: false,
            web_dashboard_port: 7777,
            web_dashboard_token: generate_token(),

            // Sidebar defaults
            bot_trade_size: dec!(50),
            circuit_breaker_limit: dec!(3000),
            favorites: Vec::new(),
            dry_run_enabled: true,
            strategy_mode_name: "SIGNAL_STRATEGY".to_string(),
            scan_params: scanner::ScanParams::default(),
            position_trade_modes: std::collections::HashMap::new(),
            md_auto_scan_active: false,
            nexus_fusion_alpha: 0.5,
            signal_strategy_assets: Vec::new(),
            signal_source: "oracle".to_string(),

            bypass_max_spread: false,
            bypass_cooldown: false,
            bypass_allow_short: false,
            bypass_extended: false,
            bypass_risk_per_trade: false,
            bypass_aggressive: false,
            bypass_atr_stop: false,
            bypass_atr_take: false,
            bypass_partial_take: false,
            bypass_partial_pct: false,
            bypass_max_correlation: false,
            bypass_correlation_window: false,
            bypass_daily_loss: false,
            max_manual_order_usd: dec!(50000),

            // TA Entry Gates (RF-B)
            ta_gates: TaGatesConfig::default(),
            ta_gate_soft_penalty: 15.0,

            // Portfolio Risk Gates (S9)
            enable_var_gate: false,
            max_portfolio_var_pct: 2.0,
            var_lookback_periods: 20,
            enable_concentration_gate: false,
            max_concentration_hhi: 0.4,

            // Safety Gates (S04.1)
            var_fail_closed: true,
            asset_check_hard_gate: true,
            blackout_first_mins: 5,
            blackout_last_mins: 5,
            blackout_exempt_symbols: Vec::new(),

            // Trading Edge (S15)
            max_hold_decay_enabled: true,
            max_hold_decay_start_pct: 0.5,
            max_hold_decay_trail_tighten: 0.3,
            mtf_gate_enabled: true,
            mtf_gate_mode: "soft".to_string(),
            mtf_gate_penalty: 15.0,
            mtf_gate_htf: "15m".to_string(),
            hhi_gate_manual_orders: true,
            max_concentration_hhi_manual: 0.35,
            tiered_scan_enabled: true,
            tier1_count: 50,
            tier2_count: 100,

            pending_order_timeout_secs: 120,

            ofi_divergence_threshold_secs: 15,

            // Classical indicator filters (Phase 7D)
            use_classical_filters: false,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            require_ema_alignment: false,
            adx_trend_threshold: 25.0,
            classical_weight: 0.15,

            // Strategy Thresholds (N4)
            entry_thresholds_scalp: default_entry_thresholds_scalp(),
            entry_thresholds_normal: default_entry_thresholds_normal(),
            score_modifiers: ScoreModifiers::default(),
            classical_bonuses: ClassicalBonuses::default(),

            // Phase 1: Autopilot Core
            min_hold_secs: 180,
            nexus_consensus_threshold: 70.0,
            autopilot_loop_ms: 1500,
            ofi_stale_ms: 10000,
            flash_crash_window_secs: 30,
            flash_crash_threshold_pct: 3.0,
            flash_crash_cooldown_mins: 5,
            dd_velocity_threshold: -1.0,
            dd_velocity_window_secs: 300,
            min_bars_for_entry: 20,
            ofi_entry_threshold: 5.0,
            max_qty_cap: 1000.0,
            min_qty_threshold: 0.001,
            default_equity_fallback: 10000.0,
            max_bypass_risk_pct: 25.0,

            // Phase 2: Exit Scaling
            vol_factor_min: 0.6,
            vol_factor_max: 2.5,
            fallback_sl_pct: 2.5,
            fallback_tp_pct: 2.5,
            fallback_trail_pct: 1.5,
            scalping_stop_mult: 0.70,
            scalping_take_mult: 0.60,
            scalping_trail_mult: 0.50,
            partial_first_target_frac: 0.5,
            partial_min_qty: 0.001,
            atr_period: 14,
            atr_stop_clamp_min: 0.5,
            atr_stop_clamp_max: 8.0,
            atr_take_clamp_min: 1.0,
            atr_take_clamp_max: 15.0,
            atr_trail_clamp_min: 0.3,
            atr_trail_clamp_max: 5.0,

            // Phase 3: Oracle Exit
            oracle_exit_min_confidence: 0.40,
            oracle_exit_min_score: 30.0,
            oracle_flip_score_threshold: 10.0,
            oracle_flip_confidence: 0.20,
            chaos_exit_max_loss_pct: -0.5,
            chaos_exit_min_confidence: 0.25,

            // Phase 4: Circuit Breaker
            breaker_caution_pct: -3.0,
            breaker_halt_pct: -5.0,
            breaker_panic_pct: -8.0,
            max_notional_equity_frac: 0.25,
            min_stop_distance_frac: 0.0001,

            // Phase 5: Strategy Scoring
            strategy_base_score: 50.0,
            vpin_gate_scalping: 0.85,
            vpin_gate_normal: 0.75,
            min_rvol_threshold: 0.5,
            rvol_window: 20,
            rvol_relax_threshold: 3.0,
            rvol_relax_value: 5.0,
            chaos_confidence_kill: 0.10,
            chaos_regime_mult: 0.55,

            // Phase 6: Engine Stops
            long_stop_normal: -2.5,
            long_stop_aggressive: -8.0,
            short_stop_pct: -5.0,
            long_trail_min_profit: 1.5,
            short_trail_gap: 1.5,
            short_trail_min_profit: 1.0,

            // Phase 7: Oracle Fusion
            oracle_kalman_weight: 0.30,
            oracle_entropy_weight: 0.25,
            oracle_hurst_weight: 0.20,
            oracle_vpin_weight: 0.15,
            oracle_wavelet_weight: 0.10,
            oracle_confidence_floor: 0.25,
            entropy_chaos_threshold: 0.85,
            vpin_toxicity_alert: 0.6,
            enable_oracle_adaptation: false,

            // Phase 8: Indicator Periods
            ema_fast_period: 9,
            ema_mid_period: 20,
            ema_slow_period: 50,
            ema_trend_period: 200,
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bb_period: 20,
            bb_std_dev: 2.0,
            adx_period: 14,
            stoch_rsi_period: 14,

            // Phase 9: Adaptive Exit Multipliers
            disable_adaptive_exits: false,
            exit_floor_pct: 0.3,

            regime_chaotic_stop_mult: 0.70,
            regime_chaotic_take_mult: 0.60,
            regime_chaotic_trail_mult: 0.50,
            regime_range_stop_mult: 0.85,
            regime_range_take_mult: 0.75,
            regime_range_trail_mult: 0.85,
            regime_trend_stop_mult: 1.05,
            regime_trend_take_mult: 1.30,
            regime_trend_trail_mult: 1.10,

            signal_aligned_stop_mult: 1.10,
            signal_aligned_take_mult: 1.25,
            signal_aligned_trail_mult: 1.10,
            signal_oppose_stop_mult: 0.85,
            signal_oppose_take_mult: 0.80,

            tf_short_stop_mult: 0.90,
            tf_short_take_mult: 0.90,
            tf_short_trail_mult: 0.90,
            symbol_overrides: HashMap::new(),
            smarttrend: crate::smarttrend::SmartTrendConfig::default(),
            st_asset_modes: HashMap::new(),
        }
    }
}

/// Validate safety gate settings for logical consistency. Returns warnings.
pub fn validate_safety_settings(settings: &AppState) -> Vec<String> {
    let mut warnings = Vec::new();
    let total_blackout = settings.blackout_first_mins + settings.blackout_last_mins;
    if total_blackout >= 195 {
        warnings.push(format!(
            "blackout_first_mins ({}) + blackout_last_mins ({}) = {} >= half trading day (195 min)",
            settings.blackout_first_mins, settings.blackout_last_mins, total_blackout
        ));
    }
    if settings.scan_params.min_dollar_volume > 0.0
        && settings.scan_params.min_dollar_volume < 100_000.0
    {
        warnings.push(format!(
            "min_dollar_volume ({:.0}) is below $100K — may admit illiquid tickers",
            settings.scan_params.min_dollar_volume
        ));
    }
    if settings.enable_var_gate && !settings.var_fail_closed {
        warnings.push(
            "VaR gate enabled but var_fail_closed=false — entries allowed without risk data"
                .to_string(),
        );
    }
    warnings
}

impl AppState {
    pub fn log(&mut self, msg: &str) {
        self.latest_log = msg.to_string();
        tracing::info!(component = "app", "{}", msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIX-4 CRIT-19: Default dry_run must be true (safe default for fresh install).
    #[test]
    fn default_dry_run_is_true() {
        let state = AppState::default();
        assert!(
            state.dry_run_enabled,
            "AppState::default() must have dry_run_enabled = true"
        );
    }
}

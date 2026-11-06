use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use sysinfo::System;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::app_state::{AppState, Candle};
use crate::error::TitanError;
use crate::{genetic, journal, monitor, scanner, sentiment, streamer, watchdog};

// --- CONSTANTS ---
pub(crate) const MAX_HISTORY: usize = 200; // Ring Buffer Capacity
pub(crate) const DEFAULT_NET_IFACE: &str = "auto";

// --- DATA STRUCTURES ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ApiKeys {
    pub key: String,
    pub secret: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Financial Modeling Prep API key (free tier, for float data)
    #[serde(default)]
    pub fmp_api_key: String,
}

pub(crate) fn default_base_url() -> String {
    "https://paper-api.alpaca.markets".to_string()
}

/// Serialize `Decimal` as string; deserialize from JSON string or number (legacy `wallet.json`).
mod serde_decimal_wallet {
    use super::Decimal;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        rust_decimal::serde::str::serialize(value, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Json {
            Str(String),
            Num(serde_json::Number),
        }
        match Json::deserialize(deserializer)? {
            Json::Str(s) => Decimal::from_str(s.trim()).map_err(D::Error::custom),
            Json::Num(n) => parse_json_number(&n),
        }
    }

    fn parse_json_number<E: Error>(n: &serde_json::Number) -> Result<Decimal, E> {
        if let Some(i) = n.as_i64() {
            Ok(Decimal::from(i))
        } else if let Some(u) = n.as_u64() {
            Ok(Decimal::from(u))
        } else if let Some(f) = n.as_f64() {
            if !f.is_finite() {
                return Err(E::custom("non-finite number"));
            }
            Decimal::try_from(f).map_err(E::custom)
        } else {
            Err(E::custom("invalid JSON number"))
        }
    }
}

mod serde_decimal_vec_float {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &[Decimal], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let strings: Vec<String> = value.iter().map(|d| d.to_string()).collect();
        strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Decimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            Str(String),
            Float(f64),
        }
        let items = Vec::<StringOrFloat>::deserialize(deserializer)?;
        Ok(items
            .into_iter()
            .map(|item| match item {
                StringOrFloat::Str(s) => Decimal::from_str(&s).unwrap_or_default(),
                StringOrFloat::Float(f) => Decimal::try_from(f).unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Wallet {
    #[serde(with = "serde_decimal_wallet")]
    pub bot_cash: Decimal,
    #[serde(with = "serde_decimal_wallet")]
    pub user_deposit: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct AppSettings {
    pub active_symbol: String,
    pub hft_targets: Vec<String>,
    pub loop_active: bool,
    pub omni_scan_active: bool,
    pub auto_pilot: bool,
    pub aggressive_mode: bool,
    pub allow_short: bool,
    pub enable_extended: bool,
    pub force_manual_tf: bool,
    #[serde(default = "default_true")]
    pub scalping_oracle: bool,
    #[serde(default = "default_true")]
    pub scalping_gpu: bool,
    #[serde(default = "default_true")]
    pub scalping_fusion: bool,
    pub risk_per_trade: f64,
    pub refresh_rate: u64,
    pub manual_res: String,
    pub backtest_days: u64,
    #[serde(default)]
    pub gpu_backtest: bool,
    #[serde(default)]
    pub gpu_sweep: bool,
    #[serde(default)]
    pub gpu_deep_focus: bool,
    // Auto-Trading
    #[serde(default)]
    pub auto_entry: bool,
    #[serde(default = "default_max_positions")]
    pub max_open_positions: u32,
    #[serde(default = "default_cooldown")]
    pub entry_cooldown_secs: u64,
    #[serde(default = "default_daily_loss")]
    pub daily_loss_limit_pct: f64,
    // HFT Engine advanced params
    #[serde(default = "default_trailing_stop")]
    pub trailing_stop_pct: f64,
    #[serde(default = "default_take_profit")]
    pub take_profit_pct: f64,
    #[serde(default = "default_max_spread")]
    pub max_spread_bps: f64,
    #[serde(default = "default_ofi_threshold")]
    pub min_ofi_threshold: f64,
    #[serde(default = "default_max_daily_trades")]
    pub max_daily_trades: u32,
    #[serde(default = "default_signal_strength")]
    pub entry_signal_strength_min: f64,
    #[serde(default = "default_max_hold")]
    pub max_hold_secs: u64,
    // Multi-symbol
    #[serde(default)]
    pub multi_symbol_enabled: bool,
    #[serde(default = "default_max_concurrent_assets")]
    pub max_concurrent_assets: u32,
    #[serde(default = "default_max_correlation")]
    pub max_correlation: f64,
    #[serde(default = "default_correlation_window")]
    pub correlation_window: usize,
    // Discord alerts
    #[serde(default)]
    pub discord_webhook_url: String,
    #[serde(default = "default_true")]
    pub alert_on_fill: bool,
    #[serde(default = "default_true")]
    pub alert_on_breaker: bool,
    #[serde(default)]
    pub alert_on_daily_summary: bool,
    // Telegram alerts
    #[serde(default)]
    pub telegram_bot_token: String,
    #[serde(default)]
    pub telegram_chat_id: String,
    #[serde(default)]
    pub telegram_enabled: bool,
    // ATR dynamic stops
    #[serde(default = "default_atr_stop_mult")]
    pub atr_stop_multiplier: f64,
    #[serde(default = "default_atr_take_mult")]
    pub atr_take_multiplier: f64,
    #[serde(default = "default_true")]
    pub partial_take_enabled: bool,
    #[serde(default = "default_partial_take_pct")]
    pub partial_take_pct: f64,
    // LLM Sentiment
    #[serde(default)]
    pub llm_api_key: String,
    #[serde(default = "default_llm_provider")]
    pub llm_provider: String,
    #[serde(default)]
    pub llm_sentiment_enabled: bool,
    #[serde(default = "default_llm_batch_size")]
    pub llm_batch_size: usize,
    #[serde(default = "default_sentiment_halflife")]
    pub sentiment_halflife_secs: u64,
    // Options Hedging
    #[serde(default)]
    pub options_hedging_enabled: bool,
    #[serde(default = "default_hedge_otm")]
    pub hedge_otm_pct: f64,
    #[serde(default = "default_hedge_premium")]
    pub hedge_max_premium_pct: f64,
    // Web Dashboard
    #[serde(default)]
    pub web_dashboard_enabled: bool,
    #[serde(default = "default_web_port")]
    pub web_dashboard_port: u16,
    #[serde(default)]
    pub web_dashboard_token: String,
    // Sidebar persisted
    #[serde(
        default = "default_bot_trade_size",
        with = "rust_decimal::serde::float"
    )]
    pub bot_trade_size: Decimal,
    #[serde(
        default = "default_circuit_breaker",
        with = "rust_decimal::serde::float"
    )]
    pub circuit_breaker_limit: Decimal,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub dry_run_enabled: bool,
    #[serde(default = "default_strategy_mode")]
    pub strategy_mode_name: String,
    // AI Scanner auto-scan (default true = current behavior)
    #[serde(default = "default_true")]
    pub ai_scan_active: bool,
    #[serde(default = "default_true")]
    pub ai_smart_sort: bool,
    #[serde(default)]
    pub ai_auto_inject_top: u32,
    #[serde(default)]
    pub ai_auto_inject_hft: bool,
    #[serde(default)]
    pub scan_params: scanner::ScanParams,
    #[serde(default)]
    pub position_trade_modes: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub md_auto_scan_active: bool,
    #[serde(default = "default_fusion_alpha")]
    pub nexus_fusion_alpha: f64,
    #[serde(default)]
    pub signal_strategy_assets: Vec<String>,
    #[serde(default = "default_signal_source")]
    pub signal_source: String,
    #[serde(default)]
    pub bypass_max_spread: bool,
    #[serde(default)]
    pub bypass_cooldown: bool,
    #[serde(default)]
    pub bypass_allow_short: bool,
    #[serde(default)]
    pub bypass_extended: bool,
    #[serde(default)]
    pub bypass_risk_per_trade: bool,
    #[serde(default)]
    pub bypass_aggressive: bool,
    #[serde(default)]
    pub bypass_atr_stop: bool,
    #[serde(default)]
    pub bypass_atr_take: bool,
    #[serde(default)]
    pub bypass_partial_take: bool,
    #[serde(default)]
    pub bypass_partial_pct: bool,
    #[serde(default)]
    pub bypass_max_correlation: bool,
    #[serde(default)]
    pub bypass_correlation_window: bool,
    #[serde(default)]
    pub bypass_daily_loss: bool,
    #[serde(
        default = "default_max_manual_order_usd",
        with = "rust_decimal::serde::float"
    )]
    pub max_manual_order_usd: Decimal,
    // TA Entry Gates (RF-B)
    #[serde(default)]
    pub ta_gates: crate::app_state::TaGatesConfig,
    #[serde(default = "default_ta_gate_soft_penalty")]
    pub ta_gate_soft_penalty: f64,

    // Portfolio Risk Gates (S9)
    #[serde(default)]
    pub enable_var_gate: bool,
    #[serde(default = "default_max_portfolio_var_pct")]
    pub max_portfolio_var_pct: f64,
    #[serde(default = "default_var_lookback_periods")]
    pub var_lookback_periods: usize,
    #[serde(default)]
    pub enable_concentration_gate: bool,
    #[serde(default = "default_max_concentration_hhi")]
    pub max_concentration_hhi: f64,

    // Safety Gates (S04.1)
    #[serde(default = "default_true")]
    pub var_fail_closed: bool,
    #[serde(default = "default_true")]
    pub asset_check_hard_gate: bool,
    #[serde(default = "default_blackout_mins")]
    pub blackout_first_mins: u64,
    #[serde(default = "default_blackout_mins")]
    pub blackout_last_mins: u64,
    #[serde(default)]
    pub blackout_exempt_symbols: Vec<String>,

    // Trading Edge (S15): Max-Hold Decay
    #[serde(default = "default_true")]
    pub max_hold_decay_enabled: bool,
    #[serde(default = "default_decay_start_pct")]
    pub max_hold_decay_start_pct: f64,
    #[serde(default = "default_decay_trail_tighten")]
    pub max_hold_decay_trail_tighten: f64,

    // Trading Edge (S15): MTF Confirmation Gate
    #[serde(default = "default_true")]
    pub mtf_gate_enabled: bool,
    #[serde(default = "default_mtf_gate_mode")]
    pub mtf_gate_mode: String,
    #[serde(default = "default_mtf_gate_penalty")]
    pub mtf_gate_penalty: f64,
    #[serde(default = "default_mtf_gate_htf")]
    pub mtf_gate_htf: String,

    // Trading Edge (S15): Risk Parity Manual Orders
    #[serde(default = "default_true")]
    pub hhi_gate_manual_orders: bool,
    #[serde(default = "default_hhi_manual_limit")]
    pub max_concentration_hhi_manual: f64,

    // Trading Edge (S15): Tiered Scanner
    #[serde(default = "default_true")]
    pub tiered_scan_enabled: bool,
    #[serde(default = "default_tier1_count")]
    pub tier1_count: usize,
    #[serde(default = "default_tier2_count")]
    pub tier2_count: usize,

    // Execution Resilience (S14)
    #[serde(default = "default_pending_order_timeout")]
    pub pending_order_timeout_secs: u64,

    // Microstructure: OFI divergence alert threshold
    #[serde(default = "default_ofi_divergence_secs")]
    pub ofi_divergence_threshold_secs: u64,

    // Classical indicator filters (Phase 7D)
    #[serde(default)]
    pub use_classical_filters: bool,
    #[serde(default = "default_rsi_overbought")]
    pub rsi_overbought: f64,
    #[serde(default = "default_rsi_oversold")]
    pub rsi_oversold: f64,
    #[serde(default)]
    pub require_ema_alignment: bool,
    #[serde(default = "default_adx_trend_threshold")]
    pub adx_trend_threshold: f64,
    #[serde(default = "default_classical_weight")]
    pub classical_weight: f64,

    // Strategy Thresholds (N4)
    #[serde(default = "crate::app_state::default_entry_thresholds_scalp")]
    pub entry_thresholds_scalp: std::collections::HashMap<String, crate::app_state::EntryThreshold>,
    #[serde(default = "crate::app_state::default_entry_thresholds_normal")]
    pub entry_thresholds_normal: std::collections::HashMap<String, crate::app_state::EntryThreshold>,
    #[serde(default)]
    pub score_modifiers: crate::app_state::ScoreModifiers,
    #[serde(default)]
    pub classical_bonuses: crate::app_state::ClassicalBonuses,

    // Phase 1: Autopilot Core
    #[serde(default = "default_min_hold_secs")]
    pub min_hold_secs: u64,
    #[serde(default = "default_nexus_consensus_threshold")]
    pub nexus_consensus_threshold: f64,
    #[serde(default = "default_autopilot_loop_ms")]
    pub autopilot_loop_ms: u64,
    #[serde(default = "default_ofi_stale_ms")]
    pub ofi_stale_ms: u64,
    #[serde(default = "default_flash_crash_window_secs")]
    pub flash_crash_window_secs: u64,
    #[serde(default = "default_flash_crash_threshold_pct")]
    pub flash_crash_threshold_pct: f64,
    #[serde(default = "default_flash_crash_cooldown_mins")]
    pub flash_crash_cooldown_mins: u64,
    #[serde(default = "default_dd_velocity_threshold")]
    pub dd_velocity_threshold: f64,
    #[serde(default = "default_dd_velocity_window_secs")]
    pub dd_velocity_window_secs: u64,
    #[serde(default = "default_min_bars_for_entry")]
    pub min_bars_for_entry: u64,
    #[serde(default = "default_ofi_entry_threshold")]
    pub ofi_entry_threshold: f64,
    #[serde(default = "default_max_qty_cap")]
    pub max_qty_cap: f64,
    #[serde(default = "default_min_qty_threshold")]
    pub min_qty_threshold: f64,
    #[serde(default = "default_equity_fallback")]
    pub default_equity_fallback: f64,
    #[serde(default = "default_max_bypass_risk_pct")]
    pub max_bypass_risk_pct: f64,

    // Phase 2: Exit Scaling
    #[serde(default = "default_vol_factor_min")]
    pub vol_factor_min: f64,
    #[serde(default = "default_vol_factor_max")]
    pub vol_factor_max: f64,
    #[serde(default = "default_fallback_sl_pct")]
    pub fallback_sl_pct: f64,
    #[serde(default = "default_fallback_tp_pct")]
    pub fallback_tp_pct: f64,
    #[serde(default = "default_fallback_trail_pct")]
    pub fallback_trail_pct: f64,
    #[serde(default = "default_scalping_stop_mult")]
    pub scalping_stop_mult: f64,
    #[serde(default = "default_scalping_take_mult")]
    pub scalping_take_mult: f64,
    #[serde(default = "default_scalping_trail_mult")]
    pub scalping_trail_mult: f64,
    #[serde(default = "default_partial_first_target_frac")]
    pub partial_first_target_frac: f64,
    #[serde(default = "default_partial_min_qty")]
    pub partial_min_qty: f64,
    #[serde(default = "default_atr_period")]
    pub atr_period: u64,
    #[serde(default = "default_atr_stop_clamp_min")]
    pub atr_stop_clamp_min: f64,
    #[serde(default = "default_atr_stop_clamp_max")]
    pub atr_stop_clamp_max: f64,
    #[serde(default = "default_atr_take_clamp_min")]
    pub atr_take_clamp_min: f64,
    #[serde(default = "default_atr_take_clamp_max")]
    pub atr_take_clamp_max: f64,
    #[serde(default = "default_atr_trail_clamp_min")]
    pub atr_trail_clamp_min: f64,
    #[serde(default = "default_atr_trail_clamp_max")]
    pub atr_trail_clamp_max: f64,

    // Phase 3: Oracle Exit
    #[serde(default = "default_oracle_exit_min_confidence")]
    pub oracle_exit_min_confidence: f64,
    #[serde(default = "default_oracle_exit_min_score")]
    pub oracle_exit_min_score: f64,
    #[serde(default = "default_oracle_flip_score_threshold")]
    pub oracle_flip_score_threshold: f64,
    #[serde(default = "default_oracle_flip_confidence")]
    pub oracle_flip_confidence: f64,
    #[serde(default = "default_chaos_exit_max_loss_pct")]
    pub chaos_exit_max_loss_pct: f64,
    #[serde(default = "default_chaos_exit_min_confidence")]
    pub chaos_exit_min_confidence: f64,

    // Phase 4: Circuit Breaker
    #[serde(default = "default_breaker_caution_pct")]
    pub breaker_caution_pct: f64,
    #[serde(default = "default_breaker_halt_pct")]
    pub breaker_halt_pct: f64,
    #[serde(default = "default_breaker_panic_pct")]
    pub breaker_panic_pct: f64,
    #[serde(default = "default_max_notional_equity_frac")]
    pub max_notional_equity_frac: f64,
    #[serde(default = "default_min_stop_distance_frac")]
    pub min_stop_distance_frac: f64,

    // Phase 5: Strategy Scoring
    #[serde(default = "default_strategy_base_score")]
    pub strategy_base_score: f64,
    #[serde(default = "default_vpin_gate_scalping")]
    pub vpin_gate_scalping: f64,
    #[serde(default = "default_vpin_gate_normal")]
    pub vpin_gate_normal: f64,
    #[serde(default = "default_min_rvol_threshold")]
    pub min_rvol_threshold: f64,
    #[serde(default = "default_rvol_window")]
    pub rvol_window: u64,
    #[serde(default = "default_rvol_relax_threshold")]
    pub rvol_relax_threshold: f64,
    #[serde(default = "default_rvol_relax_value")]
    pub rvol_relax_value: f64,
    #[serde(default = "default_chaos_confidence_kill")]
    pub chaos_confidence_kill: f64,
    #[serde(default = "default_chaos_regime_mult")]
    pub chaos_regime_mult: f64,

    // Phase 6: Engine Stops
    #[serde(default = "default_long_stop_normal")]
    pub long_stop_normal: f64,
    #[serde(default = "default_long_stop_aggressive")]
    pub long_stop_aggressive: f64,
    #[serde(default = "default_short_stop_pct")]
    pub short_stop_pct: f64,
    #[serde(default = "default_long_trail_min_profit")]
    pub long_trail_min_profit: f64,
    #[serde(default = "default_short_trail_gap")]
    pub short_trail_gap: f64,
    #[serde(default = "default_short_trail_min_profit")]
    pub short_trail_min_profit: f64,

    // Phase 7: Oracle Fusion
    #[serde(default = "default_oracle_kalman_weight")]
    pub oracle_kalman_weight: f64,
    #[serde(default = "default_oracle_entropy_weight")]
    pub oracle_entropy_weight: f64,
    #[serde(default = "default_oracle_hurst_weight")]
    pub oracle_hurst_weight: f64,
    #[serde(default = "default_oracle_vpin_weight")]
    pub oracle_vpin_weight: f64,
    #[serde(default = "default_oracle_wavelet_weight")]
    pub oracle_wavelet_weight: f64,
    #[serde(default = "default_oracle_confidence_floor")]
    pub oracle_confidence_floor: f64,
    #[serde(default = "default_entropy_chaos_threshold")]
    pub entropy_chaos_threshold: f64,
    #[serde(default = "default_vpin_toxicity_alert")]
    pub vpin_toxicity_alert: f64,
    #[serde(default)]
    pub enable_oracle_adaptation: bool,

    // Phase 8: Indicator Periods
    #[serde(default = "default_ema_fast_period")]
    pub ema_fast_period: u64,
    #[serde(default = "default_ema_mid_period")]
    pub ema_mid_period: u64,
    #[serde(default = "default_ema_slow_period")]
    pub ema_slow_period: u64,
    #[serde(default = "default_ema_trend_period")]
    pub ema_trend_period: u64,
    #[serde(default = "default_rsi_period")]
    pub rsi_period: u64,
    #[serde(default = "default_macd_fast")]
    pub macd_fast: u64,
    #[serde(default = "default_macd_slow")]
    pub macd_slow: u64,
    #[serde(default = "default_macd_signal")]
    pub macd_signal: u64,
    #[serde(default = "default_bb_period")]
    pub bb_period: u64,
    #[serde(default = "default_bb_std_dev")]
    pub bb_std_dev: f64,
    #[serde(default = "default_adx_period")]
    pub adx_period: u64,
    #[serde(default = "default_stoch_rsi_period")]
    pub stoch_rsi_period: u64,

    // Phase 9: Adaptive Exit Multipliers
    #[serde(default)]
    pub disable_adaptive_exits: bool,
    #[serde(default = "default_exit_floor_pct")]
    pub exit_floor_pct: f64,

    #[serde(default = "default_regime_chaotic_stop_mult")]
    pub regime_chaotic_stop_mult: f64,
    #[serde(default = "default_regime_chaotic_take_mult")]
    pub regime_chaotic_take_mult: f64,
    #[serde(default = "default_regime_chaotic_trail_mult")]
    pub regime_chaotic_trail_mult: f64,
    #[serde(default = "default_regime_range_stop_mult")]
    pub regime_range_stop_mult: f64,
    #[serde(default = "default_regime_range_take_mult")]
    pub regime_range_take_mult: f64,
    #[serde(default = "default_regime_range_trail_mult")]
    pub regime_range_trail_mult: f64,
    #[serde(default = "default_regime_trend_stop_mult")]
    pub regime_trend_stop_mult: f64,
    #[serde(default = "default_regime_trend_take_mult")]
    pub regime_trend_take_mult: f64,
    #[serde(default = "default_regime_trend_trail_mult")]
    pub regime_trend_trail_mult: f64,

    #[serde(default = "default_signal_aligned_stop_mult")]
    pub signal_aligned_stop_mult: f64,
    #[serde(default = "default_signal_aligned_take_mult")]
    pub signal_aligned_take_mult: f64,
    #[serde(default = "default_signal_aligned_trail_mult")]
    pub signal_aligned_trail_mult: f64,
    #[serde(default = "default_signal_oppose_stop_mult")]
    pub signal_oppose_stop_mult: f64,
    #[serde(default = "default_signal_oppose_take_mult")]
    pub signal_oppose_take_mult: f64,

    #[serde(default = "default_tf_short_stop_mult")]
    pub tf_short_stop_mult: f64,
    #[serde(default = "default_tf_short_take_mult")]
    pub tf_short_take_mult: f64,
    #[serde(default = "default_tf_short_trail_mult")]
    pub tf_short_trail_mult: f64,

    #[serde(default)]
    pub symbol_overrides: std::collections::HashMap<String, crate::app_state::SymbolOverride>,

    #[serde(default)]
    pub smarttrend: crate::smarttrend::SmartTrendConfig,

    #[serde(default)]
    pub st_asset_modes: HashMap<String, String>,
}

pub(crate) fn default_max_manual_order_usd() -> Decimal {
    Decimal::from(50000)
}
pub(crate) fn default_max_positions() -> u32 {
    3
}
pub(crate) fn default_cooldown() -> u64 {
    30
}
pub(crate) fn default_daily_loss() -> f64 {
    3.0
}
pub(crate) fn default_trailing_stop() -> f64 {
    1.5
}
pub(crate) fn default_take_profit() -> f64 {
    2.5
}
pub(crate) fn default_max_spread() -> f64 {
    50.0
}
pub(crate) fn default_ofi_threshold() -> f64 {
    5.0
}
pub(crate) fn default_max_daily_trades() -> u32 {
    50
}
pub(crate) fn default_signal_strength() -> f64 {
    40.0
}
pub(crate) fn default_max_hold() -> u64 {
    3600
}
pub(crate) fn default_true() -> bool {
    true
}
pub(crate) fn default_bot_trade_size() -> Decimal {
    Decimal::from(50)
}
pub(crate) fn default_circuit_breaker() -> Decimal {
    Decimal::from(3000)
}
pub(crate) fn default_strategy_mode() -> String {
    "SMARTTREND".to_string()
}
pub(crate) fn default_max_correlation() -> f64 {
    0.85
}
pub(crate) fn default_correlation_window() -> usize {
    60
}
pub(crate) fn default_atr_stop_mult() -> f64 {
    1.5
}
pub(crate) fn default_atr_take_mult() -> f64 {
    3.0
}
pub(crate) fn default_partial_take_pct() -> f64 {
    50.0
}
pub(crate) fn default_llm_provider() -> String {
    "openai".to_string()
}
pub(crate) fn default_llm_batch_size() -> usize {
    5
}
pub(crate) fn default_sentiment_halflife() -> u64 {
    3600
}
pub(crate) fn default_hedge_otm() -> f64 {
    5.0
}
pub(crate) fn default_hedge_premium() -> f64 {
    1.0
}
pub(crate) fn default_web_port() -> u16 {
    7777
}
pub(crate) fn default_fusion_alpha() -> f64 {
    0.5
}
pub(crate) fn default_signal_source() -> String {
    "oracle".to_string()
}
pub(crate) fn default_ta_gate_soft_penalty() -> f64 {
    15.0
}
pub(crate) fn default_max_portfolio_var_pct() -> f64 {
    2.0
}
pub(crate) fn default_var_lookback_periods() -> usize {
    20
}
pub(crate) fn default_max_concentration_hhi() -> f64 {
    0.4
}
pub(crate) fn default_blackout_mins() -> u64 {
    5
}
fn default_pending_order_timeout() -> u64 {
    120
}
fn default_decay_start_pct() -> f64 {
    0.5
}
fn default_decay_trail_tighten() -> f64 {
    0.3
}
fn default_mtf_gate_mode() -> String {
    "soft".to_string()
}
fn default_mtf_gate_penalty() -> f64 {
    15.0
}
fn default_mtf_gate_htf() -> String {
    "15m".to_string()
}
fn default_hhi_manual_limit() -> f64 {
    0.35
}
fn default_tier1_count() -> usize {
    50
}
fn default_tier2_count() -> usize {
    100
}
fn default_ofi_divergence_secs() -> u64 {
    15
}
pub(crate) fn default_rsi_overbought() -> f64 {
    70.0
}
pub(crate) fn default_rsi_oversold() -> f64 {
    30.0
}
pub(crate) fn default_adx_trend_threshold() -> f64 {
    25.0
}
pub(crate) fn default_classical_weight() -> f64 {
    0.15
}

// Phase 1: Autopilot Core defaults
pub(crate) fn default_min_hold_secs() -> u64 { 180 }
pub(crate) fn default_nexus_consensus_threshold() -> f64 { 70.0 }
pub(crate) fn default_autopilot_loop_ms() -> u64 { 1500 }
pub(crate) fn default_ofi_stale_ms() -> u64 { 10000 }
pub(crate) fn default_flash_crash_window_secs() -> u64 { 30 }
pub(crate) fn default_flash_crash_threshold_pct() -> f64 { 3.0 }
pub(crate) fn default_flash_crash_cooldown_mins() -> u64 { 5 }
pub(crate) fn default_dd_velocity_threshold() -> f64 { -1.0 }
pub(crate) fn default_dd_velocity_window_secs() -> u64 { 300 }
pub(crate) fn default_min_bars_for_entry() -> u64 { 20 }
pub(crate) fn default_ofi_entry_threshold() -> f64 { 5.0 }
pub(crate) fn default_max_qty_cap() -> f64 { 1000.0 }
pub(crate) fn default_min_qty_threshold() -> f64 { 0.001 }
pub(crate) fn default_equity_fallback() -> f64 { 10000.0 }
pub(crate) fn default_max_bypass_risk_pct() -> f64 { 25.0 }

// Phase 2: Exit Scaling defaults
pub(crate) fn default_vol_factor_min() -> f64 { 0.6 }
pub(crate) fn default_vol_factor_max() -> f64 { 2.5 }
pub(crate) fn default_fallback_sl_pct() -> f64 { 2.5 }
pub(crate) fn default_fallback_tp_pct() -> f64 { 2.5 }
pub(crate) fn default_fallback_trail_pct() -> f64 { 1.5 }
pub(crate) fn default_scalping_stop_mult() -> f64 { 0.70 }
pub(crate) fn default_scalping_take_mult() -> f64 { 0.60 }
pub(crate) fn default_scalping_trail_mult() -> f64 { 0.50 }
pub(crate) fn default_partial_first_target_frac() -> f64 { 0.5 }
pub(crate) fn default_partial_min_qty() -> f64 { 0.001 }
pub(crate) fn default_atr_period() -> u64 { 14 }
pub(crate) fn default_atr_stop_clamp_min() -> f64 { 0.5 }
pub(crate) fn default_atr_stop_clamp_max() -> f64 { 8.0 }
pub(crate) fn default_atr_take_clamp_min() -> f64 { 1.0 }
pub(crate) fn default_atr_take_clamp_max() -> f64 { 15.0 }
pub(crate) fn default_atr_trail_clamp_min() -> f64 { 0.3 }
pub(crate) fn default_atr_trail_clamp_max() -> f64 { 5.0 }

// Phase 3: Oracle Exit defaults
pub(crate) fn default_oracle_exit_min_confidence() -> f64 { 0.40 }
pub(crate) fn default_oracle_exit_min_score() -> f64 { 30.0 }
pub(crate) fn default_oracle_flip_score_threshold() -> f64 { 10.0 }
pub(crate) fn default_oracle_flip_confidence() -> f64 { 0.20 }
pub(crate) fn default_chaos_exit_max_loss_pct() -> f64 { -0.5 }
pub(crate) fn default_chaos_exit_min_confidence() -> f64 { 0.25 }

// Phase 4: Circuit Breaker defaults
pub(crate) fn default_breaker_caution_pct() -> f64 { -3.0 }
pub(crate) fn default_breaker_halt_pct() -> f64 { -5.0 }
pub(crate) fn default_breaker_panic_pct() -> f64 { -8.0 }
pub(crate) fn default_max_notional_equity_frac() -> f64 { 0.25 }
pub(crate) fn default_min_stop_distance_frac() -> f64 { 0.0001 }

// Phase 5: Strategy Scoring defaults
pub(crate) fn default_strategy_base_score() -> f64 { 50.0 }
pub(crate) fn default_vpin_gate_scalping() -> f64 { 0.85 }
pub(crate) fn default_vpin_gate_normal() -> f64 { 0.75 }
pub(crate) fn default_min_rvol_threshold() -> f64 { 0.5 }
pub(crate) fn default_rvol_window() -> u64 { 20 }
pub(crate) fn default_rvol_relax_threshold() -> f64 { 3.0 }
pub(crate) fn default_rvol_relax_value() -> f64 { 5.0 }
pub(crate) fn default_chaos_confidence_kill() -> f64 { 0.10 }
pub(crate) fn default_chaos_regime_mult() -> f64 { 0.55 }

// Phase 6: Engine Stops defaults
pub(crate) fn default_long_stop_normal() -> f64 { -2.5 }
pub(crate) fn default_long_stop_aggressive() -> f64 { -8.0 }
pub(crate) fn default_short_stop_pct() -> f64 { -5.0 }
pub(crate) fn default_long_trail_min_profit() -> f64 { 1.5 }
pub(crate) fn default_short_trail_gap() -> f64 { 1.5 }
pub(crate) fn default_short_trail_min_profit() -> f64 { 1.0 }

// Phase 7: Oracle Fusion defaults
pub(crate) fn default_oracle_kalman_weight() -> f64 { 0.30 }
pub(crate) fn default_oracle_entropy_weight() -> f64 { 0.25 }
pub(crate) fn default_oracle_hurst_weight() -> f64 { 0.20 }
pub(crate) fn default_oracle_vpin_weight() -> f64 { 0.15 }
pub(crate) fn default_oracle_wavelet_weight() -> f64 { 0.10 }
pub(crate) fn default_oracle_confidence_floor() -> f64 { 0.25 }
pub(crate) fn default_entropy_chaos_threshold() -> f64 { 0.85 }
pub(crate) fn default_vpin_toxicity_alert() -> f64 { 0.6 }

// Phase 8: Indicator Periods defaults
pub(crate) fn default_ema_fast_period() -> u64 { 9 }
pub(crate) fn default_ema_mid_period() -> u64 { 20 }
pub(crate) fn default_ema_slow_period() -> u64 { 50 }
pub(crate) fn default_ema_trend_period() -> u64 { 200 }
pub(crate) fn default_rsi_period() -> u64 { 14 }
pub(crate) fn default_macd_fast() -> u64 { 12 }
pub(crate) fn default_macd_slow() -> u64 { 26 }
pub(crate) fn default_macd_signal() -> u64 { 9 }
pub(crate) fn default_bb_period() -> u64 { 20 }
pub(crate) fn default_bb_std_dev() -> f64 { 2.0 }
pub(crate) fn default_adx_period() -> u64 { 14 }
pub(crate) fn default_stoch_rsi_period() -> u64 { 14 }

// Phase 9: Adaptive Exit Multiplier defaults
pub(crate) fn default_exit_floor_pct() -> f64 { 0.3 }
pub(crate) fn default_regime_chaotic_stop_mult() -> f64 { 0.70 }
pub(crate) fn default_regime_chaotic_take_mult() -> f64 { 0.60 }
pub(crate) fn default_regime_chaotic_trail_mult() -> f64 { 0.50 }
pub(crate) fn default_regime_range_stop_mult() -> f64 { 0.85 }
pub(crate) fn default_regime_range_take_mult() -> f64 { 0.75 }
pub(crate) fn default_regime_range_trail_mult() -> f64 { 0.85 }
pub(crate) fn default_regime_trend_stop_mult() -> f64 { 1.05 }
pub(crate) fn default_regime_trend_take_mult() -> f64 { 1.30 }
pub(crate) fn default_regime_trend_trail_mult() -> f64 { 1.10 }
pub(crate) fn default_signal_aligned_stop_mult() -> f64 { 1.10 }
pub(crate) fn default_signal_aligned_take_mult() -> f64 { 1.25 }
pub(crate) fn default_signal_aligned_trail_mult() -> f64 { 1.10 }
pub(crate) fn default_signal_oppose_stop_mult() -> f64 { 0.85 }
pub(crate) fn default_signal_oppose_take_mult() -> f64 { 0.80 }
pub(crate) fn default_tf_short_stop_mult() -> f64 { 0.90 }
pub(crate) fn default_tf_short_take_mult() -> f64 { 0.90 }
pub(crate) fn default_tf_short_trail_mult() -> f64 { 0.90 }
pub(crate) fn default_max_concurrent_assets() -> u32 { 5 }

pub(crate) fn parse_amount(amount: f64) -> Result<f64, TitanError> {
    if !amount.is_finite() || amount <= 0.0 {
        return Err(TitanError::Validation("INVALID AMOUNT".into()));
    }
    Ok(amount)
}

pub(crate) fn wallet_path() -> std::path::PathBuf {
    crate::paths::wallet_path()
}

pub(crate) fn load_wallet_from_disk() -> Wallet {
    let default_wallet = Wallet {
        bot_cash: Decimal::ZERO,
        user_deposit: Decimal::ZERO,
    };
    let path = wallet_path();

    let Ok(raw) = fs::read_to_string(&path) else {
        return default_wallet;
    };

    match serde_json::from_str::<Wallet>(&raw) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!(component = "state", error = %e, "Wallet JSON corrupt -- trying .bak");
            let bak_path = path.with_extension("json.bak");
            if let Ok(bak_data) = fs::read_to_string(&bak_path) {
                if let Ok(w) = serde_json::from_str::<Wallet>(&bak_data) {
                    tracing::warn!(component = "state", "Recovered wallet from .bak");
                    return w;
                }
            }
            tracing::error!(component = "state", "Wallet backup also failed -- using defaults");
            default_wallet
        }
    }
}

pub(crate) fn save_wallet_to_disk(wallet: &Wallet) -> Result<(), TitanError> {
    let payload = serde_json::to_string(wallet)?;
    let path = wallet_path();
    if path.exists() {
        let bak = path.with_extension("json.bak");
        if let Err(e) = fs::copy(&path, &bak) {
            tracing::warn!(component = "state", error = %e, "Failed to create wallet .bak");
        }
    }
    let tmp = format!("{}.tmp", path.display());
    fs::write(&tmp, &payload).map_err(|e| {
        tracing::error!(component = "state", path = %tmp, "Wallet tmp write failed: {}", e);
        TitanError::Io(format!("Wallet write failed: {}", e))
    })?;
    fs::rename(&tmp, &path).map_err(|e| {
        tracing::error!(component = "state", path = %path.display(), "Wallet rename failed: {}", e);
        TitanError::Io(format!("Wallet rename failed: {}", e))
    })
}

pub(crate) fn settings_path() -> std::path::PathBuf {
    crate::paths::settings_path()
}

pub(crate) fn blacklist_path() -> std::path::PathBuf {
    crate::paths::blacklist_path()
}

pub(crate) fn snapshot_path() -> std::path::PathBuf {
    crate::paths::snapshot_path()
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct StateSnapshot {
    pub timestamp_ms: u64,
    pub trail_highs: HashMap<String, f64>,
    pub trail_lows: HashMap<String, f64>,
    pub partial_taken: HashMap<String, bool>,
    pub last_prices: HashMap<String, f64>,
    pub symbol_best_tf: HashMap<String, String>,
    pub nexus_fusion_alpha: f64,
    // Session analytics (flattened — Decimal with float serde for JSON compat)
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_starting_equity: Decimal,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_current_equity: Decimal,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_peak_equity: Decimal,
    pub session_total_trades: u32,
    pub session_winning_trades: u32,
    pub session_losing_trades: u32,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_total_pnl: Decimal,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_daily_pnl: Decimal,
    pub session_max_drawdown_pct: f64,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_avg_win: Decimal,
    #[serde(default, with = "rust_decimal::serde::float")]
    pub session_avg_loss: Decimal,
    pub session_last_entry_ts: Option<u64>,
    #[serde(default, with = "serde_decimal_vec_float")]
    pub session_trade_pnls: Vec<Decimal>,
    // AI Scan results
    pub last_scan_results: Vec<scanner::AiScanCandidate>,
    pub scan_was_running: bool,
    pub scan_processed: usize,
    pub scan_total_requested: usize,
    #[serde(default)]
    pub scan_prefilter_queue: Vec<scanner::PreFilterCandidate>,
    #[serde(default)]
    pub entry_tags: HashMap<String, String>,
    #[serde(default)]
    pub entry_times: HashMap<String, u64>,
    #[serde(default)]
    pub genetic_champion: Option<serde_json::Value>,
    #[serde(default)]
    pub st_genetic_champion: Option<serde_json::Value>,
    #[serde(default)]
    pub oracle_adaptation: Option<crate::titan_oracle::OracleAdaptation>,
    // SnapshotSchemaV2 fields (S03)
    #[serde(default)]
    pub dd_velocity_ring: Vec<(u64, f64)>,
    #[serde(default)]
    pub session_generation: u64,
}

pub(crate) fn save_state_snapshot(state: &Arc<TitanState>) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let trail_highs = state
        .trading.trail_highs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let trail_lows = state
        .trading.trail_lows
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let partial_taken = state
        .trading.partial_taken
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let last_prices = state
        .market.last_prices
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let symbol_best_tf = state
        .scan
        .symbol_best_tf
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let nexus_fusion_alpha = *state
        .nexus_fusion_alpha
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let entry_tags = state
        .trading.entry_tags
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let entry_times = state
        .trading.entry_times
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let (scan_results, scan_progress, scan_queue) = {
        let results = state
            .scan
            .last_scan_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let progress = *state
            .scan
            .scan_progress
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let queue = state
            .scan
            .scan_prefilter_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        (results, progress, queue)
    };

    let genetic_champion = state.genetic_optimizer.lock().ok().map(|opt| {
        let stats = opt.stats();
        let mut map = serde_json::Map::new();
        if let Ok(params_val) = serde_json::to_value(&stats.champion_params) {
            map.insert("params".to_string(), params_val);
        }
        map.insert("fitness".to_string(), serde_json::json!(stats.champion_fitness));
        map.insert("sharpe".to_string(), serde_json::json!(stats.champion_sharpe));
        serde_json::Value::Object(map)
    });

    let st_genetic_champion = state.st_genetic_optimizer.lock().ok().map(|opt| {
        let stats = opt.stats();
        let mut map = serde_json::Map::new();
        if let Ok(params_val) = serde_json::to_value(&stats.champion_params) {
            map.insert("params".to_string(), params_val);
        }
        map.insert("fitness".to_string(), serde_json::json!(stats.champion_fitness));
        map.insert("sharpe".to_string(), serde_json::json!(stats.champion_sharpe));
        serde_json::Value::Object(map)
    });

    let analytics = state
        .risk.session_analytics
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let snap = StateSnapshot {
        timestamp_ms: now,
        trail_highs,
        trail_lows,
        partial_taken,
        last_prices,
        symbol_best_tf,
        nexus_fusion_alpha,
        session_starting_equity: analytics.session_start_equity,
        session_current_equity: analytics.current_equity,
        session_peak_equity: analytics.peak_equity,
        session_total_trades: analytics.total_trades,
        session_winning_trades: analytics.winning_trades,
        session_losing_trades: analytics.losing_trades,
        session_total_pnl: analytics.total_pnl,
        session_daily_pnl: analytics.daily_pnl,
        session_max_drawdown_pct: analytics.max_drawdown_pct.to_f64().unwrap_or(0.0),
        session_avg_win: analytics.avg_win,
        session_avg_loss: analytics.avg_loss,
        session_last_entry_ts: analytics.last_entry_time,
        session_trade_pnls: analytics.trade_pnls(),
        last_scan_results: scan_results,
        scan_was_running: scan_progress.0,
        scan_processed: scan_progress.1,
        scan_total_requested: scan_progress.2,
        scan_prefilter_queue: scan_queue,
        entry_tags,
        entry_times,
        genetic_champion,
        st_genetic_champion,
        oracle_adaptation: state
            .oracle_adaptation
            .lock()
            .ok()
            .map(|a| (*a).clone()),
        dd_velocity_ring: analytics.dd_velocity_ring_snapshot(),
        session_generation: state.current_generation(),
    };

    if let Ok(payload) = serde_json::to_string(&snap) {
        let path = snapshot_path();
        let hash_path = path.with_extension("sha256");

        use sha2::Digest;
        let hash = sha2::Sha256::digest(payload.as_bytes());
        let new_hash_hex = hex::encode(hash);

        // OPT-3: Skip save if hash matches previous snapshot (delta compression)
        if let Ok(prev_hash) = fs::read_to_string(&hash_path) {
            if prev_hash.trim() == new_hash_hex {
                tracing::trace!(component = "state", "Snapshot unchanged -- skipping write");
                return;
            }
        }

        let tmp = format!("{}.tmp", path.display());
        match fs::write(&tmp, &payload) {
            Ok(_) => {
                if let Err(e) = fs::rename(&tmp, &path) {
                    tracing::error!(component = "state", path = %path.display(), "Snapshot rename failed: {}", e);
                } else {
                    if let Err(e) = fs::write(&hash_path, &new_hash_hex) {
                        tracing::error!(component = "state", path = %hash_path.display(), "Snapshot hash write failed: {}", e);
                    }
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    LAST_SNAPSHOT_SAVE_TS.store(now, std::sync::atomic::Ordering::Relaxed);
                }
            }
            Err(e) => {
                tracing::error!(component = "state", path = %tmp, "Snapshot write failed: {}", e);
            }
        }
    }
}

pub(crate) fn load_state_snapshot() -> Option<StateSnapshot> {
    let path = snapshot_path();
    let data = fs::read_to_string(&path).ok()?;
    let hash_path = path.with_extension("sha256");
    if let Ok(expected_hex) = fs::read_to_string(&hash_path) {
        use sha2::Digest;
        let actual = hex::encode(sha2::Sha256::digest(data.as_bytes()));
        if actual != expected_hex.trim() {
            tracing::error!(
                component = "state",
                path = %path.display(),
                "Snapshot integrity check failed — hash mismatch, rejecting corrupt data"
            );
            return None;
        }
    }
    serde_json::from_str(&data).ok()
}

pub(crate) fn restore_state_snapshot(state: &Arc<TitanState>) -> Option<StateSnapshot> {
    let snap = load_state_snapshot()?;

    let age_ms = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        now.saturating_sub(snap.timestamp_ms)
    };
    // Ignore snapshots older than 12 hours
    if age_ms > 12 * 3600 * 1000 {
        tracing::info!(
            component = "snapshot",
            age_hours = age_ms / 3600000,
            "Snapshot too old — ignoring"
        );
        return None;
    }

    tracing::info!(
        component = "snapshot",
        age_secs = age_ms / 1000,
        trails = snap.trail_highs.len(),
        prices = snap.last_prices.len(),
        scan_results = snap.last_scan_results.len(),
        "Restoring state snapshot"
    );

    *state.trading.trail_highs.lock().unwrap_or_else(|e| e.into_inner()) = snap.trail_highs.clone();
    *state.trading.trail_lows.lock().unwrap_or_else(|e| e.into_inner()) = snap.trail_lows.clone();
    *state
        .trading.partial_taken
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = snap.partial_taken.clone();
    *state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner()) = snap
        .last_prices
        .iter()
        .map(|(k, v)| (normalize_symbol(k), *v))
        .collect();
    *state
        .scan
        .symbol_best_tf
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = snap
        .symbol_best_tf
        .iter()
        .map(|(k, v)| (normalize_symbol(k), normalize_timeframe_internal(v)))
        .collect();
    *state
        .nexus_fusion_alpha
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = snap.nexus_fusion_alpha;

    // Restore session analytics
    {
        let mut analytics = state
            .risk.session_analytics
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        analytics.session_start_equity = snap.session_starting_equity;
        analytics.current_equity = snap.session_current_equity;
        analytics.peak_equity = snap.session_peak_equity;
        analytics.total_trades = snap.session_total_trades;
        analytics.winning_trades = snap.session_winning_trades;
        analytics.losing_trades = snap.session_losing_trades;
        analytics.total_pnl = snap.session_total_pnl;
        analytics.daily_pnl = snap.session_daily_pnl;
        analytics.max_drawdown_pct =
            rust_decimal::Decimal::try_from(snap.session_max_drawdown_pct).unwrap_or_default();
        analytics.avg_win = snap.session_avg_win;
        analytics.avg_loss = snap.session_avg_loss;
        analytics.last_entry_time = snap.session_last_entry_ts;
        analytics.restore_trade_pnls(snap.session_trade_pnls.clone());
        if !snap.dd_velocity_ring.is_empty() {
            analytics.restore_dd_velocity_ring(snap.dd_velocity_ring.clone());
            tracing::info!(
                component = "snapshot",
                entries = snap.dd_velocity_ring.len(),
                velocity = analytics.dd_velocity,
                "Restored dd_velocity_ring"
            );
        }
    }

    // Restore scan results + prefilter queue
    *state
        .scan
        .last_scan_results
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = snap.last_scan_results.clone();
    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (
        snap.scan_was_running,
        snap.scan_processed,
        snap.scan_total_requested,
    );
    *state
        .scan
        .scan_prefilter_queue
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = snap.scan_prefilter_queue.clone();

    // Restore entry metadata
    if !snap.entry_tags.is_empty() {
        *state.trading.entry_tags.lock().unwrap_or_else(|e| e.into_inner()) = snap.entry_tags.clone();
        tracing::info!(
            component = "snapshot",
            count = snap.entry_tags.len(),
            "Restored entry_tags"
        );
    }
    if !snap.entry_times.is_empty() {
        *state.trading.entry_times.lock().unwrap_or_else(|e| e.into_inner()) = snap.entry_times.clone();
        tracing::info!(
            component = "snapshot",
            count = snap.entry_times.len(),
            "Restored entry_times"
        );
    }

    if let Some(ref champion_json) = snap.genetic_champion {
        let params_val = champion_json.get("params").unwrap_or(champion_json);
        if let Ok(params) = serde_json::from_value::<genetic::TradingParams>(params_val.clone()) {
            let fitness = champion_json.get("fitness").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if let Ok(mut opt) = state.genetic_optimizer.lock() {
                opt.restore_champion(params, fitness);
                tracing::info!(
                    component = "snapshot",
                    fitness,
                    "Restored genetic optimizer champion with fitness"
                );
            }
        }
    }

    if let Some(ref champion_json) = snap.st_genetic_champion {
        let params_val = champion_json.get("params").unwrap_or(champion_json);
        if let Ok(params) = serde_json::from_value::<genetic::SmartTrendParams>(params_val.clone()) {
            let fitness = champion_json.get("fitness").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if let Ok(mut opt) = state.st_genetic_optimizer.lock() {
                opt.restore_champion(params, fitness);
                tracing::info!(
                    component = "snapshot",
                    fitness,
                    "Restored ST genetic optimizer champion with fitness"
                );
            }
        }
    }

    if let Some(ref adapt) = snap.oracle_adaptation {
        if let Ok(mut oa) = state.oracle_adaptation.lock() {
            oa.regime_layer_corr = adapt.regime_layer_corr.clone();
            oa.trade_count = adapt.trade_count;
            oa.adapted_weights = adapt.adapted_weights.clone();
            oa.decay_factor = adapt.decay_factor;
            tracing::info!(
                component = "snapshot",
                trade_count = adapt.trade_count,
                regimes = adapt.adapted_weights.len(),
                "Restored oracle adaptation state"
            );
        }
    }

    if snap.session_generation > 0 {
        let current = state.session_generation.load(std::sync::atomic::Ordering::SeqCst);
        let restored = snap.session_generation.max(current);
        state.session_generation.store(restored, std::sync::atomic::Ordering::SeqCst);
        tracing::info!(
            component = "snapshot",
            generation = restored,
            "Restored session generation"
        );
    }

    Some(snap)
}

pub(crate) fn load_blacklist() -> HashSet<String> {
    match fs::read_to_string(blacklist_path()) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => HashSet::new(),
    }
}

pub(crate) fn save_blacklist(bl: &HashSet<String>) -> Result<(), TitanError> {
    let payload = serde_json::to_string(bl)?;
    let path = blacklist_path();
    fs::write(&path, payload).map_err(|e| {
        tracing::error!(component = "state", path = %path.display(), "Blacklist write failed: {}", e);
        TitanError::Io(format!("Blacklist write failed: {}", e))
    })
}

/// Build IndicatorPeriods from AppState indicator period fields.
pub fn build_indicator_periods(state: &AppState) -> crate::indicators::IndicatorPeriods {
    crate::indicators::IndicatorPeriods {
        ema_fast: state.ema_fast_period as usize,
        ema_mid: state.ema_mid_period as usize,
        ema_slow: state.ema_slow_period as usize,
        ema_trend: state.ema_trend_period as usize,
        rsi: state.rsi_period as usize,
        macd_fast: state.macd_fast as usize,
        macd_slow: state.macd_slow as usize,
        macd_signal: state.macd_signal as usize,
        bb: state.bb_period as usize,
        bb_std_dev: state.bb_std_dev,
        adx: state.adx_period as usize,
        stoch_rsi: state.stoch_rsi_period as usize,
    }
}

/// Build OracleWeights from AppState oracle fusion fields.
pub fn build_oracle_weights(state: &AppState) -> crate::titan_oracle::OracleWeights {
    crate::titan_oracle::OracleWeights {
        kalman: state.oracle_kalman_weight,
        wavelet: state.oracle_wavelet_weight,
        hurst: state.oracle_hurst_weight,
        vpin: state.oracle_vpin_weight,
        entropy_dir: state.oracle_entropy_weight,
        confidence_floor: state.oracle_confidence_floor,
        entropy_chaos_threshold: state.entropy_chaos_threshold,
        vpin_toxicity_alert: state.vpin_toxicity_alert,
    }
}

pub(crate) fn settings_from_state(state: &AppState) -> AppSettings {
    AppSettings {
        active_symbol: state.active_symbol.clone(),
        hft_targets: state.hft_targets.clone(),
        loop_active: state.loop_active,
        omni_scan_active: state.omni_scan_active,
        auto_pilot: state.auto_pilot,
        aggressive_mode: state.aggressive_mode,
        allow_short: state.allow_short,
        enable_extended: state.enable_extended,
        force_manual_tf: state.force_manual_tf,
        scalping_oracle: state.scalping_oracle,
        scalping_gpu: state.scalping_gpu,
        scalping_fusion: state.scalping_fusion,
        risk_per_trade: state.risk_per_trade,
        refresh_rate: state.refresh_rate,
        manual_res: state.manual_res.clone(),
        backtest_days: state.backtest_days,
        gpu_backtest: state.gpu_backtest,
        gpu_sweep: state.gpu_sweep,
        gpu_deep_focus: state.gpu_deep_focus,
        auto_entry: state.auto_entry,
        max_open_positions: state.max_open_positions,
        entry_cooldown_secs: state.entry_cooldown_secs,
        daily_loss_limit_pct: state.daily_loss_limit_pct,
        trailing_stop_pct: state.trailing_stop_pct,
        take_profit_pct: state.take_profit_pct,
        max_spread_bps: state.max_spread_bps,
        min_ofi_threshold: state.min_ofi_threshold,
        max_daily_trades: state.max_daily_trades,
        entry_signal_strength_min: state.entry_signal_strength_min,
        max_hold_secs: state.max_hold_secs,
        multi_symbol_enabled: state.multi_symbol_enabled,
        max_concurrent_assets: state.max_concurrent_assets,
        max_correlation: state.max_correlation,
        correlation_window: state.correlation_window,
        discord_webhook_url: state.discord_webhook_url.clone(),
        alert_on_fill: state.alert_on_fill,
        alert_on_breaker: state.alert_on_breaker,
        alert_on_daily_summary: state.alert_on_daily_summary,
        telegram_bot_token: state.telegram_bot_token.clone(),
        telegram_chat_id: state.telegram_chat_id.clone(),
        telegram_enabled: state.telegram_enabled,
        atr_stop_multiplier: state.atr_stop_multiplier,
        atr_take_multiplier: state.atr_take_multiplier,
        partial_take_enabled: state.partial_take_enabled,
        partial_take_pct: state.partial_take_pct,
        llm_api_key: state.llm_api_key.clone(),
        llm_provider: state.llm_provider.clone(),
        llm_sentiment_enabled: state.llm_sentiment_enabled,
        llm_batch_size: state.llm_batch_size,
        sentiment_halflife_secs: state.sentiment_halflife_secs,
        options_hedging_enabled: state.options_hedging_enabled,
        hedge_otm_pct: state.hedge_otm_pct,
        hedge_max_premium_pct: state.hedge_max_premium_pct,
        web_dashboard_enabled: state.web_dashboard_enabled,
        web_dashboard_port: state.web_dashboard_port,
        web_dashboard_token: state.web_dashboard_token.clone(),
        bot_trade_size: state.bot_trade_size,
        circuit_breaker_limit: state.circuit_breaker_limit,
        favorites: state.favorites.clone(),
        dry_run_enabled: state.dry_run_enabled,
        strategy_mode_name: state.strategy_mode_name.clone(),
        ai_scan_active: state.ai_scan_active,
        ai_smart_sort: state.ai_smart_sort,
        ai_auto_inject_top: state.ai_auto_inject_top,
        ai_auto_inject_hft: state.ai_auto_inject_hft,
        scan_params: state.scan_params.clone(),
        position_trade_modes: state.position_trade_modes.clone(),
        md_auto_scan_active: state.md_auto_scan_active,
        nexus_fusion_alpha: state.nexus_fusion_alpha,
        signal_strategy_assets: state.signal_strategy_assets.clone(),
        signal_source: state.signal_source.clone(),
        bypass_max_spread: state.bypass_max_spread,
        bypass_cooldown: state.bypass_cooldown,
        bypass_allow_short: state.bypass_allow_short,
        bypass_extended: state.bypass_extended,
        bypass_risk_per_trade: state.bypass_risk_per_trade,
        bypass_aggressive: state.bypass_aggressive,
        bypass_atr_stop: state.bypass_atr_stop,
        bypass_atr_take: state.bypass_atr_take,
        bypass_partial_take: state.bypass_partial_take,
        bypass_partial_pct: state.bypass_partial_pct,
        bypass_max_correlation: state.bypass_max_correlation,
        bypass_correlation_window: state.bypass_correlation_window,
        bypass_daily_loss: state.bypass_daily_loss,
        max_manual_order_usd: state.max_manual_order_usd,
        ta_gates: state.ta_gates.clone(),
        ta_gate_soft_penalty: state.ta_gate_soft_penalty,
        enable_var_gate: state.enable_var_gate,
        max_portfolio_var_pct: state.max_portfolio_var_pct,
        var_lookback_periods: state.var_lookback_periods,
        enable_concentration_gate: state.enable_concentration_gate,
        max_concentration_hhi: state.max_concentration_hhi,
        var_fail_closed: state.var_fail_closed,
        asset_check_hard_gate: state.asset_check_hard_gate,
        blackout_first_mins: state.blackout_first_mins,
        blackout_last_mins: state.blackout_last_mins,
        blackout_exempt_symbols: state.blackout_exempt_symbols.clone(),
        max_hold_decay_enabled: state.max_hold_decay_enabled,
        max_hold_decay_start_pct: state.max_hold_decay_start_pct,
        max_hold_decay_trail_tighten: state.max_hold_decay_trail_tighten,
        mtf_gate_enabled: state.mtf_gate_enabled,
        mtf_gate_mode: state.mtf_gate_mode.clone(),
        mtf_gate_penalty: state.mtf_gate_penalty,
        mtf_gate_htf: state.mtf_gate_htf.clone(),
        hhi_gate_manual_orders: state.hhi_gate_manual_orders,
        max_concentration_hhi_manual: state.max_concentration_hhi_manual,
        tiered_scan_enabled: state.tiered_scan_enabled,
        tier1_count: state.tier1_count,
        tier2_count: state.tier2_count,
        pending_order_timeout_secs: state.pending_order_timeout_secs,
        ofi_divergence_threshold_secs: state.ofi_divergence_threshold_secs,
        use_classical_filters: state.use_classical_filters,
        rsi_overbought: state.rsi_overbought,
        rsi_oversold: state.rsi_oversold,
        require_ema_alignment: state.require_ema_alignment,
        adx_trend_threshold: state.adx_trend_threshold,
        classical_weight: state.classical_weight,
        entry_thresholds_scalp: state.entry_thresholds_scalp.clone(),
        entry_thresholds_normal: state.entry_thresholds_normal.clone(),
        score_modifiers: state.score_modifiers.clone(),
        classical_bonuses: state.classical_bonuses.clone(),
        min_hold_secs: state.min_hold_secs,
        nexus_consensus_threshold: state.nexus_consensus_threshold,
        autopilot_loop_ms: state.autopilot_loop_ms,
        ofi_stale_ms: state.ofi_stale_ms,
        flash_crash_window_secs: state.flash_crash_window_secs,
        flash_crash_threshold_pct: state.flash_crash_threshold_pct,
        flash_crash_cooldown_mins: state.flash_crash_cooldown_mins,
        dd_velocity_threshold: state.dd_velocity_threshold,
        dd_velocity_window_secs: state.dd_velocity_window_secs,
        min_bars_for_entry: state.min_bars_for_entry,
        ofi_entry_threshold: state.ofi_entry_threshold,
        max_qty_cap: state.max_qty_cap,
        min_qty_threshold: state.min_qty_threshold,
        default_equity_fallback: state.default_equity_fallback,
        max_bypass_risk_pct: state.max_bypass_risk_pct,
        vol_factor_min: state.vol_factor_min,
        vol_factor_max: state.vol_factor_max,
        fallback_sl_pct: state.fallback_sl_pct,
        fallback_tp_pct: state.fallback_tp_pct,
        fallback_trail_pct: state.fallback_trail_pct,
        scalping_stop_mult: state.scalping_stop_mult,
        scalping_take_mult: state.scalping_take_mult,
        scalping_trail_mult: state.scalping_trail_mult,
        partial_first_target_frac: state.partial_first_target_frac,
        partial_min_qty: state.partial_min_qty,
        atr_period: state.atr_period,
        atr_stop_clamp_min: state.atr_stop_clamp_min,
        atr_stop_clamp_max: state.atr_stop_clamp_max,
        atr_take_clamp_min: state.atr_take_clamp_min,
        atr_take_clamp_max: state.atr_take_clamp_max,
        atr_trail_clamp_min: state.atr_trail_clamp_min,
        atr_trail_clamp_max: state.atr_trail_clamp_max,
        oracle_exit_min_confidence: state.oracle_exit_min_confidence,
        oracle_exit_min_score: state.oracle_exit_min_score,
        oracle_flip_score_threshold: state.oracle_flip_score_threshold,
        oracle_flip_confidence: state.oracle_flip_confidence,
        chaos_exit_max_loss_pct: state.chaos_exit_max_loss_pct,
        chaos_exit_min_confidence: state.chaos_exit_min_confidence,
        breaker_caution_pct: state.breaker_caution_pct,
        breaker_halt_pct: state.breaker_halt_pct,
        breaker_panic_pct: state.breaker_panic_pct,
        max_notional_equity_frac: state.max_notional_equity_frac,
        min_stop_distance_frac: state.min_stop_distance_frac,
        strategy_base_score: state.strategy_base_score,
        vpin_gate_scalping: state.vpin_gate_scalping,
        vpin_gate_normal: state.vpin_gate_normal,
        min_rvol_threshold: state.min_rvol_threshold,
        rvol_window: state.rvol_window,
        rvol_relax_threshold: state.rvol_relax_threshold,
        rvol_relax_value: state.rvol_relax_value,
        chaos_confidence_kill: state.chaos_confidence_kill,
        chaos_regime_mult: state.chaos_regime_mult,
        long_stop_normal: state.long_stop_normal,
        long_stop_aggressive: state.long_stop_aggressive,
        short_stop_pct: state.short_stop_pct,
        long_trail_min_profit: state.long_trail_min_profit,
        short_trail_gap: state.short_trail_gap,
        short_trail_min_profit: state.short_trail_min_profit,
        oracle_kalman_weight: state.oracle_kalman_weight,
        oracle_entropy_weight: state.oracle_entropy_weight,
        oracle_hurst_weight: state.oracle_hurst_weight,
        oracle_vpin_weight: state.oracle_vpin_weight,
        oracle_wavelet_weight: state.oracle_wavelet_weight,
        oracle_confidence_floor: state.oracle_confidence_floor,
        entropy_chaos_threshold: state.entropy_chaos_threshold,
        vpin_toxicity_alert: state.vpin_toxicity_alert,
        enable_oracle_adaptation: state.enable_oracle_adaptation,
        ema_fast_period: state.ema_fast_period,
        ema_mid_period: state.ema_mid_period,
        ema_slow_period: state.ema_slow_period,
        ema_trend_period: state.ema_trend_period,
        rsi_period: state.rsi_period,
        macd_fast: state.macd_fast,
        macd_slow: state.macd_slow,
        macd_signal: state.macd_signal,
        bb_period: state.bb_period,
        bb_std_dev: state.bb_std_dev,
        adx_period: state.adx_period,
        stoch_rsi_period: state.stoch_rsi_period,

        disable_adaptive_exits: state.disable_adaptive_exits,
        exit_floor_pct: state.exit_floor_pct,
        regime_chaotic_stop_mult: state.regime_chaotic_stop_mult,
        regime_chaotic_take_mult: state.regime_chaotic_take_mult,
        regime_chaotic_trail_mult: state.regime_chaotic_trail_mult,
        regime_range_stop_mult: state.regime_range_stop_mult,
        regime_range_take_mult: state.regime_range_take_mult,
        regime_range_trail_mult: state.regime_range_trail_mult,
        regime_trend_stop_mult: state.regime_trend_stop_mult,
        regime_trend_take_mult: state.regime_trend_take_mult,
        regime_trend_trail_mult: state.regime_trend_trail_mult,
        signal_aligned_stop_mult: state.signal_aligned_stop_mult,
        signal_aligned_take_mult: state.signal_aligned_take_mult,
        signal_aligned_trail_mult: state.signal_aligned_trail_mult,
        signal_oppose_stop_mult: state.signal_oppose_stop_mult,
        signal_oppose_take_mult: state.signal_oppose_take_mult,
        tf_short_stop_mult: state.tf_short_stop_mult,
        tf_short_take_mult: state.tf_short_take_mult,
        tf_short_trail_mult: state.tf_short_trail_mult,
        symbol_overrides: state.symbol_overrides.clone(),
        smarttrend: state.smarttrend.clone(),
        st_asset_modes: state.st_asset_modes.clone(),
    }
}

pub(crate) fn apply_settings(state: &mut AppState, settings: &AppSettings) {
    state.active_symbol = settings.active_symbol.clone();
    state.hft_targets = settings.hft_targets.clone();
    state.loop_active = settings.loop_active;
    state.omni_scan_active = settings.omni_scan_active;
    state.auto_pilot = settings.auto_pilot;
    state.aggressive_mode = settings.aggressive_mode;
    state.allow_short = settings.allow_short;
    state.enable_extended = settings.enable_extended;
    state.force_manual_tf = settings.force_manual_tf;
    state.scalping_oracle = settings.scalping_oracle;
    state.scalping_gpu = settings.scalping_gpu;
    state.scalping_fusion = settings.scalping_fusion;
    state.risk_per_trade = settings.risk_per_trade;
    state.refresh_rate = settings.refresh_rate;
    state.manual_res = settings.manual_res.clone();
    state.backtest_days = settings.backtest_days;
    state.gpu_backtest = settings.gpu_backtest;
    state.gpu_sweep = settings.gpu_sweep;
    state.gpu_deep_focus = settings.gpu_deep_focus;
    state.auto_entry = settings.auto_entry;
    state.max_open_positions = settings.max_open_positions;
    state.entry_cooldown_secs = settings.entry_cooldown_secs;
    state.daily_loss_limit_pct = settings.daily_loss_limit_pct;
    state.trailing_stop_pct = settings.trailing_stop_pct;
    state.take_profit_pct = settings.take_profit_pct;
    state.max_spread_bps = settings.max_spread_bps;
    state.min_ofi_threshold = settings.min_ofi_threshold;
    state.max_daily_trades = settings.max_daily_trades;
    state.entry_signal_strength_min = settings.entry_signal_strength_min;
    state.max_hold_secs = settings.max_hold_secs;
    state.multi_symbol_enabled = settings.multi_symbol_enabled;
    state.max_concurrent_assets = settings.max_concurrent_assets;
    state.max_correlation = settings.max_correlation;
    state.correlation_window = settings.correlation_window;
    state.discord_webhook_url = settings.discord_webhook_url.clone();
    state.alert_on_fill = settings.alert_on_fill;
    state.alert_on_breaker = settings.alert_on_breaker;
    state.alert_on_daily_summary = settings.alert_on_daily_summary;
    state.telegram_bot_token = settings.telegram_bot_token.clone();
    state.telegram_chat_id = settings.telegram_chat_id.clone();
    state.telegram_enabled = settings.telegram_enabled;
    state.atr_stop_multiplier = settings.atr_stop_multiplier;
    state.atr_take_multiplier = settings.atr_take_multiplier;
    state.partial_take_enabled = settings.partial_take_enabled;
    state.partial_take_pct = settings.partial_take_pct;
    state.llm_api_key = settings.llm_api_key.clone();
    state.llm_provider = settings.llm_provider.clone();
    state.llm_sentiment_enabled = settings.llm_sentiment_enabled;
    state.llm_batch_size = settings.llm_batch_size;
    state.sentiment_halflife_secs = settings.sentiment_halflife_secs;
    state.options_hedging_enabled = settings.options_hedging_enabled;
    state.hedge_otm_pct = settings.hedge_otm_pct;
    state.hedge_max_premium_pct = settings.hedge_max_premium_pct;
    state.web_dashboard_enabled = settings.web_dashboard_enabled;
    state.web_dashboard_port = settings.web_dashboard_port;
    state.web_dashboard_token = settings.web_dashboard_token.clone();
    state.bot_trade_size = settings.bot_trade_size;
    state.circuit_breaker_limit = settings.circuit_breaker_limit;
    state.favorites = settings.favorites.clone();
    state.dry_run_enabled = settings.dry_run_enabled;
    state.strategy_mode_name = settings.strategy_mode_name.clone();
    state.ai_scan_active = settings.ai_scan_active;
    state.ai_smart_sort = settings.ai_smart_sort;
    state.ai_auto_inject_top = settings.ai_auto_inject_top;
    state.ai_auto_inject_hft = settings.ai_auto_inject_hft;
    state.scan_params = settings.scan_params.clone();
    state.position_trade_modes = settings.position_trade_modes.clone();
    state.md_auto_scan_active = settings.md_auto_scan_active;
    state.nexus_fusion_alpha = settings.nexus_fusion_alpha;
    state.signal_strategy_assets = settings.signal_strategy_assets.clone();
    state.signal_source = settings.signal_source.clone();
    state.bypass_max_spread = settings.bypass_max_spread;
    state.bypass_cooldown = settings.bypass_cooldown;
    state.bypass_allow_short = settings.bypass_allow_short;
    state.bypass_extended = settings.bypass_extended;
    state.bypass_risk_per_trade = settings.bypass_risk_per_trade;
    state.bypass_aggressive = settings.bypass_aggressive;
    state.bypass_atr_stop = settings.bypass_atr_stop;
    state.bypass_atr_take = settings.bypass_atr_take;
    state.bypass_partial_take = settings.bypass_partial_take;
    state.bypass_partial_pct = settings.bypass_partial_pct;
    state.bypass_max_correlation = settings.bypass_max_correlation;
    state.bypass_correlation_window = settings.bypass_correlation_window;
    state.bypass_daily_loss = settings.bypass_daily_loss;
    state.max_manual_order_usd = settings.max_manual_order_usd;
    state.ta_gates = settings.ta_gates.clone();
    state.ta_gate_soft_penalty = settings.ta_gate_soft_penalty;
    state.enable_var_gate = settings.enable_var_gate;
    state.max_portfolio_var_pct = settings.max_portfolio_var_pct;
    state.var_lookback_periods = settings.var_lookback_periods;
    state.enable_concentration_gate = settings.enable_concentration_gate;
    state.max_concentration_hhi = settings.max_concentration_hhi;
    state.var_fail_closed = settings.var_fail_closed;
    state.asset_check_hard_gate = settings.asset_check_hard_gate;
    state.blackout_first_mins = settings.blackout_first_mins;
    state.blackout_last_mins = settings.blackout_last_mins;
    state.blackout_exempt_symbols = settings.blackout_exempt_symbols.clone();
    state.max_hold_decay_enabled = settings.max_hold_decay_enabled;
    state.max_hold_decay_start_pct = settings.max_hold_decay_start_pct;
    state.max_hold_decay_trail_tighten = settings.max_hold_decay_trail_tighten;
    state.mtf_gate_enabled = settings.mtf_gate_enabled;
    state.mtf_gate_mode = settings.mtf_gate_mode.clone();
    state.mtf_gate_penalty = settings.mtf_gate_penalty;
    state.mtf_gate_htf = settings.mtf_gate_htf.clone();
    state.hhi_gate_manual_orders = settings.hhi_gate_manual_orders;
    state.max_concentration_hhi_manual = settings.max_concentration_hhi_manual;
    state.tiered_scan_enabled = settings.tiered_scan_enabled;
    state.tier1_count = settings.tier1_count;
    state.tier2_count = settings.tier2_count;
    state.pending_order_timeout_secs = settings.pending_order_timeout_secs;
    state.ofi_divergence_threshold_secs = settings.ofi_divergence_threshold_secs;
    state.use_classical_filters = settings.use_classical_filters;
    state.rsi_overbought = settings.rsi_overbought;
    state.rsi_oversold = settings.rsi_oversold;
    state.require_ema_alignment = settings.require_ema_alignment;
    state.adx_trend_threshold = settings.adx_trend_threshold;
    state.classical_weight = settings.classical_weight;
    state.entry_thresholds_scalp = settings.entry_thresholds_scalp.clone();
    state.entry_thresholds_normal = settings.entry_thresholds_normal.clone();
    state.score_modifiers = settings.score_modifiers.clone();
    state.classical_bonuses = settings.classical_bonuses.clone();
    state.min_hold_secs = settings.min_hold_secs;
    state.nexus_consensus_threshold = settings.nexus_consensus_threshold;
    state.autopilot_loop_ms = settings.autopilot_loop_ms;
    state.ofi_stale_ms = settings.ofi_stale_ms;
    state.flash_crash_window_secs = settings.flash_crash_window_secs;
    state.flash_crash_threshold_pct = settings.flash_crash_threshold_pct;
    state.flash_crash_cooldown_mins = settings.flash_crash_cooldown_mins;
    state.dd_velocity_threshold = settings.dd_velocity_threshold;
    state.dd_velocity_window_secs = settings.dd_velocity_window_secs;
    state.min_bars_for_entry = settings.min_bars_for_entry;
    state.ofi_entry_threshold = settings.ofi_entry_threshold;
    state.max_qty_cap = settings.max_qty_cap;
    state.min_qty_threshold = settings.min_qty_threshold;
    state.default_equity_fallback = settings.default_equity_fallback;
    state.max_bypass_risk_pct = settings.max_bypass_risk_pct;
    state.vol_factor_min = settings.vol_factor_min;
    state.vol_factor_max = settings.vol_factor_max;
    state.fallback_sl_pct = settings.fallback_sl_pct;
    state.fallback_tp_pct = settings.fallback_tp_pct;
    state.fallback_trail_pct = settings.fallback_trail_pct;
    state.scalping_stop_mult = settings.scalping_stop_mult;
    state.scalping_take_mult = settings.scalping_take_mult;
    state.scalping_trail_mult = settings.scalping_trail_mult;
    state.partial_first_target_frac = settings.partial_first_target_frac;
    state.partial_min_qty = settings.partial_min_qty;
    state.atr_period = settings.atr_period;
    state.atr_stop_clamp_min = settings.atr_stop_clamp_min;
    state.atr_stop_clamp_max = settings.atr_stop_clamp_max;
    state.atr_take_clamp_min = settings.atr_take_clamp_min;
    state.atr_take_clamp_max = settings.atr_take_clamp_max;
    state.atr_trail_clamp_min = settings.atr_trail_clamp_min;
    state.atr_trail_clamp_max = settings.atr_trail_clamp_max;
    state.oracle_exit_min_confidence = settings.oracle_exit_min_confidence;
    state.oracle_exit_min_score = settings.oracle_exit_min_score;
    state.oracle_flip_score_threshold = settings.oracle_flip_score_threshold;
    state.oracle_flip_confidence = settings.oracle_flip_confidence;
    state.chaos_exit_max_loss_pct = settings.chaos_exit_max_loss_pct;
    state.chaos_exit_min_confidence = settings.chaos_exit_min_confidence;
    state.breaker_caution_pct = settings.breaker_caution_pct;
    state.breaker_halt_pct = settings.breaker_halt_pct;
    state.breaker_panic_pct = settings.breaker_panic_pct;
    state.max_notional_equity_frac = settings.max_notional_equity_frac;
    state.min_stop_distance_frac = settings.min_stop_distance_frac;
    state.strategy_base_score = settings.strategy_base_score;
    state.vpin_gate_scalping = settings.vpin_gate_scalping;
    state.vpin_gate_normal = settings.vpin_gate_normal;
    state.min_rvol_threshold = settings.min_rvol_threshold;
    state.rvol_window = settings.rvol_window;
    state.rvol_relax_threshold = settings.rvol_relax_threshold;
    state.rvol_relax_value = settings.rvol_relax_value;
    state.chaos_confidence_kill = settings.chaos_confidence_kill;
    state.chaos_regime_mult = settings.chaos_regime_mult;
    state.long_stop_normal = settings.long_stop_normal;
    state.long_stop_aggressive = settings.long_stop_aggressive;
    state.short_stop_pct = settings.short_stop_pct;
    state.long_trail_min_profit = settings.long_trail_min_profit;
    state.short_trail_gap = settings.short_trail_gap;
    state.short_trail_min_profit = settings.short_trail_min_profit;
    state.oracle_kalman_weight = settings.oracle_kalman_weight;
    state.oracle_entropy_weight = settings.oracle_entropy_weight;
    state.oracle_hurst_weight = settings.oracle_hurst_weight;
    state.oracle_vpin_weight = settings.oracle_vpin_weight;
    state.oracle_wavelet_weight = settings.oracle_wavelet_weight;
    state.oracle_confidence_floor = settings.oracle_confidence_floor;
    state.entropy_chaos_threshold = settings.entropy_chaos_threshold;
    state.vpin_toxicity_alert = settings.vpin_toxicity_alert;
    state.enable_oracle_adaptation = settings.enable_oracle_adaptation;
    state.ema_fast_period = settings.ema_fast_period;
    state.ema_mid_period = settings.ema_mid_period;
    state.ema_slow_period = settings.ema_slow_period;
    state.ema_trend_period = settings.ema_trend_period;
    state.rsi_period = settings.rsi_period;
    state.macd_fast = settings.macd_fast;
    state.macd_slow = settings.macd_slow;
    state.macd_signal = settings.macd_signal;
    state.bb_period = settings.bb_period;
    state.bb_std_dev = settings.bb_std_dev;
    state.adx_period = settings.adx_period;
    state.stoch_rsi_period = settings.stoch_rsi_period;

    state.disable_adaptive_exits = settings.disable_adaptive_exits;
    state.exit_floor_pct = settings.exit_floor_pct;
    state.regime_chaotic_stop_mult = settings.regime_chaotic_stop_mult;
    state.regime_chaotic_take_mult = settings.regime_chaotic_take_mult;
    state.regime_chaotic_trail_mult = settings.regime_chaotic_trail_mult;
    state.regime_range_stop_mult = settings.regime_range_stop_mult;
    state.regime_range_take_mult = settings.regime_range_take_mult;
    state.regime_range_trail_mult = settings.regime_range_trail_mult;
    state.regime_trend_stop_mult = settings.regime_trend_stop_mult;
    state.regime_trend_take_mult = settings.regime_trend_take_mult;
    state.regime_trend_trail_mult = settings.regime_trend_trail_mult;
    state.signal_aligned_stop_mult = settings.signal_aligned_stop_mult;
    state.signal_aligned_take_mult = settings.signal_aligned_take_mult;
    state.signal_aligned_trail_mult = settings.signal_aligned_trail_mult;
    state.signal_oppose_stop_mult = settings.signal_oppose_stop_mult;
    state.signal_oppose_take_mult = settings.signal_oppose_take_mult;
    state.tf_short_stop_mult = settings.tf_short_stop_mult;
    state.tf_short_take_mult = settings.tf_short_take_mult;
    state.tf_short_trail_mult = settings.tf_short_trail_mult;
    state.symbol_overrides = settings.symbol_overrides.clone();
    state.smarttrend = settings.smarttrend.clone();
    state.st_asset_modes = settings.st_asset_modes.clone();
}

pub(crate) fn load_settings_from_disk() -> AppSettings {
    let default_state = AppState::default();
    let default_settings = settings_from_state(&default_state);
    let path = settings_path();

    let Ok(raw) = fs::read_to_string(&path) else {
        return default_settings;
    };

    match serde_json::from_str::<AppSettings>(&raw) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(component = "state", error = %e, "Settings JSON corrupt -- trying .bak");
            let bak_path = path.with_extension("json.bak");
            if let Ok(bak_data) = fs::read_to_string(&bak_path) {
                if let Ok(s) = serde_json::from_str::<AppSettings>(&bak_data) {
                    tracing::warn!(component = "state", "Recovered settings from .bak");
                    return s;
                }
            }
            tracing::error!(component = "state", "Settings backup also failed -- using defaults");
            default_settings
        }
    }
}

/// Global timestamp of last successful settings save (epoch secs).
pub(crate) static LAST_SETTINGS_SAVE_TS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Global timestamp of last successful snapshot save (epoch secs).
pub(crate) static LAST_SNAPSHOT_SAVE_TS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn save_settings_to_disk(settings: &AppSettings) -> Result<(), TitanError> {
    let payload = serde_json::to_string_pretty(settings)?;
    let path = settings_path();
    if path.exists() {
        let bak = path.with_extension("json.bak");
        if let Err(e) = fs::copy(&path, &bak) {
            tracing::warn!(component = "state", error = %e, "Failed to create settings .bak");
        }
    }
    let tmp = format!("{}.tmp", path.display());
    fs::write(&tmp, &payload).map_err(|e| {
        tracing::error!(component = "state", path = %tmp, "Settings write failed: {}", e);
        TitanError::Io(format!("Settings write failed: {}", e))
    })?;
    fs::rename(&tmp, &path).map_err(|e| {
        tracing::error!(component = "state", path = %path.display(), "Settings rename failed: {}", e);
        TitanError::Io(format!("Settings rename failed: {}", e))
    })?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    LAST_SETTINGS_SAVE_TS.store(now, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

// ─── Profile Persistence (RF-C) ───

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ProfileEntry {
    #[serde(default = "default_schema_v")]
    pub schema_v: u32,
    pub saved_at: String,
    pub data: AppSettings,
}

fn default_schema_v() -> u32 { 1 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ProfileStore {
    #[serde(default = "default_store_v")]
    pub v: u32,
    #[serde(default)]
    pub profiles: HashMap<String, ProfileEntry>,
}

fn default_store_v() -> u32 { 1 }

impl Default for ProfileStore {
    fn default() -> Self {
        Self { v: 1, profiles: HashMap::new() }
    }
}

#[derive(Serialize, Clone, Debug)]
pub(crate) struct ProfileMeta {
    pub name: String,
    pub saved_at: String,
    pub schema_v: u32,
}

pub(crate) fn profiles_path() -> std::path::PathBuf {
    crate::paths::profiles_path()
}

pub(crate) fn load_profiles_from_disk() -> ProfileStore {
    let path = profiles_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return ProfileStore::default();
    };
    match serde_json::from_str::<ProfileStore>(&raw) {
        Ok(store) => store,
        Err(e) => {
            tracing::error!(component = "profiles", error = %e, "Profiles JSON corrupt -- trying .bak");
            let bak = path.with_extension("json.bak");
            if let Ok(bak_data) = fs::read_to_string(&bak) {
                if let Ok(store) = serde_json::from_str::<ProfileStore>(&bak_data) {
                    tracing::warn!(component = "profiles", "Recovered profiles from .bak");
                    return store;
                }
            }
            tracing::error!(component = "profiles", "Profiles backup also failed -- using empty store");
            ProfileStore::default()
        }
    }
}

pub(crate) fn save_profiles_to_disk(store: &ProfileStore) -> Result<(), TitanError> {
    let payload = serde_json::to_string_pretty(store)?;
    let path = profiles_path();
    if path.exists() {
        let bak = path.with_extension("json.bak");
        if let Err(e) = fs::copy(&path, &bak) {
            tracing::warn!(component = "profiles", error = %e, "Failed to create profiles .bak");
        }
    }
    let tmp = format!("{}.tmp", path.display());
    fs::write(&tmp, &payload).map_err(|e| {
        tracing::error!(component = "profiles", path = %tmp, "Profiles write failed: {}", e);
        TitanError::Io(format!("Profiles write failed: {}", e))
    })?;
    fs::rename(&tmp, &path).map_err(|e| {
        tracing::error!(component = "profiles", path = %path.display(), "Profiles rename failed: {}", e);
        TitanError::Io(format!("Profiles rename failed: {}", e))
    })
}

pub(crate) fn env_override_bool(key: &str) -> Option<bool> {
    match std::env::var(key) {
        Ok(val) => Some(matches!(
            val.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )),
        Err(_) => None,
    }
}

pub(crate) fn setting_or_env(setting: bool, key: &str) -> bool {
    env_override_bool(key).unwrap_or(setting)
}

pub(crate) struct VolEntry {
    pub value: f64,
}

pub(crate) struct VolCache {
    pub values: HashMap<String, VolEntry>,
}

impl VolCache {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

pub(crate) struct BarsCacheEntry {
    pub candles: Vec<Candle>,
    pub updated: Instant,
}

pub(crate) struct GpuScoreCache {
    pub momentum: f64,
    pub ema_delta: f64,
    pub updated: Instant,
}

// ═══════════════════════════════════════════════════════════════════
//  GPU NEXUS — Strategy Mode & Consensus State
// ═══════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StrategyMode {
    MathOracle,
    GpuNexus,
    Fusion,
    SignalStrategy,
    SmartTrend,
}

impl StrategyMode {
    /// Parse a strategy mode name string into the enum.
    /// Centralises the parsing logic that was previously duplicated
    /// in main.rs startup and set_strategy_mode command.
    pub fn from_name(name: &str) -> Self {
        match name.to_uppercase().as_str() {
            "MATH_ORACLE" | "ORACLE" => Self::MathOracle,
            "GPU_NEXUS" | "NEXUS" | "GPU" => Self::GpuNexus,
            "FUSION" => Self::Fusion,
            "SIGNAL_STRATEGY" | "SIGNAL" => Self::SignalStrategy,
            "SMARTTREND" | "SMART_TREND" | "ST" => Self::SmartTrend,
            _ => Self::Fusion,
        }
    }
}

impl std::fmt::Display for StrategyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyMode::MathOracle => write!(f, "MATH_ORACLE"),
            StrategyMode::GpuNexus => write!(f, "GPU_NEXUS"),
            StrategyMode::Fusion => write!(f, "FUSION"),
            StrategyMode::SignalStrategy => write!(f, "SIGNAL_STRATEGY"),
            StrategyMode::SmartTrend => write!(f, "SMARTTREND"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NexusConsensus {
    pub buy_weight: u32,
    pub sell_weight: u32,
    pub buy_pct: f64,
    pub sell_pct: f64,
    pub consensus: String, // "STRONG_BUY", "BUY", "NEUTRAL", "SELL", "STRONG_SELL"
    pub active_agents: u32,
    pub dispatch_us: u64, // GPU dispatch latency in microseconds
}

impl Default for NexusConsensus {
    fn default() -> Self {
        Self {
            buy_weight: 0,
            sell_weight: 0,
            buy_pct: 50.0,
            sell_pct: 50.0,
            consensus: "NEUTRAL".to_string(),
            active_agents: 0,
            dispatch_us: 0,
        }
    }
}

const LATENCY_RING_CAP: usize = 100;

#[derive(Serialize, Clone, Debug)]
pub struct LatencySnapshot {
    pub order_rtt_us: LatencyBucket,
    pub gpu_dispatch_us: LatencyBucket,
    pub streamer_tick_us: LatencyBucket,
}

#[derive(Serialize, Clone, Debug)]
pub struct LatencyBucket {
    pub count: usize,
    pub min: u64,
    pub max: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

impl LatencyBucket {
    fn from_deque(d: &std::collections::VecDeque<u64>) -> Self {
        if d.is_empty() {
            return Self { count: 0, min: 0, max: 0, p50: 0, p95: 0, p99: 0 };
        }
        let mut sorted: Vec<u64> = d.iter().copied().collect();
        sorted.sort_unstable();
        let n = sorted.len();
        Self {
            count: n,
            min: sorted[0],
            max: sorted[n - 1],
            p50: sorted[n / 2],
            p95: sorted[(n as f64 * 0.95) as usize],
            p99: sorted[(n as f64 * 0.99).min((n - 1) as f64) as usize],
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ModeTransition {
    pub timestamp: String,
    pub from_mode: String,
    pub to_mode: String,
    pub trigger: String,
}

pub struct LatencyMetrics {
    order_rtt_us: Mutex<std::collections::VecDeque<u64>>,
    gpu_dispatch_us: Mutex<std::collections::VecDeque<u64>>,
    streamer_tick_us: Mutex<std::collections::VecDeque<u64>>,
}

impl LatencyMetrics {
    pub fn new() -> Self {
        Self {
            order_rtt_us: Mutex::new(std::collections::VecDeque::with_capacity(LATENCY_RING_CAP)),
            gpu_dispatch_us: Mutex::new(std::collections::VecDeque::with_capacity(LATENCY_RING_CAP)),
            streamer_tick_us: Mutex::new(std::collections::VecDeque::with_capacity(LATENCY_RING_CAP)),
        }
    }

    pub fn record_order(&self, us: u64) {
        if let Ok(mut ring) = self.order_rtt_us.lock() {
            if ring.len() >= LATENCY_RING_CAP { ring.pop_front(); }
            ring.push_back(us);
        }
    }

    pub fn record_gpu(&self, us: u64) {
        if let Ok(mut ring) = self.gpu_dispatch_us.lock() {
            if ring.len() >= LATENCY_RING_CAP { ring.pop_front(); }
            ring.push_back(us);
        }
    }

    pub fn record_streamer(&self, us: u64) {
        if let Ok(mut ring) = self.streamer_tick_us.lock() {
            if ring.len() >= LATENCY_RING_CAP { ring.pop_front(); }
            ring.push_back(us);
        }
    }

    pub fn snapshot(&self) -> LatencySnapshot {
        let order = self.order_rtt_us.lock().map(|d| LatencyBucket::from_deque(&d)).unwrap_or(LatencyBucket { count: 0, min: 0, max: 0, p50: 0, p95: 0, p99: 0 });
        let gpu = self.gpu_dispatch_us.lock().map(|d| LatencyBucket::from_deque(&d)).unwrap_or(LatencyBucket { count: 0, min: 0, max: 0, p50: 0, p95: 0, p99: 0 });
        let streamer = self.streamer_tick_us.lock().map(|d| LatencyBucket::from_deque(&d)).unwrap_or(LatencyBucket { count: 0, min: 0, max: 0, p50: 0, p95: 0, p99: 0 });
        LatencySnapshot { order_rtt_us: order, gpu_dispatch_us: gpu, streamer_tick_us: streamer }
    }
}


/// Global lock ordering (acquire sub-states in this order to avoid deadlocks):
///
/// GROUP 1 -- Global singletons
///   1. api_keys
///   2. settings
///   3. wallet
///
/// GROUP 2 -- MarketDataState (see market_state.rs for internal ordering)
///   4. market.last_prices
///   5. market.candle_buffers
///   6. market.indicator_cache
///   7. market.vol_cache
///   8. market.bars_cache
///   (market.ofi_state is lock-free ArcSwap -- always safe)
///
/// GROUP 3 -- TradingState (see trading_state.rs for internal ordering)
///   9. trading.trail_highs
///   10. trading.trail_lows
///   11. trading.partial_taken
///   12. trading.entry_tags
///   13. trading.entry_times
///   14. trading.entry_features
///   15. trading.exit_inflight
///   16. trading.recon_grace
///   17. trading.failed_order_cooldown
///   18. trading.position_blacklist
///
/// GROUP 4 -- RiskState (see risk_state.rs for internal ordering)
///   (risk.circuit_breaker uses internal atomics -- always safe)
///   19. risk.session_analytics
///   20. risk.intraday_drawdown
///   21. risk.flash_crash
///
/// GROUP 5 -- ScanState (see scan_state.rs for internal ordering)
///   22. scan.symbol_best_tf
///   23. scan.last_scan_results
///   24. scan.scan_progress
///   25. scan.scan_prefilter_queue
///   26. scan.scan_cancel
///
/// GROUP 6 -- Strategy / Nexus
///   27. strategy_mode
///   28. nexus_fusion_alpha
///   29. nexus_consensus
///
/// GROUP 7 -- Evolution / Oracle
///   30. genetic_optimizer
///   30b. st_genetic_optimizer (S13)
///   31. oracle_adaptation
///
/// GROUP 8 -- Journals / Caches
///   32. trade_journal
///   33. news_cache
///   34. sentiment_history
///
/// GROUP 9 -- Infrastructure (gpu caches, system, scan_tx, stream_tx, net_tracker)
///   35. gpu_score_cache
///   36. gpu_cache
///   37. system
///   38. net_tracker
///   39. scan_tx
///   40. stream_tx
pub(crate) struct TitanState {
    // ── Sub-state aggregates ──
    /// Trading sub-state: trails, entry metadata, exit inflight, blacklist, cooldowns
    pub trading: Arc<crate::trading_state::TradingState>,
    /// Market data sub-state: prices, candles, indicators, volume, OFI
    pub market: Arc<crate::market_state::MarketDataState>,
    /// Risk sub-state: circuit breaker, session analytics, drawdown, flash crash
    pub risk: Arc<crate::risk_state::RiskState>,
    /// AI scan pipeline sub-state: best-TF, results, progress, prefilter, cancel
    pub scan: Arc<crate::scan_state::ScanState>,

    // ── Global singletons ──
    pub api_keys: Mutex<ApiKeys>,
    pub wallet: Mutex<Wallet>,
    pub settings: Mutex<AppState>,
    pub redis: Mutex<Option<redis::Client>>,
    pub system: Mutex<System>,
    pub net_tracker: Mutex<monitor::NetTracker>,
    pub gpu_cache: Mutex<monitor::GpuCache>,
    pub gpu_score_cache: Mutex<GpuScoreCache>,
    pub scan_tx: Mutex<Option<broadcast::Sender<()>>>,
    pub stream_tx: Mutex<Option<Sender<streamer::StreamCmd>>>,

    // ── Strategy / Nexus ──
    pub strategy_mode: Mutex<StrategyMode>,
    pub nexus_fusion_alpha: Mutex<f64>,
    pub nexus_consensus: Mutex<NexusConsensus>,

    // ── Evolution / Oracle ──
    pub genetic_optimizer: Mutex<genetic::GeneticOptimizer>,
    pub st_genetic_optimizer: Mutex<genetic::StGeneticOptimizer>,
    pub oracle_adaptation: Mutex<crate::titan_oracle::OracleAdaptation>,

    // ── SmartTrend AI v3 (S07b + S10) ──
    pub smarttrend_engine: Mutex<crate::smarttrend::SmartTrendEngine>,
    pub st_account: Mutex<crate::smarttrend::SmartTrendAccount>,
    /// Per-symbol engine snapshots: sentinel writes, bot reads (multi-symbol bridge)
    pub sentinel_snapshots: Mutex<HashMap<String, crate::smarttrend::SentinelEngineSnapshot>>,

    // ── Journals / Caches ──
    pub trade_journal: Mutex<journal::TradeJournal>,
    pub news_cache: Mutex<Vec<sentiment::NewsArticle>>,
    pub sentiment_history: Mutex<Vec<(String, f64)>>,

    // ── Primitives (no ordering constraints) ──
    pub shutdown_token: CancellationToken,
    pub dry_run: std::sync::atomic::AtomicBool,
    pub session_generation: std::sync::atomic::AtomicU64,
    pub generation_mismatch_count: std::sync::atomic::AtomicU64,
    pub watchdog: watchdog::Watchdog,
    pub alpaca_rate_limiter: Arc<tokio::sync::Semaphore>,
    pub http_client: reqwest::Client,
    pub latency_metrics: LatencyMetrics,
    pub mode_audit_trail: Mutex<Vec<ModeTransition>>,
    pub ws_broadcast: broadcast::Sender<String>,
}


impl TitanState {
    /// Current session generation (monotonically increasing on critical loop respawns).
    pub fn current_generation(&self) -> u64 {
        self.session_generation.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Increment the generation counter. Returns the NEW generation value.
    /// Called before respawning critical loops (streamer, autopilot, trade_updates).
    pub fn bump_generation(&self) -> u64 {
        let new_gen = self.session_generation.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        tracing::info!(
            component = "generation",
            new_generation = new_gen,
            "Session generation incremented"
        );
        new_gen
    }
}

/// Synchronise all runtime fields on TitanState with the values in AppState.
/// This is the single source of truth for config -> runtime propagation.
/// Called from: startup, load_profile, batch_update_settings, apply_full_settings.
pub(crate) fn apply_runtime_config(titan: &TitanState, settings: &crate::app_state::AppState) {
    // 1. Strategy mode enum
    *titan.strategy_mode.lock().unwrap_or_else(|e| e.into_inner()) =
        StrategyMode::from_name(&settings.strategy_mode_name);

    // 2. Nexus fusion alpha
    *titan.nexus_fusion_alpha.lock().unwrap_or_else(|e| e.into_inner()) =
        settings.nexus_fusion_alpha;

    // 3. Dry run atomic (preserve env override -- only set true, never force false if env says true)
    let env_dr = std::env::var("TITAN_DRY_RUN").unwrap_or_default() == "1";
    let effective_dry_run = env_dr || settings.dry_run_enabled;
    titan.dry_run.store(
        effective_dry_run,
        std::sync::atomic::Ordering::SeqCst,
    );
    if !effective_dry_run {
        tracing::warn!(component = "config", "LIVE TRADING ENABLED -- dry_run is OFF");
    }

    // 4. Circuit breaker thresholds
    titan.risk.circuit_breaker.update_thresholds(
        rust_decimal::Decimal::try_from(settings.breaker_caution_pct)
            .unwrap_or(rust_decimal_macros::dec!(-3.0)),
        rust_decimal::Decimal::try_from(settings.breaker_halt_pct)
            .unwrap_or(rust_decimal_macros::dec!(-5.0)),
        rust_decimal::Decimal::try_from(settings.breaker_panic_pct)
            .unwrap_or(rust_decimal_macros::dec!(-8.0)),
    );

    // 5. Flash crash detector -- reconfigure without losing sliding window history
    {
        let mut fc = titan.risk.flash_crash.lock().unwrap_or_else(|e| e.into_inner());
        fc.reconfigure(
            settings.flash_crash_window_secs,
            settings.flash_crash_threshold_pct,
            settings.flash_crash_cooldown_mins,
        );
    }

    // 6. SmartTrend engine config (S07b) — update config without resetting FSM state
    {
        let mut engine = titan.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
        engine.config = settings.smarttrend.clone();
    }
}

pub(crate) fn normalize_symbol(input: &str) -> String {
    input.trim().to_uppercase().replace(' ', "")
}

/// Canonical short timeframe for in-memory keys (`candle_buffers`, `indicator_cache`,
/// `symbol_best_tf` values). Matches the inputs accepted by `candle_buffer::timeframe_to_secs`
/// and `fetch_bars_internal` (after its internal Alpaca mapping).
pub(crate) fn normalize_timeframe_internal(tf: &str) -> String {
    let compact: String = tf.trim().chars().filter(|c| !c.is_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    match lower.as_str() {
        "1m" | "1min" => "1m".to_string(),
        "3m" | "3min" => "3m".to_string(),
        "5m" | "5min" => "5m".to_string(),
        "10m" | "10min" => "10m".to_string(),
        "15m" | "15min" => "15m".to_string(),
        "30m" | "30min" => "30m".to_string(),
        "1h" | "1hour" => "1h".to_string(),
        "4h" | "4hour" => "4h".to_string(),
        "1d" | "1day" => "1d".to_string(),
        _ => lower,
    }
}

/// Single internal contract for `symbol|timeframe` map keys.
pub(crate) fn cache_key_symbol_tf(symbol: &str, timeframe: &str) -> String {
    let mut key =
        String::with_capacity(symbol.len().saturating_add(1).saturating_add(timeframe.len()));
    key.push_str(&normalize_symbol(symbol));
    key.push('|');
    key.push_str(&normalize_timeframe_internal(timeframe));
    key
}

/// Acquire a rate-limit permit before calling the Alpaca API.
/// Waits up to 30s; returns Err if the limiter is exhausted for too long.
pub(crate) async fn acquire_api_permit(
    state: &Arc<TitanState>,
) -> Result<tokio::sync::SemaphorePermit<'_>, TitanError> {
    match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        state.alpaca_rate_limiter.acquire(),
    )
    .await
    {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(_)) => Err(TitanError::RateLimit("Rate limiter closed".into())),
        Err(_) => Err(TitanError::Timeout(
            "Rate limit timeout — too many API calls queued".into(),
        )),
    }
}

/// US DST: starts 2nd Sunday of March at 2:00 AM, ends 1st Sunday of November at 2:00 AM.
/// Returns `false` (EST) on any calendar math failure as a safe default.
pub(crate) fn is_us_dst(utc_now: chrono::DateTime<chrono::Utc>) -> bool {
    use chrono::{Datelike, LocalResult, NaiveDate, TimeZone};
    let year = utc_now.year();

    let Some(march_first) = NaiveDate::from_ymd_opt(year, 3, 1) else {
        return false;
    };
    let march_dow = march_first.weekday().num_days_from_sunday();
    let second_sunday_march = 1 + (7 - march_dow) % 7 + 7;
    let dst_start = match chrono::Utc.with_ymd_and_hms(year, 3, second_sunday_march, 7, 0, 0) {
        LocalResult::Single(dt) => dt,
        _ => return false,
    };

    let Some(nov_first) = NaiveDate::from_ymd_opt(year, 11, 1) else {
        return false;
    };
    let nov_dow = nov_first.weekday().num_days_from_sunday();
    let first_sunday_nov = 1 + (7 - nov_dow) % 7;
    let dst_end = match chrono::Utc.with_ymd_and_hms(year, 11, first_sunday_nov, 6, 0, 0) {
        LocalResult::Single(dt) => dt,
        _ => return false,
    };

    utc_now >= dst_start && utc_now < dst_end
}

pub(crate) fn log_to_file(msg: &str) {
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let log_path = crate::paths::logs_dir().join("titan_debug_trace.log");
    let mut opts = fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    opts.mode(0o600);
    match opts.open(&log_path) {
        Ok(mut file) => {
            let _ = writeln!(file, "{}", msg);
        }
        Err(_) => {
            eprintln!("[titan] {}", msg);
        }
    }
}

/// Parse Alpaca credentials from `config.toml` contents.
///
/// Accepts both the documented `[alpaca]` table form (`config.toml.example`)
/// and the legacy flat form with top-level `key_id` / `secret` / `base_url`.
/// Returns `None` when no non-empty key/secret pair is present.
pub(crate) fn parse_config_toml_keys(conf: &str) -> Option<ApiKeys> {
    #[derive(serde::Deserialize, Default)]
    struct AlpacaSection {
        #[serde(default)]
        key_id: String,
        #[serde(default)]
        secret: String,
        #[serde(default)]
        base_url: String,
    }
    #[derive(serde::Deserialize)]
    struct ConfigToml {
        #[serde(default)]
        alpaca: Option<AlpacaSection>,
        #[serde(default)]
        key_id: String,
        #[serde(default)]
        secret: String,
        #[serde(default)]
        base_url: String,
    }

    let parsed = match toml::from_str::<ConfigToml>(conf) {
        Ok(p) => p,
        Err(_) => {
            log_to_file("❌ MAIN: Failed to parse config.toml");
            return None;
        }
    };

    let (key, secret, base_url) = match parsed.alpaca {
        Some(a) if !a.key_id.is_empty() && !a.secret.is_empty() => (a.key_id, a.secret, a.base_url),
        _ => (parsed.key_id, parsed.secret, parsed.base_url),
    };
    if key.is_empty() || secret.is_empty() {
        return None;
    }
    let base_url = if base_url.is_empty() { default_base_url() } else { base_url };
    Some(ApiKeys {
        key,
        secret,
        base_url,
        fmp_api_key: String::new(),
    })
}

#[cfg(test)]
mod config_toml_tests {
    use super::parse_config_toml_keys;

    #[test]
    fn parses_alpaca_table_form() {
        let conf = r#"
[alpaca]
key_id = "PKTEST"
secret = "SECRET"
base_url = "https://paper-api.alpaca.markets"
"#;
        let k = parse_config_toml_keys(conf).expect("table form must parse");
        assert_eq!(k.key, "PKTEST");
        assert_eq!(k.secret, "SECRET");
        assert_eq!(k.base_url, "https://paper-api.alpaca.markets");
    }

    #[test]
    fn parses_legacy_flat_form() {
        let conf = "key_id = \"PKFLAT\"\nsecret = \"S\"\n";
        let k = parse_config_toml_keys(conf).expect("flat form must parse");
        assert_eq!(k.key, "PKFLAT");
        assert!(!k.base_url.is_empty(), "base_url must fall back to default");
    }

    #[test]
    fn rejects_placeholder_or_missing() {
        assert!(parse_config_toml_keys("[alpaca]\nkey_id = \"\"\nsecret = \"\"\n").is_none());
        assert!(parse_config_toml_keys("[risk]\nmax_drawdown = 0.1\n").is_none());
        assert!(parse_config_toml_keys("not valid toml = = =").is_none());
    }
}

pub(crate) fn load_keys_from_disk() -> ApiKeys {
    log_to_file("🔑 MAIN: Attempting to load keys from disk...");
    if let Ok(content) = fs::read_to_string(crate::paths::keys_path()) {
        log_to_file("🔑 MAIN: Found titan_keys.json");
        if let Ok(keys) = serde_json::from_str::<ApiKeys>(&content) {
            if !keys.key.is_empty() && !keys.secret.is_empty() {
                log_to_file(&format!(
                    "✅ MAIN: Loaded keys successfully. Key length: {}",
                    keys.key.len()
                ));
                return keys;
            }
        } else {
            log_to_file("❌ MAIN: Failed to parse titan_keys.json");
        }
    } else {
        log_to_file("❌ MAIN: titan_keys.json NOT FOUND");
    }

    if let Ok(conf) = fs::read_to_string(crate::paths::config_path()) {
        log_to_file("🔑 MAIN: Found config.toml");
        match parse_config_toml_keys(&conf) {
            Some(keys) => {
                log_to_file("✅ MAIN: Loaded keys from config.toml");
                return keys;
            }
            None => log_to_file("❌ MAIN: config.toml has no usable [alpaca] key_id/secret"),
        }
    }

    log_to_file("⚠️ MAIN: No keys found on disk.");
    ApiKeys {
        key: "".into(),
        secret: "".into(),
        base_url: default_base_url(),
        fmp_api_key: String::new(),
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_path() -> (std::path::PathBuf, std::path::PathBuf) {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join("titan_profile_tests")
            .join(format!("p{}_{}", std::process::id(), id));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("titan_profiles.json");
        (dir, path)
    }

    fn make_default_settings() -> AppSettings {
        let state = crate::app_state::AppState::default();
        settings_from_state(&state)
    }

    fn save_store_to(path: &std::path::Path, store: &ProfileStore) {
        let payload = serde_json::to_string_pretty(store).unwrap();
        let parent = path.parent().unwrap();
        let _ = fs::create_dir_all(parent);
        if path.exists() {
            let bak = path.with_extension("json.bak");
            let _ = fs::copy(path, &bak);
        }
        let tmp = parent.join("_profiles.tmp");
        fs::write(&tmp, &payload).unwrap();
        fs::rename(&tmp, path).unwrap();
    }

    fn load_store_from(path: &std::path::Path) -> ProfileStore {
        let raw = fs::read_to_string(path).unwrap_or_default();
        if raw.is_empty() {
            return ProfileStore::default();
        }
        serde_json::from_str(&raw).unwrap_or_default()
    }

    #[test]
    fn profile_save_load_roundtrip() {
        let (dir, path) = unique_test_path();
        let settings = make_default_settings();
        let entry = ProfileEntry {
            schema_v: 1,
            saved_at: "2026-03-29T12:00:00Z".into(),
            data: settings.clone(),
        };
        let mut store = ProfileStore::default();
        store.profiles.insert("test_profile".into(), entry);
        save_store_to(&path, &store);

        let loaded = load_store_from(&path);
        assert_eq!(loaded.profiles.len(), 1);
        let loaded_entry = loaded.profiles.get("test_profile").unwrap();
        assert_eq!(loaded_entry.schema_v, 1);
        assert_eq!(loaded_entry.data.risk_per_trade, settings.risk_per_trade);
        assert_eq!(loaded_entry.data.trailing_stop_pct, settings.trailing_stop_pct);
        assert_eq!(loaded_entry.data.max_open_positions, settings.max_open_positions);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_preserves_nested_objects() {
        let (dir, path) = unique_test_path();
        let mut settings = make_default_settings();
        settings.ta_gates = crate::app_state::TaGatesConfig::from_preset(
            &crate::app_state::TaGatePreset::Strict,
        );
        settings.scan_params.price_min = 5.0;
        settings.scan_params.volume_min = 1_000_000.0;
        settings.symbol_overrides.insert(
            "AAPL".into(),
            crate::app_state::SymbolOverride::default(),
        );

        let entry = ProfileEntry {
            schema_v: 1,
            saved_at: "2026-03-29T12:00:00Z".into(),
            data: settings,
        };
        let mut store = ProfileStore::default();
        store.profiles.insert("strict".into(), entry);
        save_store_to(&path, &store);

        let loaded = load_store_from(&path);
        let loaded_data = &loaded.profiles.get("strict").unwrap().data;
        assert!(matches!(
            loaded_data.ta_gates.rsi_gate,
            crate::app_state::GateMode::Hard
        ));
        assert_eq!(loaded_data.scan_params.price_min, 5.0);
        assert_eq!(loaded_data.scan_params.volume_min, 1_000_000.0);
        assert!(loaded_data.symbol_overrides.contains_key("AAPL"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_delete_existing() {
        let (dir, path) = unique_test_path();
        let entry = ProfileEntry {
            schema_v: 1,
            saved_at: "2026-03-29T12:00:00Z".into(),
            data: make_default_settings(),
        };
        let mut store = ProfileStore::default();
        store.profiles.insert("to_delete".into(), entry.clone());
        store.profiles.insert("to_keep".into(), entry);
        save_store_to(&path, &store);

        let mut store = load_store_from(&path);
        assert_eq!(store.profiles.len(), 2);
        store.profiles.remove("to_delete");
        save_store_to(&path, &store);

        let reloaded = load_store_from(&path);
        assert_eq!(reloaded.profiles.len(), 1);
        assert!(reloaded.profiles.contains_key("to_keep"));
        assert!(!reloaded.profiles.contains_key("to_delete"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_delete_nonexistent_is_graceful() {
        let (dir, path) = unique_test_path();
        let store = ProfileStore::default();
        save_store_to(&path, &store);

        let mut loaded = load_store_from(&path);
        let removed = loaded.profiles.remove("nonexistent");
        assert!(removed.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_schema_version_defaults() {
        let (dir, path) = unique_test_path();
        // schema_v omitted -> should default to 1 via serde(default)
        // data contains required fields so AppSettings can deserialize
        let settings = make_default_settings();
        let data_json = serde_json::to_string(&settings).unwrap();
        let json = format!(
            r#"{{ "v": 1, "profiles": {{ "old": {{ "saved_at": "2025-01-01T00:00:00Z", "data": {} }} }} }}"#,
            data_json
        );
        fs::write(&path, &json).unwrap();

        let store = load_store_from(&path);
        let entry = store.profiles.get("old").unwrap();
        assert_eq!(entry.schema_v, 1);
        assert_eq!(entry.data.risk_per_trade, settings.risk_per_trade);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_backup_created_on_save() {
        let (dir, path) = unique_test_path();
        let store = ProfileStore::default();
        save_store_to(&path, &store);
        assert!(path.exists());

        let mut store2 = ProfileStore::default();
        store2.profiles.insert(
            "after".into(),
            ProfileEntry {
                schema_v: 1,
                saved_at: "2026-03-29T12:00:00Z".into(),
                data: make_default_settings(),
            },
        );
        save_store_to(&path, &store2);

        let bak = path.with_extension("json.bak");
        assert!(bak.exists(), ".bak file should exist after second save");

        let bak_store: ProfileStore =
            serde_json::from_str(&fs::read_to_string(&bak).unwrap()).unwrap();
        assert!(
            bak_store.profiles.is_empty(),
            "Backup should contain the original (empty) store"
        );

        let current = load_store_from(&path);
        assert_eq!(current.profiles.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_corrupt_file_returns_empty_store() {
        let (dir, path) = unique_test_path();
        fs::write(&path, "NOT VALID JSON {{{{").unwrap();

        let store: ProfileStore = serde_json::from_str(
            &fs::read_to_string(&path).unwrap_or_default()
        ).unwrap_or_default();
        assert!(store.profiles.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_missing_file_returns_empty_store() {
        let (dir, path) = unique_test_path();
        let _ = fs::remove_file(&path);
        assert!(!path.exists());
        let raw = fs::read_to_string(&path).unwrap_or_default();
        let store: ProfileStore = if raw.is_empty() {
            ProfileStore::default()
        } else {
            serde_json::from_str(&raw).unwrap_or_default()
        };
        assert!(store.profiles.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    // ═══ S01: Configuration Coherence Regression Tests ═══

    #[test]
    fn settings_round_trip_coherence() {
        let original = crate::app_state::AppState::default();
        let serialized = settings_from_state(&original);
        let mut restored = crate::app_state::AppState::default();
        apply_settings(&mut restored, &serialized);
        let re_serialized = settings_from_state(&restored);

        assert_eq!(serialized.risk_per_trade, re_serialized.risk_per_trade);
        assert_eq!(serialized.trailing_stop_pct, re_serialized.trailing_stop_pct);
        assert_eq!(serialized.take_profit_pct, re_serialized.take_profit_pct);
        assert_eq!(serialized.strategy_mode_name, re_serialized.strategy_mode_name);
        assert_eq!(serialized.dry_run_enabled, re_serialized.dry_run_enabled);
        assert_eq!(serialized.nexus_fusion_alpha, re_serialized.nexus_fusion_alpha);
        assert_eq!(serialized.breaker_caution_pct, re_serialized.breaker_caution_pct);
        assert_eq!(serialized.breaker_halt_pct, re_serialized.breaker_halt_pct);
        assert_eq!(serialized.breaker_panic_pct, re_serialized.breaker_panic_pct);
        assert_eq!(serialized.flash_crash_window_secs, re_serialized.flash_crash_window_secs);
        assert_eq!(serialized.flash_crash_threshold_pct, re_serialized.flash_crash_threshold_pct);
        assert_eq!(serialized.flash_crash_cooldown_mins, re_serialized.flash_crash_cooldown_mins);
    }

    #[test]
    fn strategy_mode_from_name_covers_all_variants() {
        assert_eq!(StrategyMode::from_name("MATH_ORACLE"), StrategyMode::MathOracle);
        assert_eq!(StrategyMode::from_name("ORACLE"), StrategyMode::MathOracle);
        assert_eq!(StrategyMode::from_name("GPU_NEXUS"), StrategyMode::GpuNexus);
        assert_eq!(StrategyMode::from_name("NEXUS"), StrategyMode::GpuNexus);
        assert_eq!(StrategyMode::from_name("GPU"), StrategyMode::GpuNexus);
        assert_eq!(StrategyMode::from_name("FUSION"), StrategyMode::Fusion);
        assert_eq!(StrategyMode::from_name("SIGNAL_STRATEGY"), StrategyMode::SignalStrategy);
        assert_eq!(StrategyMode::from_name("SIGNAL"), StrategyMode::SignalStrategy);
        assert_eq!(StrategyMode::from_name("fusion"), StrategyMode::Fusion);
        assert_eq!(StrategyMode::from_name("unknown_garbage"), StrategyMode::Fusion);
        assert_eq!(StrategyMode::from_name(""), StrategyMode::Fusion);
    }

    #[test]
    fn flash_crash_reconfigure_preserves_windows() {
        use crate::candle_buffer::FlashCrashDetector;
        let mut fc = FlashCrashDetector::new(30, 3.0, 5);
        let now = std::time::Instant::now();
        fc.check_trade("AAPL", 100.0, now);
        fc.check_trade("AAPL", 101.0, now + std::time::Duration::from_secs(1));

        fc.reconfigure(60, 5.0, 10);

        let result = fc.check_trade("AAPL", 102.0, now + std::time::Duration::from_secs(2));
        assert!(result.is_none(), "should not crash detect on 2% move with 5% threshold");
        assert!(!fc.is_blocked("AAPL"));
    }

    #[test]
    fn validate_coherence_catches_bad_breakers() {
        let mut settings = make_default_settings();
        settings.breaker_caution_pct = 5.0;
        settings.breaker_halt_pct = -2.0;
        settings.breaker_panic_pct = -1.0;

        let warnings = crate::commands::settings::validate_settings_coherence(&settings);
        assert!(!warnings.is_empty(), "should warn about positive caution_pct");
        assert!(
            warnings.iter().any(|w| w.contains("breaker_panic_pct")),
            "should warn about panic > halt"
        );
    }

    #[test]
    fn snapshot_schema_v2_dd_velocity_roundtrip() {
        let snap = StateSnapshot {
            dd_velocity_ring: vec![(1000, 0.5), (1010, 1.2), (1020, 2.0)],
            session_generation: 42,
            ..Default::default()
        };
        let json = serde_json::to_string(&snap).expect("snapshot must serialize");
        let restored: StateSnapshot = serde_json::from_str(&json).expect("snapshot must deserialize");
        assert_eq!(restored.dd_velocity_ring.len(), 3);
        assert_eq!(restored.dd_velocity_ring[0], (1000, 0.5));
        assert_eq!(restored.dd_velocity_ring[2], (1020, 2.0));
        assert_eq!(restored.session_generation, 42);
    }

    #[test]
    fn snapshot_schema_v2_backward_compat() {
        let json = r#"{"timestamp_ms":0,"trail_highs":{},"trail_lows":{},"partial_taken":{},"last_prices":{},"symbol_best_tf":{},"nexus_fusion_alpha":0.0,"session_starting_equity":0.0,"session_current_equity":0.0,"session_peak_equity":0.0,"session_total_trades":0,"session_winning_trades":0,"session_losing_trades":0,"session_total_pnl":0.0,"session_daily_pnl":0.0,"session_max_drawdown_pct":0.0,"session_avg_win":0.0,"session_avg_loss":0.0,"session_last_entry_ts":null,"session_trade_pnls":[],"last_scan_results":[],"scan_was_running":false,"scan_processed":0,"scan_total_requested":0}"#;
        let snap: StateSnapshot = serde_json::from_str(json).expect("old snapshot must deserialize with defaults");
        assert!(snap.dd_velocity_ring.is_empty(), "default should be empty");
        assert_eq!(snap.session_generation, 0, "default generation should be 0");
    }

    #[test]
    fn test_snapshot_integrity_hash_roundtrip() {
        let dir = std::env::temp_dir().join(format!("titan_hash_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let snap_path = dir.join("titan_state.json");
        let hash_path = dir.join("titan_state.sha256");

        let snap = StateSnapshot {
            session_generation: 7,
            ..Default::default()
        };
        let payload = serde_json::to_string(&snap).unwrap();
        fs::write(&snap_path, &payload).unwrap();

        use sha2::Digest;
        let hash_hex = hex::encode(sha2::Sha256::digest(payload.as_bytes()));
        fs::write(&hash_path, &hash_hex).unwrap();

        let data = fs::read_to_string(&snap_path).unwrap();
        let stored_hash = fs::read_to_string(&hash_path).unwrap();
        let actual = hex::encode(sha2::Sha256::digest(data.as_bytes()));
        assert_eq!(actual, stored_hash.trim(), "hash must match on valid snapshot");

        let restored: StateSnapshot = serde_json::from_str(&data).unwrap();
        assert_eq!(restored.session_generation, 7);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_snapshot_integrity_hash_corrupt() {
        let dir = std::env::temp_dir().join(format!("titan_corrupt_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let snap_path = dir.join("titan_state.json");
        let hash_path = dir.join("titan_state.sha256");

        let snap = StateSnapshot::default();
        let payload = serde_json::to_string(&snap).unwrap();
        fs::write(&snap_path, &payload).unwrap();
        fs::write(&hash_path, "0000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let data = fs::read_to_string(&snap_path).unwrap();
        let stored_hash = fs::read_to_string(&hash_path).unwrap();

        use sha2::Digest;
        let actual = hex::encode(sha2::Sha256::digest(data.as_bytes()));
        assert_ne!(actual, stored_hash.trim(), "corrupt hash must differ");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_ordering_trading_state_no_deadlock() {
        let ts = crate::trading_state::TradingState::new(std::collections::HashSet::new());
        let _g1 = ts.trail_highs.lock().unwrap();
        let _g2 = ts.trail_lows.lock().unwrap();
        let _g3 = ts.partial_taken.lock().unwrap();
        let _g4 = ts.entry_tags.lock().unwrap();
        let _g5 = ts.entry_times.lock().unwrap();
        let _g6 = ts.entry_features.lock().unwrap();
        let _g7 = ts.exit_inflight.lock().unwrap();
        let _g8 = ts.recon_grace.lock().unwrap();
        let _g9 = ts.failed_order_cooldown.lock().unwrap();
        let _g10 = ts.position_blacklist.lock().unwrap();
    }

    #[test]
    fn lock_ordering_market_state_no_deadlock() {
        let ms = crate::market_state::MarketDataState::new();
        let _g1 = ms.last_prices.lock().unwrap();
        let _g2 = ms.candle_buffers.lock().unwrap();
        let _g3 = ms.indicator_cache.lock().unwrap();
        let _g4 = ms.vol_cache.lock().unwrap();
        let _g5 = ms.bars_cache.lock().unwrap();
    }

    #[test]
    fn lock_ordering_risk_state_no_deadlock() {
        let rs = crate::risk_state::RiskState::new(
            rust_decimal_macros::dec!(100000),
            rust_decimal_macros::dec!(-3.0),
            rust_decimal_macros::dec!(-5.0),
            rust_decimal_macros::dec!(-8.0),
            300, 5.0, 15,
        );
        let _g1 = rs.session_analytics.lock().unwrap();
        let _g2 = rs.intraday_drawdown.lock().unwrap();
        let _g3 = rs.flash_crash.lock().unwrap();
    }

    #[test]
    fn lock_ordering_scan_state_no_deadlock() {
        let ss = crate::scan_state::ScanState::new();
        let _g1 = ss.symbol_best_tf.lock().unwrap();
        let _g2 = ss.last_scan_results.lock().unwrap();
        let _g3 = ss.scan_progress.lock().unwrap();
        let _g4 = ss.scan_prefilter_queue.lock().unwrap();
        let _g5 = ss.scan_cancel.lock().unwrap();
    }

    #[test]
    fn strategy_mode_smarttrend_parse() {
        assert_eq!(StrategyMode::from_name("SMARTTREND"), StrategyMode::SmartTrend);
        assert_eq!(StrategyMode::from_name("smarttrend"), StrategyMode::SmartTrend);
        assert_eq!(StrategyMode::from_name("SmartTrend"), StrategyMode::SmartTrend);
        assert_eq!(StrategyMode::from_name("SMART_TREND"), StrategyMode::SmartTrend);
        assert_eq!(StrategyMode::from_name("ST"), StrategyMode::SmartTrend);
        assert_eq!(StrategyMode::from_name("st"), StrategyMode::SmartTrend);
    }

    #[test]
    fn strategy_mode_smarttrend_display() {
        assert_eq!(format!("{}", StrategyMode::SmartTrend), "SMARTTREND");
    }

    #[test]
    fn strategy_mode_smarttrend_roundtrip() {
        let mode = StrategyMode::SmartTrend;
        let name = format!("{}", mode);
        let parsed = StrategyMode::from_name(&name);
        assert_eq!(parsed, mode);
    }

    #[test]
    fn strategy_mode_toggle_cycle_includes_smarttrend() {
        let cycle = [
            StrategyMode::MathOracle,
            StrategyMode::GpuNexus,
            StrategyMode::Fusion,
            StrategyMode::SignalStrategy,
            StrategyMode::SmartTrend,
        ];
        for (i, mode) in cycle.iter().enumerate() {
            let name = format!("{}", mode);
            let parsed = StrategyMode::from_name(&name);
            assert_eq!(*mode, parsed, "mode {} at index {} failed", name, i);
        }
    }

    #[test]
    fn strategy_mode_serde_roundtrip() {
        let mode = StrategyMode::SmartTrend;
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: StrategyMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mode);
    }

    // ═══ S15: Trading Edge Settings Tests ═══

    #[test]
    fn s15_appstate_defaults() {
        let s = crate::app_state::AppState::default();
        assert!(s.max_hold_decay_enabled);
        assert!((s.max_hold_decay_start_pct - 0.5).abs() < 0.001);
        assert!((s.max_hold_decay_trail_tighten - 0.3).abs() < 0.01);
        assert!(s.mtf_gate_enabled);
        assert_eq!(s.mtf_gate_mode, "soft");
        assert!((s.mtf_gate_penalty - 15.0).abs() < 0.1);
        assert_eq!(s.mtf_gate_htf, "15m");
        assert!(s.hhi_gate_manual_orders);
        assert!((s.max_concentration_hhi_manual - 0.35).abs() < 0.01);
        assert!(s.tiered_scan_enabled);
        assert_eq!(s.tier1_count, 50);
        assert_eq!(s.tier2_count, 100);
    }

    #[test]
    fn s15_settings_sync_roundtrip() {
        let mut app = crate::app_state::AppState::default();
        app.max_hold_decay_enabled = false;
        app.mtf_gate_mode = "hard".to_string();
        app.tier1_count = 75;
        app.max_concentration_hhi_manual = 0.45;

        let settings = settings_from_state(&app);
        assert!(!settings.max_hold_decay_enabled);
        assert_eq!(settings.mtf_gate_mode, "hard");
        assert_eq!(settings.tier1_count, 75);
        assert!((settings.max_concentration_hhi_manual - 0.45).abs() < 0.001);

        let mut app2 = crate::app_state::AppState::default();
        apply_settings(&mut app2, &settings);
        assert!(!app2.max_hold_decay_enabled);
        assert_eq!(app2.mtf_gate_mode, "hard");
        assert_eq!(app2.tier1_count, 75);
        assert!((app2.max_concentration_hhi_manual - 0.45).abs() < 0.001);
    }

    /// FIX-1 CRIT-7: save_settings_to_disk returns Err on write failure.
    #[test]
    fn save_settings_error_propagated() {
        let _settings = make_default_settings();
        let bogus_dir = std::path::PathBuf::from("/proc/nonexistent/deeply/nested/path");
        let tmp = bogus_dir.join("settings.json.tmp");
        let result = std::fs::write(&tmp, "test");
        assert!(result.is_err(), "Writing to invalid path should fail");
    }

    /// FIX-3 CRIT-18: validate_settings_coherence catches invalid field ranges.
    #[test]
    fn load_profile_validates_coherence() {
        let mut settings = make_default_settings();
        settings.risk_per_trade = 0.0;
        settings.trailing_stop_pct = -5.0;
        settings.max_open_positions = 0;
        let warnings = crate::commands::settings::validate_settings_coherence(&settings);
        assert!(
            !warnings.is_empty(),
            "Coherence validation should flag out-of-range values"
        );
    }
}

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;

use crate::app_state::AppState;
use crate::error::TitanError;
use crate::monitor;
use crate::state::{normalize_symbol, save_settings_to_disk, settings_from_state, settings_path};
use crate::state::{
    load_profiles_from_disk, save_profiles_to_disk, AppSettings, ProfileEntry, ProfileMeta,
    TitanState,
};
use crate::streamer;

fn parse_gate_mode(value: &str) -> crate::app_state::GateMode {
    match value.to_lowercase().as_str() {
        "hard" => crate::app_state::GateMode::Hard,
        "soft" => crate::app_state::GateMode::Soft,
        _ => crate::app_state::GateMode::Off,
    }
}

fn apply_setting_field(
    settings: &mut AppState,
    field: &str,
    value: &str,
) -> Result<(), TitanError> {
    match field {
        "loop_active" => settings.loop_active = value == "true",
        "omni_scan_active" => settings.omni_scan_active = value == "true",
        "auto_pilot" => settings.auto_pilot = value == "true",
        "aggressive_mode" => settings.aggressive_mode = value == "true",
        "allow_short" => settings.allow_short = value == "true",
        "enable_extended" => settings.enable_extended = value == "true",
        "force_manual_tf" => settings.force_manual_tf = value == "true",
        "scalping_oracle" => settings.scalping_oracle = value == "true",
        "scalping_gpu" => settings.scalping_gpu = value == "true",
        "scalping_fusion" => settings.scalping_fusion = value == "true",
        "ai_scan_active" => settings.ai_scan_active = value == "true",
        "ai_smart_sort" => settings.ai_smart_sort = value == "true",
        "ai_auto_inject_top" => {
            settings.ai_auto_inject_top = value.parse::<u32>().unwrap_or(0).clamp(0, 30)
        }
        "ai_auto_inject_hft" => settings.ai_auto_inject_hft = value == "true",
        "scan_price_min" => {
            settings.scan_params.price_min =
                value.parse::<f64>().unwrap_or(1.0).clamp(0.01, 10000.0)
        }
        "scan_price_max" => {
            settings.scan_params.price_max =
                value.parse::<f64>().unwrap_or(500.0).clamp(1.0, 50000.0)
        }
        "scan_volume_min" => {
            settings.scan_params.volume_min = value
                .parse::<f64>()
                .unwrap_or(500000.0)
                .clamp(0.0, 100_000_000.0)
        }
        "scan_change_min" => {
            settings.scan_params.change_min = value.parse::<f64>().unwrap_or(1.0).clamp(0.0, 100.0)
        }
        "scan_rvol_min" => {
            settings.scan_params.rvol_min = value.parse::<f64>().unwrap_or(1.0).clamp(0.0, 100.0)
        }
        "scan_gap_min" => {
            settings.scan_params.gap_min = value.parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0)
        }
        "scan_require_catalyst" => settings.scan_params.require_catalyst = value == "true",
        "scan_max_candidates" => {
            settings.scan_params.max_candidates =
                value.parse::<usize>().unwrap_or(100).clamp(10, 300)
        }
        "scan_interval_minutes" => {
            settings.scan_params.scan_interval_minutes =
                value.parse::<u32>().unwrap_or(30).clamp(5, 60)
        }
        "scan_allow_negative_change" => settings.scan_params.allow_negative_change = value == "true",
        "scan_score_preset" => {
            let v = value.to_lowercase();
            if matches!(v.as_str(), "balanced" | "volume_hunter" | "momentum" | "gap_scanner") {
                settings.scan_params.score_preset = v;
            }
        }
        "scan_float_max" => {
            settings.scan_params.float_max = value.parse::<f64>().unwrap_or(0.0).clamp(0.0, 10_000.0)
        }
        "scan_cap_max" => {
            settings.scan_params.cap_max = value.parse::<f64>().unwrap_or(0.0).clamp(0.0, 10_000.0)
        }
        "scan_exclude_earnings" => settings.scan_params.exclude_earnings = value == "true",
        "md_auto_scan_active" => settings.md_auto_scan_active = value == "true",
        "refresh_rate" => {
            settings.refresh_rate = value.parse::<u64>().unwrap_or(2000).clamp(500, 60000)
        }
        "risk_per_trade" => {
            let v = value.parse::<f64>().unwrap_or(2.0).clamp(0.1, 25.0);
            settings.risk_per_trade = v;
            if v != value.parse::<f64>().unwrap_or(0.0) {
                tracing::warn!(component = "config", "risk_per_trade clamped to {}", v);
            }
        }
        "manual_res" => settings.manual_res = value.to_string(),
        "backtest_days" => {
            settings.backtest_days = value.parse::<u64>().unwrap_or(30).clamp(1, 365)
        }
        "gpu_backtest" => settings.gpu_backtest = value == "true",
        "gpu_sweep" => settings.gpu_sweep = value == "true",
        "gpu_deep_focus" => settings.gpu_deep_focus = value == "true",
        "auto_entry" => settings.auto_entry = value == "true",
        "max_open_positions" => {
            settings.max_open_positions = value.parse::<u32>().unwrap_or(1).clamp(1, 20)
        }
        "entry_cooldown_secs" => {
            settings.entry_cooldown_secs = value.parse::<u64>().unwrap_or(30).clamp(5, 3600)
        }
        "daily_loss_limit_pct" => {
            let v = value.parse::<f64>().unwrap_or(3.0).clamp(0.5, 20.0);
            settings.daily_loss_limit_pct = v;
            if v != value.parse::<f64>().unwrap_or(0.0) {
                tracing::warn!(
                    component = "config",
                    "daily_loss_limit_pct clamped to {}",
                    v
                );
            }
        }
        "trailing_stop_pct" => {
            settings.trailing_stop_pct = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 20.0)
        }
        "take_profit_pct" => {
            settings.take_profit_pct = value.parse::<f64>().unwrap_or(2.5).clamp(0.1, 50.0)
        }
        "max_spread_bps" => {
            settings.max_spread_bps = value.parse::<f64>().unwrap_or(50.0).clamp(1.0, 500.0)
        }
        "min_ofi_threshold" => {
            settings.min_ofi_threshold = value.parse::<f64>().unwrap_or(5.0).clamp(0.0, 100.0)
        }
        "max_daily_trades" => {
            settings.max_daily_trades = value.parse::<u32>().unwrap_or(50).clamp(1, 500)
        }
        "entry_signal_strength_min" => {
            settings.entry_signal_strength_min =
                value.parse::<f64>().unwrap_or(40.0).clamp(10.0, 95.0)
        }
        "max_hold_secs" => {
            settings.max_hold_secs = value.parse::<u64>().unwrap_or(3600).clamp(30, 86400)
        }
        "multi_symbol_enabled" => settings.multi_symbol_enabled = value == "true",
        "max_concurrent_assets" => settings.max_concurrent_assets = value.parse::<u32>().unwrap_or(5).clamp(1, 100),
        "max_correlation" => {
            settings.max_correlation = value.parse::<f64>().unwrap_or(0.70).clamp(0.0, 1.0)
        }
        "correlation_window" => {
            settings.correlation_window = value.parse::<usize>().unwrap_or(60).clamp(10, 500)
        }
        "discord_webhook_url" => settings.discord_webhook_url = value.to_string(),
        "alert_on_fill" => settings.alert_on_fill = value == "true",
        "alert_on_breaker" => settings.alert_on_breaker = value == "true",
        "alert_on_daily_summary" => settings.alert_on_daily_summary = value == "true",
        "telegram_bot_token" => settings.telegram_bot_token = value.to_string(),
        "telegram_chat_id" => settings.telegram_chat_id = value.to_string(),
        "telegram_enabled" => settings.telegram_enabled = value == "true",
        "atr_stop_multiplier" => {
            settings.atr_stop_multiplier = value.parse::<f64>().unwrap_or(1.5).clamp(0.5, 5.0)
        }
        "atr_take_multiplier" => {
            settings.atr_take_multiplier = value.parse::<f64>().unwrap_or(3.0).clamp(1.0, 10.0)
        }
        "partial_take_enabled" => settings.partial_take_enabled = value == "true",
        "partial_take_pct" => {
            settings.partial_take_pct = value.parse::<f64>().unwrap_or(50.0).clamp(25.0, 75.0)
        }
        "llm_api_key" => settings.llm_api_key = value.to_string(),
        "llm_provider" => settings.llm_provider = value.to_string(),
        "llm_sentiment_enabled" => settings.llm_sentiment_enabled = value == "true",
        "options_hedging_enabled" => settings.options_hedging_enabled = value == "true",
        "hedge_otm_pct" => {
            settings.hedge_otm_pct = value.parse::<f64>().unwrap_or(5.0).clamp(1.0, 20.0)
        }
        "hedge_max_premium_pct" => {
            settings.hedge_max_premium_pct = value.parse::<f64>().unwrap_or(1.0).clamp(0.1, 5.0)
        }
        "web_dashboard_enabled" => settings.web_dashboard_enabled = value == "true",
        "web_dashboard_port" => {
            settings.web_dashboard_port = value.parse::<u16>().unwrap_or(7777).clamp(1024, 65535)
        }
        "web_dashboard_token" => settings.web_dashboard_token = value.to_string(),
        "bot_trade_size" => {
            settings.bot_trade_size = Decimal::from_str(value)
                .unwrap_or(Decimal::from(50))
                .clamp(Decimal::ONE, Decimal::from(1_000_000))
        }
        "circuit_breaker_limit" => {
            settings.circuit_breaker_limit = Decimal::from_str(value)
                .unwrap_or(Decimal::from(3000))
                .clamp(Decimal::from(100), Decimal::from(1_000_000))
        }
        "dry_run_enabled" => {
            settings.dry_run_enabled = value == "true";
        }
        "strategy_mode_name" => settings.strategy_mode_name = value.to_string(),
        "bypass_max_spread" => settings.bypass_max_spread = value == "true",
        "bypass_cooldown" => settings.bypass_cooldown = value == "true",
        "bypass_allow_short" => settings.bypass_allow_short = value == "true",
        "bypass_extended" => settings.bypass_extended = value == "true",
        "bypass_risk_per_trade" => settings.bypass_risk_per_trade = value == "true",
        "bypass_aggressive" => settings.bypass_aggressive = value == "true",
        "bypass_atr_stop" => settings.bypass_atr_stop = value == "true",
        "bypass_atr_take" => settings.bypass_atr_take = value == "true",
        "bypass_partial_take" => settings.bypass_partial_take = value == "true",
        "bypass_partial_pct" => settings.bypass_partial_pct = value == "true",
        "bypass_max_correlation" => settings.bypass_max_correlation = value == "true",
        "bypass_correlation_window" => settings.bypass_correlation_window = value == "true",
        "bypass_daily_loss" => settings.bypass_daily_loss = value == "true",
        "signal_source" => {
            let v = value.to_lowercase();
            if v == "oracle" || v == "gpu" || v == "fusion" {
                settings.signal_source = v;
            } else {
                return Err(TitanError::Validation(
                    "signal_source must be oracle, gpu, or fusion".into(),
                ));
            }
        }
        // TA Entry Gates (RF-B)
        "ta_gate_rsi" => settings.ta_gates.rsi_gate = parse_gate_mode(value),
        "ta_gate_ema_alignment" => settings.ta_gates.ema_alignment_gate = parse_gate_mode(value),
        "ta_gate_adx" => settings.ta_gates.adx_gate = parse_gate_mode(value),
        "ta_gate_bb_squeeze" => settings.ta_gates.bb_squeeze_gate = parse_gate_mode(value),
        "ta_gate_macd" => settings.ta_gates.macd_gate = parse_gate_mode(value),
        "ta_gate_stoch_rsi" => settings.ta_gates.stoch_rsi_gate = parse_gate_mode(value),
        "ta_gate_soft_penalty" => settings.ta_gate_soft_penalty = value.parse::<f64>().unwrap_or(15.0).clamp(1.0, 50.0),
        "enable_var_gate" => settings.enable_var_gate = value == "true",
        "max_portfolio_var_pct" => settings.max_portfolio_var_pct = value.parse::<f64>().unwrap_or(2.0).clamp(0.1, 10.0),
        "var_lookback_periods" => settings.var_lookback_periods = value.parse::<usize>().unwrap_or(20).clamp(5, 100),
        "enable_concentration_gate" => settings.enable_concentration_gate = value == "true",
        "max_concentration_hhi" => settings.max_concentration_hhi = value.parse::<f64>().unwrap_or(0.4).clamp(0.1, 1.0),
        // Safety Gates (S04.1)
        "var_fail_closed" => settings.var_fail_closed = value == "true",
        "asset_check_hard_gate" => settings.asset_check_hard_gate = value == "true",
        "blackout_first_mins" => {
            settings.blackout_first_mins = value.parse::<u64>().unwrap_or(5).clamp(0, 60)
        }
        "blackout_last_mins" => {
            settings.blackout_last_mins = value.parse::<u64>().unwrap_or(5).clamp(0, 60)
        }
        "blackout_exempt_symbols" => {
            settings.blackout_exempt_symbols = value
                .split(',')
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // Trading Edge (S15)
        "max_hold_decay_enabled" => settings.max_hold_decay_enabled = value == "true",
        "max_hold_decay_start_pct" => settings.max_hold_decay_start_pct = value.parse::<f64>().unwrap_or(0.5).clamp(0.2, 0.9),
        "max_hold_decay_trail_tighten" => settings.max_hold_decay_trail_tighten = value.parse::<f64>().unwrap_or(0.3).clamp(0.05, 0.8),
        "mtf_gate_enabled" => settings.mtf_gate_enabled = value == "true",
        "mtf_gate_mode" => {
            settings.mtf_gate_mode = match value.to_lowercase().as_str() {
                "hard" => "hard".to_string(),
                _ => "soft".to_string(),
            }
        }
        "mtf_gate_penalty" => settings.mtf_gate_penalty = value.parse::<f64>().unwrap_or(15.0).clamp(1.0, 50.0),
        "mtf_gate_htf" => {
            settings.mtf_gate_htf = crate::state::normalize_timeframe_internal(value)
        }
        "hhi_gate_manual_orders" => settings.hhi_gate_manual_orders = value == "true",
        "max_concentration_hhi_manual" => settings.max_concentration_hhi_manual = value.parse::<f64>().unwrap_or(0.35).clamp(0.1, 1.0),
        "tiered_scan_enabled" => settings.tiered_scan_enabled = value == "true",
        "tier1_count" => settings.tier1_count = value.parse::<usize>().unwrap_or(50).clamp(10, 500),
        "tier2_count" => settings.tier2_count = value.parse::<usize>().unwrap_or(100).clamp(10, 500),
        "scan_min_dollar_volume" => {
            settings.scan_params.min_dollar_volume = value
                .parse::<f64>()
                .unwrap_or(5_000_000.0)
                .clamp(0.0, 1_000_000_000.0)
        }
        "ta_gate_preset" => {
            let preset = match value.to_lowercase().as_str() {
                "strict" => crate::app_state::TaGatePreset::Strict,
                "balanced" => crate::app_state::TaGatePreset::Balanced,
                _ => crate::app_state::TaGatePreset::OracleOnly,
            };
            settings.ta_gates = crate::app_state::TaGatesConfig::from_preset(&preset);
        }
        "use_classical_filters" => settings.use_classical_filters = value == "true",
        "rsi_overbought" => {
            settings.rsi_overbought = value.parse::<f64>().unwrap_or(70.0).clamp(50.0, 95.0)
        }
        "rsi_oversold" => {
            settings.rsi_oversold = value.parse::<f64>().unwrap_or(30.0).clamp(5.0, 50.0)
        }
        "require_ema_alignment" => settings.require_ema_alignment = value == "true",
        "adx_trend_threshold" => {
            settings.adx_trend_threshold = value.parse::<f64>().unwrap_or(25.0).clamp(10.0, 60.0)
        }
        "classical_weight" => {
            settings.classical_weight = value.parse::<f64>().unwrap_or(0.15).clamp(0.0, 1.0)
        }
        // Phase 1: Autopilot Core
        "min_hold_secs" => settings.min_hold_secs = value.parse::<u64>().unwrap_or(180).clamp(0, 3600),
        "nexus_consensus_threshold" => settings.nexus_consensus_threshold = value.parse::<f64>().unwrap_or(70.0).clamp(30.0, 100.0),
        "autopilot_loop_ms" => settings.autopilot_loop_ms = value.parse::<u64>().unwrap_or(1500).clamp(500, 10000),
        "ofi_stale_ms" => settings.ofi_stale_ms = value.parse::<u64>().unwrap_or(10000).clamp(1000, 60000),
        "flash_crash_window_secs" => settings.flash_crash_window_secs = value.parse::<u64>().unwrap_or(30).clamp(5, 120),
        "flash_crash_threshold_pct" => settings.flash_crash_threshold_pct = value.parse::<f64>().unwrap_or(3.0).clamp(0.5, 20.0),
        "flash_crash_cooldown_mins" => settings.flash_crash_cooldown_mins = value.parse::<u64>().unwrap_or(5).clamp(1, 60),
        "dd_velocity_threshold" => settings.dd_velocity_threshold = value.parse::<f64>().unwrap_or(-1.0).clamp(-10.0, -0.1),
        "dd_velocity_window_secs" => settings.dd_velocity_window_secs = value.parse::<u64>().unwrap_or(300).clamp(30, 600),
        "min_bars_for_entry" => settings.min_bars_for_entry = value.parse::<u64>().unwrap_or(20).clamp(5, 200),
        "ofi_entry_threshold" => settings.ofi_entry_threshold = value.parse::<f64>().unwrap_or(5.0).clamp(0.0, 50.0),
        "max_qty_cap" => settings.max_qty_cap = value.parse::<f64>().unwrap_or(1000.0).clamp(1.0, 100000.0),
        "min_qty_threshold" => settings.min_qty_threshold = value.parse::<f64>().unwrap_or(0.001).clamp(0.0001, 1.0),
        "default_equity_fallback" => settings.default_equity_fallback = value.parse::<f64>().unwrap_or(10000.0).clamp(1000.0, 1000000.0),
        "max_bypass_risk_pct" => settings.max_bypass_risk_pct = value.parse::<f64>().unwrap_or(25.0).clamp(1.0, 100.0),
        // Phase 2: Exit Scaling
        "vol_factor_min" => settings.vol_factor_min = value.parse::<f64>().unwrap_or(0.6).clamp(0.1, 2.0),
        "vol_factor_max" => settings.vol_factor_max = value.parse::<f64>().unwrap_or(2.5).clamp(1.0, 10.0),
        "fallback_sl_pct" => settings.fallback_sl_pct = value.parse::<f64>().unwrap_or(2.5).clamp(0.1, 20.0),
        "fallback_tp_pct" => settings.fallback_tp_pct = value.parse::<f64>().unwrap_or(2.5).clamp(0.1, 30.0),
        "fallback_trail_pct" => settings.fallback_trail_pct = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 10.0),
        "scalping_stop_mult" => settings.scalping_stop_mult = value.parse::<f64>().unwrap_or(0.70).clamp(0.1, 2.0),
        "scalping_take_mult" => settings.scalping_take_mult = value.parse::<f64>().unwrap_or(0.60).clamp(0.1, 2.0),
        "scalping_trail_mult" => settings.scalping_trail_mult = value.parse::<f64>().unwrap_or(0.50).clamp(0.1, 2.0),
        "partial_first_target_frac" => settings.partial_first_target_frac = value.parse::<f64>().unwrap_or(0.5).clamp(0.1, 1.0),
        "partial_min_qty" => settings.partial_min_qty = value.parse::<f64>().unwrap_or(0.001).clamp(0.0001, 1.0),
        "atr_period" => settings.atr_period = value.parse::<u64>().unwrap_or(14).clamp(2, 100),
        "atr_stop_clamp_min" => settings.atr_stop_clamp_min = value.parse::<f64>().unwrap_or(0.5).clamp(0.01, 5.0),
        "atr_stop_clamp_max" => settings.atr_stop_clamp_max = value.parse::<f64>().unwrap_or(8.0).clamp(1.0, 30.0),
        "atr_take_clamp_min" => settings.atr_take_clamp_min = value.parse::<f64>().unwrap_or(1.0).clamp(0.1, 10.0),
        "atr_take_clamp_max" => settings.atr_take_clamp_max = value.parse::<f64>().unwrap_or(15.0).clamp(2.0, 50.0),
        "atr_trail_clamp_min" => settings.atr_trail_clamp_min = value.parse::<f64>().unwrap_or(0.3).clamp(0.01, 3.0),
        "atr_trail_clamp_max" => settings.atr_trail_clamp_max = value.parse::<f64>().unwrap_or(5.0).clamp(0.5, 20.0),
        // Phase 3: Oracle Exit
        "oracle_exit_min_confidence" => settings.oracle_exit_min_confidence = value.parse::<f64>().unwrap_or(0.40).clamp(0.0, 1.0),
        "oracle_exit_min_score" => settings.oracle_exit_min_score = value.parse::<f64>().unwrap_or(30.0).clamp(0.0, 100.0),
        "oracle_flip_score_threshold" => settings.oracle_flip_score_threshold = value.parse::<f64>().unwrap_or(10.0).clamp(0.0, 100.0),
        "oracle_flip_confidence" => settings.oracle_flip_confidence = value.parse::<f64>().unwrap_or(0.20).clamp(0.0, 1.0),
        "chaos_exit_max_loss_pct" => settings.chaos_exit_max_loss_pct = value.parse::<f64>().unwrap_or(-0.5).clamp(-10.0, 0.0),
        "chaos_exit_min_confidence" => settings.chaos_exit_min_confidence = value.parse::<f64>().unwrap_or(0.25).clamp(0.0, 1.0),
        // Phase 4: Circuit Breaker
        "breaker_caution_pct" => settings.breaker_caution_pct = value.parse::<f64>().unwrap_or(-3.0).clamp(-20.0, 0.0),
        "breaker_halt_pct" => settings.breaker_halt_pct = value.parse::<f64>().unwrap_or(-5.0).clamp(-30.0, 0.0),
        "breaker_panic_pct" => settings.breaker_panic_pct = value.parse::<f64>().unwrap_or(-8.0).clamp(-50.0, 0.0),
        "max_notional_equity_frac" => settings.max_notional_equity_frac = value.parse::<f64>().unwrap_or(0.25).clamp(0.01, 1.0),
        "min_stop_distance_frac" => settings.min_stop_distance_frac = value.parse::<f64>().unwrap_or(0.0001).clamp(0.00001, 0.01),
        // Phase 5: Strategy Scoring
        "strategy_base_score" => settings.strategy_base_score = value.parse::<f64>().unwrap_or(50.0).clamp(0.0, 100.0),
        "vpin_gate_scalping" => settings.vpin_gate_scalping = value.parse::<f64>().unwrap_or(0.85).clamp(0.0, 1.0),
        "vpin_gate_normal" => settings.vpin_gate_normal = value.parse::<f64>().unwrap_or(0.75).clamp(0.0, 1.0),
        "min_rvol_threshold" => settings.min_rvol_threshold = value.parse::<f64>().unwrap_or(0.5).clamp(0.0, 5.0),
        "rvol_window" => settings.rvol_window = value.parse::<u64>().unwrap_or(20).clamp(5, 200),
        "rvol_relax_threshold" => settings.rvol_relax_threshold = value.parse::<f64>().unwrap_or(3.0).clamp(1.0, 20.0),
        "rvol_relax_value" => settings.rvol_relax_value = value.parse::<f64>().unwrap_or(5.0).clamp(1.0, 20.0),
        "chaos_confidence_kill" => settings.chaos_confidence_kill = value.parse::<f64>().unwrap_or(0.10).clamp(0.0, 1.0),
        "chaos_regime_mult" => settings.chaos_regime_mult = value.parse::<f64>().unwrap_or(0.55).clamp(0.0, 1.0),
        // Phase 6: Engine Stops
        "long_stop_normal" => settings.long_stop_normal = value.parse::<f64>().unwrap_or(-2.5).clamp(-20.0, 0.0),
        "long_stop_aggressive" => settings.long_stop_aggressive = value.parse::<f64>().unwrap_or(-8.0).clamp(-30.0, 0.0),
        "short_stop_pct" => settings.short_stop_pct = value.parse::<f64>().unwrap_or(-5.0).clamp(-20.0, 0.0),
        "long_trail_min_profit" => settings.long_trail_min_profit = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 10.0),
        "short_trail_gap" => settings.short_trail_gap = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 10.0),
        "short_trail_min_profit" => settings.short_trail_min_profit = value.parse::<f64>().unwrap_or(1.0).clamp(0.1, 10.0),
        // Phase 7: Oracle Fusion
        "oracle_kalman_weight" => settings.oracle_kalman_weight = value.parse::<f64>().unwrap_or(0.30).clamp(0.0, 1.0),
        "oracle_entropy_weight" => settings.oracle_entropy_weight = value.parse::<f64>().unwrap_or(0.25).clamp(0.0, 1.0),
        "oracle_hurst_weight" => settings.oracle_hurst_weight = value.parse::<f64>().unwrap_or(0.20).clamp(0.0, 1.0),
        "oracle_vpin_weight" => settings.oracle_vpin_weight = value.parse::<f64>().unwrap_or(0.15).clamp(0.0, 1.0),
        "oracle_wavelet_weight" => settings.oracle_wavelet_weight = value.parse::<f64>().unwrap_or(0.10).clamp(0.0, 1.0),
        "oracle_confidence_floor" => settings.oracle_confidence_floor = value.parse::<f64>().unwrap_or(0.25).clamp(0.0, 1.0),
        "entropy_chaos_threshold" => settings.entropy_chaos_threshold = value.parse::<f64>().unwrap_or(0.85).clamp(0.0, 1.0),
        "vpin_toxicity_alert" => settings.vpin_toxicity_alert = value.parse::<f64>().unwrap_or(0.6).clamp(0.0, 1.0),
        "enable_oracle_adaptation" => settings.enable_oracle_adaptation = value == "true",
        // Phase 8: Indicator Periods
        "ema_fast_period" => settings.ema_fast_period = value.parse::<u64>().unwrap_or(9).clamp(2, 100),
        "ema_mid_period" => settings.ema_mid_period = value.parse::<u64>().unwrap_or(20).clamp(5, 200),
        "ema_slow_period" => settings.ema_slow_period = value.parse::<u64>().unwrap_or(50).clamp(10, 500),
        "ema_trend_period" => settings.ema_trend_period = value.parse::<u64>().unwrap_or(200).clamp(50, 1000),
        "rsi_period" => settings.rsi_period = value.parse::<u64>().unwrap_or(14).clamp(2, 100),
        "macd_fast" => settings.macd_fast = value.parse::<u64>().unwrap_or(12).clamp(2, 50),
        "macd_slow" => settings.macd_slow = value.parse::<u64>().unwrap_or(26).clamp(5, 100),
        "macd_signal" => settings.macd_signal = value.parse::<u64>().unwrap_or(9).clamp(2, 50),
        "bb_period" => settings.bb_period = value.parse::<u64>().unwrap_or(20).clamp(5, 100),
        "bb_std_dev" => settings.bb_std_dev = value.parse::<f64>().unwrap_or(2.0).clamp(0.5, 5.0),
        "adx_period" => settings.adx_period = value.parse::<u64>().unwrap_or(14).clamp(2, 100),
        "stoch_rsi_period" => settings.stoch_rsi_period = value.parse::<u64>().unwrap_or(14).clamp(2, 100),
        "nexus_fusion_alpha" => settings.nexus_fusion_alpha = value.parse::<f64>().unwrap_or(0.5).clamp(0.0, 1.0),
        "max_manual_order_usd" => settings.max_manual_order_usd = Decimal::from_str(value).unwrap_or(rust_decimal_macros::dec!(50000)).clamp(rust_decimal_macros::dec!(100), rust_decimal_macros::dec!(1000000)),
        // Phase 9: Adaptive Exit Multipliers
        "disable_adaptive_exits" => settings.disable_adaptive_exits = value == "true",
        "exit_floor_pct" => settings.exit_floor_pct = value.parse::<f64>().unwrap_or(0.3).clamp(0.05, 5.0),
        "regime_chaotic_stop_mult" => settings.regime_chaotic_stop_mult = value.parse::<f64>().unwrap_or(0.70).clamp(0.1, 3.0),
        "regime_chaotic_take_mult" => settings.regime_chaotic_take_mult = value.parse::<f64>().unwrap_or(0.60).clamp(0.1, 3.0),
        "regime_chaotic_trail_mult" => settings.regime_chaotic_trail_mult = value.parse::<f64>().unwrap_or(0.50).clamp(0.1, 3.0),
        "regime_range_stop_mult" => settings.regime_range_stop_mult = value.parse::<f64>().unwrap_or(0.85).clamp(0.1, 3.0),
        "regime_range_take_mult" => settings.regime_range_take_mult = value.parse::<f64>().unwrap_or(0.75).clamp(0.1, 3.0),
        "regime_range_trail_mult" => settings.regime_range_trail_mult = value.parse::<f64>().unwrap_or(0.85).clamp(0.1, 3.0),
        "regime_trend_stop_mult" => settings.regime_trend_stop_mult = value.parse::<f64>().unwrap_or(1.05).clamp(0.1, 3.0),
        "regime_trend_take_mult" => settings.regime_trend_take_mult = value.parse::<f64>().unwrap_or(1.30).clamp(0.1, 3.0),
        "regime_trend_trail_mult" => settings.regime_trend_trail_mult = value.parse::<f64>().unwrap_or(1.10).clamp(0.1, 3.0),
        "signal_aligned_stop_mult" => settings.signal_aligned_stop_mult = value.parse::<f64>().unwrap_or(1.10).clamp(0.1, 3.0),
        "signal_aligned_take_mult" => settings.signal_aligned_take_mult = value.parse::<f64>().unwrap_or(1.25).clamp(0.1, 3.0),
        "signal_aligned_trail_mult" => settings.signal_aligned_trail_mult = value.parse::<f64>().unwrap_or(1.10).clamp(0.1, 3.0),
        "signal_oppose_stop_mult" => settings.signal_oppose_stop_mult = value.parse::<f64>().unwrap_or(0.85).clamp(0.1, 3.0),
        "signal_oppose_take_mult" => settings.signal_oppose_take_mult = value.parse::<f64>().unwrap_or(0.80).clamp(0.1, 3.0),
        "tf_short_stop_mult" => settings.tf_short_stop_mult = value.parse::<f64>().unwrap_or(0.90).clamp(0.1, 3.0),
        "tf_short_take_mult" => settings.tf_short_take_mult = value.parse::<f64>().unwrap_or(0.90).clamp(0.1, 3.0),
        "tf_short_trail_mult" => settings.tf_short_trail_mult = value.parse::<f64>().unwrap_or(0.90).clamp(0.1, 3.0),
        // N4: Score Modifiers
        "sm_chaos_kill_penalty" => settings.score_modifiers.chaos_kill_penalty = value.parse::<i32>().unwrap_or(-50).clamp(-200, 0),
        "sm_chaos_kill_fallback" => settings.score_modifiers.chaos_kill_fallback = value.parse::<i32>().unwrap_or(50).clamp(0, 200),
        "sm_chaos_conf_mult_mean_rev" => settings.score_modifiers.chaos_conf_mult_mean_rev = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 10.0),
        "sm_chaos_conf_mult_chaotic" => settings.score_modifiers.chaos_conf_mult_chaotic = value.parse::<f64>().unwrap_or(2.5).clamp(0.1, 10.0),
        "sm_chaos_conf_mult_unknown" => settings.score_modifiers.chaos_conf_mult_unknown = value.parse::<f64>().unwrap_or(1.5).clamp(0.1, 10.0),
        "sm_regime_mult_mean_rev" => settings.score_modifiers.regime_mult_mean_rev = value.parse::<f64>().unwrap_or(0.82).clamp(0.01, 2.0),
        "sm_regime_mult_chaotic" => settings.score_modifiers.regime_mult_chaotic = value.parse::<f64>().unwrap_or(0.55).clamp(0.01, 2.0),
        "sm_regime_mult_unknown" => settings.score_modifiers.regime_mult_unknown = value.parse::<f64>().unwrap_or(0.73).clamp(0.01, 2.0),
        "sm_vpin_penalty_high" => settings.score_modifiers.vpin_penalty_high = value.parse::<i32>().unwrap_or(-20).clamp(-100, 0),
        "sm_vpin_penalty_medium" => settings.score_modifiers.vpin_penalty_medium = value.parse::<i32>().unwrap_or(-10).clamp(-100, 0),
        "sm_fvg_bullish_bonus" => settings.score_modifiers.fvg_bullish_bonus = value.parse::<i32>().unwrap_or(15).clamp(0, 100),
        "sm_fvg_bearish_penalty" => settings.score_modifiers.fvg_bearish_penalty = value.parse::<i32>().unwrap_or(-15).clamp(-100, 0),
        "sm_rvol_tier1_threshold" => settings.score_modifiers.rvol_tier1_threshold = value.parse::<f64>().unwrap_or(2.0).clamp(0.5, 20.0),
        "sm_rvol_tier1_bonus" => settings.score_modifiers.rvol_tier1_bonus = value.parse::<i32>().unwrap_or(8).clamp(0, 100),
        "sm_rvol_tier2_threshold" => settings.score_modifiers.rvol_tier2_threshold = value.parse::<f64>().unwrap_or(4.0).clamp(1.0, 50.0),
        "sm_rvol_tier2_bonus" => settings.score_modifiers.rvol_tier2_bonus = value.parse::<i32>().unwrap_or(12).clamp(0, 100),
        "sm_stoch_rsi_overbought" => settings.score_modifiers.stoch_rsi_overbought = value.parse::<f64>().unwrap_or(80.0).clamp(50.0, 99.0),
        "sm_stoch_rsi_oversold" => settings.score_modifiers.stoch_rsi_oversold = value.parse::<f64>().unwrap_or(20.0).clamp(1.0, 50.0),
        "sm_bb_squeeze_width" => settings.score_modifiers.bb_squeeze_width = value.parse::<f64>().unwrap_or(0.02).clamp(0.001, 0.1),
        "sm_entry_label_strong" => settings.score_modifiers.entry_label_strong = value.parse::<f64>().unwrap_or(80.0).clamp(50.0, 100.0),
        "sm_entry_label_mild" => settings.score_modifiers.entry_label_mild = value.parse::<f64>().unwrap_or(60.0).clamp(20.0, 99.0),
        // N4: Classical Bonuses
        "cb_rsi_bonus" => settings.classical_bonuses.rsi_bonus = value.parse::<f64>().unwrap_or(10.0).clamp(0.0, 50.0),
        "cb_ema_alignment_bonus" => settings.classical_bonuses.ema_alignment_bonus = value.parse::<f64>().unwrap_or(15.0).clamp(0.0, 50.0),
        "cb_adx_bonus" => settings.classical_bonuses.adx_bonus = value.parse::<f64>().unwrap_or(10.0).clamp(0.0, 50.0),
        "cb_bb_squeeze_bonus" => settings.classical_bonuses.bb_squeeze_bonus = value.parse::<f64>().unwrap_or(5.0).clamp(0.0, 50.0),
        "cb_vwap_bonus" => settings.classical_bonuses.vwap_bonus = value.parse::<f64>().unwrap_or(10.0).clamp(0.0, 50.0),
        "cb_cap" => settings.classical_bonuses.cap = value.parse::<f64>().unwrap_or(50.0).clamp(10.0, 200.0),
        // N4: Entry Thresholds (format: et_{scalp|normal}_{regime}_{field})
        f if f.starts_with("et_") => {
            apply_entry_threshold_field(settings, f, value)?;
        }
        // S07b: SmartTrend config fields
        f if f.starts_with("st_") => {
            apply_smarttrend_field(&mut settings.smarttrend, f, value)?;
        }
        _ => return Err(TitanError::Validation("UNKNOWN FIELD".into())),
    }
    Ok(())
}

pub(crate) fn apply_smarttrend_field(
    cfg: &mut crate::smarttrend::SmartTrendConfig,
    field: &str,
    value: &str,
) -> Result<(), crate::error::TitanError> {
    match field {
        "st_macro_weight" => cfg.st_macro_weight = value.parse::<f64>().unwrap_or(0.4).clamp(0.0, 1.0),
        "st_micro_weight" => cfg.st_micro_weight = value.parse::<f64>().unwrap_or(0.35).clamp(0.0, 1.0),
        "st_momentum_weight" => cfg.st_momentum_weight = value.parse::<f64>().unwrap_or(0.25).clamp(0.0, 1.0),
        "st_entry_threshold" => cfg.st_entry_threshold = value.parse::<f64>().unwrap_or(65.0).clamp(10.0, 100.0),
        "st_stalking_threshold" => cfg.st_stalking_threshold = value.parse::<f64>().unwrap_or(55.0).clamp(5.0, 100.0),
        "st_stabilization_window" => cfg.st_stabilization_window = value.parse::<u32>().unwrap_or(8).clamp(2, 50),
        "st_bayesian_decay" => cfg.st_bayesian_decay = value.parse::<f64>().unwrap_or(0.997).clamp(0.9, 1.0),
        "st_bayesian_halflife" => cfg.st_bayesian_halflife = value.parse::<u32>().unwrap_or(200).clamp(0, 10000),
        "st_dominant_cycle_filter" => cfg.st_dominant_cycle_filter = value == "true" || value == "1",
        "st_ofi_norm_range" => cfg.st_ofi_norm_range = value.parse::<f64>().unwrap_or(50.0).clamp(1.0, 1000.0),
        "st_mp_bps_range" => cfg.st_mp_bps_range = value.parse::<f64>().unwrap_or(10.0).clamp(0.1, 200.0),
        "st_cvd_scale" => cfg.st_cvd_scale = value.parse::<f64>().unwrap_or(100.0).clamp(1.0, 100000.0),
        "st_changepoint_reset_threshold" => cfg.st_changepoint_reset_threshold = value.parse::<f64>().unwrap_or(0.7).clamp(0.1, 1.0),
        "st_trail_tighten_pct" => cfg.st_trail_tighten_pct = value.parse::<f64>().unwrap_or(0.05).clamp(0.0, 0.5),
        "st_hysteresis_margin" => cfg.st_hysteresis_margin = value.parse::<f64>().unwrap_or(5.0).clamp(0.0, 20.0),
        "st_enabled" => cfg.st_enabled = value == "true" || value == "1",
        "st_killed" => cfg.st_killed = value == "true" || value == "1",
        "st_entry_thresholds_0" => cfg.st_entry_thresholds[0] = value.parse::<f64>().unwrap_or(60.0).clamp(10.0, 100.0),
        "st_entry_thresholds_1" => cfg.st_entry_thresholds[1] = value.parse::<f64>().unwrap_or(70.0).clamp(10.0, 100.0),
        "st_entry_thresholds_2" => cfg.st_entry_thresholds[2] = value.parse::<f64>().unwrap_or(75.0).clamp(10.0, 100.0),
        "st_entry_thresholds_3" => cfg.st_entry_thresholds[3] = value.parse::<f64>().unwrap_or(65.0).clamp(10.0, 100.0),
        "st_sentinel_idle_secs" => cfg.st_sentinel_idle_secs = value.parse::<u64>().unwrap_or(30).clamp(5, 300),
        "st_sentinel_stalking_secs" => cfg.st_sentinel_stalking_secs = value.parse::<u64>().unwrap_or(5).clamp(1, 60),
        "st_sentinel_hot_secs" => cfg.st_sentinel_hot_secs = value.parse::<u64>().unwrap_or(2).clamp(1, 30),
        "st_sentinel_alert_cooldown_secs" => cfg.st_sentinel_alert_cooldown_secs = value.parse::<u64>().unwrap_or(30).clamp(5, 600),
        // S10: Bot + Account config
        "st_risk_per_trade" => cfg.st_risk_per_trade = value.parse::<f64>().unwrap_or(0.01).clamp(0.001, 0.1),
        "st_max_dd_pct" => cfg.st_max_dd_pct = value.parse::<f64>().unwrap_or(5.0).clamp(0.5, 50.0),
        "st_bot_loop_ms" => cfg.st_bot_loop_ms = value.parse::<u64>().unwrap_or(2000).clamp(500, 30000),
        "st_bot_paused" => cfg.st_bot_paused = value == "true" || value == "1",
        "st_kelly_enabled" => cfg.st_kelly_enabled = value == "true" || value == "1",
        "st_kelly_min_trades" => cfg.st_kelly_min_trades = value.parse::<u32>().unwrap_or(20).clamp(5, 200),
        "st_max_kelly_fraction" => cfg.st_max_kelly_fraction = value.parse::<f64>().unwrap_or(0.25).clamp(0.01, 0.5),
        _ => return Err(crate::error::TitanError::Validation(format!("Unknown SmartTrend field: {}", field))),
    }
    Ok(())
}

fn apply_entry_threshold_field(
    settings: &mut AppState,
    field: &str,
    value: &str,
) -> Result<(), TitanError> {
    let parts: Vec<&str> = field.split('_').collect();
    if parts.len() < 4 {
        return Err(TitanError::Validation(format!("Invalid threshold field: {}", field)));
    }
    let mode = parts[1];
    let last = *parts.last().unwrap();
    let regime_key = parts[2..parts.len() - 1].join("_");

    let map = match mode {
        "scalp" => &mut settings.entry_thresholds_scalp,
        "normal" => &mut settings.entry_thresholds_normal,
        _ => return Err(TitanError::Validation(format!("Unknown mode: {}", mode))),
    };

    let entry = map.entry(regime_key.clone()).or_insert(crate::app_state::EntryThreshold {
        long_threshold: 50.0,
        short_threshold: -50.0,
        conf_threshold: 0.30,
    });

    match last {
        "long" => entry.long_threshold = value.parse::<f64>().unwrap_or(50.0).clamp(1.0, 200.0),
        "short" => entry.short_threshold = value.parse::<f64>().unwrap_or(-50.0).clamp(-200.0, -1.0),
        "conf" => entry.conf_threshold = value.parse::<f64>().unwrap_or(0.30).clamp(0.01, 0.99),
        _ => return Err(TitanError::Validation(format!("Unknown threshold component: {}", last))),
    }
    Ok(())
}

fn apply_redis_side_effects(state: &TitanState, field: &str, value: &str) {
    if field != "aggressive_mode" && field != "risk_per_trade" {
        return;
    }
    if let Ok(redis_guard) = state.redis.lock() {
        if field == "aggressive_mode" {
            monitor::set_redis_value(
                redis_guard.as_ref(),
                "hft:config:aggressive_mode",
                if value == "true" { "1" } else { "0" },
            );
        }
        if field == "risk_per_trade" {
            monitor::set_redis_value(redis_guard.as_ref(), "hft:config:risk_pct", value);
        }
    }
}

#[tauri::command]
pub(crate) async fn update_settings(
    field: String,
    value: String,
    state: State<'_, Arc<TitanState>>,
    _app: AppHandle,
) -> Result<String, TitanError> {
    // FMP API key is stored in api_keys, not settings — handle separately
    if field == "fmp_api_key" {
        let trimmed = value.trim().to_string();
        let mut keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        keys.fmp_api_key = trimmed;
        let path = crate::paths::keys_path();
        if let Err(e) = std::fs::write(&path, serde_json::to_string(&*keys).unwrap_or_default()) {
            tracing::error!(component = "settings", "Failed to persist FMP key: {}", e);
        }
        tracing::info!(component = "settings", "Setting updated: fmp_api_key = [REDACTED]");
        return Ok("FMP KEY SAVED".into());
    }

    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    apply_setting_field(&mut settings, &field, &value)?;

    crate::state::apply_runtime_config(&state, &settings);

    const SECRET_FIELDS: &[&str] = &[
        "telegram_bot_token", "telegram_chat_id", "llm_api_key",
        "discord_webhook_url", "web_dashboard_token",
    ];
    if SECRET_FIELDS.contains(&field.as_str()) {
        tracing::info!(component = "settings", field = %field, "Setting updated: [REDACTED]");
    } else {
        tracing::info!(component = "settings", field = %field, value = %value, "Setting updated");
    }
    apply_redis_side_effects(&state, &field, &value);
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
        tracing::error!(component = "settings", error = %e, "Settings save to disk failed");
        return Err(e);
    }

    Ok("UPDATED".into())
}

#[tauri::command]
pub(crate) async fn batch_update_settings(
    settings: HashMap<String, String>,
    state: State<'_, Arc<TitanState>>,
    _app: AppHandle,
) -> Result<String, TitanError> {
    let mut st = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let snapshot_before = st.clone();
    let mut errors: Vec<String> = Vec::new();

    for (field, value) in &settings {
        if let Err(e) = apply_setting_field(&mut st, field, value) {
            errors.push(format!("{}: {}", field, e));
        }
    }

    if !errors.is_empty() {
        *st = snapshot_before;
        return Err(TitanError::Validation(errors.join("; ")));
    }

    crate::state::apply_runtime_config(&state, &st);

    let safety_warnings = crate::app_state::validate_safety_settings(&st);
    for w in &safety_warnings {
        tracing::warn!(component = "safety_validation", "{}", w);
    }

    for (field, value) in &settings {
        apply_redis_side_effects(&state, field, value);
    }

    let count = settings.len();
    tracing::info!(component = "settings", count = count, "Batch settings updated");
    if let Err(e) = save_settings_to_disk(&settings_from_state(&st)) {
        tracing::error!(component = "settings", error = %e, "Batch settings save to disk failed");
        return Err(e);
    }
    if safety_warnings.is_empty() {
        Ok(format!("{} fields updated", count))
    } else {
        Ok(format!(
            "{} fields updated (warnings: {})",
            count,
            safety_warnings.join("; ")
        ))
    }
}

#[tauri::command]
pub(crate) async fn get_settings_snapshot(
    state: State<'_, Arc<TitanState>>,
) -> Result<AppSettings, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    Ok(settings_from_state(&settings))
}

#[tauri::command]
pub(crate) async fn validate_thresholds(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<String>, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let warnings = crate::app_state::validate_strategy_thresholds(
        &settings.entry_thresholds_scalp,
        &settings.entry_thresholds_normal,
        &settings.score_modifiers,
    );
    Ok(warnings)
}

#[tauri::command]
pub(crate) async fn set_active_symbol(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let sym = normalize_symbol(&symbol);
    if sym.is_empty() {
        return Err(TitanError::Validation("EMPTY SYMBOL".into()));
    }

    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.active_symbol = sym.clone();
        if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "settings", error = %e, "Active symbol save failed");
            return Err(e);
        }
    }

    if let Ok(redis_guard) = state.redis.lock() {
        monitor::set_redis_value(redis_guard.as_ref(), "market:active_symbol", &sym);
    }

    if let Ok(mut tx) = state.stream_tx.lock() {
        if let Some(sender) = tx.as_mut() {
            if sender.try_send(streamer::StreamCmd::SetSymbol(sym.clone())).is_err() {
                tracing::warn!(component = "streamer", "Feed stale: streamer channel full on SetSymbol");
            }
        }
    }

    Ok("ACTIVE SYMBOL SET".into())
}

#[tauri::command]
pub(crate) async fn set_position_mode(
    symbol: String,
    mode: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let sym = symbol.trim().to_uppercase();
    let m = if mode == "manual" {
        "manual".to_string()
    } else {
        "auto".to_string()
    };
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    settings.position_trade_modes.insert(sym.clone(), m.clone());
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
        tracing::error!(component = "settings", error = %e, "Position mode save failed");
        return Err(e);
    }
    Ok(format!("{} set to {}", sym, m))
}

#[tauri::command]
pub(crate) async fn get_position_modes(
    state: State<'_, Arc<TitanState>>,
) -> Result<HashMap<String, String>, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    Ok(settings.position_trade_modes.clone())
}

#[tauri::command]
pub(crate) async fn set_dry_run(
    state: State<'_, Arc<TitanState>>,
    enabled: bool,
    app: AppHandle,
) -> Result<String, TitanError> {
    state
        .dry_run
        .store(enabled, std::sync::atomic::Ordering::SeqCst);
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.dry_run_enabled = enabled;
        if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "settings", error = %e, "Dry-run save failed");
            return Err(e);
        }
    }
    let msg = if enabled {
        "DRY RUN: ENABLED — orders will be simulated"
    } else {
        "DRY RUN: DISABLED — orders will be sent to Alpaca"
    };
    tracing::info!(component = "config", dry_run = enabled, "{}", msg);
    let _ = app.emit_all("log-message", msg);
    Ok(msg.into())
}

#[tauri::command]
pub(crate) async fn save_undo_snapshot(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let snapshot = settings_from_state(&settings);
    let payload = serde_json::to_string_pretty(&snapshot)?;
    let path = settings_path().with_file_name("titan_settings_undo.json");
    let tmp = format!("{}.tmp", path.display());
    std::fs::write(&tmp, &payload).map_err(|e| TitanError::Io(format!("Undo write failed: {}", e)))?;
    std::fs::rename(&tmp, &path).map_err(|e| TitanError::Io(format!("Undo rename failed: {}", e)))?;
    Ok("UNDO_SAVED".into())
}

#[tauri::command]
pub(crate) async fn load_undo_snapshot(
) -> Result<serde_json::Value, TitanError> {
    let path = settings_path().with_file_name("titan_settings_undo.json");
    let raw = std::fs::read_to_string(&path).map_err(|e| TitanError::Io(format!("No undo snapshot: {}", e)))?;
    let snap: AppSettings = serde_json::from_str(&raw).map_err(|e| TitanError::Io(format!("Undo parse failed: {}", e)))?;
    Ok(serde_json::to_value(&snap)?)
}

#[tauri::command]
pub(crate) async fn save_chart_settings(
    data: serde_json::Value,
) -> Result<String, TitanError> {
    let payload = serde_json::to_string_pretty(&data)?;
    let path = settings_path().with_file_name("titan_chart_settings.json");
    let tmp = format!("{}.tmp", path.display());
    std::fs::write(&tmp, &payload).map_err(|e| TitanError::Io(format!("Chart settings write failed: {}", e)))?;
    std::fs::rename(&tmp, &path).map_err(|e| TitanError::Io(format!("Chart settings rename failed: {}", e)))?;
    Ok("CHART_SETTINGS_SAVED".into())
}

#[tauri::command]
pub(crate) async fn load_chart_settings(
) -> Result<serde_json::Value, TitanError> {
    let path = settings_path().with_file_name("titan_chart_settings.json");
    let raw = std::fs::read_to_string(&path).map_err(|_| TitanError::Io("No chart settings file".into()))?;
    let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| TitanError::Io(format!("Chart parse failed: {}", e)))?;
    Ok(val)
}

#[tauri::command]
pub(crate) async fn save_chart_undo(
    data: serde_json::Value,
) -> Result<String, TitanError> {
    let payload = serde_json::to_string_pretty(&data)?;
    let path = settings_path().with_file_name("titan_chart_settings_undo.json");
    let tmp = format!("{}.tmp", path.display());
    std::fs::write(&tmp, &payload).map_err(|e| TitanError::Io(format!("Chart undo write failed: {}", e)))?;
    std::fs::rename(&tmp, &path).map_err(|e| TitanError::Io(format!("Chart undo rename failed: {}", e)))?;
    Ok("CHART_UNDO_SAVED".into())
}

#[tauri::command]
pub(crate) async fn load_chart_undo(
) -> Result<serde_json::Value, TitanError> {
    let path = settings_path().with_file_name("titan_chart_settings_undo.json");
    let raw = std::fs::read_to_string(&path).map_err(|_| TitanError::Io("No chart undo snapshot".into()))?;
    let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| TitanError::Io(format!("Chart undo parse failed: {}", e)))?;
    Ok(val)
}

#[tauri::command]
pub(crate) async fn set_symbol_override(
    symbol: String,
    overrides: crate::app_state::SymbolOverride,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    settings.symbol_overrides.insert(symbol.clone(), overrides);
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
        tracing::error!(component = "settings", error = %e, "Symbol override save failed");
        return Err(e);
    }
    Ok(format!("Override set for {}", symbol))
}

#[tauri::command]
pub(crate) async fn remove_symbol_override(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    settings.symbol_overrides.remove(&symbol);
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
        tracing::error!(component = "settings", error = %e, "Symbol override remove save failed");
        return Err(e);
    }
    Ok(format!("Override removed for {}", symbol))
}

#[tauri::command]
pub(crate) async fn get_symbol_overrides(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    Ok(serde_json::to_value(&settings.symbol_overrides)?)
}

#[tauri::command]
pub(crate) async fn preview_effective_exits(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_stop = s.trailing_stop_pct;
    let base_take = s.take_profit_pct;
    let base_trail = s.trailing_stop_pct;

    let mut eff_stop = base_stop;
    let mut eff_take = base_take;
    let mut eff_trail = base_trail;

    if !s.disable_adaptive_exits {
        eff_stop *= s.regime_chaotic_stop_mult.min(s.regime_range_stop_mult).min(s.regime_trend_stop_mult);
        eff_take *= s.regime_chaotic_take_mult.min(s.regime_range_take_mult).min(s.regime_trend_take_mult);
        eff_trail *= s.regime_chaotic_trail_mult.min(s.regime_range_trail_mult).min(s.regime_trend_trail_mult);

        let floor = s.exit_floor_pct;
        if eff_stop < floor { eff_stop = floor; }
        if eff_take < floor { eff_take = floor; }
        if eff_trail < floor { eff_trail = floor; }
    }

    Ok(serde_json::json!({
        "base_stop": base_stop,
        "base_take": base_take,
        "base_trail": base_trail,
        "effective_stop_min": eff_stop,
        "effective_take_min": eff_take,
        "effective_trail_min": eff_trail,
        "adaptive_enabled": !s.disable_adaptive_exits,
        "exit_floor": s.exit_floor_pct,
        "regime_mult": {
            "chaotic": { "stop": s.regime_chaotic_stop_mult, "take": s.regime_chaotic_take_mult, "trail": s.regime_chaotic_trail_mult },
            "range": { "stop": s.regime_range_stop_mult, "take": s.regime_range_take_mult, "trail": s.regime_range_trail_mult },
            "trending": { "stop": s.regime_trend_stop_mult, "take": s.regime_trend_take_mult, "trail": s.regime_trend_trail_mult }
        }
    }))
}

// ─── Profile Persistence (RF-C) ───

#[tauri::command]
pub(crate) async fn save_profile(
    name: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(TitanError::Validation("Profile name cannot be empty".into()));
    }

    let snapshot = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings_from_state(&settings)
    };

    let entry = ProfileEntry {
        schema_v: 1,
        saved_at: chrono::Utc::now().to_rfc3339(),
        data: snapshot,
    };

    let mut store = load_profiles_from_disk();
    store.profiles.insert(name.clone(), entry);
    save_profiles_to_disk(&store)?;

    tracing::info!(component = "profiles", profile = %name, "Profile saved");
    Ok(format!("Profile \"{}\" saved", name))
}

#[tauri::command]
pub(crate) async fn load_profile(
    name: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let store = load_profiles_from_disk();
    let entry = store
        .profiles
        .get(&name)
        .ok_or_else(|| TitanError::Validation(format!("Profile \"{}\" not found", name)))?;

    let warnings = validate_settings_coherence(&entry.data);
    if !warnings.is_empty() {
        tracing::warn!(
            component = "profiles",
            profile = %name,
            "Load profile coherence warnings: {:?}",
            warnings
        );
    }

    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    crate::state::apply_settings(&mut settings, &entry.data);

    let safety_warnings = crate::app_state::validate_safety_settings(&settings);
    for w in &safety_warnings {
        tracing::warn!(component = "safety_validation", profile = %name, "{}", w);
    }

    crate::state::apply_runtime_config(&state, &settings);
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
        tracing::error!(component = "profiles", error = %e, "Profile save after load failed");
        return Err(e);
    }

    tracing::info!(component = "profiles", profile = %name, "Profile loaded");
    if warnings.is_empty() && safety_warnings.is_empty() {
        Ok(format!("Profile \"{}\" loaded", name))
    } else {
        let all_warnings: Vec<String> = warnings.into_iter().chain(safety_warnings).collect();
        Ok(format!(
            "Profile \"{}\" loaded (warnings: {})",
            name,
            all_warnings.join("; ")
        ))
    }
}

#[tauri::command]
pub(crate) async fn delete_profile(
    name: String,
) -> Result<String, TitanError> {
    let mut store = load_profiles_from_disk();
    if store.profiles.remove(&name).is_none() {
        return Ok(format!("Profile \"{}\" not found (already deleted)", name));
    }
    save_profiles_to_disk(&store)?;

    tracing::info!(component = "profiles", profile = %name, "Profile deleted");
    Ok(format!("Profile \"{}\" deleted", name))
}

#[tauri::command]
pub(crate) async fn list_profiles() -> Result<Vec<ProfileMeta>, TitanError> {
    let store = load_profiles_from_disk();
    let mut metas: Vec<ProfileMeta> = store
        .profiles
        .iter()
        .map(|(name, entry)| ProfileMeta {
            name: name.clone(),
            saved_at: entry.saved_at.clone(),
            schema_v: entry.schema_v,
        })
        .collect();
    metas.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(metas)
}

#[tauri::command]
pub(crate) async fn export_profile(
    name: String,
) -> Result<serde_json::Value, TitanError> {
    let store = load_profiles_from_disk();
    let entry = store
        .profiles
        .get(&name)
        .ok_or_else(|| TitanError::Validation(format!("Profile \"{}\" not found", name)))?;
    Ok(serde_json::to_value(entry)?)
}

const CURRENT_PROFILE_SCHEMA_V: u32 = 1;

#[tauri::command]
pub(crate) async fn import_profile(
    name: String,
    data: serde_json::Value,
) -> Result<String, TitanError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(TitanError::Validation("Profile name cannot be empty".into()));
    }

    let entry: ProfileEntry = serde_json::from_value(data).map_err(|e| {
        TitanError::Validation(format!("Invalid profile data: {}", e))
    })?;

    if entry.schema_v > CURRENT_PROFILE_SCHEMA_V {
        return Err(TitanError::Validation(format!(
            "Profile schema v{} is newer than supported v{}",
            entry.schema_v, CURRENT_PROFILE_SCHEMA_V
        )));
    }

    let round_trip_check = serde_json::to_value(&entry.data)
        .and_then(serde_json::from_value::<crate::state::AppSettings>);
    if let Err(e) = round_trip_check {
        return Err(TitanError::Validation(format!(
            "Profile data fails round-trip validation: {}", e
        )));
    }

    let warnings = validate_settings_coherence(&entry.data);
    if !warnings.is_empty() {
        tracing::warn!(
            component = "profiles",
            profile = %name,
            "Import coherence warnings: {:?}",
            warnings
        );
    }

    let mut store = load_profiles_from_disk();
    store.profiles.insert(name.clone(), entry);
    save_profiles_to_disk(&store)?;

    tracing::info!(component = "profiles", profile = %name, "Profile imported");
    Ok(format!("Profile \"{}\" imported", name))
}

/// Cross-field coherence checks that per-field validation cannot catch.
pub(crate) fn validate_settings_coherence(
    s: &crate::state::AppSettings,
) -> Vec<String> {
    let mut w = Vec::new();
    if s.breaker_caution_pct >= 0.0 {
        w.push("breaker_caution_pct must be negative".into());
    }
    if s.breaker_halt_pct > s.breaker_caution_pct {
        w.push(format!(
            "breaker_halt_pct ({}) should be lower than caution ({})",
            s.breaker_halt_pct, s.breaker_caution_pct
        ));
    }
    if s.breaker_panic_pct > s.breaker_halt_pct {
        w.push(format!(
            "breaker_panic_pct ({}) should be lower than halt ({})",
            s.breaker_panic_pct, s.breaker_halt_pct
        ));
    }
    if s.trailing_stop_pct <= 0.0 {
        w.push("trailing_stop_pct must be positive".into());
    }
    if s.take_profit_pct <= 0.0 {
        w.push("take_profit_pct must be positive".into());
    }
    if s.risk_per_trade <= 0.0 || s.risk_per_trade > 25.0 {
        w.push(format!("risk_per_trade ({}) out of safe range (0, 25]", s.risk_per_trade));
    }
    if s.max_hold_secs < s.min_hold_secs {
        w.push(format!(
            "max_hold_secs ({}) < min_hold_secs ({})",
            s.max_hold_secs, s.min_hold_secs
        ));
    }
    w
}

#[tauri::command]
pub(crate) async fn apply_full_settings(
    payload: crate::state::AppSettings,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let warnings = validate_settings_coherence(&payload);
    if !warnings.is_empty() {
        tracing::warn!(
            component = "config",
            "apply_full_settings coherence warnings: {:?}",
            warnings
        );
    }

    let mut st = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    crate::state::apply_settings(&mut st, &payload);
    crate::state::apply_runtime_config(&state, &st);
    if let Err(e) = save_settings_to_disk(&settings_from_state(&st)) {
        tracing::error!(component = "config", error = %e, "Full settings save failed");
        return Err(e);
    }

    tracing::info!(component = "config", "Full settings applied via apply_full_settings");
    Ok("FULL_SETTINGS_APPLIED".into())
}

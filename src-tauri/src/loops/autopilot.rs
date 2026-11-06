use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::app_state::{AppState, Candle, OrderSide};
use crate::candle_buffer::timeframe_to_secs;
use crate::commands::data::{
    close_position, compute_gpu_scores, compute_strategy_score_with_mode, fetch_bars_internal,
    fetch_positions, get_signal_markers_internal, has_pending_orders_for_symbol, is_market_open,
    liquidate_all_internal, send_discord_alert, send_order_internal,
};
use crate::error::TitanError;
use crate::state::{
    cache_key_symbol_tf, normalize_symbol, normalize_timeframe_internal, StrategyMode, TitanState,
};
use crate::strategy::TradeDecision;
#[cfg(feature = "telegram")]
use crate::telegram;
use crate::types::{AlpacaPosition, LegacyOrder, StrategyScore};
use crate::journal::{TradeFeatures, TradeRecordInput};
use crate::{correlation, indicators, risk};

// ═══════════════════════════════════════════════════════════════
//  PARALLEL EVALUATION STRUCTS (S5+S6)
//  Read-only data gathered in parallel; side effects applied sequentially.
// ═══════════════════════════════════════════════════════════════

#[derive(Debug)]
struct ExitEvaluation {
    symbol: String,
    qty: f64,
    is_short: bool,
    current_price: f64,
    entry_price: f64,
    pnl_pct: f64,
    stop_loss: f64,
    take_profit: f64,
    trail_pct: f64,
    force_exit: bool,
    force_exit_reason: String,
    partial_close: Option<PartialCloseData>,
    hold_secs: u64,
    cand_tf: String,
}

#[derive(Debug)]
struct PartialCloseData {
    partial_qty: f64,
    partial_side: String,
}

#[derive(Debug)]
struct EntryEvaluation {
    symbol: String,
    decision: Option<TradeDecision>,
    cand_tf: String,
    vol_factor: f64,
}

/// Gather read-only exit data for a single position.
/// All state reads are short-lived lock snapshots; no side effects.
#[allow(clippy::too_many_arguments)]
async fn evaluate_position_exit(
    state: &Arc<TitanState>,
    p: &AlpacaPosition,
    settings: &AppState,
    strategy: &Option<StrategyScore>,
    strategy_mode: StrategyMode,
    tf: &str,
    atr_stop_mult: f64,
    atr_take_mult: f64,
    evo_sl: f64,
    evo_tp: f64,
    evo_trail: f64,
    eff_aggressive: bool,
    partial_enabled: bool,
    partial_pct: f64,
) -> Option<ExitEvaluation> {
    let qty = p.qty.parse::<f64>().unwrap_or(0.0).abs();
    if qty <= 0.0 {
        return None;
    }

    {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        if s.position_trade_modes
            .get(&p.symbol)
            .map(|m| m == "manual")
            .unwrap_or(false)
        {
            return None;
        }
    }

    {
        let inflight = state.trading.exit_inflight.lock().unwrap_or_else(|e| e.into_inner());
        if inflight.contains_key(&p.symbol) {
            return None;
        }
    }

    let current_price = p
        .current_price
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let entry_price = p
        .avg_entry_price
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    if current_price <= 0.0 || entry_price <= 0.0 {
        return None;
    }

    let sym_key = normalize_symbol(&p.symbol);
    let vol_fallback = {
        let cache = state.market.vol_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.values.get(&sym_key).map(|e| e.value).unwrap_or(0.0)
    };
    let vol_factor = (vol_fallback / 1.0).clamp(settings.vol_factor_min, settings.vol_factor_max);
    let base_stop_fallback = if eff_aggressive {
        (evo_sl * 0.52).max(0.5)
    } else {
        (evo_sl * 0.32).max(0.3)
    } * vol_factor;
    let base_take_fallback = if eff_aggressive {
        (evo_tp * 1.68).max(1.0)
    } else {
        (evo_tp * 1.04).max(0.8)
    } * (vol_factor * 1.2).clamp(0.8, 2.8);
    let base_trail_fallback = if eff_aggressive {
        (evo_trail * 0.67).max(0.3)
    } else {
        (evo_trail * 0.40).max(0.2)
    } * vol_factor;

    let pnl_pct = ((current_price - entry_price) / entry_price) * 100.0;
    let is_short = p.side.eq_ignore_ascii_case("short");

    let cand_tf = {
        let tf_map = state
            .scan
            .symbol_best_tf
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        tf_map
            .get(&sym_key)
            .cloned()
            .unwrap_or_else(|| normalize_timeframe_internal(tf))
    };
    let (stop_loss, take_profit_val, trail_pct_val) = if let Ok(candles) =
        get_candles(state, &p.symbol, &cand_tf, 1).await
    {
        if let Some(atr) = indicators::calculate_atr(&candles, settings.atr_period as usize) {
            if atr > 0.0 && current_price > 0.0 {
                let atr_pct = (atr / current_price) * 100.0;
                (
                    (atr_pct * atr_stop_mult).clamp(settings.atr_stop_clamp_min, settings.atr_stop_clamp_max),
                    (atr_pct * atr_take_mult).clamp(settings.atr_take_clamp_min, settings.atr_take_clamp_max),
                    (atr_pct * 1.0).clamp(settings.atr_trail_clamp_min, settings.atr_trail_clamp_max),
                )
            } else {
                (base_stop_fallback, base_take_fallback, base_trail_fallback)
            }
        } else {
            (base_stop_fallback, base_take_fallback, base_trail_fallback)
        }
    } else {
        (base_stop_fallback, base_take_fallback, base_trail_fallback)
    };

    let mut local_stop = stop_loss;
    let mut local_take = take_profit_val;
    let mut local_trail = trail_pct_val;

    if let Some(ovr) = settings.symbol_overrides.get(&p.symbol) {
        if let Some(v) = ovr.trailing_stop_pct { local_trail = v; }
        if let Some(v) = ovr.take_profit_pct { local_take = v; }
    }

    let mut force_exit = false;
    let mut force_exit_reason = String::new();

    let hold_secs = {
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let fsm_opened = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            s.positions.iter().find(|pp| pp.symbol == p.symbol).and_then(|pp| pp.opened_at)
        };
        let entry_ts = fsm_opened.unwrap_or_else(|| {
            state
                .trading
                .entry_times
                .lock()
                .ok()
                .and_then(|et| et.get(&p.symbol).copied())
                .unwrap_or(0)
        });
        if entry_ts > 0 {
            now_ts.saturating_sub(entry_ts)
        } else {
            0
        }
    };
    let min_hold_secs: u64 = settings.min_hold_secs;

    // SIGNAL STRATEGY EXIT
    if matches!(strategy_mode, StrategyMode::SignalStrategy) && hold_secs >= min_hold_secs {
        let sym_tf = {
            let tf_map = state
                .scan
                .symbol_best_tf
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            tf_map
                .get(&sym_key)
                .cloned()
                .unwrap_or_else(|| normalize_timeframe_internal(tf))
        };
        if let Ok(resp) = get_signal_markers_internal(state, &p.symbol, &sym_tf).await {
            let markers = resp.markers;
            if let Some(last) = markers.last() {
                let candle_secs: i64 = match sym_tf.as_str() {
                    "1m" => 60, "5m" => 300, "10m" => 600, "15m" => 900,
                    "30m" => 1800, "1h" => 3600, "4h" => 14400, "1d" => 86400, _ => 300,
                };
                let now_ts = chrono::Utc::now().timestamp();
                let is_fresh = (now_ts - last.time) <= candle_secs * 2;
                if is_fresh {
                    let is_exit_signal =
                        last.text == "TRAIL" || last.text == "EXIT" || last.text == "CHAOS";
                    let is_opposing = if !is_short {
                        last.text.contains("SELL") || last.text.contains("SHORT")
                    } else {
                        last.text.contains("BUY")
                    };
                    if is_exit_signal || is_opposing {
                        force_exit = true;
                        force_exit_reason = format!("SIGNAL EXIT: '{}'", last.text);
                    }
                }
            }
        }
    }

    // ORACLE / CHAOS / OPPOSING signal exits
    if let Some(sig) = strategy {
        let align_long = !is_short && sig.signal == "LONG";
        let align_short = is_short && sig.signal == "SHORT";
        let oppose = (!is_short && sig.signal == "SHORT") || (is_short && sig.signal == "LONG");

        if hold_secs >= min_hold_secs
            && oppose
            && sig.oracle_confidence > settings.oracle_exit_min_confidence
            && sig.oracle_score.abs() > settings.oracle_exit_min_score
        {
            force_exit = true;
            force_exit_reason = format!(
                "OPPOSING SIGNAL (Oracle: {:.1}, Conf: {:.2})",
                sig.oracle_score, sig.oracle_confidence
            );
        }

        if !force_exit && hold_secs >= min_hold_secs {
            let oracle_flip = if !is_short {
                sig.oracle_score < -settings.oracle_flip_score_threshold && sig.oracle_confidence > settings.oracle_flip_confidence
            } else {
                sig.oracle_score > settings.oracle_flip_score_threshold && sig.oracle_confidence > settings.oracle_flip_confidence
            };
            if oracle_flip {
                force_exit = true;
                force_exit_reason = format!(
                    "ORACLE FLIP (Oracle: {:.1}, Conf: {:.2})",
                    sig.oracle_score, sig.oracle_confidence
                );
            }
        }

        if !force_exit
            && hold_secs >= min_hold_secs
            && (sig.oracle_regime == "Chaotic" || sig.trend == "CHAOTIC")
        {
            let effective = if is_short {
                ((entry_price - current_price) / entry_price) * 100.0
            } else {
                pnl_pct
            };
            if effective < settings.chaos_exit_max_loss_pct && sig.oracle_confidence > settings.chaos_exit_min_confidence {
                force_exit = true;
                force_exit_reason = format!(
                    "CHAOS EXIT (PnL: {:.2}%, Conf: {:.2})",
                    effective, sig.oracle_confidence
                );
            }
        }

        if !settings.disable_adaptive_exits {
            if (align_long || align_short) && sig.score >= settings.nexus_consensus_threshold {
                local_stop *= settings.signal_aligned_stop_mult;
                local_take *= settings.signal_aligned_take_mult;
                local_trail *= settings.signal_aligned_trail_mult;
            } else if oppose && sig.score >= 60.0 {
                local_stop *= settings.signal_oppose_stop_mult;
                local_take *= settings.signal_oppose_take_mult;
                local_trail *= settings.signal_oppose_stop_mult;
            } else if sig.score < 40.0 {
                local_stop *= 0.9;
                local_take *= 0.9;
            }

            let regime_str = &sig.oracle_regime;
            if regime_str == "Chaotic" || sig.trend == "CHAOTIC" {
                local_stop *= settings.regime_chaotic_stop_mult;
                local_take *= settings.regime_chaotic_take_mult;
                local_trail *= settings.regime_chaotic_trail_mult;
            } else if sig.trend == "RANGE" || sig.adx < 18.0 {
                local_stop *= settings.regime_range_stop_mult;
                local_take *= settings.regime_range_take_mult;
                local_trail *= settings.regime_range_trail_mult;
            } else if (sig.trend == "UPTREND" && align_long && sig.adx >= 25.0)
                || (sig.trend == "DOWNTREND" && align_short && sig.adx >= 25.0)
            {
                local_stop *= settings.regime_trend_stop_mult;
                local_take *= settings.regime_trend_take_mult;
                local_trail *= settings.regime_trend_trail_mult;
            }

            if sig.gpu_boost >= 2.0 && (align_long || align_short) {
                local_stop *= 1.05;
                local_take *= 1.2;
            } else if sig.gpu_boost <= -2.0 {
                local_stop *= 0.9;
                local_take *= 0.9;
            }

            if tf == "1m" || tf == "3m" {
                local_stop *= settings.tf_short_stop_mult;
                local_take *= settings.tf_short_take_mult;
                local_trail *= settings.tf_short_trail_mult;
            } else if tf == "1h" || tf == "4h" || tf == "1d" {
                local_stop *= 1.15;
                local_take *= 1.25;
                local_trail *= 1.1;
            }
        }
    }

    // Max-Hold Decay (S15): progressive trail tightening as position ages
    if !force_exit && settings.max_hold_decay_enabled && settings.max_hold_secs > 0 {
        let max_hold = settings.max_hold_secs as f64;
        let decay_start = max_hold * settings.max_hold_decay_start_pct;
        let elapsed = hold_secs as f64;
        if elapsed >= max_hold {
            force_exit = true;
            force_exit_reason = format!(
                "MAX_HOLD_DECAY force exit (held {}s >= max {}s)",
                hold_secs, settings.max_hold_secs
            );
            tracing::info!(
                component = "autopilot",
                symbol = %p.symbol,
                hold_secs,
                max_hold_secs = settings.max_hold_secs,
                "[MAX_HOLD_DECAY] Force exit"
            );
        } else if elapsed > decay_start {
            let progress = ((elapsed - decay_start) / (max_hold - decay_start)).clamp(0.0, 1.0);
            let tighten_factor = 1.0 - progress * settings.max_hold_decay_trail_tighten;
            let tighten_factor = tighten_factor.clamp(0.2, 1.0);
            local_trail *= tighten_factor;
            if rand::random::<f64>() < 0.01 {
                tracing::info!(
                    component = "autopilot",
                    symbol = %p.symbol,
                    hold_secs,
                    progress_pct = format_args!("{:.0}", progress * 100.0),
                    trail_tightened = format_args!("{:.4}", local_trail),
                    "[MAX_HOLD_DECAY] Trail tightened"
                );
            }
        }
    }

    // Scalping adjustment
    {
        let is_scalping = match strategy_mode {
            StrategyMode::MathOracle => settings.scalping_oracle,
            StrategyMode::GpuNexus => settings.scalping_gpu,
            StrategyMode::Fusion | StrategyMode::SignalStrategy | StrategyMode::SmartTrend => settings.scalping_fusion,
        };
        if is_scalping {
            local_stop *= settings.scalping_stop_mult;
            local_take *= settings.scalping_take_mult;
            local_trail *= settings.scalping_trail_mult;
        }
    }

    if !settings.disable_adaptive_exits && settings.exit_floor_pct > 0.0 {
        let floor = settings.exit_floor_pct;
        if local_stop.abs() < floor {
            local_stop = -floor;
        }
        if local_take < floor {
            local_take = floor;
        }
        if local_trail < floor * 0.5 {
            local_trail = floor * 0.5;
        }
    }

    let effective_pnl = if is_short {
        ((entry_price - current_price) / entry_price) * 100.0
    } else {
        pnl_pct
    };

    // Partial profit check (read-only)
    let already_partial = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.positions.iter().find(|pp| pp.symbol == p.symbol).map(|pp| pp.partial_taken).unwrap_or(false)
    };
    let partial_target = local_take * settings.partial_first_target_frac;
    let partial_close = if partial_enabled && !already_partial && effective_pnl >= partial_target && qty > settings.partial_min_qty * 2.0 {
        let partial_qty = (qty * partial_pct / 100.0).max(settings.partial_min_qty);
        let partial_side = if is_short { "buy" } else { "sell" };
        Some(PartialCloseData {
            partial_qty,
            partial_side: partial_side.to_string(),
        })
    } else {
        None
    };

    Some(ExitEvaluation {
        symbol: p.symbol.clone(),
        qty,
        is_short,
        current_price,
        entry_price,
        pnl_pct,
        stop_loss: local_stop,
        take_profit: local_take,
        trail_pct: local_trail,
        force_exit,
        force_exit_reason,
        partial_close,
        hold_secs,
        cand_tf,
    })
}

/// Gather read-only entry data for a single candidate symbol.
/// After strategy scoring, applies microstructure score modifiers:
/// spread narrowing bonus, NBBO imbalance alignment, dual VPIN divergence,
/// and stream VPIN toxicity penalty.
async fn evaluate_candidate_entry(
    state: &Arc<TitanState>,
    candidate_symbol: &str,
    settings: &AppState,
    cand_tf: &str,
) -> EntryEvaluation {
    let mut decision = if let Ok(candles) =
        get_candles(state, candidate_symbol, cand_tf, settings.min_bars_for_entry as usize).await
    {
        if candles.len() >= settings.min_bars_for_entry as usize {
            let periods = crate::state::build_indicator_periods(settings);
            let base_w = crate::state::build_oracle_weights(settings);
            let oracle_w = if settings.enable_oracle_adaptation {
                let prior_regime = {
                    let cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
                    cache.peek(candidate_symbol, cand_tf)
                        .and_then(|ind| ind.oracle_regime)
                        .unwrap_or(crate::titan_oracle::MarketRegime::Unknown)
                };
                let adapt = state.oracle_adaptation.lock().unwrap_or_else(|e| e.into_inner());
                adapt.adapt_weights(&base_w, &prior_regime)
            } else {
                base_w
            };
            let indicators = {
                let mut cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
                cache.get_or_compute(candidate_symbol, cand_tf, &candles, &periods, &oracle_w)
            };
            if settings.enable_oracle_adaptation {
                let snap = crate::titan_oracle::calculate(&candles, &oracle_w);
                if let Ok(mut adapt) = state.oracle_adaptation.lock() {
                    adapt.capture_entry_signals(candidate_symbol, &snap);
                }
            }
            crate::strategy::check_entry(candidate_symbol, &candles, &indicators, settings)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref mut d) = decision {
        let micro = state.market.ofi_state.load();
        let candle_vpin = {
            let cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.peek(candidate_symbol, cand_tf)
                .and_then(|ind| ind.vpin)
                .unwrap_or(0.5)
        };
        let micro_adj = apply_microstructure_modifier(
            &micro,
            candle_vpin,
            d.signal == crate::strategy::Signal::Buy,
        );
        d.score += micro_adj;

        // MTF Confirmation Gate (S15): penalize or block entries against HTF trend
        if settings.mtf_gate_enabled {
            let htf_tf = if settings.mtf_gate_htf.is_empty() {
                crate::strategy::ltf_to_htf(cand_tf).map(|s| s.to_string())
            } else {
                Some(crate::state::normalize_timeframe_internal(&settings.mtf_gate_htf))
            };
            if let Some(htf_str) = htf_tf {
                if let Ok(htf_candles) = get_candles(state, candidate_symbol, &htf_str, 60).await {
                    let base_w = crate::state::build_oracle_weights(settings);
                    let is_buy = d.signal == crate::strategy::Signal::Buy;
                    let (aligned, htf_score) =
                        crate::strategy::is_htf_aligned(&htf_candles, &base_w, is_buy);
                    if !aligned {
                        if settings.mtf_gate_mode == "hard" {
                            tracing::info!(
                                component = "autopilot",
                                symbol = candidate_symbol,
                                htf = %htf_str,
                                htf_score = format_args!("{:.1}", htf_score),
                                "[MTF_GATE] Hard block"
                            );
                            decision = None;
                        } else {
                            let penalty = settings.mtf_gate_penalty as i32;
                            d.score -= penalty;
                            tracing::info!(
                                component = "autopilot",
                                symbol = candidate_symbol,
                                htf = %htf_str,
                                htf_score = format_args!("{:.1}", htf_score),
                                penalty,
                                "[MTF_GATE] Soft penalty"
                            );
                        }
                    }
                }
            }
        }
    }

    let cand_vol_key = normalize_symbol(candidate_symbol);
    let vol_fallback = {
        let cache = state.market.vol_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.values.get(&cand_vol_key).map(|e| e.value).unwrap_or(0.0)
    };
    let vol_factor = (vol_fallback / 1.0).clamp(settings.vol_factor_min, settings.vol_factor_max);

    EntryEvaluation {
        symbol: candidate_symbol.to_string(),
        decision,
        cand_tf: cand_tf.to_string(),
        vol_factor,
    }
}

/// Compute microstructure-based score adjustment from live OfiState.
/// Returns an additive i32 modifier (positive = bonus, negative = penalty).
fn apply_microstructure_modifier(
    micro: &crate::l2_heatmap::OfiState,
    candle_vpin: f64,
    is_buy: bool,
) -> i32 {
    let mut adj: i32 = 0;

    if micro.spread_narrowing {
        adj += 5;
    }

    let imb_aligned = (is_buy && micro.imbalance_ratio > 0.3)
        || (!is_buy && micro.imbalance_ratio < -0.3);
    if imb_aligned {
        adj += 4;
    }

    let divergence = micro.stream_vpin - candle_vpin;
    if divergence > 0.15 {
        adj -= 8;
    } else if divergence < -0.15 {
        adj += 5;
    }

    if micro.stream_vpin > 0.85 {
        adj -= 10;
    }

    adj
}

/// Build a `TradeFeatures` snapshot from cached indicators and live OFI state.
/// Called immediately after a successful entry order so the journal captures
/// the market context at the exact moment of entry.
fn snapshot_entry_features(
    state: &Arc<TitanState>,
    symbol: &str,
    tf: &str,
    decision: &TradeDecision,
    _settings: &AppState,
) -> TradeFeatures {
    let ofi_snap = state.market.ofi_state.load();

    let indicators = {
        let cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.peek(symbol, tf)
    };

    let (oracle_score, oracle_confidence, oracle_regime, rsi, adx, ema9, ema50, vpin, hurst) =
        if let Some(ind) = &indicators {
            (
                ind.oracle_score.unwrap_or(0.0),
                ind.oracle_confidence.unwrap_or(0.0),
                ind.oracle_regime
                    .as_ref()
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_default(),
                ind.rsi.unwrap_or(0.0),
                ind.adx.unwrap_or(0.0),
                ind.ema_9.unwrap_or(0.0),
                ind.ema_50.unwrap_or(0.0),
                ind.vpin.unwrap_or(0.0),
                0.0,
            )
        } else {
            (0.0, 0.0, String::new(), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

    let rvol = {
        let vol_key = normalize_symbol(symbol);
        state
            .market
            .vol_cache
            .lock()
            .ok()
            .and_then(|c| c.values.get(&vol_key).map(|e| e.value))
            .unwrap_or(0.0)
    };

    let strategy_mode_str = state
        .strategy_mode
        .lock()
        .map(|m| m.to_string())
        .unwrap_or_else(|_| "UNKNOWN".into());

    let stream_vpin_val = ofi_snap.stream_vpin;
    let vpin_divergence = stream_vpin_val - vpin;

    TradeFeatures {
        oracle_score,
        oracle_confidence,
        oracle_regime,
        rsi,
        adx,
        atr_pct: 0.0,
        ema9,
        ema50,
        vpin: stream_vpin_val,
        hurst,
        rvol,
        ofi: ofi_snap.current_ofi,
        spread_bps: ofi_snap.spread_bps,
        imbalance_ratio: ofi_snap.imbalance_ratio,
        strategy_score: decision.score,
        signal_source: strategy_mode_str,
        fvg_type: format!("{:?}", decision.fvg),
        entry_type: decision.entry_type.clone(),
        ta_gates_passed: 0,
        ta_gates_total: 0,
        vpin_divergence,
        spread_narrowing: ofi_snap.spread_narrowing,
    }
}

pub(crate) fn snapshot_exit_features(
    state: &Arc<TitanState>,
    symbol: &str,
    tf: &str,
) -> TradeFeatures {
    let ofi_snap = state.market.ofi_state.load();

    let indicators = {
        let cached = {
            let cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.peek(symbol, tf)
        };
        if cached.is_some() {
            cached
        } else {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let periods = crate::state::build_indicator_periods(&settings);
            let oracle_weights = crate::state::build_oracle_weights(&settings);
            let candles = {
                let bufs = state.market.candle_buffers.lock().unwrap_or_else(|e| e.into_inner());
                let key = crate::state::cache_key_symbol_tf(symbol, tf);
                bufs.get(&key).map(|b| b.snapshot()).unwrap_or_default()
            };
            if !candles.is_empty() {
                let mut cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
                Some(cache.get_or_compute(symbol, tf, &candles, &periods, &oracle_weights))
            } else {
                None
            }
        }
    };

    let (oracle_score, oracle_confidence, oracle_regime, rsi, adx, ema9, ema50, vpin, hurst) =
        if let Some(ind) = &indicators {
            (
                ind.oracle_score.unwrap_or(0.0),
                ind.oracle_confidence.unwrap_or(0.0),
                ind.oracle_regime
                    .as_ref()
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_default(),
                ind.rsi.unwrap_or(0.0),
                ind.adx.unwrap_or(0.0),
                ind.ema_9.unwrap_or(0.0),
                ind.ema_50.unwrap_or(0.0),
                ind.vpin.unwrap_or(0.0),
                0.0,
            )
        } else {
            (0.0, 0.0, String::new(), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

    let rvol = {
        let vol_key = normalize_symbol(symbol);
        state
            .market
            .vol_cache
            .lock()
            .ok()
            .and_then(|c| c.values.get(&vol_key).map(|e| e.value))
            .unwrap_or(0.0)
    };

    let stream_vpin_val = ofi_snap.stream_vpin;
    let vpin_divergence = stream_vpin_val - vpin;

    TradeFeatures {
        oracle_score,
        oracle_confidence,
        oracle_regime,
        rsi,
        adx,
        atr_pct: 0.0,
        ema9,
        ema50,
        vpin: stream_vpin_val,
        hurst,
        rvol,
        ofi: ofi_snap.current_ofi,
        spread_bps: ofi_snap.spread_bps,
        imbalance_ratio: ofi_snap.imbalance_ratio,
        strategy_score: 0,
        signal_source: "EXIT".to_string(),
        fvg_type: String::new(),
        entry_type: "EXIT".to_string(),
        ta_gates_passed: 0,
        ta_gates_total: 0,
        vpin_divergence,
        spread_narrowing: ofi_snap.spread_narrowing,
    }
}

/// Try the in-memory candle ring buffer first; fall back to REST on miss.
/// On a REST fallback, the result is also seeded into the ring buffer so
/// subsequent calls within the same candle interval hit the fast path.
async fn get_candles(
    state: &Arc<TitanState>,
    symbol: &str,
    timeframe: &str,
    min_candles: usize,
) -> Result<Vec<Candle>, TitanError> {
    let buf_key = cache_key_symbol_tf(symbol, timeframe);
    let norm_tf = normalize_timeframe_internal(timeframe);

    {
        let bufs = state.market.candle_buffers.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(buf) = bufs.get(&buf_key) {
            if buf.available() >= min_candles {
                return Ok(buf.snapshot());
            }
        }
    }

    let candles = fetch_bars_internal(state.clone(), symbol, &norm_tf).await?;

    {
        let tf_s = timeframe_to_secs(&norm_tf);
        let mut bufs = state.market.candle_buffers.lock().unwrap_or_else(|e| e.into_inner());
        let buf = bufs
            .entry(buf_key)
            .or_insert_with(|| crate::candle_buffer::CandleRingBuffer::new(tf_s));
        if buf.len() == 0 {
            buf.seed(candles.clone());
        }
    }

    Ok(candles)
}

// Reads from settings.nexus_consensus_threshold at runtime

/// Refresh positions from `AppState` (updated on fills) into Alpaca-shaped rows for this loop.
fn positions_snapshot_from_settings(state: &Arc<TitanState>) -> Vec<AlpacaPosition> {
    let app_positions = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.positions.clone()
    };
    app_positions
        .into_iter()
        .map(|p| {
            let is_short = matches!(p.side, OrderSide::Sell);
            let market_val = p.current_price * p.qty.abs();
            AlpacaPosition {
                symbol: p.symbol,
                qty: format!("{}", p.qty),
                side: if is_short {
                    "short".to_string()
                } else {
                    "long".to_string()
                },
                current_price: Some(p.current_price.to_string()),
                avg_entry_price: Some(p.entry_price.to_string()),
                market_value: format!("{}", market_val),
                unrealized_pl: format!("{}", p.pnl),
                unrealized_plpc: format!("{}", p.pnl_percent),
            }
        })
        .collect()
}

#[tracing::instrument(skip_all, name = "autopilot")]
pub(crate) async fn autopilot_loop(state: Arc<TitanState>, app: AppHandle) {
    let my_generation = state.current_generation();
    tracing::info!(
        component = "autopilot",
        generation = my_generation,
        "Autopilot loop started"
    );

    let mut last_liquidated_date = String::new();
    let mut last_reset_date = String::new();

    'autopilot: loop {
        use chrono::Timelike;
        let now_et = chrono::Utc::now().with_timezone(&chrono_tz::US::Eastern);
        let today_str = now_et.format("%Y-%m-%d").to_string();
        let hour = now_et.hour();
        let min = now_et.minute();

        // 1. Daily Reset at 04:00 ET
        if hour == 4 && min < 5 && last_reset_date != today_str {
            tracing::info!("DAILY RESET INITIATED (04:00 ET)");
            let _ = app.emit_all("log-message", "🔄 DAILY RESET: Fresh born for premarket.");
            {
                let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                s.hft_targets.clear(); // just in case
                let _ = crate::state::save_settings_to_disk(&crate::state::settings_from_state(&s));
            }
            {
                let mut acc = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                acc.realized_pnl_today = 0.0;
                acc.trade_count_today = 0;
                acc.win_count_today = 0;
                acc.loss_count_today = 0;
                acc.max_drawdown_pct = 0.0;
                let _ = acc.save_to_disk();
            }
            last_reset_date = today_str.clone();
        }

        // 2. Market Close Liquidation at 15:55 ET
        if hour == 15 && min >= 55 && last_liquidated_date != today_str {
            tracing::warn!("MARKET CLOSE LIQUIDATION INITIATED (15:55 ET)");
            let _ = app.emit_all("log-message", "⚠️ MARKET CLOSE: Liquidating all positions.");
            {
                let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                s.hft_targets.clear();
                let _ = crate::state::save_settings_to_disk(&crate::state::settings_from_state(&s));
            }

            let _ = crate::commands::data::liquidate_all_internal(state.clone(), &app).await;

            last_liquidated_date = today_str.clone();
        }
        let (settings, sleep_ms) = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let base = s.autopilot_loop_ms;
            let has_positions = !s.positions.is_empty();
            let adaptive_ms = if has_positions {
                (base / 2).max(500)
            } else if !s.auto_pilot {
                base.min(5000)
            } else {
                base
            };
            (s.clone(), adaptive_ms)
        };
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        if state.shutdown_token.is_cancelled() {
            tracing::info!(
                component = "autopilot",
                "Shutdown signal received — exiting loop"
            );
            break;
        }
        if state.current_generation() != my_generation {
            tracing::warn!(
                component = "autopilot",
                my_gen = my_generation,
                current_gen = state.current_generation(),
                "Generation mismatch — exiting stale autopilot instance"
            );
            state.generation_mismatch_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            break;
        }
        state.watchdog.heartbeat("autopilot", 0);

        // T2/S14: Pending order watchdog -- cancel stale or generation-mismatched orders
        {
            let timeout_secs = {
                let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                s.pending_order_timeout_secs
            };
            let current_gen = state.current_generation();
            let now_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let stale: Vec<crate::trading_state::PendingOrder> = {
                let pending = state.trading.pending_orders.lock()
                    .unwrap_or_else(|e| e.into_inner());
                pending.iter()
                    .filter(|po| {
                        po.generation_id < current_gen
                            || now_epoch.saturating_sub(po.submitted_at) > timeout_secs
                    })
                    .cloned()
                    .collect()
            };
            if !stale.is_empty() {
                let (key, secret, base_url) = {
                    let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
                    (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
                };
                for po in &stale {
                    let reason = if po.generation_id < current_gen {
                        "generation_mismatch"
                    } else {
                        "timeout"
                    };
                    tracing::warn!(
                        component = "pending_watchdog",
                        symbol = %po.symbol,
                        broker_id = %po.broker_id,
                        reason,
                        "Cancelling stale pending order"
                    );
                    if !po.broker_id.is_empty() && !po.broker_id.starts_with("DRY_") {
                        let cancel_url = format!(
                            "{}/v2/orders/{}",
                            base_url.trim_end_matches('/'),
                            po.broker_id
                        );
                        let _ = state.http_client
                            .delete(&cancel_url)
                            .header("APCA-API-KEY-ID", &key)
                            .header("APCA-API-SECRET-KEY", &secret)
                            .send()
                            .await;
                    }
                    let _ = app.emit_all("log-message", format!(
                        "PENDING WATCHDOG: cancelled {} {} [{}]",
                        po.symbol, po.side, reason
                    ));
                }
                let cancelled_ids: std::collections::HashSet<String> =
                    stale.iter().map(|po| po.broker_id.clone()).collect();
                let mut pending = state.trading.pending_orders.lock()
                    .unwrap_or_else(|e| e.into_inner());
                pending.retain(|po| !cancelled_ids.contains(&po.broker_id));
            }
        }

        if rand::random::<f64>() < 0.01 {
            let mut cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.evict_stale(std::time::Duration::from_secs(3600));
        }

        let tf = settings.manual_res.clone();

        if !settings.auto_pilot {
            continue;
        }

        if settings.force_manual_tf {
            continue;
        }

        // CIRCUIT BREAKER — evaluate risk every cycle
        // If kill_switch already active, skip re-evaluation to prevent PANIC spam loop
        if state
            .risk
            .circuit_breaker
            .kill_switch
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        let (breaker_level, panic_reason) = {
            let mut analytics = state
                .risk
                .session_analytics
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let now_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            analytics.record_dd_snapshot(now_epoch);
            let daily_pnl_pct = analytics.daily_pnl_pct();
            let max_dd = analytics.max_drawdown_pct;
            let level = state.risk.circuit_breaker.evaluate(daily_pnl_pct, -max_dd);
            let reason = if level == risk::BreakerLevel::Panic {
                if -max_dd <= Decimal::from(-8) {
                    format!("Max Drawdown {:.1}% exceeded -8% limit", max_dd)
                } else {
                    format!("Daily PnL {}% exceeded -8% limit", daily_pnl_pct)
                }
            } else {
                String::new()
            };
            (level, reason)
        };

        let dd_velocity_halt = {
            let analytics = state
                .risk
                .session_analytics
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            analytics.check_dd_velocity_halt(
                settings.dd_velocity_threshold,
                settings.dd_velocity_window_secs,
            )
        };
        if dd_velocity_halt
            && !matches!(
                breaker_level,
                risk::BreakerLevel::Halt | risk::BreakerLevel::Panic
            )
            && rand::random::<f64>() < 0.05
        {
            let velocity = state
                .risk
                .session_analytics
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .dd_velocity;
            let _ = app.emit_all(
                "log-message",
                format!(
                    "DD VELOCITY HALT — Drawdown speed {:.2}%/min exceeds {:.1}%/min threshold. Blocking new entries.",
                    velocity, settings.dd_velocity_threshold
                ),
            );
        }

        match breaker_level {
            risk::BreakerLevel::Panic => {
                let _ = app.emit_all(
                    "log-message",
                    format!(
                        "CIRCUIT BREAKER: PANIC — LIQUIDATING ALL ({})",
                        panic_reason
                    ),
                );
                let _ = app.emit_all(
                    "breaker-triggered",
                    serde_json::json!({
                        "level": "PANIC", "reason": &panic_reason
                    }),
                );
                let (webhook, alert_breaker, tg_token, tg_chat, tg_enabled) = {
                    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    (
                        s.discord_webhook_url.clone(),
                        s.alert_on_breaker,
                        s.telegram_bot_token.clone(),
                        s.telegram_chat_id.clone(),
                        s.telegram_enabled,
                    )
                };
                if alert_breaker && !webhook.is_empty() {
                    let msg = format!("PANIC LIQUIDATION - {}", panic_reason);
                    let st = state.clone();
                    tokio::spawn(async move {
                        send_discord_alert(&st, &webhook, "CIRCUIT BREAKER: PANIC", &msg, 0xFF0000)
                            .await;
                    });
                }
                #[cfg(feature = "telegram")]
                if tg_enabled {
                    let tg_msg = telegram::format_breaker_alert("PANIC", &panic_reason);
                    let hc = state.http_client.clone();
                    tokio::spawn(async move {
                        telegram::send_telegram_alert(&hc, &tg_token, &tg_chat, &tg_msg).await;
                    });
                }
                let _ = liquidate_all_internal(state.clone(), &app).await;
                continue;
            }
            risk::BreakerLevel::Halt => {
                if rand::random::<f64>() < 0.01 {
                    let daily_pnl = state
                        .risk
                        .session_analytics
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .daily_pnl_pct();
                    let _ = app.emit_all(
                        "log-message",
                        format!(
                            "CIRCUIT BREAKER: HALT — No new entries (daily: {}%)",
                            daily_pnl
                        ),
                    );
                }
            }
            risk::BreakerLevel::Caution => {
                if rand::random::<f64>() < 0.02 {
                    let daily_pnl = state
                        .risk
                        .session_analytics
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .daily_pnl_pct();
                    let _ = app.emit_all(
                        "log-message",
                        format!(
                            "CIRCUIT BREAKER: CAUTION — Sizes halved (daily: {}%)",
                            daily_pnl
                        ),
                    );
                }
            }
            risk::BreakerLevel::Normal => {}
        }

        let strategy_mode = {
            *state
                .strategy_mode
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };
        let fusion_alpha = {
            *state
                .nexus_fusion_alpha
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };

        // SPREAD GUARD — from live L2 stream (no REST call needed)
        let mut entries_allowed = !matches!(
            breaker_level,
            risk::BreakerLevel::Halt | risk::BreakerLevel::Panic
        );
        if matches!(strategy_mode, StrategyMode::SignalStrategy) && settings.bypass_daily_loss {
            entries_allowed = !matches!(breaker_level, risk::BreakerLevel::Panic);
        }
        if dd_velocity_halt {
            entries_allowed = false;
        }
        // FIX-8: Stream stale soft gate -- skip new entries when market data is stale
        if entries_allowed && state.market.is_stream_stale() {
            entries_allowed = false;
            if rand::random::<f64>() < 0.01 {
                tracing::warn!(component = "autopilot", "[STALE_DATA] Entries blocked -- stream data stale (reconnecting)");
            }
        }
        // SmartTrend kill switch: block all entries when killed (S08)
        if matches!(strategy_mode, StrategyMode::SmartTrend) {
            let st_killed = state
                .smarttrend_engine
                .lock()
                .map(|e| e.config.st_killed)
                .unwrap_or(true);
            if st_killed {
                tracing::warn!("AUTOPILOT: SmartTrend KILLED — skipping all entries");
                entries_allowed = false;
            }
        }
        let max_spread_pct = settings.max_spread_bps / 100.0;
        let (spread_ok, live_ofi, ofi_stale) = {
            let ofi = state.market.ofi_state.load();
            let stale = ofi.is_stale(settings.ofi_stale_ms.into());
            let no_data = ofi.spread_bps <= 0.0 && ofi.best_bid <= 0.0;
            let safe = no_data || ofi.spread_is_safe(max_spread_pct);
            let spread_ok = safe && !stale;
            if !safe && !no_data && rand::random::<f64>() < 0.05 {
                let _ = app.emit_all(
                    "log-message",
                    format!(
                        "SPREAD GUARD: {:.1} bps (limit: {:.0} bps) — too wide, skipping",
                        ofi.spread_bps, settings.max_spread_bps
                    ),
                );
            }
            if ofi.update_count % 40 == 1 && !no_data {
                println!("[SPREAD DEBUG] bps={:.1} max_bps={:.0} max_pct={:.3} bid={:.2} ask={:.2} safe={} stale={}",
                    ofi.spread_bps, settings.max_spread_bps, max_spread_pct, ofi.best_bid, ofi.best_ask, safe, stale);
            }
            (spread_ok, ofi.current_ofi, stale)
        };

        let spread_ok_for_entry = if matches!(strategy_mode, StrategyMode::SignalStrategy)
            && settings.bypass_max_spread
        {
            true
        } else {
            spread_ok
        };

        let strategy = {
            let symbol = settings.active_symbol.clone();
            let tf = tf.clone();
            if symbol.is_empty() {
                None
            } else if let Ok(candles) = get_candles(&state, &symbol, &tf, 1).await {
                let gpu_mom = if matches!(
                    strategy_mode,
                    StrategyMode::GpuNexus | StrategyMode::Fusion | StrategyMode::SignalStrategy | StrategyMode::SmartTrend
                ) {
                    compute_gpu_scores(state.clone(), &candles)
                        .await
                        .map(|(m, _)| m)
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let periods = crate::state::build_indicator_periods(&settings);
                let oracle_w = crate::state::build_oracle_weights(&settings);
                let score = compute_strategy_score_with_mode(
                    &candles,
                    strategy_mode,
                    fusion_alpha,
                    gpu_mom,
                    &periods,
                    &oracle_w,
                );
                Some(score)
            } else {
                None
            }
        };

        // ATR-based dynamic stops — per-symbol volatility-adaptive
        let sig_strat = matches!(strategy_mode, StrategyMode::SignalStrategy);
        let atr_stop_mult = if sig_strat && settings.bypass_atr_stop {
            100.0
        } else {
            settings.atr_stop_multiplier
        };
        let atr_take_mult = if sig_strat && settings.bypass_atr_take {
            100.0
        } else {
            settings.atr_take_multiplier
        };
        let partial_enabled =
            settings.partial_take_enabled && !(sig_strat && settings.bypass_partial_take);
        let partial_pct = if sig_strat && settings.bypass_partial_pct {
            100.0
        } else {
            settings.partial_take_pct
        };

        // Fallback: read evolved params for vol_cache-based baseline when ATR unavailable
        let evo_params = state
            .genetic_optimizer
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .current_params();
        let evo_sl = evo_params.sl_percent.to_f64().unwrap_or(settings.fallback_sl_pct);
        let evo_tp = evo_params.tp_percent.to_f64().unwrap_or(settings.fallback_tp_pct);
        let evo_trail = evo_params.trail_gap_pct.to_f64().unwrap_or(settings.fallback_trail_pct);
        let eff_aggressive = settings.aggressive_mode || (sig_strat && settings.bypass_aggressive);

        let mut positions = match fetch_positions(state.clone()).await {
            Ok(p) => p,
            Err(_) => continue,
        };

        // SG#8 Fix: Prevent autopilot from exiting SmartTrend positions, avoiding conflict with smarttrend_bot
        positions.retain(|p| {
            let p_loc = settings.positions.iter().find(|pp| pp.symbol == p.symbol);
            let tag = p_loc.and_then(|pp| pp.entry_tag.as_deref()).unwrap_or("");
            !tag.starts_with("ST|")
        });

        // ═══════════════════════════════════════════════════════════════
        //  S5+S6: PARALLEL EXIT EVALUATION
        //  Phase 1: Gather exit decisions for ALL positions concurrently
        //  Phase 2: Apply decisions sequentially (state mutations + API calls)
        // ═══════════════════════════════════════════════════════════════
        let exit_futures = positions.iter().map(|p| {
            let state_ref = &state;
            let settings_ref = &settings;
            let strategy_ref = &strategy;
            let tf_ref = &tf;
            evaluate_position_exit(
                state_ref, p, settings_ref, strategy_ref, strategy_mode,
                tf_ref, atr_stop_mult, atr_take_mult, evo_sl, evo_tp, evo_trail,
                eff_aggressive, partial_enabled, partial_pct,
            )
        });
        let exit_evals: Vec<Option<ExitEvaluation>> =
            futures::future::join_all(exit_futures).await;

        // Phase 2: Apply exit decisions sequentially
        for eval in exit_evals.into_iter().flatten() {
            // --- PARTIAL TAKE (side-effect: API call + state mutation) ---
            if let Some(ref partial) = eval.partial_close {
                {
                    let mut prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                    prices
                        .entry(normalize_symbol(&eval.symbol))
                        .or_insert(eval.current_price);
                }
                match close_position(state.clone(), &eval.symbol, partial.partial_qty, &partial.partial_side, &app)
                    .await
                {
                    Ok(()) => {
                        {
                            state
                                .trading
                                .partial_taken
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .insert(eval.symbol.clone(), true);
                        }
                        {
                            let mut settings_lock = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                            if let Some(pos) = settings_lock.positions.iter_mut().find(|pp| pp.symbol == eval.symbol) {
                                pos.partial_taken = true;
                                pos.state = crate::app_state::PositionState::PartialExit;
                            }
                        }
                        let partial_pnl = if eval.is_short {
                            (eval.entry_price - eval.current_price) * partial.partial_qty
                        } else {
                            (eval.current_price - eval.entry_price) * partial.partial_qty
                        };
                        let _ = app.emit_all(
                            "log-message",
                            format!(
                                "PARTIAL TAKE: {} {:.0}% of {} — PnL: ${:.2}",
                                partial.partial_side.to_uppercase(),
                                partial_pct,
                                eval.symbol,
                                partial_pnl
                            ),
                        );
                        if let Ok(dec_pnl) = Decimal::try_from(partial_pnl) {
                            if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                                analytics.record_trade_with_duration(dec_pnl, eval.hold_secs);
                            }
                            if settings.enable_oracle_adaptation {
                                let base_w = crate::state::build_oracle_weights(&settings);
                                if let Ok(mut adapt) = state.oracle_adaptation.lock() {
                                    adapt.record_partial_outcome(
                                        &eval.symbol,
                                        partial_pnl,
                                        &base_w,
                                    );
                                }
                            }
                            if let Ok(mut journal) = state.trade_journal.lock() {
                                let side_label = if eval.is_short { "SHORT" } else { "LONG" };
                                let features = state.trading.entry_features.lock().ok().and_then(|m| m.get(&eval.symbol).cloned());
                                journal.record_trade(&TradeRecordInput {
                                    symbol: &eval.symbol,
                                    side: side_label,
                                    qty: partial.partial_qty,
                                    entry_price: eval.entry_price,
                                    exit_price: eval.current_price,
                                    pnl: partial_pnl,
                                    strategy_mode: &format!("{:?}", strategy_mode),
                                    source: "AUTOPILOT",
                                    timeframe: &eval.cand_tf,
                                    duration_secs: eval.hold_secs,
                                    reason: if eval.force_exit && !eval.force_exit_reason.is_empty() {
                                        &eval.force_exit_reason
                                    } else {
                                        "PARTIAL_TAKE"
                                    },
                                    entry_features: features,
                                    exit_features: None,
                                    broker_order_id: None,
                                });
                            }
                            let _ = state.ws_broadcast.send(serde_json::json!({
                                "type": "trade",
                                "action": "PARTIAL_EXIT",
                                "symbol": &eval.symbol,
                                "side": partial.partial_side.to_uppercase(),
                                "pnl": partial_pnl,
                            }).to_string());
                        }
                    }
                    Err(e) => {
                        tracing::warn!(component = "autopilot", symbol = %eval.symbol, error = %e, "Partial take failed");
                    }
                }
                let _ = positions_snapshot_from_settings(&state);
                continue 'autopilot;
            }

            // --- FULL EXIT EVALUATION (trailing + stop/take + force) ---
            let (should_close, side) = if eval.force_exit {
                let s = if eval.is_short { "buy" } else { "sell" };
                let _ = app.emit_all(
                    "log-message",
                    format!(
                        "FORCE EXIT: {} {} — {}",
                        s.to_uppercase(),
                        eval.symbol,
                        eval.force_exit_reason
                    ),
                );
                (true, s)
            } else if !eval.is_short {
                let drop_pct = {
                    let mut trail_highs =
                        state.trading.trail_highs.lock().unwrap_or_else(|e| e.into_inner());
                    let high = trail_highs.entry(eval.symbol.clone()).or_insert(eval.current_price);
                    if eval.current_price > *high {
                        *high = eval.current_price;
                    }
                    if let Ok(mut tlu) = state.trading.trail_last_update.lock() {
                        let now_s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                        tlu.insert(eval.symbol.clone(), now_s);
                    }
                    ((*high - eval.current_price) / *high) * 100.0
                };
                (
                    eval.pnl_pct <= -eval.stop_loss || eval.pnl_pct >= eval.take_profit || drop_pct >= eval.trail_pct,
                    "sell",
                )
            } else {
                let rise_pct = {
                    let mut trail_lows = state.trading.trail_lows.lock().unwrap_or_else(|e| e.into_inner());
                    let low = trail_lows.entry(eval.symbol.clone()).or_insert(eval.current_price);
                    if eval.current_price < *low {
                        *low = eval.current_price;
                    }
                    if let Ok(mut tlu) = state.trading.trail_last_update.lock() {
                        let now_s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                        tlu.insert(eval.symbol.clone(), now_s);
                    }
                    ((eval.current_price - *low) / *low) * 100.0
                };
                let short_pnl = ((eval.entry_price - eval.current_price) / eval.entry_price) * 100.0;
                (
                    short_pnl <= -eval.stop_loss || short_pnl >= eval.take_profit || rise_pct >= eval.trail_pct,
                    "buy",
                )
            };

            if should_close {
                if has_pending_orders_for_symbol(state.clone(), &eval.symbol).await {
                    tracing::debug!(component = "autopilot", symbol = %eval.symbol,
                        "Skipping exit — broker already has open order");
                    continue 'autopilot;
                }

                {
                    let cooldowns = state.trading.failed_order_cooldown.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(&(cooldown_until, attempts)) = cooldowns.get(&eval.symbol) {
                        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                        if now_ms < cooldown_until {
                            if rand::random::<f64>() < 0.02 {
                                tracing::debug!(component = "autopilot", symbol = %eval.symbol,
                                    attempts, remaining_ms = cooldown_until - now_ms,
                                    "Exit on cooldown — skipping");
                            }
                            continue;
                        }
                    }
                }

                {
                    let mut prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                    let sym_p = normalize_symbol(&eval.symbol);
                    prices.entry(sym_p.clone()).or_insert(eval.current_price);
                    let cached = prices.get(&sym_p).copied().unwrap_or(0.0);
                    if cached <= 0.0 {
                        prices.insert(sym_p, eval.current_price);
                    }
                }

                let realized_pnl = if !eval.is_short {
                    (eval.current_price - eval.entry_price) * eval.qty
                } else {
                    (eval.entry_price - eval.current_price) * eval.qty
                };

                match close_position(state.clone(), &eval.symbol, eval.qty, side, &app).await {
                    Ok(()) => {
                        state.trading.failed_order_cooldown.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol);
                        if let Ok(dec_pnl) = rust_decimal::Decimal::try_from(realized_pnl) {
                            if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                                analytics.record_trade_with_duration(dec_pnl, eval.hold_secs);
                            }
                        }
                        if settings.enable_oracle_adaptation {
                            let base_w = crate::state::build_oracle_weights(&settings);
                            if let Ok(mut adapt) = state.oracle_adaptation.lock() {
                                adapt.record_trade_outcome(
                                    &eval.symbol,
                                    realized_pnl,
                                    &base_w,
                                );
                            }
                        }
                        let strategy_name = state
                            .strategy_mode
                            .lock()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|_| "UNKNOWN".into());
                        let journal_source = match strategy_mode {
                            StrategyMode::MathOracle => "ORACLE".to_string(),
                            StrategyMode::GpuNexus => "GPU".to_string(),
                            StrategyMode::Fusion => "FUSION".to_string(),
                            StrategyMode::SignalStrategy => settings.signal_source.to_ascii_uppercase(),
                            StrategyMode::SmartTrend => "SMARTTREND".to_string(),
                        };
                        if let Ok(mut journal) = state.trade_journal.lock() {
                            let exit_side = if eval.is_short { "SHORT" } else { "LONG" };
                            let features = state.trading.entry_features.lock().ok().and_then(|mut m| m.remove(&eval.symbol));
                            let exit_feat = snapshot_exit_features(&state, &eval.symbol, &eval.cand_tf);
                            journal.record_trade(&TradeRecordInput {
                                symbol: &eval.symbol,
                                side: exit_side,
                                qty: eval.qty,
                                entry_price: eval.entry_price,
                                exit_price: eval.current_price,
                                pnl: realized_pnl,
                                strategy_mode: &strategy_name,
                                source: &journal_source,
                                timeframe: &eval.cand_tf,
                                duration_secs: eval.hold_secs,
                                reason: if eval.force_exit && !eval.force_exit_reason.is_empty() {
                                    &eval.force_exit_reason
                                } else {
                                    "TRAIL/STOP/TP"
                                },
                                entry_features: features,
                                exit_features: Some(exit_feat),
                                broker_order_id: None,
                            });
                        }
                        {
                            state.trading.trail_highs.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol);
                        }
                        {
                            state.trading.trail_lows.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol);
                        }
                        {
                            state.trading.partial_taken.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol);
                        }
                        {
                            let mut settings_lock = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                            if let Some(pos) = settings_lock.positions.iter_mut().find(|p| p.symbol == eval.symbol) {
                                pos.state = crate::app_state::PositionState::PendingClose;
                                pos.closed_at = Some(
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                );
                            }
                        }
                        {
                            let grace_until = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64 + 30_000;
                            if let Ok(mut grace) = state.trading.recon_grace.lock() {
                                grace.insert(eval.symbol.clone(), grace_until);
                            }
                        }
                        let _ = state.ws_broadcast.send(serde_json::json!({
                            "type": "trade",
                            "action": "EXIT",
                            "symbol": &eval.symbol,
                            "side": side.to_uppercase(),
                            "pnl": realized_pnl.to_f64().unwrap_or(0.0),
                            "duration_secs": eval.hold_secs,
                        }).to_string());
                        let _ = app.emit_all(
                            "log-message",
                            format!(
                                "AUTOPILOT EXIT: {} {} — PnL: ${:.2}",
                                side.to_uppercase(),
                                eval.symbol,
                                realized_pnl
                            ),
                        );
                        let _ = positions_snapshot_from_settings(&state);
                        continue 'autopilot;
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        let is_permanent = err_str.contains("422")
                            || err_str.contains("invalid")
                            || err_str.contains("does not exist")
                            || err_str.contains("not found")
                            || err_str.contains("NO POSITION")
                            || err_str.contains("not tradable")
                            || err_str.contains("delisted");
                        if is_permanent {
                            { state.trading.trail_highs.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol); }
                            { state.trading.trail_lows.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol); }
                            { state.trading.partial_taken.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol); }
                            { state.trading.exit_inflight.lock().unwrap_or_else(|e| e.into_inner()).remove(&eval.symbol); }
                            {
                                let mut bl = state.trading.position_blacklist.lock().unwrap_or_else(|e| e.into_inner());
                                bl.insert(eval.symbol.clone());
                                let _ = crate::state::save_blacklist(&bl);
                            }
                            tracing::error!(component = "autopilot", symbol = %eval.symbol, error = %e, "Permanent exit failure — blacklisting symbol");
                            let _ = app.emit_all(
                                "log-message",
                                format!("AUTOPILOT EXIT PERMANENT FAIL: {} {} — {} (blacklisted)", side.to_uppercase(), eval.symbol, err_str),
                            );
                        } else {
                            let mut cooldowns = state.trading.failed_order_cooldown.lock().unwrap_or_else(|er| er.into_inner());
                            let prev_attempts = cooldowns.get(&eval.symbol).map(|&(_, a)| a).unwrap_or(0);
                            let attempts = prev_attempts + 1;
                            let cooldown_secs = 300u64.min(30u64.saturating_mul(1u64 << (attempts - 1).min(10)));
                            let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                            cooldowns.insert(eval.symbol.clone(), (now_ms + cooldown_secs * 1000, attempts));
                            tracing::warn!(component = "autopilot", symbol = %eval.symbol, error = %e,
                                attempts, cooldown_secs, "Close position failed — backoff applied");
                            let _ = app.emit_all(
                                "log-message",
                                format!("AUTOPILOT EXIT FAILED: {} {} — {} (backoff {}s, attempt #{})", side.to_uppercase(), eval.symbol, err_str, cooldown_secs, attempts),
                            );
                        }
                        let _ = positions_snapshot_from_settings(&state);
                        continue 'autopilot;
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        //  AUTOPILOT ENTRY LOGIC — gated by breaker, spread, cooldown
        //  Supports multi-symbol when enabled (iterates hft_targets)
        // ═══════════════════════════════════════════════════════════════
        if !settings.auto_entry || !entries_allowed || !spread_ok_for_entry {
            if rand::random::<f64>() < 0.008 {
                let reason = if !settings.auto_entry {
                    "auto_entry=OFF".to_string()
                } else if !entries_allowed {
                    if dd_velocity_halt {
                        format!("DD velocity halt (threshold {:.2}%, window {}s)",
                            settings.dd_velocity_threshold, settings.dd_velocity_window_secs)
                    } else {
                        format!("circuit breaker {:?}", breaker_level)
                    }
                } else {
                    let ofi_snap = state.market.ofi_state.load();
                    let stale = ofi_snap.is_stale(settings.ofi_stale_ms.into());
                    if stale {
                        format!("L2 data stale (no update in {}ms, spread {:.1} bps)",
                            settings.ofi_stale_ms, ofi_snap.spread_bps)
                    } else {
                        format!("spread {:.1} bps > limit {:.0} bps",
                            ofi_snap.spread_bps, settings.max_spread_bps)
                    }
                };
                let _ = app.emit_all("log-message", format!("⛔ ENTRY BLOCKED: {}", reason));
            }
            continue;
        }

        let active_positions = positions.len();
        if active_positions >= settings.max_open_positions as usize {
            continue;
        }

        // Cooldown gate
        let cooldown_ok = {
            let analytics = state
                .risk
                .session_analytics
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            analytics.cooldown_elapsed(settings.entry_cooldown_secs)
        };
        let cooldown_ok_for_entry =
            if matches!(strategy_mode, StrategyMode::SignalStrategy) && settings.bypass_cooldown {
                true
            } else {
                cooldown_ok
            };
        if !cooldown_ok_for_entry {
            continue;
        }

        // Market hours gate: skip entries if market is closed and extended hours not enabled
        let extended_ok_for_entry = settings.enable_extended
            || (matches!(strategy_mode, StrategyMode::SignalStrategy) && settings.bypass_extended);
        if !extended_ok_for_entry {
            let market_open = is_market_open(state.clone()).await;
            if !market_open {
                continue;
            }
        }

        // Intraday blackout gate (S04.1): skip new entries in first/last N minutes.
        // Exempt symbols bypass the blackout (B4).
        let in_blackout = if settings.blackout_first_mins > 0 || settings.blackout_last_mins > 0 {
            let now_utc = chrono::Utc::now();
            let offset_h = if crate::state::is_us_dst(now_utc) { 4 } else { 5 };
            let tz = chrono::FixedOffset::west_opt(offset_h * 3600)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("UTC offset"));
            let now_et = now_utc.with_timezone(&tz);
            let today = now_et.date_naive();
            let market_open_time = today.and_hms_opt(9, 30, 0);
            let market_close_time = today.and_hms_opt(16, 0, 0);
            if let (Some(open_t), Some(close_t)) = (market_open_time, market_close_time) {
                let now_naive = now_et.naive_local();
                let mins_since_open = (now_naive - open_t).num_minutes();
                let mins_until_close = (close_t - now_naive).num_minutes();
                let bl = (mins_since_open >= 0
                    && (mins_since_open as u64) < settings.blackout_first_mins)
                    || (mins_until_close >= 0
                        && (mins_until_close as u64) < settings.blackout_last_mins);
                if bl && rand::random::<f64>() < 0.02 {
                    tracing::info!(
                        component = "autopilot",
                        "BLACKOUT active ({}m since open, {}m until close), exempt: {:?}",
                        mins_since_open, mins_until_close, settings.blackout_exempt_symbols
                    );
                }
                bl
            } else {
                false
            }
        } else {
            false
        };

        // Build candidate symbol list: Signal Strategy uses its own asset list
        let candidate_symbols: Vec<String> =
            if matches!(strategy_mode, StrategyMode::SignalStrategy)
                && !settings.signal_strategy_assets.is_empty()
            {
                settings.signal_strategy_assets.clone()
            } else if settings.multi_symbol_enabled && !settings.hft_targets.is_empty() {
                settings.hft_targets.clone()
            } else {
                vec![settings.active_symbol.clone()]
            };

        // ═══════════════════════════════════════════════════════════════
        //  S5+S6: PARALLEL CANDIDATE ENTRY EVALUATION
        //  Phase 1: Pre-filter candidates sequentially (flash crash, holding, pending, cooldown, correlation)
        //  Phase 2: Signal Strategy candidates processed sequentially (different flow)
        //  Phase 3: Non-signal candidates evaluated in parallel via join_all
        //  Phase 4: Best entry selected and executed
        // ═══════════════════════════════════════════════════════════════

        // Phase 1: Pre-filter candidates (sequential -- needs state reads + async pending checks)
        let mut filtered_candidates: Vec<(String, String, f64)> = Vec::new(); // (symbol, cand_tf, vol_factor)
        for candidate_symbol in &candidate_symbols {
            if candidate_symbol.is_empty() {
                continue;
            }
            if in_blackout
                && !settings
                    .blackout_exempt_symbols
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(candidate_symbol))
            {
                continue;
            }
            {
                let fc = state.risk.flash_crash.lock().unwrap_or_else(|e| e.into_inner());
                if fc.is_blocked(candidate_symbol) {
                    continue;
                }
            }
            let already_holding = positions
                .iter()
                .any(|p| p.symbol.eq_ignore_ascii_case(candidate_symbol));
            if already_holding && !matches!(strategy_mode, StrategyMode::SignalStrategy) {
                continue;
            }
            let has_pending = has_pending_orders_for_symbol(state.clone(), candidate_symbol).await;
            if has_pending {
                continue;
            }
            {
                let cooldowns = state
                    .trading
                    .failed_order_cooldown
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if let Some(&(cooldown_until, _attempts)) = cooldowns.get(candidate_symbol) {
                    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                    if now_ms < cooldown_until {
                        continue;
                    }
                }
            }
            let current_pos_count = positions.len();
            if current_pos_count >= settings.max_open_positions as usize {
                break;
            }

            // CORRELATION GUARD
            let skip_correlation = matches!(strategy_mode, StrategyMode::SignalStrategy)
                && (settings.bypass_max_correlation || settings.bypass_correlation_window);
            if settings.multi_symbol_enabled && !positions.is_empty() && !skip_correlation {
                let held_symbols: Vec<String> = positions
                    .iter()
                    .filter(|p| !p.symbol.eq_ignore_ascii_case(candidate_symbol))
                    .map(|p| p.symbol.clone())
                    .collect();
                let cand_tf_corr = {
                    let tf_map = state
                        .scan
                        .symbol_best_tf
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    tf_map
                        .get(&normalize_symbol(candidate_symbol))
                        .cloned()
                        .unwrap_or_else(|| normalize_timeframe_internal(&tf))
                };
                let cand_candles = get_candles(&state, candidate_symbol, &cand_tf_corr, 1).await;
                if let Ok(ref cc) = cand_candles {
                    let cand_closes: Vec<f64> = cc.iter().rev().take(settings.correlation_window)
                        .map(|c| c.close).collect::<Vec<_>>().into_iter().rev().collect();
                    let cand_returns = correlation::returns_from_closes(&cand_closes);
                    let mut held_returns: Vec<(String, Vec<f64>)> = Vec::new();
                    for hs in &held_symbols {
                        if let Ok(hc) = get_candles(&state, hs, &cand_tf_corr, 1).await {
                            let hc_closes: Vec<f64> = hc.iter().rev().take(settings.correlation_window)
                                .map(|c| c.close).collect::<Vec<_>>().into_iter().rev().collect();
                            held_returns.push((hs.clone(), correlation::returns_from_closes(&hc_closes)));
                        }
                    }
                    let (safe, reason) = correlation::check_correlation_safe(&cand_returns, &held_returns, settings.max_correlation);
                    if !safe {
                        if let Some(msg) = reason {
                            if rand::random::<f64>() < 0.02 {
                                let _ = app.emit_all("log-message", format!("🔗 {} [{}]", msg, candidate_symbol));
                            }
                        }
                        continue;
                    }
                }
            }

            let cand_tf = {
                let tf_map = state
                    .scan
                    .symbol_best_tf
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                tf_map
                    .get(&normalize_symbol(candidate_symbol))
                    .cloned()
                    .unwrap_or_else(|| normalize_timeframe_internal(&tf))
            };
            let cand_vol_key = normalize_symbol(candidate_symbol);
            let vol_fallback = {
                let cache = state.market.vol_cache.lock().unwrap_or_else(|e| e.into_inner());
                cache.values.get(&cand_vol_key).map(|e| e.value).unwrap_or(0.0)
            };
            let vol_factor = (vol_fallback / 1.0).clamp(settings.vol_factor_min, settings.vol_factor_max);

            filtered_candidates.push((candidate_symbol.clone(), cand_tf, vol_factor));
        }

        // Phase 2: Signal Strategy candidates -- processed sequentially (unchanged flow)
        if matches!(strategy_mode, StrategyMode::SignalStrategy) {
            for (candidate_symbol, cand_tf, vol_factor) in &filtered_candidates {
                if let Ok(resp) =
                    get_signal_markers_internal(&state, candidate_symbol, cand_tf).await
                {
                    let markers = resp.markers;
                    if let Some(last) = markers.last() {
                        let candle_secs: i64 = match cand_tf.as_str() {
                            "1m" => 60, "5m" => 300, "10m" => 600, "15m" => 900,
                            "30m" => 1800, "1h" => 3600, "4h" => 14400, "1d" => 86400, _ => 300,
                        };
                        let now_ts = chrono::Utc::now().timestamp();
                        let is_fresh = (now_ts - last.time) <= candle_secs * 2;
                        if is_fresh {
                            let is_buy_signal = last.text.contains("BUY");
                            let is_sell_signal = last.text.contains("SELL") || last.text.contains("SHORT");

                            let held_long = positions.iter().any(|p| {
                                p.symbol.eq_ignore_ascii_case(candidate_symbol)
                                    && p.side.eq_ignore_ascii_case("long")
                            });
                            if is_buy_signal && held_long {
                                continue;
                            }
                            if is_sell_signal && !held_long {
                                continue;
                            }

                            if is_sell_signal && held_long {
                                let held_qty = positions.iter()
                                    .find(|p| p.symbol.eq_ignore_ascii_case(candidate_symbol) && p.side.eq_ignore_ascii_case("long"))
                                    .and_then(|p| p.qty.parse::<f64>().ok())
                                    .unwrap_or(0.0)
                                    .abs();
                                if held_qty <= 0.0 { continue; }
                                let _ = app.emit_all("log-message", format!("📍 SIGNAL STRATEGY: SELL {} — signal='{}' on {} (close long, qty={:.6})", candidate_symbol, last.text, cand_tf, held_qty));
                                match close_position(state.clone(), candidate_symbol, held_qty, "sell", &app).await {
                                    Ok(()) => {
                                        let _ = app.emit_all("log-message", format!("✅ SIGNAL EXIT: {} — closed long {:.6}", candidate_symbol, held_qty));
                                        state.trading.failed_order_cooldown.lock().unwrap_or_else(|e| e.into_inner()).remove(candidate_symbol.as_str());
                                        let _ = positions_snapshot_from_settings(&state);
                                        continue 'autopilot;
                                    }
                                    Err(e) => {
                                        let mut cooldowns = state.trading.failed_order_cooldown.lock().unwrap_or_else(|er| er.into_inner());
                                        let prev_attempts = cooldowns.get(candidate_symbol.as_str()).map(|&(_, a)| a).unwrap_or(0);
                                        let attempts = prev_attempts + 1;
                                        let cooldown_secs = 300u64.min(30u64.saturating_mul(1u64 << (attempts - 1).min(10)));
                                        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                                        cooldowns.insert(candidate_symbol.to_string(), (now_ms + cooldown_secs * 1000, attempts));
                                        let _ = app.emit_all("log-message", format!("❌ SIGNAL EXIT FAILED: {} — {} (backoff {}s, attempt #{})", candidate_symbol, e, cooldown_secs, attempts));
                                    }
                                }
                            } else if is_buy_signal {
                                let side = "buy";
                                let stop_dist = if settings.aggressive_mode || settings.bypass_aggressive { 1.3 * vol_factor } else { 0.8 * vol_factor };
                                let equity = {
                                    let analytics = state.risk.session_analytics.lock().unwrap_or_else(|e| e.into_inner());
                                    analytics.current_equity.to_f64().unwrap_or(settings.default_equity_fallback)
                                };
                                let price_est = match get_candles(&state, candidate_symbol, cand_tf, 1).await {
                                    Ok(candles) if !candles.is_empty() => candles.last().map(|c| c.close).unwrap_or(0.0),
                                    _ => { continue; }
                                };
                                if price_est <= 0.0 { continue; }
                                let perf_multiplier = {
                                    let j = state.trade_journal.lock().unwrap_or_else(|e| e.into_inner());
                                    let perfs = correlation::compute_symbol_performance(&j);
                                    correlation::get_risk_multiplier_for_symbol(&perfs, candidate_symbol)
                                };
                                let base_risk = if settings.bypass_risk_per_trade { settings.max_bypass_risk_pct } else { settings.risk_per_trade };
                                let adjusted_risk = base_risk * perf_multiplier;
                                let sizing = risk::calculate_position_size(&risk::SizingParams {
                                    equity: rust_decimal::Decimal::try_from(equity).unwrap_or(rust_decimal::Decimal::ZERO),
                                    risk_pct: rust_decimal::Decimal::try_from(adjusted_risk.min(settings.max_bypass_risk_pct)).unwrap_or(rust_decimal_macros::dec!(2)),
                                    entry_price: rust_decimal::Decimal::try_from(price_est).unwrap_or(rust_decimal::Decimal::ZERO),
                                    stop_loss_price: rust_decimal::Decimal::try_from(price_est * (1.0 - stop_dist / 100.0)).unwrap_or(rust_decimal::Decimal::ZERO),
                                    breaker_multiplier: state.risk.circuit_breaker.size_multiplier(),
                                    max_notional_frac: rust_decimal::Decimal::try_from(settings.max_notional_equity_frac).unwrap_or(rust_decimal_macros::dec!(0.25)),
                                    min_stop_frac: rust_decimal::Decimal::try_from(settings.min_stop_distance_frac).unwrap_or(rust_decimal_macros::dec!(0.0001)),
                                });
                                let qty = match sizing { Some(s) => s.qty.to_f64().unwrap_or(0.0).min(settings.max_qty_cap), None => { continue; } };
                                if qty >= settings.min_qty_threshold {
                                    let order = LegacyOrder { symbol: candidate_symbol.to_string(), side: side.to_string(), qty, take_profit: None, stop_loss: None };
                                    let _ = app.emit_all("log-message", format!("📍 SIGNAL STRATEGY: {} {} — signal='{}' on {} (price={:.2}, qty={:.1})", side.to_uppercase(), candidate_symbol, last.text, cand_tf, price_est, qty));
                                    match send_order_internal(state.clone(), order, app.clone()).await {
                                        Ok(res) => {
                                            let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                            let tag = format!("SIGNAL|{}", last.text);
                                            { let mut analytics = state.risk.session_analytics.lock().unwrap_or_else(|e| e.into_inner()); analytics.record_entry_time(); }
                                            { let mut et = state.trading.entry_tags.lock().unwrap_or_else(|e| e.into_inner()); et.insert(candidate_symbol.to_string(), tag.clone()); }
                                            { let mut times = state.trading.entry_times.lock().unwrap_or_else(|e| e.into_inner()); times.insert(candidate_symbol.to_string(), now_epoch); }
                                            { let mut settings_lock = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                              if let Some(pos) = settings_lock.positions.iter_mut().find(|pp| pp.symbol == *candidate_symbol) {
                                                  if pos.opened_at.is_none() { pos.opened_at = Some(now_epoch); }
                                                  pos.entry_tag = Some(tag);
                                                  pos.state = crate::app_state::PositionState::Open;
                                              }
                                            }
                                            state.trading.failed_order_cooldown.lock().unwrap_or_else(|e| e.into_inner()).remove(candidate_symbol.as_str());
                                            {
                                                let signal_decision = TradeDecision {
                                                    signal: if side == "buy" { crate::strategy::Signal::Buy } else { crate::strategy::Signal::Short },
                                                    symbol: candidate_symbol.to_string(),
                                                    score: 0,
                                                    reason: format!("SIGNAL|{}", last.text),
                                                    fvg: crate::strategy::FvgType::None,
                                                    entry_type: "SIGNAL_STRATEGY".to_string(),
                                                };
                                                let features = snapshot_entry_features(&state, candidate_symbol, cand_tf, &signal_decision, &settings);
                                                if let Ok(mut ef) = state.trading.entry_features.lock() { ef.insert(candidate_symbol.to_string(), features); }
                                            }
                                            let _ = app.emit_all("log-message", format!("✅ SIGNAL ENTRY: {} — [{}]", candidate_symbol, res.broker_id));
                                            tracing::info!(component = "backoff", symbol = %candidate_symbol, "Backoff reset on successful order");
                                            let _ = positions_snapshot_from_settings(&state);
                                            continue 'autopilot;
                                        }
                                        Err(e) => {
                                            let mut cooldowns = state.trading.failed_order_cooldown.lock().unwrap_or_else(|er| er.into_inner());
                                            let prev_attempts = cooldowns.get(candidate_symbol.as_str()).map(|&(_, a)| a).unwrap_or(0);
                                            let attempts = prev_attempts + 1;
                                            let cooldown_secs = 300u64.min(30u64.saturating_mul(1u64 << (attempts - 1).min(10)));
                                            let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                                            cooldowns.insert(candidate_symbol.to_string(), (now_ms + cooldown_secs * 1000, attempts));
                                            let _ = app.emit_all("log-message", format!("❌ SIGNAL ENTRY FAILED: {} — {} (backoff {}s, attempt #{})", candidate_symbol, e, cooldown_secs, attempts));
                                            tracing::warn!(component = "backoff", symbol = %candidate_symbol, attempt = attempts, cooldown_secs, "Exponential backoff applied");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Phase 3: Non-signal candidates -- evaluate check_entry in PARALLEL via join_all
            let entry_futures = filtered_candidates.iter().map(|(sym, cand_tf, _)| {
                let state_ref = &state;
                let settings_ref = &settings;
                evaluate_candidate_entry(state_ref, sym, settings_ref, cand_tf)
            });
            let entry_evals: Vec<EntryEvaluation> =
                futures::future::join_all(entry_futures).await;

            // Phase 4: Select best entry by score and execute
            let best_entry = entry_evals.iter()
                .filter_map(|e| {
                    let d = e.decision.as_ref()?;
                    let side = match d.signal {
                        crate::strategy::Signal::Buy => Some("buy"),
                        crate::strategy::Signal::Short if settings.allow_short => Some("sell"),
                        _ => None,
                    }?;
                    Some((e, d, side))
                })
                .max_by(|a, b| a.1.score.cmp(&b.1.score));

            if let Some((eval, decision, side)) = best_entry {
                let symbol = &eval.symbol;
                let is_short = side == "sell";
                let is_buy = !is_short;

                // OFI FILTER
                let is_active_sym = symbol.eq_ignore_ascii_case(&settings.active_symbol);
                let mut ofi_blocked = false;
                if evo_params.ofi_filter_enabled && !ofi_stale && is_active_sym {
                    let ofi_threshold = settings.ofi_entry_threshold;
                    if !risk::ofi_confirms_signal(live_ofi, is_buy, ofi_threshold) {
                        if rand::random::<f64>() < 0.05 {
                            let _ = app.emit_all("log-message",
                                format!("OFI FILTER: {} {} blocked — OFI={:.2} contradicts signal", symbol, if is_buy { "BUY" } else { "SELL" }, live_ofi));
                        }
                        ofi_blocked = true;
                    }
                }
                // NEXUS CONSENSUS FILTER
                let mut nexus_blocked = false;
                if !ofi_blocked && is_active_sym {
                    let nexus_threshold = evo_params.nexus_buy_threshold_pct.to_f64().unwrap_or(60.0);
                    let consensus = state.nexus_consensus.lock().unwrap_or_else(|e| e.into_inner());
                    let nexus_agrees = if is_buy { consensus.buy_pct >= nexus_threshold } else { consensus.sell_pct >= nexus_threshold };
                    if consensus.active_agents > 0 && !nexus_agrees {
                        if rand::random::<f64>() < 0.05 {
                            let _ = app.emit_all("log-message",
                                format!("NEXUS FILTER: {} {} blocked — buy_pct={:.1}% sell_pct={:.1}% (need {:.0}%)",
                                    symbol, if is_buy { "BUY" } else { "SELL" }, consensus.buy_pct, consensus.sell_pct, nexus_threshold));
                        }
                        nexus_blocked = true;
                    }
                }

                // Portfolio Risk Gates (S9): VaR + HHI (S14: notional uses live prices)
                let mut portfolio_risk_blocked = false;
                if !ofi_blocked && !nexus_blocked {
                    let prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                    let current_notionals: Vec<f64> = settings.positions.iter()
                        .filter(|p| p.qty.abs() > rust_decimal::Decimal::ZERO)
                        .map(|p| {
                            let live_price = prices.get(&normalize_symbol(&p.symbol))
                                .copied()
                                .filter(|&px| px > 0.0)
                                .unwrap_or_else(|| p.entry_price.to_f64().unwrap_or(0.0));
                            p.qty.abs().to_f64().unwrap_or(0.0) * live_price
                        })
                        .collect();
                    drop(prices);

                    if settings.enable_concentration_gate && !current_notionals.is_empty() {
                        let current_price = {
                            let px = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                            px.get(&normalize_symbol(symbol)).copied().unwrap_or(0.0)
                        };
                        let proposed_qty = settings.risk_per_trade / 100.0 * {
                            let analytics = state.risk.session_analytics.lock().unwrap_or_else(|e| e.into_inner());
                            analytics.current_equity.to_f64().unwrap_or(settings.default_equity_fallback)
                        };
                        let est_notional = if current_price > 0.0 {
                            proposed_qty / current_price * current_price
                        } else {
                            current_notionals.iter().sum::<f64>() * 0.1_f64.max(settings.risk_per_trade / 100.0)
                        };
                        let hhi_pre = risk::concentration_hhi(&current_notionals);
                        let blocked = risk::hhi_gate_blocks_entry(&current_notionals, est_notional, settings.max_concentration_hhi);
                        if blocked {
                            let mut with_new = current_notionals.clone();
                            with_new.push(est_notional);
                            let hhi_post = risk::concentration_hhi(&with_new);
                            tracing::info!(
                                component = "hhi_gate",
                                symbol,
                                hhi_pre,
                                hhi_post,
                                blocked = true,
                                est_notional,
                                max_hhi = settings.max_concentration_hhi,
                                "HHI gate blocked entry"
                            );
                            if rand::random::<f64>() < 0.1 {
                                let _ = app.emit_all("log-message",
                                    format!("HHI GATE: {} blocked — concentration would exceed {:.0}%", symbol, settings.max_concentration_hhi * 100.0));
                            }
                            portfolio_risk_blocked = true;
                        }
                    }

                    if !portfolio_risk_blocked && settings.enable_var_gate {
                        let current_positions: Vec<risk::PortfolioPosition> = settings.positions.iter()
                            .filter(|p| p.qty.abs() > rust_decimal::Decimal::ZERO)
                            .map(|p| risk::PortfolioPosition {
                                symbol: p.symbol.clone(),
                                notional: (p.qty.abs() * p.entry_price).to_f64().unwrap_or(0.0),
                            })
                            .collect();
                        let est_notional = {
                            let analytics = state.risk.session_analytics.lock().unwrap_or_else(|e| e.into_inner());
                            let equity = analytics.current_equity.to_f64().unwrap_or(settings.default_equity_fallback);
                            equity * settings.risk_per_trade / 100.0
                        };
                        let mut returns_map = std::collections::HashMap::new();
                        let bufs = state.market.candle_buffers.lock().unwrap_or_else(|e| e.into_inner());
                        let lookback = settings.var_lookback_periods;
                        let mut all_syms: Vec<String> = current_positions.iter().map(|p| p.symbol.clone()).collect();
                        if !all_syms.contains(&symbol.to_string()) {
                            all_syms.push(symbol.to_string());
                        }
                        for s in &all_syms {
                            let buf_key = cache_key_symbol_tf(s, &eval.cand_tf);
                            if let Some(buf) = bufs.get(&buf_key) {
                                let snap = buf.snapshot();
                                let closes: Vec<f64> = snap.iter().rev().take(lookback + 1).map(|c| c.close).collect::<Vec<_>>().into_iter().rev().collect();
                                let rets = risk::compute_returns(&closes);
                                if !rets.is_empty() {
                                    returns_map.insert(s.clone(), rets);
                                }
                            }
                        }
                        drop(bufs);
                        if risk::var_gate_blocks_entry(&current_positions, symbol, est_notional, &returns_map, 0.99, settings.max_portfolio_var_pct, settings.var_fail_closed) {
                            let has_data = returns_map.values().any(|r| r.len() >= 2);
                            let reason = if !has_data && settings.var_fail_closed {
                                "insufficient returns data (fail-closed)"
                            } else {
                                "portfolio VaR would exceed threshold"
                            };
                            let _ = app.emit_all("var-gate-blocked", serde_json::json!({
                                "symbol": symbol,
                                "reason": reason,
                                "max_var_pct": settings.max_portfolio_var_pct,
                                "fail_closed": settings.var_fail_closed,
                            }));
                            if rand::random::<f64>() < 0.1 {
                                let _ = app.emit_all("log-message",
                                    format!("VaR GATE: {} blocked — {}", symbol, reason));
                            }
                            portfolio_risk_blocked = true;
                        }
                    }
                }

                if !ofi_blocked && !nexus_blocked && !portfolio_risk_blocked {
                    let stop_dist = if settings.aggressive_mode { 1.3 * eval.vol_factor } else { 0.8 * eval.vol_factor };
                    let equity = {
                        let analytics = state.risk.session_analytics.lock().unwrap_or_else(|e| e.into_inner());
                        analytics.current_equity.to_f64().unwrap_or(settings.default_equity_fallback)
                    };
                    let price_est = match get_candles(&state, symbol, &eval.cand_tf, 1).await {
                        Ok(candles) if !candles.is_empty() => candles.last().map(|c| c.close).unwrap_or(0.0),
                        _ => 0.0,
                    };
                    if price_est > 0.0 {
                        let perf_multiplier = {
                            let j = state.trade_journal.lock().unwrap_or_else(|e| e.into_inner());
                            let perfs = correlation::compute_symbol_performance(&j);
                            correlation::get_risk_multiplier_for_symbol(&perfs, symbol)
                        };
                        let adjusted_risk = settings.risk_per_trade * perf_multiplier;
                        let sizing = risk::calculate_position_size(&risk::SizingParams {
                            equity: rust_decimal::Decimal::try_from(equity).unwrap_or(rust_decimal::Decimal::ZERO),
                            risk_pct: rust_decimal::Decimal::try_from(adjusted_risk).unwrap_or(rust_decimal_macros::dec!(2)),
                            entry_price: rust_decimal::Decimal::try_from(price_est).unwrap_or(rust_decimal::Decimal::ZERO),
                            stop_loss_price: rust_decimal::Decimal::try_from(if is_short { price_est * (1.0 + stop_dist / 100.0) } else { price_est * (1.0 - stop_dist / 100.0) }).unwrap_or(rust_decimal::Decimal::ZERO),
                            breaker_multiplier: state.risk.circuit_breaker.size_multiplier(),
                            max_notional_frac: rust_decimal::Decimal::try_from(settings.max_notional_equity_frac).unwrap_or(rust_decimal_macros::dec!(0.25)),
                            min_stop_frac: rust_decimal::Decimal::try_from(settings.min_stop_distance_frac).unwrap_or(rust_decimal_macros::dec!(0.0001)),
                        });
                        if let Some(s) = sizing {
                            let qty = s.qty.to_f64().unwrap_or(0.0).min(settings.max_qty_cap);
                            if qty >= settings.min_qty_threshold {
                                let order = LegacyOrder { symbol: symbol.to_string(), side: side.to_string(), qty, take_profit: None, stop_loss: None };
                                let is_scalp_entry = match strategy_mode {
                                    StrategyMode::MathOracle => settings.scalping_oracle,
                                    StrategyMode::GpuNexus => settings.scalping_gpu,
                                    StrategyMode::Fusion | StrategyMode::SignalStrategy | StrategyMode::SmartTrend => settings.scalping_fusion,
                                };
                                match send_order_internal(state.clone(), order.clone(), app.clone()).await {
                                    Ok(order_result) => {
                                        let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                        if let Ok(mut analytics) = state.risk.session_analytics.lock() { analytics.record_entry_time(); }
                                        let tag = if is_scalp_entry { format!("SCALP|{}", decision.entry_type) } else { decision.entry_type.clone() };
                                        if let Ok(mut tags) = state.trading.entry_tags.lock() { tags.insert(symbol.to_string(), tag.clone()); }
                                        if let Ok(mut et) = state.trading.entry_times.lock() { et.insert(symbol.to_string(), now_epoch); }
                                        { let mut settings_lock = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                          if let Some(pos) = settings_lock.positions.iter_mut().find(|pp| pp.symbol == *symbol) {
                                              if pos.opened_at.is_none() { pos.opened_at = Some(now_epoch); }
                                              pos.entry_tag = Some(tag.clone());
                                              pos.state = crate::app_state::PositionState::Open;
                                              pos.broker_order_id = Some(order_result.broker_id.clone());
                                          }
                                        }
                                        state.trading.failed_order_cooldown.lock().unwrap_or_else(|e| e.into_inner()).remove(&symbol.to_string());
                                        {
                                            let features = snapshot_entry_features(&state, symbol, &eval.cand_tf, decision, &settings);
                                            if let Ok(mut ef) = state.trading.entry_features.lock() { ef.insert(symbol.to_string(), features); }
                                        }
                                        let _ = app.emit_all("log-message", format!("AUTOPILOT ENTRY: {} {} x{:.4} — score: {} [{}]", side.to_uppercase(), symbol, order.qty, decision.score, tag));
                                        let _ = positions_snapshot_from_settings(&state);
                                        continue 'autopilot;
                                    }
                                    Err(e) => {
                                        let mut cooldowns = state.trading.failed_order_cooldown.lock().unwrap_or_else(|er| er.into_inner());
                                        let prev_attempts = cooldowns.get(&symbol.to_string()).map(|&(_, a)| a).unwrap_or(0);
                                        let attempts = prev_attempts + 1;
                                        let cooldown_secs = 300u64.min(30u64.saturating_mul(1u64 << (attempts - 1).min(10)));
                                        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                                        cooldowns.insert(symbol.to_string(), (now_ms + cooldown_secs * 1000, attempts));
                                        let _ = app.emit_all("log-message", format!("AUTOPILOT ENTRY FAILED: {} — {} (backoff {}s, attempt #{})", symbol, e, cooldown_secs, attempts));
                                        tracing::warn!(component = "backoff", symbol = %symbol, attempt = attempts, cooldown_secs, "Exponential backoff applied");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } // end entry evaluation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{FvgType, Signal};

    fn make_exit_eval(symbol: &str, is_short: bool, pnl_pct: f64, stop: f64, take: f64, trail: f64) -> ExitEvaluation {
        ExitEvaluation {
            symbol: symbol.to_string(),
            qty: 10.0,
            is_short,
            current_price: 100.0,
            entry_price: if is_short { 100.0 / (1.0 - pnl_pct / 100.0) } else { 100.0 / (1.0 + pnl_pct / 100.0) },
            pnl_pct,
            stop_loss: stop,
            take_profit: take,
            trail_pct: trail,
            force_exit: false,
            force_exit_reason: String::new(),
            partial_close: None,
            hold_secs: 120,
            cand_tf: "5m".to_string(),
        }
    }

    fn make_entry_eval(symbol: &str, signal: Signal, score: i32) -> EntryEvaluation {
        EntryEvaluation {
            symbol: symbol.to_string(),
            decision: Some(TradeDecision {
                signal,
                symbol: symbol.to_string(),
                score,
                reason: "test".to_string(),
                fvg: FvgType::None,
                entry_type: "TEST".to_string(),
            }),
            cand_tf: "5m".to_string(),
            vol_factor: 1.0,
        }
    }

    #[test]
    fn exit_eval_force_exit_takes_priority() {
        let mut eval = make_exit_eval("AAPL", false, 2.0, 3.0, 5.0, 2.0);
        eval.force_exit = true;
        eval.force_exit_reason = "ORACLE FLIP".to_string();
        assert!(eval.force_exit);
        assert_eq!(eval.force_exit_reason, "ORACLE FLIP");
    }

    #[test]
    fn exit_eval_stop_loss_triggers_long() {
        let eval = make_exit_eval("AAPL", false, -4.0, 3.0, 5.0, 2.0);
        let should_close = eval.pnl_pct <= -eval.stop_loss;
        assert!(should_close, "PnL {:.1}% below -stop {:.1}% should trigger close", eval.pnl_pct, eval.stop_loss);
    }

    #[test]
    fn exit_eval_take_profit_triggers_long() {
        let eval = make_exit_eval("AAPL", false, 6.0, 3.0, 5.0, 2.0);
        let should_close = eval.pnl_pct >= eval.take_profit;
        assert!(should_close, "PnL {:.1}% above take {:.1}% should trigger close", eval.pnl_pct, eval.take_profit);
    }

    #[test]
    fn exit_eval_no_action_in_range() {
        let eval = make_exit_eval("AAPL", false, 2.0, 3.0, 5.0, 2.0);
        let should_close = eval.pnl_pct <= -eval.stop_loss || eval.pnl_pct >= eval.take_profit;
        assert!(!should_close, "PnL {:.1}% within range should NOT trigger close", eval.pnl_pct);
    }

    #[test]
    fn exit_eval_partial_close_data() {
        let mut eval = make_exit_eval("TSLA", false, 3.0, 2.0, 5.0, 1.5);
        eval.partial_close = Some(PartialCloseData {
            partial_qty: 5.0,
            partial_side: "sell".to_string(),
        });
        assert!(eval.partial_close.is_some());
        let partial = eval.partial_close.as_ref().unwrap();
        assert_eq!(partial.partial_qty, 5.0);
        assert_eq!(partial.partial_side, "sell");
    }

    #[test]
    fn entry_eval_selects_highest_score() {
        let evals = vec![
            make_entry_eval("AAPL", Signal::Buy, 72),
            make_entry_eval("TSLA", Signal::Buy, 85),
            make_entry_eval("MSFT", Signal::Buy, 60),
        ];
        let best = evals.iter()
            .filter_map(|e| {
                let d = e.decision.as_ref()?;
                match d.signal {
                    Signal::Buy => Some((e, d)),
                    _ => None,
                }
            })
            .max_by(|a, b| a.1.score.cmp(&b.1.score));
        assert!(best.is_some());
        let (best_eval, best_decision) = best.unwrap();
        assert_eq!(best_eval.symbol, "TSLA");
        assert_eq!(best_decision.score, 85);
    }

    #[test]
    fn entry_eval_skips_hold_signals() {
        let evals = vec![
            make_entry_eval("AAPL", Signal::Hold, 50),
            make_entry_eval("TSLA", Signal::Hold, 70),
        ];
        let best = evals.iter()
            .filter_map(|e| {
                let d = e.decision.as_ref()?;
                match d.signal {
                    Signal::Buy | Signal::Short => Some((e, d)),
                    _ => None,
                }
            })
            .max_by(|a, b| a.1.score.cmp(&b.1.score));
        assert!(best.is_none(), "Hold signals should be filtered out");
    }

    #[test]
    fn entry_eval_none_decision_filtered() {
        let evals = vec![
            EntryEvaluation {
                symbol: "AAPL".to_string(),
                decision: None,
                cand_tf: "5m".to_string(),
                vol_factor: 1.0,
            },
            make_entry_eval("TSLA", Signal::Buy, 65),
        ];
        let best = evals.iter()
            .filter_map(|e| {
                let d = e.decision.as_ref()?;
                match d.signal {
                    Signal::Buy | Signal::Short => Some((e, d)),
                    _ => None,
                }
            })
            .max_by(|a, b| a.1.score.cmp(&b.1.score));
        assert!(best.is_some());
        assert_eq!(best.unwrap().0.symbol, "TSLA");
    }

    #[test]
    fn exit_eval_short_stop_loss() {
        let eval = make_exit_eval("NVDA", true, -5.0, 3.0, 5.0, 2.0);
        let short_pnl = ((eval.entry_price - eval.current_price) / eval.entry_price) * 100.0;
        let should_close = short_pnl <= -eval.stop_loss;
        assert!(should_close, "Short with adverse move should trigger stop loss");
    }

    fn make_ofi_state(stream_vpin: f64, imbalance_ratio: f64, spread_narrowing: bool) -> crate::l2_heatmap::OfiState {
        crate::l2_heatmap::OfiState {
            stream_vpin,
            imbalance_ratio,
            spread_narrowing,
            ..Default::default()
        }
    }

    #[test]
    fn micro_modifier_spread_narrowing_bonus() {
        let ofi = make_ofi_state(0.3, 0.0, true);
        let adj = apply_microstructure_modifier(&ofi, 0.3, true);
        assert!(adj > 0, "Spread narrowing should give positive bonus, got {}", adj);
        assert_eq!(adj, 5);
    }

    #[test]
    fn micro_modifier_imbalance_aligned_buy() {
        let ofi = make_ofi_state(0.3, 0.5, false);
        let adj = apply_microstructure_modifier(&ofi, 0.3, true);
        assert_eq!(adj, 4, "Buy with positive imbalance > 0.3 should get +4 bonus");
    }

    #[test]
    fn micro_modifier_imbalance_aligned_sell() {
        let ofi = make_ofi_state(0.3, -0.5, false);
        let adj = apply_microstructure_modifier(&ofi, 0.3, false);
        assert_eq!(adj, 4, "Sell with negative imbalance < -0.3 should get +4 bonus");
    }

    #[test]
    fn micro_modifier_imbalance_misaligned() {
        let ofi = make_ofi_state(0.3, -0.5, false);
        let adj = apply_microstructure_modifier(&ofi, 0.3, true);
        assert_eq!(adj, 0, "Buy with negative imbalance should get 0 bonus");
    }

    #[test]
    fn micro_modifier_dual_vpin_hidden_toxicity() {
        let ofi = make_ofi_state(0.7, 0.0, false);
        let candle_vpin = 0.3;
        let adj = apply_microstructure_modifier(&ofi, candle_vpin, true);
        assert!(adj < 0, "Stream VPIN >> candle VPIN should penalize (hidden toxicity), got {}", adj);
        assert_eq!(adj, -8, "Divergence 0.4 > 0.15 threshold should give -8");
    }

    #[test]
    fn micro_modifier_dual_vpin_false_alarm_filtered() {
        let ofi = make_ofi_state(0.3, 0.0, false);
        let candle_vpin = 0.7;
        let adj = apply_microstructure_modifier(&ofi, candle_vpin, true);
        assert!(adj > 0, "Stream VPIN << candle VPIN should bonus (false alarm filtered), got {}", adj);
        assert_eq!(adj, 5, "Divergence -0.4 < -0.15 threshold should give +5");
    }

    #[test]
    fn micro_modifier_stream_vpin_extreme_toxicity() {
        let ofi = make_ofi_state(0.92, 0.0, false);
        let adj = apply_microstructure_modifier(&ofi, 0.92, true);
        assert_eq!(adj, -10, "Stream VPIN > 0.85 should give -10 penalty");
    }

    #[test]
    fn micro_modifier_combined_all_bonuses() {
        let ofi = make_ofi_state(0.25, 0.5, true);
        let candle_vpin = 0.6;
        let adj = apply_microstructure_modifier(&ofi, candle_vpin, true);
        assert_eq!(adj, 5 + 4 + 5, "Narrowing(+5) + imbalance aligned(+4) + false alarm(+5) = +14");
    }

    #[test]
    fn micro_modifier_no_data_zero_adjustment() {
        let ofi = crate::l2_heatmap::OfiState::default();
        let adj = apply_microstructure_modifier(&ofi, 0.0, true);
        assert_eq!(adj, 0, "Default OfiState with matching zero candle VPIN should produce 0 adjustment");
    }

    #[test]
    fn exit_floor_triggers_when_configured() {
        let floor = 0.5_f64;
        let disable_adaptive = false;

        let mut stop = -0.2_f64;
        let mut take = 0.3_f64;
        let mut trail = 0.1_f64;

        if !disable_adaptive && floor > 0.0 {
            if stop.abs() < floor { stop = -floor; }
            if take < floor { take = floor; }
            if trail < floor * 0.5 { trail = floor * 0.5; }
        }

        assert!(
            (stop - (-0.5)).abs() < 1e-9,
            "Stop -0.2 should be floored to -0.5, got {stop}"
        );
        assert!(
            (take - 0.5).abs() < 1e-9,
            "Take 0.3 should be floored to 0.5, got {take}"
        );
        assert!(
            (trail - 0.25).abs() < 1e-9,
            "Trail 0.1 should be floored to 0.25, got {trail}"
        );

        let mut stop2 = -1.0_f64;
        let mut take2 = 1.0_f64;
        let mut trail2 = 0.5_f64;
        if !disable_adaptive && floor > 0.0 {
            if stop2.abs() < floor { stop2 = -floor; }
            if take2 < floor { take2 = floor; }
            if trail2 < floor * 0.5 { trail2 = floor * 0.5; }
        }
        assert!(
            (stop2 - (-1.0)).abs() < 1e-9,
            "Stop -1.0 (already larger than floor) should stay -1.0, got {stop2}"
        );
        assert!(
            (take2 - 1.0).abs() < 1e-9,
            "Take 1.0 (already larger than floor) should stay 1.0, got {take2}"
        );
        assert!(
            (trail2 - 0.5).abs() < 1e-9,
            "Trail 0.5 (already larger than floor*0.5) should stay 0.5, got {trail2}"
        );
    }

    #[test]
    fn max_hold_decay_uses_direct_factor() {
        let tighten = 0.3_f64;
        let progress = 0.5_f64;
        let factor = 1.0 - progress * tighten;
        let factor = factor.clamp(0.2, 1.0);
        assert!(
            (factor - 0.85).abs() < 1e-9,
            "0.3 tighten at 50% progress should give factor 0.85, got {factor}"
        );

        let full_progress = 1.0_f64;
        let factor_full = (1.0 - full_progress * tighten).clamp(0.2, 1.0);
        assert!(
            (factor_full - 0.7).abs() < 1e-9,
            "0.3 tighten at 100% progress should give factor 0.7, got {factor_full}"
        );

        let aggressive = 0.8_f64;
        let factor_agg = (1.0 - full_progress * aggressive).clamp(0.2, 1.0);
        assert!(
            (factor_agg - 0.2).abs() < 1e-9,
            "0.8 tighten at 100% progress should clamp to 0.2, got {factor_agg}"
        );
    }
}

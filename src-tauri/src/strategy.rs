use crate::app_state::{AppState, Candle, EntryThreshold, GateMode};
use crate::indicators::Indicators;
use crate::titan_oracle::MarketRegime;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Signal {
    Buy,
    Short,
    Hold,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum FvgType {
    Bullish,
    Bearish,
    None,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TradeDecision {
    pub signal: Signal,
    pub symbol: String,
    pub score: i32,
    pub reason: String,
    pub fvg: FvgType,
    pub entry_type: String,
}

fn calculate_rvol(candles: &[Candle], window: usize) -> f64 {
    if candles.len() < window + 1 {
        return 1.0;
    }
    let last_idx = candles.len() - 1;
    let current_vol = candles[last_idx].volume;

    let start_idx = last_idx.saturating_sub(window);
    let sum_vol: f64 = candles[start_idx..last_idx].iter().map(|c| c.volume).sum();
    let avg_vol = sum_vol / window as f64;

    if avg_vol > 0.0 {
        current_vol / avg_vol
    } else {
        1.0
    }
}

fn lookup_entry_threshold(
    map: &HashMap<String, EntryThreshold>,
    regime: MarketRegime,
    aggressive: bool,
) -> EntryThreshold {
    let (base_key, aggro_key) = match regime {
        MarketRegime::Trending => ("trending", "trending_aggro"),
        MarketRegime::MeanReverting => ("mean_rev", "mean_rev_aggro"),
        MarketRegime::Chaotic => ("chaotic", "chaotic"),
        MarketRegime::Unknown => ("unknown", "unknown_aggro"),
    };

    let key = if aggressive && regime != MarketRegime::Chaotic {
        aggro_key
    } else {
        base_key
    };

    map.get(key).cloned().unwrap_or(EntryThreshold {
        long_threshold: 50.0,
        short_threshold: -50.0,
        conf_threshold: 0.30,
    })
}

pub fn detect_smart_money_fvg(candles: &[Candle]) -> FvgType {
    if candles.len() < 3 {
        return FvgType::None;
    }

    let c1 = &candles[candles.len() - 3];
    let c2 = &candles[candles.len() - 2];
    let c3 = &candles[candles.len() - 1];

    if c2.close > c2.open {
        let gap_low = c1.high;
        let gap_high = c3.low;
        if gap_high > gap_low {
            return FvgType::Bullish;
        }
    }

    if c2.close < c2.open {
        let gap_high = c1.low;
        let gap_low = c3.high;
        if gap_low < gap_high {
            return FvgType::Bearish;
        }
    }

    FvgType::None
}

// ═══════════════════════════════════════════════════════════════════
//  CLASSICAL INDICATOR CONFIRMATION (Phase 7D)
// ═══════════════════════════════════════════════════════════════════
pub fn compute_classical_confirmation(
    indicators: &Indicators,
    is_long: bool,
    settings: &AppState,
) -> f64 {
    if !settings.use_classical_filters {
        return 0.0;
    }
    let cb = &settings.classical_bonuses;
    let mut score = 0.0f64;

    if let Some(rsi) = indicators.rsi {
        if is_long && rsi < settings.rsi_overbought {
            score += cb.rsi_bonus;
        }
        if !is_long && rsi > settings.rsi_oversold {
            score += cb.rsi_bonus;
        }
    }

    if settings.require_ema_alignment {
        if let (Some(ema20), Some(ema50)) = (indicators.ema_20, indicators.ema_50) {
            if is_long && ema20 > ema50 {
                score += cb.ema_alignment_bonus;
            }
            if !is_long && ema20 < ema50 {
                score += cb.ema_alignment_bonus;
            }
        }
    }

    if let Some(adx) = indicators.adx {
        if adx > settings.adx_trend_threshold {
            score += cb.adx_bonus;
        }
    }

    if let (Some(upper), Some(lower), Some(mid)) =
        (indicators.bb_upper, indicators.bb_lower, indicators.bb_mid)
    {
        if mid > 0.0 {
            let width = (upper - lower) / mid;
            if width < settings.score_modifiers.bb_squeeze_width {
                score += cb.bb_squeeze_bonus;
            }
        }
    }

    if let Some(vwap) = indicators.vwap {
        if let Some(ema9) = indicators.ema_9 {
            if is_long && ema9 > vwap {
                score += cb.vwap_bonus;
            }
            if !is_long && ema9 < vwap {
                score += cb.vwap_bonus;
            }
        }
    }

    score.min(cb.cap) * settings.classical_weight
}

// ═══════════════════════════════════════════════════════════════════
//  TA ENTRY GATES — Pre-entry filters with Soft/Hard modes (RF-B)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct TaGateResult {
    pub blocked: bool,
    pub penalty: f64,
    pub failed_gates: Vec<&'static str>,
}

/// Evaluate 6 TA gates against current indicators and trade direction.
/// Returns whether the entry should be blocked (Hard gate failed) and
/// the total score penalty from Soft gates.
pub fn check_ta_gates(
    indicators: &Indicators,
    is_long: bool,
    settings: &AppState,
) -> TaGateResult {
    let cfg = &settings.ta_gates;
    let penalty_per_gate = settings.ta_gate_soft_penalty;
    let mut total_penalty = 0.0;
    let mut blocked = false;
    let mut failed = Vec::new();

    // Gate 1: RSI — block/penalize when RSI pushes into overbought (long) or oversold (short)
    if cfg.rsi_gate != GateMode::Off {
        if let Some(rsi) = indicators.rsi {
            let rsi_failed = (is_long && rsi > settings.rsi_overbought)
                || (!is_long && rsi < settings.rsi_oversold);
            if rsi_failed {
                match cfg.rsi_gate {
                    GateMode::Hard => { blocked = true; failed.push("RSI"); }
                    GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("RSI"); }
                    GateMode::Off => {}
                }
            }
        }
    }

    // Gate 2: EMA Alignment — EMA9 > EMA20 > EMA50 (long) or inverse (short)
    if cfg.ema_alignment_gate != GateMode::Off {
        if let (Some(e9), Some(e20), Some(e50)) =
            (indicators.ema_9, indicators.ema_20, indicators.ema_50)
        {
            let aligned = if is_long { e9 > e20 && e20 > e50 } else { e9 < e20 && e20 < e50 };
            if !aligned {
                match cfg.ema_alignment_gate {
                    GateMode::Hard => { blocked = true; failed.push("EMA_ALIGN"); }
                    GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("EMA_ALIGN"); }
                    GateMode::Off => {}
                }
            }
        }
    }

    // Gate 3: ADX — require sufficient trend strength
    if cfg.adx_gate != GateMode::Off {
        if let Some(adx) = indicators.adx {
            if adx < settings.adx_trend_threshold {
                match cfg.adx_gate {
                    GateMode::Hard => { blocked = true; failed.push("ADX"); }
                    GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("ADX"); }
                    GateMode::Off => {}
                }
            }
        }
    }

    // Gate 4: Bollinger Squeeze — tight bands indicate consolidation, avoid entries
    if cfg.bb_squeeze_gate != GateMode::Off {
        if let (Some(upper), Some(lower), Some(mid)) =
            (indicators.bb_upper, indicators.bb_lower, indicators.bb_mid)
        {
            if mid > 0.0 {
                let width = (upper - lower) / mid;
                if width < settings.score_modifiers.bb_squeeze_width {
                    match cfg.bb_squeeze_gate {
                        GateMode::Hard => { blocked = true; failed.push("BB_SQUEEZE"); }
                        GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("BB_SQUEEZE"); }
                        GateMode::Off => {}
                    }
                }
            }
        }
    }

    // Gate 5: MACD — histogram must align with trade direction
    if cfg.macd_gate != GateMode::Off {
        if let Some(hist) = indicators.macd_hist {
            let macd_aligned = (is_long && hist > 0.0) || (!is_long && hist < 0.0);
            if !macd_aligned {
                match cfg.macd_gate {
                    GateMode::Hard => { blocked = true; failed.push("MACD"); }
                    GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("MACD"); }
                    GateMode::Off => {}
                }
            }
        }
    }

    // Gate 6: Stochastic RSI — avoid entry when StochRSI is in the opposite extreme
    if cfg.stoch_rsi_gate != GateMode::Off {
        if let Some(stoch) = indicators.stoch_rsi {
            let sm = &settings.score_modifiers;
            let stoch_extreme = (is_long && stoch > sm.stoch_rsi_overbought)
                || (!is_long && stoch < sm.stoch_rsi_oversold);
            if stoch_extreme {
                match cfg.stoch_rsi_gate {
                    GateMode::Hard => { blocked = true; failed.push("STOCH_RSI"); }
                    GateMode::Soft => { total_penalty += penalty_per_gate; failed.push("STOCH_RSI"); }
                    GateMode::Off => {}
                }
            }
        }
    }

    TaGateResult { blocked, penalty: total_penalty, failed_gates: failed }
}

// ═══════════════════════════════════════════════════════════════════
//  SCORING ENGINE — Regime-Adaptive with VPIN + RVOL
// ═══════════════════════════════════════════════════════════════════
pub fn calculate_score(candles: &[Candle], indicators: &Indicators, fvg: &FvgType, app_state_: &AppState) -> i32 {
    let mut score: i32 = app_state_.strategy_base_score as i32;
    let sm = &app_state_.score_modifiers;

    let rvol = calculate_rvol(candles, app_state_.rvol_window as usize);
    let regime = indicators.oracle_regime.unwrap_or(MarketRegime::Unknown);

    if let Some(oracle) = indicators.oracle_score {
        let confidence = indicators.oracle_confidence.unwrap_or(0.5);

        let chaos_threshold = match regime {
            MarketRegime::Trending => app_state_.chaos_confidence_kill,
            MarketRegime::MeanReverting => app_state_.chaos_confidence_kill * sm.chaos_conf_mult_mean_rev,
            MarketRegime::Chaotic => app_state_.chaos_confidence_kill * sm.chaos_conf_mult_chaotic,
            MarketRegime::Unknown => app_state_.chaos_confidence_kill * sm.chaos_conf_mult_unknown,
        };

        if confidence < chaos_threshold {
            score += sm.chaos_kill_penalty;
            return score.max(0);
        }

        let regime_multiplier = match regime {
            MarketRegime::Trending => app_state_.chaos_regime_mult,
            MarketRegime::MeanReverting => app_state_.chaos_regime_mult * sm.regime_mult_mean_rev,
            MarketRegime::Chaotic => app_state_.chaos_regime_mult * sm.regime_mult_chaotic,
            MarketRegime::Unknown => app_state_.chaos_regime_mult * sm.regime_mult_unknown,
        };
        let oracle_contribution = (oracle * regime_multiplier * confidence) as i32;
        score += oracle_contribution;
    } else {
        return sm.chaos_kill_fallback;
    }

    if let Some(vpin) = indicators.vpin {
        if vpin > app_state_.vpin_gate_scalping {
            score += sm.vpin_penalty_high;
        } else if vpin > app_state_.vpin_gate_normal {
            score += sm.vpin_penalty_medium;
        }
    }

    match fvg {
        FvgType::Bullish => score += sm.fvg_bullish_bonus,
        FvgType::Bearish => score += sm.fvg_bearish_penalty,
        _ => {}
    }

    if rvol > sm.rvol_tier1_threshold {
        score += sm.rvol_tier1_bonus;
    }
    if rvol > sm.rvol_tier2_threshold {
        score += sm.rvol_tier2_bonus;
    }

    score
}

// ═══════════════════════════════════════════════════════════════════
//  ENTRY CHECK — Regime-Adaptive with VPIN + RVOL Gates
// ═══════════════════════════════════════════════════════════════════
#[tracing::instrument(skip(candles, indicators, app_state_))]
pub fn check_entry(
    symbol: &str,
    candles: &[Candle],
    indicators: &Indicators,
    app_state_: &AppState,
) -> Option<TradeDecision> {
    if candles.is_empty() {
        return None;
    }

    let fvg = detect_smart_money_fvg(candles);
    let score = calculate_score(candles, indicators, &fvg, app_state_);

    let oracle_score = indicators.oracle_score.unwrap_or(0.0);
    let oracle_confidence = indicators.oracle_confidence.unwrap_or(0.0);
    let regime = indicators.oracle_regime.unwrap_or(MarketRegime::Unknown);
    let vpin = indicators.vpin.unwrap_or(0.5);
    let rvol = calculate_rvol(candles, app_state_.rvol_window as usize);

    let is_scalping = match app_state_.strategy_mode_name.as_str() {
        "MATH_ORACLE" | "ORACLE" => app_state_.scalping_oracle,
        "GPU_NEXUS" | "NEXUS" | "GPU" => app_state_.scalping_gpu,
        _ => app_state_.scalping_fusion,
    };

    // === GATE 1: VPIN Toxicity Filter ===
    let vpin_limit = if is_scalping { app_state_.vpin_gate_scalping } else { app_state_.vpin_gate_normal };
    if vpin > vpin_limit {
        return None;
    }

    // === GATE 2: Minimum Volume ===
    if rvol < app_state_.min_rvol_threshold {
        return None;
    }

    // === Regime-Adaptive Thresholds (from configurable matrix) ===
    let thresh_map = if is_scalping {
        &app_state_.entry_thresholds_scalp
    } else {
        &app_state_.entry_thresholds_normal
    };
    let et = lookup_entry_threshold(thresh_map, regime, app_state_.aggressive_mode);
    let (long_threshold, short_threshold, conf_threshold) =
        (et.long_threshold, et.short_threshold, et.conf_threshold);

    // RVOL boost: high volume can slightly relax thresholds
    let rvol_adj = if rvol > app_state_.rvol_relax_threshold { app_state_.rvol_relax_value } else { 0.0 };

    let mut signal = Signal::Hold;
    let mut reason = String::new();
    let mut entry_type = "NONE".to_string();

    // ═══════════════════════════════════════════════════════════════
    //  LONG LOGIC
    // ═══════════════════════════════════════════════════════════════
    if oracle_score > (long_threshold - rvol_adj) && oracle_confidence > conf_threshold {
        signal = Signal::Buy;

        let regime_tag = match regime {
            MarketRegime::Trending => "TREND",
            MarketRegime::MeanReverting => "RANGE",
            MarketRegime::Chaotic => "CHAOS",
            MarketRegime::Unknown => "UNK",
        };

        let sm = &app_state_.score_modifiers;
        if oracle_score > sm.entry_label_strong {
            entry_type = format!("BUY STRONG [{}]", regime_tag);
        } else if oracle_score > sm.entry_label_mild {
            entry_type = format!("BUY [{}]", regime_tag);
        } else {
            entry_type = format!("BUY MILD [{}]", regime_tag);
        }

        if let FvgType::Bullish = fvg {
            entry_type = format!("BUY+FVG [{}]", regime_tag);
        }

        reason = format!(
            "Oracle: {:.1} | Conf: {:.2} | Regime: {:?} | VPIN: {:.2} | RVOL: {:.1} | Score: {} | {}",
            oracle_score, oracle_confidence, regime, vpin, rvol, score, entry_type
        );
    }

    // ═══════════════════════════════════════════════════════════════
    //  SHORT LOGIC
    // ═══════════════════════════════════════════════════════════════
    if matches!(signal, Signal::Hold)
        && app_state_.allow_short
        && oracle_score < (short_threshold + rvol_adj)
        && oracle_confidence > conf_threshold
    {
        signal = Signal::Short;

        let regime_tag = match regime {
            MarketRegime::Trending => "TREND",
            MarketRegime::MeanReverting => "RANGE",
            MarketRegime::Chaotic => "CHAOS",
            MarketRegime::Unknown => "UNK",
        };

        let sm = &app_state_.score_modifiers;
        if oracle_score < -sm.entry_label_strong {
            entry_type = format!("SHORT STRONG [{}]", regime_tag);
        } else if oracle_score < -sm.entry_label_mild {
            entry_type = format!("SHORT [{}]", regime_tag);
        } else {
            entry_type = format!("SHORT MILD [{}]", regime_tag);
        }

        if let FvgType::Bearish = fvg {
            entry_type = format!("SHORT+FVG [{}]", regime_tag);
        }

        reason = format!(
                "Oracle: {:.1} | Conf: {:.2} | Regime: {:?} | VPIN: {:.2} | RVOL: {:.1} | Score: {} | {}",
                oracle_score, oracle_confidence, regime, vpin, rvol, score, entry_type
            );
    }

    if matches!(signal, Signal::Hold) {
        return None;
    }

    let is_long = matches!(signal, Signal::Buy);

    // TA Gates: Hard gates block entry, Soft gates reduce score
    let gate_result = check_ta_gates(indicators, is_long, app_state_);
    if gate_result.blocked {
        tracing::debug!(
            component = "strategy",
            symbol = symbol,
            gates = ?gate_result.failed_gates,
            "Entry blocked by TA hard gate(s)"
        );
        return None;
    }

    let classical_bonus =
        compute_classical_confirmation(indicators, is_long, app_state_);
    let score = score + classical_bonus as i32 - gate_result.penalty as i32;

    Some(TradeDecision {
        signal,
        symbol: symbol.to_string(),
        score,
        reason,
        fvg,
        entry_type,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  MTF CONFIRMATION GATE (S15) — Higher-timeframe trend alignment
// ═══════════════════════════════════════════════════════════════════

/// Map a lower timeframe to its corresponding higher timeframe for MTF confirmation.
pub fn ltf_to_htf(ltf: &str) -> Option<&'static str> {
    match ltf {
        "1m" => Some("5m"),
        "3m" => Some("15m"),
        "5m" => Some("15m"),
        "10m" => Some("1h"),
        "15m" => Some("1h"),
        "30m" => Some("4h"),
        "1h" => Some("4h"),
        "4h" => Some("1d"),
        _ => None,
    }
}

/// Determine if a given direction (buy=true) is aligned with the oracle on the
/// higher timeframe. Returns `(aligned, htf_score)`.
pub fn is_htf_aligned(
    htf_candles: &[crate::app_state::Candle],
    oracle_weights: &crate::titan_oracle::OracleWeights,
    is_buy: bool,
) -> (bool, f64) {
    if htf_candles.len() < 60 {
        return (true, 0.0);
    }
    let snap = crate::titan_oracle::calculate(htf_candles, oracle_weights);
    let score = snap.oracle_score;
    let conf = snap.confidence;
    let min_conf = 0.15;
    let min_score = 5.0;
    if conf < min_conf || score.abs() < min_score {
        return (true, score);
    }
    let aligned = if is_buy { score > 0.0 } else { score < 0.0 };
    (aligned, score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{GateMode, TaGatesConfig, TaGatePreset};

    fn make_indicators(overrides: impl FnOnce(&mut Indicators)) -> Indicators {
        let mut ind = Indicators {
            ema_9: Some(105.0),
            ema_20: Some(103.0),
            ema_50: Some(100.0),
            ema_200: Some(95.0),
            rsi: Some(55.0),
            macd: Some(1.0),
            macd_signal: Some(0.5),
            macd_hist: Some(0.5),
            bb_upper: Some(110.0),
            bb_lower: Some(90.0),
            bb_mid: Some(100.0),
            adx: Some(30.0),
            vwap: Some(101.0),
            stoch_rsi: Some(50.0),
            stoch_k: Some(50.0),
            stoch_d: Some(50.0),
            oracle_score: Some(70.0),
            oracle_confidence: Some(0.8),
            oracle_regime: Some(MarketRegime::Trending),
            vpin: Some(0.3),
            vpin_buy_pressure: Some(0.6),
            vpin_toxicity: Some(false),
            kalman_gain: None,
            price_surprise: None,
            changepoint_prob: None,
            run_length: None,
            lyapunov_exponent: None,
        };
        overrides(&mut ind);
        ind
    }

    fn make_settings(gates: TaGatesConfig) -> AppState {
        let mut s = AppState::default();
        s.ta_gates = gates;
        s.ta_gate_soft_penalty = 15.0;
        s.rsi_overbought = 70.0;
        s.rsi_oversold = 30.0;
        s.adx_trend_threshold = 25.0;
        s
    }

    #[test]
    fn all_gates_off_no_effect() {
        let ind = make_indicators(|_| {});
        let settings = make_settings(TaGatesConfig::default());
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert_eq!(result.penalty, 0.0);
        assert!(result.failed_gates.is_empty());
    }

    #[test]
    fn rsi_hard_blocks_overbought_long() {
        let ind = make_indicators(|i| { i.rsi = Some(75.0); });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(result.blocked);
        assert!(result.failed_gates.contains(&"RSI"));
    }

    #[test]
    fn rsi_hard_allows_normal_long() {
        let ind = make_indicators(|i| { i.rsi = Some(65.0); });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert!(result.failed_gates.is_empty());
    }

    #[test]
    fn rsi_soft_penalizes_overbought_long() {
        let ind = make_indicators(|i| { i.rsi = Some(75.0); });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Soft,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert_eq!(result.penalty, 15.0);
        assert!(result.failed_gates.contains(&"RSI"));
    }

    #[test]
    fn rsi_hard_blocks_oversold_short() {
        let ind = make_indicators(|i| { i.rsi = Some(25.0); });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, false, &settings);
        assert!(result.blocked);
        assert!(result.failed_gates.contains(&"RSI"));
    }

    #[test]
    fn ema_hard_blocks_misaligned_long() {
        let ind = make_indicators(|i| {
            i.ema_9 = Some(99.0);
            i.ema_20 = Some(103.0);
            i.ema_50 = Some(100.0);
        });
        let settings = make_settings(TaGatesConfig {
            ema_alignment_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(result.blocked);
        assert!(result.failed_gates.contains(&"EMA_ALIGN"));
    }

    #[test]
    fn adx_soft_penalizes_weak_trend() {
        let ind = make_indicators(|i| { i.adx = Some(20.0); });
        let settings = make_settings(TaGatesConfig {
            adx_gate: GateMode::Soft,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert_eq!(result.penalty, 15.0);
        assert!(result.failed_gates.contains(&"ADX"));
    }

    #[test]
    fn macd_hard_blocks_negative_hist_long() {
        let ind = make_indicators(|i| { i.macd_hist = Some(-0.5); });
        let settings = make_settings(TaGatesConfig {
            macd_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(result.blocked);
        assert!(result.failed_gates.contains(&"MACD"));
    }

    #[test]
    fn stoch_rsi_hard_blocks_extreme_long() {
        let ind = make_indicators(|i| { i.stoch_rsi = Some(85.0); });
        let settings = make_settings(TaGatesConfig {
            stoch_rsi_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(result.blocked);
        assert!(result.failed_gates.contains(&"STOCH_RSI"));
    }

    #[test]
    fn bb_squeeze_soft_penalizes_tight_bands() {
        let ind = make_indicators(|i| {
            i.bb_upper = Some(100.8);
            i.bb_lower = Some(99.2);
            i.bb_mid = Some(100.0);
        });
        let settings = make_settings(TaGatesConfig {
            bb_squeeze_gate: GateMode::Soft,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert_eq!(result.penalty, 15.0);
        assert!(result.failed_gates.contains(&"BB_SQUEEZE"));
    }

    #[test]
    fn multiple_soft_gates_accumulate_penalty() {
        let ind = make_indicators(|i| {
            i.rsi = Some(75.0);
            i.adx = Some(15.0);
            i.macd_hist = Some(-0.3);
        });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Soft,
            adx_gate: GateMode::Soft,
            macd_gate: GateMode::Soft,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked);
        assert_eq!(result.penalty, 45.0);
        assert_eq!(result.failed_gates.len(), 3);
    }

    #[test]
    fn preset_strict_all_hard() {
        let cfg = TaGatesConfig::from_preset(&TaGatePreset::Strict);
        assert_eq!(cfg.rsi_gate, GateMode::Hard);
        assert_eq!(cfg.ema_alignment_gate, GateMode::Hard);
        assert_eq!(cfg.adx_gate, GateMode::Hard);
        assert_eq!(cfg.bb_squeeze_gate, GateMode::Hard);
        assert_eq!(cfg.macd_gate, GateMode::Hard);
        assert_eq!(cfg.stoch_rsi_gate, GateMode::Hard);
    }

    #[test]
    fn preset_balanced_rsi_adx_soft_rest_off() {
        let cfg = TaGatesConfig::from_preset(&TaGatePreset::Balanced);
        assert_eq!(cfg.rsi_gate, GateMode::Soft);
        assert_eq!(cfg.adx_gate, GateMode::Soft);
        assert_eq!(cfg.ema_alignment_gate, GateMode::Off);
        assert_eq!(cfg.bb_squeeze_gate, GateMode::Off);
        assert_eq!(cfg.macd_gate, GateMode::Off);
        assert_eq!(cfg.stoch_rsi_gate, GateMode::Off);
    }

    #[test]
    fn preset_oracle_only_all_off() {
        let cfg = TaGatesConfig::from_preset(&TaGatePreset::OracleOnly);
        assert_eq!(cfg.rsi_gate, GateMode::Off);
        assert_eq!(cfg.ema_alignment_gate, GateMode::Off);
        assert_eq!(cfg.adx_gate, GateMode::Off);
        assert_eq!(cfg.bb_squeeze_gate, GateMode::Off);
        assert_eq!(cfg.macd_gate, GateMode::Off);
        assert_eq!(cfg.stoch_rsi_gate, GateMode::Off);
    }

    #[test]
    fn rsi_exact_on_threshold_does_not_trigger_long() {
        let ind = make_indicators(|i| { i.rsi = Some(70.0); });
        let settings = make_settings(TaGatesConfig {
            rsi_gate: GateMode::Hard,
            ..TaGatesConfig::default()
        });
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked, "RSI exactly at threshold should not trigger gate");
    }

    #[test]
    fn none_indicators_skip_gate_gracefully() {
        let ind = make_indicators(|i| {
            i.rsi = None;
            i.ema_9 = None;
            i.adx = None;
            i.macd_hist = None;
            i.stoch_rsi = None;
            i.bb_upper = None;
        });
        let settings = make_settings(TaGatesConfig::from_preset(&TaGatePreset::Strict));
        let result = check_ta_gates(&ind, true, &settings);
        assert!(!result.blocked, "Missing indicators should not block entry");
        assert_eq!(result.penalty, 0.0);
    }

    // ═══════════════════════════════════════════════════════════════
    //  N4: Strategy Thresholds tests
    // ═══════════════════════════════════════════════════════════════

    use crate::app_state::{
        ScoreModifiers, ClassicalBonuses, EntryThreshold,
        default_entry_thresholds_scalp, default_entry_thresholds_normal,
        validate_strategy_thresholds,
    };

    #[test]
    fn score_modifiers_defaults_match_hardcoded() {
        let sm = ScoreModifiers::default();
        assert_eq!(sm.chaos_kill_penalty, -50);
        assert_eq!(sm.chaos_kill_fallback, 50);
        assert_eq!(sm.chaos_conf_mult_mean_rev, 1.5);
        assert_eq!(sm.chaos_conf_mult_chaotic, 2.5);
        assert_eq!(sm.chaos_conf_mult_unknown, 1.5);
        assert_eq!(sm.regime_mult_mean_rev, 0.82);
        assert_eq!(sm.regime_mult_chaotic, 0.55);
        assert_eq!(sm.regime_mult_unknown, 0.73);
        assert_eq!(sm.vpin_penalty_high, -20);
        assert_eq!(sm.vpin_penalty_medium, -10);
        assert_eq!(sm.fvg_bullish_bonus, 15);
        assert_eq!(sm.fvg_bearish_penalty, -15);
        assert_eq!(sm.rvol_tier1_threshold, 2.0);
        assert_eq!(sm.rvol_tier1_bonus, 8);
        assert_eq!(sm.rvol_tier2_threshold, 4.0);
        assert_eq!(sm.rvol_tier2_bonus, 12);
        assert_eq!(sm.stoch_rsi_overbought, 80.0);
        assert_eq!(sm.stoch_rsi_oversold, 20.0);
        assert_eq!(sm.bb_squeeze_width, 0.02);
        assert_eq!(sm.entry_label_strong, 80.0);
        assert_eq!(sm.entry_label_mild, 60.0);
    }

    #[test]
    fn classical_bonuses_defaults_match_hardcoded() {
        let cb = ClassicalBonuses::default();
        assert_eq!(cb.rsi_bonus, 10.0);
        assert_eq!(cb.ema_alignment_bonus, 15.0);
        assert_eq!(cb.adx_bonus, 10.0);
        assert_eq!(cb.bb_squeeze_bonus, 5.0);
        assert_eq!(cb.vwap_bonus, 10.0);
        assert_eq!(cb.cap, 50.0);
    }

    #[test]
    fn entry_thresholds_defaults_match_hardcoded() {
        let scalp = default_entry_thresholds_scalp();
        let normal = default_entry_thresholds_normal();

        let st = scalp.get("trending_aggro").unwrap();
        assert_eq!((st.long_threshold, st.short_threshold, st.conf_threshold), (15.0, -15.0, 0.12));

        let st = scalp.get("trending").unwrap();
        assert_eq!((st.long_threshold, st.short_threshold, st.conf_threshold), (22.0, -22.0, 0.18));

        let st = scalp.get("chaotic").unwrap();
        assert_eq!((st.long_threshold, st.short_threshold, st.conf_threshold), (50.0, -50.0, 0.38));

        let nt = normal.get("trending").unwrap();
        assert_eq!((nt.long_threshold, nt.short_threshold, nt.conf_threshold), (35.0, -35.0, 0.25));

        let nt = normal.get("chaotic").unwrap();
        assert_eq!((nt.long_threshold, nt.short_threshold, nt.conf_threshold), (70.0, -70.0, 0.55));

        assert_eq!(scalp.len(), 7);
        assert_eq!(normal.len(), 7);
    }

    #[test]
    fn calculate_score_with_custom_fvg_bonus() {
        let candles = make_candle_vec(50, 100.0);
        let ind = make_indicators(|i| {
            i.oracle_score = Some(40.0);
            i.oracle_confidence = Some(0.8);
            i.oracle_regime = Some(MarketRegime::Trending);
            i.vpin = Some(0.3);
        });
        let mut settings = AppState::default();
        let score_default = calculate_score(&candles, &ind, &FvgType::Bullish, &settings);

        settings.score_modifiers.fvg_bullish_bonus = 30;
        let score_boosted = calculate_score(&candles, &ind, &FvgType::Bullish, &settings);

        assert_eq!(score_boosted - score_default, 15, "extra 15 from doubled FVG bonus");
    }

    #[test]
    fn check_entry_with_custom_thresholds() {
        let candles = make_candle_vec(50, 100.0);
        let ind = make_indicators(|i| {
            i.oracle_score = Some(18.0);
            i.oracle_confidence = Some(0.20);
            i.oracle_regime = Some(MarketRegime::Trending);
            i.vpin = Some(0.3);
        });

        let mut settings = AppState::default();
        settings.scalping_oracle = true;
        settings.strategy_mode_name = "ORACLE".to_string();
        let result_default = check_entry("AAPL", &candles, &ind, &settings);
        assert!(result_default.is_none(), "default threshold should block this weak signal");

        settings.entry_thresholds_scalp.insert(
            "trending".into(),
            EntryThreshold { long_threshold: 10.0, short_threshold: -10.0, conf_threshold: 0.10 },
        );
        let result_relaxed = check_entry("AAPL", &candles, &ind, &settings);
        assert!(result_relaxed.is_some(), "relaxed threshold should allow this signal");
    }

    #[test]
    fn score_modifiers_serde_roundtrip() {
        let sm = ScoreModifiers {
            fvg_bullish_bonus: 25,
            rvol_tier1_bonus: 15,
            ..ScoreModifiers::default()
        };
        let json = serde_json::to_string(&sm).unwrap();
        let deserialized: ScoreModifiers = serde_json::from_str(&json).unwrap();
        assert_eq!(sm, deserialized);
    }

    #[test]
    fn validation_catches_inverted_thresholds() {
        let mut scalp = default_entry_thresholds_scalp();
        scalp.insert("bad".into(), EntryThreshold {
            long_threshold: -10.0,
            short_threshold: 10.0,
            conf_threshold: 0.30,
        });
        let normal = default_entry_thresholds_normal();
        let warnings = validate_strategy_thresholds(&scalp, &normal, &ScoreModifiers::default());
        assert!(warnings.iter().any(|w| w.contains("long_threshold") && w.contains("positive")));
        assert!(warnings.iter().any(|w| w.contains("short_threshold") && w.contains("negative")));
    }

    #[test]
    fn validation_catches_scalp_stricter_than_normal() {
        let mut scalp = default_entry_thresholds_scalp();
        scalp.insert("trending".into(), EntryThreshold {
            long_threshold: 99.0,
            short_threshold: -99.0,
            conf_threshold: 0.90,
        });
        let normal = default_entry_thresholds_normal();
        let warnings = validate_strategy_thresholds(&scalp, &normal, &ScoreModifiers::default());
        assert!(warnings.iter().any(|w| w.contains("scalping") && w.contains("more restrictive")));
    }

    fn make_candle_vec(n: usize, base_price: f64) -> Vec<Candle> {
        (0..n).map(|i| Candle {
            time: 1700000000 + (i as i64) * 60,
            open: base_price + (i as f64) * 0.1,
            high: base_price + (i as f64) * 0.1 + 0.5,
            low: base_price + (i as f64) * 0.1 - 0.5,
            close: base_price + (i as f64) * 0.1 + 0.2,
            volume: 10000.0 + (i as f64) * 100.0,
        }).collect()
    }

    // ═══ S15: MTF Helper Tests ═══

    #[test]
    fn ltf_to_htf_mapping() {
        assert_eq!(ltf_to_htf("1m"), Some("5m"));
        assert_eq!(ltf_to_htf("5m"), Some("15m"));
        assert_eq!(ltf_to_htf("15m"), Some("1h"));
        assert_eq!(ltf_to_htf("1h"), Some("4h"));
        assert_eq!(ltf_to_htf("4h"), Some("1d"));
        assert_eq!(ltf_to_htf("1d"), None);
        assert_eq!(ltf_to_htf("bogus"), None);
    }

    #[test]
    fn htf_aligned_insufficient_data() {
        let candles = make_candle_vec(10, 100.0);
        let w = crate::titan_oracle::OracleWeights::default();
        let (aligned, _score) = is_htf_aligned(&candles, &w, true);
        assert!(aligned, "insufficient data should default to aligned");
    }

    #[test]
    fn htf_aligned_with_enough_data() {
        let candles = make_candle_vec(200, 100.0);
        let w = crate::titan_oracle::OracleWeights::default();
        let (_, score) = is_htf_aligned(&candles, &w, true);
        assert!(score.is_finite(), "score should be finite");
    }

    // ═══ S15: Max-Hold Decay Settings Test ═══

    #[test]
    fn max_hold_decay_settings_defaults() {
        let settings = AppState::default();
        assert!(settings.max_hold_decay_enabled);
        assert!((settings.max_hold_decay_start_pct - 0.5).abs() < 0.001);
        assert!((settings.max_hold_decay_trail_tighten - 0.3).abs() < 0.01);
        assert_eq!(settings.max_hold_secs, 3600);
    }

    #[test]
    fn mtf_gate_settings_defaults() {
        let settings = AppState::default();
        assert!(settings.mtf_gate_enabled);
        assert_eq!(settings.mtf_gate_mode, "soft");
        assert!((settings.mtf_gate_penalty - 15.0).abs() < 0.1);
        assert_eq!(settings.mtf_gate_htf, "15m");
    }

    #[test]
    fn hhi_manual_settings_defaults() {
        let settings = AppState::default();
        assert!(settings.hhi_gate_manual_orders);
        assert!((settings.max_concentration_hhi_manual - 0.35).abs() < 0.01);
    }
}

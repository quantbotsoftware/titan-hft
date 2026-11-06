use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use tauri::Manager;
use tokio_util::sync::CancellationToken;

use crate::app_state::Candle;
use crate::error::TitanError;
#[cfg(feature = "gpu")]
use crate::gpu;
use crate::state::{normalize_symbol, normalize_timeframe_internal, StrategyMode, TitanState};
use crate::types::*;
use crate::{indicators, scanner, titan_oracle};

use super::market_data::fetch_bars_internal;

// ═══════════════════════════════════════════════════════════════
//  MARKER COLORS — aligned with cyber-HUD design system tokens
// ═══════════════════════════════════════════════════════════════

const COLOR_ORACLE_BUY: &str = "#00FF41";
const COLOR_ORACLE_SELL: &str = "#FF2B2B";
const COLOR_ORACLE_EXIT: &str = "#C77DFF";

const COLOR_GPU_BUY: &str = "#00BFFF";
const COLOR_GPU_SELL: &str = "#FF8C00";
const COLOR_GPU_EXIT: &str = "#00D9FF";
const COLOR_GPU_SOLO_SELL: &str = "#FF6EC7";

const COLOR_FUSION_BUY: &str = "#FFD700";
const COLOR_FUSION_SELL: &str = "#FF4FC3";
const COLOR_FUSION_EXIT: &str = "#FFB347";
const COLOR_FUSION_SOLO_SELL: &str = "#FF4500";

const COLOR_ST_BUY: &str = "#00E5FF";
const COLOR_ST_EXIT: &str = "#FFD700";
const COLOR_ST_TRAIL: &str = "#BF00FF";
const COLOR_ST_STALK: &str = "#FFD700";

const COLOR_DEFAULT_EXIT: &str = "#FF00FF";
const COLOR_CHANGEPOINT: &str = "#FF8800";
const COLOR_HTF_BLOCKED: &str = "#777777";

// ═══════════════════════════════════════════════════════════════
//  EXIT MARKER CLASSIFICATION
// ═══════════════════════════════════════════════════════════════

/// Checks whether a marker text represents an exit signal by examining
/// the last whitespace-delimited token against known exit keywords.
/// Prevents false positives from substrings like "PRE-EXIT ALERT".
fn is_exit_marker(text: &str) -> bool {
    const EXIT_TOKENS: &[&str] = &["EXIT", "TRAIL", "CHAOS", "BREAK", "STALK"];
    text.split_whitespace()
        .last()
        .map(|tok| EXIT_TOKENS.contains(&tok))
        .unwrap_or(false)
}

// ═══════════════════════════════════════════════════════════════
//  BACKTEST COMPUTATION
// ═══════════════════════════════════════════════════════════════

pub(crate) fn build_backtest_inputs_cpu(candles: &[Candle]) -> BacktestInputs {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    BacktestInputs {
        ema9: indicators::calculate_ema_series(&closes, 9),
        ema50: indicators::calculate_ema_series(&closes, 50),
        rsi: indicators::calculate_rsi_series(&closes, 14),
    }
}

#[cfg(feature = "gpu")]
pub(crate) fn backtest_inputs_from_gpu(
    inputs: gpu::GpuBacktestInputs,
    len: usize,
) -> Option<BacktestInputs> {
    if inputs.ema_fast.len() != len || inputs.ema_slow.len() != len || inputs.rsi.len() != len {
        return None;
    }

    let mut ema9 = Vec::with_capacity(len);
    let mut ema50 = Vec::with_capacity(len);
    let mut rsi = Vec::with_capacity(len);

    for v in inputs.ema_fast {
        if !v.is_finite() {
            return None;
        }
        ema9.push(Some(v as f64));
    }
    for v in inputs.ema_slow {
        if !v.is_finite() {
            return None;
        }
        ema50.push(Some(v as f64));
    }
    for v in inputs.rsi {
        if !v.is_finite() || v < 0.0 {
            rsi.push(None);
        } else {
            rsi.push(Some(v as f64));
        }
    }

    Some(BacktestInputs { ema9, ema50, rsi })
}

#[cfg(feature = "gpu")]
pub(crate) fn backtest_inputs_from_gpu_batch(
    outputs: gpu::GpuBatchOutputs,
) -> Option<Vec<BacktestInputs>> {
    let total_len = outputs.ema_fast.len();
    if outputs.ema_slow.len() != total_len || outputs.rsi.len() != total_len {
        return None;
    }
    if outputs.offsets.len() != outputs.lengths.len() {
        return None;
    }

    let mut result: Vec<BacktestInputs> = Vec::with_capacity(outputs.offsets.len());
    for (off, len) in outputs.offsets.iter().zip(outputs.lengths.iter()) {
        let start = *off as usize;
        let series_len = *len as usize;
        if series_len == 0 {
            result.push(BacktestInputs {
                ema9: vec![],
                ema50: vec![],
                rsi: vec![],
            });
            continue;
        }
        if start.saturating_add(series_len) > total_len {
            return None;
        }

        let mut ema9 = Vec::with_capacity(series_len);
        let mut ema50 = Vec::with_capacity(series_len);
        let mut rsi = Vec::with_capacity(series_len);

        for i in 0..series_len {
            let idx = start + i;
            let e9 = outputs.ema_fast[idx];
            let e50 = outputs.ema_slow[idx];
            let r = outputs.rsi[idx];
            if !e9.is_finite() || !e50.is_finite() {
                return None;
            }
            ema9.push(Some(e9 as f64));
            ema50.push(Some(e50 as f64));
            if !r.is_finite() || r < 0.0 {
                rsi.push(None);
            } else {
                rsi.push(Some(r as f64));
            }
        }

        result.push(BacktestInputs { ema9, ema50, rsi });
    }

    Some(result)
}

#[cfg(feature = "gpu")]
pub(crate) async fn build_backtest_inputs_gpu(candles: &[Candle]) -> Option<BacktestInputs> {
    let closes: Vec<f32> = candles.iter().map(|c| c.close as f32).collect();
    let inputs = gpu::gpu_backtest_inputs(&closes, 9, 50, 14).await.ok()?;
    backtest_inputs_from_gpu(inputs, candles.len())
}

#[cfg(not(feature = "gpu"))]
pub(crate) async fn build_backtest_inputs_gpu(_candles: &[Candle]) -> Option<BacktestInputs> {
    None
}

pub(crate) fn compute_backtest_metrics(candles: &[Candle], allow_short: bool) -> BacktestResult {
    let inputs = build_backtest_inputs_cpu(candles);
    compute_backtest_metrics_from_inputs(candles, allow_short, &inputs)
}

pub(crate) fn compute_backtest_metrics_from_inputs(
    candles: &[Candle],
    allow_short: bool,
    inputs: &BacktestInputs,
) -> BacktestResult {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut position: i32 = 0;
    let mut entry_price = 0.0;
    let mut trades = 0usize;
    let mut wins = 0usize;
    let mut win_sum = 0.0;
    let mut loss_sum = 0.0;
    let mut equity = 1.0;
    let mut peak = 1.0;
    let mut max_dd = 0.0;
    let mut equity_curve = Vec::with_capacity(closes.len());
    let mut trade_returns: Vec<f64> = Vec::new();

    equity_curve.push(equity);
    for i in 1..closes.len() {
        let prev = closes[i - 1];
        let price = closes[i];
        if prev > 0.0 && position != 0 {
            let ret = (price - prev) / prev;
            equity *= 1.0 + (position as f64) * ret;
        }
        if equity > peak {
            peak = equity;
        }
        let dd = if peak > 0.0 {
            ((peak - equity) / peak) * 100.0
        } else {
            0.0
        };
        if dd > max_dd {
            max_dd = dd;
        }
        equity_curve.push(equity);

        let ema9 = inputs.ema9.get(i).and_then(|v| *v);
        let ema50 = inputs.ema50.get(i).and_then(|v| *v);
        let rsi = inputs.rsi.get(i).and_then(|v| *v);
        if ema9.is_none() || ema50.is_none() || rsi.is_none() {
            continue;
        }
        let ema9 = ema9.unwrap();
        let ema50 = ema50.unwrap();
        let rsi = rsi.unwrap();

        if position == 0 {
            if ema9 > ema50 && rsi > 50.0 {
                position = 1;
                entry_price = price;
                trades += 1;
            } else if allow_short && ema9 < ema50 && rsi < 50.0 {
                position = -1;
                entry_price = price;
                trades += 1;
            }
        } else if position == 1 && ema9 < ema50 {
            let pnl = (price - entry_price) / entry_price;
            if pnl > 0.0 {
                wins += 1;
                win_sum += pnl;
            } else {
                loss_sum += pnl.abs();
            }
            trade_returns.push(pnl);
            position = 0;
        } else if position == -1 && ema9 > ema50 {
            let pnl = (entry_price - price) / entry_price;
            if pnl > 0.0 {
                wins += 1;
                win_sum += pnl;
            } else {
                loss_sum += pnl.abs();
            }
            trade_returns.push(pnl);
            position = 0;
        }
    }

    if position != 0 && entry_price > 0.0 {
        let last = *closes.last().unwrap_or(&entry_price);
        let pnl = if position == 1 {
            (last - entry_price) / entry_price
        } else {
            (entry_price - last) / entry_price
        };
        if pnl > 0.0 {
            wins += 1;
            win_sum += pnl;
        } else {
            loss_sum += pnl.abs();
        }
        trade_returns.push(pnl);
    }

    let win_rate = if trades > 0 {
        (wins as f64 / trades as f64) * 100.0
    } else {
        0.0
    };
    let return_pct = (equity - 1.0) * 100.0;
    let profit_factor = if loss_sum > 0.0 {
        win_sum / loss_sum
    } else {
        0.0
    };
    let expectancy = if trades > 0 {
        (win_sum - loss_sum) / trades as f64
    } else {
        0.0
    };
    let sharpe = {
        if equity_curve.len() < 2 {
            0.0
        } else {
            let mut rets = Vec::with_capacity(equity_curve.len() - 1);
            for i in 1..equity_curve.len() {
                let prev = equity_curve[i - 1];
                let cur = equity_curve[i];
                if prev > 0.0 {
                    rets.push((cur - prev) / prev);
                }
            }
            if rets.is_empty() {
                0.0
            } else {
                let mean = rets.iter().copied().sum::<f64>() / rets.len() as f64;
                let var =
                    rets.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / rets.len() as f64;
                let std = var.sqrt();
                if std > 0.0 {
                    mean / std * (rets.len() as f64).sqrt()
                } else {
                    0.0
                }
            }
        }
    };

    BacktestResult {
        metrics: BacktestMetrics {
            trades,
            win_rate,
            return_pct,
            max_drawdown: max_dd,
            profit_factor,
            sharpe,
            expectancy,
        },
        equity_curve,
    }
}

pub(crate) fn compute_backtest_oracle(candles: &[Candle], allow_short: bool) -> BacktestResult {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let n = closes.len();
    if n < 60 {
        return BacktestResult {
            metrics: BacktestMetrics::default(),
            equity_curve: vec![],
        };
    }

    let oracle_hist = titan_oracle::calculate_history(candles, &titan_oracle::OracleWeights::default());
    let atr_series = indicators::calculate_atr_series(candles, 14);

    let mut position: i32 = 0;
    let mut entry_price = 0.0;
    let mut trades = 0usize;
    let mut wins = 0usize;
    let mut win_sum = 0.0;
    let mut loss_sum = 0.0;
    let mut equity = 1.0;
    let mut peak = 1.0;
    let mut max_dd = 0.0;
    let mut equity_curve = Vec::with_capacity(n);
    equity_curve.push(equity);
    let mut highest_since_entry = 0.0_f64;
    let mut lowest_since_entry = f64::MAX;
    let mut trade_returns: Vec<f64> = Vec::new();

    for i in 1..n {
        let prev = closes[i - 1];
        let price = closes[i];
        if prev > 0.0 && position != 0 {
            let ret = (price - prev) / prev;
            equity *= 1.0 + (position as f64) * ret;
        }
        if equity > peak {
            peak = equity;
        }
        let dd = if peak > 0.0 {
            ((peak - equity) / peak) * 100.0
        } else {
            0.0
        };
        if dd > max_dd {
            max_dd = dd;
        }
        equity_curve.push(equity);

        if position == 1 && price > highest_since_entry {
            highest_since_entry = price;
        }
        if position == -1 && price < lowest_since_entry {
            lowest_since_entry = price;
        }

        let oracle_score = oracle_hist
            .oracle_scores
            .get(i)
            .and_then(|v| *v)
            .unwrap_or(0.0);
        let confidence = oracle_hist
            .confidence_series
            .get(i)
            .and_then(|v| *v)
            .unwrap_or(0.0);
        let regime = oracle_hist
            .regime_series
            .get(i)
            .and_then(|v| *v)
            .unwrap_or(titan_oracle::MarketRegime::Unknown);
        let atr = atr_series.get(i).and_then(|v| *v).unwrap_or(0.0);
        let atr_pct = if price > 0.0 {
            (atr / price) * 100.0
        } else {
            1.0
        };

        let (long_thresh, short_thresh, conf_thresh) = match regime {
            titan_oracle::MarketRegime::Trending => (35.0, -35.0, 0.25),
            titan_oracle::MarketRegime::MeanReverting => (50.0, -50.0, 0.35),
            titan_oracle::MarketRegime::Chaotic => (70.0, -70.0, 0.55),
            titan_oracle::MarketRegime::Unknown => (50.0, -50.0, 0.30),
        };

        let sl_pct = (atr_pct * 1.5).clamp(0.5, 8.0);
        let tp_pct = match regime {
            titan_oracle::MarketRegime::Trending => (atr_pct * 2.5).clamp(1.0, 15.0),
            _ => (atr_pct * 1.5).clamp(1.0, 10.0),
        };
        let trail_pct = (atr_pct * 1.0).clamp(0.3, 5.0);

        if position != 0 {
            let pnl_pct = if position == 1 {
                ((price - entry_price) / entry_price) * 100.0
            } else {
                ((entry_price - price) / entry_price) * 100.0
            };

            let trail_drop = if position == 1 {
                ((highest_since_entry - price) / highest_since_entry) * 100.0
            } else {
                ((price - lowest_since_entry) / lowest_since_entry) * 100.0
            };

            let opposing = (position == 1 && oracle_score < -30.0 && confidence > 0.35)
                || (position == -1 && oracle_score > 30.0 && confidence > 0.35);

            let should_exit = pnl_pct <= -sl_pct
                || pnl_pct >= tp_pct
                || (pnl_pct > 0.5 && trail_drop >= trail_pct)
                || opposing;

            if should_exit {
                let pnl = pnl_pct / 100.0;
                if pnl > 0.0 {
                    wins += 1;
                    win_sum += pnl;
                } else {
                    loss_sum += pnl.abs();
                }
                trade_returns.push(pnl);
                position = 0;
            }
        }

        if position == 0 {
            if oracle_score > long_thresh && confidence > conf_thresh {
                position = 1;
                entry_price = price;
                highest_since_entry = price;
                trades += 1;
            } else if allow_short && oracle_score < short_thresh && confidence > conf_thresh {
                position = -1;
                entry_price = price;
                lowest_since_entry = price;
                trades += 1;
            }
        }
    }

    if position != 0 && entry_price > 0.0 {
        let last = *closes.last().unwrap_or(&entry_price);
        let pnl = if position == 1 {
            (last - entry_price) / entry_price
        } else {
            (entry_price - last) / entry_price
        };
        if pnl > 0.0 {
            wins += 1;
            win_sum += pnl;
        } else {
            loss_sum += pnl.abs();
        }
        trade_returns.push(pnl);
    }

    let win_rate = if trades > 0 {
        (wins as f64 / trades as f64) * 100.0
    } else {
        0.0
    };
    let return_pct = (equity - 1.0) * 100.0;
    let profit_factor = if loss_sum > 0.0 {
        win_sum / loss_sum
    } else {
        0.0
    };
    let expectancy = if trades > 0 {
        (win_sum - loss_sum) / trades as f64
    } else {
        0.0
    };
    let sharpe = {
        if equity_curve.len() < 2 {
            0.0
        } else {
            let mut rets = Vec::with_capacity(equity_curve.len() - 1);
            for i in 1..equity_curve.len() {
                let prev_eq = equity_curve[i - 1];
                let cur_eq = equity_curve[i];
                if prev_eq > 0.0 {
                    rets.push((cur_eq - prev_eq) / prev_eq);
                }
            }
            if rets.is_empty() {
                0.0
            } else {
                let mean = rets.iter().copied().sum::<f64>() / rets.len() as f64;
                let var =
                    rets.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / rets.len() as f64;
                let std = var.sqrt();
                if std > 0.0 {
                    mean / std * (rets.len() as f64).sqrt()
                } else {
                    0.0
                }
            }
        }
    };

    BacktestResult {
        metrics: BacktestMetrics {
            trades,
            win_rate,
            return_pct,
            max_drawdown: max_dd,
            profit_factor,
            sharpe,
            expectancy,
        },
        equity_curve,
    }
}

// ═══════════════════════════════════════════════════════════════
//  STRATEGY SCORE COMPUTATION
// ═══════════════════════════════════════════════════════════════

pub(crate) fn compute_strategy_score_with_mode(
    candles: &[Candle],
    mode: StrategyMode,
    alpha: f64,
    gpu_momentum: f64,
    periods: &indicators::IndicatorPeriods,
    oracle_weights: &titan_oracle::OracleWeights,
) -> StrategyScore {
    let ind = indicators::calculate_all_with_periods(candles, periods, oracle_weights);
    let _last_close = candles.last().map(|c| c.close).unwrap_or(0.0);

    let ema9 = ind.ema_9.unwrap_or(0.0);
    let ema50 = ind.ema_50.unwrap_or(0.0);
    let rsi = ind.rsi.unwrap_or(50.0);
    let adx = ind.adx.unwrap_or(0.0);

    let oracle_val = ind.oracle_score.unwrap_or(0.0);
    let oracle_conf = ind.oracle_confidence.unwrap_or(0.0);

    let (score, signal): (f64, &str) = match mode {
        StrategyMode::MathOracle => {
            let s = (50.0 + oracle_val * oracle_conf).clamp(0.0, 100.0);
            let sig = if oracle_val > 8.0 && oracle_conf > 0.10 {
                "LONG"
            } else if oracle_val < -8.0 && oracle_conf > 0.10 {
                "SHORT"
            } else {
                "NEUTRAL"
            };
            (s, sig)
        }
        StrategyMode::GpuNexus => {
            let gpu_scaled = gpu_momentum * 100.0;
            let s = (50.0 + gpu_scaled).clamp(0.0, 100.0);
            let sig = if gpu_momentum > 0.003 {
                "LONG"
            } else if gpu_momentum < -0.003 {
                "SHORT"
            } else {
                "NEUTRAL"
            };
            (s, sig)
        }
        StrategyMode::Fusion | StrategyMode::SignalStrategy => {
            let fusion_val = oracle_val * (1.0 - alpha) + (gpu_momentum * 100.0) * alpha;
            let fusion_conf = if oracle_conf > 0.1 || gpu_momentum.abs() > 0.003 {
                1.0
            } else {
                0.0
            };
            let s = (50.0 + fusion_val * fusion_conf).clamp(0.0, 100.0);
            let fusion_thresh = (8.0 * (1.0 - alpha) + 2.0 * alpha).min(6.0);
            let sig = if fusion_val > fusion_thresh && fusion_conf > 0.0 {
                "LONG"
            } else if fusion_val < -fusion_thresh && fusion_conf > 0.0 {
                "SHORT"
            } else {
                "NEUTRAL"
            };
            (s, sig)
        }
        StrategyMode::SmartTrend => {
            let s = (50.0 + oracle_val * oracle_conf).clamp(0.0, 100.0);
            let sig = if oracle_val > 5.0 && oracle_conf > 0.08 {
                "LONG"
            } else if oracle_val < -5.0 && oracle_conf > 0.08 {
                "SHORT"
            } else {
                "NEUTRAL"
            };
            (s, sig)
        }
    };

    let oracle_snap = titan_oracle::calculate(candles, oracle_weights);
    let regime = format!("{:?}", oracle_snap.dominant_regime);
    let trend = match oracle_snap.dominant_regime {
        titan_oracle::MarketRegime::Trending => {
            if oracle_val > 0.0 {
                "UPTREND"
            } else {
                "DOWNTREND"
            }
        }
        titan_oracle::MarketRegime::MeanReverting => "RANGE",
        titan_oracle::MarketRegime::Chaotic => "CHAOTIC",
        titan_oracle::MarketRegime::Unknown => "RANGE",
    };

    StrategyScore {
        score,
        signal: signal.to_string(),
        rsi,
        ema9,
        ema50,
        adx,
        trend: trend.to_string(),
        gpu_boost: gpu_momentum,
        oracle_score: oracle_val,
        oracle_confidence: oracle_conf,
        oracle_regime: regime,
    }
}

#[cfg(feature = "gpu")]
pub(crate) async fn compute_gpu_scores(
    state: Arc<TitanState>,
    candles: &[Candle],
) -> Option<(f64, f64)> {
    if candles.len() < 128 {
        return None;
    }
    let now = Instant::now();
    {
        let cache = state
            .gpu_score_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if cache.updated.elapsed() < StdDuration::from_secs(5) {
            return Some((cache.momentum, cache.ema_delta));
        }
    }

    async fn dot_window(slice: &[Candle]) -> Option<f64> {
        let mut returns: Vec<f32> = Vec::with_capacity(slice.len().saturating_sub(1));
        for i in 1..slice.len() {
            let prev = slice[i - 1].close;
            let cur = slice[i].close;
            if prev > 0.0 {
                returns.push(((cur - prev) / prev) as f32);
            }
        }
        if returns.is_empty() {
            return None;
        }
        let mut weights: Vec<f32> = Vec::with_capacity(returns.len());
        let n = returns.len() as f32;
        for i in 0..returns.len() {
            let t = (i as f32) / n;
            weights.push(0.4 + t);
        }
        let dot = gpu::gpu_dot(&returns, &weights).await.ok()?;
        Some((dot / (returns.len() as f32)).tanh() as f64)
    }

    let s64 = &candles[candles.len().saturating_sub(64)..];
    let s128 = &candles[candles.len().saturating_sub(128)..];
    let s256 = &candles[candles.len().saturating_sub(256)..];

    let short = dot_window(s64).await.unwrap_or(0.0);
    let mid = dot_window(s128).await.unwrap_or(0.0);
    let long = dot_window(s256).await.unwrap_or(0.0);

    let momentum = (short * 0.5) + (mid * 0.3) + (long * 0.2);

    let slice = &candles[candles.len().saturating_sub(256)..];
    let mut closes: Vec<f32> = Vec::with_capacity(slice.len());
    for c in slice {
        closes.push(c.close as f32);
    }
    let n = closes.len().max(1) as f32;
    let mut weights: Vec<f32> = Vec::with_capacity(closes.len());
    let k = 0.05_f32;
    for i in 0..closes.len() {
        let t = (i as f32) / n;
        weights.push((k * (t * n)).exp());
    }
    let sum_w: f32 = weights.iter().sum();
    if sum_w > 0.0 {
        for w in &mut weights {
            *w /= sum_w;
        }
    }
    let ema = gpu::gpu_dot(&closes, &weights).await.ok().unwrap_or(0.0);
    let last = closes.last().copied().unwrap_or(0.0);
    let ema_delta = if ema > 0.0 {
        ((last - ema) / ema) as f64 * 100.0
    } else {
        0.0
    };

    let mut cache = state
        .gpu_score_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *cache = crate::state::GpuScoreCache {
        momentum,
        ema_delta,
        updated: now,
    };
    Some((momentum, ema_delta))
}

#[cfg(not(feature = "gpu"))]
pub(crate) async fn compute_gpu_scores(
    _state: Arc<TitanState>,
    _candles: &[Candle],
) -> Option<(f64, f64)> {
    None
}

// ═══════════════════════════════════════════════════════════════
//  VOLATILITY COMPUTATION
// ═══════════════════════════════════════════════════════════════

pub(crate) fn compute_volatility_cpu(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let mean = returns.iter().copied().sum::<f64>() / returns.len() as f64;
    let var = returns.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / returns.len() as f64;
    var.max(0.0).sqrt() * 100.0
}

#[cfg(feature = "gpu")]
pub(crate) async fn compute_volatility_gpu(returns: &[f64]) -> Option<f64> {
    if returns.len() < 64 {
        return None;
    }
    let returns_f32: Vec<f32> = returns.iter().map(|v| *v as f32).collect();
    let ones = vec![1.0_f32; returns_f32.len()];
    let sum = gpu::gpu_dot(&returns_f32, &ones).await.ok()? as f64;
    let sum_sq = gpu::gpu_dot(&returns_f32, &returns_f32).await.ok()? as f64;
    let n = returns_f32.len() as f64;
    if n <= 0.0 {
        return None;
    }
    let mean = sum / n;
    let var = (sum_sq / n) - (mean * mean);
    Some(var.max(0.0).sqrt() * 100.0)
}

#[cfg(not(feature = "gpu"))]
pub(crate) async fn compute_volatility_gpu(_returns: &[f64]) -> Option<f64> {
    None
}

// ═══════════════════════════════════════════════════════════════
//  SIGNAL BACKTEST
// ═══════════════════════════════════════════════════════════════

pub(crate) fn compute_signal_backtest(
    candles: &[Candle],
    markers: &[SignalMarker],
    allow_short: bool,
) -> BacktestResult {
    if candles.len() < 10 {
        return BacktestResult {
            metrics: BacktestMetrics::default(),
            equity_curve: vec![],
        };
    }

    let time_to_idx: HashMap<i64, usize> = candles
        .iter()
        .enumerate()
        .map(|(i, c)| (c.time, i))
        .collect();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut signals_at: HashMap<usize, &str> = HashMap::new();
    for m in markers {
        if let Some(&idx) = time_to_idx.get(&m.time) {
            let priority = match m.text.as_str() {
                t if t.contains("BUY") => 3,
                t if t.contains("SELL") || t.contains("SHORT") => 3,
                t if is_exit_marker(t) => 2,
                _ => 1,
            };
            let existing_priority = signals_at
                .get(&idx)
                .map(|t| match *t {
                    t if t.contains("BUY") || t.contains("SELL") || t.contains("SHORT") => 3,
                    t if is_exit_marker(t) => 2,
                    _ => 1,
                })
                .unwrap_or(0);
            if priority >= existing_priority {
                signals_at.insert(idx, &m.text);
            }
        }
    }

    let mut position: i32 = 0;
    let mut entry_price = 0.0;
    let mut trades = 0usize;
    let mut wins = 0usize;
    let mut win_sum = 0.0;
    let mut loss_sum = 0.0;
    let mut equity = 1.0;
    let mut peak = 1.0;
    let mut max_dd = 0.0;
    let mut equity_curve = Vec::with_capacity(closes.len());

    for i in 0..closes.len() {
        let price = closes[i];
        if i > 0 && position != 0 {
            let prev = closes[i - 1];
            if prev > 0.0 {
                let ret = (price - prev) / prev;
                equity *= 1.0 + (position as f64) * ret;
            }
        }
        if equity > peak {
            peak = equity;
        }
        let dd = if peak > 0.0 {
            ((peak - equity) / peak) * 100.0
        } else {
            0.0
        };
        if dd > max_dd {
            max_dd = dd;
        }
        equity_curve.push(equity);

        if let Some(sig) = signals_at.get(&i) {
            let is_buy = sig.contains("BUY");
            let is_sell = sig.contains("SELL") || sig.contains("SHORT");
            let is_exit = is_exit_marker(sig);

            if is_exit && position != 0 {
                let pnl = if position == 1 {
                    (price - entry_price) / entry_price
                } else {
                    (entry_price - price) / entry_price
                };
                if pnl > 0.0 {
                    wins += 1;
                    win_sum += pnl;
                } else {
                    loss_sum += pnl.abs();
                }
                position = 0;
            } else if is_buy {
                if position == -1 {
                    let pnl = (entry_price - price) / entry_price;
                    if pnl > 0.0 {
                        wins += 1;
                        win_sum += pnl;
                    } else {
                        loss_sum += pnl.abs();
                    }
                    position = 0;
                }
                if position == 0 {
                    position = 1;
                    entry_price = price;
                    trades += 1;
                }
            } else if is_sell {
                if position == 1 {
                    let pnl = (price - entry_price) / entry_price;
                    if pnl > 0.0 {
                        wins += 1;
                        win_sum += pnl;
                    } else {
                        loss_sum += pnl.abs();
                    }
                    position = 0;
                }
                if position == 0 && allow_short {
                    position = -1;
                    entry_price = price;
                    trades += 1;
                }
            }
        }
    }

    if position != 0 && entry_price > 0.0 {
        let last = *closes.last().unwrap_or(&entry_price);
        let pnl = if position == 1 {
            (last - entry_price) / entry_price
        } else {
            (entry_price - last) / entry_price
        };
        if pnl > 0.0 {
            wins += 1;
            win_sum += pnl;
        } else {
            loss_sum += pnl.abs();
        }
    }

    let win_rate = if trades > 0 {
        (wins as f64 / trades as f64) * 100.0
    } else {
        0.0
    };
    let return_pct = (equity - 1.0) * 100.0;
    let profit_factor = if loss_sum > 0.0 {
        win_sum / loss_sum
    } else {
        0.0
    };
    let expectancy = if trades > 0 {
        (win_sum - loss_sum) / trades as f64
    } else {
        0.0
    };
    let sharpe = if equity_curve.len() < 2 {
        0.0
    } else {
        let mut rets = Vec::with_capacity(equity_curve.len() - 1);
        for i in 1..equity_curve.len() {
            if equity_curve[i - 1] > 0.0 {
                rets.push((equity_curve[i] - equity_curve[i - 1]) / equity_curve[i - 1]);
            }
        }
        if rets.is_empty() {
            0.0
        } else {
            let mean = rets.iter().copied().sum::<f64>() / rets.len() as f64;
            let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rets.len() as f64;
            let std = var.sqrt();
            if std > 0.0 {
                mean / std * (rets.len() as f64).sqrt()
            } else {
                0.0
            }
        }
    };

    BacktestResult {
        metrics: BacktestMetrics {
            trades,
            win_rate,
            return_pct,
            max_drawdown: max_dd,
            profit_factor,
            sharpe,
            expectancy,
        },
        equity_curve,
    }
}

// ═══════════════════════════════════════════════════════════════
//  SIGNAL MARKERS
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn get_signal_markers_internal(
    state: &Arc<TitanState>,
    symbol: &str,
    tf: &str,
) -> Result<SignalMarkersResponse, TitanError> {
    let symbol = normalize_symbol(symbol);
    let tf = if tf.is_empty() {
        "5m".to_string()
    } else {
        tf.to_string()
    };
    let candles = fetch_bars_internal(state.clone(), &symbol, &tf).await?;
    if candles.len() < 100 {
        tracing::warn!(
            "MARKERS: insufficient data for {} {} ({} candles, need 100 for oracle warmup)",
            symbol,
            tf,
            candles.len()
        );
        return Ok(SignalMarkersResponse {
            markers: vec![],
            reason: Some(format!(
                "Insufficient data: {} candles (need 100+ for oracle warmup)",
                candles.len()
            )),
            st_scores: None,
        });
    }

    let mode = {
        *state
            .strategy_mode
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    };
    let alpha = {
        *state
            .nexus_fusion_alpha
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    };
    let (scalp_oracle, scalp_gpu, scalp_fusion, signal_source) = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (
            s.scalping_oracle,
            s.scalping_gpu,
            s.scalping_fusion,
            s.signal_source.to_ascii_lowercase(),
        )
    };
    let (signal_buy_color, signal_sell_color, signal_buy_text, signal_sell_text, signal_exit_color, signal_prefix) =
        match signal_source.as_str() {
            "oracle" => (COLOR_ORACLE_BUY, COLOR_ORACLE_SELL, "O BUY", "O SELL", COLOR_ORACLE_EXIT, "O"),
            "gpu" => (COLOR_GPU_BUY, COLOR_GPU_SELL, "G BUY", "G SELL", COLOR_GPU_EXIT, "G"),
            _ => (COLOR_FUSION_BUY, COLOR_FUSION_SELL, "F BUY", "F SELL", COLOR_FUSION_EXIT, "F"),
        };
    let (periods, oracle_w) = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (crate::state::build_indicator_periods(&s), crate::state::build_oracle_weights(&s))
    };
    let hist = indicators::calculate_all_history_with_periods(&candles, &periods, &oracle_w);
    let mut markers = Vec::new();
    let mut st_score_points: Vec<StScorePoint> = Vec::new();

    let oracle_scores: Vec<Option<f64>> = hist
        .oracle
        .as_ref()
        .map(|o| o.oracle_scores.clone())
        .unwrap_or_default();

    let gpu_scores: Vec<Option<f64>> = if matches!(
        mode,
        StrategyMode::GpuNexus | StrategyMode::Fusion | StrategyMode::SignalStrategy | StrategyMode::SmartTrend
    ) {
        #[cfg(feature = "gpu")]
        async fn chart_dot_window(candles: &[Candle], end: usize, window: usize) -> Option<f64> {
            let start = end.saturating_sub(window);
            let slice = &candles[start..=end];
            let mut returns: Vec<f32> = Vec::with_capacity(slice.len().saturating_sub(1));
            for j in 1..slice.len() {
                let prev = slice[j - 1].close;
                let cur = slice[j].close;
                if prev > 0.0 {
                    returns.push(((cur - prev) / prev) as f32);
                }
            }
            if returns.is_empty() {
                return None;
            }
            let n = returns.len() as f32;
            let weights: Vec<f32> = (0..returns.len()).map(|k| 0.4 + (k as f32) / n).collect();
            let cpu_dot: f32 = returns
                .iter()
                .zip(weights.iter())
                .map(|(r, w)| r * w)
                .sum();
            let dot = match gpu::gpu_dot(&returns, &weights).await {
                Ok(v) if v.is_finite() => v,
                _ => cpu_dot,
            };
            Some((dot / n).tanh() as f64)
        }
        #[cfg(not(feature = "gpu"))]
        async fn chart_dot_window(candles: &[Candle], end: usize, window: usize) -> Option<f64> {
            let start = end.saturating_sub(window);
            let slice = &candles[start..=end];
            let mut returns: Vec<f32> = Vec::with_capacity(slice.len().saturating_sub(1));
            for j in 1..slice.len() {
                let prev = slice[j - 1].close;
                let cur = slice[j].close;
                if prev > 0.0 {
                    returns.push(((cur - prev) / prev) as f32);
                }
            }
            if returns.is_empty() {
                return None;
            }
            let n = returns.len() as f32;
            let weights: Vec<f32> = (0..returns.len()).map(|k| 0.4 + (k as f32) / n).collect();
            let dot: f32 = returns
                .iter()
                .zip(weights.iter())
                .map(|(r, w)| r * w)
                .sum();
            Some((dot / n).tanh() as f64)
        }

        let mut scores = vec![None; candles.len()];
        for (i, slot) in scores.iter_mut().enumerate().skip(32) {
            let short = chart_dot_window(&candles, i, 64).await.unwrap_or(0.0);
            let mid = chart_dot_window(&candles, i, 128).await.unwrap_or(0.0);
            let long = chart_dot_window(&candles, i, 256).await.unwrap_or(0.0);
            let blended = short * 0.6 + mid * 0.3 + long * 0.1;
            if blended.is_finite() {
                *slot = Some(blended);
            }
        }
        scores
    } else {
        vec![None; candles.len()]
    };

    fn percentile(sorted: &[f64], pct: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = ((sorted.len() as f64 - 1.0) * pct).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Zone {
        Buy,
        Sell,
        Neutral,
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_zone_signals(
        scores: &[Option<f64>],
        candles: &[Candle],
        min_buy: f64,
        max_sell: f64,
        fallback_buy: f64,
        fallback_sell: f64,
        lookback: usize,
        base_cooldown: usize,
        buy_color: &str,
        sell_color: &str,
        buy_text: &str,
        sell_text: &str,
        ema_filter: Option<&[Option<f64>]>,
        regime_series: Option<&[Option<titan_oracle::MarketRegime>]>,
        confidence_series: Option<&[Option<f64>]>,
        atr_series: Option<&[Option<f64>]>,
        scalping: bool,
    ) -> Vec<SignalMarker> {
        let mut out = Vec::new();
        let mut last_signal_i: Option<usize> = None;
        let mut prev_zone = Zone::Neutral;
        let mut dbg_valid = 0u32;
        let mut dbg_conf_pass = 0u32;
        let mut dbg_zone_cross = 0u32;

        for i in 0..candles.len() {
            let score = match scores.get(i).and_then(|v| *v) {
                Some(s) => s,
                None => {
                    prev_zone = Zone::Neutral;
                    continue;
                }
            };
            dbg_valid += 1;

            let regime = regime_series
                .and_then(|rs| rs.get(i).and_then(|v| *v))
                .unwrap_or(titan_oracle::MarketRegime::Unknown);

            let bar_confidence = confidence_series
                .and_then(|cs| cs.get(i).and_then(|v| *v))
                .unwrap_or(0.5);

            let (buy_pct, sell_pct, confidence_gate) = if scalping {
                match regime {
                    titan_oracle::MarketRegime::Trending => (0.60, 0.40, 0.05),
                    titan_oracle::MarketRegime::MeanReverting => (0.65, 0.35, 0.08),
                    titan_oracle::MarketRegime::Chaotic => (0.75, 0.25, 0.15),
                    titan_oracle::MarketRegime::Unknown => (0.62, 0.38, 0.06),
                }
            } else {
                match regime {
                    titan_oracle::MarketRegime::Trending => (0.70, 0.30, 0.05),
                    titan_oracle::MarketRegime::MeanReverting => (0.80, 0.20, 0.10),
                    titan_oracle::MarketRegime::Chaotic => (0.90, 0.10, 0.20),
                    titan_oracle::MarketRegime::Unknown => (0.75, 0.25, 0.08),
                }
            };

            if bar_confidence < confidence_gate {
                prev_zone = Zone::Neutral;
                continue;
            }
            dbg_conf_pass += 1;

            let start = i.saturating_sub(lookback);
            let mut window: Vec<f64> = scores[start..i].iter().filter_map(|v| *v).collect();

            let (buy_thresh, sell_thresh) = if window.len() >= 15 {
                window.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let bt = percentile(&window, buy_pct).max(min_buy);
                let st = percentile(&window, sell_pct).min(max_sell);
                (bt, st)
            } else {
                (fallback_buy, fallback_sell)
            };

            let zone = if score > buy_thresh {
                Zone::Buy
            } else if score < sell_thresh {
                Zone::Sell
            } else {
                Zone::Neutral
            };

            let cooldown = if let Some(atr_s) = atr_series {
                if let (Some(Some(cur_atr)), Some(Some(prev_atr))) =
                    (atr_s.get(i), atr_s.get(i.saturating_sub(20)))
                {
                    let atr_ratio = cur_atr / prev_atr.max(1e-12);
                    if atr_ratio > 1.5 {
                        (base_cooldown as f64 * 1.5) as usize
                    } else if atr_ratio < 0.7 {
                        (base_cooldown as f64 * 0.7).max(2.0) as usize
                    } else {
                        base_cooldown
                    }
                } else {
                    base_cooldown
                }
            } else {
                base_cooldown
            };

            let cooled = last_signal_i.is_none_or(|li| i.saturating_sub(li) >= cooldown);

            let trend_ok = if let Some(ema) = ema_filter {
                if i >= 5 {
                    if let (Some(cur), Some(prev)) =
                        (ema.get(i).and_then(|v| *v), ema.get(i - 5).and_then(|v| *v))
                    {
                        let slope_pct = (cur - prev) / prev.abs().max(1e-12);
                        let deadband = 0.0015;
                        match zone {
                            Zone::Buy => slope_pct > -deadband,
                            Zone::Sell => slope_pct < deadband,
                            Zone::Neutral => true,
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            };

            if zone == Zone::Buy && prev_zone != Zone::Buy && cooled && trend_ok {
                dbg_zone_cross += 1;
                out.push(SignalMarker {
                    time: candles[i].time,
                    position: "belowBar".to_string(),
                    color: buy_color.to_string(),
                    shape: "arrowUp".to_string(),
                    text: buy_text.to_string(),
                });
                last_signal_i = Some(i);
            } else if zone == Zone::Sell && prev_zone != Zone::Sell && cooled && trend_ok {
                dbg_zone_cross += 1;
                out.push(SignalMarker {
                    time: candles[i].time,
                    position: "aboveBar".to_string(),
                    color: sell_color.to_string(),
                    shape: "arrowDown".to_string(),
                    text: sell_text.to_string(),
                });
                last_signal_i = Some(i);
            }

            prev_zone = zone;
        }
        tracing::debug!(
            "generate_zone_signals: {} bars, {} valid scores, {} passed confidence, {} zone crossings => {} signals (scalp={})",
            candles.len(), dbg_valid, dbg_conf_pass, dbg_zone_cross, out.len(), scalping
        );
        out
    }

    let atr_series = indicators::calculate_atr_series(&candles, 14);

    let (regime_s, conf_s) = hist
        .oracle
        .as_ref()
        .map(|o| (o.regime_series.clone(), o.confidence_series.clone()))
        .unwrap_or_else(|| (vec![None; candles.len()], vec![None; candles.len()]));

    match mode {
        StrategyMode::MathOracle => {
            let (cd, lb, mb, ms, fb, fs) = if scalp_oracle {
                (
                    match tf.as_str() {
                        "1m" => 4,
                        "5m" => 5,
                        "15m" => 4,
                        "10m" => 4,
                        _ => 3,
                    },
                    18,
                    1.5,
                    -1.5,
                    3.0,
                    -3.0,
                )
            } else {
                (
                    match tf.as_str() {
                        "1m" => 8,
                        "5m" => 7,
                        "15m" => 5,
                        "10m" => 5,
                        _ => 4,
                    },
                    28,
                    2.0,
                    -2.0,
                    4.0,
                    -4.0,
                )
            };
            markers.extend(generate_zone_signals(
                &oracle_scores,
                &candles,
                mb,
                ms,
                fb,
                fs,
                lb,
                cd,
                COLOR_ORACLE_BUY,
                COLOR_ORACLE_SELL,
                "BUY",
                "SELL",
                None,
                Some(&regime_s),
                Some(&conf_s),
                Some(&atr_series),
                scalp_oracle,
            ));
        }
        StrategyMode::GpuNexus => {
            let (cd, lb) = if scalp_gpu {
                (
                    match tf.as_str() {
                        "1m" => 3,
                        "5m" => 4,
                        "15m" => 3,
                        "10m" => 3,
                        _ => 2,
                    },
                    14,
                )
            } else {
                (
                    match tf.as_str() {
                        "1m" => 6,
                        "5m" => 5,
                        "15m" => 4,
                        "10m" => 4,
                        _ => 3,
                    },
                    22,
                )
            };
            markers.extend(generate_zone_signals(
                &gpu_scores,
                &candles,
                0.001,
                -0.001,
                0.003,
                -0.003,
                lb,
                cd,
                COLOR_GPU_BUY,
                COLOR_GPU_SOLO_SELL,
                "GPU BUY",
                "GPU SELL",
                Some(&hist.ema_50),
                Some(&regime_s),
                Some(&conf_s),
                Some(&atr_series),
                scalp_gpu,
            ));
        }
        StrategyMode::Fusion => {
            let mut fusion_scores: Vec<Option<f64>> = vec![None; candles.len()];

            for (i, slot) in fusion_scores.iter_mut().enumerate() {
                let bar_regime = regime_s
                    .get(i)
                    .and_then(|v| *v)
                    .unwrap_or(titan_oracle::MarketRegime::Unknown);
                let dynamic_alpha = match bar_regime {
                    titan_oracle::MarketRegime::Trending => (alpha * 1.3).min(0.8),
                    titan_oracle::MarketRegime::MeanReverting => (alpha * 0.5).max(0.2),
                    titan_oracle::MarketRegime::Chaotic => (alpha * 0.3).max(0.1),
                    titan_oracle::MarketRegime::Unknown => alpha,
                };

                let o = oracle_scores.get(i).and_then(|v| *v);
                let g = gpu_scores.get(i).and_then(|v| *v);
                *slot = match (o, g) {
                    (Some(ov), Some(gv)) => {
                        Some(ov * (1.0 - dynamic_alpha) + (gv * 100.0) * dynamic_alpha)
                    }
                    (Some(ov), None) => Some(ov),
                    (None, Some(gv)) => Some(gv * 100.0),
                    _ => None,
                };
            }

            let (cd, lb, mb, ms, fb, fs) = if scalp_fusion {
                (
                    match tf.as_str() {
                        "1m" => 4,
                        "5m" => 5,
                        "15m" => 4,
                        "10m" => 4,
                        _ => 3,
                    },
                    18,
                    1.5,
                    -1.5,
                    3.0,
                    -3.0,
                )
            } else {
                (
                    match tf.as_str() {
                        "1m" => 8,
                        "5m" => 7,
                        "15m" => 5,
                        "10m" => 5,
                        _ => 4,
                    },
                    28,
                    2.0,
                    -2.0,
                    4.0,
                    -4.0,
                )
            };
            markers.extend(generate_zone_signals(
                &fusion_scores,
                &candles,
                mb,
                ms,
                fb,
                fs,
                lb,
                cd,
                COLOR_FUSION_BUY,
                COLOR_FUSION_SOLO_SELL,
                "FUSION BUY",
                "FUSION SELL",
                None,
                Some(&regime_s),
                Some(&conf_s),
                Some(&atr_series),
                scalp_fusion,
            ));
        }
        StrategyMode::SignalStrategy => {
            match signal_source.as_str() {
                "oracle" => {
                    let (cd, lb, mb, ms, fb, fs) = if scalp_oracle {
                        (
                            match tf.as_str() {
                                "1m" => 2,
                                "3m" => 2,
                                "5m" => 3,
                                "10m" => 3,
                                "15m" => 3,
                                "30m" => 2,
                                "1h" => 2,
                                "4h" => 2,
                                _ => 3,
                            },
                            14,
                            0.8,
                            -0.8,
                            1.5,
                            -1.5,
                        )
                    } else {
                        (
                            match tf.as_str() {
                                "1m" => 6,
                                "3m" => 5,
                                "5m" => 5,
                                "10m" => 4,
                                "15m" => 4,
                                "30m" => 3,
                                "1h" => 3,
                                "4h" => 3,
                                _ => 4,
                            },
                            22,
                            1.5,
                            -1.5,
                            3.0,
                            -3.0,
                        )
                    };
                    markers.extend(generate_zone_signals(
                        &oracle_scores,
                        &candles,
                        mb,
                        ms,
                        fb,
                        fs,
                        lb,
                        cd,
                        signal_buy_color,
                        signal_sell_color,
                        signal_buy_text,
                        signal_sell_text,
                        None,
                        Some(&regime_s),
                        Some(&conf_s),
                        Some(&atr_series),
                        scalp_oracle,
                    ));
                }
                "gpu" => {
                    let (cd, lb, mb, ms, fb, fs) = if scalp_gpu {
                        (
                            match tf.as_str() {
                                "1m" => 2,
                                "3m" => 2,
                                "5m" => 3,
                                "10m" => 2,
                                "15m" => 2,
                                "30m" => 2,
                                "1h" => 2,
                                "4h" => 2,
                                _ => 2,
                            },
                            12,
                            0.00015,
                            -0.00015,
                            0.00035,
                            -0.00035,
                        )
                    } else {
                        (
                            match tf.as_str() {
                                "1m" => 5,
                                "3m" => 4,
                                "5m" => 4,
                                "10m" => 3,
                                "15m" => 3,
                                "30m" => 3,
                                "1h" => 3,
                                "4h" => 3,
                                _ => 3,
                            },
                            18,
                            0.00025,
                            -0.00025,
                            0.00060,
                            -0.00060,
                        )
                    };
                    let gpu_valid = gpu_scores.iter().flatten().count();
                    let gpu_nonzero = gpu_scores
                        .iter()
                        .flatten()
                        .filter(|v| v.abs() > 1e-8)
                        .count();
                    tracing::debug!(
                        "SIGNAL_GPU [{} {}]: valid_scores={} nonzero_scores={}",
                        symbol,
                        tf,
                        gpu_valid,
                        gpu_nonzero
                    );
                    markers.extend(generate_zone_signals(
                        &gpu_scores,
                        &candles,
                        mb,
                        ms,
                        fb,
                        fs,
                        lb,
                        cd,
                        signal_buy_color,
                        signal_sell_color,
                        signal_buy_text,
                        signal_sell_text,
                        Some(&hist.ema_50),
                        Some(&regime_s),
                        Some(&conf_s),
                        Some(&atr_series),
                        scalp_gpu,
                    ));
                }
                _ => {
                    let mut fusion_scores: Vec<Option<f64>> = vec![None; candles.len()];
                    for (i, slot) in fusion_scores.iter_mut().enumerate() {
                        let bar_regime = regime_s
                            .get(i)
                            .and_then(|v| *v)
                            .unwrap_or(titan_oracle::MarketRegime::Unknown);
                        let dynamic_alpha = match bar_regime {
                            titan_oracle::MarketRegime::Trending => (alpha * 1.3).min(0.8),
                            titan_oracle::MarketRegime::MeanReverting => (alpha * 0.5).max(0.2),
                            titan_oracle::MarketRegime::Chaotic => (alpha * 0.3).max(0.1),
                            titan_oracle::MarketRegime::Unknown => alpha,
                        };
                        let o = oracle_scores.get(i).and_then(|v| *v);
                        let g = gpu_scores.get(i).and_then(|v| *v);
                        *slot = match (o, g) {
                            (Some(ov), Some(gv)) => {
                                Some(ov * (1.0 - dynamic_alpha) + (gv * 100.0) * dynamic_alpha)
                            }
                            (Some(ov), None) => Some(ov),
                            (None, Some(gv)) => Some(gv * 100.0),
                            _ => None,
                        };
                    }
                    let (cd, lb, mb, ms, fb, fs) = if scalp_fusion {
                        (
                            match tf.as_str() {
                                "1m" => 2,
                                "3m" => 2,
                                "5m" => 3,
                                "10m" => 3,
                                "15m" => 3,
                                "30m" => 2,
                                "1h" => 2,
                                "4h" => 2,
                                _ => 3,
                            },
                            14,
                            0.8,
                            -0.8,
                            1.5,
                            -1.5,
                        )
                    } else {
                        (
                            match tf.as_str() {
                                "1m" => 6,
                                "3m" => 5,
                                "5m" => 5,
                                "10m" => 4,
                                "15m" => 4,
                                "30m" => 3,
                                "1h" => 3,
                                "4h" => 3,
                                _ => 4,
                            },
                            22,
                            1.5,
                            -1.5,
                            3.0,
                            -3.0,
                        )
                    };
                    markers.extend(generate_zone_signals(
                        &fusion_scores,
                        &candles,
                        mb,
                        ms,
                        fb,
                        fs,
                        lb,
                        cd,
                        signal_buy_color,
                        signal_sell_color,
                        signal_buy_text,
                        signal_sell_text,
                        None,
                        Some(&regime_s),
                        Some(&conf_s),
                        Some(&atr_series),
                        scalp_fusion,
                    ));
                }
            }
        }
        StrategyMode::SmartTrend => {
            let st_config = {
                let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                s.smarttrend.clone()
            };
            if st_config.st_enabled && !st_config.st_killed {
                let mut engine = crate::smarttrend::SmartTrendEngine::new(st_config);
                let warmup = 50_usize;
                for (i, candle) in candles.iter().enumerate().skip(warmup) {
                    let oracle_score = oracle_scores.get(i).and_then(|v| *v).unwrap_or(0.0);
                    let oracle_conf = conf_s.get(i).and_then(|v| *v).unwrap_or(0.0);
                    let regime = regime_s.get(i).and_then(|v| *v)
                        .unwrap_or(titan_oracle::MarketRegime::Unknown);
                    let cp_prob = hist.oracle.as_ref()
                        .and_then(|o| o.changepoint_series.get(i).and_then(|v| *v))
                        .unwrap_or(0.0);
                    let kalman_gain = hist.oracle.as_ref()
                        .and_then(|o| o.kalman_gain_series.get(i).and_then(|v| *v))
                        .unwrap_or(0.0);
                    let price_surprise = hist.oracle.as_ref()
                        .and_then(|o| o.price_surprise_series.get(i).and_then(|v| *v))
                        .unwrap_or(0.0);
                    let run_length = hist.oracle.as_ref()
                        .and_then(|o| o.run_length_series.get(i).and_then(|v| *v))
                        .unwrap_or(0.0) as usize;
                    let dom_cycle = hist.oracle.as_ref()
                        .and_then(|o| o.dominant_cycle_series.get(i).and_then(|v| *v))
                        .unwrap_or(20.0);
                    let gpu_buy = gpu_scores.get(i).and_then(|v| *v)
                        .map(|g| if g > 0.0 { (g * 100.0).clamp(0.0, 100.0) } else { 0.0 })
                        .unwrap_or(50.0);
                    let gpu_sell = gpu_scores.get(i).and_then(|v| *v)
                        .map(|g| if g < 0.0 { (g.abs() * 100.0).clamp(0.0, 100.0) } else { 0.0 })
                        .unwrap_or(50.0);

                    let velocity = if i >= 2 {
                        let prev = oracle_scores.get(i - 1).and_then(|v| *v).unwrap_or(0.0);
                        oracle_score - prev
                    } else {
                        0.0
                    };

                    let input = crate::smarttrend::SmartTrendInput {
                        macro_input: crate::smarttrend::MacroInput {
                            oracle_score,
                            confidence: oracle_conf,
                            regime,
                            kalman_gain,
                            price_surprise,
                            kalman_velocity: velocity,
                            run_length,
                            changepoint_prob: cp_prob,
                        },
                        micro_input: crate::smarttrend::MicroInput {
                            ofi_value: 0.0,
                            microprice: 0.0,
                            mid_price: 0.0,
                            cvd: 0.0,
                            spread_bps: 0.0,
                            imbalance_ratio: 0.0,
                            has_live_data: false,
                        },
                        momentum_input: crate::smarttrend::MomentumInput {
                            gpu_buy_pct: gpu_buy,
                            gpu_sell_pct: gpu_sell,
                            dominant_cycle: dom_cycle as usize,
                            momentum_bars: 0,
                            rvol: 0.0,
                        },
                        current_price: candle.close,
                        changepoint_prob: cp_prob,
                        dominant_cycle: dom_cycle,
                        tf_minutes: crate::candle_buffer::timeframe_to_secs(
                            &tf
                        ) as f64 / 60.0,
                    };
                    let result = engine.evaluate(&input);
                    st_score_points.push(StScorePoint {
                        time: candle.time,
                        score: result.score,
                        state: format!("{}", result.state),
                    });
                    for mk in &result.markers {
                        let (position, color, shape, text) = match mk.kind {
                            crate::smarttrend::SmartTrendMarkerKind::Entry => {
                                let regime_color = match regime {
                                    titan_oracle::MarketRegime::Trending => COLOR_ST_BUY,
                                    titan_oracle::MarketRegime::MeanReverting => "rgba(0,229,255,0.6)",
                                    titan_oracle::MarketRegime::Chaotic => "rgba(0,229,255,0.5)",
                                    titan_oracle::MarketRegime::Unknown => "rgba(0,229,255,0.4)",
                                };
                                (
                                    "belowBar",
                                    regime_color,
                                    "arrowUp",
                                    format!("ST {}", if matches!(mk.direction, crate::smarttrend::PillarDirection::Bullish) { "BUY" } else { "SELL" }),
                                )
                            }
                            crate::smarttrend::SmartTrendMarkerKind::Exit => (
                                "aboveBar",
                                COLOR_ST_EXIT,
                                "arrowDown",
                                "ST EXIT".to_string(),
                            ),
                            crate::smarttrend::SmartTrendMarkerKind::Trail => (
                                "aboveBar",
                                COLOR_ST_TRAIL,
                                "circle",
                                "ST TRAIL".to_string(),
                            ),
                            crate::smarttrend::SmartTrendMarkerKind::Stalking => (
                                "belowBar",
                                COLOR_ST_STALK,
                                "circle",
                                "ST STALK".to_string(),
                            ),
                            crate::smarttrend::SmartTrendMarkerKind::ChangePointReset => (
                                "aboveBar",
                                COLOR_CHANGEPOINT,
                                "circle",
                                "ST BREAK".to_string(),
                            ),
                        };
                        markers.push(SignalMarker {
                            time: candle.time,
                            position: position.to_string(),
                            color: color.to_string(),
                            shape: shape.to_string(),
                            text,
                        });
                    }
                }
            }
        }
    }

    // Exit signals — SmartTrend handles exits via its own FSM markers
    if mode != StrategyMode::SmartTrend {
        let is_scalping = match mode {
            StrategyMode::MathOracle => scalp_oracle,
            StrategyMode::GpuNexus => scalp_gpu,
            StrategyMode::Fusion => scalp_fusion,
            StrategyMode::SignalStrategy => scalp_oracle || scalp_gpu || scalp_fusion,
            StrategyMode::SmartTrend => scalp_fusion,
        };
        let (trail_mult, trail_lo, trail_hi) = if is_scalping {
            (0.5, 0.15, 2.5)
        } else {
            (1.0, 0.3, 5.0)
        };
        let (flip_long, flip_short, flip_conf, chaos_conf) = if is_scalping {
            (-10.0_f64, 10.0_f64, 0.20_f64, 0.25_f64)
        } else {
            (-20.0, 20.0, 0.30, 0.40)
        };

        let mut in_long = false;
        let mut in_short = false;
        let mut entry_price = 0.0_f64;
        let mut highest_since = 0.0_f64;
        let mut lowest_since = f64::MAX;
        let (exit_color, exit_prefix): (&str, Option<&str>) = match mode {
            StrategyMode::SignalStrategy => (signal_exit_color, Some(signal_prefix)),
            _ => (COLOR_DEFAULT_EXIT, None),
        };

        let buy_times: HashSet<i64> = markers
            .iter()
            .filter(|m| m.shape == "arrowUp" && m.text.ends_with("BUY"))
            .map(|m| m.time)
            .collect();
        let sell_times: HashSet<i64> = markers
            .iter()
            .filter(|m| m.shape == "arrowDown" && m.text.ends_with("SELL"))
            .map(|m| m.time)
            .collect();

        for (i, candle) in candles.iter().enumerate() {
            let price = candle.close;
            let oracle_val = oracle_scores.get(i).and_then(|v| *v).unwrap_or(0.0);
            let conf = conf_s.get(i).and_then(|v| *v).unwrap_or(0.0);
            let regime = regime_s
                .get(i)
                .and_then(|v| *v)
                .unwrap_or(titan_oracle::MarketRegime::Unknown);
            let atr = atr_series.get(i).and_then(|v| *v).unwrap_or(0.0);
            let atr_pct = if price > 0.0 {
                (atr / price) * 100.0
            } else {
                1.0
            };
            let trail_pct = (atr_pct * trail_mult).clamp(trail_lo, trail_hi);

            let has_buy = buy_times.contains(&candle.time);
            let has_sell = sell_times.contains(&candle.time);

            if has_buy && !in_long {
                in_long = true;
                in_short = false;
                entry_price = price;
                highest_since = price;
                lowest_since = f64::MAX;
            } else if has_sell && !in_short {
                in_short = true;
                in_long = false;
                entry_price = price;
                lowest_since = price;
                highest_since = 0.0;
            }

            if in_long {
                if price > highest_since {
                    highest_since = price;
                }
                let trail_drop = if highest_since > 0.0 {
                    ((highest_since - price) / highest_since) * 100.0
                } else {
                    0.0
                };

                let oracle_flip = oracle_val < flip_long && conf > flip_conf;
                let trailing_hit = highest_since > entry_price && trail_drop >= trail_pct;
                let chaos_exit = matches!(regime, titan_oracle::MarketRegime::Chaotic)
                    && conf > chaos_conf
                    && price < entry_price;

                if oracle_flip || trailing_hit || chaos_exit {
                    let reason = if oracle_flip {
                        "EXIT"
                    } else if trailing_hit {
                        "TRAIL"
                    } else {
                        "CHAOS"
                    };
                    let text = match exit_prefix {
                        Some(prefix) => format!("{prefix} {reason}"),
                        None => reason.to_string(),
                    };
                    markers.push(SignalMarker {
                        time: candle.time,
                        position: "aboveBar".to_string(),
                        color: exit_color.to_string(),
                        shape: "arrowDown".to_string(),
                        text,
                    });
                    in_long = false;
                }
            } else if in_short {
                if price < lowest_since {
                    lowest_since = price;
                }
                let trail_rise = if lowest_since > 0.0 {
                    ((price - lowest_since) / lowest_since) * 100.0
                } else {
                    0.0
                };

                let oracle_flip = oracle_val > flip_short && conf > flip_conf;
                let trailing_hit = lowest_since < entry_price && trail_rise >= trail_pct;
                let chaos_exit = matches!(regime, titan_oracle::MarketRegime::Chaotic)
                    && conf > chaos_conf
                    && price > entry_price;

                if oracle_flip || trailing_hit || chaos_exit {
                    let reason = if oracle_flip {
                        "EXIT"
                    } else if trailing_hit {
                        "TRAIL"
                    } else {
                        "CHAOS"
                    };
                    let text = match exit_prefix {
                        Some(prefix) => format!("{prefix} {reason}"),
                        None => reason.to_string(),
                    };
                    markers.push(SignalMarker {
                        time: candle.time,
                        position: "belowBar".to_string(),
                        color: exit_color.to_string(),
                        shape: "arrowUp".to_string(),
                        text,
                    });
                    in_short = false;
                }
            }
        }
    }

    if let Some(ref oracle_hist) = hist.oracle {
        for (i, candle) in candles.iter().enumerate().skip(1) {
            if let Some(cp_prob) = oracle_hist.changepoint_series.get(i).and_then(|v| *v) {
                if cp_prob > 0.7 {
                    markers.push(SignalMarker {
                        time: candle.time,
                        position: "aboveBar".to_string(),
                        color: COLOR_CHANGEPOINT.to_string(),
                        shape: "circle".to_string(),
                        text: "BREAK".to_string(),
                    });
                }
            }
        }
    }

    let raw_count = markers.len();
    let entry_count = markers
        .iter()
        .filter(|m| m.text.ends_with("BUY") || m.text.ends_with("SELL"))
        .count();
    let exit_count = markers
        .iter()
        .filter(|m| is_exit_marker(&m.text))
        .count();

    // Multi-timeframe confirmation
    let htf = match tf.as_str() {
        "1m" => Some("5m"),
        "3m" => Some("15m"),
        "5m" => Some("15m"),
        "10m" => Some("1h"),
        "15m" => Some("1h"),
        "30m" => Some("4h"),
        "1h" => Some("4h"),
        "4h" => Some("1d"),
        _ => None,
    };

    if let Some(htf_str) = htf {
        if let Ok(htf_candles) = fetch_bars_internal(state.clone(), &symbol, htf_str).await {
            if htf_candles.len() >= 60 {
                let htf_snapshot = titan_oracle::calculate(&htf_candles, &oracle_w);
                let htf_direction =
                    if htf_snapshot.oracle_score > 5.0 && htf_snapshot.confidence > 0.15 {
                        1
                    } else if htf_snapshot.oracle_score < -5.0 && htf_snapshot.confidence > 0.15 {
                        -1
                    } else {
                        0
                    };

                if htf_direction != 0 {
                    for m in markers.iter_mut() {
                        if is_exit_marker(&m.text) {
                            continue;
                        }
                        let blocked = (m.shape == "arrowUp" && htf_direction < 0)
                            || (m.shape == "arrowDown" && htf_direction > 0);
                        if blocked {
                            m.color = COLOR_HTF_BLOCKED.to_string();
                            m.text = format!("~{}", m.text);
                        }
                    }
                }
            }
        }
    }

    markers.sort_by(|a, b| a.time.cmp(&b.time).then_with(|| a.text.cmp(&b.text)));
    markers.dedup_by(|a, b| a.time == b.time && a.text == b.text);

    if markers.len() > 300 {
        markers = markers[markers.len() - 300..].to_vec();
    }

    let htf_removed = raw_count.saturating_sub(markers.len());
    tracing::debug!(
        "MARKERS [{}]: {} final (raw={} [entries={} exits={}] htf_removed={}) for {} {} ({} candles)",
        mode,
        markers.len(),
        raw_count,
        entry_count,
        exit_count,
        htf_removed,
        symbol,
        tf,
        candles.len()
    );
    let reason = if markers.is_empty() {
        let valid_scores = oracle_scores.iter().filter(|s| s.is_some()).count();
        let active_scalp = match mode {
            StrategyMode::MathOracle => scalp_oracle,
            StrategyMode::GpuNexus => scalp_gpu,
            StrategyMode::Fusion => scalp_fusion,
            StrategyMode::SignalStrategy => match signal_source.as_str() {
                "oracle" => scalp_oracle,
                "gpu" => scalp_gpu,
                _ => scalp_fusion,
            },
            StrategyMode::SmartTrend => scalp_fusion,
        };
        Some(format!(
            "No signals: {} candles, {} valid scores, mode={}, scalp={}",
            candles.len(), valid_scores, mode, active_scalp
        ))
    } else {
        None
    };
    let st_scores = if st_score_points.is_empty() { None } else { Some(st_score_points) };
    Ok(SignalMarkersResponse { markers, reason, st_scores })
}

// ═══════════════════════════════════════════════════════════════
//  AI SCANNER SCORING
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn score_candidate(
    state: &Arc<TitanState>,
    pf: &scanner::PreFilterCandidate,
    fusion_alpha: f64,
    allow_short: bool,
) -> Option<scanner::AiScanCandidate> {
    const MIN_BARS: usize = 15;
    let fetch_result: Result<Vec<Candle>, TitanError> =
        match fetch_bars_internal(state.clone(), &pf.symbol, "5m").await {
            Ok(c) if c.len() >= MIN_BARS => Ok(c),
            Ok(c) => Err(TitanError::Validation(format!(
                "only {} bars (need {})",
                c.len(),
                MIN_BARS
            ))),
            Err(e) => Err(e),
        };
    let candles = match fetch_result {
        Ok(c) => c,
        Err(first_err) => {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            match fetch_bars_internal(state.clone(), &pf.symbol, "5m").await {
                Ok(c) if c.len() >= MIN_BARS => c,
                Ok(c) => {
                    tracing::warn!(
                        "SCANNER DROP {} (retry): only {} bars (need {})",
                        pf.symbol,
                        c.len(),
                        MIN_BARS
                    );
                    return None;
                }
                Err(e2) => {
                    tracing::warn!(
                        "SCANNER DROP {} (retry): {} | original: {}",
                        pf.symbol,
                        e2,
                        first_err
                    );
                    return None;
                }
            }
        }
    };

    let oracle_snap = titan_oracle::calculate(&candles, &titan_oracle::OracleWeights::default());
    let oracle_score = oracle_snap.oracle_score;
    let oracle_conf = oracle_snap.confidence;
    let oracle_regime = format!("{:?}", oracle_snap.dominant_regime);

    let last_close = candles.last().map(|c| c.close as f32).unwrap_or(0.0);
    let prev_close = if candles.len() >= 2 {
        candles[candles.len() - 2].close as f32
    } else {
        last_close
    };
    let velocity = if prev_close > 0.0 {
        (last_close - prev_close) / prev_close
    } else {
        0.0
    };
    let vol_f32 = candles.iter().map(|c| c.close).collect::<Vec<f64>>();
    let volatility = if vol_f32.len() >= 20 {
        let slice = &vol_f32[vol_f32.len() - 20..];
        let mean = slice.iter().sum::<f64>() / slice.len() as f64;
        let var = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / slice.len() as f64;
        var.sqrt() as f32 / mean as f32
    } else {
        0.01
    };

    let (nexus_buy_pct, nexus_consensus) = {
        #[cfg(feature = "gpu")]
        {
            match gpu::nexus_dispatch_stateless(last_close, velocity, volatility).await {
                Ok(nr) => (nr.buy_pct, nr.consensus),
                Err(_) => (50.0, "N/A".to_string()),
            }
        }
        #[cfg(not(feature = "gpu"))]
        {
            (50.0, "N/A".to_string())
        }
    };

    let nexus_signal = if nexus_buy_pct > 60.0 {
        1.0
    } else if nexus_buy_pct < 40.0 {
        -1.0
    } else {
        0.0
    };
    let oracle_norm = oracle_score / 100.0;
    let fusion_raw = oracle_norm * (1.0 - fusion_alpha) + nexus_signal * fusion_alpha;
    let fusion_score = (fusion_raw * 100.0).clamp(-100.0, 100.0);
    let fusion_signal = if fusion_score > 30.0 && oracle_conf > 0.2 {
        "LONG"
    } else if fusion_score < -30.0 && oracle_conf > 0.2 {
        "SHORT"
    } else {
        "NEUTRAL"
    };

    let sweep_tfs = ["1m", "10m", "15m", "30m", "1h", "4h", "1d"];
    let mut best_bt = compute_backtest_metrics(&candles, allow_short);
    let mut best_tf = "5m".to_string();
    let mut best_score = best_bt.sharpe + best_bt.return_pct * 0.01;
    for &stf in &sweep_tfs {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Ok(tf_candles) = fetch_bars_internal(state.clone(), &pf.symbol, stf).await {
            if tf_candles.len() >= MIN_BARS {
                let tf_bt = compute_backtest_metrics(&tf_candles, allow_short);
                let tf_score = tf_bt.sharpe + tf_bt.return_pct * 0.01;
                if tf_score > best_score {
                    best_score = tf_score;
                    best_bt = tf_bt;
                    best_tf = stf.to_string();
                }
            }
        }
    }
    let bt = best_bt;

    let fusion_normalized = (fusion_score + 100.0) / 200.0 * 100.0;
    let sharpe_normalized = bt.sharpe.clamp(0.0, 5.0) / 5.0 * 100.0;
    let pf_normalized = bt.profit_factor.clamp(0.0, 5.0) / 5.0 * 100.0;

    let ai_score = if bt.trades >= 2 && oracle_conf > 0.15 {
        0.40 * fusion_normalized
            + 0.25 * sharpe_normalized
            + 0.20 * bt.win_rate
            + 0.15 * pf_normalized
    } else {
        fusion_normalized * 0.5
    };

    let float_category = if pf.volume > 10_000_000.0 {
        "HIGH"
    } else if pf.volume > 3_000_000.0 {
        "MED"
    } else {
        "LOW"
    };

    let mut has_catalyst = false;
    let mut catalyst_headline = String::new();
    let mut catalyst_url = String::new();
    {
        let (key, secret) = {
            let k = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (k.key.clone(), k.secret.clone())
        };
        if !key.is_empty() {
            let articles =
                crate::sentiment::fetch_news_feed(&state.http_client, &pf.symbol, &key, &secret, 5)
                    .await;
            if let Some(best) = articles.iter().max_by(|a, b| {
                a.sentiment_score
                    .abs()
                    .partial_cmp(&b.sentiment_score.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                if best.sentiment_score.abs() > 0.1 {
                    has_catalyst = true;
                    catalyst_headline = best.headline.clone();
                    catalyst_url = best.url.clone();
                }
            }
        }
    }

    let (smart_score, bt_pillar, momentum_pillar, quality_pillar) = {
        let bt_p =
            if bt.trades < 3 || bt.sharpe <= 0.0 || bt.win_rate < 15.0 || bt.expectancy < -0.01 {
                fusion_normalized * 0.10
            } else {
                let trades_norm = (bt.trades.min(30) as f64 / 30.0) * 100.0;
                let sharpe_norm = bt.sharpe.clamp(0.0, 4.0) / 4.0 * 100.0;
                let pf_norm = bt.profit_factor.clamp(0.0, 4.0) / 4.0 * 100.0;
                let expect_norm = (bt.expectancy * 100.0).clamp(0.0, 80.0);
                let base = 0.30 * bt.win_rate
                    + 0.25 * sharpe_norm
                    + 0.20 * trades_norm
                    + 0.15 * pf_norm
                    + 0.10 * expect_norm;

                let win_mult = if bt.win_rate < 30.0 {
                    0.25
                } else if bt.win_rate < 38.0 {
                    0.45
                } else if bt.win_rate < 45.0 {
                    0.60
                } else if bt.win_rate < 50.0 {
                    0.82
                } else if bt.win_rate >= 70.0 {
                    1.20
                } else if bt.win_rate >= 60.0 {
                    1.12
                } else {
                    1.0
                };
                let trades_mult = if bt.trades < 5 {
                    0.35
                } else if bt.trades < 8 {
                    0.45
                } else if bt.trades < 12 {
                    0.85
                } else if bt.trades >= 20 {
                    1.05
                } else {
                    1.0
                };
                let sharpe_mult = if bt.sharpe < 0.3 {
                    0.45
                } else if bt.sharpe < 0.7 {
                    0.70
                } else if bt.sharpe < 1.0 {
                    0.88
                } else if bt.sharpe >= 2.0 {
                    1.12
                } else {
                    1.0
                };
                let pf_mult = if bt.profit_factor < 0.8 {
                    0.55
                } else if bt.profit_factor < 1.0 {
                    0.75
                } else if bt.profit_factor < 1.2 {
                    0.90
                } else {
                    1.0
                };
                let expect_mult = if bt.expectancy < 0.0 {
                    0.40
                } else if bt.expectancy < 0.003 {
                    0.80
                } else {
                    1.0
                };
                let consistency = if bt.win_rate >= 50.0 && bt.sharpe >= 1.0 && bt.trades >= 10 {
                    1.15
                } else if bt.win_rate >= 55.0 && bt.sharpe >= 1.0 {
                    1.08
                } else {
                    1.0
                };
                base * win_mult * trades_mult * sharpe_mult * pf_mult * expect_mult * consistency
            };

        let chg_score = if pf.change_percent > 10.0 {
            100.0
        } else if pf.change_percent > 5.0 {
            80.0
        } else if pf.change_percent > 2.0 {
            60.0
        } else if pf.change_percent > 0.0 {
            40.0
        } else if pf.change_percent > -2.0 {
            20.0
        } else if pf.change_percent > -5.0 {
            10.0
        } else {
            0.0
        };
        let rvol_score = if pf.rvol > 5.0 {
            100.0
        } else if pf.rvol > 3.0 {
            90.0
        } else if pf.rvol > 2.0 {
            75.0
        } else if pf.rvol > 1.0 {
            50.0
        } else {
            20.0
        };
        let regime_score = match oracle_regime.as_str() {
            "Trending" => 100.0,
            "MeanReverting" => 60.0,
            "Unknown" => 40.0,
            _ => 20.0,
        };
        let mom_p = 0.40 * chg_score + 0.30 * rvol_score + 0.30 * regime_score;

        let liquidity_mult = match float_category {
            "HIGH" => 1.0,
            "MED" => 0.92,
            _ => {
                if pf.price < 3.0 {
                    0.50
                } else {
                    0.75
                }
            }
        };
        let catalyst_mult = if has_catalyst { 1.20 } else { 1.0 };
        let oracle_dir = if oracle_score > 0.0 { 1.0 } else { -1.0 };
        let price_dir = if pf.change_percent > 0.0 { 1.0 } else { -1.0 };
        let alignment_mult = if oracle_dir == price_dir { 1.10 } else { 0.75 };
        let qual_p = 100.0 * liquidity_mult * catalyst_mult * alignment_mult;

        (
            0.50 * bt_p + 0.35 * mom_p + 0.15 * qual_p,
            bt_p,
            mom_p,
            qual_p,
        )
    };

    Some(scanner::AiScanCandidate {
        rank: 0,
        symbol: pf.symbol.clone(),
        price: pf.price,
        change_percent: pf.change_percent,
        volume: pf.volume,
        rvol: pf.rvol,
        gap_percent: pf.gap_percent,
        multi_day_strength: pf.change_percent,
        float_category: float_category.to_string(),
        prefilter_score: pf.prefilter_score,
        oracle_score,
        oracle_confidence: oracle_conf,
        oracle_regime,
        nexus_consensus,
        nexus_buy_pct,
        fusion_score,
        fusion_signal: fusion_signal.to_string(),
        bt_best_tf: best_tf,
        bt_trades: bt.trades,
        bt_win_rate: bt.win_rate,
        bt_return_pct: bt.return_pct,
        bt_sharpe: bt.sharpe,
        bt_profit_factor: bt.profit_factor,
        bt_max_drawdown: bt.max_drawdown,
        bt_expectancy: bt.expectancy,
        ai_score,
        smart_score,
        has_catalyst,
        catalyst_headline,
        catalyst_url,
        reasoning: format!(
            "P1(BT): {:.0}% | P2(MOM): {:.0}% | P3(QUAL): {:.0}% | Win: {:.1}% | Vol: {:.1}x",
            bt_pillar, momentum_pillar, quality_pillar, bt.win_rate, pf.rvol
        ),
        market_cap: 0.0,
        insider_signal: String::new(),
    })
}

pub(crate) async fn score_prefiltered_candidates(
    state: &Arc<TitanState>,
    prefiltered: Vec<scanner::PreFilterCandidate>,
    snap_count: usize,
    allow_short: bool,
    fusion_alpha: f64,
    app: &tauri::AppHandle,
) -> ScanScoringResult {
    let total = prefiltered.len();
    let mut results: Vec<scanner::AiScanCandidate> = Vec::with_capacity(total);
    let mut dropped: usize = 0;

    {
        let mut queue = state
            .scan
            .scan_prefilter_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        queue.clear();
        queue.extend(prefiltered.iter().cloned());
    }
    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (true, 0, total);

    let cancel_token = {
        let mut ct = state.scan.scan_cancel.lock().unwrap_or_else(|e| e.into_inner());
        ct.cancel();
        *ct = CancellationToken::new();
        ct.clone()
    };

    use futures::StreamExt;
    let mut results_stream = futures::stream::iter(prefiltered.into_iter())
        .map(|pf| {
            let st = state.clone();
            async move { score_candidate(&st, &pf, fusion_alpha, allow_short).await }
        })
        .buffer_unordered(10);

    let mut idx = 0;
    while let Some(maybe_candidate) = results_stream.next().await {
        if cancel_token.is_cancelled() {
            break;
        }
        idx += 1;
        if idx % 5 == 0 {
            let _ = app.emit_all(
                "ai-scan-progress",
                serde_json::json!({
                    "phase": "fusion", "done": idx, "total": total,
                    "scored": results.len(), "dropped": dropped
                }),
            );
            *state
                .scan
                .scan_progress
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = (true, idx, total);
        }
        if let Some(candidate) = maybe_candidate {
            results.push(candidate);
        } else {
            dropped += 1;
        }
    }

    let scored = results.len();

    results.sort_by(|a, b| {
        b.ai_score
            .partial_cmp(&a.ai_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    {
        let mut tf_map = state
            .scan
            .symbol_best_tf
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for r in &results {
            tf_map.insert(
                normalize_symbol(&r.symbol),
                normalize_timeframe_internal(&r.bt_best_tf),
            );
        }
    }

    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (false, total, total);
    state
        .scan
        .scan_prefilter_queue
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();

    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({
            "phase": "done", "done": total, "total": total,
            "scored": scored, "dropped": dropped,
            "prefiltered": total, "total_snapshots": snap_count
        }),
    );
    let _ = app.emit_all("ai-scan-update", &results);
    let _ = app.emit_all("log-message", format!("🔍 AI SCANNER: {scored}/{total} scored, {dropped} dropped, from {snap_count} snapshots"));

    *state
        .scan
        .last_scan_results
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = results.clone();

    ScanScoringResult {
        results,
        scored,
        dropped,
        total,
        snap_count,
    }
}

// ═══════════════════════════════════════════════════════════════
//  AUTO-RESUME SCAN
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn auto_resume_scan(
    state: Arc<TitanState>,
    app: tauri::AppHandle,
    saved_queue: Vec<scanner::PreFilterCandidate>,
    partial_results: Vec<scanner::AiScanCandidate>,
) -> Result<(), TitanError> {
    let already_done: HashSet<String> = partial_results.iter().map(|r| r.symbol.clone()).collect();
    let remaining: Vec<scanner::PreFilterCandidate> = saved_queue
        .into_iter()
        .filter(|pf| !already_done.contains(&pf.symbol))
        .collect();

    let done_count = partial_results.len();
    let total = done_count + remaining.len();

    let _ = app.emit_all(
        "log-message",
        format!(
            "AUTO-RESUME: Continuing scan from {}/{} ({} remaining)",
            done_count,
            total,
            remaining.len()
        ),
    );
    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({
            "phase": "fusion", "done": done_count, "total": total
        }),
    );

    if remaining.is_empty() {
        let mut results = partial_results;
        results.sort_by(|a, b| {
            b.ai_score
                .partial_cmp(&a.ai_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, r) in results.iter_mut().enumerate() {
            r.rank = i + 1;
        }
        *state
            .scan
            .last_scan_results
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = results.clone();
        *state
            .scan
            .scan_progress
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = (false, total, total);
        state
            .scan
            .scan_prefilter_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        let _ = app.emit_all(
            "ai-scan-progress",
            serde_json::json!({"phase": "done", "done": total, "total": total}),
        );
        let _ = app.emit_all("ai-scan-update", &results);
        let _ = app.emit_all(
            "log-message",
            format!(
                "AUTO-RESUME: Scan was already complete — {} assets ranked",
                results.len()
            ),
        );
        return Ok(());
    }

    let allow_short = {
        state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .allow_short
    };
    let fusion_alpha = {
        *state
            .nexus_fusion_alpha
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    };

    let mut new_results: Vec<scanner::AiScanCandidate> = Vec::with_capacity(remaining.len());
    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (true, done_count, total);

    let cancel_token = {
        let mut ct = state.scan.scan_cancel.lock().unwrap_or_else(|e| e.into_inner());
        ct.cancel();
        *ct = CancellationToken::new();
        ct.clone()
    };
    for (idx, pf) in remaining.into_iter().enumerate() {
        if cancel_token.is_cancelled() {
            break;
        }
        let global_idx = done_count + idx;
        if idx % 5 == 0 {
            let _ = app.emit_all(
                "ai-scan-progress",
                serde_json::json!({
                    "phase": "fusion", "done": global_idx, "total": total, "symbol": &pf.symbol
                }),
            );
            *state
                .scan
                .scan_progress
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = (true, global_idx, total);
        }
        if let Some(candidate) = score_candidate(&state, &pf, fusion_alpha, allow_short).await {
            new_results.push(candidate);
        }
        let mut merged_so_far = partial_results.clone();
        merged_so_far.extend(new_results.clone());
        *state
            .scan
            .last_scan_results
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = merged_so_far;
    }

    let mut results = partial_results;
    results.extend(new_results);
    results.sort_by(|a, b| {
        b.ai_score
            .partial_cmp(&a.ai_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    {
        let mut tf_map = state
            .scan
            .symbol_best_tf
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for r in &results {
            tf_map.insert(
                normalize_symbol(&r.symbol),
                normalize_timeframe_internal(&r.bt_best_tf),
            );
        }
    }

    *state
        .scan
        .last_scan_results
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = results.clone();
    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (false, total, total);
    state
        .scan
        .scan_prefilter_queue
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();

    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({"phase": "done", "done": total, "total": total}),
    );
    let _ = app.emit_all("ai-scan-update", &results);
    let _ = app.emit_all(
        "log-message",
        format!(
            "AUTO-RESUME SCAN: Completed — {} assets ranked (resumed from {})",
            results.len(),
            done_count
        ),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_marker_recognizes_standard_exits() {
        assert!(is_exit_marker("O EXIT"));
        assert!(is_exit_marker("GPU TRAIL"));
        assert!(is_exit_marker("F CHAOS"));
        assert!(is_exit_marker("BREAK"));
        assert!(is_exit_marker("EXIT"));
    }

    #[test]
    fn exit_marker_rejects_false_positives() {
        assert!(!is_exit_marker("PRE-EXIT ALERT"));
        assert!(!is_exit_marker("TRAILING STOP INFO"));
        assert!(!is_exit_marker("CHAOS-WARNING BUY"));
        assert!(!is_exit_marker("O BUY"));
        assert!(!is_exit_marker(""));
    }

    #[test]
    fn exit_marker_recognizes_smarttrend_exits() {
        assert!(is_exit_marker("ST EXIT"));
        assert!(is_exit_marker("ST TRAIL"));
        assert!(is_exit_marker("ST BREAK"));
        assert!(is_exit_marker("ST STALK"));
    }

    #[test]
    fn exit_marker_rejects_smarttrend_entries() {
        assert!(!is_exit_marker("ST BUY"));
        assert!(!is_exit_marker("ST SELL"));
    }

    #[test]
    fn smarttrend_scoring_branch_neutral() {
        use crate::app_state::Candle;
        let candles: Vec<Candle> = (0..5).map(|i| Candle {
            time: i,
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.0,
            volume: 1000.0,
        }).collect();
        let periods = crate::indicators::IndicatorPeriods::default();
        let oracle_w = crate::titan_oracle::OracleWeights::default();
        let score = compute_strategy_score_with_mode(
            &candles,
            StrategyMode::SmartTrend,
            0.5,
            0.0,
            &periods,
            &oracle_w,
        );
        assert!(score.score >= 0.0 && score.score <= 100.0);
    }

    #[test]
    fn smarttrend_score_colors_defined() {
        assert_eq!(COLOR_ST_BUY, "#00E5FF");
        assert_eq!(COLOR_ST_EXIT, "#FFD700");
        assert_eq!(COLOR_ST_TRAIL, "#BF00FF");
        assert_eq!(COLOR_ST_STALK, "#FFD700");
    }
}

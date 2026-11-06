//! Backtest-related Tauri commands (bodies match `main.rs`).

use std::fs;
use std::sync::Arc;

use futures_util::future::join_all;
use rand::Rng;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use tokio::time::{timeout, Duration};

use crate::app_state::Candle;
#[cfg(feature = "gpu")]
use crate::commands::data::backtest_inputs_from_gpu_batch;
use crate::commands::data::{
    build_backtest_inputs_cpu, build_backtest_inputs_gpu, compute_backtest_metrics,
    compute_backtest_metrics_from_inputs, compute_backtest_oracle, compute_gpu_scores,
    compute_signal_backtest, compute_strategy_score_with_mode, compute_volatility_cpu,
    compute_volatility_gpu, fetch_alpaca_history_internal, fetch_bars_internal,
    fetch_bars_with_period, get_signal_markers_internal,
};
use crate::error::TitanError;
use crate::genetic;
#[cfg(feature = "gpu")]
use crate::gpu;
use crate::state::{normalize_symbol, setting_or_env, StrategyMode, TitanState};
use crate::titan_oracle;
use crate::types::*;

#[tauri::command]
pub(crate) async fn optimize_timeframe(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<OptimizeResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let timeframes = vec!["1m", "3m", "5m", "10m", "15m", "30m", "1h", "4h", "1d"];
    let mut results = Vec::new();

    for tf in timeframes {
        let data =
            fetch_alpaca_history_internal(state.inner().clone(), symbol.clone(), tf.to_string())
                .await?;
        let candles = &data.candles;
        if candles.len() >= 2 {
            let first = candles.first().map(|c| c.open).unwrap_or(0.0);
            let last = candles.last().map(|c| c.close).unwrap_or(0.0);
            if first > 0.0 {
                let roi = ((last - first) / first) * 100.0;
                results.push(TimeframeScore {
                    tf: tf.to_string(),
                    roi,
                });
            }
        }
    }

    if results.is_empty() {
        return Err(TitanError::Validation("NO DATA".into()));
    }

    results.sort_by(|a, b| {
        b.roi
            .partial_cmp(&a.roi)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best = results[0].clone();
    Ok(OptimizeResult {
        best_tf: best.tf,
        best_roi: best.roi,
        results,
    })
}

#[tauri::command]
pub(crate) async fn run_backtest(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<BacktestResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }

    let (allow_short, gpu_backtest) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_backtest)
    };
    let use_gpu = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST");

    let inputs = if use_gpu {
        build_backtest_inputs_gpu(&candles)
            .await
            .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
    } else {
        build_backtest_inputs_cpu(&candles)
    };

    Ok(compute_backtest_metrics_from_inputs(
        &candles,
        allow_short,
        &inputs,
    ))
}

#[tauri::command]
pub(crate) async fn run_backtest_oracle(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<BacktestResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }
    let allow_short = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.allow_short
    };
    Ok(compute_backtest_oracle(&candles, allow_short))
}

#[tauri::command]
pub(crate) async fn run_backtest_sweep(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<BacktestSweepResult>, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let timeframes = vec!["1m", "3m", "5m", "10m", "15m", "30m", "1h", "4h", "1d"];
    let (allow_short, gpu_sweep) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_sweep)
    };
    let use_gpu = setting_or_env(gpu_sweep, "TITAN_GPU_SWEEP");

    if use_gpu {
        let futures = timeframes.into_iter().map(|tf| {
            let tf_str = tf.to_string();
            let state_cloned = state.inner().clone();
            let symbol_cloned = symbol.clone();
            async move {
                let candles = fetch_bars_internal(state_cloned, &symbol_cloned, &tf_str)
                    .await
                    .ok()?;
                if candles.len() < 60 {
                    return None;
                }
                Some((tf_str, candles))
            }
        });

        let results = join_all(futures).await;
        let mut series: Vec<(String, Vec<Candle>)> = Vec::new();
        let mut closes_list: Vec<Vec<f32>> = Vec::new();
        for item in results.into_iter().flatten() {
            let closes: Vec<f32> = item.1.iter().map(|c| c.close as f32).collect();
            series.push((item.0, item.1));
            closes_list.push(closes);
        }

        #[cfg(feature = "gpu")]
        let batch_inputs = gpu::gpu_backtest_inputs_batch(&closes_list, 9, 50, 14)
            .await
            .ok()
            .and_then(backtest_inputs_from_gpu_batch);
        #[cfg(not(feature = "gpu"))]
        let batch_inputs: Option<Vec<BacktestInputs>> = None;

        let mut clean: Vec<BacktestSweepResult> = Vec::new();
        for (idx, (tf_str, candles)) in series.into_iter().enumerate() {
            let inputs = if let Some(ref batch) = batch_inputs {
                batch
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
            } else {
                build_backtest_inputs_cpu(&candles)
            };
            let res = compute_backtest_metrics_from_inputs(&candles, allow_short, &inputs);
            clean.push(BacktestSweepResult {
                tf: tf_str,
                metrics: res.metrics,
            });
        }

        clean.sort_by(|a, b| {
            b.metrics
                .return_pct
                .partial_cmp(&a.metrics.return_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(clean)
    } else {
        let futures = timeframes.into_iter().map(|tf| {
            let tf_str = tf.to_string();
            let state_cloned = state.inner().clone();
            let symbol_cloned = symbol.clone();
            async move {
                let candles = fetch_bars_internal(state_cloned, &symbol_cloned, &tf_str)
                    .await
                    .ok()?;
                if candles.len() < 60 {
                    return None;
                }
                let res = compute_backtest_metrics(&candles, allow_short);
                Some(BacktestSweepResult {
                    tf: tf_str,
                    metrics: res.metrics,
                })
            }
        });

        let results = join_all(futures).await;
        let mut clean: Vec<BacktestSweepResult> = results.into_iter().flatten().collect();
        clean.sort_by(|a, b| {
            b.return_pct
                .partial_cmp(&a.return_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(clean)
    }
}

#[tauri::command]
pub(crate) async fn run_backtest_sweep_multi(
    symbols: Vec<String>,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<BacktestSweepMultiResult>, TitanError> {
    let mut clean_symbols: Vec<String> = symbols
        .into_iter()
        .map(|s| normalize_symbol(&s))
        .filter(|s| !s.is_empty())
        .collect();
    clean_symbols.sort();
    clean_symbols.dedup();
    if clean_symbols.is_empty() {
        return Err(TitanError::Validation("NO SYMBOLS".into()));
    }

    let timeframes = vec!["1m", "3m", "5m", "10m", "15m", "30m", "1h", "4h", "1d"];
    let (allow_short, gpu_sweep) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_sweep)
    };
    let use_gpu = setting_or_env(gpu_sweep, "TITAN_GPU_SWEEP");
    let state_arc = state.inner().clone();

    let futures = clean_symbols.iter().flat_map(|sym| {
        let state_arc = state_arc.clone();
        timeframes.iter().map(move |tf| {
            let sym = sym.clone();
            let tf_str = tf.to_string();
            let state_cloned = state_arc.clone();
            async move {
                let candles = fetch_bars_internal(state_cloned, &sym, &tf_str)
                    .await
                    .ok()?;
                if candles.len() < 60 {
                    return None;
                }
                Some((sym, tf_str, candles))
            }
        })
    });

    let results = join_all(futures).await;
    let mut series: Vec<(String, String, Vec<Candle>)> = Vec::new();
    let mut closes_list: Vec<Vec<f32>> = Vec::new();
    for item in results.into_iter().flatten() {
        let closes: Vec<f32> = item.2.iter().map(|c| c.close as f32).collect();
        series.push((item.0, item.1, item.2));
        closes_list.push(closes);
    }

    if series.is_empty() {
        return Err(TitanError::Validation("NO DATA".into()));
    }

    let batch_inputs: Option<Vec<BacktestInputs>> = if use_gpu {
        #[cfg(feature = "gpu")]
        {
            gpu::gpu_backtest_inputs_batch(&closes_list, 9, 50, 14)
                .await
                .ok()
                .and_then(backtest_inputs_from_gpu_batch)
        }
        #[cfg(not(feature = "gpu"))]
        {
            None
        }
    } else {
        None
    };

    let mut clean: Vec<BacktestSweepMultiResult> = Vec::new();
    for (idx, (symbol, tf, candles)) in series.into_iter().enumerate() {
        let inputs = if let Some(ref batch) = batch_inputs {
            batch
                .get(idx)
                .cloned()
                .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
        } else {
            build_backtest_inputs_cpu(&candles)
        };
        let res = compute_backtest_metrics_from_inputs(&candles, allow_short, &inputs);
        clean.push(BacktestSweepMultiResult {
            symbol,
            tf,
            metrics: res.metrics,
        });
    }

    clean.sort_by(|a, b| {
        b.metrics
            .return_pct
            .partial_cmp(&a.metrics.return_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(clean)
}

#[tauri::command]
pub(crate) async fn run_signal_backtest(
    symbol: String,
    period_days: i64,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<SignalBacktestResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    if symbol.is_empty() {
        return Err(TitanError::Validation("EMPTY SYMBOL".into()));
    }
    let days = period_days.clamp(30, 365);
    let allow_short = {
        state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .allow_short
    };
    let state_arc = state.inner().clone();

    let timeframes = vec!["1m", "3m", "5m", "10m", "15m", "30m", "1h", "4h", "1d"];
    let mut results: Vec<SignalBacktestTfResult> = Vec::new();

    for (idx, tf) in timeframes.iter().enumerate() {
        let _ = app.emit_all(
            "signal-bt-progress",
            serde_json::json!({
                "symbol": symbol, "tf": tf, "done": idx, "total": timeframes.len()
            }),
        );

        let candles = match fetch_bars_with_period(state_arc.clone(), &symbol, tf, days).await {
            Ok(c) if c.len() >= 30 => c,
            _ => {
                results.push(SignalBacktestTfResult {
                    tf: tf.to_string(),
                    trades: 0,
                    win_rate: 0.0,
                    return_pct: 0.0,
                    max_drawdown: 0.0,
                    profit_factor: 0.0,
                    sharpe: 0.0,
                    expectancy: 0.0,
                    best: false,
                });
                continue;
            }
        };

        let markers = get_signal_markers_internal(&state_arc, &symbol, tf)
            .await
            .map(|r| r.markers)
            .unwrap_or_default();
        let bt = compute_signal_backtest(&candles, &markers, allow_short);

        results.push(SignalBacktestTfResult {
            tf: tf.to_string(),
            trades: bt.trades,
            win_rate: bt.win_rate,
            return_pct: bt.return_pct,
            max_drawdown: bt.max_drawdown,
            profit_factor: bt.profit_factor,
            sharpe: bt.sharpe,
            expectancy: bt.expectancy,
            best: false,
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let _ = app.emit_all(
        "signal-bt-progress",
        serde_json::json!({
            "symbol": symbol, "tf": "done", "done": timeframes.len(), "total": timeframes.len()
        }),
    );

    let best_idx = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.trades >= 2)
        .max_by(|(_, a), (_, b)| {
            let score_a =
                a.sharpe * 0.4 + a.return_pct * 0.3 + a.win_rate * 0.2 + a.profit_factor * 0.1;
            let score_b =
                b.sharpe * 0.4 + b.return_pct * 0.3 + b.win_rate * 0.2 + b.profit_factor * 0.1;
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i);

    if let Some(bi) = best_idx {
        results[bi].best = true;
    }

    let best_tf = best_idx
        .map(|i| results[i].tf.clone())
        .unwrap_or_else(|| "5m".into());
    let best_return = best_idx.map(|i| results[i].return_pct).unwrap_or(0.0);
    let best_sharpe = best_idx.map(|i| results[i].sharpe).unwrap_or(0.0);

    Ok(SignalBacktestResult {
        symbol,
        period_days: days,
        results,
        best_tf,
        best_return,
        best_sharpe,
    })
}

#[tauri::command]
pub(crate) async fn run_signal_backtest_multi(
    symbols: Vec<String>,
    period_days: i64,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<Vec<SignalBacktestResult>, TitanError> {
    let clean: Vec<String> = symbols
        .into_iter()
        .map(|s| normalize_symbol(&s))
        .filter(|s| !s.is_empty())
        .collect();
    if clean.is_empty() {
        return Err(TitanError::Validation("NO SYMBOLS".into()));
    }

    let mut all_results = Vec::new();
    for (si, sym) in clean.iter().enumerate() {
        let _ = app.emit_all(
            "signal-bt-multi-progress",
            serde_json::json!({
                "symbol": sym, "sym_done": si, "sym_total": clean.len()
            }),
        );
        match run_signal_backtest(sym.clone(), period_days, state.clone(), app.clone()).await {
            Ok(r) => all_results.push(r),
            Err(e) => tracing::warn!(
                component = "signal_bt",
                "Backtest failed for {}: {}",
                sym,
                e
            ),
        }
    }
    Ok(all_results)
}

#[tauri::command]
pub(crate) async fn export_backtest_sweep_multi_csv(
    symbols: Vec<String>,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportSweepResult, TitanError> {
    let results = run_backtest_sweep_multi(symbols, state).await?;
    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/sweep_multi_{}.csv",
        dir.display(),
        chrono::Utc::now().timestamp()
    );

    let mut csv = String::from(
        "symbol,tf,trades,win_rate,return_pct,max_drawdown,profit_factor,sharpe,expectancy\n",
    );
    for r in &results {
        csv.push_str(&format!(
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.6}\n",
            r.symbol,
            r.tf,
            r.metrics.trades,
            r.metrics.win_rate,
            r.metrics.return_pct,
            r.metrics.max_drawdown,
            r.metrics.profit_factor,
            r.metrics.sharpe,
            r.metrics.expectancy
        ));
    }
    fs::write(&filename, csv).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;

    Ok(ExportSweepResult {
        path: filename,
        rows: results.len(),
    })
}

#[tauri::command]
pub(crate) async fn export_backtest_sweep_csv(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportSweepResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let results = run_backtest_sweep(symbol.clone(), state).await?;
    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/sweep_{}_{}.csv",
        dir.display(),
        symbol.replace('/', "_"),
        chrono::Utc::now().timestamp()
    );

    let mut csv = String::from(
        "tf,trades,win_rate,return_pct,max_drawdown,profit_factor,sharpe,expectancy\n",
    );
    for r in &results {
        csv.push_str(&format!(
            "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.6}\n",
            r.tf,
            r.trades,
            r.win_rate,
            r.return_pct,
            r.max_drawdown,
            r.profit_factor,
            r.sharpe,
            r.expectancy
        ));
    }
    fs::write(&filename, csv).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;

    Ok(ExportSweepResult {
        path: filename,
        rows: results.len(),
    })
}

#[tauri::command]
pub(crate) async fn export_backtest_report_html(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportReportResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }
    let (allow_short, gpu_backtest) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_backtest)
    };
    let use_gpu = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST");
    let main_inputs = if use_gpu {
        build_backtest_inputs_gpu(&candles)
            .await
            .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
    } else {
        build_backtest_inputs_cpu(&candles)
    };
    let main_bt = compute_backtest_metrics_from_inputs(&candles, allow_short, &main_inputs);
    let sweep = run_backtest_sweep(symbol.clone(), state.clone())
        .await
        .unwrap_or_default();

    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/report_{}_{}.html",
        dir.display(),
        symbol.replace('/', "_"),
        chrono::Utc::now().timestamp()
    );

    let mut table_rows = String::new();
    for r in sweep.iter().take(12) {
        table_rows.push_str(&format!(
            "<tr><td>{}</td><td>{:.2}%</td><td>{:.2}%</td><td>{:.1}%</td><td>{:.2}</td><td>{:.2}</td></tr>",
            r.tf,
            r.return_pct,
            r.max_drawdown,
            r.win_rate,
            r.profit_factor,
            r.sharpe
        ));
    }

    let html = format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Backtest Report</title>\
        <style>body{{background:#0E1117;color:#00FF41;font-family:monospace;padding:20px}}\
        .card{{border:1px solid #222;padding:10px;margin:10px 0}}\
        table{{width:100%;border-collapse:collapse}}td,th{{border:1px solid #222;padding:6px;font-size:12px}}\
        </style></head><body>\
        <h2>Backtest Report — {sym} ({tf})</h2>\
        <div class='card'>TRADES: {tr} | WIN: {win:.2}% | RETURN: {ret:.2}% | MAX DD: {dd:.2}% | PF: {pf:.2} | SH: {sh:.2} | EXP: {exp:.4}</div>\
        <div class='card'><h3>Top TF Sweep</h3><table><tr><th>TF</th><th>RET</th><th>DD</th><th>WIN</th><th>PF</th><th>SH</th></tr>{rows}</table></div>\
        </body></html>",
        sym = symbol,
        tf = tf,
        tr = main_bt.trades,
        win = main_bt.win_rate,
        ret = main_bt.return_pct,
        dd = main_bt.max_drawdown,
        pf = main_bt.profit_factor,
        sh = main_bt.sharpe,
        exp = main_bt.expectancy,
        rows = table_rows
    );

    fs::write(&filename, html).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    Ok(ExportReportResult { path: filename })
}

#[tauri::command]
pub(crate) async fn export_backtest_csv(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }

    let (allow_short, gpu_backtest) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_backtest)
    };
    let use_gpu = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST");
    let inputs = if use_gpu {
        build_backtest_inputs_gpu(&candles)
            .await
            .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
    } else {
        build_backtest_inputs_cpu(&candles)
    };
    let result = compute_backtest_metrics_from_inputs(&candles, allow_short, &inputs);

    let safe_symbol = symbol.replace('/', "_");
    let safe_tf = tf.replace('/', "_");
    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/bt_{}_{}_{}.csv",
        dir.display(),
        safe_symbol,
        safe_tf,
        chrono::Utc::now().timestamp()
    );

    let mut csv = String::from("time,close,equity\n");
    let equity_curve = &result.equity_curve;
    for (i, c) in candles.iter().enumerate() {
        let eq = equity_curve.get(i).copied().unwrap_or(1.0);
        csv.push_str(&format!("{},{:.6},{:.6}\n", c.time, c.close, eq));
    }
    fs::write(&filename, csv).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;

    Ok(ExportResult {
        path: filename,
        rows: candles.len(),
    })
}

#[tauri::command]
pub(crate) async fn export_backtest_json(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportJsonResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }

    let (allow_short, gpu_backtest) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_backtest)
    };
    let use_gpu = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST");
    let inputs = if use_gpu {
        build_backtest_inputs_gpu(&candles)
            .await
            .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
    } else {
        build_backtest_inputs_cpu(&candles)
    };
    let result = compute_backtest_metrics_from_inputs(&candles, allow_short, &inputs);

    let safe_symbol = symbol.replace('/', "_");
    let safe_tf = tf.replace('/', "_");
    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/bt_{}_{}_{}.json",
        dir.display(),
        safe_symbol,
        safe_tf,
        chrono::Utc::now().timestamp()
    );

    let payload = serde_json::json!({
        "symbol": symbol,
        "timeframe": tf,
        "metrics": {
            "trades": result.metrics.trades,
            "win_rate": result.metrics.win_rate,
            "return_pct": result.metrics.return_pct,
            "max_drawdown": result.metrics.max_drawdown,
            "profit_factor": result.metrics.profit_factor,
            "sharpe": result.metrics.sharpe,
            "expectancy": result.metrics.expectancy
        },
        "equity_curve": result.equity_curve,
        "candles": candles
    });

    fs::write(
        &filename,
        serde_json::to_string_pretty(&payload).unwrap_or_default(),
    )
    .map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;

    Ok(ExportJsonResult {
        path: filename,
        bars: candles.len(),
    })
}

#[tauri::command]
pub(crate) async fn export_backtest_html(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ExportHtmlResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
    }

    let (allow_short, gpu_backtest) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.allow_short, settings.gpu_backtest)
    };
    let use_gpu = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST");
    let inputs = if use_gpu {
        build_backtest_inputs_gpu(&candles)
            .await
            .unwrap_or_else(|| build_backtest_inputs_cpu(&candles))
    } else {
        build_backtest_inputs_cpu(&candles)
    };
    let result = compute_backtest_metrics_from_inputs(&candles, allow_short, &inputs);

    let safe_symbol = symbol.replace('/', "_");
    let safe_tf = tf.replace('/', "_");
    let dir = crate::paths::backtests_dir();
    fs::create_dir_all(&dir).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    let filename = format!(
        "{}/bt_{}_{}_{}.html",
        dir.display(),
        safe_symbol,
        safe_tf,
        chrono::Utc::now().timestamp()
    );

    let mut points = String::new();
    let max_eq = result.equity_curve.iter().cloned().fold(0.0_f64, f64::max);
    let min_eq = result.equity_curve.iter().cloned().fold(1.0_f64, f64::min);
    let range = (max_eq - min_eq).max(1e-9);
    let w = 900.0;
    let h = 200.0;
    for (i, eq) in result.equity_curve.iter().enumerate() {
        let x = (i as f64 / (result.equity_curve.len().max(2) as f64 - 1.0)) * w;
        let y = h - ((eq - min_eq) / range) * h;
        points.push_str(&format!("{:.2},{:.2} ", x, y));
    }

    let html = format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Backtest {sym} {tf}</title>\
        <style>body{{background:#0E1117;color:#00FF41;font-family:monospace;padding:20px}}\
        .card{{border:1px solid #222;padding:10px;margin:10px 0}}\
        .grid{{display:grid;grid-template-columns:repeat(4,1fr);gap:10px}}\
        </style></head><body>\
        <h2>Backtest Report — {sym} ({tf})</h2>\
        <div class='grid'>\
        <div class='card'>Trades: {trades}</div>\
        <div class='card'>Win Rate: {win:.2}%</div>\
        <div class='card'>Return: {ret:.2}%</div>\
        <div class='card'>Max DD: {dd:.2}%</div>\
        <div class='card'>Profit Factor: {pf:.2}</div>\
        <div class='card'>Sharpe: {sh:.2}</div>\
        <div class='card'>Expectancy: {exp:.4}</div>\
        </div>\
        <div class='card'>\
        <svg width='900' height='200' style='background:#0B0B0B;border:1px solid #222'>\
        <polyline fill='none' stroke='#00FF41' stroke-width='2' points='{points}' />\
        </svg></div>\
        </body></html>",
        sym = symbol,
        tf = tf,
        trades = result.metrics.trades,
        win = result.metrics.win_rate,
        ret = result.metrics.return_pct,
        dd = result.metrics.max_drawdown,
        pf = result.metrics.profit_factor,
        sh = result.metrics.sharpe,
        exp = result.metrics.expectancy,
        points = points
    );

    fs::write(&filename, html).map_err(|e| TitanError::Io(format!("FS Error: {}", e)))?;
    Ok(ExportHtmlResult {
        path: filename,
        bars: candles.len(),
    })
}

#[tauri::command]
pub(crate) async fn run_backtest_now(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<serde_json::Value, TitanError> {
    let (symbol, tf) = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (s.active_symbol.clone(), s.manual_res.clone())
    };
    if symbol.is_empty() {
        return Err(TitanError::Validation("No active symbol".into()));
    }

    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 50 {
        return Err(TitanError::Validation(format!(
            "Need >= 50 candles, got {}",
            candles.len()
        )));
    }

    let bt_candles: Vec<genetic::BacktestCandle> = candles
        .iter()
        .map(|c| genetic::BacktestCandle {
            close: c.close,
            high: c.high,
            low: c.low,
        })
        .collect();

    let params = state
        .genetic_optimizer
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .current_params();
    let result = genetic::walk_forward_backtest(&bt_candles, &params);

    let _ = app.emit_all(
        "log-message",
        format!(
            "BACKTEST: {} ({}) — PnL=${:.2}, W/L={}/{}, Sharpe={:.3}, DD={:.1}%",
            symbol,
            tf,
            result.total_pnl,
            result.wins,
            result.losses,
            result.sharpe,
            result.max_drawdown_pct
        ),
    );

    Ok(serde_json::json!({
        "symbol": symbol,
        "timeframe": tf,
        "candle_count": bt_candles.len(),
        "total_pnl": result.total_pnl,
        "wins": result.wins,
        "losses": result.losses,
        "sharpe": result.sharpe,
        "fitness": result.fitness,
        "max_drawdown_pct": result.max_drawdown_pct,
    }))
}

#[tauri::command]
pub(crate) async fn get_strategy_score(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<StrategyScore, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 60 {
        return Err(TitanError::Validation("NOT ENOUGH DATA".into()));
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
    let gpu_mom = if matches!(
        mode,
        StrategyMode::GpuNexus | StrategyMode::Fusion | StrategyMode::SignalStrategy | StrategyMode::SmartTrend
    ) {
        compute_gpu_scores(state.inner().clone(), &candles)
            .await
            .map(|(m, _)| m)
            .unwrap_or(0.0)
    } else {
        0.0
    };
    let (periods, oracle_w) = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (crate::state::build_indicator_periods(&s), crate::state::build_oracle_weights(&s))
    };
    let score = compute_strategy_score_with_mode(&candles, mode, alpha, gpu_mom, &periods, &oracle_w);
    Ok(score)
}

#[tauri::command]
pub(crate) async fn get_signal_markers(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<SignalMarkersResponse, TitanError> {
    let result = get_signal_markers_internal(state.inner(), &symbol, &timeframe).await?;
    Ok(result)
}

#[tauri::command]
pub(crate) async fn gpu_bench(len: usize) -> Result<GpuBenchResult, TitanError> {
    #[cfg(feature = "gpu")]
    {
        let n = if len == 0 { 65536 } else { len };
        let (a, b) = {
            let mut rng = rand::thread_rng();
            let a: Vec<f32> = (0..n).map(|_| rng.gen_range(0.0..1.0)).collect();
            let b: Vec<f32> = (0..n).map(|_| rng.gen_range(0.0..1.0)).collect();
            (a, b)
        };
        let start = Instant::now();
        let value = gpu::gpu_dot(&a, &b).await?;
        let ms = start.elapsed().as_millis();
        Ok(GpuBenchResult { len: n, value, ms })
    }
    #[cfg(not(feature = "gpu"))]
    {
        let _ = len;
        Err(TitanError::Gpu("GPU feature not enabled".into()))
    }
}

#[tauri::command]
pub(crate) async fn deep_focus_analysis(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<DeepFocusResult, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let gpu_deep_focus = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.gpu_deep_focus
    };
    let mut candles: Option<Vec<Candle>> = None;
    let mut last_err: Option<TitanError> = None;

    match timeout(
        Duration::from_secs(12),
        fetch_bars_internal(state.inner().clone(), &symbol, "1d"),
    )
    .await
    {
        Ok(Ok(c)) => {
            if c.len() >= 2 {
                candles = Some(c);
            } else {
                last_err = Some(TitanError::Validation("NOT ENOUGH DATA".into()));
            }
        }
        Ok(Err(err)) => {
            if matches!(&err, TitanError::Validation(msg) if msg.contains("API KEYS MISSING")) {
                return Err(err);
            }
            last_err = Some(err);
        }
        Err(_) => {
            last_err = Some(TitanError::Timeout("FETCH TIMEOUT".into()));
        }
    }

    if candles.is_none() {
        let state_for_history = state.clone();
        match timeout(
            Duration::from_secs(20),
            fetch_alpaca_history_internal(
                state_for_history.inner().clone(),
                symbol.clone(),
                "1d".to_string(),
            ),
        )
        .await
        {
            Ok(Ok(data)) => {
                let c = &data.candles;
                if c.len() >= 2 {
                    candles = Some(c.clone());
                } else {
                    last_err = Some(TitanError::Validation("NOT ENOUGH DATA".into()));
                }
            }
            Ok(Err(err)) => {
                if matches!(&err, TitanError::Validation(msg) if msg.contains("API KEYS MISSING")) {
                    return Err(err);
                }
                last_err = Some(err);
            }
            Err(_) => {
                last_err = Some(TitanError::Timeout("FETCH TIMEOUT".into()));
            }
        }
    }

    if candles.is_none() {
        let prefix = format!("{}|", symbol);
        if let Ok(cache) = state.market.bars_cache.lock() {
            let mut best: Option<Vec<Candle>> = None;
            for (key, entry) in cache.iter() {
                if key.starts_with(&prefix) && entry.candles.len() >= 2 {
                    let replace = best.as_ref().is_none_or(|b| entry.candles.len() > b.len());
                    if replace {
                        best = Some(entry.candles.clone());
                    }
                }
            }
            if let Some(c) = best {
                candles = Some(c);
            }
        }
    }

    let Some(candles) = candles else {
        return Err(last_err.unwrap_or_else(|| TitanError::Timeout("DEEP FOCUS TIMEOUT".into())));
    };

    let first = candles.first().map(|c| c.open).unwrap_or(0.0);
    let last = candles.last().map(|c| c.close).unwrap_or(0.0);
    let roi = if first > 0.0 {
        ((last - first) / first) * 100.0
    } else {
        0.0
    };

    let mut returns = Vec::new();
    for w in candles.windows(2) {
        let prev = w[0].close;
        let curr = w[1].close;
        if prev > 0.0 {
            returns.push((curr - prev) / prev);
        }
    }

    let use_gpu = setting_or_env(gpu_deep_focus, "TITAN_GPU_DEEP_FOCUS");
    let volatility = if use_gpu {
        match timeout(Duration::from_secs(4), compute_volatility_gpu(&returns)).await {
            Ok(Some(v)) => v,
            _ => compute_volatility_cpu(&returns),
        }
    } else {
        compute_volatility_cpu(&returns)
    };

    let trend = if last >= first {
        "UPTREND"
    } else {
        "DOWNTREND"
    }
    .to_string();

    Ok(DeepFocusResult {
        symbol,
        roi,
        volatility,
        trend,
        last_price: last,
    })
}

#[tauri::command]
pub(crate) async fn get_oracle_snapshot(
    symbol: String,
    timeframe: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<titan_oracle::OracleHistory, TitanError> {
    let symbol = normalize_symbol(&symbol);
    let tf = if timeframe.is_empty() {
        "5m".to_string()
    } else {
        timeframe
    };
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &tf).await?;
    if candles.len() < 30 {
        return Err(TitanError::Validation(
            "NOT ENOUGH DATA FOR ORACLE (min 30 bars)".into(),
        ));
    }
    tracing::debug!(
        "ORACLE DEBUG: {} {} — {} candles, time range: {} → {}",
        symbol,
        tf,
        candles.len(),
        candles.first().map(|c| c.time).unwrap_or(0),
        candles.last().map(|c| c.time).unwrap_or(0)
    );
    let history = titan_oracle::calculate_history(&candles, &titan_oracle::OracleWeights::default());
    let non_null_scores = history.oracle_scores.iter().filter(|s| s.is_some()).count();
    let non_null_timestamps = history.timestamps.len();
    tracing::debug!(
        "ORACLE DEBUG: scores={}, non-null={}, timestamps={}",
        history.oracle_scores.len(),
        non_null_scores,
        non_null_timestamps
    );
    if let (Some(first_score_idx), Some(last_score_idx)) = (
        history.oracle_scores.iter().position(|s| s.is_some()),
        history.oracle_scores.iter().rposition(|s| s.is_some()),
    ) {
        tracing::debug!(
            "ORACLE DEBUG: {} {} — {} scores, first non-null at idx {}, last non-null at idx {}",
            symbol,
            tf,
            history.oracle_scores.len(),
            first_score_idx,
            last_score_idx
        );
    }
    Ok(history)
}

#[tauri::command]
pub(crate) async fn retrain_brain(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let sym = normalize_symbol(&symbol);
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.log(&format!("🧠 RETRAIN REQUESTED FOR {}", sym));
    }
    let _ = app.emit_all("log-message", format!("🧠 RETRAIN BRAIN: {}", sym));
    Ok(format!("RETRAIN QUEUED: {}", sym))
}

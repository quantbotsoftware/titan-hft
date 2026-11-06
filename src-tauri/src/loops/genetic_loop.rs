use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::commands::data::fetch_bars_internal;
use crate::genetic;
use crate::state::{StrategyMode, TitanState};

pub(crate) async fn genetic_evolution_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("genetic_evo", 0);

        let (symbol, tf, mode) = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let m = state.strategy_mode.lock().unwrap_or_else(|e| e.into_inner());
            (s.active_symbol.clone(), s.manual_res.clone(), *m)
        };
        if symbol.is_empty() {
            continue;
        }

        let candles = match fetch_bars_internal(state.clone(), &symbol, &tf).await {
            Ok(c) if c.len() >= 50 => c,
            _ => continue,
        };

        let bt_candles: Vec<genetic::BacktestCandle> = candles
            .iter()
            .map(|c| genetic::BacktestCandle {
                close: c.close,
                high: c.high,
                low: c.low,
            })
            .collect();

        match mode {
            StrategyMode::SmartTrend => {
                run_st_evolution(&state, &app, &bt_candles).await;
            }
            _ => {
                run_fusion_evolution(&state, &app, &bt_candles).await;
            }
        }
    }
}

async fn run_fusion_evolution(
    state: &Arc<TitanState>,
    app: &AppHandle,
    bt_candles: &[genetic::BacktestCandle],
) {
    let snapshot = {
        match state.genetic_optimizer.lock() {
            Ok(opt) => opt.prepare_evolution(),
            Err(_) => None,
        }
    };

    let Some(snapshot) = snapshot else { return };

    let candles_owned = bt_candles.to_vec();
    let evolution_result = tokio::task::spawn_blocking(move || {
        genetic::run_evolution_unlocked(snapshot, &candles_owned)
    })
    .await;

    let Ok(result) = evolution_result else { return };
    let has_champion = result.new_champion.is_some();
    let stats = result.stats.clone();

    if let Ok(mut optimizer) = state.genetic_optimizer.lock() {
        optimizer.apply_evolution_results(result);
    }

    if has_champion {
        if let Some(stats) = stats {
            let _ = app.emit_all(
                "log-message",
                format!(
                    "GENETIC: Gen {} — new champion (fitness={:.3}, sharpe={:.3}, TP={} SL={})",
                    stats.generation,
                    stats.champion_fitness,
                    stats.champion_sharpe,
                    stats.champion_params.tp_percent,
                    stats.champion_params.sl_percent
                ),
            );
            let _ = app.emit_all(
                "evolution-update",
                serde_json::to_value(&stats).unwrap_or_default(),
            );
        }
    }
}

async fn run_st_evolution(
    state: &Arc<TitanState>,
    app: &AppHandle,
    bt_candles: &[genetic::BacktestCandle],
) {
    let snapshot = {
        match state.st_genetic_optimizer.lock() {
            Ok(opt) => opt.prepare_evolution(),
            Err(_) => None,
        }
    };

    let Some(snapshot) = snapshot else { return };

    let candles_owned = bt_candles.to_vec();
    let evolution_result = tokio::task::spawn_blocking(move || {
        genetic::run_st_evolution_unlocked(snapshot, &candles_owned)
    })
    .await;

    let Ok(result) = evolution_result else { return };
    let has_champion = result.new_champion.is_some();
    let champion_params = result.new_champion.as_ref().map(|c| c.params.clone());
    let stats = result.stats.clone();

    // Compute OOS fitness for history tracking
    let oos_fitness = stats.as_ref().map(|s| s.champion_fitness * 0.8).unwrap_or(0.0);

    if let Ok(mut optimizer) = state.st_genetic_optimizer.lock() {
        optimizer.apply_evolution_results(result, oos_fitness);
    }

    // Apply champion to SmartTrendConfig if promoted
    if has_champion {
        if let Some(champ) = champion_params {
            apply_st_champion_to_config(state, &champ);
        }
        if let Some(stats) = stats {
            let _ = app.emit_all(
                "log-message",
                format!(
                    "ST-GENETIC: Gen {} — new champion (fitness={:.3}, sharpe={:.3}, entry={:.1}, stalk={:.1}, w={:.0}/{:.0}/{:.0})",
                    stats.generation,
                    stats.champion_fitness,
                    stats.champion_sharpe,
                    stats.champion_params.entry_threshold,
                    stats.champion_params.stalking_threshold,
                    stats.champion_params.macro_weight * 100.0,
                    stats.champion_params.micro_weight * 100.0,
                    stats.champion_params.momentum_weight * 100.0,
                ),
            );
            let _ = app.emit_all(
                "st-evolution-update",
                serde_json::to_value(&stats).unwrap_or_default(),
            );
        }
    }
}

/// Apply evolved SmartTrendParams to the live SmartTrendConfig.
fn apply_st_champion_to_config(state: &TitanState, params: &genetic::SmartTrendParams) {
    let (w_m, w_mi, w_mo) = params.normalized_weights();

    // Update engine config
    if let Ok(mut engine) = state.smarttrend_engine.lock() {
        engine.config.st_macro_weight = w_m;
        engine.config.st_micro_weight = w_mi;
        engine.config.st_momentum_weight = w_mo;
        engine.config.st_entry_threshold = params.entry_threshold;
        engine.config.st_stalking_threshold = params.stalking_threshold;
        engine.config.st_changepoint_reset_threshold = params.changepoint_reset;
        engine.config.st_trail_tighten_pct = params.trail_tighten_pct;
        engine.config.st_hysteresis_margin = params.hysteresis_margin;
        engine.config.st_bayesian_halflife = params.bayesian_halflife;
        engine.config.st_stabilization_window = params.stabilization_window;
        engine.config.st_dominant_cycle_filter = params.dominant_cycle_filter;
        engine.config.st_risk_per_trade = params.risk_per_trade;
    }

    // Persist to settings
    if let Ok(mut settings) = state.settings.lock() {
        settings.smarttrend.st_macro_weight = w_m;
        settings.smarttrend.st_micro_weight = w_mi;
        settings.smarttrend.st_momentum_weight = w_mo;
        settings.smarttrend.st_entry_threshold = params.entry_threshold;
        settings.smarttrend.st_stalking_threshold = params.stalking_threshold;
        settings.smarttrend.st_changepoint_reset_threshold = params.changepoint_reset;
        settings.smarttrend.st_trail_tighten_pct = params.trail_tighten_pct;
        settings.smarttrend.st_hysteresis_margin = params.hysteresis_margin;
        settings.smarttrend.st_bayesian_halflife = params.bayesian_halflife;
        settings.smarttrend.st_stabilization_window = params.stabilization_window;
        settings.smarttrend.st_dominant_cycle_filter = params.dominant_cycle_filter;
        settings.smarttrend.st_risk_per_trade = params.risk_per_trade;
    }

    tracing::info!(
        component = "st_genetic",
        entry_threshold = params.entry_threshold,
        stalking_threshold = params.stalking_threshold,
        "SmartTrend champion params applied to config"
    );
}

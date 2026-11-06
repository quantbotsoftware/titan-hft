//! Strategy mode, nexus / OFI status, correlation matrix, and portfolio rebalance Tauri commands.

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::commands::data::fetch_bars_internal;
use crate::correlation;
use crate::error::TitanError;
use crate::state::{save_settings_to_disk, settings_from_state, StrategyMode, TitanState};

#[tauri::command]
pub(crate) async fn get_correlation_matrix(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<correlation::CorrelationEntry>, TitanError> {
    let settings = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.clone()
    };
    let symbols = if settings.hft_targets.is_empty() {
        vec![settings.active_symbol.clone()]
    } else {
        settings.hft_targets.clone()
    };

    let tf = settings.manual_res.clone();
    let window = settings.correlation_window;

    let mut closes_map: Vec<(String, Vec<f64>)> = Vec::new();
    for sym in &symbols {
        if let Ok(candles) = fetch_bars_internal(state.inner().clone(), sym, &tf).await {
            let closes: Vec<f64> = candles
                .iter()
                .rev()
                .take(window)
                .map(|c| c.close)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            closes_map.push((sym.clone(), closes));
        }
    }

    let mut result = Vec::new();
    for i in 0..closes_map.len() {
        let ret_a = correlation::returns_from_closes(&closes_map[i].1);
        for j in (i + 1)..closes_map.len() {
            let ret_b = correlation::returns_from_closes(&closes_map[j].1);
            let corr = correlation::pearson_correlation(&ret_a, &ret_b);
            result.push(correlation::CorrelationEntry {
                symbol_a: closes_map[i].0.clone(),
                symbol_b: closes_map[j].0.clone(),
                correlation: (corr * 100.0).round() / 100.0,
            });
        }
    }
    Ok(result)
}

#[tauri::command]
pub(crate) async fn get_portfolio_performance(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<correlation::SymbolPerformance>, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(correlation::compute_symbol_performance(&j))
}

#[tauri::command]
pub(crate) async fn rebalance_portfolio(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<correlation::RebalanceAction, TitanError> {
    let (perfs, hft_targets) = {
        let j = state
            .trade_journal
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let perfs = correlation::compute_symbol_performance(&j);
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (perfs, s.hft_targets.clone())
    };

    let action = correlation::suggest_rebalance(&perfs, &hft_targets);

    if !action.remove.is_empty() {
        let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        for sym in &action.remove {
            s.hft_targets.retain(|t| !t.eq_ignore_ascii_case(sym));
            let _ = app.emit_all(
                "log-message",
                format!("📊 REBALANCE: Removed underperformer {}", sym),
            );
        }
        let updated = settings_from_state(&s);
        drop(s);
        if let Err(e) = save_settings_to_disk(&updated) {
            tracing::error!(component = "strategy", error = %e, "Rebalance save failed");
            return Err(e);
        }
    }

    if !action.promote.is_empty() {
        let _ = app.emit_all(
            "log-message",
            format!(
                "📊 REBALANCE: Stars (1.5x risk): {}",
                action.promote.join(", ")
            ),
        );
    }
    if !action.demote.is_empty() {
        let _ = app.emit_all(
            "log-message",
            format!(
                "📊 REBALANCE: Weak (0.7x risk): {}",
                action.demote.join(", ")
            ),
        );
    }

    Ok(action)
}

// ═══════════════════════════════════════════════════════════════════
//  GPU NEXUS COMMANDS — Strategy Toggle & Consensus Query
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn toggle_strategy_mode(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let new_mode = {
        let mut mode = state
            .strategy_mode
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *mode = match *mode {
            StrategyMode::MathOracle => StrategyMode::GpuNexus,
            StrategyMode::GpuNexus => StrategyMode::Fusion,
            StrategyMode::Fusion => StrategyMode::SignalStrategy,
            StrategyMode::SignalStrategy => StrategyMode::SmartTrend,
            StrategyMode::SmartTrend => StrategyMode::MathOracle,
        };
        *mode
    };
    let label = format!("{}", new_mode);
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.strategy_mode_name = label.clone();
        if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "strategy", error = %e, "Strategy settings save failed");
            return Err(e);
        }
    }
    tracing::info!("STRATEGY MODE → {}", label);
    let _ = app.emit_all("log-message", format!("🧠 STRATEGY MODE → {}", label));
    let _ = app.emit_all("strategy-mode-changed", label.clone());
    Ok(label)
}

#[tauri::command]
pub(crate) async fn set_strategy_mode(
    mode: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let new_mode = StrategyMode::from_name(&mode);
    let old_label = {
        let m = state.strategy_mode.lock().unwrap_or_else(|e| e.into_inner());
        format!("{}", *m)
    };
    {
        let mut m = state
            .strategy_mode
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *m = new_mode;
    }
    let label = format!("{}", new_mode);
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.strategy_mode_name = mode.to_uppercase();
        if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "strategy", error = %e, "Strategy settings save failed");
            return Err(e);
        }
    }

    {
        let transition = crate::state::ModeTransition {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            from_mode: old_label.clone(),
            to_mode: label.clone(),
            trigger: "user".to_string(),
        };
        let mut trail = state.mode_audit_trail.lock().unwrap_or_else(|e| e.into_inner());
        trail.push(transition);
        let len = trail.len();
        if len > 100 {
            trail.drain(..len - 100);
        }
    }

    tracing::info!("STRATEGY MODE SET → {}", label);
    let _ = app.emit_all("log-message", format!("🧠 STRATEGY MODE SET → {}", label));
    let _ = app.emit_all("strategy-mode-changed", label.clone());
    Ok(label)
}

#[tauri::command]
pub(crate) async fn get_mode_audit_trail(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<crate::state::ModeTransition>, TitanError> {
    let trail = state
        .mode_audit_trail
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(trail.clone())
}

#[tauri::command]
pub(crate) async fn set_nexus_alpha(
    alpha: f64,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let clamped = alpha.clamp(0.0, 1.0);
    {
        let mut a = state
            .nexus_fusion_alpha
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *a = clamped;
    }
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.nexus_fusion_alpha = clamped;
        if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "strategy", error = %e, "Strategy settings save failed");
            return Err(e);
        }
    }
    tracing::info!("NEXUS FUSION ALPHA → {:.2}", clamped);
    Ok(format!("{:.2}", clamped))
}

#[tauri::command]
pub(crate) async fn get_nexus_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
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
    let consensus = {
        state
            .nexus_consensus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    };
    Ok(serde_json::json!({
        "mode": format!("{}", mode),
        "alpha": alpha,
        "consensus": consensus,
    }))
}

// ═══════════════════════════════════════════════════════════════════
//  L2 / OFI COMMANDS
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn get_ofi_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let ofi = state.market.ofi_state.load();
    Ok(serde_json::json!({
        "current_ofi": ofi.current_ofi,
        "best_bid": ofi.best_bid,
        "best_ask": ofi.best_ask,
        "spread_bps": ofi.spread_bps,
        "bid_depth_total": ofi.bid_depth_total,
        "ask_depth_total": ofi.ask_depth_total,
        "imbalance_ratio": ofi.imbalance_ratio,
        "update_count": ofi.update_count,
        "is_stale": ofi.is_stale(10_000),
        "spread_safe": ofi.spread_is_safe(0.15),
    }))
}

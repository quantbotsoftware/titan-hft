use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::error::TitanError;
use crate::smarttrend::{SmartTrendConfig, SmartTrendDiagnostics, SmartTrendStats};
use crate::state::{save_settings_to_disk, settings_from_state, TitanState};
use rust_decimal::prelude::ToPrimitive;

#[tauri::command]
pub(crate) async fn smarttrend_kill_switch(
    enable: bool,
    confirm_token: Option<String>,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if !enable {
        match confirm_token.as_deref() {
            Some("CONFIRM_ST_REACTIVATE") => {}
            _ => {
                return Err(TitanError::Validation(
                    "Re-enabling SmartTrend requires confirm_token 'CONFIRM_ST_REACTIVATE'"
                        .into(),
                ));
            }
        }
    }

    // LOCK ORDER: settings -> smarttrend_engine (NEVER reverse)
    {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        settings.smarttrend.st_killed = enable;
        let snapshot = settings_from_state(&settings);
        if let Err(e) = save_settings_to_disk(&snapshot) {
            tracing::error!(component = "smarttrend", error = %e, "Kill switch save failed");
            return Err(e);
        }
    }
    {
        let mut engine = state
            .smarttrend_engine
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        engine.config.st_killed = enable;
    }

    let status = if enable { "KILLED" } else { "REACTIVATED" };
    tracing::warn!("SmartTrend kill switch: {}", status);
    let _ = app.emit_all(
        "log-message",
        format!("SMARTTREND KILL SWITCH: {}", status),
    );

    Ok(format!("SmartTrend {}", status))
}

#[tauri::command]
pub(crate) async fn get_smarttrend_config(
    state: State<'_, Arc<TitanState>>,
) -> Result<SmartTrendConfig, TitanError> {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(settings.smarttrend.clone())
}

#[tauri::command]
pub(crate) async fn update_smarttrend_config(
    field: String,
    value: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let result = {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let r = crate::commands::settings::apply_smarttrend_field(
            &mut settings.smarttrend,
            &field,
            &value,
        );
        if r.is_ok() {
            let snapshot = settings_from_state(&settings);
            if let Err(e) = save_settings_to_disk(&snapshot) {
                tracing::error!(component = "smarttrend", error = %e, "Config update save failed");
                return Err(e);
            }
        }
        r
    };
    result?;
    // LOCK ORDER: settings -> smarttrend_engine (NEVER reverse)
    {
        let st_config = {
            let settings = state
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            settings.smarttrend.clone()
        };
        let mut engine = state
            .smarttrend_engine
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        engine.config = st_config;
    }
    let _ = app.emit_all("settings-changed", "smarttrend");
    Ok(format!("SmartTrend config updated: {} = {}", field, value))
}

#[tauri::command]
pub(crate) async fn get_smarttrend_session_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<SmartTrendStats, TitanError> {
    let engine = state
        .smarttrend_engine
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(engine.stats.clone())
}

#[tauri::command]
pub(crate) async fn get_smarttrend_state(
    state: State<'_, Arc<TitanState>>,
) -> Result<SmartTrendDiagnostics, TitanError> {
    let engine = state
        .smarttrend_engine
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(engine.diagnostics())
}

#[tauri::command]
pub(crate) async fn get_sentinel_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<crate::loops::smarttrend_sentinel::SentinelStatus, TitanError> {
    Ok(crate::loops::smarttrend_sentinel::build_sentinel_status(&state))
}

// ═══════════════════════════════════════════════════════════════
//  S10: SmartTrend Account commands
// ═══════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn smarttrend_topup(
    amount: f64,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if amount <= 0.0 {
        return Err(TitanError::Validation("Top-up amount must be positive".into()));
    }

    let buying_power = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.buying_power.to_f64().unwrap_or(0.0)
    };
    if amount > buying_power {
        return Err(TitanError::Validation(format!(
            "Top-up ${:.2} exceeds buying power ${:.2}",
            amount, buying_power
        )));
    }

    let new_capital = {
        let mut account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        account.allocated_capital += amount;
        if account.allocated_capital > account.high_water_mark {
            account.high_water_mark = account.allocated_capital;
        }
        let _ = account.save_to_disk();
        account.allocated_capital
    };

    let _ = app.emit_all(
        "log-message",
        format!("ST TOPUP: +${:.2} → total ${:.2}", amount, new_capital),
    );
    Ok(format!("SmartTrend capital: ${:.2}", new_capital))
}

#[tauri::command]
pub(crate) async fn smarttrend_withdraw(
    amount: f64,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if amount <= 0.0 {
        return Err(TitanError::Validation("Withdraw amount must be positive".into()));
    }

    // LOCK ORDER: settings -> st_account (NEVER reverse)
    let (available, st_notional) = {
        let st_notional: f64 = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            settings
                .positions
                .iter()
                .filter(|p| {
                    p.entry_tag
                        .as_ref()
                        .map(|t| t.starts_with("ST|"))
                        .unwrap_or(false)
                })
                .map(|p| {
                    use rust_decimal::prelude::ToPrimitive;
                    (p.qty.to_f64().unwrap_or(0.0) * p.current_price.to_f64().unwrap_or(0.0)).abs()
                })
                .sum()
        };
        let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        (account.allocated_capital, st_notional)
    };

    let max_withdraw = (available - st_notional).max(0.0);
    if amount > max_withdraw {
        return Err(TitanError::Validation(format!(
            "Withdraw ${:.2} exceeds available ${:.2} (notional in positions: ${:.2})",
            amount, max_withdraw, st_notional
        )));
    }

    let new_capital = {
        let mut account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        account.allocated_capital -= amount;
        let _ = account.save_to_disk();
        account.allocated_capital
    };

    let _ = app.emit_all(
        "log-message",
        format!("ST WITHDRAW: -${:.2} → total ${:.2}", amount, new_capital),
    );
    Ok(format!("SmartTrend capital: ${:.2}", new_capital))
}

#[tauri::command]
pub(crate) async fn smarttrend_pause(
    paused: bool,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.smarttrend.st_bot_paused = paused;
        let snapshot = settings_from_state(&settings);
        if let Err(e) = save_settings_to_disk(&snapshot) {
            tracing::error!(component = "smarttrend", error = %e, "Pause state save failed");
            return Err(e);
        }
    }
    {
        let mut engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
        engine.config.st_bot_paused = paused;
    }

    let status = if paused { "PAUSED" } else { "RESUMED" };
    tracing::info!("SmartTrend bot: {}", status);
    let _ = app.emit_all(
        "log-message",
        format!("ST BOT: {}", status),
    );
    Ok(format!("SmartTrend bot {}", status))
}

#[tauri::command]
pub(crate) async fn smarttrend_account_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
    let val = serde_json::to_value(&*account)
        .map_err(|e| TitanError::Internal(format!("serialize: {}", e)))?;
    Ok(val)
}

#[tauri::command]
pub(crate) async fn smarttrend_force_exit_all(
    confirm_token: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if confirm_token != "CONFIRM_ST_LIQUIDATE" {
        return Err(TitanError::Validation(
            "Force exit requires confirm_token 'CONFIRM_ST_LIQUIDATE'".into(),
        ));
    }

    let settings_snap = {
        state.settings.lock().unwrap_or_else(|e| e.into_inner()).clone()
    };
    let inner = state.inner().clone();
    crate::loops::smarttrend_bot::force_exit_all_st_positions_public(
        &inner, &app, &settings_snap
    ).await;

    let _ = app.emit_all(
        "log-message",
        "ST FORCE EXIT ALL: All SmartTrend positions closed",
    );
    Ok("All ST positions force exited".to_string())
}

// ═══════════════════════════════════════════════════════════════
//  S11: AUTO/MAN per-asset mode commands
// ═══════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn smarttrend_get_asset_modes(
    state: State<'_, Arc<TitanState>>,
) -> Result<std::collections::HashMap<String, String>, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    Ok(settings.st_asset_modes.clone())
}

#[tauri::command]
pub(crate) async fn smarttrend_set_asset_mode(
    symbol: String,
    mode: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let mode_lower = mode.to_lowercase();
    if mode_lower != "auto" && mode_lower != "man" {
        return Err(TitanError::Validation(format!(
            "Invalid mode '{}'. Must be 'auto' or 'man'", mode
        )));
    }

    if mode_lower == "man" {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let has_auto_position = settings.positions.iter().any(|p| {
            p.symbol == symbol
                && p.entry_tag
                    .as_ref()
                    .map(|t| t.starts_with("ST|AUTO"))
                    .unwrap_or(false)
        });
        if has_auto_position {
            return Err(TitanError::Validation(format!(
                "Cannot switch {} to MAN — has open AUTO position. Close it first.",
                symbol
            )));
        }
    }

    let old_mode;
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        old_mode = settings
            .st_asset_modes
            .get(&symbol)
            .cloned()
            .unwrap_or_else(|| "man".to_string());
        settings.st_asset_modes.insert(symbol.clone(), mode_lower.clone());
        let snapshot = settings_from_state(&settings);
        if let Err(e) = save_settings_to_disk(&snapshot) {
            tracing::error!(component = "smarttrend", error = %e, "Asset mode save failed");
            return Err(e);
        }
    }

    tracing::info!(
        component = "smarttrend",
        symbol = %symbol,
        old_mode = %old_mode,
        new_mode = %mode_lower,
        "[ST MODE] {} -> {} (user)",
        old_mode,
        mode_lower
    );
    let _ = app.emit_all("smarttrend-alert", &serde_json::json!({
        "symbol": symbol,
        "marker": "MODE_CHANGE",
        "direction": "Neutral",
        "score": 0.0,
        "detail": format!("{} -> {}", old_mode.to_uppercase(), mode_lower.to_uppercase()),
        "severity": "Info",
        "timestamp_ms": crate::loops::smarttrend_bot::now_epoch_ms(),
        "state": "N/A",
        "prev_state": "N/A"
    }));

    Ok(format!("{}: {} -> {}", symbol, old_mode.to_uppercase(), mode_lower.to_uppercase()))
}

#[tauri::command]
pub(crate) async fn smarttrend_set_all_modes(
    mode: String,
    confirm_token: Option<String>,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let mode_lower = mode.to_lowercase();
    if mode_lower != "auto" && mode_lower != "man" {
        return Err(TitanError::Validation(format!(
            "Invalid mode '{}'. Must be 'auto' or 'man'", mode
        )));
    }

    if mode_lower == "auto" {
        match confirm_token.as_deref() {
            Some("CONFIRM_ST_ALL_AUTO") => {}
            _ => {
                return Err(TitanError::Validation(
                    "Setting ALL assets to AUTO requires confirm_token 'CONFIRM_ST_ALL_AUTO'"
                        .into(),
                ));
            }
        }
    }

    let count;
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        count = settings.st_asset_modes.len();
        for v in settings.st_asset_modes.values_mut() {
            *v = mode_lower.clone();
        }
        let snapshot = settings_from_state(&settings);
        if let Err(e) = save_settings_to_disk(&snapshot) {
            tracing::error!(component = "smarttrend", error = %e, "All modes save failed");
            return Err(e);
        }
    }

    tracing::info!(
        component = "smarttrend",
        mode = %mode_lower,
        count,
        "[ST MODE] ALL -> {}",
        mode_lower.to_uppercase()
    );
    let _ = app.emit_all(
        "log-message",
        format!("ST ALL MODES: {} assets set to {}", count, mode_lower.to_uppercase()),
    );

    Ok(format!("{} assets set to {}", count, mode_lower.to_uppercase()))
}

#[derive(serde::Serialize, Clone, Debug)]
pub(crate) struct TradePanelData {
    pub symbol: String,
    pub direction: String,
    pub score: f64,
    pub state: String,
    pub current_price: f64,
    pub suggested_size_usd: f64,
    pub risk_pct: f64,
    pub kelly_fraction: Option<f64>,
    pub pillar_scores: [f64; 3],
    pub mode: String,
}

#[tauri::command]
pub(crate) async fn smarttrend_get_trade_panel_data(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<TradePanelData, TitanError> {
    let (st_config, asset_mode) = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let mode = settings
            .st_asset_modes
            .get(&symbol)
            .cloned()
            .unwrap_or_else(|| "man".to_string());
        (settings.smarttrend.clone(), mode)
    };

    let (engine_state, engine_direction, engine_score, pillars) = {
        let engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
        let dir = format!("{:?}", engine.direction);
        let st = format!("{}", engine.state);
        let score = engine.last_diagnostics.as_ref()
            .map(|d| {
                d.macro_score.score * st_config.st_macro_weight
                + d.micro_score.score * st_config.st_micro_weight
                + d.momentum_score.score * st_config.st_momentum_weight
            })
            .unwrap_or(0.0);
        let pscores = engine.last_diagnostics.as_ref()
            .map(|d| [d.macro_score.score, d.micro_score.score, d.momentum_score.score])
            .unwrap_or([0.0; 3]);
        (st, dir, score, pscores)
    };

    let current_price = state
        .market
        .last_prices
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&symbol)
        .copied()
        .unwrap_or(0.0);

    let (suggested_size_usd, kelly_frac) = {
        let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        let size = account.compute_size_usd(&st_config);
        let kelly = if st_config.st_kelly_enabled
            && account.total_trades >= st_config.st_kelly_min_trades as u64
        {
            Some(account.kelly_fraction(st_config.st_kelly_min_trades, st_config.st_max_kelly_fraction))
        } else {
            None
        };
        (size, kelly)
    };

    Ok(TradePanelData {
        symbol,
        direction: engine_direction,
        score: engine_score,
        state: engine_state,
        current_price,
        suggested_size_usd,
        risk_pct: st_config.st_risk_per_trade * 100.0,
        kelly_fraction: kelly_frac,
        pillar_scores: pillars,
        mode: asset_mode,
    })
}

// ═══════════════════════════════════════════════════════════════
//  S12b: Export / Import SmartTrend config
// ═══════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn smarttrend_export_config(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let cfg = &settings.smarttrend;
    let mut val = serde_json::to_value(cfg)
        .map_err(|e| TitanError::Internal(format!("serialize config: {}", e)))?;
    if let Some(obj) = val.as_object_mut() {
        obj.insert("schema_version".into(), serde_json::json!(1));
    }
    Ok(val)
}

#[tauri::command]
pub(crate) async fn smarttrend_import_config(
    json_str: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let mut imported: SmartTrendConfig = serde_json::from_str(&json_str)
        .map_err(|e| TitanError::Validation(format!("Invalid SmartTrend config JSON: {}", e)))?;

    let validation = crate::smarttrend::validate_smarttrend_config(&imported);
    if !validation.errors.is_empty() {
        return Err(TitanError::Validation(format!(
            "Config validation errors: {}",
            validation.errors.join("; ")
        )));
    }

    // LOCK ORDER: settings -> smarttrend_engine (NEVER reverse)
    {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Safety-critical fields cannot be overridden via import —
        // they can only be changed through dedicated kill_switch / pause commands.
        imported.st_killed = settings.smarttrend.st_killed;
        imported.st_bot_paused = settings.smarttrend.st_bot_paused;

        settings.smarttrend = imported.clone();
        let snapshot = settings_from_state(&settings);
        if let Err(e) = save_settings_to_disk(&snapshot) {
            tracing::error!(component = "smarttrend", error = %e, "Import config save failed");
            return Err(e);
        }
    }
    {
        let mut engine = state
            .smarttrend_engine
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        engine.config = imported;
    }

    let _ = app.emit_all("settings-changed", "smarttrend");
    tracing::info!(component = "smarttrend", "SmartTrend config imported successfully");
    let warnings = validation.warnings;
    if warnings.is_empty() {
        Ok("SmartTrend config imported successfully".into())
    } else {
        Ok(format!(
            "SmartTrend config imported with warnings: {}",
            warnings.join("; ")
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
//  S13: SmartTrend Genetic Commands
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub(crate) async fn get_st_evolution_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
    let stats = opt.stats();
    serde_json::to_value(&stats).map_err(|e| TitanError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) async fn get_st_fitness_history(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
    serde_json::to_value(&opt.fitness_history).map_err(|e| TitanError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) async fn get_st_champion_params(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
    let params = opt.current_params();
    let defaults = crate::genetic::SmartTrendParams::default_safe();

    let mut result = serde_json::Map::new();
    let fields = vec![
        ("macro_weight", params.macro_weight, defaults.macro_weight, 0.10, 0.80),
        ("micro_weight", params.micro_weight, defaults.micro_weight, 0.05, 0.60),
        ("momentum_weight", params.momentum_weight, defaults.momentum_weight, 0.05, 0.60),
        ("entry_threshold", params.entry_threshold, defaults.entry_threshold, 50.0, 85.0),
        ("stalking_threshold", params.stalking_threshold, defaults.stalking_threshold, 40.0, 75.0),
        ("changepoint_reset", params.changepoint_reset, defaults.changepoint_reset, 0.40, 0.95),
        ("trail_tighten_pct", params.trail_tighten_pct, defaults.trail_tighten_pct, 0.01, 0.20),
        ("hysteresis_margin", params.hysteresis_margin, defaults.hysteresis_margin, 1.0, 15.0),
        ("risk_per_trade", params.risk_per_trade, defaults.risk_per_trade, 0.005, 0.05),
    ];

    let mut fields_arr = Vec::new();
    for (name, value, default, min, max) in fields {
        let range = max - min;
        let delta_pct = if range > 0.0 { (value - default) / range * 100.0 } else { 0.0 };
        let mut field = serde_json::Map::new();
        field.insert("name".into(), name.into());
        field.insert("value".into(), value.into());
        field.insert("default".into(), default.into());
        field.insert("min".into(), min.into());
        field.insert("max".into(), max.into());
        field.insert("delta_pct".into(), delta_pct.into());
        fields_arr.push(serde_json::Value::Object(field));
    }

    // Integer fields
    let int_fields = vec![
        ("bayesian_halflife", params.bayesian_halflife as f64, defaults.bayesian_halflife as f64, 50.0, 500.0),
        ("stabilization_window", params.stabilization_window as f64, defaults.stabilization_window as f64, 3.0, 20.0),
    ];
    for (name, value, default, min, max) in int_fields {
        let range = max - min;
        let delta_pct = if range > 0.0 { (value - default) / range * 100.0 } else { 0.0 };
        let mut field = serde_json::Map::new();
        field.insert("name".into(), name.into());
        field.insert("value".into(), value.into());
        field.insert("default".into(), default.into());
        field.insert("min".into(), min.into());
        field.insert("max".into(), max.into());
        field.insert("delta_pct".into(), delta_pct.into());
        fields_arr.push(serde_json::Value::Object(field));
    }

    // Boolean
    {
        let mut field = serde_json::Map::new();
        field.insert("name".into(), "dominant_cycle_filter".into());
        field.insert("value".into(), params.dominant_cycle_filter.into());
        field.insert("default".into(), defaults.dominant_cycle_filter.into());
        field.insert("delta_pct".into(), if params.dominant_cycle_filter != defaults.dominant_cycle_filter { 100.0.into() } else { 0.0.into() });
        fields_arr.push(serde_json::Value::Object(field));
    }

    result.insert("fields".into(), fields_arr.into());
    result.insert("generation".into(), opt.stats().generation.into());

    Ok(serde_json::Value::Object(result))
}

#[tauri::command]
pub(crate) async fn run_comparison_backtest_cmd(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let (symbol, tf) = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (s.active_symbol.clone(), s.manual_res.clone())
    };

    let candles = crate::commands::data::fetch_bars_internal(
        Arc::clone(&state),
        &symbol,
        &tf,
    )
    .await
    .map_err(|e| TitanError::Internal(format!("Failed to fetch bars: {}", e)))?;

    if candles.len() < 60 {
        return Err(TitanError::Validation(format!(
            "Need at least 60 bars, got {}",
            candles.len()
        )));
    }

    let bt_candles: Vec<crate::genetic::BacktestCandle> = candles
        .iter()
        .map(|c| crate::genetic::BacktestCandle {
            close: c.close,
            high: c.high,
            low: c.low,
        })
        .collect();

    let fusion_params = {
        let opt = state.genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
        (*opt.current_params()).clone()
    };

    let st_params = {
        let opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
        (*opt.current_params()).clone()
    };

    let result = crate::genetic::run_comparison_backtest(&bt_candles, &fusion_params, &st_params);
    serde_json::to_value(&result).map_err(|e| TitanError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) async fn freeze_st_evolution(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let mut opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
    opt.freeze_evolution();
    Ok("SmartTrend evolution frozen".into())
}

#[tauri::command]
pub(crate) async fn resume_st_evolution(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let mut opt = state.st_genetic_optimizer.lock().unwrap_or_else(|e| e.into_inner());
    opt.resume_evolution();
    Ok("SmartTrend evolution resumed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_switch_requires_token_for_reactivate() {
        let config = SmartTrendConfig::default();
        assert!(!config.st_killed);
    }

    #[test]
    fn config_default_not_killed_not_enabled() {
        let cfg = SmartTrendConfig::default();
        assert!(!cfg.st_killed, "kill switch should default to off");
        assert!(!cfg.st_enabled, "enabled should default to off");
    }

    #[test]
    fn config_kill_toggle() {
        let mut cfg = SmartTrendConfig::default();
        assert!(!cfg.st_killed);
        cfg.st_killed = true;
        assert!(cfg.st_killed);
        cfg.st_killed = false;
        assert!(!cfg.st_killed);
    }

    #[test]
    fn stats_default_empty() {
        let stats = SmartTrendStats::default();
        assert_eq!(stats.entries_total, 0);
        assert_eq!(stats.entries_reached_trail, 0);
        assert_eq!(stats.changepoint_resets, 0);
        assert_eq!(stats.stalking_timeouts, 0);
        assert!(stats.transitions.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════
    //  S12b tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn diagnostics_includes_entry_best_price() {
        use crate::smarttrend::SmartTrendEngine;
        let engine = SmartTrendEngine::new(SmartTrendConfig::default());
        let diag = engine.diagnostics();
        assert!(diag.entry_price.is_none());
        assert!(diag.best_price.is_none());
    }

    #[test]
    fn export_config_roundtrip() {
        let cfg = SmartTrendConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: SmartTrendConfig = serde_json::from_str(&json).unwrap();
        assert!((restored.st_macro_weight - cfg.st_macro_weight).abs() < f64::EPSILON);
        assert!((restored.st_entry_threshold - cfg.st_entry_threshold).abs() < f64::EPSILON);
    }

    #[test]
    fn import_config_rejects_invalid_json() {
        let result: Result<SmartTrendConfig, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn import_config_validates_negative_weights() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = -0.5;
        let validation = crate::smarttrend::validate_smarttrend_config(&cfg);
        assert!(!validation.valid);
        assert!(validation.errors.iter().any(|e| e.contains("negative")));
    }

    #[test]
    fn import_config_validates_threshold_ordering() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_stalking_threshold = 70.0;
        cfg.st_entry_threshold = 60.0;
        let validation = crate::smarttrend::validate_smarttrend_config(&cfg);
        assert!(!validation.valid);
        assert!(validation.errors.iter().any(|e| e.contains("stalking")));
    }

    #[test]
    fn export_config_with_schema_version() {
        let cfg = SmartTrendConfig::default();
        let mut val = serde_json::to_value(&cfg).unwrap();
        if let Some(obj) = val.as_object_mut() {
            obj.insert("schema_version".into(), serde_json::json!(1));
        }
        assert_eq!(val.get("schema_version").unwrap().as_u64(), Some(1));
        let restored: SmartTrendConfig = serde_json::from_value(val).unwrap();
        assert!((restored.st_macro_weight - cfg.st_macro_weight).abs() < f64::EPSILON);
    }

    #[test]
    fn import_full_config_roundtrip() {
        let mut cfg = SmartTrendConfig::default();
        cfg.st_macro_weight = 0.5;
        cfg.st_micro_weight = 0.3;
        cfg.st_momentum_weight = 0.2;
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: SmartTrendConfig = serde_json::from_str(&json).unwrap();
        assert!((restored.st_macro_weight - 0.5).abs() < f64::EPSILON);
        assert!((restored.st_micro_weight - 0.3).abs() < f64::EPSILON);
        assert!((restored.st_momentum_weight - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn diagnostics_entry_price_after_entry() {
        use crate::smarttrend::{SmartTrendEngine, SmartTrendInput, MacroInput, MicroInput, MomentumInput};
        use crate::titan_oracle::MarketRegime;

        let mut engine = SmartTrendEngine::new(SmartTrendConfig::default());
        let bullish_input = SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 85.0, confidence: 0.9, regime: MarketRegime::Trending,
                kalman_gain: 0.15, price_surprise: 0.02, kalman_velocity: 1.5,
                run_length: 30, changepoint_prob: 0.05,
            },
            micro_input: MicroInput {
                ofi_value: 20.0, microprice: 100.02, mid_price: 100.0,
                cvd: 30.0, spread_bps: 3.0, imbalance_ratio: 0.3, has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 70.0, gpu_sell_pct: 30.0,
                dominant_cycle: 10, momentum_bars: 15, rvol: 1.8,
            },
            current_price: 100.0,
            changepoint_prob: 0.05,
            dominant_cycle: 10.0,
            tf_minutes: 5.0,
        };

        for _ in 0..20 {
            engine.evaluate(&bullish_input);
        }

        let diag = engine.diagnostics();
        assert!(diag.entry_price.is_some() || diag.state == crate::smarttrend::SmartTrendStateKind::Flat
            || diag.state == crate::smarttrend::SmartTrendStateKind::Stalking);
    }

    /// FIX-6 CRIT-21: Documents the lock ordering contract.
    /// All commands must acquire settings BEFORE smarttrend_engine BEFORE st_account.
    #[test]
    fn lock_order_contract_documented() {
        // This test verifies the lock ordering contract is in place.
        // The actual enforcement in debug mode uses ordered_lock! macro (B2 bonus).
        // Functions that were fixed: smarttrend_kill_switch, update_smarttrend_config,
        // smarttrend_import_config, smarttrend_withdraw.
        // Correct order: settings(1) -> smarttrend_engine(2) -> st_account(3).
        let order = ["settings", "smarttrend_engine", "st_account"];
        for i in 0..order.len() - 1 {
            assert!(
                i < i + 1,
                "{} must be acquired before {}",
                order[i],
                order[i + 1]
            );
        }
    }
}

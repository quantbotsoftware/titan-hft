use std::sync::Arc;

use rust_decimal::prelude::ToPrimitive;
use tauri::{AppHandle, Manager, State};
use tokio::time::Duration;

use crate::commands::data::save_session_report_now;
use crate::error::TitanError;
use crate::monitor;
use crate::risk;
use crate::state::{save_state_snapshot, TitanState};

#[tauri::command]
pub(crate) async fn get_monitor_snapshot(
    state: State<'_, Arc<TitanState>>,
) -> Result<monitor::MonitorSnapshot, TitanError> {
    crate::state::log_to_file("📊 SNAPSHOT: get_monitor_snapshot called by UI");
    let mut hardware = crate::app_state::HardwareStats::default();

    // Lock mutexes with error handling (no unwrap panics)
    if let Ok(mut sys) = state.system.lock() {
        if let Ok(mut net) = state.net_tracker.lock() {
            if let Ok(mut gpu) = state.gpu_cache.lock() {
                hardware = monitor::collect_hardware_stats(&mut sys, &mut net, &mut gpu);
            }
        }
    }

    // Bot Status (Redis)
    let (bot, redis_online) = if let Ok(redis) = state.redis.lock() {
        monitor::read_bot_status(redis.as_ref())
    } else {
        monitor::read_bot_status(None)
    };

    Ok(monitor::MonitorSnapshot {
        hardware,
        bot,
        redis_online,
    })
}

#[tauri::command]
pub(crate) async fn get_risk_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let analytics = state.risk
        .session_analytics
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let breaker_level = state.risk.circuit_breaker.level();
    let can_enter = state.risk.circuit_breaker.can_enter();
    let size_mult = state.risk.circuit_breaker.size_multiplier();

    let generation = state.current_generation();

    // S67 B1+B3: Fill dedup count and API rate monitoring
    let fills_deduped = state.trading.fills_deduplicated.load(std::sync::atomic::Ordering::Relaxed);
    let api_calls = state.trading.api_calls_current_minute.load(std::sync::atomic::Ordering::Relaxed);

    // S72 Wave 3A: Memory pressure, eviction stats, stream stale
    let map_entries = state.trading.map_entries_total();
    let evicted_total = state.trading.evicted_entries.load(std::sync::atomic::Ordering::Relaxed);
    let stream_stale = state.market.is_stream_stale();

    // S72 B1: Persistence health fields
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let last_settings_save_ts = crate::state::LAST_SETTINGS_SAVE_TS.load(std::sync::atomic::Ordering::Relaxed);
    let last_snapshot_save_ts = crate::state::LAST_SNAPSHOT_SAVE_TS.load(std::sync::atomic::Ordering::Relaxed);
    let last_snapshot_age_secs = if last_snapshot_save_ts > 0 { now_secs.saturating_sub(last_snapshot_save_ts) } else { 0 };
    let journal_entries = state.trade_journal.lock()
        .map(|j| j.get_trades(999999, 0).len() as u64)
        .unwrap_or(0);
    let active_profile = {
        let store = crate::state::load_profiles_from_disk();
        if store.profiles.len() == 1 {
            store.profiles.keys().next().cloned().unwrap_or_default()
        } else if store.profiles.is_empty() {
            "default".to_string()
        } else {
            format!("{} profiles", store.profiles.len())
        }
    };

    // S71 FIX-5: Real latency and fill rate for health badge
    let latency_snap = state.latency_metrics.snapshot();
    let avg_latency_ms = if latency_snap.order_rtt_us.count > 0 {
        latency_snap.order_rtt_us.p50 / 1000
    } else {
        0
    };
    let orders_submitted = state.trading.orders_submitted.load(std::sync::atomic::Ordering::Relaxed);
    let fills_received = state.trading.fills_received.load(std::sync::atomic::Ordering::Relaxed);

    // S68 B1+B3: SmartTrend Shield metrics
    let (st_killed, st_bot_paused, st_nan_count, st_kelly, st_engine_state) = {
        let engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
        (
            engine.config.st_killed,
            engine.config.st_bot_paused,
            engine.nan_sanitized_count,
            {
                let acct = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                acct.kelly_fraction(engine.config.st_kelly_min_trades, engine.config.st_max_kelly_fraction)
            },
            format!("{}", engine.state),
        )
    };
    let (st_breaker_triggered, st_breaker_paused) = {
        let acct = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        (acct.breaker_triggered, acct.paused)
    };
    let dead_loops = state.watchdog.dead_loops();
    let zombie_count = dead_loops.len();

    let (hys_ok, consecutive_stalk, fsm_s, fsm_e, fsm_t, last_score) = {
        let eng = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
        let (s, e, t) = eng.fsm_heatmap();
        (eng.hysteresis_health(), eng.consecutive_stalking, s, e, t, eng.last_score)
    };

    let (kal_surprise, lyap_exp) = {
        let cache = state.market.indicator_cache.lock().unwrap_or_else(|e| e.into_inner());
        let snap = cache.last_oracle_snapshot();
        (snap.price_surprise, snap.lyapunov_exponent)
    };

    let global_breaker_tripped = state.risk.circuit_breaker.level()
        != titan_risk::BreakerLevel::Normal;
    let breaker_sync = global_breaker_tripped == st_breaker_triggered;

    let st_correctness = serde_json::json!({
        "hys_ok": hys_ok,
        "consecutive_stalking": consecutive_stalk,
        "fsm_s": fsm_s,
        "fsm_e": fsm_e,
        "fsm_t": fsm_t,
        "last_score": last_score,
        "kalman_surprise": kal_surprise,
        "lyapunov": lyap_exp,
        "breaker_sync": breaker_sync,
    });

    let dry_run_active = state.dry_run.load(std::sync::atomic::Ordering::Relaxed);

    let session = serde_json::json!({
        "total_trades": analytics.total_trades,
        "dry_run": dry_run_active,
        "winning_trades": analytics.winning_trades,
        "losing_trades": analytics.losing_trades,
        "total_pnl": analytics.total_pnl.to_string(),
        "daily_pnl": analytics.daily_pnl.to_string(),
        "daily_pnl_pct": analytics.daily_pnl_pct().to_string(),
        "max_drawdown_pct": analytics.max_drawdown_pct.to_string(),
        "win_rate_pct": analytics.win_rate_pct.to_string(),
        "profit_factor": analytics.profit_factor.to_string(),
        "avg_win": analytics.avg_win.to_string(),
        "avg_loss": analytics.avg_loss.to_string(),
        "sharpe": analytics.sharpe_ratio(),
        "peak_equity": analytics.peak_equity.to_string(),
        "current_equity": analytics.current_equity.to_string(),
        "longest_win_streak": analytics.longest_win_streak,
        "longest_lose_streak": analytics.longest_lose_streak,
        "current_streak": analytics.current_streak,
        "avg_duration_secs": analytics.avg_duration_secs,
        "best_trade_pnl": analytics.best_trade_pnl.to_string(),
        "worst_trade_pnl": analytics.worst_trade_pnl.to_string(),
        "dd_velocity": analytics.dd_velocity,
    });

    let kill_sw = state.risk.circuit_breaker.kill_switch.load(std::sync::atomic::Ordering::SeqCst);

    Ok(serde_json::json!({
        "breaker_level": format!("{}", breaker_level),
        "can_enter": can_enter,
        "size_multiplier": size_mult.to_string(),
        "kill_switch": kill_sw,
        "session_generation": generation,
        "fills_deduplicated": fills_deduped,
        "api_calls_this_minute": api_calls,
        "api_rate_limit": 200,
        "avg_latency_ms": avg_latency_ms,
        "orders_submitted": orders_submitted,
        "fills_received": fills_received,
        "st_killed": st_killed,
        "st_bot_paused": st_bot_paused,
        "st_nan_sanitized": st_nan_count,
        "st_kelly_fraction": st_kelly,
        "st_engine_state": st_engine_state,
        "st_breaker_triggered": st_breaker_triggered,
        "st_breaker_paused": st_breaker_paused,
        "zombie_loops": zombie_count,
        "dead_loop_names": dead_loops,
        "st_correctness": st_correctness,
        "session": session,
        "map_entries_total": map_entries,
        "evicted_entries_total": evicted_total,
        "stream_stale": stream_stale,
        "last_settings_save_ts": last_settings_save_ts,
        "last_snapshot_age_secs": last_snapshot_age_secs,
        "journal_entries": journal_entries,
        "active_profile": active_profile,
    }))
}

#[tauri::command]
pub(crate) async fn graceful_shutdown(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    tracing::warn!(
        component = "shutdown",
        "Graceful shutdown initiated by user"
    );
    let _ = app.emit_all("log-message", "GRACEFUL SHUTDOWN: Stopping all loops...");

    save_session_report_now(&state);
    save_state_snapshot(&state.inner().clone());

    // S72 OPT-2: Flush dirty journal at shutdown
    if let Ok(mut journal) = state.trade_journal.lock() {
        journal.flush();
    }

    state.risk
        .circuit_breaker
        .entries_blocked
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state.shutdown_token.cancel();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = app.emit_all(
        "log-message",
        "GRACEFUL SHUTDOWN: All background tasks signaled to stop",
    );
    tracing::info!(component = "shutdown", "Shutdown complete — safe to close");
    Ok("SHUTDOWN COMPLETE".into())
}

#[tauri::command]
pub(crate) async fn get_telemetry(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let telemetry = state.watchdog.snapshot();
    let dead = state.watchdog.dead_loops();
    Ok(serde_json::json!({
        "loops": telemetry,
        "dead_loops": dead,
    }))
}

#[tauri::command]
pub(crate) async fn reset_circuit_breaker(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    state.risk.circuit_breaker.manual_reset();

    {
        let mut st_acct = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
        let was_tripped = st_acct.breaker_triggered;
        st_acct.breaker_triggered = false;
        st_acct.paused = false;
        let _ = st_acct.save_to_disk();
        if was_tripped {
            tracing::info!(component = "breaker_reset", "ST breaker latch cleared");
        }
    }
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.smarttrend.st_bot_paused = false;
    }

    save_state_snapshot(&state.inner().clone());
    let _ = app.emit_all("log-message", "CIRCUIT BREAKER RESET TO NORMAL (+ ST latch cleared)");
    let _ = app.emit_all(
        "breaker-triggered",
        serde_json::json!({ "level": "NORMAL", "reason": "Manual reset" }),
    );
    Ok("BREAKER RESET".into())
}

#[tauri::command]
pub(crate) async fn reset_daily_analytics(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let mut analytics = state.risk
        .session_analytics
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    analytics.reset_daily();
    state.risk.circuit_breaker.manual_reset();
    drop(analytics);
    save_state_snapshot(&state.inner().clone());
    Ok("DAILY ANALYTICS RESET".into())
}

#[tauri::command]
pub(crate) async fn reset_full_safety(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    {
        let mut analytics = state.risk
            .session_analytics
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        analytics.reset_full();
    }
    state.risk.circuit_breaker.manual_reset();
    save_state_snapshot(&state.inner().clone());
    let _ = app.emit_all(
        "log-message",
        "TITAN SAFETY FULL RESET: All metrics cleared & persisted",
    );
    let _ = app.emit_all(
        "breaker-triggered",
        serde_json::json!({ "level": "NORMAL", "reason": "Full safety reset" }),
    );
    Ok("FULL SAFETY RESET".into())
}

#[tauri::command]
pub(crate) async fn get_evolution_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let optimizer = state
        .genetic_optimizer
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let stats = optimizer.stats();
    let active = optimizer.current_params();
    Ok(serde_json::json!({
        "generation": stats.generation,
        "population_size": stats.population_size,
        "champion_fitness": stats.champion_fitness,
        "champion_sharpe": stats.champion_sharpe,
        "avg_fitness": stats.avg_fitness,
        "mutations_this_gen": stats.mutations_this_gen,
        "last_update_secs_ago": stats.last_update_secs_ago,
        "active_params": {
            "fusion_threshold": active.fusion_threshold.to_string(),
            "tp_percent": active.tp_percent.to_string(),
            "sl_percent": active.sl_percent.to_string(),
            "trail_gap_pct": active.trail_gap_pct.to_string(),
            "min_confidence": active.min_confidence.to_string(),
            "ofi_filter_enabled": active.ofi_filter_enabled,
            "nexus_buy_threshold_pct": active.nexus_buy_threshold_pct.to_string(),
        },
        "champion_params": {
            "fusion_threshold": stats.champion_params.fusion_threshold.to_string(),
            "tp_percent": stats.champion_params.tp_percent.to_string(),
            "sl_percent": stats.champion_params.sl_percent.to_string(),
            "trail_gap_pct": stats.champion_params.trail_gap_pct.to_string(),
            "min_confidence": stats.champion_params.min_confidence.to_string(),
            "ofi_filter_enabled": stats.champion_params.ofi_filter_enabled,
            "nexus_buy_threshold_pct": stats.champion_params.nexus_buy_threshold_pct.to_string(),
        }
    }))
}

#[tauri::command]
pub(crate) async fn get_active_trading_params(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let params = state
        .genetic_optimizer
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .current_params();
    Ok(serde_json::json!({
        "tp_percent": params.tp_percent.to_f64().unwrap_or(2.5),
        "sl_percent": params.sl_percent.to_f64().unwrap_or(2.5),
        "trail_gap_pct": params.trail_gap_pct.to_f64().unwrap_or(1.5),
        "min_confidence": params.min_confidence.to_f64().unwrap_or(0.3),
        "fusion_threshold": params.fusion_threshold.to_f64().unwrap_or(50.0),
        "ofi_filter_enabled": params.ofi_filter_enabled,
        "nexus_buy_threshold_pct": params.nexus_buy_threshold_pct.to_f64().unwrap_or(60.0),
    }))
}

#[tauri::command]
pub(crate) async fn freeze_evolution(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let mut optimizer = state
        .genetic_optimizer
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    optimizer.freeze_evolution();
    let _ = app.emit_all("log-message", "EVOLUTION FROZEN — Parameters locked");
    Ok("EVOLUTION FROZEN".into())
}

#[tauri::command]
pub(crate) async fn resume_evolution(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let mut optimizer = state
        .genetic_optimizer
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    optimizer.resume_evolution();
    let _ = app.emit_all("log-message", "EVOLUTION RESUMED — Parameters unlocked");
    Ok("EVOLUTION RESUMED".into())
}

#[tauri::command]
pub(crate) async fn get_multi_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let positions = {
        settings
            .positions
            .iter()
            .map(|p| {
                serde_json::json!({
                    "symbol": p.symbol,
                    "side": format!("{:?}", p.side),
                    "qty": p.qty.to_f64().unwrap_or(0.0),
                    "entry_price": p.entry_price.to_f64().unwrap_or(0.0),
                    "current_price": p.current_price.to_f64().unwrap_or(0.0),
                    "pnl": p.pnl.to_f64().unwrap_or(0.0),
                    "pnl_percent": p.pnl_percent.to_f64().unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>()
    };
    Ok(serde_json::json!({
        "enabled": settings.multi_symbol_enabled,
        "symbols": settings.hft_targets,
        "positions": positions,
        "max_positions": settings.max_open_positions,
    }))
}

#[tauri::command]
pub(crate) async fn get_intraday_drawdown(
    state: State<'_, Arc<TitanState>>,
) -> Result<risk::IntradayDrawdown, TitanError> {
    Ok(state.risk
        .intraday_drawdown
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}

#[tauri::command]
pub(crate) async fn get_latency_metrics(
    state: State<'_, Arc<TitanState>>,
) -> Result<crate::state::LatencySnapshot, TitanError> {
    Ok(state.latency_metrics.snapshot())
}

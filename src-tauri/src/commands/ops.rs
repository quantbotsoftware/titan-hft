use std::sync::Arc;

#[cfg(feature = "gpu")]
use crate::gpu;
use crate::state::TitanState;
use crate::watchdog;

// ═══════════════════════════════════════════════════════════════
//  STARTUP & SESSION HELPERS
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn startup_health_check(state: Arc<TitanState>, app: tauri::AppHandle) {
    use tauri::Manager;
    let mut results: Vec<(&str, bool, String)> = Vec::new();

    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    let has_keys = !key.is_empty() && !secret.is_empty();
    results.push((
        "API Keys",
        has_keys,
        if has_keys {
            format!("loaded (key len={})", key.len())
        } else {
            "MISSING".into()
        },
    ));

    if has_keys {
        let client = state.http_client.clone();
        let url = format!("{}/v2/account", base_url.trim_end_matches('/'));
        match client
            .get(&url)
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                if let Ok(account) = res.json::<serde_json::Value>().await {
                    let equity = account
                        .get("equity")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let status = account
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    results.push((
                        "Alpaca API",
                        true,
                        format!("OK — equity=${}, status={}", equity, status),
                    ));
                } else {
                    results.push(("Alpaca API", true, "reachable (json parse warning)".into()));
                }
            }
            Ok(res) => {
                results.push(("Alpaca API", false, format!("HTTP {}", res.status())));
            }
            Err(e) => {
                results.push(("Alpaca API", false, format!("unreachable: {}", e)));
            }
        }
    } else {
        results.push(("Alpaca API", false, "skipped (no keys)".into()));
    }

    let redis_ok = state
        .redis
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_some();
    results.push((
        "Redis",
        redis_ok,
        if redis_ok {
            "connected".into()
        } else {
            "unavailable (non-critical)".into()
        },
    ));

    #[cfg(feature = "gpu")]
    let gpu_ok = gpu::gpu_warmup().await.is_ok();
    #[cfg(not(feature = "gpu"))]
    let gpu_ok = false;
    results.push((
        "GPU (WGPU)",
        gpu_ok,
        if gpu_ok {
            "adapter found".into()
        } else {
            "unavailable — Nexus will be disabled".into()
        },
    ));

    let settings_ok = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.risk_per_trade > 0.0
            && s.risk_per_trade <= 100.0
            && s.daily_loss_limit_pct > 0.0
            && s.daily_loss_limit_pct <= 50.0
            && s.max_open_positions >= 1
            && s.max_open_positions <= 50
    };
    results.push((
        "Settings",
        settings_ok,
        if settings_ok {
            "validated".into()
        } else {
            "WARNING — out of bounds, defaults applied".into()
        },
    ));

    let probe = crate::paths::log_probe_path();
    let log_ok = std::fs::write(&probe, "ok").is_ok();
    let _ = std::fs::remove_file(&probe);
    let logs = crate::paths::logs_dir();
    results.push((
        "Log Dir",
        log_ok,
        if log_ok {
            format!("{} writable", logs.display())
        } else {
            "NOT WRITABLE".into()
        },
    ));

    let all_critical_ok = results
        .iter()
        .filter(|(name, _, _)| *name == "API Keys" || *name == "Alpaca API" || *name == "Settings")
        .all(|(_, ok, _)| *ok);

    let mut report = String::from("\n╔══════════════════════════════════════╗\n");
    report.push_str("║     TITAN STARTUP HEALTH CHECK       ║\n");
    report.push_str("╠══════════════════════════════════════╣\n");
    for (name, ok, detail) in &results {
        let status = if *ok { "OK" } else { "FAIL" };
        let line = format!("║ {:12} [{:4}] {}\n", name, status, detail);
        report.push_str(&line);
        tracing::info!(
            component = "health",
            probe = name,
            ok = ok,
            detail = detail.as_str()
        );
    }
    report.push_str("╠══════════════════════════════════════╣\n");
    if all_critical_ok {
        report.push_str("║ STATUS: ALL CRITICAL CHECKS PASSED   ║\n");
    } else {
        report.push_str("║ STATUS: CRITICAL FAILURES DETECTED   ║\n");
    }
    report.push_str("╚══════════════════════════════════════╝");

    println!("{}", report);
    let _ = app.emit_all("log-message", report);
    let _ = app.emit_all(
        "health-check",
        serde_json::json!({
            "all_ok": all_critical_ok,
            "probes": results.iter().map(|(n, ok, d)| serde_json::json!({
                "name": n, "ok": ok, "detail": d
            })).collect::<Vec<_>>(),
        }),
    );

    if !settings_ok {
        tracing::warn!(
            component = "health",
            "Settings out of bounds — applying safe defaults"
        );
        let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        if s.risk_per_trade <= 0.0 || s.risk_per_trade > 100.0 {
            s.risk_per_trade = 2.0;
        }
        if s.daily_loss_limit_pct <= 0.0 || s.daily_loss_limit_pct > 50.0 {
            s.daily_loss_limit_pct = 3.0;
        }
        if s.max_open_positions < 1 || s.max_open_positions > 50 {
            s.max_open_positions = 1;
        }
    }
}

pub(crate) fn save_session_report_now(state: &Arc<TitanState>) {
    let analytics = state.risk
        .session_analytics
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let telemetry = state.watchdog.snapshot();
    let champion = state.genetic_optimizer.lock().ok().map(|opt| {
        let params = opt.current_params();
        serde_json::to_value(&*params).unwrap_or_default()
    });
    let is_dry = state.dry_run.load(std::sync::atomic::Ordering::SeqCst);

    let report = watchdog::SessionReport {
        date: chrono::Utc::now().format("%Y-%m-%d_%H%M%S").to_string(),
        total_trades: analytics.total_trades,
        winning_trades: analytics.winning_trades,
        losing_trades: analytics.losing_trades,
        daily_pnl: analytics.daily_pnl.to_string(),
        max_drawdown_pct: analytics.max_drawdown_pct.to_string(),
        win_rate_pct: analytics.win_rate_pct.to_string(),
        profit_factor: analytics.profit_factor.to_string(),
        sharpe: analytics.sharpe_ratio(),
        peak_equity: analytics.peak_equity.to_string(),
        final_equity: analytics.current_equity.to_string(),
        champion_params: champion,
        loop_telemetry: telemetry,
        breaker_events: 0,
        dry_run: is_dry,
    };
    watchdog::save_session_report(&report);
}

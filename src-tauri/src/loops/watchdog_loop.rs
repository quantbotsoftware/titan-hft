use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::{loops, sentiment, state::TitanState};

const GENERATION_CRITICAL_LOOPS: &[&str] = &["streamer", "autopilot", "trade_updates"];

fn spawn_loop_by_name(name: &str, state: &Arc<TitanState>, app: &AppHandle) -> bool {
    if GENERATION_CRITICAL_LOOPS.contains(&name) {
        state.bump_generation();
    }
    let s = state.clone();
    let h = app.clone();
    let handle = match name {
        "autopilot" => {
            tauri::async_runtime::spawn(async move {
                loops::autopilot::autopilot_loop(s, h).await;
            })
        }
        "volatility" => {
            tauri::async_runtime::spawn(async move {
                loops::volatility::update_volatility_loop(s).await;
            })
        }
        "nexus" => {
            tauri::async_runtime::spawn(async move {
                loops::nexus_loop::nexus_loop(s, h).await;
            })
        }
        "daily_reset" => {
            tauri::async_runtime::spawn(async move {
                loops::daily_reset::daily_reset_loop(s, h).await;
            })
        }
        "genetic_evo" => {
            tauri::async_runtime::spawn(async move {
                loops::genetic_loop::genetic_evolution_loop(s, h).await;
            })
        }
        "trade_updates" => {
            tauri::async_runtime::spawn(async move {
                loops::trade_updates::trade_updates_stream(s, h).await;
            })
        }
        "position_recon" => {
            tauri::async_runtime::spawn(async move {
                loops::position_recon::position_reconciliation_loop(s, h).await;
            })
        }
        "equity_sync" => {
            tauri::async_runtime::spawn(async move {
                loops::equity_sync::equity_sync_loop(s, h).await;
            })
        }
        "news" => {
            tauri::async_runtime::spawn(async move {
                loops::news_loop::news_poll_loop(s, h).await;
            })
        }
        "catalyst_watchdog" => {
            tauri::async_runtime::spawn(async move {
                sentiment::catalyst_watchdog_loop(s, h).await;
            })
        }
        "ai_scanner" => {
            tauri::async_runtime::spawn(async move {
                loops::ai_scan_loop::ai_scan_loop(s, h).await;
            })
        }
        "smarttrend_sentinel" => {
            tauri::async_runtime::spawn(async move {
                loops::smarttrend_sentinel::smarttrend_sentinel_loop(s, h).await;
            })
        }
        "smarttrend_bot" => {
            tauri::async_runtime::spawn(async move {
                loops::smarttrend_bot::smarttrend_bot_loop(s, h).await;
            })
        }
        "streamer" => {
            let (new_tx, new_rx) = tokio::sync::mpsc::channel::<crate::streamer::StreamCmd>(256);
            {
                let mut tx_guard = s.stream_tx.lock().unwrap_or_else(|e| e.into_inner());
                *tx_guard = Some(new_tx);
            }
            tauri::async_runtime::spawn(async move {
                crate::streamer::run_streamer(s, new_rx, h).await;
            })
        }
        #[cfg(feature = "telegram")]
        "telegram" => {
            tauri::async_runtime::spawn(async move {
                crate::telegram::telegram_command_loop(s).await;
            })
        }
        _ => {
            tracing::warn!(component = "watchdog", loop_name = name, "No spawner registered for loop");
            return false;
        }
    };
    state.watchdog.abort_and_replace(name, handle.inner().abort_handle());
    true
}

pub(crate) async fn watchdog_supervisor(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }

        let health = state.watchdog.snapshot();
        let _ = app.emit_all("watchdog-health", serde_json::json!({
            "loops": &health,
            "generation": state.current_generation(),
            "generation_mismatches": state.generation_mismatch_count.load(std::sync::atomic::Ordering::Relaxed),
        }));

        let dead = state.watchdog.dead_loops();
        if dead.is_empty() {
            continue;
        }

        let msg = format!("WATCHDOG ALERT: Dead loops detected: {}", dead.join(", "));
        tracing::error!(component = "watchdog", "{}", msg);
        let _ = app.emit_all("log-message", &msg);
        let _ = app.emit_all(
            "watchdog-alert",
            serde_json::json!({ "dead_loops": &dead }),
        );

        for name in &dead {
            if !state.watchdog.can_respawn(name) {
                let count = state.watchdog.respawn_count(name);
                tracing::warn!(
                    component = "watchdog",
                    loop_name = %name,
                    respawn_count = count,
                    "Respawn skipped (cooldown or max attempts reached)"
                );
                continue;
            }

            if spawn_loop_by_name(name, &state, &app) {
                state.watchdog.record_respawn(name);
                let count = state.watchdog.respawn_count(name);
                let recovery_msg = format!(
                    "WATCHDOG RECOVERY: Respawned '{}' (attempt {}/3)",
                    name, count
                );
                tracing::warn!(component = "watchdog", "{}", recovery_msg);
                let _ = app.emit_all("log-message", &recovery_msg);
                let _ = app.emit_all(
                    "watchdog-recovery",
                    serde_json::json!({
                        "loop": name,
                        "attempt": count,
                    }),
                );
                let _ = state.ws_broadcast.send(serde_json::json!({
                    "type": "alert",
                    "message": format!("Watchdog recovered '{}'  (attempt {})", name, count),
                }).to_string());
            }
        }
    }
}

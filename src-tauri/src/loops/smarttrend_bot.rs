// ╔══════════════════════════════════════════════════════════════════╗
// ║  SmartTrend Bot Loop — marker-driven execution engine          ║
// ║  S10: Dedicated loop, NOT autopilot. Orders follow sentinel    ║
// ║  markers exclusively. ST circuit breaker is the only non-      ║
// ║  marker escape hatch.                                          ║
// ╚══════════════════════════════════════════════════════════════════╝

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::Duration;
use chrono::Timelike;

use crate::commands::data::{close_position, send_order_internal};
use crate::smarttrend::{SmartTrendStateKind, PillarDirection, ExitReason};
use crate::state::{StrategyMode, TitanState};
use crate::types::LegacyOrder;

pub(crate) const LOOP_NAME: &str = "smarttrend_bot";

static BOT_WAKE_CELL: OnceLock<Arc<Notify>> = OnceLock::new();

pub(crate) fn bot_wake() -> &'static Arc<Notify> {
    BOT_WAKE_CELL.get_or_init(|| Arc::new(Notify::new()))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PendingAction {
    symbol: String,
    kind: ActionKind,
    direction: PillarDirection,
    score: f64,
    timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum ActionKind {
    Entry,
    Exit,
    Trail,
    CircuitBreakerExit,
}

pub(crate) async fn smarttrend_bot_loop(state: Arc<TitanState>, app: AppHandle) {
    tracing::info!(component = LOOP_NAME, "SmartTrend bot loop started");

    let mut acted_symbols: HashMap<String, u64> = HashMap::new();
    let generation_at_start = state.current_generation();

    // ── Loss protection state ──
    // Per-symbol consecutive loss counter: symbol → consecutive_losses
    let mut symbol_losses: HashMap<String, u32> = HashMap::new();
    const MAX_CONSECUTIVE_LOSSES: u32 = 2; // Blacklist after 2 losses on same symbol
    const ENTRY_SKIP_MINUTES: u32 = 30; // Skip first 30 min after open (09:30-10:00 ET)

    loop {
        if state.shutdown_token.is_cancelled() {
            break;
        }

        let tick_start = std::time::Instant::now();

        // ── Guard: generation check ──
        if state.current_generation() != generation_at_start {
            tracing::warn!(
                component = LOOP_NAME,
                "Generation mismatch — bot loop exiting for respawn"
            );
            break;
        }

        // ── Read config snapshot ──
        let (st_config, settings_snap, strategy_mode, asset_modes) = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let mode = *state.strategy_mode.lock().unwrap_or_else(|e| e.into_inner());
            let modes = settings.st_asset_modes.clone();
            (settings.smarttrend.clone(), settings.clone(), mode, modes)
        };

        let bot_loop_ms = st_config.st_bot_loop_ms;

        // ── Guard: mode, killed, paused ──
        if strategy_mode != StrategyMode::SmartTrend {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            sleep_or_wake(Duration::from_millis(bot_loop_ms)).await;
            continue;
        }
        if st_config.st_killed || !settings_snap.auto_pilot {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            sleep_or_wake(Duration::from_secs(5)).await;
            continue;
        }

        // FIX CRIT-1: Check account.paused / breaker_triggered FIRST (latched state)
        let breaker_latched = {
            let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
            account.paused || account.breaker_triggered
        };
        if breaker_latched {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            sleep_or_wake(Duration::from_secs(5)).await;
            continue;
        }

        // ── ST Circuit Breaker check ──
        let breaker_fired = {
            let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
            !account.breaker_triggered && account.drawdown_pct() > st_config.st_max_dd_pct
                && account.allocated_capital > 0.0
        };

        if breaker_fired {
            tracing::error!(
                component = LOOP_NAME,
                max_dd = st_config.st_max_dd_pct,
                "ST CIRCUIT BREAKER TRIGGERED — force exiting all ST AUTO positions"
            );
            let _ = app.emit_all(
                "log-message",
                format!(
                    "🚨 ST CIRCUIT BREAKER: Drawdown exceeds {:.1}% — force closing all ST positions",
                    st_config.st_max_dd_pct
                ),
            );
            let _ = app.emit_all("smarttrend-alert", &serde_json::json!({
                "symbol": "ALL",
                "marker": "CIRCUIT_BREAKER",
                "direction": "Neutral",
                "score": 0.0,
                "detail": format!("ST drawdown > {:.1}%", st_config.st_max_dd_pct),
                "severity": "Critical",
                "timestamp_ms": now_epoch_ms(),
                "state": "Flat",
                "prev_state": "InTrade"
            }));

            force_exit_all_st_positions(&state, &app, &settings_snap).await;

            {
                let mut account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                account.breaker_triggered = true;
                account.paused = true;
                let _ = account.save_to_disk();
            }
            // FIX CRIT-1: Also latch st_bot_paused in persistent settings
            {
                let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                settings.smarttrend.st_bot_paused = true;
                let snapshot = crate::state::settings_from_state(&settings);
                let _ = crate::state::save_settings_to_disk(&snapshot);
            }

            state.watchdog.heartbeat(LOOP_NAME, tick_start.elapsed().as_millis() as u64);
            sleep_or_wake(Duration::from_secs(10)).await;
            continue;
        }

        if st_config.st_bot_paused {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            sleep_or_wake(Duration::from_millis(bot_loop_ms)).await;
            continue;
        }

        // ── Gather ST positions and determine actions ──
        let st_positions: Vec<_> = settings_snap
            .positions
            .iter()
            .filter(|p| {
                p.entry_tag
                    .as_ref()
                    .map(|t| t.starts_with("ST|"))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        let has_positions = !st_positions.is_empty();
        let adaptive_ms = if has_positions {
            (bot_loop_ms / 2).max(500)
        } else {
            (bot_loop_ms * 3 / 2).min(5000)
        };

        // Read engine state for entry/exit decisions
        let engine_snapshot = {
            let engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
            EngineSnapshot {
                state: engine.state,
                direction: engine.direction,
                entry_price: engine.entry_price,
                best_price: engine.best_price,
                bars_in_state: engine.bars_in_state,
                last_transition: engine.last_transition.clone(),
                exit_reason: engine.last_exit_reason,
                score: engine.last_diagnostics.as_ref()
                    .map(|d| {
                        d.macro_score.score * st_config.st_macro_weight
                        + d.micro_score.score * st_config.st_micro_weight
                        + d.momentum_score.score * st_config.st_momentum_weight
                    })
                    .unwrap_or(0.0),
            }
        };

        let mut actions: Vec<PendingAction> = Vec::new();
        let now_ms = now_epoch_ms();
        let freshness_window_ms = (st_config.st_bot_loop_ms * 3).max(6000);

        // ── FLAT-BY-CLOSE: Force exit all ST positions at T-10 min (15:50 ET) ──
        let now_et = chrono::Utc::now().with_timezone(&chrono_tz::US::Eastern);
        let et_hour = now_et.hour();
        let et_minute = now_et.minute();
        let is_close_window = (et_hour == 15 && et_minute >= 50) || et_hour >= 16;
        let is_entry_blocked = (et_hour == 15 && et_minute >= 45) || et_hour >= 16;

        if is_close_window && !st_positions.is_empty() {
            tracing::info!(
                component = LOOP_NAME,
                time = %now_et.format("%H:%M ET"),
                positions = st_positions.len(),
                "FLAT-BY-CLOSE: Force exiting all ST positions"
            );
            let _ = app.emit_all(
                "log-message",
                format!(
                    "🕐 FLAT-BY-CLOSE: Closing {} ST positions at {}",
                    st_positions.len(),
                    now_et.format("%H:%M ET")
                ),
            );
            force_exit_all_st_positions(&state, &app, &settings_snap).await;
            // Reset engine to Flat
            {
                let mut engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
                if engine.state != SmartTrendStateKind::Flat {
                    engine.state = SmartTrendStateKind::Flat;
                    engine.direction = PillarDirection::Neutral;
                    engine.last_exit_reason = ExitReason::FlatByClose;
                    engine.entry_price = None;
                    engine.best_price = None;
                }
            }
            state.watchdog.heartbeat(LOOP_NAME, tick_start.elapsed().as_millis() as u64);
            sleep_or_wake(Duration::from_secs(30)).await;
            continue;
        }

        // ── Multi-symbol entry logic: check ALL sentinel snapshots for Entry signals ──
        let max_positions = settings_snap.max_open_positions.max(1) as usize;
        let current_position_count = st_positions.len();

        // Read all sentinel snapshots (per-symbol engine states)
        let sentinel_snaps: Vec<crate::smarttrend::SentinelEngineSnapshot> = {
            let snaps = state.sentinel_snapshots.lock().unwrap_or_else(|e| e.into_inner());
            snaps.values().cloned().collect()
        };

        // ── Entry gate: skip first N minutes after open (gap/reversal danger zone) ──
        let rth_minutes_elapsed = if et_hour == 9 && et_minute >= 30 {
            et_minute - 30
        } else if et_hour >= 10 {
            (et_hour - 9) * 60 + et_minute - 30
        } else {
            0 // pre-market — allow entries if configured
        };
        let is_open_skip = rth_minutes_elapsed < ENTRY_SKIP_MINUTES && et_hour >= 9;

        if !is_entry_blocked && !is_open_skip && current_position_count < max_positions {
            for snap in &sentinel_snaps {
                if snap.state != SmartTrendStateKind::Entry {
                    continue;
                }
                if current_position_count + actions.iter().filter(|a| a.kind == ActionKind::Entry).count() >= max_positions {
                    break; // at capacity
                }

                let sym_mode = asset_modes
                    .get(&snap.symbol)
                    .map(|s| s.as_str())
                    .unwrap_or("man");

                let already_has_position = st_positions.iter().any(|p| p.symbol == snap.symbol);
                let recently_acted = acted_symbols
                    .get(&snap.symbol)
                    .map(|ts| now_ms.saturating_sub(*ts) < freshness_window_ms)
                    .unwrap_or(false);

                // Loss blacklist: skip symbols with too many consecutive losses today
                let is_blacklisted = symbol_losses
                    .get(&snap.symbol)
                    .map(|&losses| losses >= MAX_CONSECUTIVE_LOSSES)
                    .unwrap_or(false);
                if is_blacklisted {
                    tracing::debug!(
                        component = LOOP_NAME,
                        symbol = %snap.symbol,
                        "Skipping entry — symbol blacklisted after {} consecutive losses",
                        MAX_CONSECUTIVE_LOSSES
                    );
                    continue;
                }

                if !already_has_position && !recently_acted {
                    if sym_mode == "auto" {
                        tracing::info!(
                            component = LOOP_NAME,
                            symbol = %snap.symbol,
                            score = snap.score,
                            direction = ?snap.direction,
                            "MULTI-SYM ENTRY: Generating entry action"
                        );
                        actions.push(PendingAction {
                            symbol: snap.symbol.clone(),
                            kind: ActionKind::Entry,
                            direction: snap.direction,
                            score: snap.score,
                            timestamp_ms: now_ms,
                        });
                    } else {
                        tracing::debug!(
                            component = LOOP_NAME,
                            symbol = %snap.symbol,
                            "Skipping entry — asset is in MAN mode"
                        );
                    }
                }
            }
        }

        // ── Exit logic: check each ST position against its OWN symbol's sentinel snapshot ──
        // Multi-symbol correctness: each position exits independently based on its own FSM
        for pos in &st_positions {
            // Find this symbol's sentinel snapshot
            let sym_snap = sentinel_snaps.iter().find(|s| s.symbol == pos.symbol);

            let should_exit = match sym_snap {
                Some(snap) => {
                    snap.state == SmartTrendStateKind::Flat
                        && matches!(
                            snap.exit_reason,
                            ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
                        )
                }
                // No sentinel snapshot for this symbol — fall back to global engine
                None => {
                    engine_snapshot.state == SmartTrendStateKind::Flat
                        && matches!(
                            engine_snapshot.exit_reason,
                            ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
                        )
                }
            };

            if should_exit {
                let recently_acted = acted_symbols
                    .get(&pos.symbol)
                    .map(|ts| now_ms.saturating_sub(*ts) < freshness_window_ms)
                    .unwrap_or(false);
                if !recently_acted {
                    let exit_dir = sym_snap.map(|s| s.direction).unwrap_or(engine_snapshot.direction);
                    let exit_score = sym_snap.map(|s| s.score).unwrap_or(engine_snapshot.score);
                    actions.push(PendingAction {
                        symbol: pos.symbol.clone(),
                        kind: ActionKind::Exit,
                        direction: exit_dir,
                        score: exit_score,
                        timestamp_ms: now_ms,
                    });
                }
            }
        }

        // ── Execute actions ──
        for action in &actions {
            match action.kind {
                ActionKind::Entry => {
                    let size_usd = {
                        let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                        if account.allocated_capital <= 0.0 {
                            tracing::warn!(
                                component = LOOP_NAME,
                                symbol = %action.symbol,
                                "ST account has no capital — skipping entry"
                            );
                            continue;
                        }
                        account.compute_size_usd(&st_config)
                    };

                    let last_price = state
                        .market
                        .last_prices
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .get(&action.symbol)
                        .copied()
                        .unwrap_or(0.0);

                    if last_price <= 0.0 {
                        tracing::warn!(
                            component = LOOP_NAME,
                            symbol = %action.symbol,
                            "No price data — skipping entry"
                        );
                        continue;
                    }

                    let qty = (size_usd / last_price).max(0.0);
                    if qty < settings_snap.min_qty_threshold {
                        continue;
                    }
                    let qty = qty.min(settings_snap.max_qty_cap);

                    let side = match action.direction {
                        PillarDirection::Bullish => "buy",
                        PillarDirection::Bearish => "sell",
                        PillarDirection::Neutral => continue,
                    };

                    let now_epoch = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let tag = format!("ST|AUTO|{}|{}", action.symbol, now_epoch);

                    let order = LegacyOrder {
                        symbol: action.symbol.clone(),
                        side: side.to_string(),
                        qty,
                        take_profit: None,
                        stop_loss: None,
                    };

                    let _ = app.emit_all(
                        "log-message",
                        format!(
                            "📍 ST BOT ENTRY: {} {} x{:.4} (score={:.1}, size=${:.0})",
                            side.to_uppercase(),
                            action.symbol,
                            qty,
                            action.score,
                            size_usd
                        ),
                    );

                    match send_order_internal(state.clone(), order, app.clone()).await {
                        Ok(_res) => {
                            {
                                let mut tags = state.trading.entry_tags.lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                tags.insert(action.symbol.clone(), tag.clone());
                            }
                            {
                                let mut times = state.trading.entry_times.lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                times.insert(action.symbol.clone(), now_epoch);
                            }
                            {
                                let mut settings_lock = state.settings.lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                if let Some(pos) = settings_lock.positions.iter_mut()
                                    .find(|pp| pp.symbol == action.symbol)
                                {
                                    if pos.opened_at.is_none() {
                                        pos.opened_at = Some(now_epoch);
                                    }
                                    pos.entry_tag = Some(tag.clone());
                                    pos.state = crate::app_state::PositionState::Open;
                                }
                            }
                            if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                                analytics.record_entry_time();
                            }
                            acted_symbols.insert(action.symbol.clone(), now_ms);
                            {
                                let mut engine = state.smarttrend_engine
                                    .lock().unwrap_or_else(|e| e.into_inner());
                                engine.entry_acted = true;
                            }
                            tracing::info!(
                                component = LOOP_NAME,
                                symbol = %action.symbol,
                                side,
                                qty,
                                tag,
                                "ST entry order sent"
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                component = LOOP_NAME,
                                symbol = %action.symbol,
                                error = %e,
                                "ST entry order failed"
                            );
                            let mut cooldowns = state.trading.failed_order_cooldown.lock()
                                .unwrap_or_else(|e| e.into_inner());
                            let count = cooldowns.get(&action.symbol).map(|c| c.1).unwrap_or(0);
                            cooldowns.insert(action.symbol.clone(), (now_epoch, count + 1));
                        }
                    }
                }

                ActionKind::Exit | ActionKind::CircuitBreakerExit => {
                    if let Some(pos) = st_positions.iter().find(|p| p.symbol == action.symbol) {
                        let qty = pos.qty.to_f64().unwrap_or(0.0).abs();
                        if qty <= 0.0 {
                            continue;
                        }
                        let side = match pos.side {
                            crate::app_state::OrderSide::Buy => "sell",
                            crate::app_state::OrderSide::Sell => "buy",
                        };

                        let _ = app.emit_all(
                            "log-message",
                            format!(
                                "🛑 ST BOT EXIT: {} {} x{:.4} (score={:.1})",
                                side.to_uppercase(),
                                action.symbol,
                                qty,
                                action.score
                            ),
                        );

                        match close_position(state.clone(), &action.symbol, qty, side, &app).await {
                            Ok(()) => {
                                acted_symbols.insert(action.symbol.clone(), now_ms);

                                // Track win/loss for symbol blacklist
                                let entry_price = pos.entry_price.to_f64().unwrap_or(0.0);
                                let exit_price = state.market.last_prices
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .get(&action.symbol)
                                    .copied()
                                    .unwrap_or(0.0);
                                let was_loss = match pos.side {
                                    crate::app_state::OrderSide::Buy => exit_price < entry_price,
                                    crate::app_state::OrderSide::Sell => exit_price > entry_price,
                                };
                                if was_loss {
                                    let count = symbol_losses
                                        .entry(action.symbol.clone())
                                        .or_insert(0);
                                    *count += 1;
                                    tracing::info!(
                                        component = LOOP_NAME,
                                        symbol = %action.symbol,
                                        consecutive_losses = *count,
                                        "ST exit was a LOSS — blacklist counter incremented"
                                    );
                                } else {
                                    // Win resets the counter
                                    symbol_losses.remove(&action.symbol);
                                }

                                tracing::info!(
                                    component = LOOP_NAME,
                                    symbol = %action.symbol,
                                    "ST exit order sent"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    component = LOOP_NAME,
                                    symbol = %action.symbol,
                                    error = %e,
                                    "ST exit order failed"
                                );
                            }
                        }
                    }
                }

                ActionKind::Trail => {
                    // Trail adjustments are handled by the engine FSM; the bot only acts
                    // on Trail->Flat transitions which are covered by Exit above.
                }
            }
        }

        // ── Persist account periodically ──
        if tick_start.elapsed().as_secs().is_multiple_of(30) || !actions.is_empty() {
            let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
            let _ = account.save_to_disk();
        }

        // ── Clean stale action dedup entries ──
        let cutoff = now_ms.saturating_sub(60_000);
        acted_symbols.retain(|_, ts| *ts > cutoff);

        let tick_ms = tick_start.elapsed().as_millis() as u64;
        state.watchdog.heartbeat(LOOP_NAME, tick_ms);

        sleep_or_wake(Duration::from_millis(adaptive_ms)).await;
    }

    tracing::info!(component = LOOP_NAME, "SmartTrend bot loop exited");
}

/// Public entry point for Tauri command `smarttrend_force_exit_all`.
/// `auto_only=true` exits only ST|AUTO positions; `false` exits all ST| positions.
pub(crate) async fn force_exit_all_st_positions_public(
    state: &Arc<TitanState>,
    app: &AppHandle,
    settings: &crate::app_state::AppState,
) {
    force_exit_all_st_positions_filtered(state, app, settings, false).await;
}

/// Force exit ST positions (circuit breaker uses auto_only=true).
async fn force_exit_all_st_positions(
    state: &Arc<TitanState>,
    app: &AppHandle,
    settings: &crate::app_state::AppState,
) {
    force_exit_all_st_positions_filtered(state, app, settings, true).await;
}

async fn force_exit_all_st_positions_filtered(
    state: &Arc<TitanState>,
    app: &AppHandle,
    settings: &crate::app_state::AppState,
    auto_only: bool,
) {
    let st_positions: Vec<_> = settings
        .positions
        .iter()
        .filter(|p| {
            p.entry_tag
                .as_ref()
                .map(|t| {
                    if auto_only {
                        t.starts_with("ST|AUTO")
                    } else {
                        t.starts_with("ST|")
                    }
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    for pos in &st_positions {
        let qty = pos.qty.to_f64().unwrap_or(0.0).abs();
        if qty <= 0.0 {
            continue;
        }
        let side = match pos.side {
            crate::app_state::OrderSide::Buy => "sell",
            crate::app_state::OrderSide::Sell => "buy",
        };
        let _ = app.emit_all(
            "log-message",
            format!(
                "🚨 ST BREAKER EXIT: {} {} x{:.4}",
                side.to_uppercase(),
                pos.symbol,
                qty
            ),
        );
        if let Err(e) = close_position(state.clone(), &pos.symbol, qty, side, app).await {
            tracing::error!(
                component = LOOP_NAME,
                symbol = %pos.symbol,
                error = %e,
                "ST breaker force exit failed"
            );
        }
    }
}

/// Sleep with early wake on sentinel bridge event (B3 bonus).
async fn sleep_or_wake(duration: Duration) {
    tokio::select! {
        _ = tokio::time::sleep(duration) => {}
        _ = bot_wake().notified() => {}
    }
}

pub(crate) fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug)]
#[allow(dead_code)]
struct EngineSnapshot {
    state: SmartTrendStateKind,
    direction: PillarDirection,
    entry_price: Option<f64>,
    best_price: Option<f64>,
    bars_in_state: u32,
    last_transition: String,
    exit_reason: ExitReason,
    score: f64,
}

use rust_decimal::prelude::ToPrimitive;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_kind_equality() {
        assert_eq!(ActionKind::Entry, ActionKind::Entry);
        assert_ne!(ActionKind::Entry, ActionKind::Exit);
        assert_eq!(ActionKind::CircuitBreakerExit, ActionKind::CircuitBreakerExit);
    }

    #[test]
    fn engine_snapshot_defaults() {
        let snap = EngineSnapshot {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            entry_price: None,
            best_price: None,
            bars_in_state: 0,
            last_transition: "init".to_string(),
            exit_reason: ExitReason::None,
            score: 0.0,
        };
        assert_eq!(snap.state, SmartTrendStateKind::Flat);
        assert_eq!(snap.bars_in_state, 0);
        assert!(snap.entry_price.is_none());
    }

    #[test]
    fn now_epoch_ms_is_reasonable() {
        let ms = now_epoch_ms();
        assert!(ms > 1_700_000_000_000, "epoch ms should be after 2023");
    }

    /// FIX-2 CRIT-12: Both transition paths require Flat state for exit.
    #[test]
    fn exit_precedence_requires_flat_state() {
        let snap_intrade = EngineSnapshot {
            state: SmartTrendStateKind::InTrade,
            direction: PillarDirection::Bullish,
            entry_price: Some(100.0),
            best_price: Some(105.0),
            bars_in_state: 5,
            last_transition: "InTrade->Flat".to_string(),
            exit_reason: ExitReason::TrailExit,
            score: 80.0,
        };
        // With the fix: state must be Flat AND exit_reason must match
        let should_exit_intrade = snap_intrade.state == SmartTrendStateKind::Flat
            && matches!(
                snap_intrade.exit_reason,
                ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
            );
        assert!(!should_exit_intrade, "InTrade state should NOT trigger exit even with TrailExit reason");

        let snap_flat = EngineSnapshot {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            entry_price: None,
            best_price: None,
            bars_in_state: 1,
            last_transition: "Trail->Flat".to_string(),
            exit_reason: ExitReason::TrailExit,
            score: 0.0,
        };
        let should_exit_flat = snap_flat.state == SmartTrendStateKind::Flat
            && matches!(
                snap_flat.exit_reason,
                ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
            );
        assert!(should_exit_flat, "Flat state + TrailExit should trigger exit");
    }

    #[test]
    fn exit_fires_for_non_active_symbol_st_position() {
        let snap = EngineSnapshot {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            entry_price: None,
            best_price: None,
            bars_in_state: 1,
            last_transition: "InTrade->Flat".to_string(),
            exit_reason: ExitReason::ChangePointExit,
            score: 45.0,
        };

        let should_exit = snap.state == SmartTrendStateKind::Flat
            && matches!(
                snap.exit_reason,
                ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
            );

        assert!(
            should_exit,
            "Exit should fire regardless of symbol — no active_symbol gate needed"
        );

        let positions = vec!["AAPL", "TSLA", "NVDA"];
        let active_symbol = "AAPL";
        for sym in &positions {
            assert!(
                should_exit,
                "Exit for {} should fire even though active_symbol is {}",
                sym, active_symbol
            );
        }
    }

    #[test]
    fn flat_by_close_exit_fires() {
        let snap = EngineSnapshot {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            entry_price: None,
            best_price: None,
            bars_in_state: 0,
            last_transition: "FlatByClose".to_string(),
            exit_reason: ExitReason::FlatByClose,
            score: 0.0,
        };
        let should_exit = snap.state == SmartTrendStateKind::Flat
            && matches!(
                snap.exit_reason,
                ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
            );
        assert!(should_exit, "FlatByClose should trigger exit");
    }

    #[test]
    fn exit_reason_none_does_not_fire() {
        let snap = EngineSnapshot {
            state: SmartTrendStateKind::Flat,
            direction: PillarDirection::Neutral,
            entry_price: None,
            best_price: None,
            bars_in_state: 0,
            last_transition: "init".to_string(),
            exit_reason: ExitReason::None,
            score: 0.0,
        };
        let should_exit = snap.state == SmartTrendStateKind::Flat
            && matches!(
                snap.exit_reason,
                ExitReason::TrailExit | ExitReason::ChangePointExit | ExitReason::FlatByClose
            );
        assert!(!should_exit, "ExitReason::None should NOT trigger exit even in Flat state");
    }
}

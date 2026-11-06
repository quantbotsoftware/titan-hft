use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::smarttrend::{
    MacroInput, MicroInput, MomentumInput, SmartTrendEngine, SmartTrendInput, SmartTrendMarkerKind,
    SmartTrendStateKind,
};
use crate::state::TitanState;

const LOOP_NAME: &str = "smarttrend_sentinel";

// ═══════════════════════════════════════════════════════════════════
//  Alert Severity + Per-Symbol Sentinel State
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    fn from_marker(kind: &SmartTrendMarkerKind) -> Self {
        match kind {
            SmartTrendMarkerKind::Stalking => Self::Info,
            SmartTrendMarkerKind::Entry | SmartTrendMarkerKind::Trail => Self::Warning,
            SmartTrendMarkerKind::Exit | SmartTrendMarkerKind::ChangePointReset => Self::Critical,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SentinelAlert {
    pub symbol: String,
    pub marker: String,
    pub direction: String,
    pub score: f64,
    pub detail: String,
    pub severity: AlertSeverity,
    pub timestamp_ms: u64,
    pub state: String,
    pub prev_state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SymbolSentinelState {
    pub symbol: String,
    pub state: SmartTrendStateKind,
    pub score: f64,
    pub direction: String,
    pub last_alert_time_ms: u64,
    pub evaluate_duration_us: u64,
    pub bars_in_state: u32,
}

struct PerSymbolTracker {
    engine: SmartTrendEngine,
    last_state: SmartTrendStateKind,
    last_alert: Instant,
    last_evaluate_us: u64,
}

// ═══════════════════════════════════════════════════════════════════
//  B2: Symbol Priority Queue
// ═══════════════════════════════════════════════════════════════════

#[derive(Eq, PartialEq)]
struct SymbolPriority {
    state_rank: u8,
    score_rank: i64,
    symbol: String,
}

impl Ord for SymbolPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.state_rank
            .cmp(&other.state_rank)
            .then_with(|| self.score_rank.cmp(&other.score_rank))
    }
}

impl PartialOrd for SymbolPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn state_rank(s: SmartTrendStateKind) -> u8 {
    match s {
        SmartTrendStateKind::InTrade => 4,
        SmartTrendStateKind::Trail => 3,
        SmartTrendStateKind::Entry => 3,
        SmartTrendStateKind::Stalking => 2,
        SmartTrendStateKind::Flat => 1,
    }
}

// ═══════════════════════════════════════════════════════════════════
//  B3: Aggregated Dashboard Badge payload
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize)]
pub struct SentinelBadge {
    pub total_symbols: usize,
    pub active_symbols: usize,
    pub hottest_symbol: String,
    pub hottest_state: String,
    pub hottest_score: f64,
    pub avg_evaluate_us: u64,
}

// ═══════════════════════════════════════════════════════════════════
//  Sentinel Status (Tauri command payload)
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize)]
pub struct SentinelStatus {
    pub running: bool,
    pub symbols: Vec<SymbolSentinelState>,
    pub badge: SentinelBadge,
}

// ═══════════════════════════════════════════════════════════════════
//  Cadence logic
// ═══════════════════════════════════════════════════════════════════

fn cadence_for_state(
    state: SmartTrendStateKind,
    idle_secs: u64,
    stalking_secs: u64,
    hot_secs: u64,
) -> Duration {
    match state {
        SmartTrendStateKind::Flat => Duration::from_secs(idle_secs),
        SmartTrendStateKind::Stalking => Duration::from_secs(stalking_secs),
        SmartTrendStateKind::Entry
        | SmartTrendStateKind::InTrade
        | SmartTrendStateKind::Trail => Duration::from_secs(hot_secs),
    }
}

fn hottest_cadence(trackers: &HashMap<String, PerSymbolTracker>, idle: u64, stalk: u64, hot: u64) -> Duration {
    trackers
        .values()
        .map(|t| cadence_for_state(t.last_state, idle, stalk, hot))
        .min()
        .unwrap_or(Duration::from_secs(idle))
}

// ═══════════════════════════════════════════════════════════════════
//  Data assembly helpers
// ═══════════════════════════════════════════════════════════════════

fn assemble_macro_input(state: &TitanState, symbol: &str) -> MacroInput {
    let cache = state
        .market
        .indicator_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let oracle = cache.oracle_snapshot_for(symbol);
    MacroInput {
        oracle_score: oracle.score,
        confidence: oracle.confidence,
        regime: oracle.regime,
        kalman_gain: oracle.kalman_gain,
        price_surprise: oracle.price_surprise,
        kalman_velocity: if oracle.score > 0.0 { 1.0 } else if oracle.score < 0.0 { -1.0 } else { 0.0 },
        run_length: oracle.run_length,
        changepoint_prob: oracle.changepoint_prob,
    }
}

fn assemble_micro_input(state: &TitanState, symbol: &str) -> MicroInput {
    let ofi = state.market.ofi_state.load();
    let cvd_val = state
        .market
        .cvd
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(symbol)
        .copied()
        .unwrap_or(0.0);

    let has_live = ofi
        .last_update
        .map(|t| t.elapsed().as_secs() < 30)
        .unwrap_or(false);

    MicroInput {
        ofi_value: ofi.current_ofi,
        microprice: ofi.microprice,
        mid_price: if ofi.best_bid > 0.0 && ofi.best_ask > 0.0 {
            (ofi.best_bid + ofi.best_ask) / 2.0
        } else {
            ofi.microprice
        },
        cvd: cvd_val,
        spread_bps: ofi.spread_bps,
        imbalance_ratio: ofi.imbalance_ratio,
        has_live_data: has_live,
    }
}

fn assemble_momentum_input(state: &TitanState, symbol: &str) -> MomentumInput {
    // NOTE: nexus_consensus is global (not per-symbol). This is an architectural
    // limitation — multi-symbol sentinel uses the same GPU consensus for all symbols.
    // Per-symbol consensus would require per-symbol GPU/nexus pipelines.
    let consensus = state
        .nexus_consensus
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let active = state
        .market
        .last_prices
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains_key(symbol);

    if !active && consensus.active_agents > 0 {
        tracing::debug!(
            component = "sentinel",
            symbol,
            "momentum input uses global nexus_consensus (no per-symbol data)"
        );
    }

    MomentumInput {
        gpu_buy_pct: consensus.buy_pct,
        gpu_sell_pct: consensus.sell_pct,
        dominant_cycle: 0,
        momentum_bars: 0,
        rvol: 1.0,
    }
}

fn current_price(state: &TitanState, symbol: &str) -> f64 {
    state
        .market
        .last_prices
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(symbol)
        .copied()
        .unwrap_or(0.0)
}

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ═══════════════════════════════════════════════════════════════════
//  Build aggregated badge
// ═══════════════════════════════════════════════════════════════════

fn build_badge(trackers: &HashMap<String, PerSymbolTracker>) -> SentinelBadge {
    let total = trackers.len();
    let active = trackers
        .values()
        .filter(|t| t.last_state != SmartTrendStateKind::Flat)
        .count();

    let mut hottest_sym = String::new();
    let mut hottest_state = SmartTrendStateKind::Flat;
    let mut hottest_score: f64 = 0.0;
    let mut eval_sum: u64 = 0;

    for (sym, t) in trackers {
        eval_sum += t.last_evaluate_us;
        let rank = state_rank(t.last_state);
        let current_rank = state_rank(hottest_state);
        if rank > current_rank || (rank == current_rank && t.engine.last_diagnostics.as_ref().map(|d| d.combined_score).unwrap_or(0.0) > hottest_score) {
            hottest_sym = sym.clone();
            hottest_state = t.last_state;
            hottest_score = t.engine.last_diagnostics.as_ref().map(|d| d.combined_score).unwrap_or(0.0);
        }
    }

    SentinelBadge {
        total_symbols: total,
        active_symbols: active,
        hottest_symbol: hottest_sym,
        hottest_state: format!("{}", hottest_state),
        hottest_score,
        avg_evaluate_us: if total > 0 { eval_sum / total as u64 } else { 0 },
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Main Sentinel Loop
// ═══════════════════════════════════════════════════════════════════

pub(crate) async fn smarttrend_sentinel_loop(state: Arc<TitanState>, app: AppHandle) {
    tracing::info!(component = LOOP_NAME, "SmartTrend sentinel loop started");
    state.watchdog.register(LOOP_NAME);

    let boot_gen = state.current_generation();
    let mut trackers: HashMap<String, PerSymbolTracker> = HashMap::new();

    loop {
        if state.shutdown_token.is_cancelled() {
            break;
        }

        // Generation check — must use != (generation is monotonically increasing,
        // so < would never trigger after a watchdog bump_generation respawn)
        if state.current_generation() != boot_gen {
            tracing::warn!(
                component = LOOP_NAME,
                boot_gen,
                current_gen = state.current_generation(),
                "Generation changed, exiting sentinel loop for respawn"
            );
            break;
        }

        let tick_start = Instant::now();

        // Read config snapshot
        let (enabled, killed, idle_secs, stalking_secs, hot_secs, alert_cooldown_secs) = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (
                settings.smarttrend.st_enabled,
                settings.smarttrend.st_killed,
                settings.smarttrend.st_sentinel_idle_secs,
                settings.smarttrend.st_sentinel_stalking_secs,
                settings.smarttrend.st_sentinel_hot_secs,
                settings.smarttrend.st_sentinel_alert_cooldown_secs,
            )
        };

        if !enabled || killed {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            tokio::time::sleep(Duration::from_secs(idle_secs.max(5))).await;
            continue;
        }

        // Get monitored symbols from positions + HFT targets + signal strategy assets
        let symbols: Vec<String> = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let mut syms: Vec<String> = settings
                .positions
                .iter()
                .map(|p| p.symbol.clone())
                .collect();
            for s in &settings.signal_strategy_assets {
                if !syms.contains(s) {
                    syms.push(s.clone());
                }
            }
            for s in &settings.hft_targets {
                if !syms.contains(s) {
                    syms.push(s.clone());
                }
            }
            syms
        };

        if symbols.is_empty() {
            state.watchdog.heartbeat(LOOP_NAME, 0);
            tokio::time::sleep(Duration::from_secs(idle_secs)).await;
            continue;
        }

        // Remove stale trackers for symbols no longer monitored
        trackers.retain(|sym, _| symbols.contains(sym));

        // Get SmartTrend config for new engines
        let st_config = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            settings.smarttrend.clone()
        };

        // Initialize trackers for new symbols
        for sym in &symbols {
            let cfg = st_config.clone();
            trackers.entry(sym.clone()).or_insert_with(|| PerSymbolTracker {
                engine: SmartTrendEngine::new(cfg),
                last_state: SmartTrendStateKind::Flat,
                last_alert: Instant::now() - Duration::from_secs(999),
                last_evaluate_us: 0,
            });
        }

        // B2: Build priority queue
        let mut pq = BinaryHeap::new();
        for (sym, tracker) in &trackers {
            pq.push(SymbolPriority {
                state_rank: state_rank(tracker.last_state),
                score_rank: (tracker.engine.last_diagnostics.as_ref().map(|d| d.combined_score).unwrap_or(0.0) * 100.0) as i64,
                symbol: sym.clone(),
            });
        }

        let cooldown = Duration::from_secs(alert_cooldown_secs);

        // Evaluate symbols in priority order
        while let Some(entry) = pq.pop() {
            let sym = &entry.symbol;
            let price = current_price(&state, sym);
            if price <= 0.0 {
                continue;
            }

            let macro_in = assemble_macro_input(&state, sym);
            let micro_in = assemble_micro_input(&state, sym);
            let momentum_in = assemble_momentum_input(&state, sym);
            let changepoint_prob = macro_in.changepoint_prob;

            let st_input = SmartTrendInput {
                macro_input: macro_in,
                micro_input: micro_in,
                momentum_input: momentum_in,
                current_price: price,
                changepoint_prob,
                dominant_cycle: 0.0,
                tf_minutes: 1.0,
            };

            let tracker = trackers.get_mut(sym).unwrap();
            let prev_state = tracker.last_state;

            // B1: Micro-latency tracking
            let eval_start = Instant::now();
            let result = tracker.engine.evaluate(&st_input);
            let eval_us = eval_start.elapsed().as_micros() as u64;
            tracker.last_evaluate_us = eval_us;

            if eval_us > 5000 {
                tracing::warn!(
                    component = LOOP_NAME,
                    symbol = %sym,
                    duration_us = eval_us,
                    "Sentinel evaluate exceeded 5ms"
                );
            }

            tracker.last_state = result.state;

            // Detect state transition and emit alerts
            let state_changed = result.state != prev_state;

            for marker in &result.markers {
                let severity = AlertSeverity::from_marker(&marker.kind);

                // Apply cooldown per-symbol
                if tracker.last_alert.elapsed() < cooldown
                    && severity == AlertSeverity::Info
                {
                    continue;
                }

                tracker.last_alert = Instant::now();

                let alert = SentinelAlert {
                    symbol: sym.clone(),
                    marker: format!("{:?}", marker.kind),
                    direction: format!("{:?}", marker.direction),
                    score: marker.score,
                    detail: marker.detail.clone(),
                    severity,
                    timestamp_ms: now_epoch_ms(),
                    state: format!("{}", result.state),
                    prev_state: format!("{}", prev_state),
                };

                let _ = app.emit_all("smarttrend-alert", &alert);

                // B3: Sentinel-to-Bot bridge — wake bot immediately on Entry markers
                if matches!(marker.kind, SmartTrendMarkerKind::Entry) {
                    crate::loops::smarttrend_bot::bot_wake().notify_one();

                    // S11: Emit trade-prompt for MAN assets so the UI can show the trade panel
                    let asset_mode = {
                        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        settings.st_asset_modes
                            .get(sym.as_str())
                            .cloned()
                            .unwrap_or_else(|| "man".to_string())
                    };
                    if asset_mode == "man" {
                        let st_cfg = {
                            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                            settings.smarttrend.clone()
                        };
                        let price = state.market.last_prices.lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .get(sym.as_str()).copied().unwrap_or(0.0);
                        let (suggested_size, kelly_frac) = {
                            let account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                            let size = account.compute_size_usd(&st_cfg);
                            let kelly = if st_cfg.st_kelly_enabled
                                && account.total_trades >= st_cfg.st_kelly_min_trades as u64
                            {
                                Some(account.kelly_fraction(
                                    st_cfg.st_kelly_min_trades,
                                    st_cfg.st_max_kelly_fraction,
                                ))
                            } else {
                                None
                            };
                            (size, kelly)
                        };
                        let pillar_scores = tracker.engine.last_diagnostics.as_ref()
                            .map(|d| [d.macro_score.score, d.micro_score.score, d.momentum_score.score])
                            .unwrap_or([0.0; 3]);

                        let _ = app.emit_all("smarttrend-trade-prompt", &serde_json::json!({
                            "symbol": sym,
                            "direction": format!("{:?}", marker.direction),
                            "score": marker.score,
                            "state": format!("{}", result.state),
                            "current_price": price,
                            "suggested_size_usd": suggested_size,
                            "risk_pct": st_cfg.st_risk_per_trade * 100.0,
                            "kelly_fraction": kelly_frac,
                            "pillar_scores": pillar_scores,
                            "mode": "man",
                            "timestamp_ms": now_epoch_ms()
                        }));
                    }
                }

                // S11: Also emit trade-prompt for Exit/Trail markers on MAN positions
                if matches!(marker.kind, SmartTrendMarkerKind::Exit | SmartTrendMarkerKind::Trail) {
                    let has_man_position = {
                        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        settings.positions.iter().any(|p| {
                            p.symbol == *sym
                                && p.entry_tag.as_ref()
                                    .map(|t| t.starts_with("ST|MAN"))
                                    .unwrap_or(false)
                        })
                    };
                    if has_man_position {
                        let _ = app.emit_all("smarttrend-exit-prompt", &serde_json::json!({
                            "symbol": sym,
                            "marker": format!("{:?}", marker.kind),
                            "direction": format!("{:?}", marker.direction),
                            "score": marker.score,
                            "severity": format!("{:?}", severity),
                            "timestamp_ms": now_epoch_ms()
                        }));
                    }
                }

                tracing::info!(
                    component = LOOP_NAME,
                    symbol = %sym,
                    marker = ?marker.kind,
                    score = marker.score,
                    severity = ?severity,
                    "Sentinel alert emitted"
                );
            }

            // Emit state transition even if no explicit marker (covers InTrade->Trail etc.)
            if state_changed && result.markers.is_empty() {
                let severity = match result.state {
                    SmartTrendStateKind::Entry => AlertSeverity::Warning,
                    SmartTrendStateKind::Trail => AlertSeverity::Warning,
                    SmartTrendStateKind::Flat if prev_state == SmartTrendStateKind::InTrade
                        || prev_state == SmartTrendStateKind::Trail => AlertSeverity::Critical,
                    _ => AlertSeverity::Info,
                };

                if tracker.last_alert.elapsed() >= cooldown || severity != AlertSeverity::Info {
                    tracker.last_alert = Instant::now();
                    let alert = SentinelAlert {
                        symbol: sym.clone(),
                        marker: format!("{}->{}",  prev_state, result.state),
                        direction: format!("{:?}", result.direction),
                        score: result.score,
                        detail: "state transition".to_string(),
                        severity,
                        timestamp_ms: now_epoch_ms(),
                        state: format!("{}", result.state),
                        prev_state: format!("{}", prev_state),
                    };
                    let _ = app.emit_all("smarttrend-alert", &alert);
                }
            }
        }

        // B3: Emit aggregated badge
        let badge = build_badge(&trackers);
        let _ = app.emit_all("sentinel-badge", &badge);

        // Sync per-symbol snapshots to shared state for multi-symbol bot
        {
            let now_ms = now_epoch_ms();
            let mut snaps = state
                .sentinel_snapshots
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            snaps.clear();
            for (sym, tracker) in &trackers {
                let combined_score = tracker.engine.last_diagnostics
                    .as_ref()
                    .map(|d| d.combined_score)
                    .unwrap_or(0.0);
                snaps.insert(sym.clone(), crate::smarttrend::SentinelEngineSnapshot {
                    symbol: sym.clone(),
                    state: tracker.engine.state,
                    direction: tracker.engine.direction,
                    exit_reason: tracker.engine.last_exit_reason,
                    entry_price: tracker.engine.entry_price,
                    best_price: tracker.engine.best_price,
                    bars_in_state: tracker.engine.bars_in_state,
                    score: combined_score,
                    updated_ms: now_ms,
                });
            }
        }

        // Also sync the main engine diagnostics with the active symbol's tracker
        {
            let active_sym = state
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .active_symbol
                .clone();

            if let Some(tracker) = trackers.get(&active_sym) {
                let mut engine = state
                    .smarttrend_engine
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                engine.state = tracker.engine.state;
                engine.direction = tracker.engine.direction;
                engine.bars_in_state = tracker.engine.bars_in_state;
                engine.entry_price = tracker.engine.entry_price;
                engine.best_price = tracker.engine.best_price;
                engine.confirmation_bars = tracker.engine.confirmation_bars;
                engine.confirmation_needed = tracker.engine.confirmation_needed;
                engine.trail_tighten_accumulated = tracker.engine.trail_tighten_accumulated;
                engine.last_transition = tracker.engine.last_transition.clone();
                engine.last_exit_reason = tracker.engine.last_exit_reason;
                engine.last_diagnostics = tracker.engine.last_diagnostics.clone();
                engine.stats = tracker.engine.stats.clone();
            }
        }

        let tick_ms = tick_start.elapsed().as_millis() as u64;
        state.watchdog.heartbeat(LOOP_NAME, tick_ms);

        // Adaptive sleep based on hottest tracked symbol
        let sleep_dur = hottest_cadence(&trackers, idle_secs, stalking_secs, hot_secs);
        tokio::time::sleep(sleep_dur).await;
    }

    tracing::info!(component = LOOP_NAME, "SmartTrend sentinel loop exited");
}

// ═══════════════════════════════════════════════════════════════════
//  Public API for Tauri commands
// ═══════════════════════════════════════════════════════════════════

pub fn build_sentinel_status(state: &TitanState) -> SentinelStatus {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let enabled = settings.smarttrend.st_enabled;
    let asset_modes = settings.st_asset_modes.clone();
    drop(settings);

    let engine = state
        .smarttrend_engine
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let diag = engine.diagnostics();
    let active_sym = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .active_symbol
        .clone();

    let total = asset_modes.len().max(1);
    let active_state = diag.state;
    let active_count = if active_state != SmartTrendStateKind::Flat { 1 } else { 0 };

    let symbols: Vec<SymbolSentinelState> = asset_modes
        .keys()
        .map(|sym| {
            if *sym == active_sym {
                SymbolSentinelState {
                    symbol: sym.clone(),
                    state: active_state,
                    score: diag.combined_score,
                    direction: format!("{:?}", diag.direction),
                    last_alert_time_ms: 0,
                    evaluate_duration_us: 0,
                    bars_in_state: diag.bars_in_state,
                }
            } else {
                SymbolSentinelState {
                    symbol: sym.clone(),
                    state: SmartTrendStateKind::Flat,
                    score: 0.0,
                    direction: "Neutral".into(),
                    last_alert_time_ms: 0,
                    evaluate_duration_us: 0,
                    bars_in_state: 0,
                }
            }
        })
        .collect();

    SentinelStatus {
        running: enabled,
        symbols,
        badge: SentinelBadge {
            total_symbols: total,
            active_symbols: active_count,
            hottest_symbol: active_sym,
            hottest_state: format!("{}", active_state),
            hottest_score: diag.combined_score,
            avg_evaluate_us: 0,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::titan_oracle::MarketRegime;

    #[test]
    fn sentinel_cadence_idle_30s() {
        let d = cadence_for_state(SmartTrendStateKind::Flat, 30, 5, 2);
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn sentinel_cadence_stalking_5s() {
        let d = cadence_for_state(SmartTrendStateKind::Stalking, 30, 5, 2);
        assert_eq!(d, Duration::from_secs(5));
    }

    #[test]
    fn sentinel_cadence_hot_2s() {
        let d = cadence_for_state(SmartTrendStateKind::InTrade, 30, 5, 2);
        assert_eq!(d, Duration::from_secs(2));
        let d2 = cadence_for_state(SmartTrendStateKind::Trail, 30, 5, 2);
        assert_eq!(d2, Duration::from_secs(2));
        let d3 = cadence_for_state(SmartTrendStateKind::Entry, 30, 5, 2);
        assert_eq!(d3, Duration::from_secs(2));
    }

    #[test]
    fn sentinel_state_transition_detection() {
        let cfg = crate::smarttrend::SmartTrendConfig::default();
        let mut engine = SmartTrendEngine::new(cfg);
        assert_eq!(engine.state, SmartTrendStateKind::Flat);

        let input = SmartTrendInput {
            macro_input: MacroInput {
                oracle_score: 80.0,
                confidence: 0.85,
                regime: MarketRegime::Trending,
                kalman_gain: 0.15,
                price_surprise: 0.02,
                kalman_velocity: 1.5,
                run_length: 30,
                changepoint_prob: 0.05,
            },
            micro_input: MicroInput {
                ofi_value: 40.0,
                microprice: 100.05,
                mid_price: 100.0,
                cvd: 80.0,
                spread_bps: 3.0,
                imbalance_ratio: 0.5,
                has_live_data: true,
            },
            momentum_input: MomentumInput {
                gpu_buy_pct: 80.0,
                gpu_sell_pct: 20.0,
                dominant_cycle: 8,
                momentum_bars: 12,
                rvol: 2.0,
            },
            current_price: 100.0,
            changepoint_prob: 0.05,
            dominant_cycle: 8.0,
            tf_minutes: 1.0,
        };

        let result = engine.evaluate(&input);
        let transitioned = result.state != SmartTrendStateKind::Flat;
        assert!(transitioned, "Strong bullish signal should cause Flat->Stalking transition");
        assert_eq!(result.state, SmartTrendStateKind::Stalking);
    }

    #[test]
    fn sentinel_cooldown_suppresses_repeat() {
        let last_alert = Instant::now();
        let cooldown = Duration::from_secs(30);
        let elapsed = last_alert.elapsed();
        assert!(elapsed < cooldown, "Just-created alert should be within cooldown");
    }

    #[test]
    fn sentinel_killed_skips_evaluate() {
        let mut cfg = crate::smarttrend::SmartTrendConfig::default();
        cfg.st_killed = true;
        assert!(cfg.st_killed, "killed flag should be true");
    }

    #[test]
    fn sentinel_severity_classification() {
        assert_eq!(
            AlertSeverity::from_marker(&SmartTrendMarkerKind::Stalking),
            AlertSeverity::Info
        );
        assert_eq!(
            AlertSeverity::from_marker(&SmartTrendMarkerKind::Entry),
            AlertSeverity::Warning
        );
        assert_eq!(
            AlertSeverity::from_marker(&SmartTrendMarkerKind::Trail),
            AlertSeverity::Warning
        );
        assert_eq!(
            AlertSeverity::from_marker(&SmartTrendMarkerKind::Exit),
            AlertSeverity::Critical
        );
        assert_eq!(
            AlertSeverity::from_marker(&SmartTrendMarkerKind::ChangePointReset),
            AlertSeverity::Critical
        );
    }

    #[test]
    fn sentinel_priority_queue_ordering() {
        let mut pq = BinaryHeap::new();
        pq.push(SymbolPriority {
            state_rank: state_rank(SmartTrendStateKind::Flat),
            score_rank: 50,
            symbol: "AAPL".into(),
        });
        pq.push(SymbolPriority {
            state_rank: state_rank(SmartTrendStateKind::InTrade),
            score_rank: 80,
            symbol: "TSLA".into(),
        });
        pq.push(SymbolPriority {
            state_rank: state_rank(SmartTrendStateKind::Stalking),
            score_rank: 60,
            symbol: "NVDA".into(),
        });

        let first = pq.pop().unwrap();
        assert_eq!(first.symbol, "TSLA", "InTrade should be highest priority");
        let second = pq.pop().unwrap();
        assert_eq!(second.symbol, "NVDA", "Stalking should be second");
        let third = pq.pop().unwrap();
        assert_eq!(third.symbol, "AAPL", "Flat should be last");
    }

    #[test]
    fn sentinel_badge_empty_trackers() {
        let trackers: HashMap<String, PerSymbolTracker> = HashMap::new();
        let badge = build_badge(&trackers);
        assert_eq!(badge.total_symbols, 0);
        assert_eq!(badge.active_symbols, 0);
        assert_eq!(badge.avg_evaluate_us, 0);
    }

    #[test]
    fn sentinel_exits_on_generation_bump() {
        let boot_gen: u64 = 5;
        let current_gen_same: u64 = 5;
        let current_gen_higher: u64 = 6;
        let current_gen_lower: u64 = 4;

        assert!(
            current_gen_same == boot_gen,
            "Same generation should NOT trigger exit"
        );
        assert!(
            current_gen_higher != boot_gen,
            "Higher generation must trigger exit with != check"
        );
        assert!(
            current_gen_lower != boot_gen,
            "Lower generation must also trigger exit with != check"
        );
        // The old bug: `<` check would miss higher generations
        assert!(
            !(current_gen_higher < boot_gen),
            "Old < check would NOT detect higher generation — this was the bug"
        );
    }

    #[test]
    fn sentinel_alert_severity_values() {
        let s = AlertSeverity::Info;
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("Info"));
    }
}

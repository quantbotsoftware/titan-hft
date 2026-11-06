use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::task::AbortHandle;

const STALE_THRESHOLD_SECS: u64 = 120;
const RESPAWN_COOLDOWN_SECS: u64 = 120;
const MAX_RESPAWNS_PER_LOOP: u32 = 3;

#[derive(Debug, Clone, Serialize)]
pub struct LoopTelemetry {
    pub name: String,
    pub tick_count: u64,
    pub error_count: u64,
    pub last_tick_ms: u64,
    pub avg_tick_ms: f64,
    pub last_heartbeat_secs_ago: f64,
    pub is_alive: bool,
    pub uptime_secs: f64,
}

#[derive(Debug)]
struct LoopState {
    tick_count: u64,
    error_count: u64,
    last_heartbeat: Instant,
    last_tick_ms: u64,
    tick_sum_ms: u64,
    started_at: Instant,
}

#[derive(Debug)]
struct RespawnState {
    last_respawn: Option<Instant>,
    count: u32,
}

#[derive(Clone)]
pub struct Watchdog {
    loops: Arc<Mutex<HashMap<String, LoopState>>>,
    respawns: Arc<Mutex<HashMap<String, RespawnState>>>,
    handles: Arc<Mutex<HashMap<String, AbortHandle>>>,
}

impl std::fmt::Debug for Watchdog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handle_count = self.handles.lock().map(|h| h.len()).unwrap_or(0);
        f.debug_struct("Watchdog")
            .field("loops", &self.loops)
            .field("respawns", &self.respawns)
            .field("handles_count", &handle_count)
            .finish()
    }
}

impl Watchdog {
    pub fn new() -> Self {
        Self {
            loops: Arc::new(Mutex::new(HashMap::new())),
            respawns: Arc::new(Mutex::new(HashMap::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, name: &str) {
        let now = Instant::now();
        let mut map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(
            name.to_string(),
            LoopState {
                tick_count: 0,
                error_count: 0,
                last_heartbeat: now,
                last_tick_ms: 0,
                tick_sum_ms: 0,
                started_at: now,
            },
        );
    }

    /// Called by each loop at the end of every iteration.
    pub fn heartbeat(&self, name: &str, tick_duration_ms: u64) {
        let mut map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = map.get_mut(name) {
            state.tick_count += 1;
            state.last_heartbeat = Instant::now();
            state.last_tick_ms = tick_duration_ms;
            state.tick_sum_ms += tick_duration_ms;
        }
    }

    pub fn record_error(&self, name: &str) {
        let mut map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = map.get_mut(name) {
            state.error_count += 1;
        }
    }

    pub fn snapshot(&self) -> Vec<LoopTelemetry> {
        let map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
        let mut result: Vec<LoopTelemetry> = map
            .iter()
            .map(|(name, s)| {
                let ago = s.last_heartbeat.elapsed().as_secs_f64();
                let avg = if s.tick_count > 0 {
                    s.tick_sum_ms as f64 / s.tick_count as f64
                } else {
                    0.0
                };
                LoopTelemetry {
                    name: name.clone(),
                    tick_count: s.tick_count,
                    error_count: s.error_count,
                    last_tick_ms: s.last_tick_ms,
                    avg_tick_ms: avg,
                    last_heartbeat_secs_ago: ago,
                    is_alive: ago < STALE_THRESHOLD_SECS as f64,
                    uptime_secs: s.started_at.elapsed().as_secs_f64(),
                }
            })
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    pub fn dead_loops(&self) -> Vec<String> {
        let map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
        map.iter()
            .filter(|(_, s)| {
                s.last_heartbeat.elapsed().as_secs() > STALE_THRESHOLD_SECS
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Check whether a loop is eligible for respawn (cooldown + max cap).
    pub fn can_respawn(&self, name: &str) -> bool {
        let map = self.respawns.lock().unwrap_or_else(|e| e.into_inner());
        match map.get(name) {
            None => true,
            Some(rs) => {
                if rs.count >= MAX_RESPAWNS_PER_LOOP {
                    return false;
                }
                match rs.last_respawn {
                    None => true,
                    Some(t) => t.elapsed().as_secs() >= RESPAWN_COOLDOWN_SECS,
                }
            }
        }
    }

    /// Record that a respawn was performed for the named loop and reset its heartbeat.
    pub fn record_respawn(&self, name: &str) {
        {
            let mut map = self.respawns.lock().unwrap_or_else(|e| e.into_inner());
            let entry = map.entry(name.to_string()).or_insert(RespawnState {
                last_respawn: None,
                count: 0,
            });
            entry.last_respawn = Some(Instant::now());
            entry.count += 1;
        }
        {
            let now = Instant::now();
            let mut map = self.loops.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(state) = map.get_mut(name) {
                state.last_heartbeat = now;
                state.tick_count = 0;
                state.error_count = 0;
                state.last_tick_ms = 0;
                state.tick_sum_ms = 0;
                state.started_at = now;
            }
        }
    }

    pub fn respawn_count(&self, name: &str) -> u32 {
        let map = self.respawns.lock().unwrap_or_else(|e| e.into_inner());
        map.get(name).map(|rs| rs.count).unwrap_or(0)
    }

    /// Store an AbortHandle for a named loop. If one already exists, the old handle is NOT aborted
    /// (use `abort_and_replace` for that pattern).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn store_handle(&self, name: &str, handle: AbortHandle) {
        let mut map = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(name.to_string(), handle);
    }

    /// Abort the old task for a named loop (if running) and store the new handle.
    /// Returns true if an old handle was aborted.
    pub fn abort_and_replace(&self, name: &str, new_handle: AbortHandle) -> bool {
        let mut map = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        let had_old = if let Some(old) = map.remove(name) {
            old.abort();
            tracing::info!(
                component = "watchdog",
                loop_name = name,
                "Aborted old task handle before respawn"
            );
            true
        } else {
            false
        };
        map.insert(name.to_string(), new_handle);
        had_old
    }

    /// Abort a specific loop by name without replacing. Returns true if aborted.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn abort_loop(&self, name: &str) -> bool {
        let mut map = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(handle) = map.remove(name) {
            handle.abort();
            true
        } else {
            false
        }
    }
}

/// Session report persisted to disk at daily reset or shutdown.
#[derive(Debug, Clone, Serialize)]
pub struct SessionReport {
    pub date: String,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub daily_pnl: String,
    pub max_drawdown_pct: String,
    pub win_rate_pct: String,
    pub profit_factor: String,
    pub sharpe: f64,
    pub peak_equity: String,
    pub final_equity: String,
    pub champion_params: Option<serde_json::Value>,
    pub loop_telemetry: Vec<LoopTelemetry>,
    pub breaker_events: u32,
    pub dry_run: bool,
}

pub fn save_session_report(report: &SessionReport) {
    let reports_dir = crate::paths::reports_dir();
    let _ = std::fs::create_dir_all(&reports_dir);
    let filename = format!("{}/session_{}.json", reports_dir.display(), report.date);
    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&filename, &json) {
                tracing::error!(
                    component = "watchdog",
                    "Failed to write session report: {}",
                    e
                );
            } else {
                tracing::info!(component = "watchdog", file = %filename, "Session report saved");
            }
        }
        Err(e) => {
            tracing::error!(
                component = "watchdog",
                "Failed to serialize session report: {}",
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_heartbeat_and_snapshot() {
        let wd = Watchdog::new();
        wd.register("autopilot");
        wd.register("nexus");

        wd.heartbeat("autopilot", 15);
        wd.heartbeat("autopilot", 20);
        wd.heartbeat("nexus", 5);
        wd.record_error("nexus");

        let snap = wd.snapshot();
        assert_eq!(snap.len(), 2);

        let auto = snap
            .iter()
            .find(|s| s.name == "autopilot")
            .expect("watchdog snapshot must contain 'autopilot'");
        assert_eq!(auto.tick_count, 2);
        assert_eq!(auto.error_count, 0);
        assert!(auto.is_alive);
        assert!((auto.avg_tick_ms - 17.5).abs() < 0.01);

        let nexus = snap
            .iter()
            .find(|s| s.name == "nexus")
            .expect("watchdog snapshot must contain 'nexus'");
        assert_eq!(nexus.tick_count, 1);
        assert_eq!(nexus.error_count, 1);
    }

    #[test]
    fn test_dead_loops_empty_when_fresh() {
        let wd = Watchdog::new();
        wd.register("test_loop");
        wd.heartbeat("test_loop", 10);
        assert!(wd.dead_loops().is_empty());
    }

    #[test]
    fn test_session_report_serialization() {
        let report = SessionReport {
            date: "2026-03-03".into(),
            total_trades: 15u32,
            winning_trades: 10u32,
            losing_trades: 5u32,
            daily_pnl: "125.50".into(),
            max_drawdown_pct: "1.2".into(),
            win_rate_pct: "66.67".into(),
            profit_factor: "2.5".into(),
            sharpe: 1.8,
            peak_equity: "10500.00".into(),
            final_equity: "10125.50".into(),
            champion_params: None,
            loop_telemetry: vec![],
            breaker_events: 0,
            dry_run: true,
        };
        let json = serde_json::to_string(&report).expect("SessionReport must be serializable");
        assert!(json.contains("10125.50"));
        assert!(json.contains("\"dry_run\":true"));
    }

    #[test]
    fn test_handle_abort_on_replace() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let wd = Watchdog::new();
        wd.register("test_loop");

        let handle1 = rt.spawn(async { tokio::time::sleep(std::time::Duration::from_secs(300)).await });
        wd.store_handle("test_loop", handle1.abort_handle());

        let handle2 = rt.spawn(async {});
        let had_old = wd.abort_and_replace("test_loop", handle2.abort_handle());
        assert!(had_old, "abort_and_replace should detect an existing handle");

        let handle3 = rt.spawn(async {});
        let had_old2 = wd.abort_and_replace("nonexistent_loop", handle3.abort_handle());
        assert!(!had_old2, "abort_and_replace on unknown loop should return false");
    }

    #[test]
    fn test_abort_loop_explicit() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let wd = Watchdog::new();
        assert!(!wd.abort_loop("ghost"), "aborting non-existent returns false");

        let handle = rt.spawn(async { tokio::time::sleep(std::time::Duration::from_secs(300)).await });
        wd.store_handle("streamer", handle.abort_handle());
        assert!(wd.abort_loop("streamer"), "aborting existing handle returns true");
        assert!(!wd.abort_loop("streamer"), "second abort returns false (already removed)");
    }

    #[test]
    fn test_dead_loops_includes_registered_stale() {
        let wd = Watchdog::new();
        wd.register("news");
        wd.register("telegram");
        wd.register("autopilot");
        wd.heartbeat("autopilot", 5);

        // Manually set last_heartbeat to a stale time for news and telegram
        {
            let mut map = wd.loops.lock().unwrap();
            if let Some(s) = map.get_mut("news") {
                s.last_heartbeat = Instant::now() - std::time::Duration::from_secs(STALE_THRESHOLD_SECS + 10);
            }
            if let Some(s) = map.get_mut("telegram") {
                s.last_heartbeat = Instant::now() - std::time::Duration::from_secs(STALE_THRESHOLD_SECS + 10);
            }
        }

        let dead = wd.dead_loops();
        assert!(dead.contains(&"news".to_string()), "news should be dead");
        assert!(dead.contains(&"telegram".to_string()), "telegram should be dead");
        assert!(!dead.contains(&"autopilot".to_string()), "autopilot should be alive");
    }

    #[test]
    fn test_uptime_tracking() {
        let wd = Watchdog::new();
        wd.register("uptime_test");
        std::thread::sleep(std::time::Duration::from_millis(50));
        wd.heartbeat("uptime_test", 5);

        let snap = wd.snapshot();
        let entry = snap.iter().find(|s| s.name == "uptime_test").unwrap();
        assert!(entry.uptime_secs >= 0.04, "uptime must reflect elapsed time since register");

        wd.record_respawn("uptime_test");
        let snap2 = wd.snapshot();
        let entry2 = snap2.iter().find(|s| s.name == "uptime_test").unwrap();
        assert!(entry2.uptime_secs < entry.uptime_secs, "uptime must reset on respawn");
    }

    #[test]
    fn initial_spawn_registers_abort_handle() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let wd = Watchdog::new();

        let loop_names = [
            "streamer", "autopilot", "volatility", "nexus", "daily_reset",
            "genetic_evo", "trade_updates", "position_recon", "equity_sync",
            "news", "catalyst_watchdog", "ai_scanner",
            "smarttrend_sentinel", "smarttrend_bot",
        ];

        for name in &loop_names {
            wd.register(name);
            let handle = rt.spawn(async { tokio::time::sleep(std::time::Duration::from_secs(300)).await });
            wd.abort_and_replace(name, handle.abort_handle());
        }

        for name in &loop_names {
            let new_handle = rt.spawn(async {});
            let had_old = wd.abort_and_replace(name, new_handle.abort_handle());
            assert!(
                had_old,
                "Loop '{}' should have a registered handle from initial spawn",
                name
            );
        }
    }

    #[test]
    fn test_respawn_cooldown_and_max() {
        let wd = Watchdog::new();
        wd.register("test_loop");
        assert!(wd.can_respawn("test_loop"));

        wd.record_respawn("test_loop");
        assert_eq!(wd.respawn_count("test_loop"), 1);
        assert!(!wd.can_respawn("test_loop"), "cooldown not elapsed yet");

        wd.record_respawn("test_loop");
        wd.record_respawn("test_loop");
        assert_eq!(wd.respawn_count("test_loop"), 3);
        assert!(!wd.can_respawn("test_loop"), "max respawns reached");
    }
}

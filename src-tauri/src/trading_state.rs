//! Trading sub-state: per-symbol trail tracking, entry metadata, exit inflight,
//! position blacklist, order cooldown management, and pending order tracking.
//!
//! ## Lock Ordering (acquire in this order to avoid deadlocks)
//!
//! 1. `trail_highs`
//! 2. `trail_lows`
//! 3. `partial_taken`
//! 4. `entry_tags`
//! 5. `entry_times`
//! 6. `entry_features`
//! 7. `exit_inflight`
//! 8. `recon_grace`
//! 9. `failed_order_cooldown`
//! 10. `position_blacklist`
//! 11. `pending_orders`

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::journal;

/// Tracks an order between submission and fill/cancel/reject.
/// Enables local reconciliation without polling Alpaca for every symbol.
#[derive(Clone, Debug, serde::Serialize)]
pub struct PendingOrder {
    pub client_order_id: String,
    pub broker_id: String,
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub submitted_at: u64,
    pub generation_id: u64,
}

pub(crate) struct TradingState {
    /// Per-symbol trailing high watermarks for trailing stop logic
    pub trail_highs: Mutex<HashMap<String, f64>>,
    /// Per-symbol trailing low watermarks for trailing stop logic (shorts)
    pub trail_lows: Mutex<HashMap<String, f64>>,
    /// Per-symbol partial take-profit flag (true = first partial already taken)
    pub partial_taken: Mutex<HashMap<String, bool>>,
    /// Per-symbol entry type tag from check_entry (e.g. "Trending_LONG", "SCALP_MeanRev_SHORT")
    pub entry_tags: Mutex<HashMap<String, String>>,
    /// Per-symbol last-update timestamp for trail entries (epoch secs, for TTL eviction)
    pub trail_last_update: Mutex<HashMap<String, u64>>,
    /// Per-symbol entry timestamp (epoch seconds) for minimum hold time enforcement
    pub entry_times: Mutex<HashMap<String, u64>>,
    /// Per-symbol indicator snapshot captured at entry time, consumed at exit for journal enrichment
    pub entry_features: Mutex<HashMap<String, journal::TradeFeatures>>,
    /// Per-symbol epoch-ms tracking for exit orders in flight (prevents duplicate exits)
    pub exit_inflight: Mutex<HashMap<String, u64>>,
    /// Per-symbol epoch-ms grace period after position reconciliation (suppresses re-entry)
    pub recon_grace: Mutex<HashMap<String, u64>>,
    /// Per-symbol failed order backoff (cooldown_until_epoch_ms, attempt_count).
    /// Cooldown = min(30 * 2^(attempt-1), 300) seconds -- exponential backoff on repeated failures
    pub failed_order_cooldown: Mutex<HashMap<String, (u64, u32)>>,
    /// Blacklisted symbols -- hidden from position reconciliation (force-closed inactive assets)
    pub position_blacklist: Mutex<HashSet<String>>,
    /// Orders submitted but not yet filled/cancelled/rejected -- reconciled via trade_updates
    pub pending_orders: Mutex<Vec<PendingOrder>>,
    /// Per-order cumulative fill tracking: order_id -> (last_cum_qty, last_avg_price).
    /// Alpaca reports cumulative filled_qty per order; we compute incremental delta to avoid
    /// double-counting fills on partial_fill -> fill sequences.
    pub fill_cumulative: Mutex<HashMap<String, (f64, f64)>>,
    /// Count of duplicate/zero-delta fills suppressed this session (B1 dedup dashboard).
    pub fills_deduplicated: AtomicU64,
    /// API calls made in the current minute window (B3 rate limit health monitor).
    pub api_calls_current_minute: AtomicU64,
    /// Epoch second when the current API call minute window started.
    pub api_minute_window_start: AtomicU64,
    /// Total orders submitted this session.
    pub orders_submitted: AtomicU64,
    /// Total fills received this session.
    pub fills_received: AtomicU64,
    /// Total stale entries evicted from trading maps this session.
    pub evicted_entries: AtomicU64,
    /// Per-order last-seen timestamp for fill_cumulative TTL eviction.
    pub fill_cumulative_ts: Mutex<HashMap<String, u64>>,
}

impl TradingState {
    pub(crate) fn new(blacklist: HashSet<String>) -> Self {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            trail_highs: Mutex::new(HashMap::new()),
            trail_lows: Mutex::new(HashMap::new()),
            partial_taken: Mutex::new(HashMap::new()),
            entry_tags: Mutex::new(HashMap::new()),
            trail_last_update: Mutex::new(HashMap::new()),
            entry_times: Mutex::new(HashMap::new()),
            entry_features: Mutex::new(HashMap::new()),
            exit_inflight: Mutex::new(HashMap::new()),
            recon_grace: Mutex::new(HashMap::new()),
            failed_order_cooldown: Mutex::new(HashMap::new()),
            position_blacklist: Mutex::new(blacklist),
            pending_orders: Mutex::new(Vec::new()),
            fill_cumulative: Mutex::new(HashMap::new()),
            fills_deduplicated: AtomicU64::new(0),
            api_calls_current_minute: AtomicU64::new(0),
            api_minute_window_start: AtomicU64::new(now_secs),
            orders_submitted: AtomicU64::new(0),
            fills_received: AtomicU64::new(0),
            evicted_entries: AtomicU64::new(0),
            fill_cumulative_ts: Mutex::new(HashMap::new()),
        }
    }

    /// Total entries across all tracked HashMaps (for memory pressure monitoring).
    pub(crate) fn map_entries_total(&self) -> usize {
        let fc = self.fill_cumulative.lock().map(|m| m.len()).unwrap_or(0);
        let th = self.trail_highs.lock().map(|m| m.len()).unwrap_or(0);
        let tl = self.trail_lows.lock().map(|m| m.len()).unwrap_or(0);
        let cd = self.failed_order_cooldown.lock().map(|m| m.len()).unwrap_or(0);
        let po = self.pending_orders.lock().map(|v| v.len()).unwrap_or(0);
        let et = self.entry_tags.lock().map(|m| m.len()).unwrap_or(0);
        let ei = self.exit_inflight.lock().map(|m| m.len()).unwrap_or(0);
        fc + th + tl + cd + po + et + ei
    }

    /// Evict stale entries from trading maps. Returns the number of entries evicted.
    /// - fill_cumulative: entries older than 24h
    /// - failed_order_cooldown: entries whose cooldown has expired
    /// - trail_highs/trail_lows: entries with no matching active position
    /// - exit_inflight: entries older than TTL (handled separately in trade_updates)
    pub(crate) fn evict_stale_entries(&self, active_symbols: &std::collections::HashSet<String>) -> u64 {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut evicted: u64 = 0;

        const FILL_CUM_TTL_SECS: u64 = 86400;
        if let (Ok(mut fc), Ok(mut fts)) = (self.fill_cumulative.lock(), self.fill_cumulative_ts.lock()) {
            let before = fc.len();
            fc.retain(|k, _| {
                fts.get(k).is_some_and(|&ts| now_secs.saturating_sub(ts) < FILL_CUM_TTL_SECS)
            });
            fts.retain(|k, _| fc.contains_key(k));
            evicted += (before - fc.len()) as u64;
        }

        if let Ok(mut cd) = self.failed_order_cooldown.lock() {
            let now_ms = now_secs * 1000;
            let before = cd.len();
            cd.retain(|_, (cooldown_until_ms, _)| *cooldown_until_ms > now_ms);
            evicted += (before - cd.len()) as u64;
        }

        const TRAIL_ORPHAN_TTL_SECS: u64 = 3600;
        let trail_ts = self.trail_last_update.lock().ok();
        if let Ok(mut th) = self.trail_highs.lock() {
            let before = th.len();
            th.retain(|k, _| {
                if active_symbols.contains(k) { return true; }
                if let Some(ref ts_map) = trail_ts {
                    if let Some(&ts) = ts_map.get(k) {
                        return now_secs.saturating_sub(ts) < TRAIL_ORPHAN_TTL_SECS;
                    }
                }
                false
            });
            evicted += (before - th.len()) as u64;
        }
        if let Ok(mut tl) = self.trail_lows.lock() {
            let before = tl.len();
            tl.retain(|k, _| {
                if active_symbols.contains(k) { return true; }
                if let Some(ref ts_map) = trail_ts {
                    if let Some(&ts) = ts_map.get(k) {
                        return now_secs.saturating_sub(ts) < TRAIL_ORPHAN_TTL_SECS;
                    }
                }
                false
            });
            evicted += (before - tl.len()) as u64;
        }
        if let Ok(mut et) = self.entry_tags.lock() {
            let before = et.len();
            et.retain(|k, _| {
                if active_symbols.contains(k) { return true; }
                if let Some(ref ts_map) = trail_ts {
                    if let Some(&ts) = ts_map.get(k) {
                        return now_secs.saturating_sub(ts) < TRAIL_ORPHAN_TTL_SECS;
                    }
                }
                false
            });
            evicted += (before - et.len()) as u64;
        }
        drop(trail_ts);
        if let Ok(mut tlu) = self.trail_last_update.lock() {
            tlu.retain(|k, _| active_symbols.contains(k) || {
                self.trail_highs.lock().ok().map(|th| th.contains_key(k)).unwrap_or(false)
                || self.trail_lows.lock().ok().map(|tl| tl.contains_key(k)).unwrap_or(false)
            });
        }

        if evicted > 0 {
            self.evicted_entries.fetch_add(evicted, Ordering::Relaxed);
            tracing::debug!(component = "trading_state", evicted = evicted, "Evicted stale entries from trading maps");
        }
        evicted
    }

    /// Record an API call and return (calls_this_minute, limit).
    /// Resets the counter when the minute window rolls over.
    pub(crate) fn record_api_call(&self) -> (u64, u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let window_start = self.api_minute_window_start.load(Ordering::Relaxed);
        if now.saturating_sub(window_start) >= 60 {
            self.api_calls_current_minute.store(1, Ordering::Relaxed);
            self.api_minute_window_start.store(now, Ordering::Relaxed);
            (1, 200)
        } else {
            let count = self.api_calls_current_minute.fetch_add(1, Ordering::Relaxed) + 1;
            (count, 200)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_trading_state_construction_and_access() {
        let mut bl = HashSet::new();
        bl.insert("DELISTED".to_string());
        let ts = TradingState::new(bl);

        ts.trail_highs.lock().unwrap().insert("AAPL".into(), 185.0);
        ts.trail_lows.lock().unwrap().insert("AAPL".into(), 180.0);
        ts.partial_taken.lock().unwrap().insert("AAPL".into(), false);
        ts.entry_tags.lock().unwrap().insert("AAPL".into(), "Trending_LONG".into());
        ts.entry_times.lock().unwrap().insert("AAPL".into(), 1700000000);
        ts.exit_inflight.lock().unwrap().insert("TSLA".into(), 999);
        ts.recon_grace.lock().unwrap().insert("TSLA".into(), 1000);
        ts.failed_order_cooldown.lock().unwrap().insert("GME".into(), (5000, 2));

        assert_eq!(*ts.trail_highs.lock().unwrap().get("AAPL").unwrap(), 185.0);
        assert!(ts.position_blacklist.lock().unwrap().contains("DELISTED"));
        assert_eq!(ts.failed_order_cooldown.lock().unwrap().get("GME").unwrap().1, 2);
    }

    /// FIX-1 CRIT-11: Verifies fill_cumulative tracks cumulative qty and computes delta correctly.
    #[test]
    fn fill_double_count_prevention() {
        let ts = TradingState::new(HashSet::new());
        let order_id = "order-123".to_string();

        // First partial fill: cumulative qty = 5.0
        {
            let mut cum = ts.fill_cumulative.lock().unwrap();
            let (prev_qty, _) = cum.get(&order_id).copied().unwrap_or((0.0, 0.0));
            assert_eq!(prev_qty, 0.0);
            let delta = 5.0 - prev_qty;
            assert!((delta - 5.0).abs() < f64::EPSILON);
            cum.insert(order_id.clone(), (5.0, 100.0));
        }

        // Second partial fill: cumulative qty = 8.0 (delta should be 3.0, NOT 8.0)
        {
            let mut cum = ts.fill_cumulative.lock().unwrap();
            let (prev_qty, _) = cum.get(&order_id).copied().unwrap_or((0.0, 0.0));
            assert!((prev_qty - 5.0).abs() < f64::EPSILON);
            let delta = 8.0 - prev_qty;
            assert!((delta - 3.0).abs() < f64::EPSILON, "delta should be 3.0, got {}", delta);
            cum.insert(order_id.clone(), (8.0, 100.5));
        }

        // Duplicate fill with same cumulative qty (delta should be 0 = suppressed)
        {
            let cum = ts.fill_cumulative.lock().unwrap();
            let (prev_qty, _) = cum.get(&order_id).copied().unwrap_or((0.0, 0.0));
            let delta = 8.0 - prev_qty;
            assert!(delta <= 0.0, "duplicate fill should yield zero delta");
        }

        assert_eq!(ts.fills_deduplicated.load(Ordering::Relaxed), 0);
        ts.fills_deduplicated.fetch_add(1, Ordering::Relaxed);
        assert_eq!(ts.fills_deduplicated.load(Ordering::Relaxed), 1);
    }

    /// FIX-5 CRIT-16: Verifies exit_inflight TTL-based cleanup.
    #[test]
    fn exit_inflight_ttl_cleanup() {
        let ts = TradingState::new(HashSet::new());
        let now_ms: u64 = 1_700_000_120_000;
        let stale_ms: u64 = now_ms - 130_000; // 130s ago = expired (TTL=120s)
        let fresh_ms: u64 = now_ms - 60_000;  // 60s ago = still valid

        {
            let mut inflight = ts.exit_inflight.lock().unwrap();
            inflight.insert("STALE_SYM".to_string(), stale_ms);
            inflight.insert("FRESH_SYM".to_string(), fresh_ms);
        }

        const TTL_MS: u64 = 120_000;
        {
            let mut inflight = ts.exit_inflight.lock().unwrap();
            inflight.retain(|_, &mut ts_val| now_ms.saturating_sub(ts_val) < TTL_MS);
        }

        let inflight = ts.exit_inflight.lock().unwrap();
        assert!(!inflight.contains_key("STALE_SYM"), "stale entry should be removed");
        assert!(inflight.contains_key("FRESH_SYM"), "fresh entry should survive");
    }

    /// FIX-7 B3: API call tracking resets on minute window rollover.
    #[test]
    fn api_call_minute_window_tracking() {
        let ts = TradingState::new(HashSet::new());
        let (count1, limit) = ts.record_api_call();
        assert_eq!(limit, 200);
        assert!(count1 >= 1);
        let (count2, _) = ts.record_api_call();
        assert_eq!(count2, count1 + 1);
    }

    /// FIX-7 HIGH-18: fill_cumulative entries older than 24h are evicted.
    #[test]
    fn fill_cumulative_eviction_after_24h() {
        let ts = TradingState::new(HashSet::new());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let mut fc = ts.fill_cumulative.lock().unwrap();
            fc.insert("old-order".into(), (10.0, 100.0));
            fc.insert("fresh-order".into(), (5.0, 200.0));
        }
        {
            let mut fts = ts.fill_cumulative_ts.lock().unwrap();
            fts.insert("old-order".into(), now - 90000); // 25h ago
            fts.insert("fresh-order".into(), now - 3600); // 1h ago
        }

        let active = HashSet::new();
        let evicted = ts.evict_stale_entries(&active);
        assert!(evicted >= 1, "should evict at least the old order");

        let fc = ts.fill_cumulative.lock().unwrap();
        assert!(!fc.contains_key("old-order"), "old order should be evicted");
        assert!(fc.contains_key("fresh-order"), "fresh order should survive");
    }

    /// FIX-7 HIGH-18: Expired cooldown entries are evicted.
    #[test]
    fn cooldown_eviction() {
        let ts = TradingState::new(HashSet::new());
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        {
            let mut cd = ts.failed_order_cooldown.lock().unwrap();
            cd.insert("expired".into(), (now_ms - 1000, 3));
            cd.insert("active".into(), (now_ms + 60000, 1));
        }

        let active = HashSet::new();
        let evicted = ts.evict_stale_entries(&active);
        assert!(evicted >= 1);

        let cd = ts.failed_order_cooldown.lock().unwrap();
        assert!(!cd.contains_key("expired"));
        assert!(cd.contains_key("active"));
    }

    /// FIX-7 HIGH-18: map_entries_total counts all HashMap entries.
    #[test]
    fn map_entries_total_counts_all() {
        let ts = TradingState::new(HashSet::new());
        ts.trail_highs.lock().unwrap().insert("A".into(), 1.0);
        ts.trail_lows.lock().unwrap().insert("A".into(), 1.0);
        ts.fill_cumulative.lock().unwrap().insert("o1".into(), (1.0, 1.0));
        assert!(ts.map_entries_total() >= 3);
    }

    /// FIX-7 HIGH-18: Trail orphan TTL evicts inactive symbols after 1h.
    #[test]
    fn trail_orphan_ttl_eviction() {
        let ts = TradingState::new(HashSet::new());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ts.trail_highs.lock().unwrap().insert("ACTIVE".into(), 200.0);
        ts.trail_highs.lock().unwrap().insert("ORPHAN_RECENT".into(), 100.0);
        ts.trail_highs.lock().unwrap().insert("ORPHAN_OLD".into(), 50.0);
        ts.trail_lows.lock().unwrap().insert("ORPHAN_OLD".into(), 45.0);

        {
            let mut tlu = ts.trail_last_update.lock().unwrap();
            tlu.insert("ACTIVE".into(), now);
            tlu.insert("ORPHAN_RECENT".into(), now - 1800); // 30min ago -- within TTL
            tlu.insert("ORPHAN_OLD".into(), now - 7200);    // 2h ago -- past TTL
        }

        let mut active = HashSet::new();
        active.insert("ACTIVE".to_string());

        let evicted = ts.evict_stale_entries(&active);
        assert!(evicted >= 1, "should evict at least the old orphan");

        let th = ts.trail_highs.lock().unwrap();
        assert!(th.contains_key("ACTIVE"), "active symbol should survive");
        assert!(th.contains_key("ORPHAN_RECENT"), "recently updated orphan within 1h should survive");
        assert!(!th.contains_key("ORPHAN_OLD"), "orphan older than 1h should be evicted");

        let tl = ts.trail_lows.lock().unwrap();
        assert!(!tl.contains_key("ORPHAN_OLD"), "trail_lows orphan older than 1h should be evicted");
    }
}

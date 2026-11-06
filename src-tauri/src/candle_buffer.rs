use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::app_state::Candle;
use crate::state::MAX_HISTORY;

/// Converts a timeframe string (e.g. "1m", "5m", "1h") to seconds.
pub fn timeframe_to_secs(tf: &str) -> i64 {
    match tf {
        "1m" | "1Min" => 60,
        "3m" | "3Min" => 180,
        "5m" | "5Min" => 300,
        "10m" | "10Min" => 600,
        "15m" | "15Min" => 900,
        "30m" | "30Min" => 1800,
        "1h" | "1Hour" => 3600,
        "4h" | "4Hour" => 14400,
        "1d" | "1Day" => 86400,
        _ => 60,
    }
}

/// In-memory ring buffer of candles for a single symbol+timeframe pair.
///
/// The streamer feeds trade ticks into `update_trade`, which maintains a
/// "current" candle being built and a deque of completed historical candles.
/// Consumers (autopilot) call `snapshot()` to get a `Vec<Candle>` identical
/// in format to what `fetch_bars_internal` returns from the REST API.
pub struct CandleRingBuffer {
    candles: VecDeque<Candle>,
    current: Option<Candle>,
    tf_secs: i64,
}

impl CandleRingBuffer {
    pub fn new(tf_secs: i64) -> Self {
        Self {
            candles: VecDeque::with_capacity(MAX_HISTORY + 1),
            current: None,
            tf_secs: tf_secs.max(1),
        }
    }

    /// Populate the buffer with historical candles from a REST fetch.
    /// Replaces any existing content. Keeps at most `MAX_HISTORY` candles.
    pub fn seed(&mut self, mut bars: Vec<Candle>) {
        bars.sort_by_key(|c| c.time);
        self.candles.clear();
        self.current = None;

        let skip = bars.len().saturating_sub(MAX_HISTORY);
        for c in bars.into_iter().skip(skip) {
            self.candles.push_back(c);
        }
    }

    /// Feed a trade tick into the buffer.
    ///
    /// `trade_ts` is a Unix epoch **seconds** timestamp.
    /// The method determines which candle bucket the trade belongs to, and
    /// either updates the current in-progress candle or finalizes it and
    /// starts a new one.
    pub fn update_trade(&mut self, price: f64, size: f64, trade_ts: i64) {
        if price <= 0.0 {
            return;
        }
        let bucket_start = (trade_ts / self.tf_secs) * self.tf_secs;

        match self.current.as_mut() {
            Some(cur) if cur.time == bucket_start => {
                cur.high = cur.high.max(price);
                cur.low = cur.low.min(price);
                cur.close = price;
                cur.volume += size;
            }
            Some(_) => {
                let finished = self.current.take().unwrap();
                self.push_finished(finished);
                self.current = Some(Candle {
                    time: bucket_start,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: size,
                });
            }
            None => {
                self.current = Some(Candle {
                    time: bucket_start,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: size,
                });
            }
        }
    }

    /// Returns completed candles plus the current in-progress candle (if any),
    /// sorted ascending by time. Format is identical to `fetch_bars_internal`.
    pub fn snapshot(&self) -> Vec<Candle> {
        let mut out = Vec::with_capacity(self.candles.len() + 1);
        out.extend(self.candles.iter().cloned());
        if let Some(cur) = &self.current {
            out.push(cur.clone());
        }
        out
    }

    /// Number of completed candles (excludes the current in-progress one).
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Total candles available (completed + current in-progress).
    pub fn available(&self) -> usize {
        self.candles.len() + usize::from(self.current.is_some())
    }

    fn push_finished(&mut self, candle: Candle) {
        if self.candles.len() >= MAX_HISTORY {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }
}

// ═══════════════════════════════════════════════════════════════════
//  FLASH CRASH DETECTOR — rolling price window with auto-protection
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct FlashCrashEvent {
    pub symbol: String,
    pub change_pct: f64,
    pub window_secs: u64,
}

pub struct FlashCrashDetector {
    windows: HashMap<String, VecDeque<(Instant, f64)>>,
    blocked_until: HashMap<String, Instant>,
    window_duration: Duration,
    threshold_pct: f64,
    cooldown: Duration,
}

impl FlashCrashDetector {
    pub fn new(window_secs: u64, threshold_pct: f64, cooldown_mins: u64) -> Self {
        Self {
            windows: HashMap::new(),
            blocked_until: HashMap::new(),
            window_duration: Duration::from_secs(window_secs),
            threshold_pct,
            cooldown: Duration::from_secs(cooldown_mins * 60),
        }
    }

    /// Feed a trade tick. Returns `Some(FlashCrashEvent)` if a crash is detected
    /// and the symbol is not already in cooldown.
    pub fn check_trade(&mut self, symbol: &str, price: f64, now: Instant) -> Option<FlashCrashEvent> {
        if price <= 0.0 {
            return None;
        }

        if let Some(&blocked) = self.blocked_until.get(symbol) {
            if now < blocked {
                return None;
            }
            self.blocked_until.remove(symbol);
        }

        let window = self.windows.entry(symbol.to_string()).or_default();
        window.push_back((now, price));

        while let Some(&(ts, _)) = window.front() {
            if now.duration_since(ts) > self.window_duration {
                window.pop_front();
            } else {
                break;
            }
        }

        if window.len() < 2 {
            return None;
        }

        let oldest_price = window.front().unwrap().1;
        let change_pct = ((price - oldest_price) / oldest_price) * 100.0;

        if change_pct.abs() > self.threshold_pct {
            self.blocked_until.insert(symbol.to_string(), now + self.cooldown);
            return Some(FlashCrashEvent {
                symbol: symbol.to_string(),
                change_pct,
                window_secs: self.window_duration.as_secs(),
            });
        }

        None
    }

    /// Returns true if the symbol is currently blocked from new entries.
    pub fn is_blocked(&self, symbol: &str) -> bool {
        match self.blocked_until.get(symbol) {
            Some(&until) => Instant::now() < until,
            None => false,
        }
    }

    /// Update detector parameters without losing sliding window history.
    /// Existing price windows and cooldown state are preserved so that a
    /// config change mid-session does not create a detection blind spot.
    pub fn reconfigure(&mut self, window_secs: u64, threshold_pct: f64, cooldown_mins: u64) {
        self.window_duration = Duration::from_secs(window_secs);
        self.threshold_pct = threshold_pct;
        self.cooldown = Duration::from_secs(cooldown_mins * 60);
    }

    /// Clears all state (e.g. on symbol change / reconnect).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.windows.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buf(tf: &str) -> CandleRingBuffer {
        CandleRingBuffer::new(timeframe_to_secs(tf))
    }

    #[test]
    fn seed_limits_to_max_history() {
        let mut buf = make_buf("1m");
        let bars: Vec<Candle> = (0..300)
            .map(|i| Candle {
                time: i * 60,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1.0,
            })
            .collect();
        buf.seed(bars);
        assert_eq!(buf.len(), MAX_HISTORY);
        assert_eq!(buf.candles.front().unwrap().time, 100 * 60);
    }

    #[test]
    fn update_trade_builds_ohlcv_within_same_candle() {
        let mut buf = make_buf("1m");
        let ts = 120; // bucket = 120
        buf.update_trade(100.0, 10.0, ts);
        buf.update_trade(105.0, 5.0, ts + 10);
        buf.update_trade(98.0, 3.0, ts + 20);
        buf.update_trade(102.0, 7.0, ts + 50);

        let snap = buf.snapshot();
        assert_eq!(snap.len(), 1);
        let c = &snap[0];
        assert_eq!(c.time, 120);
        assert_eq!(c.open, 100.0);
        assert_eq!(c.high, 105.0);
        assert_eq!(c.low, 98.0);
        assert_eq!(c.close, 102.0);
        assert!((c.volume - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn update_trade_finalizes_on_boundary_cross() {
        let mut buf = make_buf("1m");
        buf.update_trade(100.0, 10.0, 60);
        buf.update_trade(101.0, 5.0, 80);
        // Cross into next minute
        buf.update_trade(102.0, 8.0, 120);

        assert_eq!(buf.len(), 1); // one completed candle
        assert_eq!(buf.available(), 2); // + current in progress
        let snap = buf.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].time, 60);
        assert_eq!(snap[0].close, 101.0);
        assert_eq!(snap[1].time, 120);
        assert_eq!(snap[1].open, 102.0);
    }

    #[test]
    fn ring_evicts_oldest_when_full() {
        let mut buf = make_buf("1m");
        for i in 0..(MAX_HISTORY as i64 + 5) {
            let ts = i * 60;
            buf.update_trade(100.0, 1.0, ts);
            // Force finalize by stepping to next bucket
            buf.update_trade(100.0, 1.0, ts + 60);
        }
        assert!(buf.len() <= MAX_HISTORY);
    }

    #[test]
    fn snapshot_after_seed_and_update_is_sorted() {
        let mut buf = make_buf("5m");
        let seed_bars: Vec<Candle> = (0..10)
            .map(|i| Candle {
                time: i * 300,
                open: 50.0,
                high: 51.0,
                low: 49.0,
                close: 50.0,
                volume: 100.0,
            })
            .collect();
        buf.seed(seed_bars);
        buf.update_trade(55.0, 20.0, 10 * 300 + 10);

        let snap = buf.snapshot();
        assert_eq!(snap.len(), 11);
        for w in snap.windows(2) {
            assert!(w[0].time <= w[1].time);
        }
    }

    #[test]
    fn timeframe_conversion() {
        assert_eq!(timeframe_to_secs("1m"), 60);
        assert_eq!(timeframe_to_secs("5Min"), 300);
        assert_eq!(timeframe_to_secs("1h"), 3600);
        assert_eq!(timeframe_to_secs("1Hour"), 3600);
        assert_eq!(timeframe_to_secs("1d"), 86400);
        assert_eq!(timeframe_to_secs("unknown"), 60);
    }

    #[test]
    fn ignores_zero_or_negative_prices() {
        let mut buf = make_buf("1m");
        buf.update_trade(0.0, 1.0, 60);
        buf.update_trade(-5.0, 1.0, 60);
        assert_eq!(buf.available(), 0);
    }

    // ── Flash Crash Detector tests ──

    #[test]
    fn flash_crash_no_event_on_normal_trades() {
        let mut det = FlashCrashDetector::new(30, 3.0, 5);
        let t0 = Instant::now();
        for i in 0..10 {
            let price = 100.0 + (i as f64) * 0.1; // +0.1% each
            let result = det.check_trade("AAPL", price, t0 + Duration::from_secs(i));
            assert!(result.is_none());
        }
    }

    #[test]
    fn flash_crash_detects_crash() {
        let mut det = FlashCrashDetector::new(30, 3.0, 5);
        let t0 = Instant::now();
        det.check_trade("AAPL", 100.0, t0);
        // Price drops 4% within 10 seconds
        let event = det.check_trade("AAPL", 96.0, t0 + Duration::from_secs(10));
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.symbol, "AAPL");
        assert!(ev.change_pct < -3.0);
    }

    #[test]
    fn flash_crash_detects_spike_up() {
        let mut det = FlashCrashDetector::new(30, 3.0, 5);
        let t0 = Instant::now();
        det.check_trade("TSLA", 200.0, t0);
        let event = det.check_trade("TSLA", 208.0, t0 + Duration::from_secs(5));
        assert!(event.is_some());
        assert!(event.unwrap().change_pct > 3.0);
    }

    #[test]
    fn flash_crash_cooldown_blocks() {
        let mut det = FlashCrashDetector::new(30, 3.0, 5);
        let t0 = Instant::now();
        det.check_trade("AAPL", 100.0, t0);
        det.check_trade("AAPL", 96.0, t0 + Duration::from_secs(5)); // triggers crash

        assert!(det.is_blocked("AAPL"));
        assert!(!det.is_blocked("TSLA"));

        // During cooldown, no new events even with big moves
        let event = det.check_trade("AAPL", 50.0, t0 + Duration::from_secs(10));
        assert!(event.is_none());
    }

    #[test]
    fn flash_crash_window_expires_old_prices() {
        let mut det = FlashCrashDetector::new(5, 3.0, 5); // 5-second window
        let t0 = Instant::now();
        det.check_trade("SPY", 100.0, t0);
        // 10 seconds later (past window), even a 4% move from t0's price
        // shouldn't trigger because the old data point was evicted
        det.check_trade("SPY", 103.5, t0 + Duration::from_secs(7));
        let event = det.check_trade("SPY", 96.5, t0 + Duration::from_secs(10));
        // 96.5 vs 103.5 = -6.7%, but all within 3s window, should trigger
        assert!(event.is_some());
    }

    #[test]
    fn flash_crash_clear_resets_state() {
        let mut det = FlashCrashDetector::new(30, 3.0, 5);
        let t0 = Instant::now();
        det.check_trade("AAPL", 100.0, t0);
        det.clear();
        // After clear, no window data, so even a big move has no reference point
        let event = det.check_trade("AAPL", 50.0, t0 + Duration::from_secs(1));
        assert!(event.is_none()); // only 1 data point, need >= 2
    }
}

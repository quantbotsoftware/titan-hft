use std::collections::HashMap;
use std::time::Instant;

use crate::app_state::Candle;
use crate::indicators::{IndicatorPeriods, Indicators, StreamingIndicators};
use crate::state::cache_key_symbol_tf;
use crate::titan_oracle::OracleWeights;

/// Cached indicator entry for a single (symbol, timeframe) pair.
/// Stores streaming state for O(1) incremental updates and caches
/// the full Indicators snapshot (streaming + oracle) to avoid
/// recomputation within the same candle interval.
struct CacheEntry {
    streaming: StreamingIndicators,
    last_candle_ts: i64,
    snapshot: Indicators,
    last_access: Instant,
}

/// Per-symbol+timeframe indicator cache with hybrid strategy:
/// - EMA/RSI/MACD/BB/ADX/VWAP/StochRSI: O(1) incremental via StreamingIndicators
/// - Oracle (7-layer): batch recalculated only at candle boundary (new timestamp)
/// - Cache hit (same candle timestamp): returns stored snapshot instantly
pub struct IndicatorCache {
    entries: HashMap<String, CacheEntry>,
}

impl IndicatorCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Look up or compute indicators for a given symbol+timeframe.
    ///
    /// Three paths:
    /// 1. Cache hit (same candle timestamp) -> return stored snapshot, zero computation
    /// 2. Cache miss (new candle) -> streaming.update() O(1) + oracle batch -> merge -> store
    /// 3. Cold start (no entry) -> StreamingIndicators::cold_start() + oracle batch -> store
    pub fn get_or_compute(
        &mut self,
        symbol: &str,
        timeframe: &str,
        candles: &[Candle],
        periods: &IndicatorPeriods,
        oracle_weights: &OracleWeights,
    ) -> Indicators {
        if candles.is_empty() {
            return empty_indicators();
        }

        let last_candle = &candles[candles.len() - 1];
        let last_ts = last_candle.time;
        let key = cache_key_symbol_tf(symbol, timeframe);

        if let Some(entry) = self.entries.get_mut(&key) {
            if entry.last_candle_ts == last_ts {
                entry.last_access = Instant::now();
                return entry.snapshot.clone();
            }

            let streaming_snap = entry.streaming.update(last_candle);
            let oracle = crate::titan_oracle::calculate(candles, oracle_weights);
            let merged = merge_streaming_oracle(streaming_snap, &oracle);

            entry.last_candle_ts = last_ts;
            entry.snapshot = merged;
            entry.last_access = Instant::now();
            return entry.snapshot.clone();
        }

        let streaming = StreamingIndicators::cold_start(candles, periods);

        // At cold start, use the full batch computation for the initial snapshot.
        // This guarantees bit-identical results with the non-cached path and avoids
        // computing oracle twice (cold_start doesn't compute oracle, batch does).
        let snapshot = crate::indicators::calculate_all_with_periods(candles, periods, oracle_weights);

        self.entries.insert(
            key,
            CacheEntry {
                streaming,
                last_candle_ts: last_ts,
                snapshot,
                last_access: Instant::now(),
            },
        );

        self.entries.get(&cache_key_symbol_tf(symbol, timeframe))
            .map(|e| e.snapshot.clone())
            .unwrap_or_else(empty_indicators)
    }

    /// Remove entries not accessed for longer than `max_age`.
    pub fn evict_stale(&mut self, max_age: std::time::Duration) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            now.duration_since(entry.last_access) < max_age
        });
    }

    /// Read-only lookup: returns a clone of the cached snapshot if present,
    /// without triggering any computation or updating access time.
    pub fn peek(&self, symbol: &str, timeframe: &str) -> Option<Indicators> {
        self.entries
            .get(&cache_key_symbol_tf(symbol, timeframe))
            .map(|entry| entry.snapshot.clone())
    }

    /// Return oracle snapshot for a specific symbol (most recent timeframe entry).
    /// Falls back to global last-accessed entry if no per-symbol data exists.
    pub fn oracle_snapshot_for(&self, symbol: &str) -> OracleSnapshot {
        let prefix = format!("{}:", symbol);
        let per_sym = self
            .entries
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .max_by_key(|(_, e)| e.last_access);

        let entry = match per_sym {
            Some((_, e)) => e,
            None => {
                return self
                    .entries
                    .values()
                    .max_by_key(|e| e.last_access)
                    .map(Self::snapshot_from_entry)
                    .unwrap_or_default();
            }
        };
        Self::snapshot_from_entry(entry)
    }

    /// Return oracle snapshot from the most recently accessed cache entry (any symbol).
    pub fn last_oracle_snapshot(&self) -> OracleSnapshot {
        self.entries
            .values()
            .max_by_key(|e| e.last_access)
            .map(Self::snapshot_from_entry)
            .unwrap_or_default()
    }

    fn snapshot_from_entry(entry: &CacheEntry) -> OracleSnapshot {
        OracleSnapshot {
            score: entry.snapshot.oracle_score.unwrap_or(0.0),
            confidence: entry.snapshot.oracle_confidence.unwrap_or(0.0),
            regime: entry.snapshot.oracle_regime.unwrap_or(
                crate::titan_oracle::MarketRegime::Unknown,
            ),
            kalman_gain: entry.snapshot.kalman_gain.unwrap_or(0.2),
            price_surprise: entry.snapshot.price_surprise.unwrap_or(0.0),
            changepoint_prob: entry.snapshot.changepoint_prob.unwrap_or(0.1),
            run_length: entry.snapshot.run_length.unwrap_or(20),
            lyapunov_exponent: entry.snapshot.lyapunov_exponent.unwrap_or(0.0),
        }
    }

    /// Number of cached (symbol, timeframe) pairs.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Lightweight snapshot of oracle state for sentinel/live data consumers.
pub struct OracleSnapshot {
    pub score: f64,
    pub confidence: f64,
    pub regime: crate::titan_oracle::MarketRegime,
    pub kalman_gain: f64,
    pub price_surprise: f64,
    pub changepoint_prob: f64,
    pub run_length: usize,
    pub lyapunov_exponent: f64,
}

impl Default for OracleSnapshot {
    fn default() -> Self {
        Self {
            score: 0.0,
            confidence: 0.0,
            regime: crate::titan_oracle::MarketRegime::Unknown,
            kalman_gain: 0.2,
            price_surprise: 0.0,
            changepoint_prob: 0.1,
            run_length: 20,
            lyapunov_exponent: 0.0,
        }
    }
}

fn merge_streaming_oracle(
    mut streaming: Indicators,
    oracle: &crate::titan_oracle::OracleSnapshot,
) -> Indicators {
    streaming.oracle_score = Some(oracle.oracle_score);
    streaming.oracle_confidence = Some(oracle.confidence);
    streaming.oracle_regime = Some(oracle.dominant_regime);
    streaming.kalman_gain = Some(oracle.kalman.kalman_gain);
    streaming.price_surprise = Some(oracle.kalman.price_vs_predicted);
    streaming.changepoint_prob = Some(oracle.changepoint.changepoint_probability);
    streaming.run_length = Some(oracle.changepoint.run_length);
    streaming.vpin = Some(oracle.vpin.vpin);
    streaming.vpin_buy_pressure = Some(oracle.vpin.buy_pressure);
    streaming.vpin_toxicity = Some(oracle.vpin.toxicity_alert);
    streaming.lyapunov_exponent = Some(oracle.lyapunov.exponent);
    streaming
}

fn empty_indicators() -> Indicators {
    Indicators {
        ema_9: None,
        ema_20: None,
        ema_50: None,
        ema_200: None,
        rsi: None,
        macd: None,
        macd_signal: None,
        macd_hist: None,
        bb_upper: None,
        bb_lower: None,
        bb_mid: None,
        adx: None,
        vwap: None,
        stoch_rsi: None,
        stoch_k: None,
        stoch_d: None,
        oracle_score: None,
        oracle_confidence: None,
        oracle_regime: None,
        vpin: None,
        vpin_buy_pressure: None,
        vpin_toxicity: None,
        kalman_gain: None,
        price_surprise: None,
        changepoint_prob: None,
        run_length: None,
        lyapunov_exponent: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(time: i64, close: f64) -> Candle {
        Candle {
            time,
            open: close - 0.5,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 1000.0,
        }
    }

    fn make_candles(n: usize, base_time: i64) -> Vec<Candle> {
        (0..n)
            .map(|i| {
                let t = base_time + i as i64 * 60;
                let close = 100.0 + (i as f64) * 0.1 + ((i as f64) * 0.7).sin() * 2.0;
                make_candle(t, close)
            })
            .collect()
    }

    fn default_periods() -> IndicatorPeriods {
        IndicatorPeriods::default()
    }

    fn default_oracle_weights() -> OracleWeights {
        OracleWeights::default()
    }

    #[test]
    fn cold_start_populates_cache() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        let result = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);

        assert_eq!(cache.len(), 1);
        assert!(result.ema_9.is_some());
        assert!(result.rsi.is_some());
        assert!(result.oracle_score.is_some());
        assert!(result.vpin.is_some());
    }

    #[test]
    fn cache_hit_returns_same_snapshot() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        let first = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);
        let second = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);

        assert_eq!(first.ema_9, second.ema_9);
        assert_eq!(first.rsi, second.rsi);
        assert_eq!(first.oracle_score, second.oracle_score);
        assert_eq!(first.macd, second.macd);
        assert_eq!(first.adx, second.adx);
    }

    #[test]
    fn new_candle_updates_cache() {
        let mut cache = IndicatorCache::new();
        let mut candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        let first = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);

        candles.push(make_candle(1000 + 200 * 60, 115.0));
        let second = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);

        assert_ne!(first.ema_9, second.ema_9, "EMA should change with new candle");
        assert!(second.oracle_score.is_some());
    }

    #[test]
    fn multi_symbol_independent_caches() {
        let mut cache = IndicatorCache::new();
        let candles_a = make_candles(200, 1000);
        let candles_b: Vec<Candle> = (0..200)
            .map(|i| {
                let t = 2000 + i as i64 * 60;
                let close = 250.0 + (i as f64) * 0.3 + ((i as f64) * 1.2).sin() * 5.0;
                make_candle(t, close)
            })
            .collect();
        let periods = default_periods();
        let ow = default_oracle_weights();

        let a = cache.get_or_compute("AAPL", "5m", &candles_a, &periods, &ow);
        let b = cache.get_or_compute("TSLA", "5m", &candles_b, &periods, &ow);

        assert_eq!(cache.len(), 2);
        assert_ne!(a.ema_9, b.ema_9, "different price series should produce different EMAs");
    }

    #[test]
    fn same_symbol_different_tf_independent() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        cache.get_or_compute("AAPL", "1m", &candles, &periods, &ow);
        cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn evict_stale_removes_old_entries() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);
        assert_eq!(cache.len(), 1);

        cache.evict_stale(std::time::Duration::from_secs(0));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn evict_stale_keeps_recent_entries() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);
        cache.evict_stale(std::time::Duration::from_secs(3600));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn empty_candles_returns_empty_indicators() {
        let mut cache = IndicatorCache::new();
        let periods = default_periods();
        let ow = default_oracle_weights();

        let result = cache.get_or_compute("AAPL", "5m", &[], &periods, &ow);

        assert!(result.ema_9.is_none());
        assert!(result.rsi.is_none());
        assert!(result.oracle_score.is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cold_start_matches_batch() {
        let mut cache = IndicatorCache::new();
        let candles = make_candles(200, 1000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        let cached = cache.get_or_compute("AAPL", "5m", &candles, &periods, &ow);
        let batch = crate::indicators::calculate_all_with_periods(&candles, &periods, &ow);

        assert_eq!(cached.ema_9, batch.ema_9);
        assert_eq!(cached.ema_20, batch.ema_20);
        assert_eq!(cached.ema_50, batch.ema_50);
        assert_eq!(cached.ema_200, batch.ema_200);
        assert_eq!(cached.rsi, batch.rsi);
        assert_eq!(cached.macd, batch.macd);
        assert_eq!(cached.macd_signal, batch.macd_signal);
        assert_eq!(cached.macd_hist, batch.macd_hist);
        assert_eq!(cached.bb_upper, batch.bb_upper);
        assert_eq!(cached.bb_lower, batch.bb_lower);
        assert_eq!(cached.bb_mid, batch.bb_mid);
        assert_eq!(cached.adx, batch.adx);
        assert_eq!(cached.vwap, batch.vwap);
        assert_eq!(cached.stoch_rsi, batch.stoch_rsi);
        assert_eq!(cached.stoch_k, batch.stoch_k);
        assert_eq!(cached.stoch_d, batch.stoch_d);
        assert_eq!(cached.oracle_score, batch.oracle_score);
        assert_eq!(cached.oracle_confidence, batch.oracle_confidence);
        assert_eq!(cached.vpin, batch.vpin);
    }

    #[test]
    fn oracle_snapshot_for_uses_per_symbol() {
        let mut cache = IndicatorCache::new();
        let candles_aapl = make_candles(200, 1000);
        let candles_tsla = make_candles(200, 2000);
        let periods = default_periods();
        let ow = default_oracle_weights();

        cache.get_or_compute("AAPL", "5m", &candles_aapl, &periods, &ow);
        cache.get_or_compute("TSLA", "5m", &candles_tsla, &periods, &ow);

        let aapl_snap = cache.oracle_snapshot_for("AAPL");
        let tsla_snap = cache.oracle_snapshot_for("TSLA");

        assert!(
            aapl_snap.score != 0.0 || tsla_snap.score != 0.0,
            "At least one symbol should have a non-zero oracle score"
        );

        let unknown = cache.oracle_snapshot_for("UNKNOWN");
        assert!(
            unknown.score == aapl_snap.score || unknown.score == tsla_snap.score || unknown.score == 0.0,
            "Unknown symbol should fall back to global last-accessed entry"
        );
    }
}

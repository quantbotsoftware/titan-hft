//! Market data sub-state: price snapshots, candle ring buffers, indicator cache,
//! volume/bars caches, and lock-free OFI state.
//!
//! ## Lock Ordering (acquire in this order to avoid deadlocks)
//!
//! 1. `last_prices`
//! 2. `candle_buffers`
//! 3. `indicator_cache`
//! 4. `vol_cache`
//! 5. `bars_cache`
//! 6. `cvd`
//!
//! `ofi_state` is lock-free (`ArcSwap`) and can be accessed at any time.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::candle_buffer::CandleRingBuffer;
use crate::indicator_cache::IndicatorCache;
use crate::l2_heatmap;
use crate::state::{BarsCacheEntry, VolCache};

pub(crate) struct MarketDataState {
    /// Latest known price per symbol (updated by streamer, consumed by autopilot/strategy/UI)
    pub last_prices: Mutex<HashMap<String, f64>>,
    /// Streaming candle ring buffers per symbol|tf (updated by websocket streamer)
    pub candle_buffers: Mutex<HashMap<String, CandleRingBuffer>>,
    /// Indicator cache -- hybrid streaming O(1) + oracle batch at candle boundary
    pub indicator_cache: Mutex<IndicatorCache>,
    /// Volatility estimate cache per symbol
    pub vol_cache: Mutex<VolCache>,
    /// Historical bars cache per symbol (fetched bars, with expiry)
    pub bars_cache: Mutex<HashMap<String, BarsCacheEntry>>,
    /// L2 live OFI state (updated by websocket stream, lock-free via ArcSwap)
    pub ofi_state: arc_swap::ArcSwap<l2_heatmap::OfiState>,
    /// Cumulative Volume Delta per symbol (buy volume - sell volume, Lee-Ready classified)
    pub cvd: Mutex<HashMap<String, f64>>,
    /// True when the market data stream is stale (disconnected/reconnecting).
    /// Set to true on disconnect, cleared after receiving fresh data for 30s.
    pub stream_stale: AtomicBool,
    /// Epoch seconds when stream last reconnected (for fresh-data timer).
    pub stream_reconnected_at: std::sync::atomic::AtomicU64,
}

impl MarketDataState {
    pub(crate) fn new() -> Self {
        Self {
            last_prices: Mutex::new(HashMap::new()),
            candle_buffers: Mutex::new(HashMap::new()),
            indicator_cache: Mutex::new(IndicatorCache::new()),
            vol_cache: Mutex::new(VolCache::new()),
            bars_cache: Mutex::new(HashMap::new()),
            ofi_state: arc_swap::ArcSwap::from_pointee(l2_heatmap::OfiState::default()),
            cvd: Mutex::new(HashMap::new()),
            stream_stale: AtomicBool::new(false),
            stream_reconnected_at: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Mark stream as stale (disconnected/reconnecting).
    pub(crate) fn mark_stream_stale(&self) {
        self.stream_stale.store(true, Ordering::SeqCst);
    }

    /// Called when stream receives data after reconnect. Clears stale after warmup period.
    pub(crate) fn mark_stream_data_received(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let recon_at = self.stream_reconnected_at.load(Ordering::Relaxed);
        if recon_at > 0 && now.saturating_sub(recon_at) >= 30 && self.stream_stale.load(Ordering::Relaxed) {
            self.stream_stale.store(false, Ordering::SeqCst);
            tracing::info!(component = "streamer", "Stream data fresh -- stale flag cleared");
        }
    }

    /// Mark the moment of reconnect (starts the 30s warmup timer).
    pub(crate) fn mark_stream_reconnected(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.stream_reconnected_at.store(now, Ordering::SeqCst);
    }

    pub(crate) fn is_stream_stale(&self) -> bool {
        self.stream_stale.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// FIX-8 HIGH-19: Stream stale flag tracks disconnect/reconnect.
    #[test]
    fn stream_stale_flag_on_disconnect() {
        let ms = MarketDataState::new();
        assert!(!ms.is_stream_stale(), "should start fresh");

        ms.mark_stream_stale();
        assert!(ms.is_stream_stale(), "should be stale after disconnect");

        ms.mark_stream_reconnected();
        ms.mark_stream_data_received();
        assert!(ms.is_stream_stale(), "should still be stale within 30s warmup");

        ms.stream_reconnected_at.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .saturating_sub(31),
            Ordering::SeqCst,
        );
        ms.mark_stream_data_received();
        assert!(!ms.is_stream_stale(), "should be fresh after 30s warmup");
    }

    #[test]
    fn test_market_state_construction_and_ofi_swap() {
        let ms = MarketDataState::new();

        ms.last_prices.lock().unwrap().insert("AAPL".into(), 185.50);
        assert_eq!(*ms.last_prices.lock().unwrap().get("AAPL").unwrap(), 185.50);

        assert!(ms.candle_buffers.lock().unwrap().is_empty());
        assert!(ms.bars_cache.lock().unwrap().is_empty());

        let ofi = ms.ofi_state.load();
        let _ = Arc::clone(&ofi);

        let new_ofi = l2_heatmap::OfiState::default();
        ms.ofi_state.store(Arc::new(new_ofi));
    }

    #[test]
    fn test_cvd_accumulation() {
        let ms = MarketDataState::new();

        {
            let mut cvd = ms.cvd.lock().unwrap();
            *cvd.entry("AAPL".into()).or_insert(0.0) += 100.0;
            *cvd.entry("AAPL".into()).or_insert(0.0) += 50.0;
            *cvd.entry("AAPL".into()).or_insert(0.0) -= 30.0;
        }

        let cvd = ms.cvd.lock().unwrap();
        let val = *cvd.get("AAPL").unwrap();
        assert!(
            (val - 120.0).abs() < 1e-10,
            "CVD should be 100+50-30=120, got {}",
            val
        );
    }

    #[test]
    fn test_cvd_reset() {
        let ms = MarketDataState::new();

        {
            let mut cvd = ms.cvd.lock().unwrap();
            cvd.insert("AAPL".into(), 500.0);
            cvd.insert("TSLA".into(), -200.0);
        }

        ms.cvd.lock().unwrap().clear();

        assert!(ms.cvd.lock().unwrap().is_empty());
    }
}

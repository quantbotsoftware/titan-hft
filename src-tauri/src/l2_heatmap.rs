use serde::Serialize;
use std::time::Instant;

use crate::risk;

#[derive(Serialize, Clone, Debug)]
pub struct L2Level {
    pub p: f64,
    pub s: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct L2Update {
    pub bids: Vec<L2Level>,
    pub asks: Vec<L2Level>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct OfiState {
    pub current_ofi: f64,
    pub best_bid: f64,
    pub best_ask: f64,
    pub spread_bps: f64,
    pub bid_depth_total: f64,
    pub ask_depth_total: f64,
    pub imbalance_ratio: f64,
    pub update_count: u64,
    pub last_update: Option<Instant>,
    pub stream_vpin: f64,
    /// Raw fast/slow EMAs retained for diagnostics and future UI display.
    pub spread_ema_fast: f64,
    pub spread_ema_slow: f64,
    pub spread_narrowing: bool,
    /// Depth-weighted microprice: (bid*ask_size + ask*bid_size) / (bid_size + ask_size).
    /// More informative than simple mid when book is asymmetric.
    pub microprice: f64,
}

/// Compute depth-weighted microprice. Falls back to simple mid if sizes are zero.
pub fn compute_microprice(bid: f64, ask: f64, bid_size: f64, ask_size: f64) -> f64 {
    let total_size = bid_size + ask_size;
    if bid <= 0.0 || ask <= 0.0 {
        return 0.0;
    }
    if total_size <= 0.0 {
        return (bid + ask) * 0.5;
    }
    (bid * ask_size + ask * bid_size) / total_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn microprice_symmetric_book_equals_mid() {
        let mp = compute_microprice(100.0, 101.0, 100.0, 100.0);
        let mid = 100.5;
        assert!(
            (mp - mid).abs() < 1e-10,
            "Symmetric sizes should give mid={}, got {}",
            mid, mp
        );
    }

    #[test]
    fn microprice_asymmetric_book_skews_toward_larger_side() {
        // bid=100, ask=101, bid_size=200, ask_size=100
        // mp = (100*100 + 101*200) / 300 = (10000 + 20200) / 300 = 100.6667
        let mp = compute_microprice(100.0, 101.0, 200.0, 100.0);
        assert!(
            (mp - 100.6667).abs() < 0.001,
            "Asymmetric book should skew toward ask (larger bid), got {}",
            mp
        );
        assert!(mp > 100.5, "Should be above mid when bid_size > ask_size");
    }

    #[test]
    fn microprice_zero_sizes_falls_back_to_mid() {
        let mp = compute_microprice(100.0, 102.0, 0.0, 0.0);
        assert!(
            (mp - 101.0).abs() < 1e-10,
            "Zero sizes should fallback to mid=101.0, got {}",
            mp
        );
    }

    #[test]
    fn microprice_zero_prices_returns_zero() {
        assert!((compute_microprice(0.0, 0.0, 100.0, 100.0) - 0.0).abs() < f64::EPSILON);
        assert!((compute_microprice(0.0, 101.0, 100.0, 100.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ofi_state_stale_detection() {
        let s = OfiState::default();
        assert!(s.is_stale(1000));

        let s2 = OfiState {
            last_update: Some(Instant::now()),
            ..Default::default()
        };
        assert!(!s2.is_stale(5000));
    }
}

impl OfiState {
    pub fn is_stale(&self, max_age_ms: u128) -> bool {
        match self.last_update {
            None => true,
            Some(t) => t.elapsed().as_millis() > max_age_ms,
        }
    }

    pub fn spread_is_safe(&self, max_spread_pct: f64) -> bool {
        risk::spread_is_safe(self.best_bid, self.best_ask, max_spread_pct)
    }
}

//! AI scan pipeline sub-state: best-TF map, prefilter queue, fusion progress,
//! ranked results, and cancel token.
//!
//! ## Lock Ordering (acquire in this order to avoid deadlocks)
//!
//! 1. `symbol_best_tf`
//! 2. `last_scan_results`
//! 3. `scan_progress`
//! 4. `scan_prefilter_queue`
//! 5. `scan_cancel`

use std::collections::HashMap;
use std::sync::Mutex;

use tokio_util::sync::CancellationToken;

use crate::scanner;

pub(crate) struct ScanState {
    /// Per-symbol best timeframe determined by AI scanner
    pub symbol_best_tf: Mutex<HashMap<String, String>>,
    /// Last completed scan results (ranked candidates)
    pub last_scan_results: Mutex<Vec<scanner::AiScanCandidate>>,
    /// Scan progress tuple: (running, processed, total)
    pub scan_progress: Mutex<(bool, usize, usize)>,
    /// Pre-filter candidate queue awaiting full scoring
    pub scan_prefilter_queue: Mutex<Vec<scanner::PreFilterCandidate>>,
    /// Cancellation token for the current scan run
    pub scan_cancel: Mutex<CancellationToken>,
}

impl ScanState {
    pub(crate) fn new() -> Self {
        Self {
            symbol_best_tf: Mutex::new(HashMap::new()),
            last_scan_results: Mutex::new(Vec::new()),
            scan_progress: Mutex::new((false, 0, 0)),
            scan_prefilter_queue: Mutex::new(Vec::new()),
            scan_cancel: Mutex::new(CancellationToken::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_state_construction_and_cancel_flow() {
        let ss = ScanState::new();

        ss.symbol_best_tf.lock().unwrap().insert("AAPL".into(), "5m".into());
        assert_eq!(
            ss.symbol_best_tf.lock().unwrap().get("AAPL").unwrap(),
            "5m"
        );

        *ss.scan_progress.lock().unwrap() = (true, 5, 20);
        let (running, processed, total) = *ss.scan_progress.lock().unwrap();
        assert!(running);
        assert_eq!(processed, 5);
        assert_eq!(total, 20);

        let token = ss.scan_cancel.lock().unwrap();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}

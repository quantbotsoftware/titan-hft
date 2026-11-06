//! Risk sub-state: circuit breaker, session analytics, intraday drawdown,
//! and flash crash detector.
//!
//! ## Lock Ordering (acquire in this order to avoid deadlocks)
//!
//! 1. `circuit_breaker` (internal atomics, no Mutex -- always safe)
//! 2. `session_analytics`
//! 3. `intraday_drawdown`
//! 4. `flash_crash`

use std::sync::Mutex;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::candle_buffer::FlashCrashDetector;
use crate::risk;

pub(crate) struct RiskState {
    /// Circuit breaker -- escalating halt levels based on drawdown (internal atomics)
    pub circuit_breaker: risk::CircuitBreaker,
    /// Session-level analytics: equity curve, trade PnLs, drawdown velocity
    pub session_analytics: Mutex<risk::SessionAnalytics>,
    /// Intraday drawdown tracking (peak-to-trough within the trading day)
    pub intraday_drawdown: Mutex<risk::IntradayDrawdown>,
    /// Flash crash detector -- rolling price window with auto-exit protection
    pub flash_crash: Mutex<FlashCrashDetector>,
}

impl RiskState {
    pub(crate) fn new(
        starting_equity: Decimal,
        breaker_caution: Decimal,
        breaker_halt: Decimal,
        breaker_panic: Decimal,
        flash_window_secs: u64,
        flash_threshold_pct: f64,
        flash_cooldown_mins: u64,
    ) -> Self {
        let mut dd = risk::IntradayDrawdown::default();
        dd.reset(starting_equity.to_f64().unwrap_or(0.0));

        Self {
            circuit_breaker: risk::CircuitBreaker::new(breaker_caution, breaker_halt, breaker_panic),
            session_analytics: Mutex::new(risk::SessionAnalytics::new(starting_equity)),
            intraday_drawdown: Mutex::new(dd),
            flash_crash: Mutex::new(FlashCrashDetector::new(
                flash_window_secs,
                flash_threshold_pct,
                flash_cooldown_mins,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_risk_state_construction_and_breaker() {
        let rs = RiskState::new(
            dec!(100000),
            dec!(-3.0),
            dec!(-5.0),
            dec!(-8.0),
            300,
            5.0,
            15,
        );

        let analytics = rs.session_analytics.lock().unwrap();
        assert_eq!(analytics.session_start_equity, dec!(100000));
        drop(analytics);

        let dd = rs.intraday_drawdown.lock().unwrap();
        assert!(dd.peak_equity > 0.0);
        drop(dd);

        let _fc = rs.flash_crash.lock().unwrap();
    }
}

use std::sync::Arc;

use tokio::time::Duration;

use crate::commands::data::fetch_bars_internal;
use crate::state::{normalize_symbol, TitanState, VolEntry};

pub(crate) async fn update_volatility_loop(state: Arc<TitanState>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("volatility", 0);
        let (symbol, tf) = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (settings.active_symbol.clone(), settings.manual_res.clone())
        };
        if symbol.is_empty() {
            continue;
        }

        let candles = match fetch_bars_internal(state.clone(), &symbol, &tf).await {
            Ok(v) => v,
            Err(_) => continue,
        };
        if candles.len() < 5 {
            continue;
        }

        let mut returns = Vec::new();
        for w in candles.windows(2) {
            let prev = w[0].close;
            let curr = w[1].close;
            if prev > 0.0 {
                returns.push((curr - prev) / prev);
            }
        }
        if returns.is_empty() {
            continue;
        }

        let mean = returns.iter().copied().sum::<f64>() / returns.len() as f64;
        let var =
            returns.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / returns.len() as f64;
        let vol_pct = var.sqrt() * 100.0;

        let sym = normalize_symbol(&symbol);
        let mut cache = state.market.vol_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.values.insert(sym, VolEntry { value: vol_pct });
    }
}

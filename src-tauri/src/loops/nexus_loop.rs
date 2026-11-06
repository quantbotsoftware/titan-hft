use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::commands::data::fetch_bars_internal;
#[cfg(feature = "gpu")]
use crate::gpu;
use crate::state::{StrategyMode, TitanState};

pub(crate) async fn nexus_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    let mut tick_counter: u32 = 0;
    let mut prev_price: f64 = 0.0;
    tracing::info!("NEXUS LOOP: Background task started (500ms interval)");

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("nexus", 0);
        tick_counter = tick_counter.wrapping_add(1);

        let mode = {
            *state
                .strategy_mode
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };
        if mode != StrategyMode::GpuNexus && mode != StrategyMode::Fusion {
            continue; // Skip dispatch when in MathOracle/SmartTrend mode
        }

        // Get active symbol and timeframe
        let (symbol, tf) = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (settings.active_symbol.clone(), settings.manual_res.clone())
        };
        if symbol.is_empty() {
            continue;
        }

        // Get latest price from candle data
        let current_price = match fetch_bars_internal(state.clone(), &symbol, &tf).await {
            Ok(candles) => {
                let Some(c) = candles.last() else {
                    continue;
                };
                c.close
            }
            Err(_) => continue,
        };

        // Calculate velocity
        let velocity = if prev_price > 0.0 {
            (current_price - prev_price) as f32
        } else {
            0.0
        };
        let vol = (current_price * 0.0001) as f32; // Approximate volatility

        // Log first few dispatches
        if tick_counter <= 3 {
            println!(
                "🤖 NEXUS: Dispatch #{} — sym={}, price={:.2}, velocity={:.4}, vol={:.6}",
                tick_counter, symbol, current_price, velocity, vol
            );
        }

        // Dispatch to GPU
        #[cfg(feature = "gpu")]
        match gpu::nexus_dispatch(
            current_price as f32,
            velocity,
            vol,
            tick_counter,
            prev_price as f32,
        )
        .await
        {
            Ok(result) => {
                if let Ok(mut consensus) = state.nexus_consensus.lock() {
                    consensus.buy_weight = result.buy_weight;
                    consensus.sell_weight = result.sell_weight;
                    consensus.buy_pct = result.buy_pct;
                    consensus.sell_pct = result.sell_pct;
                    consensus.consensus = result.consensus.clone();
                    consensus.active_agents = gpu::NEXUS_AGENT_COUNT;
                    consensus.dispatch_us = result.dispatch_us;
                }
                state.latency_metrics.record_gpu(result.dispatch_us);

                let _ = app.emit_all(
                    "nexus-update",
                    serde_json::json!({
                        "buy_pct": result.buy_pct,
                        "sell_pct": result.sell_pct,
                        "consensus": result.consensus,
                        "dispatch_us": result.dispatch_us,
                        "mode": format!("{}", mode),
                    }),
                );

                if tick_counter <= 3 || tick_counter.is_multiple_of(100) {
                    println!(
                        "🤖 NEXUS: {} | BUY {:.1}% SELL {:.1}% | {}µs | {}",
                        result.consensus,
                        result.buy_pct,
                        result.sell_pct,
                        result.dispatch_us,
                        symbol
                    );
                }
            }
            Err(e) => {
                if tick_counter <= 5 || tick_counter.is_multiple_of(100) {
                    println!("⚠️ NEXUS ERROR (tick {}): {}", tick_counter, e);
                    let _ = app.emit_all("log-message", format!("⚠️ NEXUS GPU ERROR: {}", e));
                }
                let _ = app.emit_all(
                    "nexus-update",
                    serde_json::json!({
                        "buy_pct": 50.0,
                        "sell_pct": 50.0,
                        "consensus": "ERROR",
                        "dispatch_us": 0,
                        "error": format!("{}", e),
                    }),
                );
            }
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = (&app, &symbol, velocity, vol, tick_counter);
        }

        prev_price = current_price;
    }
}

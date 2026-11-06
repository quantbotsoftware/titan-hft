//! Misc Tauri commands (Ghost Engine scan, news, alerts, replay, options hedge).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration as StdDuration;

use rust_decimal::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::app_state::Candle;
use crate::commands::data::fetch_bars_internal;
use crate::error::TitanError;
use crate::journal::TradeRecordInput;
#[cfg(feature = "gpu")]
use crate::gpu;
use crate::monitor;
#[cfg(feature = "options")]
use crate::options;
use crate::sentiment;
use crate::state::{StrategyMode, TitanState, MAX_HISTORY};
#[cfg(feature = "telegram")]
use crate::telegram;
use crate::trading_engine;

#[derive(Serialize, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    time: i64,
}

// Heatmap: Flatten Output (Protocol Buffer style for JS)
#[derive(Serialize, Clone)]
struct HeatmapPacket {
    s: String,   // Symbol
    b: Vec<f64>, // Flattened Bids [p1, v1, p2, v2...]
    a: Vec<f64>, // Flattened Asks
}

#[derive(Serialize, Clone)]
pub(crate) struct BuildInfo {
    version: &'static str,
    git_hash: &'static str,
    build_date: &'static str,
}

#[tauri::command]
pub(crate) async fn get_build_info() -> Result<BuildInfo, TitanError> {
    Ok(BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        git_hash: env!("TITAN_GIT_HASH"),
        build_date: env!("TITAN_BUILD_DATE"),
    })
}

#[derive(Serialize, Clone)]
pub(crate) struct SubStateHealth {
    name: &'static str,
    fields: u32,
    locks_ok: u32,
    locks_contended: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct StateHealth {
    sub_states: Vec<SubStateHealth>,
    total_mutex_fields: u32,
    total_locks_ok: u32,
    total_locks_contended: u32,
    session_generation: u64,
    uptime_secs: u64,
}

#[tauri::command]
pub(crate) async fn get_state_health(
    state: State<'_, Arc<TitanState>>,
) -> Result<StateHealth, TitanError> {
    use std::time::SystemTime;

    let gen = state.current_generation();
    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let trading = {
        let mut ok = 0u32;
        let mut contended = 0u32;
        let fields = 10u32;
        macro_rules! probe {
            ($m:expr) => { if $m.try_lock().is_ok() { ok += 1; } else { contended += 1; } };
        }
        probe!(state.trading.trail_highs);
        probe!(state.trading.trail_lows);
        probe!(state.trading.partial_taken);
        probe!(state.trading.entry_tags);
        probe!(state.trading.entry_times);
        probe!(state.trading.entry_features);
        probe!(state.trading.exit_inflight);
        probe!(state.trading.recon_grace);
        probe!(state.trading.failed_order_cooldown);
        probe!(state.trading.position_blacklist);
        SubStateHealth { name: "Trading", fields, locks_ok: ok, locks_contended: contended }
    };

    let market = {
        let mut ok = 0u32;
        let mut contended = 0u32;
        let fields = 5u32;
        macro_rules! probe {
            ($m:expr) => { if $m.try_lock().is_ok() { ok += 1; } else { contended += 1; } };
        }
        probe!(state.market.last_prices);
        probe!(state.market.candle_buffers);
        probe!(state.market.indicator_cache);
        probe!(state.market.vol_cache);
        probe!(state.market.bars_cache);
        SubStateHealth { name: "Market", fields, locks_ok: ok, locks_contended: contended }
    };

    let risk_h = {
        let mut ok = 0u32;
        let mut contended = 0u32;
        let fields = 3u32;
        macro_rules! probe {
            ($m:expr) => { if $m.try_lock().is_ok() { ok += 1; } else { contended += 1; } };
        }
        probe!(state.risk.session_analytics);
        probe!(state.risk.intraday_drawdown);
        probe!(state.risk.flash_crash);
        SubStateHealth { name: "Risk", fields, locks_ok: ok, locks_contended: contended }
    };

    let scan_h = {
        let mut ok = 0u32;
        let mut contended = 0u32;
        let fields = 5u32;
        macro_rules! probe {
            ($m:expr) => { if $m.try_lock().is_ok() { ok += 1; } else { contended += 1; } };
        }
        probe!(state.scan.symbol_best_tf);
        probe!(state.scan.last_scan_results);
        probe!(state.scan.scan_progress);
        probe!(state.scan.scan_prefilter_queue);
        probe!(state.scan.scan_cancel);
        SubStateHealth { name: "Scan", fields, locks_ok: ok, locks_contended: contended }
    };

    let subs = vec![trading, market, risk_h, scan_h];
    let total_fields: u32 = subs.iter().map(|s| s.fields).sum();
    let total_ok: u32 = subs.iter().map(|s| s.locks_ok).sum();
    let total_contended: u32 = subs.iter().map(|s| s.locks_contended).sum();

    Ok(StateHealth {
        sub_states: subs,
        total_mutex_fields: total_fields,
        total_locks_ok: total_ok,
        total_locks_contended: total_contended,
        session_generation: gen,
        uptime_secs: uptime,
    })
}

#[tauri::command]
pub(crate) async fn start_scan_command(
    symbol: String,
    initial_price: f64,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let (tx, _) = broadcast::channel(1);

    {
        let mut s_tx = state.scan_tx.lock().unwrap_or_else(|e| e.into_inner());
        if s_tx.is_some() {
            return Err(TitanError::Validation("ALREADY SCANNING".into()));
        }
        *s_tx = Some(tx.clone());
    }

    let symbol_clone = symbol.clone();

    // NOTIFY PYTHON AI
    if let Ok(redis_guard) = state.redis.lock() {
        monitor::set_redis_value(redis_guard.as_ref(), "market:active_symbol", &symbol_clone);
    }
    if let Ok(mut settings) = state.settings.lock() {
        settings.active_symbol = symbol_clone.clone();
    }

    let app_clone = app.clone();
    let mut rx = tx.subscribe();
    let state = state.inner().clone(); // Clone Arc for async move
    let http_client = state.http_client.clone();

    tokio::spawn(async move {
        let _ = app_clone.emit_all(
            "log-message",
            format!("🚀 GHOST ENGINE STARTED: {}", symbol_clone),
        );

        let mut ticker = interval(Duration::from_millis(100)); // 100ms updates
        let mut mock_price = if initial_price > 0.0 {
            initial_price
        } else {
            50000.0
        }; // Use real start price
        let mut prev_price: f64 = mock_price; // [NEXUS] For velocity calculation
        let mut tick_counter: u32 = 0; // [NEXUS] Monotonic tick counter
        let mut prices_history: VecDeque<crate::app_state::Candle> =
            VecDeque::with_capacity(MAX_HISTORY);

        // Sympathy Map
        let sympathy_map = HashMap::from([("MARA", "BTC-USD"), ("AMD", "NVDA")]);
        let leader_sym = sympathy_map
            .get(symbol_clone.as_str())
            .unwrap_or(&"BTC-USD");

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    let _ = app_clone.emit_all("log-message", "🛑 ENGINE SHUTDOWN");
                    break;
                }
                _ = ticker.tick() => {
                    tick_counter = tick_counter.wrapping_add(1); // Move here — always increment
                    // [DIAGNOSTIC] Print every 100 ticks to prove loop runs
                    if tick_counter <= 3 || tick_counter.is_multiple_of(200) {
                        let dbg_mode = { *state.strategy_mode.lock().unwrap_or_else(|e| e.into_inner()) };
                        println!("🔍 TICK LOOP #{}: mode={:?} — loop is running!", tick_counter, dbg_mode);
                    }
                    // 1. SIMULATE DATA (Random Walk)
                    let volatility = mock_price * 0.0001; // 0.01% per tick
                    mock_price += (rand::random::<f64>() - 0.5) * volatility;

                    // Update Buffer
                    let timestamp = chrono::Utc::now().timestamp();
                    let current_candle = crate::app_state::Candle {
                         time: timestamp,
                         open: mock_price, high: mock_price, low: mock_price, close: mock_price,
                         volume: 1000.0 + (rand::random::<f64>() * 5000.0)
                    };

                    if prices_history.len() >= 200 { prices_history.pop_front(); }
                    prices_history.push_back(current_candle);

                    // Convert to slice for strategy
                    let candles_vec: Vec<crate::app_state::Candle> = prices_history.iter().cloned().collect();

                    // 2. EMIT PRICE UPDATE (For Chart)
                    let update = PriceUpdate {
                        symbol: symbol_clone.clone(),
                        price: mock_price,
                        time: timestamp,
                    };
                    let _ = app_clone.emit_all("price-update", update);

                    // --- TRADING ENGINE INTEGRATION (THE BRAIN) ---
                    // PART 1: EXITS (Fast, Synchronous)
                    {
                        // Scope for Mutex Lock
                        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        let current_prices = HashMap::from([(symbol_clone.clone(), mock_price)]);

                        // A. MANAGE EXITS
                        let exit_actions = trading_engine::manage_positions(&mut settings, &current_prices);
                        for action in &exit_actions {
                             let _ = app_clone.emit_all("log-message", format!("EXIT: {} {} @ ${:.2} | {}", action.action, action.symbol, action.price, action.reason));
                        }
                        drop(settings);

                        for action in exit_actions {
                            if let Some(pnl) = action.pnl {
                                if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                                    analytics.record_trade(pnl);
                                }
                                let strategy_name = state.strategy_mode.lock()
                                    .map(|m| m.to_string()).unwrap_or_else(|_| "UNKNOWN".into());
                                let (journal_source, journal_tf) = {
                                    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                    let src = match strategy_name.as_str() {
                                        "MATH_ORACLE" => "ORACLE".to_string(),
                                        "GPU_NEXUS" => "GPU".to_string(),
                                        "FUSION" => "FUSION".to_string(),
                                        "SIGNAL_STRATEGY" => s.signal_source.to_ascii_uppercase(),
                                        _ => "UNKNOWN".to_string(),
                                    };
                                    (src, s.manual_res.clone())
                                };
                                if let Ok(mut journal) = state.trade_journal.lock() {
                                    let price_f = action.price.to_f64().unwrap_or(0.0);
                                    journal.record_trade(&TradeRecordInput {
                                        symbol: &action.symbol,
                                        side: &action.action,
                                        qty: action.qty.to_f64().unwrap_or(0.0),
                                        entry_price: price_f,
                                        exit_price: price_f,
                                        pnl: pnl.to_f64().unwrap_or(0.0),
                                        strategy_mode: &strategy_name,
                                        source: &journal_source,
                                        timeframe: &journal_tf,
                                        duration_secs: 0,
                                        reason: &action.reason,
                                        entry_features: None,
                                        exit_features: None,
                                        broker_order_id: None,
                                    });
                                }
                            }
                        }
                    }

                    // PART 2: ENTRIES (Complex, Async)
                    // First Check Technicals (Read Only Lock)
                    let should_check_sentiment = {
                        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        if settings.auto_pilot && candles_vec.len() > 20 {
                            let periods = crate::state::build_indicator_periods(&settings);
                            let oracle_w = crate::state::build_oracle_weights(&settings);
                            let indicators = crate::indicators::calculate_all_with_periods(&candles_vec, &periods, &oracle_w);
                            crate::strategy::check_entry(&symbol_clone, &candles_vec, &indicators, &settings)
                        } else {
                            None
                        }
                    }; // Lock Released Here

                    if let Some(decision) = should_check_sentiment {
                          // 2. SENTIMENT FILTER (Async, No Lock Held)
                          let (k, s) = {
                              let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
                              (keys.key.clone(), keys.secret.clone())
                          };

                          // Run Async Sentiment Fetch
                          let sentiment_score = crate::sentiment::get_sentiment_score(&http_client, &symbol_clone, &k, &s).await;

                          // Filter Logic
                          let mut confirmed = false;
                          let is_short = matches!(decision.signal, crate::strategy::Signal::Short);

                          if !is_short && sentiment_score > 0.1 { confirmed = true; }
                          if is_short && sentiment_score < -0.1 { confirmed = true; }
                          if sentiment_score.abs() < 0.1 { confirmed = true; } // Neutral ok
                          if decision.score > 80 { confirmed = true; } // Tech Override

                          if confirmed {
                              let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                              let entry_ctx = trading_engine::EntryContext {
                                  equity: settings.equity,
                                  risk_per_trade_pct: settings.risk_per_trade,
                                  stop_loss_distance_pct: if settings.aggressive_mode { 8.0 } else { 2.5 },
                                  breaker_multiplier: state.risk.circuit_breaker.size_multiplier(),
                                  aggressive_mode: settings.aggressive_mode,
                                  max_notional_equity_frac: settings.max_notional_equity_frac,
                                  min_stop_distance_frac: settings.min_stop_distance_frac,
                              };
                              if let Some(entry_action) = trading_engine::execute_entry(&mut settings, &symbol_clone, mock_price, &decision.entry_type, is_short, &entry_ctx) {
                                  let _ = app_clone.emit_all("log-message", format!("⚡ ENTRY: {} {} @ ${:.2} | Score: {} | News: {:.2}", entry_action.action, entry_action.symbol, entry_action.price, decision.score, sentiment_score));
                              }
                          } else if rand::random::<f64>() > 0.95 {
                               let _ = app_clone.emit_all("log-message", format!("✋ FILTERED: Tech Signal but Sentiment {:.2} Weak", sentiment_score));
                          }
                    }

                    // ═══════════════════════════════════════════
                    //  GPU NEXUS DISPATCH (Per-Tick Agent Consensus)
                    // ═══════════════════════════════════════════
                    {
                        let mode = { *state.strategy_mode.lock().unwrap_or_else(|e| e.into_inner()) };
                        // [DIAGNOSTIC] Log mode every 100 ticks unconditionally
                        if tick_counter.is_multiple_of(100) || tick_counter < 5 {
                            println!("🔍 NEXUS TICK #{}: mode={:?}, checking GpuNexus={}, Fusion={}",
                                tick_counter, mode, mode == StrategyMode::GpuNexus, mode == StrategyMode::Fusion);
                        }
                        if mode == StrategyMode::GpuNexus || mode == StrategyMode::Fusion {
                            let velocity = (mock_price - prev_price) as f32;
                            let vol = (mock_price * 0.0001) as f32; // Approximate volatility

                            // Log first dispatch attempt
                            if tick_counter <= 3 {
                                println!("🤖 NEXUS: Dispatch attempt #{} — price={}, velocity={}, vol={}", tick_counter, mock_price, velocity, vol);
                            }

                            #[cfg(feature = "gpu")]
                            match gpu::nexus_dispatch(
                                mock_price as f32,
                                velocity,
                                vol,
                                tick_counter,
                                prev_price as f32,
                            ).await {
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

                                    let _ = app_clone.emit_all("nexus-update", serde_json::json!({
                                        "buy_pct": result.buy_pct,
                                        "sell_pct": result.sell_pct,
                                        "consensus": result.consensus,
                                        "dispatch_us": result.dispatch_us,
                                        "mode": format!("{}", mode),
                                    }));

                                    if tick_counter == 1 || tick_counter.is_multiple_of(50) {
                                        println!("🤖 NEXUS: {} | BUY {:.1}% SELL {:.1}% | {}µs",
                                            result.consensus, result.buy_pct, result.sell_pct, result.dispatch_us);
                                    }
                                }
                                Err(e) => {
                                    if tick_counter <= 3 || tick_counter.is_multiple_of(50) {
                                        println!("⚠️ NEXUS DISPATCH ERROR (tick {}): {}", tick_counter, e);
                                        let _ = app_clone.emit_all("log-message",
                                            format!("⚠️ NEXUS GPU ERROR: {}", e));
                                    }
                                    let _ = app_clone.emit_all("nexus-update", serde_json::json!({
                                        "buy_pct": 50.0,
                                        "sell_pct": 50.0,
                                        "consensus": "ERROR",
                                        "dispatch_us": 0,
                                        "mode": format!("{}", mode),
                                        "error": format!("{}", e),
                                    }));
                                }
                            }
                            #[cfg(not(feature = "gpu"))]
                            {
                                let _ = (velocity, vol);
                            }
                        }
                        prev_price = mock_price;
                    }

                    // 3. HEATMAP GENERATION (Flattened)
                    let mut bids = Vec::with_capacity(100);
                    let mut asks = Vec::with_capacity(100);
                    for i in 0..50 {
                        bids.push(mock_price - i as f64 * (volatility * 0.1));
                        bids.push(rand::random::<f64>() * 5.0);
                        asks.push(mock_price + i as f64 * (volatility * 0.1));
                        asks.push(rand::random::<f64>() * 5.0);
                    }

                    let packet = HeatmapPacket {
                        s: symbol_clone.clone(),
                        b: bids,
                        a: asks,
                    };
                    let _ = app_clone.emit_all("heatmap-update", packet);

                    // 4. ORDER FLOW & AI TARGET SIMULATION
                    if let Ok(redis_guard) = state.redis.lock() {
                         // A. Order Flow Simulation (Whale Pressure)
                         let flow_buy = (rand::random::<f64>() * 100000.0) + 50000.0;
                         let flow_sell = (rand::random::<f64>() * 100000.0) + 50000.0;
                         let flow_state = if flow_buy > flow_sell * 1.2 { "WHALE BUY" }
                                          else if flow_sell > flow_buy * 1.2 { "WHALE SELL" }
                                          else { "NEUTRAL" };

                         monitor::set_redis_value(redis_guard.as_ref(), "market:flow:buy", &flow_buy.to_string());
                         monitor::set_redis_value(redis_guard.as_ref(), "market:flow:sell", &flow_sell.to_string());
                         monitor::set_redis_value(redis_guard.as_ref(), "market:flow:state", flow_state);

                         // B. Update AI Target (if active)
                         // Note: AI Sentinel is usually independent, but we ensure values are fresh here
                         // Retrain Brain button triggers an event check, see retrain_brain command
                    }

                    // 5. SYMPATHY CHECK
                    if rand::random::<f64>() > 0.99 {
                        let _ = app_clone.emit_all("log-message", format!("🔗 SYMPATHY ALERT: {} Follows {}", symbol_clone, leader_sym));
                    }
                }
            }
        }

        let _ = app_clone.emit_all("scan-status", "STOPPED");
    });

    Ok("STARTED".into())
}

#[tauri::command]
pub(crate) async fn stop_scan_command(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let tx = {
        let mut s = state.scan_tx.lock().unwrap_or_else(|e| e.into_inner());
        s.take()
    };

    if let Some(t) = tx {
        let _ = t.send(());
        let _ = app.emit_all("log-message", "🛑 STOP COMMAND RECEIVED");
        Ok("STOPPED".into())
    } else {
        Err(TitanError::Validation("NOT RUNNING".into()))
    }
}

#[tauri::command]
pub(crate) async fn test_catalyst_alert(app: AppHandle) -> Result<(), TitanError> {
    let event = crate::sentiment::CatalystEvent {
        symbol: "NVDA".to_string(),
        headline: "BREAKING: NVIDIA Announces Artificial Superintelligence Breakthrough, Blows Past Earnings".to_string(),
        summary: "NVIDIA shares skyrocketed in after-hours trading following the announcement of a revolutionary new architecture that achieves AGI benchmarks, sending ripples across the entire tech sector.".to_string(),
        url: "https://example.com/nvda-agi".to_string(),
        sentiment_score: 0.98,
        timestamp: "2026-03-06T15:30:00Z".to_string(),
        severity: crate::sentiment::catalyst_severity(0.98),
    };
    app.emit_all("catalyst-alert", &event)
        .map_err(|e| TitanError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) async fn test_catalyst_bearish(app: AppHandle) -> Result<(), TitanError> {
    let event = crate::sentiment::CatalystEvent {
        symbol: "TSLA".to_string(),
        headline: "BREAKING: SEC Opens Fraud Investigation Into Tesla's Self-Driving Claims".to_string(),
        summary: "Tesla shares plunged in after-hours trading after the SEC announced a formal investigation into allegedly misleading statements about the company's autonomous driving capabilities.".to_string(),
        url: "https://example.com/tsla-sec".to_string(),
        sentiment_score: -0.92,
        timestamp: "2026-03-06T16:45:00Z".to_string(),
        severity: crate::sentiment::catalyst_severity(-0.92),
    };
    app.emit_all("catalyst-alert", &event)
        .map_err(|e| TitanError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) async fn get_news_feed(
    state: State<'_, Arc<TitanState>>,
) -> Result<sentiment::SentimentSnapshot, TitanError> {
    let articles = state
        .news_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let history = state
        .sentiment_history
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let overall = if articles.is_empty() {
        0.0
    } else {
        articles.iter().map(|a| a.sentiment_score).sum::<f64>() / articles.len() as f64
    };
    let label = sentiment::label_from_score(overall);
    Ok(sentiment::SentimentSnapshot {
        overall_score: overall,
        overall_label: label,
        articles,
        history,
    })
}

#[tauri::command]
pub(crate) async fn refresh_news(state: State<'_, Arc<TitanState>>) -> Result<String, TitanError> {
    let (symbol, key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        let k = keys.key.clone();
        let s = keys.secret.clone();
        drop(keys);
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (settings.active_symbol.clone(), k, s)
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("No API keys".into()));
    }
    let articles = sentiment::fetch_news_feed(&state.http_client, &symbol, &key, &secret, 20).await;
    let count = articles.len();
    let overall = if articles.is_empty() {
        0.0
    } else {
        articles.iter().map(|a| a.sentiment_score).sum::<f64>() / articles.len() as f64
    };
    {
        let ts = chrono::Utc::now().format("%H:%M").to_string();
        let mut hist = state
            .sentiment_history
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        hist.push((ts, overall));
        let excess = hist.len().saturating_sub(288);
        if excess > 0 {
            hist.drain(0..excess);
        }
    }
    *state.news_cache.lock().unwrap_or_else(|e| e.into_inner()) = articles;
    Ok(format!("Refreshed {} articles for {}", count, symbol))
}

#[derive(serde::Serialize, Clone, Debug)]
pub(crate) struct CatalystCheckResult {
    symbol: String,
    has_catalyst: bool,
    headline: String,
    url: String,
    sentiment_score: f64,
}

/// Unified catalyst threshold (bullish ≥ threshold, bearish ≤ -threshold)
const CHECK_CATALYST_THRESHOLD: f64 = 0.50;

#[tauri::command]
pub(crate) async fn check_catalyst(
    symbols: Vec<String>,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<CatalystCheckResult>, TitanError> {
    let (key, secret) = {
        let k = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (k.key.clone(), k.secret.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("No API keys configured".into()));
    }

    // Parallel fetch with concurrency cap via chunks
    let http_client = state.http_client.clone();
    let mut all_results = Vec::with_capacity(symbols.len());
    for chunk in symbols.chunks(5) {
        let futures = chunk.iter().map(|sym| {
            let sym = sym.clone();
            let key = key.clone();
            let secret = secret.clone();
            let hc = http_client.clone();
            async move {
                let fetch_result = tokio::time::timeout(
                    StdDuration::from_secs(10),
                    sentiment::fetch_news_feed(&hc, &sym, &key, &secret, 5),
                )
                .await;

                let articles = fetch_result.unwrap_or_default();

                let best = articles.iter().max_by(|a, b| {
                    a.sentiment_score
                        .abs()
                        .partial_cmp(&b.sentiment_score.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some(article) = best {
                    if article.sentiment_score.abs() >= CHECK_CATALYST_THRESHOLD {
                        CatalystCheckResult {
                            symbol: sym,
                            has_catalyst: true,
                            headline: article.headline.clone(),
                            url: article.url.clone(),
                            sentiment_score: article.sentiment_score,
                        }
                    } else {
                        CatalystCheckResult {
                            symbol: sym,
                            has_catalyst: false,
                            headline: articles
                                .first()
                                .map(|a| a.headline.clone())
                                .unwrap_or_default(),
                            url: articles.first().map(|a| a.url.clone()).unwrap_or_default(),
                            sentiment_score: articles
                                .first()
                                .map(|a| a.sentiment_score)
                                .unwrap_or(0.0),
                        }
                    }
                } else {
                    CatalystCheckResult {
                        symbol: sym,
                        has_catalyst: false,
                        headline: String::new(),
                        url: String::new(),
                        sentiment_score: 0.0,
                    }
                }
            }
        });

        let chunk_results = futures_util::future::join_all(futures).await;
        all_results.extend(chunk_results);
    }

    Ok(all_results)
}

async fn send_discord_alert(
    client: &reqwest::Client,
    webhook_url: &str,
    title: &str,
    message: &str,
    color: u32,
) {
    if webhook_url.is_empty() {
        return;
    }
    let payload = serde_json::json!({
        "embeds": [{
            "title": title,
            "description": message,
            "color": color,
            "footer": { "text": "TITAN HFT Bot" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }]
    });
    let _ = client.post(webhook_url).json(&payload).send().await;
}

#[tauri::command]
pub(crate) async fn test_discord_alert(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let url = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.discord_webhook_url.clone()
    };
    if url.is_empty() {
        return Err(TitanError::Validation(
            "Discord webhook URL not configured".into(),
        ));
    }
    let client = state.http_client.clone();
    send_discord_alert(
        &client,
        &url,
        "TITAN HFT Test",
        "Discord alerting is working!",
        0x00FF41,
    )
    .await;
    Ok("Test alert sent".into())
}

#[tauri::command]
pub(crate) async fn test_telegram_alert(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    #[cfg(feature = "telegram")]
    {
        let (token, chat_id) = {
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (
                settings.telegram_bot_token.clone(),
                settings.telegram_chat_id.clone(),
            )
        };
        if token.is_empty() || chat_id.is_empty() {
            return Err(TitanError::Validation(
                "Telegram Bot Token or Chat ID not configured".into(),
            ));
        }
        let msg = "✅ <b>TITAN HFT</b>\nTelegram alerting is working!";
        let client = state.http_client.clone();
        telegram::send_telegram_alert(&client, &token, &chat_id, msg).await;
        Ok("Telegram test alert sent".into())
    }
    #[cfg(not(feature = "telegram"))]
    {
        let _ = state;
        Err(TitanError::Validation(
            "Telegram feature not enabled".into(),
        ))
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct ReplayTrade {
    time: i64,
    side: String,
    price: f64,
    pnl: f64,
    reason: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct ReplayData {
    candles: Vec<Candle>,
    trades: Vec<ReplayTrade>,
}

#[tauri::command]
pub(crate) async fn get_replay_symbols(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<String>, TitanError> {
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let trades = journal.get_trades(1000, 0);
    let mut symbols: Vec<String> = trades
        .iter()
        .map(|t| t.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    symbols.sort();
    Ok(symbols)
}

#[tauri::command]
pub(crate) async fn get_replay_data(
    symbol: String,
    timeframe: String,
    date_from: String,
    date_to: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<ReplayData, TitanError> {
    let candles = fetch_bars_internal(state.inner().clone(), &symbol, &timeframe).await?;

    let from_ts = chrono::NaiveDate::parse_from_str(&date_from, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0);
    let to_ts = chrono::NaiveDate::parse_from_str(&date_to, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(i64::MAX);

    let filtered_candles: Vec<Candle> = candles
        .into_iter()
        .filter(|c| c.time >= from_ts && c.time <= to_ts)
        .collect();

    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let all_trades = journal.get_trades(1000, 0);
    let replay_trades: Vec<ReplayTrade> = all_trades
        .iter()
        .filter(|t| t.symbol == symbol)
        .filter(|t| {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&t.timestamp, "%Y-%m-%d %H:%M:%S")
            {
                let ts = dt.and_utc().timestamp();
                ts >= from_ts && ts <= to_ts
            } else {
                false
            }
        })
        .map(|t| ReplayTrade {
            time: chrono::NaiveDateTime::parse_from_str(&t.timestamp, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0),
            side: t.side.clone(),
            price: if t.side.eq_ignore_ascii_case("buy") {
                t.entry_price.to_f64().unwrap_or(0.0)
            } else {
                t.exit_price.to_f64().unwrap_or(0.0)
            },
            pnl: t.pnl.to_f64().unwrap_or(0.0),
            reason: t.reason.clone(),
        })
        .collect();

    Ok(ReplayData {
        candles: filtered_candles,
        trades: replay_trades,
    })
}

#[tauri::command]
pub(crate) async fn get_option_chain(
    symbol: String,
    expiry: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    #[cfg(feature = "options")]
    {
        let chain = options::fetch_option_chain(state.inner(), &symbol, &expiry).await?;
        Ok(serde_json::to_value(chain).unwrap_or_default())
    }
    #[cfg(not(feature = "options"))]
    {
        let _ = (symbol, expiry, state);
        Err(TitanError::Validation("Options feature not enabled".into()))
    }
}

#[tauri::command]
pub(crate) async fn hedge_position(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    #[cfg(feature = "options")]
    {
        let positions = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            s.positions.clone()
        };
        let pos = positions
            .iter()
            .find(|p| p.symbol == symbol)
            .ok_or_else(|| TitanError::Validation(format!("No open position for {}", symbol)))?;

        let (otm_pct, max_premium_pct) = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (s.hedge_otm_pct, s.hedge_max_premium_pct)
        };

        let expiry = (chrono::Utc::now() + chrono::Duration::days(14))
            .format("%Y-%m-%d")
            .to_string();
        let chain = options::fetch_option_chain(state.inner(), &symbol, &expiry).await?;
        let risk_budget = pos.current_price.to_f64().unwrap_or(0.0)
            * pos.qty.to_f64().unwrap_or(0.0)
            * max_premium_pct
            / 100.0;

        let suggestion = options::suggest_hedge(pos, &chain, risk_budget, otm_pct)
            .ok_or_else(|| TitanError::Validation("No suitable hedge found".into()))?;

        let result =
            options::submit_option_order(state.inner(), &suggestion.contract_symbol, 1.0, "buy")
                .await?;
        Ok(format!(
            "Hedge placed: {} — {}",
            suggestion.contract_symbol, result
        ))
    }
    #[cfg(not(feature = "options"))]
    {
        let _ = (symbol, state);
        Err(TitanError::Validation("Options feature not enabled".into()))
    }
}

#[tauri::command]
pub(crate) async fn get_hedge_status(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<serde_json::Value>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    let client = state.http_client.clone();
    let resp = client
        .get("https://paper-api.alpaca.markets/v2/positions")
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(e.to_string()))?;

    let positions: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| TitanError::Internal(e.to_string()))?;
    let hedges: Vec<serde_json::Value> = positions
        .into_iter()
        .filter(|p| p.get("asset_class").and_then(|a| a.as_str()) == Some("us_option"))
        .collect();
    Ok(hedges)
}

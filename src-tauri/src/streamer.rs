use crate::candle_buffer::{timeframe_to_secs, CandleRingBuffer};
use crate::l2_heatmap::{L2Level, L2Update};
use rust_decimal::Decimal;
use crate::monitor;
use crate::risk;
use crate::state::{cache_key_symbol_tf, normalize_symbol, normalize_timeframe_internal};
use crate::TitanState;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Instant;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::Receiver;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;

#[derive(Debug, Clone)]
pub enum StreamCmd {
    SetSymbol(String),
    Reconnect,
}

#[derive(Clone, Debug, Default)]
struct QuoteTop {
    bid: f64,
    ask: f64,
    bid_size: f64,
    ask_size: f64,
}

#[derive(Clone, Debug, Default)]
struct FlowStats {
    buy_count: u64,
    sell_count: u64,
    buy_notional: f64,
    sell_notional: f64,
    last_price: f64,
    total_trades: u64,
    classified_trades: u64,
    trade_intensity_ema: f64,
    last_intensity_update: Option<tokio::time::Instant>,
}

impl FlowStats {
    fn classified_pct(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.classified_trades as f64 / self.total_trades as f64) * 100.0
        }
    }

    fn update_intensity(&mut self, now: tokio::time::Instant) {
        const HALFLIFE_SECS: f64 = 30.0;
        let alpha = 1.0 - (-1.0 / HALFLIFE_SECS).exp();
        match self.last_intensity_update {
            None => {
                self.trade_intensity_ema = 1.0;
                self.last_intensity_update = Some(now);
            }
            Some(prev) => {
                let dt = now.duration_since(prev).as_secs_f64().max(0.001);
                let instant_rate = 1.0 / dt;
                self.trade_intensity_ema =
                    alpha * instant_rate + (1.0 - alpha) * self.trade_intensity_ema;
                self.last_intensity_update = Some(now);
            }
        }
    }
}

/// Volume-synced VPIN from trade-by-trade flow classification.
/// More precise than candle-based BVC VPIN because it uses actual
/// Lee-Ready trade classification instead of close-position-in-range proxy.
#[derive(Clone, Debug)]
struct StreamVpin {
    buckets: std::collections::VecDeque<VpinBucket>,
    current_buy_vol: f64,
    current_sell_vol: f64,
    bucket_vol_threshold: f64,
    ema_vpin: f64,
    max_buckets: usize,
}

#[derive(Clone, Debug)]
struct VpinBucket {
    buy_vol: f64,
    sell_vol: f64,
}

impl StreamVpin {
    fn new(bucket_vol_threshold: f64, max_buckets: usize) -> Self {
        Self {
            buckets: std::collections::VecDeque::with_capacity(max_buckets),
            current_buy_vol: 0.0,
            current_sell_vol: 0.0,
            bucket_vol_threshold: bucket_vol_threshold.max(1.0),
            ema_vpin: 0.0,
            max_buckets,
        }
    }

    fn update(&mut self, is_buy: bool, volume: f64) {
        if is_buy {
            self.current_buy_vol += volume;
        } else {
            self.current_sell_vol += volume;
        }

        while self.current_buy_vol + self.current_sell_vol >= self.bucket_vol_threshold {
            let bucket = VpinBucket {
                buy_vol: self.current_buy_vol,
                sell_vol: self.current_sell_vol,
            };
            if self.buckets.len() >= self.max_buckets {
                self.buckets.pop_front();
            }
            self.buckets.push_back(bucket);

            let overflow = (self.current_buy_vol + self.current_sell_vol) - self.bucket_vol_threshold;
            self.current_buy_vol = 0.0;
            self.current_sell_vol = 0.0;

            if overflow > 0.0 {
                if is_buy {
                    self.current_buy_vol = overflow;
                } else {
                    self.current_sell_vol = overflow;
                }
            }

            let raw = self.compute_raw();
            const EMA_ALPHA: f64 = 0.15;
            if self.buckets.len() == 1 && (self.ema_vpin - 0.0).abs() < f64::EPSILON {
                self.ema_vpin = raw;
            } else {
                self.ema_vpin = EMA_ALPHA * raw + (1.0 - EMA_ALPHA) * self.ema_vpin;
            }
        }
    }

    fn compute_raw(&self) -> f64 {
        if self.buckets.is_empty() {
            return 0.0;
        }
        let mut abs_imb_sum = 0.0;
        let mut total_vol = 0.0;
        for b in &self.buckets {
            abs_imb_sum += (b.buy_vol - b.sell_vol).abs();
            total_vol += b.buy_vol + b.sell_vol;
        }
        if total_vol > 0.0 {
            abs_imb_sum / total_vol
        } else {
            0.0
        }
    }

    fn vpin(&self) -> f64 {
        self.ema_vpin
    }

    fn reset(&mut self) {
        self.buckets.clear();
        self.current_buy_vol = 0.0;
        self.current_sell_vol = 0.0;
        self.ema_vpin = 0.0;
    }
}

/// Dual EMA spread regime tracker.
/// Detects spread narrowing: when the fast EMA drops below 80% of the slow EMA,
/// market makers are tightening quotes -- a pre-directional signal.
#[derive(Clone, Debug)]
struct SpreadRegime {
    ema_fast: f64,
    ema_slow: f64,
    initialized: bool,
}

impl SpreadRegime {
    fn new() -> Self {
        Self {
            ema_fast: 0.0,
            ema_slow: 0.0,
            initialized: false,
        }
    }

    fn update(&mut self, spread_bps: f64) {
        if spread_bps <= 0.0 {
            return;
        }
        const ALPHA_FAST: f64 = 0.15;
        const ALPHA_SLOW: f64 = 0.03;
        if !self.initialized {
            self.ema_fast = spread_bps;
            self.ema_slow = spread_bps;
            self.initialized = true;
        } else {
            self.ema_fast = ALPHA_FAST * spread_bps + (1.0 - ALPHA_FAST) * self.ema_fast;
            self.ema_slow = ALPHA_SLOW * spread_bps + (1.0 - ALPHA_SLOW) * self.ema_slow;
        }
    }

    fn is_narrowing(&self) -> bool {
        self.initialized && self.ema_slow > 0.0 && self.ema_fast < self.ema_slow * 0.80
    }

    fn reset(&mut self) {
        self.ema_fast = 0.0;
        self.ema_slow = 0.0;
        self.initialized = false;
    }
}

/// Tracks microprice vs OFI directional divergence.
/// When microprice moves up but OFI moves down (or vice versa) for a sustained period,
/// this signals a potential reversal.
#[derive(Clone, Debug)]
struct OfiDivergence {
    prev_microprice: f64,
    prev_ofi: f64,
    last_mp_delta: f64,
    last_ofi_delta: f64,
    divergence_start: Option<tokio::time::Instant>,
    initialized: bool,
}

impl OfiDivergence {
    fn new() -> Self {
        Self {
            prev_microprice: 0.0,
            prev_ofi: 0.0,
            last_mp_delta: 0.0,
            last_ofi_delta: 0.0,
            divergence_start: None,
            initialized: false,
        }
    }

    fn update(&mut self, microprice: f64, ofi: f64) {
        if !self.initialized {
            self.prev_microprice = microprice;
            self.prev_ofi = ofi;
            self.initialized = true;
            return;
        }
        self.last_mp_delta = microprice - self.prev_microprice;
        self.last_ofi_delta = ofi - self.prev_ofi;
        let diverging = (self.last_mp_delta > 0.0 && self.last_ofi_delta < 0.0)
            || (self.last_mp_delta < 0.0 && self.last_ofi_delta > 0.0);
        if diverging {
            if self.divergence_start.is_none() {
                self.divergence_start = Some(tokio::time::Instant::now());
            }
        } else {
            self.divergence_start = None;
        }
        self.prev_microprice = microprice;
        self.prev_ofi = ofi;
    }

    fn check_alert(&mut self, threshold_secs: u64) -> Option<(&'static str, &'static str)> {
        if let Some(start) = self.divergence_start {
            if start.elapsed().as_secs() >= threshold_secs {
                self.divergence_start = None;
                let mp_dir = if self.last_mp_delta >= 0.0 { "UP" } else { "DOWN" };
                let ofi_dir = if self.last_ofi_delta >= 0.0 { "UP" } else { "DOWN" };
                return Some((mp_dir, ofi_dir));
            }
        }
        None
    }
}

fn is_crypto_symbol(symbol: &str) -> bool {
    symbol.contains('/') || symbol.starts_with("BTC") || symbol.starts_with("ETH")
}

fn stream_config(symbol: &str) -> (String, serde_json::Value) {
    if is_crypto_symbol(symbol) {
        let url = "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string();
        let sub = json!({ "action": "subscribe", "trades": [symbol], "quotes": [symbol], "orderbooks": [symbol] });
        (url, sub)
    } else {
        let url = "wss://stream.data.alpaca.markets/v2/sip".to_string();
        let sub = json!({ "action": "subscribe", "trades": [symbol], "quotes": [symbol] });
        (url, sub)
    }
}

fn val_to_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
}

fn parse_quote(item: &serde_json::Value) -> Option<QuoteTop> {
    let bid = item
        .get("bp")
        .and_then(val_to_f64)
        .or_else(|| item.get("bid_price").and_then(val_to_f64))
        .or_else(|| item.get("b").and_then(val_to_f64))
        .unwrap_or(0.0);
    let ask = item
        .get("ap")
        .and_then(val_to_f64)
        .or_else(|| item.get("ask_price").and_then(val_to_f64))
        .or_else(|| item.get("a").and_then(val_to_f64))
        .unwrap_or(0.0);
    let bid_size = item
        .get("bs")
        .and_then(val_to_f64)
        .or_else(|| item.get("bid_size").and_then(val_to_f64))
        .unwrap_or(0.0);
    let ask_size = item
        .get("as")
        .and_then(val_to_f64)
        .or_else(|| item.get("ask_size").and_then(val_to_f64))
        .unwrap_or(0.0);

    if bid > 0.0 || ask > 0.0 {
        Some(QuoteTop {
            bid,
            ask,
            bid_size,
            ask_size,
        })
    } else {
        None
    }
}

fn parse_trade_price(item: &serde_json::Value) -> Option<f64> {
    item.get("p")
        .and_then(val_to_f64)
        .or_else(|| item.get("price").and_then(val_to_f64))
}

fn parse_trade_size(item: &serde_json::Value) -> Option<f64> {
    item.get("s")
        .and_then(val_to_f64)
        .or_else(|| item.get("size").and_then(val_to_f64))
        .or_else(|| item.get("q").and_then(val_to_f64))
}

fn parse_ts(item: &serde_json::Value) -> Option<i64> {
    if let Some(ts) = item.get("t").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            return Some(dt.timestamp());
        }
    }
    if let Some(v) = item.get("t").and_then(|v| v.as_i64()) {
        if v > 1_000_000_000_000 {
            return Some(v / 1_000_000_000);
        }
        return Some(v);
    }
    None
}

/// Lee-Ready trade classification with tick test fallback.
/// 1. At or above ask => buy. At or below bid => sell.
/// 2. Inside spread: above mid => buy, below mid => sell.
/// 3. At mid: tick test using previous trade price.
fn classify_trade(price: f64, quote: &QuoteTop, last_trade_price: f64) -> Option<bool> {
    if quote.bid <= 0.0 || quote.ask <= 0.0 {
        return None;
    }
    if price >= quote.ask {
        Some(true)
    } else if price <= quote.bid {
        Some(false)
    } else {
        let mid = (quote.bid + quote.ask) * 0.5;
        if price > mid {
            Some(true)
        } else if price < mid {
            Some(false)
        } else if last_trade_price > 0.0
            && (price - last_trade_price).abs() > f64::EPSILON
        {
            Some(price > last_trade_price)
        } else if last_trade_price <= 0.0 {
            Some(true) // First trade at mid: tick-up convention
        } else {
            None
        }
    }
}

pub async fn run_streamer(
    state: std::sync::Arc<TitanState>,
    mut rx: Receiver<StreamCmd>,
    app: AppHandle,
) {
    let my_generation = state.current_generation();
    tracing::info!(
        component = "streamer",
        generation = my_generation,
        "Streamer started"
    );
    let mut symbol = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        if s.active_symbol.is_empty() {
            "BTC/USD".to_string()
        } else {
            s.active_symbol.clone()
        }
    };
    let mut quote = QuoteTop::default();
    let mut flow = FlowStats::default();
    let mut stream_vpin = StreamVpin::new(5000.0, 20);
    let mut spread_regime = SpreadRegime::new();
    let mut ofi_divergence = OfiDivergence::new();
    let ofi_divergence_secs: u64 = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.ofi_divergence_threshold_secs
    };
    let mut last_emit = tokio::time::Instant::now();
    let mut last_l2_emit = tokio::time::Instant::now();
    let mut last_top_emit = tokio::time::Instant::now();
    let mut last_price_emit = tokio::time::Instant::now();

    let mut book_bids: BTreeMap<ordered_float::OrderedFloat<f64>, f64> = BTreeMap::new();
    let mut book_asks: BTreeMap<ordered_float::OrderedFloat<f64>, f64> = BTreeMap::new();
    let mut nbbo_bids: BTreeMap<ordered_float::OrderedFloat<f64>, f64> = BTreeMap::new();
    let mut nbbo_asks: BTreeMap<ordered_float::OrderedFloat<f64>, f64> = BTreeMap::new();
    let mut prev_bids: Vec<(f64, f64)> = Vec::with_capacity(20);
    let mut prev_asks: Vec<(f64, f64)> = Vec::with_capacity(20);
    let mut bids_top_buf: Vec<(f64, f64)> = Vec::with_capacity(20);
    let mut asks_top_buf: Vec<(f64, f64)> = Vec::with_capacity(20);
    let mut _ofi_msg_count: u64 = 0;

    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(10));
    let mut backoff_secs: u64 = 1;
    loop {
        if state.shutdown_token.is_cancelled() {
            break;
        }
        if state.current_generation() != my_generation {
            tracing::warn!(
                component = "streamer",
                my_gen = my_generation,
                current_gen = state.current_generation(),
                "Generation mismatch — exiting stale streamer instance"
            );
            state.generation_mismatch_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            break;
        }
        state.watchdog.heartbeat("streamer", 0);
        let (key, secret) = {
            let k = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (k.key.clone(), k.secret.clone())
        };

        // [FIX] Update Status in Redis to CONNECTING
        if let Ok(redis) = state.redis.lock() {
            monitor::set_redis_value(redis.as_ref(), "system:status", "CONNECTING");
        }

        if key.is_empty() || secret.is_empty() {
            crate::state::log_to_file(&format!("⚠️ STREAMER: Keys missing, retrying in {}s...", backoff_secs));
            sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(60);
            continue;
        }

        let (url, subscribe) = stream_config(&symbol);
        crate::state::log_to_file(&format!("🚀 STREAMER: Connecting to {}...", url));
        let ws = connect_async(&url).await;
        let Ok((mut socket, _)) = ws else {
            crate::state::log_to_file(&format!("❌ STREAMER: Connection Failed, retrying in {}s...", backoff_secs));
            sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(60);
            continue;
        };

        let auth = json!({ "action": "auth", "key": key, "secret": secret });
        crate::state::log_to_file("🔑 STREAMER: Sending Auth...");
        if let Err(e) = socket
            .send(tokio_tungstenite::tungstenite::Message::Text(
                auth.to_string(),
            ))
            .await
        {
            crate::state::log_to_file(&format!("❌ STREAMER: Auth Send Error: {}, retrying in {}s...", e, backoff_secs));
            sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(60);
            continue;
        }

        // Wait for Auth Response (Loop to skip 'connected' message)
        let mut authenticated = false;
        let start = tokio::time::Instant::now();
        crate::state::log_to_file("⏳ STREAMER: Waiting for Auth response...");

        while start.elapsed() < Duration::from_secs(10) {
            match tokio::time::timeout(Duration::from_secs(2), socket.next()).await {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(msg)))) => {
                    crate::state::log_to_file(&format!("📩 STREAMER: Recv: {}", msg));
                    let parsed = serde_json::from_str::<Value>(&msg);
                    let items: Vec<&Value> = match &parsed {
                        Ok(Value::Array(arr)) => arr.iter().collect(),
                        Ok(v) => vec![v],
                        Err(_) => Vec::new(),
                    };
                    let mut handled = false;
                    for item in &items {
                        let msg_field = item.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                        let code_field = item.get("code").and_then(|v| v.as_u64());
                        if msg_field == "authenticated" {
                            authenticated = true;
                            handled = true;
                            break;
                        } else if msg_field == "connected" {
                            crate::state::log_to_file(
                                "ℹ️ STREAMER: Handshake received, waiting for auth...",
                            );
                            handled = true;
                        } else if msg_field == "connection limit exceeded" || code_field == Some(406) {
                            crate::state::log_to_file(
                                "❌ STREAMER: Connection Limit Exceeded! Sleeping 60s...",
                            );
                            sleep(Duration::from_secs(60)).await;
                            handled = true;
                            break;
                        } else if msg_field == "auth failed" || code_field == Some(404) {
                            crate::state::log_to_file("❌ STREAMER: Auth Failed/Timeout. Retrying...");
                            handled = true;
                            break;
                        }
                    }
                    if authenticated || (!handled && !items.is_empty()) {
                        break;
                    }
                }
                Ok(Some(Err(e))) => {
                    crate::state::log_to_file(&format!("❌ STREAMER: Socket Error: {}", e));
                    break;
                }
                Ok(None) => {
                    crate::state::log_to_file("❌ STREAMER: Socket Closed");
                    break;
                }
                Err(_) => {
                    continue;
                }
                _ => {}
            }
        }

        if !authenticated {
            crate::state::log_to_file(&format!("❌ STREAMER: Auth Failed, retrying in {}s...", backoff_secs));
            sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(60);
            continue;
        }

        crate::state::log_to_file("✅ STREAMER: Authenticated! Sending Subscription...");
        backoff_secs = 1;
        state.market.mark_stream_reconnected();

        // [FIX] Update Redis Status to LIVE
        if let Ok(redis) = state.redis.lock() {
            monitor::set_redis_value(redis.as_ref(), "system:status", "LIVE");
        }
        let _ = app.emit_all("connection-status", "LIVE"); // Emit direct event just in case

        if let Err(e) = socket
            .send(tokio_tungstenite::tungstenite::Message::Text(
                subscribe.to_string(),
            ))
            .await
        {
            crate::state::log_to_file(&format!("❌ STREAMER: Subscribe Send Error: {} -- reconnecting", e));
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        loop {
            if state.shutdown_token.is_cancelled() {
                break;
            }
            tokio::select! {
                _ = state.shutdown_token.cancelled() => { break; }
                Some(cmd) = rx.recv() => {
                    match cmd {
                        StreamCmd::SetSymbol(sym) => {
                            symbol = sym;
                            quote = QuoteTop::default();
                            flow = FlowStats::default();
                            stream_vpin.reset();
                            spread_regime.reset();
                            book_bids.clear();
                            book_asks.clear();
                            nbbo_bids.clear();
                            nbbo_asks.clear();
                            prev_bids.clear();
                            prev_asks.clear();
                            _ofi_msg_count = 0;
                            ofi_divergence = OfiDivergence::new();
                        }
                        StreamCmd::Reconnect => {}
                    }
                    break;
                }
                _ = heartbeat_interval.tick() => {
                    state.watchdog.heartbeat("streamer", 0);
                }
                msg = socket.next() => {
                    match msg {
                        Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                            let _msg_t0 = std::time::Instant::now();
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(arr) = val.as_array() {
                                    for item in arr {
                                        let item: &Value = item;
                                        let sym = item.get("S").and_then(|v: &Value| v.as_str()).unwrap_or("");
                                        if sym.is_empty() { continue; }

                                        let mut saw_trade = false;
                                        if let Some(q) = parse_quote(item) {
                                            quote = q;
                                            if (quote.bid > 0.0 || quote.ask > 0.0)
                                                && last_top_emit.elapsed() >= Duration::from_millis(200) {
                                                let spread = if quote.bid > 0.0 && quote.ask > 0.0 {
                                                    quote.ask - quote.bid
                                                } else { 0.0 };
                                                let payload = serde_json::json!({
                                                    "symbol": sym,
                                                    "bid": quote.bid,
                                                    "ask": quote.ask,
                                                    "bid_size": quote.bid_size,
                                                    "ask_size": quote.ask_size,
                                                    "spread": spread
                                                });
                                                let _ = app.emit_all("l2-top", payload);
                                                last_top_emit = tokio::time::Instant::now();
                                            }
                                        }

                                        if let Some(price) = parse_trade_price(item) {
                                            let size = parse_trade_size(item).unwrap_or(0.0);
                                            let notional = if size > 0.0 { price * size } else { price };
                                            saw_trade = true;
                                            let prev_price = flow.last_price;
                                            flow.last_price = price;
                                            flow.total_trades += 1;
                                            flow.update_intensity(tokio::time::Instant::now());
                                            {
                                                let mut prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                                                prices.insert(normalize_symbol(sym), price);
                                            }
                                            state.market.mark_stream_data_received();
                                            let ts = parse_ts(item).unwrap_or_else(|| chrono::Utc::now().timestamp());
                                            let payload = serde_json::json!({
                                                "symbol": sym,
                                                "price": price,
                                                "size": size,
                                                "time": ts,
                                                "source": "trade"
                                            });
                                            let _ = app.emit_all("price-update", payload);
                                            if let Ok(redis_guard) = state.redis.lock() {
                                                monitor::set_redis_value(redis_guard.as_ref(), "market:active_symbol", sym);
                                            }

                                            if let Some(is_buy) = classify_trade(price, &quote, prev_price) {
                                                flow.classified_trades += 1;
                                                if is_buy {
                                                    flow.buy_count += 1;
                                                    flow.buy_notional += notional;
                                                } else {
                                                    flow.sell_count += 1;
                                                    flow.sell_notional += notional;
                                                }
                                                let trade_vol = size.max(notional);
                                                stream_vpin.update(is_buy, trade_vol);
                                                {
                                                    let signed_vol = if is_buy { trade_vol } else { -trade_vol };
                                                    let mut cvd_map = state.market.cvd.lock().unwrap_or_else(|e| e.into_inner());
                                                    *cvd_map.entry(normalize_symbol(sym)).or_insert(0.0) += signed_vol;
                                                }
                                            }

                                            {
                                                let active_tf = {
                                                    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                                    s.manual_res.clone()
                                                };
                                                let norm_tf = normalize_timeframe_internal(&active_tf);
                                                let tf_s = timeframe_to_secs(&norm_tf);
                                                let buf_key = cache_key_symbol_tf(sym, &active_tf);
                                                let mut bufs = state.market.candle_buffers.lock().unwrap_or_else(|e| e.into_inner());
                                                bufs.entry(buf_key)
                                                    .or_insert_with(|| CandleRingBuffer::new(tf_s))
                                                    .update_trade(price, size, ts);
                                            }

                                            {
                                                let now = Instant::now();
                                                let event = {
                                                    let mut fc = state.risk.flash_crash.lock().unwrap_or_else(|e| e.into_inner());
                                                    fc.check_trade(sym, price, now)
                                                };
                                                if let Some(ev) = event {
                                                    tracing::warn!(
                                                        component = "flash_crash",
                                                        symbol = %ev.symbol,
                                                        change_pct = %format!("{:.2}", ev.change_pct),
                                                        "FLASH CRASH DETECTED: {} moved {:.2}% in {}s",
                                                        ev.symbol, ev.change_pct, ev.window_secs,
                                                    );
                                                    let _ = app.emit_all("flash-crash", serde_json::json!({
                                                        "symbol": ev.symbol,
                                                        "change_pct": ev.change_pct,
                                                        "window_secs": ev.window_secs,
                                                    }));
                                                    let position_info: Option<(f64, String)> = {
                                                        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                                        s.positions.iter()
                                                            .find(|p| p.symbol == ev.symbol && p.qty.abs() > Decimal::ZERO)
                                                            .map(|p| {
                                                                let qty = p.qty.abs().to_string().parse::<f64>().unwrap_or(0.0);
                                                                let close_side = match p.side {
                                                                    crate::app_state::OrderSide::Buy => "sell".to_string(),
                                                                    crate::app_state::OrderSide::Sell => "buy".to_string(),
                                                                };
                                                                (qty, close_side)
                                                            })
                                                    };
                                                    if let Some((qty, close_side)) = position_info {
                                                        let st = state.clone();
                                                        let ap = app.clone();
                                                        let sym_owned = ev.symbol.clone();
                                                        tokio::spawn(async move {
                                                            tracing::warn!(component = "flash_crash", symbol = %sym_owned, "Force-closing position");
                                                            let _ = crate::commands::data::close_position(
                                                                st, &sym_owned, qty, &close_side, &ap,
                                                            ).await;
                                                        });
                                                    }
                                                    #[cfg(feature = "telegram")]
                                                    {
                                                        let (tg_token, tg_chat, tg_enabled) = {
                                                            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                                            (s.telegram_bot_token.clone(), s.telegram_chat_id.clone(), s.telegram_enabled)
                                                        };
                                                        if tg_enabled {
                                                            let hc = state.http_client.clone();
                                                            let msg = format!(
                                                                "🚨 FLASH CRASH: {} moved {:.2}% in {}s. Position auto-closed.",
                                                                ev.symbol, ev.change_pct, ev.window_secs,
                                                            );
                                                            tokio::spawn(async move {
                                                                crate::telegram::send_telegram_alert(&hc, &tg_token, &tg_chat, &msg).await;
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if !saw_trade && quote.bid > 0.0 && quote.ask > 0.0
                                            && last_price_emit.elapsed() >= Duration::from_millis(500) {
                                            let mid = (quote.bid + quote.ask) / 2.0;
                                            {
                                                let mut prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                                                prices.insert(normalize_symbol(sym), mid);
                                            }
                                            let ts = parse_ts(item).unwrap_or_else(|| chrono::Utc::now().timestamp());
                                            let payload = serde_json::json!({
                                                "symbol": sym,
                                                "price": mid,
                                                "time": ts,
                                                "source": "quote"
                                            });
                                            let _ = app.emit_all("price-update", payload);
                                            last_price_emit = tokio::time::Instant::now();
                                        }

                                        let item_type = item.get("T").and_then(|v: &Value| v.as_str()).unwrap_or("");

                                        if item_type == "o" || item_type == "b" {
                                            let is_reset = item.get("r").and_then(|v| v.as_bool()).unwrap_or(false);
                                            if is_reset {
                                                book_bids.clear();
                                                book_asks.clear();
                                            }

                                            if let Some(b_list) = item.get("b").and_then(|v: &Value| v.as_array()) {
                                                for b in b_list {
                                                    let p = b.get("p").and_then(val_to_f64).unwrap_or(0.0);
                                                    let s = b.get("s").and_then(val_to_f64).unwrap_or(0.0);
                                                    let key = ordered_float::OrderedFloat(p);
                                                    if s == 0.0 { book_bids.remove(&key); }
                                                    else if p > 0.0 { book_bids.insert(key, s); }
                                                }
                                            }
                                            if let Some(a_list) = item.get("a").and_then(|v: &Value| v.as_array()) {
                                                for a in a_list {
                                                    let p = a.get("p").and_then(val_to_f64).unwrap_or(0.0);
                                                    let s = a.get("s").and_then(val_to_f64).unwrap_or(0.0);
                                                    let key = ordered_float::OrderedFloat(p);
                                                    if s == 0.0 { book_asks.remove(&key); }
                                                    else if p > 0.0 { book_asks.insert(key, s); }
                                                }
                                            }

                                            bids_top_buf.clear();
                                            bids_top_buf.extend(book_bids.iter().rev().take(20)
                                                .map(|(p, s)| (p.into_inner(), *s)));
                                            asks_top_buf.clear();
                                            asks_top_buf.extend(book_asks.iter().take(20)
                                                .map(|(p, s)| (p.into_inner(), *s)));

                                            if !prev_bids.is_empty() && !prev_asks.is_empty() {
                                                let ofi = risk::calculate_ofi(&bids_top_buf, &asks_top_buf, &prev_bids, &prev_asks, 10);
                                                let bid_depth: f64 = bids_top_buf.iter().map(|(_, s)| s).sum();
                                                let ask_depth: f64 = asks_top_buf.iter().map(|(_, s)| s).sum();
                                                let total = bid_depth + ask_depth;
                                                let imbalance = if total > 0.0 { (bid_depth - ask_depth) / total } else { 0.0 };
                                                let best_bid = bids_top_buf.first().map(|(p, _)| *p).unwrap_or(0.0);
                                                let best_ask = asks_top_buf.first().map(|(p, _)| *p).unwrap_or(0.0);
                                                let spread_bps = if best_bid > 0.0 { ((best_ask - best_bid) / best_bid) * 10000.0 } else { 0.0 };

                                                {
                                                    let prev = state.market.ofi_state.load();
                                                    let alpha = 0.3;
                                                    let new_ofi = prev.current_ofi * (1.0 - alpha) + ofi * alpha;
                                                    let new_count = prev.update_count + 1;
                                                    spread_regime.update(spread_bps);
                                                    let bid_sz_top = bids_top_buf.first().map(|(_, s)| *s).unwrap_or(0.0);
                                                    let ask_sz_top = asks_top_buf.first().map(|(_, s)| *s).unwrap_or(0.0);
                                                    let microprice = crate::l2_heatmap::compute_microprice(best_bid, best_ask, bid_sz_top, ask_sz_top);
                                                    state.market.ofi_state.store(std::sync::Arc::new(crate::l2_heatmap::OfiState {
                                                        current_ofi: new_ofi,
                                                        best_bid,
                                                        best_ask,
                                                        spread_bps,
                                                        bid_depth_total: bid_depth,
                                                        ask_depth_total: ask_depth,
                                                        imbalance_ratio: imbalance,
                                                        update_count: new_count,
                                                        last_update: Some(Instant::now()),
                                                        stream_vpin: stream_vpin.vpin(),
                                                        spread_ema_fast: spread_regime.ema_fast,
                                                        spread_ema_slow: spread_regime.ema_slow,
                                                        spread_narrowing: spread_regime.is_narrowing(),
                                                        microprice,
                                                    }));

                                                    ofi_divergence.update(microprice, new_ofi);

                                                    if new_count.is_multiple_of(10) {
                                                        let _ = app.emit_all("ofi-update", json!({
                                                            "current_ofi": new_ofi,
                                                            "spread_bps": spread_bps,
                                                            "bid_depth_total": bid_depth,
                                                            "ask_depth_total": ask_depth,
                                                            "imbalance_ratio": imbalance,
                                                            "stream_vpin": stream_vpin.vpin(),
                                                            "spread_narrowing": spread_regime.is_narrowing(),
                                                            "microprice": microprice,
                                                        }));
                                                        if let Some(alert) = ofi_divergence.check_alert(ofi_divergence_secs) {
                                                            let _ = app.emit_all("ofi-divergence", json!({
                                                                "microprice_dir": alert.0,
                                                                "ofi_dir": alert.1,
                                                            }));
                                                        }
                                                    }
                                                }
                                            }

                                            if last_l2_emit.elapsed() >= Duration::from_millis(200) {
                                                let l2 = L2Update {
                                                    bids: bids_top_buf.iter().take(10).map(|(p, s)| L2Level { p: *p, s: *s }).collect(),
                                                    asks: asks_top_buf.iter().take(10).map(|(p, s)| L2Level { p: *p, s: *s }).collect(),
                                                };
                                                let _ = app.emit_all("l2-update", &l2);
                                                last_l2_emit = tokio::time::Instant::now();
                                            }

                                            if let (Some(&(bb, _)), Some(&(ba, _))) = (bids_top_buf.first(), asks_top_buf.first()) {
                                                if bb > 0.0 || ba > 0.0 {
                                                    let spread = if bb > 0.0 && ba > 0.0 { ba - bb } else { 0.0 };
                                                    let bs = bids_top_buf.first().map(|(_, s)| *s).unwrap_or(0.0);
                                                    let az = asks_top_buf.first().map(|(_, s)| *s).unwrap_or(0.0);
                                                    let _ = app.emit_all("l2-top", json!({
                                                        "symbol": sym, "bid": bb, "ask": ba,
                                                        "bid_size": bs, "ask_size": az, "spread": spread
                                                    }));
                                                    last_top_emit = tokio::time::Instant::now();
                                                }
                                            }

                                            prev_bids.clear();
                                            prev_bids.extend_from_slice(&bids_top_buf);
                                            prev_asks.clear();
                                            prev_asks.extend_from_slice(&asks_top_buf);
                                            _ofi_msg_count += 1;
                                        }

                                        if item_type == "q" {
                                            let bp = item.get("bp").and_then(val_to_f64).unwrap_or(0.0);
                                            let ap = item.get("ap").and_then(val_to_f64).unwrap_or(0.0);
                                            let bs = item.get("bs").and_then(val_to_f64).unwrap_or(0.0);
                                            let az = item.get("as").and_then(val_to_f64).unwrap_or(0.0);
                                            if bp > 0.0 && ap > 0.0 {
                                                {
                                                    let prev = state.market.ofi_state.load();
                                                    let prev_bid_sz = prev.bid_depth_total;
                                                    let prev_ask_sz = prev.ask_depth_total;
                                                    let spread_bps_q = ((ap - bp) / bp) * 10000.0;
                                                    let total = bs + az;
                                                    let imbalance = if total > 0.0 { (bs - az) / total } else { 0.0 };
                                                    let bid_delta = bs - prev_bid_sz;
                                                    let ask_delta = az - prev_ask_sz;
                                                    let ofi_tick = bid_delta - ask_delta;
                                                    let alpha = 0.3;
                                                    let new_ofi = prev.current_ofi * (1.0 - alpha) + ofi_tick * alpha;
                                                    let new_count = prev.update_count + 1;
                                                    spread_regime.update(spread_bps_q);
                                                    let microprice_q = crate::l2_heatmap::compute_microprice(bp, ap, bs, az);
                                                    state.market.ofi_state.store(std::sync::Arc::new(crate::l2_heatmap::OfiState {
                                                        current_ofi: new_ofi,
                                                        best_bid: bp,
                                                        best_ask: ap,
                                                        spread_bps: spread_bps_q,
                                                        bid_depth_total: bs,
                                                        ask_depth_total: az,
                                                        imbalance_ratio: imbalance,
                                                        update_count: new_count,
                                                        last_update: Some(Instant::now()),
                                                        stream_vpin: stream_vpin.vpin(),
                                                        spread_ema_fast: spread_regime.ema_fast,
                                                        spread_ema_slow: spread_regime.ema_slow,
                                                        spread_narrowing: spread_regime.is_narrowing(),
                                                        microprice: microprice_q,
                                                    }));
                                                    ofi_divergence.update(microprice_q, new_ofi);
                                                    if new_count.is_multiple_of(20) {
                                                        let _ = app.emit_all("ofi-update", json!({
                                                            "current_ofi": new_ofi,
                                                            "spread_bps": spread_bps_q,
                                                            "bid_depth_total": bs,
                                                            "ask_depth_total": az,
                                                            "imbalance_ratio": imbalance,
                                                            "stream_vpin": stream_vpin.vpin(),
                                                            "spread_narrowing": spread_regime.is_narrowing(),
                                                            "microprice": microprice_q,
                                                        }));
                                                    }
                                                }

                                                if book_bids.is_empty() && book_asks.is_empty() {
                                                    let key_b = ordered_float::OrderedFloat(bp);
                                                    let key_a = ordered_float::OrderedFloat(ap);
                                                    nbbo_bids.insert(key_b, bs);
                                                    nbbo_asks.insert(key_a, az);
                                                    const MAX_NBBO: usize = 10;
                                                    while nbbo_bids.len() > MAX_NBBO {
                                                        if let Some(k) = nbbo_bids.keys().next().cloned() { nbbo_bids.remove(&k); }
                                                    }
                                                    while nbbo_asks.len() > MAX_NBBO {
                                                        if let Some(k) = nbbo_asks.keys().next_back().cloned() { nbbo_asks.remove(&k); }
                                                    }
                                                    if last_l2_emit.elapsed() >= Duration::from_millis(500) {
                                                        let synth_bids: Vec<L2Level> = nbbo_bids.iter().rev().take(10)
                                                            .map(|(p, s)| L2Level { p: p.into_inner(), s: *s }).collect();
                                                        let synth_asks: Vec<L2Level> = nbbo_asks.iter().take(10)
                                                            .map(|(p, s)| L2Level { p: p.into_inner(), s: *s }).collect();
                                                        let l2 = L2Update { bids: synth_bids, asks: synth_asks };
                                                        let _ = app.emit_all("l2-update", &l2);
                                                        last_l2_emit = tokio::time::Instant::now();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            state.latency_metrics.record_streamer(_msg_t0.elapsed().as_micros() as u64);
                        }
                        Some(Ok(_)) => {}
                        Some(Err(_)) | None => {
                            break;
                        }
                    }
                }
            }

            if last_emit.elapsed() >= Duration::from_millis(500) {
                let sym_cvd = {
                    let cvd_map = state.market.cvd.lock().unwrap_or_else(|e| e.into_inner());
                    cvd_map.get(&normalize_symbol(&symbol)).copied().unwrap_or(0.0)
                };
                let payload = serde_json::json!({
                    "buy_count": flow.buy_count,
                    "sell_count": flow.sell_count,
                    "buy_notional": flow.buy_notional,
                    "sell_notional": flow.sell_notional,
                    "last_price": flow.last_price,
                    "cvd": sym_cvd,
                    "classified_pct": flow.classified_pct(),
                    "trade_intensity": flow.trade_intensity_ema,
                });
                let _ = app.emit_all("flow-update", payload);
                last_emit = tokio::time::Instant::now();
            }
        }

        state.market.mark_stream_stale();
        backoff_secs = 2;
        sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Lee-Ready classification tests ──

    #[test]
    fn classify_at_ask_is_buy() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        assert_eq!(classify_trade(101.0, &q, 0.0), Some(true));
        assert_eq!(classify_trade(101.50, &q, 0.0), Some(true));
    }

    #[test]
    fn classify_at_bid_is_sell() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        assert_eq!(classify_trade(100.0, &q, 0.0), Some(false));
        assert_eq!(classify_trade(99.50, &q, 0.0), Some(false));
    }

    #[test]
    fn classify_above_mid_is_buy() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        // mid = 100.5, price = 100.75 => above mid => buy
        assert_eq!(classify_trade(100.75, &q, 0.0), Some(true));
    }

    #[test]
    fn classify_below_mid_is_sell() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        // mid = 100.5, price = 100.25 => below mid => sell
        assert_eq!(classify_trade(100.25, &q, 0.0), Some(false));
    }

    #[test]
    fn classify_at_mid_uses_tick_test() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        // mid = 100.5, price = 100.5, last_trade = 100.0 => uptick => buy
        assert_eq!(classify_trade(100.5, &q, 100.0), Some(true));
        // mid = 100.5, price = 100.5, last_trade = 101.0 => downtick => sell
        assert_eq!(classify_trade(100.5, &q, 101.0), Some(false));
        // no previous trade at mid => tick-up convention (buy)
        assert_eq!(classify_trade(100.5, &q, 0.0), Some(true));
    }

    #[test]
    fn classify_invalid_quote_returns_none() {
        let q = QuoteTop { bid: 0.0, ask: 0.0, bid_size: 0.0, ask_size: 0.0 };
        assert_eq!(classify_trade(100.0, &q, 99.0), None);
    }

    // ── Flow stats tests ──

    #[test]
    fn flow_classified_pct_correct() {
        let mut f = FlowStats::default();
        f.total_trades = 10;
        f.classified_trades = 8;
        let pct = f.classified_pct();
        assert!((pct - 80.0).abs() < 0.01, "Expected 80%, got {}", pct);
    }

    #[test]
    fn flow_classified_pct_zero_trades() {
        let f = FlowStats::default();
        assert!((f.classified_pct() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn flow_trade_intensity_increases_on_burst() {
        let mut f = FlowStats::default();
        let start = tokio::time::Instant::now();
        f.update_intensity(start);
        let initial = f.trade_intensity_ema;
        // Simulate rapid trades (10ms apart)
        for i in 1..=20 {
            f.update_intensity(start + Duration::from_millis(i * 10));
        }
        assert!(
            f.trade_intensity_ema > initial,
            "Intensity should increase on burst: initial={}, final={}",
            initial, f.trade_intensity_ema
        );
    }

    // ── OFI Divergence tests ──

    #[test]
    fn ofi_divergence_detects_opposing_directions() {
        let mut d = OfiDivergence::new();
        d.update(100.0, 0.5);
        d.update(101.0, -0.5); // microprice UP, OFI DOWN
        assert!(d.divergence_start.is_some());
    }

    #[test]
    fn ofi_divergence_clears_on_agreement() {
        let mut d = OfiDivergence::new();
        d.update(100.0, 0.5);
        d.update(101.0, -0.5); // diverge
        assert!(d.divergence_start.is_some());
        d.update(102.0, 0.5); // both UP => no divergence
        assert!(d.divergence_start.is_none());
    }

    #[test]
    fn ofi_divergence_alert_uses_delta_not_level() {
        let mut d = OfiDivergence::new();
        d.update(150.0, 10.0); // init: microprice=$150, OFI=10
        d.update(149.0, 11.0); // mp DOWN, OFI UP => diverge
        assert!(d.divergence_start.is_some());
        // check_alert should report mp=DOWN (delta <0), not UP (level >0)
        // Can't trigger time-based alert in unit test, but verify deltas are stored
        assert!(d.last_mp_delta < 0.0, "mp delta should be negative (price fell)");
        assert!(d.last_ofi_delta > 0.0, "ofi delta should be positive (OFI rose)");
    }

    #[test]
    fn classify_first_trade_at_mid_is_buy() {
        let q = QuoteTop { bid: 100.0, ask: 101.0, bid_size: 100.0, ask_size: 100.0 };
        // First trade (last_trade_price=0) at exactly mid => tick-up convention => buy
        assert_eq!(classify_trade(100.5, &q, 0.0), Some(true));
        // Negative last_trade_price (uninitialized) at mid => also buy
        assert_eq!(classify_trade(100.5, &q, -1.0), Some(true));
    }

    #[test]
    fn stream_vpin_balanced_flow() {
        let mut vpin = StreamVpin::new(100.0, 20);
        for _ in 0..50 {
            vpin.update(true, 10.0);
            vpin.update(false, 10.0);
        }
        assert!(
            vpin.vpin() < 0.15,
            "Balanced buy/sell flow should produce VPIN near 0, got {}",
            vpin.vpin()
        );
    }

    #[test]
    fn stream_vpin_informed_flow() {
        let mut vpin = StreamVpin::new(100.0, 20);
        for _ in 0..100 {
            vpin.update(true, 50.0);
        }
        assert!(
            vpin.vpin() > 0.85,
            "100% buy flow should produce VPIN near 1.0, got {}",
            vpin.vpin()
        );
    }

    #[test]
    fn stream_vpin_partial_classification() {
        let mut vpin = StreamVpin::new(100.0, 20);
        for _ in 0..50 {
            vpin.update(true, 12.0);
            vpin.update(false, 8.0);
        }
        let v = vpin.vpin();
        assert!(v > 0.0 && v < 1.0, "Mixed flow VPIN should be between 0 and 1, got {}", v);
        assert!(v < 0.5, "60/40 buy/sell split should produce moderate VPIN, got {}", v);
        assert!(!vpin.buckets.is_empty(), "Should have accumulated buckets");
    }

    #[test]
    fn stream_vpin_reset_clears_state() {
        let mut vpin = StreamVpin::new(100.0, 20);
        for _ in 0..50 {
            vpin.update(true, 50.0);
        }
        assert!(vpin.vpin() > 0.0);
        vpin.reset();
        assert!((vpin.vpin() - 0.0).abs() < f64::EPSILON);
        assert!(vpin.buckets.is_empty());
    }

    #[test]
    fn spread_regime_narrowing_detected() {
        let mut sr = SpreadRegime::new();
        for _ in 0..200 {
            sr.update(50.0);
        }
        assert!(!sr.is_narrowing(), "Stable spread should not be narrowing");
        for _ in 0..50 {
            sr.update(5.0);
        }
        assert!(
            sr.is_narrowing(),
            "Fast drop from 50bps to 5bps should detect narrowing (fast={:.2}, slow={:.2})",
            sr.ema_fast, sr.ema_slow
        );
    }

    #[test]
    fn spread_regime_widening_no_signal() {
        let mut sr = SpreadRegime::new();
        for _ in 0..200 {
            sr.update(10.0);
        }
        for _ in 0..200 {
            sr.update(50.0);
        }
        assert!(
            !sr.is_narrowing(),
            "Widening spread should not flag narrowing (fast={:.2}, slow={:.2})",
            sr.ema_fast, sr.ema_slow
        );
    }

    #[test]
    fn spread_regime_reset_clears_state() {
        let mut sr = SpreadRegime::new();
        sr.update(50.0);
        assert!(sr.initialized);
        sr.reset();
        assert!(!sr.initialized);
        assert!(!sr.is_narrowing());
    }

    #[test]
    fn spread_regime_ignores_zero_spread() {
        let mut sr = SpreadRegime::new();
        sr.update(0.0);
        assert!(!sr.initialized, "Zero spread should not initialize");
        sr.update(-5.0);
        assert!(!sr.initialized, "Negative spread should not initialize");
    }
}

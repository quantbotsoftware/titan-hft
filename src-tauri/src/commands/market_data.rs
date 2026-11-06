use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use crate::app_state::Candle;
use crate::error::TitanError;
use crate::indicators;
use crate::state::{BarsCacheEntry, TitanState};
use crate::types::*;

use super::order_exec::send_order_internal;

// ═══════════════════════════════════════════════════════════════
//  BARS DATA FETCHING
// ═══════════════════════════════════════════════════════════════

pub(crate) fn normalize_alpaca_tf(tf: &str) -> &'static str {
    match tf {
        "1m" => "1Min",
        "3m" => "3Min",
        "5m" => "5Min",
        "10m" => "10Min",
        "15m" => "15Min",
        "30m" => "30Min",
        "1h" => "1Hour",
        "4h" => "4Hour",
        "1d" => "1Day",
        _ => "5Min",
    }
}

#[tracing::instrument(skip(state), fields(symbol, tf))]
pub(crate) async fn fetch_bars_internal(
    state: Arc<TitanState>,
    symbol: &str,
    timeframe: &str,
) -> Result<Vec<Candle>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let symbol = symbol.trim().to_uppercase();
    let tf = normalize_alpaca_tf(timeframe);
    let cache_key = format!("{}|{}", symbol, tf);
    {
        let cache = state.market.bars_cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = cache.get(&cache_key) {
            if entry.updated.elapsed() < StdDuration::from_secs(30) {
                return Ok(entry.candles.clone());
            }
        }
    }

    let crypto_bases = [
        "BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "AVAX", "DOT", "LINK", "LTC", "BCH",
    ];
    let is_crypto = if symbol.contains('/') {
        true
    } else if symbol.ends_with("USD") && symbol.len() > 3 {
        let base = &symbol[..symbol.len() - 3];
        crypto_bases.contains(&base)
    } else {
        false
    };
    let client = state.http_client.clone();
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(365);
    let mut params: Vec<(String, String)> = vec![
        ("symbols".to_string(), symbol.clone()),
        ("timeframe".to_string(), tf.to_string()),
        ("limit".to_string(), "1000".to_string()),
        ("start".to_string(), start.to_rfc3339()),
        ("end".to_string(), end.to_rfc3339()),
        ("sort".to_string(), "desc".to_string()),
    ];
    if !is_crypto {
        let feed = "sip".to_string();
        params.push(("feed".to_string(), feed));
        let adjustment =
            std::env::var("TITAN_STOCK_ADJUSTMENT").unwrap_or_else(|_| "all".to_string());
        params.push(("adjustment".to_string(), adjustment));
    }
    let url = if is_crypto {
        "https://data.alpaca.markets/v1beta3/crypto/us/bars"
    } else {
        "https://data.alpaca.markets/v2/stocks/bars"
    };

    let _permit = crate::state::acquire_api_permit(&state).await?;
    let res = client
        .get(url)
        .query(&params)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Alpaca API Error {}: {}",
            status, txt
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| TitanError::Internal(format!("JSON Error: {}", e)))?;
    let bars_container = json
        .get("bars")
        .ok_or_else(|| TitanError::Internal("No 'bars' field in response".into()))?;
    let symbol_no_slash = symbol.replace("/", "");
    let bars_array = bars_container
        .get(symbol.as_str())
        .or_else(|| bars_container.get(symbol_no_slash.as_str()))
        .or_else(|| {
            if let Some(obj) = bars_container.as_object() {
                obj.values().next()
            } else {
                None
            }
        });

    if let Some(bars) = bars_array.and_then(|b| b.as_array()) {
        let mut candles: Vec<Candle> = bars
            .iter()
            .filter_map(|b| {
                let t_str = b.get("t").and_then(|v| v.as_str()).unwrap_or("");
                let t = chrono::DateTime::parse_from_rfc3339(t_str)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);

                let close = b.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if close <= 0.0 {
                    return None;
                }

                Some(Candle {
                    time: t,
                    open: b.get("o").and_then(|v| v.as_f64()).unwrap_or(close),
                    high: b.get("h").and_then(|v| v.as_f64()).unwrap_or(close),
                    low: b.get("l").and_then(|v| v.as_f64()).unwrap_or(close),
                    close,
                    volume: b.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
            })
            .collect();
        candles.sort_by(|a, b| a.time.cmp(&b.time));
        {
            let mut cache = state.market.bars_cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.insert(
                cache_key,
                BarsCacheEntry {
                    candles: candles.clone(),
                    updated: Instant::now(),
                },
            );
            const MAX_CACHE_ENTRIES: usize = 200;
            while cache.len() > MAX_CACHE_ENTRIES {
                if let Some(oldest_key) = cache
                    .iter()
                    .min_by_key(|(_, v)| v.updated)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                } else {
                    break;
                }
            }
        }
        return Ok(candles);
    }

    Err(TitanError::Internal("NO BARS".into()))
}

pub(crate) async fn fetch_bars_with_period(
    state: Arc<TitanState>,
    symbol: &str,
    timeframe: &str,
    days: i64,
) -> Result<Vec<Candle>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let symbol = symbol.trim().to_uppercase();
    let tf = normalize_alpaca_tf(timeframe);

    let crypto_bases = [
        "BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "AVAX", "DOT", "LINK", "LTC", "BCH",
    ];
    let is_crypto = if symbol.contains('/') {
        true
    } else if symbol.ends_with("USD") && symbol.len() > 3 {
        crypto_bases.contains(&&symbol[..symbol.len() - 3])
    } else {
        false
    };

    let client = state.http_client.clone();
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(days);
    let limit = match timeframe {
        "1m" => 10000,
        "3m" => 5000,
        "5m" => 5000,
        _ => 2000,
    };
    let mut params: Vec<(String, String)> = vec![
        ("symbols".into(), symbol.clone()),
        ("timeframe".into(), tf.to_string()),
        ("limit".into(), format!("{}", limit)),
        ("start".into(), start.to_rfc3339()),
        ("end".into(), end.to_rfc3339()),
        ("sort".into(), "desc".into()),
    ];
    if !is_crypto {
        params.push(("feed".into(), "sip".into()));
        params.push((
            "adjustment".into(),
            std::env::var("TITAN_STOCK_ADJUSTMENT").unwrap_or_else(|_| "all".into()),
        ));
    }
    let url = if is_crypto {
        "https://data.alpaca.markets/v1beta3/crypto/us/bars"
    } else {
        "https://data.alpaca.markets/v2/stocks/bars"
    };

    let _permit = crate::state::acquire_api_permit(&state).await?;
    let res = client
        .get(url)
        .query(&params)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!("Alpaca {}: {}", status, txt)));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| TitanError::Internal(format!("JSON: {}", e)))?;
    let bars_container = json
        .get("bars")
        .ok_or_else(|| TitanError::Internal("No 'bars' field".into()))?;
    let symbol_no_slash = symbol.replace("/", "");
    let bars_array = bars_container
        .get(symbol.as_str())
        .or_else(|| bars_container.get(symbol_no_slash.as_str()))
        .or_else(|| {
            bars_container
                .as_object()
                .and_then(|obj| obj.values().next())
        });

    if let Some(bars) = bars_array.and_then(|b| b.as_array()) {
        let mut candles: Vec<Candle> = bars
            .iter()
            .filter_map(|b| {
                let t = chrono::DateTime::parse_from_rfc3339(b.get("t")?.as_str()?)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);
                let close = b.get("c")?.as_f64()?;
                if close <= 0.0 {
                    return None;
                }
                Some(Candle {
                    time: t,
                    open: b.get("o").and_then(|v| v.as_f64()).unwrap_or(close),
                    high: b.get("h").and_then(|v| v.as_f64()).unwrap_or(close),
                    low: b.get("l").and_then(|v| v.as_f64()).unwrap_or(close),
                    close,
                    volume: b.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
            })
            .collect();
        candles.sort_by(|a, b| a.time.cmp(&b.time));
        return Ok(candles);
    }
    Err(TitanError::Internal("NO BARS".into()))
}

// ═══════════════════════════════════════════════════════════════
//  POSITION / ORDER HELPERS
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn fetch_positions(
    state: Arc<TitanState>,
) -> Result<Vec<AlpacaPosition>, TitanError> {
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let client = state.http_client.clone();
    let url = format!("{}/v2/positions", base_url.trim_end_matches('/'));
    let res = client
        .get(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Alpaca API Error {}: {}",
            status, txt
        )));
    }

    res.json::<Vec<AlpacaPosition>>()
        .await
        .map_err(|e| TitanError::Internal(format!("JSON Error: {}", e)))
}

pub(crate) async fn is_market_open(state: Arc<TitanState>) -> bool {
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return false;
    }

    let client = state.http_client.clone();

    let url = format!("{}/v2/clock", base_url.trim_end_matches('/'));
    match client
        .get(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => match res.json::<serde_json::Value>().await {
            Ok(v) => v.get("is_open").and_then(|b| b.as_bool()).unwrap_or(false),
            Err(_) => false,
        },
        _ => false,
    }
}

pub(crate) async fn has_pending_orders_for_symbol(state: Arc<TitanState>, symbol: &str) -> bool {
    if state.dry_run.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return false;
    }

    let client = state.http_client.clone();

    let url = format!(
        "{}/v2/orders?status=open&symbols={}&limit=1",
        base_url.trim_end_matches('/'),
        symbol
    );
    match client
        .get(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => match res.json::<Vec<serde_json::Value>>().await {
            Ok(orders) => !orders.is_empty(),
            Err(_) => false,
        },
        _ => false,
    }
}

pub(crate) async fn cancel_open_orders_for_symbol(
    state: Arc<TitanState>,
    symbol: &str,
    _app: &tauri::AppHandle,
) -> Result<(), TitanError> {
    if state.dry_run.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Ok(());
    }

    let client = state.http_client.clone();

    // FIX CRIT-13: Acquire rate limit permit before Alpaca API call
    let _permit = crate::state::acquire_api_permit(&state).await?;
    state.trading.record_api_call();

    let url = format!(
        "{}/v2/orders?symbols={}",
        base_url.trim_end_matches('/'),
        symbol
    );
    let res = client
        .delete(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    if res.status().is_success() {
        tracing::debug!(component = "autopilot", symbol = %symbol, "Cleared potential open limit orders");
    }
    Ok(())
}

pub(crate) async fn close_position(
    state: Arc<TitanState>,
    symbol: &str,
    qty: f64,
    side: &str,
    app: &tauri::AppHandle,
) -> Result<(), TitanError> {
    let _ = cancel_open_orders_for_symbol(state.clone(), symbol, app).await;
    tokio::time::sleep(StdDuration::from_millis(500)).await;

    let order = LegacyOrder {
        symbol: symbol.to_string(),
        side: side.to_string(),
        qty,
        take_profit: None,
        stop_loss: None,
    };
    // FIX CRIT-16: Allow overwrite if existing entry is older than TTL (120s)
    {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        const EXIT_INFLIGHT_TTL_MS: u64 = 120_000;
        if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
            if let Some(&existing_ts) = inflight.get(symbol) {
                if now_ms.saturating_sub(existing_ts) < EXIT_INFLIGHT_TTL_MS {
                    tracing::warn!(
                        component = "close_position",
                        symbol = %symbol,
                        age_ms = now_ms.saturating_sub(existing_ts),
                        "[EXIT_INFLIGHT] Already in flight, allowing re-attempt"
                    );
                }
            }
            inflight.insert(symbol.to_string(), now_ms);
        }
    }
    use tauri::Manager;
    let _ = app.emit_all(
        "log-message",
        format!("🛑 AUTO EXIT: {} {} {}", side, qty, symbol),
    );
    let sym_owned = symbol.to_string();
    let result = send_order_internal(state.clone(), order, app.clone()).await;
    match &result {
        Ok(_) => {
            if let Ok(mut tags) = state.trading.entry_tags.lock() {
                tags.remove(&sym_owned);
            }
            if let Ok(mut et) = state.trading.entry_times.lock() {
                et.remove(&sym_owned);
            }
        }
        Err(_) => {
            // FIX CRIT-16: Clear exit_inflight on failure to prevent permanent lock
            if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
                inflight.remove(&sym_owned);
            }
        }
    }
    result.map(|_| ())
}

pub(crate) async fn liquidate_all_internal(
    state: Arc<TitanState>,
    app: &tauri::AppHandle,
) -> Result<(), TitanError> {
    use tauri::Manager;
    if state.dry_run.load(std::sync::atomic::Ordering::Relaxed) {
        tracing::warn!(
            component = "dry_run",
            "LIQUIDATE ALL — simulated (dry run mode)"
        );
        let _ = app.emit_all(
            "log-message",
            "[DRY RUN] WOULD LIQUIDATE ALL positions and cancel all orders",
        );
        return Ok(());
    }

    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let client = state.http_client.clone();
    let mut errors: Vec<String> = Vec::new();

    let _permit = crate::state::acquire_api_permit(&state).await?;
    match client
        .delete(format!("{}/v2/orders", base_url.trim_end_matches('/')))
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
    {
        Ok(resp) if !resp.status().is_success() => {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            let msg = format!("Cancel orders failed ({}): {}", status, txt);
            tracing::error!(component = "liquidate", "{}", msg);
            let _ = app.emit_all("log-message", &msg);
            errors.push(msg);
        }
        Err(e) => {
            let msg = format!("Cancel orders network error: {}", e);
            tracing::error!(component = "liquidate", "{}", msg);
            let _ = app.emit_all("log-message", &msg);
            errors.push(msg);
        }
        _ => {}
    }

    let _permit2 = crate::state::acquire_api_permit(&state).await?;
    match client
        .delete(format!("{}/v2/positions", base_url.trim_end_matches('/')))
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
    {
        Ok(resp) if !resp.status().is_success() => {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            let msg = format!("Liquidate positions failed ({}): {}", status, txt);
            tracing::error!(component = "liquidate", "{}", msg);
            let _ = app.emit_all("log-message", &msg);
            errors.push(msg);
        }
        Err(e) => {
            let msg = format!("Liquidate positions network error: {}", e);
            tracing::error!(component = "liquidate", "{}", msg);
            let _ = app.emit_all("log-message", &msg);
            errors.push(msg);
        }
        _ => {}
    }

    if errors.is_empty() {
        let _ = app.emit_all(
            "log-message",
            "PANIC: All orders cancelled, all positions liquidated",
        );
        Ok(())
    } else {
        let _ = app.emit_all(
            "log-message",
            format!("LIQUIDATION PARTIAL — {} errors", errors.len()),
        );
        Err(TitanError::Broker(errors.join("; ")))
    }
}

#[tauri::command]
pub(crate) async fn fetch_alpaca_history(
    symbol: String,
    timeframe: String,
    state: tauri::State<'_, Arc<TitanState>>,
) -> Result<ChartDataResponse, TitanError> {
    fetch_alpaca_history_internal(state.inner().clone(), symbol, timeframe).await
}

pub(crate) async fn fetch_alpaca_history_internal(
    state: Arc<TitanState>,
    symbol: String,
    timeframe: String,
) -> Result<ChartDataResponse, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };

    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let symbol = symbol.trim().to_uppercase();

    tracing::debug!("Fetching {} {}", symbol, timeframe);

    let tf = normalize_alpaca_tf(&timeframe);

    let crypto_bases = [
        "BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "AVAX", "DOT", "LINK", "LTC", "BCH",
    ];
    let is_crypto = if symbol.contains('/') {
        true
    } else if symbol.ends_with("USD") && symbol.len() > 3 {
        let base = &symbol[..symbol.len() - 3];
        crypto_bases.contains(&base)
    } else {
        false
    };

    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(365);
    let start_str = start.to_rfc3339();
    let end_str = end.to_rfc3339();
    let client = state.http_client.clone();
    let mut params: Vec<(String, String)> = vec![
        ("symbols".to_string(), symbol.clone()),
        ("timeframe".to_string(), tf.to_string()),
        ("limit".to_string(), "1000".to_string()),
        ("start".to_string(), start_str),
        ("end".to_string(), end_str),
        ("sort".to_string(), "desc".to_string()),
    ];
    let url = if is_crypto {
        "https://data.alpaca.markets/v1beta3/crypto/us/bars"
    } else {
        "https://data.alpaca.markets/v2/stocks/bars"
    };
    if !is_crypto {
        let feed = "sip".to_string();
        params.push(("feed".to_string(), feed));
        let adjustment =
            std::env::var("TITAN_STOCK_ADJUSTMENT").unwrap_or_else(|_| "all".to_string());
        params.push(("adjustment".to_string(), adjustment));
    }

    let res = client
        .get(url)
        .query(&params)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Alpaca API Error {}: {}",
            status, txt
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| TitanError::Internal(format!("JSON Error: {}", e)))?;

    let bars_container = json
        .get("bars")
        .ok_or_else(|| TitanError::Internal("No 'bars' field in response".into()))?;

    let symbol_no_slash = symbol.replace("/", "");
    let bars_array = bars_container
        .get(symbol.as_str())
        .or_else(|| bars_container.get(symbol_no_slash.as_str()))
        .or_else(|| {
            if let Some(obj) = bars_container.as_object() {
                obj.values().next()
            } else {
                None
            }
        });

    if let Some(bars) = bars_array.and_then(|b| b.as_array()) {
        let mut candles: Vec<Candle> = bars
            .iter()
            .filter_map(|b| {
                let t_str = b.get("t").and_then(|v| v.as_str()).unwrap_or("");
                let t = chrono::DateTime::parse_from_rfc3339(t_str)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);

                let close = b.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if close <= 0.0 {
                    return None;
                }

                Some(Candle {
                    time: t,
                    open: b.get("o").and_then(|v| v.as_f64()).unwrap_or(close),
                    high: b.get("h").and_then(|v| v.as_f64()).unwrap_or(close),
                    low: b.get("l").and_then(|v| v.as_f64()).unwrap_or(close),
                    close,
                    volume: b.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
            })
            .collect();
        candles.sort_by(|a, b| a.time.cmp(&b.time));

        let indicators_history = if candles.len() > 20 {
            let (periods, oracle_w) = {
                let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                (crate::state::build_indicator_periods(&s), crate::state::build_oracle_weights(&s))
            };
            Some(indicators::calculate_all_history_with_periods(&candles, &periods, &oracle_w))
        } else {
            None
        };

        return Ok(ChartDataResponse {
            candles,
            indicators_history,
        });
    }

    let keys = bars_container
        .as_object()
        .map(|o| o.keys().collect::<Vec<_>>());
    Err(TitanError::Broker(format!(
        "Symbol '{}' not found. API returned: {:?}",
        symbol, keys
    )))
}

#[tauri::command]
pub(crate) async fn get_cvd(
    symbol: String,
    state: tauri::State<'_, Arc<TitanState>>,
) -> Result<f64, TitanError> {
    let norm = crate::state::normalize_symbol(&symbol);
    let cvd_map = state.market.cvd.lock().unwrap_or_else(|e| e.into_inner());
    Ok(cvd_map.get(&norm).copied().unwrap_or(0.0))
}

#[tauri::command]
pub(crate) async fn get_smarttrend_diagnostics(
    state: tauri::State<'_, Arc<TitanState>>,
) -> Result<crate::smarttrend::SmartTrendDiagnostics, TitanError> {
    let engine = state.smarttrend_engine.lock().unwrap_or_else(|e| e.into_inner());
    Ok(engine.diagnostics())
}

#[tauri::command]
pub(crate) async fn get_microstructure(
    state: tauri::State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    let ofi = state.market.ofi_state.load();
    Ok(serde_json::json!({
        "microprice": ofi.microprice,
        "ofi": ofi.current_ofi,
        "spread_bps": ofi.spread_bps,
        "vpin": ofi.stream_vpin,
        "imbalance": ofi.imbalance_ratio,
        "spread_narrowing": ofi.spread_narrowing,
        "best_bid": ofi.best_bid,
        "best_ask": ofi.best_ask,
    }))
}

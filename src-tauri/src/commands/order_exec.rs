use std::sync::Arc;
use std::time::Instant;

use rust_decimal::prelude::ToPrimitive;

use crate::error::TitanError;
use crate::state::{normalize_symbol, TitanState};
use crate::types::*;

use super::market_data::fetch_positions;

/// Structured result from a successful order submission.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrderResult {
    pub broker_id: String,
    pub client_order_id: String,
    pub status: String,
    pub symbol: String,
    pub qty: f64,
    pub side: String,
}

// ═══════════════════════════════════════════════════════════════
//  ORDER AUDIT TRAIL
// ═══════════════════════════════════════════════════════════════

pub fn log_order_attempt(
    symbol: &str,
    side: &str,
    qty: f64,
    payload: &serde_json::Value,
    response: &str,
    latency_ms: u64,
    client_order_id: Option<&str>,
    broker_id: Option<&str>,
    retry_count: u32,
) {
    let log_dir = crate::paths::data_dir().join("orders");
    let _ = std::fs::create_dir_all(&log_dir);
    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "symbol": symbol,
        "side": side,
        "qty": qty,
        "payload": payload,
        "response": response,
        "latency_ms": latency_ms,
        "client_order_id": client_order_id,
        "broker_id": broker_id,
        "retry_count": retry_count,
    });
    let path = log_dir.join("order_log.jsonl");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", entry);
    }
}

// ═══════════════════════════════════════════════════════════════
//  ORDER SUBMISSION
// ═══════════════════════════════════════════════════════════════

#[tracing::instrument(skip(state, app), fields(symbol = %order.symbol, side = %order.side, qty = %order.qty))]
pub(crate) async fn send_order_internal(
    state: Arc<TitanState>,
    order: LegacyOrder,
    app: tauri::AppHandle,
) -> Result<OrderResult, TitanError> {
    use tauri::Manager;

    // FIX HIGH-24: Capture generation_id BEFORE any await to ensure it matches
    // the decision/submit generation, not the post-response state.
    let generation_id = state.current_generation();

    if state.dry_run.load(std::sync::atomic::Ordering::Relaxed) {
        let bracket_tag = if order.take_profit.is_some() || order.stop_loss.is_some() {
            " [BRACKET]"
        } else {
            ""
        };
        let msg = format!(
            "[DRY RUN] WOULD SEND: {} {} x{:.4} {} TP:{:?} SL:{:?}{}",
            order.side.to_uppercase(),
            order.symbol,
            order.qty,
            order.symbol,
            order.take_profit,
            order.stop_loss,
            bracket_tag
        );
        tracing::info!(component = "dry_run", symbol = %order.symbol, side = %order.side, qty = %order.qty,
            tp = ?order.take_profit, sl = ?order.stop_loss, "Order simulated");
        let _ = app.emit_all("log-message", &msg);
        let _ = app.emit_all(
            "dry-run-order",
            serde_json::json!({
                "symbol": order.symbol,
                "side": order.side,
                "qty": order.qty,
                "take_profit": order.take_profit,
                "stop_loss": order.stop_loss,
                "simulated": true,
            }),
        );
        return Ok(OrderResult {
            broker_id: format!("DRY_{}", uuid::Uuid::new_v4()),
            client_order_id: uuid::Uuid::new_v4().to_string(),
            status: "dry_run".to_string(),
            symbol: order.symbol.clone(),
            qty: order.qty,
            side: order.side.clone(),
        });
    }

    let _ = app.emit_all(
        "log-message",
        format!("ORDER: {} {} {}", order.side, order.qty, order.symbol),
    );

    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let symbol = order.symbol.trim().to_uppercase();
    let side = order.side.to_lowercase();
    let mut qty = order.qty;

    if !qty.is_finite() || qty <= 0.0 {
        return Err(TitanError::Validation(format!(
            "Invalid qty={} for {}", qty, symbol
        )));
    }
    if let Some(tp) = order.take_profit {
        if !tp.is_finite() || tp <= 0.0 {
            return Err(TitanError::Validation(format!(
                "Invalid take_profit={} for {}", tp, symbol
            )));
        }
    }
    if let Some(sl) = order.stop_loss {
        if !sl.is_finite() || sl <= 0.0 {
            return Err(TitanError::Validation(format!(
                "Invalid stop_loss={} for {}", sl, symbol
            )));
        }
    }

    let asset_check_hard = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.asset_check_hard_gate
    };

    let mut is_fractionable = true;
    {
        let asset_url = format!("{}/v2/assets/{}", base_url.trim_end_matches('/'), symbol);
        let client = state.http_client.clone();
        let _permit = crate::state::acquire_api_permit(&state).await?;
        state.trading.record_api_call();
        match client
            .get(&asset_url)
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return Err(TitanError::Broker(format!(
                        "Asset {} not found on Alpaca — may be delisted",
                        symbol
                    )));
                }
                // FIX CRIT-5: Treat JSON parse failure as a failed check, not silent pass
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let tradable = json["tradable"].as_bool().unwrap_or(false);
                        let status = json["status"].as_str().unwrap_or("unknown");
                        if !tradable || status != "active" {
                            return Err(TitanError::Broker(format!(
                                "Asset {} is not tradable (status: {})",
                                symbol, status
                            )));
                        }
                        is_fractionable = json["fractionable"].as_bool().unwrap_or(false);
                    }
                    Err(parse_err) => {
                        if asset_check_hard {
                            return Err(TitanError::Broker(format!(
                                "Asset check for {} failed: JSON parse error: {} (hard gate enabled)",
                                symbol, parse_err
                            )));
                        }
                        tracing::warn!(
                            component = "order",
                            symbol = %symbol,
                            "Asset check JSON parse failed (soft): {} — defaulting is_fractionable=false",
                            parse_err
                        );
                        is_fractionable = false;
                    }
                }
            }
            Err(e) => {
                if asset_check_hard {
                    return Err(TitanError::Broker(format!(
                        "Asset check unavailable for {}: {} (hard gate enabled)",
                        symbol, e
                    )));
                }
                tracing::warn!(component = "order", symbol = %symbol, "Asset check failed (soft): {}", e);
            }
        }
    }

    let allow_short = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.allow_short
    };
    if side == "sell" && !allow_short {
        let raw_positions = fetch_positions(state.clone()).await?;
        let has_position = raw_positions
            .iter()
            .any(|p| p.symbol.eq_ignore_ascii_case(&symbol));
        if !has_position {
            return Err(TitanError::Validation(
                "SHORT DISABLED (NO POSITION)".into(),
            ));
        }
    }

    // HHI concentration gate for manual orders (S15)
    {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        if settings.hhi_gate_manual_orders && settings.enable_concentration_gate {
            let positions = &settings.positions;
            if !positions.is_empty() {
                let mut notionals: Vec<(String, f64)> = Vec::new();
                for p in positions {
                    let p_qty = p.qty.to_f64().unwrap_or(0.0).abs();
                    let p_price = p.entry_price.to_f64().unwrap_or(0.0);
                    if p_qty > 0.0 && p_price > 0.0 {
                        notionals.push((p.symbol.clone(), p_qty * p_price));
                    }
                }
                let new_price_est = {
                    let last_prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
                    last_prices.get(&crate::state::normalize_symbol(&symbol)).copied().unwrap_or(0.0)
                };
                let new_notional = qty * if new_price_est > 0.0 { new_price_est } else { 1.0 };
                notionals.push((symbol.clone(), new_notional));
                let total: f64 = notionals.iter().map(|(_, n)| n).sum();
                if total > 0.0 {
                    let hhi: f64 = notionals.iter().map(|(_, n)| {
                        let w = n / total;
                        w * w
                    }).sum();
                    let hhi_pre: f64 = {
                        let pre_notionals: Vec<&(String, f64)> = notionals.iter()
                            .filter(|(s, _)| !s.eq_ignore_ascii_case(&symbol))
                            .collect();
                        let pre_total: f64 = pre_notionals.iter().map(|(_, n)| n).sum();
                        if pre_total > 0.0 {
                            pre_notionals.iter().map(|(_, n)| { let w = n / pre_total; w * w }).sum()
                        } else {
                            0.0
                        }
                    };
                    tracing::info!(
                        component = "order",
                        symbol = %symbol,
                        hhi_pre = format_args!("{:.3}", hhi_pre),
                        hhi_post = format_args!("{:.3}", hhi),
                        limit = format_args!("{:.3}", settings.max_concentration_hhi_manual),
                        "[HHI_GATE_MANUAL]"
                    );
                    if hhi > settings.max_concentration_hhi_manual {
                        return Err(TitanError::Validation(format!(
                            "HHI concentration {:.2} exceeds manual limit {:.2} for {}",
                            hhi, settings.max_concentration_hhi_manual, symbol
                        )));
                    }
                }
            }
        }
    }

    let risk_pct = {
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.risk_per_trade
    };
    if risk_pct > 0.0 {
        let price_key = normalize_symbol(&symbol);
        let mut last_price = {
            let prices = state.market.last_prices.lock().unwrap_or_else(|e| e.into_inner());
            prices.get(&price_key).copied().unwrap_or(0.0)
        };
        if last_price <= 0.0 {
            tracing::info!(component = "order", symbol = %symbol, "No cached price — fetching latest trade from API");
            let fallback_url = format!(
                "https://data.alpaca.markets/v2/stocks/{}/trades/latest",
                symbol.replace("/", "")
            );
            if let Ok(resp) = state
                .http_client
                .get(&fallback_url)
                .header("APCA-API-KEY-ID", &key)
                .header("APCA-API-SECRET-KEY", &secret)
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(p) = json["trade"]["p"].as_f64() {
                        if p > 0.0 {
                            last_price = p;
                            state.market
                                .last_prices
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .insert(normalize_symbol(&symbol), p);
                            tracing::info!(component = "order", symbol = %symbol, price = p, "Fetched fallback price");
                        }
                    }
                }
            }
        }
        if last_price <= 0.0 {
            tracing::warn!(component = "order", symbol = %symbol, "No price from cache or API — blocking order");
            return Err(TitanError::Validation(format!(
                "NO PRICE DATA for {} — cannot enforce risk limits",
                symbol
            )));
        }
        {
            let account_url = format!("{}/v2/account", base_url.trim_end_matches('/'));
            let client = state.http_client.clone();
            let account_res = client
                .get(&account_url)
                .header("APCA-API-KEY-ID", &key)
                .header("APCA-API-SECRET-KEY", &secret)
                .send()
                .await
                .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;
            if !account_res.status().is_success() {
                let status = account_res.status();
                let txt = account_res.text().await.unwrap_or_default();
                return Err(TitanError::Broker(format!(
                    "Alpaca API Error {}: {}",
                    status, txt
                )));
            }
            let account: AlpacaAccount = account_res
                .json()
                .await
                .map_err(|e| TitanError::Internal(format!("JSON Error: {}", e)))?;
            let equity = account.equity.parse::<f64>().unwrap_or(0.0);
            let max_notional = equity * (risk_pct / 100.0);
            let max_qty = if max_notional > 0.0 {
                max_notional / last_price
            } else {
                0.0
            };
            if max_qty <= 0.0 {
                return Err(TitanError::Validation("RISK LIMIT HIT".into()));
            }
            if qty > max_qty {
                qty = (max_qty * 1_000_000.0).floor() / 1_000_000.0;
            }
        }
    }

    if !is_fractionable {
        qty = qty.floor();
        if qty < 1.0 {
            return Err(TitanError::Validation(format!(
                "{} is not fractionable and qty rounded to 0",
                symbol
            )));
        }
    }

    let is_crypto = symbol.contains("USD")
        || symbol.contains("/")
        || symbol.starts_with("BTC")
        || symbol.starts_with("ETH")
        || symbol.starts_with("LTC")
        || symbol.starts_with("DOGE")
        || symbol.starts_with("SOL")
        || symbol.starts_with("AVAX")
        || symbol.starts_with("UNI")
        || symbol.starts_with("AAVE")
        || symbol.starts_with("LINK")
        || symbol.starts_with("DOT");
    let enable_extended = {
        state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .enable_extended
    };
    let tif = if is_crypto { "gtc" } else { "day" };

    let has_bracket = order.take_profit.is_some() || order.stop_loss.is_some();

    let qty_str = format!("{:.6}", qty);
    let mut body = if has_bracket && !is_crypto {
        let mut b = serde_json::json!({
            "symbol": symbol,
            "qty": qty_str,
            "side": side,
            "type": "market",
            "time_in_force": tif,
            "order_class": "bracket"
        });
        if let Some(tp) = order.take_profit {
            if tp > 0.0 {
                b["take_profit"] = serde_json::json!({ "limit_price": format!("{:.2}", tp) });
            }
        }
        if let Some(sl) = order.stop_loss {
            if sl > 0.0 {
                b["stop_loss"] = serde_json::json!({ "stop_price": format!("{:.2}", sl) });
            }
        }
        tracing::info!(component = "order", symbol = %symbol, "BRACKET order: TP={:?} SL={:?}", order.take_profit, order.stop_loss);
        b
    } else {
        serde_json::json!({
            "symbol": symbol,
            "qty": qty_str,
            "side": side,
            "type": "market",
            "time_in_force": tif
        })
    };

    if enable_extended && !is_crypto && !has_bracket {
        let limit_price = {
            let sym_key = normalize_symbol(&symbol);
            let cached = state.market
                .last_prices
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&sym_key)
                .copied()
                .unwrap_or(0.0);
            if cached > 0.0 {
                cached
            } else {
                let fallback_url = format!(
                    "https://data.alpaca.markets/v2/stocks/{}/trades/latest",
                    symbol.replace("/", "")
                );
                let mut p = 0.0_f64;
                if let Ok(resp) = state
                    .http_client
                    .get(&fallback_url)
                    .header("APCA-API-KEY-ID", &key)
                    .header("APCA-API-SECRET-KEY", &secret)
                    .send()
                    .await
                {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(val) = json["trade"]["p"].as_f64() {
                            p = val;
                        }
                    }
                }
                if p > 0.0 {
                    state.market
                        .last_prices
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(sym_key, p);
                }
                p
            }
        };
        if limit_price > 0.0 {
            let smart_limit = {
                let ofi = state.market.ofi_state.load();
                let fresh = !ofi.is_stale(5_000) && ofi.best_bid > 0.0 && ofi.best_ask > 0.0;
                if fresh {
                    let mid = (ofi.best_bid + ofi.best_ask) / 2.0;
                    let slippage = if side == "buy" { 1.001 } else { 0.999 };
                    Some(mid * slippage)
                } else {
                    None
                }
            };
            let final_limit = match smart_limit {
                Some(l) => {
                    tracing::info!(component = "smart_route", symbol = %symbol, "Extended hours: using L2 midpoint limit {:.2}", l);
                    l
                }
                None => {
                    let fallback_slippage = if side == "buy" { 1.005 } else { 0.995 };
                    limit_price * fallback_slippage
                }
            };
            body["type"] = serde_json::json!("limit");
            body["limit_price"] = serde_json::json!(format!("{:.2}", final_limit));
            body["time_in_force"] = serde_json::json!("day");
            body["extended_hours"] = serde_json::json!(true);
        }
    }

    let client_order_id = uuid::Uuid::new_v4().to_string();
    body["client_order_id"] = serde_json::json!(&client_order_id);

    let client = state.http_client.clone();
    let url = format!("{}/v2/orders", base_url.trim_end_matches('/'));

    let max_retries: u32 = 2;
    let mut last_err = String::new();
    let mut total_latency_ms: u64;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            tracing::warn!(
                component = "order",
                symbol = %symbol,
                attempt,
                client_order_id = %client_order_id,
                "Retrying order submission (idempotent)"
            );
            tokio::time::sleep(std::time::Duration::from_millis(500 * u64::from(attempt))).await;
        }

        let _permit2 = crate::state::acquire_api_permit(&state).await?;
        state.trading.record_api_call();
        state.trading.orders_submitted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let order_t0 = Instant::now();
        let send_result = client
            .post(&url)
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .json(&body)
            .send()
            .await;
        let latency_ms = order_t0.elapsed().as_millis() as u64;
        total_latency_ms = latency_ms;
        state.latency_metrics.record_order(latency_ms * 1000);

        match send_result {
            Err(e) => {
                last_err = format!("Net Error: {}", e);
                log_order_attempt(
                    &symbol, &side, qty, &body,
                    &format!("NET_ERR attempt {}: {}", attempt, e),
                    latency_ms, Some(&client_order_id), None, attempt,
                );
                if attempt < max_retries {
                    continue;
                }
                return Err(TitanError::Network(last_err));
            }
            Ok(res) => {
                let status_code = res.status();
                if status_code.is_server_error() && attempt < max_retries {
                    let txt = res.text().await.unwrap_or_default();
                    last_err = format!("Server Error {}: {}", status_code, txt);
                    log_order_attempt(
                        &symbol, &side, qty, &body,
                        &format!("5xx attempt {}: {} {}", attempt, status_code, txt),
                        latency_ms, Some(&client_order_id), None, attempt,
                    );
                    continue;
                }

                if !status_code.is_success() {
                    let txt = res.text().await.unwrap_or_default();
                    log_order_attempt(
                        &symbol, &side, qty, &body,
                        &format!("ERR {}: {}", status_code, txt),
                        latency_ms, Some(&client_order_id), None, attempt,
                    );
                    return Err(TitanError::Broker(format!(
                        "Alpaca API Error {}: {}", status_code, txt
                    )));
                }

                let resp_json: serde_json::Value = res.json().await.unwrap_or_default();
                let broker_id = resp_json["id"].as_str().unwrap_or("").to_string();
                let order_status = resp_json["status"].as_str().unwrap_or("unknown").to_string();

                let suffix = if has_bracket { " (BRACKET: TP+SL)" } else { "" };
                log_order_attempt(
                    &symbol, &side, qty, &body,
                    &format!("OK{} broker_id={} status={}", suffix, broker_id, order_status),
                    latency_ms, Some(&client_order_id), Some(&broker_id), attempt,
                );

                {
                    let mut pending = state.trading.pending_orders.lock()
                        .unwrap_or_else(|e| e.into_inner());
                    pending.push(crate::trading_state::PendingOrder {
                        client_order_id: client_order_id.clone(),
                        broker_id: broker_id.clone(),
                        symbol: symbol.clone(),
                        side: side.clone(),
                        qty,
                        submitted_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        generation_id,
                    });
                }

                let _ = app.emit_all(
                    "log-message",
                    format!(
                        "ORDER ACCEPTED{}: {} {} x{:.4} {} [{}]",
                        suffix,
                        side.to_uppercase(),
                        symbol,
                        qty,
                        symbol,
                        &broker_id[..8.min(broker_id.len())]
                    ),
                );

                let _ = app.emit_all("order-lifecycle", serde_json::json!({
                    "event": "submitted",
                    "broker_id": broker_id,
                    "client_order_id": client_order_id,
                    "symbol": symbol,
                    "side": side,
                    "qty": qty,
                    "status": order_status,
                    "latency_ms": total_latency_ms,
                    "retries": attempt,
                }));

                return Ok(OrderResult {
                    broker_id,
                    client_order_id,
                    status: order_status,
                    symbol,
                    qty,
                    side,
                });
            }
        }
    }

    Err(TitanError::Network(format!(
        "Order failed after {} retries: {}", max_retries, last_err
    )))
}

// ═══════════════════════════════════════════════════════════════
//  DISCORD ALERTING
// ═══════════════════════════════════════════════════════════════

pub(crate) async fn send_discord_alert(
    state: &Arc<TitanState>,
    webhook_url: &str,
    title: &str,
    message: &str,
    color: u32,
) {
    if webhook_url.is_empty() {
        return;
    }
    let client = state.http_client.clone();
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

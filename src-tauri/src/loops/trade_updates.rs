use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::app_state::{self, OrderSide};
use crate::commands::data::send_discord_alert;
use crate::state::TitanState;
#[cfg(feature = "telegram")]
use crate::telegram;

pub(crate) async fn trade_updates_stream(state: Arc<TitanState>, app: AppHandle) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::protocol::Message;

    let my_generation = state.current_generation();
    tracing::info!(
        component = "trade_updates",
        generation = my_generation,
        "Trade updates stream started"
    );
    let mut backoff_secs: u64 = 2;
    let mut reconnect_count: u32 = 0;
    const MAX_BACKOFF: u64 = 60;

    loop {
        if state.shutdown_token.is_cancelled() {
            break;
        }
        if state.current_generation() != my_generation {
            tracing::warn!(
                component = "trade_updates",
                my_gen = my_generation,
                current_gen = state.current_generation(),
                "Generation mismatch — exiting stale trade_updates instance"
            );
            state.generation_mismatch_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            break;
        }
        state.watchdog.heartbeat("trade_updates", 0);
        let (key, secret, base_url) = {
            let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
        };

        let key = key.trim().to_string();
        let secret = secret.trim().to_string();

        if key.is_empty() || secret.is_empty() {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        let ws_url = if base_url.contains("paper") {
            "wss://paper-api.alpaca.markets/stream".to_string()
        } else {
            "wss://api.alpaca.markets/stream".to_string()
        };

        println!(
            "TRADE UPDATES: Connecting to {} (key_len={}, secret_len={})",
            ws_url,
            key.len(),
            secret.len()
        );
        let connect_time = Instant::now();

        let (mut ws_stream, _) = match connect_async(&ws_url).await {
            Ok(v) => {
                println!("TRADE UPDATES: WebSocket connected");
                backoff_secs = 2;
                reconnect_count += 1;
                let _ = app.emit_all("trade-updates-health", serde_json::json!({
                    "connected": true,
                    "reconnect_count": reconnect_count,
                    "last_disconnect_reason": serde_json::Value::Null,
                }));
                v
            }
            Err(e) => {
                let _ = app.emit_all(
                    "log-message",
                    format!("TRADE UPDATES: Connection failed — {}", e),
                );
                let _ = app.emit_all("trade-updates-health", serde_json::json!({
                    "connected": false,
                    "reconnect_count": reconnect_count,
                    "last_disconnect_reason": format!("{}", e),
                }));
                backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF);
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                continue;
            }
        };

        // Read ALL initial messages and log them (Alpaca sends a welcome array)
        let mut handshake_count = 0u32;
        let mut got_welcome = false;
        loop {
            match tokio::time::timeout(StdDuration::from_secs(10), ws_stream.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => {
                    handshake_count += 1;
                    println!(
                        "TRADE UPDATES HANDSHAKE [{}]: {}",
                        handshake_count,
                        &t[..t.len().min(400)]
                    );
                    got_welcome = true;
                    break;
                }
                Ok(Some(Ok(Message::Ping(p)))) => {
                    let _ = ws_stream.send(Message::Pong(p)).await;
                }
                _ => {
                    eprintln!("TRADE UPDATES: No welcome within 10s");
                    break;
                }
            }
        }

        if !got_welcome {
            let _ = ws_stream.close(None).await;
            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF);
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            continue;
        }

        // Alpaca Trading API stream uses: action=authenticate, data={key_id, secret_key}
        let auth = serde_json::json!({
            "action": "authenticate",
            "data": { "key_id": &key, "secret_key": &secret }
        });
        println!(
            "TRADE UPDATES: Sending auth (key={}...)",
            &key[..key.len().min(8)]
        );
        let _ = ws_stream.send(Message::Text(auth.to_string())).await;

        let mut authed = false;
        let auth_deadline = Instant::now() + StdDuration::from_secs(15);
        loop {
            if Instant::now() > auth_deadline {
                eprintln!("TRADE UPDATES: Auth timeout after 15s");
                break;
            }
            match tokio::time::timeout(StdDuration::from_secs(10), ws_stream.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => {
                    handshake_count += 1;
                    println!(
                        "TRADE UPDATES AUTH [{}]: {}",
                        handshake_count,
                        &t[..t.len().min(400)]
                    );
                    let _ = app.emit_all(
                        "log-message",
                        format!("TRADE_UPDATES_AUTH: {}", &t[..t.len().min(200)]),
                    );
                    if t.contains("authorized") || t.contains("authenticated") {
                        authed = true;
                        break;
                    } else if t.contains("unauthorized") || t.contains("forbidden") {
                        eprintln!("TRADE UPDATES: Auth rejected: {}", t);
                        // Try legacy format as fallback
                        let alt_auth = serde_json::json!({
                            "action": "auth",
                            "key": &key,
                            "secret": &secret
                        });
                        println!("TRADE UPDATES: Trying legacy auth format...");
                        let _ = ws_stream.send(Message::Text(alt_auth.to_string())).await;
                    }
                }
                Ok(Some(Ok(Message::Ping(p)))) => {
                    let _ = ws_stream.send(Message::Pong(p)).await;
                }
                _ => {
                    eprintln!("TRADE UPDATES: Auth stream ended or timed out");
                    break;
                }
            }
        }

        if !authed {
            let _ = ws_stream.close(None).await;
            let _ = app.emit_all(
                "log-message",
                "TRADE UPDATES: Auth failed — check Activity Log for server messages",
            );
            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF);
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            continue;
        }

        let sub = serde_json::json!({
            "action": "listen",
            "data": { "streams": ["trade_updates"] }
        });
        let _ = ws_stream.send(Message::Text(sub.to_string())).await;

        let _ = app.emit_all(
            "log-message",
            "TRADE UPDATES: Subscribed — listening for fills",
        );

        let mut eviction_tick: u64 = 0;
        while let Some(Ok(msg)) = ws_stream.next().await {
            if state.shutdown_token.is_cancelled() {
                break;
            }
            state.watchdog.heartbeat("trade_updates", 0);

            // S72 FIX-7: Periodic HashMap eviction (every 100 messages)
            eviction_tick += 1;
            if eviction_tick.is_multiple_of(100) {
                let active_syms: std::collections::HashSet<String> = {
                    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    settings.positions.iter().map(|p| p.symbol.clone()).collect()
                };
                state.trading.evict_stale_entries(&active_syms);
            }

            // FIX CRIT-16: TTL-based cleanup for exit_inflight (120s)
            {
                let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                const EXIT_INFLIGHT_TTL_MS: u64 = 120_000;
                if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
                    inflight.retain(|sym, &mut ts| {
                        let keep = now_ms.saturating_sub(ts) < EXIT_INFLIGHT_TTL_MS;
                        if !keep {
                            tracing::warn!(
                                component = "trade_updates",
                                symbol = %sym,
                                age_ms = now_ms.saturating_sub(ts),
                                "[EXIT_INFLIGHT_TTL] Expired stale exit_inflight entry"
                            );
                        }
                        keep
                    });
                }
            }

            let text = match msg {
                Message::Text(t) => t,
                Message::Ping(payload) => {
                    let _ = ws_stream.send(Message::Pong(payload)).await;
                    continue;
                }
                _ => continue,
            };

            let parsed: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let stream = parsed.get("stream").and_then(|v| v.as_str()).unwrap_or("");
            if stream != "trade_updates" {
                continue;
            }

            let data = match parsed.get("data") {
                Some(d) => d,
                None => continue,
            };

            let event = data.get("event").and_then(|v| v.as_str()).unwrap_or("");
            let order = data.get("order");

            match event {
                "fill" | "partial_fill" => {
                    let symbol = order
                        .and_then(|o| o.get("symbol"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("???")
                        .to_string();
                    let side = order
                        .and_then(|o| o.get("side"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("???")
                        .to_string();
                    let order_id = order
                        .and_then(|o| o.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let cum_filled_qty = order
                        .and_then(|o| o.get("filled_qty"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let cum_filled_avg_price = order
                        .and_then(|o| o.get("filled_avg_price"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let event_owned = event.to_string();

                    // FIX CRIT-11: Alpaca reports cumulative filled_qty per order.
                    // Compute incremental delta to prevent double-counting.
                    let (filled_qty, filled_avg_price) = if !order_id.is_empty() {
                        let mut cum = state.trading.fill_cumulative.lock()
                            .unwrap_or_else(|e| e.into_inner());
                        let (prev_qty, _prev_price) = cum
                            .get(&order_id)
                            .copied()
                            .unwrap_or((0.0, 0.0));
                        let delta_qty = cum_filled_qty - prev_qty;
                        cum.insert(order_id.clone(), (cum_filled_qty, cum_filled_avg_price));
                        drop(cum);
                        {
                            let now_secs = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            if let Ok(mut fts) = state.trading.fill_cumulative_ts.lock() {
                                fts.insert(order_id.clone(), now_secs);
                            }
                        }
                        if delta_qty <= 0.0 {
                            state.trading.fills_deduplicated.fetch_add(
                                1, std::sync::atomic::Ordering::Relaxed,
                            );
                            tracing::debug!(
                                component = "trade_updates",
                                order_id = %order_id,
                                symbol = %symbol,
                                cum_qty = cum_filled_qty,
                                prev_qty = prev_qty,
                                "[FILL_DEDUP] Suppressed zero-delta fill"
                            );
                            continue;
                        }
                        (delta_qty, cum_filled_avg_price)
                    } else {
                        (cum_filled_qty, cum_filled_avg_price)
                    };

                    state.trading.fills_received.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let _ = app.emit_all(
                        "log-message",
                        format!(
                            "FILL: {} {} x{:.4} @ ${:.2} [{}]",
                            side.to_uppercase(),
                            symbol,
                            filled_qty,
                            filled_avg_price,
                            event_owned
                        ),
                    );

                    let _ = app.emit_all(
                        "trade-fill",
                        serde_json::json!({
                            "event": &event_owned,
                            "symbol": &symbol,
                            "side": &side,
                            "filled_qty": filled_qty,
                            "filled_avg_price": filled_avg_price,
                        }),
                    );

                    // S10: SmartTrend fill tracking
                    let is_st_fill = {
                        let tags = state.trading.entry_tags.lock().unwrap_or_else(|e| e.into_inner());
                        tags.get(&symbol).map(|t| t.starts_with("ST|")).unwrap_or(false)
                    };
                    if is_st_fill {
                        let entry_price_for_pnl = {
                            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                            settings.positions.iter()
                                .find(|p| p.symbol == symbol)
                                .map(|p| p.entry_price.to_f64().unwrap_or(0.0))
                                .unwrap_or(0.0)
                        };
                        let is_closing = {
                            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                            settings.positions.iter()
                                .find(|p| p.symbol == symbol)
                                .map(|p| {
                                    let is_buy = side.eq_ignore_ascii_case("buy");
                                    (is_buy && matches!(p.side, OrderSide::Sell))
                                        || (!is_buy && matches!(p.side, OrderSide::Buy))
                                })
                                .unwrap_or(false)
                        };
                        if is_closing && entry_price_for_pnl > 0.0 {
                            let pnl = if side.eq_ignore_ascii_case("buy") {
                                (entry_price_for_pnl - filled_avg_price) * filled_qty
                            } else {
                                (filled_avg_price - entry_price_for_pnl) * filled_qty
                            };
                            let mut account = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                            account.record_fill(pnl);
                            let _ = account.save_to_disk();
                            tracing::info!(
                                component = "trade_updates",
                                symbol = %symbol,
                                pnl,
                                allocated_capital = account.allocated_capital,
                                "[ST FILL] SmartTrend account updated"
                            );
                        }
                        let st_tag = {
                            let tags = state.trading.entry_tags.lock().unwrap_or_else(|e| e.into_inner());
                            tags.get(&symbol).cloned().unwrap_or_default()
                        };
                        let _ = app.emit_all(
                            "smarttrend-fill",
                            serde_json::json!({
                                "symbol": &symbol,
                                "side": &side,
                                "qty": filled_qty,
                                "price": filled_avg_price,
                                "tag": &st_tag,
                                "is_closing": is_closing,
                            }),
                        );
                    }

                    // Discord + Telegram alert on fill
                    let (webhook, alert_fill, tg_token, tg_chat, tg_enabled) = {
                        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        (
                            s.discord_webhook_url.clone(),
                            s.alert_on_fill,
                            s.telegram_bot_token.clone(),
                            s.telegram_chat_id.clone(),
                            s.telegram_enabled,
                        )
                    };
                    if alert_fill && !webhook.is_empty() {
                        let msg = format!(
                            "{} {} x{:.4} @ ${:.2}",
                            side.to_uppercase(),
                            symbol,
                            filled_qty,
                            filled_avg_price
                        );
                        let title = format!("FILL: {}", event_owned);
                        let color = if side == "buy" {
                            0x00FF41u32
                        } else {
                            0xFF6644u32
                        };
                        let st = state.clone();
                        tokio::spawn(async move {
                            send_discord_alert(&st, &webhook, &title, &msg, color).await;
                        });
                    }
                    #[cfg(feature = "telegram")]
                    if tg_enabled {
                        let tg_msg = telegram::format_trade_fill(
                            &symbol,
                            &side,
                            filled_qty,
                            filled_avg_price,
                            None,
                        );
                        let hc = state.http_client.clone();
                        tokio::spawn(async move {
                            telegram::send_telegram_alert(&hc, &tg_token, &tg_chat, &tg_msg).await;
                        });
                    }

                    // Update local position state immediately on fill (delta-based, CRIT-11 fixed)
                    if filled_qty > 0.0 {
                        let is_buy = side.eq_ignore_ascii_case("buy");
                        let filled_qty_dec = Decimal::try_from(filled_qty).unwrap_or(Decimal::ZERO);
                        let filled_price_dec =
                            Decimal::try_from(filled_avg_price).unwrap_or(Decimal::ZERO);
                        let is_partial = event_owned == "partial_fill";
                        let now_epoch = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());

                        if let Some(pos) =
                            settings.positions.iter_mut().find(|p| p.symbol == symbol)
                        {
                            let same_direction = (is_buy && matches!(pos.side, OrderSide::Buy))
                                || (!is_buy && matches!(pos.side, OrderSide::Sell));

                            if same_direction {
                                let old_cost = pos.entry_price * pos.qty;
                                let new_cost = filled_price_dec * filled_qty_dec;
                                pos.qty += filled_qty_dec;
                                if pos.qty > Decimal::ZERO {
                                    pos.entry_price = (old_cost + new_cost) / pos.qty;
                                }
                                if pos.state == app_state::PositionState::PendingEntry {
                                    pos.state = app_state::PositionState::Open;
                                    pos.opened_at = Some(now_epoch);
                                }
                            } else {
                                pos.qty -= filled_qty_dec;
                                if pos.qty <= Decimal::try_from(0.001).unwrap_or(Decimal::ZERO) {
                                    pos.state = app_state::PositionState::Closed;
                                    pos.closed_at = Some(now_epoch);
                                    let sym_clone = symbol.clone();
                                    settings.positions.retain(|p| p.symbol != sym_clone);
                                } else if is_partial {
                                    pos.state = app_state::PositionState::PartialExit;
                                    pos.partial_taken = true;
                                }
                            }

                            if let Some(pos) =
                                settings.positions.iter_mut().find(|p| p.symbol == symbol)
                            {
                                pos.current_price = filled_price_dec;
                                if filled_price_dec > pos.highest_price {
                                    pos.highest_price = filled_price_dec;
                                }
                                pos.lowest_price = if pos.lowest_price > Decimal::ZERO {
                                    pos.lowest_price.min(filled_price_dec)
                                } else {
                                    filled_price_dec
                                };
                                let pnl = if matches!(pos.side, OrderSide::Sell) {
                                    (pos.entry_price - pos.current_price) * pos.qty
                                } else {
                                    (pos.current_price - pos.entry_price) * pos.qty
                                };
                                pos.pnl = pnl;
                                let hundred = Decimal::from(100);
                                pos.pnl_percent = if pos.entry_price > Decimal::ZERO {
                                    if matches!(pos.side, OrderSide::Sell) {
                                        (pos.entry_price - pos.current_price) / pos.entry_price
                                            * hundred
                                    } else {
                                        (pos.current_price - pos.entry_price) / pos.entry_price
                                            * hundred
                                    }
                                } else {
                                    Decimal::ZERO
                                };
                            }
                        } else if is_buy || side.eq_ignore_ascii_case("sell") {
                            let is_exiting = state.trading.exit_inflight.lock()
                                .map(|inf| inf.contains_key(&symbol))
                                .unwrap_or(false);
                            if !is_exiting {
                                let order_side = if is_buy {
                                    OrderSide::Buy
                                } else {
                                    OrderSide::Sell
                                };
                                let fill_broker_id = order
                                    .and_then(|o| o.get("id"))
                                    .and_then(|v| v.as_str())
                                    .map(|s: &str| s.to_string());
                                if let Some(ref bid) = fill_broker_id {
                                    let mut pending = state.trading.pending_orders.lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    pending.retain(|po| po.broker_id != *bid);
                                }
                                settings.positions.push(app_state::Position {
                                    symbol: symbol.clone(),
                                    qty: filled_qty_dec,
                                    entry_price: filled_price_dec,
                                    current_price: filled_price_dec,
                                    pnl: Decimal::ZERO,
                                    pnl_percent: Decimal::ZERO,
                                    side: order_side,
                                    highest_price: filled_price_dec,
                                    lowest_price: filled_price_dec,
                                    state: app_state::PositionState::Open,
                                    opened_at: Some(now_epoch),
                                    closed_at: None,
                                    partial_taken: false,
                                    entry_tag: None,
                                    broker_order_id: fill_broker_id,
                                });
                            }
                        }
                    }

                    // Clean fill_cumulative on final fill to prevent unbounded growth
                    if event_owned == "fill" && !order_id.is_empty() {
                        if let Ok(mut cum) = state.trading.fill_cumulative.lock() {
                            cum.remove(&order_id);
                        }
                    }

                    // FIX CRIT-16: Clear exit_inflight on complete fill (exit side)
                    if event_owned == "fill" {
                        if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
                            inflight.remove(&symbol);
                        }
                    }
                }
                "rejected" | "canceled" => {
                    let symbol = order
                        .and_then(|o| o.get("symbol"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("???");
                    let reason = data
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    if let Some(bid) = order.and_then(|o| o.get("id")).and_then(|v| v.as_str()) {
                        let mut pending = state.trading.pending_orders.lock()
                            .unwrap_or_else(|e| e.into_inner());
                        pending.retain(|po| po.broker_id != bid);
                        // Clean fill_cumulative tracking for rejected/canceled orders
                        if let Ok(mut cum) = state.trading.fill_cumulative.lock() {
                            cum.remove(bid);
                        }
                    }
                    // FIX CRIT-16: Clear exit_inflight on rejected/canceled exit
                    if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
                        inflight.remove(symbol);
                    }
                    let _ = app.emit_all(
                        "log-message",
                        format!("ORDER {}: {} — {}", event.to_uppercase(), symbol, reason),
                    );
                }
                "new" | "accepted" | "pending_new" => {
                    // Silently track — no log spam
                }
                _ => {
                    if rand::random::<f64>() < 0.1 {
                        let _ = app.emit_all(
                            "log-message",
                            format!("TRADE UPDATE: {} — {:?}", event, data),
                        );
                    }
                }
            }
        }

        let _ = ws_stream.close(None).await;

        let session_duration = connect_time.elapsed();
        if session_duration.as_secs() > 60 {
            backoff_secs = 2;
        } else {
            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF);
        }

        let _ = app.emit_all("trade-updates-health", serde_json::json!({
            "connected": false,
            "reconnect_count": reconnect_count,
            "last_disconnect_reason": format!("session ended after {}s", session_duration.as_secs()),
        }));

        let _ = app.emit_all(
            "log-message",
            format!(
                "TRADE UPDATES: Disconnected, reconnecting in {}s",
                backoff_secs
            ),
        );
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
    }
}

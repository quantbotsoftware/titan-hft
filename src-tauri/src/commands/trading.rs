//! Tauri trading / emergency commands. Shared order + panic paths live in
//! `crate::commands::data` (`send_order_internal`, `liquidate_all_internal`).

use std::sync::Arc;

use rust_decimal::Decimal;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::data::send_order_internal;
use crate::error::TitanError;
use crate::state::{normalize_symbol, save_blacklist, TitanState};

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LegacyOrder {
    symbol: String,
    side: String,
    qty: f64,
    #[serde(default)]
    take_profit: Option<f64>,
    #[serde(default)]
    stop_loss: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ManualOrder {
    symbol: String,
    side: String,
    #[serde(default)]
    usd_amount: Option<f64>,
    #[serde(default)]
    share_qty: Option<f64>,
    #[serde(default)]
    take_profit: Option<f64>,
    #[serde(default)]
    stop_loss: Option<f64>,
    #[serde(default)]
    order_type: Option<String>,
    #[serde(default)]
    limit_price: Option<f64>,
}

impl From<LegacyOrder> for crate::types::LegacyOrder {
    fn from(o: LegacyOrder) -> Self {
        crate::types::LegacyOrder {
            symbol: o.symbol,
            side: o.side,
            qty: o.qty,
            take_profit: o.take_profit,
            stop_loss: o.stop_loss,
        }
    }
}

#[tauri::command]
pub(crate) async fn liquidate_all_positions(
    confirm_token: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if confirm_token != "CONFIRM_LIQUIDATE" {
        return Err(TitanError::Validation(
            "Missing confirmation token. Pass 'CONFIRM_LIQUIDATE' to confirm.".into(),
        ));
    }
    let _ = app.emit_all("log-message", "☢️ LIQUIDATE ALL POSITIONS");
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let client = state.http_client.clone();
    let url = format!("{}/v2/positions", base_url.trim_end_matches('/'));
    let _permit = crate::state::acquire_api_permit(&state).await?;
    let res = client
        .delete(&url)
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

    let grace_until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64 + 30_000;
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let mut grace = state.trading.recon_grace.lock().unwrap_or_else(|e| e.into_inner());
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for pos in settings.positions.iter_mut() {
            pos.state = crate::app_state::PositionState::PendingClose;
            pos.closed_at = Some(now_epoch);
            grace.insert(pos.symbol.clone(), grace_until);
        }
        settings.positions.clear();
    }
    state.trading.trail_highs.lock().unwrap_or_else(|e| e.into_inner()).clear();
    state.trading.trail_lows.lock().unwrap_or_else(|e| e.into_inner()).clear();
    state.trading.partial_taken.lock().unwrap_or_else(|e| e.into_inner()).clear();
    state.trading.entry_tags.lock().unwrap_or_else(|e| e.into_inner()).clear();
    state.trading.entry_times.lock().unwrap_or_else(|e| e.into_inner()).clear();

    tracing::warn!(component = "trading", "All positions liquidated -- local state cleared");

    Ok("LIQUIDATION SENT — local state cleared".into())
}

#[tauri::command]
pub(crate) async fn force_close_position(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let sym = normalize_symbol(&symbol);
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }
    let url = format!("{}/v2/positions/{}", base_url.trim_end_matches('/'), sym);
    let client = state.http_client.clone();
    let _permit = crate::state::acquire_api_permit(&state).await?;
    let res = client
        .delete(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    let status = res.status();
    if !status.is_success() {
        let txt = res.text().await.unwrap_or_default();
        tracing::warn!(component = "force_close", symbol = %sym, "API close failed ({}): {}", status, txt);
        return Err(TitanError::Broker(format!(
            "API close failed for {} ({}): {}",
            sym, status, txt
        )));
    }

    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(pos) = settings.positions.iter_mut().find(|p| p.symbol == sym) {
            pos.state = crate::app_state::PositionState::PendingClose;
            pos.closed_at = Some(now_epoch);
        }
        settings.positions.retain(|p| p.symbol != sym);
    }
    {
        let grace_until = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64 + 30_000;
        let mut grace = state.trading.recon_grace.lock().unwrap_or_else(|e| e.into_inner());
        grace.insert(sym.clone(), grace_until);
    }
    if let Ok(mut tags) = state.trading.entry_tags.lock() {
        tags.remove(&sym);
    }
    if let Ok(mut et) = state.trading.entry_times.lock() {
        et.remove(&sym);
    }
    {
        let mut bl = state
            .trading
            .position_blacklist
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        bl.insert(sym.clone());
        let _ = save_blacklist(&bl);
    }

    Ok(format!("Position {} closed via API + blacklisted", sym))
}

#[tauri::command]
pub(crate) async fn cancel_all_orders(
    confirm_token: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if confirm_token != "CONFIRM_CANCEL" {
        return Err(TitanError::Validation(
            "Missing confirmation token. Pass 'CONFIRM_CANCEL' to confirm.".into(),
        ));
    }
    let _ = app.emit_all("log-message", "🛑 CANCEL ALL ORDERS");
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let client = state.http_client.clone();
    let url = format!("{}/v2/orders", base_url.trim_end_matches('/'));
    let _permit = crate::state::acquire_api_permit(&state).await?;
    let res = client
        .delete(&url)
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

    Ok("CANCELED".into())
}

#[tauri::command]
pub(crate) async fn submit_order(
    order: LegacyOrder,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let result = send_order_internal(state.inner().clone(), order.into(), app).await?;
    Ok(format!("ORDER SENT [{}]", result.broker_id))
}

#[tauri::command]
pub(crate) async fn submit_manual_order(
    order: ManualOrder,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let symbol = normalize_symbol(&order.symbol);

    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };

    let price = {
        let cached = state
            .market
            .last_prices
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&symbol)
            .copied()
            .unwrap_or(0.0);
        if cached > 0.0 {
            cached
        } else {
            let url = format!(
                "https://data.alpaca.markets/v2/stocks/{}/trades/latest",
                symbol
            );
            let resp = state
                .http_client
                .get(&url)
                .header("APCA-API-KEY-ID", &key)
                .header("APCA-API-SECRET-KEY", &secret)
                .send()
                .await
                .map_err(|e| TitanError::Network(e.to_string()))?;
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| TitanError::Internal(e.to_string()))?;
            json["trade"]["p"].as_f64().unwrap_or(0.0)
        }
    };

    if price <= 0.0 {
        return Err(TitanError::Broker(format!(
            "Cannot get price for {}",
            symbol
        )));
    }

    let (qty, usd) = if let Some(q) = order.share_qty {
        if q <= 0.0 {
            return Err(TitanError::Validation(
                "Share quantity must be positive".into(),
            ));
        }
        (q, q * price)
    } else if let Some(u) = order.usd_amount {
        if u <= 0.0 {
            return Err(TitanError::Validation("USD amount must be positive".into()));
        }
        let calculated_qty = (u / price * 1_000_000.0).floor() / 1_000_000.0;
        (calculated_qty, u)
    } else {
        return Err(TitanError::Validation(
            "Either USD amount or Share quantity must be provided".into(),
        ));
    };

    if qty <= 0.0 {
        return Err(TitanError::Validation("Calculated qty is zero".into()));
    }

    {
        let max_usd = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .max_manual_order_usd;
        if let Some(usd_dec) = Decimal::from_f64_retain(usd) {
            if usd_dec > max_usd {
                return Err(TitanError::Validation(format!(
                    "Order ${:.0} exceeds max_manual_order_usd limit (${:.0})",
                    usd, max_usd
                )));
            }
        }
    }

    // Determine order type (market/limit/stop)
    let manual_order_type = order
        .order_type
        .as_deref()
        .unwrap_or("market")
        .to_lowercase();
    let manual_limit_price = order.limit_price;

    let legacy = LegacyOrder {
        symbol,
        side: order.side,
        qty,
        take_profit: order.take_profit,
        stop_loss: order.stop_loss,
    };
    let order_type_tag = if manual_order_type != "market" {
        format!(
            " [{}@${:.2}]",
            manual_order_type.to_uppercase(),
            manual_limit_price.unwrap_or(0.0)
        )
    } else {
        String::new()
    };
    let bracket_info = if legacy.take_profit.is_some() || legacy.stop_loss.is_some() {
        format!(
            " [TP:{} SL:{}]",
            legacy
                .take_profit
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "--".into()),
            legacy
                .stop_loss
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "--".into())
        )
    } else {
        String::new()
    };
    let _ = app.emit_all(
        "log-message",
        format!(
            "MANUAL TRADE: {} {} ${:.2} ({:.6} shares @ ${:.2}){}{}",
            legacy.side.to_uppercase(),
            legacy.symbol,
            usd,
            qty,
            price,
            order_type_tag,
            bracket_info
        ),
    );

    // If order type is limit or stop, handle directly (Alpaca needs explicit type field)
    if manual_order_type == "limit" || manual_order_type == "stop" {
        let lp = manual_limit_price.unwrap_or(price);
        if lp <= 0.0 {
            return Err(TitanError::Validation(
                "Limit/Stop price must be > 0".into(),
            ));
        }
        if state
            .inner()
            .risk
            .circuit_breaker
            .entries_blocked
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(TitanError::Validation(
                "Circuit breaker is active -- entries blocked".into(),
            ));
        }
        {
            let settings = state
                .inner()
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let pos_count = settings.positions.len();
            if pos_count >= settings.max_open_positions as usize {
                return Err(TitanError::Validation(format!(
                    "Max positions reached ({}/{})",
                    pos_count, settings.max_open_positions
                )));
            }
        }
        // DRY RUN CHECK
        if state
            .inner()
            .dry_run
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let msg = format!(
                "[DRY RUN] WOULD SEND: {} {} x{:.4} {} TYPE:{} PRICE:${:.2}",
                legacy.side.to_uppercase(),
                legacy.symbol,
                qty,
                legacy.symbol,
                manual_order_type.to_uppercase(),
                lp
            );
            let _ = app.emit_all("log-message", &msg);
            return Ok(format!(
                "DRY_RUN_OK: {} {} {} @${:.2}",
                legacy.side, qty, legacy.symbol, lp
            ));
        }

        let (key, secret, base_url) = {
            let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
        };
        if key.is_empty() || secret.is_empty() {
            return Err(TitanError::Validation("API KEYS MISSING".into()));
        }

        let is_crypto = legacy.symbol.contains("USD") || legacy.symbol.contains("/");
        let tif = if is_crypto { "gtc" } else { "day" };

        let body = if manual_order_type == "limit" {
            serde_json::json!({
                "symbol": legacy.symbol,
                "qty": qty,
                "side": legacy.side.to_lowercase(),
                "type": "limit",
                "limit_price": format!("{:.2}", lp),
                "time_in_force": tif
            })
        } else {
            // stop order
            serde_json::json!({
                "symbol": legacy.symbol,
                "qty": qty,
                "side": legacy.side.to_lowercase(),
                "type": "stop",
                "stop_price": format!("{:.2}", lp),
                "time_in_force": tif
            })
        };

        let client = state.http_client.clone();
        let url = format!("{}/v2/orders", base_url.trim_end_matches('/'));
        let _permit = crate::state::acquire_api_permit(&state).await?;
        let res = client
            .post(&url)
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .json(&body)
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
        let _ = app.emit_all(
            "log-message",
            format!(
                "{} ORDER SENT: {} {} x{:.4} @ ${:.2}",
                manual_order_type.to_uppercase(),
                legacy.side.to_uppercase(),
                legacy.symbol,
                qty,
                lp
            ),
        );
        let _ = app.emit_all(
            "manual-order-placed",
            serde_json::json!({
                "symbol": legacy.symbol,
                "side": legacy.side,
                "qty": qty,
                "order_type": manual_order_type.to_uppercase(),
                "price": lp,
                "usd": usd
            }),
        );
        return Ok(format!(
            "{} ORDER SENT @ ${:.2}",
            manual_order_type.to_uppercase(),
            lp
        ));
    }

    let result = send_order_internal(state.inner().clone(), legacy.into(), app).await?;
    Ok(format!("ORDER SENT [{}]", result.broker_id))
}

#[tauri::command]
pub(crate) async fn emergency_liquidate(
    confirm_token: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if confirm_token != "CONFIRM_EMERGENCY" {
        return Err(TitanError::Validation(
            "Missing confirmation token. Pass 'CONFIRM_EMERGENCY' to confirm.".into(),
        ));
    }
    state
        .risk
        .circuit_breaker
        .is_armed
        .store(false, std::sync::atomic::Ordering::SeqCst);
    state
        .risk
        .circuit_breaker
        .kill_switch
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state
        .risk
        .circuit_breaker
        .entries_blocked
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let _ = app.emit_all("log-message", "EMERGENCY LIQUIDATION TRIGGERED");
    let _ = app.emit_all(
        "breaker-triggered",
        serde_json::json!({ "level": "PANIC", "reason": "Emergency liquidate by user" }),
    );

    liquidate_all_positions("CONFIRM_LIQUIDATE".into(), state.clone(), app.clone()).await?;
    cancel_all_orders("CONFIRM_CANCEL".into(), state, app).await?;

    Ok("EMERGENCY LIQUIDATION COMPLETE".into())
}

#[tauri::command]
pub(crate) async fn calc_position_size(
    equity: f64,
    risk_pct: f64,
    entry_price: f64,
    stop_loss_price: f64,
    state: State<'_, Arc<TitanState>>,
) -> Result<serde_json::Value, TitanError> {
    use crate::risk;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal::Decimal;

    let breaker_mult = state.risk.circuit_breaker.size_multiplier();
    let sizing = risk::calculate_position_size(&risk::SizingParams {
        equity: Decimal::try_from(equity).unwrap_or(Decimal::ZERO),
        risk_pct: Decimal::try_from(risk_pct).unwrap_or(rust_decimal_macros::dec!(2)),
        entry_price: Decimal::try_from(entry_price).unwrap_or(Decimal::ZERO),
        stop_loss_price: Decimal::try_from(stop_loss_price).unwrap_or(Decimal::ZERO),
        breaker_multiplier: breaker_mult,
        max_notional_frac: rust_decimal_macros::dec!(0.25),
        min_stop_frac: rust_decimal_macros::dec!(0.0001),
    });

    match sizing {
        Some(s) => Ok(serde_json::json!({
            "qty": s.qty.to_f64().unwrap_or(0.0),
            "notional": s.notional.to_f64().unwrap_or(0.0),
            "risk_amount": s.risk_amount.to_f64().unwrap_or(0.0),
        })),
        None => Err(TitanError::Validation(
            "Cannot calculate position size (invalid inputs)".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyOrder;
    use crate::types::LegacyOrder as TypesLegacyOrder;

    #[test]
    fn legacy_order_from_command_maps_to_types_legacy_order() {
        let cmd = LegacyOrder {
            symbol: "AAPL".into(),
            side: "buy".into(),
            qty: 12.5,
            take_profit: Some(200.0),
            stop_loss: Some(150.0),
        };
        let t: TypesLegacyOrder = cmd.into();
        assert_eq!(t.symbol, "AAPL");
        assert_eq!(t.side, "buy");
        assert_eq!(t.qty, 12.5);
        assert_eq!(t.take_profit, Some(200.0));
        assert_eq!(t.stop_loss, Some(150.0));
    }

    #[test]
    fn legacy_order_from_command_optional_brackets_default() {
        let cmd = LegacyOrder {
            symbol: "QQQ".into(),
            side: "sell".into(),
            qty: 1.0,
            take_profit: None,
            stop_loss: None,
        };
        let t: TypesLegacyOrder = cmd.into();
        assert_eq!(t.take_profit, None);
        assert_eq!(t.stop_loss, None);
    }

    #[test]
    fn max_manual_order_usd_enforcement() {
        use rust_decimal::Decimal;
        use rust_decimal_macros::dec;

        let max_usd = dec!(50000);
        let order_usd = 75000.0_f64;

        if let Some(usd_dec) = Decimal::from_f64_retain(order_usd) {
            assert!(usd_dec > max_usd, "Order should exceed limit");
        }

        let within_usd = 30000.0_f64;
        if let Some(usd_dec) = Decimal::from_f64_retain(within_usd) {
            assert!(usd_dec <= max_usd, "Order should be within limit");
        }
    }
}

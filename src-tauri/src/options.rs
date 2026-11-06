use crate::app_state::Position;
use crate::error::TitanError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionContract {
    pub symbol: String,
    pub underlying: String,
    pub strike: f64,
    pub expiry: String,
    pub option_type: String,
    pub bid: f64,
    pub ask: f64,
}

pub struct HedgeSuggestion {
    pub contract_symbol: String,
    #[allow(dead_code)]
    pub strike: f64,
    #[allow(dead_code)]
    pub premium: f64,
    #[allow(dead_code)]
    pub option_type: String,
}

pub async fn fetch_option_chain(
    state: &std::sync::Arc<crate::TitanState>,
    underlying: &str,
    expiry: &str,
) -> Result<Vec<OptionContract>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    let url = format!(
        "https://paper-api.alpaca.markets/v2/options/contracts?underlying_symbols={}&expiration_date={}&status=active&limit=100",
        underlying, expiry
    );
    let client = state.http_client.clone();
    let resp = client
        .get(&url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Broker(format!("Options API error: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Options API {} — {}",
            status, body
        )));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| TitanError::Broker(format!("Options parse: {}", e)))?;
    let contracts = data
        .get("option_contracts")
        .or_else(|| data.as_array().map(|_| &data))
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|c| {
            Some(OptionContract {
                symbol: c.get("symbol")?.as_str()?.to_string(),
                underlying: c.get("underlying_symbol")?.as_str()?.to_string(),
                strike: c
                    .get("strike_price")?
                    .as_str()?
                    .parse()
                    .ok()
                    .or_else(|| c.get("strike_price")?.as_f64())?,
                expiry: c.get("expiration_date")?.as_str()?.to_string(),
                option_type: c.get("type")?.as_str()?.to_string(),
                bid: c
                    .get("bid")
                    .and_then(|p| p.as_str()?.parse().ok())
                    .or_else(|| c.get("bid").and_then(|p| p.as_f64()))
                    .unwrap_or_else(|| {
                        let close = c
                            .get("close_price")
                            .and_then(|p| p.as_str()?.parse().ok())
                            .or_else(|| c.get("close_price").and_then(|p| p.as_f64()))
                            .unwrap_or(0.0);
                        close * 0.98
                    }),
                ask: c
                    .get("ask")
                    .and_then(|p| p.as_str()?.parse().ok())
                    .or_else(|| c.get("ask").and_then(|p| p.as_f64()))
                    .unwrap_or_else(|| {
                        let close = c
                            .get("close_price")
                            .and_then(|p| p.as_str()?.parse().ok())
                            .or_else(|| c.get("close_price").and_then(|p| p.as_f64()))
                            .unwrap_or(0.0);
                        close * 1.02
                    }),
            })
        })
        .collect();

    Ok(contracts)
}

pub async fn submit_option_order(
    state: &std::sync::Arc<crate::TitanState>,
    contract_symbol: &str,
    qty: f64,
    side: &str,
) -> Result<String, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };

    let payload = serde_json::json!({
        "symbol": contract_symbol,
        "qty": qty.to_string(),
        "side": side,
        "type": "market",
        "time_in_force": "day",
    });

    let client = state.http_client.clone();
    let resp = client
        .post("https://paper-api.alpaca.markets/v2/orders")
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .json(&payload)
        .send()
        .await
        .map_err(|e| TitanError::Broker(format!("Option order error: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Option order failed {} — {}",
            status, body
        )));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| TitanError::Broker(format!("Option order parse: {}", e)))?;
    let order_id = result
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("unknown");
    Ok(format!("Order {} submitted", order_id))
}

pub fn suggest_hedge(
    position: &Position,
    chain: &[OptionContract],
    risk_budget: f64,
    otm_pct: f64,
) -> Option<HedgeSuggestion> {
    use rust_decimal::prelude::ToPrimitive;
    let is_long = matches!(position.side, crate::app_state::OrderSide::Buy);
    let target_type = if is_long { "put" } else { "call" };
    let cur_price = position.current_price.to_f64().unwrap_or(0.0);
    let target_strike = if is_long {
        cur_price * (1.0 - otm_pct / 100.0)
    } else {
        cur_price * (1.0 + otm_pct / 100.0)
    };

    let mut best: Option<&OptionContract> = None;
    let mut best_dist = f64::MAX;

    for c in chain {
        if !c.option_type.eq_ignore_ascii_case(target_type) {
            continue;
        }
        if c.bid <= 0.0 && c.ask <= 0.0 {
            continue;
        }
        let mid = if c.ask > 0.0 && c.bid > 0.0 {
            (c.ask + c.bid) / 2.0
        } else if c.ask > 0.0 {
            c.ask
        } else {
            c.bid
        };
        if mid * 100.0 > risk_budget {
            continue;
        }
        let spread_pct = if c.bid > 0.0 {
            (c.ask - c.bid) / c.bid
        } else {
            0.5
        };
        if spread_pct > 0.20 {
            continue;
        }
        let dist = (c.strike - target_strike).abs();
        if dist < best_dist {
            best_dist = dist;
            best = Some(c);
        }
    }

    best.map(|c| {
        let premium = if c.ask > 0.0 && c.bid > 0.0 {
            (c.ask + c.bid) / 2.0
        } else {
            c.ask.max(c.bid)
        };
        HedgeSuggestion {
            contract_symbol: c.symbol.clone(),
            strike: c.strike,
            premium,
            option_type: c.option_type.clone(),
        }
    })
}

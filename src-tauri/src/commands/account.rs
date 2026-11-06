use std::fs;
use std::sync::Arc;
use std::time::Instant;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::TitanError;
use crate::state::{
    default_base_url, parse_amount, save_wallet_to_disk, GpuScoreCache, TitanState, VolCache,
};
use crate::types::{AlpacaAccount, AlpacaPosition, WalletSnapshot};

#[derive(Serialize, Clone)]
pub(crate) struct LegacyAppState {
    cpu_usage: f64,
    cpu_temp: f64,
    gpu_temp: f64,
    equity: f64,
}

#[derive(Serialize)]
pub(crate) struct PositionView {
    symbol: String,
    qty: f64,
    side: String,
    current_price: f64,
    avg_entry_price: f64,
    market_value: f64,
    unrealized_pl: f64,
    unrealized_plpc: f64,
    entry_tag: String,
    fsm_state: String,
}

#[derive(Serialize)]
pub(crate) struct AccountSnapshot {
    equity: f64,
    cash: f64,
    portfolio_value: f64,
    pnl: f64,
    positions: Vec<PositionView>,
}

#[tauri::command]
pub(crate) async fn update_api_keys(
    key: String,
    secret: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    connect_api(key, secret, state, app).await
}

#[tauri::command]
pub(crate) async fn init_api_connection(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }
    connect_api(key, secret, state, app).await
}

#[tauri::command]
pub(crate) async fn connect_api(
    key: String,
    secret: String,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<String, TitanError> {
    if key.len() < 5 || secret.len() < 5 {
        return Err(TitanError::Validation("API key / secret too short".into()));
    }

    let base_url = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        if keys.base_url.is_empty() {
            default_base_url()
        } else {
            keys.base_url.clone()
        }
    };

    let client = state.http_client.clone();
    let res = client
        .get(format!("{}/v2/account", base_url))
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("API validation failed: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Invalid API keys ({}) — {}",
            status, txt
        )));
    }

    {
        let mut keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        keys.key = key.clone();
        keys.secret = secret.clone();
        keys.base_url = base_url;
        let path = crate::paths::keys_path();
        if let Err(e) = fs::write(&path, serde_json::to_string(&*keys).unwrap_or_default()) {
            tracing::error!(component = "account", path = %path.display(), "Failed to persist API keys: {}", e);
        }
    }

    let warm_key = key.clone();
    let warm_secret = secret.clone();
    tauri::async_runtime::spawn(async move {
        match crate::scanner::fetch_active_assets(&warm_key, &warm_secret).await {
            Ok(assets) => tracing::info!(
                component = "account",
                "Asset cache warmed after API connect ({} assets)",
                assets.len()
            ),
            Err(e) => tracing::warn!(component = "account", "Asset cache warmup failed: {}", e),
        }
    });

    let _ = app.emit_all("log-message", "🔌 API CONNECTED: KEYS VALIDATED & SECURED");
    if let Ok(mut tx) = state.stream_tx.lock() {
        if let Some(sender) = tx.as_mut() {
            if sender.try_send(crate::streamer::StreamCmd::Reconnect).is_err() {
                tracing::warn!(component = "streamer", "Feed stale: streamer channel full on Reconnect");
            }
        }
    }
    Ok("CONNECTED".into())
}

#[tauri::command]
pub(crate) async fn get_app_state(
    state: State<'_, Arc<TitanState>>,
) -> Result<LegacyAppState, TitanError> {
    let mut sys = state.system.lock().unwrap_or_else(|e| e.into_inner());
    let mut net = state.net_tracker.lock().unwrap_or_else(|e| e.into_inner());
    let mut gpu_cache = state.gpu_cache.lock().unwrap_or_else(|e| e.into_inner());

    let hw = crate::monitor::collect_hardware_stats(&mut sys, &mut net, &mut gpu_cache);
    let wallet = state.wallet.lock().unwrap_or_else(|e| e.into_inner());

    Ok(LegacyAppState {
        cpu_usage: hw.cpu_usage,
        cpu_temp: hw.cpu_temp,
        gpu_temp: hw.gpu_temp,
        equity: (wallet.bot_cash + wallet.user_deposit)
            .to_f64()
            .unwrap_or(0.0),
    })
}

#[tauri::command]
pub(crate) async fn get_wallet_snapshot(
    state: State<'_, Arc<TitanState>>,
) -> Result<WalletSnapshot, TitanError> {
    let wallet = state.wallet.lock().unwrap_or_else(|e| e.into_inner());
    Ok(WalletSnapshot {
        bot_cash: wallet.bot_cash.to_f64().unwrap_or(0.0),
        user_deposit: wallet.user_deposit.to_f64().unwrap_or(0.0),
    })
}

#[tauri::command]
pub(crate) async fn topup_user(
    amount: f64,
    state: State<'_, Arc<TitanState>>,
) -> Result<WalletSnapshot, TitanError> {
    let amount = parse_amount(amount)?;
    let dec =
        Decimal::try_from(amount).map_err(|_| TitanError::Validation("INVALID AMOUNT".into()))?;
    let mut wallet = state.wallet.lock().unwrap_or_else(|e| e.into_inner());
    wallet.user_deposit += dec;
    let _ = save_wallet_to_disk(&wallet);
    Ok(WalletSnapshot {
        bot_cash: wallet.bot_cash.to_f64().unwrap_or(0.0),
        user_deposit: wallet.user_deposit.to_f64().unwrap_or(0.0),
    })
}

#[tauri::command]
pub(crate) async fn topup_bot(
    amount: f64,
    state: State<'_, Arc<TitanState>>,
) -> Result<WalletSnapshot, TitanError> {
    let amount = parse_amount(amount)?;
    let dec =
        Decimal::try_from(amount).map_err(|_| TitanError::Validation("INVALID AMOUNT".into()))?;
    let mut wallet = state.wallet.lock().unwrap_or_else(|e| e.into_inner());
    wallet.bot_cash += dec;
    let _ = save_wallet_to_disk(&wallet);
    Ok(WalletSnapshot {
        bot_cash: wallet.bot_cash.to_f64().unwrap_or(0.0),
        user_deposit: wallet.user_deposit.to_f64().unwrap_or(0.0),
    })
}

#[tauri::command]
pub(crate) async fn withdraw_bot(
    amount: f64,
    state: State<'_, Arc<TitanState>>,
) -> Result<WalletSnapshot, TitanError> {
    let amount = parse_amount(amount)?;
    let dec =
        Decimal::try_from(amount).map_err(|_| TitanError::Validation("INVALID AMOUNT".into()))?;
    let mut wallet = state.wallet.lock().unwrap_or_else(|e| e.into_inner());
    if wallet.bot_cash < dec {
        return Err(TitanError::Validation("INSUFFICIENT BOT FUNDS".into()));
    }
    wallet.bot_cash -= dec;
    let _ = save_wallet_to_disk(&wallet);
    Ok(WalletSnapshot {
        bot_cash: wallet.bot_cash.to_f64().unwrap_or(0.0),
        user_deposit: wallet.user_deposit.to_f64().unwrap_or(0.0),
    })
}

#[tauri::command]
pub(crate) async fn clear_cache(state: State<'_, Arc<TitanState>>) -> Result<String, TitanError> {
    if let Ok(mut cache) = state.market.bars_cache.lock() {
        cache.clear();
    }
    if let Ok(mut cache) = state.market.vol_cache.lock() {
        *cache = VolCache::new();
    }
    if let Ok(mut cache) = state.gpu_score_cache.lock() {
        *cache = GpuScoreCache {
            momentum: 0.0,
            ema_delta: 0.0,
            updated: Instant::now(),
        };
    }
    if let Ok(mut cache) = state.market.last_prices.lock() {
        cache.clear();
    }
    if let Ok(mut cache) = state.trading.trail_highs.lock() {
        cache.clear();
    }
    if let Ok(mut cache) = state.trading.trail_lows.lock() {
        cache.clear();
    }
    if let Ok(mut cache) = state.trading.partial_taken.lock() {
        cache.clear();
    }

    Ok("CACHE CLEARED".into())
}

#[tauri::command]
pub(crate) async fn get_account_snapshot(
    state: State<'_, Arc<TitanState>>,
) -> Result<AccountSnapshot, TitanError> {
    let (key, secret, base_url) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
    };
    if key.is_empty() || secret.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let client = state.http_client.clone();
    let account_url = format!("{}/v2/account", base_url.trim_end_matches('/'));
    let positions_url = format!("{}/v2/positions", base_url.trim_end_matches('/'));

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
    let cash = account.cash.parse::<f64>().unwrap_or(0.0);
    let portfolio_value = account.portfolio_value.parse::<f64>().unwrap_or(0.0);
    let last_equity = account.last_equity.parse::<f64>().unwrap_or(0.0);
    let pnl = equity - last_equity;

    let positions_res = client
        .get(&positions_url)
        .header("APCA-API-KEY-ID", &key)
        .header("APCA-API-SECRET-KEY", &secret)
        .send()
        .await
        .map_err(|e| TitanError::Network(format!("Net Error: {}", e)))?;

    if !positions_res.status().is_success() {
        let status = positions_res.status();
        let txt = positions_res.text().await.unwrap_or_default();
        return Err(TitanError::Broker(format!(
            "Alpaca API Error {}: {}",
            status, txt
        )));
    }

    let raw_positions: Vec<AlpacaPosition> = positions_res
        .json()
        .await
        .map_err(|e| TitanError::Internal(format!("JSON Error: {}", e)))?;
    let blacklist = state
        .trading.position_blacklist
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let tags_snapshot = state
        .trading.entry_tags
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let fsm_states: std::collections::HashMap<String, String> = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.positions
            .iter()
            .map(|p| (p.symbol.clone(), format!("{:?}", p.state)))
            .collect()
    };
    let positions = raw_positions
        .into_iter()
        .filter(|p| !blacklist.contains(&p.symbol))
        .map(|p| {
            let tag = tags_snapshot.get(&p.symbol).cloned().unwrap_or_default();
            let fsm = fsm_states.get(&p.symbol).cloned().unwrap_or_else(|| "Open".to_string());
            PositionView {
                symbol: p.symbol,
                qty: p.qty.parse::<f64>().unwrap_or(0.0),
                side: p.side,
                current_price: p
                    .current_price
                    .as_deref()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0),
                avg_entry_price: p
                    .avg_entry_price
                    .as_deref()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0),
                market_value: p.market_value.parse::<f64>().unwrap_or(0.0),
                unrealized_pl: p.unrealized_pl.parse::<f64>().unwrap_or(0.0),
                unrealized_plpc: p.unrealized_plpc.parse::<f64>().unwrap_or(0.0),
                entry_tag: tag,
                fsm_state: fsm,
            }
        })
        .collect();

    Ok(AccountSnapshot {
        equity,
        cash,
        portfolio_value,
        pnl,
        positions,
    })
}

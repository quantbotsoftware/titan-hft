use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use rust_decimal::Decimal;

use crate::state::TitanState;
use crate::types::AlpacaAccount;

#[tracing::instrument(skip_all, name = "equity_sync")]
pub(crate) async fn equity_sync_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    let mut fail_count: u32 = 0;
    let mut last_success = std::time::Instant::now();

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("equity_sync", 0);

        let (key, secret, base_url) = {
            let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (keys.key.clone(), keys.secret.clone(), keys.base_url.clone())
        };
        if key.is_empty() || secret.is_empty() {
            continue;
        }

        let client = state.http_client.clone();

        let url = format!("{}/v2/account", base_url.trim_end_matches('/'));
        let res = match client
            .get(&url)
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                fail_count += 1;
                if fail_count <= 3 || fail_count.is_multiple_of(20) {
                    let _ = app.emit_all(
                        "log-message",
                        format!("EQUITY SYNC: Failed to reach Alpaca ({})", e),
                    );
                }
                let elapsed = last_success.elapsed().as_secs();
                if fail_count >= 5 {
                    tracing::warn!(component = "equity_sync", fail_count, elapsed, "Equity sync critical");
                    let _ = app.emit_all("equity-sync-critical", serde_json::json!({
                        "fail_count": fail_count,
                        "last_success_ago_secs": elapsed,
                    }));
                } else if fail_count >= 3 {
                    let _ = app.emit_all("equity-sync-degraded", serde_json::json!({
                        "fail_count": fail_count,
                        "last_success_ago_secs": elapsed,
                    }));
                }
                continue;
            }
        };

        if !res.status().is_success() {
            fail_count += 1;
            let elapsed = last_success.elapsed().as_secs();
            if fail_count >= 5 {
                tracing::warn!(component = "equity_sync", fail_count, elapsed, "Equity sync critical (HTTP)");
                let _ = app.emit_all("equity-sync-critical", serde_json::json!({
                    "fail_count": fail_count,
                    "last_success_ago_secs": elapsed,
                }));
            } else if fail_count >= 3 {
                let _ = app.emit_all("equity-sync-degraded", serde_json::json!({
                    "fail_count": fail_count,
                    "last_success_ago_secs": elapsed,
                }));
            }
            continue;
        }

        let account: AlpacaAccount = match res.json().await {
            Ok(a) => a,
            Err(_) => continue,
        };

        let equity = account.equity.parse::<f64>().unwrap_or(0.0);
        let cash = account.cash.parse::<f64>().unwrap_or(0.0);
        let buying_power = account.buying_power.parse::<f64>().unwrap_or(0.0);

        if equity > 0.0 {
            fail_count = 0;
            last_success = std::time::Instant::now();

            if let Ok(dec_equity) = rust_decimal::Decimal::try_from(equity) {
                if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                    analytics.update_equity(dec_equity);
                }
            }

            if let Ok(mut wallet) = state.wallet.lock() {
                wallet.bot_cash = Decimal::try_from(cash).unwrap_or(Decimal::ZERO);
            }

            {
                let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                settings.equity = Decimal::try_from(equity).unwrap_or(settings.equity);
                settings.buying_power = Decimal::try_from(buying_power).unwrap_or(Decimal::ZERO);
            }

            let _ = app.emit_all(
                "equity-sync",
                serde_json::json!({
                    "equity": equity,
                    "cash": cash,
                    "buying_power": buying_power,
                    "portfolio_value": account.portfolio_value.parse::<f64>().unwrap_or(0.0),
                    "last_equity": account.last_equity.parse::<f64>().unwrap_or(0.0),
                }),
            );
        }
    }
}

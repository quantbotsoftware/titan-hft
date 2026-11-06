use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::sentiment;
use crate::state::TitanState;

pub(crate) async fn news_poll_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(90));
    tokio::time::sleep(Duration::from_secs(10)).await;

    loop {
        let tick_start = std::time::Instant::now();
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }

        let (symbol, key, secret) = {
            let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            let k = keys.key.clone();
            let s = keys.secret.clone();
            drop(keys);
            let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (settings.active_symbol.clone(), k, s)
        };

        if key.is_empty() || secret.is_empty() || symbol.is_empty() || symbol == "---" {
            state.watchdog.heartbeat("news", tick_start.elapsed().as_millis() as u64);
            continue;
        }

        let articles =
            sentiment::fetch_news_feed(&state.http_client, &symbol, &key, &secret, 20).await;
        if articles.is_empty() {
            state.watchdog.heartbeat("news", tick_start.elapsed().as_millis() as u64);
            continue;
        }

        let overall =
            articles.iter().map(|a| a.sentiment_score).sum::<f64>() / articles.len() as f64;
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

        let _ = app.emit_all("sentiment-update", serde_json::json!({
            "score": overall,
            "label": if overall >= 0.15 { "BULLISH" } else if overall > -0.15 { "NEUTRAL" } else { "BEARISH" },
        }));

        state.watchdog.heartbeat("news", tick_start.elapsed().as_millis() as u64);
    }
}

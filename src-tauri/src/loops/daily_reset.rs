use chrono::{Datelike, Timelike};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::commands::data::send_discord_alert;
use crate::state::TitanState;
#[cfg(feature = "telegram")]
use crate::telegram;
use crate::watchdog;
use rust_decimal::prelude::*;

pub(crate) async fn daily_reset_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    let mut last_reset_day: Option<u32> = None;

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("daily_reset", 0);

        let now = chrono::Utc::now();
        let offset_hours = if crate::state::is_us_dst(now) { 4 } else { 5 };
        let tz = chrono::FixedOffset::west_opt(offset_hours * 3600)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("UTC offset"));
        let eastern = now.with_timezone(&tz);
        let day = eastern.ordinal();
        let hour = eastern.hour();
        let minute = eastern.minute();
        let past_market_open = hour > 9 || (hour == 9 && minute >= 30);

        // Reset once per day on or after 09:30 ET (robust: won't miss if tick skips minute 30)
        if past_market_open && last_reset_day != Some(day) {
            save_session_report_now(&state);
            last_reset_day = Some(day);
            if let Ok(mut analytics) = state.risk.session_analytics.lock() {
                analytics.reset_daily();
            }
            if let Ok(mut cvd) = state.market.cvd.lock() {
                cvd.clear();
            }
            state.risk.circuit_breaker.manual_reset();

            {
                let mut st_acct = state.st_account.lock().unwrap_or_else(|e| e.into_inner());
                let was_tripped = st_acct.breaker_triggered;
                let pnl_today = st_acct.realized_pnl_today;
                let trades_today = st_acct.trade_count_today;
                let wins_today = st_acct.win_count_today;
                st_acct.daily_reset();
                let _ = st_acct.save_to_disk();
                if was_tripped {
                    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    settings.smarttrend.st_bot_paused = false;
                }
                tracing::info!(
                    component = "daily_reset",
                    pnl_today,
                    trades_today,
                    wins_today,
                    capital = st_acct.allocated_capital,
                    breaker_was_tripped = was_tripped,
                    "ST Account daily reset"
                );
            }
            let _ = app.emit_all("smarttrend-daily-reset", "daily_reset");

            let _ = app.emit_all(
                "log-message",
                "DAILY RESET: Session saved, analytics cleared, breaker reset, ST account reset",
            );

            // Discord + Telegram daily summary
            let (webhook, alert_daily, tg_token, tg_chat, tg_enabled) = {
                let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                (
                    s.discord_webhook_url.clone(),
                    s.alert_on_daily_summary,
                    s.telegram_bot_token.clone(),
                    s.telegram_chat_id.clone(),
                    s.telegram_enabled,
                )
            };
            let journal_stats = {
                let j = state
                    .trade_journal
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                j.get_stats()
            };
            if alert_daily && !webhook.is_empty() {
                let summary = format!(
                    "Trades: {} | Win Rate: {:.1}% | PnL: ${:.2} | PF: {:.2}",
                    journal_stats.total_trades,
                    journal_stats.win_rate.to_f64().unwrap_or(0.0),
                    journal_stats.total_pnl.to_f64().unwrap_or(0.0),
                    journal_stats.profit_factor.to_f64().unwrap_or(0.0),
                );
                let wh = webhook.clone();
                let st = state.clone();
                tokio::spawn(async move {
                    send_discord_alert(&st, &wh, "DAILY SUMMARY", &summary, 0x3683D4).await;
                });
            }
            #[cfg(feature = "telegram")]
            if tg_enabled {
                let tg_msg = telegram::format_daily_summary(&journal_stats);
                let hc = state.http_client.clone();
                tokio::spawn(async move {
                    telegram::send_telegram_alert(&hc, &tg_token, &tg_chat, &tg_msg).await;
                });
            }
        }
    }
}

pub(crate) fn save_session_report_now(state: &Arc<TitanState>) {
    let analytics = state
        .risk
        .session_analytics
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let telemetry = state.watchdog.snapshot();
    let champion = state.genetic_optimizer.lock().ok().map(|opt| {
        let params = opt.current_params();
        serde_json::to_value(&*params).unwrap_or_default()
    });
    let is_dry = state.dry_run.load(std::sync::atomic::Ordering::SeqCst);

    let report = watchdog::SessionReport {
        date: chrono::Utc::now().format("%Y-%m-%d_%H%M%S").to_string(),
        total_trades: analytics.total_trades,
        winning_trades: analytics.winning_trades,
        losing_trades: analytics.losing_trades,
        daily_pnl: analytics.daily_pnl.to_string(),
        max_drawdown_pct: analytics.max_drawdown_pct.to_string(),
        win_rate_pct: analytics.win_rate_pct.to_string(),
        profit_factor: analytics.profit_factor.to_string(),
        sharpe: analytics.sharpe_ratio(),
        peak_equity: analytics.peak_equity.to_string(),
        final_equity: analytics.current_equity.to_string(),
        champion_params: champion,
        loop_telemetry: telemetry,
        breaker_events: 0,
        dry_run: is_dry,
    };
    watchdog::save_session_report(&report);
}

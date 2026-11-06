use crate::journal::JournalStats;
use crate::sentiment::CatalystEvent;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;

static TG_RATE_LIMITER: std::sync::LazyLock<std::sync::Mutex<std::collections::VecDeque<std::time::Instant>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::VecDeque::with_capacity(25)));

const TG_RATE_WINDOW_SECS: u64 = 60;
const TG_RATE_MAX_MESSAGES: usize = 20;

pub async fn send_telegram_alert(
    client: &reqwest::Client,
    token: &str,
    chat_id: &str,
    message: &str,
) {
    if token.is_empty() || chat_id.is_empty() {
        return;
    }

    {
        let now = std::time::Instant::now();
        let cutoff = now - std::time::Duration::from_secs(TG_RATE_WINDOW_SECS);
        let mut ring = TG_RATE_LIMITER.lock().unwrap_or_else(|e| e.into_inner());
        while ring.front().is_some_and(|&t| t < cutoff) {
            ring.pop_front();
        }
        if ring.len() >= TG_RATE_MAX_MESSAGES {
            tracing::warn!(component = "telegram", "Rate limit exceeded ({}/min), dropping message", TG_RATE_MAX_MESSAGES);
            return;
        }
        ring.push_back(now);
    }

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "HTML",
        "disable_web_page_preview": true,
    });
    match client.post(&url).json(&payload).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::warn!(
                    component = "telegram",
                    status = %resp.status(),
                    "Telegram sendMessage failed"
                );
            }
        }
        Err(e) => {
            tracing::warn!(component = "telegram", error = %e, "Telegram send error");
        }
    }
}

pub async fn telegram_command_loop(state: Arc<crate::TitanState>) {
    let mut last_update_id: i64 = 0;
    let client = state.http_client.clone();
    let mut pending_confirm: Option<String> = None;

    loop {
        let tick_start = std::time::Instant::now();
        let (token, chat_id, enabled) = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (
                s.telegram_bot_token.clone(),
                s.telegram_chat_id.clone(),
                s.telegram_enabled,
            )
        };
        if !enabled || token.is_empty() || chat_id.is_empty() {
            state.watchdog.heartbeat("telegram", tick_start.elapsed().as_millis() as u64);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        }

        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30&allowed_updates=[\"message\"]",
            token, last_update_id
        );
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(component = "telegram", error = %e, "getUpdates failed");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(component = "telegram", error = %e, "getUpdates parse failed");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Some(results) = body.get("result").and_then(|r| r.as_array()) {
            for update in results {
                if let Some(uid) = update.get("update_id").and_then(|u| u.as_i64()) {
                    last_update_id = uid + 1;
                }
                let msg = match update.get("message") {
                    Some(m) => m,
                    None => continue,
                };
                let from_chat = msg
                    .get("chat")
                    .and_then(|c| c.get("id"))
                    .and_then(|i| i.as_i64())
                    .unwrap_or(0)
                    .to_string();
                if from_chat != chat_id {
                    continue;
                }
                let text = msg
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_lowercase();

                tracing::info!(component = "telegram", command = %text, "Telegram command received");

                let reply = if let Some(ref pending_cmd) = pending_confirm {
                    if text == "yes" || text == "da" || text == "/confirm" {
                        let result = execute_critical_command(pending_cmd, &state);
                        pending_confirm = None;
                        result
                    } else {
                        pending_confirm = None;
                        "❌ Command cancelled.".to_string()
                    }
                } else if text == "/start" || text == "/stop" {
                    pending_confirm = Some(text.clone());
                    let action = if text == "/start" { "START" } else { "STOP" };
                    format!(
                        "⚠️ <b>CONFIRM: {} autopilot?</b>\n\nReply <b>yes</b> to confirm or anything else to cancel.",
                        action
                    )
                } else {
                    handle_command(&text, &state)
                };
                send_telegram_alert(&client, &token, &chat_id, &reply).await;
            }
        }

        state.watchdog.heartbeat("telegram", tick_start.elapsed().as_millis() as u64);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

fn execute_critical_command(cmd: &str, state: &Arc<crate::TitanState>) -> String {
    match cmd {
        "/start" => cmd_start(state),
        "/stop" => cmd_stop(state),
        _ => "❓ Unknown pending command".to_string(),
    }
}

fn handle_command(cmd: &str, state: &Arc<crate::TitanState>) -> String {
    let cmd = cmd.trim();
    match cmd {
        "/status" => cmd_status(state),
        "/positions" => cmd_positions(state),
        "/trades" => cmd_trades(state),
        "/stop" => cmd_stop(state),
        "/start" => cmd_start(state),
        "/stats" => cmd_stats(state),
        "/help" => cmd_help(),
        _ => "❓ Unknown command. Send /help for available commands.".to_string(),
    }
}

fn cmd_status(state: &Arc<crate::TitanState>) -> String {
    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let breaker = state.risk.circuit_breaker.level();
    let positions = s.positions.len();
    let equity = s.equity.to_f64().unwrap_or(0.0);
    let autopilot = if s.auto_pilot { "ON ✅" } else { "OFF ❌" };
    let strategy = &s.strategy_mode_name;
    let dry_run = if s.dry_run_enabled { " [DRY RUN]" } else { "" };

    format!(
        "📊 <b>TITAN STATUS</b>{dry}\n\n\
         💰 Equity: <b>${eq:.2}</b>\n\
         🤖 Autopilot: <b>{ap}</b>\n\
         🎯 Strategy: <b>{strat}</b>\n\
         🛡️ Breaker: <b>{breaker}</b>\n\
         📦 Positions: <b>{pos}</b>\n\
         🏦 Buying Power: <b>${bp:.2}</b>",
        dry = dry_run,
        eq = equity,
        ap = autopilot,
        strat = strategy,
        breaker = breaker,
        pos = positions,
        bp = s.buying_power.to_f64().unwrap_or(0.0),
    )
}

fn cmd_positions(state: &Arc<crate::TitanState>) -> String {
    let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if s.positions.is_empty() {
        return "📦 <b>No open positions</b>".to_string();
    }
    let mut lines = vec!["📦 <b>OPEN POSITIONS</b>\n".to_string()];
    for p in &s.positions {
        let pnl_f = p.pnl.to_f64().unwrap_or(0.0);
        let icon = if pnl_f >= 0.0 { "🟢" } else { "🔴" };
        let side_str = match p.side {
            crate::app_state::OrderSide::Buy => "LONG",
            crate::app_state::OrderSide::Sell => "SHORT",
        };
        lines.push(format!(
            "{icon} <b>{sym}</b> {side} x{qty:.4}\n  Entry: ${entry:.2} | Now: ${now:.2}\n  PnL: <b>${pnl:.2}</b> ({pct:+.2}%)\n",
            icon = icon,
            sym = p.symbol,
            side = side_str,
            qty = p.qty.to_f64().unwrap_or(0.0),
            entry = p.entry_price.to_f64().unwrap_or(0.0),
            now = p.current_price.to_f64().unwrap_or(0.0),
            pnl = pnl_f,
            pct = p.pnl_percent.to_f64().unwrap_or(0.0),
        ));
    }
    lines.join("")
}

fn cmd_trades(state: &Arc<crate::TitanState>) -> String {
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let trades = journal.get_trades(5, 0);
    if trades.is_empty() {
        return "📒 <b>No trades recorded yet</b>".to_string();
    }
    let mut lines = vec!["📒 <b>LAST 5 TRADES</b>\n".to_string()];
    for t in trades.iter().rev() {
        let icon = if t.pnl >= Decimal::ZERO { "✅" } else { "❌" };
        lines.push(format!(
            "{icon} <b>{sym}</b> {side} | PnL: <b>${pnl:.2}</b> | {ts}\n",
            icon = icon,
            sym = t.symbol,
            side = t.side.to_uppercase(),
            pnl = t.pnl.to_f64().unwrap_or(0.0),
            ts = if t.timestamp.len() >= 16 { &t.timestamp[..16] } else { &t.timestamp },
        ));
    }
    lines.join("")
}

fn cmd_stop(state: &Arc<crate::TitanState>) -> String {
    let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if !s.auto_pilot {
        return "🤖 Autopilot is already <b>OFF</b>".to_string();
    }
    s.auto_pilot = false;
    "🛑 Autopilot <b>STOPPED</b>".to_string()
}

fn cmd_start(state: &Arc<crate::TitanState>) -> String {
    let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if s.auto_pilot {
        return "🤖 Autopilot is already <b>ON</b>".to_string();
    }
    s.auto_pilot = true;
    "✅ Autopilot <b>STARTED</b>".to_string()
}

fn cmd_stats(state: &Arc<crate::TitanState>) -> String {
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let st = journal.get_stats();
    format_daily_summary(&st)
}

fn cmd_help() -> String {
    "🤖 <b>TITAN COMMANDS</b>\n\n\
     /status — Equity, autopilot, breaker, positions\n\
     /positions — Open positions with PnL\n\
     /trades — Last 5 trades from journal\n\
     /stop — Stop autopilot\n\
     /start — Start autopilot\n\
     /stats — Journal stats (win rate, PnL, PF)\n\
     /help — This message"
        .to_string()
}

pub fn format_trade_fill(
    symbol: &str,
    side: &str,
    qty: f64,
    price: f64,
    pnl: Option<f64>,
) -> String {
    let icon = if side.eq_ignore_ascii_case("buy") {
        "🟢"
    } else {
        "🔴"
    };
    let pnl_line = match pnl {
        Some(p) if p != 0.0 => {
            let pnl_icon = if p > 0.0 { "💰" } else { "📉" };
            format!("\n{} PnL: <b>${:.2}</b>", pnl_icon, p)
        }
        _ => String::new(),
    };
    format!(
        "{icon} <b>TRADE FILL</b>\n\
         {side} <b>{symbol}</b> x{qty:.4} @ ${price:.2}{pnl_line}",
        icon = icon,
        side = side.to_uppercase(),
        symbol = symbol,
        qty = qty,
        price = price,
        pnl_line = pnl_line,
    )
}

pub fn format_breaker_alert(level: &str, reason: &str) -> String {
    format!(
        "🚨 <b>CIRCUIT BREAKER: {level}</b>\n\
         {reason}\n\
         ⚠️ All positions liquidated",
        level = level,
        reason = reason,
    )
}

pub fn format_catalyst(event: &CatalystEvent) -> String {
    format!(
        "🔥 <b>CATALYST: {symbol}</b>\n\
         Score: {score:.2}\n\
         {headline}\n\
         <i>{summary}</i>",
        symbol = event.symbol,
        score = event.sentiment_score,
        headline = event.headline,
        summary = if event.summary.len() > 200 {
            format!("{}...", &event.summary[..200])
        } else {
            event.summary.clone()
        },
    )
}

pub fn format_daily_summary(stats: &JournalStats) -> String {
    let icon = if stats.total_pnl >= Decimal::ZERO {
        "📊"
    } else {
        "📉"
    };
    format!(
        "{icon} <b>DAILY SUMMARY</b>\n\
         Trades: {trades} | Win Rate: {wr:.1}%\n\
         PnL: <b>${pnl:.2}</b> | PF: {pf:.2}\n\
         Best: ${best:.2} | Worst: ${worst:.2}",
        icon = icon,
        trades = stats.total_trades,
        wr = stats.win_rate.to_f64().unwrap_or(0.0),
        pnl = stats.total_pnl.to_f64().unwrap_or(0.0),
        pf = stats.profit_factor.to_f64().unwrap_or(0.0),
        best = stats.best_trade.to_f64().unwrap_or(0.0),
        worst = stats.worst_trade.to_f64().unwrap_or(0.0),
    )
}

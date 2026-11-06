use axum::{
    extract::{State as AxumState, WebSocketUpgrade, ws::Message},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tower_http::cors::{CorsLayer, AllowOrigin};

type SharedState = Arc<crate::TitanState>;

const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const RATE_LIMIT_MAX_REQUESTS: usize = 60;
const WS_MSG_BUDGET_PER_TICK: u32 = 10;

#[derive(Clone)]
struct RateLimiterState {
    requests: Arc<std::sync::Mutex<VecDeque<Instant>>>,
}

impl RateLimiterState {
    fn new() -> Self {
        Self {
            requests: Arc::new(std::sync::Mutex::new(
                VecDeque::with_capacity(RATE_LIMIT_MAX_REQUESTS + 1),
            )),
        }
    }

    fn check(&self) -> bool {
        let now = Instant::now();
        let cutoff =
            now - std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
        let mut ring =
            self.requests.lock().unwrap_or_else(|e| e.into_inner());
        while ring.front().is_some_and(|&t| t < cutoff) {
            ring.pop_front();
        }
        if ring.len() >= RATE_LIMIT_MAX_REQUESTS {
            return false;
        }
        ring.push_back(now);
        true
    }
}

#[allow(clippy::result_large_err)]
fn verify_bearer_token(
    headers: &HeaderMap,
    expected_token: &str,
) -> Result<(), Response> {
    if expected_token.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Dashboard token not configured"})),
        )
            .into_response());
    }

    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = auth.strip_prefix("Bearer ").unwrap_or(auth);

    if token.is_empty() || token != expected_token {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or missing token"})),
        )
            .into_response());
    }

    Ok(())
}

static AUTH_FAIL_COUNT: AtomicU64 = AtomicU64::new(0);

async fn rate_limit_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let limiter = req.extensions().get::<RateLimiterState>().cloned();
    if let Some(lim) = limiter {
        if !lim.check() {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "Rate limit exceeded (60 req/min)"})),
            )
                .into_response();
        }
    }
    next.run(req).await
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

async fn auth_middleware(
    AxumState(state): AxumState<SharedState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let expected_token = {
        let s = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        s.web_dashboard_token.clone()
    };

    if let Err(resp) = verify_bearer_token(req.headers(), &expected_token)
    {
        let count = AUTH_FAIL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(5) {
            tracing::error!(
                component = "web_dashboard",
                consecutive_failures = count,
                "Repeated auth failures -- possible brute-force attempt"
            );
        } else {
            tracing::warn!(
                component = "web_dashboard",
                "Auth failure (total: {})",
                count
            );
        }
        return resp;
    }

    AUTH_FAIL_COUNT.store(0, Ordering::Relaxed);
    next.run(req).await
}

static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub async fn start_web_dashboard(state: SharedState, port: u16) {
    START_TIME.get_or_init(Instant::now);

    let limiter = RateLimiterState::new();

    let origin_127 = format!("http://127.0.0.1:{}", port)
        .parse()
        .expect("valid origin");
    let origin_localhost = format!("http://localhost:{}", port)
        .parse()
        .expect("valid origin");

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([origin_127, origin_localhost]))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    let health_router = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/build-info", get(api_build_info));

    let ws_router = Router::new()
        .route("/ws", get(ws_handler))
        .route_layer(middleware::from_fn(rate_limit_middleware));

    let protected_router = Router::new()
        .route("/", get(dashboard_html))
        .route("/api/status", get(api_status))
        .route("/api/positions", get(api_positions))
        .route("/api/trades", get(api_trades))
        .route("/api/stats", get(api_stats))
        .route("/api/autopilot", post(api_autopilot_toggle))
        .route_layer(middleware::from_fn(rate_limit_middleware))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = health_router
        .merge(ws_router)
        .merge(protected_router)
        .layer(axum::Extension(limiter))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    tracing::info!(
        component = "web_dashboard",
        "Starting web dashboard on http://{}",
        addr
    );

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                component = "web_dashboard",
                "Failed to bind {}: {}",
                addr,
                e
            );
            return;
        }
    };
    let _ = axum::serve(listener, app).await;
}

async fn api_health() -> impl IntoResponse {
    let uptime = START_TIME
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs();
    Json(serde_json::json!({
        "status": "ok",
        "uptime_secs": uptime,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn api_build_info() -> impl IntoResponse {
    let uptime = START_TIME
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs();
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": env!("TITAN_GIT_HASH"),
        "build_date": env!("TITAN_BUILD_DATE"),
        "uptime_secs": uptime,
    }))
}

async fn api_status(
    AxumState(state): AxumState<SharedState>,
) -> impl IntoResponse {
    let s = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let breaker = state.risk.circuit_breaker.level();
    let eq = s.equity.to_f64().unwrap_or(0.0);
    Json(serde_json::json!({
        "equity": eq,
        "buying_power": eq,
        "autopilot": s.auto_pilot,
        "strategy": s.strategy_mode_name,
        "breaker": format!("{}", breaker),
        "dry_run": s.dry_run_enabled,
        "positions_count": s.positions.len(),
        "multi_symbol": s.multi_symbol_enabled,
    }))
}

async fn api_positions(
    AxumState(state): AxumState<SharedState>,
) -> impl IntoResponse {
    let s = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let positions: Vec<serde_json::Value> = s
        .positions
        .iter()
        .map(|p| {
            serde_json::json!({
                "symbol": p.symbol,
                "qty": p.qty.to_f64().unwrap_or(0.0),
                "side": format!("{:?}", p.side),
                "entry_price": p.entry_price.to_f64().unwrap_or(0.0),
                "current_price": p.current_price.to_f64().unwrap_or(0.0),
                "pnl": p.pnl.to_f64().unwrap_or(0.0),
                "pnl_percent": p.pnl_percent.to_f64().unwrap_or(0.0),
            })
        })
        .collect();
    Json(positions)
}

async fn api_trades(
    AxumState(state): AxumState<SharedState>,
) -> impl IntoResponse {
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let trades = journal.get_trades(20, 0);
    let result: Vec<serde_json::Value> = trades
        .iter()
        .rev()
        .map(|t| {
            let entry_summary = t.entry_features.as_ref().map(|ef| {
                serde_json::json!({
                    "oracle_score": ef.oracle_score,
                    "oracle_regime": ef.oracle_regime,
                    "rsi": ef.rsi,
                    "spread_bps": ef.spread_bps,
                    "strategy_score": ef.strategy_score,
                    "signal_source": ef.signal_source,
                })
            });
            let exit_summary = t.exit_features.as_ref().map(|ef| {
                serde_json::json!({
                    "oracle_score": ef.oracle_score,
                    "oracle_regime": ef.oracle_regime,
                    "rsi": ef.rsi,
                    "spread_bps": ef.spread_bps,
                })
            });
            serde_json::json!({
                "timestamp": t.timestamp,
                "symbol": t.symbol,
                "side": t.side,
                "qty": t.qty.to_f64().unwrap_or(0.0),
                "entry_price": t.entry_price.to_f64().unwrap_or(0.0),
                "exit_price": t.exit_price.to_f64().unwrap_or(0.0),
                "pnl": t.pnl.to_f64().unwrap_or(0.0),
                "pnl_pct": t.pnl_pct.to_f64().unwrap_or(0.0),
                "reason": t.reason,
                "duration_secs": t.duration_secs,
                "strategy_mode": t.strategy_mode,
                "broker_order_id": t.broker_order_id,
                "entry_features": entry_summary,
                "exit_features": exit_summary,
            })
        })
        .collect();
    Json(result)
}

async fn api_stats(
    AxumState(state): AxumState<SharedState>,
) -> impl IntoResponse {
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let st = journal.get_stats();
    Json(serde_json::json!({
        "total_trades": st.total_trades,
        "winning_trades": st.winning_trades,
        "losing_trades": st.losing_trades,
        "win_rate": st.win_rate.to_f64().unwrap_or(0.0),
        "total_pnl": st.total_pnl.to_f64().unwrap_or(0.0),
        "avg_pnl": st.avg_pnl.to_f64().unwrap_or(0.0),
        "best_trade": st.best_trade.to_f64().unwrap_or(0.0),
        "worst_trade": st.worst_trade.to_f64().unwrap_or(0.0),
        "profit_factor": st.profit_factor.to_f64().unwrap_or(0.0),
    }))
}

async fn api_autopilot_toggle(
    AxumState(state): AxumState<SharedState>,
) -> impl IntoResponse {
    let new_state = {
        let mut s = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        s.auto_pilot = !s.auto_pilot;
        s.auto_pilot
    };
    Json(serde_json::json!({
        "autopilot": new_state,
        "message": if new_state { "Autopilot STARTED" } else { "Autopilot STOPPED" }
    }))
}

async fn ws_handler(
    AxumState(state): AxumState<SharedState>,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let expected_token = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.web_dashboard_token.clone()
    };

    let query_token = query.get("token").cloned().unwrap_or_default();
    let bearer_ok = verify_bearer_token(&headers, &expected_token).is_ok();
    let query_ok = !query_token.is_empty() && query_token == expected_token;

    if !bearer_ok && !query_ok {
        AUTH_FAIL_COUNT.fetch_add(1, Ordering::Relaxed);
        tracing::warn!(component = "web_dashboard", "WS auth failure (no valid token in header or query)");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or missing token"})),
        )
            .into_response();
    }
    AUTH_FAIL_COUNT.store(0, Ordering::Relaxed);

    ws.on_upgrade(move |mut socket| async move {
        let mut rx = state.ws_broadcast.subscribe();
        let mut status_interval =
            tokio::time::interval(std::time::Duration::from_secs(2));
        let mut client_msg_budget = WS_MSG_BUDGET_PER_TICK;

        loop {
            tokio::select! {
                _ = status_interval.tick() => {
                    client_msg_budget = WS_MSG_BUDGET_PER_TICK;
                    let status = build_ws_status(&state);
                    if socket.send(Message::Text(status)).await.is_err() {
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if socket.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::debug!(
                                component = "ws_dashboard",
                                "Client lagged by {} messages", n
                            );
                        }
                        Err(_) => break,
                    }
                }
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(_)) => {
                            if client_msg_budget == 0 {
                                tracing::warn!(
                                    component = "ws_dashboard",
                                    "Client exceeded WS message rate limit, disconnecting"
                                );
                                break;
                            }
                            client_msg_budget = client_msg_budget.saturating_sub(1);
                        }
                        Some(Err(_)) | None => break,
                    }
                }
            }
        }
    })
}

fn build_ws_status(state: &SharedState) -> String {
    let s = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let breaker = state.risk.circuit_breaker.level();
    let latency = state.latency_metrics.snapshot();
    let positions: Vec<serde_json::Value> = s
        .positions
        .iter()
        .map(|p| {
            serde_json::json!({
                "symbol": p.symbol,
                "side": format!("{:?}", p.side),
                "qty": p.qty.to_f64().unwrap_or(0.0),
                "pnl": p.pnl.to_f64().unwrap_or(0.0),
                "state": format!("{:?}", p.state),
            })
        })
        .collect();

    serde_json::json!({
        "type": "status",
        "equity": s.equity.to_f64().unwrap_or(0.0),
        "autopilot": s.auto_pilot,
        "strategy": s.strategy_mode_name,
        "breaker": format!("{}", breaker),
        "positions": positions,
        "latency": {
            "order_p50": latency.order_rtt_us.p50,
            "order_p95": latency.order_rtt_us.p95,
            "gpu_p50": latency.gpu_dispatch_us.p50,
            "gpu_p95": latency.gpu_dispatch_us.p95,
            "streamer_p50": latency.streamer_tick_us.p50,
            "streamer_p95": latency.streamer_tick_us.p95,
        },
    })
    .to_string()
}

async fn dashboard_html(
    AxumState(state): AxumState<SharedState>,
) -> Html<String> {
    let (equity, autopilot, strategy, positions, dry_run, breaker, token) =
        {
            let s = state
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let b = state.risk.circuit_breaker.level();
            (
                s.equity.to_f64().unwrap_or(0.0),
                s.auto_pilot,
                s.strategy_mode_name.clone(),
                s.positions.clone(),
                s.dry_run_enabled,
                format!("{}", b),
                s.web_dashboard_token.clone(),
            )
        };
    let journal = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let stats = journal.get_stats();
    let recent_trades = journal.get_trades(10, 0);

    let ap_cls = if autopilot { "val-green" } else { "val-red" };
    let ap_text = if autopilot { "ON" } else { "OFF" };
    let dry_badge = if dry_run {
        r#" <span class="dry-tag">DRY RUN</span>"#
    } else {
        ""
    };

    let mut pos_rows = String::new();
    for p in &positions {
        let pnl_f = p.pnl.to_f64().unwrap_or(0.0);
        let pnl_cls = if pnl_f >= 0.0 {
            "val-green"
        } else {
            "val-red"
        };
        let side = match p.side {
            crate::app_state::OrderSide::Buy => "LONG",
            crate::app_state::OrderSide::Sell => "SHORT",
        };
        pos_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:.4}</td><td>${:.2}</td>\
             <td>${:.2}</td><td class='{}'>${:.2} ({:+.2}%)</td></tr>",
            escape_html(&p.symbol),
            side,
            p.qty.to_f64().unwrap_or(0.0),
            p.entry_price.to_f64().unwrap_or(0.0),
            p.current_price.to_f64().unwrap_or(0.0),
            pnl_cls,
            pnl_f,
            p.pnl_percent.to_f64().unwrap_or(0.0)
        ));
    }
    if positions.is_empty() {
        pos_rows = "<tr><td colspan='6' class='empty-row'>\
                    No open positions</td></tr>"
            .to_string();
    }

    let mut trade_rows = String::new();
    for t in recent_trades.iter().rev() {
        let pnl_cls = if t.pnl >= Decimal::ZERO {
            "val-green"
        } else {
            "val-red"
        };
        let ts_display = if t.timestamp.len() >= 16 {
            &t.timestamp[..16]
        } else {
            &t.timestamp
        };
        trade_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td>\
             <td class='{}'>${:.2}</td></tr>",
            escape_html(ts_display),
            escape_html(&t.symbol),
            escape_html(&t.side.to_uppercase()),
            pnl_cls,
            t.pnl.to_f64().unwrap_or(0.0)
        ));
    }
    if recent_trades.is_empty() {
        trade_rows = "<tr><td colspan='4' class='empty-row'>\
                      No trades yet</td></tr>"
            .to_string();
    }

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>TITAN HFT Dashboard</title>
<style>
@import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap');
*{{margin:0;padding:0;box-sizing:border-box}}
:root{{
  --neon-cyan:#00BFFF;--neon-green:#00FF41;--neon-gold:#FFD700;
  --neon-red:#FF2B2B;--bg-dark:#000;--bg-panel:#050508;
  --bg-card:#080812;--border-dim:#111122;
  --border-cyber:rgba(0,191,255,0.4);--text-primary:#FAFAFA;
  --text-dim:#778899;--font-mono:'Share Tech Mono',monospace;
  --glow-sm:0 0 6px;--glow-md:0 0 12px;
}}
body{{background:var(--bg-dark);color:var(--text-primary);
  font-family:var(--font-mono);padding:12px;min-height:100vh}}
.hud-header{{text-align:center;margin-bottom:20px;position:relative}}
.hud-header h1{{color:var(--neon-gold);font-size:1.5em;
  letter-spacing:4px;text-transform:uppercase;
  text-shadow:var(--glow-md) var(--neon-gold)}}
.hud-header::after{{content:'';display:block;height:1px;margin-top:8px;
  background:linear-gradient(90deg,transparent,var(--neon-cyan),transparent)}}
.dry-tag{{display:inline-block;padding:2px 8px;
  background:rgba(255,43,43,0.2);border:1px solid var(--neon-red);
  border-radius:3px;color:var(--neon-red);font-size:0.6em;
  letter-spacing:2px;vertical-align:middle;margin-left:8px;
  animation:pulse-red 2s ease-in-out infinite}}
@keyframes pulse-red{{0%,100%{{opacity:1}}50%{{opacity:0.5}}}}
@keyframes scan-line{{0%{{top:-100%}}100%{{top:200%}}}}
.cards{{display:grid;
  grid-template-columns:repeat(auto-fit,minmax(140px,1fr));
  gap:10px;margin-bottom:20px}}
.card{{background:linear-gradient(135deg,var(--bg-card),var(--bg-panel));
  border:1px solid var(--border-dim);border-radius:6px;
  padding:14px 10px;text-align:center;position:relative;
  overflow:hidden;transition:border-color .25s,box-shadow .25s}}
.card:hover{{border-color:var(--border-cyber);
  box-shadow:var(--glow-sm) var(--neon-cyan)}}
.card .label{{font-size:.65em;color:var(--text-dim);
  text-transform:uppercase;letter-spacing:2px}}
.card .value{{font-size:1.4em;font-weight:700;color:var(--neon-gold);
  margin-top:6px;text-shadow:var(--glow-sm) rgba(255,215,0,0.3)}}
.val-green{{color:var(--neon-green)!important;
  text-shadow:var(--glow-sm) rgba(0,255,65,0.3)!important}}
.val-red{{color:var(--neon-red)!important;
  text-shadow:var(--glow-sm) rgba(255,43,43,0.3)!important}}
.section{{background:linear-gradient(180deg,var(--bg-card),var(--bg-panel));
  border:1px solid var(--border-dim);border-radius:6px;
  padding:14px;margin-bottom:14px;position:relative;overflow:hidden}}
.section::before{{content:'';position:absolute;top:0;left:0;right:0;
  height:2px;
  background:linear-gradient(90deg,transparent,var(--neon-cyan),transparent)}}
.section::after{{content:'';position:absolute;top:-100%;left:0;right:0;
  height:50%;
  background:linear-gradient(180deg,rgba(0,191,255,0.03),transparent);
  animation:scan-line 8s linear infinite;pointer-events:none}}
.section h2{{color:var(--neon-cyan);font-size:.9em;letter-spacing:2px;
  text-transform:uppercase;margin-bottom:10px;
  text-shadow:var(--glow-sm) rgba(0,191,255,0.3)}}
table{{width:100%;border-collapse:collapse;font-size:.8em}}
th{{background:rgba(0,191,255,0.08);color:var(--neon-gold);
  padding:8px 6px;text-align:left;
  border-bottom:1px solid var(--border-cyber);
  font-size:.85em;letter-spacing:1px;text-transform:uppercase}}
td{{padding:7px 6px;border-bottom:1px solid var(--border-dim)}}
tr:nth-child(even){{background:rgba(0,191,255,0.02)}}
tr:hover{{background:rgba(0,191,255,0.06)}}
.empty-row{{text-align:center;color:var(--text-dim);font-style:italic}}
#ws-status{{text-align:center;font-size:.75em;margin-top:18px;
  padding:6px;border-radius:4px;transition:all .3s}}
.ws-live{{color:var(--neon-green);
  text-shadow:var(--glow-sm) rgba(0,255,65,0.4)}}
.ws-dead{{color:var(--neon-red);
  text-shadow:var(--glow-sm) rgba(255,43,43,0.4)}}
.ws-alert{{color:var(--neon-gold);
  text-shadow:var(--glow-sm) rgba(255,215,0,0.4)}}
.build-info{{display:flex;justify-content:center;gap:8px;
  margin-top:14px;flex-wrap:wrap}}
.build-tag{{display:inline-block;padding:2px 10px;
  border:1px solid var(--border-cyber);border-radius:3px;
  color:var(--neon-cyan);font-size:.65em;letter-spacing:1px;
  background:rgba(0,191,255,0.05);
  text-shadow:var(--glow-sm) rgba(0,191,255,0.2)}}
.build-tag:hover{{box-shadow:var(--glow-sm) var(--neon-cyan);
  border-color:var(--neon-cyan)}}
.footer{{text-align:center;color:var(--text-dim);font-size:.6em;
  margin-top:6px;opacity:.5;letter-spacing:1px}}
@media(max-width:600px){{body{{padding:6px}}
  .cards{{grid-template-columns:repeat(2,1fr);gap:6px}}
  .card .value{{font-size:1.1em}}
  .hud-header h1{{font-size:1.1em}}}}
</style>
</head>
<body>
<div class="hud-header"><h1>TITAN HFT{dry_badge}</h1></div>
<div class="cards">
  <div class="card"><div class="label">Equity</div>
    <div class="value" id="eq-val">${equity:.2}</div></div>
  <div class="card"><div class="label">PnL Today</div>
    <div class="value {pnl_cls}" id="pnl-val">${pnl:.2}</div></div>
  <div class="card"><div class="label">Win Rate</div>
    <div class="value">{wr:.1}%</div></div>
  <div class="card"><div class="label">Positions</div>
    <div class="value" id="pos-count">{pos_count}</div></div>
  <div class="card"><div class="label">Autopilot</div>
    <div class="value {ap_cls}" id="ap-val">{ap_text}</div></div>
  <div class="card"><div class="label">Strategy</div>
    <div class="value" style="font-size:.9em">{strategy}</div></div>
  <div class="card"><div class="label">Breaker</div>
    <div class="value" style="font-size:.9em" id="brk-val">{breaker}</div></div>
  <div class="card"><div class="label">Trades</div>
    <div class="value">{total_trades}</div></div>
</div>
<div class="section">
  <h2>Open Positions</h2>
  <div style="overflow-x:auto"><table>
    <thead><tr><th>Symbol</th><th>Side</th><th>Qty</th>
      <th>Entry</th><th>Current</th><th>PnL</th></tr></thead>
    <tbody id="pos-body">{pos_rows}</tbody>
  </table></div>
</div>
<div class="section">
  <h2>Recent Trades</h2>
  <div style="overflow-x:auto"><table>
    <thead><tr><th>Time</th><th>Symbol</th><th>Side</th>
      <th>PnL</th></tr></thead>
    <tbody>{trade_rows}</tbody>
  </table></div>
</div>
<div id="ws-status" class="ws-dead">Connecting WebSocket...</div>
<div class="build-info">
  <span class="build-tag">v{version}</span>
  <span class="build-tag">{git_hash}</span>
  <span class="build-tag">{build_date}</span>
</div>
<p class="footer">TITAN HFT | WebSocket Live Push</p>
<script>
(function(){{
  var token='{token}';
  var proto=location.protocol==='https:'?'wss:':'ws:';
  var ws,reconDelay=1000;
  function connect(){{
    ws=new WebSocket(proto+'//'+location.host+'/ws?token='+encodeURIComponent(token));
    ws.onopen=function(){{
      var el=document.getElementById('ws-status');
      el.textContent='WEBSOCKET LIVE';el.className='ws-live';
      reconDelay=1000;
    }};
    ws.onmessage=function(e){{
      try{{
        var d=JSON.parse(e.data);
        if(d.type==='status'){{
          var eq=document.getElementById('eq-val');
          if(eq)eq.textContent='$'+(d.equity||0).toFixed(2);
          var ap=document.getElementById('ap-val');
          if(ap){{ap.textContent=d.autopilot?'ON':'OFF';
            ap.className='value '+(d.autopilot?'val-green':'val-red')}}
          var brk=document.getElementById('brk-val');
          if(brk)brk.textContent=d.breaker||'';
          var pc=document.getElementById('pos-count');
          if(pc&&d.positions)pc.textContent=d.positions.length;
        }}
        if(d.type==='trade'){{
          var el=document.getElementById('ws-status');
          el.textContent='TRADE: '+(d.symbol||'')+' '+(d.side||'')+
            ' $'+(d.pnl||0).toFixed(2);
          el.className=(d.pnl||0)>=0?'ws-live':'ws-dead';
          setTimeout(function(){{el.textContent='WEBSOCKET LIVE';
            el.className='ws-live'}},5000);
        }}
        if(d.type==='alert'){{
          var el=document.getElementById('ws-status');
          el.textContent='ALERT: '+(d.message||'');
          el.className='ws-alert';
        }}
      }}catch(_){{}}
    }};
    ws.onclose=function(){{
      var el=document.getElementById('ws-status');
      el.textContent='RECONNECTING...';el.className='ws-dead';
      setTimeout(connect,reconDelay);
      reconDelay=Math.min(reconDelay*2,30000);
    }};
    ws.onerror=function(){{ws.close()}};
  }}
  connect();
}})();
</script>
</body>
</html>"##,
        dry_badge = dry_badge,
        equity = equity,
        pnl = stats.total_pnl.to_f64().unwrap_or(0.0),
        pnl_cls = if stats.total_pnl >= Decimal::ZERO {
            "val-green"
        } else {
            "val-red"
        },
        wr = stats.win_rate.to_f64().unwrap_or(0.0),
        pos_count = positions.len(),
        ap_cls = ap_cls,
        ap_text = ap_text,
        strategy = escape_html(&strategy),
        breaker = breaker,
        total_trades = stats.total_trades,
        pos_rows = pos_rows,
        trade_rows = trade_rows,
        token = escape_html(&token),
        version = env!("CARGO_PKG_VERSION"),
        git_hash = env!("TITAN_GIT_HASH"),
        build_date = env!("TITAN_BUILD_DATE"),
    );

    Html(html)
}

/// Tauri command: rotate the dashboard token (generates new random token).
#[tauri::command]
pub async fn rotate_dashboard_token(
    state: tauri::State<'_, SharedState>,
) -> Result<String, crate::error::TitanError> {
    let new_token = {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    };
    {
        let mut s = state
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        s.web_dashboard_token = new_token.clone();
        let snapshot = crate::state::settings_from_state(&s);
        let _ = crate::state::save_settings_to_disk(&snapshot);
    }
    tracing::info!(
        component = "web_dashboard",
        "Dashboard token rotated successfully"
    );
    Ok(new_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_bearer_valid_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer abc123".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "abc123").is_ok());
    }

    #[test]
    fn verify_bearer_wrong_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer wrong".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "abc123").is_err());
    }

    #[test]
    fn verify_bearer_missing_header() {
        let headers = HeaderMap::new();
        assert!(verify_bearer_token(&headers, "abc123").is_err());
    }

    #[test]
    fn verify_bearer_empty_expected() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer foo".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "").is_err());
    }

    #[test]
    fn rate_limiter_allows_under_limit() {
        let lim = RateLimiterState::new();
        for _ in 0..RATE_LIMIT_MAX_REQUESTS {
            assert!(lim.check());
        }
    }

    #[test]
    fn rate_limiter_blocks_over_limit() {
        let lim = RateLimiterState::new();
        for _ in 0..RATE_LIMIT_MAX_REQUESTS {
            lim.check();
        }
        assert!(!lim.check());
    }

    #[test]
    fn escape_html_special_chars() {
        assert_eq!(
            escape_html("<script>alert('xss')&\"</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&amp;&quot;\
             &lt;/script&gt;"
        );
    }

    #[test]
    fn build_info_env_vars_present() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "CARGO_PKG_VERSION must be set");
        let git_hash = env!("TITAN_GIT_HASH");
        assert!(!git_hash.is_empty(), "TITAN_GIT_HASH must be set");
        let build_date = env!("TITAN_BUILD_DATE");
        assert!(
            !build_date.is_empty(),
            "TITAN_BUILD_DATE must be set"
        );
    }

    /// FIX-5 CRIT-2: WS auth accepts token via query param.
    #[test]
    fn ws_auth_via_query_param() {
        let expected_token = "my_secret_token_123";
        let query: std::collections::HashMap<String, String> =
            [("token".to_string(), expected_token.to_string())].into_iter().collect();
        let query_token = query.get("token").cloned().unwrap_or_default();
        let headers = HeaderMap::new();
        let bearer_ok = verify_bearer_token(&headers, expected_token).is_ok();
        let query_ok = !query_token.is_empty() && query_token == expected_token;
        assert!(!bearer_ok, "Bearer should fail without header");
        assert!(query_ok, "Query param auth should succeed with matching token");
        assert!(bearer_ok || query_ok, "At least one auth method should pass");
    }

    /// FIX-5: WS auth rejects empty query token.
    #[test]
    fn ws_auth_rejects_empty_query_token() {
        let expected_token = "my_secret_token_123";
        let query: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let query_token = query.get("token").cloned().unwrap_or_default();
        let headers = HeaderMap::new();
        let bearer_ok = verify_bearer_token(&headers, expected_token).is_ok();
        let query_ok = !query_token.is_empty() && query_token == expected_token;
        assert!(!bearer_ok && !query_ok, "Auth should fail with no token at all");
    }

    /// FIX-5: WS auth rejects wrong query token.
    #[test]
    fn ws_auth_rejects_wrong_query_token() {
        let expected_token = "my_secret_token_123";
        let query: std::collections::HashMap<String, String> =
            [("token".to_string(), "wrong_token".to_string())].into_iter().collect();
        let query_token = query.get("token").cloned().unwrap_or_default();
        let headers = HeaderMap::new();
        let bearer_ok = verify_bearer_token(&headers, expected_token).is_ok();
        let query_ok = !query_token.is_empty() && query_token == expected_token;
        assert!(!bearer_ok && !query_ok, "Auth should fail with wrong query token");
    }
}

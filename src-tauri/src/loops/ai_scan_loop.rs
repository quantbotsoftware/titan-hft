use chrono::{Datelike, Timelike};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::commands::data::score_prefiltered_candidates;
use crate::scanner;
use crate::state::{
    is_us_dst, normalize_symbol, save_settings_to_disk, settings_from_state, TitanState,
};

pub(crate) async fn ai_scan_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    let shutdown = state.shutdown_token.clone();
    let mut ticks_since_scan: u32 = 30; // trigger first scan immediately
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = heartbeat_interval.tick() => {
        state.watchdog.heartbeat("ai_scanner", 60_000);

        ticks_since_scan += 1;

        if !state.settings.lock().unwrap_or_else(|e| e.into_inner()).ai_scan_active {
            continue; // AI Scanner auto-scan disabled via toggle
        }

        let now = chrono::Utc::now();
        let is_dst = is_us_dst(now);
        let utc_offset = if is_dst { 4 } else { 5 };
        let tz = chrono::FixedOffset::west_opt(utc_offset * 3600)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("UTC offset"));
        let ny = now.with_timezone(&tz);
        
        let weekday = ny.weekday().num_days_from_monday();
        if weekday >= 5 { continue; } // 5=Sat, 6=Sun

        let hour = ny.hour();
        let min = ny.minute();
        let current_mins_of_day = hour * 60 + min;
        
        let premarket_start = 4 * 60 + 5; // 04:05 ET
        let rth_start = 9 * 60 + 30; // 09:30 ET
        let rth_end = 15 * 60 + 30; // 15:30 ET
        
        let required_interval = if current_mins_of_day >= premarket_start && current_mins_of_day < rth_start {
            10 // 10 minutes in premarket
        } else if current_mins_of_day >= rth_start && current_mins_of_day < rth_end {
            30 // 30 minutes in RTH
        } else {
            0 // Don't scan outside these hours
        };
        
        if required_interval == 0 {
            continue;
        }
        
        if ticks_since_scan < required_interval {
            continue;
        }
        ticks_since_scan = 0;

        let (key, secret) = {
            let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (keys.key.clone(), keys.secret.clone())
        };
        if key.is_empty() { continue; }

        let _ = app.emit_all("log-message", "🔍 AI SCANNER: Auto-scan starting...".to_string());

        let allow_short = state.settings.lock().unwrap_or_else(|e| e.into_inner()).allow_short;
        let fusion_alpha = { *state.nexus_fusion_alpha.lock().unwrap_or_else(|e| e.into_inner()) };

        let assets = match scanner::fetch_active_assets(&key, &secret).await {
            Ok(a) => a,
            Err(e) => {
                let hint = match &e {
                    crate::error::TitanError::Timeout(_) => " (try again in a few minutes)",
                    crate::error::TitanError::Broker(_) => " (check API keys / Alpaca status)",
                    _ => "",
                };
                let _ = app.emit_all("log-message", format!("🔍 AI SCANNER ERROR [assets]: {}{}", e, hint));
                continue;
            }
        };
        let symbols: Vec<String> = assets.into_iter()
            .filter(|a| a.tradable && scanner::is_us_exchange(&a.exchange))
            .map(|a| a.symbol)
            .collect();
        tracing::info!("AI SCANNER AUTO: {} US exchange-listed symbols to scan", symbols.len());
        let _ = app.emit_all("log-message", format!("🔍 AI SCANNER: Scanning {} US stocks...", symbols.len()));

        let snaps = scanner::fetch_snapshots_batch(&key, &secret, &symbols).await;

        let scan_params = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            let mut p = s.scan_params.clone();
            p.allow_negative_change = allow_short;
            p
        };
        let prefiltered = scanner::quant_prefilter(&snaps, &scan_params);

        if prefiltered.is_empty() {
            let _ = app.emit_all("log-message", "🔍 AI SCANNER: No candidates passed pre-filter".to_string());
            continue;
        }

        let snap_count = snaps.len();
        let sr = score_prefiltered_candidates(&state, prefiltered, snap_count, allow_short, fusion_alpha, &app).await;
        let results = sr.results;
        tracing::info!("AI SCANNER AUTO: {}/{} scored, {} dropped, from {} snapshots", sr.scored, sr.total, sr.dropped, sr.snap_count);

        // --- CLEANUP: remove underperformers from hft_targets ---
        {
            let stale_symbols: Vec<String> = results.iter()
                .filter(|r| r.smart_score < 10.0)
                .map(|r| r.symbol.clone())
                .collect();
            if !stale_symbols.is_empty() {
                let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                let before = settings.hft_targets.len();
                settings.hft_targets.retain(|t| !stale_symbols.iter().any(|s| s.eq_ignore_ascii_case(t)));
                let removed = before - settings.hft_targets.len();
                if removed > 0 {
                    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
                        tracing::error!(component = "ai_scan", error = %e, "Cleanup save failed");
                    }
                    let _ = app.emit_all("log-message", format!("🧹 AI SCANNER CLEANUP: removed {} underperformers from HFT targets", removed));
                }
            }
        }

        // --- AUTO-INJECT top N into HFT Engine ---
        let (auto_inject_hft, auto_inject_n) = {
            let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            (s.ai_auto_inject_hft, s.ai_auto_inject_top as usize)
        };
        if auto_inject_hft && auto_inject_n > 0 {
            let mut smart_sorted: Vec<&scanner::AiScanCandidate> = results.iter()
                .filter(|r| r.bt_trades >= 2 && r.bt_sharpe > 0.0 && r.bt_win_rate > 20.0
                    && r.smart_score > 20.0 && r.change_percent > -5.0 && r.price >= 1.50)
                .collect();
            smart_sorted.sort_by(|a, b| b.smart_score.partial_cmp(&a.smart_score).unwrap_or(std::cmp::Ordering::Equal));
            smart_sorted.truncate(auto_inject_n);

            if !smart_sorted.is_empty() {
                let inject_symbols: Vec<String> = smart_sorted.iter().map(|r| r.symbol.clone()).collect();
                {
                    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    for sym in &inject_symbols {
                        if settings.hft_targets.len() >= 20 { break; }
                        let s = normalize_symbol(sym);
                        if !s.is_empty() && !settings.hft_targets.contains(&s) {
                            settings.hft_targets.push(s);
                        }
                    }
                    settings.multi_symbol_enabled = true;
                    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
                        tracing::error!(component = "ai_scan", error = %e, "Auto-inject save failed");
                    }
                }
                let _ = app.emit_all("log-message", format!("⚡ AI SCANNER AUTO-INJECT: {} assets → HFT Engine: {:?}", inject_symbols.len(), inject_symbols));
                let _ = app.emit_all("smart-picks-injected", serde_json::json!({
                    "symbols": inject_symbols,
                    "mode": "auto"
                }));
            }
        }
            }
        }
    }
}

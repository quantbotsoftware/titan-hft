//! Scanner-related Tauri commands (bodies match `main.rs`).

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::error::TitanError;
use crate::scanner;
use crate::state::TitanState;

#[tauri::command]
pub(crate) async fn run_market_scan(
    strategy: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<scanner::ScanResult>, TitanError> {
    let (key, secret, fmp_key) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone(), keys.fmp_api_key.clone())
    };

    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    // Read user's scan params from state and sync allow_short → allow_negative_change
    let scan_params = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let mut p = s.scan_params.clone();
        p.allow_negative_change = s.allow_short;
        p
    };

    match strategy.as_str() {
        "SHARK" | "DISCOVERY" => scanner::scan_discovery(&key, &secret, &scan_params, &fmp_key).await,
        _ => Err(TitanError::Validation("UNKNOWN STRATEGY".into())),
    }
}

#[tauri::command]
pub(crate) async fn run_ai_scan(
    max_candidates: usize,
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<Vec<scanner::AiScanCandidate>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }

    let max_c = max_candidates.clamp(10, 300);
    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({"phase": "assets", "done": 0, "total": max_c}),
    );

    let assets = match scanner::fetch_active_assets(&key, &secret).await {
        Ok(a) => a,
        Err(e) => {
            let _ = app.emit_all(
                "ai-scan-progress",
                serde_json::json!({"phase": "error", "done": 0, "total": 0, "message": e.to_string()}),
            );
            return Err(e);
        }
    };
    let symbols: Vec<String> = assets
        .into_iter()
        .filter(|a| a.tradable && scanner::is_us_exchange(&a.exchange))
        .map(|a| a.symbol)
        .collect();
    tracing::info!(
        "AI SCANNER: {} US exchange-listed symbols to scan",
        symbols.len()
    );

    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({"phase": "snapshots", "done": 0, "total": max_c}),
    );
    let _ = app.emit_all(
        "log-message",
        format!(
            "🔍 AI SCANNER: Fetching snapshots for {} US stocks...",
            symbols.len()
        ),
    );
    let snaps = scanner::fetch_snapshots_batch(&key, &secret, &symbols).await;

    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({
            "phase": "prefilter", "done": 0, "total": max_c, "total_snapshots": snaps.len()
        }),
    );

    let scan_params = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        let mut p = s.scan_params.clone();
        p.max_candidates = max_c;
        p
    };
    let prefiltered = scanner::quant_prefilter(&snaps, &scan_params);

    tracing::info!(
        "AI SCANNER: prefilter passed {}/{} snapshots",
        prefiltered.len(),
        snaps.len()
    );
    let _ = app.emit_all(
        "ai-scan-progress",
        serde_json::json!({
            "phase": "prefilter", "done": 1, "total": max_c,
            "passed": prefiltered.len(), "total_snapshots": snaps.len()
        }),
    );

    if prefiltered.is_empty() {
        let _ = app.emit_all(
            "log-message",
            format!(
                "🔍 AI SCANNER: 0/{} snapshots passed pre-filter (vol>={}, chg>={}%, rvol>={})",
                snaps.len(),
                scan_params.volume_min,
                scan_params.change_min,
                scan_params.rvol_min
            ),
        );
        let _ = app.emit_all(
            "ai-scan-progress",
            serde_json::json!({
                "phase": "done", "done": 0, "total": 0,
                "scored": 0, "dropped": 0, "prefiltered": 0, "total_snapshots": snaps.len()
            }),
        );
        return Ok(vec![]);
    }

    let allow_short = {
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.allow_short
    };
    let fusion_alpha = {
        *state
            .nexus_fusion_alpha
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    };

    let snap_count = snaps.len();
    let state_arc = state.inner().clone();
    let sr = crate::commands::data::score_prefiltered_candidates(
        &state_arc,
        prefiltered,
        snap_count,
        allow_short,
        fusion_alpha,
        &app,
    )
    .await;
    tracing::info!(
        "AI SCANNER: {}/{} scored, {} dropped, from {} snapshots",
        sr.scored,
        sr.total,
        sr.dropped,
        sr.snap_count
    );

    Ok(sr.results)
}

#[tauri::command]
pub(crate) async fn activate_scan_targets(
    symbols: Vec<String>,
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let targets = {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        for sym in &symbols {
            if settings.hft_targets.len() >= 20 {
                break;
            }
            let s = crate::state::normalize_symbol(sym);
            if !s.is_empty() && !settings.hft_targets.contains(&s) {
                settings.hft_targets.push(s.clone());
            }
        }
        settings.multi_symbol_enabled = true;
        let _ = crate::state::save_settings_to_disk(&crate::state::settings_from_state(&settings));
        settings.hft_targets.clone()
    };

    Ok(format!(
        "ACTIVATED {} symbols. HFT targets: {:?}",
        symbols.len(),
        targets
    ))
}

#[tauri::command]
pub(crate) async fn cancel_ai_scan(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    state
        .scan
        .scan_cancel
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .cancel();
    *state
        .scan
        .scan_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = (false, 0, 0);
    Ok("AI Scan cancelled".to_string())
}

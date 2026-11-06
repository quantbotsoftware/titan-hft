use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::data::score_candidate;
use crate::error::TitanError;
use crate::scanner;
use crate::state::TitanState;
use crate::state::{normalize_symbol, save_settings_to_disk, settings_from_state};

#[derive(Serialize)]
pub(crate) struct HftTargetsResponse {
    pub targets: Vec<String>,
    pub active_symbols: Vec<String>,
}

#[tauri::command]
pub(crate) async fn get_hft_targets(
    state: State<'_, Arc<TitanState>>,
) -> Result<HftTargetsResponse, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let mut list = settings.hft_targets.clone();
    list.sort();
    let active: Vec<String> = settings
        .positions
        .iter()
        .map(|p| p.symbol.clone())
        .collect();
    Ok(HftTargetsResponse {
        targets: list,
        active_symbols: active,
    })
}

#[tauri::command]
pub(crate) async fn add_hft_target(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<HftTargetsResponse, TitanError> {
    let sym = normalize_symbol(&symbol);
    if sym.is_empty() {
        return Err(TitanError::Validation("EMPTY SYMBOL".into()));
    }

    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if !settings.hft_targets.contains(&sym) {
        settings.hft_targets.push(sym);
    }
    let mut list = settings.hft_targets.clone();
    list.sort();
    let active: Vec<String> = settings
        .positions
        .iter()
        .map(|p| p.symbol.clone())
        .collect();
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "hft_targets", error = %e, "HFT targets save failed");
            return Err(e);
        }
    Ok(HftTargetsResponse {
        targets: list,
        active_symbols: active,
    })
}

#[tauri::command]
pub(crate) async fn remove_hft_target(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<HftTargetsResponse, TitanError> {
    let sym = normalize_symbol(&symbol);
    if sym.is_empty() {
        return Err(TitanError::Validation("EMPTY SYMBOL".into()));
    }

    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    settings.hft_targets.retain(|s| s != &sym);
    let mut list = settings.hft_targets.clone();
    list.sort();
    let active: Vec<String> = settings
        .positions
        .iter()
        .map(|p| p.symbol.clone())
        .collect();
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "hft_targets", error = %e, "HFT targets save failed");
            return Err(e);
        }
    Ok(HftTargetsResponse {
        targets: list,
        active_symbols: active,
    })
}

#[tauri::command]
pub(crate) async fn clear_all_hft_targets(
    state: State<'_, Arc<TitanState>>,
) -> Result<HftTargetsResponse, TitanError> {
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let active_set: std::collections::HashSet<String> = settings
        .positions
        .iter()
        .map(|p| p.symbol.clone())
        .collect();
    settings.hft_targets.retain(|s| active_set.contains(s));
    let mut list = settings.hft_targets.clone();
    list.sort();
    let active: Vec<String> = active_set.into_iter().collect();
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "hft_targets", error = %e, "HFT targets save failed");
            return Err(e);
        }
    Ok(HftTargetsResponse {
        targets: list,
        active_symbols: active,
    })
}

#[tauri::command]
pub(crate) async fn get_hft_target_enriched(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
) -> Result<Vec<scanner::AiScanCandidate>, TitanError> {
    let (key, secret, symbols) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (
            keys.key.clone(),
            keys.secret.clone(),
            settings.hft_targets.clone(),
        )
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }
    if symbols.is_empty() {
        return Ok(Vec::new());
    }

    let cached: Vec<scanner::AiScanCandidate> = {
        state
            .scan
            .last_scan_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    };
    let cached_map: std::collections::HashMap<String, scanner::AiScanCandidate> =
        cached.into_iter().map(|c| (c.symbol.clone(), c)).collect();

    let mut results: Vec<scanner::AiScanCandidate> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for sym in &symbols {
        if let Some(c) = cached_map.get(sym) {
            results.push(c.clone());
        } else {
            missing.push(sym.clone());
        }
    }

    if !missing.is_empty() {
        let _ = app.emit_all(
            "hft-enrich-progress",
            serde_json::json!({
                "phase": "snapshots", "done": 0, "total": missing.len(), "symbol": ""
            }),
        );

        let snaps = scanner::fetch_snapshots_batch(&key, &secret, &missing).await;

        let allow_short = {
            state
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .allow_short
        };
        let fusion_alpha = {
            *state
                .nexus_fusion_alpha
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };
        let state_arc = state.inner().clone();

        for (idx, sym) in missing.iter().enumerate() {
            let _ = app.emit_all(
                "hft-enrich-progress",
                serde_json::json!({
                    "phase": "scoring", "done": idx, "total": missing.len(), "symbol": sym
                }),
            );

            if let Some(snap) = snaps.get(sym) {
                let close = snap.daily_bar.c;
                let open = snap.daily_bar.o;
                let volume = snap.daily_bar.v;
                let prev_close = snap.prev_daily_bar.as_ref().map(|p| p.c).unwrap_or(open);
                let prev_volume = snap.prev_daily_bar.as_ref().map(|p| p.v).unwrap_or(volume);
                let change_pct = if prev_close > 0.0 {
                    ((close - prev_close) / prev_close) * 100.0
                } else {
                    0.0
                };
                let rvol = if prev_volume > 0.0 {
                    volume / prev_volume
                } else {
                    1.0
                };
                let gap_pct = if prev_close > 0.0 {
                    ((open - prev_close) / prev_close) * 100.0
                } else {
                    0.0
                };

                let pf = scanner::PreFilterCandidate {
                    symbol: sym.clone(),
                    price: close,
                    change_percent: change_pct,
                    volume,
                    prev_volume,
                    rvol,
                    gap_percent: gap_pct,
                    prefilter_score: 0.0,
                    dollar_volume: close * volume,
                    is_stale: false,
                };

                if let Some(candidate) =
                    score_candidate(&state_arc, &pf, fusion_alpha, allow_short).await
                {
                    results.push(candidate);
                } else {
                    results.push(scanner::AiScanCandidate {
                        rank: 0,
                        symbol: sym.clone(),
                        price: close,
                        change_percent: change_pct,
                        volume,
                        rvol,
                        gap_percent: gap_pct,
                        multi_day_strength: 0.0,
                        float_category: if volume > 10_000_000.0 {
                            "HIGH".into()
                        } else if volume > 3_000_000.0 {
                            "MED".into()
                        } else {
                            "LOW".into()
                        },
                        prefilter_score: 0.0,
                        oracle_score: 0.0,
                        oracle_confidence: 0.0,
                        oracle_regime: "N/A".into(),
                        nexus_consensus: "N/A".into(),
                        nexus_buy_pct: 0.0,
                        fusion_score: 0.0,
                        fusion_signal: "N/A".into(),
                        bt_best_tf: "-".into(),
                        bt_trades: 0,
                        bt_win_rate: 0.0,
                        bt_return_pct: 0.0,
                        bt_sharpe: 0.0,
                        bt_profit_factor: 0.0,
                        bt_max_drawdown: 0.0,
                        bt_expectancy: 0.0,
                        ai_score: 0.0,
                        smart_score: 0.0,
                        has_catalyst: false,
                        catalyst_headline: String::new(),
                        catalyst_url: String::new(),
                        reasoning: String::new(),
                        market_cap: 0.0,
                        insider_signal: String::new(),
                    });
                }
            }
        }

        let _ = app.emit_all(
            "hft-enrich-progress",
            serde_json::json!({
                "phase": "done", "done": missing.len(), "total": missing.len(), "symbol": ""
            }),
        );

        let new_candidates: Vec<scanner::AiScanCandidate> = results
            .iter()
            .filter(|c| missing.contains(&c.symbol))
            .cloned()
            .collect();
        if !new_candidates.is_empty() {
            let mut scan_cache = state
                .scan
                .last_scan_results
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            for nc in new_candidates {
                if !scan_cache.iter().any(|c| c.symbol == nc.symbol) {
                    scan_cache.push(nc);
                }
            }
        }
    }

    // --- FMP enrichment for CAP + INS ---
    let fmp_key = {
        state.api_keys.lock().unwrap_or_else(|e| e.into_inner()).fmp_api_key.clone()
    };
    if !fmp_key.is_empty() {
        let all_syms: Vec<String> = results.iter().map(|r| r.symbol.clone()).collect();
        let (_, profile_map) = scanner::fetch_profile_and_float_batch(&fmp_key, &all_syms).await;
        let insider_map = scanner::fetch_insider_batch(&fmp_key, &all_syms).await;
        for r in results.iter_mut() {
            if let Some(p) = profile_map.get(&r.symbol) {
                r.market_cap = p.market_cap_m;
            }
            if let Some(ins) = insider_map.get(&r.symbol) {
                r.insider_signal = ins.signal.clone();
            }
        }
    }

    results.sort_by(|a, b| {
        let ia = symbols
            .iter()
            .position(|s| s == &a.symbol)
            .unwrap_or(usize::MAX);
        let ib = symbols
            .iter()
            .position(|s| s == &b.symbol)
            .unwrap_or(usize::MAX);
        ia.cmp(&ib)
    });
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    Ok(results)
}

#[tauri::command]
pub(crate) async fn get_watchlist_enriched(
    state: State<'_, Arc<TitanState>>,
    app: AppHandle,
    symbols: Vec<String>,
) -> Result<Vec<scanner::AiScanCandidate>, TitanError> {
    let (key, secret) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        (keys.key.clone(), keys.secret.clone())
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }
    if symbols.is_empty() {
        return Ok(Vec::new());
    }

    let cached: Vec<scanner::AiScanCandidate> = {
        state
            .scan
            .last_scan_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    };
    let cached_map: std::collections::HashMap<String, scanner::AiScanCandidate> =
        cached.into_iter().map(|c| (c.symbol.clone(), c)).collect();

    let mut results: Vec<scanner::AiScanCandidate> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for sym in &symbols {
        if let Some(c) = cached_map.get(sym) {
            results.push(c.clone());
        } else {
            missing.push(sym.clone());
        }
    }

    if !missing.is_empty() {
        let _ = app.emit_all(
            "wl-enrich-progress",
            serde_json::json!({
                "phase": "snapshots", "done": 0, "total": missing.len(), "symbol": ""
            }),
        );

        let snaps = scanner::fetch_snapshots_batch(&key, &secret, &missing).await;

        let allow_short = {
            state
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .allow_short
        };
        let fusion_alpha = {
            *state
                .nexus_fusion_alpha
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };
        let state_arc = state.inner().clone();

        for (idx, sym) in missing.iter().enumerate() {
            let _ = app.emit_all(
                "wl-enrich-progress",
                serde_json::json!({
                    "phase": "scoring", "done": idx, "total": missing.len(), "symbol": sym
                }),
            );

            if let Some(snap) = snaps.get(sym) {
                let close = snap.daily_bar.c;
                let open = snap.daily_bar.o;
                let volume = snap.daily_bar.v;
                let prev_close = snap.prev_daily_bar.as_ref().map(|p| p.c).unwrap_or(open);
                let prev_volume = snap.prev_daily_bar.as_ref().map(|p| p.v).unwrap_or(volume);
                let change_pct = if prev_close > 0.0 {
                    ((close - prev_close) / prev_close) * 100.0
                } else {
                    0.0
                };
                let rvol = if prev_volume > 0.0 {
                    volume / prev_volume
                } else {
                    1.0
                };
                let gap_pct = if prev_close > 0.0 {
                    ((open - prev_close) / prev_close) * 100.0
                } else {
                    0.0
                };

                let pf = scanner::PreFilterCandidate {
                    symbol: sym.clone(),
                    price: close,
                    change_percent: change_pct,
                    volume,
                    prev_volume,
                    rvol,
                    gap_percent: gap_pct,
                    prefilter_score: 0.0,
                    dollar_volume: close * volume,
                    is_stale: false,
                };

                if let Some(candidate) =
                    score_candidate(&state_arc, &pf, fusion_alpha, allow_short).await
                {
                    results.push(candidate);
                } else {
                    results.push(scanner::AiScanCandidate {
                        rank: 0,
                        symbol: sym.clone(),
                        price: close,
                        change_percent: change_pct,
                        volume,
                        rvol,
                        gap_percent: gap_pct,
                        multi_day_strength: 0.0,
                        float_category: if volume > 10_000_000.0 {
                            "HIGH".into()
                        } else if volume > 3_000_000.0 {
                            "MED".into()
                        } else {
                            "LOW".into()
                        },
                        prefilter_score: 0.0,
                        oracle_score: 0.0,
                        oracle_confidence: 0.0,
                        oracle_regime: "N/A".into(),
                        nexus_consensus: "N/A".into(),
                        nexus_buy_pct: 0.0,
                        fusion_score: 0.0,
                        fusion_signal: "N/A".into(),
                        bt_best_tf: "-".into(),
                        bt_trades: 0,
                        bt_win_rate: 0.0,
                        bt_return_pct: 0.0,
                        bt_sharpe: 0.0,
                        bt_profit_factor: 0.0,
                        bt_max_drawdown: 0.0,
                        bt_expectancy: 0.0,
                        ai_score: 0.0,
                        smart_score: 0.0,
                        has_catalyst: false,
                        catalyst_headline: String::new(),
                        catalyst_url: String::new(),
                        reasoning: String::new(),
                        market_cap: 0.0,
                        insider_signal: String::new(),
                    });
                }
            }
        }

        let _ = app.emit_all(
            "wl-enrich-progress",
            serde_json::json!({
                "phase": "done", "done": missing.len(), "total": missing.len(), "symbol": ""
            }),
        );

        let new_candidates: Vec<scanner::AiScanCandidate> = results
            .iter()
            .filter(|c| missing.contains(&c.symbol))
            .cloned()
            .collect();
        if !new_candidates.is_empty() {
            let mut scan_cache = state
                .scan
                .last_scan_results
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            for nc in new_candidates {
                if !scan_cache.iter().any(|c| c.symbol == nc.symbol) {
                    scan_cache.push(nc);
                }
            }
        }
    }

    // --- FMP enrichment for CAP + INS ---
    let fmp_key = {
        state.api_keys.lock().unwrap_or_else(|e| e.into_inner()).fmp_api_key.clone()
    };
    if !fmp_key.is_empty() {
        let all_syms: Vec<String> = results.iter().map(|r| r.symbol.clone()).collect();
        let (_, profile_map) = scanner::fetch_profile_and_float_batch(&fmp_key, &all_syms).await;
        let insider_map = scanner::fetch_insider_batch(&fmp_key, &all_syms).await;
        for r in results.iter_mut() {
            if let Some(p) = profile_map.get(&r.symbol) {
                r.market_cap = p.market_cap_m;
            }
            if let Some(ins) = insider_map.get(&r.symbol) {
                r.insider_signal = ins.signal.clone();
            }
        }
    }

    results.sort_by(|a, b| {
        let ia = symbols
            .iter()
            .position(|s| s == &a.symbol)
            .unwrap_or(usize::MAX);
        let ib = symbols
            .iter()
            .position(|s| s == &b.symbol)
            .unwrap_or(usize::MAX);
        ia.cmp(&ib)
    });
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    Ok(results)
}

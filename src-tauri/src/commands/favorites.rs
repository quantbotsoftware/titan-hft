use std::sync::Arc;

use tauri::State;

use crate::error::TitanError;
use crate::scanner;
use crate::state::*;

#[tauri::command]
pub(crate) async fn get_favorites(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<String>, TitanError> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    Ok(settings.favorites.clone())
}

#[tauri::command]
pub(crate) async fn add_favorite(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<String>, TitanError> {
    let sym = symbol.trim().to_uppercase();
    if sym.is_empty() {
        return Err(TitanError::Validation("EMPTY SYMBOL".into()));
    }
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if !settings.favorites.contains(&sym) {
        settings.favorites.push(sym);
        settings.favorites.sort();
    }
    let list = settings.favorites.clone();
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "favorites", error = %e, "Favorites save failed");
            return Err(e);
        }
    Ok(list)
}

#[tauri::command]
pub(crate) async fn remove_favorite(
    symbol: String,
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<String>, TitanError> {
    let sym = symbol.trim().to_uppercase();
    let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    settings.favorites.retain(|s| s != &sym);
    let list = settings.favorites.clone();
    if let Err(e) = save_settings_to_disk(&settings_from_state(&settings)) {
            tracing::error!(component = "favorites", error = %e, "Favorites save failed");
            return Err(e);
        }
    Ok(list)
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct WatchlistEntry {
    pub symbol: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub rvol: f64,
    pub gap_percent: f64,
    pub score: f64,
    pub float_category: String,
    pub market_cap_m: f64,
    pub insider_signal: String,
}

#[tauri::command]
pub(crate) async fn get_watchlist_snapshots(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<WatchlistEntry>, TitanError> {
    let (key, secret, symbols, fmp_key) = {
        let keys = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
        let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        (
            keys.key.clone(),
            keys.secret.clone(),
            settings.favorites.clone(),
            keys.fmp_api_key.clone(),
        )
    };
    if key.is_empty() {
        return Err(TitanError::Validation("API KEYS MISSING".into()));
    }
    if symbols.is_empty() {
        return Ok(Vec::new());
    }

    let snaps = scanner::fetch_snapshots_batch(&key, &secret, &symbols).await;

    // FMP enrichment for market cap + insider
    let (_, profile_map) = if !fmp_key.is_empty() {
        scanner::fetch_profile_and_float_batch(&fmp_key, &symbols).await
    } else {
        (std::collections::HashMap::new(), std::collections::HashMap::new())
    };
    let insider_map = if !fmp_key.is_empty() {
        scanner::fetch_insider_batch(&fmp_key, &symbols).await
    } else {
        std::collections::HashMap::new()
    };

    let mut entries: Vec<WatchlistEntry> = Vec::new();

    for sym in &symbols {
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
            let mut score: f64 = 0.0;
            score += change_pct.abs().min(20.0) * 2.0;
            if rvol > 2.0 {
                score += (rvol - 1.0).min(10.0) * 5.0;
            }
            if gap_pct.abs() > 1.0 {
                score += gap_pct.abs().min(10.0) * 3.0;
            }
            if volume > 5_000_000.0 {
                score += 10.0;
            } else if volume > 2_000_000.0 {
                score += 5.0;
            }
            let float_cat = if volume > 10_000_000.0 {
                "HIGH"
            } else if volume > 3_000_000.0 {
                "MED"
            } else {
                "LOW"
            };
            let cap_m = profile_map.get(sym).map(|p| p.market_cap_m).unwrap_or(0.0);
            let ins = insider_map.get(sym).map(|i| i.signal.clone()).unwrap_or_default();
            entries.push(WatchlistEntry {
                symbol: sym.clone(),
                price: close,
                change_percent: change_pct,
                volume,
                rvol,
                gap_percent: gap_pct,
                score,
                float_category: float_cat.to_string(),
                market_cap_m: cap_m,
                insider_signal: ins,
            });
        } else {
            entries.push(WatchlistEntry {
                symbol: sym.clone(),
                price: 0.0,
                change_percent: 0.0,
                volume: 0.0,
                rvol: 0.0,
                gap_percent: 0.0,
                score: 0.0,
                float_category: String::new(),
                market_cap_m: 0.0,
                insider_signal: String::new(),
            });
        }
    }
    Ok(entries)
}

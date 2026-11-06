use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::error::TitanError;
use crate::journal;
use crate::state::*;

#[derive(Serialize, Clone, Debug)]
pub(crate) struct FillQualityStats {
    pub symbol: String,
    pub avg_slippage_bps: f64,
    pub fill_rate_pct: f64,
    pub avg_duration_secs: f64,
    pub trade_count: usize,
}

#[tauri::command]
pub(crate) async fn get_journal_trades(
    state: State<'_, Arc<TitanState>>,
    symbol: Option<String>,
    side: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    min_pnl: Option<f64>,
) -> Result<Vec<journal::TradeRecord>, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(j.get_trades_filtered(
        symbol.as_deref(),
        side.as_deref(),
        date_from.as_deref(),
        date_to.as_deref(),
        min_pnl,
    ))
}

#[tauri::command]
pub(crate) async fn get_journal_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<journal::JournalStats, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(j.get_stats())
}

#[tauri::command]
pub(crate) async fn add_journal_note(
    state: State<'_, Arc<TitanState>>,
    trade_id: u64,
    note: String,
) -> Result<bool, TitanError> {
    let mut j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(j.add_note(trade_id, &note))
}

#[tauri::command]
pub(crate) async fn export_journal_csv(
    state: State<'_, Arc<TitanState>>,
) -> Result<String, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let csv = j.export_csv();
    let path = crate::paths::journal_export_path();
    std::fs::write(&path, &csv)?;
    Ok(format!(
        "Exported {} trades to {}",
        j.get_stats().total_trades,
        path.display()
    ))
}

#[tauri::command]
pub(crate) async fn get_journal_equity_curve(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<(String, f64)>, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(j.get_equity_curve())
}

#[tauri::command]
pub(crate) async fn get_journal_daily_pnl(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<(String, f64)>, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(j.get_daily_pnl())
}

#[tauri::command]
pub(crate) async fn get_fill_quality_stats(
    state: State<'_, Arc<TitanState>>,
) -> Result<Vec<FillQualityStats>, TitanError> {
    let j = state
        .trade_journal
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let trades = j.get_trades_filtered(None, None, None, None, None);

    let mut per_sym: HashMap<String, (Vec<f64>, Vec<u64>, usize)> = HashMap::new();

    for t in &trades {
        let entry = per_sym
            .entry(t.symbol.clone())
            .or_insert_with(|| (Vec::new(), Vec::new(), 0));
        entry.2 += 1;

        if let Some(ref ef) = t.entry_features {
            entry.0.push(ef.spread_bps);
        }
        entry.1.push(t.duration_secs);
    }

    let mut result: Vec<FillQualityStats> = per_sym
        .into_iter()
        .map(|(sym, (slips, durations, count))| {
            let avg_slip = if slips.is_empty() {
                0.0
            } else {
                slips.iter().sum::<f64>() / slips.len() as f64
            };
            let avg_dur = if durations.is_empty() {
                0.0
            } else {
                durations.iter().sum::<u64>() as f64 / durations.len() as f64
            };
            FillQualityStats {
                symbol: sym,
                avg_slippage_bps: (avg_slip * 100.0).round() / 100.0,
                fill_rate_pct: 100.0,
                avg_duration_secs: avg_dur,
                trade_count: count,
            }
        })
        .collect();
    result.sort_by(|a, b| b.trade_count.cmp(&a.trade_count));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_quality_stats_serializes() {
        let stats = FillQualityStats {
            symbol: "AAPL".into(),
            avg_slippage_bps: 2.5,
            fill_rate_pct: 100.0,
            avg_duration_secs: 120.0,
            trade_count: 5,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("AAPL"));
        assert!(json.contains("2.5"));
    }
}

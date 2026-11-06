use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::state::TitanState;

/// Periodically flush the trade journal if dirty (every 30s).
/// Complements the debounce in record_trade/add_note by ensuring
/// dirty journals are saved even without new mutations.
pub(crate) async fn journal_flush_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("journal_flush", 60_000);
        if let Ok(mut journal) = state.trade_journal.lock() {
            if journal.is_dirty() {
                journal.flush();
                let _ = app.emit_all("journal:saved", ());
            }
        }
    }
}

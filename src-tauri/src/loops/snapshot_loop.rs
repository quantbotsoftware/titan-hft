use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::state::{save_state_snapshot, TitanState, LAST_SNAPSHOT_SAVE_TS};

pub(crate) async fn state_snapshot_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        let ts_before = LAST_SNAPSHOT_SAVE_TS.load(std::sync::atomic::Ordering::Relaxed);
        save_state_snapshot(&state);
        let ts_after = LAST_SNAPSHOT_SAVE_TS.load(std::sync::atomic::Ordering::Relaxed);
        if ts_after > ts_before {
            let _ = app.emit_all("snapshot:saved", ());
        }
    }
}

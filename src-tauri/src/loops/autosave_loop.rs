use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::state::{save_settings_to_disk, settings_from_state, TitanState};

pub(crate) async fn settings_autosave_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        let s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        match save_settings_to_disk(&settings_from_state(&s)) {
            Ok(()) => { let _ = app.emit_all("settings:saved", ()); }
            Err(e) => {
                tracing::error!(component = "autosave", error = %e, "Periodic settings save failed");
                let _ = app.emit_all("settings:save-failed", ());
            }
        }
    }
}

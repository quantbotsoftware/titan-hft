use std::path::PathBuf;
use std::sync::OnceLock;

static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn base_dir() -> &'static PathBuf {
    BASE_DIR.get_or_init(|| {
        if let Ok(val) = std::env::var("TITAN_DATA_DIR") {
            return PathBuf::from(val);
        }
        std::env::current_exe()
            .ok()
            .and_then(|exe| {
                let parent = exe.parent()?;
                // In dev the binary lives in target/release; walk up to project root
                if parent.ends_with("target/release") || parent.ends_with("target/debug") {
                    parent.parent()?.parent().map(|p| p.to_path_buf())
                } else {
                    // Installed to /usr/bin -- fall back to home-based data dir
                    dirs_fallback()
                }
            })
            .unwrap_or_else(|| PathBuf::from("."))
    })
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join("HFT_Bot").join("titan_hft_rust"))
}

pub fn data_dir() -> PathBuf {
    let d = base_dir().join("data");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn logs_dir() -> PathBuf {
    let d = base_dir().join("logs");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn backtests_dir() -> PathBuf {
    let d = base_dir().join("backtests");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn reports_dir() -> PathBuf {
    let d = base_dir().join("reports");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn journal_dir() -> PathBuf {
    let d = base_dir().join("journal");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn wallet_path() -> PathBuf {
    data_dir().join("titan_wallet.json")
}
pub fn settings_path() -> PathBuf {
    data_dir().join("titan_settings.json")
}
pub fn profiles_path() -> PathBuf {
    data_dir().join("titan_profiles.json")
}
pub fn blacklist_path() -> PathBuf {
    data_dir().join("titan_position_blacklist.json")
}
pub fn snapshot_path() -> PathBuf {
    data_dir().join("titan_state_snapshot.json")
}
pub fn keys_path() -> PathBuf {
    data_dir().join("titan_keys.json")
}
pub fn asset_cache_path() -> PathBuf {
    data_dir().join("titan_assets_cache.json")
}
pub fn config_path() -> PathBuf {
    base_dir().join("config.toml")
}
pub fn journal_file() -> PathBuf {
    journal_dir().join("trades.json")
}
pub fn journal_export_path() -> PathBuf {
    journal_dir().join("trades_export.csv")
}
pub fn log_probe_path() -> PathBuf {
    logs_dir().join(".probe")
}

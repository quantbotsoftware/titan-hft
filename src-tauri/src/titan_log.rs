use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init_logging() {
    let log_dir = crate::paths::logs_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = rolling::daily(&log_dir, "titan.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let _ = LOG_GUARD.set(guard);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,titan_hft_rust=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_writer(non_blocking),
        )
        .with(
            fmt::layer()
                .with_target(false)
                .compact()
                .with_writer(std::io::stdout),
        )
        .init();

    tracing::info!(
        component = "startup",
        "Titan logging initialized — dir: {}",
        log_dir.display()
    );
}

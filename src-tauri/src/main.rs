#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use tauri::Manager;
use tokio::time::Duration;

mod app_state;
mod commands;
mod error;
mod loops;
mod paths;
mod state;
mod types;

mod candle_buffer;
mod correlation;
mod genetic;
mod indicator_cache;
#[cfg(feature = "gpu")]
mod gpu;
mod indicators;
mod journal;
mod l2_heatmap;
mod llm_sentiment;
mod market_state;
mod monitor;
#[cfg(feature = "options")]
mod options;
mod risk;
mod risk_state;
mod scan_state;
mod scanner;
mod sentiment;
mod smarttrend;
mod strategy;
mod streamer;
#[cfg(feature = "telegram")]
mod telegram;
mod titan_log;
mod titan_oracle;
mod trading_engine;
mod trading_state;
mod watchdog;
#[cfg(feature = "web-dashboard")]
mod web_dashboard;

use commands::data::*;
use state::*;

fn main() {
    if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    titan_log::init_logging();
    tracing::info!(component = "startup", "TITAN ENGINE V2 — starting up");
    let loaded_keys = load_keys_from_disk();

    let redis_client = redis::Client::open("redis://127.0.0.1:6379/")
        .map_err(|e| println!("❌ Redis Init Error: {}", e))
        .ok();

    let preferred_iface =
        std::env::var("TITAN_NET_IFACE").unwrap_or_else(|_| DEFAULT_NET_IFACE.to_string());
    let net_iface = monitor::resolve_iface(&preferred_iface);

    let startup_settings = load_settings_from_disk();
    let startup_wallet = load_wallet_from_disk();

    let state = Arc::new(TitanState {
        // ── Sub-state aggregates ──
        trading: Arc::new(trading_state::TradingState::new(load_blacklist())),
        market: Arc::new(market_state::MarketDataState::new()),
        risk: Arc::new(risk_state::RiskState::new(
            startup_wallet.bot_cash,
            rust_decimal::Decimal::try_from(startup_settings.breaker_caution_pct).unwrap_or(rust_decimal_macros::dec!(-3.0)),
            rust_decimal::Decimal::try_from(startup_settings.breaker_halt_pct).unwrap_or(rust_decimal_macros::dec!(-5.0)),
            rust_decimal::Decimal::try_from(startup_settings.breaker_panic_pct).unwrap_or(rust_decimal_macros::dec!(-8.0)),
            startup_settings.flash_crash_window_secs,
            startup_settings.flash_crash_threshold_pct,
            startup_settings.flash_crash_cooldown_mins,
        )),
        scan: Arc::new(scan_state::ScanState::new()),

        // ── Global singletons ──
        api_keys: std::sync::Mutex::new(loaded_keys),
        wallet: std::sync::Mutex::new(startup_wallet.clone()),
        settings: {
            let mut s = app_state::AppState::default();
            apply_settings(&mut s, &startup_settings);
            std::sync::Mutex::new(s)
        },
        redis: std::sync::Mutex::new(redis_client),
        system: std::sync::Mutex::new(sysinfo::System::new()),
        net_tracker: std::sync::Mutex::new(monitor::NetTracker::new(net_iface)),
        gpu_cache: std::sync::Mutex::new(monitor::GpuCache::new(std::time::Duration::from_secs(2))),
        gpu_score_cache: std::sync::Mutex::new(GpuScoreCache {
            momentum: 0.0,
            ema_delta: 0.0,
            updated: std::time::Instant::now(),
        }),
        scan_tx: std::sync::Mutex::new(None),
        stream_tx: std::sync::Mutex::new(None),

        // ── Strategy / Nexus ──
        strategy_mode: std::sync::Mutex::new(
            StrategyMode::from_name(&startup_settings.strategy_mode_name),
        ),
        nexus_fusion_alpha: std::sync::Mutex::new(startup_settings.nexus_fusion_alpha),
        nexus_consensus: std::sync::Mutex::new(NexusConsensus::default()),

        // ── Evolution / Oracle ──
        genetic_optimizer: std::sync::Mutex::new(genetic::GeneticOptimizer::with_defaults()),
        st_genetic_optimizer: std::sync::Mutex::new(genetic::StGeneticOptimizer::with_defaults()),
        oracle_adaptation: std::sync::Mutex::new(crate::titan_oracle::OracleAdaptation::default()),

        // ── SmartTrend AI v3 (S07b + S10) ──
        smarttrend_engine: std::sync::Mutex::new(
            smarttrend::SmartTrendEngine::new(startup_settings.smarttrend.clone()),
        ),
        st_account: std::sync::Mutex::new(
            smarttrend::SmartTrendAccount::load_from_disk(),
        ),
        sentinel_snapshots: std::sync::Mutex::new(std::collections::HashMap::new()),

        // ── Journals / Caches ──
        trade_journal: std::sync::Mutex::new(journal::TradeJournal::load()),
        news_cache: std::sync::Mutex::new(Vec::new()),
        sentiment_history: std::sync::Mutex::new(Vec::new()),

        // ── Primitives ──
        shutdown_token: tokio_util::sync::CancellationToken::new(),
        dry_run: std::sync::atomic::AtomicBool::new({
            let env_dr = std::env::var("TITAN_DRY_RUN").unwrap_or_default() == "1";
            env_dr || startup_settings.dry_run_enabled
        }),
        session_generation: std::sync::atomic::AtomicU64::new(1),
        generation_mismatch_count: std::sync::atomic::AtomicU64::new(0),
        watchdog: {
            let wd = watchdog::Watchdog::new();
            wd.register("autopilot");
            wd.register("nexus");
            wd.register("volatility");
            wd.register("equity_sync");
            wd.register("position_recon");
            wd.register("trade_updates");
            wd.register("ai_scanner");
            wd.register("daily_reset");
            wd.register("genetic_evo");
            wd.register("catalyst_watchdog");
            wd.register("streamer");
            wd.register("news");
            wd.register("smarttrend_sentinel");
            wd.register("smarttrend_bot");
            #[cfg(feature = "telegram")]
            wd.register("telegram");
            wd
        },
        alpaca_rate_limiter: Arc::new(tokio::sync::Semaphore::new(10)),
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client"),
        latency_metrics: crate::state::LatencyMetrics::new(),
        mode_audit_trail: std::sync::Mutex::new(Vec::new()),
        ws_broadcast: tokio::sync::broadcast::channel(256).0,
    });

    {
        static PANIC_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let panic_state = state.clone();
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let count = PANIC_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");
            let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Box<dyn Any>".to_string()
            };

            tracing::error!(
                component = "panic",
                thread = thread_name,
                location = location.as_deref().unwrap_or("unknown"),
                panic_count = count,
                payload = %payload,
                "PANIC #{} on thread '{}' — emergency state flush",
                count,
                thread_name
            );

            if let Ok(journal) = panic_state.trade_journal.lock() {
                let path = crate::paths::journal_file();
                if let Ok(json) = serde_json::to_string_pretty(&journal.get_trades(999999, 0)) {
                    let tmp = path.with_extension("json.tmp");
                    if std::fs::write(&tmp, &json).is_ok() {
                        let _ = std::fs::rename(&tmp, &path);
                    }
                }
            }
            if let Ok(wallet) = panic_state.wallet.lock() {
                let _ = save_wallet_to_disk(&wallet);
            }
            save_state_snapshot(&panic_state);
            default_hook(info);
        }));
    }

    let restored_snapshot = restore_state_snapshot(&state);

    let (stream_tx, stream_rx) = tokio::sync::mpsc::channel::<streamer::StreamCmd>(256);
    {
        let mut tx_guard = state.stream_tx.lock().unwrap_or_else(|e| e.into_inner());
        *tx_guard = Some(stream_tx);
    }

    let stream_state = state.clone();
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::account::connect_api,
            commands::account::init_api_connection,
            commands::account::update_api_keys,
            commands::account::get_app_state,
            commands::account::get_wallet_snapshot,
            commands::account::topup_user,
            commands::account::topup_bot,
            commands::account::withdraw_bot,
            commands::account::clear_cache,
            commands::account::get_account_snapshot,
            commands::settings::update_settings,
            commands::settings::batch_update_settings,
            commands::settings::get_settings_snapshot,
            commands::settings::validate_thresholds,
            commands::settings::save_undo_snapshot,
            commands::settings::load_undo_snapshot,
            commands::settings::save_chart_settings,
            commands::settings::load_chart_settings,
            commands::settings::save_chart_undo,
            commands::settings::load_chart_undo,
            commands::settings::set_symbol_override,
            commands::settings::remove_symbol_override,
            commands::settings::get_symbol_overrides,
            commands::settings::preview_effective_exits,
            commands::settings::save_profile,
            commands::settings::load_profile,
            commands::settings::delete_profile,
            commands::settings::list_profiles,
            commands::settings::export_profile,
            commands::settings::import_profile,
            commands::settings::apply_full_settings,
            commands::settings::set_active_symbol,
            commands::settings::set_position_mode,
            commands::settings::get_position_modes,
            commands::settings::set_dry_run,
            commands::trading::submit_order,
            commands::trading::submit_manual_order,
            commands::trading::liquidate_all_positions,
            commands::trading::force_close_position,
            commands::trading::cancel_all_orders,
            commands::trading::emergency_liquidate,
            commands::trading::calc_position_size,
            commands::backtest::optimize_timeframe,
            commands::backtest::run_backtest,
            commands::backtest::run_backtest_oracle,
            commands::backtest::run_backtest_sweep,
            commands::backtest::run_backtest_sweep_multi,
            commands::backtest::run_signal_backtest,
            commands::backtest::run_signal_backtest_multi,
            commands::backtest::export_backtest_sweep_multi_csv,
            commands::backtest::export_backtest_csv,
            commands::backtest::export_backtest_json,
            commands::backtest::export_backtest_html,
            commands::backtest::export_backtest_sweep_csv,
            commands::backtest::export_backtest_report_html,
            commands::backtest::run_backtest_now,
            commands::backtest::get_strategy_score,
            commands::backtest::get_signal_markers,
            commands::backtest::gpu_bench,
            commands::backtest::deep_focus_analysis,
            commands::backtest::retrain_brain,
            commands::backtest::get_oracle_snapshot,
            commands::scanner_cmd::run_market_scan,
            commands::scanner_cmd::run_ai_scan,
            commands::scanner_cmd::activate_scan_targets,
            commands::scanner_cmd::cancel_ai_scan,
            commands::hft_targets::get_hft_targets,
            commands::hft_targets::add_hft_target,
            commands::hft_targets::remove_hft_target,
            commands::hft_targets::clear_all_hft_targets,
            commands::hft_targets::get_hft_target_enriched,
            commands::hft_targets::get_watchlist_enriched,
            commands::signal::get_signal_assets,
            commands::signal::add_signal_asset,
            commands::signal::remove_signal_asset,
            commands::signal::clear_signal_assets,
            commands::signal::inject_to_signal_assets,
            commands::signal::inject_signal_to_hft,
            commands::signal::get_signal_asset_enriched,
            commands::favorites::get_favorites,
            commands::favorites::add_favorite,
            commands::favorites::remove_favorite,
            commands::favorites::get_watchlist_snapshots,
            commands::strategy_cmd::toggle_strategy_mode,
            commands::strategy_cmd::set_strategy_mode,
            commands::strategy_cmd::get_mode_audit_trail,
            commands::strategy_cmd::set_nexus_alpha,
            commands::strategy_cmd::get_nexus_status,
            commands::strategy_cmd::get_ofi_status,
            commands::strategy_cmd::get_correlation_matrix,
            commands::strategy_cmd::get_portfolio_performance,
            commands::strategy_cmd::rebalance_portfolio,
            commands::journal_cmd::get_journal_trades,
            commands::journal_cmd::get_journal_stats,
            commands::journal_cmd::add_journal_note,
            commands::journal_cmd::export_journal_csv,
            commands::journal_cmd::get_journal_equity_curve,
            commands::journal_cmd::get_journal_daily_pnl,
            commands::journal_cmd::get_fill_quality_stats,
            commands::monitoring::get_monitor_snapshot,
            commands::monitoring::get_risk_status,
            commands::monitoring::graceful_shutdown,
            commands::monitoring::get_telemetry,
            commands::monitoring::reset_circuit_breaker,
            commands::monitoring::reset_daily_analytics,
            commands::monitoring::reset_full_safety,
            commands::monitoring::get_evolution_stats,
            commands::monitoring::get_active_trading_params,
            commands::monitoring::freeze_evolution,
            commands::monitoring::resume_evolution,
            commands::monitoring::get_multi_status,
            commands::monitoring::get_intraday_drawdown,
            commands::monitoring::get_latency_metrics,
            commands::misc::get_build_info,
            commands::misc::get_state_health,
            commands::misc::start_scan_command,
            commands::misc::stop_scan_command,
            commands::misc::test_catalyst_alert,
            commands::misc::test_catalyst_bearish,
            commands::misc::test_discord_alert,
            commands::misc::test_telegram_alert,
            commands::misc::get_news_feed,
            commands::misc::refresh_news,
            commands::misc::check_catalyst,
            commands::misc::get_replay_symbols,
            commands::misc::get_replay_data,
            commands::misc::get_option_chain,
            commands::misc::hedge_position,
            commands::misc::get_hedge_status,
            commands::data::fetch_alpaca_history,
            commands::market_data::get_cvd,
            commands::market_data::get_microstructure,
            commands::market_data::get_smarttrend_diagnostics,
            commands::smarttrend_cmd::smarttrend_kill_switch,
            commands::smarttrend_cmd::get_smarttrend_config,
            commands::smarttrend_cmd::update_smarttrend_config,
            commands::smarttrend_cmd::get_smarttrend_session_stats,
            commands::smarttrend_cmd::get_smarttrend_state,
            commands::smarttrend_cmd::get_sentinel_status,
            commands::smarttrend_cmd::smarttrend_topup,
            commands::smarttrend_cmd::smarttrend_withdraw,
            commands::smarttrend_cmd::smarttrend_pause,
            commands::smarttrend_cmd::smarttrend_account_stats,
            commands::smarttrend_cmd::smarttrend_force_exit_all,
            commands::smarttrend_cmd::smarttrend_get_asset_modes,
            commands::smarttrend_cmd::smarttrend_set_asset_mode,
            commands::smarttrend_cmd::smarttrend_set_all_modes,
            commands::smarttrend_cmd::smarttrend_get_trade_panel_data,
            commands::smarttrend_cmd::smarttrend_export_config,
            commands::smarttrend_cmd::smarttrend_import_config,
            commands::smarttrend_cmd::get_st_evolution_stats,
            commands::smarttrend_cmd::get_st_fitness_history,
            commands::smarttrend_cmd::get_st_champion_params,
            commands::smarttrend_cmd::run_comparison_backtest_cmd,
            commands::smarttrend_cmd::freeze_st_evolution,
            commands::smarttrend_cmd::resume_st_evolution,
            web_dashboard::rotate_dashboard_token,
        ])
        .setup(move |app| {
            // Replenish rate-limit permits: ~3/sec (180/min, safely under Alpaca's 200/min)
            {
                let limiter = stream_state.alpaca_rate_limiter.clone();
                let shutdown = stream_state.shutdown_token.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown.cancelled() => break,
                            _ = tokio::time::sleep(Duration::from_millis(334)) => {
                                if limiter.available_permits() < 10 {
                                    limiter.add_permits(1);
                                }
                            }
                        }
                    }
                });
            }

            let health_state = stream_state.clone();
            let health_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                startup_health_check(health_state, health_handle).await;
            });

            {
                let preload_state = stream_state.clone();
                tauri::async_runtime::spawn(async move {
                    let (key, secret) = {
                        let keys = preload_state
                            .api_keys
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        (keys.key.clone(), keys.secret.clone())
                    };
                    if key.is_empty() || secret.is_empty() {
                        return;
                    }
                    match scanner::fetch_active_assets(&key, &secret).await {
                        Ok(assets) => tracing::info!(
                            component = "startup",
                            "Asset cache warmed on startup ({} assets)",
                            assets.len()
                        ),
                        Err(e) => tracing::warn!(
                            component = "startup",
                            "Startup asset warmup failed: {}",
                            e
                        ),
                    }
                });
            }

            if let Some(snap) = restored_snapshot {
                let scan_results = snap.last_scan_results.clone();
                if !scan_results.is_empty() {
                    let _ = app.emit_all("ai-scan-update", &scan_results);
                    let _ = app.emit_all(
                        "log-message",
                        format!(
                            "SNAPSHOT RESTORE: {} scan results loaded from previous session",
                            scan_results.len()
                        ),
                    );
                }
                if snap.scan_was_running && !snap.scan_prefilter_queue.is_empty() {
                    let resume_state = stream_state.clone();
                    let resume_handle = app.handle();
                    let saved_queue = snap.scan_prefilter_queue.clone();
                    let partial_results = snap.last_scan_results.clone();
                    let done = partial_results.len();
                    let queue_len = saved_queue.len();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        let _ = resume_handle.emit_all(
                            "log-message",
                            format!(
                                "AUTO-RESUME: Continuing interrupted scan ({}/{} done)...",
                                done, queue_len
                            ),
                        );
                        let _ = auto_resume_scan(
                            resume_state,
                            resume_handle,
                            saved_queue,
                            partial_results,
                        )
                        .await;
                    });
                }
                let trails = snap.trail_highs.len();
                let prices = snap.last_prices.len();
                let _ = app.emit_all(
                    "log-message",
                    format!(
                        "SNAPSHOT RESTORE: {} trail stops, {} cached prices, analytics restored",
                        trails, prices
                    ),
                );
            }

            // All initial spawns register AbortHandles via abort_and_replace so
            // the watchdog can cleanly abort them on first respawn (prevents zombie
            // dual-instances where the original task keeps running alongside the
            // respawned one).
            macro_rules! spawn_with_handle {
                ($wd:expr, $name:expr, $body:expr) => {{
                    let jh = tauri::async_runtime::spawn($body);
                    $wd.abort_and_replace($name, jh.inner().abort_handle());
                }};
            }

            let handle = app.handle();
            let stream_state_for_streamer = stream_state.clone();
            spawn_with_handle!(stream_state.watchdog, "streamer", async move {
                streamer::run_streamer(stream_state_for_streamer, stream_rx, handle).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "autopilot", async move {
                loops::autopilot::autopilot_loop(s, h).await;
            });

            let s = stream_state.clone();
            spawn_with_handle!(stream_state.watchdog, "volatility", async move {
                loops::volatility::update_volatility_loop(s).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "nexus", async move {
                loops::nexus_loop::nexus_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "daily_reset", async move {
                loops::daily_reset::daily_reset_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "genetic_evo", async move {
                loops::genetic_loop::genetic_evolution_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "trade_updates", async move {
                loops::trade_updates::trade_updates_stream(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "position_recon", async move {
                loops::position_recon::position_reconciliation_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "equity_sync", async move {
                loops::equity_sync::equity_sync_loop(s, h).await;
            });

            // L2 orderbook — now handled inside streamer (single WebSocket connection)

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "news", async move {
                loops::news_loop::news_poll_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "catalyst_watchdog", async move {
                sentiment::catalyst_watchdog_loop(s, h).await;
            });

            #[cfg(feature = "telegram")]
            {
                let s = stream_state.clone();
                spawn_with_handle!(stream_state.watchdog, "telegram", async move {
                    telegram::telegram_command_loop(s).await;
                });
            }

            #[cfg(feature = "web-dashboard")]
            {
                let web_state = stream_state.clone();
                let (web_enabled, web_port) = {
                    let s = web_state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    (s.web_dashboard_enabled, s.web_dashboard_port)
                };
                if web_enabled {
                    tauri::async_runtime::spawn(async move {
                        web_dashboard::start_web_dashboard(web_state, web_port).await;
                    });
                }
            }

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "watchdog_supervisor", async move {
                loops::watchdog_loop::watchdog_supervisor(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "snapshot", async move {
                loops::snapshot_loop::state_snapshot_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "autosave", async move {
                loops::autosave_loop::settings_autosave_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "journal_flush", async move {
                loops::journal_flush_loop::journal_flush_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "ai_scanner", async move {
                loops::ai_scan_loop::ai_scan_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "smarttrend_sentinel", async move {
                loops::smarttrend_sentinel::smarttrend_sentinel_loop(s, h).await;
            });

            let s = stream_state.clone();
            let h = app.handle();
            spawn_with_handle!(stream_state.watchdog, "smarttrend_bot", async move {
                loops::smarttrend_bot::smarttrend_bot_loop(s, h).await;
            });

            #[cfg(feature = "gpu")]
            {
                let gpu_state = stream_state.clone();
                tauri::async_runtime::spawn(async move {
                    let (gpu_backtest, gpu_sweep, gpu_deep_focus) = {
                        let settings = gpu_state.settings.lock().unwrap_or_else(|e| e.into_inner());
                        (
                            settings.gpu_backtest,
                            settings.gpu_sweep,
                            settings.gpu_deep_focus,
                        )
                    };
                    let should_warm = setting_or_env(gpu_backtest, "TITAN_GPU_BACKTEST")
                        || setting_or_env(gpu_sweep, "TITAN_GPU_SWEEP")
                        || setting_or_env(gpu_deep_focus, "TITAN_GPU_DEEP_FOCUS");
                    if should_warm {
                        let _ = gpu::gpu_warmup().await;
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}

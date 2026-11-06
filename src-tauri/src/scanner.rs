use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::error::TitanError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaAsset {
    pub symbol: String,
    #[allow(dead_code)]
    pub status: String,
    pub tradable: bool,
    #[allow(dead_code)]
    pub shortable: bool,
    #[allow(dead_code)]
    pub fractionable: bool,
    #[serde(default)]
    pub exchange: String,
}

pub fn is_us_exchange(exchange: &str) -> bool {
    matches!(
        exchange.to_uppercase().as_str(),
        "NYSE" | "NASDAQ" | "AMEX" | "ARCA" | "BATS" | "NYSEARCA" | "IEX"
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RvolMethod {
    #[default]
    TimeAdjusted,
    Naive,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotDailyBar {
    pub c: f64,
    pub o: f64,
    pub v: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SnapshotTrade {
    pub p: f64,
    pub s: f64,
    pub t: String,
    pub c: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotData {
    #[serde(rename = "dailyBar")]
    pub daily_bar: SnapshotDailyBar,
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<SnapshotDailyBar>,
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<SnapshotTrade>,
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<SnapshotDailyBar>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanResult {
    pub symbol: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub rvol: f64,
    pub gap_percent: f64,
    pub score: f64,
    pub strategy: String,
    #[serde(default)]
    pub float_shares: f64,
    /// Market cap in millions (0.0 if unavailable)
    #[serde(default)]
    pub market_cap: f64,
    #[serde(default)]
    pub sector: String,
    /// "BUY", "SELL", or "" (none)
    #[serde(default)]
    pub insider_signal: String,
    #[serde(default)]
    pub insider_detail: String,
    #[serde(default)]
    pub has_earnings_soon: bool,
    /// Analyst upside % from consensus target (0.0 if unavailable)
    #[serde(default)]
    pub analyst_upside_pct: f64,
    /// Dollar volume (price * shares traded)
    #[serde(default)]
    pub dollar_volume: f64,
    #[serde(default)]
    pub is_stale: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiScanCandidate {
    pub rank: usize,
    pub symbol: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub rvol: f64,
    pub gap_percent: f64,
    pub multi_day_strength: f64,
    pub float_category: String,
    pub prefilter_score: f64,
    pub oracle_score: f64,
    pub oracle_confidence: f64,
    pub oracle_regime: String,
    pub nexus_consensus: String,
    pub nexus_buy_pct: f64,
    pub fusion_score: f64,
    pub fusion_signal: String,
    pub bt_best_tf: String,
    pub bt_trades: usize,
    pub bt_win_rate: f64,
    pub bt_return_pct: f64,
    pub bt_sharpe: f64,
    pub bt_profit_factor: f64,
    pub bt_max_drawdown: f64,
    pub bt_expectancy: f64,
    pub ai_score: f64,
    pub smart_score: f64,
    #[serde(default)]
    pub has_catalyst: bool,
    #[serde(default)]
    pub catalyst_headline: String,
    #[serde(default)]
    pub catalyst_url: String,
    #[serde(default)]
    pub reasoning: String,
    /// Market cap in millions (0.0 if unavailable)
    #[serde(default)]
    pub market_cap: f64,
    /// "BUY", "SELL", or "" (none)
    #[serde(default)]
    pub insider_signal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreFilterCandidate {
    pub symbol: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub prev_volume: f64,
    pub rvol: f64,
    pub gap_percent: f64,
    pub prefilter_score: f64,
    #[serde(default)]
    pub dollar_volume: f64,
    #[serde(default)]
    pub is_stale: bool,
}

// --- ASSET & FMP CACHES ---
type AssetCacheType = Arc<Mutex<Option<(std::time::Instant, Vec<AlpacaAsset>)>>>;
type EarningsCacheType = Arc<Mutex<Option<(std::time::Instant, HashMap<String, bool>)>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAssetCache {
    saved_at_unix_secs: u64,
    assets: Vec<AlpacaAsset>,
}

lazy_static::lazy_static! {
    static ref ASSET_CACHE: AssetCacheType = Arc::new(Mutex::new(None));
    static ref FLOAT_CACHE: Arc<Mutex<HashMap<String, (f64, std::time::Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref PROFILE_CACHE: Arc<Mutex<HashMap<String, (FmpProfileData, std::time::Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref INSIDER_CACHE: Arc<Mutex<HashMap<String, (InsiderSignal, std::time::Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref EARNINGS_CACHE: EarningsCacheType = Arc::new(Mutex::new(None));
    static ref TARGETS_CACHE: Arc<Mutex<HashMap<String, (f64, std::time::Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
}
const ASSET_CACHE_TTL: Duration = Duration::from_secs(600);
const ASSET_STALE_FALLBACK_TTL: Duration = Duration::from_secs(86400);
const ASSET_FETCH_ATTEMPTS: usize = 2;
const FLOAT_CACHE_TTL: Duration = Duration::from_secs(86400);
const PROFILE_CACHE_TTL: Duration = Duration::from_secs(86400);
const INSIDER_CACHE_TTL: Duration = Duration::from_secs(86400);
const EARNINGS_CACHE_TTL: Duration = Duration::from_secs(43200); // 12h
const TARGETS_CACHE_TTL: Duration = Duration::from_secs(604800); // 7 days

/// Unified profile entry from /stable/profile — gives us marketCap, sector,
/// AND lets us estimate float_shares (≈ shares outstanding) from marketCap/price.
#[derive(Debug, Clone, Deserialize)]
struct FmpProfileEntry {
    #[serde(default)]
    #[allow(dead_code)]
    symbol: String,
    #[serde(rename = "marketCap", default)]
    market_cap: f64,
    #[serde(default)]
    price: f64,
    #[serde(default)]
    sector: String,
    #[serde(default)]
    #[allow(dead_code)]
    industry: String,
}

#[derive(Debug, Clone)]
pub struct FmpProfileData {
    pub market_cap_m: f64,
    pub sector: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FmpInsiderEntry {
    #[serde(alias = "transactionType", default)]
    transaction_type: String,
    #[serde(alias = "securitiesTransacted", default)]
    securities_transacted: f64,
    #[serde(default)]
    price: f64,
    #[serde(alias = "reportingName", default)]
    reporting_name: String,
}

#[derive(Debug, Clone)]
pub struct InsiderSignal {
    pub signal: String,  // "BUY", "SELL", or ""
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FmpEarningEntry {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    #[allow(dead_code)]
    date: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FmpTargetEntry {
    #[serde(rename = "targetConsensus", default)]
    target_consensus: f64,
}

fn build_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new())
}

fn build_assets_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| Client::new())
}

fn classify_reqwest(e: reqwest::Error, phase: &str) -> TitanError {
    if e.is_timeout() {
        TitanError::Timeout(format!("{}: operation timed out", phase))
    } else if e.is_connect() {
        TitanError::Network(format!("{}: connection failed - {}", phase, e))
    } else {
        TitanError::Network(format!("{}: {}", phase, e))
    }
}

fn asset_cache_age(saved_at_unix_secs: u64) -> Option<Duration> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(Duration::from_secs(now.saturating_sub(saved_at_unix_secs)))
}

fn load_asset_cache_from_disk(max_age: Duration) -> Option<(Vec<AlpacaAsset>, Duration)> {
    let path = crate::paths::asset_cache_path();
    let raw = std::fs::read(&path).ok()?;
    let cached: PersistedAssetCache = serde_json::from_slice(&raw).ok()?;
    if cached.assets.is_empty() {
        return None;
    }
    let age = asset_cache_age(cached.saved_at_unix_secs)?;
    if age > max_age {
        return None;
    }
    Some((cached.assets, age))
}

fn save_asset_cache_to_disk(assets: &[AlpacaAsset]) {
    let payload = PersistedAssetCache {
        saved_at_unix_secs: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        assets: assets.to_vec(),
    };
    let path = crate::paths::asset_cache_path();
    match serde_json::to_vec(&payload) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(&path, bytes) {
                tracing::warn!(
                    component = "scanner",
                    path = %path.display(),
                    "Failed to persist asset cache: {}",
                    e
                );
            }
        }
        Err(e) => {
            tracing::warn!(component = "scanner", "Failed to serialize asset cache: {}", e);
        }
    }
}

async fn set_asset_mem_cache(assets: &[AlpacaAsset]) {
    let mut cache = ASSET_CACHE.lock().await;
    *cache = Some((std::time::Instant::now(), assets.to_vec()));
}

async fn fetch_active_assets_remote(
    client: &Client,
    api_key: &str,
    secret_key: &str,
) -> Result<Vec<AlpacaAsset>, TitanError> {
    let url = "https://paper-api.alpaca.markets/v2/assets?status=active&asset_class=us_equity";
    let resp = client
        .get(url)
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret_key)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(component = "scanner", "fetch_active_assets send failed: {}", e);
            classify_reqwest(e, "fetch_assets")
        })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!(
            component = "scanner",
            "fetch_active_assets HTTP {}: {}",
            status.as_u16(),
            body
        );
        return Err(TitanError::Broker(format!(
            "fetch_assets: Alpaca returned HTTP {} - {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        )));
    }

    let bytes = resp.bytes().await.map_err(|e| {
        tracing::warn!(component = "scanner", "fetch_active_assets body read failed: {}", e);
        classify_reqwest(e, "fetch_assets_body")
    })?;

    serde_json::from_slice(&bytes).map_err(|e| {
        tracing::warn!(
            component = "scanner",
            "fetch_active_assets JSON decode failed ({} bytes): {}",
            bytes.len(),
            e
        );
        TitanError::Internal(format!("parse_assets: invalid JSON - {}", e))
    })
}

pub async fn fetch_active_assets(
    api_key: &str,
    secret_key: &str,
) -> Result<Vec<AlpacaAsset>, TitanError> {
    let stale_mem_cache = {
        let cache = ASSET_CACHE.lock().await;
        if let Some((ts, assets)) = cache.as_ref() {
            if ts.elapsed() < ASSET_CACHE_TTL {
                tracing::debug!(
                    component = "scanner",
                    "Using cached asset list ({} assets)",
                    assets.len()
                );
                return Ok(assets.clone());
            }
            Some((assets.clone(), ts.elapsed()))
        } else {
            None
        }
    };

    if let Some((assets, age)) = load_asset_cache_from_disk(ASSET_CACHE_TTL) {
        tracing::debug!(
            component = "scanner",
            age_secs = age.as_secs(),
            "Using disk-cached asset list ({} assets)",
            assets.len()
        );
        set_asset_mem_cache(&assets).await;
        return Ok(assets);
    }

    let stale_disk_cache = load_asset_cache_from_disk(ASSET_STALE_FALLBACK_TTL);
    let stale_fallback = match (stale_mem_cache, stale_disk_cache) {
        (Some((assets, age)), Some((disk_assets, disk_age))) => {
            if age <= disk_age {
                Some((assets, age, "memory"))
            } else {
                Some((disk_assets, disk_age, "disk"))
            }
        }
        (Some((assets, age)), None) => Some((assets, age, "memory")),
        (None, Some((assets, age))) => Some((assets, age, "disk")),
        (None, None) => None,
    };

    let client = build_assets_client();
    let mut last_err: Option<TitanError> = None;
    for attempt in 1..=ASSET_FETCH_ATTEMPTS {
        match fetch_active_assets_remote(&client, api_key, secret_key).await {
            Ok(assets) => {
                tracing::info!(
                    component = "scanner",
                    attempt,
                    "Fetched {} active assets from Alpaca",
                    assets.len()
                );
                set_asset_mem_cache(&assets).await;
                save_asset_cache_to_disk(&assets);
                return Ok(assets);
            }
            Err(e) => {
                tracing::warn!(
                    component = "scanner",
                    attempt,
                    max_attempts = ASSET_FETCH_ATTEMPTS,
                    "fetch_active_assets attempt failed: {}",
                    e
                );
                last_err = Some(e);
                if attempt < ASSET_FETCH_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(1200 * attempt as u64)).await;
                }
            }
        }
    }

    if let Some((assets, age, source)) = stale_fallback {
        tracing::warn!(
            component = "scanner",
            source,
            age_secs = age.as_secs(),
            "Using stale asset cache after remote fetch failure ({} assets)",
            assets.len()
        );
        set_asset_mem_cache(&assets).await;
        return Ok(assets);
    }

    let assets = last_err
        .ok_or_else(|| TitanError::Internal("fetch_assets: unknown failure".into()))?;

    Err(assets)
}

pub async fn fetch_snapshots_batch(
    api_key: &str,
    secret_key: &str,
    symbols: &[String],
) -> HashMap<String, SnapshotData> {
    let client = build_client();
    let mut all: HashMap<String, SnapshotData> = HashMap::new();
    let total_chunks = symbols.len().div_ceil(1000);

    for (i, chunk) in symbols.chunks(1000).enumerate() {
        if i > 0 {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        let sym_list = chunk.join(",");
        let url = format!(
            "https://data.alpaca.markets/v2/stocks/snapshots?symbols={}&feed=sip",
            sym_list
        );
        match client
            .get(&url)
            .header("APCA-API-KEY-ID", api_key)
            .header("APCA-API-SECRET-KEY", secret_key)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<HashMap<String, SnapshotData>>().await {
                Ok(snaps) => {
                    tracing::debug!(
                        component = "scanner",
                        "Snapshot chunk {}/{}: {} symbols OK",
                        i + 1,
                        total_chunks,
                        snaps.len()
                    );
                    all.extend(snaps);
                }
                Err(e) => {
                    tracing::warn!(
                        component = "scanner",
                        "Snapshot chunk {}/{} parse error: {}",
                        i + 1,
                        total_chunks,
                        e
                    );
                }
            },
            Err(e) => {
                tracing::warn!(
                    component = "scanner",
                    "Snapshot chunk {}/{} fetch error: {}",
                    i + 1,
                    total_chunks,
                    e
                );
            }
        }
    }

    tracing::info!(
        component = "scanner",
        "Fetched {} snapshots from {} symbols ({} chunks)",
        all.len(),
        symbols.len(),
        total_chunks
    );
    all
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanParams {
    pub price_min: f64,
    pub price_max: f64,
    pub volume_min: f64,
    pub change_min: f64,
    pub rvol_min: f64,
    pub gap_min: f64,
    pub require_catalyst: bool,
    pub max_candidates: usize,
    pub scan_interval_minutes: u32,
    /// If true, negative change (bearish stocks) also passes the change filter.
    /// Tied to allow_short in the UI.
    #[serde(default)]
    pub allow_negative_change: bool,
    /// Score weight preset: "balanced", "volume_hunter", "momentum", "gap_scanner"
    #[serde(default = "default_score_preset")]
    pub score_preset: String,
    /// Maximum float in millions (0 = no filter). Ross Cameron prefers < 20M.
    #[serde(default)]
    pub float_max: f64,
    /// Maximum market cap in billions (0 = no filter). Ross Cameron: < 2B.
    #[serde(default)]
    pub cap_max: f64,
    /// If true, exclude stocks with earnings in ±3 days.
    #[serde(default)]
    pub exclude_earnings: bool,
    /// Minimum dollar volume (price * shares). Filters low-notional tickers.
    #[serde(default = "default_min_dollar_volume")]
    pub min_dollar_volume: f64,
    /// Tiered scan: pre-rank by dollar volume, split into tiers (S15).
    #[serde(default = "default_true_scan")]
    pub tiered_scan_enabled: bool,
    /// Tier 1: top N by dollar volume -- always scored
    #[serde(default = "default_tier1")]
    pub tier1_count: usize,
    /// Tier 2: next N by dollar volume -- scored only if RVOL or change% passes gate
    #[serde(default = "default_tier2")]
    pub tier2_count: usize,
    #[serde(default)]
    pub rvol_method: RvolMethod,
}

fn default_min_dollar_volume() -> f64 { 5_000_000.0 }
fn default_true_scan() -> bool { true }
fn default_tier1() -> usize { 50 }
fn default_tier2() -> usize { 100 }

fn default_score_preset() -> String { "balanced".to_string() }

impl Default for ScanParams {
    fn default() -> Self {
        Self {
            price_min: 1.0,
            price_max: 500.0,
            volume_min: 500_000.0,
            change_min: 1.0,
            rvol_min: 1.0,
            gap_min: 0.0,
            require_catalyst: false,
            max_candidates: 100,
            scan_interval_minutes: 30,
            allow_negative_change: false,
            score_preset: "balanced".to_string(),
            float_max: 0.0,
            cap_max: 0.0,
            exclude_earnings: false,
            min_dollar_volume: 5_000_000.0,
            tiered_scan_enabled: true,
            tier1_count: 50,
            tier2_count: 100,
            rvol_method: RvolMethod::TimeAdjusted,
        }
    }
}

/// Returns (w_change, w_rvol, w_gap, w_volume) weights based on preset name.
pub fn score_weights_for_preset(preset: &str) -> (f64, f64, f64, f64) {
    match preset {
        "volume_hunter" => (15.0, 20.0, 10.0, 55.0),
        "momentum"      => (50.0, 20.0, 10.0, 20.0),
        "gap_scanner"   => (10.0, 15.0, 55.0, 20.0),
        _               => (25.0, 25.0, 25.0, 25.0), // balanced
    }
}

#[allow(dead_code)]
fn compute_candidate_score(
    change_pct: f64,
    rvol: f64,
    gap_pct: f64,
    volume: f64,
    prev_volume: f64,
) -> f64 {
    compute_candidate_score_weighted(change_pct, rvol, gap_pct, volume, prev_volume, 25.0, 25.0, 25.0, 25.0)
}

fn compute_candidate_score_weighted(
    change_pct: f64,
    rvol: f64,
    gap_pct: f64,
    volume: f64,
    prev_volume: f64,
    w_change: f64,
    w_rvol: f64,
    w_gap: f64,
    w_volume: f64,
) -> f64 {
    let change_score = (change_pct.abs().min(20.0) / 20.0) * w_change;
    let rvol_score = if rvol > 1.0 {
        ((rvol - 1.0).min(10.0) / 10.0) * w_rvol
    } else {
        0.0
    };
    let gap_score = (gap_pct.abs().min(10.0) / 10.0) * w_gap;
    let vol_score = if volume > 5_000_000.0 {
        w_volume
    } else if volume > 2_000_000.0 {
        w_volume * 0.72
    } else if volume > 2.0 * prev_volume && prev_volume > 100_000.0 {
        w_volume * 0.48
    } else {
        (volume / 5_000_000.0).min(1.0) * w_volume * 0.4
    };
    change_score + rvol_score + gap_score + vol_score
}

use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;

const PRE_OPEN_MIN: f64 = 4.0 * 60.0;
const REG_OPEN_MIN: f64 = 9.0 * 60.0 + 30.0;
const REG_CLOSE_MIN: f64 = 16.0 * 60.0;
const EXT_CLOSE_MIN: f64 = 20.0 * 60.0;
const PROGRESS_AT_REG_OPEN: f64 = 0.08;
const PROGRESS_AT_REG_CLOSE: f64 = 0.92;

pub(super) fn compute_session_progress(now_et: DateTime<Tz>) -> f64 {
    let wd = now_et.weekday();
    if wd == chrono::Weekday::Sat || wd == chrono::Weekday::Sun {
        return 0.0;
    }

    let t = now_et.time();
    let mins = t.hour() as f64 * 60.0 + t.minute() as f64 + t.second() as f64 / 60.0;

    if mins < PRE_OPEN_MIN { return 0.0; }
    if mins < REG_OPEN_MIN {
        let frac = (mins - PRE_OPEN_MIN) / (REG_OPEN_MIN - PRE_OPEN_MIN);
        return frac * PROGRESS_AT_REG_OPEN;
    }
    if mins < REG_CLOSE_MIN {
        let frac = (mins - REG_OPEN_MIN) / (REG_CLOSE_MIN - REG_OPEN_MIN);
        return PROGRESS_AT_REG_OPEN + frac * (PROGRESS_AT_REG_CLOSE - PROGRESS_AT_REG_OPEN);
    }
    if mins < EXT_CLOSE_MIN {
        let frac = (mins - REG_CLOSE_MIN) / (EXT_CLOSE_MIN - REG_CLOSE_MIN);
        return PROGRESS_AT_REG_CLOSE + frac * (1.0 - PROGRESS_AT_REG_CLOSE);
    }
    1.0
}

pub(crate) fn time_adjusted_rvol(vol_today: f64, prev_volume: f64, now_et: DateTime<Tz>) -> f64 {
    if !prev_volume.is_finite() || prev_volume <= 0.0 { return 1.0; }
    let progress = compute_session_progress(now_et);
    if progress <= 0.001 { return 1.0; }
    let expected = prev_volume * progress;
    if !expected.is_finite() || expected <= 0.0 { return 1.0; }
    vol_today / expected
}

fn compute_is_stale(now_et: DateTime<Tz>, last_trade_age: Duration) -> bool {
    let wd = now_et.weekday();
    if wd == chrono::Weekday::Sat || wd == chrono::Weekday::Sun { return false; }
    let t = now_et.time();
    let mins = t.hour() as f64 * 60.0 + t.minute() as f64;
    let secs = last_trade_age.as_secs();

    if (9.5 * 60.0..16.0 * 60.0).contains(&mins) {
        secs > 60 // RTH
    } else if (4.0 * 60.0..20.0 * 60.0).contains(&mins) {
        secs > 300 // Premarket / Extended
    } else {
        secs > 3600 // Dead time
    }
}

fn extract_bar_metrics(snap: &SnapshotData, now_et: DateTime<Tz>) -> (f64, f64, f64, f64, f64, f64, bool) {
    let close = if let Some(lt) = &snap.latest_trade {
        if lt.p > 0.0 { lt.p } else { snap.minute_bar.as_ref().map(|m| m.c).unwrap_or(snap.daily_bar.c) }
    } else {
        snap.minute_bar.as_ref().map(|m| m.c).unwrap_or(snap.daily_bar.c)
    };
    
    let is_stale = if let Some(lt) = &snap.latest_trade {
        if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&lt.t) {
            let age = now_et.signed_duration_since(ts.with_timezone(&chrono_tz::US::Eastern));
            if age.num_seconds() > 0 {
                compute_is_stale(now_et, Duration::from_secs(age.num_seconds() as u64))
            } else { false }
        } else { true }
    } else { true };

    let open_price = snap.daily_bar.o;
    let volume = snap.daily_bar.v;
    let prev_close = snap
        .prev_daily_bar
        .as_ref()
        .map(|p| p.c)
        .unwrap_or(open_price);
    let prev_volume = snap.prev_daily_bar.as_ref().map(|p| p.v).unwrap_or(volume);
    let change_pct = if prev_close > 0.0 {
        ((close - prev_close) / prev_close) * 100.0
    } else {
        0.0
    };
    (
        close,
        open_price,
        volume,
        prev_close,
        prev_volume,
        change_pct,
        is_stale
    )
}

pub fn quant_prefilter(
    snapshots: &HashMap<String, SnapshotData>,
    params: &ScanParams,
) -> Vec<PreFilterCandidate> {
    let (w_change, w_rvol, w_gap, w_volume) = score_weights_for_preset(&params.score_preset);
    let mut candidates: Vec<PreFilterCandidate> = Vec::new();
    let now_et = chrono::Utc::now().with_timezone(&chrono_tz::US::Eastern);

    for (sym, snap) in snapshots {
        let (close, open_price, volume, prev_close, prev_volume, change_pct, is_stale) =
            extract_bar_metrics(snap, now_et);

        if is_stale { continue; }

        if !close.is_finite() || close <= 0.0 {
            continue;
        }
        if prev_close <= 0.0 {
            continue;
        }
        if !(params.price_min..=params.price_max).contains(&close) {
            continue;
        }
        if volume < params.volume_min {
            continue;
        }
        if params.min_dollar_volume > 0.0 && close * volume < params.min_dollar_volume {
            continue;
        }

        // Change filter: abs() only if allow_negative_change (shorts enabled)
        if params.allow_negative_change {
            if change_pct.abs() < params.change_min {
                continue;
            }
        } else if change_pct < params.change_min {
            continue;
        }

        let rvol = match params.rvol_method {
            RvolMethod::TimeAdjusted => time_adjusted_rvol(volume, prev_volume, now_et),
            RvolMethod::Naive => if prev_volume > 0.0 { volume / prev_volume } else { 1.0 },
        };
        if rvol < params.rvol_min {
            continue;
        }

        let gap_pct = if prev_close > 0.0 {
            ((open_price - prev_close) / prev_close) * 100.0
        } else {
            0.0
        };
        // Gap filter: abs() only if allow_negative_change
        if params.allow_negative_change {
            if gap_pct.abs() < params.gap_min {
                continue;
            }
        } else if params.gap_min > 0.0 && gap_pct < params.gap_min {
            continue;
        }

        let score = compute_candidate_score_weighted(
            change_pct, rvol, gap_pct, volume, prev_volume,
            w_change, w_rvol, w_gap, w_volume,
        );

        candidates.push(PreFilterCandidate {
            symbol: sym.clone(),
            price: close,
            change_percent: change_pct,
            volume,
            prev_volume,
            rvol,
            gap_percent: gap_pct,
            prefilter_score: score,
            dollar_volume: close * volume,
            is_stale,
        });
    }

    // Tiered Scanner (S15): pre-rank by dollar volume, then split into tiers
    let tiered_enabled = params.tiered_scan_enabled;
    let tier1_count = params.tier1_count;
    let tier2_count = params.tier2_count;

    if tiered_enabled && candidates.len() > tier1_count {
        // Sort by dollar volume DESC for tier assignment
        candidates.sort_by(|a, b| {
            b.dollar_volume
                .partial_cmp(&a.dollar_volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let tier1: Vec<PreFilterCandidate> = candidates
            .iter()
            .take(tier1_count)
            .cloned()
            .collect();

        let rvol_gate = params.rvol_min.max(1.5);
        let change_gate = 3.0_f64;
        let tier2: Vec<PreFilterCandidate> = candidates
            .iter()
            .skip(tier1_count)
            .take(tier2_count)
            .filter(|c| c.rvol >= rvol_gate || c.change_percent.abs() >= change_gate)
            .cloned()
            .collect();

        let tier2_scored = tier2.len();
        candidates = tier1.into_iter().chain(tier2).collect();

        // Re-sort merged set by prefilter score
        candidates.sort_by(|a, b| {
            b.prefilter_score
                .partial_cmp(&a.prefilter_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol.cmp(&b.symbol))
        });

        tracing::info!(
            component = "scanner",
            tier1 = tier1_count,
            tier2_scored,
            total = candidates.len(),
            "[TIERED_SCAN] Pre-rank complete"
        );
    } else {
        // Flat sort: score DESC, then symbol ASC for determinism
        candidates.sort_by(|a, b| {
            b.prefilter_score
                .partial_cmp(&a.prefilter_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol.cmp(&b.symbol))
        });
    }

    candidates.truncate(params.max_candidates);
    candidates
}

/// Fetch float (estimated) + profile data from FMP /stable/profile.
/// Float is estimated as marketCap / price ≈ shares outstanding (in millions).
/// Returns (float_map, profile_map) in a single pass — saves API calls!
pub async fn fetch_profile_and_float_batch(
    fmp_key: &str,
    symbols: &[String],
) -> (HashMap<String, f64>, HashMap<String, FmpProfileData>) {
    let mut float_map: HashMap<String, f64> = HashMap::new();
    let mut profile_map: HashMap<String, FmpProfileData> = HashMap::new();
    if fmp_key.is_empty() {
        return (float_map, profile_map);
    }

    let mut to_fetch: Vec<String> = Vec::new();
    {
        let fcache = FLOAT_CACHE.lock().await;
        let pcache = PROFILE_CACHE.lock().await;
        for sym in symbols {
            let have_float = fcache.get(sym).is_some_and(|(_, ts)| ts.elapsed() < FLOAT_CACHE_TTL);
            let have_profile = pcache.get(sym).is_some_and(|(_, ts)| ts.elapsed() < PROFILE_CACHE_TTL);
            if have_float && have_profile {
                if let Some((fv, _)) = fcache.get(sym) { float_map.insert(sym.clone(), *fv); }
                if let Some((pd, _)) = pcache.get(sym) { profile_map.insert(sym.clone(), pd.clone()); }
                continue;
            }
            to_fetch.push(sym.clone());
        }
    }

    if to_fetch.is_empty() {
        tracing::debug!(component = "scanner", "Profile+float cache hit for all {} symbols", symbols.len());
        return (float_map, profile_map);
    }

    tracing::info!(
        component = "scanner",
        "Fetching profile+float for {} symbols from FMP /stable/profile ({} cached)",
        to_fetch.len(),
        float_map.len()
    );

    let client = build_client();
    for sym in &to_fetch {
        let url = format!(
            "https://financialmodelingprep.com/stable/profile?symbol={}&apikey={}",
            sym, fmp_key
        );
        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(entries) = resp.json::<Vec<FmpProfileEntry>>().await {
                    if let Some(e) = entries.first() {
                        let cap_m = e.market_cap / 1_000_000.0;
                        // Estimate float ≈ sharesOutstanding = marketCap / price
                        let est_float_m = if e.price > 0.0 {
                            (e.market_cap / e.price) / 1_000_000.0
                        } else { 0.0 };
                        float_map.insert(sym.clone(), est_float_m);
                        let pdata = FmpProfileData { market_cap_m: cap_m, sector: e.sector.clone() };
                        profile_map.insert(sym.clone(), pdata.clone());
                        let mut fc = FLOAT_CACHE.lock().await;
                        fc.insert(sym.clone(), (est_float_m, std::time::Instant::now()));
                        let mut pc = PROFILE_CACHE.lock().await;
                        pc.insert(sym.clone(), (pdata, std::time::Instant::now()));
                    }
                } else {
                    tracing::warn!(component = "scanner", "FMP profile parse failed for {}", sym);
                }
            }
            Err(e) => {
                tracing::warn!(component = "scanner", "FMP profile fetch failed for {}: {}", sym, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    (float_map, profile_map)
}

// NOTE: fetch_profile_batch is now merged into fetch_profile_and_float_batch above.
// Kept as thin wrapper for backward compatibility.
#[allow(dead_code)]
pub async fn fetch_profile_batch(
    fmp_key: &str,
    symbols: &[String],
) -> HashMap<String, FmpProfileData> {
    let (_float, profile) = fetch_profile_and_float_batch(fmp_key, symbols).await;
    profile
}

/// Fetch insider trading signals from FMP.
pub async fn fetch_insider_batch(
    fmp_key: &str,
    symbols: &[String],
) -> HashMap<String, InsiderSignal> {
    let mut result: HashMap<String, InsiderSignal> = HashMap::new();
    if fmp_key.is_empty() { return result; }
    let mut to_fetch: Vec<String> = Vec::new();
    {
        let cache = INSIDER_CACHE.lock().await;
        for sym in symbols {
            if let Some((sig, ts)) = cache.get(sym) {
                if ts.elapsed() < INSIDER_CACHE_TTL {
                    result.insert(sym.clone(), sig.clone());
                    continue;
                }
            }
            to_fetch.push(sym.clone());
        }
    }
    if to_fetch.is_empty() { return result; }
    tracing::info!(component = "scanner", "FMP insider fetch: {} new, {} cached", to_fetch.len(), result.len());
    let client = build_client();
    for sym in &to_fetch {
        let url = format!("https://financialmodelingprep.com/stable/insider-trading?symbol={}&limit=5&apikey={}", sym, fmp_key);
        let sig = match client.get(&url).send().await {
            Ok(resp) => {
                match resp.json::<Vec<FmpInsiderEntry>>().await {
                    Ok(entries) => {
                        if let Some(latest) = entries.first() {
                            let is_buy = latest.transaction_type.contains("P-Purchase") || latest.transaction_type.contains("Purchase");
                            let is_sell = latest.transaction_type.contains("S-Sale") || latest.transaction_type.contains("Sale");
                            let signal = if is_buy { "BUY" } else if is_sell { "SELL" } else { "" };
                            let detail = if !signal.is_empty() {
                                format!("{} {} {}sh @${:.2}", latest.reporting_name, signal, latest.securities_transacted as i64, latest.price)
                            } else { String::new() };
                            InsiderSignal { signal: signal.to_string(), detail }
                        } else {
                            InsiderSignal { signal: String::new(), detail: String::new() }
                        }
                    }
                    Err(_) => InsiderSignal { signal: String::new(), detail: String::new() },
                }
            }
            Err(_) => InsiderSignal { signal: String::new(), detail: String::new() },
        };
        result.insert(sym.clone(), sig.clone());
        INSIDER_CACHE.lock().await.insert(sym.clone(), (sig, std::time::Instant::now()));
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    result
}

/// Fetch earnings calendar — returns set of symbols with earnings in ±3 days.
pub async fn fetch_earnings_calendar(
    fmp_key: &str,
) -> HashMap<String, bool> {
    if fmp_key.is_empty() { return HashMap::new(); }
    // Check cache
    {
        let cache = EARNINGS_CACHE.lock().await;
        if let Some((ts, data)) = cache.as_ref() {
            if ts.elapsed() < EARNINGS_CACHE_TTL {
                return data.clone();
            }
        }
    }
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let future = (chrono::Utc::now() + chrono::Duration::days(3)).format("%Y-%m-%d").to_string();
    let url = format!("https://financialmodelingprep.com/stable/earning-calendar?from={}&to={}&apikey={}", today, future, fmp_key);
    let client = build_client();
    let mut map: HashMap<String, bool> = HashMap::new();
    if let Ok(resp) = client.get(&url).send().await {
        if let Ok(entries) = resp.json::<Vec<FmpEarningEntry>>().await {
            for e in &entries {
                if !e.symbol.is_empty() {
                    map.insert(e.symbol.clone(), true);
                }
            }
        }
    }
    tracing::info!(component = "scanner", "FMP earnings calendar: {} symbols with earnings in ±3 days", map.len());
    *EARNINGS_CACHE.lock().await = Some((std::time::Instant::now(), map.clone()));
    map
}

/// Fetch analyst price target consensus from FMP.
pub async fn fetch_targets_batch(
    fmp_key: &str,
    symbols: &[String],
    prices: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let mut result: HashMap<String, f64> = HashMap::new();
    if fmp_key.is_empty() { return result; }
    let mut to_fetch: Vec<String> = Vec::new();
    {
        let cache = TARGETS_CACHE.lock().await;
        for sym in symbols {
            if let Some((upside, ts)) = cache.get(sym) {
                if ts.elapsed() < TARGETS_CACHE_TTL {
                    result.insert(sym.clone(), *upside);
                    continue;
                }
            }
            to_fetch.push(sym.clone());
        }
    }
    if to_fetch.is_empty() { return result; }
    tracing::info!(component = "scanner", "FMP targets fetch: {} new, {} cached", to_fetch.len(), result.len());
    let client = build_client();
    for sym in &to_fetch {
        let url = format!("https://financialmodelingprep.com/stable/price-target-consensus?symbol={}&apikey={}", sym, fmp_key);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(entries) = resp.json::<Vec<FmpTargetEntry>>().await {
                if let Some(e) = entries.first() {
                    let current_price = prices.get(sym).copied().unwrap_or(0.0);
                    let upside = if current_price > 0.0 && e.target_consensus > 0.0 {
                        ((e.target_consensus - current_price) / current_price) * 100.0
                    } else { 0.0 };
                    result.insert(sym.clone(), upside);
                    TARGETS_CACHE.lock().await.insert(sym.clone(), (upside, std::time::Instant::now()));
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    result
}

/// New unified discovery scan that respects ALL user-set ScanParams.
pub async fn scan_discovery(
    api_key: &str,
    secret_key: &str,
    params: &ScanParams,
    fmp_key: &str,
) -> Result<Vec<ScanResult>, TitanError> {
    let assets = fetch_active_assets(api_key, secret_key).await?;
    let symbols: Vec<String> = assets
        .into_iter()
        .filter(|a| a.tradable && is_us_exchange(&a.exchange))
        .map(|a| a.symbol)
        .collect();
    tracing::info!(
        component = "scanner",
        "DISCOVERY scanning {} symbols | price {}-{}, vol>={}, chg>={}%, rvol>={}, gap>={}%, float_max={}M, cap_max={}B",
        symbols.len(),
        params.price_min, params.price_max,
        params.volume_min, params.change_min,
        params.rvol_min, params.gap_min,
        params.float_max, params.cap_max
    );

    let snaps = fetch_snapshots_batch(api_key, secret_key, &symbols).await;
    let prefiltered = quant_prefilter(&snaps, params);
    let pf_symbols: Vec<String> = prefiltered.iter().map(|p| p.symbol.clone()).collect();

    // --- FMP enrichment (all post-filter, saves API calls) ---
    // Unified call: profile + float in ONE pass (saves 50% API calls)
    let (float_map, profile_map) = fetch_profile_and_float_batch(fmp_key, &pf_symbols).await;
    let insider_map = fetch_insider_batch(fmp_key, &pf_symbols).await;
    let earnings_map = fetch_earnings_calendar(fmp_key).await;
    let price_map: HashMap<String, f64> = prefiltered.iter().map(|p| (p.symbol.clone(), p.price)).collect();
    let targets_map = fetch_targets_batch(fmp_key, &pf_symbols, &price_map).await;

    let results: Vec<ScanResult> = prefiltered
        .into_iter()
        .filter_map(|pf| {
            let float_m = float_map.get(&pf.symbol).copied().unwrap_or(0.0);
            if params.float_max > 0.0 && float_m > 0.0 && float_m > params.float_max {
                return None;
            }
            let profile = profile_map.get(&pf.symbol);
            let cap_m = profile.map(|p| p.market_cap_m).unwrap_or(0.0);
            let cap_b = cap_m / 1000.0;
            if params.cap_max > 0.0 && cap_b > 0.0 && cap_b > params.cap_max {
                return None;
            }
            let has_earnings = earnings_map.get(&pf.symbol).copied().unwrap_or(false);
            if params.exclude_earnings && has_earnings {
                return None;
            }
            let insider = insider_map.get(&pf.symbol);
            let upside = targets_map.get(&pf.symbol).copied().unwrap_or(0.0);

            Some(ScanResult {
                symbol: pf.symbol,
                price: pf.price,
                change_percent: pf.change_percent,
                volume: pf.volume,
                rvol: pf.rvol,
                gap_percent: pf.gap_percent,
                score: pf.prefilter_score,
                strategy: "DISCOVERY_V8".to_string(),
                float_shares: float_m,
                market_cap: cap_m,
                sector: profile.map(|p| p.sector.clone()).unwrap_or_default(),
                insider_signal: insider.map(|i| i.signal.clone()).unwrap_or_default(),
                insider_detail: insider.map(|i| i.detail.clone()).unwrap_or_default(),
                has_earnings_soon: has_earnings,
                analyst_upside_pct: upside,
                dollar_volume: pf.dollar_volume,
                is_stale: false,
            })
        })
        .collect();

    tracing::info!(
        component = "scanner",
        "DISCOVERY V8: {} candidates passed all filters (from {} prefiltered, {} snapshots)",
        results.len(), pf_symbols.len(), snaps.len()
    );

    Ok(results)
}

/// Legacy scan_shark — kept for backward compatibility but deprecated.
#[deprecated(note = "Use scan_discovery() with ScanParams instead")]
#[allow(dead_code)]
pub async fn scan_shark(api_key: &str, secret_key: &str) -> Result<Vec<ScanResult>, TitanError> {
    scan_discovery(api_key, secret_key, &ScanParams::default(), "").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_snapshot(close: f64, open: f64, volume: f64, prev_close: f64, prev_vol: f64) -> SnapshotData {
        let recent_time = chrono::Utc::now().to_rfc3339();
        SnapshotData {
            latest_trade: Some(SnapshotTrade {
                p: close,
                s: 100.0,
                t: recent_time,
                c: None,
            }),
            minute_bar: None,
            daily_bar: SnapshotDailyBar { c: close, o: open, v: volume },
            prev_daily_bar: Some(SnapshotDailyBar { c: prev_close, o: prev_close, v: prev_vol }),
        }
    }

    #[test]
    fn test_dollar_volume_gate_rejects_low_notional() {
        let mut snaps = HashMap::new();
        // Penny stock: $0.50, 1M shares = $500K dollar volume
        snaps.insert("PENNY".to_string(), make_snapshot(0.50, 0.45, 1_000_000.0, 0.40, 500_000.0));
        let params = ScanParams {
            price_min: 0.01,
            price_max: 1000.0,
            volume_min: 100_000.0,
            change_min: 0.0,
            rvol_min: 0.0,
            gap_min: 0.0,
            min_dollar_volume: 5_000_000.0,
            ..ScanParams::default()
        };
        let results = quant_prefilter(&snaps, &params);
        assert!(
            results.is_empty(),
            "Penny stock with $500K dollar volume should be rejected by $5M gate"
        );
    }

    #[test]
    fn test_dollar_volume_gate_passes_high_notional() {
        let mut snaps = HashMap::new();
        // Normal stock: $50, 2M shares = $100M dollar volume
        snaps.insert("AAPL".to_string(), make_snapshot(50.0, 48.0, 2_000_000.0, 47.0, 1_800_000.0));
        let params = ScanParams {
            price_min: 0.01,
            price_max: 1000.0,
            volume_min: 100_000.0,
            change_min: 0.0,
            rvol_min: 0.0,
            gap_min: 0.0,
            min_dollar_volume: 5_000_000.0,
            ..ScanParams::default()
        };
        let results = quant_prefilter(&snaps, &params);
        assert_eq!(results.len(), 1, "Stock with $100M dollar volume should pass $5M gate");
        assert!(
            (results[0].dollar_volume - 100_000_000.0).abs() < 1.0,
            "dollar_volume should be populated: got {}",
            results[0].dollar_volume
        );
    }

    // ═══ S15: Tiered Scanner Tests ═══

    #[test]
    fn tiered_scan_splits_by_dollar_volume() {
        let mut snaps = HashMap::new();
        for i in 0..80 {
            let price = 10.0 + (i as f64);
            let volume = 1_000_000.0 + (i as f64) * 100_000.0;
            let prev_close = price * 0.97;
            snaps.insert(
                format!("SYM{:03}", i),
                make_snapshot(price, price * 0.98, volume, prev_close, volume * 0.8),
            );
        }
        let params = ScanParams {
            price_min: 1.0,
            price_max: 1000.0,
            volume_min: 100_000.0,
            change_min: 0.0,
            rvol_min: 0.0,
            gap_min: 0.0,
            min_dollar_volume: 0.0,
            tiered_scan_enabled: true,
            tier1_count: 20,
            tier2_count: 30,
            max_candidates: 50,
            ..ScanParams::default()
        };
        let results = quant_prefilter(&snaps, &params);
        assert!(results.len() <= 50, "Should not exceed max_candidates");
        assert!(results.len() >= 20, "Should have at least tier1 count");
    }

    #[test]
    fn tiered_scan_disabled_uses_flat_sort() {
        let mut snaps = HashMap::new();
        for i in 0..30 {
            let price = 50.0;
            let volume = 2_000_000.0;
            let prev_close = 48.0;
            snaps.insert(
                format!("FLAT{:02}", i),
                make_snapshot(price, price * 0.98, volume, prev_close, volume * 0.9),
            );
        }
        let params = ScanParams {
            price_min: 1.0,
            price_max: 1000.0,
            volume_min: 100_000.0,
            change_min: 0.0,
            rvol_min: 0.0,
            gap_min: 0.0,
            min_dollar_volume: 0.0,
            tiered_scan_enabled: false,
            max_candidates: 20,
            ..ScanParams::default()
        };
        let results = quant_prefilter(&snaps, &params);
        assert_eq!(results.len(), 20, "Flat sort should truncate to max_candidates");
        for i in 1..results.len() {
            assert!(
                results[i - 1].prefilter_score >= results[i].prefilter_score,
                "Should be sorted by score DESC"
            );
        }
    }

    #[test]
    fn scan_params_serde_roundtrip_with_tiered() {
        let params = ScanParams {
            tiered_scan_enabled: true,
            tier1_count: 40,
            tier2_count: 80,
            ..ScanParams::default()
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ScanParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tiered_scan_enabled, true);
        assert_eq!(deserialized.tier1_count, 40);
        assert_eq!(deserialized.tier2_count, 80);
    }
}

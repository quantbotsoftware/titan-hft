use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::OnceLock;
use tauri::Manager;

// ═══════════════════════════════════════════════════════════════
//  DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize, Debug)]
struct AlpacaNewsItem {
    headline: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct AlpacaNewsResponse {
    news: Vec<AlpacaNewsItem>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CatalystEvent {
    pub symbol: String,
    pub headline: String,
    pub summary: String,
    pub url: String,
    pub sentiment_score: f64,
    pub timestamp: String,
    /// Severity label: EXPLOSIVE, STRONG, BEARISH_STRONG, BEARISH_EXPLOSIVE
    pub severity: String,
}

pub fn catalyst_severity(score: f64) -> String {
    if score >= 0.85 {
        "EXPLOSIVE".into()
    } else if score >= 0.70 {
        "STRONG".into()
    } else if score <= -0.85 {
        "BEARISH_EXPLOSIVE".into()
    } else if score <= -0.70 {
        "BEARISH_STRONG".into()
    } else {
        "MODERATE".into()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct NewsArticle {
    pub headline: String,
    pub summary: String,
    pub source: String,
    pub url: String,
    pub timestamp: String,
    pub sentiment_score: f64,
    pub sentiment_label: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SentimentSnapshot {
    pub overall_score: f64,
    pub overall_label: String,
    pub articles: Vec<NewsArticle>,
    pub history: Vec<(String, f64)>,
}

// ═══════════════════════════════════════════════════════════════
//  LEXICON — keyword → weight (bullish > 0, bearish < 0)
// ═══════════════════════════════════════════════════════════════

pub struct LexiconEntry {
    pub pattern: Regex,
    pub weight: f64,
}

/// Negation tokens — if any of these appear in the 4 words before a
/// keyword match, the keyword's weight is flipped.
const NEGATION_TOKENS: &[&str] = &[
    "not",
    "no",
    "never",
    "neither",
    "nor",
    "without",
    "didn't",
    "doesn't",
    "don't",
    "hasn't",
    "haven't",
    "won't",
    "wouldn't",
    "couldn't",
    "shouldn't",
    "fails",
    "failed",
    "unable",
    "unlikely",
];

fn build_lexicon() -> Vec<LexiconEntry> {
    let terms: Vec<(&str, f64)> = vec![
        // ── BULLISH (+0.3 to +1.0) ──────────────────────────
        // Earnings
        ("beat", 0.80),
        ("beats", 0.80),
        ("surpass", 0.75),
        ("surpasses", 0.75),
        ("exceeded", 0.75),
        ("top estimate", 0.85),
        ("beat estimate", 0.90),
        ("above estimate", 0.85),
        ("record revenue", 0.80),
        ("record earnings", 0.80),
        ("eps beat", 0.85),
        ("raised guidance", 0.90),
        ("raises guidance", 0.90),
        ("upbeat guidance", 0.85),
        // Price action
        ("surge", 0.70),
        ("surges", 0.70),
        ("soar", 0.70),
        ("soars", 0.70),
        ("jump", 0.60),
        ("jumps", 0.60),
        ("rally", 0.70),
        ("rallies", 0.70),
        ("breakout", 0.60),
        ("rebound", 0.60),
        ("recover", 0.50),
        ("momentum", 0.45),
        ("bullish", 0.65),
        ("all-time high", 0.80),
        // Ratings
        ("upgrade", 0.85),
        ("outperform", 0.80),
        ("overweight", 0.70),
        ("buy rating", 0.70),
        ("target raised", 0.85),
        ("raises target", 0.85),
        ("top pick", 0.70),
        ("strong buy", 0.80),
        ("accumulate", 0.60),
        ("upside", 0.55),
        // Corporate
        ("approval", 0.80),
        ("approved", 0.80),
        ("partnership", 0.55),
        ("acquisition", 0.55),
        ("launch", 0.45),
        ("expand", 0.40),
        ("innovation", 0.40),
        ("growth", 0.45),
        ("profit", 0.55),
        ("dividend", 0.40),
        ("catalyst", 0.45),
        ("tailwind", 0.45),
        // ── BEARISH (-0.3 to -1.0) ──────────────────────────
        // Earnings
        ("miss", -0.80),
        ("misses", -0.80),
        ("missed", -0.80),
        ("miss estimate", -0.90),
        ("below estimate", -0.85),
        ("cut guidance", -0.90),
        ("cuts guidance", -0.90),
        ("lowers guidance", -0.85),
        ("weak guidance", -0.80),
        // Price action
        ("plunge", -0.80),
        ("plunges", -0.80),
        ("crash", -0.90),
        ("crashes", -0.90),
        ("tumble", -0.70),
        ("tumbles", -0.70),
        ("slide", -0.50),
        ("slides", -0.50),
        ("drop", -0.55),
        ("drops", -0.55),
        ("fall", -0.50),
        ("falls", -0.50),
        ("decline", -0.55),
        ("declines", -0.55),
        ("slump", -0.65),
        ("sell-off", -0.70),
        ("selloff", -0.70),
        ("correction", -0.50),
        ("bearish", -0.65),
        ("all-time low", -0.80),
        // Ratings
        ("downgrade", -0.85),
        ("underperform", -0.80),
        ("underweight", -0.70),
        ("sell rating", -0.70),
        ("target cut", -0.85),
        ("cuts target", -0.85),
        ("reduces target", -0.80),
        ("overvalued", -0.55),
        ("downside", -0.55),
        // Corporate / risk
        ("lawsuit", -0.80),
        ("investigation", -0.75),
        ("fraud", -1.00),
        ("bankruptcy", -1.00),
        ("default", -0.85),
        ("recall", -0.65),
        ("layoff", -0.60),
        ("layoffs", -0.60),
        ("restructuring", -0.50),
        ("warning", -0.65),
        ("recession", -0.75),
        ("headwind", -0.45),
        ("volatility", -0.30),
        ("risk", -0.30),
        ("concern", -0.35),
        ("delay", -0.45),
        ("delayed", -0.45),
    ];

    terms
        .into_iter()
        .map(|(term, weight)| {
            // Escape regex special chars, then wrap with word boundaries
            let escaped = regex::escape(term);
            let pattern =
                Regex::new(&format!(r"(?i)\b{}\b", escaped)).expect("invalid lexicon regex");
            LexiconEntry { pattern, weight }
        })
        .collect()
}

/// Cached lexicon singleton — compiled once, reused forever.
static LEXICON: OnceLock<Vec<LexiconEntry>> = OnceLock::new();

pub fn get_lexicon() -> &'static [LexiconEntry] {
    LEXICON.get_or_init(build_lexicon)
}

// ═══════════════════════════════════════════════════════════════
//  SCORING ENGINE
// ═══════════════════════════════════════════════════════════════

/// Check if any negation token appears in the N words before `match_start`
/// in the original text.
fn is_negated(text: &str, match_start: usize, lookback_words: usize) -> bool {
    // Grab the chunk of text before the match position
    let prefix = &text[..match_start];
    let words: Vec<&str> = prefix.split_whitespace().collect();
    let window = if words.len() > lookback_words {
        &words[words.len() - lookback_words..]
    } else {
        &words
    };

    let window_lower: Vec<String> = window.iter().map(|w| w.to_lowercase()).collect();
    for neg in NEGATION_TOKENS {
        if window_lower.iter().any(|w| w == neg) {
            return true;
        }
    }
    false
}

pub fn score_text(text: &str, lexicon: &[LexiconEntry]) -> f64 {
    let lower = text.to_lowercase();
    let mut score = 0.0;
    let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

    // Sort lexicon by term length DESC — longer patterns first to avoid
    // double-counting substrings (e.g. "beat estimate" before "beat")
    // Note: we iterate in the order they're built; build_lexicon already
    //       has multi-word terms interleaved, so we just skip overlapping matches.

    for entry in lexicon {
        for m in entry.pattern.find_iter(&lower) {
            let start = m.start();
            let end = m.end();

            // Skip if this region was already scored by a longer/earlier term
            if matched_ranges.iter().any(|&(s, e)| start >= s && end <= e) {
                continue;
            }

            let weight = if is_negated(&lower, start, 4) {
                -entry.weight
            } else {
                entry.weight
            };

            score += weight;
            matched_ranges.push((start, end));
        }
    }

    score.clamp(-1.0, 1.0)
}

pub fn label_from_score(score: f64) -> String {
    if score >= 0.5 {
        "BULLISH".into()
    } else if score >= 0.15 {
        "SLIGHTLY BULLISH".into()
    } else if score > -0.15 {
        "NEUTRAL".into()
    } else if score > -0.5 {
        "SLIGHTLY BEARISH".into()
    } else {
        "BEARISH".into()
    }
}

// ═══════════════════════════════════════════════════════════════
//  PUBLIC API — per-symbol sentiment
// ═══════════════════════════════════════════════════════════════

pub async fn get_sentiment_score(
    client: &reqwest::Client,
    symbol: &str,
    api_key: &str,
    secret: &str,
) -> f64 {
    let url = format!(
        "https://data.alpaca.markets/v1beta1/news?symbols={}&limit=5",
        symbol
    );
    let lexicon = get_lexicon();

    let res = match client
        .get(&url)
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                component = "sentiment",
                symbol = symbol,
                "News API request failed: {}",
                e
            );
            return 0.0;
        }
    };

    if !res.status().is_success() {
        tracing::warn!(component = "sentiment", symbol = symbol, status = %res.status(), "News API non-success");
        return 0.0;
    }

    let news_data: AlpacaNewsResponse = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                component = "sentiment",
                symbol = symbol,
                "News API JSON parse error: {}",
                e
            );
            return 0.0;
        }
    };

    if news_data.news.is_empty() {
        return 0.0;
    }

    let mut total = 0.0;
    for item in &news_data.news {
        let text = format!("{} {}", item.headline, item.summary);
        total += score_text(&text, lexicon);
    }

    let final_score = total / news_data.news.len() as f64;
    tracing::info!(
        component = "sentiment",
        symbol = symbol,
        score = format!("{:.2}", final_score).as_str(),
        "Sentiment scored"
    );
    final_score
}

pub async fn fetch_news_feed(
    client: &reqwest::Client,
    symbol: &str,
    api_key: &str,
    secret: &str,
    limit: u32,
) -> Vec<NewsArticle> {
    let url = format!(
        "https://data.alpaca.markets/v1beta1/news?symbols={}&limit={}",
        symbol, limit
    );
    let lexicon = get_lexicon();

    let res = match client
        .get(&url)
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                component = "sentiment",
                symbol = symbol,
                "News feed request failed: {}",
                e
            );
            return Vec::new();
        }
    };

    if !res.status().is_success() {
        tracing::warn!(component = "sentiment", symbol = symbol, status = %res.status(), "News feed non-success");
        return Vec::new();
    }

    let news_data: AlpacaNewsResponse = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                component = "sentiment",
                symbol = symbol,
                "News feed JSON parse error: {}",
                e
            );
            return Vec::new();
        }
    };

    news_data
        .news
        .into_iter()
        .map(|item| {
            let text = format!("{} {}", item.headline, item.summary);
            let score = score_text(&text, lexicon);
            NewsArticle {
                headline: item.headline,
                summary: item.summary,
                source: item.source,
                url: item.url,
                timestamp: item.created_at,
                sentiment_score: score,
                sentiment_label: label_from_score(score),
            }
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════
//  CATALYST WATCHDOG LOOP — whole-market news scanner
// ═══════════════════════════════════════════════════════════════

const CATALYST_THRESHOLD: f64 = 0.50;
const SEEN_HEADLINES_CAP: usize = 500;
const POLL_INTERVAL_SECS: u64 = 60;
/// Per-symbol cooldown to prevent alert spam (5 minutes)
const SYMBOL_COOLDOWN_SECS: u64 = 300;

pub async fn catalyst_watchdog_loop(
    state: std::sync::Arc<crate::TitanState>,
    app: tauri::AppHandle,
) {
    // FIFO dedup queue — evicts oldest headlines one at a time, never bulk clears
    let mut seen_headlines: VecDeque<String> = VecDeque::with_capacity(SEEN_HEADLINES_CAP + 1);
    // Per-symbol cooldown tracker: symbol → last alert Instant
    let mut symbol_cooldowns: HashMap<String, std::time::Instant> = HashMap::new();
    let lexicon = get_lexicon();
    let watchdog_start = std::time::Instant::now();
    let mut total_articles_scanned: u64 = 0;
    let mut total_alerts_fired: u64 = 0;

    tracing::info!(
        component = "catalyst_watchdog",
        "Catalyst Watchdog started — polling every {}s, threshold ≥ {}",
        POLL_INTERVAL_SECS,
        CATALYST_THRESHOLD
    );

    loop {
        let tick_start = std::time::Instant::now();

        let (api_key, secret) = {
            let k = state.api_keys.lock().unwrap_or_else(|e| e.into_inner());
            (k.key.clone(), k.secret.clone())
        };

        if api_key.is_empty() || secret.is_empty() {
            state.watchdog.heartbeat("catalyst_watchdog", 0);
            // Emit stats even when no keys
            let _ = app.emit_all(
                "catalyst-watchdog-stats",
                serde_json::json!({
                    "status": "NO_KEYS",
                    "articles_scanned": total_articles_scanned,
                    "alerts_fired": total_alerts_fired,
                    "uptime_secs": watchdog_start.elapsed().as_secs(),
                    "last_poll_ms": 0,
                }),
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue;
        }

        // Fetch entire market news (no symbol filter)
        let url = "https://data.alpaca.markets/v1beta1/news?limit=20";
        let client = state.http_client.clone();

        let res = client
            .get(url)
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &secret)
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AlpacaNewsResponse>().await {
                        Ok(news_data) => {
                            total_articles_scanned += news_data.news.len() as u64;

                            for item in news_data.news {
                                // Dedup by headline
                                if seen_headlines.iter().any(|h| h == &item.headline) {
                                    continue;
                                }

                                // Filter out crypto symbols (e.g., BTC/USD, BTCUSD)
                                let stock_symbols: Vec<String> = item
                                    .symbols
                                    .into_iter()
                                    .filter(|s| {
                                        !s.contains('/')
                                            && !s.contains("USD")
                                            && !s.starts_with("CRYPTO")
                                    })
                                    .collect();

                                if stock_symbols.is_empty() {
                                    continue;
                                }

                                let text = format!("{} {}", item.headline, item.summary);
                                let mut score = score_text(&text, lexicon);

                                // LLM sentiment override (if enabled)
                                let (llm_on, llm_key, llm_prov) = {
                                    let s =
                                        state.settings.lock().unwrap_or_else(|e| e.into_inner());
                                    (
                                        s.llm_sentiment_enabled,
                                        s.llm_api_key.clone(),
                                        s.llm_provider.clone(),
                                    )
                                };
                                if llm_on && !llm_key.is_empty() {
                                    let first_sym =
                                        stock_symbols.first().cloned().unwrap_or_default();
                                    if let Some(llm_result) = crate::llm_sentiment::score_with_llm(
                                        &state.http_client,
                                        &item.headline,
                                        &item.summary,
                                        &first_sym,
                                        &llm_key,
                                        &llm_prov,
                                    )
                                    .await
                                    {
                                        score = llm_result.score;
                                        tracing::info!(
                                            component = "catalyst_watchdog",
                                            llm_score = format!("{:.2}", llm_result.score).as_str(),
                                            reasoning = llm_result.reasoning.as_str(),
                                            "LLM sentiment override"
                                        );
                                    }
                                }

                                // Threshold check — bullish OR bearish
                                let is_catalyst =
                                    score >= CATALYST_THRESHOLD || score <= -CATALYST_THRESHOLD;
                                if is_catalyst {
                                    let severity = catalyst_severity(score);
                                    let emoji = if score < 0.0 { "🔻" } else { "🔥" };

                                    // Emit one event per stock symbol in the article
                                    for symbol in &stock_symbols {
                                        // Per-symbol cooldown check
                                        if let Some(last_alert) =
                                            symbol_cooldowns.get(symbol.as_str())
                                        {
                                            if last_alert.elapsed().as_secs() < SYMBOL_COOLDOWN_SECS
                                            {
                                                tracing::debug!(
                                                    component = "catalyst_watchdog",
                                                    symbol = symbol.as_str(),
                                                    "Skipping — cooldown active"
                                                );
                                                continue;
                                            }
                                        }

                                        let event = CatalystEvent {
                                            symbol: symbol.clone(),
                                            headline: item.headline.clone(),
                                            summary: item.summary.clone(),
                                            url: item.url.clone(),
                                            sentiment_score: score,
                                            timestamp: item.created_at.clone(),
                                            severity: severity.clone(),
                                        };

                                        crate::state::log_to_file(&format!(
                                            "{} CATALYST WATCHDOG: {} — {} (Score: {:.2}, {})",
                                            emoji, symbol, item.headline, score, severity
                                        ));
                                        tracing::info!(
                                            component = "catalyst_watchdog",
                                            symbol = symbol.as_str(),
                                            score = format!("{:.2}", score).as_str(),
                                            severity = severity.as_str(),
                                            headline = item.headline.as_str(),
                                            "Catalyst detected"
                                        );

                                        let _ = app.emit_all("catalyst-alert", &event);
                                        total_alerts_fired += 1;

                                        // Update per-symbol cooldown
                                        symbol_cooldowns
                                            .insert(symbol.clone(), std::time::Instant::now());

                                        // Telegram catalyst alert
                                        #[cfg(feature = "telegram")]
                                        {
                                            let (tg_token, tg_chat, tg_on) = {
                                                let s = state
                                                    .settings
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                (
                                                    s.telegram_bot_token.clone(),
                                                    s.telegram_chat_id.clone(),
                                                    s.telegram_enabled,
                                                )
                                            };
                                            if tg_on {
                                                let tg_msg =
                                                    crate::telegram::format_catalyst(&event);
                                                let tok = tg_token;
                                                let cid = tg_chat;
                                                let tg_client = state.http_client.clone();
                                                tokio::spawn(async move {
                                                    crate::telegram::send_telegram_alert(
                                                        &tg_client, &tok, &cid, &tg_msg,
                                                    )
                                                    .await;
                                                });
                                            }
                                        }
                                    }
                                }

                                // Track headline in FIFO dedup (regardless of score)
                                seen_headlines.push_back(item.headline);
                                while seen_headlines.len() > SEEN_HEADLINES_CAP {
                                    seen_headlines.pop_front();
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                component = "catalyst_watchdog",
                                "JSON parse error: {}",
                                e
                            );
                            state.watchdog.record_error("catalyst_watchdog");
                        }
                    }
                } else {
                    tracing::warn!(component = "catalyst_watchdog", status = %response.status(), "API non-success response");
                    state.watchdog.record_error("catalyst_watchdog");
                }
            }
            Err(e) => {
                tracing::warn!(component = "catalyst_watchdog", "API request failed: {}", e);
                state.watchdog.record_error("catalyst_watchdog");
            }
        }

        let elapsed_ms = tick_start.elapsed().as_millis() as u64;
        state.watchdog.heartbeat("catalyst_watchdog", elapsed_ms);

        // Emit watchdog health stats for frontend Tab 7
        let _ = app.emit_all(
            "catalyst-watchdog-stats",
            serde_json::json!({
                "status": "LIVE",
                "articles_scanned": total_articles_scanned,
                "alerts_fired": total_alerts_fired,
                "uptime_secs": watchdog_start.elapsed().as_secs(),
                "last_poll_ms": elapsed_ms,
            }),
        );

        // Purge stale cooldowns (older than 2x cooldown period)
        symbol_cooldowns.retain(|_, v| v.elapsed().as_secs() < SYMBOL_COOLDOWN_SECS * 2);

        tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

// ═══════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundary_no_false_positive() {
        let lexicon = build_lexicon();
        // "executive" should NOT trigger "cut" (bearish)
        let score = score_text("The executive announced new plans", &lexicon);
        assert!(
            score.abs() < 0.01,
            "executive should not trigger any keyword, got {:.2}",
            score
        );
    }

    #[test]
    fn test_word_boundary_exact_match() {
        let lexicon = build_lexicon();
        // "beat" alone should trigger
        let score = score_text("Company earnings beat expectations", &lexicon);
        assert!(score > 0.5, "beat should score bullish, got {:.2}", score);
    }

    #[test]
    fn test_negation_flips_score() {
        let lexicon = build_lexicon();
        // "didn't beat" should be negative
        let score = score_text("The company didn't beat estimates", &lexicon);
        assert!(
            score < 0.0,
            "negated beat should be negative, got {:.2}",
            score
        );
    }

    #[test]
    fn test_negation_no_false_negative() {
        let lexicon = build_lexicon();
        // Normal "beat" without negation
        let score_pos = score_text("Revenue beat all expectations", &lexicon);
        // "didn't beat"
        let score_neg = score_text("Revenue didn't beat expectations", &lexicon);
        assert!(
            score_pos > score_neg,
            "negated score ({:.2}) should be less than positive ({:.2})",
            score_neg,
            score_pos
        );
    }

    #[test]
    fn test_score_clamp_positive() {
        let lexicon = build_lexicon();
        // Stack many bullish terms — should clamp at 1.0
        let text =
            "beat surge soar rally jump upgrade approval breakout bullish growth profit outperform";
        let score = score_text(text, &lexicon);
        assert!(
            (score - 1.0).abs() < 0.001,
            "should clamp to 1.0, got {:.4}",
            score
        );
    }

    #[test]
    fn test_score_clamp_negative() {
        let lexicon = build_lexicon();
        // Stack many bearish terms — should clamp at -1.0
        let text = "crash plunge fraud bankruptcy downgrade miss decline slump sell-off";
        let score = score_text(text, &lexicon);
        assert!(
            (score - (-1.0)).abs() < 0.001,
            "should clamp to -1.0, got {:.4}",
            score
        );
    }

    #[test]
    fn test_label_thresholds() {
        assert_eq!(label_from_score(0.8), "BULLISH");
        assert_eq!(label_from_score(0.5), "BULLISH");
        assert_eq!(label_from_score(0.3), "SLIGHTLY BULLISH");
        assert_eq!(label_from_score(0.15), "SLIGHTLY BULLISH");
        assert_eq!(label_from_score(0.0), "NEUTRAL");
        assert_eq!(label_from_score(-0.1), "NEUTRAL");
        assert_eq!(label_from_score(-0.3), "SLIGHTLY BEARISH");
        assert_eq!(label_from_score(-0.5), "BEARISH"); // exactly -0.5 is NOT > -0.5
        assert_eq!(label_from_score(-0.51), "BEARISH");
        assert_eq!(label_from_score(-1.0), "BEARISH");
    }

    #[test]
    fn test_multi_word_term() {
        let lexicon = build_lexicon();
        // "beat estimate" should score high
        let score = score_text(
            "Shares rose after EPS beat estimate by a wide margin",
            &lexicon,
        );
        assert!(
            score > 0.6,
            "beat estimate should score high, got {:.2}",
            score
        );
    }

    #[test]
    fn test_neutral_text() {
        let lexicon = build_lexicon();
        let score = score_text("The weather in New York is cloudy today", &lexicon);
        assert!(
            score.abs() < 0.01,
            "neutral text should score ~0, got {:.2}",
            score
        );
    }

    #[test]
    fn test_case_insensitive() {
        let lexicon = build_lexicon();
        let score_lower = score_text("surge in revenue", &lexicon);
        let score_upper = score_text("SURGE IN REVENUE", &lexicon);
        assert!(
            (score_lower - score_upper).abs() < 0.001,
            "case should not matter"
        );
    }
}

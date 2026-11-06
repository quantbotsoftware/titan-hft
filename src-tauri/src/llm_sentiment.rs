use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmScore {
    pub score: f64,
    pub reasoning: String,
    pub confidence: f64,
    #[serde(default)]
    pub timestamp_secs: u64,
    #[serde(default)]
    pub decay_factor: f64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
pub fn apply_decay(score: &LlmScore, halflife_secs: u64) -> LlmScore {
    if halflife_secs == 0 || score.timestamp_secs == 0 {
        return score.clone();
    }
    let age = now_secs().saturating_sub(score.timestamp_secs) as f64;
    let lambda = (2.0_f64).ln() / halflife_secs as f64;
    let factor = (-lambda * age).exp();
    LlmScore {
        score: score.score,
        reasoning: score.reasoning.clone(),
        confidence: score.confidence * factor,
        timestamp_secs: score.timestamp_secs,
        decay_factor: factor,
    }
}

fn hash_headline(headline: &str, symbol: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    symbol.to_uppercase().hash(&mut hasher);
    headline.to_lowercase().hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct CacheEntry {
    score: LlmScore,
    timestamp: u64,
}

static SCORED_CACHE: LazyLock<Mutex<HashMap<u64, CacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn score_with_llm(
    client: &reqwest::Client,
    headline: &str,
    summary: &str,
    symbol: &str,
    api_key: &str,
    provider: &str,
) -> Option<LlmScore> {
    if api_key.is_empty() {
        return None;
    }

    let h = hash_headline(headline, symbol);
    if let Ok(cache) = SCORED_CACHE.lock() {
        if let Some(entry) = cache.get(&h) {
            return Some(entry.score.clone());
        }
    }

    let prompt = format!(
        "Rate this financial news for {} on a scale of -1.0 (very bearish) to +1.0 (very bullish).\n\
         Headline: {}\n\
         Summary: {}\n\n\
         Return ONLY valid JSON: {{\"score\": <float>, \"reasoning\": \"<1 sentence>\", \"confidence\": <0.0-1.0>}}",
        symbol,
        headline,
        if summary.len() > 300 { &summary[..300] } else { summary }
    );

    let result = match provider.to_lowercase().as_str() {
        "anthropic" | "claude" => call_anthropic(client, api_key, &prompt).await,
        _ => call_openai(client, api_key, &prompt).await,
    };

    if let Some(ref score) = result {
        let mut timestamped = score.clone();
        timestamped.timestamp_secs = now_secs();
        timestamped.decay_factor = 1.0;
        if let Ok(mut cache) = SCORED_CACHE.lock() {
            if cache.len() > 500 {
                cache.clear();
            }
            cache.insert(
                h,
                CacheEntry {
                    score: timestamped.clone(),
                    timestamp: now_secs(),
                },
            );
        }
        return Some(timestamped);
    }
    result
}

#[allow(dead_code)]
pub async fn score_batch_with_llm(
    client: &reqwest::Client,
    headlines: &[(String, String, String)],
    api_key: &str,
    provider: &str,
) -> Vec<(String, Option<LlmScore>)> {
    if api_key.is_empty() || headlines.is_empty() {
        return headlines
            .iter()
            .map(|(h, _, _)| (h.clone(), None))
            .collect();
    }

    let mut results = Vec::with_capacity(headlines.len());
    let batch_prompt = headlines
        .iter()
        .enumerate()
        .map(|(i, (headline, summary, symbol))| {
            let sum_trunc = if summary.len() > 200 {
                &summary[..200]
            } else {
                summary.as_str()
            };
            format!(
                "{}. [{}] {}: {}",
                i + 1,
                symbol,
                headline,
                sum_trunc
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Rate each financial news item on -1.0 (bearish) to +1.0 (bullish).\n\n\
         {}\n\n\
         Return ONLY a JSON array: [{{\"score\": <float>, \"reasoning\": \"<brief>\", \"confidence\": <0-1>}}, ...]",
        batch_prompt
    );

    let raw = match provider.to_lowercase().as_str() {
        "anthropic" | "claude" => call_anthropic_raw(client, api_key, &prompt).await,
        _ => call_openai_raw(client, api_key, &prompt).await,
    };

    if let Some(text) = raw {
        if let Some(parsed) = parse_batch_json(&text, headlines.len()) {
            for (i, score) in parsed.into_iter().enumerate() {
                let sym = &headlines[i].2;
                let headline = &headlines[i].0;
                if let Some(ref s) = score {
                    let mut timestamped = s.clone();
                    timestamped.timestamp_secs = now_secs();
                    timestamped.decay_factor = 1.0;
                    let h = hash_headline(headline, sym);
                    if let Ok(mut cache) = SCORED_CACHE.lock() {
                        cache.insert(
                            h,
                            CacheEntry {
                                score: timestamped.clone(),
                                timestamp: now_secs(),
                            },
                        );
                    }
                    results.push((headline.clone(), Some(timestamped)));
                } else {
                    results.push((headline.clone(), None));
                }
            }
            return results;
        }
    }

    for (headline, summary, symbol) in headlines {
        let s =
            score_with_llm(client, headline, summary, symbol, api_key, provider)
                .await;
        results.push((headline.clone(), s));
    }
    results
}

async fn call_openai(client: &reqwest::Client, api_key: &str, prompt: &str) -> Option<LlmScore> {
    let payload = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You are a financial sentiment analyst. Return only JSON."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1,
        "max_tokens": 150,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let content = body
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()?;

    parse_llm_json(content)
}

async fn call_anthropic(client: &reqwest::Client, api_key: &str, prompt: &str) -> Option<LlmScore> {
    let payload = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 150,
        "messages": [
            {"role": "user", "content": prompt}
        ],
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let content = body.get("content")?.get(0)?.get("text")?.as_str()?;

    parse_llm_json(content)
}

async fn call_openai_raw(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
) -> Option<String> {
    let payload = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You are a financial sentiment analyst. Return only JSON."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1,
        "max_tokens": 600,
    });
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

async fn call_anthropic_raw(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
) -> Option<String> {
    let payload = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 600,
        "messages": [{"role": "user", "content": prompt}],
    });
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("content")?.get(0)?.get("text")?.as_str().map(|s| s.to_string())
}

fn parse_batch_json(text: &str, expected: usize) -> Option<Vec<Option<LlmScore>>> {
    let trimmed = text.trim();
    let start = trimmed.find('[')?;
    let end = trimmed.rfind(']')?;
    let arr_str = &trimmed[start..=end];
    let arr: Vec<serde_json::Value> = serde_json::from_str(arr_str).ok()?;
    if arr.len() != expected {
        return None;
    }
    Some(
        arr.into_iter()
            .map(|v| {
                let score = v.get("score")?.as_f64()?.clamp(-1.0, 1.0);
                let reasoning = v
                    .get("reasoning")
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string();
                let confidence = v
                    .get("confidence")
                    .and_then(|c| c.as_f64())
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0);
                Some(LlmScore {
                    score,
                    reasoning,
                    confidence,
                    timestamp_secs: 0,
                    decay_factor: 1.0,
                })
            })
            .collect(),
    )
}

fn parse_llm_json(text: &str) -> Option<LlmScore> {
    let trimmed = text.trim();
    let json_str = if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            &trimmed[start..=end]
        } else {
            return None;
        }
    } else {
        return None;
    };

    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let score = v.get("score")?.as_f64()?.clamp(-1.0, 1.0);
    let reasoning = v
        .get("reasoning")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();
    let confidence = v
        .get("confidence")
        .and_then(|c| c.as_f64())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    Some(LlmScore {
        score,
        reasoning,
        confidence,
        timestamp_secs: 0,
        decay_factor: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_halves_confidence_at_halflife() {
        let score = LlmScore {
            score: 0.8,
            reasoning: "test".into(),
            confidence: 1.0,
            timestamp_secs: now_secs() - 3600,
            decay_factor: 1.0,
        };
        let decayed = apply_decay(&score, 3600);
        assert!((decayed.confidence - 0.5).abs() < 0.05);
        assert!((decayed.decay_factor - 0.5).abs() < 0.05);
    }

    #[test]
    fn decay_zero_halflife_no_change() {
        let score = LlmScore {
            score: 0.5,
            reasoning: "test".into(),
            confidence: 0.9,
            timestamp_secs: now_secs() - 7200,
            decay_factor: 1.0,
        };
        let decayed = apply_decay(&score, 0);
        assert!((decayed.confidence - 0.9).abs() < 0.001);
    }

    #[test]
    fn parse_batch_valid() {
        let json = r#"[{"score": 0.5, "reasoning": "good", "confidence": 0.8}, {"score": -0.3, "reasoning": "bad", "confidence": 0.6}]"#;
        let result = parse_batch_json(json, 2);
        assert!(result.is_some());
        let scores = result.unwrap();
        assert_eq!(scores.len(), 2);
        assert!((scores[0].as_ref().unwrap().score - 0.5).abs() < 0.001);
        assert!((scores[1].as_ref().unwrap().score - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn parse_batch_wrong_count() {
        let json = r#"[{"score": 0.5, "reasoning": "x", "confidence": 0.8}]"#;
        assert!(parse_batch_json(json, 2).is_none());
    }

    #[test]
    fn llm_score_serde_roundtrip() {
        let score = LlmScore {
            score: 0.75,
            reasoning: "bullish".into(),
            confidence: 0.9,
            timestamp_secs: 1700000000,
            decay_factor: 0.85,
        };
        let json = serde_json::to_string(&score).unwrap();
        let restored: LlmScore = serde_json::from_str(&json).unwrap();
        assert!((restored.score - 0.75).abs() < 0.001);
        assert_eq!(restored.timestamp_secs, 1700000000);
    }
}

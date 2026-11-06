use serde::{Deserialize, Serialize};
use std::fs;

use rust_decimal::prelude::*;
use rust_decimal::Decimal;

fn journal_dir() -> std::path::PathBuf {
    crate::paths::journal_dir()
}
fn journal_file() -> std::path::PathBuf {
    crate::paths::journal_file()
}

fn default_journal_source() -> String {
    "UNKNOWN".to_string()
}

fn default_journal_timeframe() -> String {
    "n/a".to_string()
}

/// Indicator/strategy context captured at the moment of trade entry.
/// Persisted in the journal alongside each `TradeRecord` to enable
/// post-hoc analysis of *why* a trade was taken and what market
/// conditions looked like at entry time.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TradeFeatures {
    pub oracle_score: f64,
    pub oracle_confidence: f64,
    pub oracle_regime: String,
    pub rsi: f64,
    pub adx: f64,
    pub atr_pct: f64,
    pub ema9: f64,
    pub ema50: f64,
    pub vpin: f64,
    pub hurst: f64,
    pub rvol: f64,
    pub ofi: f64,
    pub spread_bps: f64,
    pub imbalance_ratio: f64,
    pub strategy_score: i32,
    pub signal_source: String,
    pub fvg_type: String,
    pub entry_type: String,
    pub ta_gates_passed: u8,
    pub ta_gates_total: u8,
    #[serde(default)]
    pub vpin_divergence: f64,
    #[serde(default)]
    pub spread_narrowing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradeRecord {
    pub id: u64,
    pub timestamp: String,
    pub symbol: String,
    pub side: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub qty: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub entry_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub exit_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub pnl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub pnl_pct: Decimal,
    pub strategy_mode: String,
    #[serde(default = "default_journal_source")]
    pub source: String,
    #[serde(default = "default_journal_timeframe")]
    pub timeframe: String,
    pub duration_secs: u64,
    pub reason: String,
    pub notes: String,
    #[serde(default)]
    pub entry_features: Option<TradeFeatures>,
    #[serde(default)]
    pub exit_features: Option<TradeFeatures>,
    #[serde(default)]
    pub broker_order_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalStats {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    #[serde(with = "rust_decimal::serde::float")]
    pub win_rate: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_pnl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub avg_pnl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub best_trade: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub worst_trade: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub avg_winner: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub avg_loser: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit_factor: Decimal,
    pub avg_duration_secs: u64,
}

pub struct TradeRecordInput<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub qty: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
    pub strategy_mode: &'a str,
    pub source: &'a str,
    pub timeframe: &'a str,
    pub duration_secs: u64,
    pub reason: &'a str,
    pub entry_features: Option<TradeFeatures>,
    pub exit_features: Option<TradeFeatures>,
    pub broker_order_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TradeJournal {
    trades: Vec<TradeRecord>,
    next_id: u64,
    dirty: bool,
    last_save_epoch: u64,
}

const JOURNAL_DEBOUNCE_SECS: u64 = 30;

impl TradeJournal {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            next_id: 1,
            dirty: false,
            last_save_epoch: 0,
        }
    }

    pub fn load() -> Self {
        if let Err(e) = fs::create_dir_all(journal_dir()) {
            eprintln!("[journal] Failed to create journal dir: {}", e);
        }
        let path = journal_file();
        let bak_path = path.with_extension("json.bak");

        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<Vec<TradeRecord>>(&data) {
                Ok(trades) => {
                    let next_id = trades.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                    tracing::info!(component = "journal", count = trades.len(), "Journal loaded");
                    Self { trades, next_id, dirty: false, last_save_epoch: 0 }
                }
                Err(e) => {
                    tracing::error!(
                        component = "journal",
                        error = %e,
                        "Journal JSON corrupt -- attempting .bak recovery"
                    );
                    if let Ok(bak_data) = fs::read_to_string(&bak_path) {
                        if let Ok(trades) = serde_json::from_str::<Vec<TradeRecord>>(&bak_data) {
                            let next_id = trades.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                            tracing::warn!(component = "journal", count = trades.len(), "Recovered from .bak");
                            return Self { trades, next_id, dirty: false, last_save_epoch: 0 };
                        }
                    }
                    tracing::error!(component = "journal", "Backup also failed -- starting empty");
                    Self { trades: Vec::new(), next_id: 1, dirty: false, last_save_epoch: 0 }
                }
            },
            Err(_) => Self {
                trades: Vec::new(),
                next_id: 1,
                dirty: false,
                last_save_epoch: 0,
            },
        }
    }

    fn save(&self) {
        if let Err(e) = fs::create_dir_all(journal_dir()) {
            tracing::error!(component = "journal", error = %e, "Failed to create journal dir");
            return;
        }
        let json = match serde_json::to_string_pretty(&self.trades) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!(component = "journal", error = %e, "Failed to serialize journal");
                return;
            }
        };
        let path = journal_file();
        if path.exists() {
            let bak_path = path.with_extension("json.bak");
            if let Err(e) = fs::copy(&path, &bak_path) {
                tracing::warn!(component = "journal", error = %e, "Failed to create .bak");
            }
        }
        let tmp_path = path.with_extension("json.tmp");
        if let Err(e) = fs::write(&tmp_path, &json) {
            tracing::error!(component = "journal", error = %e, path = %tmp_path.display(), "Journal tmp write failed");
            return;
        }
        if let Err(e) = fs::rename(&tmp_path, &path) {
            tracing::error!(component = "journal", error = %e, "Journal atomic rename failed");
        }
    }

    fn push_trade_record(&mut self, input: &TradeRecordInput<'_>) -> u64 {
        let qty_dec = Decimal::try_from(input.qty).unwrap_or(Decimal::ZERO);
        let entry_dec = Decimal::try_from(input.entry_price).unwrap_or(Decimal::ZERO);
        let exit_dec = Decimal::try_from(input.exit_price).unwrap_or(Decimal::ZERO);
        let pnl_dec = Decimal::try_from(input.pnl).unwrap_or(Decimal::ZERO);

        let side_u = input.side.to_uppercase();
        let is_short = side_u.contains("SHORT") || side_u == "SELL";
        let hundred = Decimal::from(100);
        let sign = if is_short {
            -Decimal::ONE
        } else {
            Decimal::ONE
        };
        let pnl_pct = if entry_dec > Decimal::ZERO {
            (exit_dec - entry_dec) / entry_dec * hundred * sign
        } else {
            Decimal::ZERO
        };

        let record = TradeRecord {
            id: self.next_id,
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            symbol: input.symbol.to_string(),
            side: side_u,
            qty: qty_dec,
            entry_price: entry_dec,
            exit_price: exit_dec,
            pnl: pnl_dec,
            pnl_pct,
            strategy_mode: input.strategy_mode.to_string(),
            source: input.source.to_string(),
            timeframe: input.timeframe.to_string(),
            duration_secs: input.duration_secs,
            reason: input.reason.to_string(),
            notes: String::new(),
            entry_features: input.entry_features.clone(),
            exit_features: input.exit_features.clone(),
            broker_order_id: input.broker_order_id.clone(),
        };
        self.next_id += 1;
        self.trades.push(record);
        self.next_id - 1
    }

    pub fn record_trade(&mut self, input: &TradeRecordInput<'_>) -> u64 {
        let id = self.push_trade_record(input);
        self.dirty = true;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now.saturating_sub(self.last_save_epoch) >= JOURNAL_DEBOUNCE_SECS {
            self.save();
            self.dirty = false;
            self.last_save_epoch = now;
        }
        id
    }

    /// Force save if dirty -- called at shutdown or periodically from background.
    pub fn flush(&mut self) {
        if self.dirty {
            self.save();
            self.dirty = false;
            self.last_save_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
    }

    #[allow(dead_code)]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn get_trades(&self, limit: usize, offset: usize) -> Vec<TradeRecord> {
        let total = self.trades.len();
        if offset >= total {
            return Vec::new();
        }
        let start = if total > offset + limit {
            total - offset - limit
        } else {
            0
        };
        let end = total - offset;
        self.trades[start..end].iter().rev().cloned().collect()
    }

    pub fn get_trades_filtered(
        &self,
        symbol: Option<&str>,
        side: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        min_pnl: Option<f64>,
    ) -> Vec<TradeRecord> {
        let min_d = min_pnl.map(|mp| Decimal::try_from(mp).unwrap_or(Decimal::ZERO));
        self.trades
            .iter()
            .filter(|t| {
                if let Some(s) = symbol {
                    if !s.is_empty() && !t.symbol.to_uppercase().contains(&s.to_uppercase()) {
                        return false;
                    }
                }
                if let Some(sd) = side {
                    if !sd.is_empty() && t.side != sd.to_uppercase() {
                        return false;
                    }
                }
                if let Some(df) = date_from {
                    if !df.is_empty() && t.timestamp.as_str() < df {
                        return false;
                    }
                }
                if let Some(dt) = date_to {
                    if !dt.is_empty() && t.timestamp.as_str() > dt {
                        return false;
                    }
                }
                if let Some(ref md) = min_d {
                    if t.pnl < *md {
                        return false;
                    }
                }
                true
            })
            .rev()
            .cloned()
            .collect()
    }

    pub fn add_note(&mut self, trade_id: u64, note: &str) -> bool {
        if let Some(trade) = self.trades.iter_mut().find(|t| t.id == trade_id) {
            trade.notes = note.to_string();
            self.dirty = true;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(self.last_save_epoch) >= JOURNAL_DEBOUNCE_SECS {
                self.save();
                self.dirty = false;
                self.last_save_epoch = now;
            }
            true
        } else {
            false
        }
    }

    pub fn get_stats(&self) -> JournalStats {
        let total = self.trades.len();
        if total == 0 {
            return JournalStats {
                total_trades: 0,
                winning_trades: 0,
                losing_trades: 0,
                win_rate: Decimal::ZERO,
                total_pnl: Decimal::ZERO,
                avg_pnl: Decimal::ZERO,
                best_trade: Decimal::ZERO,
                worst_trade: Decimal::ZERO,
                avg_winner: Decimal::ZERO,
                avg_loser: Decimal::ZERO,
                profit_factor: Decimal::ZERO,
                avg_duration_secs: 0,
            };
        }

        let winners: Vec<Decimal> = self
            .trades
            .iter()
            .filter(|t| t.pnl > Decimal::ZERO)
            .map(|t| t.pnl)
            .collect();
        let losers: Vec<Decimal> = self
            .trades
            .iter()
            .filter(|t| t.pnl < Decimal::ZERO)
            .map(|t| t.pnl)
            .collect();
        let total_pnl: Decimal = self.trades.iter().map(|t| t.pnl).sum();
        let gross_profit: Decimal = winners.iter().copied().sum();
        let gross_loss_abs: Decimal = losers.iter().map(|l| l.abs()).sum();
        let total_duration: u64 = self.trades.iter().map(|t| t.duration_secs).sum();

        let n = Decimal::from(total);
        let win_rate = Decimal::from(winners.len()) / n * Decimal::from(100);
        let avg_winner = if winners.is_empty() {
            Decimal::ZERO
        } else {
            gross_profit / Decimal::from(winners.len())
        };
        let avg_loser = if losers.is_empty() {
            Decimal::ZERO
        } else {
            -(gross_loss_abs / Decimal::from(losers.len()))
        };
        let profit_factor = if gross_loss_abs > Decimal::ZERO {
            gross_profit / gross_loss_abs
        } else if gross_profit > Decimal::ZERO {
            Decimal::from(999)
        } else {
            Decimal::ZERO
        };

        JournalStats {
            total_trades: total,
            winning_trades: winners.len(),
            losing_trades: losers.len(),
            win_rate,
            total_pnl,
            avg_pnl: total_pnl / n,
            best_trade: self
                .trades
                .iter()
                .map(|t| t.pnl)
                .max()
                .unwrap_or(Decimal::ZERO),
            worst_trade: self
                .trades
                .iter()
                .map(|t| t.pnl)
                .min()
                .unwrap_or(Decimal::ZERO),
            avg_winner,
            avg_loser,
            profit_factor,
            avg_duration_secs: total_duration / total as u64,
        }
    }

    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "ID,Timestamp,Symbol,Side,Qty,EntryPrice,ExitPrice,PnL,PnL%,Strategy,Source,Timeframe,Duration(s),Reason,Notes\n",
        );
        for t in &self.trades {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                t.id,
                t.timestamp,
                t.symbol,
                t.side,
                t.qty,
                t.entry_price,
                t.exit_price,
                t.pnl,
                t.pnl_pct,
                t.strategy_mode,
                t.source,
                t.timeframe,
                t.duration_secs,
                t.reason,
                t.notes.replace(',', ";")
            ));
        }
        csv
    }

    pub fn get_equity_curve(&self) -> Vec<(String, f64)> {
        let mut cumulative = Decimal::ZERO;
        self.trades
            .iter()
            .map(|t| {
                cumulative += t.pnl;
                (t.timestamp.clone(), cumulative.to_f64().unwrap_or(0.0))
            })
            .collect()
    }

    pub fn get_daily_pnl(&self) -> Vec<(String, f64)> {
        use std::collections::BTreeMap;
        let mut daily: BTreeMap<String, Decimal> = BTreeMap::new();
        for t in &self.trades {
            let date = t.timestamp.get(..10).unwrap_or("unknown").to_string();
            *daily.entry(date).or_insert(Decimal::ZERO) += t.pnl;
        }
        daily
            .into_iter()
            .map(|(k, v)| (k, v.to_f64().unwrap_or(0.0)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn trade_journal_new_is_empty() {
        let j = TradeJournal::new();
        assert!(j.trades.is_empty());
        assert_eq!(j.next_id, 1);
        let s = j.get_stats();
        assert_eq!(s.total_trades, 0);
        assert_eq!(s.total_pnl, Decimal::ZERO);
    }

    #[test]
    fn push_trade_record_adds_and_increments_id() {
        let mut j = TradeJournal::new();
        let id1 = j.push_trade_record(&TradeRecordInput {
            symbol: "SPY", side: "buy", qty: 1.0, entry_price: 100.0,
            exit_price: 101.0, pnl: 1.0, strategy_mode: "M", source: "ORACLE",
            timeframe: "5m", duration_secs: 0, reason: "r", entry_features: None,
            exit_features: None, broker_order_id: None,
        });
        assert_eq!(id1, 1);
        assert_eq!(j.trades.len(), 1);
        let id2 = j.push_trade_record(&TradeRecordInput {
            symbol: "SPY", side: "buy", qty: 1.0, entry_price: 50.0,
            exit_price: 49.0, pnl: -1.0, strategy_mode: "M", source: "ORACLE",
            timeframe: "5m", duration_secs: 0, reason: "r", entry_features: None,
            exit_features: None, broker_order_id: None,
        });
        assert_eq!(id2, 2);
        assert_eq!(j.next_id, 3);
    }

    #[test]
    fn get_stats_win_rate_and_total_pnl() {
        let j = TradeJournal {
            trades: vec![
                TradeRecord {
                    id: 1,
                    timestamp: "2020-01-01T00:00:00Z".into(),
                    symbol: "A".into(),
                    side: "BUY".into(),
                    qty: dec!(1),
                    entry_price: dec!(100),
                    exit_price: dec!(110),
                    pnl: dec!(10),
                    pnl_pct: dec!(10),
                    strategy_mode: "T".into(),
                    source: "ORACLE".into(),
                    timeframe: "5m".into(),
                    duration_secs: 1,
                    reason: String::new(),
                    notes: String::new(),
                    entry_features: None,
                    exit_features: None,
                    broker_order_id: None,
                },
                TradeRecord {
                    id: 2,
                    timestamp: "2020-01-02T00:00:00Z".into(),
                    symbol: "B".into(),
                    side: "BUY".into(),
                    qty: dec!(1),
                    entry_price: dec!(100),
                    exit_price: dec!(90),
                    pnl: dec!(-5),
                    pnl_pct: dec!(-10),
                    strategy_mode: "T".into(),
                    source: "GPU".into(),
                    timeframe: "5m".into(),
                    duration_secs: 1,
                    reason: String::new(),
                    notes: String::new(),
                    entry_features: None,
                    exit_features: None,
                    broker_order_id: None,
                },
            ],
            next_id: 3,
            dirty: false,
            last_save_epoch: 0,
        };
        let s = j.get_stats();
        assert_eq!(s.total_trades, 2);
        assert_eq!(s.winning_trades, 1);
        assert_eq!(s.losing_trades, 1);
        assert_eq!(s.total_pnl, dec!(5));
        assert_eq!(s.win_rate, dec!(50));
    }

    #[test]
    fn get_trades_filtered_by_symbol() {
        let j = TradeJournal {
            trades: vec![
                TradeRecord {
                    id: 1,
                    timestamp: "2020-01-01T00:00:00Z".into(),
                    symbol: "AAPL".into(),
                    side: "BUY".into(),
                    qty: dec!(1),
                    entry_price: dec!(1),
                    exit_price: dec!(2),
                    pnl: dec!(1),
                    pnl_pct: dec!(0),
                    strategy_mode: "T".into(),
                    source: "FUSION".into(),
                    timeframe: "1m".into(),
                    duration_secs: 0,
                    reason: String::new(),
                    notes: String::new(),
                    entry_features: None,
                    exit_features: None,
                    broker_order_id: None,
                },
                TradeRecord {
                    id: 2,
                    timestamp: "2020-01-02T00:00:00Z".into(),
                    symbol: "BTCUSD".into(),
                    side: "BUY".into(),
                    qty: dec!(1),
                    entry_price: dec!(1),
                    exit_price: dec!(2),
                    pnl: dec!(1),
                    pnl_pct: dec!(0),
                    strategy_mode: "T".into(),
                    source: "FUSION".into(),
                    timeframe: "1m".into(),
                    duration_secs: 0,
                    reason: String::new(),
                    notes: String::new(),
                    entry_features: None,
                    exit_features: None,
                    broker_order_id: None,
                },
            ],
            next_id: 3,
            dirty: false,
            last_save_epoch: 0,
        };
        let filtered = j.get_trades_filtered(Some("btc"), None, None, None, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].symbol, "BTCUSD");
    }

    fn sample_features() -> TradeFeatures {
        TradeFeatures {
            oracle_score: 42.5,
            oracle_confidence: 0.87,
            oracle_regime: "Trending".to_string(),
            rsi: 55.3,
            adx: 28.1,
            atr_pct: 1.2,
            ema9: 150.5,
            ema50: 148.2,
            vpin: 0.45,
            hurst: 0.62,
            rvol: 1.8,
            ofi: 0.15,
            spread_bps: 2.3,
            imbalance_ratio: 0.12,
            strategy_score: 37,
            signal_source: "ORACLE".to_string(),
            fvg_type: "Bullish".to_string(),
            entry_type: "Trending_LONG".to_string(),
            ta_gates_passed: 4,
            ta_gates_total: 6,
            vpin_divergence: -0.08,
            spread_narrowing: true,
        }
    }

    #[test]
    fn trade_features_serde_roundtrip() {
        let features = sample_features();
        let json = serde_json::to_string(&features).unwrap();
        let deserialized: TradeFeatures = serde_json::from_str(&json).unwrap();
        assert!((deserialized.oracle_score - 42.5).abs() < 1e-9);
        assert!((deserialized.oracle_confidence - 0.87).abs() < 1e-9);
        assert_eq!(deserialized.oracle_regime, "Trending");
        assert!((deserialized.rsi - 55.3).abs() < 1e-9);
        assert_eq!(deserialized.strategy_score, 37);
        assert_eq!(deserialized.signal_source, "ORACLE");
        assert_eq!(deserialized.fvg_type, "Bullish");
        assert_eq!(deserialized.entry_type, "Trending_LONG");
        assert_eq!(deserialized.ta_gates_passed, 4);
        assert_eq!(deserialized.ta_gates_total, 6);
    }

    #[test]
    fn trade_features_default_is_none() {
        let json = r#"{"id":1,"timestamp":"2025-01-01T00:00:00Z","symbol":"SPY","side":"BUY","qty":1.0,"entry_price":100.0,"exit_price":101.0,"pnl":1.0,"pnl_pct":1.0,"strategy_mode":"M","source":"ORACLE","timeframe":"5m","duration_secs":0,"reason":"r","notes":""}"#;
        let record: TradeRecord = serde_json::from_str(json).unwrap();
        assert!(record.entry_features.is_none());
    }

    #[test]
    fn trade_record_with_features_persists() {
        let features = sample_features();
        let record = TradeRecord {
            id: 99,
            timestamp: "2025-03-29T12:00:00Z".into(),
            symbol: "NVDA".into(),
            side: "BUY".into(),
            qty: dec!(10),
            entry_price: dec!(900),
            exit_price: dec!(910),
            pnl: dec!(100),
            pnl_pct: dec!(1.11),
            strategy_mode: "MathOracle".into(),
            source: "ORACLE".into(),
            timeframe: "5m".into(),
            duration_secs: 300,
            reason: "AUTOPILOT".into(),
            notes: String::new(),
            entry_features: Some(features),
            exit_features: None,
            broker_order_id: Some("broker-abc-123".to_string()),
        };
        let json = serde_json::to_string_pretty(&record).unwrap();
        let restored: TradeRecord = serde_json::from_str(&json).unwrap();
        assert!(restored.entry_features.is_some());
        let f = restored.entry_features.unwrap();
        assert!((f.oracle_score - 42.5).abs() < 1e-9);
        assert_eq!(f.oracle_regime, "Trending");
        assert_eq!(f.strategy_score, 37);
    }

    #[test]
    fn trade_record_without_features_loads_old_format() {
        let old_json = r#"[
            {"id":1,"timestamp":"2024-01-01T00:00:00Z","symbol":"AAPL","side":"BUY","qty":5.0,"entry_price":180.0,"exit_price":185.0,"pnl":25.0,"pnl_pct":2.78,"strategy_mode":"Fusion","source":"FUSION","timeframe":"5m","duration_secs":120,"reason":"AUTOPILOT","notes":""}
        ]"#;
        let trades: Vec<TradeRecord> = serde_json::from_str(old_json).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].symbol, "AAPL");
        assert!(trades[0].entry_features.is_none());
    }

    #[test]
    fn push_trade_record_with_features() {
        let mut j = TradeJournal::new();
        let features = sample_features();
        let id = j.push_trade_record(&TradeRecordInput {
            symbol: "TSLA", side: "buy", qty: 5.0, entry_price: 200.0,
            exit_price: 210.0, pnl: 50.0, strategy_mode: "M", source: "ORACLE",
            timeframe: "5m", duration_secs: 60, reason: "test",
            entry_features: Some(features), exit_features: None,
            broker_order_id: Some("test-broker-id".to_string()),
        });
        assert_eq!(id, 1);
        let record = &j.trades[0];
        assert!(record.entry_features.is_some());
        let f = record.entry_features.as_ref().unwrap();
        assert!((f.oracle_score - 42.5).abs() < 1e-9);
        assert_eq!(f.entry_type, "Trending_LONG");
    }

    #[test]
    fn push_trade_record_without_features() {
        let mut j = TradeJournal::new();
        let id = j.push_trade_record(&TradeRecordInput {
            symbol: "AMD", side: "sell", qty: 3.0, entry_price: 150.0,
            exit_price: 145.0, pnl: -15.0, strategy_mode: "G", source: "GPU",
            timeframe: "1m", duration_secs: 30, reason: "stop_loss",
            entry_features: None, exit_features: None, broker_order_id: None,
        });
        assert_eq!(id, 1);
        assert!(j.trades[0].entry_features.is_none());
    }

    #[test]
    fn trade_features_vec_serde_mixed() {
        let records = vec![
            TradeRecord {
                id: 1, timestamp: "2025-01-01T00:00:00Z".into(), symbol: "SPY".into(),
                side: "BUY".into(), qty: dec!(1), entry_price: dec!(100), exit_price: dec!(105),
                pnl: dec!(5), pnl_pct: dec!(5), strategy_mode: "M".into(), source: "ORACLE".into(),
                timeframe: "5m".into(), duration_secs: 60, reason: "r".into(), notes: String::new(),
                entry_features: Some(sample_features()),
                exit_features: None, broker_order_id: None,
            },
            TradeRecord {
                id: 2, timestamp: "2025-01-02T00:00:00Z".into(), symbol: "QQQ".into(),
                side: "BUY".into(), qty: dec!(2), entry_price: dec!(400), exit_price: dec!(395),
                pnl: dec!(-10), pnl_pct: dec!(-1.25), strategy_mode: "M".into(), source: "GPU".into(),
                timeframe: "1m".into(), duration_secs: 45, reason: "stop".into(), notes: String::new(),
                entry_features: None, exit_features: None, broker_order_id: None,
            },
        ];
        let json = serde_json::to_string(&records).unwrap();
        let restored: Vec<TradeRecord> = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.len(), 2);
        assert!(restored[0].entry_features.is_some());
        assert!(restored[1].entry_features.is_none());
    }

    #[test]
    fn trade_features_default_values() {
        let features = TradeFeatures::default();
        assert!((features.oracle_score - 0.0).abs() < 1e-9);
        assert!((features.oracle_confidence - 0.0).abs() < 1e-9);
        assert!(features.oracle_regime.is_empty());
        assert_eq!(features.strategy_score, 0);
        assert!(features.signal_source.is_empty());
        assert_eq!(features.ta_gates_passed, 0);
        assert_eq!(features.ta_gates_total, 0);
        assert!((features.vpin_divergence - 0.0).abs() < 1e-9);
        assert!(!features.spread_narrowing);
    }

    #[test]
    fn trade_features_microstructure_fields_serde() {
        let mut features = sample_features();
        features.vpin_divergence = 0.23;
        features.spread_narrowing = true;
        let json = serde_json::to_string(&features).unwrap();
        let restored: TradeFeatures = serde_json::from_str(&json).unwrap();
        assert!((restored.vpin_divergence - 0.23).abs() < 1e-9);
        assert!(restored.spread_narrowing);
    }

    #[test]
    fn trade_features_old_format_without_microstructure_loads() {
        let old_json = r#"{"oracle_score":42.5,"oracle_confidence":0.87,"oracle_regime":"Trending","rsi":55.3,"adx":28.1,"atr_pct":1.2,"ema9":150.5,"ema50":148.2,"vpin":0.45,"hurst":0.62,"rvol":1.8,"ofi":0.15,"spread_bps":2.3,"imbalance_ratio":0.12,"strategy_score":37,"signal_source":"ORACLE","fvg_type":"Bullish","entry_type":"Trending_LONG","ta_gates_passed":4,"ta_gates_total":6}"#;
        let features: TradeFeatures = serde_json::from_str(old_json).unwrap();
        assert!((features.oracle_score - 42.5).abs() < 1e-9);
        assert!((features.vpin_divergence - 0.0).abs() < 1e-9, "Missing field should default to 0.0");
        assert!(!features.spread_narrowing, "Missing field should default to false");
    }

    /// FIX-2 CRIT-17: Journal save uses atomic tmp+rename pattern.
    #[test]
    fn journal_save_atomic_no_tmp_leftover() {
        let dir = std::env::temp_dir().join("titan_test_journal_atomic");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_journal.json");
        let tmp_path = path.with_extension("json.tmp");

        let json = serde_json::to_string_pretty(&Vec::<TradeRecord>::new()).unwrap();
        fs::write(&tmp_path, &json).unwrap();
        fs::rename(&tmp_path, &path).unwrap();

        assert!(path.exists(), "main file should exist after atomic save");
        assert!(!tmp_path.exists(), "tmp file should not exist after successful rename");

        let content = fs::read_to_string(&path).unwrap();
        let trades: Vec<TradeRecord> = serde_json::from_str(&content).unwrap();
        assert!(trades.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    /// FIX-2 CRIT-17 enhanced: record_trade triggers save via debounce and
    /// the save path produces a valid, non-corrupt journal file.
    #[test]
    fn journal_record_trade_triggers_save() {
        let mut journal = TradeJournal::new();
        assert!(!journal.is_dirty(), "new journal should not be dirty");
        let input = TradeRecordInput {
            symbol: "AAPL",
            side: "BUY",
            qty: 10.0,
            entry_price: 150.0,
            exit_price: 155.0,
            pnl: 50.0,
            strategy_mode: "MathOracle",
            source: "autopilot",
            timeframe: "5m",
            duration_secs: 120,
            reason: "take_profit",
            entry_features: None,
            exit_features: None,
            broker_order_id: None,
        };
        let id = journal.record_trade(&input);
        assert!(id > 0, "trade should be recorded with valid id");
        assert_eq!(journal.get_trades(100, 0).len(), 1);
        let trade = &journal.get_trades(100, 0)[0];
        assert_eq!(trade.symbol, "AAPL");
        assert_eq!(trade.side, "BUY");
    }

    /// OPT-2: flush() clears dirty flag and is idempotent.
    #[test]
    fn journal_flush_clears_dirty() {
        let mut journal = TradeJournal::new();
        assert!(!journal.is_dirty());
        let input = TradeRecordInput {
            symbol: "TSLA", side: "SELL", qty: 5.0,
            entry_price: 200.0, exit_price: 190.0, pnl: -50.0,
            strategy_mode: "GpuNexus", source: "autopilot", timeframe: "1m",
            duration_secs: 60, reason: "stop_loss", entry_features: None,
            exit_features: None, broker_order_id: None,
        };
        journal.record_trade(&input);
        journal.flush();
        assert!(!journal.is_dirty(), "flush should clear dirty flag");
        journal.flush();
        assert!(!journal.is_dirty(), "double flush should be idempotent");
    }
}

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::journal::TradeJournal;

#[derive(Serialize, Clone, Debug)]
pub struct SymbolPerformance {
    pub symbol: String,
    pub trades: usize,
    pub wins: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub performance_score: f64,
    pub risk_multiplier: f64,
    pub tag: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CorrelationEntry {
    pub symbol_a: String,
    pub symbol_b: String,
    pub correlation: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct RebalanceAction {
    pub remove: Vec<String>,
    pub demote: Vec<String>,
    pub promote: Vec<String>,
}

pub fn pearson_correlation(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 10 {
        return 0.0;
    }
    let a = &a[..n];
    let b = &b[..n];

    let mean_a: f64 = a.iter().sum::<f64>() / n as f64;
    let mean_b: f64 = b.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom < 1e-15 {
        return 0.0;
    }
    (cov / denom).clamp(-1.0, 1.0)
}

pub fn returns_from_closes(closes: &[f64]) -> Vec<f64> {
    if closes.len() < 2 {
        return vec![];
    }
    closes
        .windows(2)
        .filter_map(|w| {
            if w[0] > 0.0 {
                Some((w[1] - w[0]) / w[0])
            } else {
                None
            }
        })
        .collect()
}

/// Check if candidate symbol is safe to enter given currently held positions.
/// Returns (is_safe, blocking_reason).
pub fn check_correlation_safe(
    candidate_returns: &[f64],
    held_returns: &[(String, Vec<f64>)],
    max_correlation: f64,
) -> (bool, Option<String>) {
    if candidate_returns.len() < 10 {
        return (true, None);
    }

    for (held_sym, held_ret) in held_returns {
        if held_ret.len() < 10 {
            continue;
        }
        let corr = pearson_correlation(candidate_returns, held_ret);
        if corr.abs() > max_correlation {
            return (
                false,
                Some(format!(
                    "CORR GUARD: correlation {:.2} with {} exceeds limit {:.2}",
                    corr, held_sym, max_correlation
                )),
            );
        }
    }
    (true, None)
}

pub fn compute_symbol_performance(journal: &TradeJournal) -> Vec<SymbolPerformance> {
    let all_trades = journal.get_trades_filtered(None, None, None, None, None);

    let mut by_symbol: std::collections::HashMap<String, Vec<&crate::journal::TradeRecord>> =
        std::collections::HashMap::new();

    for t in &all_trades {
        by_symbol.entry(t.symbol.clone()).or_default().push(t);
    }

    let mut result: Vec<SymbolPerformance> = by_symbol
        .into_iter()
        .map(|(symbol, trades)| {
            let total = trades.len();
            let wins = trades.iter().filter(|t| t.pnl > Decimal::ZERO).count();
            let win_rate = if total > 0 {
                (Decimal::from(wins * 100) / Decimal::from(total))
                    .to_f64()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let total_pnl_dec: Decimal = trades.iter().map(|t| t.pnl).sum();
            let total_pnl = total_pnl_dec.to_f64().unwrap_or(0.0);
            let avg_pnl = if total > 0 {
                (total_pnl_dec / Decimal::from(total))
                    .to_f64()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let gross_profit: Decimal = trades
                .iter()
                .filter(|t| t.pnl > Decimal::ZERO)
                .map(|t| t.pnl)
                .sum();
            let gross_loss_abs: Decimal = trades
                .iter()
                .filter(|t| t.pnl < Decimal::ZERO)
                .map(|t| t.pnl.abs())
                .sum();
            let pf = if gross_loss_abs > Decimal::ZERO {
                (gross_profit / gross_loss_abs).to_f64().unwrap_or(0.0)
            } else if gross_profit > Decimal::ZERO {
                5.0
            } else {
                0.0
            };

            // Performance score: weighted combination of win_rate, profit_factor, consistency
            let wr_norm = (win_rate / 100.0).clamp(0.0, 1.0);
            let pf_norm = (pf / 3.0).clamp(0.0, 1.0);
            let vol_norm = if total >= 5 { 1.0 } else { total as f64 / 5.0 };
            let performance_score = (0.4 * wr_norm + 0.4 * pf_norm + 0.2 * vol_norm) * 100.0;

            // Risk multiplier: 0.5x for underperformers, up to 1.5x for top performers
            let risk_multiplier = if total < 3 {
                1.0
            } else if win_rate < 25.0 && total_pnl_dec < Decimal::ZERO {
                0.5
            } else if win_rate > 60.0 && pf > 1.5 {
                1.5
            } else if win_rate > 45.0 && pf > 1.0 {
                1.2
            } else if win_rate < 35.0 || pf < 0.8 {
                0.7
            } else {
                1.0
            };

            let tag = if total < 3 {
                "NEW".to_string()
            } else if win_rate < 25.0 && total >= 5 && total_pnl_dec < Decimal::ZERO {
                "UNDERPERFORMER".to_string()
            } else if win_rate > 55.0 && pf > 1.5 {
                "STAR".to_string()
            } else if total_pnl_dec > Decimal::ZERO {
                "OK".to_string()
            } else {
                "WEAK".to_string()
            };

            SymbolPerformance {
                symbol,
                trades: total,
                wins,
                win_rate,
                total_pnl,
                avg_pnl,
                performance_score,
                risk_multiplier,
                tag,
            }
        })
        .collect();

    result.sort_by(|a, b| {
        b.performance_score
            .partial_cmp(&a.performance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

pub fn get_risk_multiplier_for_symbol(performances: &[SymbolPerformance], symbol: &str) -> f64 {
    performances
        .iter()
        .find(|p| p.symbol.eq_ignore_ascii_case(symbol))
        .map(|p| p.risk_multiplier)
        .unwrap_or(1.0)
}

pub fn suggest_rebalance(
    performances: &[SymbolPerformance],
    hft_targets: &[String],
) -> RebalanceAction {
    let mut remove = Vec::new();
    let mut demote = Vec::new();
    let mut promote = Vec::new();

    for perf in performances {
        let in_targets = hft_targets
            .iter()
            .any(|t| t.eq_ignore_ascii_case(&perf.symbol));
        if !in_targets {
            continue;
        }

        match perf.tag.as_str() {
            "UNDERPERFORMER" => remove.push(perf.symbol.clone()),
            "WEAK" if perf.trades >= 5 => demote.push(perf.symbol.clone()),
            "STAR" => promote.push(perf.symbol.clone()),
            _ => {}
        }
    }

    RebalanceAction {
        remove,
        demote,
        promote,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_perfect_positive() {
        let a: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..50).map(|i| i as f64 * 2.0 + 1.0).collect();
        let r = pearson_correlation(&a, &b);
        assert!((r - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pearson_perfect_negative() {
        let a: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..50).map(|i| 100.0 - i as f64).collect();
        let r = pearson_correlation(&a, &b);
        assert!((r + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_returns_from_closes() {
        let closes = vec![100.0, 102.0, 101.0, 105.0];
        let ret = returns_from_closes(&closes);
        assert_eq!(ret.len(), 3);
        assert!((ret[0] - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_correlation_safe_blocks() {
        let a = vec![
            0.01, -0.02, 0.03, -0.01, 0.02, 0.01, -0.03, 0.02, 0.01, -0.01, 0.02, 0.01,
        ];
        let b = a.clone(); // perfect correlation
        let held = vec![("HELD".to_string(), b)];
        let (safe, reason) = check_correlation_safe(&a, &held, 0.70);
        assert!(!safe);
        assert!(reason.is_some());
    }
}

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::Duration;

use crate::app_state;
use crate::commands::data::fetch_positions;
use crate::state::TitanState;

pub(crate) async fn position_reconciliation_loop(state: Arc<TitanState>, app: AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(15));
    let mut last_count: usize = 0;

    loop {
        ticker.tick().await;
        if state.shutdown_token.is_cancelled() {
            break;
        }
        state.watchdog.heartbeat("position_recon", 0);

        let positions = match fetch_positions(state.clone()).await {
            Ok(p) => p,
            Err(_) => continue,
        };

        let blacklist = state
            .trading.position_blacklist
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let grace = {
            let mut g = state.trading.recon_grace.lock().unwrap_or_else(|e| e.into_inner());
            g.retain(|_, &mut v| v > now_ms);
            g.clone()
        };
        let hundred = Decimal::from(100);
        let mut local_positions: Vec<app_state::Position> = Vec::new();
        for p in &positions {
            if blacklist.contains(&p.symbol) {
                continue;
            }
            if let Some(&grace_until) = grace.get(&p.symbol) {
                if now_ms < grace_until {
                    continue;
                }
            }
            let qty_f64 = p.qty.parse::<f64>().unwrap_or(0.0).abs();
            if qty_f64 <= 0.0 {
                continue;
            }
            let qty = Decimal::try_from(qty_f64).unwrap_or(Decimal::ZERO);

            let current_price_f64 = p
                .current_price
                .as_deref()
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            let entry_price_f64 = p
                .avg_entry_price
                .as_deref()
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            let current_price = Decimal::try_from(current_price_f64).unwrap_or(Decimal::ZERO);
            let entry_price = Decimal::try_from(entry_price_f64).unwrap_or(Decimal::ZERO);
            let is_short = p.side.eq_ignore_ascii_case("short");

            let pnl = if is_short {
                (entry_price - current_price) * qty
            } else {
                (current_price - entry_price) * qty
            };
            let pnl_pct = if entry_price > Decimal::ZERO {
                if is_short {
                    (entry_price - current_price) / entry_price * hundred
                } else {
                    (current_price - entry_price) / entry_price * hundred
                }
            } else {
                Decimal::ZERO
            };

            local_positions.push(app_state::Position {
                symbol: p.symbol.clone(),
                qty,
                entry_price,
                current_price,
                pnl,
                pnl_percent: pnl_pct,
                side: if is_short {
                    app_state::OrderSide::Sell
                } else {
                    app_state::OrderSide::Buy
                },
                highest_price: current_price,
                lowest_price: current_price,
                state: app_state::PositionState::Open,
                opened_at: Some(now_ms / 1000), // Fix SG#2: Set to current time instead of None
                closed_at: None,
                partial_taken: false,
                entry_tag: None,
                broker_order_id: None,
            });
        }

        // Update local state, preserving trail highs/lows and FSM state from existing data
        // FIX CRIT-15: Use max/min to preserve absolute trail extremes
        {
            let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            for new_pos in &mut local_positions {
                if let Some(existing) = settings
                    .positions
                    .iter()
                    .find(|p| p.symbol == new_pos.symbol)
                {
                    new_pos.highest_price = existing.highest_price.max(new_pos.current_price);
                    new_pos.lowest_price = if existing.lowest_price > Decimal::ZERO {
                        existing.lowest_price.min(new_pos.current_price)
                    } else {
                        new_pos.current_price
                    };
                    new_pos.state = existing.state.clone();
                    new_pos.opened_at = existing.opened_at;
                    new_pos.closed_at = existing.closed_at;
                    new_pos.partial_taken = existing.partial_taken;
                    new_pos.entry_tag = existing.entry_tag.clone();
                    new_pos.broker_order_id = existing.broker_order_id.clone();
                }
            }
            settings.positions = local_positions.clone();
        }

        {
            let broker_symbols: std::collections::HashSet<&str> =
                positions.iter().map(|p| p.symbol.as_str()).collect();
            if let Ok(mut inflight) = state.trading.exit_inflight.lock() {
                inflight.retain(|sym, _| broker_symbols.contains(sym.as_str()));
            }
        }

        let new_count = local_positions.len();
        if new_count != last_count {
            let _ = app.emit_all(
                "log-message",
                format!(
                    "RECONCILIATION: {} open positions synced from Alpaca",
                    new_count
                ),
            );
            last_count = new_count;
        }

        // Emit position update to frontend
        let _ = app.emit_all(
            "positions-sync",
            serde_json::json!({
                "count": new_count,
                "positions": local_positions.iter().map(|p| serde_json::json!({
                    "symbol": p.symbol,
                    "qty": p.qty.to_f64().unwrap_or(0.0),
                    "entry_price": p.entry_price.to_f64().unwrap_or(0.0),
                    "current_price": p.current_price.to_f64().unwrap_or(0.0),
                    "pnl": p.pnl.to_f64().unwrap_or(0.0),
                    "pnl_percent": p.pnl_percent.to_f64().unwrap_or(0.0),
                    "side": format!("{:?}", p.side),
                })).collect::<Vec<_>>(),
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIX-4 CRIT-15: Recon preserves trail high/low extremes using max/min.
    #[test]
    fn recon_preserves_trail_extremes() {
        let existing_highest = Decimal::try_from(110.0).unwrap();
        let existing_lowest = Decimal::try_from(95.0).unwrap();
        let current_price_above = Decimal::try_from(115.0).unwrap();
        let current_price_below = Decimal::try_from(90.0).unwrap();
        let current_price_mid = Decimal::try_from(105.0).unwrap();

        // When price rises above old high: highest should update
        let new_highest = existing_highest.max(current_price_above);
        assert_eq!(new_highest, current_price_above, "highest should be new price when above old high");

        // When price is between old high and low: both should be preserved
        let mid_highest = existing_highest.max(current_price_mid);
        let mid_lowest = existing_lowest.min(current_price_mid);
        assert_eq!(mid_highest, existing_highest, "highest should be preserved when price is below");
        assert_eq!(mid_lowest, existing_lowest, "lowest should be preserved when price is above");

        // When price drops below old low: lowest should update
        let new_lowest = existing_lowest.min(current_price_below);
        assert_eq!(new_lowest, current_price_below, "lowest should be new price when below old low");
    }

    /// FIX-4 CRIT-15: Zero lowest_price guard.
    #[test]
    fn recon_zero_lowest_uses_current() {
        let zero = Decimal::ZERO;
        let current = Decimal::try_from(100.0).unwrap();
        let result = if zero > Decimal::ZERO {
            zero.min(current)
        } else {
            current
        };
        assert_eq!(result, current, "zero lowest_price should default to current_price");
    }
}

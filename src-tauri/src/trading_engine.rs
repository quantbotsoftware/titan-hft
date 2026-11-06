use crate::app_state::{AppState, OrderSide, Position};
use crate::risk;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

pub struct TradeAction {
    pub symbol: String,
    pub action: String,
    pub reason: String,
    pub price: Decimal,
    pub qty: Decimal,
    pub pnl: Option<Decimal>,
}

pub fn manage_positions(
    state: &mut AppState,
    current_prices: &std::collections::HashMap<String, f64>,
) -> Vec<TradeAction> {
    let pos_count = state.positions.len();
    let mut actions = Vec::with_capacity(pos_count);
    let mut indices_to_remove = Vec::with_capacity(pos_count);

    for (i, position) in state.positions.iter_mut().enumerate() {
        if let Some(&current_price) = current_prices.get(&position.symbol) {
            if !current_price.is_finite() || current_price <= 0.0 {
                continue;
            }
            let current_price_dec = Decimal::try_from(current_price).unwrap_or(Decimal::ZERO);
            let is_long = position.side == OrderSide::Buy;
            let qty = position.qty.abs();
            let entry_price = position.entry_price;

            let mut should_close = false;
            let mut reason = String::new();

            if is_long {
                let pl_pct = if entry_price != Decimal::ZERO {
                    (current_price_dec - entry_price) / entry_price
                } else {
                    Decimal::ZERO
                };

                if current_price_dec > position.highest_price {
                    position.highest_price = current_price_dec;
                }
                let drawdown = if position.highest_price != Decimal::ZERO {
                    (current_price_dec - position.highest_price) / position.highest_price
                } else {
                    Decimal::ZERO
                };

                let sl_threshold: Decimal = if state.aggressive_mode {
                    Decimal::try_from(state.long_stop_aggressive / 100.0).unwrap_or(dec!(-0.08))
                } else {
                    Decimal::try_from(state.long_stop_normal / 100.0).unwrap_or(dec!(-0.025))
                };
                if pl_pct <= sl_threshold {
                    should_close = true;
                    reason = "STOP LOSS".to_string();
                }

                let base_trail = Decimal::try_from(-state.trailing_stop_pct / 100.0).unwrap_or(dec!(-0.02));
                let trail_gap = if state.aggressive_mode {
                    if pl_pct > dec!(0.50) {
                        base_trail * dec!(0.25)
                    } else if pl_pct > dec!(0.20) {
                        base_trail * dec!(0.33)
                    } else if pl_pct > dec!(0.10) {
                        base_trail * dec!(0.50)
                    } else if pl_pct > dec!(0.05) {
                        base_trail * dec!(0.67)
                    } else {
                        base_trail
                    }
                } else if pl_pct > dec!(0.05) {
                    base_trail * dec!(0.50)
                } else {
                    base_trail
                };

                let min_profit = Decimal::try_from(state.long_trail_min_profit / 100.0).unwrap_or(dec!(0.015));
                if pl_pct > min_profit && drawdown <= trail_gap {
                    should_close = true;
                    let tg_f64 = trail_gap.to_f64().unwrap_or(0.0);
                    reason = format!("TRAILING STOP ({:.1}%)", tg_f64 * 100.0);
                }
            } else {
                let pl_pct = if entry_price != Decimal::ZERO {
                    (entry_price - current_price_dec) / entry_price
                } else {
                    Decimal::ZERO
                };

                if current_price_dec < position.lowest_price
                    || position.lowest_price == Decimal::ZERO
                {
                    position.lowest_price = current_price_dec;
                }
                let bounce = if position.lowest_price != Decimal::ZERO {
                    (current_price_dec - position.lowest_price) / position.lowest_price
                } else {
                    Decimal::ZERO
                };

                let sl_threshold = Decimal::try_from(state.short_stop_pct / 100.0).unwrap_or(dec!(-0.05));
                if pl_pct <= sl_threshold {
                    should_close = true;
                    reason = "STOP LOSS (SHORT)".to_string();
                }

                let trail_gap = Decimal::try_from(state.short_trail_gap / 100.0).unwrap_or(dec!(0.015));
                let short_min_profit = Decimal::try_from(state.short_trail_min_profit / 100.0).unwrap_or(dec!(0.01));
                if pl_pct > short_min_profit && bounce >= trail_gap {
                    should_close = true;
                    let tg_f64 = trail_gap.to_f64().unwrap_or(0.0);
                    reason = format!("TRAILING COVER ({:.1}%)", tg_f64 * 100.0);
                }
            }

            if should_close {
                let pnl_dec = if is_long {
                    (current_price_dec - entry_price) * qty
                } else {
                    (entry_price - current_price_dec) * qty
                };

                position.state = crate::app_state::PositionState::Closed;
                position.closed_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );

                actions.push(TradeAction {
                    symbol: position.symbol.clone(),
                    action: if is_long {
                        "SELL".into()
                    } else {
                        "COVER".into()
                    },
                    reason,
                    price: current_price_dec,
                    qty,
                    pnl: Some(pnl_dec),
                });
                indices_to_remove.push(i);
            }
        }
    }

    indices_to_remove.sort_by(|a, b| b.cmp(a));
    for i in indices_to_remove {
        state.positions.remove(i);
    }

    actions
}

pub struct EntryContext {
    pub equity: Decimal,
    pub risk_per_trade_pct: f64,
    pub stop_loss_distance_pct: f64,
    pub breaker_multiplier: Decimal,
    pub aggressive_mode: bool,
    pub max_notional_equity_frac: f64,
    pub min_stop_distance_frac: f64,
}

pub fn execute_entry(
    state: &mut AppState,
    symbol: &str,
    price: f64,
    entry_type: &str,
    is_short: bool,
    ctx: &EntryContext,
) -> Option<TradeAction> {
    if state.positions.iter().any(|p| p.symbol == symbol) {
        return None;
    }

    if state.positions.len() >= state.max_open_positions as usize {
        return None;
    }

    if price <= 0.0 || ctx.equity <= Decimal::ZERO {
        return None;
    }

    let sl_distance_pct = if ctx.stop_loss_distance_pct > 0.0 {
        ctx.stop_loss_distance_pct
    } else if ctx.aggressive_mode {
        8.0
    } else {
        2.5
    };

    let stop_price = if is_short {
        price * (1.0 + sl_distance_pct / 100.0)
    } else {
        price * (1.0 - sl_distance_pct / 100.0)
    };

    let price_dec = Decimal::try_from(price).unwrap_or(Decimal::ZERO);
    let stop_price_dec = Decimal::try_from(stop_price).unwrap_or(Decimal::ZERO);

    let sizing = risk::calculate_position_size(&risk::SizingParams {
        equity: ctx.equity,
        risk_pct: Decimal::try_from(ctx.risk_per_trade_pct).unwrap_or(dec!(2)),
        entry_price: price_dec,
        stop_loss_price: stop_price_dec,
        breaker_multiplier: ctx.breaker_multiplier,
        max_notional_frac: Decimal::try_from(ctx.max_notional_equity_frac).unwrap_or(dec!(0.25)),
        min_stop_frac: Decimal::try_from(ctx.min_stop_distance_frac).unwrap_or(dec!(0.0001)),
    });

    let qty_dec = match sizing {
        Some(s) => s.qty,
        None => return None,
    };

    if qty_dec < dec!(0.0001) {
        return None;
    }

    let side = if is_short {
        OrderSide::Sell
    } else {
        OrderSide::Buy
    };
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let new_pos = Position {
        symbol: symbol.to_string(),
        qty: qty_dec,
        entry_price: price_dec,
        current_price: price_dec,
        pnl: Decimal::ZERO,
        pnl_percent: Decimal::ZERO,
        side,
        highest_price: price_dec,
        lowest_price: if is_short { price_dec } else { Decimal::ZERO },
        state: crate::app_state::PositionState::Open,
        opened_at: Some(now_epoch),
        closed_at: None,
        partial_taken: false,
        entry_tag: Some(entry_type.to_string()),
        broker_order_id: None,
    };

    state.positions.push(new_pos);

    Some(TradeAction {
        symbol: symbol.to_string(),
        action: if is_short {
            "SHORT".into()
        } else {
            "BUY".into()
        },
        reason: format!(
            "{} (Risk: {:.1}%, SL: {:.1}%)",
            entry_type, ctx.risk_per_trade_pct, sl_distance_pct
        ),
        price: price_dec,
        qty: qty_dec,
        pnl: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{AppState, OrderSide, Position};
    use rust_decimal_macros::dec;

    fn sample_ctx() -> EntryContext {
        EntryContext {
            equity: dec!(100_000),
            risk_per_trade_pct: 2.0,
            stop_loss_distance_pct: 2.5,
            breaker_multiplier: dec!(1),
            aggressive_mode: false,
            max_notional_equity_frac: 0.25,
            min_stop_distance_frac: 0.0001,
        }
    }

    #[test]
    fn execute_entry_valid_returns_some_trade_action() {
        let mut state = AppState::default();
        let ctx = sample_ctx();
        let action = execute_entry(&mut state, "TEST", 100.0, "SIGNAL", false, &ctx);
        assert!(action.is_some());
        let a = action.unwrap();
        assert_eq!(a.symbol, "TEST");
        assert_eq!(a.action, "BUY");
        assert!(a.qty > Decimal::ZERO);
    }

    #[test]
    fn execute_entry_effectively_zero_qty_returns_none() {
        let mut state = AppState::default();
        let ctx = EntryContext {
            equity: Decimal::ONE,
            risk_per_trade_pct: 0.00001,
            stop_loss_distance_pct: 2.5,
            breaker_multiplier: dec!(1),
            aggressive_mode: false,
            max_notional_equity_frac: 0.25,
            min_stop_distance_frac: 0.0001,
        };
        assert!(execute_entry(&mut state, "T2", 100.0, "SIGNAL", false, &ctx).is_none());
    }

    #[test]
    fn manage_positions_long_stop_loss_pnl() {
        let mut state = AppState {
            aggressive_mode: false,
            ..AppState::default()
        };
        state.positions.push(Position {
            symbol: "LONG1".into(),
            qty: dec!(10),
            entry_price: dec!(100),
            current_price: dec!(100),
            pnl: dec!(0),
            pnl_percent: dec!(0),
            side: OrderSide::Buy,
            highest_price: dec!(100),
            lowest_price: dec!(0),
            state: crate::app_state::PositionState::Open,
            opened_at: None,
            closed_at: None,
            partial_taken: false,
            entry_tag: None,
            broker_order_id: None,
        });
        let mut prices = std::collections::HashMap::new();
        prices.insert("LONG1".into(), 97.4);
        let actions = manage_positions(&mut state, &prices);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, "SELL");
        assert!((actions[0].pnl.unwrap() - dec!(-26)).abs() < dec!(0.01));
        assert!(state.positions.is_empty());
    }

    #[test]
    fn manage_positions_short_stop_loss_pnl() {
        let mut state = AppState::default();
        state.positions.push(Position {
            symbol: "SHT1".into(),
            qty: dec!(10),
            entry_price: dec!(100),
            current_price: dec!(100),
            pnl: dec!(0),
            pnl_percent: dec!(0),
            side: OrderSide::Sell,
            highest_price: dec!(0),
            lowest_price: dec!(100),
            state: crate::app_state::PositionState::Open,
            opened_at: None,
            closed_at: None,
            partial_taken: false,
            entry_tag: None,
            broker_order_id: None,
        });
        let mut prices = std::collections::HashMap::new();
        prices.insert("SHT1".into(), 106.0);
        let actions = manage_positions(&mut state, &prices);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, "COVER");
        assert!((actions[0].pnl.unwrap() - dec!(-60)).abs() < dec!(0.01));
    }

    #[test]
    fn manage_positions_trailing_stop_long() {
        let mut state = AppState {
            aggressive_mode: false,
            ..AppState::default()
        };
        state.positions.push(Position {
            symbol: "TRL".into(),
            qty: dec!(1),
            entry_price: dec!(100),
            current_price: dec!(100),
            pnl: dec!(0),
            pnl_percent: dec!(0),
            side: OrderSide::Buy,
            highest_price: dec!(100),
            lowest_price: dec!(0),
            state: crate::app_state::PositionState::Open,
            opened_at: None,
            closed_at: None,
            partial_taken: false,
            entry_tag: None,
            broker_order_id: None,
        });
        let mut prices = std::collections::HashMap::new();
        prices.insert("TRL".into(), 115.0);
        let _ = manage_positions(&mut state, &prices);
        assert_eq!(state.positions.len(), 1);
        prices.insert("TRL".into(), 112.0);
        let actions = manage_positions(&mut state, &prices);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].reason.contains("TRAILING"));
        assert_eq!(actions[0].action, "SELL");
    }
}

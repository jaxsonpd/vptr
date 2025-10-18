use std::collections::HashMap;

use crate::data::{MarketData, Order, TradeFill, Symbol, OrderType, Side};

pub struct Broker {
    pub cash: f64,
    pub positions: HashMap<Symbol, f64>,
}

impl Broker {
    pub fn new(cash: f64) -> Self {
        Self {
            cash: cash,
            positions: HashMap::new(),
        }
    }

    pub fn execute_orders(&mut self, orders: &[Order], data: &MarketData) -> Vec<TradeFill> {
        let mut fills = vec![];

        for order in orders {
            if order.symbol != data.symbol {
                continue;
            }

            let fill_price = match order.order_type {
                OrderType::Market => data.price,
                OrderType::Limit(limit) => {
                    // For simplicity, assume we fill if price crosses the limit.
                    match order.side {
                        Side::Buy if data.price <= limit => data.price,
                        Side::Sell if data.price >= limit => data.price,
                        _ => continue,
                    }
                }
            };

            // Update cash + positions
            match order.side {
                Side::Buy => {
                    self.cash -= fill_price * order.qty;
                    *self.positions.entry(order.symbol.clone()).or_insert(0.0) += order.qty;
                }
                Side::Sell => {
                    self.cash += fill_price * order.qty;
                    *self.positions.entry(order.symbol.clone()).or_insert(0.0) -= order.qty;
                }
            }

            fills.push(TradeFill {
                symbol: order.symbol.clone(),
                qty: order.qty,
                price: fill_price,
                side: order.side.clone(),
            });
            println!(
                "[{:?}] {:?} {:?} @ {:.2}, Cash: {}",
                fills[fills.len()-1].symbol, fills[fills.len()-1].side, 
                fills[fills.len()-1].qty, fills[fills.len()-1].price,
                self.cash
            );

        }
        fills
    }
}

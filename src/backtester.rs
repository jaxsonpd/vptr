// The backtesting framework that tests traders

use std::collections::HashMap;
use std::result;

use crate::broker::Broker;
use crate::data::{MarketData, MarketEvent, Symbol, TradeFill, Side};
use crate::trader::Trader;

pub struct Backtester<T: Trader> {
    trader: T,
    broker: Broker,
    market_data: Vec<MarketData>,
}

impl<T: Trader> Backtester<T> {
    pub fn new(trader: T, market_data: Vec<MarketData>, starting_cash: f64) -> Self {
        Self {
            trader,
            broker: Broker::new(starting_cash),
            market_data,
        }
    }

    /// Calculate the pnl for the given list of trades, returns a list of
    /// symbols with the pnl for each symbol per trade fill
    fn calculate_pnl(trades: Vec<TradeFill>) -> Vec<(Symbol, Vec<(TradeFill, f64)>)> {
        let mut results: HashMap<Symbol, Vec<(TradeFill, f64)>> = HashMap::new();
        let mut avg_cost: HashMap<Symbol, f64> = HashMap::new();
        let mut position: HashMap<Symbol, f64> = HashMap::new();

        for trade in trades {
            let sym = trade.symbol.clone();
            let entry_avg = avg_cost.entry(sym.clone()).or_insert(0.0);
            let entry_pos = position.entry(sym.clone()).or_insert(0.0);

            let qty_signed = match trade.side {
                Side::Buy => trade.qty,
                Side::Sell => -trade.qty,
            };

            let mut pnl = 0.0;

            // Case 1: same direction — just adjust position and avg cost
            if entry_pos.signum() == qty_signed.signum() || *entry_pos == 0.0 {
                let new_pos = *entry_pos + qty_signed;

                if new_pos != 0.0 {
                    *entry_avg = ((*entry_avg * *entry_pos) + (trade.price * qty_signed)) / new_pos;
                } else {
                    *entry_avg = 0.0;
                }

                *entry_pos = new_pos;
            } 
            // Case 2: reducing or flipping position
            else {
                let closing_qty = if qty_signed.abs() > entry_pos.abs() {
                    entry_pos.abs()
                } else {
                    qty_signed.abs()
                };

                pnl = (entry_pos.signum()) * (trade.price - *entry_avg) * closing_qty;

                let new_pos = *entry_pos + qty_signed;

                if new_pos.signum() != entry_pos.signum() {
                    *entry_avg = trade.price;
                }

                *entry_pos = new_pos;
            }

            results.entry(sym).or_default().push((trade, pnl));
        }

        results.into_iter().collect()
    }

    fn calculate_cumulative_pnl(pnl_trades: Vec<(Symbol, Vec<(TradeFill, f64)>)>) -> Vec<(Symbol, f64)> {
        let mut results: Vec<(Symbol, f64)> = vec![]; 

        for (symbol, trades) in pnl_trades.iter() {
            let mut total_pnl = 0.0;
            for (_, pnl) in trades {
                total_pnl += pnl;
            }
            results.push((symbol.clone(), total_pnl));
        } 

        results
    }

    pub fn run(&mut self) {
        println!("Starting backtest with cash: {:.2}", self.broker.cash);
        let mut trades: Vec<TradeFill> = vec![];

        for tick in &self.market_data {
            self.trader.tick(Some(tick));

            let orders = self.trader.get_open();
            let fills = self.broker.execute_orders(&orders, tick);

            for fill in fills {
                trades.push(fill.clone());
                self.trader.on_event(MarketEvent::OrderFilled(fill));
            }
        }

        self.trader.on_event(MarketEvent::MarketClosed);
        let pnl: Vec<(Symbol, Vec<(TradeFill, f64)>)> = Backtester::<T>::calculate_pnl(trades);

        for (sym, fills) in &pnl {
            println!("Symbol: {:?}", sym.0);
            for (fill, pnl) in fills {
                println!("{:?} => PnL: {:.2}", fill, pnl);
            }
        }

        for (symbol, total_pnl) in Backtester::<T>::calculate_cumulative_pnl(pnl) {
            println!("{:?}: {}", symbol, total_pnl)
        }

        println!("Final cash: {:.2}", self.broker.cash);
        println!("Positions: {:?}", self.broker.positions);
    }
}

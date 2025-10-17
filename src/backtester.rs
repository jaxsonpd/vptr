// The backtesting framework that tests traders

use crate::broker::Broker;
use crate::data::{MarketData, MarketEvent};
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

    pub fn run(&mut self) {
        println!("Starting backtest with cash: {:.2}", self.broker.cash);
        for tick in &self.market_data {
            self.trader.tick(Some(tick));

            let orders = self.trader.get_open();
            let fills = self.broker.execute_orders(&orders, tick);

            for fill in fills {
                self.trader.on_event(MarketEvent::OrderFilled(fill));
            }
        }

        self.trader.on_event(MarketEvent::MarketClosed);
        println!("Final cash: {:.2}", self.broker.cash);
        println!("Positions: {:?}", self.broker.positions);
    }
}

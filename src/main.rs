mod backtester;
mod broker;
mod data;
mod trader;

use crate::data::{AssetClass, MarketData, MarketEvent, Order, OrderType, Symbol, Side};
use crate::trader::Trader;
use crate::backtester::Backtester;

/// ======================
/// Example strategy
/// ======================

pub struct MeanReversionTrader {
    pub target_symbol: Symbol,
    pub asset_class: AssetClass,
    pub last_price: Option<f64>,
    open_orders: Vec<Order>,
}

impl Trader for MeanReversionTrader {
    fn symbols(&self) -> Vec<Symbol> {
        vec![self.target_symbol.clone()]
    }

    fn get_open(&self) -> Vec<Order> {
        self.open_orders.clone()
    }

    fn tick(&mut self, data: Option<&MarketData>) {
        let data = data.unwrap();

        if data.symbol != self.target_symbol {
            return;
        }

        if let Some(prev) = self.last_price {
            if data.price < prev * 0.95 {
                // Buy dip
                self.open_orders.push(Order {
                    symbol: data.symbol.clone(),
                    qty: 1.0,
                    side: Side::Buy,
                    order_type: OrderType::Market,
                });
            } else if data.price > prev * 1.05 {
                // Sell rally
                self.open_orders.push(Order {
                    symbol: data.symbol.clone(),
                    qty: 1.0,
                    side: Side::Sell,
                    order_type: OrderType::Market,
                });
            }
        }

        self.last_price = Some(data.price);
    }

    fn on_event(&mut self, event: MarketEvent) {
        match event {
            MarketEvent::OrderFilled(fill) => {
                println!(
                    "[{:?}] {:?} {:?} @ {:.2}",
                    fill.symbol, fill.side, fill.qty, fill.price
                );
            }
            MarketEvent::MarketClosed => {
                println!("Market closed");
            }
        }
    }
}

/// ======================
/// Example usage
/// ======================

fn main() {
    // Simulated mixed-asset feed (equities + crypto)
    let market_data = vec![
        MarketData {
            symbol: Symbol("AAPL".into()),
            asset_class: AssetClass::Equity,
            price: 190.0,
            time: 1,
        },
        MarketData {
            symbol: Symbol("BTC-USD".into()),
            asset_class: AssetClass::Crypto,
            price: 60000.0,
            time: 1,
        },
        MarketData {
            symbol: Symbol("AAPL".into()),
            asset_class: AssetClass::Equity,
            price: 180.0,
            time: 2,
        },
        MarketData {
            symbol: Symbol("BTC-USD".into()),
            asset_class: AssetClass::Crypto,
            price: 63000.0,
            time: 2,
        },
        MarketData {
            symbol: Symbol("AAPL".into()),
            asset_class: AssetClass::Equity,
            price: 195.0,
            time: 3,
        },
        MarketData {
            symbol: Symbol("BTC-USD".into()),
            asset_class: AssetClass::Crypto,
            price: 59000.0,
            time: 3,
        },
    ];

    let trader = MeanReversionTrader {
        target_symbol: Symbol("AAPL".into()),
        asset_class: AssetClass::Equity,
        last_price: None,
        open_orders: vec![]
    };

    let mut backtester = Backtester::new(trader, market_data, 10000.0);
    backtester.run();
}

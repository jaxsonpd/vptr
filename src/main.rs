mod backtester;
mod broker;
mod data;
mod trader;

use std::collections::HashMap;
use std::error::Error;
use std::future;

use crate::data::{AlpacaData, AssetClass, DataStore, Interval, MarketData, MarketEvent, Order, OrderType, Side, Symbol};
use crate::trader::Trader;
use crate::backtester::Backtester;

pub struct MeanReversionTrader {
    pub target_symbol: Symbol,
    pub asset_class: AssetClass,
    pub last_price: Option<f64>,
    open_orders: Vec<Order>,
    positions: HashMap<Symbol, f64>,
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
        let position = *self.positions.entry(data.symbol.clone()).or_insert(0.0);


        if data.symbol != self.target_symbol {
            return;
        }

        if let Some(prev) = self.last_price {
            if (data.price < prev * 0.95) && (position < 1.0) {
                // Buy dip
                self.open_orders.push(Order {
                    symbol: data.symbol.clone(),
                    qty: 1.0,
                    side: Side::Buy,
                    order_type: OrderType::Market,
                });
            } else if data.price > prev * 1.05 {
                // Sell rally
                if position > 0.0 {
                    self.open_orders.push(Order {
                        symbol: data.symbol.clone(),
                        qty: position,
                        side: Side::Sell,
                        order_type: OrderType::Market,
                    });
                }
            }
        }

        self.last_price = Some(data.price);
    }

    fn on_event(&mut self, event: MarketEvent) {
        match event {
            MarketEvent::MarketClosed => {
                println!("Market Closed!");
            }
            MarketEvent::OrderFilled(fill) => {
                self.open_orders = vec![];
                match fill.side {
                    Side::Buy => {
                        *self.positions.entry(fill.symbol).or_insert(0.0) += fill.qty;
                    },
                    Side::Sell => {
                        *self.positions.entry(fill.symbol).or_insert(0.0) -= fill.qty;
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Simulated mixed-asset feed (equities + crypto)
    let (api_key, api_secret) = AlpacaData::get_alpaca_api("secrets.toml")?;
    let alpaca = AlpacaData::new(&api_key, &api_secret);
    let market_data = alpaca.get_historic(Symbol("BTC/USD".to_string()), 
                                    AssetClass::Crypto, "2024-01-01", "2024-12-31", 
                                    Interval::Day)?;
    
    let trader = MeanReversionTrader {
        target_symbol: Symbol("BTC/USD".into()),
        asset_class: AssetClass::Crypto,
        last_price: None,
        open_orders: vec![],
        positions: HashMap::new()
    };

    let mut backtester = Backtester::new(trader, market_data, 10000.0);
    backtester.run();

    Ok(())
}

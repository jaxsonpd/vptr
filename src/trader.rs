// Trader trait and associated methods

use crate::data::{MarketData, MarketEvent, Order, Symbol};

pub trait Trader {
    /// Get the symbols that this stratergy trades
    fn symbols(&self) -> Vec<Symbol>;

    /// Called every time new data is recevied or on tick
    fn tick(&mut self, data: Option<&MarketData>);

    /// Returns the current open orders for the stratergy
    fn get_open(&self) -> Vec<Order>;

    /// Callback for when market events occur
    fn on_event(&mut self, event: MarketEvent);
}

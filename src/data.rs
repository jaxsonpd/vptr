/// Core symbol data type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

#[derive(Debug, Clone)]
pub enum AssetClass {
    Equity,
    Crypto,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: Symbol,
    pub asset_class: AssetClass,
    pub price: f64,
    pub time: u64,
}

/// An order request issued by the strategy.
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: Symbol,
    pub qty: f64,
    pub side: Side,
    pub order_type: OrderType,
}

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit(f64),
}

/// Result of an order execution.
#[derive(Debug, Clone)]
pub struct TradeFill {
    pub symbol: Symbol,
    pub qty: f64,
    pub price: f64,
    pub side: Side,
}

#[derive(Debug, Clone)]
pub enum MarketEvent {
    OrderFilled(TradeFill),
    MarketClosed,
}

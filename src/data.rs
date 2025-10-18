use core::time;
use std::{error::Error};

use reqwest::{blocking::{Client, Response}, header::{HeaderMap, HeaderValue}};

use serde_json::Value;

use chrono::{DateTime, Utc};

use serde::Deserialize;
use std::fs;


/// Core symbol data type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub enum Interval {
    Day,
    Hour,
    FiveMinute,
    Minute,
    Second,
}

pub trait DataStore {
    /// Get data on a symbol
    fn get_historic(&self, symbol: Symbol, asset_type: AssetClass, start_date: &str, end_date: &str, interval: Interval) -> Result<Vec<MarketData>, Box<dyn Error>>;
    
    /// Get current data
    fn get(&self, symbol: Symbol) -> MarketData;
}

#[derive(Debug, Deserialize)]
struct Config {
    alpaca: AlpacaKeys,
}

#[derive(Debug, Deserialize)]
struct AlpacaKeys {
    #[serde(rename = "api-key")]
    api_key: String,

    #[serde(rename = "api-secret")]
    api_secret: String,
}

pub struct AlpacaData {
    api_key: String,
    api_secret: String,
    headers: HeaderMap
}

impl AlpacaData {
    pub fn new(api_key: &str, api_secret: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("APCA-API-KEY-ID", HeaderValue::from_str(api_key).unwrap());
        headers.insert("APCA-API-SECRET-KEY", HeaderValue::from_str(api_secret).unwrap());

        AlpacaData {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            headers: headers
        }
    }

    pub fn get_alpaca_api(filename: &str) -> Result<(String, String), Box<dyn Error>> {
        let contents = fs::read_to_string(filename)?;
        let config: Config = toml::from_str(&contents)?;

        Ok((config.alpaca.api_key, config.alpaca.api_secret))
    }
}

impl DataStore for AlpacaData {
    fn get_historic(&self, symbol: Symbol, asset_type: AssetClass, start_date: &str, end_date: &str, interval: Interval) -> Result<Vec<MarketData>, Box<dyn Error>> {
        let base_url = "https://data.alpaca.markets";

        let timeframe = match interval {
            Interval::Day => "1Day",
            Interval::Hour => "1Hour",
            Interval::FiveMinute => "5Min",
            Interval::Minute => "1Min",
            Interval::Second => "Bad"
        };

        let mut market_data_vec: Vec<MarketData> = vec![];

        let client = Client::new();

        if asset_type == AssetClass::Crypto {
            let response = client
                .get(&format!("{}/v1beta3/crypto/us/bars?symbols={}&timeframe={}&start={}&end={}", base_url, symbol.0, timeframe, start_date, end_date))
                .headers(self.headers.clone())
                .send()?
                .json::<Value>()?;

            if let Some(bars) = response.get("bars") {
                for (symbol_str, bar_array) in bars.as_object().unwrap() {
                    for bar in bar_array.as_array().unwrap() {
                        let price = bar.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let time_str = bar.get("t").and_then(|v| v.as_str()).unwrap_or("");
                        let time = DateTime::parse_from_rfc3339(time_str)?.timestamp() as u64;

                        market_data_vec.push(MarketData {
                            symbol: Symbol(symbol_str.clone()),
                            asset_class: AssetClass::Crypto,
                            price,
                            time,
                        });
                    }
                }
            }
        } else if asset_type == AssetClass::Equity {
            let response: serde_json::Value = client
                .get(&format!("{}/v2/stocks/{}/bars?timeframe={}&start={}&end={}", base_url, symbol.0, timeframe, start_date, end_date))
                .headers(self.headers.clone())
                .send()?
                .json::<Value>()?;

            println!("{:#}", response);
        } else {
            return Err("Unsupported asset type".into());
        }        

        Ok(market_data_vec)
    }

    fn get(&self, symbol: Symbol) -> MarketData {
        println!("Get: {:?}", symbol);

        MarketData { symbol: symbol, asset_class: AssetClass::Equity, price: 10.0, time: 0 }
    }
}

mod tests {
    use super::*;

    #[test]
    fn check_historic() {
        let (api_key, api_secret) = AlpacaData::get_alpaca_api("secrets.toml").unwrap();
        let data = AlpacaData::new(&api_key, &api_secret);
        
        println!("{:?}", data.get_historic(Symbol("AAPL".to_string()), AssetClass::Equity, "2024-01-01", "2024-01-02", Interval::Day));
        println!("{:?}", data.get_historic(Symbol("BTC/USD".to_string()), AssetClass::Crypto, "2024-01-01", "2024-01-02", Interval::Day));
    }

}
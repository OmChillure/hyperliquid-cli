// types for status and spot market
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub markets: Vec<MarketInfo>,
    pub total_markets: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MarketInfo {
    pub symbol: String,
    pub mark_price: f64,
    pub volume_24h: f64,
    pub funding_rate: f64,
    pub max_leverage: u32,
    pub open_interest: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceResponse {
    pub account_value: f64,
    pub withdrawable: f64,
    pub cross_margin_used: f64,
    pub positions: Vec<PositionInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PositionInfo {
    pub symbol: String,
    pub size: f64,
    pub entry_price: f64,
    pub leverage: u32,
    pub unrealized_pnl: f64,
    pub position_value: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SpotResponse {
    pub tokens: Vec<SpotTokenInfo>,
    pub pairs: Vec<SpotPairInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct SpotTokenInfo {
    pub name: String,
    pub decimals: u32,
    pub token_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpotPairInfo {
    pub name: String,
    pub mark_price: f64,
    pub mid_price: f64,
    pub volume_24h: f64,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub symbol: String,
    pub is_buy: bool,
    pub qty: f64,
    pub limit_price: Option<f64>,
    pub leverage: Option<u32>,
    pub reduce_only: bool,
    pub tif: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub status: String,
    pub result: OrderResult,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrderResult {
    Success {
        order_id: u64,
        filled_qty: f64,
        avg_price: Option<f64>,
    },
    Error {
        message: String,
    },
    Resting {
        order_id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    pub order_id: u64,
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub price: f64,
    pub filled_qty: f64,
    pub remaining_qty: f64,
    pub status: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub size: f64,
    pub side: String,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub leverage: u32,
    pub margin_used: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummary {
    pub account_value: f64,
    pub withdrawable: f64,
    pub total_margin_used: f64,
    pub total_unrealized_pnl: f64,
}
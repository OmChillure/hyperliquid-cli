use std::{collections::HashMap};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: String,
    pub ws_url: String,
    pub private_key: String,
    pub risk_limits: RiskLimits,
}

#[derive(Debug, Clone)]
pub struct SymbolLimits {
    pub max_leverage: u32,             
    pub max_notional: f64,             
    pub enabled: bool,
}


#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_notional_per_order: f64,
    pub max_notional_per_symbol: f64,
    pub symbol_limits: HashMap<String, SymbolLimits>,
}


// config to load api keys with fallback urls and risk parameters
use std::{env, collections::HashMap};
use anyhow::Result;
use crate::types::{Config, SymbolLimits, RiskLimits};

impl Default for RiskLimits {
    fn default() -> Self {
        let mut symbol_limits = HashMap::new();
        
        symbol_limits.insert("BTC".to_string(), SymbolLimits {
            max_leverage: 10,
            max_notional: 50_000.0,
            enabled: true,
        });
        
        symbol_limits.insert("ETH".to_string(), SymbolLimits {
            max_leverage: 15,
            max_notional: 30_000.0,
            enabled: true,
        });
        
        symbol_limits.insert("SOL".to_string(), SymbolLimits {
            max_leverage: 20,
            max_notional: 20_000.0,
            enabled: true,
        });

        symbol_limits.insert("ARB".to_string(), SymbolLimits {
            max_leverage: 25,
            max_notional: 15_000.0, 
            enabled: true,
        });

        symbol_limits.insert("AVAX".to_string(), SymbolLimits {
            max_leverage: 20,
            max_notional: 15_000.0,
            enabled: true,
        });
        
        // if you need more tokens please add it
        
        // global limits
        Self {
            max_notional_per_order: 10_000.0,   
            max_notional_per_symbol: 25_000.0,
            symbol_limits,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            api_url: env::var("HYPERLIQUID_API_URL")
                .unwrap_or_else(|_| "https://api.hyperliquid-testnet.xyz".to_string()),
            ws_url: env::var("HYPERLIQUID_WS_URL")
                .unwrap_or_else(|_| "wss://api.hyperliquid-testnet.xyz/ws".to_string()),
            private_key: env::var("PRIVATE_KEY")
                .map_err(|_| anyhow::anyhow!("PRIVATE_KEY must be set"))?,
            risk_limits: RiskLimits::default(),
        })
    }
    
    pub fn get_symbol_limits(&self, symbol: &str) -> SymbolLimits {
        self.risk_limits.symbol_limits
            .get(symbol)
            .cloned()
            .unwrap_or(SymbolLimits {
                max_leverage: 10,
                max_notional: self.risk_limits.max_notional_per_symbol,
                enabled: true,
            })
    }

    pub fn is_symbol_enabled(&self, symbol: &str) -> bool {
        self.get_symbol_limits(symbol).enabled
    }
    
    pub fn get_max_leverage(&self, symbol: &str) -> u32 {
        self.get_symbol_limits(symbol).max_leverage
    }
    
    pub fn get_max_notional(&self, symbol: &str) -> f64 {
        self.get_symbol_limits(symbol).max_notional
    }
}

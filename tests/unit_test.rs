use anyhow::Result;
use hyperliquid_cli::{
    types::{Config, RiskLimits, SymbolLimits, OrderRequest},
};
use std::collections::HashMap;

#[cfg(test)]
mod risk_policy_tests {
    use super::*;

    fn create_simple_risk_limits() -> RiskLimits {
        let mut symbol_limits = HashMap::new();

        symbol_limits.insert("BTC".to_string(), SymbolLimits {
            max_leverage: 5,
            max_notional: 50_000.0,
            enabled: true,
        });
        
        symbol_limits.insert("ETH".to_string(), SymbolLimits {
            max_leverage: 10,
            max_notional: 30_000.0,
            enabled: true,
        });
        
        RiskLimits {
            max_notional_per_order: 10_000.0,
            max_notional_per_symbol: 25_000.0,
            symbol_limits,
        }
    }
    
    fn create_test_config() -> Config {
        Config {
            api_url: "https://api.hyperliquid-testnet.xyz".to_string(),
            ws_url: "wss://api.hyperliquid-testnet.xyz/ws".to_string(),
            //random walllet key.
            private_key: "0xbe4526735a0c6h8c6c79fb806143f6d4e1abbbd9a487e6a37451adeda6510ee1".to_string(),
            risk_limits: create_simple_risk_limits(),
        }
    }

    #[test]
    fn test_btc_risk_policy() {
        let config = create_test_config();

        assert!(config.is_symbol_enabled("BTC"), "BTC should be enabled");
        assert_eq!(config.get_max_leverage("BTC"), 5, "BTC max leverage should be 5x");
        assert_eq!(config.get_max_notional("BTC"), 50_000.0, "BTC max notional should be 50k");
        
        // Test BTC order within limits
        let valid_btc_order = OrderRequest {
            symbol: "BTC".to_string(),
            is_buy: true,
            qty: 0.1,
            limit_price: Some(50_000.0),
            leverage: Some(3),
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &valid_btc_order).is_ok(), 
               "Valid BTC order should pass");
        
        // Test BTC order with too high leverage
        let high_leverage_btc = OrderRequest {
            symbol: "BTC".to_string(),
            is_buy: true,
            qty: 0.1,
            limit_price: Some(50_000.0),
            leverage: Some(10), 
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &high_leverage_btc).is_err(), 
               "BTC order with 10x leverage should fail (max 5x)");
        
        // Test BTC order with too high notional
        let high_notional_btc = OrderRequest {
            symbol: "BTC".to_string(),
            is_buy: true,
            qty: 1.0,
            limit_price: Some(15_000.0),
            leverage: Some(3),
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &high_notional_btc).is_err(), 
               "BTC order exceeding 10k per-order limit should fail");
    }

    #[test]
    fn test_eth_risk_policy() {
        let config = create_test_config();

        assert!(config.is_symbol_enabled("ETH"), "ETH should be enabled");
        assert_eq!(config.get_max_leverage("ETH"), 10, "ETH max leverage should be 10x");
        assert_eq!(config.get_max_notional("ETH"), 30_000.0, "ETH max notional should be 30k");
        
        // Test valid ETH order
        let valid_eth_order = OrderRequest {
            symbol: "ETH".to_string(),
            is_buy: true,
            qty: 2.0,
            limit_price: Some(3_000.0),
            leverage: Some(8),
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &valid_eth_order).is_ok(), 
               "Valid ETH order should pass");
        
        // Test ETH order with too high leverage
        let high_leverage_eth = OrderRequest {
            symbol: "ETH".to_string(),
            is_buy: true,
            qty: 2.0,
            limit_price: Some(3_000.0),
            leverage: Some(15),
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &high_leverage_eth).is_err(), 
               "ETH order with 15x leverage should fail (max 10x)");
        
        // Test ETH order exceeding per-order limit
        let high_notional_eth = OrderRequest {
            symbol: "ETH".to_string(),
            is_buy: true,
            qty: 4.0,
            limit_price: Some(3_000.0),
            leverage: Some(5),
            reduce_only: false,
            tif: "Gtc".to_string(),
        };
        
        assert!(validate_order_request(&config, &high_notional_eth).is_err(), 
               "ETH order exceeding 10k per-order limit should fail");
    }

    
    // Helper function to simulate order validation (simplified)
    fn validate_order_request(config: &Config, order: &OrderRequest) -> Result<()> {
        if !config.is_symbol_enabled(&order.symbol) {
            anyhow::bail!("Trading disabled for symbol: {}", order.symbol);
        }
        
        if let Some(leverage) = order.leverage {
            let max_leverage = config.get_max_leverage(&order.symbol);
            if leverage > max_leverage {
                anyhow::bail!("Leverage {}x exceeds maximum {}x for {}", 
                             leverage, max_leverage, order.symbol);
            }
        }
        
        if let Some(price) = order.limit_price {
            let notional = order.qty * price;
            
            if notional > config.risk_limits.max_notional_per_order {
                anyhow::bail!("Order notional ${:.2} exceeds per-order limit ${:.2}",
                             notional, config.risk_limits.max_notional_per_order);
            }
            
            let symbol_max = config.get_max_notional(&order.symbol);
            if notional > symbol_max {
                anyhow::bail!("Order notional ${:.2} exceeds symbol limit ${:.2} for {}",
                             notional, symbol_max, order.symbol);
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_btc_vs_eth_comparison() {
        let config = create_test_config();
        
        // BTC has stricter leverage (5x vs 10x)
        assert!(config.get_max_leverage("BTC") < config.get_max_leverage("ETH"),
               "BTC should have stricter leverage than ETH");
        
        // BTC has higher notional limit (50k vs 30k)
        assert!(config.get_max_notional("BTC") > config.get_max_notional("ETH"),
               "BTC should have higher notional limit than ETH");
        
        // Both should be enabled
        assert!(config.is_symbol_enabled("BTC") && config.is_symbol_enabled("ETH"),
               "Both BTC and ETH should be enabled");
    }
}

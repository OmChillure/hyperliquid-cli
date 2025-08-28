use crate::types::{Config, OrderRequest, OrderResponse, OrderResult};
use anyhow::{Context, Result};
use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, ExchangeDataStatus,
    ExchangeResponseStatus, InfoClient, MarketCloseParams, MarketOrderParams,
};

pub struct TradingService {
    exchange_client: ExchangeClient,
    info_client: InfoClient,
    config: Config,
}

impl TradingService {
    pub async fn new(config: Config) -> Result<Self> {
        let wallet: LocalWallet = config
            .private_key
            .parse()
            .context("Failed to parse private key")?;

        let base_url = BaseUrl::Testnet;

        let exchange_client = ExchangeClient::new(None, wallet, Some(base_url), None, None)
            .await
            .context("Failed to create exchange client")?;

        let info_client = InfoClient::new(None, Some(base_url))
            .await
            .context("Failed to create info client")?;

        Ok(Self {
            exchange_client,
            info_client,
            config,
        })
    }

    // Main order placement with validation
    pub async fn place_order(&self, order_request: OrderRequest) -> Result<OrderResponse> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;

        // Validate order before placement
        if let Err(validation_error) = self.validate_order(&order_request).await {
            return Ok(OrderResponse {
                status: "error".to_string(),
                result: OrderResult::Error {
                    message: validation_error.to_string(),
                },
                timestamp,
            });
        }

        // Set leverage if specified
        if let Some(leverage) = order_request.leverage {
            self.set_leverage(&order_request.symbol, leverage).await?;
        }

        let result = if order_request.limit_price.is_some() {
            self.place_limit_order(order_request).await?
        } else {
            self.place_market_order(order_request).await?
        };

        match result {
            ExchangeResponseStatus::Ok(response) => {
                if let Some(data) = response.data {
                    if let Some(status) = data.statuses.first() {
                        let order_result = match status {
                            ExchangeDataStatus::Success => OrderResult::Success {
                                order_id: 0,
                                filled_qty: 0.0,
                                avg_price: None,
                            },
                            ExchangeDataStatus::Filled(filled) => OrderResult::Success {
                                order_id: filled.oid,
                                filled_qty: filled.total_sz.parse().unwrap_or(0.0),
                                avg_price: Some(filled.avg_px.parse().unwrap_or(0.0)),
                            },
                            ExchangeDataStatus::Resting(resting) => OrderResult::Resting {
                                order_id: resting.oid,
                            },
                            ExchangeDataStatus::Error(msg) => OrderResult::Error {
                                message: msg.clone(),
                            },
                            _ => OrderResult::Error {
                                message: "Unknown status".to_string(),
                            },
                        };

                        return Ok(OrderResponse {
                            status: "success".to_string(),
                            result: order_result,
                            timestamp,
                        });
                    }
                }

                Ok(OrderResponse {
                    status: "error".to_string(),
                    result: OrderResult::Error {
                        message: "No response data".to_string(),
                    },
                    timestamp,
                })
            }
            ExchangeResponseStatus::Err(error) => Ok(OrderResponse {
                status: "error".to_string(),
                result: OrderResult::Error { message: error },
                timestamp,
            }),
        }
    }

    // Comprehensive order validation
    async fn validate_order(&self, order_request: &OrderRequest) -> Result<()> {
        if !self.config.is_symbol_enabled(&order_request.symbol) {
            anyhow::bail!("Trading disabled for symbol: {}", order_request.symbol);
        }

        self.validate_leverage(&order_request.symbol, order_request.leverage)
            .await?;
        self.validate_notional(order_request).await?;

        Ok(())
    }

    async fn validate_leverage(&self, symbol: &str, requested_leverage: Option<u32>) -> Result<()> {
        if let Some(leverage) = requested_leverage {
            let config_max_leverage = self.config.get_max_leverage(symbol);
            if leverage > config_max_leverage {
                anyhow::bail!(
                    "Requested leverage {}x exceeds configured maximum {}x for {}",
                    leverage,
                    config_max_leverage,
                    symbol
                );
            }
        }

        Ok(())
    }

    async fn validate_notional(&self, order_request: &OrderRequest) -> Result<()> {
        let price = if let Some(limit_price) = order_request.limit_price {
            limit_price
        } else {
            self.get_market_price(&order_request.symbol).await?
        };

        let order_notional = order_request.qty * price;

        if order_notional > self.config.risk_limits.max_notional_per_order {
            anyhow::bail!(
                "Order notional ${:.2} exceeds per-order limit ${:.2}",
                order_notional,
                self.config.risk_limits.max_notional_per_order
            );
        }

        let symbol_max_notional = self.config.get_max_notional(&order_request.symbol);
        if order_notional > symbol_max_notional {
            anyhow::bail!(
                "Order notional ${:.2} exceeds symbol limit ${:.2} for {}",
                order_notional,
                symbol_max_notional,
                order_request.symbol
            );
        }

        println!(
            "Order validation: {} {} @ ${:.4} = ${:.2} notional (per-order limit: ${:.2}, symbol limit: ${:.2})",
            order_request.qty,
            order_request.symbol,
            price,
            order_notional,
            self.config.risk_limits.max_notional_per_order,
            symbol_max_notional
        );

        Ok(())
    }

    async fn get_market_price(&self, symbol: &str) -> Result<f64> {
        let all_mids = self
            .info_client
            .all_mids()
            .await
            .context("Failed to fetch market prices")?;

        let price_str = all_mids
            .get(symbol)
            .ok_or_else(|| anyhow::anyhow!("Price not found for symbol: {}", symbol))?;

        price_str
            .parse::<f64>()
            .context("Failed to parse market price")
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        match self
            .exchange_client
            .update_leverage(leverage, symbol, true, None)
            .await
        {
            Ok(ExchangeResponseStatus::Ok(_)) => {
                println!("Leverage set to {}x for {}", leverage, symbol);
                Ok(())
            }
            Ok(ExchangeResponseStatus::Err(error)) => {
                anyhow::bail!("Failed to set leverage: {}", error)
            }
            Err(e) => Err(e.into()),
        }
    }

    // Place limit order
    async fn place_limit_order(
        &self,
        order_request: OrderRequest,
    ) -> Result<ExchangeResponseStatus> {
        let client_order = ClientOrderRequest {
            asset: order_request.symbol.clone(),
            is_buy: order_request.is_buy,
            reduce_only: order_request.reduce_only,
            limit_px: order_request.limit_price.unwrap(),
            sz: order_request.qty,
            cloid: None,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: order_request.tif,
            }),
        };

        self.exchange_client
            .order(client_order, None)
            .await
            .context("Failed to place limit order")
    }

    // Place market order
    async fn place_market_order(
        &self,
        order_request: OrderRequest,
    ) -> Result<ExchangeResponseStatus> {
        let market_params = MarketOrderParams {
            asset: &order_request.symbol,
            is_buy: order_request.is_buy,
            sz: order_request.qty,
            px: None,
            slippage: Some(0.05),
            cloid: None,
            wallet: None,
        };

        if order_request.reduce_only {
            let close_params = MarketCloseParams {
                asset: &order_request.symbol,
                sz: Some(order_request.qty),
                px: None,
                slippage: Some(0.05),
                cloid: None,
                wallet: None,
            };

            self.exchange_client
                .market_close(close_params)
                .await
                .context("Failed to place market close order")
        } else {
            self.exchange_client
                .market_open(market_params)
                .await
                .context("Failed to place market order")
        }
    }

    // Cancel order
    pub async fn cancel_order(&self, symbol: &str, order_id: u64) -> Result<()> {
        use hyperliquid_rust_sdk::ClientCancelRequest;

        let cancel_request = ClientCancelRequest {
            asset: symbol.to_string(),
            oid: order_id,
        };

        match self.exchange_client.cancel(cancel_request, None).await {
            Ok(ExchangeResponseStatus::Ok(_)) => Ok(()),
            Ok(ExchangeResponseStatus::Err(error)) => {
                anyhow::bail!("Cancel failed: {}", error)
            }
            Err(e) => Err(e.into()),
        }
    }
}

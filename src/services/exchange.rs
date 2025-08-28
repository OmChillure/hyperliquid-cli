use crate::types::*;
use anyhow::{Context, Result};
use alloy::signers::{local::PrivateKeySigner};
use reqwest::Client;

#[derive(Clone)]
pub struct ExchangeService {
    client: Client,
    config: Config,
}

impl ExchangeService {
    // client initialization
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }
     
    // get metadata of markets and ctxs
    pub async fn get_status(&self) -> Result<StatusResponse> {
        let (universe, contexts) = self.get_meta_and_asset_ctxs().await?;

        let markets: Vec<MarketInfo> = universe
            .iter()
            .zip(contexts.iter())
            .filter(|(asset, _)| !asset.is_delisted)
            .map(|(asset, context)| MarketInfo {
                symbol: asset.name.clone(),
                mark_price: context
                    .mark_px
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                volume_24h: context
                    .day_ntl_vlm
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                funding_rate: context
                    .funding
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                max_leverage: asset.max_leverage,
                open_interest: context
                    .open_interest
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
            })
            .collect();

        Ok(StatusResponse {
            total_markets: markets.len(),
            markets,
        })
    }

    // get balances and positions
    pub async fn get_balances(&self) -> Result<BalanceResponse> {
        let wallet_address = self.get_wallet_address()?;
        let state = self.get_clearinghouse_state(&wallet_address).await?;

        let positions: Vec<PositionInfo> = state
            .asset_positions
            .iter()
            .filter(|asset_pos| asset_pos.position.szi.parse::<f64>().unwrap_or(0.0).abs() > 0.0001)
            .map(|asset_pos| {
                let pos = &asset_pos.position;
                PositionInfo {
                    symbol: pos.coin.clone(),
                    size: pos.szi.parse().unwrap_or(0.0),
                    entry_price: pos
                        .entry_px
                        .as_ref()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(0.0),
                    leverage: pos.leverage.value,
                    unrealized_pnl: pos.unrealized_pnl.parse().unwrap_or(0.0),
                    position_value: pos.position_value.parse().unwrap_or(0.0),
                }
            })
            .collect();

        Ok(BalanceResponse {
            account_value: state.margin_summary.account_value.parse().unwrap_or(0.0),
            withdrawable: state.withdrawable.parse().unwrap_or(0.0),
            cross_margin_used: state
                .cross_margin_used
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            positions,
        })
    }

    // get spot markets
    pub async fn get_spot_markets(&self) -> Result<SpotResponse> {
        let (spot_meta, spot_contexts) = self.get_spot_meta_and_asset_ctxs().await?;

        let tokens: Vec<SpotTokenInfo> = spot_meta
            .tokens
            .iter()
            .map(|token| SpotTokenInfo {
                name: token.name.clone(),
                decimals: token.sz_decimals,
                token_id: token.token_id.clone(),
            })
            .collect();

        let pairs: Vec<SpotPairInfo> = spot_meta
            .universe
            .iter()
            .zip(spot_contexts.iter())
            .map(|(pair, context)| SpotPairInfo {
                name: pair.name.clone(),
                mark_price: context
                    .mark_px
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                mid_price: context
                    .mid_px
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                volume_24h: context
                    .day_ntl_vlm
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
            })
            .collect();

        Ok(SpotResponse { tokens, pairs })
    }

    // Private helper methods
    async fn get_meta_and_asset_ctxs(&self) -> Result<(Vec<AssetInfo>, Vec<AssetContext>)> {
        let request = InfoRequest {
            request_type: "metaAndAssetCtxs".to_string(),
            user: None,
        };

        let response = self
            .client
            .post(&format!("{}/info", self.config.api_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send metaAndAssetCtxs request")?;

        let json: serde_json::Value = response.json().await.context("Failed to parse response")?;

        let array = json.as_array().context("Expected array response")?;
        if array.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2 elements in response"));
        }

        let universe_obj = array[0]
            .as_object()
            .context("Expected object for universe")?;
        let universe_array = universe_obj
            .get("universe")
            .and_then(|v| v.as_array())
            .context("Expected universe array")?;

        let universe: Vec<AssetInfo> =
            serde_json::from_value(serde_json::Value::Array(universe_array.clone()))?;
        let contexts: Vec<AssetContext> = serde_json::from_value(array[1].clone())?;

        Ok((universe, contexts))
    }

    async fn get_clearinghouse_state(&self, user_address: &str) -> Result<ClearinghouseState> {
        let request = InfoRequest {
            request_type: "clearinghouseState".to_string(),
            user: Some(user_address.to_string()),
        };

        let response = self
            .client
            .post(&format!("{}/info", self.config.api_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send clearinghouseState request")?;

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;

        let json: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse JSON")?;

        serde_json::from_value(json).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse clearinghouse state: {}. Raw response was logged above.",
                e
            )
        })
    }

    async fn get_spot_meta_and_asset_ctxs(&self) -> Result<(SpotMeta, Vec<SpotAssetContext>)> {
        let request = InfoRequest {
            request_type: "spotMetaAndAssetCtxs".to_string(),
            user: None,
        };

        let response = self
            .client
            .post(&format!("{}/info", self.config.api_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        let json: serde_json::Value = response.json().await.context("Failed to parse response")?;
        let array = json.as_array().context("Expected array response")?;
        if array.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2 elements in response"));
        }

        let spot_meta: SpotMeta = serde_json::from_value(array[0].clone())?;
        let spot_contexts: Vec<SpotAssetContext> = serde_json::from_value(array[1].clone())?;

        Ok((spot_meta, spot_contexts))
    }

    fn get_wallet_address(&self) -> Result<String> {
        let wallet: PrivateKeySigner = self
            .config
            .private_key
            .parse()
            .context("Failed to parse private key")?;
        Ok(format!("{:?}", wallet.address()))
    }
}
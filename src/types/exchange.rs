use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct InfoRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct AssetInfo {
    pub name: String,
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u32,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: u32,
    #[serde(rename = "onlyIsolated", default)]
    pub only_isolated: bool,
    #[serde(rename = "isDelisted", default)]
    pub is_delisted: bool,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AssetContext {
    #[serde(rename = "markPx")]
    pub mark_px: Option<String>,
    #[serde(rename = "midPx")]
    pub mid_px: Option<String>,
    #[serde(rename = "dayNtlVlm")]
    pub day_ntl_vlm: Option<String>,
    pub funding: Option<String>,
    #[serde(rename = "openInterest")]
    pub open_interest: Option<String>,
    #[serde(rename = "prevDayPx")]
    pub prev_day_px: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ClearinghouseState {
    #[serde(rename = "marginSummary")]
    pub margin_summary: MarginSummary,
    #[serde(rename = "withdrawable")]
    pub withdrawable: String,
    #[serde(rename = "crossMarginUsed", default)]
    pub cross_margin_used: Option<String>,
    #[serde(rename = "assetPositions")]
    pub asset_positions: Vec<AssetPosition>,
}
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct MarginSummary {
    #[serde(rename = "accountValue")]
    pub account_value: String,
    #[serde(rename = "totalNtlPos")]
    pub total_ntl_pos: String,
    #[serde(rename = "totalRawUsd")]
    pub total_raw_usd: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AssetPosition {
    pub position: Position,
    #[serde(rename = "type")]
    pub position_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Position {
    pub coin: String,
    #[serde(rename = "entryPx")]
    pub entry_px: Option<String>,
    pub leverage: Leverage,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
    #[serde(rename = "positionValue")]
    pub position_value: String,
    pub szi: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u32,
}

#[derive(Deserialize, Debug)]
pub struct SpotMeta {
    pub tokens: Vec<SpotToken>,
    pub universe: Vec<SpotPair>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SpotToken {
    pub name: String,
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u32,
    pub index: u32,
    #[serde(rename = "tokenId")]
    pub token_id: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SpotPair {
    pub name: String,
    pub tokens: [u32; 2],
    pub index: u32,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SpotAssetContext {
    #[serde(rename = "dayNtlVlm")]
    pub day_ntl_vlm: Option<String>,
    #[serde(rename = "markPx")]
    pub mark_px: Option<String>,
    #[serde(rename = "midPx")]
    pub mid_px: Option<String>,
    #[serde(rename = "prevDayPx")]
    pub prev_day_px: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct WsSubscription {
    pub method: String,
    pub subscription: WsSubscriptionData,
}

#[derive(Serialize, Debug)]
pub struct WsSubscriptionData {
    #[serde(rename = "type")]
    pub sub_type: String,
    pub coin: String,
}

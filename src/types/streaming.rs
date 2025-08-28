use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct SubscriptionRequest {
    pub method: String,
    pub subscription: TradesSubscription,
}

#[derive(Serialize, Debug)]
pub struct TradesSubscription {
    #[serde(rename = "type")]
    pub sub_type: String,
    pub coin: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct WSMessage {
    pub channel: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct TradeData {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
    pub tid: u64,
    pub users: (String, String),
}



#[derive(Deserialize, Debug)]
pub struct TradesResponse {
    pub data: Vec<TradeData>,
}
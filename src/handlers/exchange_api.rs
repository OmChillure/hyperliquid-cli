use axum::{extract::State, Json};
use anyhow::Result;
use crate::{services::ExchangeService, types::*};

// health check 
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: "0.1.0".to_string(),
    })
}


// chain status [markets]
pub async fn get_status(
    State(exchange): State<ExchangeService>
) -> Result<Json<StatusResponse>, String> {
    match exchange.get_status().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => Err(format!("Failed to get status: {}", e)),
    }
}

// balances and positions of users
pub async fn get_balances(
    State(exchange): State<ExchangeService>
) -> Result<Json<BalanceResponse>, String> {
    match exchange.get_balances().await {
        Ok(balances) => Ok(Json(balances)),
        Err(e) => Err(format!("Failed to get balances: {}", e)),
    }
}


// extra get spot markets
pub async fn get_spot_markets(
    State(exchange): State<ExchangeService>
) -> Result<Json<SpotResponse>, String> {
    match exchange.get_spot_markets().await {
        Ok(spot_data) => Ok(Json(spot_data)),
        Err(e) => Err(format!("Failed to get spot markets: {}", e)),
    }
}

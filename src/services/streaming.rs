use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use crate::types::{Config, streaming::*};
use tokio::time::{Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone)]
pub struct StreamingService {
    config: Config,
}

impl StreamingService {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }
   

    pub async fn stream_data(&self, symbol: &str, _stream_type: &str, duration: u64) -> Result<()> {
        let ws_url = self.config.ws_url.clone();

        println!("Connecting to WebSocket: {}", ws_url);

        let (ws_stream, _) = connect_async(ws_url)
            .await
            .context("Failed to connect to WebSocket")?;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let subscription = SubscriptionRequest {
            method: "subscribe".to_string(),
            subscription: TradesSubscription {
                sub_type: "trades".to_string(),
                coin: symbol.to_string(),
            },
        };

        let subscription_msg = serde_json::to_string(&subscription)
            .context("Failed to serialize subscription")?;

        ws_sender
            .send(Message::Text(subscription_msg))
            .await
            .context("Failed to send subscription")?;

        println!("Subscribed to trades for {}", symbol);

        self.print_stream_header(symbol, duration);

        let start_time = std::time::Instant::now();
        let mut message_count = 0;
        let mut trade_count = 0;

        let ping_interval = Duration::from_secs(30);
        let mut last_ping = std::time::Instant::now();

        let mut subscription_confirmed = false;
        let mut no_message_count = 0;
        let max_no_message_cycles = 100; 

        loop {
            if start_time.elapsed().as_secs() >= duration {
                println!("\nStream duration of {}s reached", duration);
                break;
            }

            if last_ping.elapsed() >= ping_interval {
                let ping_msg = serde_json::json!({"method": "ping"});
                if let Ok(ping_str) = serde_json::to_string(&ping_msg) {
                    let _ = ws_sender.send(Message::Text(ping_str)).await;
                    last_ping = std::time::Instant::now();
                }
            }

            let timeout_duration = Duration::from_millis(100);
            
            match tokio::time::timeout(timeout_duration, ws_receiver.next()).await {
                Ok(Some(Ok(msg))) => {
                    message_count += 1;
                    no_message_count = 0;
                    
                    match msg {
                        Message::Text(text) => {
                            if let Ok(ws_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                if ws_msg.get("channel").and_then(|c| c.as_str()) == Some("subscriptionResponse") {
                                    println!("Subscription confirmed for {}", symbol);
                                    subscription_confirmed = true;
                                    continue;
                                }

                                if ws_msg.get("channel").and_then(|c| c.as_str()) == Some("pong") {
                                    continue;
                                }

                                if ws_msg.get("channel").and_then(|c| c.as_str()) == Some("trades") {
                                    if let Ok(trades_resp) = serde_json::from_value::<TradesResponse>(ws_msg) {
                                        for trade in trades_resp.data {
                                            trade_count += 1;
                                            self.print_trade(&trade);
                                        }
                                    }
                                }
                            }
                        }
                        Message::Pong(_) => {
                        }
                        Message::Close(_) => {
                            println!("WebSocket connection closed by server");
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(Some(Err(e))) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                Ok(None) => {
                    println!("WebSocket connection ended");
                    break;
                }
                Err(_) => {
                    no_message_count += 1;
                    
                    if subscription_confirmed && no_message_count % 50 == 0 {
                        let elapsed = start_time.elapsed().as_secs();
                        let remaining = duration.saturating_sub(elapsed);
                        print!("\rWaiting for trades... ({}s remaining)", remaining);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap_or(());
                    }
                    

                    if no_message_count == max_no_message_cycles {
                        println!("\nNo trades received for 10+ seconds. Market might be quiet or connection issue.");
                    }
                }
            }
        }

        let unsubscribe = SubscriptionRequest {
            method: "unsubscribe".to_string(),
            subscription: TradesSubscription {
                sub_type: "trades".to_string(),
                coin: symbol.to_string(),
            },
        };

        if let Ok(unsubscribe_msg) = serde_json::to_string(&unsubscribe) {
            let _ = ws_sender.send(Message::Text(unsubscribe_msg)).await;
        }

        let _ = ws_sender.close().await;

        println!("\n═══════════════════════════════════════════════");
        println!("Stream completed!");
        println!("Duration: {}s", start_time.elapsed().as_secs());
        println!("Total WebSocket messages: {}", message_count);
        println!("Total trades received: {}", trade_count);
        
        if trade_count == 0 {
            println!("No trades received - this could mean:");
            println!("   • Market is quiet for {} right now", symbol);
            println!("   • Symbol might not exist (try: ETH, BTC, SOL, etc.)");
            println!("   • Try a longer duration (--duration 60)");
        }

        Ok(())
    }

    fn print_stream_header(&self, symbol: &str, duration: u64) {
        let network = if self.config.api_url.contains("testnet") {
            "TESTNET"
        } else {
            "MAINNET"
        };

        println!("\n═══════════════════════════════════════════════");
        println!("  HYPERLIQUID {} TRADE STREAM", network);
        println!("═══════════════════════════════════════════════");
        println!("Symbol: {}", symbol);
        println!("Type: TRADES");
        println!("Duration: {}s", duration);
        println!("Started: {}", Utc::now().format("%H:%M:%S UTC"));
        println!("═══════════════════════════════════════════════");
        println!("{:<12} {:<6} {:<12} {:<12} {:<10} {:<8}", 
            "TIME", "SIDE", "PRICE", "SIZE", "TRADE_ID", "HASH");
        println!("─────────────────────────────────────────────────────────────────────");
    }

    fn print_trade(&self, trade: &crate::types::streaming::TradeData) {
        let datetime = DateTime::from_timestamp_millis(trade.time as i64)
            .unwrap_or_else(|| Utc::now());
        let time_str = datetime.format("%H:%M:%S").to_string();

        let side_colored = if trade.side == "B" { 
            format!("BUY")
        } else { 
            format!("SELL")
        };

        let price: f64 = trade.px.parse().unwrap_or(0.0);
        let size: f64 = trade.sz.parse().unwrap_or(0.0);

        let short_hash = if trade.hash.len() > 8 {
            format!("{}...", &trade.hash[..8])
        } else {
            trade.hash.clone()
        };
        
        println!("{:<12} {:<6} ${:<11.4} {:<12.4} {:<10} {:<8}", 
            time_str,
            side_colored,
            price,
            size,
            trade.tid,
            short_hash
        );
    }
}
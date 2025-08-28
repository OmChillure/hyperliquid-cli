use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::{
    services::{ExchangeService, TradingService}, 
    types::{Config, OrderRequest}
};

#[derive(Parser)]
#[command(name = "hl")]
#[command(about = "Hyperliquid Testnet Trader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Status, 
    Balances,
    Spot,
    Stream {
        symbol: String,
        #[arg(short, long, default_value = "30", help = "Duration in seconds")]
        duration: u64,
    },
    Buy {
        symbol: String,
        qty: f64,
        #[arg(long, help = "Limit price (if not specified, places market order)")]
        limit: Option<f64>,
        #[arg(long, help = "Leverage multiplier")]
        leverage: Option<u32>,
        #[arg(long, help = "Reduce only order")]
        reduce_only: bool,
        #[arg(long, default_value = "Gtc", help = "Time in force (Gtc, Ioc, Alo)")]
        tif: String,
        #[arg(long, help = "Slippage tolerance for market orders (e.g., 0.01 = 1%)")]
        slippage: Option<f64>,
        #[arg(long, help = "Custom tick size for price rounding (e.g., 0.01, 0.1, 1.0)")]
        tick_size: Option<f64>,
    },
    Sell {
        symbol: String,
        qty: f64,
        #[arg(long, help = "Limit price (if not specified, places market order)")]
        limit: Option<f64>,
        #[arg(long, help = "Leverage multiplier")]
        leverage: Option<u32>,
        #[arg(long, help = "Reduce only order")]
        reduce_only: bool,
        #[arg(long, default_value = "Gtc", help = "Time in force (Gtc, Ioc, Alo)")]
        tif: String,
        #[arg(long, help = "Slippage tolerance for market orders (e.g., 0.01 = 1%)")]
        slippage: Option<f64>,
        #[arg(long, help = "Custom tick size for price rounding (e.g., 0.01, 0.1, 1.0)")]
        tick_size: Option<f64>,
    },
    Cancel {
        symbol: String,
        order_id: u64,
    }
}

pub async fn run_cli(cli: Cli) -> Result<()> {
    let config = Config::load()?;
    
    match cli.command {
        Commands::Status => {
            let exchange = ExchangeService::new(config)?;
            println!("Fetching exchange status...");
            let status = exchange.get_status().await?;
            print_status(&status);
        },
        Commands::Balances => {
            let exchange = ExchangeService::new(config)?;
            println!("Fetching account balances...");
            let balances = exchange.get_balances().await?;
            print_balances(&balances);
        },
        Commands::Spot => {
            let exchange = ExchangeService::new(config)?;
            println!("Fetching spot markets...");
            let spot_data = exchange.get_spot_markets().await?;
            print_spot_markets(&spot_data);
        },
        Commands::Stream { symbol, duration } => {
            use crate::services::streaming::StreamingService;
            println!("Starting trade stream for {} ({}s)", symbol, duration);
            let streaming = StreamingService::new(config)?;
            streaming.stream_data(&symbol, "trades", duration).await?;
        },
        Commands::Buy { symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size } => {
            let trading = TradingService::new(config).await?;
            
            if limit.is_none() && slippage.is_some() {
                let slippage_pct = slippage.unwrap();
                if slippage_pct < 0.0 || slippage_pct > 0.1 {
                    eprintln!("Error: Slippage must be between 0% and 10% (0.0 to 0.1)");
                    std::process::exit(1);
                }
            }
            
            if let Some(ts) = tick_size {
                if ts <= 0.0 {
                    eprintln!("Error: Tick size must be greater than 0");
                    std::process::exit(1);
                }
                println!("Using custom tick size: {}", ts);
            }
            
            let order_type = if limit.is_some() { "LIMIT BUY" } else { "MARKET BUY" };
            println!("Placing {} order for {} {}", order_type, qty, symbol);
            
            let order_request = OrderRequest {
                symbol: symbol.clone(),
                is_buy: true,
                qty,
                limit_price: limit,
                leverage,
                reduce_only,
                tif,
            };
            
            match trading.place_order(order_request).await {
                Ok(response) => {
                    print_order_response(&response, "BUY", &symbol, qty, limit.is_none());
                },
                Err(e) => {
                    eprintln!("Failed to place BUY order: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Sell { symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size } => {
            let trading = TradingService::new(config).await?;
            
            if limit.is_none() && slippage.is_some() {
                let slippage_pct = slippage.unwrap();
                if slippage_pct < 0.0 || slippage_pct > 0.1 {
                    eprintln!("Error: Slippage must be between 0% and 10% (0.0 to 0.1)");
                    std::process::exit(1);
                }
            }
            
            if let Some(ts) = tick_size {
                if ts <= 0.0 {
                    eprintln!("Error: Tick size must be greater than 0");
                    std::process::exit(1);
                }
                println!("Using custom tick size: {}", ts);
            }
            
            let order_type = if limit.is_some() { "LIMIT SELL" } else { "MARKET SELL" };
            println!("Placing {} order for {} {}", order_type, qty, symbol);
            
            let order_request = OrderRequest {
                symbol: symbol.clone(),
                is_buy: false,
                qty,
                limit_price: limit,
                leverage,
                reduce_only,
                tif,
            };
            
            match trading.place_order(order_request).await {
                Ok(response) => {
                    print_order_response(&response, "SELL", &symbol, qty, limit.is_none());
                },
                Err(e) => {
                    eprintln!("Failed to place SELL order: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Cancel { symbol, order_id } => {
            let trading = TradingService::new(config).await?;
            println!("Cancelling order {} for {}", order_id, symbol);
            
            match trading.cancel_order(&symbol, order_id).await {
                Ok(_) => {
                    println!("Order {} cancelled successfully", order_id);
                },
                Err(e) => {
                    eprintln!("Failed to cancel order: {}", e);
                    std::process::exit(1);
                }
            }
        },
    }
    
    Ok(())
}

fn print_order_response(response: &crate::types::OrderResponse, side: &str, symbol: &str, qty: f64, is_market: bool) {
    let order_type = if is_market { "MARKET" } else { "LIMIT" };
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║           ORDER CONFIRMATION         ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Type: {} {:<25} ║", order_type, side);
    println!("║ Symbol: {:<29} ║", symbol);
    println!("║ Quantity: {:<27.4} ║", qty);
    println!("║ Status: {:<29} ║", response.status);
    
    match &response.result {
        crate::types::OrderResult::Success { order_id, filled_qty, avg_price } => {
            println!("║ Order ID: {:<27} ║", order_id);
            if *filled_qty > 0.0 {
                println!("║ Filled: {:.4} @ ${:<15.4} ║", filled_qty, avg_price.unwrap_or(0.0));
                if is_market {
                    println!("║ Market order executed!            ║");
                }
            } else {
                if is_market {
                    println!("║ Market order awaiting fill         ║");
                } else {
                    println!("║ Limit order resting on book       ║");
                }
            }
        },
        crate::types::OrderResult::Error { message } => {
            println!("║ Error: {:<26} ║", message);
        },
        crate::types::OrderResult::Resting { order_id } => {
            println!("║ Order resting - ID: {:<15} ║", order_id);
            if is_market {
                println!("║ Market order resting (low liq)    ║");
            }
        }
    }
    println!("║ Timestamp: {:<23} ║", response.timestamp);
    println!("╚═══════════════════════════════════════╝");
    println!("Order submitted successfully!");
}

fn print_status(status: &crate::types::StatusResponse) {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                  HYPERLIQUID TESTNET STATUS                   ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Available Markets: {:<39} ║", status.total_markets);
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║{:<12} {:<12} {:<12} {:<12} {:<8} {:<12}║", 
        "SYMBOL", "MARK PRICE", "24H VOLUME", "FUNDING", "MAX LEV", "OPEN INT");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    
    for market in status.markets.iter().take(10) {
        println!("║{:<12} ${:<11.4} ${:<11.0} {:<11.6} {:<8}x ${:<11.0}║", 
            market.symbol,
            market.mark_price,
            market.volume_24h,
            market.funding_rate * 100.0,
            market.max_leverage,
            market.open_interest
        );
    }
    
    println!("╚═══════════════════════════════════════════════════════════════╝");
    if status.markets.len() > 10 {
        println!("... and {} more markets", status.markets.len() - 10);
    }
    println!("Status retrieved successfully!");
}

fn print_balances(balances: &crate::types::BalanceResponse) {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                        ACCOUNT SUMMARY                        ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Account Value: ${:<43.2} ║", balances.account_value);
    println!("║ Withdrawable: ${:<44.2} ║", balances.withdrawable);
    println!("║ Cross Margin Used: ${:<38.2} ║", balances.cross_margin_used);
    
    if !balances.positions.is_empty() {
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║                           POSITIONS                           ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║{:<8} {:<12} {:<12} {:<8} {:<12} {:<12}║", 
            "ASSET", "SIZE", "ENTRY PRICE", "LEVERAGE", "UNREALIZED", "VALUE");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        
        for pos in &balances.positions {
            let size_colored = if pos.size > 0.0 {
                format!("LONG {:.4}", pos.size)
            } else {
                format!("SHORT {:.4}", pos.size)
            };
            
            let pnl_colored = if pos.unrealized_pnl > 0.0 {
                format!("PROFIT ${:.2}", pos.unrealized_pnl)
            } else {
                format!("LOSS ${:.2}", pos.unrealized_pnl)
            };
            
            println!("║{:<8} {:<12} ${:<11.4} {:<8}x {:<12} ${:<11.2}║", 
                pos.symbol,
                size_colored,
                pos.entry_price,
                pos.leverage,
                pnl_colored,
                pos.position_value
            );
        }
    } else {
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║                     No open positions                         ║");
    }
    
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("Balances retrieved successfully!");
}

fn print_spot_markets(spot_data: &crate::types::SpotResponse) {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                         SPOT MARKETS                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Available Tokens: {:<43} ║", spot_data.tokens.len());
    println!("║ Trading Pairs: {:<46} ║", spot_data.pairs.len());
    
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║                            TOKENS                             ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║{:<12} {:<8} {:<20}║", "NAME", "DECIMALS", "TOKEN ID");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    
    for token in spot_data.tokens.iter().take(5) {
        let short_id = if token.token_id.len() > 20 {
            format!("{}...{}", &token.token_id[..8], &token.token_id[token.token_id.len()-6..])
        } else {
            token.token_id.clone()
        };
        
        println!("║{:<12} {:<8} {:<20}║", 
            token.name,
            token.decimals,
            short_id
        );
    }
    
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║                         TRADING PAIRS                         ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║{:<15} {:<12} {:<12} {:<12}║", 
        "PAIR", "MARK PRICE", "MID PRICE", "24H VOLUME");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    
    for pair in spot_data.pairs.iter().take(10) {
        println!("║{:<15} ${:<11.6} ${:<11.6} ${:<11.0}║", 
            pair.name,
            pair.mark_price,
            pair.mid_price,
            pair.volume_24h
        );
    }
    
    println!("╚═══════════════════════════════════════════════════════════════╝");
    if spot_data.pairs.len() > 10 {
        println!("... and {} more pairs", spot_data.pairs.len() - 10);
    }
    println!("Spot markets retrieved successfully!");
}
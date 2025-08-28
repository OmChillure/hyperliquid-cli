use axum::{
    routing::get,
    Router,
};
use clap::{Parser, Subcommand};
use tower_http::cors::CorsLayer;
use anyhow::Result;
use types::Config;

mod types;
mod services;
mod handlers;
mod cli;
mod config;

#[derive(Parser)]
#[command(name = "hl")]
#[command(about = "Hyperliquid Testnet Trader")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    server: bool,
    
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    Status,
    Balances,
    Spot,
    Stream {
        symbol: String,
        #[arg(short, long, default_value = "30")]
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.server {
        start_server(args.port).await
    } else {
        match args.command {
            Some(Commands::Status) => {
                let cli = cli::Cli { command: cli::Commands::Status };
                cli::run_cli(cli).await
            },
            Some(Commands::Balances) => {
                let cli = cli::Cli { command: cli::Commands::Balances };
                cli::run_cli(cli).await
            },
            Some(Commands::Spot) => {
                let cli = cli::Cli { command: cli::Commands::Spot };
                cli::run_cli(cli).await
            },
            Some(Commands::Stream { symbol, duration }) => {
                let cli = cli::Cli {
                    command: cli::Commands::Stream { symbol, duration }
                };
                cli::run_cli(cli).await
            },
            Some(Commands::Buy { symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size }) => {
                let cli = cli::Cli {
                    command: cli::Commands::Buy { 
                        symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size
                    }
                };
                cli::run_cli(cli).await
            },
            Some(Commands::Sell { symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size }) => {
                let cli = cli::Cli {
                    command: cli::Commands::Sell { 
                        symbol, qty, limit, leverage, reduce_only, tif, slippage, tick_size
                    }
                };
                cli::run_cli(cli).await
            },
            Some(Commands::Cancel { symbol, order_id }) => {
                let cli = cli::Cli {
                    command: cli::Commands::Cancel { symbol, order_id }
                };
                cli::run_cli(cli).await
            },
            None => {
                eprintln!("Please specify a command or use --server");
                eprintln!("Try 'hl --help' for more information.");
                eprintln!();
                eprintln!("Available commands:");
                eprintln!("  status                    - Get exchange status");
                eprintln!("  balances                  - Get account balances");
                eprintln!("  spot                      - Get spot markets");
                eprintln!("  buy <symbol> <qty>        - Place buy order");
                eprintln!("    --limit <price>         - Limit price (market order if not specified)");
                eprintln!("    --leverage <n>          - Leverage multiplier");
                eprintln!("    --reduce-only           - Reduce only order");
                eprintln!("    --tif <Gtc|Ioc|Alo>     - Time in force");
                eprintln!("    --slippage <pct>        - Slippage tolerance (0.01 = 1%)");
                eprintln!("    --tick-size <size>      - Custom price tick size");
                eprintln!("  sell <symbol> <qty>       - Place sell order (same options as buy)");
                eprintln!("  cancel <symbol> <id>      - Cancel order");
                eprintln!("  orders                    - List open orders");
                eprintln!("    --open                  - Show only open orders");
                eprintln!("  stream <symbol>           - Stream live trades");
                eprintln!("    --duration <secs>       - Stream duration (default: 30s)");
                eprintln!("  --server                  - Start HTTP API server");
                eprintln!("    --port <port>           - Server port (default: 8080)");
                std::process::exit(1);
            }
        }
    }
}

async fn start_server(port: u16) -> Result<()> {
    let config = Config::load()?;
    let exchange_service = services::ExchangeService::new(config)?;
    
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
        .route("/balances", get(handlers::get_balances))
        .route("/spot", get(handlers::get_spot_markets))
        .layer(CorsLayer::permissive())
        .with_state(exchange_service);
    
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    
    println!("Hyperliquid Server running on http://localhost:{}", port);
    println!("Available endpoints:");
    println!("   GET  /health       - Health check");
    println!("   GET  /status       - Exchange status");
    println!("   GET  /balances     - Account balances");
    println!("   GET  /spot         - Spot markets");
    println!();
    println!("Press Ctrl+C to stop the server");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
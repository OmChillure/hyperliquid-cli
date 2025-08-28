# Hyperliquid CLI Trading Bot
A cli and HTTP APIs for trading on Hyperliquid DEX.

## Features

- **CLI Trading**: Execute buy/sell orders directly from command line
- **Risk Management**: Configurable leverage and notional limits per symbol
- **Market Data**: Real-time trade streaming and market status
- **HTTP API**: RESTful endpoints for programmatic access
- **Order Management**: Support for limit, market, and reduce-only orders

## Video
[Demo Video](https://g2yvn1909f.ufs.sh/f/juQEJbMbBjvrFekz7li0QpgY5RL12W3V6AhcyvlESjxfPuze)

## Links
[Design Docs](https://docs.google.com/document/d/1zJNNnkXs5G7tCmoHRYJJrwZ3BwbM-dvSWa3FszC6MdI/edit?usp=sharing)

## Bonus 
Simple REST server (axum) exposing /health, /positions, /orders.
```bash
cargo run -- --server 
```

## Quick Start

### Installation
```bash
git clone <repository-url>
cargo build
```

### Environment Setup
Create `.env` file:
```bash
PRIVATE_KEY=
HYPERLIQUID_API_URL=https://api.hyperliquid-testnet.xyz
HYPERLIQUID_WS_URL=wss://api.hyperliquid-testnet.xyz/ws
```

## Command Reference
### Market Information

#### Exchange Status
```bash
cargo run status
```
Shows all available markets with prices, volumes, and limits.

#### Account Balances
```bash
cargo run balances
```
Displays account value, positions, and margin usage.

#### Spot Markets
```bash
cargo run spot
```
Lists available spot trading pairs and tokens.


### Trading Commands
#### Buy Orders
```bash
# Market buy
cargo run buy ETH 0.1

# Limit buy
cargo run buy ETH 0.1 --limit 2000

# Leveraged buy
cargo run buy ETH 0.1 --leverage 10 --limit 2000

# Advanced options
cargo run buy BTC 0.01 \
  --limit 45000 \
  --leverage 5 \
  --tif Ioc \
  --slippage 0.02
```


#### Sell Orders
```bash
# Market sell
cargo run sell ETH 0.1

# Limit sell
cargo run sell ETH 0.1 --limit 2100

# Close position (reduce-only)
cargo run sell ETH 0.1 --reduce-only
```

#### Order Management
```bash
# Cancel order
cargo run cancel ETH 12345678
```

### Data Streaming
```bash
# Stream trades (30s default)
cargo run stream ETH

# Custom duration
cargo run stream BTC --duration 120
```

### HTTP API Server
```bash
# Start server on port 8080
cargo run -- --server
```

## Trading Parameters

### Order Types
- **Market Orders**: Execute immediately at best available price
- **Limit Orders**: Execute only at specified price or better
- **Reduce-Only**: Close existing positions without opening new ones

### Time in Force
- **Gtc** (Good Till Cancelled): Active until filled or cancelled
- **Ioc** (Immediate or Cancel): Execute immediately or cancel
- **Alo** (Add Liquidity Only): Only if adding liquidity

### Risk Parameters [in config]
```
Symbol Limits:
- BTC: 10x max leverage, $50k max notional
- ETH: 15x max leverage, $30k max notional  
- SOL: 20x max leverage, $20k max notional

Global Limits:
- $10k max per order
- $25k max per symbol
```

## API Endpoints
Base URL: `http://localhost:8080`

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/status` | GET | Market status and trading pairs |
| `/balances` | GET | Account balances and positions |
| `/spot` | GET | Spot market information |


### Risk Management
Modify risk limits in `src/config/loader.rs`;
can configure more in laoder.rs

```rust
// Per-symbol limits
symbol_limits.insert("ETH".to_string(), SymbolLimits {
    max_leverage: 15,
    max_notional: 30_000.0,
    enabled: true,
});

// Global limits
RiskLimits {
    max_notional_per_order: 10_000.0,
    max_notional_per_symbol: 25_000.0,
    symbol_limits,
}
```

##Tests
- unit_test.rs : mocks data for risk policy decisions
- folder : tests/unit_test.rs

### Network Configuration
- **Testnet**: `https://api.hyperliquid-testnet.xyz`
  
## Examples
### Trading Strategies

#### Long Position
```bash
# Enter long ETH with 10x leverage
cargo run buy ETH 0.5 --leverage 10 --limit 2000

# Close position
cargo run sell ETH 0.7 --reduce-only --limit 2100

# Quick entry
cargo run buy ETH 0.1 --limit 2000 --tif Ioc

# Quick exit
cargo run sell ETH 0.1 --limit 2005 --tif Ioc
```


## Error Handling
### Common Errors

**Validation Errors**:
```
Error: Requested leverage 20x exceeds maximum 15x for ETH
Error: Order $20000.00 exceeds per-order limit $10000.00
Error: Trading disabled for symbol: UNKNOWN
```

**Connection Errors**:
```
Error: Failed to connect to Hyperliquid API
Error: WebSocket connection failed
```

**Order Errors**:
```
Error: Insufficient balance for order
Error: Order rejected by exchange
```

## Project Structure
```
src/
├── config/             # Configuration management
│   └── loader.rs       # Risk limits and settings
│   └── mod.rs          # Modules
├── handlers/           # HTTP API handlers
│   └── exchange_api.rs # API endpoints
│   └── mod.rs          # Modules
├── services/           # Core business logic
│   ├── exchange.rs     # Market data service
│   ├── trading.rs      # Order execution service
│   ├── mode.rs         # Modules
│   └── streaming.rs    # Real-time data streaming
├── types/              # Data structures
│   ├── api.rs          # API response types
│   ├── exchange.rs     # Exchange data types
│   ├── trading.rs      # Trading request types
│   ├── mode.rs         # Modules
│   ├── streaming.rs    # Streaming services
│   └── risk.rs         # Risk management types
├── cli.rs              # Command line interface
├── lib.rs              # module export for tests
└── main.rs             # Application entry point
tests/
├── unit_test/          # Unit test for risk policy decision
```

## Security note
- For now I have used ".env" to load variables like apis and stuff, in prod we can encrypt the apis keys before using it for the apis.
- comprehenisve audits and testing is must before making it live.\n
- Made sure that there is no logging of privatekeys.\n


## Dependencies
- **hyperliquid-rust-sdk**: Official Hyperliquid SDK
- **tokio**: Async runtime
- **axum**: HTTP server framework  
- **clap**: Command line parsing
- **serde**: Serialization framework
- **anyhow**: Error handling
- **tokio-tungstenite**: WebSocket client


## AI USE
- I have used AI to make the repsonse more visually appealing, like the boxes and stuff.
- Also used it while I was stuck in some errors especailly during making the trading work.
- hyperlqiuid rust sdk had no proper docs, so I used ai to extract definations and params of functions.


## NOTE :
- I have written the unit_test with help of ai, and iterated over that to make eveyrhting work fine, improts bugs etc and was not able to complete the integration_test I dont have much exp there but surely can learn. 


# MaxFun Contract Event Monitoring System

## 1. Project Description
MaxFun Event Monitor is a Rust-built backend system for blockchain event monitoring and processing. Core functionalities include:
- Contract event monitoring and processing
- User transaction tracking
- Token price and exchange rate updates
- Trading activity monitoring
- User balance and token summary management

## 2. Tech Stack
- **Programming Language**: Rust
- **Web Framework**: Axum
- **ORM Framework**: SeaORM
- **Smart Contract Integration**: Alloy
- **Database**: PostgreSQL
- **Cache**: Redis

## 3. Project Structure
```
.
├── data                # Data directory
│   └── abi            # Smart contract ABI files
├── src
│   ├── core           # Core configuration and functionality
│   │   ├── consts.rs  # Constant definitions
│   │   ├── pool.rs    # Database connection pool
│   │   └── mod.rs
│   ├── cron           # Scheduled tasks
│   │   ├── cron_price.rs  # Price update task
│   │   ├── cron_rate.rs   # Exchange rate update task
│   │   └── mod.rs
│   ├── entity         # Database entities
│   │   ├── db_evt_balance_log.rs   # Balance log
│   │   ├── db_evt_token_log.rs     # Token log
│   │   ├── db_evt_trade_log.rs     # Trade log
│   │   ├── db_evt_transfer_log.rs  # Transfer log
│   │   ├── db_evt_txn_log.rs       # Transaction log
│   │   ├── db_kline_5m.rs          # K-line data
│   │   ├── db_token_info.rs        # Token information
│   │   ├── db_user.rs              # User information
│   │   └── mod.rs
│   ├── evt            # Event handling
│   │   ├── evt.rs     # Event handling core
│   │   ├── evt_trade.rs # Trade event handling
│   │   └── mod.rs
│   ├── svc            # Service layer
│   │   ├── token.rs   # Token-related services
│   │   └── mod.rs
│   ├── util           # Utility functions
│   │   ├── error.rs   # Error handling
│   │   ├── log.rs     # Logging tools
│   │   ├── period.rs  # Time period handling
│   │   └── mod.rs
│   └── main.rs        # Program entry point
└── Cargo.toml         # Project dependency configuration
```

## 4. Core Business Logic

### 4.1 Contract Event Monitoring

The system monitors the following smart contract events:

1. Token issuance event (Launched)
   - Listens for new token issuance
   - Records token basic information: contract address, issuer, initial price, etc.
   - Updates `db_token_info` table
   - Creates initial token state log

2. Trade event (Bought/Sold)
   - Listens for token buying and selling operations
   - Processing flow:
     * Records trade log in `db_evt_trade_log`
     * Updates user balance `db_evt_balance_log`
     * Updates token summary information `db_token_summary`
     * Generates K-line data `db_kline_5m`
     * Calculates 24-hour trading volume and price changes

3. Initial purchase and update event (InitialBuyAndUpdate)
   - Listens for token's initial purchase
   - Updates token's initial price and liquidity information
   - Records trade log and updates user balance

4. Graduation event (Graduated)
   - Listens for token's graduation status changes
   - Updates token's graduation status information
   - Records status change log
   - Updates token summary information

Each event's handling includes the following common steps:
1. Event data validation and parsing
2. Transaction information recording (block height, timestamp, Gas, etc.)
3. Database transaction processing to ensure data consistency
4. Error handling and logging

The system scans events by block range, supporting:
- Real-time monitoring of the latest blocks
- Historical block scanning and data supplementation
- Automatic handling of forks and reorganization

### 4.2 Scheduled Tasks
The system uses `tokio-cron-scheduler` for periodic task processing:
1. Price update (every 10 minutes)
   ```rust
   "5 */10 * * * *" // Updates token prices from the oracle
   ```
2. Exchange rate update (daily)
   ```rust
   "5 0 * * * *" // Updates 24-hour price changes
   ```

### 4.3 Token Management
- Price oracle integration
- Balance tracking
- Supply monitoring
- Trading statistics

## 5. Development Environment Setup

### Prerequisites
- Rust (latest stable)
- PostgreSQL
- Redis
- Contract deployment addresses (for monitoring)

### Configuration
Environment variables are used for configuration (see `.env.example`):

```bash
# Logging and environment
RUST_BACKTRACE=1
RUST_LOG=info
APP_ENV=test

# Database
PG_URL=postgresql://localhost:5432/maxfun_dev
REDIS_URL=redis://localhost:6379/1

# Blockchain
PROVIDER=https://sepolia.base.org
INIT_BLOCK=21608205
FACTORY_CONTRACT_ADDR=0x1196285b248ba9b7760308bb991094f33de337da
```

1. Copy `.env.example` to `.env`
2. Modify configuration values according to your environment
3. Ensure database and Redis services are running

### Running the Application
1. Start monitoring service:
```bash
cargo run
```
2. The service will:
   - Initialize database connections
   - Start scheduled tasks
   - Begin contract event monitoring

use alloy::transports::TransportErrorKind;
// use ethers::middleware::SignerMiddleware;
// use ethers::prelude::{Http, LocalWallet, Provider};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("need environment variable: {0}")]
    BadEnv(#[from] std::env::VarError),

    #[error("{0}")]
    SeaOrmError(#[from] sea_orm::DbErr),

    #[error("format error: {0}")]
    FormatError(#[from] std::fmt::Error),

    #[error("parse int error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("ethers error: {0}")]
    AlloyEthersError(#[from] alloy::transports::RpcError<TransportErrorKind>),

    #[error("sol error: {0}")]
    AlloySolError(#[from] alloy::sol_types::Error),

    #[error("alloy hex error: {0}")]
    AlloyHexError(#[from] alloy::hex::FromHexError),

    #[error("contract error: {0}")]
    AlloyContractError(#[from] alloy::contract::Error),

    #[error("base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("units error: {0}")]
    AlloyUnitsError(#[from] alloy::primitives::utils::UnitsError),

    #[error("Parse error: {0}")]
    AlloyParseError(#[from] alloy::primitives::ruint::ParseError),

    #[error("ParseSignedError error: {0}")]
    ParseSignedError(#[from] alloy::primitives::ParseSignedError),

    // #[error("contract error: {0}")]
    // ContractError(#[from] ethers::contract::ContractError<SignerMiddleware<Provider<Http>, LocalWallet>>),
    //
    // #[error("chain error: {0}")]
    // ChainError(#[from] ethers::contract::ContractError<Provider<Http>>),
    //
    // #[error("abi error: {0}")]
    // AbiError(#[from] ethers::abi::Error),

    #[error("CronJob err: {0}")]
    CronJobError(#[from] tokio_cron_scheduler::JobSchedulerError),

    #[error("decimal err: {0}")]
    DecimalError(#[from] rust_decimal::Error),

    #[error("serde_json error: {0}")]
    SerdeJsonErr(#[from] serde_json::Error),

    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}
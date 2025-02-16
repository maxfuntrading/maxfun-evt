use std::sync::LazyLock;
use alloy::sol;

pub static PG_URL: LazyLock<String> = LazyLock::new(||
    std::env::var("PG_URL").expect("env not found PG_URL")
);
pub static REDIS_URL: LazyLock<String> = LazyLock::new(||
    std::env::var("REDIS_URL").expect("env not found REDIS_URL")
);

/// bsc rpc provider
pub static PROVIDER: LazyLock<String> = LazyLock::new(||
    std::env::var("PROVIDER").expect("env not found PROVIDER")
);


pub static FACTORY_CONTRACT_ADDR: LazyLock<String> = LazyLock::new(||
    std::env::var("FACTORY_CONTRACT_ADDR").expect("env not found FACTORY_CONTRACT_ADDR")
);

pub static INIT_BLOCK: LazyLock<u64> = LazyLock::new(||
    std::env::var("INIT_BLOCK").expect("env not found INIT_BLOCK").parse().expect("parse error INIT_BLOCK")
);


// pub const FACTORY_ABI_FILE: &str = "data/abi/MaxFunFactory.json";
// pub const MANAGER_ABI_FILE: &str = "data/abi/MaxFunManager.json";
// pub const ERC20_ABI_FILE: &str = "data/abi/ERC20.json";
pub const GAP_BLOCK: u64 = 5;
pub const POLL_INTERVAL: u64 = 2;
pub const MAX_BLOCK_RANGE: u64 = 10000;
pub const PK_BLOCK_NUM: &str = "block_num";


sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "data/abi/Token.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    FACTORY,
    "data/abi/Factory-0216.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ORACLE,
    "data/abi/Oracle.json"
);
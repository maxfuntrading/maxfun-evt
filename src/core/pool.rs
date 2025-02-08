use std::time::Duration;
use redis::{Client};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;

use super::consts;

pub type DB = DatabaseConnection;
pub type RedisPool = Client;

#[derive(Clone, Debug)]
pub struct Store {
    pub db_pool: DB,
    pub redis_pool: RedisPool,
}


// 初始化所有连接池
pub async fn init_pool() -> Store {
    let db_pool = create_db_pool().await;
    let redis_pool = create_redis_pool().await;
    Store {
        db_pool,
        redis_pool,
    }
}

pub async fn create_db_pool() -> DB {
    let mut opt = ConnectOptions::new(consts::PG_URL.as_str());
    opt.max_connections(20)
        .min_connections(3)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info);

    Database::connect(opt).await.expect("could not create db_pool due to")
}


// redis 连接池
pub async fn create_redis_pool() -> RedisPool {
    let pool = Client::open(consts::REDIS_URL.as_str()).expect("could not create redis client");
    pool
}
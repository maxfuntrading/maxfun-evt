use std::time::Duration;

use alloy::eips::BlockId;
use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::{Filter, RawLog};
use alloy::sol_types::SolEvent;
use redis::AsyncCommands;
use rust_decimal::Decimal;
use sea_orm::prelude::Expr;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
};

use super::evt_trade::handle_trade;
use crate::core::{consts, Store};
use crate::entity::*;
use crate::svc::TOKEN;
use crate::util::{LibError, LibResult, PeriodType};


pub struct Evt {
    store: Store,
    provider: RootProvider<Ethereum>,
    factory_contract: Address,
}

impl Evt {
    pub fn new(store: Store) -> Self {
        let url = consts::PROVIDER
            .parse()
            .expect("Could not parse the Provider URL");
        let factory_contract = consts::FACTORY_CONTRACT_ADDR
            .as_str()
            .parse()
            .expect("Could not parse the Factory Contract Address");
        let provider = RootProvider::new_http(url);

        Self {
            store,
            provider,
            factory_contract,
        }
    }

    pub async fn run(&self) -> LibResult<()> {
        let start_block = self.get_block().await?;
        let (mut start_block, mut latest_block) = self.block_range(start_block).await;
        // catch up block
        if latest_block - start_block >= consts::GAP_BLOCK {
            tracing::info!("catch up block. start block: {start_block}, end block: {latest_block}");
            loop {
                if let Err(e) = self.filter(start_block, latest_block).await {
                    tracing::error!("filter err: {}", e);
                    continue;
                }
                (start_block, latest_block) = self.block_range(latest_block).await;
                if latest_block - start_block < consts::GAP_BLOCK {
                    tracing::info!("catch up block success!");
                    break;
                }
            }
        }
        // start filter
        self.run_filter(start_block, latest_block).await?;
        Ok(())
    }

    async fn run_filter(&self, mut start_block: u64, mut latest_block: u64) -> LibResult<()> {
        loop {
            if let Err(e) = self.filter(start_block, latest_block).await {
                tracing::error!("filter err: {}", e);
                continue;
            }
            (start_block, latest_block) = self.block_range(latest_block).await;
        }
    }

    async fn filter(&self, start_block: u64, latest_block: u64) -> LibResult<()> {
        tracing::info!("start block: {start_block}, end block: {latest_block}");
        let addrs = vec![self.factory_contract];

        let filter = Filter::new()
            .address(addrs)
            .from_block(start_block)
            .to_block(latest_block)
            .events(vec![
                // "Transfer(address,address,uint256)",
                "Launched(address,address,address,uint256,uint256)",
                "InitialBuyAndUpdate(address,address,uint256,uint256,uint256)",
                "Sold(address,address,uint256,uint256,uint256)",
                "Bought(address,address,uint256,uint256,uint256)",
                "Graduated(address,address)",
            ]);
        let logs = match self.provider.get_logs(&filter).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("get logs err={e}");
                return Err(LibError::AlloyEthersError(e));
            }
        };

        for log in logs.iter() {
            let block_number = log.block_number.unwrap();
            let block_info = self
                .provider
                .get_block(BlockId::from(block_number), Default::default())
                .await?;
            let block_time = block_info.unwrap().header.timestamp as i64;
            let topic = format!("{:#x}", log.topic0().unwrap());
            let txn_hash = format!("{:#x}", log.transaction_hash.unwrap());
            let txn_model = db_evt_txn_log::Model {
                block_number: block_number as i64,
                block_time,
                txn_hash: txn_hash.clone(),
                txn_index: log.transaction_index.unwrap() as i64,
                log_index: log.log_index.unwrap() as i64,
                address: format!("{:#x}", log.address()),
                topic_0: topic.clone(),
                topic_1: log.topics().get(1).map(|v| format!("{:#x}", v)),
                topic_2: log.topics().get(2).map(|v| format!("{:#x}", v)),
                topic_3: log.topics().get(3).map(|v| format!("{:#x}", v)),
                data: Some(format!("{:#x}", log.data().data)),
            };

            let raw_log = RawLog {
                address: log.address(),
                topics: log.topics().to_vec(),
                data: log.data().clone().data,
            };
            match topic.as_str() {
                "0xec774f0683e9ac48e8d835f412f9f877a8a5dee9af3170d78cf3ef33149d15e7" => {
                    // launched
                    if let Err(e) = self.handle_launched_evt(raw_log, txn_model).await {
                        tracing::error!("handle_launched_evt. txn_hash={txn_hash}, err={e}")
                    }
                }
                "0x1685b8781b8be9c9242e31a14f2ca289c99bf831d0ad45bf23613f6e646e480d" => {
                    // initial buy and update
                    if let Err(e) = self
                        .handle_initial_buy_and_update_evt(raw_log, txn_model)
                        .await
                    {
                        tracing::error!(
                            "handle_initial_buy_and_update_evt. txn_hash={txn_hash}, err={e}"
                        )
                    }
                }
                "0x9be8a5ca22b7e6e81f04b5879f0248227bb770114291bd47dfaee4c3a82ad60e" => {
                    // sold
                    if let Err(e) = self.handle_sold_evt(raw_log, txn_model).await {
                        tracing::error!("handle_sold_evt. txn_hash={txn_hash}, err={e}")
                    }
                }
                "0x7ce543d1780f3bdc3dac42da06c95da802653cd1b212b8d74ec3e3c33ad7095c" => {
                    // bought
                    if let Err(e) = self.handle_bought_evt(raw_log, txn_model).await {
                        tracing::error!("handle_bought_evt. txn_hash={txn_hash}, err={e}")
                    }
                }
                "381d54fa425631e6266af114239150fae1d5db67bb65b4fa9ecc65013107e07e" => {
                    // graduated
                    if let Err(e) = self.handle_graduated_evt(raw_log, txn_model).await {
                        tracing::error!("handle_graduated_evt. txn_hash={txn_hash}, err={e}")
                    }
                }
                _ => tracing::warn!("unknown evt {txn_model:#?}"),
            }
        }
        self.set_block(latest_block).await?;
        Ok(())
    }

    async fn block_range(&self, latest_block: u64) -> (u64, u64) {
        let mut new_num;
        loop {
            tokio::time::sleep(Duration::new(consts::POLL_INTERVAL, 0)).await;
            new_num = match self.provider.get_block_number().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("get block number err={e}");
                    continue;
                }
            };
            if new_num > latest_block {
                break;
            }
        }
        if new_num - latest_block > consts::MAX_BLOCK_RANGE {
            new_num = latest_block + consts::MAX_BLOCK_RANGE;
        }
        (latest_block + 1, new_num)
    }

    async fn get_block(&self) -> LibResult<u64> {
        let mut conn = self
            .store
            .redis_pool
            .get_multiplexed_async_connection()
            .await?;
        let re: Option<u64> = conn.get(consts::PK_BLOCK_NUM).await?;
        match re {
            Some(v) => Ok(v),
            None => {
                self.set_block(*consts::INIT_BLOCK).await?;
                Ok(*consts::INIT_BLOCK)
            }
        }
    }

    async fn set_block(&self, block: u64) -> LibResult<()> {
        let mut conn = self
            .store
            .redis_pool
            .get_multiplexed_async_connection()
            .await?;
        conn.set::<_, _, ()>(consts::PK_BLOCK_NUM, block).await?;
        Ok(())
    }

    async fn handle_launched_evt(
        &self,
        raw_log: RawLog,
        txn_model: db_evt_txn_log::Model,
    ) -> LibResult<()> {
        let data = consts::FACTORY::Launched::decode_raw_log(raw_log.topics, raw_log.data.as_ref(), true)?;
        let token = format!("{:#x}", data.token);
        let asset = format!("{:#x}", data.asset);
        let pair = format!("{:#x}", data.pair);
        let id = data.id.to::<i64>();
        let total_supply = TOKEN.total_supply(&token).await?;
        // 1. update token_info
        // 2. insert txn_log
        // 3. insert evt_token_log
        let token_log_model = db_evt_token_log::ActiveModel {
            block_number: Set(txn_model.block_number),
            txn_index: Set(txn_model.txn_index),
            log_index: Set(txn_model.txn_index),
            block_time: Set(txn_model.block_time),
            txn_hash: Set(txn_model.txn_hash.clone()),
            token_address: Set(token.clone()),
            raised_address: Set(asset.clone()),
            pair_address: Set(pair.clone()),
            token_id: Set(id),
        };
        let token_summary_model = db_token_summary::ActiveModel {
            token_address: Set(token.clone()),
            raised_token: Set(asset.clone()),
            pair_address: Set(pair.clone()),
            price: Set(Decimal::ZERO),
            price_token: Set(Decimal::ZERO),
            price_rate24h: Set(Decimal::ZERO),
            volume_24h: Set(Decimal::ZERO),
            total_supply: Set(total_supply),
            market_cap: Set(Decimal::ZERO),
            liquidity_token: Set(Decimal::ZERO),
            liquidity: Set(Decimal::ZERO),
            bonding_curve: Set(Decimal::ZERO),
            uniswap_pool: Set("".to_string()),
            last_trade_ts: Set(txn_model.block_time),
        };
        let open_ts = PeriodType::M5.open_ts(txn_model.block_time);
        let close_ts = PeriodType::M5.close_ts(open_ts);
        let kline_model = db_kline_5m::ActiveModel {
            token_address: Set(token.clone()),
            open_ts: Set(open_ts),
            close_ts: Set(close_ts),
            high: Set(Decimal::ZERO),
            low: Set(Decimal::ZERO),
            open: Set(Decimal::ZERO),
            close: Set(Decimal::ZERO),
            volume: Set(Decimal::ZERO),
            amount: Set(Decimal::ZERO),
            txn_num: Set(0),
        };
        let tx = self.store.db_pool.begin().await?;
        db_token_info::Entity::update_many()
            .filter(db_token_info::Column::Id.eq(id))
            .col_expr(db_token_info::Column::TokenAddress, Expr::value(token))
            .col_expr(
                db_token_info::Column::LaunchTs,
                Expr::value(txn_model.block_time),
            )
            .exec(&tx)
            .await?;
        token_log_model.insert(&tx).await?;
        token_summary_model.insert(&tx).await?;
        kline_model.insert(&tx).await?;
        txn_model.into_active_model().insert(&tx).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn handle_initial_buy_and_update_evt(
        &self,
        raw_log: RawLog,
        txn_model: db_evt_txn_log::Model,
    ) -> LibResult<()> {
        let data = consts::FACTORY::InitialBuyAndUpdate::decode_raw_log(
            raw_log.topics,
            raw_log.data.as_ref(),
            true,
        )?;
        let (user, token) = (format!("{:#x}", data.user), format!("{:#x}", data.token));
        handle_trade(
            &self.store,
            user,
            token,
            data.amountIn,
            data.amountOut,
            data.price,
            0,
            txn_model,
        )
        .await?;

        Ok(())
    }

    async fn handle_bought_evt(
        &self,
        raw_log: RawLog,
        txn_model: db_evt_txn_log::Model,
    ) -> LibResult<()> {
        let data = consts::FACTORY::Bought::decode_raw_log(raw_log.topics, raw_log.data.as_ref(), true)?;

        let (user, token) = (format!("{:#x}", data.user), format!("{:#x}", data.token));
        handle_trade(
            &self.store,
            user,
            token,
            data.amountIn,
            data.amountOut,
            data.price,
            0,
            txn_model,
        )
        .await?;
        Ok(())
    }

    async fn handle_sold_evt(
        &self,
        raw_log: RawLog,
        txn_model: db_evt_txn_log::Model,
    ) -> LibResult<()> {
        let data = consts::FACTORY::Sold::decode_raw_log(raw_log.topics, raw_log.data.as_ref(), true)?;

        let (user, token) = (format!("{:#x}", data.user), format!("{:#x}", data.token));
        handle_trade(
            &self.store,
            user,
            token,
            data.amountIn,
            data.amountOut,
            data.price,
            1,
            txn_model,
        )
        .await?;
        Ok(())
    }

    async fn handle_graduated_evt(
        &self,
        raw_log: RawLog,
        txn_model: db_evt_txn_log::Model,
    ) -> LibResult<()> {
        let data = consts::FACTORY::Graduated::decode_raw_log(raw_log.topics, raw_log.data.as_ref(), true)?;
        let token = format!("{:#x}", data.token);
        let uniswap_pool = format!("{:#x}", data.uniswapV2Pair);

        // 1. update token_info
        // 2. update token_summary
        let tx = self.store.db_pool.begin().await?;
        db_token_info::Entity::update_many()
            .filter(db_token_info::Column::TokenAddress.eq(&token))
            .col_expr(db_token_info::Column::IsLaunched, Expr::value(true))
            .col_expr(
                db_token_info::Column::LaunchTs,
                Expr::value(txn_model.block_time),
            )
            .exec(&tx)
            .await?;
        db_token_summary::Entity::update_many()
            .filter(db_token_summary::Column::TokenAddress.eq(&token))
            .col_expr(
                db_token_summary::Column::UniswapPool,
                Expr::value(uniswap_pool),
            )
            .exec(&tx)
            .await?;
        txn_model.into_active_model().insert(&tx).await?;
        tx.commit().await?;

        Ok(())
    }
}

use std::ops::{Div, Sub};
use std::str::FromStr;

use alloy::primitives::utils::{format_ether, ParseUnits, Unit};
use alloy::primitives::U256;
use rust_decimal::Decimal;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};

use crate::core::Store;
use crate::entity::*;
use crate::svc::TOKEN;
use crate::util::PeriodType;
use crate::util::{LibError, LibResult};

/// Handles a trade event by updating various database tables with trade information.
///
/// This function processes a trade event, updating user summaries, token summaries,
/// and creating trade log entries. It handles both buy and sell operations.
///
/// # Arguments
///
/// * `store` - A reference to the data store
/// * `user` - The address of the user involved in the trade
/// * `token` - The address of the token being traded
/// * `amount_in` - The amount of tokens or currency input for the trade
/// * `amount_out` - The amount of tokens or currency output from the trade
/// * `price` - The price at which the trade occurred
/// * `trade_type` - An integer representing the type of trade (0 for buy, 1 for sell)
/// * `txn_model` - The transaction model containing additional transaction information
///
/// # Returns
///
/// Returns a `LibResult<()>` which is `Ok(())` if the operation succeeds, or an error if it fails.
pub async fn handle_trade(
    store: &Store,
    user: String,
    token: String,
    amount_in: U256,
    amount_out: U256,
    price: U256,
    trade_type: i32,
    txn_model: db_evt_txn_log::Model,
) -> LibResult<()> {
    // get raised token info
    let (raised_decimal, raised_address, oracle_address) = db_token_summary::Entity::find()
        .filter(db_token_summary::Column::TokenAddress.eq(token.clone()))
        .inner_join(db_raised_token::Entity)
        .select_only()
        .column(db_raised_token::Column::Decimal)
        .column(db_raised_token::Column::Address)
        .column(db_raised_token::Column::Oracle)
        .into_tuple::<(i32, String, String)>()
        .one(&store.db_pool)
        .await?
        .ok_or_else(|| LibError::InternalError("".into()))?;

    // convert amount to decimal
    let unit = Unit::new(raised_decimal as u8).unwrap();
    let (amount0, amount1) = if trade_type == 0 {
        let amount0 = Decimal::from_str(&format_ether(amount_out))?;
        let value = ParseUnits::from(amount_in).format_units(unit);
        let amount1 = Decimal::from_str(&value)?;
        (amount0, amount1)
    } else {
        let amount0 = Decimal::from_str(&format_ether(amount_in))?;
        let value = ParseUnits::from(amount_out).format_units(unit);
        let amount1 = Decimal::from_str(&value)?;
        (amount0, amount1)
    };

    let price_value = Decimal::from_str(&format_ether(price))?;
    let oracle_price = TOKEN.oracle_price(&oracle_address).await?;
    let price_usd = price_value * oracle_price;

    let user_balance = TOKEN.balance_of(&token, &user).await?;

    //     // 1. insert evt_trade_log
    //     // 2. update user_summary
    //     // 3. update token_summary
    //     // 4. update kline_5m
    let trade_log_model = db_evt_trade_log::Model {
        block_number: txn_model.block_number,
        txn_index: txn_model.txn_index,
        log_index: txn_model.log_index,
        block_time: txn_model.block_time,
        txn_hash: txn_model.txn_hash.clone(),
        token_address: token.clone(),
        user_address: user.clone(),
        trade_type,
        token0: token.clone(),
        amount0,
        token1: raised_address.clone(),
        amount1,
        price: price_usd,
        price_token: price_value,
    };

    let user_summary_model = db_user_summary::ActiveModel {
        user_address: Set(user.clone()),
        token_address: Set(token.clone()),
        amount: Set(user_balance),
        update_ts: Set(txn_model.block_time),
    };

    let user_onconflict = OnConflict::columns([
        db_user_summary::Column::UserAddress,
        db_user_summary::Column::TokenAddress,
    ])
    .update_column(db_user_summary::Column::Amount)
    .update_column(db_user_summary::Column::UpdateTs)
    .to_owned();

    let tx = store.db_pool.begin().await?;
    txn_model.into_active_model().insert(&tx).await?;

    handle_kline_5m(&tx, &trade_log_model, PeriodType::M5).await?;
    handle_token_summary(&tx, &trade_log_model).await?;
    trade_log_model.into_active_model().insert(&tx).await?;
    db_user_summary::Entity::insert(user_summary_model)
        .on_conflict(user_onconflict)
        .exec(&tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

/// Update the token summary in the database based on the latest trade log
/// 
/// This function performs the following operations:
/// 1. Calculates the 24-hour volume and price change
/// 2. Updates the token's price, volume, and other related metrics
/// 3. Handles conflicts by updating specific columns if the token already exists
///
/// Parameters:
/// - tx: A reference to the database transaction
/// - exchange: A reference to the trade log model containing the latest trade information
///
/// Returns:
/// - A LibResult indicating success or failure of the operation
async fn handle_token_summary(
    tx: &DatabaseTransaction,
    exchange: &db_evt_trade_log::Model,
) -> LibResult<()> {
    let now_ts = chrono::Utc::now().timestamp();
    let start_ts = now_ts - now_ts % 300;
    let end_ts = start_ts - 3600 * 24;

    let last_kline = db_kline_5m::Entity::find()
        .filter(db_kline_5m::Column::TokenAddress.eq(&exchange.token_address))
        .filter(db_kline_5m::Column::OpenTs.lte(end_ts))
        .order_by_desc(db_kline_5m::Column::OpenTs)
        .limit(1)
        .one(tx)
        .await?;

    let volume_24h: Decimal = db_kline_5m::Entity::find()
        .filter(db_kline_5m::Column::TokenAddress.eq(&exchange.token_address))
        .filter(db_kline_5m::Column::OpenTs.gte(end_ts))
        .select_only()
        .column_as(db_kline_5m::Column::Volume.sum(), "volume")
        .into_tuple::<Option<Decimal>>()
        .one(tx)
        .await?
        .unwrap_or(Some(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO)
        + exchange.amount0;

    let last_price = if let Some(kline) = last_kline {
        kline.close
    } else {
        db_kline_5m::Entity::find()
            .filter(db_kline_5m::Column::TokenAddress.eq(&exchange.token_address))
            .order_by_asc(db_kline_5m::Column::OpenTs)
            .limit(1).one(tx).await?.unwrap().close
    };

    let last_price = if last_price == Decimal::ZERO {
        Decimal::new(1, 18)
    } else {
        last_price
    };

    let rate_24h = exchange.price.sub(last_price).div(last_price);
    let (bonding_curve, liquidity_token) = TOKEN.curve_process(&exchange.token_address).await?;
    let liquidity = liquidity_token * exchange.price;

    db_token_summary::Entity::update_many()
        .filter(db_token_summary::Column::TokenAddress.eq(&exchange.token_address))
        .col_expr(db_token_summary::Column::Volume24h, Expr::value(volume_24h))
        .col_expr(db_token_summary::Column::Price, Expr::value(exchange.price))
        .col_expr(
            db_token_summary::Column::PriceToken,
            Expr::value(exchange.price_token),
        )
        .col_expr(
            db_token_summary::Column::PriceRate24h,
            Expr::value(rate_24h),
        )
        .col_expr(
            db_token_summary::Column::BondingCurve,
            Expr::value(bonding_curve),
        )
        .col_expr(
            db_token_summary::Column::LiquidityToken,
            Expr::value(liquidity_token),
        )
        .col_expr(db_token_summary::Column::Liquidity, Expr::value(liquidity))
        .col_expr(
            db_token_summary::Column::MarketCap,
            Expr::col(db_token_summary::Column::TotalSupply).mul(exchange.price),
        )
        .col_expr(
            db_token_summary::Column::LastTradeTs,
            Expr::value(exchange.block_time),
        )
        .exec(tx)
        .await?;

    Ok(())
}

/// handle one trade log, do the following:
/// 1. update user_summary
/// 2. update token_summary
/// 3. update kline_5m
async fn handle_kline_5m(
    tx: &DatabaseTransaction,
    exchange: &db_evt_trade_log::Model,
    period: PeriodType,
) -> LibResult<()> {
    let open_ts = period.open_ts(exchange.block_time);
    let close_ts = period.close_ts(open_ts);

    let kline = db_kline_5m::ActiveModel {
        token_address: Set(exchange.token_address.clone()),
        open_ts: Set(open_ts),
        close_ts: Set(close_ts),
        high: Set(exchange.price),
        low: Set(exchange.price),
        open: Set(exchange.price),
        close: Set(exchange.price),
        volume: Set(exchange.amount0),
        amount: Set(exchange.amount0 * exchange.price),
        txn_num: Set(1),
    };

    let conflict = OnConflict::columns([
        db_kline_5m::Column::TokenAddress,
        db_kline_5m::Column::OpenTs,
    ])
    .update_column(db_kline_5m::Column::Close)
    .value(
        db_kline_5m::Column::High,
        Expr::case(
            Expr::col((db_kline_5m::Entity, db_kline_5m::Column::High)).lt(exchange.price),
            Expr::val(exchange.price),
        )
        .finally(Expr::col((db_kline_5m::Entity, db_kline_5m::Column::High))),
    )
    .value(
        db_kline_5m::Column::Low,
        Expr::case(
            Expr::col((db_kline_5m::Entity, db_kline_5m::Column::Low)).gt(exchange.price),
            Expr::val(exchange.price),
        )
        .finally(Expr::col((db_kline_5m::Entity, db_kline_5m::Column::Low))),
    )
    .value(
        db_kline_5m::Column::Volume,
        Expr::col((db_kline_5m::Entity, db_kline_5m::Column::Volume))
            .add(Expr::val(exchange.amount0)),
    )
    .value(
        db_kline_5m::Column::Amount,
        Expr::col((db_kline_5m::Entity, db_kline_5m::Column::Amount))
            .add(Expr::val(exchange.amount0 * exchange.price)),
    )
    .value(
        db_kline_5m::Column::TxnNum,
        Expr::col((db_kline_5m::Entity, db_kline_5m::Column::TxnNum)).add(Expr::val(1)),
    )
    .to_owned();

    db_kline_5m::Entity::insert(kline)
        .on_conflict(conflict)
        .exec(tx)
        .await?;

    Ok(())
}

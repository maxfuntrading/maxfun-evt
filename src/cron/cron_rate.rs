use std::ops::{Div, Sub};

use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, QuerySelect, QueryFilter, QueryOrder};
use sea_orm::prelude::Expr;

use crate::core::Store;
use crate::entity::*;
use crate::util::LibResult;

pub struct CronRate {
    store: Store,
}

impl CronRate {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub async fn run(&self) -> LibResult<()> {
        let tokens = db_token_summary::Entity::find()
            .select_only()
            .column_as(db_token_summary::Column::TokenAddress, "token")
            .column_as(db_token_summary::Column::PriceToken, "price")
            .into_tuple::<(String, Decimal)>()
            .all(&self.store.db_pool)
            .await?;

        for (token, price) in tokens {
            self.handle_token(&token, price).await?;
        }

        Ok(())
    }

    async fn handle_token(&self, token: &str, price: Decimal) -> LibResult<()> {
        let now_ts = chrono::Utc::now().timestamp();
        let start_ts = now_ts - now_ts % 300;
        let end_ts = start_ts - 3600 * 24;
        let last_kline = db_kline_5m::Entity::find()
            .filter(db_kline_5m::Column::TokenAddress.eq(token))
            .filter(db_kline_5m::Column::OpenTs.lte(end_ts))
            .order_by_desc(db_kline_5m::Column::OpenTs)
            .limit(1)
            .one(&self.store.db_pool)
            .await?;
        let volume_24h: Decimal = db_kline_5m::Entity::find()
            .filter(db_kline_5m::Column::TokenAddress.eq(token))
            .filter(db_kline_5m::Column::OpenTs.gte(end_ts))
            .select_only()
            .column_as(db_kline_5m::Column::Amount.sum(), "volume")
            .into_tuple::<Option<Decimal>>()
            .one(&self.store.db_pool)
            .await?
            .unwrap_or(Some(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO);

        let last_price = if let Some(kline) = last_kline {
            kline.close
        } else {
            db_kline_5m::Entity::find()
                .filter(db_kline_5m::Column::TokenAddress.eq(token))
                .order_by_asc(db_kline_5m::Column::OpenTs)
                .limit(1).one(&self.store.db_pool).await?.unwrap().close
        };

        let last_price = if last_price == Decimal::ZERO {
            Decimal::new(1, 18)
        } else {
            last_price
        };


        let rate_24h = price.sub(last_price).div(last_price);
        db_token_summary::Entity::update_many()
            .filter(db_token_summary::Column::TokenAddress.eq(token))
            .col_expr(db_token_summary::Column::Volume24h, Expr::value(volume_24h))
            .col_expr(
                db_token_summary::Column::PriceRate24h,
                Expr::value(rate_24h),
            )
            .exec(&self.store.db_pool)
            .await?;
        Ok(())
    }
}

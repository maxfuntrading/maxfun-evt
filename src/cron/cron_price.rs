use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::core::Store;
use crate::entity::*;
use crate::svc::TOKEN;
use crate::util::LibResult;

pub struct CronPrice {
    store: Store,
}

impl CronPrice {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub async fn run(&self) -> LibResult<()> {
        tracing::info!("cron price start");
        let tokens = db_raised_token::Entity::find()
            .all(&self.store.db_pool)
            .await?;
        for token in tokens {
            self.handle_token(&token.address, &token.oracle).await?;
        }
        tracing::info!("cron price end");
        Ok(())
    }

    async fn handle_token(&self, token: &str, oracle: &str) -> LibResult<()> {
        let price = TOKEN.oracle_price(oracle).await?;
        db_token_summary::Entity::update_many()
            .filter(db_token_summary::Column::RaisedToken.eq(token))
            .col_expr(
                db_token_summary::Column::Price,
                Expr::col(db_token_summary::Column::PriceToken).mul(price),
            )
            .col_expr(
                db_token_summary::Column::MarketCap,
                Expr::col(db_token_summary::Column::TotalSupply).mul(price),
            )
            .col_expr(
                db_token_summary::Column::Liquidity,
                Expr::col(db_token_summary::Column::LiquidityToken).mul(price),
            )
            .exec(&self.store.db_pool)
            .await?;

        db_raised_token::Entity::update_many()
            .filter(db_raised_token::Column::Address.eq(token))
            .col_expr(db_raised_token::Column::Price, Expr::value(price))
            .exec(&self.store.db_pool)
            .await?;

        Ok(())
    }
}

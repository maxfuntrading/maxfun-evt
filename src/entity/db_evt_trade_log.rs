use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "evt_trade_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub block_number: i64,
    #[sea_orm(primary_key)]
    pub txn_index: i64,
    #[sea_orm(primary_key)]
    pub log_index: i64,
    pub block_time: i64,
    pub txn_hash: String,
    pub token_address: String,
    pub user_address: String,
    pub trade_type: i32,
    pub token0: String,
    pub amount0: Decimal,
    pub token1: String,
    pub amount1: Decimal,
    pub price: Decimal,
    pub price_token: Decimal
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// impl Entity {
//     pub async fn find_latest_trades(db: &DatabaseConnection) -> LibResult<Vec<(String, i32, String, Decimal, String, String, Option<String>)>> {
//         let stmt = Statement::from_string(
//             db.get_database_backend(),
//             r#"
//             SELECT
//                 etl.user_address,
//                 etl.trace_type,
//                 etl.token_address,
//                 etl.amount1 as amount,
//                 ti.icon,
//                 ti.symbol,
//                 ti.tag
//             FROM evt_trade_log etl
//             LEFT JOIN token_info ti ON etl.token_address = ti.token_address
//             ORDER BY etl.block_time DESC
//             LIMIT 100
//             "#.to_owned(),
//         );
//
//         let rows = db.query_all(stmt).await?;
//
//         let trades = rows
//             .into_iter()
//             .map(|row| {
//                 Ok((
//                     row.try_get::<String>("", "user_address")?,
//                     row.try_get::<i32>("", "trace_type")?,
//                     row.try_get::<String>("", "token_address")?,
//                     row.try_get::<Decimal>("", "amount")?,
//                     row.try_get::<String>("", "icon")?,
//                     row.try_get::<String>("", "symbol")?,
//                     row.try_get::<Option<String>>("", "tag")?,
//                 ))
//             })
//             .collect::<Result<Vec<_>, DbErr>>()?;
//
//         Ok(trades)
//     }
// }
use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "evt_balance_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub block_number: i64,
    #[sea_orm(primary_key)]
    pub txn_index: i64,
    #[sea_orm(primary_key)]
    pub log_index: i64,
    #[sea_orm(primary_key)]
    pub user_address: String,
    pub token_address: String,
    pub block_time: i64,
    pub txn_hash: String,
    pub delta_amount: Decimal,
    pub total_amount: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {} 
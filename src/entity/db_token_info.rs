use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "token_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub token_address: String,
    pub user_address: String,// 代币创建者
    pub name: String,// 代币全称
    pub icon: String,
    pub symbol: String, // 代币缩写
    pub description: String,
    pub tag: String,
    pub website: String,
    pub twitter: String,
    pub telegram: String,
    pub total_supply: Decimal,
    pub raised_token: String,
    pub raised_amount: Decimal,
    pub sale_ratio: Decimal,
    pub reserved_ratio: Decimal,
    pub pool_ratio: Decimal,
    pub launch_ts: i64, // 代币开启外盘时间
    pub create_ts: i64, // 代币创建时间
    pub is_launched: bool, // 代币是否开启外盘
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::db_token_summary::Entity")]
    TokenSummary,
}

impl Related<super::db_token_summary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TokenSummary.def()
    }
}

impl ActiveModelBehavior for ActiveModel {} 
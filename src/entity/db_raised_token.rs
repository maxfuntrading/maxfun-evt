use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "raised_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimal: i32,
    pub icon: String,
    pub oracle: String,
    pub price: Decimal,
    pub create_ts: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::db_token_summary::Entity")]
    TokenSummary,
}

impl Related<super::db_token_summary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TokenSummary.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
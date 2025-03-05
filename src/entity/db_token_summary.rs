use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "token_summary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub token_address: String,
    pub raised_token: String,
    pub pair_address: String,
    pub price: Decimal,
    pub price_token: Decimal,
    pub price_rate24h: Decimal,
    pub volume_24h: Decimal,
    pub total_supply: Decimal,
    pub market_cap: Decimal,
    pub liquidity: Decimal,
    pub liquidity_token: Decimal,
    pub bonding_curve: Decimal,
    pub uniswap_pool: String,
    pub last_trade_ts: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::db_token_info::Entity", from = "Column::TokenAddress", to = "super::db_token_info::Column::TokenAddress")]
    TokenInfo,
    #[sea_orm(belongs_to = "super::db_raised_token::Entity", from = "Column::RaisedToken", to = "super::db_raised_token::Column::Address")]
    RaisedToken,
}

impl Related<super::db_token_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TokenInfo.def()
    }
}

impl Related<super::db_raised_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RaisedToken.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
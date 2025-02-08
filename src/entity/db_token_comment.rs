use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "token_comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub token_address: String,
    pub user_address: String,
    pub comment: String,
    pub create_ts: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::db_user::Entity",
        from = "Column::UserAddress",
        to = "super::db_user::Column::Address"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::db_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
} 
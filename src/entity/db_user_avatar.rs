use sea_orm::entity::prelude::*;
// use rand::seq::SliceRandom;
// use crate::utility::LibResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_avatar")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub avatar: String,
    pub create_ts: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// impl Entity {
//     pub async fn get_random_avatar(db: &DatabaseConnection) -> LibResult<String> {
//         let avatars = Self::find()
//             .all(db)
//             .await?;
//
//         Ok(avatars
//             .choose(&mut rand::thread_rng())
//             .map(|avatar| avatar.avatar.clone())
//             .unwrap_or_default())
//     }
// }
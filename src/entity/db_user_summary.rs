use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_summary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_address: String,
    #[sea_orm(primary_key)]
    pub token_address: String,
    pub amount: Decimal,
    pub update_ts: i64
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// impl Entity {
//     pub async fn find_token_owned(
//         db: &DatabaseConnection,
//         user_address: String,
//         keyword: Option<String>,
//         page: u64,
//         page_size: u64,
//     ) -> LibResult<Vec<(String, String, Decimal, Decimal)>> {
//         let mut sql = format!(
//             r#"
//             SELECT
//                 t2.icon,
//                 t2.symbol,
//                 t1.amount AS quantity,
//                 COALESCE(t3.price * t1.amount, 0) as value
//             FROM
//                 user_summary t1
//                 LEFT JOIN token_info t2 ON t1.token_address = t2.token_address
//                 LEFT JOIN token_summary t3 ON t1.token_address = t3.token_address
//             WHERE
//                 t1.user_address = '{}'
//         "#,
//             user_address
//         );
//
//         if let Some(keyword) = keyword {
//             sql.push_str(&format!(
//                 r#"
//                 AND (
//                     t2.token_address LIKE '%{}%'
//                     OR t2.name LIKE '%{}%'
//                     OR t2.symbol LIKE '%{}%'
//                 )
//             "#,
//                 keyword, keyword, keyword
//             ));
//         }
//         sql.push_str(" ORDER BY value DESC NULLS LAST");
//         let offset = (page - 1) * page_size;
//         sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));
//
//         let stmt = Statement::from_string(db.get_database_backend(), sql);
//
//         let rows = db.query_all(stmt).await?;
//         let tokens = rows
//             .into_iter()
//             .map(|row| {
//                 Ok((
//                     row.try_get::<String>("", "icon")?,
//                     row.try_get::<String>("", "symbol")?,
//                     row.try_get::<Decimal>("", "quantity")?,
//                     row.try_get::<Decimal>("", "value")?,
//                 ))
//             })
//             .collect::<Result<Vec<_>, DbErr>>()?;
//         Ok(tokens)
//     }
// }

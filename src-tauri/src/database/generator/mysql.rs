use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool};

use crate::Result;

// #[derive(Default, Debug, Serialize, Deserialize, FromRow)]
// struct Table {
//     table_schema: String,
//     table_name: String,
//     table_comment: String,
// }
//
// #[derive(Default, Debug, Serialize, Deserialize, FromRow)]
// struct TableColumn {
//     table_schema: String,
//     table_name: String,
//     column_name: String,
//     /// 字段顺序
//     ordinal_position: Option<u32>,
//     /// 默认值
//     column_default: Option<String>,
//     /// 是否允许为null
//     is_nullable: String,
//     data_type: Option<String>,
//     character_maximum_length: Option<i64>,
//     column_type: String,
//     column_comment: String,
// }
//
// impl From<Table> for super::Table {
//     fn from(t: Table) -> Self {
//         Self {
//             schema: t.table_schema,
//             name: t.table_name,
//             comment: t.table_comment,
//         }
//     }
// }
//
// impl From<TableColumn> for super::Column {
//     fn from(c: TableColumn) -> Self {
//         let ty = t2t(&c.column_type.clone().to_uppercase()).to_string();
//         Self {
//             schema: Some(c.table_schema.clone()),
//             table_name: Some(c.table_name.clone()),
//             name: Some(super::column_keywords(c.column_name.clone().as_str())),
//             default: c.column_default.clone(),
//             is_nullable: {
//                 if ty.contains("Time") {
//                     true
//                 } else {
//                     c.is_nullable.eq_ignore_ascii_case("yes")
//                 }
//             },
//             column_type: Some(c.column_type),
//             comment: Some(c.column_comment.clone()),
//             field_type: ty,
//             // multi_world: Some(c.column_name.clone().contains(|c| c == '_' || c == '-')),
//             max_length: c.character_maximum_length,
//         }
//     }
// }

// pub async fn tables(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Table>> {
//     let mut sql = "SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, TABLE_COMMENT table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"
//         .to_string();
//
//     if !table_names.is_empty() {
//         sql.push_str(&format!(
//             "AND FIND_IN_SET(TABLE_NAME, '{}')",
//             table_names.join(",")
//         ));
//     }
//
//     Ok(sqlx::query_as::<_, Table>(&sql)
//         .fetch_all(pool)
//         .await?
//         .into_iter()
//         .map(|t| t.into())
//         .collect::<Vec<_>>())
// }

// pub async fn columns(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Column>> {
//     let mut sql = r#"SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, COLUMN_NAME column_name, ORDINAL_POSITION ordinal_position, COLUMN_DEFAULT column_default, IS_NULLABLE is_nullable, DATA_TYPE data_type, CHARACTER_MAXIMUM_LENGTH character_maximum_length, COLUMN_TYPE column_type, COLUMN_COMMENT column_comment FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"#
//         .to_string();
//
//     if !table_names.is_empty() {
//         sql.push_str(&format!(
//             "AND FIND_IN_SET(TABLE_NAME, '{}')",
//             table_names.join(",")
//         ));
//     }
//
//     Ok(sqlx::query_as::<_, TableColumn>(&sql)
//         .fetch_all(pool)
//         .await?
//         .into_iter()
//         .map(|col| col.into())
//         .collect::<Vec<super::Column>>())
// }

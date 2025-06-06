use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool};

use crate::Result;

/// 表信息来自 sqlite_master
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Table {
    /// 项目的类型：table，index，view，trigger
    r#type: String,
    /// 项目的名称
    name: String,
    /// 所从属的表名，如索引所在的表名
    tbl_name: String,
    /// 项目在数据库页中存储的编号
    rootpage: Option<i64>,
    /// SQL语句
    sql: Option<String>,
}

/// 表列信息
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct TableColumn {
    /// 列ID
    cid: Option<u32>,
    /// 列名
    name: String,
    /// 类型：如：varchar(50)  int
    r#type: Option<String>,
    /// 是否为空：1-不为空，0-为空
    notnull: Option<u8>,
    dflt_value: Option<String>,
    /// 是否为主键：1-主键，0-非主键
    pk: Option<u8>,
}

// pub async fn tables(pool: &Pool<sqlx::Sqlite>, table_names: &[&str]) -> Result<Vec<super::Table>> {
//     let mut sql =
//         "SELECT type, name, tbl_name, rootpage, sql FROM sqlite_master WHERE type = 'table'"
//             .to_string();
//
//     if !table_names.is_empty() {
//         let table_names = table_names
//             .iter()
//             .map(|&t| format!("'{t}'"))
//             .collect::<Vec<_>>()
//             .join(",");
//         sql.push_str(&format!(" AND name in({table_names}) "));
//     }
//
//     Ok(sqlx::query_as::<_, Table>(&sql)
//         .fetch_all(pool)
//         .await?
//         .into_iter()
//         .map(|t| t.into())
//         .collect::<Vec<_>>())
// }

// pub async fn columns(
//     pool: &Pool<sqlx::Sqlite>,
//     table_names: &[&str],
// ) -> Result<Vec<super::Column>> {
//     let mut cols = vec![];
//     for table_name in table_names.iter() {
//         let columns =
//             sqlx::query_as::<_, TableColumn>(&format!("pragma table_info('{table_name}');"))
//                 .fetch_all(pool)
//                 .await?;
//
//         let mut columns = columns
//             .iter()
//             .map(|c| c.into())
//             .collect::<Vec<super::Column>>()
//             .iter_mut()
//             .map(|c| {
//                 c.table_name = table_name.to_string();
//                 c.to_owned()
//             })
//             .collect::<Vec<_>>();
//         cols.append(&mut columns);
//     }
//     Ok(cols)
// }

#![allow(unused)]
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Database, Pool};
use thiserror::Error;

mod mysql;
mod postgres;
mod sqlite;

pub use mysql::MysqlMetadata;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    E(&'static str),
    #[error(transparent)]
    SqlxErr(#[from] sqlx::Error),
    #[error("未知错误")]
    Unknown,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// 数据库元数据
pub trait DatabaseMetadata {
    type Pool;

    /// 获取所有的库
    async fn schemas(pool: &Self::Pool) -> Result<Vec<Schema>>;
    /// 获取所有的表
    async fn tables(pool: &Self::Pool, schema: &str) -> Result<Vec<Table>>;
    /// 获取表的字段
    async fn columns(pool: &Self::Pool, schema: &str, table_name: &str) -> Result<Vec<Column>>;
}

/// 库
#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
}

/// 表信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub comment: String,
}

/// 列信息
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Column {
    pub schema: String,
    pub table_name: String,
    pub name: String,
    pub r#type: String,
    pub max_length: Option<i64>,
    pub is_nullable: String,
    pub key: Option<String>,
    pub default: Option<String>,
    pub comment: Option<String>,
}

/// 驱动类型
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Driver {
    Mysql,
    Postgres,
    Sqlite,
}

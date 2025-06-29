#![allow(unused)]
use std::{fmt::Display, pin::Pin};

use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Database, Pool};
use thiserror::Error;

mod mysql;
mod postgres;
mod sqlite;

pub use mysql::MysqlMetadata;
pub use postgres::PostgresMetadata;
pub use sqlite::SqliteMetadata;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    E(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SqlxErr(#[from] sqlx::Error),
    #[error(transparent)]
    TeraErr(#[from] tera::Error),
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

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// 数据库元数据
pub trait DatabaseMetadata: Send + Sync {
    /// 获取所有的库
    fn schemas(&self) -> BoxFuture<'_, Result<Vec<Schema>>>;
    /// 获取所有的表
    fn tables<'a>(&'a self, schema: &'a str) -> BoxFuture<'a, Result<Vec<Table>>>;
    /// 获取表的字段
    fn columns<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<Column>>>;
    /// 获取表索引
    fn indexs<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<Index>>>;
    /// 创建表SQL
    fn create_table_sql<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> BoxFuture<'a, Result<String>>;
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
    // 库名
    pub database: String,
    // 模式
    pub schema: String,
    // 表名
    pub table_name: String,
    /// 字段名
    pub name: String,
    /// 字段类型
    pub r#type: Option<ColumnType>,
    /// 字段长度，可以为空
    pub length: Option<i32>,
    /// 字段精度
    pub scale: Option<i32>,
    /// 默认值
    pub default: Option<String>,
    /// 枚举值列表
    pub enum_values: Option<Vec<String>>,
    /// 备注
    pub comment: String,

    /// 是否为空
    pub is_null: bool,
    /// 是否自增
    pub is_auto_incr: bool,
    /// 是否唯一
    pub is_unique: bool,
    /// 是否主键
    pub is_primary_key: bool,
    /// 是否无符号
    pub is_unsigned: bool,

    // 对应 Rust 类型
    pub rust_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    /// 表名
    pub table_name: String,
    /// 如果索引不能包括重复词，则为0。如果可以，则为1。
    pub non_unique: i32,
    /// 索引的名称
    pub key_name: String,
    /// 索引中的列的序号。对于组合索引，这表示列在索引中的位置。
    pub seq_in_index: u32,
    /// 作用于列名称
    pub column_name: String,
    /// 索引的前缀长度。对于部分索引，这表示索引的前缀长度。
    pub sub_part: Option<i32>,
    /// 用过的索引方法（BTREE, FULLTEXT, HASH, RTREE）
    pub index_type: String,
    /// 索引的注释
    pub index_comment: String,
}

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColumnType {
    Bigint,
    Binary,
    Bit,
    Blob,
    Char,
    Date,
    DateTime,
    Decimal,
    Double,
    Enum,
    Float,
    Geometry,
    GeometryCollection,
    Int,
    Integer,
    Json,
    LineString,
    LongBlob,
    LongText,
    MediumBlob,
    MediumInt,
    MediumText,
    MultilineString,
    MultiPoint,
    Numeric,
    Point,
    Polygon,
    Real,
    Set,
    SmallInt,
    Text,
    Time,
    Timestamp,
    TinyBlob,
    TinyInt,
    TinyText,
    Varbinary,
    VarChar,
    Year,
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Bigint => f.write_str("BIGINT"),
            ColumnType::Binary => f.write_str("BIGINT"),
            ColumnType::Bit => f.write_str("BIGINT"),
            ColumnType::Blob => f.write_str("BIGINT"),
            ColumnType::Char => f.write_str("BIGINT"),
            ColumnType::Date => f.write_str("BIGINT"),
            ColumnType::DateTime => f.write_str("BIGINT"),
            ColumnType::Decimal => f.write_str("BIGINT"),
            ColumnType::Double => f.write_str("BIGINT"),
            ColumnType::Enum => f.write_str("BIGINT"),
            ColumnType::Float => f.write_str("BIGINT"),
            ColumnType::Geometry => f.write_str("BIGINT"),
            ColumnType::GeometryCollection => f.write_str("BIGINT"),
            ColumnType::Int => f.write_str("BIGINT"),
            ColumnType::Integer => f.write_str("BIGINT"),
            ColumnType::Json => f.write_str("BIGINT"),
            ColumnType::LineString => f.write_str("BIGINT"),
            ColumnType::LongBlob => f.write_str("BIGINT"),
            ColumnType::LongText => f.write_str("BIGINT"),
            ColumnType::MediumBlob => f.write_str("BIGINT"),
            ColumnType::MediumInt => f.write_str("BIGINT"),
            ColumnType::MediumText => f.write_str("BIGINT"),
            ColumnType::MultilineString => f.write_str("BIGINT"),
            ColumnType::MultiPoint => f.write_str("BIGINT"),
            ColumnType::Numeric => f.write_str("BIGINT"),
            ColumnType::Point => f.write_str("BIGINT"),
            ColumnType::Polygon => f.write_str("BIGINT"),
            ColumnType::Real => f.write_str("BIGINT"),
            ColumnType::Set => f.write_str("BIGINT"),
            ColumnType::SmallInt => f.write_str("BIGINT"),
            ColumnType::Text => f.write_str("BIGINT"),
            ColumnType::Time => f.write_str("BIGINT"),
            ColumnType::Timestamp => f.write_str("BIGINT"),
            ColumnType::TinyBlob => f.write_str("BIGINT"),
            ColumnType::TinyInt => f.write_str("BIGINT"),
            ColumnType::TinyText => f.write_str("BIGINT"),
            ColumnType::Varbinary => f.write_str("BIGINT"),
            ColumnType::VarChar => f.write_str("BIGINT"),
            ColumnType::Year => f.write_str("BIGINT"),
        }
    }
}

impl From<String> for ColumnType {
    fn from(value: String) -> Self {
        match value.to_uppercase().as_str() {
            "BIGINT" => Self::Bigint,
            "BINARY" => Self::Binary,
            "BIT" => Self::Bit,
            "BLOB" => Self::Blob,
            "CHAR" => Self::Char,
            "DATE" => Self::Date,
            "DATETIME" => Self::DateTime,
            "DECIMAL" => Self::Decimal,
            "DOUBLE" => Self::Double,
            "ENUM" => Self::Enum,
            "FLOAT" => Self::Float,
            "GEOMETRY" => Self::Geometry,
            "GEOMETRYCOLLECTION" => Self::GeometryCollection,
            "INT" => Self::Int,
            "INTEGER" => Self::Integer,
            "JSON" => Self::Json,
            "LINESTRING" => Self::LineString,
            "LONGBLOB" => Self::LongBlob,
            "LONGTEXT" => Self::LongText,
            "MEDIUMBLOB" => Self::MediumBlob,
            "MEDIUMINT" => Self::MediumInt,
            "MEDIUMTEXT" => Self::MediumText,
            "MULTILINESTRING" => Self::MultilineString,
            "MULTIPOINT" => Self::MultiPoint,
            "NUMERIC" => Self::Numeric,
            "POINT" => Self::Point,
            "POLYGON" => Self::Polygon,
            "REAL" => Self::Real,
            "SET" => Self::Set,
            "SMALLINT" => Self::SmallInt,
            "TEXT" => Self::Text,
            "TIME" => Self::Time,
            "TIMESTAMP" => Self::Timestamp,
            "TINYBLOB" => Self::TinyBlob,
            "TINYINT" => Self::TinyInt,
            "TINYTEXT" => Self::TinyText,
            "VARBINARY" => Self::Varbinary,
            "VARCHAR" => Self::VarChar,
            "YEAR" => Self::Year,
            _ => unimplemented!(),
        }
    }
}

/// 驱动类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Driver {
    Mysql,
    Postgres,
    Sqlite,
}

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
    /// 获取表索引
    async fn indexs(pool: &Self::Pool, schema: &str, table_name: &str) -> Result<Vec<Index>>;
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
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    /// 表名
    table_name: String,
    /// 如果索引不能包括重复词，则为0。如果可以，则为1。
    non_unique: i32,
    /// 索引的名称
    key_name: String,
    /// 索引中的列的序号。对于组合索引，这表示列在索引中的位置。
    seq_in_index: u32,
    /// 作用于列名称
    column_name: String,
    /// 索引的前缀长度。对于部分索引，这表示索引的前缀长度。
    sub_part: Option<i32>,
    /// 用过的索引方法（BTREE, FULLTEXT, HASH, RTREE）
    index_type: String,
    /// 索引的注释
    index_comment: String,
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
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Driver {
    Mysql,
    Postgres,
    Sqlite,
}

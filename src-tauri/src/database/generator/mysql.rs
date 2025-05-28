use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool};

use crate::Result;

/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
/// | Field           | Type                                                               | Null | Key | Default | Extra |
/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
/// | TABLE_CATALOG   | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_SCHEMA    | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_NAME      | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_TYPE      | enum('BASE TABLE','VIEW','SYSTEM VIEW')                            | NO   |     | NULL    |       |
/// | ENGINE          | varchar(64)                                                        | YES  |     | NULL    |       |
/// | VERSION         | int                                                                | YES  |     | NULL    |       |
/// | ROW_FORMAT      | enum('Fixed','Dynamic','Compressed','Redundant','Compact','Paged') | YES  |     | NULL    |       |
/// | TABLE_ROWS      | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | AVG_ROW_LENGTH  | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | DATA_LENGTH     | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | MAX_DATA_LENGTH | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | INDEX_LENGTH    | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | DATA_FREE       | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | AUTO_INCREMENT  | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | CREATE_TIME     | timestamp                                                          | NO   |     | NULL    |       |
/// | UPDATE_TIME     | datetime                                                           | YES  |     | NULL    |       |
/// | CHECK_TIME      | datetime                                                           | YES  |     | NULL    |       |
/// | TABLE_COLLATION | varchar(64)                                                        | YES  |     | NULL    |       |
/// | CHECKSUM        | bigint                                                             | YES  |     | NULL    |       |
/// | CREATE_OPTIONS  | varchar(256)                                                       | YES  |     | NULL    |       |
/// | TABLE_COMMENT   | text                                                               | YES  |     | NULL    |       |
/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
#[derive(Default, Debug, Serialize, Deserialize, FromRow)]
struct Table {
    table_schema: String,
    table_name: String,
    table_comment: String,
}

/// +--------------------------+----------------------------+------+-----+---------+-------+
/// | Field                    | Type                       | Null | Key | Default | Extra |
/// +--------------------------+----------------------------+------+-----+---------+-------+
/// | TABLE_CATALOG            | varchar(64)                | NO   |     | NULL    |       |
/// | TABLE_SCHEMA             | varchar(64)                | NO   |     | NULL    |       |
/// | TABLE_NAME               | varchar(64)                | NO   |     | NULL    |       |
/// | COLUMN_NAME              | varchar(64)                | YES  |     | NULL    |       |
/// | ORDINAL_POSITION         | int unsigned               | NO   |     | NULL    |       |
/// | COLUMN_DEFAULT           | text                       | YES  |     | NULL    |       |
/// | IS_NULLABLE              | varchar(3)                 | NO   |     |         |       |
/// | DATA_TYPE                | longtext                   | YES  |     | NULL    |       |
/// | CHARACTER_MAXIMUM_LENGTH | bigint                     | YES  |     | NULL    |       |
/// | CHARACTER_OCTET_LENGTH   | bigint                     | YES  |     | NULL    |       |
/// | NUMERIC_PRECISION        | bigint unsigned            | YES  |     | NULL    |       |
/// | NUMERIC_SCALE            | bigint unsigned            | YES  |     | NULL    |       |
/// | DATETIME_PRECISION       | int unsigned               | YES  |     | NULL    |       |
/// | CHARACTER_SET_NAME       | varchar(64)                | YES  |     | NULL    |       |
/// | COLLATION_NAME           | varchar(64)                | YES  |     | NULL    |       |
/// | COLUMN_TYPE              | mediumtext                 | NO   |     | NULL    |       |
/// | COLUMN_KEY               | enum('','PRI','UNI','MUL') | NO   |     | NULL    |       |
/// | EXTRA                    | varchar(256)               | YES  |     | NULL    |       |
/// | PRIVILEGES               | varchar(154)               | YES  |     | NULL    |       |
/// | COLUMN_COMMENT           | text                       | NO   |     | NULL    |       |
/// | GENERATION_EXPRESSION    | longtext                   | NO   |     | NULL    |       |
/// | SRS_ID                   | int unsigned               | YES  |     | NULL    |       |
/// +--------------------------+----------------------------+------+-----+---------+-------+
#[derive(Default, Debug, Serialize, Deserialize, FromRow)]
struct TableColumn {
    table_schema: String,
    table_name: String,
    column_name: String,
    /// 字段顺序
    ordinal_position: Option<u32>,
    /// 默认值
    column_default: Option<String>,
    /// 是否允许为null
    is_nullable: String,
    data_type: Option<String>,
    character_maximum_length: Option<i64>,
    column_type: String,
    column_comment: String,
}

impl From<Table> for super::Table {
    fn from(t: Table) -> Self {
        Self {
            schema: t.table_schema,
            name: t.table_name,
            comment: t.table_comment,
        }
    }
}

impl From<TableColumn> for super::Column {
    fn from(c: TableColumn) -> Self {
        let ty = t2t(&c.column_type.clone().to_uppercase()).to_string();
        Self {
            schema: Some(c.table_schema.clone()),
            table_name: Some(c.table_name.clone()),
            name: Some(super::column_keywords(c.column_name.clone().as_str())),
            default: c.column_default.clone(),
            is_nullable: {
                if ty.contains("Time") {
                    true
                } else {
                    c.is_nullable.eq_ignore_ascii_case("yes")
                }
            },
            column_type: Some(c.column_type),
            comment: Some(c.column_comment.clone()),
            field_type: ty,
            // multi_world: Some(c.column_name.clone().contains(|c| c == '_' || c == '-')),
            max_length: c.character_maximum_length,
        }
    }
}

/// Rust type             MySQL type(s)
/// bool                    TINYINT(1), BOOLEAN
/// i8                      TINYINT
/// i16                     SMALLINT
/// i32                     INT
/// i64                     BIGINT
/// u8                      TINYINT UNSIGNED
/// u16                     SMALLINT UNSIGNED
/// u32                     INT UNSIGNED
/// u64                     BIGINT UNSIGNED
/// f32                     FLOAT
/// f64                     DOUBLE
/// &str, String            VARCHAR, CHAR, TEXT
/// &[u8], Vec<u8>          VARBINARY, BINARY, BLOB
///
/// time::PrimitiveDateTime DATETIME
/// time::OffsetDateTime    TIMESTAMP
/// time::Date              DATE
/// time::Time              TIME
///
/// bigdecimal::BigDecimal  DECIMAL
///
/// uuid::Uuid              BYTE(16), VARCHAR, CHAR, TEXT
/// uuid::fmt::Hyphenated   CHAR(36)
/// uuid::fmt::Simple       CHAR(32)
///
/// serde_json::JsonValue  JSON
///
/// Mysql 类型转换为Rust对应类型
fn t2t(ty: &str) -> &str {
    match ty.to_uppercase().as_str() {
        "TINYINT(1)" | "BOOLEAN" => "bool",
        "TINYINT" => "i8",
        "TINYINT UNSIGNED" | "BIT" => "u8",
        "SMALLINT" => "i16",
        "SMALLINT UNSIGNED" => "u16",
        "INT" | "MEDIUMINT" => "i32",
        "INT UNSIGNED" | "MEDIUMINT UNSIGNED" => "u32",
        "BIGINT" => "i64",
        "BIGINT UNSIGNED" => "u64",
        "FLOAT" => "f32",
        "DOUBLE" | "NUMERIC" => "f64",
        "VARBINARY" | "BINARY" | "BLOB" => "Vec<u8>",
        "YEAR" => "time::Date",
        "DATE" => "time::Date",
        "TIME" => "time::Time",
        "DATETIME" => "time::PrimitiveDateTime",
        "TIMESTAMP" => "time::offsetDateTime",
        "DECIMAL" => "bigdecimal::BigDecimal",
        "JSON" => "serde_json:JsonValue",
        _ => "String",
    }
}

pub async fn tables(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Table>> {
    let mut sql = "SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, TABLE_COMMENT table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"
        .to_string();

    if !table_names.is_empty() {
        sql.push_str(&format!(
            "AND FIND_IN_SET(TABLE_NAME, '{}')",
            table_names.join(",")
        ));
    }

    Ok(sqlx::query_as::<_, Table>(&sql)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|t| t.into())
        .collect::<Vec<_>>())
}

pub async fn columns(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Column>> {
    let mut sql = r#"SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, COLUMN_NAME column_name, ORDINAL_POSITION ordinal_position, COLUMN_DEFAULT column_default, IS_NULLABLE is_nullable, DATA_TYPE data_type, CHARACTER_MAXIMUM_LENGTH character_maximum_length, COLUMN_TYPE column_type, COLUMN_COMMENT column_comment FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"#
        .to_string();

    if !table_names.is_empty() {
        sql.push_str(&format!(
            "AND FIND_IN_SET(TABLE_NAME, '{}')",
            table_names.join(",")
        ));
    }

    Ok(sqlx::query_as::<_, TableColumn>(&sql)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|col| col.into())
        .collect::<Vec<super::Column>>())
}

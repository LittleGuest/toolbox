use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, AnyPool, ColumnIndex, Database, FromRow, MySql, MySqlPool, Pool, Row};

use super::{ColumnType, DatabaseMetadata, Result};

const SHOW_DATABASES: &str = "SHOW DATABASES";
const SHOW_COLUMNS: &str = "SHOW FULL COLUMNS FROM ? FROM ?";
const SHOW_TABLES:&str = "SELECT table_schema, table_name, CAST(TABLE_TYPE AS CHAR) TABLE_TYPE, table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ?";
const SHOW_INDEX: &str = "SHOW INDEX FROM ? FROM ?";
const SHOW_CREATE_TABLE: &str = "SHOW CREATE TABLE ?";
const WORD_UNSIGNED: &str = "unsigned";
const WORD_PRIMARY: &str = "PRIMARY";

pub struct MysqlMetadata;

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Schema {
    name: String,
}

impl From<Schema> for super::Schema {
    fn from(s: Schema) -> Self {
        Self { name: s.name }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Table {
    #[sqlx(rename = "TABLE_SCHEMA")]
    table_schema: String,
    #[sqlx(rename = "TABLE_NAME")]
    table_name: String,
    #[sqlx(rename = "TABLE_TYPE")]
    table_type: String,
    #[sqlx(rename = "TABLE_COMMENT")]
    table_comment: String,
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

#[derive(Debug, Default, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Column {
    // 模式
    schema: String,
    // 表名
    table_name: String,
    /// 字段名
    name: String,
    /// 字段类型
    r#type: Option<ColumnType>,
    /// 字段长度，可以为空
    length: Option<i32>,
    /// 字段精度
    scale: Option<i32>,
    /// 默认值
    default: Option<String>,
    /// 枚举值列表
    enum_values: Option<Vec<String>>,
    /// 备注
    comment: String,
    /// 是否为空
    is_null: bool,
    /// 是否自增
    is_auto_incr: bool,
    /// 是否唯一
    is_unique: bool,
    /// 是否主键
    is_primary_key: bool,
    /// 是否无符号
    is_unsigned: bool,
}

impl Column {
    pub fn handle_column_as_type(&mut self, r#type: &str) -> Result<()> {
        if r#type.is_empty() {
            return Ok(());
        }

        self.is_unsigned = r#type.contains(WORD_UNSIGNED);
        let column_type = r#type.replace(WORD_UNSIGNED, "");
        let mut meta_type = String::new();
        let mut meta_length = String::new();
        let mut scale = String::new();
        let mut enum_values = vec![];

        if let Some(pos) = column_type.find('(') {
            meta_type = column_type[0..pos].to_string();
            meta_length = column_type[pos + 1..column_type.len() - 1].to_string();

            if meta_length.contains("enum") {
                // enum('1','2','3')
                enum_values = meta_length
                    .replace("enum", "")
                    .replace(['(', ')'], "")
                    .split(",")
                    .map(|v| v.replace("'", ""))
                    .collect::<Vec<_>>();
            } else {
                // double(11,8)
                let ml = meta_length.clone();
                let mut sc = ml.split(',').collect::<Vec<_>>();
                meta_length = sc.first().unwrap().replace('(', "");
                if sc.len() == 2 {
                    scale = sc.get(1).unwrap().replace(')', "");
                }
            }
        } else {
            meta_type = column_type;
        }

        if enum_values.is_empty() {
            if !meta_type.starts_with("enum") {
                if let Some(pos) = meta_length.find(" ") {
                    meta_length = meta_length.split(" ").next().unwrap().into();
                }

                self.length = if meta_length.is_empty() {
                    None
                } else {
                    Some(meta_length.parse::<i32>().unwrap_or(0))
                };
                self.scale = if scale.is_empty() {
                    None
                } else {
                    Some(scale.parse::<i32>().unwrap_or(0))
                };
            }
        } else {
            self.enum_values = Some(enum_values);
        }
        self.r#type = Some(ColumnType::from(meta_type));
        Ok(())
    }

    fn handle_primary_key(&mut self, key: &str) {
        self.is_primary_key = "PRI".eq(key);
    }

    fn handle_is_null(&mut self, null: &str) {
        match null {
            "yes" => self.is_null = true,
            "no" => self.is_null = false,
            _ => {}
        }
    }

    fn handle_is_auto_incr(&mut self, auto_incr: &str) {
        self.is_auto_incr = auto_incr.to_lowercase().contains("auto_increment");
    }
}

impl From<Column> for super::Column {
    fn from(col: Column) -> Self {
        let rust_type = t2t(&format!("{}", col.r#type.unwrap())).into();
        Self {
            database: col.schema.clone(),
            schema: col.schema,
            table_name: col.table_name,
            name: col.name,
            r#type: col.r#type,
            length: col.length,
            scale: col.scale,
            default: col.default,
            enum_values: col.enum_values,
            comment: col.comment,
            is_null: col.is_null,
            is_auto_incr: col.is_auto_incr,
            is_unique: col.is_unique,
            is_primary_key: col.is_primary_key,
            is_unsigned: col.is_unsigned,

            rust_type,
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

#[derive(Debug, Default, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
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
    // /// 排序规则：A or D: ASC or DESC
    // collation: String,
    // cardinality: i32,
    /// 索引的前缀长度。对于部分索引，这表示索引的前缀长度。
    sub_part: Option<i32>,
    // packed: Option<String>,
    // null: String,
    /// 用过的索引方法（BTREE, FULLTEXT, HASH, RTREE）
    index_type: String,
    // comment: String,
    /// 索引的注释
    index_comment: String,
    // visible: String,
    // expression: Option<String>,
}

impl From<Index> for super::Index {
    fn from(ix: Index) -> Self {
        Self {
            table_name: ix.table_name,
            non_unique: ix.non_unique,
            key_name: ix.key_name,
            seq_in_index: ix.seq_in_index,
            column_name: ix.column_name,
            sub_part: ix.sub_part,
            index_type: ix.index_type,
            index_comment: ix.index_comment,
        }
    }
}

impl DatabaseMetadata for MysqlMetadata {
    type Pool = MySqlPool;

    async fn schemas(pool: &Self::Pool) -> Result<Vec<super::Schema>> {
        let rows = sqlx::query(SHOW_DATABASES)
            .map(|row: MySqlRow| Schema { name: row.get(0) })
            .map(|row| row.into())
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn tables(pool: &Self::Pool, schema: &str) -> Result<Vec<super::Table>> {
        let rows: Vec<Table> = sqlx::query_as(SHOW_TABLES)
            .bind(schema)
            .fetch_all(pool)
            .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
    }

    async fn columns(
        pool: &Self::Pool,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<super::Column>> {
        let rows: Vec<Column> = sqlx::query(&format!(
            "SHOW FULL COLUMNS FROM {table_name} FROM {schema}"
        ))
        .bind(table_name)
        .bind(schema)
        .map(|row: MySqlRow| {
            let field = row.get(0);
            let r#type: Vec<u8> = row.get(1);
            let r#type = String::from_utf8_lossy(&r#type).to_string();
            let null = row.get(3);
            let key: Vec<u8> = row.get(4);
            let key = String::from_utf8_lossy(&key).to_string();
            let default: Option<Vec<u8>> = row.get(5);
            let default = default.map(|d| String::from_utf8_lossy(&d).to_string());
            let extra = row.get(6);
            let comment: Vec<u8> = row.get(8);
            let comment = String::from_utf8_lossy(&comment).to_string();

            let mut coloumn = Column {
                schema: schema.into(),
                table_name: table_name.into(),
                name: field,
                default,
                comment,
                ..Default::default()
            };
            coloumn.handle_column_as_type(&r#type).is_ok();
            coloumn.handle_primary_key(&key);
            coloumn.handle_is_null(null);
            coloumn.handle_is_auto_incr(extra);
            coloumn
        })
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
    }

    async fn indexs(
        pool: &Self::Pool,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<super::Index>> {
        let rows: Vec<Index> = sqlx::query(&format!("SHOW INDEX FROM {table_name} FROM {schema}"))
            .bind(table_name)
            .bind(schema)
            .map(|row: MySqlRow| {
                let table_name = row.get(0);
                let non_unique = row.get(1);
                let key_name = row.get(2);
                let seq_in_index = row.get(3);
                let column_name = row.get(4);
                let sub_part = row.get(7);
                let index_type: Vec<u8> = row.get(10);
                let index_type = String::from_utf8_lossy(&index_type).to_string();
                let index_comment: Vec<u8> = row.get(12);
                let index_comment = String::from_utf8_lossy(&index_comment).to_string();

                Index {
                    table_name,
                    non_unique,
                    key_name,
                    seq_in_index,
                    column_name,
                    sub_part,
                    index_type,
                    index_comment,
                }
            })
            .fetch_all(pool)
            .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
    }

    async fn create_table_sql(pool: &Self::Pool, schema: &str, table_name: &str) -> Result<String> {
        let rows: String = sqlx::query(&format!("SHOW CREATE TABLE {schema}.{table_name}"))
            .bind(table_name)
            .bind(schema)
            .map(|row: MySqlRow| {
                let sql: Vec<u8> = row.get(1);
                String::from_utf8_lossy(&sql).to_string()
            })
            .fetch_one(pool)
            .await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "mysql://root:123456@localhost:3306/test";

    #[tokio::test]
    async fn test_schemas() {
        let pool = MySqlPool::connect(URL).await.unwrap();
        let schemas = MysqlMetadata::schemas(&pool).await.unwrap();
        assert!(!schemas.is_empty())
    }

    #[tokio::test]
    async fn test_tables() {
        let pool = MySqlPool::connect(URL).await.unwrap();
        let tables = MysqlMetadata::tables(&pool, "differ").await.unwrap();
        assert!(!tables.is_empty())
    }

    #[tokio::test]
    async fn test_columns() {
        let pool = MySqlPool::connect(URL).await.unwrap();
        let columns = MysqlMetadata::columns(&pool, "differ", "db_detail")
            .await
            .unwrap();
        assert!(!columns.is_empty())
    }

    #[tokio::test]
    async fn test_indexs() {
        let pool = MySqlPool::connect(URL).await.unwrap();
        let indexs = MysqlMetadata::indexs(&pool, "differ", "db_detail")
            .await
            .unwrap();
        dbg!(&indexs);
        assert!(!indexs.is_empty())
    }
}

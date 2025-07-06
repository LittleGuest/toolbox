use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, FromRow, Pool, Row, any::AnyRow};

use crate::database::cores::{ColumnType, DatabaseMetadata, Result};

const SHOW_DATABASES: &str = "";
const SHOW_COLUMNS: &str = "";
const SHOW_TABLES: &str = "SELECT tb.table_catalog, tb.table_schema, tb.TABLE_NAME, d.description FROM information_schema.tables tb JOIN pg_class C ON C.relname = tb. TABLE_NAME LEFT JOIN pg_description d ON d.objoid = C.OID  AND d.objsubid = '0' WHERE tb.table_catalog = $1 and tb.table_schema = $2 ";
const SHOW_INDEX: &str = "";
const SHOW_CREATE_TABLE: &str = "";
const WORD_UNSIGNED: &str = "unsigned";
const WORD_PRIMARY: &str = "PRIMARY";

pub struct PostgresMetadata(AnyPool);

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Schema {
    name: String,
}

impl From<Schema> for super::Schema {
    fn from(s: Schema) -> Self {
        Self { name: s.name }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Table {
    /// 表所属的数据库名称
    // #[sqlx(rename = "table_catalog")]
    table_catalog: String,
    /// 表所属的模式名称 (如 public)
    // #[sqlx(rename = "table_schema")]
    table_schema: String,
    /// 表的名称
    // #[sqlx(rename = "table_name")]
    table_name: String,
    /// 表的类型
    /// - 'BASE TABLE': 普通表
    /// - 'VIEW': 视图
    /// - 'FOREIGN TABLE': 外部表
    /// - 'LOCAL TEMPORARY': 临时表
    // #[sqlx(rename = "table_type")]
    table_type: String,
    /// 自引用列的名称 (通常为 NULL)
    // #[sqlx(rename = "self_referencing_column_name")]
    self_referencing_column_name: Option<String>,
    /// 引用生成方式 (如 'SYSTEM GENERATED'，通常为 NULL)
    // #[sqlx(rename = "reference_generation")]
    reference_generation: Option<String>,
    /// 用户定义类型所属的数据库 (通常为 NULL)
    // #[sqlx(rename = "user_defined_type_catalog")]
    user_defined_type_catalog: Option<String>,
    /// 用户定义类型所属的模式 (通常为 NULL)
    // #[sqlx(rename = "user_defined_type_schema")]
    user_defined_type_schema: Option<String>,
    /// 用户定义类型的名称 (通常为 NULL)
    // #[sqlx(rename = "user_defined_type_name")]
    user_defined_type_name: Option<String>,
    /// 是否可向表中插入数据 ('YES'/'NO')
    // #[sqlx(rename = "is_insertable_into")]
    is_insertable_into: String,
    /// 是否为类型化表 ('YES'/'NO')
    // #[sqlx(rename = "is_typed")]
    is_typed: String,
    /// 提交动作 (PostgreSQL 中通常为 NULL)
    // #[sqlx(rename = "commit_action")]
    commit_action: Option<String>,
    /// 描述
    description: Option<String>,
}

impl From<Table> for super::Table {
    fn from(t: Table) -> Self {
        Self {
            schema: t.table_schema,
            name: t.table_name.clone(),
            comment: t.description.unwrap_or(t.table_name),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Column {
    /// 列所属的数据库名称
    table_catalog: String,
    /// 列所属的模式名称 (如 public)
    table_schema: String,
    /// 列所属的表名称
    table_name: String,
    /// 列的名称
    column_name: String,
    /// 列在表中的位置序号 (从1开始)
    ordinal_position: i32,
    /// 列的默认值表达式
    column_default: Option<String>,
    /// 列是否允许 NULL 值
    is_nullable: String, // 'YES' or 'NO'
    /// 列的标准SQL数据类型
    data_type: String,
    /// 字符类型列的最大长度
    character_maximum_length: Option<i32>,
    /// 字符类型列的最大字节长度
    character_octet_length: Option<i32>,
    /// 数值类型列的精度
    numeric_precision: Option<i32>,
    /// 数值精度的基数 (2=二进制, 10=十进制)
    numeric_precision_radix: Option<i32>,
    /// 数值类型列的小数位数
    numeric_scale: Option<i32>,
    /// 时间类型列的秒小数精度
    datetime_precision: Option<i32>,
    /// 间隔类型信息
    interval_type: Option<String>,
    /// 间隔类型的精度
    interval_precision: Option<i32>,
    /// 字符集所属的数据库 (通常为NULL)
    character_set_catalog: Option<String>,
    /// 字符集所属的模式 (通常为NULL)
    character_set_schema: Option<String>,
    /// 字符集名称 (如 UTF8)
    character_set_name: Option<String>,
    /// 排序规则所属的数据库 (通常为NULL)
    collation_catalog: Option<String>,
    /// 排序规则所属的模式 (通常为NULL)
    collation_schema: Option<String>,
    /// 排序规则名称 (如 en_US.utf8)
    collation_name: Option<String>,
    /// 域所属的数据库 (未使用域时为NULL)
    domain_catalog: Option<String>,
    /// 域所属的模式 (未使用域时为NULL)
    domain_schema: Option<String>,
    /// 域的名称 (未使用域时为NULL)
    domain_name: Option<String>,
    /// 底层类型的数据库名
    udt_catalog: String,
    /// 底层类型所在的模式
    udt_schema: String,
    /// 底层类型名称 (如 int4, varchar)
    udt_name: String,
    /// 引用范围所属的数据库 (通常为NULL)
    scope_catalog: Option<String>,
    /// 引用范围所属的模式 (通常为NULL)
    scope_schema: Option<String>,
    /// 引用范围的名称 (通常为NULL)
    scope_name: Option<String>,
    /// 数组类型的最大维度
    maximum_cardinality: Option<i32>,
    /// 列在表定义中的唯一标识符
    dtd_identifier: String,
    /// 列是否自引用 (如外键引用自身表) 'YES' or 'NO'
    is_self_referencing: String,
    /// 列是否为标识列 (Identity Column) 'YES' or 'NO'
    is_identity: String,
    /// 标识列的生成方式 (ALWAYS 或 BY DEFAULT)
    identity_generation: Option<String>,
    /// 标识列的起始值
    identity_start: Option<String>,
    /// 标识列的递增值
    identity_increment: Option<String>,
    /// 标识列的最大值
    identity_maximum: Option<String>,
    /// 标识列的最小值
    identity_minimum: Option<String>,
    /// 标识列是否循环 (序列达到极值后重置) 'YES' or 'NO'
    identity_cycle: String,
    /// 列值是否生成 (ALWAYS=存储生成列, NEVER=普通列)
    is_generated: String,
    /// 生成列的表达式
    generation_expression: Option<String>,
    /// 列是否可更新 'YES' or 'NO'
    is_updatable: String,
}

impl From<Column> for super::Column {
    fn from(c: Column) -> Self {
        let ty = t2t(&c.data_type.clone().to_uppercase()).to_string();
        Self {
            database: todo!(),
            schema: todo!(),
            table_name: todo!(),
            name: todo!(),
            r#type: todo!(),
            length: todo!(),
            scale: todo!(),
            default: todo!(),
            enum_values: todo!(),
            comment: todo!(),
            is_null: todo!(),
            is_auto_incr: todo!(),
            is_unique: todo!(),
            is_primary_key: todo!(),
            is_unsigned: todo!(),
            rust_type: todo!(),
        }
        // Self {
        //     schema: Some(c.table_schema.clone()),
        //     table_name: Some(c.table_name.clone()),
        //     name: Some(super::column_keywords(c.column_name.clone().as_str())),
        //     default: c.column_default.clone(),
        //     is_nullable: {
        //         if ty.contains("Time") {
        //             true
        //         } else {
        //             c.is_nullable.eq_ignore_ascii_case("yes")
        //         }
        //     },
        //     column_type: Some(c.data_type),
        //     comment: c.description,
        //     field_type: ty,
        //     // multi_world: Some(c.column_name.clone().contains(|c| c == '_' || c == '-')),
        //     max_length: {
        //         if let Some(l) = c.character_maximum_length {
        //             Some(l as i64)
        //         } else {
        //             Some(50)
        //         }
        //     },
        // }
    }
}

/// Rust type            Postgres type(s)
/// bool                    BOOL
/// i8                      “CHAR”
/// i16                     SMALLINT, SMALLSERIAL, INT2
/// i32                     INT, SERIAL, INT4
/// i64                     BIGINT, BIGSERIAL, INT8
/// f32                     REAL, FLOAT4
/// f64                     DOUBLE PRECISION, FLOAT8
/// &str, String            VARCHAR, CHAR(N), TEXT, NAME
/// &[u8], Vec<u8>          BYTEA
/// ()                      VOID
/// PgInterval              INTERVAL
/// PgRange<T>              INT8RANGE, INT4RANGE, TSRANGE, TSTZRANGE, DATERANGE, NUMRANGE
/// PgMoney                 MONEY
/// PgLTree                 LTREE
/// PgLQuery                LQUERY
///
/// bigdecimal::BigDecimal  NUMERIC
///
/// time::PrimitiveDateTime TIMESTAMP
/// time::OffsetDateTime    TIMESTAMPTZ
/// time::Date              DATE
/// time::Time              TIME
/// [PgTimeTz]              TIMETZ
///
/// uuid::Uuid              UUID
///
/// ipnetwork::IpNetwork    INET, CIDR
/// std::net::IpAddr        INET, CIDR
///
/// mac_address::MacAddress MACADDR
///
/// bit_vec::BitVec         BIT, VARBIT
///
/// serde_json::Value       JSON, JSONB
///
/// PostgreSQL 类型转换为Rust对应类型
fn t2t(ty: &str) -> &str {
    match ty.to_uppercase().as_str() {
        "BOOL" => "bool",
        "CHAR" => "i8",
        "SMALLINT" | "SMALLSERIAL" | "INT2" => "i16",
        "INT" | "SERIAL" | "INT4" => "i32",
        "BIGINT" | "BIGSERIAL" | "INT8" => "i64",
        "REAL" | "FLOAT4" => "f32",
        "DOUBLE PRECISION" | "FLOAT8" => "f64",
        "BYTEA" => "Vec<u8>",
        "VOID" => "()",
        "INTERVAL" => "sqlx_postgres::types::PgInterval",
        "INT8RANGE" | "INT4RANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" | "NUMRANGE" => {
            "sqlx_postgres::types::PgRange<T> "
        }
        "MONEY" => "sqlx_postgres::types::PgMoney",
        "LTREE" => "sqlx_postgres::types::PgLTree",
        "LQUERY" => "sqlx_postgres::types::PgLQuery",
        "YEAR" => "time::Date",
        "DATE" => "time::Date",
        "TIME" => "time::Time",
        "TIMESTAMP" => "time::PrimitiveDateTime",
        "TIMESTAMPTZ" => "time::OffsetDateTime",
        "TIMETZ" => "sqlx_postgres::types::PgTimeTz",
        "NUMERIC" => "bigdecimal::BigDecimal",
        "JSON" | "JSONB" => "serde_json:JsonValue",
        "UUID" => "uuid::Uuid",
        "INET" | "CIDR" => "std::net::IpAddr",
        "MACADDR" => "mac_address::MacAddress",
        "BIT" | "VARBIT" => "bit_vec::BitVec",
        _ => "String",
    }
}

impl PostgresMetadata {
    pub fn new(pool: AnyPool) -> Self {
        Self(pool)
    }
}

impl DatabaseMetadata for PostgresMetadata {
    fn schemas(&self) -> super::BoxFuture<'_, Result<Vec<super::Schema>>> {
        todo!()
    }

    fn tables<'a>(&'a self, schema: &'a str) -> super::BoxFuture<'a, Result<Vec<super::Table>>> {
        Box::pin(async move {
            let rows: Vec<Table> = sqlx::query_as(SHOW_TABLES)
                .bind(schema)
                .bind(schema)
                .fetch_all(&self.0)
                .await?;
            Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
        })
    }

    fn columns<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> super::BoxFuture<'a, Result<Vec<super::Column>>> {
        //         let mut sql = format!(
        //             "
        // SELECT
        // 	col.table_catalog,
        // 	col.table_schema,
        // 	col.TABLE_NAME,
        // 	col.COLUMN_NAME,
        // 	col.ordinal_position,
        // 	col.column_default,
        // 	col.is_nullable,
        // 	col.udt_name as data_type,
        // 	col.character_maximum_length,
        // 	d.description
        // FROM
        // 	information_schema.COLUMNS col
        // 	JOIN pg_class C ON C.relname = col.
        // 	TABLE_NAME LEFT JOIN pg_description d ON d.objoid = C.OID
        // 	AND d.objsubid = col.ordinal_position
        // WHERE
        // 	col.table_catalog = ?	AND col.table_schema = ?
        // ORDER BY
        // 	col.TABLE_NAME,
        // 	col.ordinal_position;",
        //         );
        //
        //         let rows: Vec<Column> = sqlx::query(&format!(
        //             "SHOW FULL COLUMNS FROM {table_name} FROM {schema}"
        //         ))
        //         .bind(&table_name)
        //         .bind(&schema)
        //         .map(|row: AnyRow| {
        //             let field = row.get(0);
        //             let r#type: Vec<u8> = row.get(1);
        //             let r#type = String::from_utf8_lossy(&r#type).to_string();
        //             let null: String = row.get(3);
        //             let key: String = row.get(4);
        //             let default: Option<Vec<u8>> = row.get(5);
        //             let default = default.map(|d| String::from_utf8_lossy(&d).to_string());
        //             let extra: String = row.get(6);
        //             let comment: Vec<u8> = row.get(8);
        //             let comment = String::from_utf8_lossy(&comment).to_string();
        //
        //             let mut coloumn = Column {
        //                 table_schema: schema.into(),
        //                 table_name: table_name.into(),
        //                 column_name: field,
        //                 ..Default::default()
        //             };
        //             // coloumn.handle_column_as_type(&r#type).is_ok();
        //             // coloumn.handle_primary_key(&key);
        //             // coloumn.handle_is_null(null);
        //             // coloumn.handle_is_auto_incr(extra);
        //             coloumn
        //         })
        //         .fetch_all(&self.0)
        //         .await?;
        //         Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
        todo!()
    }

    fn indexs<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> super::BoxFuture<'a, Result<Vec<super::Index>>> {
        todo!()
    }

    fn create_table_sql<'a>(
        &'a self,
        schema: &'a str,
        table_name: &'a str,
    ) -> super::BoxFuture<'a, Result<String>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "postgres://postgres:postgres@localhost:5432/postgres";

    async fn meta() -> Result<PostgresMetadata> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(URL).await?;
        Ok(PostgresMetadata::new(pool))
    }

    #[tokio::test]
    async fn test_schemas() -> Result<()> {
        let meta = meta().await?;
        let _ = meta.schemas().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_tables() -> Result<()> {
        let meta = meta().await?;
        let _ = meta.tables("differ").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_columns() -> Result<()> {
        let meta = meta().await?;
        let _ = meta.columns("differ", "db_detail").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_indexs() -> Result<()> {
        let meta = meta().await?;
        let _ = meta.indexs("differ", "db_detail").await?;
        Ok(())
    }
}

use std::collections::HashMap;

use database_core::{
    Column, ColumnType, Driver, Index, database_metadata,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection, Row};

use super::DatasourceInfo;

mod report;
mod standard_check;

pub use report::{DiffReport, FieldInfo, IndexInfo, TableInfo, diff_report, diff_sql};
pub use standard_check::{CheckReportBo, StandardCheck, Suggest, standard_check};

/// 所有表结构信息
pub async fn table_struct(datasource_info: &DatasourceInfo) -> Result<HashMap<String, TableBo>> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_table_struct(datasource_info).await;
    }

    let meta = database_metadata(&datasource_info.url()).await;

    let Some(database) = &datasource_info.database else {
        return Err(Error::E("choose database"));
    };

    let tables = meta.tables(database, "").await?;
    let mut data = HashMap::with_capacity(tables.len());
    for table in tables.into_iter() {
        // 表字段
        let columns = meta
            .columns(database, "", &table.name)
            .await?
            .into_iter()
            .map(|c| (c.name.clone(), c.into()))
            .collect::<HashMap<String, FieldBo>>();
        // 索引
        let indexs: HashMap<String, IndexBo> = {
            // 根据索引名称分组（将组合索引合并在一起）
            let indexs = meta
                .indexs(database, "", &table.name)
                .await?
                .into_iter()
                .map(IndexBo::from)
                .fold(
                    HashMap::new(),
                    |mut map: HashMap<String, Vec<IndexBo>>, ix| {
                        map.entry(ix.key_name.clone()).or_default().push(ix);
                        map
                    },
                );
            let indexs_len = indexs.len();
            indexs.into_iter().fold(
                HashMap::with_capacity(indexs_len),
                |mut map, (key_name, ixs)| {
                    let mut ix = ixs[0].clone();
                    ix.column_name = merge_index_name(ixs);
                    map.insert(key_name, ix);
                    map
                },
            )
        };

        data.insert(
            table.name.clone(),
            TableBo {
                name: table.name,
                comment: table.comment,
                columns,
                indexs,
                is_both_has: false,
            },
        );
    }
    Ok(data)
}

/// 合并组合索引的名称
fn merge_index_name(mut indexs: Vec<IndexBo>) -> String {
    indexs.sort_by_key(|ix| ix.seq_in_index);
    indexs
        .into_iter()
        .map(|ix| {
            format!("`{}`{}", ix.column_name, {
                match ix.sub_part {
                    Some(sp) => format!("({sp})"),
                    None => "".into(),
                }
            })
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn merge_pg_index_name(mut indexs: Vec<IndexBo>) -> String {
    indexs.sort_by_key(|ix| ix.seq_in_index);
    indexs
        .into_iter()
        .map(|ix| quote_pg_ident(&ix.column_name))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn pg_column_definition(column: &FieldBo) -> String {
    let mut sql = format!(
        "{} {}",
        quote_pg_ident(&column.name),
        pg_column_db_type(column)
    );

    if let Some(default) = &column.default {
        sql.push_str(" DEFAULT ");
        sql.push_str(default);
    }

    if !column.is_null || column.is_primary_key {
        sql.push_str(" NOT NULL");
    }

    sql
}

pub(crate) fn pg_column_db_type(column: &FieldBo) -> String {
    if !column.db_type.is_empty() {
        return column.db_type.clone();
    }

    match column.r#type.unwrap_or(ColumnType::Text) {
        ColumnType::Bigint => "bigint".to_string(),
        ColumnType::SmallInt => "smallint".to_string(),
        ColumnType::Int | ColumnType::Integer => "integer".to_string(),
        ColumnType::Decimal | ColumnType::Numeric => {
            if let Some(length) = column.length {
                if let Some(scale) = column.scale {
                    format!("numeric({length},{scale})")
                } else {
                    format!("numeric({length})")
                }
            } else {
                "numeric".to_string()
            }
        }
        ColumnType::Float | ColumnType::Real => "real".to_string(),
        ColumnType::Double => "double precision".to_string(),
        ColumnType::Date => "date".to_string(),
        ColumnType::Time => "time".to_string(),
        ColumnType::Timestamp | ColumnType::DateTime => "timestamp without time zone".to_string(),
        ColumnType::Json => "jsonb".to_string(),
        ColumnType::Char => match column.length {
            Some(length) if length > 0 => format!("character({length})"),
            _ => "character".to_string(),
        },
        ColumnType::VarChar => match column.length {
            Some(length) if length > 0 => format!("character varying({length})"),
            _ => "character varying".to_string(),
        },
        ColumnType::Text | ColumnType::LongText | ColumnType::MediumText | ColumnType::TinyText => {
            "text".to_string()
        }
        ColumnType::Blob | ColumnType::LongBlob | ColumnType::MediumBlob | ColumnType::TinyBlob => {
            "bytea".to_string()
        }
        ColumnType::TinyInt | ColumnType::Bit => "boolean".to_string(),
        _ => "text".to_string(),
    }
}

pub(crate) fn pg_create_index_sql(index: &IndexBo) -> String {
    let (schema, table) = split_pg_table_name(&index.table_name);
    let mut sql = String::from("CREATE ");
    if index.non_unique == 0 {
        sql.push_str("UNIQUE ");
    }
    sql.push_str("INDEX ");
    sql.push_str(&quote_pg_ident(&index.key_name));
    sql.push_str(" ON ");
    sql.push_str(&quote_pg_table(schema, table));
    if !index.index_type.is_empty() {
        sql.push_str(" USING ");
        sql.push_str(&index.index_type.to_lowercase());
    }
    sql.push_str(" (");
    sql.push_str(&index.column_name);
    sql.push(')');
    sql
}

pub(crate) fn quote_pg_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

pub(crate) fn quote_pg_table(schema: &str, table: &str) -> String {
    format!("{}.{}", quote_pg_ident(schema), quote_pg_ident(table))
}

pub(crate) fn split_pg_table_name(table_name: &str) -> (&str, &str) {
    table_name.split_once('.').unwrap_or(("public", table_name))
}

pub(crate) fn escape_sql_string(value: &str) -> String {
    value.replace('\'', "''")
}

fn pg_table_key(schema: &str, table: &str) -> String {
    format!("{schema}.{table}")
}

fn pg_column_type(type_name: &str, formatted_type: &str) -> ColumnType {
    match type_name {
        "int2" | "smallserial" => ColumnType::SmallInt,
        "int4" | "serial" => ColumnType::Int,
        "int8" | "bigserial" => ColumnType::Bigint,
        "numeric" => ColumnType::Decimal,
        "float4" => ColumnType::Float,
        "float8" => ColumnType::Double,
        "bool" => ColumnType::TinyInt,
        "date" => ColumnType::Date,
        "time" | "timetz" => ColumnType::Time,
        "timestamp" | "timestamptz" => ColumnType::Timestamp,
        "varchar" => ColumnType::VarChar,
        "bpchar" => ColumnType::Char,
        "text" => ColumnType::Text,
        "json" | "jsonb" => ColumnType::Json,
        "bytea" => ColumnType::Blob,
        "uuid" => ColumnType::Char,
        _ if formatted_type.contains("char") => ColumnType::VarChar,
        _ => ColumnType::Text,
    }
}

pub async fn create_table_sql(
    datasource_info: &DatasourceInfo,
    table_name: &str,
) -> Result<String> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_create_table_sql(datasource_info, table_name).await;
    }

    let meta = database_metadata(&datasource_info.url()).await;
    let sql = meta
        .create_table_sql(
            &datasource_info.database.clone().unwrap_or_default(),
            "",
            table_name,
        )
        .await?;
    Ok(sql)
}

async fn postgres_table_struct(
    datasource_info: &DatasourceInfo,
) -> Result<HashMap<String, TableBo>> {
    let mut conn = PgConnection::connect(&datasource_info.url()).await?;
    let tables = postgres_tables(&mut conn).await?;
    let mut data = HashMap::with_capacity(tables.len());

    for table in tables {
        let table_key = pg_table_key(&table.schema, &table.name);
        let columns = postgres_columns(&mut conn, &table.schema, &table.name, &table_key)
            .await?
            .into_iter()
            .map(|field| (field.name.clone(), field))
            .collect::<HashMap<_, _>>();
        let indexs = postgres_indexes(&mut conn, &table.schema, &table.name, &table_key).await?;

        data.insert(
            table_key.clone(),
            TableBo {
                name: table_key,
                comment: table.comment,
                columns,
                indexs,
                is_both_has: false,
            },
        );
    }

    Ok(data)
}

#[derive(Debug)]
struct PgTable {
    schema: String,
    name: String,
    comment: String,
}

async fn postgres_tables(conn: &mut PgConnection) -> Result<Vec<PgTable>> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.nspname::text,
            c.relname::text,
            COALESCE(obj_description(c.oid, 'pg_class'), '')::text
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r', 'p')
          AND n.nspname NOT IN ('pg_catalog', 'information_schema')
          AND n.nspname NOT LIKE 'pg_toast%'
        ORDER BY CASE WHEN n.nspname = 'public' THEN 0 ELSE 1 END, n.nspname, c.relname
        "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PgTable {
            schema: row.get(0),
            name: row.get(1),
            comment: row.get(2),
        })
        .collect())
}

async fn postgres_columns(
    conn: &mut PgConnection,
    schema: &str,
    table: &str,
    table_key: &str,
) -> Result<Vec<FieldBo>> {
    let rows = sqlx::query(
        r#"
        SELECT
            a.attname::text,
            format_type(a.atttypid, a.atttypmod)::text AS db_type,
            t.typname::text,
            CASE
                WHEN a.atttypmod > 0 AND t.typname IN ('varchar', 'bpchar') THEN a.atttypmod - 4
                ELSE NULL
            END::int4 AS character_length,
            CASE
                WHEN a.atttypmod > 0 AND t.typname = 'numeric' THEN ((a.atttypmod - 4) >> 16) & 65535
                ELSE NULL
            END::int4 AS numeric_precision,
            CASE
                WHEN a.atttypmod > 0 AND t.typname = 'numeric' THEN (a.atttypmod - 4) & 65535
                ELSE NULL
            END::int4 AS numeric_scale,
            pg_get_expr(ad.adbin, ad.adrelid)::text AS column_default,
            COALESCE(col_description(c.oid, a.attnum), '')::text AS column_comment,
            (NOT a.attnotnull) AS is_nullable,
            (
                SELECT EXISTS (
                    SELECT 1
                    FROM pg_constraint con
                    WHERE con.conrelid = c.oid
                      AND con.contype = 'p'
                      AND a.attnum = ANY(con.conkey)
                )
            ) AS is_primary_key,
            (
                SELECT EXISTS (
                    SELECT 1
                    FROM pg_index ix
                    WHERE ix.indrelid = c.oid
                      AND ix.indisunique
                      AND a.attnum = ANY(ix.indkey)
                )
            ) AS is_unique,
            (a.attidentity <> '' OR COALESCE(pg_get_expr(ad.adbin, ad.adrelid), '') LIKE 'nextval(%') AS is_auto_incr
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_type t ON t.oid = a.atttypid
        LEFT JOIN pg_attrdef ad ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
        WHERE a.attnum > 0
          AND NOT a.attisdropped
          AND n.nspname = $1
          AND c.relname = $2
        ORDER BY a.attnum
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let db_type: String = row.get(1);
            let type_name: String = row.get(2);
            let character_length: Option<i32> = row.get(3);
            let numeric_precision: Option<i32> = row.get(4);
            FieldBo {
                table_name: table_key.to_string(),
                name: row.get(0),
                r#type: Some(pg_column_type(&type_name, &db_type)),
                db_type,
                length: character_length.or(numeric_precision),
                scale: row.get(5),
                default: row.get(6),
                enum_values: None,
                comment: row.get(7),
                is_null: row.get(8),
                is_primary_key: row.get(9),
                is_unique: row.get(10),
                is_auto_incr: row.get(11),
                is_unsigned: false,
                is_both_has: false,
            }
        })
        .collect())
}

async fn postgres_indexes(
    conn: &mut PgConnection,
    schema: &str,
    table: &str,
    table_key: &str,
) -> Result<HashMap<String, IndexBo>> {
    let rows = sqlx::query(
        r#"
        SELECT
            CASE WHEN ix.indisprimary THEN 'PRIMARY' ELSE ic.relname::text END AS key_name,
            CASE WHEN ix.indisunique THEN 0 ELSE 1 END::int4 AS non_unique,
            cols.ordinality::int4 AS seq_in_index,
            a.attname::text AS column_name,
            am.amname::text AS index_type,
            COALESCE(obj_description(ic.oid, 'pg_class'), '')::text AS index_comment
        FROM pg_index ix
        JOIN pg_class c ON c.oid = ix.indrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_class ic ON ic.oid = ix.indexrelid
        JOIN pg_am am ON am.oid = ic.relam
        JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS cols(attnum, ordinality) ON TRUE
        JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = cols.attnum
        WHERE n.nspname = $1
          AND c.relname = $2
        ORDER BY key_name, cols.ordinality
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_all(conn)
    .await?;

    let mut grouped = HashMap::<String, Vec<IndexBo>>::new();
    for row in rows {
        let key_name: String = row.get(0);
        grouped.entry(key_name.clone()).or_default().push(IndexBo {
            table_name: table_key.to_string(),
            non_unique: row.get(1),
            key_name,
            seq_in_index: row.get::<i32, _>(2) as u32,
            column_name: row.get(3),
            sub_part: None,
            index_type: row.get::<String, _>(4).to_uppercase(),
            index_comment: row.get(5),
            is_both_has: false,
        });
    }

    let mut indexs = HashMap::with_capacity(grouped.len());
    for (key_name, ixs) in grouped {
        let mut ix = ixs[0].clone();
        ix.column_name = merge_pg_index_name(ixs);
        indexs.insert(key_name, ix);
    }

    Ok(indexs)
}

async fn postgres_create_table_sql(
    datasource_info: &DatasourceInfo,
    table_name: &str,
) -> Result<String> {
    let mut conn = PgConnection::connect(&datasource_info.url()).await?;
    let (schema, table) = split_pg_table_name(table_name);
    let table_key = pg_table_key(schema, table);
    let columns = postgres_columns(&mut conn, schema, table, &table_key).await?;
    if columns.is_empty() {
        return Err(Error::E("table not found"));
    }
    let indexs = postgres_indexes(&mut conn, schema, table, &table_key).await?;
    let table_comment = postgres_table_comment(&mut conn, schema, table).await?;

    let mut lines = columns.iter().map(pg_column_definition).collect::<Vec<_>>();

    let primary_columns = columns
        .iter()
        .filter(|column| column.is_primary_key)
        .map(|column| quote_pg_ident(&column.name))
        .collect::<Vec<_>>();
    if !primary_columns.is_empty() {
        lines.push(format!("PRIMARY KEY ({})", primary_columns.join(", ")));
    }

    let mut sql = format!(
        "CREATE TABLE {} (\n  {}\n)",
        quote_pg_table(schema, table),
        lines.join(",\n  ")
    );

    if !table_comment.is_empty() {
        sql.push_str(&format!(
            ";\n\nCOMMENT ON TABLE {} IS '{}'",
            quote_pg_table(schema, table),
            escape_sql_string(&table_comment)
        ));
    }

    for column in columns.iter().filter(|column| !column.comment.is_empty()) {
        sql.push_str(&format!(
            ";\n\nCOMMENT ON COLUMN {}.{} IS '{}'",
            quote_pg_table(schema, table),
            quote_pg_ident(&column.name),
            escape_sql_string(&column.comment)
        ));
    }

    for index in indexs.values().filter(|index| index.key_name != "PRIMARY") {
        sql.push_str(";\n\n");
        sql.push_str(&pg_create_index_sql(index));
    }

    Ok(sql)
}

async fn postgres_table_comment(
    conn: &mut PgConnection,
    schema: &str,
    table: &str,
) -> Result<String> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(obj_description(c.oid, 'pg_class'), '')::text
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = $1 AND c.relname = $2
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_one(conn)
    .await?;

    Ok(row.get(0))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableBo {
    /// 表名
    name: String,
    /// 注释
    comment: String,
    /// 字段
    columns: HashMap<String, FieldBo>,
    /// 索引
    indexs: HashMap<String, IndexBo>,

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldBo {
    // 表名
    pub table_name: String,
    /// 字段名
    pub name: String,
    /// 字段类型
    pub r#type: Option<ColumnType>,
    /// 数据库原始类型，用于 PostgreSQL 等方言生成差异 SQL
    pub db_type: String,
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

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}

impl From<Column> for FieldBo {
    fn from(c: Column) -> Self {
        Self {
            table_name: c.table_name,
            name: c.name,
            r#type: c.r#type,
            db_type: String::new(),
            length: c.length,
            scale: c.scale,
            default: c.default,
            enum_values: c.enum_values,
            comment: c.comment,
            is_null: c.is_null,
            is_auto_incr: c.is_auto_incr,
            is_unique: c.is_unique,
            is_primary_key: c.is_primary_key,
            is_unsigned: c.is_unsigned,
            is_both_has: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexBo {
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

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}

impl From<Index> for IndexBo {
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
            is_both_has: false,
        }
    }
}

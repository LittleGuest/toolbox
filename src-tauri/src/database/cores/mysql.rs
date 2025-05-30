use std::fmt::format;

use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, AnyPool, Database, FromRow, MySql, MySqlPool, Pool, Row};

use super::{DatabaseMetadata, Result};

pub struct MysqlMetadata;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
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
    table_schema: String,
    table_name: String,
    // table_type: String,
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
    table_schema: String,
    table_name: String,
    column_name: String,
    ordinal_position: Option<u32>,
    column_default: Option<String>,
    is_nullable: String,
    character_maximum_length: Option<i64>,
    // data_type: Option<String>,
    // column_type: String,
    // column_key: Option<String>,
    // column_comment: Option<String>,
}

impl From<Column> for super::Column {
    fn from(c: Column) -> Self {
        Self {
            schema: c.table_schema,
            table_name: c.table_name,
            name: c.column_name,
            // r#type: c.column_type,
            max_length: c.character_maximum_length,
            is_nullable: c.is_nullable,
            // key: c.column_key,
            default: c.column_default,
            // comment: c.column_comment,
            ..Default::default()
        }
    }
}

impl DatabaseMetadata for MysqlMetadata {
    type Pool = MySqlPool;

    async fn schemas(pool: &Self::Pool) -> Result<Vec<super::Schema>> {
        let sql = "SHOW DATABASES";
        let rows = sqlx::query(sql)
            .map(|row: MySqlRow| Schema { name: row.get(0) })
            .map(|row| row.into())
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn tables(pool: &Self::Pool, schema: &str) -> Result<Vec<super::Table>> {
        let sql = "SELECT table_schema table_schema, table_name table_name, table_type table_type, table_comment table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ?";
        let rows: Vec<Table> = sqlx::query_as(sql).bind(schema).fetch_all(pool).await?;
        Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
    }

    async fn columns(
        pool: &Self::Pool,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<super::Column>> {
        let sql = "SELECT table_schema table_schema, table_name table_name,column_name column_name,ordinal_position ordinal_position,column_default column_default,is_nullable is_nullable,data_type data_type,character_maximum_length character_maximum_length,column_type column_type,column_key column_key,column_comment column_comment FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";
        let rows: Vec<Column> = sqlx::query_as(sql)
            .bind(schema)
            .bind(table_name)
            .fetch_all(pool)
            .await?;
        Ok(rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())
    }
}

use std::fmt::format;

use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, AnyPool, Database, FromRow, MySql, MySqlPool, Pool, Row};

use super::{Column, DatabaseMetadata, Result, Schema, Table};

pub struct MysqlMetadata;

impl DatabaseMetadata for MysqlMetadata {
    type Pool = MySqlPool;

    async fn schemas(pool: &Self::Pool) -> Result<Vec<Schema>> {
        let sql = "SHOW DATABASES";
        let rows = sqlx::query(sql)
            .map(|row: MySqlRow| Schema { name: row.get(0) })
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn tables(pool: &Self::Pool, schema: &str) -> Result<Vec<Table>> {
        let sql = "SELECT TABLE_NAME, TABLE_COMMENT FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ?";
        let rows = sqlx::query(sql)
            .bind(schema)
            .map(|row: MySqlRow| Table {
                schema: schema.into(),
                name: row.get(0),
                comment: row.get(1),
            })
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn columns(pool: &Self::Pool, schema: &str, table_name: &str) -> Result<Vec<Column>> {
        let sql = format!("SHOW FULL COLUMNS FROM {table_name} FROM {schema}");
        let rows = sqlx::query(&sql)
            .bind(table_name)
            .bind(schema)
            .map(|row: MySqlRow| Column {
                schema: schema.into(),
                table_name: table_name.into(),
                name: row.get(0),
                // r#type: row.get(1),
                max_length: None,
                is_nullable: row.get(3),
                // key: row.get(4),
                // default: row.get(5),
                // comment: row.get(8),
                ..Default::default()
            })
            .fetch_all(pool)
            .await
            .unwrap();
        Ok(rows)
    }
}

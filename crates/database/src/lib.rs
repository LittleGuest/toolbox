use std::collections::HashMap;

use rust_embed::Embed;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Connection, MySqlConnection, PgConnection, Row, SqliteConnection};

mod diff;
mod generator;

pub use database_core::{
    Column, ColumnType, DatabaseMetadata, Driver, Schema, Table, database_metadata,
    error::{Error, Result},
};
pub use diff::{
    CheckReportBo, DiffReport, FieldInfo, IndexInfo, Suggest, StandardCheck, TableInfo,
    diff_report, diff_sql, standard_check,
};

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasourceInfo {
    /// 数据库驱动
    #[serde(deserialize_with = "deserialize_driver")]
    pub driver: Driver,
    /// 数据源名称
    pub name: String,
    /// 数据库主机地址
    pub host: String,
    /// 数据库端口号
    pub port: Option<u16>,
    /// 数据库账号
    pub username: Option<String>,
    /// 数据库密码
    pub password: Option<String>,
    /// 指定的数据库名称
    pub database: Option<String>,
}

/// Driver::Mysql       mysql://root:root@localhost:3306/test
/// Driver::Postgres    postgres://root:root@localhost:5432/test
/// Driver::Sqlite      sqlite://test.sqlite
impl DatasourceInfo {
    pub fn url(&self) -> String {
        match self.driver {
            Driver::Mysql => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    encode_component(self.username.as_deref().unwrap_or_default()),
                    encode_component(self.password.as_deref().unwrap_or_default()),
                    self.host,
                    self.port.unwrap_or(3306),
                    encode_component(self.database.as_deref().unwrap_or_default())
                )
            }
            Driver::Postgres => format!(
                "postgres://{}:{}@{}:{}/{}",
                encode_component(self.username.as_deref().unwrap_or_default()),
                encode_component(self.password.as_deref().unwrap_or_default()),
                self.host,
                self.port.unwrap_or(5432),
                encode_component(self.database.as_deref().unwrap_or_default())
            ),
            Driver::Sqlite => format!("sqlite://{}", self.database.clone().unwrap_or_default()),
        }
    }
}

fn deserialize_driver<'de, D>(deserializer: D) -> std::result::Result<Driver, D::Error>
where
    D: Deserializer<'de>,
{
    let driver = String::deserialize(deserializer)?;
    match driver.to_ascii_lowercase().as_str() {
        "mysql" => Ok(Driver::Mysql),
        "postgres" | "postgresql" => Ok(Driver::Postgres),
        "sqlite" => Ok(Driver::Sqlite),
        other => Err(serde::de::Error::custom(format!(
            "unsupported database driver: {other}"
        ))),
    }
}

fn encode_component(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

pub async fn database_ping(datasource_info: DatasourceInfo) -> Result<()> {
    match datasource_info.driver {
        Driver::Mysql => {
            let mut conn = MySqlConnection::connect(&datasource_info.url()).await?;
            conn.ping().await.map_err(Error::Sql)
        }
        Driver::Postgres => {
            let mut conn = PgConnection::connect(&datasource_info.url()).await?;
            conn.ping().await.map_err(Error::Sql)
        }
        Driver::Sqlite => {
            let mut conn = SqliteConnection::connect(&datasource_info.url()).await?;
            conn.ping().await.map_err(Error::Sql)
        }
    }
}

pub async fn database_schemas(datasource_info: DatasourceInfo) -> Result<Vec<Schema>> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_schemas(&datasource_info).await;
    }

    database_metadata(&datasource_info.url())
        .await
        .schemas()
        .await
}

pub async fn database_tables(datasource_info: DatasourceInfo) -> Result<Vec<Table>> {
    let metadata = database_metadata(&datasource_info.url()).await;
    let database = datasource_info.database.as_deref().unwrap_or_default();
    let schemas = table_schemas(&datasource_info).await?;
    let mut data = Vec::new();

    for schema in schemas {
        data.extend(metadata.tables(database, &schema).await?);
    }
    Ok(data)
}

/// 表信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnTree {
    pub schema: String,
    pub table_name: String,
    pub table_comment: String,
    pub children: Vec<Column>,
}

pub async fn database_table_tree(datasource_info: DatasourceInfo) -> Result<Vec<TableColumnTree>> {
    let Some(database) = &datasource_info.database else {
        return Err(Error::E("choose database"));
    };
    let meta = database_metadata(&datasource_info.url()).await;
    let schemas = table_schemas(&datasource_info).await?;
    let mut data = Vec::new();
    for schema in schemas {
        let tables = meta.tables(database, &schema).await?;
        for table in tables.into_iter() {
            let columns = meta.columns(database, &table.schema, &table.name).await?;
            data.push(TableColumnTree {
                schema: table.schema,
                table_name: table.name,
                table_comment: table.comment,
                children: columns,
            });
        }
    }
    Ok(data)
}

async fn table_schemas(datasource_info: &DatasourceInfo) -> Result<Vec<String>> {
    match datasource_info.driver {
        Driver::Mysql => datasource_info
            .database
            .clone()
            .filter(|database| !database.is_empty())
            .map(|database| vec![database])
            .ok_or(Error::E("choose database")),
        Driver::Postgres => postgres_schemas(datasource_info)
            .await
            .map(|schemas| schemas.into_iter().map(|schema| schema.name).collect()),
        Driver::Sqlite => Ok(vec![String::new()]),
    }
}

async fn postgres_schemas(datasource_info: &DatasourceInfo) -> Result<Vec<Schema>> {
    let mut conn = PgConnection::connect(&datasource_info.url()).await?;
    let rows = sqlx::query(
        r#"
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
          AND schema_name NOT LIKE 'pg_toast%'
        ORDER BY CASE WHEN schema_name = 'public' THEN 0 ELSE 1 END, schema_name
        "#,
    )
    .fetch_all(&mut conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Schema { name: row.get(0) })
        .collect())
}

pub async fn database_diff_report(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<DiffReport> {
    diff::diff_report(source, target).await
}

pub async fn database_diff_sql(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<Vec<String>> {
    diff::diff_sql(source, target).await
}

pub async fn database_standard_check_codes() -> Vec<HashMap<String, String>> {
    diff::StandardCheck::codes()
}

pub async fn database_standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> Result<Vec<CheckReportBo>> {
    diff::standard_check(source, check_codes).await
}

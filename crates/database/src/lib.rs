use std::collections::HashMap;

use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Connection, MySqlConnection};

mod diff;
mod generator;

pub use database::{
    Column, DatabaseMetadata, Driver, Schema, Table, database_metadata,
    error::{Error, Result},
};
pub use diff::{CheckReportBo, DiffReport, StandardCheck, diff_report, diff_sql, standard_check};

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasourceInfo {
    /// 数据库驱动
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
                    self.username.clone().unwrap_or_default(),
                    self.password.clone().unwrap_or_default(),
                    self.host,
                    self.port.unwrap_or_default(),
                    self.database.clone().unwrap_or_default()
                )
            }
            Driver::Postgres => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username.clone().unwrap_or_default(),
                self.password.clone().unwrap_or_default(),
                self.host,
                self.port.unwrap_or_default(),
                self.database.clone().unwrap_or_default()
            ),
            Driver::Sqlite => format!("sqlite://{}", self.database.clone().unwrap_or_default()),
        }
    }
}

// TODO:#[tauri::command]
pub async fn database_ping(datasource_info: DatasourceInfo) -> Result<()> {
    match datasource_info.driver {
        Driver::Mysql => {
            let mut conn = MySqlConnection::connect(&datasource_info.url()).await?;
            conn.ping().await.map_err(Error::Sql)
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

// TODO:#[tauri::command]
pub async fn database_schemas(datasource_info: DatasourceInfo) -> Result<Vec<Schema>> {
    database_metadata(&datasource_info.url())
        .await
        .schemas()
        .await
}

// TODO:#[tauri::command]
pub async fn database_tables(datasource_info: DatasourceInfo) -> Result<Vec<Table>> {
    database_metadata(&datasource_info.url())
        .await
        .tables(&datasource_info.database.unwrap_or_default(), "")
        .await
}

/// 表信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnTree {
    schema: String,
    table_name: String,
    table_comment: String,
    children: Vec<Column>,
}

// TODO:#[tauri::command]
pub async fn database_table_tree(datasource_info: DatasourceInfo) -> Result<Vec<TableColumnTree>> {
    let Some(database) = &datasource_info.database else {
        return Err(Error::E("choose database"));
    };
    let meta = database_metadata(&datasource_info.url()).await;
    let tables = meta.tables(database, "").await?;
    let mut data = Vec::with_capacity(tables.len());
    for table in tables.into_iter() {
        let columns = meta.columns(database, "", &table.name).await?;
        data.push(TableColumnTree {
            schema: table.schema,
            table_name: table.name,
            table_comment: table.comment,
            children: columns,
        });
    }
    Ok(data)
}

// TODO:#[tauri::command]
pub async fn database_diff_report(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<DiffReport> {
    diff::diff_report(source, target).await
}

// TODO:#[tauri::command]
pub async fn database_diff_sql(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<Vec<String>> {
    diff::diff_sql(source, target).await
}

// TODO:#[tauri::command]
pub async fn database_standard_check_codes() -> Vec<HashMap<String, String>> {
    diff::StandardCheck::codes()
}

// TODO:#[tauri::command]
pub async fn database_standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> Result<Vec<CheckReportBo>> {
    diff::standard_check(source, check_codes).await
}

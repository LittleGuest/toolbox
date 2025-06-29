use std::collections::HashMap;

use cores::{Column, DatabaseMetadata, Driver, Error, MysqlMetadata, Result, Schema};
use diff::DiffReport;
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Connection, MySqlConnection};

use crate::database::{
    cores::{PostgresMetadata, SqliteMetadata, Table},
    diff::CheckReportBo,
};

mod cores;
mod diff;
mod generator;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasourceInfo {
    /// 数据库驱动
    driver: Driver,
    /// 数据源名称
    name: String,
    /// 数据库主机地址
    host: String,
    /// 数据库端口号
    port: Option<u16>,
    /// 数据库账号
    username: Option<String>,
    /// 数据库密码
    password: Option<String>,
    /// 指定的数据库名称
    database: Option<String>,
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

    pub async fn database_metadata(&self) -> Box<dyn DatabaseMetadata> {
        let pool = AnyPool::connect(&self.url()).await.unwrap();
        match self.driver {
            Driver::Mysql => Box::new(MysqlMetadata::new(pool)),
            Driver::Postgres => Box::new(PostgresMetadata::new(pool)),
            Driver::Sqlite => Box::new(SqliteMetadata::new(pool)),
        }
    }
}

#[tauri::command]
pub async fn database_ping(datasource_info: DatasourceInfo) -> Result<()> {
    match datasource_info.driver {
        Driver::Mysql => {
            let mut conn = MySqlConnection::connect(&datasource_info.url()).await?;
            conn.ping().await.map_err(Error::SqlxErr)
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

#[tauri::command]
pub async fn database_schemas(datasource_info: DatasourceInfo) -> Result<Vec<Schema>> {
    datasource_info.database_metadata().await.schemas().await
}

#[tauri::command]
pub async fn database_tables(datasource_info: DatasourceInfo) -> Result<Vec<Table>> {
    datasource_info
        .database_metadata()
        .await
        .tables(&datasource_info.database.unwrap_or_default())
        .await
}

#[derive(Debug, Serialize)]
pub struct TableColumnTree {
    table_name: String,
    children: Vec<Column>,
}

#[tauri::command]
pub async fn database_table_tree(datasource_info: DatasourceInfo) -> Result<Vec<TableColumnTree>> {
    let Some(database) = &datasource_info.database else {
        return Err(Error::E("choose database"));
    };
    let meta = datasource_info.database_metadata().await;
    let tables = meta.tables(database).await?;
    let mut data = Vec::with_capacity(tables.len());
    for table in tables.into_iter() {
        let columns = meta.columns(database, &table.name).await?;
        data.push(TableColumnTree {
            table_name: table.name,
            children: columns,
        });
    }
    Ok(data)
}

#[tauri::command]
pub async fn database_diff_report(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<DiffReport> {
    diff::diff_report(source, target).await
}

#[tauri::command]
pub async fn database_diff_sql(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> Result<Vec<String>> {
    diff::diff_sql(source, target).await
}

#[tauri::command]
pub async fn database_standard_check_codes() -> Vec<HashMap<String, String>> {
    diff::StandardCheck::codes()
}

#[tauri::command]
pub async fn database_standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> Result<Vec<CheckReportBo>> {
    diff::standard_check(source, check_codes).await
}

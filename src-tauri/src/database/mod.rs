use std::collections::HashMap;

use cores::{Column, DatabaseMetadata, Driver, Error, MysqlMetadata, Result, Schema};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

mod cores;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasourceInfo {
    driver: Driver,
    name: String,
    host: String,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
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
            Driver::Postgres => todo!(),
            Driver::Sqlite => todo!(),
        }
    }
}

#[tauri::command]
pub async fn database_test(datasource_info: DatasourceInfo) -> Result<bool> {
    todo!()
}

#[tauri::command]
pub async fn database_schemas(datasource_info: DatasourceInfo) -> Result<Vec<Schema>> {
    match datasource_info.driver {
        Driver::Mysql => {
            let pool = MySqlPool::connect(&datasource_info.url()).await?;
            MysqlMetadata::schemas(&pool).await
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

#[derive(Debug, Serialize)]
pub struct TableColumnTree {
    name: String,
    children: Vec<Column>,
}

#[tauri::command]
pub async fn database_table_tree(datasource_info: DatasourceInfo) -> Result<Vec<TableColumnTree>> {
    match datasource_info.driver {
        Driver::Mysql => {
            let Some(database) = &datasource_info.database else {
                return Err(Error::E("请选择数据库"));
            };

            let pool = MySqlPool::connect(&datasource_info.url()).await?;
            let tables = MysqlMetadata::tables(&pool, database).await?;
            let mut data = Vec::with_capacity(tables.len());
            for table in tables.into_iter() {
                let columns = MysqlMetadata::columns(&pool, database, &table.name).await?;
                data.push(TableColumnTree {
                    name: table.name,
                    children: columns,
                });
            }
            Ok(data)
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

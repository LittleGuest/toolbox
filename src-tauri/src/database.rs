use std::collections::HashMap;

use database::{
    CheckReportBo, Column, DiffReport, Driver, Schema, StandardCheck, Table, database_metadata, diff_report, diff_sql, standard_check, DatasourceInfo,
};
use serde::Serialize;
use sqlx::{Connection, MySqlConnection};

type ResultType<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn database_ping(datasource_info: DatasourceInfo) -> ResultType<()> {
    match datasource_info.driver {
        Driver::Mysql => {
            let mut conn = MySqlConnection::connect(&datasource_info.url()).await.map_err(|e| e.to_string())?;
            conn.ping().await.map_err(|e| e.to_string())
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

#[tauri::command]
pub async fn database_schemas(datasource_info: DatasourceInfo) -> ResultType<Vec<Schema>> {
    let meta = database_metadata(&datasource_info.url()).await;
    meta.schemas().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_tables(datasource_info: DatasourceInfo) -> ResultType<Vec<Table>> {
    let meta = database_metadata(&datasource_info.url()).await;
    meta.tables(&datasource_info.database.unwrap_or_default(), "").await.map_err(|e| e.to_string())
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

#[tauri::command]
pub async fn database_table_tree(datasource_info: DatasourceInfo) -> ResultType<Vec<TableColumnTree>> {
    let Some(database) = &datasource_info.database else {
        return Err("choose database".to_string());
    };
    let meta = database_metadata(&datasource_info.url()).await;
    let tables = meta.tables(database, "").await.map_err(|e| e.to_string())?;
    let mut data = Vec::with_capacity(tables.len());
    for table in tables.into_iter() {
        let columns = meta.columns(database, "", &table.name).await.map_err(|e| e.to_string())?;
        data.push(TableColumnTree {
            schema: table.schema,
            table_name: table.name,
            table_comment: table.comment,
            children: columns,
        });
    }
    Ok(data)
}

#[tauri::command]
pub async fn database_diff_report(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> ResultType<DiffReport> {
    diff_report(source, target).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_diff_sql(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> ResultType<Vec<String>> {
    diff_sql(source, target).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_standard_check_codes() -> Vec<HashMap<String, String>> {
    StandardCheck::codes()
}

#[tauri::command]
pub async fn database_standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> ResultType<Vec<CheckReportBo>> {
    standard_check(source, check_codes).await.map_err(|e| e.to_string())
}

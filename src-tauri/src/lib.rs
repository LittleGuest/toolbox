use std::sync::Mutex;

use sqlx::SqlitePool;
use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};
use thiserror::Error;

mod database;
mod libs;
mod openapi;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    E(&'static str),
    #[error("序列化错误: {0}")]
    SerializeErr(String),
    #[error("Url处理错误: {0}")]
    UrlErr(String),
    #[error("UUID生成错误: {0}")]
    UuidErr(String),
    #[error("Base64编码解码错误: {0}")]
    Base64Err(String),
    #[error("Hex编码解码错误: {0}")]
    HexErr(String),
    #[error("Ip地址转换错误: {0}")]
    IpErr(String),
    #[error("UrlParams转换错误: {0}")]
    UrlParamsErr(String),
    #[error("DateTimeErr转换错误: {0}")]
    DateTimeErr(String),
    // #[error(transparent)]
    // AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RequestErr(#[from] reqwest::Error),
    #[error(transparent)]
    TauriErr(#[from] tauri::Error),
    #[error(transparent)]
    SqlxErr(#[from] sqlx::Error),
    #[error(transparent)]
    TeraErr(#[from] tera::Error),
    #[error("未知错误")]
    Unknown,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        // .manage(DbPool(Mutex::new(pool)))
        .setup(|app| {
            // tauri_plugin_store::StoreBuilder::new(app, "store.bin").build();
            let db_url = app.path().app_data_dir().unwrap().join("toolbox.db");
            app.handle()
                .plugin(
                    tauri_plugin_sql::Builder::default()
                        .add_migrations(&db_url.to_str().unwrap(), migrations)
                        .build(),
                )
                .ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            libs::hash,
            libs::uuid,
            libs::encode_base64_text,
            libs::decode_base64_text,
            libs::encode_url,
            libs::decode_url,
            libs::cffc,
            libs::timestamp,
            libs::number_base,
            libs::qrcode,
            libs::check_ip,
            libs::ip_to_number,
            openapi::fetch_api_data,
            openapi::download,
            database::database_schemas,
            database::database_table_tree,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

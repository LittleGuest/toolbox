use rust_embed::Embed;
use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};
use thiserror::Error;

mod database;
mod datafaker;
mod libs;
mod openapi;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    E(String),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
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
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    sqlx::any::install_default_drivers();

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
                        .add_migrations(db_url.to_str().unwrap(), migrations)
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
            database::database_ping,
            database::database_schemas,
            database::database_tables,
            database::database_table_tree,
            database::database_diff_report,
            database::database_diff_sql,
            database::database_standard_check_codes,
            database::database_standard_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

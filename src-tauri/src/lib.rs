use rust_embed::Embed;
use thiserror::Error;

mod database;
mod datafaker;
mod libs;
mod migrations;
mod openapi;
mod monitor;

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

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:toolbox.db", migrations::migrations())
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            libs::hash,
            libs::checksum,
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
            datafaker::datafaker_providers,
            datafaker::datafaker_adapter_columns,
            datafaker::preview_regex,
            monitor::monitor_battery_info,
            monitor::monitor_cpu_info,
            monitor::monitor_memory_info,
            monitor::monitor_process_info,
            monitor::monitor_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

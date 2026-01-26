use thiserror::Error;

mod base;
mod database;
mod datafaker;
mod migrations;
mod monitor;

// pub type Result<T, E = Error> = std::result::Result<T, E>;

// #[derive(Debug, Error)]
// pub enum Error {
//     #[error("{0}")]
//     E(String),
//     #[error(transparent)]
//     AnyhowError(#[from] anyhow::Error),
//     #[error(transparent)]
//     Io(#[from] std::io::Error),
//     #[error(transparent)]
//     RequestErr(#[from] reqwest::Error),
//     #[error(transparent)]
//     TauriErr(#[from] tauri::Error),
//     #[error(transparent)]
//     SqlxErr(#[from] sqlx::Error),
//     #[error(transparent)]
//     TeraErr(#[from] tera::Error),
// }

// impl serde::Serialize for Error {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         serializer.serialize_str(self.to_string().as_ref())
//     }
// }

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
            base::hash,
            base::checksum,
            base::uuid,
            base::encode_base64_text,
            base::decode_base64_text,
            base::encode_url,
            base::decode_url,
            base::cffc,
            base::timestamp,
            base::number_base,
            base::qrcode,
            base::check_ip,
            base::ip_to_number,
            base::charset_encode,
            base::auto_detect_charset,
            base::recover_garbled_code,
            // openapi::fetch_api_data,
            // openapi::download,
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
            monitor::monitor_disk_info,
            monitor::monitor_memory_info,
            monitor::monitor_process_info,
            monitor::monitor_system_info,
            monitor::monitor_kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

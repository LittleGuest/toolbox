use thiserror::Error;

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
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        // .plugin(tauri_plugin_persisted_scope::init())
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
            openapi::download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};

use base::{Base, CharsetEncodeResult};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn hash(
    uppercase: bool,
    output_type: Option<&str>,
    hmac_mode: bool,
    input: Option<&str>,
) -> Result<impl Serialize> {
    base::hash(uppercase, output_type, hmac_mode, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn checksum(r#type: &str, file_path: &str) -> Result<String> {
    base::checksum(r#type, file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uuid(
    hyphens: Option<&str>,
    uppercase: bool,
    remove_connector: bool,
    version: u8,
    number: u16,
) -> Result<Vec<String>> {
    base::uuid(hyphens, uppercase, remove_connector, version, number).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn encode_base64_text(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::encode_base64_text(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decode_base64_text(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::decode_base64_text(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn encode_url(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::encode_url(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decode_url(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::decode_url(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cffc(indent: u8, ft: &str, tt: &str, input: Option<&str>) -> Result<String> {
    let Some(input) = input else {
        return Err("input empty".to_string());
    };
    base::cffc(indent, ft, tt, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn timestamp(time: Option<&str>) -> Result<HashMap<String, String>> {
    base::timestamp(time).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn number_base(input_type: Option<Base>, input: String) -> Result<HashMap<String, String>> {
    base::number_base(input_type, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn qrcode(input: Option<String>) -> Result<String> {
    base::qrcode(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_ip(t: &str, ip: Option<String>) -> Result<bool> {
    base::check_ip(t, ip).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ip_to_number(t: &str, ip: Option<String>) -> Result<HashMap<String, String>> {
    base::ip_to_number(t, ip).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn charset_encode(
    input: &str,
    input_type: &str,
    target_charset: &str,
    output_type: &str,
    delimiter: &str,
    base_format: &str,
    show_unicode: bool,
    show_escape: bool,
    show_c_array: bool,
    show_assembly: bool,
    show_auto: bool,
    invert_non_printable: bool,
    append_null: bool,
) -> Result<CharsetEncodeResult> {
    base::charset_encode(
        input,
        input_type,
        target_charset,
        output_type,
        delimiter,
        base_format,
        show_unicode,
        show_escape,
        show_c_array,
        show_assembly,
        show_auto,
        invert_non_printable,
        append_null,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn auto_detect_charset(input: &str) -> Result<String> {
    base::auto_detect_charset(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_garbled_code(input: &str) -> Result<Vec<base::RecoverGarbledCode>> {
    base::recover_garbled_code(input).map_err(|e| e.to_string())
}

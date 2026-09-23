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
    secret: Option<&str>,
    input: Option<&str>,
) -> Result<impl Serialize> {
    base::hash(uppercase, output_type, hmac_mode, secret, input)
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
pub fn encode_base64_image(file_path: &str) -> Result<HashMap<String, String>> {
    base::encode_base64_image(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decode_base64_image(input: Option<&str>) -> Result<HashMap<String, String>> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::decode_base64_image(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_base64_image(input: Option<&str>, file_path: &str) -> Result<()> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::save_base64_image(data, file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cron_parse(expression: &str, count: u32, cron_type: &str) -> Result<base::CronParseResult> {
    base::cron_parse(expression, count, cron_type).map_err(|e| e.to_string())
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
pub fn decode_jwt(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::decode_jwt(data).map_err(|e| e.to_string())
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
pub fn rmb_to_upper(
    input: Option<&str>,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
    jiao_zheng: bool,
) -> Result<String> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::rmb_to_upper(data, yuan, zheng, zheng_yuan, jiao_zheng).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rmb_to_amount(
    input: Option<&str>,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
) -> Result<base::RmbParseResult> {
    let Some(data) = input else {
        return Err("input empty".to_string());
    };
    base::rmb_to_amount(data, yuan, zheng, zheng_yuan).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_ip(t: &str, ip: Option<String>) -> Result<bool> {
    base::check_ip(t, ip).map_err(|e| e.to_string())
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

#[tauri::command]
pub fn timestamp_convert(
    mode: &str,
    unit: &str,
    tz_offset_secs: i32,
    values: Vec<String>,
) -> Result<Vec<base::TimestampRow>> {
    base::timestamp_convert(mode, unit, tz_offset_secs, &values).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn image_convert(
    inputs: Vec<String>,
    output_format: &str,
    output_dir: &str,
    quality: Option<u16>,
) -> Result<Vec<String>> {
    base::image_convert(&inputs, output_format, output_dir, quality).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn images_to_pdf(inputs: Vec<String>, output_path: &str) -> Result<()> {
    base::images_to_pdf(&inputs, output_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pdf_merge(inputs: Vec<String>, output_path: &str) -> Result<()> {
    base::pdf_merge(&inputs, output_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pdf_edit(
    input: &str,
    output_path: &str,
    delete: Vec<u32>,
    rotate: Vec<(u32, f32)>,
    order: Vec<u32>,
) -> Result<()> {
    base::pdf_edit(input, output_path, &delete, &rotate, &order).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pdf_split(input: &str, output_dir: &str, ranges: &str) -> Result<Vec<String>> {
    base::pdf_split(input, output_dir, ranges).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pdf_add_page_numbers(
    input: &str,
    output_path: &str,
    position: &str,
    font_size: f32,
    format_pattern: &str,
    start_at: i32,
) -> Result<()> {
    base::pdf_add_page_numbers(input, output_path, position, font_size, format_pattern, start_at)
        .map_err(|e| e.to_string())
}

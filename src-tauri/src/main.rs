// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use libs::cffc;
use serde::{Deserialize, Serialize};

pub mod libs;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            hash,
            uuid,
            encode_base64_text,
            decode_base64_text,
            encode_url,
            decode_url,
            cffc,
            timestamp,
            number_base,
            qrcode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn hash(
    uppercase: bool,
    output_type: Option<&str>,
    hmac_mode: bool,
    input: Option<&str>,
) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(5);

    if let Some(input) = input {
        let mut md5 = libs::hash::md5(input);
        let mut sha1 = libs::hash::sha1(input);
        let mut sha256 = libs::hash::sha256(input);
        let mut sha512 = libs::hash::sha512(input);
        let mut sha3_256 = libs::hash::sha3_256(input);
        let mut sha3_512 = libs::hash::sha3_512(input);

        if uppercase {
            md5 = md5.to_uppercase();
            sha1 = sha1.to_uppercase();
            sha256 = sha256.to_uppercase();
            sha512 = sha512.to_uppercase();
            sha3_256 = sha3_256.to_uppercase();
            sha3_512 = sha3_512.to_uppercase();
        }

        map.insert("md5".to_string(), md5);
        map.insert("sha1".to_string(), sha1);
        map.insert("sha256".to_string(), sha256);
        map.insert("sha512".to_string(), sha512);
        map.insert("sha3_256".to_string(), sha3_256);
        map.insert("sha3_512".to_string(), sha3_512);
    }
    map
}

#[tauri::command]
fn uuid(hyphens: Option<&str>, uppercase: bool, version: u8, number: u16) -> Vec<String> {
    let mut uuids = match version {
        1 => (0..number).map(|_| libs::uuid::uuid1()).collect::<Vec<_>>(),
        3 => (0..number)
            .map(|_| libs::uuid::uuid3("", "").unwrap())
            .collect::<Vec<_>>(),
        4 => (0..number).map(|_| libs::uuid::uuid4()).collect::<Vec<_>>(),
        5 => (0..number)
            .map(|_| libs::uuid::uuid5("", "").unwrap())
            .collect::<Vec<_>>(),
        _ => {
            vec![]
        }
    };

    if uppercase {
        uuids = uuids.iter().map(|u| u.to_uppercase()).collect::<Vec<_>>();
    }
    uuids
}

#[tauri::command]
fn encode_base64_text(input: Option<&str>) -> String {
    if let Some(data) = input {
        libs::base64::encode_text(data)
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn decode_base64_text(input: Option<&str>) -> String {
    if let Some(data) = input {
        libs::base64::decode_text(data).unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn encode_url(input: Option<&str>) -> String {
    if let Some(data) = input {
        libs::url::encode(data)
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn decode_url(input: Option<&str>) -> String {
    if let Some(data) = input {
        libs::url::decode(data).unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn cffc(ident: u8, ft: &str, tt: &str, input: Option<&str>) -> String {
    if let Some(input) = input {
        libs::cffc::Data::new(cffc::Ft::from(ft), cffc::Ft::from(tt), input, ident)
            .convert()
            .unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn timestamp(time: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(5);

    if let Some(time) = time {
        match time.parse::<i64>() {
            Ok(_time) => {
                todo!()
            }
            Err(_) => {}
        }
    } else {
        let now = libs::datetime::now();

        map.insert("current".to_string(), now.unix_timestamp().to_string());
        map.insert("timestamp".to_string(), now.unix_timestamp().to_string());
        map.insert(
            "timestamp_mill".to_string(),
            now.unix_timestamp().to_string(),
        );
        map.insert("utc".to_string(), "".to_string());
        map.insert(
            "datetime_utc8".to_string(),
            libs::datetime::unix_to_datetime(now.unix_timestamp()).unwrap_or("".to_string()),
        );
    }

    map
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hex,
}

#[tauri::command]
fn number_base(input_type: Option<Base>, input: String) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(4);
    if input.is_empty() {
        return map;
    }
    if let Some(input_type) = input_type {
        match input_type {
            Base::Binary => {
                let octal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE2,
                    base_converter::BASE8,
                )
                .unwrap();
                let decimal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE2,
                    base_converter::BASE10,
                )
                .unwrap();
                let hex = base_converter::base_to_base(
                    &input,
                    base_converter::BASE2,
                    base_converter::BASE16,
                )
                .unwrap();
                map.insert("binary".to_string(), input);
                map.insert("octal".to_string(), octal);
                map.insert("decimal".to_string(), decimal);
                map.insert("hex".to_string(), hex);
                map
            }
            Base::Octal => {
                let binary = base_converter::base_to_base(
                    &input,
                    base_converter::BASE8,
                    base_converter::BASE2,
                )
                .unwrap();
                let decimal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE8,
                    base_converter::BASE10,
                )
                .unwrap();
                let hex = base_converter::base_to_base(
                    &input,
                    base_converter::BASE8,
                    base_converter::BASE16,
                )
                .unwrap();
                map.insert("binary".to_string(), binary);
                map.insert("octal".to_string(), input);
                map.insert("decimal".to_string(), decimal);
                map.insert("hex".to_string(), hex);
                map
            }
            Base::Decimal => {
                let binary = base_converter::base_to_base(
                    &input,
                    base_converter::BASE10,
                    base_converter::BASE2,
                )
                .unwrap();
                let octal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE10,
                    base_converter::BASE8,
                )
                .unwrap();
                let hex = base_converter::base_to_base(
                    &input,
                    base_converter::BASE10,
                    base_converter::BASE16,
                )
                .unwrap();
                map.insert("binary".to_string(), binary);
                map.insert("octal".to_string(), octal);
                map.insert("decimal".to_string(), input);
                map.insert("hex".to_string(), hex);
                map
            }
            Base::Hex => {
                let binary = base_converter::base_to_base(
                    &input,
                    base_converter::BASE16,
                    base_converter::BASE2,
                )
                .unwrap();
                let octal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE16,
                    base_converter::BASE8,
                )
                .unwrap();
                let decimal = base_converter::base_to_base(
                    &input,
                    base_converter::BASE16,
                    base_converter::BASE10,
                )
                .unwrap();
                map.insert("binary".to_string(), binary);
                map.insert("octal".to_string(), octal);
                map.insert("decimal".to_string(), decimal);
                map.insert("hex".to_string(), input);
                map
            }
        }
    } else {
        map
    }
}

#[tauri::command]
fn qrcode(input: Option<String>) -> String {
    if let Some(input) = input {
        libs::qrcode::qrcode(&input)
    } else {
        "".to_string()
    }
}

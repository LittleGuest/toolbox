// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

pub mod libs;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            hash,
            uuid,
            encode_base64_text,
            decode_base64_text
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

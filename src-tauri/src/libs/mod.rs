#![allow(unused)]
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

mod base64;
mod cffc;
mod datetime;
mod hash;
mod hex;
mod ip;
mod jwt;
mod qrcode;
mod string;
mod url;
mod url_params;
mod uuid;

#[tauri::command]
pub fn hash(
    uppercase: bool,
    output_type: Option<&str>,
    hmac_mode: bool,
    input: Option<&str>,
) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(5);

    if let Some(input) = input {
        let mut md5 = hash::md5(input);
        let mut sha1 = hash::sha1(input);
        let mut sha256 = hash::sha256(input);
        let mut sha512 = hash::sha512(input);
        let mut sha3_256 = hash::sha3_256(input);
        let mut sha3_512 = hash::sha3_512(input);

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
pub fn uuid(hyphens: Option<&str>, uppercase: bool, version: u8, number: u16) -> Vec<String> {
    let mut uuids = match version {
        1 => (0..number).map(|_| uuid::uuid1()).collect::<Vec<_>>(),
        3 => (0..number)
            .map(|_| uuid::uuid3("", "").unwrap())
            .collect::<Vec<_>>(),
        4 => (0..number).map(|_| uuid::uuid4()).collect::<Vec<_>>(),
        5 => (0..number)
            .map(|_| uuid::uuid5("", "").unwrap())
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
pub fn encode_base64_text(input: Option<&str>) -> String {
    if let Some(data) = input {
        base64::encode_text(data)
    } else {
        "".to_string()
    }
}

#[tauri::command]
pub fn decode_base64_text(input: Option<&str>) -> String {
    if let Some(data) = input {
        base64::decode_text(data).unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
pub fn encode_url(input: Option<&str>) -> String {
    if let Some(data) = input {
        url::encode(data)
    } else {
        "".to_string()
    }
}

#[tauri::command]
pub fn decode_url(input: Option<&str>) -> String {
    if let Some(data) = input {
        url::decode(data).unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
pub fn cffc(ident: u8, ft: &str, tt: &str, input: Option<&str>) -> String {
    if let Some(input) = input {
        cffc::Data::new(cffc::Ft::from(ft), cffc::Ft::from(tt), input, ident)
            .convert()
            .unwrap()
    } else {
        "".to_string()
    }
}

#[tauri::command]
pub fn timestamp(time: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(5);

    if let Some(time) = time {
        match time.parse::<i64>() {
            Ok(_time) => {
                todo!()
            }
            Err(_) => {}
        }
    } else {
        let now = datetime::now();

        map.insert("current".to_string(), now.unix_timestamp().to_string());
        map.insert("timestamp".to_string(), now.unix_timestamp().to_string());
        map.insert(
            "timestamp_mill".to_string(),
            now.unix_timestamp().to_string(),
        );
        map.insert("utc".to_string(), "".to_string());
        map.insert(
            "datetime_utc8".to_string(),
            datetime::unix_to_datetime(now.unix_timestamp()).unwrap_or("".to_string()),
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
pub fn number_base(input_type: Option<Base>, input: String) -> HashMap<String, String> {
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

/// 二维码
#[tauri::command]
pub fn qrcode(input: Option<String>) -> String {
    if let Some(input) = input {
        qrcode::qrcode(&input)
    } else {
        "".to_string()
    }
}

/// 检查IP地址
#[tauri::command]
pub fn check_ip(t: &str, ip: Option<String>) -> bool {
    if let Some(ip) = ip {
        match t {
            "v4" => Ipv4Addr::from_str(&ip).is_ok(),
            "v6" => Ipv6Addr::from_str(&ip).is_ok(),
            _ => false,
        }
    } else {
        false
    }
}

#[tauri::command]
pub fn ip_to_number(t: &str, ip: Option<String>) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(4);
    if let Some(ip) = ip {
        match t {
            "v4" => {
                if Ipv4Addr::from_str(&ip).is_ok() {
                    let decimal = ip::ipv4_to_num(&ip).unwrap_or(0).to_string();
                    let bn = number_base(Some(Base::Decimal), decimal.to_string());
                    map.insert(
                        "binary".to_string(),
                        bn.get("binary").unwrap_or(&"".to_string()).to_owned(),
                    );
                    map.insert(
                        "octal".to_string(),
                        bn.get("octal").unwrap_or(&"".to_string()).to_owned(),
                    );
                    map.insert("decimal".to_string(), decimal);
                    map.insert(
                        "hex".to_string(),
                        bn.get("hex").unwrap_or(&"".to_string()).to_owned(),
                    );
                }
            }
            "v6" => {
                if Ipv6Addr::from_str(&ip).is_ok() {
                    map.insert(
                        "binary".to_string(),
                        ip::ipv6_to_num(&ip).unwrap_or(0).to_string(),
                    );
                    map.insert(
                        "octal".to_string(),
                        ip::ipv6_to_num(&ip).unwrap_or(0).to_string(),
                    );
                    map.insert(
                        "decimal".to_string(),
                        ip::ipv6_to_num(&ip).unwrap_or(0).to_string(),
                    );
                    map.insert(
                        "hex".to_string(),
                        ip::ipv6_to_num(&ip).unwrap_or(0).to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    map
}

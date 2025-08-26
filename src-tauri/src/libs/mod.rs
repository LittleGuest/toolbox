#![allow(unused)]
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, libs::checksum::Checksum};

mod base64;
mod cffc;
mod checksum;
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
pub async fn hash(
    uppercase: bool,
    output_type: Option<&str>,
    hmac_mode: bool,
    input: Option<&str>,
) -> Result<HashMap<String, String>> {
    let mut map = HashMap::with_capacity(6);

    if let Some(input) = input {
        let (md5, sha1, sha256, sha512, sha3_256, sha3_512) = tokio::join!(
            hash::md5(input),
            hash::sha1(input),
            hash::sha256(input),
            hash::sha512(input),
            hash::sha3_256(input),
            hash::sha3_512(input),
        );
        let (mut md5, mut sha1, mut sha256, mut sha512, mut sha3_256, mut sha3_512) =
            (md5?, sha1?, sha256?, sha512?, sha3_256?, sha3_512?);

        if uppercase {
            md5 = md5.to_uppercase();
            sha1 = sha1.to_uppercase();
            sha256 = sha256.to_uppercase();
            sha512 = sha512.to_uppercase();
            sha3_256 = sha3_256.to_uppercase();
            sha3_512 = sha3_512.to_uppercase();
        }

        map.insert("md5".into(), md5);
        map.insert("sha1".into(), sha1);
        map.insert("sha256".into(), sha256);
        map.insert("sha512".into(), sha512);
        map.insert("sha3_256".into(), sha3_256);
        map.insert("sha3_512".into(), sha3_512);
    }
    Ok(map)
}

#[tauri::command]
pub async fn checksum(r#type: &str, file_path: &str) -> Result<String> {
    Checksum::sum(r#type, file_path).await
}

#[tauri::command]
pub fn uuid(
    hyphens: Option<&str>,
    uppercase: bool,
    version: u8,
    number: u16,
) -> Result<Vec<String>> {
    let mut uuids = match version {
        1 => (0..number)
            .map(|_| uuid::uuid_v1())
            .collect::<Result<Vec<_>>>(),
        3 => (0..number)
            .map(|_| uuid::uuid_v3("", ""))
            .collect::<Result<Vec<_>>>(),
        4 => (0..number)
            .map(|_| uuid::uuid_v4())
            .collect::<Result<Vec<_>>>(),
        5 => (0..number)
            .map(|_| uuid::uuid_v5("", ""))
            .collect::<Result<Vec<_>>>(),
        6 => (0..number)
            .map(|_| uuid::uuid_v6())
            .collect::<Result<Vec<_>>>(),
        7 => (0..number)
            .map(|_| uuid::uuid_v7())
            .collect::<Result<Vec<_>>>(),
        8 => (0..number)
            .map(|_| uuid::uuid_v8())
            .collect::<Result<Vec<_>>>(),
        _ => return Err(Error::E("unsupport version".into())),
    }?;

    if uppercase {
        uuids = uuids.iter().map(|u| u.to_uppercase()).collect::<Vec<_>>();
    }
    Ok(uuids)
}

#[tauri::command]
pub fn encode_base64_text(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err(Error::E("input empty".into()));
    };
    base64::encode_text(data)
}

#[tauri::command]
pub fn decode_base64_text(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err(Error::E("input empty".into()));
    };
    base64::decode_text(data)
}

#[tauri::command]
pub fn encode_url(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err(Error::E("input empty".into()));
    };
    url::encode(data)
}

#[tauri::command]
pub fn decode_url(input: Option<&str>) -> Result<String> {
    let Some(data) = input else {
        return Err(Error::E("input empty".into()));
    };
    url::decode(data)
}

#[tauri::command]
pub fn cffc(indent: u8, ft: &str, tt: &str, input: Option<&str>) -> Result<String> {
    let Some(input) = input else {
        return Err(Error::E("input empty".into()));
    };
    cffc::Data::new(cffc::Ft::from(ft), cffc::Ft::from(tt), input, indent).transform()
}

#[tauri::command]
pub fn timestamp(time: Option<&str>) -> Result<HashMap<String, String>> {
    let mut map = HashMap::with_capacity(5);
    let Some(time) = time else {
        return Ok(map);
    };

    if let Ok(time) = time.parse::<i64>() {
        map.insert("format".to_string(), datetime::timestamp_to_str(time)?);
    } else {
        map.insert(
            "format".to_string(),
            datetime::str_to_timestamp(time)?.to_string(),
        );
    }
    Ok(map)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hex,
}

#[tauri::command]
pub fn number_base(input_type: Option<Base>, input: String) -> Result<HashMap<String, String>> {
    let mut map = HashMap::with_capacity(4);
    if input.is_empty() {
        return Ok(map);
    }
    let Some(input_type) = input_type else {
        return Ok(map);
    };
    match input_type {
        Base::Binary => {
            let octal =
                base_converter::base_to_base(&input, base_converter::BASE2, base_converter::BASE8)?;
            let decimal = base_converter::base_to_base(
                &input,
                base_converter::BASE2,
                base_converter::BASE10,
            )?;
            let hex = base_converter::base_to_base(
                &input,
                base_converter::BASE2,
                base_converter::BASE16,
            )?;
            map.insert("binary".to_string(), input);
            map.insert("octal".to_string(), octal);
            map.insert("decimal".to_string(), decimal);
            map.insert("hex".to_string(), hex);
        }
        Base::Octal => {
            let binary =
                base_converter::base_to_base(&input, base_converter::BASE8, base_converter::BASE2)?;
            let decimal = base_converter::base_to_base(
                &input,
                base_converter::BASE8,
                base_converter::BASE10,
            )?;
            let hex = base_converter::base_to_base(
                &input,
                base_converter::BASE8,
                base_converter::BASE16,
            )?;
            map.insert("binary".to_string(), binary);
            map.insert("octal".to_string(), input);
            map.insert("decimal".to_string(), decimal);
            map.insert("hex".to_string(), hex);
        }
        Base::Decimal => {
            let binary = base_converter::base_to_base(
                &input,
                base_converter::BASE10,
                base_converter::BASE2,
            )?;
            let octal = base_converter::base_to_base(
                &input,
                base_converter::BASE10,
                base_converter::BASE8,
            )?;
            let hex = base_converter::base_to_base(
                &input,
                base_converter::BASE10,
                base_converter::BASE16,
            )?;
            map.insert("binary".to_string(), binary);
            map.insert("octal".to_string(), octal);
            map.insert("decimal".to_string(), input);
            map.insert("hex".to_string(), hex);
        }
        Base::Hex => {
            let binary = base_converter::base_to_base(
                &input,
                base_converter::BASE16,
                base_converter::BASE2,
            )?;
            let octal = base_converter::base_to_base(
                &input,
                base_converter::BASE16,
                base_converter::BASE8,
            )?;
            let decimal = base_converter::base_to_base(
                &input,
                base_converter::BASE16,
                base_converter::BASE10,
            )?;
            map.insert("binary".to_string(), binary);
            map.insert("octal".to_string(), octal);
            map.insert("decimal".to_string(), decimal);
            map.insert("hex".to_string(), input);
        }
    }
    Ok(map)
}

#[tauri::command]
pub fn qrcode(input: Option<String>) -> Result<String> {
    let Some(input) = input else {
        return Err(Error::E("input empty".into()));
    };
    qrcode::qrcode(&input)
}

#[tauri::command]
pub fn check_ip(t: &str, ip: Option<String>) -> Result<bool> {
    let Some(ip) = ip else {
        return Ok(false);
    };
    let check = match t {
        "v4" => Ipv4Addr::from_str(&ip).is_ok(),
        "v6" => Ipv6Addr::from_str(&ip).is_ok(),
        _ => false,
    };
    Ok(check)
}

#[tauri::command]
pub fn ip_to_number(t: &str, ip: Option<String>) -> Result<HashMap<String, String>> {
    let mut map = HashMap::with_capacity(4);
    let Some(ip) = ip else { return Ok(map) };
    match t {
        "v4" => {
            if Ipv4Addr::from_str(&ip).is_ok() {
                let decimal = ip::ipv4_to_num(&ip)?.to_string();
                let bn = number_base(Some(Base::Decimal), decimal.to_string())?;
                map.insert(
                    "binary".to_string(),
                    bn.get("binary").unwrap_or(&String::new()).to_owned(),
                );
                map.insert(
                    "octal".to_string(),
                    bn.get("octal").unwrap_or(&String::new()).to_owned(),
                );
                map.insert("decimal".to_string(), decimal);
                map.insert(
                    "hex".to_string(),
                    bn.get("hex").unwrap_or(&String::new()).to_owned(),
                );
            }
        }
        "v6" => {
            if Ipv6Addr::from_str(&ip).is_ok() {
                map.insert("binary".to_string(), ip::ipv6_to_num(&ip)?.to_string());
                map.insert("octal".to_string(), ip::ipv6_to_num(&ip)?.to_string());
                map.insert("decimal".to_string(), ip::ipv6_to_num(&ip)?.to_string());
                map.insert("hex".to_string(), ip::ipv6_to_num(&ip)?.to_string());
            }
        }
        _ => {}
    }

    Ok(map)
}

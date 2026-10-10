#![allow(unused)]
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use anyhow::{Error, Result};
pub use charset::RecoverGarbledCode;
use serde::{Deserialize, Serialize};

use crate::checksum::Checksum;

mod base64;
pub mod blank_line;
mod cffc;
mod charset;
mod checksum;
mod cron;
mod datetime;
mod hash;
mod hex;
mod image_convert;
mod ip;
mod jwt;
mod pdf;
mod qrcode;
mod rmb;
mod string;
mod url;
mod url_params;
pub mod uuid;

pub use cron::CronParseResult;
pub use rmb::RmbParseResult;

pub async fn hash(
    uppercase: bool,
    output_type: Option<&str>,
    hmac_mode: bool,
    secret: Option<&str>,
    input: Option<&str>,
) -> Result<HashMap<String, String>> {
    let mut map = HashMap::with_capacity(6);

    if let Some(input) = input {
        let (md5, sha1, sha256, sha512, sha3_256, sha3_512) = if hmac_mode {
            let secret = secret.unwrap_or("");
            let (md5, sha1, sha256, sha512, sha3_256, sha3_512) = tokio::join!(
                hash::hmac_md5(secret, input),
                hash::hmac_sha1(secret, input),
                hash::hmac_sha256(secret, input),
                hash::hmac_sha512(secret, input),
                hash::hmac_sha3_256(secret, input),
                hash::hmac_sha3_512(secret, input),
            );
            (md5?, sha1?, sha256?, sha512?, sha3_256?, sha3_512?)
        } else {
            let (md5, sha1, sha256, sha512, sha3_256, sha3_512) = tokio::join!(
                hash::md5(input),
                hash::sha1(input),
                hash::sha256(input),
                hash::sha512(input),
                hash::sha3_256(input),
                hash::sha3_512(input),
            );
            (md5?, sha1?, sha256?, sha512?, sha3_256?, sha3_512?)
        };

        let mut values = [
            ("md5", md5),
            ("sha1", sha1),
            ("sha256", sha256),
            ("sha512", sha512),
            ("sha3_256", sha3_256),
            ("sha3_512", sha3_512),
        ];

        if output_type == Some("base64") {
            use ::base64::Engine as _;
            for (_, val) in values.iter_mut() {
                if let Ok(bytes) = hex::decode(val.as_str()) {
                    *val = ::base64::engine::general_purpose::STANDARD.encode(&bytes);
                }
            }
        }

        if uppercase {
            for (_, val) in values.iter_mut() {
                *val = val.to_uppercase();
            }
        }

        for (key, val) in values {
            map.insert(key.into(), val);
        }
    }
    Ok(map)
}

pub async fn checksum(r#type: &str, file_path: &str) -> Result<String> {
    Checksum::sum(r#type, file_path).await
}

pub fn uuid(
    hyphens: Option<&str>,
    uppercase: bool,
    remove_connector: bool,
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
        _ => return Err(Error::msg("unsupport version".to_string())),
    }?;

    if uppercase {
        uuids = uuids.iter().map(|u| u.to_uppercase()).collect::<Vec<_>>();
    }
    if remove_connector {
        uuids = uuids.iter().map(|u| u.replace("-", "")).collect::<Vec<_>>();
    }
    Ok(uuids)
}

pub fn encode_base64_text(data: &str) -> Result<String> {
    base64::encode_text(data)
}

pub fn decode_base64_text(data: &str) -> Result<String> {
    base64::decode_text(data)
}

pub fn encode_base64_image(file_path: &str) -> Result<HashMap<String, String>> {
    base64::encode_image(file_path)
}

pub fn decode_base64_image(data: &str) -> Result<HashMap<String, String>> {
    base64::decode_image(data)
}

pub fn save_base64_image(data: &str, file_path: &str) -> Result<()> {
    base64::save_image(data, file_path)
}

pub fn cron_parse(expression: &str, count: u32, cron_type: &str) -> Result<CronParseResult> {
    cron::parse_expression(expression, count, cron_type)
}

pub fn encode_url(data: &str) -> Result<String> {
    url::encode(data)
}

pub fn decode_url(data: &str) -> Result<String> {
    url::decode(data)
}

pub fn decode_jwt(data: &str) -> Result<String> {
    jwt::decode(data)
}

pub fn rmb_to_upper(
    input: &str,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
    jiao_zheng: bool,
) -> Result<String> {
    rmb::amount_to_upper(input, yuan, zheng, zheng_yuan, jiao_zheng)
}

pub fn rmb_to_amount(
    input: &str,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
) -> Result<RmbParseResult> {
    rmb::upper_to_amount(input, yuan, zheng, zheng_yuan)
}

pub fn cffc(indent: u8, ft: &str, tt: &str, input: &str) -> Result<String> {
    cffc::Data::new(cffc::Ft::from(ft), cffc::Ft::from(tt), input, indent).transform()
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hex,
}

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

pub fn qrcode(input: Option<String>) -> Result<String> {
    let Some(input) = input else {
        return Err(Error::msg("input empty".to_string()));
    };
    qrcode::qrcode(&input)
}

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
        "v6" if Ipv6Addr::from_str(&ip).is_ok() => {
            let decimal = ip::ipv6_to_num(&ip)?.to_string();
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
        _ => {}
    }

    Ok(map)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharsetEncodeResult {
    pub output: String,
    pub byte_count: usize,
    pub char_count: usize,
}

#[allow(clippy::too_many_arguments)]
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
    let mut bytes = charset::parse_bytes_from_string(input, input_type)?;

    if input_type == "text" {
        let decoded = charset::decode_bytes(&bytes, "UTF-8")?;
        bytes = charset::encode_string(&decoded, target_charset)?;
    }

    if invert_non_printable {
        bytes = charset::invert_non_printable(&bytes);
    }

    if append_null {
        bytes.push(0);
    }

    let mut output = if show_c_array {
        charset::format_as_c_array(&bytes)
    } else if show_assembly {
        charset::format_as_assembly(&bytes)
    } else {
        charset::format_bytes_to_string(&bytes, output_type, delimiter, base_format)?
    };

    let byte_count = bytes.len();
    let char_count = match charset::decode_bytes(&bytes, target_charset) {
        Ok(s) => s.chars().count(),
        Err(_) => 0,
    };

    Ok(CharsetEncodeResult {
        output,
        byte_count,
        char_count,
    })
}

pub fn auto_detect_charset(input: &str) -> Result<String> {
    let charset = charset::auto_detect_charset(input)?;
    Ok(charset)
}

pub fn recover_garbled_code(input: &str) -> Result<Vec<RecoverGarbledCode>> {
    Ok(charset::recover_garbled_code(input))
}

pub use datetime::TimestampRow;

pub fn timestamp_convert(
    mode: &str,
    unit: &str,
    tz_offset_secs: i32,
    values: &[String],
) -> Result<Vec<datetime::TimestampRow>> {
    datetime::timestamp_convert(mode, unit, tz_offset_secs, values)
}

pub fn image_convert(
    inputs: &[String],
    output_format: &str,
    output_dir: &str,
    quality: Option<u16>,
) -> Result<Vec<String>> {
    image_convert::image_convert(inputs, output_format, output_dir, quality)
}

pub fn images_to_pdf(inputs: &[String], output_path: &str) -> Result<()> {
    pdf::images_to_pdf(inputs, output_path)
}

pub fn pdf_merge(inputs: &[String], output_path: &str) -> Result<()> {
    pdf::pdf_merge(inputs, output_path)
}

pub fn pdf_edit(
    input: &str,
    output_path: &str,
    delete: &[u32],
    rotate: &[(u32, f32)],
    order: &[u32],
) -> Result<()> {
    pdf::pdf_edit(input, output_path, delete, rotate, order)
}

pub fn pdf_split(input: &str, output_dir: &str, ranges: &str) -> Result<Vec<String>> {
    pdf::pdf_split(input, output_dir, ranges)
}

pub fn pdf_add_page_numbers(
    input: &str,
    output_path: &str,
    position: &str,
    font_size: f32,
    format_pattern: &str,
    start_at: i32,
) -> Result<()> {
    pdf::pdf_add_page_numbers(
        input,
        output_path,
        position,
        font_size,
        format_pattern,
        start_at,
    )
}

mod sm_crypto;
pub use sm_crypto::*;

mod charset_transcode;
mod json_schema;
mod qrcode_decode;
pub use charset_transcode::*;
pub use json_schema::*;
pub use qrcode_decode::*;

#[cfg(test)]
mod new_tools_smoke {
    #[test]
    fn sm3_known_vector() {
        let got = crate::sm3_hex(b"abc");
        assert_eq!(
            got,
            "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
        );
    }

    #[test]
    fn sm4_roundtrip() {
        let key = "0123456789abcdeffedcba9876543210";
        let iv = "00000000000000000000000000000000";
        let msg = "国密 SM4 测试";
        let ct = crate::sm4_encrypt(msg.as_bytes(), key, "CBC", iv, true).unwrap();
        let pt = crate::sm4_decrypt(&ct, key, "CBC", iv, true).unwrap();
        assert_eq!(String::from_utf8(pt).unwrap(), msg);
    }

    #[test]
    fn sm2_sign_verify_and_pke() {
        let pair = crate::sm2_generate_keypair().unwrap();
        let sig = crate::sm2_sign(&pair.private_hex, crate::DEFAULT_DISTID, b"hello sm2").unwrap();
        assert!(
            crate::sm2_verify(&pair.public_hex, crate::DEFAULT_DISTID, b"hello sm2", &sig).unwrap()
        );
        assert!(
            !crate::sm2_verify(&pair.public_hex, crate::DEFAULT_DISTID, b"other", &sig).unwrap()
        );
        let ct = crate::sm2_encrypt(&pair.public_hex, b"secret payload").unwrap();
        let pt = crate::sm2_decrypt(&pair.private_hex, &ct).unwrap();
        assert_eq!(pt, b"secret payload");
    }

    #[test]
    fn schema_reports_expected_errors() {
        let schema = crate::schema_template();
        let bad = r#"{"id":0,"email":"nope","tags":["a","a"]}"#;
        let report = crate::validate(schema, bad, "自动").unwrap();
        assert!(!report.valid);
        assert!(
            report.total_errors >= 3,
            "issues: {:?}",
            report.issues.len()
        );
        let good = crate::instance_template();
        let ok = crate::validate(schema, good, "自动").unwrap();
        assert!(ok.valid, "{:?}", ok.issues);
    }

    #[test]
    fn charset_transcode_gbk_roundtrip() {
        let utf8 = "中文编码转换测试";
        let to_gbk = crate::transcode(utf8, "UTF-8", "GBK").unwrap();
        assert!(!to_gbk.lossy);
        assert_eq!(to_gbk.byte_count, 16);
        assert_eq!(to_gbk.text, utf8);
        let gbk_bytes = crate::parse_hex(&to_gbk.hex).unwrap();
        let back = crate::transcode_from_bytes(&gbk_bytes, "GBK", "UTF-8").unwrap();
        assert_eq!(back.text, utf8);
        let latin = crate::transcode("café", "UTF-8", "ISO-8859-1").unwrap();
        assert_eq!(latin.byte_count, 4);
    }

    #[test]
    fn qrcode_roundtrip_through_matrix() {
        use qrcode_generator::qr::{Encoder, ErrorCorrection};
        let symbol = Encoder::new(ErrorCorrection::Medium)
            .encode_text("https://workbuddy.cn")
            .unwrap();
        let matrix = symbol.to_matrix();
        let out = crate::decode_matrix(&matrix).unwrap();
        assert_eq!(out[0].content, "https://workbuddy.cn");
    }
}

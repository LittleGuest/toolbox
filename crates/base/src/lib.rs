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

        // 输出编码：base64 时把十六进制解码为字节再 base64 编码
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

/// 金额数字 → 中文大写金额
pub fn rmb_to_upper(
    input: &str,
    yuan: Option<&str>,
    zheng: Option<&str>,
    jiao_zheng: bool,
) -> Result<String> {
    rmb::amount_to_upper(input, yuan, zheng, jiao_zheng)
}

/// 中文大写金额 → 金额数字
pub fn rmb_to_amount(input: &str, yuan: Option<&str>, zheng: Option<&str>) -> Result<RmbParseResult> {
    rmb::upper_to_amount(input, yuan, zheng)
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

#[derive(Debug, Serialize, Deserialize)]
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
        "v6" => {
            if Ipv6Addr::from_str(&ip).is_ok() {
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
    // 解析输入
    let mut bytes = charset::parse_bytes_from_string(input, input_type)?;

    // 如果是文本输入，先转换为目标字符集
    if input_type == "text" {
        let decoded = charset::decode_bytes(&bytes, "UTF-8")?;
        bytes = charset::encode_string(&decoded, target_charset)?;
    }

    // 处理不可打印字符
    if invert_non_printable {
        bytes = charset::invert_non_printable(&bytes);
    }

    // 追加NUL结尾
    if append_null {
        bytes.push(0);
    }

    // 格式化输出
    let mut output = if show_c_array {
        charset::format_as_c_array(&bytes)
    } else if show_assembly {
        charset::format_as_assembly(&bytes)
    } else {
        charset::format_bytes_to_string(&bytes, output_type, delimiter, base_format)?
    };

    // 计算统计信息
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

/// 时间戳 <-> 时间 双向批量转换（支持秒/毫秒、时区切换）。
pub fn timestamp_convert(
    mode: &str,
    unit: &str,
    tz_offset_secs: i32,
    values: &[String],
) -> Result<Vec<datetime::TimestampRow>> {
    datetime::timestamp_convert(mode, unit, tz_offset_secs, values)
}

/// 批量图片格式转换。
pub fn image_convert(
    inputs: &[String],
    output_format: &str,
    output_dir: &str,
    quality: Option<u16>,
) -> Result<Vec<String>> {
    image_convert::image_convert(inputs, output_format, output_dir, quality)
}

/// 将多张图片合并为一个 PDF（每图一页）。
pub fn images_to_pdf(inputs: &[String], output_path: &str) -> Result<()> {
    pdf::images_to_pdf(inputs, output_path)
}

/// 将多个 PDF 合并成一个。
pub fn pdf_merge(inputs: &[String], output_path: &str) -> Result<()> {
    pdf::pdf_merge(inputs, output_path)
}

/// PDF 页面删除 / 旋转 / 顺序调整。
pub fn pdf_edit(
    input: &str,
    output_path: &str,
    delete: &[u32],
    rotate: &[(u32, f32)],
    order: &[u32],
) -> Result<()> {
    pdf::pdf_edit(input, output_path, delete, rotate, order)
}

/// 拆分 PDF（按页码范围切分）。
pub fn pdf_split(input: &str, output_dir: &str, ranges: &str) -> Result<Vec<String>> {
    pdf::pdf_split(input, output_dir, ranges)
}

/// 为 PDF 添加页码。
pub fn pdf_add_page_numbers(
    input: &str,
    output_path: &str,
    position: &str,
    font_size: f32,
    format_pattern: &str,
    start_at: i32,
) -> Result<()> {
    pdf::pdf_add_page_numbers(input, output_path, position, font_size, format_pattern, start_at)
}

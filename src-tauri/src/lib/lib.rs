pub mod base64;
pub mod baseconverter;
pub mod cffc;
pub mod datetime;
pub mod hash;
pub mod hex;
pub mod ip;
pub mod jwt;
pub mod qrcode;
pub mod string;
pub mod url;
pub mod url_params;
pub mod uuid;

use thiserror::Error;

/// 全局异常枚举
#[derive(Debug, Error)]
pub enum ToolError {
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
    #[error("未知错误")]
    Unknown,
}

pub type ToolResult<T, E = ToolError> = std::result::Result<T, E>;

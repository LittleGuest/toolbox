use anyhow::{Error, Result};
use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

/// 时间字符串转时间戳（毫秒）
pub fn str_to_timestamp(t: &str) -> Result<i64> {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| Error::msg(e.to_string()))?;
    let offset = OffsetDateTime::parse(t, &format).map_err(|e| Error::msg(e.to_string()))?;
    Ok(offset.unix_timestamp())
}

/// 时间戳（毫秒）转时间字符串
pub fn timestamp_to_str(t: i64) -> Result<String> {
    let datetime = OffsetDateTime::from_unix_timestamp(t).map_err(|e| Error::msg(e.to_string()))?;
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| Error::msg(e.to_string()))?;
    let s = datetime
        .format(&format)
        .map_err(|e| Error::msg(e.to_string()))?;
    Ok(s)
}

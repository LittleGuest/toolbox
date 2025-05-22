//! 时间转换

use time::OffsetDateTime;

use crate::{Error, Result};

/// 当前时间
pub fn now() -> OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

/// 时间戳转datetime
pub fn unix_to_datetime(t: i64) -> Result<String> {
    let offset =
        OffsetDateTime::from_unix_timestamp(t).map_err(|e| Error::DateTimeErr(e.to_string()))?;

    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| Error::DateTimeErr(e.to_string()))?;

    offset
        .format(&format)
        .map_err(|e| Error::DateTimeErr(e.to_string()))
}

/// datetime转时间戳
pub fn datetime_to_unix(t: &str) -> Result<i64> {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| Error::DateTimeErr(e.to_string()))?;
    let offset =
        OffsetDateTime::parse(t, &format).map_err(|e| Error::DateTimeErr(e.to_string()))?;
    Ok(offset.unix_timestamp())
}

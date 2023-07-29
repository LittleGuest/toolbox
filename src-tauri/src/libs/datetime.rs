//! 时间转换

use time::OffsetDateTime;

use super::{ToolError, ToolResult};

/// 当前时间
pub fn now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

/// 时间戳转datetime
pub fn unix_to_datetime(t: i64) -> ToolResult<String> {
    let offset = OffsetDateTime::from_unix_timestamp(t)
        .map_err(|e| ToolError::DateTimeErr(e.to_string()))?;

    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| ToolError::DateTimeErr(e.to_string()))?;

    offset
        .format(&format)
        .map_err(|e| ToolError::DateTimeErr(e.to_string()))
}

/// datetime转时间戳
pub fn datetime_to_unix(t: &str) -> ToolResult<i64> {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .map_err(|e| ToolError::DateTimeErr(e.to_string()))?;
    let offset =
        OffsetDateTime::parse(t, &format).map_err(|e| ToolError::DateTimeErr(e.to_string()))?;
    Ok(offset.unix_timestamp())
}

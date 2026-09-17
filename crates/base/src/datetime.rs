use anyhow::{Error, Result};
use serde::Serialize;
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub fn now() -> OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

/// 单条时间戳转换结果
#[derive(Serialize, Clone, Debug)]
pub struct TimestampRow {
    /// 原始输入
    pub input: String,
    /// 秒级时间戳（若可解析）
    pub second_ts: Option<i64>,
    /// 毫秒级时间戳（若可解析）
    pub milli_ts: Option<i64>,
    /// 指定时区下的格式化时间
    pub datetime: String,
}

/// 时间戳 <-> 时间字符串 双向批量转换。
///
/// - mode: `"ts_to_dt"`（时间戳->时间）或 `"dt_to_ts"`（时间->时间戳）
/// - unit: `"s"`（秒）或 `"ms"`（毫秒），仅 ts_to_dt 使用
/// - tz_offset_secs: 目标时区相对 UTC 的秒偏移（如 UTC+8 = 28800）
/// - values: 多个待转换输入
pub fn timestamp_convert(
    mode: &str,
    unit: &str,
    tz_offset_secs: i32,
    values: &[String],
) -> Result<Vec<TimestampRow>> {
    let offset = UtcOffset::from_whole_seconds(tz_offset_secs)?;
    let mut rows = Vec::with_capacity(values.len());
    let fmt = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
    );

    for value in values {
        let v = value.trim();
        let row = match mode {
            "ts_to_dt" => {
                let ts: i64 = v.parse().map_err(|_| Error::msg(format!("无效的时间戳: {v}")))?;
                let secs = if unit == "ms" {
                    ts.div_euclid(1000)
                } else {
                    ts
                };
                let millis = if unit == "ms" { ts } else { ts.saturating_mul(1000) };
                let dt = OffsetDateTime::from_unix_timestamp(secs)
                    .map_err(|e| Error::msg(e.to_string()))?
                    .to_offset(offset);
                TimestampRow {
                    input: value.to_string(),
                    second_ts: Some(secs),
                    milli_ts: Some(millis),
                    datetime: dt
                        .format(&fmt)
                        .map_err(|e| Error::msg(e.to_string()))?,
                }
            }
            _ => {
                let sec_frac = parse_datetime_in_offset(v, offset)?;
                let secs = sec_frac.0;
                let subsec_ns = sec_frac.1;
                TimestampRow {
                    input: value.to_string(),
                    second_ts: Some(secs),
                    milli_ts: Some(secs * 1000 + i64::try_from(subsec_ns / 1_000_000)?),
                    datetime: value.to_string(),
                }
            }
        };
        rows.push(row);
    }
    Ok(rows)
}

/// 解析时间字符串（按指定时区），返回 (unix秒, 亚秒纳秒)。
fn parse_datetime_in_offset(s: &str, offset: UtcOffset) -> Result<(i64, u32)> {
    let f_sec = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let f_frac = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [subsecond digits:3]"
    );
    let comp = PrimitiveDateTime::parse(s.trim(), &f_frac)
        .or_else(|_| PrimitiveDateTime::parse(s.trim(), &f_sec))
        .map_err(|e| Error::msg(format!("无法识别的时间: {s}（{e}）")))?;
    let odt = comp.assume_offset(offset);
    Ok((odt.unix_timestamp(), odt.nanosecond()))
}

/// 时间字符串转时间戳（毫秒）
pub fn str_to_timestamp(t: &str) -> Result<i64> {
    use time::format_description::well_known::Iso8601;
    let offset =
        OffsetDateTime::parse(t, &Iso8601::DEFAULT).map_err(|e| Error::msg(e.to_string()))?;
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

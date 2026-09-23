use std::collections::BTreeSet;

use anyhow::{Error, Result};
use chrono::offset::{LocalResult, TimeZone};
use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use serde::Serialize;

type ValueSet = BTreeSet<u32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronType {
    Linux,
    Spring,
    Quartz,
}

impl CronType {
    pub fn parse(s: &str) -> Result<CronType> {
        match s.trim().to_ascii_lowercase().as_str() {
            "linux" => Ok(CronType::Linux),
            "spring" => Ok(CronType::Spring),
            "quartz" => Ok(CronType::Quartz),
            _ => Err(Error::msg("cron 类型仅支持：linux / spring / quartz")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CronType::Linux => "linux",
            CronType::Spring => "spring",
            CronType::Quartz => "quartz",
        }
    }

    fn field_count(self) -> (usize, usize) {
        match self {
            CronType::Linux => (5, 5),
            CronType::Spring => (6, 6),
            CronType::Quartz => (6, 7),
        }
    }

    fn dow_range(self) -> (u32, u32) {
        match self {
            CronType::Linux | CronType::Spring => (0, 7),
            CronType::Quartz => (1, 7),
        }
    }

    fn normalize_dow(self, v: u32) -> u32 {
        match self {
            CronType::Linux | CronType::Spring => {
                if v == 7 {
                    0
                } else {
                    v
                }
            }
            CronType::Quartz => v - 1,
        }
    }

    fn dow_names(self) -> &'static [(&'static str, u32)] {
        match self {
            CronType::Linux | CronType::Spring => &[
                ("SUN", 0),
                ("MON", 1),
                ("TUE", 2),
                ("WED", 3),
                ("THU", 4),
                ("FRI", 5),
                ("SAT", 6),
            ],
            CronType::Quartz => &[
                ("SUN", 1),
                ("MON", 2),
                ("TUE", 3),
                ("WED", 4),
                ("THU", 5),
                ("FRI", 6),
                ("SAT", 7),
            ],
        }
    }
}

const MONTH_NAMES: &[(&str, u32)] = &[
    ("JAN", 1),
    ("FEB", 2),
    ("MAR", 3),
    ("APR", 4),
    ("MAY", 5),
    ("JUN", 6),
    ("JUL", 7),
    ("AUG", 8),
    ("SEP", 9),
    ("OCT", 10),
    ("NOV", 11),
    ("DEC", 12),
];

#[derive(Debug, Clone, Default)]
struct DomField {
    values: ValueSet,
    last_day: bool,
    last_offset: i32,
    weekday: Option<u32>,
    last_weekday: bool,
}

impl DomField {
    fn is_all(&self) -> bool {
        self.values.is_empty() && !self.last_day && self.weekday.is_none() && !self.last_weekday
    }

    fn resolve(&self, year: u32, month: u32, dim: u32) -> ValueSet {
        let mut days = self.values.clone();
        if self.last_day {
            let d = dim as i32 + self.last_offset;
            if d >= 1 {
                days.insert(d as u32);
            }
        }
        if let Some(w) = self.weekday {
            days.insert(nearest_weekday(year, month, w.clamp(1, dim), dim));
        }
        if self.last_weekday {
            days.insert(last_weekday(year, month, dim));
        }
        days.into_iter().filter(|d| *d >= 1 && *d <= dim).collect()
    }
}

#[derive(Debug, Clone, Default)]
struct DowField {
    values: ValueSet,
    last_x: Option<u32>,
    nth: Option<(u32, u32)>,
}

impl DowField {
    fn is_all(&self) -> bool {
        self.values.is_empty() && self.last_x.is_none() && self.nth.is_none()
    }

    fn matches(&self, wd: u32, day: u32, dim: u32) -> bool {
        if self.values.contains(&wd) {
            return true;
        }
        if let Some(lx) = self.last_x {
            if wd == lx && day + 7 > dim {
                return true;
            }
        }
        if let Some((w, n)) = self.nth {
            if wd == w && (day - 1) / 7 + 1 == n {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
struct Schedule {
    seconds: ValueSet,
    minutes: ValueSet,
    hours: ValueSet,
    months: ValueSet,
    dom: DomField,
    dow: DowField,
    years: Option<ValueSet>,
    cron_type: CronType,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CronParseResult {
    pub valid: bool,
    pub error: Option<String>,
    pub expression: String,
    #[serde(rename = "type")]
    pub cron_type: String,
    pub description: Option<String>,
    pub next_times: Vec<String>,
}

pub fn parse_expression(expr: &str, count: u32, ty_str: &str) -> Result<CronParseResult> {
    let ty = CronType::parse(ty_str)?;
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(Error::msg("请输入 Cron 表达式"));
    }

    let expanded = expand_macro(trimmed, ty)?;
    let effective = expanded.unwrap_or_else(|| trimmed.to_string());

    let parts: Vec<&str> = effective.split_whitespace().collect();
    let (min_f, max_f) = ty.field_count();
    if parts.len() < min_f || parts.len() > max_f {
        let expected = if min_f == max_f {
            format!("{}", min_f)
        } else {
            format!("{} 或 {}", min_f, max_f)
        };
        return Err(Error::msg(format!(
            "{} 类型应包含 {} 个字段（当前 {} 个）",
            ty.name(),
            expected,
            parts.len()
        )));
    }

    let (sec_s, min_s, hour_s, dom_s, month_s, dow_s, year_s) = match ty {
        CronType::Linux => ("0", parts[0], parts[1], parts[2], parts[3], parts[4], None),
        CronType::Spring => (
            parts[0],
            parts[1],
            parts[2],
            parts[3],
            parts[4],
            parts[5],
            None,
        ),
        CronType::Quartz => (
            parts[0],
            parts[1],
            parts[2],
            parts[3],
            parts[4],
            parts[5],
            parts.get(6).copied(),
        ),
    };

    let mut seconds = parse_field(sec_s, 0, 59, &[])?;
    let mut minutes = parse_field(min_s, 0, 59, &[])?;
    let mut hours = parse_field(hour_s, 0, 23, &[])?;
    let mut months = parse_field(month_s, 1, 12, MONTH_NAMES)?;
    compact_all(&mut seconds, 0, 59);
    compact_all(&mut minutes, 0, 59);
    compact_all(&mut hours, 0, 23);
    compact_all(&mut months, 1, 12);
    let dom = parse_dom(dom_s, ty)?;
    let dow = parse_dow(dow_s, ty)?;
    let years = match year_s {
        Some(y) => {
            let mut set = parse_field(y, 1970, 2099, &[])?;
            compact_all(&mut set, 1970, 2099);
            if set.is_empty() {
                None
            } else {
                Some(set)
            }
        }
        None => None,
    };

    let schedule = Schedule {
        seconds,
        minutes,
        hours,
        months,
        dom,
        dow,
        years,
        cron_type: ty,
    };

    let now = Local::now();
    let next_times = compute_next_times(&schedule, &now, count.max(1) as usize);
    if next_times.is_empty() {
        return Err(Error::msg(
            "无法计算执行时间，请检查表达式（例如「2 月 30 日」这类不存在的日期）",
        ));
    }

    Ok(CronParseResult {
        valid: true,
        error: None,
        expression: effective,
        cron_type: ty.name().to_string(),
        description: Some(describe_schedule(&schedule)),
        next_times,
    })
}

fn expand_macro(expr: &str, ty: CronType) -> Result<Option<String>> {
    let lower = expr.trim().to_ascii_lowercase();
    let macros: [(&str, &str, &str); 7] = [
        ("@yearly", "0 0 1 1 *", "0 0 0 1 1 *"),
        ("@annually", "0 0 1 1 *", "0 0 0 1 1 *"),
        ("@monthly", "0 0 1 * *", "0 0 0 1 * *"),
        ("@weekly", "0 0 * * 0", "0 0 0 * * 0"),
        ("@daily", "0 0 * * *", "0 0 0 * * *"),
        ("@midnight", "0 0 * * *", "0 0 0 * * *"),
        ("@hourly", "0 * * * *", "0 0 * * * *"),
    ];
    for (k, linux_v, spring_v) in macros {
        if lower == k {
            return match ty {
                CronType::Quartz => Err(Error::msg("Quartz 不支持 @ 宏，请使用标准表达式")),
                CronType::Linux => Ok(Some(linux_v.to_string())),
                CronType::Spring => Ok(Some(spring_v.to_string())),
            };
        }
    }
    if lower == "@reboot" {
        return match ty {
            CronType::Linux => Err(Error::msg("@reboot 表示开机时执行，无法计算后续执行时间")),
            _ => Err(Error::msg("仅 Linux cron 支持 @reboot")),
        };
    }
    if lower.starts_with('@') {
        return Err(Error::msg(format!("未知的宏：{}", expr)));
    }
    Ok(None)
}

fn parse_field(field: &str, min: u32, max: u32, names: &[(&str, u32)]) -> Result<ValueSet> {
    let field = field.trim().to_ascii_uppercase();
    if field.is_empty() {
        return Err(Error::msg("字段不能为空"));
    }
    let mut result = BTreeSet::new();
    for item in field.split(',') {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::msg(format!("字段包含空的列表项：{}", field)));
        }
        let (base, step) = match item.split_once('/') {
            Some((b, s)) => (b, parse_uint(s, "步长")?),
            None => (item, 1),
        };
        if step == 0 {
            return Err(Error::msg(format!("步长不能为 0：{}", item)));
        }
        if base == "*" {
            extend_step(&mut result, min, max, step);
        } else if let Some((a, b)) = base.split_once('-') {
            let a = parse_value(a, names, min, max, "范围起点")?;
            let b = parse_value(b, names, min, max, "范围终点")?;
            if a > b {
                return Err(Error::msg(format!("范围起点不能大于终点：{}", item)));
            }
            extend_step(&mut result, a, b, step);
        } else {
            let v = parse_value(base, names, min, max, "数值")?;
            if step > 1 {
                extend_step(&mut result, v, max, step);
            } else {
                result.insert(v);
            }
        }
    }
    Ok(result)
}

fn parse_dom(field: &str, ty: CronType) -> Result<DomField> {
    let mut dom = DomField::default();
    for item in field.split(',') {
        let item = item.trim().to_ascii_uppercase();
        if item.is_empty() {
            return Err(Error::msg("日字段包含空的列表项"));
        }
        match item.as_str() {
            "*" | "?" => {
                if item == "?" && ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 '?' 字符"));
                }
            }
            "L" => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'L' 字符"));
                }
                dom.last_day = true
            }
            "LW" => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'W' 字符"));
                }
                dom.last_weekday = true;
            }
            _ if item.ends_with('W') && item.len() >= 2 => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'W' 字符"));
                }
                let day = parse_uint(&item[..item.len() - 1], "W 前的日期")?;
                if !(1..=31).contains(&day) {
                    return Err(Error::msg(format!("nW 中的 n 应在 1-31 之间：{}", item)));
                }
                dom.weekday = Some(day);
            }
            _ if item.len() >= 3 && item.starts_with('L') && item.as_bytes()[1] == b'-' => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'L' 字符"));
                }
                let n = parse_uint(&item[2..], "L-n 中的 n")?;
                if !(1..=30).contains(&n) {
                    return Err(Error::msg(format!("L-n 中的 n 应在 1-30 之间：{}", item)));
                }
                dom.last_day = true;
                dom.last_offset = -(n as i32);
            }
            _ => {
                let vals = parse_field(&item, 1, 31, &[])?;
                dom.values.extend(vals);
            }
        }
    }
    Ok(dom)
}

fn parse_dow(field: &str, ty: CronType) -> Result<DowField> {
    let mut dow = DowField::default();
    for item in field.split(',') {
        let item = item.trim().to_ascii_uppercase();
        if item.is_empty() {
            return Err(Error::msg("周字段包含空的列表项"));
        }
        match item.as_str() {
            "*" | "?" => {
                if item == "?" && ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 '?' 字符"));
                }
            }
            "L" => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'L' 字符"));
                }
                let v = if ty == CronType::Quartz { 6 } else { 0 };
                dow.values.insert(v);
            }
            _ if item.ends_with('L') && item.len() >= 2 => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 'L' 字符"));
                }
                let raw = parse_dow_number(&item[..item.len() - 1], ty)?;
                dow.last_x = Some(ty.normalize_dow(raw));
            }
            _ if item.contains('#') => {
                if ty == CronType::Linux {
                    return Err(Error::msg("Linux cron 不支持 '#' 字符"));
                }
                let (d, n) = item.split_once('#').expect("checked contains");
                let raw = parse_dow_number(d, ty)?;
                let nth = parse_uint(n, "# 后的序号")?;
                if !(1..=5).contains(&nth) {
                    return Err(Error::msg(format!("n#m 中的 m 应在 1-5 之间：{}", item)));
                }
                dow.nth = Some((ty.normalize_dow(raw), nth));
            }
            _ => {
                let vals = parse_dow_values(&item, ty)?;
                dow.values.extend(vals);
            }
        }
    }
    Ok(dow)
}

fn parse_dow_values(field: &str, ty: CronType) -> Result<ValueSet> {
    let (min, max) = ty.dow_range();
    let raw = parse_field(field, min, max, ty.dow_names())?;
    Ok(raw.into_iter().map(|v| ty.normalize_dow(v)).collect())
}

fn parse_dow_number(s: &str, ty: CronType) -> Result<u32> {
    if let Ok(n) = s.parse::<u32>() {
        let (min, max) = ty.dow_range();
        if n < min || n > max {
            return Err(Error::msg(format!("周数值应在 {}-{} 之间：{}", min, max, s)));
        }
        Ok(n)
    } else if let Some(v) = name_value(s, ty.dow_names()) {
        Ok(v)
    } else {
        Err(Error::msg(format!("无法识别的周数值：{}", s)))
    }
}

fn parse_value(s: &str, names: &[(&str, u32)], min: u32, max: u32, what: &str) -> Result<u32> {
    if let Ok(n) = s.parse::<u32>() {
        if n < min || n > max {
            return Err(Error::msg(format!(
                "{}超出范围（{}-{}）：{}",
                what, min, max, n
            )));
        }
        Ok(n)
    } else if let Some(v) = name_value(s, names) {
        Ok(v)
    } else {
        Err(Error::msg(format!("无法识别的{}：{}", what, s)))
    }
}

fn name_value(s: &str, names: &[(&str, u32)]) -> Option<u32> {
    let upper = s.to_ascii_uppercase();
    let key = &upper[..upper.len().min(3)];
    names
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

fn parse_uint(s: &str, what: &str) -> Result<u32> {
    s.trim()
        .parse::<u32>()
        .map_err(|_| Error::msg(format!("{}无效：{}", what, s)))
}

fn extend_step(set: &mut ValueSet, start: u32, end: u32, step: u32) {
    let mut v = start;
    while v <= end {
        set.insert(v);
        v += step;
    }
}

fn compact_all(set: &mut ValueSet, min: u32, max: u32) {
    if set.len() as u32 == max - min + 1 {
        set.clear();
    }
}

fn compute_next_times(schedule: &Schedule, from: &DateTime<Local>, count: usize) -> Vec<String> {
    let mut results = Vec::new();
    let mut cursor = from.clone();
    let start_year = from.year() as u32;
    while results.len() < count {
        match find_next(schedule, &cursor, start_year) {
            Some(dt) => {
                results.push(dt.format("%Y-%m-%d %H:%M:%S").to_string());
                cursor = dt;
            }
            None => break,
        }
    }
    results
}

fn iter_vals(set: &ValueSet, min: u32, max: u32) -> Vec<u32> {
    if set.is_empty() {
        (min..=max).collect()
    } else {
        set.iter().copied().collect()
    }
}

fn range_from(set: &ValueSet, min: u32, max: u32, from: u32) -> Vec<u32> {
    if set.is_empty() {
        (from.max(min)..=max).collect()
    } else {
        set.range(from..).copied().collect()
    }
}

fn find_next(schedule: &Schedule, after: &DateTime<Local>, start_year: u32) -> Option<DateTime<Local>> {
    let ay = after.year() as u32;
    let years: Vec<u32> = match &schedule.years {
        Some(set) => set.range(ay..).copied().take(130).collect(),
        None => (start_year..=start_year + 12).collect(),
    };

    for year in years {
        if year < ay {
            continue;
        }
        let months: Vec<u32> = if year == ay {
            range_from(&schedule.months, 1, 12, after.month())
        } else {
            iter_vals(&schedule.months, 1, 12)
        };
        for month in months {
            let dim = days_in_month(month, year);
            let days: Vec<u32> = matching_days(schedule, year, month, dim);
            let days: Vec<u32> = if year == ay && month == after.month() {
                days.into_iter().filter(|d| *d >= after.day()).collect()
            } else {
                days
            };
            for day in days {
                let hours: Vec<u32> =
                    if year == ay && month == after.month() && day == after.day() {
                        range_from(&schedule.hours, 0, 23, after.hour())
                    } else {
                        iter_vals(&schedule.hours, 0, 23)
                    };
                for hour in hours {
                    let minutes: Vec<u32> = if year == ay
                        && month == after.month()
                        && day == after.day()
                        && hour == after.hour()
                    {
                        range_from(&schedule.minutes, 0, 59, after.minute())
                    } else {
                        iter_vals(&schedule.minutes, 0, 59)
                    };
                    for minute in minutes {
                        let seconds: Vec<u32> = if year == ay
                            && month == after.month()
                            && day == after.day()
                            && hour == after.hour()
                            && minute == after.minute()
                        {
                            range_from(&schedule.seconds, 0, 59, after.second() + 1)
                        } else {
                            iter_vals(&schedule.seconds, 0, 59)
                        };
                        for sec in seconds {
                            if let Some(cand) = build_dt(year, month, day, hour, minute, sec) {
                                if cand > *after {
                                    return Some(cand);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn build_dt(year: u32, month: u32, day: u32, hour: u32, minute: u32, sec: u32) -> Option<DateTime<Local>> {
    match Local.with_ymd_and_hms(year as i32, month, day, hour, minute, sec) {
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(earlier, _) => Some(earlier),
        LocalResult::None => None,
    }
}

fn matching_days(schedule: &Schedule, year: u32, month: u32, dim: u32) -> Vec<u32> {
    let dom_all = schedule.dom.is_all();
    let dow_all = schedule.dow.is_all();
    let mut days = BTreeSet::new();
    if dom_all {
        days.extend(1..=dim);
    } else {
        days.extend(schedule.dom.resolve(year, month, dim));
    }
    if !dow_all {
        if dom_all {
            days.retain(|d| schedule.dow.matches(weekday_index(year, month, *d), *d, dim));
        } else {
            for d in 1..=dim {
                if schedule.dow.matches(weekday_index(year, month, d), d, dim) {
                    days.insert(d);
                }
            }
        }
    }
    days.into_iter().collect()
}

fn weekday_index(year: u32, month: u32, day: u32) -> u32 {
    match NaiveDate::from_ymd_opt(year as i32, month, day) {
        Some(d) => d.weekday().num_days_from_sunday() as u32,
        None => 7,
    }
}

fn nearest_weekday(year: u32, month: u32, day: u32, dim: u32) -> u32 {
    let d = day.clamp(1, dim);
    match weekday_index(year, month, d) {
        0 => {
            if d + 1 <= dim {
                d + 1
            } else {
                d - 2
            }
        }
        5 => {
            if d > 1 {
                d - 1
            } else {
                d + 2
            }
        }
        6 => {
            if d + 1 <= dim {
                d + 1
            } else {
                d - 2
            }
        }
        _ => d,
    }
}

fn last_weekday(year: u32, month: u32, dim: u32) -> u32 {
    let mut d = dim;
    while d >= 1 && weekday_index(year, month, d) > 4 {
        d -= 1;
    }
    d
}

fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

const DOW_NAMES: [&str; 7] = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

fn fmt_list(set: &ValueSet) -> String {
    set.iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("、")
}

fn step_pattern(set: &ValueSet, min: u32, _max: u32) -> Option<u32> {
    if set.len() < 2 {
        return None;
    }
    let first = *set.iter().next().unwrap();
    let last = *set.iter().next_back().unwrap();
    if first != min {
        return None;
    }
    let n = set.len() as u32;
    let step = (last - first) / (n - 1);
    if step < 2 {
        return None;
    }
    let expected: ValueSet = (first..=last).step_by(step as usize).collect();
    if *set == expected {
        Some(step)
    } else {
        None
    }
}

fn contiguous_range(set: &ValueSet) -> Option<(u32, u32)> {
    let v: Vec<u32> = set.iter().copied().collect();
    if v.len() < 2 {
        return None;
    }
    for i in 1..v.len() {
        if v[i] != v[i - 1] + 1 {
            return None;
        }
    }
    Some((v[0], v[v.len() - 1]))
}

fn describe_schedule(s: &Schedule) -> String {
    let sec_all = s.seconds.is_empty();
    let min_all = s.minutes.is_empty();
    let hour_all = s.hours.is_empty();
    let dom_all = s.dom.is_all();
    let dow_all = s.dow.is_all();
    let mon_all = s.months.is_empty();

    let sec_zero = !sec_all && s.seconds.len() == 1 && s.seconds.contains(&0);
    let min_zero = !min_all && s.minutes.len() == 1 && s.minutes.contains(&0);
    let sec_eff_all = sec_all || s.cron_type == CronType::Linux;
    let sec_whole = sec_eff_all || sec_zero;

    if dom_all && dow_all && mon_all {
        if hour_all && min_all {
            if sec_eff_all {
                return match s.cron_type {
                    CronType::Linux => "每分钟执行一次".to_string(),
                    _ => "每秒执行一次".to_string(),
                };
            }
            if sec_zero {
                return "每分钟执行一次".to_string();
            }
            if let Some(k) = step_pattern(&s.seconds, 0, 59) {
                return format!("每 {} 秒执行一次", k);
            }
            return format!("第 {} 秒执行一次", fmt_list(&s.seconds));
        }
        if hour_all && sec_whole {
            if let Some(k) = step_pattern(&s.minutes, 0, 59) {
                return format!("每 {} 分钟执行一次", k);
            }
            if min_zero {
                return "每小时整点执行".to_string();
            }
            if !min_all {
                return format!("每小时第 {} 分执行一次", fmt_list(&s.minutes));
            }
        }
        if (min_all || min_zero) && sec_whole {
            if let Some(k) = step_pattern(&s.hours, 0, 23) {
                return format!("每 {} 小时执行一次", k);
            }
        }
    }

    let date_part = if !mon_all {
        let m = match step_pattern(&s.months, 1, 12) {
            Some(1) => "每月".to_string(),
            Some(k) => format!("每 {} 个月", k),
            None => format!("{} 月", fmt_list(&s.months)),
        };
        format!("{}{}", m, day_part(s, dom_all, dow_all))
    } else {
        day_part(s, dom_all, dow_all)
    };

    let time_part = if hour_all {
        if min_all && sec_all {
            String::new()
        } else if min_all {
            match step_pattern(&s.seconds, 0, 59) {
                Some(k) => format!("，每 {} 秒", k),
                None => format!("，第 {} 秒", fmt_list(&s.seconds)),
            }
        } else if min_zero {
            "，每小时整点".to_string()
        } else if let Some(k) = step_pattern(&s.minutes, 0, 59) {
            let mut t = format!("，每 {} 分钟", k);
            if !sec_all && !sec_zero {
                t = format!("{}（第 {} 秒）", t, fmt_list(&s.seconds));
            }
            t
        } else {
            format!("，每小时 {} 分", fmt_list(&s.minutes))
        }
    } else {
        let secs = if s.cron_type == CronType::Linux {
            ValueSet::new()
        } else {
            s.seconds.clone()
        };
        format!("，{}", fmt_times(&s.hours, &s.minutes, &secs))
    };

    format!("{}{}执行", date_part, time_part)
}

fn day_part(s: &Schedule, dom_all: bool, dow_all: bool) -> String {
    if dom_all && dow_all {
        return "每天".to_string();
    }
    if !dom_all {
        let mut d = describe_dom(&s.dom);
        if !dow_all {
            d = format!("{}或{}", d, describe_dow(&s.dow));
        }
        return d;
    }
    describe_dow(&s.dow)
}

fn describe_dom(dom: &DomField) -> String {
    let mut parts = Vec::new();
    if !dom.values.is_empty() {
        if let Some(k) = step_pattern(&dom.values, 1, 31) {
            parts.push(format!("每隔 {} 天", k));
        } else {
            parts.push(format!("{}日", fmt_list(&dom.values)));
        }
    }
    if dom.last_day {
        parts.push(if dom.last_offset != 0 {
            format!("倒数第 {} 天", -dom.last_offset)
        } else {
            "最后一天".to_string()
        });
    }
    if let Some(w) = dom.weekday {
        parts.push(format!("{} 号最近的工作日", w));
    }
    if dom.last_weekday {
        parts.push("最后一个工作日".to_string());
    }
    if parts.is_empty() {
        "每天".to_string()
    } else {
        format!("每月{}", parts.join("、"))
    }
}

fn describe_dow(dow: &DowField) -> String {
    let mut parts = Vec::new();
    if !dow.values.is_empty() {
        if let Some((a, b)) = contiguous_range(&dow.values) {
            if a == 0 && b == 6 {
                parts.push("每天".to_string());
            } else if a == b {
                parts.push(format!("每{}", DOW_NAMES[a as usize]));
            } else {
                parts.push(format!(
                    "每{}至{}",
                    DOW_NAMES[a as usize], DOW_NAMES[b as usize]
                ));
            }
        } else if dow.values.len() == 1 {
            let v = *dow.values.iter().next().unwrap();
            parts.push(format!("每{}", DOW_NAMES[v as usize]));
        } else {
            let names: Vec<&str> = dow
                .values
                .iter()
                .map(|v| DOW_NAMES[*v as usize])
                .collect();
            parts.push(format!("每{}", names.join("、")));
        }
    }
    if let Some(lx) = dow.last_x {
        parts.push(format!("每月最后一个{}", DOW_NAMES[lx as usize]));
    }
    if let Some((w, n)) = dow.nth {
        parts.push(format!("每月第 {} 个{}", n, DOW_NAMES[w as usize]));
    }
    if parts.is_empty() {
        "每天".to_string()
    } else {
        parts.join("、")
    }
}

fn fmt_times(hours: &ValueSet, minutes: &ValueSet, seconds: &ValueSet) -> String {
    let sec_all = seconds.is_empty();
    let min_all = minutes.is_empty();
    let total = hours.len() * minutes.len().max(1) * seconds.len().max(1);
    let mut list = Vec::new();
    'outer: for &h in hours {
        if min_all {
            list.push(format!("{} 点", h));
            if list.len() >= 6 {
                break;
            }
        } else {
            for &m in minutes {
                if sec_all {
                    list.push(format!("{:02}:{:02}", h, m));
                } else {
                    for &s in seconds {
                        list.push(format!("{:02}:{:02}:{:02}", h, m, s));
                        if list.len() >= 8 {
                            break 'outer;
                        }
                    }
                }
                if list.len() >= 8 {
                    break 'outer;
                }
            }
        }
    }
    let mut out = list.join("、");
    if total > list.len() {
        out.push('等');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next(expr: &str, ty: &str, count: u32) -> Vec<String> {
        parse_expression(expr, count, ty).unwrap().next_times
    }

    #[test]
    fn linux_basic() {
        let r = parse_expression("*/5 * * * *", 3, "linux").unwrap();
        assert!(r.valid);
        assert_eq!(r.next_times.len(), 3);
    }

    #[test]
    fn linux_macro() {
        let r = parse_expression("@daily", 2, "linux").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert_eq!(r.expression, "0 0 * * *");
    }

    #[test]
    fn spring_seconds() {
        let r = parse_expression("*/10 * * * * *", 3, "spring").unwrap();
        assert_eq!(r.next_times.len(), 3);
    }

    #[test]
    fn spring_dow_7_is_sunday() {
        let r = parse_expression("0 0 0 * * 7", 3, "spring").unwrap();
        assert_eq!(r.next_times.len(), 3);
        assert!(r.description.unwrap().contains("周日"));
    }

    #[test]
    fn quartz_dow_1_is_sunday() {
        let r = parse_expression("0 0 0 * * 1", 3, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 3);
        assert!(r.description.unwrap().contains("周日"));
    }

    #[test]
    fn quartz_last_day() {
        let r = parse_expression("0 0 0 L * ?", 2, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert!(r.description.unwrap().contains("最后一天"));
    }

    #[test]
    fn quartz_nth_weekday() {
        let r = parse_expression("0 0 10 ? * 6#3", 2, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert!(r.description.unwrap().contains("周五"));
    }

    #[test]
    fn spring_nth_weekday() {
        let r = parse_expression("0 0 10 ? * 5#3", 2, "spring").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert!(r.description.unwrap().contains("周五"));
    }

    #[test]
    fn spring_last_weekday() {
        let r = parse_expression("0 0 10 ? * 6L", 2, "spring").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert!(r.description.unwrap().contains("周六"));
    }

    #[test]
    fn quartz_year_field() {
        let r = parse_expression("0 0 0 29 2 ? 2028", 1, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 1);
        assert!(r.next_times[0].starts_with("2028-02-29"));
    }

    #[test]
    fn impossible_date() {
        assert!(parse_expression("0 0 30 2 *", 3, "linux").is_err());
    }

    #[test]
    fn wrong_field_count() {
        assert!(parse_expression("* * * * * *", 3, "linux").is_err());
        assert!(parse_expression("* * * * *", 3, "spring").is_err());
    }

    #[test]
    fn linux_rejects_special_chars() {
        assert!(parse_expression("* * * * ?", 3, "linux").is_err());
        assert!(parse_expression("* * L * *", 3, "linux").is_err());
    }

    #[test]
    fn quartz_rejects_macro() {
        assert!(parse_expression("@daily", 2, "quartz").is_err());
    }

    #[test]
    fn or_semantics() {
        let r = parse_expression("0 0 1 * 1", 5, "linux").unwrap();
        assert_eq!(r.next_times.len(), 5);
    }

    #[test]
    fn dom_l_offset() {
        let r = parse_expression("0 0 0 L-3 * ?", 1, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 1);
        assert!(r.description.unwrap().contains("倒数第 3 天"));
    }

    #[test]
    fn nearest_weekday_rule() {
        let r = parse_expression("0 0 0 1W * ?", 2, "quartz").unwrap();
        assert_eq!(r.next_times.len(), 2);
        assert!(r.description.unwrap().contains("最近的工作日"));
    }

    #[test]
    fn invalid_dow_range() {
        assert!(parse_expression("0 0 0 * * 8", 2, "quartz").is_err());
        assert!(parse_expression("0 0 0 * * 8", 2, "spring").is_err());
    }

    #[test]
    fn description_examples() {
        assert_eq!(
            parse_expression("*/5 * * * *", 2, "linux").unwrap().description.unwrap(),
            "每 5 分钟执行一次"
        );
        assert_eq!(
            parse_expression("0 9 * * 1-5", 2, "linux").unwrap().description.unwrap(),
            "每周一至周五，09:00执行"
        );
        assert_eq!(
            parse_expression("0 0 8 * * *", 2, "spring").unwrap().description.unwrap(),
            "每天，08:00:00执行"
        );
        assert_eq!(
            parse_expression("0 0 0 L * ?", 2, "quartz").unwrap().description.unwrap(),
            "每月最后一天，00:00:00执行"
        );
    }
}

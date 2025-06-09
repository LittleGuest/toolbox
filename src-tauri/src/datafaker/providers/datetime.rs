pub const YEAR_EPOCH_MIN: i32 = 1970;
pub const YEAR_EPOCH_MAX: i32 = 2038;
pub const YEAR_MIN: i32 = (i32::MIN >> 13) + 1;
pub const YEAR_MAX: i32 = (i32::MAX >> 13) - 1;
const MONTH_MIN: u32 = 1;
const MONTH_MAX: u32 = 12;
const HOUR_MIN: u32 = 0;
const HOUR_MAX: u32 = 23;
const MILLI_MIN: u32 = 0;
const MILLI_MAX: u32 = 999;
const MIN_SEC_MIN: u32 = 0;
const MIN_SEC_MAX: u32 = 59;
const MICRO_MIN: u32 = 0;
const MICRO_MAX: u32 = 999_999;
const NANO_MIN: u32 = 0;
const NANO_MAX: u32 = 999_999_999;

pub struct DateTime;

impl DateTime {}

fn year_epoch() -> i32 {
    fastrand::i32(YEAR_EPOCH_MIN..=YEAR_EPOCH_MAX)
}

fn year_unsafe() -> i32 {
    fastrand::i32(YEAR_MIN..=YEAR_MAX)
}

fn month() -> u32 {
    fastrand::u32(MONTH_MIN..=MONTH_MAX)
}

fn days(year: i32, month: u8) -> u8 {
    let days_max = days_in_month(year, month);
    fastrand::u8(1..=days_max)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    time::util::days_in_month(month.try_into().unwrap(), year)
}

fn hour() -> u32 {
    fastrand::u32(HOUR_MIN..=HOUR_MAX)
}

fn min_sec() -> u32 {
    fastrand::u32(MIN_SEC_MIN..=MIN_SEC_MAX)
}

fn min() -> u32 {
    min_sec()
}

fn sec() -> u32 {
    min_sec()
}

fn hms() -> (u32, u32, u32) {
    let h = hour();
    let m = min();
    let s = sec();

    (h, m, s)
}

fn milli() -> u32 {
    fastrand::u32(MILLI_MIN..=MILLI_MAX)
}

fn micro() -> u32 {
    fastrand::u32(MICRO_MIN..=MICRO_MAX)
}

fn nano() -> u32 {
    fastrand::u32(NANO_MIN..=NANO_MAX)
}

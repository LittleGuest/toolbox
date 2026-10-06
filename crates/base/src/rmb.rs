use anyhow::{Error, Result};
use serde::Serialize;

const UPPER_DIGITS: [&str; 10] = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];

const SECTION_UNITS: [&str; 4] = ["", "拾", "佰", "仟"];

const GROUP_UNITS: [&str; 4] = ["", "万", "亿", "万亿"];

const MAX_INT_DIGITS: usize = 16;

fn strip_currency_prefix(data: &str) -> &str {
    let mut rest = data.trim();
    loop {
        let mut matched = false;
        for prefix in ["人民币", "RMB", "rmb", "Rmb", "CNY", "cny", "¥", "￥"] {
            if let Some(stripped) = rest.strip_prefix(prefix) {
                rest = stripped.trim();
                matched = true;
                break;
            }
        }
        if !matched {
            return rest;
        }
    }
}

fn normalize_amount(input: &str) -> Result<(bool, String, String)> {
    let data = strip_currency_prefix(input)
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != '，' && *c != '_')
        .collect::<String>();

    if data.is_empty() {
        return Err(Error::msg("请输入金额"));
    }

    let (negative, unsigned) = match data.strip_prefix(['-', '−', '负']) {
        Some(rest) => (true, rest),
        None => (false, data.as_str()),
    };

    if unsigned.matches('.').count() > 1 {
        return Err(Error::msg("金额格式不正确：小数点最多一个"));
    }

    let (int_raw, frac_raw) = match unsigned.split_once('.') {
        Some((int_part, frac_part)) => (int_part, frac_part),
        None => (unsigned, ""),
    };

    if int_raw.is_empty() && frac_raw.is_empty() {
        return Err(Error::msg("请输入金额"));
    }
    if !int_raw.chars().all(|c| c.is_ascii_digit()) || !frac_raw.chars().all(|c| c.is_ascii_digit())
    {
        return Err(Error::msg(
            "金额格式不正确：仅支持数字、小数点和千分位分隔符",
        ));
    }
    if frac_raw.len() > 2 {
        return Err(Error::msg("角、分为两位小数，小数部分最多输入两位"));
    }

    let trimmed = int_raw.trim_start_matches('0');
    let int_digits = if trimmed.is_empty() { "0" } else { trimmed };
    if int_digits.len() > MAX_INT_DIGITS {
        return Err(Error::msg(format!(
            "整数部分最多支持 {MAX_INT_DIGITS} 位（万亿）"
        )));
    }

    let mut frac = frac_raw.to_string();
    while frac.len() < 2 {
        frac.push('0');
    }

    Ok((negative, int_digits.to_string(), frac))
}

pub fn amount_to_upper(
    input: &str,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
    jiao_zheng: bool,
) -> Result<String> {
    let (negative, int_digits, frac) = normalize_amount(input)?;
    let yuan = if yuan == Some("圆") { "圆" } else { "元" };
    let zheng = if zheng == Some("正") { "正" } else { "整" };
    let zero = UPPER_DIGITS[0];

    let digits: Vec<usize> = frac
        .chars()
        .map(|c| c.to_digit(10).unwrap_or(0) as usize)
        .collect();
    let jiao = digits.first().copied().unwrap_or(0);
    let fen = digits.get(1).copied().unwrap_or(0);

    let int_text = if int_digits == "0" {
        String::new()
    } else {
        digits_to_upper(&int_digits)
    };

    let mut out = String::new();
    if negative {
        out.push('负');
    }
    if int_text.is_empty() {
        out.push_str(zero);
    } else {
        out.push_str(&int_text);
    }
    out.push_str(yuan);

    if jiao == 0 && fen == 0 {
        if zheng_yuan {
            out.push_str(zheng);
        }
    } else if jiao == 0 {
        out.push_str(zero);
        out.push_str(UPPER_DIGITS[fen]);
        out.push_str("分");
    } else if fen == 0 {
        out.push_str(UPPER_DIGITS[jiao]);
        out.push_str("角");
        if jiao_zheng {
            out.push_str(zheng);
        }
    } else {
        out.push_str(UPPER_DIGITS[jiao]);
        out.push_str("角");
        out.push_str(UPPER_DIGITS[fen]);
        out.push_str("分");
    }

    Ok(out)
}

fn digits_to_upper(digits: &str) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut groups: Vec<String> = Vec::new();
    let mut end = chars.len();
    while end > 0 {
        let start = end.saturating_sub(4);
        groups.push(chars[start..end].iter().collect());
        end = start;
    }
    groups.reverse();

    let mut out = String::new();
    let mut zero_skipped = false;
    for (index, group) in groups.iter().enumerate() {
        let power = groups.len() - 1 - index;
        if group.chars().all(|c| c == '0') {
            zero_skipped = !out.is_empty();
            continue;
        }
        if !out.is_empty() && (zero_skipped || group.starts_with('0')) {
            out.push_str(UPPER_DIGITS[0]);
        }
        out.push_str(&section_to_upper(group));
        out.push_str(GROUP_UNITS[power]);
        zero_skipped = false;
    }

    out
}

fn section_to_upper(section: &str) -> String {
    let chars: Vec<char> = section.chars().collect();
    let len = chars.len();
    let mut out = String::new();
    let mut zero_pending = false;

    for (index, ch) in chars.iter().enumerate() {
        let power = len - 1 - index;
        let digit = ch.to_digit(10).unwrap_or(0) as usize;
        if digit == 0 {
            if !out.is_empty() {
                zero_pending = true;
            }
            continue;
        }
        if zero_pending {
            out.push_str(UPPER_DIGITS[0]);
            zero_pending = false;
        }
        out.push_str(UPPER_DIGITS[digit]);
        out.push_str(SECTION_UNITS[power]);
    }

    out
}

#[derive(Debug, Serialize)]
pub struct RmbParseResult {
    pub amount: String,
    pub grouped: String,
    pub upper: String,
}

pub fn upper_to_amount(
    input: &str,
    yuan: Option<&str>,
    zheng: Option<&str>,
    zheng_yuan: bool,
) -> Result<RmbParseResult> {
    let data = strip_currency_prefix(input)
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != '，' && *c != '_')
        .collect::<String>();

    if data.is_empty() {
        return Err(Error::msg("请输入中文大写金额"));
    }
    if data
        .trim_start_matches(['-', '−', '负'])
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.')
    {
        let (negative, digits, frac) = normalize_amount(&data)?;
        let amount = format!("{}{}.{}", if negative { "-" } else { "" }, digits, frac);
        let upper = amount_to_upper(&amount, yuan, zheng, zheng_yuan, false)?;
        return Ok(RmbParseResult {
            grouped: group_thousands(&amount),
            amount,
            upper,
        });
    }

    let (negative, unsigned) = match data.strip_prefix(['-', '−', '负']) {
        Some(rest) => (true, rest.to_string()),
        None => (false, data.clone()),
    };

    let body = unsigned.trim_end_matches(['整', '正']);

    let (int_text, frac_text) = match body.find(['元', '圆', '块']) {
        Some(index) => {
            let unit_len = body[index..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
            (&body[..index], &body[index + unit_len..])
        }
        None => {
            if body.contains(['角', '分']) {
                ("", body)
            } else {
                (body, "")
            }
        }
    };

    if int_text.is_empty() && frac_text.is_empty() {
        return Err(Error::msg("请输入中文大写金额"));
    }

    let int_value = parse_chinese_int(int_text)?;
    let (jiao, fen) = parse_chinese_fraction(frac_text)?;

    let amount = format!(
        "{}{}.{}{}",
        if negative { "-" } else { "" },
        int_value,
        jiao,
        fen
    );
    let upper = amount_to_upper(&amount, yuan, zheng, zheng_yuan, false)?;

    Ok(RmbParseResult {
        grouped: group_thousands(&amount),
        amount,
        upper,
    })
}

fn chinese_digit(ch: char) -> Option<i128> {
    match ch {
        '零' | '〇' => Some(0),
        '壹' | '一' => Some(1),
        '贰' | '貳' | '二' | '两' | '兩' => Some(2),
        '叁' | '參' | '三' => Some(3),
        '肆' | '四' => Some(4),
        '伍' | '五' => Some(5),
        '陆' | '陸' | '六' => Some(6),
        '柒' | '七' => Some(7),
        '捌' | '八' => Some(8),
        '玖' | '九' => Some(9),
        _ => None,
    }
}

fn parse_chinese_int(data: &str) -> Result<i128> {
    if data.is_empty() {
        return Ok(0);
    }

    for (unit, factor) in [('亿', 100_000_000i128), ('万', 10_000i128)] {
        if let Some(index) = data.find(unit) {
            let unit_len = unit.len_utf8();
            let head = parse_chinese_int(&data[..index])?;
            let tail = parse_chinese_int(&data[index + unit_len..])?;
            return head
                .checked_mul(factor)
                .and_then(|value| value.checked_add(tail))
                .ok_or_else(|| Error::msg("金额超出可计算范围"));
        }
    }

    let mut total: i128 = 0;
    let mut pending: Option<i128> = None;
    let mut zero_seen = false;

    for ch in data.chars() {
        if let Some(digit) = chinese_digit(ch) {
            if digit == 0 {
                pending = None;
                zero_seen = true;
            } else {
                pending = Some(digit);
                zero_seen = false;
            }
            continue;
        }
        let factor = match ch {
            '拾' | '十' => 10,
            '佰' | '百' => 100,
            '仟' | '千' => 1000,
            _ => return Err(Error::msg(format!("无法识别的字符：{ch}"))),
        };
        let value = pending.take().unwrap_or(if zero_seen { 0 } else { 1 });
        zero_seen = false;
        total = total
            .checked_add(value * factor)
            .ok_or_else(|| Error::msg("金额超出可计算范围"))?;
    }

    if let Some(value) = pending {
        total = total
            .checked_add(value)
            .ok_or_else(|| Error::msg("金额超出可计算范围"))?;
    }

    Ok(total)
}

fn parse_chinese_fraction(data: &str) -> Result<(u8, u8)> {
    if data.is_empty() {
        return Ok((0, 0));
    }

    let mut jiao: u8 = 0;
    let mut fen: u8 = 0;
    let mut pending: Option<u8> = None;
    let mut zero_seen = false;

    for ch in data.chars() {
        if let Some(digit) = chinese_digit(ch) {
            if digit == 0 {
                pending = None;
                zero_seen = true;
            } else {
                pending = Some(digit as u8);
                zero_seen = false;
            }
            continue;
        }
        let value = pending.take().unwrap_or(if zero_seen { 0 } else { 1 });
        zero_seen = false;
        match ch {
            '角' => jiao = value,
            '分' => fen = value,
            _ => return Err(Error::msg(format!("无法识别的字符：{ch}"))),
        }
    }

    if pending.is_some() {
        return Err(Error::msg("小数部分缺少「角」「分」单位"));
    }

    Ok((jiao, fen))
}

fn group_thousands(amount: &str) -> String {
    let (negative, unsigned) = match amount.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, amount),
    };
    let (int_part, frac_part) = match unsigned.split_once('.') {
        Some((int_part, frac_part)) => (int_part, Some(frac_part)),
        None => (unsigned, None),
    };

    let chars: Vec<char> = int_part.chars().collect();
    let mut grouped = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*ch);
    }

    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(&grouped);
    if let Some(frac) = frac_part {
        out.push('.');
        out.push_str(frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper(input: &str) -> String {
        amount_to_upper(input, None, None, true, false).unwrap()
    }

    #[test]
    fn test_amount_to_upper() {
        assert_eq!(upper("0"), "零元整");
        assert_eq!(upper("0.00"), "零元整");
        assert_eq!(upper("1"), "壹元整");
        assert_eq!(upper("10"), "壹拾元整");
        assert_eq!(upper("1001"), "壹仟零壹元整");
        assert_eq!(upper("1100"), "壹仟壹佰元整");
        assert_eq!(upper("1010"), "壹仟零壹拾元整");
        assert_eq!(upper("10000"), "壹万元整");
        assert_eq!(upper("10001"), "壹万零壹元整");
        assert_eq!(upper("10010"), "壹万零壹拾元整");
        assert_eq!(upper("10101"), "壹万零壹佰零壹元整");
        assert_eq!(upper("100000"), "壹拾万元整");
        assert_eq!(upper("100100"), "壹拾万零壹佰元整");
        assert_eq!(upper("1010000"), "壹佰零壹万元整");
        assert_eq!(upper("123456"), "壹拾贰万叁仟肆佰伍拾陆元整");
        assert_eq!(upper("100000000"), "壹亿元整");
        assert_eq!(upper("100000001"), "壹亿零壹元整");
        assert_eq!(upper("100010000"), "壹亿零壹万元整");
        assert_eq!(upper("101000000"), "壹亿零壹佰万元整");
        assert_eq!(upper("1000000000000"), "壹万亿元整");
        assert_eq!(upper("1000000000001"), "壹万亿零壹元整");

        assert_eq!(upper("1,409.50"), "壹仟肆佰零玖元伍角");
        assert_eq!(upper("6007.14"), "陆仟零柒元壹角肆分");
        assert_eq!(upper("16409.02"), "壹万陆仟肆佰零玖元零贰分");
        assert_eq!(upper("0.56"), "零元伍角陆分");
        assert_eq!(upper("1680.32"), "壹仟陆佰捌拾元叁角贰分");
        assert_eq!(upper("¥1,234.56"), "壹仟贰佰叁拾肆元伍角陆分");
        assert_eq!(upper("人民币 12.30"), "壹拾贰元叁角");
        assert_eq!(upper("-1234.56"), "负壹仟贰佰叁拾肆元伍角陆分");

        assert_eq!(upper("0.01"), "零元零壹分");
        assert_eq!(upper("1.05"), "壹元零伍分");
        assert_eq!(upper("1000.01"), "壹仟元零壹分");
        assert_eq!(upper("2030.05"), "贰仟零叁拾元零伍分");
        assert_eq!(upper("100000000.01"), "壹亿元零壹分");
        assert_eq!(upper("100.10"), "壹佰元壹角");
        assert_eq!(upper("20000000"), "贰仟万元整");
        assert_eq!(upper("10000001"), "壹仟万零壹元整");
        assert_eq!(upper("1000000010"), "壹拾亿零壹拾元整");
    }

    #[test]
    fn test_options() {
        assert_eq!(
            amount_to_upper("1234.56", Some("圆"), Some("正"), true, false).unwrap(),
            "壹仟贰佰叁拾肆圆伍角陆分"
        );
        assert_eq!(
            amount_to_upper("100", Some("圆"), Some("正"), true, false).unwrap(),
            "壹佰圆正"
        );
        assert_eq!(
            amount_to_upper("100.50", None, None, true, true).unwrap(),
            "壹佰元伍角整"
        );
        assert_eq!(
            amount_to_upper("100.56", None, None, true, true).unwrap(),
            "壹佰元伍角陆分"
        );
    }

    #[test]
    fn test_zheng_yuan_toggle() {
        assert_eq!(
            amount_to_upper("100", None, None, false, false).unwrap(),
            "壹佰元"
        );
        assert_eq!(
            amount_to_upper("0", None, None, false, false).unwrap(),
            "零元"
        );
        assert_eq!(
            amount_to_upper("10000.00", Some("圆"), Some("正"), false, false).unwrap(),
            "壹万圆"
        );

        assert_eq!(
            amount_to_upper("100.56", None, None, false, true).unwrap(),
            "壹佰元伍角陆分"
        );

        assert_eq!(
            amount_to_upper("100.50", None, None, false, true).unwrap(),
            "壹佰元伍角整"
        );
        assert_eq!(
            amount_to_upper("100.50", None, None, false, false).unwrap(),
            "壹佰元伍角"
        );
    }

    #[test]
    fn test_invalid_amount() {
        assert!(amount_to_upper("abc", None, None, true, false).is_err());
        assert!(amount_to_upper("1.234", None, None, true, false).is_err());
        assert!(amount_to_upper("", None, None, true, false).is_err());
        assert!(amount_to_upper("1.2.3", None, None, true, false).is_err());
    }

    #[test]
    fn test_upper_to_amount() {
        let cases = [
            ("壹仟贰佰叁拾肆元伍角陆分", "1234.56"),
            ("壹仟肆佰零玖元伍角", "1409.50"),
            ("壹万陆仟肆佰零玖元零贰分", "16409.02"),
            ("零元整", "0.00"),
            ("壹万元整", "10000.00"),
            ("壹拾贰万叁仟肆佰伍拾陆元整", "123456.00"),
            ("壹亿零壹元整", "100000001.00"),
            ("壹万亿零壹元整", "1000000000001.00"),
            ("负壹佰元整", "-100.00"),
            ("人民币壹佰贰拾叁元肆角伍分", "123.45"),
            ("一千二百三十四元五角六分", "1234.56"),
            ("壹拾元", "10.00"),
            ("伍角陆分", "0.56"),
            ("壹佰贰拾叁圆正", "123.00"),
        ];
        for (input, expect) in cases {
            let result = upper_to_amount(input, None, None, true).unwrap();
            assert_eq!(result.amount, expect, "输入：{input}");
        }

        let result = upper_to_amount("壹佰元", None, None, false).unwrap();
        assert_eq!(result.amount, "100.00");
        assert_eq!(result.upper, "壹佰元");
    }

    #[test]
    fn test_round_trip() {
        for amount in [
            "0.00",
            "0.05",
            "0.50",
            "1.00",
            "10.00",
            "100.00",
            "1001.10",
            "100000.01",
            "1234567890.12",
            "1000000000001.01",
        ] {
            let upper = amount_to_upper(amount, None, None, true, false).unwrap();
            let back = upper_to_amount(&upper, None, None, true).unwrap();
            assert_eq!(back.amount, amount, "大写：{upper}");
        }
    }

    #[test]
    fn test_group_thousands() {
        assert_eq!(group_thousands("1234.56"), "1,234.56");
        assert_eq!(group_thousands("-1234567.00"), "-1,234,567.00");
        assert_eq!(group_thousands("0.00"), "0.00");
    }
}

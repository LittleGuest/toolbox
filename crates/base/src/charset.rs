use anyhow::{Error, Result};
use encoding_rs::{Encoding, GBK, UTF_8, UTF_16BE, UTF_16LE};
use serde::{Deserialize, Serialize};

// 字符集编码函数
pub fn encode_string(data: &str, charset: &str) -> Result<Vec<u8>> {
    let encoding = match charset {
        "UTF-8" => UTF_8,
        "GBK" => GBK,
        "UTF-16BE" => UTF_16BE,
        "UTF-16LE" => UTF_16LE,
        "UTF-32BE" => Encoding::for_label(b"UTF-32BE")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        "UTF-32LE" => Encoding::for_label(b"UTF-32LE")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        "ASCII" => Encoding::for_label(b"ASCII")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        _ => UTF_8,
    };

    let (encoded, _, _) = encoding.encode(data);
    Ok(encoded.into_owned())
}

// 字符集解码函数
pub fn decode_bytes(data: &[u8], charset: &str) -> Result<String> {
    let encoding = match charset {
        "UTF-8" => UTF_8,
        "GBK" => GBK,
        "UTF-16BE" => UTF_16BE,
        "UTF-16LE" => UTF_16LE,
        "UTF-32BE" => Encoding::for_label(b"UTF-32BE")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        "UTF-32LE" => Encoding::for_label(b"UTF-32LE")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        "ASCII" => Encoding::for_label(b"ASCII")
            .ok_or_else(|| Error::msg("Unsupported encoding".to_string()))?,
        _ => UTF_8,
    };

    let (decoded, _, _) = encoding.decode(data);
    Ok(decoded.into_owned())
}

// 从不同进制字符串解析为字节向量
pub fn parse_bytes_from_string(input: &str, input_type: &str) -> Result<Vec<u8>> {
    let normalized_input = input.replace(|c: char| !c.is_ascii_alphanumeric(), "");

    match input_type {
        "hex" => {
            if normalized_input.len() % 2 != 0 {
                return Err(Error::msg("Hex string must have even length".to_string()));
            }

            let mut bytes = Vec::new();
            for i in (0..normalized_input.len()).step_by(2) {
                let byte = u8::from_str_radix(&normalized_input[i..i + 2], 16)
                    .map_err(|e| Error::msg(e.to_string()))?;
                bytes.push(byte);
            }
            Ok(bytes)
        }
        "decimal" => {
            let parts: Vec<&str> = input
                .split(|c| c == ' ' || c == ',' || c == ';' || c == '\n')
                .filter(|s| !s.is_empty())
                .collect();
            let mut bytes = Vec::new();

            for part in parts {
                let byte = u8::from_str_radix(part, 10).map_err(|e| Error::msg(e.to_string()))?;
                bytes.push(byte);
            }
            Ok(bytes)
        }
        "octal" => {
            let parts: Vec<&str> = input
                .split(|c| c == ' ' || c == ',' || c == ';' || c == '\n')
                .filter(|s| !s.is_empty())
                .collect();
            let mut bytes = Vec::new();

            for part in parts {
                let cleaned_part = part.trim_start_matches('0');
                let cleaned_part = if cleaned_part.is_empty() {
                    "0"
                } else {
                    cleaned_part
                };
                let byte =
                    u8::from_str_radix(cleaned_part, 8).map_err(|e| Error::msg(e.to_string()))?;
                bytes.push(byte);
            }
            Ok(bytes)
        }
        "binary" => {
            let normalized_input = normalized_input.replace(" ", "").replace("\n", "");
            if normalized_input.len() % 8 != 0 {
                return Err(Error::msg(
                    "Binary string must be multiple of 8 bits".to_string(),
                ));
            }

            let mut bytes = Vec::new();
            for i in (0..normalized_input.len()).step_by(8) {
                let byte = u8::from_str_radix(&normalized_input[i..i + 8], 2)
                    .map_err(|e| Error::msg(e.to_string()))?;
                bytes.push(byte);
            }
            Ok(bytes)
        }
        "text" | _ => Ok(input.as_bytes().to_vec()),
    }
}

// 将字节向量格式化为不同进制的字符串
pub fn format_bytes_to_string(
    bytes: &[u8],
    output_type: &str,
    delimiter: &str,
    base_format: &str,
) -> Result<String> {
    match output_type {
        "hex" => {
            let formatted = bytes
                .iter()
                .map(|b| match base_format {
                    "0x" => format!("0x{:02X}", b),
                    "h" => format!("{:02X}h", b),
                    _ => format!("{:02X}", b),
                })
                .collect::<Vec<String>>()
                .join(delimiter);
            Ok(formatted)
        }
        "decimal" => {
            let formatted = bytes
                .iter()
                .map(|b| format!("{}", b))
                .collect::<Vec<String>>()
                .join(delimiter);
            Ok(formatted)
        }
        "octal" => {
            let formatted = bytes
                .iter()
                .map(|b| match base_format {
                    "0o" => format!("0o{:03o}", b),
                    _ => format!("{:03o}", b),
                })
                .collect::<Vec<String>>()
                .join(delimiter);
            Ok(formatted)
        }
        "binary" => {
            let formatted = bytes
                .iter()
                .map(|b| match base_format {
                    "0b" => format!("0b{:08b}", b),
                    _ => format!("{:08b}", b),
                })
                .collect::<Vec<String>>()
                .join(delimiter);
            Ok(formatted)
        }
        _ => Err(Error::msg("Unsupported output type".to_string())),
    }
}

// 自动检测字符编码
pub fn auto_detect_charset(data: &str) -> Result<String> {
    let bytes = data.as_bytes();

    // 检测UTF-8
    if is_valid_utf8(bytes) {
        return Ok("UTF-8".to_string());
    }

    // 检测UTF-16
    if bytes.len() >= 2 {
        // UTF-16 BE BOM
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return Ok("UTF-16BE".to_string());
        }
        // UTF-16 LE BOM
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return Ok("UTF-16LE".to_string());
        }
        // 尝试检测无BOM的UTF-16
        if bytes.len() % 2 == 0 {
            let mut has_nulls = false;
            let mut has_non_ascii = false;

            // 检查偶数位置是否有大量null字节（UTF-16 BE）
            let mut be_null_count = 0;
            for i in (0..bytes.len()).step_by(2) {
                if bytes[i] == 0 {
                    be_null_count += 1;
                }
                if bytes[i] > 127 || bytes[i + 1] > 127 {
                    has_non_ascii = true;
                }
            }

            // 检查奇数位置是否有大量null字节（UTF-16 LE）
            let mut le_null_count = 0;
            for i in (1..bytes.len()).step_by(2) {
                if bytes[i] == 0 {
                    le_null_count += 1;
                }
            }

            has_nulls = be_null_count > bytes.len() / 4 || le_null_count > bytes.len() / 4;

            if has_nulls && has_non_ascii {
                if be_null_count > le_null_count {
                    return Ok("UTF-16BE".to_string());
                } else {
                    return Ok("UTF-16LE".to_string());
                }
            }
        }
    }

    // 检测UTF-32
    if bytes.len() >= 4 {
        // UTF-32 BE BOM
        if bytes[0] == 0x00 && bytes[1] == 0x00 && bytes[2] == 0xFE && bytes[3] == 0xFF {
            return Ok("UTF-32BE".to_string());
        }
        // UTF-32 LE BOM
        if bytes[0] == 0xFF && bytes[1] == 0xFE && bytes[2] == 0x00 && bytes[3] == 0x00 {
            return Ok("UTF-32LE".to_string());
        }
    }

    // 检测GBK
    if is_valid_gbk(bytes) {
        return Ok("GBK".to_string());
    }

    // 检测ASCII
    let mut is_ascii = true;
    for &b in bytes {
        if b > 127 {
            is_ascii = false;
            break;
        }
    }
    if is_ascii {
        return Ok("ASCII".to_string());
    }

    // 默认返回UTF-8
    Ok("UTF-8".to_string())
}

// 检查是否为有效的UTF-8
fn is_valid_utf8(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if b <= 0x7F {
            // 1字节字符
            i += 1;
        } else if b >= 0xC0 && b <= 0xDF {
            // 2字节字符
            if i + 1 >= bytes.len() || (bytes[i + 1] & 0xC0) != 0x80 {
                return false;
            }
            i += 2;
        } else if b >= 0xE0 && b <= 0xEF {
            // 3字节字符
            if i + 2 >= bytes.len()
                || (bytes[i + 1] & 0xC0) != 0x80
                || (bytes[i + 2] & 0xC0) != 0x80
            {
                return false;
            }
            i += 3;
        } else if b >= 0xF0 && b <= 0xF7 {
            // 4字节字符
            if i + 3 >= bytes.len()
                || (bytes[i + 1] & 0xC0) != 0x80
                || (bytes[i + 2] & 0xC0) != 0x80
                || (bytes[i + 3] & 0xC0) != 0x80
            {
                return false;
            }
            i += 4;
        } else {
            return false;
        }
    }
    true
}

// 检查是否可能是GBK
fn is_valid_gbk(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if b <= 0x7F {
            // ASCII字符
            i += 1;
        } else {
            // GBK双字节字符
            if i + 1 >= bytes.len() {
                return false;
            }

            let b2 = bytes[i + 1];

            // GBK编码范围检查
            if (b >= 0x81 && b <= 0xFE)
                && ((b2 >= 0x40 && b2 <= 0x7E) || (b2 >= 0x80 && b2 <= 0xFE))
            {
                i += 2;
            } else {
                return false;
            }
        }
    }
    true
}

// 格式化为C/C++数组
pub fn format_as_c_array(bytes: &[u8]) -> String {
    let formatted = bytes
        .iter()
        .map(|b| format!("0x{:02X}", b))
        .collect::<Vec<String>>()
        .join(", ");
    format!(
        "unsigned char data[] = {{{}}};\nunsigned int data_length = {};",
        formatted,
        bytes.len()
    )
}

// 格式化为汇编数据
pub fn format_as_assembly(bytes: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        result.push_str(&format!("data_{:04X}:\t", i * 16));

        let hex_part = chunk
            .iter()
            .map(|b| format!("0x{:02X}", b))
            .collect::<Vec<String>>()
            .join(", ");
        result.push_str(&hex_part);

        // 添加ASCII表示
        result.push_str(&format!("\t; "));
        for b in chunk {
            if b.is_ascii_graphic() || *b == b' ' {
                result.push(*b as char);
            } else {
                result.push('.');
            }
        }

        result.push_str("\n");
    }
    result
}

// 反转不可打印字符
pub fn invert_non_printable(data: &[u8]) -> Vec<u8> {
    data.iter()
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                *b
            } else {
                0xFF - b
            }
        })
        .collect()
}

const CHARSETS: [&str; 9] = [
    "UTF-8",
    "GBK",
    "UTF-16BE",
    "UTF-16LE",
    "UTF-32BE",
    "UTF-32LE",
    "Shift_JIS",
    "EUC-JP",
    "ISO-8859-1",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverGarbledCode {
    pub source_charset: String,
    pub target_charset: String,
    pub recovered_text: String,
    pub score: f64,
}

// 乱码恢复 - 尝试不同的编码组合
pub fn recover_garbled_code(input: &str) -> Vec<RecoverGarbledCode> {
    let mut results = Vec::new();

    // 乱码文本本身是UTF-8编码的字符串
    // 我们需要尝试将其转换为字节，然后用不同的编码解码

    // 方法1：直接用UTF-8编码乱码文本，然后用各种编码解码
    // 这适用于：原始字节被错误地当作UTF-8解码的情况
    let garbled_bytes = input.as_bytes().to_vec();

    for &charset in &CHARSETS {
        if let Ok(decoded) = decode_bytes(&garbled_bytes, charset) {
            if is_readable_text(&decoded) {
                let exists = results
                    .iter()
                    .any(|(s, t, r)| s == "UTF-8" && t == charset && r == &decoded);

                if !exists {
                    results.push(("UTF-8".to_string(), charset.to_string(), decoded));
                }
            }
        }
    }

    // 方法2：尝试更复杂的转换
    // 假设原始文本用编码A编码成字节，然后被错误地用编码B解码，现在显示为乱码
    // 恢复方法：乱码文本用编码B编码回字节，然后用编码A解码

    for &wrong_charset in &CHARSETS {
        for &correct_charset in &CHARSETS {
            if wrong_charset == correct_charset {
                continue;
            }

            // 将乱码文本用错误的编码（wrong_charset）编码回字节
            if let Ok(recovered_bytes) = encode_string(input, wrong_charset) {
                // 再用正确的编码（correct_charset）解码
                if let Ok(decoded) = decode_bytes(&recovered_bytes, correct_charset) {
                    if is_readable_text(&decoded) {
                        let exists = results.iter().any(|(s, t, r)| {
                            s == wrong_charset && t == correct_charset && r == &decoded
                        });

                        if !exists {
                            results.push((
                                wrong_charset.to_string(),
                                correct_charset.to_string(),
                                decoded,
                            ));
                        }
                    }
                }

                // 方法3：特殊处理GBK编码的乱码文本
                // 如果用GBK编码后得到包含UTF-8 BOM的字节，尝试去掉BOM后用UTF-8解码
                if wrong_charset == "GBK" && correct_charset == "UTF-8" {
                    if recovered_bytes.len() > 3
                        && recovered_bytes[0] == 0xEF
                        && recovered_bytes[1] == 0xBB
                        && recovered_bytes[2] == 0xBF
                    {
                        let bytes_without_bom = &recovered_bytes[3..];

                        // 尝试直接用UTF-8解码
                        if let Ok(decoded) = decode_bytes(bytes_without_bom, "UTF-8") {
                            if is_readable_text(&decoded) {
                                let exists = results.iter().any(|(s, t, r)| {
                                    s == "GBK (without BOM)" && t == "UTF-8" && r == &decoded
                                });

                                if !exists {
                                    results.push((
                                        "GBK (without BOM)".to_string(),
                                        "UTF-8".to_string(),
                                        decoded,
                                    ));
                                }
                            }
                        }

                        // 方法3b：尝试修复无效字节
                        // 检测并修复重复的字节序列
                        if bytes_without_bom.len() >= 3 {
                            let fixed_bytes = try_fix_duplicate_bytes(bytes_without_bom);

                            if let Ok(decoded) = decode_bytes(&fixed_bytes, "UTF-8") {
                                if is_readable_text(&decoded) {
                                    let exists = results.iter().any(|(s, t, r)| {
                                        s == "GBK (fixed)" && t == "UTF-8" && r == &decoded
                                    });

                                    if !exists {
                                        results.push((
                                            "GBK (fixed)".to_string(),
                                            "UTF-8".to_string(),
                                            decoded,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut results_with_scores: Vec<RecoverGarbledCode> = results
        .into_iter()
        .map(|(s, t, r)| {
            let score = calculate_text_score(&r);
            RecoverGarbledCode {
                source_charset: s,
                target_charset: t,
                recovered_text: r,
                score,
            }
        })
        .collect();

    results_with_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    results_with_scores
}

fn calculate_text_score(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let (readable_count, total_count) = text.chars().fold((0, 0), |(readable, total), c| {
        let is_readable = c.is_ascii_graphic()
            || c == ' '
            || (c >= '\u{4e00}' && c <= '\u{9fa5}')
            || (c >= '\u{3040}' && c <= '\u{309f}')
            || (c >= '\u{30a0}' && c <= '\u{30ff}')
            || r#"，。！？、；：''""（）》《》【】「」『』·…—￥$€£%&=+-*/|～<>{}"#.contains(c);

        (readable + is_readable as usize, total + 1)
    });

    let score = readable_count as f64 / total_count as f64;
    (score * 10000.0).round() / 10000.0
}

// 检查文本是否可读（包含足够多的可打印字符或中文字符）
fn is_readable_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    let readable_count = text
        .chars()
        .filter(|c| {
            // ASCII 可打印字符或空格
            c.is_ascii_graphic() || *c == ' '
        // 中文字符（Unicode 范围）
        || (*c >= '\u{4e00}' && *c <= '\u{9fa5}')
        // 日文平假名
        || (*c >= '\u{3040}' && *c <= '\u{309f}')
        // 日文片假名
        || (*c >= '\u{30a0}' && *c <= '\u{30ff}')
        // 常见标点符号
        || r#"，。！？、；：''""（）》《》【】「」『』·…—￥$€£%&=+-*/|～<>{}"# .contains(*c)
        })
        .count();

    let total_count = text.chars().count();

    // 至少60%的字符是可读的
    let ratio = readable_count as f64 / total_count as f64;
    ratio >= 0.6
}

// 尝试修复重复的字节序列
// 检测并修复UTF-8编码中的重复字节问题
fn try_fix_duplicate_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut fixed = bytes.to_vec();
    let mut i = 0;

    while i < fixed.len() - 1 {
        // 检测连续重复的字节（如0xE5 0xE5）
        if fixed[i] == fixed[i + 1] {
            let duplicate_byte = fixed[i];

            // 检查这是否是UTF-8 3字节字符的开始字节（0xE0-0xEF）
            if duplicate_byte >= 0xE0 && duplicate_byte <= 0xEF {
                // 尝试删除重复的字节
                let mut test_fixed = fixed.clone();
                test_fixed.remove(i + 1);

                // 验证修复后的字节序列是否是有效的UTF-8
                if is_valid_utf8(&test_fixed) {
                    // 尝试解码并检查是否可读
                    if let Ok(decoded) = decode_bytes(&test_fixed, "UTF-8") {
                        if is_readable_text(&decoded) {
                            fixed = test_fixed;
                            // 继续检查当前位置，不要跳过
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    fixed
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINAL_TEXT: &str = "路漫漫其修远兮，吾将上下而求索。";
    const ORIGINAL_TEXT_BIG5: &str = "路漫漫其修遠兮，吾將上下而求索。";

    #[test]
    fn test_encode_decode() -> Result<()> {
        let data = "Hello, 世界!";
        let encoded = encode_string(data, "UTF-8")?;
        let decoded = decode_bytes(&encoded, "UTF-8")?;
        assert_eq!(data, decoded);
        Ok(())
    }

    #[test]
    fn test_hex_conversion() -> Result<()> {
        let data = "Hello";
        let encoded = encode_string(data, "UTF-8")?;
        let hex_string = format_bytes_to_string(&encoded, "hex", " ", "")?;
        let parsed_bytes = parse_bytes_from_string(&hex_string, "hex")?;
        assert_eq!(encoded, parsed_bytes);
        Ok(())
    }

    #[test]
    fn test_recover_garbled_code_utf8_to_gbk() {
        // UTF-8解码 -> GBK编码
        let garbled = "璺极婕叾淇繙鍏紝鍚惧皢涓婁笅鑰屾眰绱€€�";
        let results = recover_garbled_code(&garbled);
        for item in &results {
            println!(
                "{} -> {}: {}",
                item.source_charset, item.target_charset, item.recovered_text
            );
        }
    }

    #[test]
    fn test_recover_garbled_code_gbk_to_utf8() {
        // GBK编码 -> UTF-8解码
        let garbled = "·��������Զ�⣬�Ὣ���¶�������";
        let results = recover_garbled_code(&garbled);
        for item in &results {
            println!(
                "{} -> {}: {}",
                item.source_charset, item.target_charset, item.recovered_text
            );
        }
    }
}

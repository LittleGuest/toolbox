use anyhow::{Error, Result};
use encoding_rs::{Encoding, GBK, UTF_8, UTF_16BE, UTF_16LE};
use serde::{Deserialize, Serialize};

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

pub fn auto_detect_charset(data: &str) -> Result<String> {
    let bytes = data.as_bytes();

    if is_valid_utf8(bytes) {
        return Ok("UTF-8".to_string());
    }

    if bytes.len() >= 2 {
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return Ok("UTF-16BE".to_string());
        }
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return Ok("UTF-16LE".to_string());
        }
        if bytes.len() % 2 == 0 {
            let mut has_nulls = false;
            let mut has_non_ascii = false;

            let mut be_null_count = 0;
            for i in (0..bytes.len()).step_by(2) {
                if bytes[i] == 0 {
                    be_null_count += 1;
                }
                if bytes[i] > 127 || bytes[i + 1] > 127 {
                    has_non_ascii = true;
                }
            }

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

    if bytes.len() >= 4 {
        if bytes[0] == 0x00 && bytes[1] == 0x00 && bytes[2] == 0xFE && bytes[3] == 0xFF {
            return Ok("UTF-32BE".to_string());
        }
        if bytes[0] == 0xFF && bytes[1] == 0xFE && bytes[2] == 0x00 && bytes[3] == 0x00 {
            return Ok("UTF-32LE".to_string());
        }
    }

    if is_valid_gbk(bytes) {
        return Ok("GBK".to_string());
    }

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

    Ok("UTF-8".to_string())
}

fn is_valid_utf8(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if b <= 0x7F {
            i += 1;
        } else if b >= 0xC0 && b <= 0xDF {
            if i + 1 >= bytes.len() || (bytes[i + 1] & 0xC0) != 0x80 {
                return false;
            }
            i += 2;
        } else if b >= 0xE0 && b <= 0xEF {
            if i + 2 >= bytes.len()
                || (bytes[i + 1] & 0xC0) != 0x80
                || (bytes[i + 2] & 0xC0) != 0x80
            {
                return false;
            }
            i += 3;
        } else if b >= 0xF0 && b <= 0xF7 {
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

fn is_valid_gbk(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if b <= 0x7F {
            i += 1;
        } else {
            if i + 1 >= bytes.len() {
                return false;
            }

            let b2 = bytes[i + 1];

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverGarbledCode {
    pub source_charset: String,
    pub target_charset: String,
    pub recovered_text: String,
    pub score: f64,
}

pub fn recover_garbled_code(input: &str) -> Vec<RecoverGarbledCode> {
    let mut results = Vec::new();

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

    for &wrong_charset in &CHARSETS {
        for &correct_charset in &CHARSETS {
            if wrong_charset == correct_charset {
                continue;
            }

            if let Ok(recovered_bytes) = encode_string(input, wrong_charset) {
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

                if wrong_charset == "GBK" && correct_charset == "UTF-8" {
                    if recovered_bytes.len() > 3
                        && recovered_bytes[0] == 0xEF
                        && recovered_bytes[1] == 0xBB
                        && recovered_bytes[2] == 0xBF
                    {
                        let bytes_without_bom = &recovered_bytes[3..];

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

fn is_readable_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    let readable_count = text
        .chars()
        .filter(|c| {
            c.is_ascii_graphic()
                || *c == ' '
                || (*c >= '\u{4e00}' && *c <= '\u{9fa5}')
                || (*c >= '\u{3040}' && *c <= '\u{309f}')
                || (*c >= '\u{30a0}' && *c <= '\u{30ff}')
                || r#"，。！？、；：''""（）》《》【】「」『』·…—￥$€£%&=+-*/|～<>{}"#.contains(*c)
        })
        .count();

    let total_count = text.chars().count();

    let ratio = readable_count as f64 / total_count as f64;
    ratio >= 0.6
}

fn try_fix_duplicate_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut fixed = bytes.to_vec();
    let mut i = 0;

    while i < fixed.len() - 1 {
        if fixed[i] == fixed[i + 1] {
            let duplicate_byte = fixed[i];

            if duplicate_byte >= 0xE0 && duplicate_byte <= 0xEF {
                let mut test_fixed = fixed.clone();
                test_fixed.remove(i + 1);

                if is_valid_utf8(&test_fixed) {
                    if let Ok(decoded) = decode_bytes(&test_fixed, "UTF-8") {
                        if is_readable_text(&decoded) {
                            fixed = test_fixed;
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

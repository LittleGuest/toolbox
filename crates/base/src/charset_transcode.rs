use anyhow::{Result, anyhow};
use encoding_rs::{BIG5, EUC_JP, GB18030, GBK, SHIFT_JIS, UTF_16BE, UTF_16LE};

pub const CHARSETS: &[&str] = &[
    "UTF-8",
    "GBK",
    "GB18030",
    "GB2312",
    "BIG5",
    "UTF-16LE",
    "UTF-16BE",
    "Shift_JIS",
    "EUC-JP",
    "ISO-8859-1",
    "Windows-1252",
    "KOI8-R",
];

pub struct TranscodeResult {
    pub text: String,
    pub source: String,
    pub target: String,
    pub byte_count: usize,
    pub lossy: bool,
    pub hex: String,
}

pub struct CharsetGuess {
    pub charset: String,
    pub text: String,
    pub ratio: f32,
}

fn encoding_of(charset: &str) -> Result<&'static encoding_rs::Encoding> {
    let enc = match charset {
        "UTF-8" => encoding_rs::UTF_8,
        "GBK" | "GB2312" => GBK,
        "GB18030" => GB18030,
        "BIG5" => BIG5,
        "UTF-16LE" => UTF_16LE,
        "UTF-16BE" => UTF_16BE,
        "Shift_JIS" => SHIFT_JIS,
        "EUC-JP" => EUC_JP,
        "ISO-8859-1" => encoding_rs::WINDOWS_1252,
        "Windows-1252" => encoding_rs::WINDOWS_1252,
        "KOI8-R" => encoding_rs::KOI8_R,
        other => return Err(anyhow!("不支持的字符集：{other}")),
    };
    Ok(enc)
}

fn bad_ratio(text: &str) -> f32 {
    let total = text.chars().count();
    if total == 0 {
        return 1.0;
    }
    let bad = text.chars().filter(|c| *c == '\u{FFFD}').count();
    1.0 - bad as f32 / total as f32
}

pub fn decode_with(bytes: &[u8], charset: &str) -> Result<String> {
    let enc = encoding_of(charset)?;
    let (text, _, err) = enc.decode(bytes);
    if err {
        return Err(anyhow!("按 {charset} 解码时遇到非法字节序列，请确认源编码"));
    }
    Ok(text.into_owned())
}

pub fn encode_with(text: &str, charset: &str) -> Result<Vec<u8>> {
    let enc = encoding_of(charset)?;
    let (bytes, _, err) = enc.encode(text);
    if err {
        return Err(anyhow!("存在无法用 {charset} 表示的字符"));
    }
    Ok(bytes.into_owned())
}

pub fn transcode(input: &str, source: &str, target: &str) -> Result<TranscodeResult> {
    let text = decode_with(input.as_bytes(), source)?;
    if source == target {
        return Ok(TranscodeResult {
            text: input.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            byte_count: input.len(),
            lossy: false,
            hex: hex_bytes(input.as_bytes()),
        });
    }
    let bytes = encode_with(&text, target)?;
    let round = decode_with(&bytes, target).unwrap_or_else(|_| text.clone());
    let lossy = round != text;
    Ok(TranscodeResult {
        text: round,
        source: source.to_string(),
        target: target.to_string(),
        byte_count: bytes.len(),
        lossy,
        hex: hex_bytes(&bytes),
    })
}

pub fn transcode_from_bytes(bytes: &[u8], source: &str, target: &str) -> Result<TranscodeResult> {
    let text = decode_with(bytes, source)?;
    let out = encode_with(&text, target)?;
    let round = decode_with(&out, target).unwrap_or_else(|_| text.clone());
    let lossy = round != text;
    Ok(TranscodeResult {
        text: round,
        source: source.to_string(),
        target: target.to_string(),
        byte_count: out.len(),
        lossy,
        hex: hex_bytes(&out),
    })
}

pub fn detect(input: &str) -> Vec<CharsetGuess> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    for cs in CHARSETS {
        let enc = match encoding_of(cs) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let (text, _, err) = enc.decode(bytes);
        let ratio = if err { bad_ratio(&text) * 0.4 } else { 1.0 };
        out.push(CharsetGuess {
            charset: (*cs).to_string(),
            text: text.into_owned(),
            ratio,
        });
    }
    out.sort_by(|a, b| {
        b.ratio
            .partial_cmp(&a.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

pub fn best_guess(input: &str) -> Option<String> {
    detect(input).into_iter().next().map(|g| g.charset)
}

pub fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_hex(text: &str) -> Result<Vec<u8>> {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != 'x' && *c != 'X')
        .collect();
    if compact.is_empty() {
        return Ok(Vec::new());
    }
    let target = if compact.len() % 2 == 1 {
        format!("0{compact}")
    } else {
        compact.clone()
    };
    hex::decode(&target).map_err(|_| anyhow!("输入不是有效的十六进制"))
}

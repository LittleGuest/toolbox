//! URL编码解码

use super::{ToolError, ToolResult};

/// 编码URL
pub fn encode(data: &str) -> String {
    if data.is_empty() {
        return "".to_string();
    }

    let (f, s) = check(data);
    if let Some(s) = s {
        format!("{f}?{}", urlencoding::encode(s).into_owned())
    } else {
        urlencoding::encode(f).into_owned()
    }
}

/// 解码URL
pub fn decode(data: &str) -> ToolResult<String> {
    if data.is_empty() {
        return Ok("".to_string());
    }

    let (f, s) = check(data);
    if let Some(s) = s {
        Ok(format!(
            "{f}?{}",
            urlencoding::decode(s)
                .map_err(|e| ToolError::UrlErr(e.to_string()))?
                .into_owned()
        ))
    } else {
        Ok(urlencoding::decode(f)
            .map_err(|e| ToolError::UrlErr(e.to_string()))?
            .into_owned())
    }
}

fn check(data: &str) -> (&str, Option<&str>) {
    if data.starts_with("http") || data.starts_with("https") || data.contains('?') {
        if let Some(data) = data.split_once('?') {
            return (data.0, Some(data.1));
        }
    }
    (data, None)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(
            "This%20string%20will%20be%20URL%20encoded.",
            encode("This string will be URL encoded.")
        );

        assert_eq!(
            "https://crates.io/search?q%3Durlencoding",
            encode("https://crates.io/search?q=urlencoding")
        )
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            "👾 Exterminate!",
            decode("%F0%9F%91%BE%20Exterminate%21").unwrap()
        );
        assert_eq!(
            "https://crates.io/search?q=urlencoding",
            decode("https://crates.io/search?q%3Durlencoding").unwrap()
        )
    }
}

use crate::{Error, Result};

pub fn encode(data: &str) -> Result<String> {
    if data.is_empty() {
        return Ok(String::new());
    }

    let (f, s) = check(data);
    if let Some(s) = s {
        Ok(format!("{f}?{}", urlencoding::encode(s).into_owned()))
    } else {
        Ok(urlencoding::encode(f).into_owned())
    }
}

pub fn decode(data: &str) -> Result<String> {
    if data.is_empty() {
        return Ok(String::new());
    }

    let (f, s) = check(data);
    if let Some(s) = s {
        Ok(format!(
            "{f}?{}",
            urlencoding::decode(s)
                .map_err(|e| Error::E(e.to_string()))?
                .into_owned()
        ))
    } else {
        Ok(urlencoding::decode(f)
            .map_err(|e| Error::E(e.to_string()))?
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
mod tests {

    use super::*;

    #[test]
    fn test_encode() -> Result<()> {
        assert_eq!(
            "This%20string%20will%20be%20URL%20encoded.",
            encode("This string will be URL encoded.")?
        );
        assert_eq!(
            "https://crates.io/search?q%3Durlencoding",
            encode("https://crates.io/search?q=urlencoding")?
        );
        Ok(())
    }

    #[test]
    fn test_decode() -> Result<()> {
        assert_eq!("👾 Exterminate!", decode("%F0%9F%91%BE%20Exterminate%21")?);
        assert_eq!(
            "https://crates.io/search?q=urlencoding",
            decode("https://crates.io/search?q%3Durlencoding")?
        );
        Ok(())
    }
}

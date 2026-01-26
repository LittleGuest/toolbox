use anyhow::{Error, Result};

pub fn encode(data: &str) -> Result<String> {
    Ok(hex::encode(data))
}

pub fn decode(data: &str) -> Result<String> {
    let data = hex::decode(data).map_err(|e| Error::msg(e.to_string()))?;
    String::from_utf8(data).map_err(|e| Error::msg(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<()> {
        let data = "Hello world!";
        assert_eq!(data, decode(&encode(data)?)?);
        Ok(())
    }
}

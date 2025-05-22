//! 16进制

use crate::{Error, Result};

/// 16进制编码
pub fn encode(data: &str) -> String {
    hex::encode(data)
}

/// 16进制解码
pub fn decode(data: &str) -> Result<String> {
    let data = hex::decode(data).map_err(|e| Error::HexErr(e.to_string()))?;
    String::from_utf8(data).map_err(|e| Error::HexErr(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        let data = "Hello world!";
        assert_eq!(data, decode(&encode(data)).unwrap());
    }
}

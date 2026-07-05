use anyhow::Error;
use base64::Engine as _;
use serde_json::json;

use crate::Result;

pub fn decode(data: &str) -> Result<String> {
    let parts = data.split('.').collect::<Vec<_>>();
    if parts.len() < 2 {
        return Err(Error::msg("invalid jwt format"));
    }

    let header = decode_part(parts[0])?;
    let payload = decode_part(parts[1])?;
    let header = serde_json::from_slice::<serde_json::Value>(&header)?;
    let payload = serde_json::from_slice::<serde_json::Value>(&payload)?;

    Ok(serde_json::to_string_pretty(&json!({
        "header": header,
        "payload": payload,
    }))?)
}

fn decode_part(part: &str) -> Result<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(part)
        .map_err(|e| Error::msg(e.to_string()))
}

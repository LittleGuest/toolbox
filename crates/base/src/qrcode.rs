use anyhow::{Error, Result};
use qrcode_generator::{
    Renderer,
    qr::{Encoder, ErrorCorrection},
};

pub fn qrcode(d: &str) -> Result<String> {
    let symbol = Encoder::new(ErrorCorrection::Medium)
        .encode_bytes(d.as_bytes())
        .map_err(|e| Error::msg(e.to_string()))?;
    Renderer::new(&symbol, 512)
        .to_svg_string(Some(""))
        .map_err(|e| Error::msg(e.to_string()))
}

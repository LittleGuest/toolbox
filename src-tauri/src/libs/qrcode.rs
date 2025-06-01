use crate::{Error, Result};

pub fn qrcode(d: &str) -> Result<String> {
    qrcode_generator::to_svg_to_string(
        d.as_bytes(),
        qrcode_generator::QrCodeEcc::Medium,
        40,
        Some(""),
    )
    .map_err(|e| Error::E(e.to_string()))
}

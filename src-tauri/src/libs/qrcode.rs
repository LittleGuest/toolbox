//! 二维码生成器

pub fn qrcode(d: &str) -> String {
    qrcode_generator::to_svg_to_string(
        d.as_bytes(),
        qrcode_generator::QrCodeEcc::Medium,
        40,
        Some(""),
    )
    .unwrap_or("".to_string())
}

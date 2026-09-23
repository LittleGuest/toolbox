use std::collections::HashMap;

use anyhow::{Error, Result};
use base64::Engine as _;
use image::{ImageFormat, ImageReader};

pub fn encode_text(data: &str) -> Result<String> {
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(data))
}

pub fn decode_text(data: &str) -> Result<String> {
    String::from_utf8(
        base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(data)
            .map_err(|e| Error::msg(e.to_string()))?,
    )
    .map_err(|e| Error::msg(e.to_string()))
}

pub fn encode_img(path: &str) -> Result<String> {
    let image = ImageReader::open(path)
        .map_err(|e| Error::msg(e.to_string()))?
        .decode()
        .map_err(|e| Error::msg(e.to_string()))?;
    let hasher = image_hasher::HasherConfig::new().to_hasher();
    let hash = hasher.hash_image(&image);
    Ok(hash.to_base64())
}

pub fn decode_img(data: &str) -> Result<Vec<u8>> {
    Ok(image_hasher::ImageHash::<Box<[u8]>>::from_base64(data)
        .map_err(|e| Error::msg(format!("{e:?}")))?
        .as_bytes()
        .to_vec())
}

const STANDARD: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

fn mime_from_bytes(bytes: &[u8]) -> Option<String> {
    let format = image::guess_format(bytes).ok()?;
    Some(mime_from_format(format))
}

fn mime_from_format(format: ImageFormat) -> String {
    match format {
        ImageFormat::Png => "image/png".into(),
        ImageFormat::Jpeg => "image/jpeg".into(),
        ImageFormat::Gif => "image/gif".into(),
        ImageFormat::WebP => "image/webp".into(),
        ImageFormat::Bmp => "image/bmp".into(),
        ImageFormat::Tiff => "image/tiff".into(),
        ImageFormat::Ico => "image/x-icon".into(),
        ImageFormat::Avif => "image/avif".into(),
        ImageFormat::Pnm => "image/x-portable-anymap".into(),
        ImageFormat::Tga => "image/x-targa".into(),
        ImageFormat::Dds => "image/vnd.ms-dds".into(),
        ImageFormat::Qoi => "image/qoi".into(),
        ImageFormat::Farbfeld => "image/x-farbfeld".into(),
        ImageFormat::OpenExr => "image/x-exr".into(),
        ImageFormat::Hdr => "image/vnd.radiance".into(),
        _ => "application/octet-stream".into(),
    }
}

fn mime_from_ext(path: &str) -> Option<String> {
    let ext = std::path::Path::new(path)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    Some(
        match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "avif" => "image/avif",
            _ => return None,
        }
        .into(),
    )
}

pub fn encode_image(path: &str) -> Result<HashMap<String, String>> {
    let bytes = std::fs::read(path).map_err(|e| Error::msg(e.to_string()))?;
    let mime = mime_from_bytes(&bytes)
        .or_else(|| mime_from_ext(path))
        .unwrap_or_else(|| "application/octet-stream".into());

    if mime == "application/octet-stream" {
        return Err(Error::msg("所选文件不是有效的图片格式"));
    }

    let mut map = HashMap::with_capacity(3);
    map.insert("mime".to_string(), mime.clone());
    map.insert(
        "dataUrl".to_string(),
        format!("data:{};base64,{}", mime, STANDARD.encode(&bytes)),
    );
    map.insert("size".to_string(), bytes.len().to_string());
    Ok(map)
}

pub fn decode_image(data: &str) -> Result<HashMap<String, String>> {
    let (mime, bytes) = decode_image_bytes(data)?;
    let mut map = HashMap::with_capacity(3);
    map.insert("mime".to_string(), mime.clone());
    map.insert(
        "dataUrl".to_string(),
        format!("data:{};base64,{}", mime, STANDARD.encode(&bytes)),
    );
    map.insert("size".to_string(), bytes.len().to_string());
    Ok(map)
}

pub fn save_image(data: &str, path: &str) -> Result<()> {
    let (_, bytes) = decode_image_bytes(data)?;
    std::fs::write(path, &bytes).map_err(|e| Error::msg(e.to_string()))
}

fn decode_image_bytes(data: &str) -> Result<(String, Vec<u8>)> {
    let trimmed = data.trim();
    let (mime, b64) = if let Some((prefix, b64)) = trimmed.split_once(',') {
        if prefix.starts_with("data:") && prefix.ends_with(";base64") {
            let mime = prefix
                .trim_start_matches("data:")
                .trim_end_matches(";base64")
                .to_string();
            (Some(mime), b64)
        } else {
            (None, trimmed)
        }
    } else {
        (None, trimmed)
    };

    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|e| Error::msg(format!("Base64 解码失败: {e}")))?;

    let mime = match mime {
        Some(m) if !m.is_empty() && m != "application/octet-stream" => m,
        _ => mime_from_bytes(&bytes).unwrap_or_else(|| "application/octet-stream".into()),
    };

    if mime == "application/octet-stream" {
        return Err(Error::msg("解码结果不是有效的图片数据"));
    }

    Ok((mime, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_text() {
        let str = "hello rust";
        assert_eq!(str, decode_text(&encode_text(str).unwrap()).unwrap());
    }

    #[test]
    fn test_decode_data_url() {
        let (mime, bytes) = decode_image_bytes("data:image/png;base64,iVBORw0KGgo=").unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, STANDARD.decode("iVBORw0KGgo=").unwrap());
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_image_bytes("!!!not-base64!!!").is_err());
    }
}

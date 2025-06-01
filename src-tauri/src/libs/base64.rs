use base64::Engine as _;
use image::ImageReader;

use crate::{Error, Result};

pub fn encode_text(data: &str) -> Result<String> {
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(data))
}

pub fn decode_text(data: &str) -> Result<String> {
    String::from_utf8(
        base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(data)
            .map_err(|e| Error::E(e.to_string()))?,
    )
    .map_err(|e| Error::E(e.to_string()))
}

pub fn encode_img(path: &str) -> Result<String> {
    let image = ImageReader::open(path)
        .map_err(|e| Error::E(e.to_string()))?
        .decode()
        .map_err(|e| Error::E(e.to_string()))?;
    let hasher = image_hasher::HasherConfig::new().to_hasher();
    let hash = hasher.hash_image(&image);
    Ok(hash.to_base64())
}

pub fn decode_img(data: &str) -> Result<Vec<u8>> {
    Ok(image_hasher::ImageHash::<Box<[u8]>>::from_base64(data)
        .map_err(|e| Error::E(format!("{e:?}")))?
        .as_bytes()
        .to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_text() {
        let str = "hello rust";
        assert_eq!(str, decode_text(&encode_text(str).unwrap()).unwrap());
    }
}

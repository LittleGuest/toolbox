use std::{fs::File, io::Read};

use anyhow::{Error, Result};
use serde::Serialize;
use sha2::Digest;

#[derive(Debug, Serialize)]
pub struct Checksum {}

fn digest_file<D: Digest>(file: &mut File) -> Result<String> {
    let mut hasher = D::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

impl Checksum {
    pub async fn sum(r#type: &str, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path)?;
        let checksum = match r#type {
            "md5sum" => digest_file::<md5::Md5>(&mut file)?,
            "sha1sum" => digest_file::<sha1::Sha1>(&mut file)?,
            "sha2_224sum" => digest_file::<sha2::Sha224>(&mut file)?,
            "sha2_256sum" => digest_file::<sha2::Sha256>(&mut file)?,
            "sha2_384sum" => digest_file::<sha2::Sha384>(&mut file)?,
            "sha2_512sum" => digest_file::<sha2::Sha512>(&mut file)?,
            "sha3_256sum" => digest_file::<sha3::Sha3_256>(&mut file)?,
            "sha3_384sum" => digest_file::<sha3::Sha3_384>(&mut file)?,
            "sha3_512sum" => digest_file::<sha3::Sha3_512>(&mut file)?,
            _ => {
                return Err(Error::msg("不支持的算法".to_string()));
            }
        };
        Ok(checksum)
    }
}

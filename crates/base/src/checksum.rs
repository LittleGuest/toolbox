use std::{collections::HashMap, fs::File, io, sync::LazyLock};

use anyhow::{Error, Result};
use dashmap::DashMap;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Checksum {}

impl Checksum {
    pub async fn sum(r#type: &str, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path)?;
        let checksum = match r#type {
            "md5sum" => {
                use md5::Digest;
                let mut md5_hasher = md5::Md5::new();
                io::copy(&mut file, &mut md5_hasher)?;
                format!("{:x}", md5_hasher.finalize())
            }
            "sha1sum" => {
                use sha1::Digest;
                let mut sha1_hasher = sha1::Sha1::new();
                io::copy(&mut file, &mut sha1_hasher)?;
                format!("{:x}", sha1_hasher.finalize())
            }
            "sha2_224sum" => {
                use sha2::Digest;
                let mut sha2_224_hasher = sha2::Sha224::new();
                io::copy(&mut file, &mut sha2_224_hasher)?;
                format!("{:x}", sha2_224_hasher.finalize())
            }
            "sha2_256sum" => {
                use sha2::Digest;
                let mut sha256_hasher = sha2::Sha256::new();
                io::copy(&mut file, &mut sha256_hasher)?;
                format!("{:x}", sha256_hasher.finalize())
            }
            "sha2_384sum" => {
                use sha2::Digest;
                let mut sha2_384_hasher = sha2::Sha384::new();
                io::copy(&mut file, &mut sha2_384_hasher)?;
                format!("{:x}", sha2_384_hasher.finalize())
            }
            "sha2_512sum" => {
                use sha2::Digest;
                let mut sha512_hasher = sha2::Sha512::new();
                io::copy(&mut file, &mut sha512_hasher)?;
                format!("{:x}", sha512_hasher.finalize())
            }
            "sha3_256sum" => {
                use sha3::Digest;
                let mut sha3_256_hasher = sha3::Sha3_256::new();
                io::copy(&mut file, &mut sha3_256_hasher)?;
                format!("{:x}", sha3_256_hasher.finalize())
            }
            "sha3_384sum" => {
                use sha3::Digest;
                let mut sha3_384_hasher = sha3::Sha3_384::new();
                io::copy(&mut file, &mut sha3_384_hasher)?;
                format!("{:x}", sha3_384_hasher.finalize())
            }
            "sha3_512sum" => {
                use sha3::Digest;
                let mut sha3_512_hasher = sha3::Sha3_512::new();
                io::copy(&mut file, &mut sha3_512_hasher)?;
                format!("{:x}", sha3_512_hasher.finalize())
            }
            _ => {
                return Err(Error::msg("不支持的算法".to_string()));
            }
        };
        Ok(checksum)
    }
}

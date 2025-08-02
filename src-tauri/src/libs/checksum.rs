use std::{collections::HashMap, sync::LazyLock};

use dashmap::DashMap;
use serde::Serialize;
use sha3::Digest;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{Error, Result};

/// 分块大小
static CHUNK_SIZE: usize = 1024 * 1024 * 2;
/// 分块缓存，key，文件名，value，分块数据
static CHUNK_MAP: LazyLock<DashMap<String, Vec<u8>>> = LazyLock::new(DashMap::new);

pub struct ChunkMeta {
    /// 文件名
    pub filename: String,
    /// 分块索引
    pub chunk_index: usize,
    /// 分块数据
    pub chunk_data: Vec<u8>,
    /// 总分块数
    pub total_chunks: usize,
}

#[derive(Debug, Serialize)]
pub struct Checksum {}

impl Checksum {
    pub async fn sum(r#type: &str, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path).await?;
        // let metadata = file.metadata().await?;
        // let file_size = metadata.len();
        let mut buffer = [0; 128 * 1024];
        let checksum = match r#type {
            "md5sum" => {
                let mut md5_hasher = md5::Md5::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    md5_hasher.update(&buffer[..n]);
                }
                format!("{:x}", md5_hasher.finalize())
            }
            "sha1sum" => {
                let mut sha1_hasher = sha1::Sha1::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha1_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha1_hasher.finalize())
            }
            "sha2_224sum" => {
                let mut sha2_224_hasher = sha2::Sha224::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha2_224_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha2_224_hasher.finalize())
            }
            "sha2_256sum" => {
                let mut sha256_hasher = sha2::Sha256::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha256_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha256_hasher.finalize())
            }
            "sha2_384sum" => {
                let mut sha2_384_hasher = sha2::Sha384::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha2_384_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha2_384_hasher.finalize())
            }
            "sha2_512sum" => {
                let mut sha512_hasher = sha2::Sha512::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha512_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha512_hasher.finalize())
            }
            "sha3_256sum" => {
                let mut sha3_256_hasher = sha3::Sha3_256::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha3_256_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha3_256_hasher.finalize())
            }
            "sha3_384sum" => {
                let mut sha3_384_hasher = sha3::Sha3_384::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha3_384_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha3_384_hasher.finalize())
            }
            "sha3_512sum" => {
                let mut sha3_512_hasher = sha3::Sha3_512::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    sha3_512_hasher.update(&buffer[..n]);
                }
                format!("{:x}", sha3_512_hasher.finalize())
            }
            _ => {
                return Err(Error::E("不支持的算法".into()));
            }
        };
        Ok(checksum)
    }
}

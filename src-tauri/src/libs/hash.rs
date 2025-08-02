use sha1::Digest;

use crate::{Error, Result};

pub async fn md5(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = md5::Md5::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha1(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha1::Sha1::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha256(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha512(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha2::Sha512::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha2_224(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha2::Sha224::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha2_384(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha2::Sha384::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha3_256(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha3_384(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha3::Sha3_384::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn sha3_512(data: impl AsRef<[u8]>) -> Result<String> {
    let mut hasher = sha3::Sha3_512::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_md5() -> Result<()> {
        assert_eq!(
            "5eb63bbbe01eeed093cb22bb8f5acdc3",
            md5("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha1() -> Result<()> {
        assert_eq!(
            "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed",
            sha1("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha256() -> Result<()> {
        assert_eq!(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            sha256("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha512() -> Result<()> {
        assert_eq!(
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f",
            sha512("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha2_224() -> Result<()> {
        assert_eq!(
            "2f05477fc24bb4faefd86517156dafdecec45b8ad3cf2522a563582b",
            sha2_224("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha2_384() -> Result<()> {
        assert_eq!(
            "fdbd8e75a67f29f701a4e040385e2e23986303ea10239211af907fcbb83578b3e417cb71ce646efd0819dd8c088de1bd",
            sha2_384("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha3_256() -> Result<()> {
        assert_eq!(
            "644bcc7e564373040999aac89e7622f3ca71fba1d972fd94a31c3bfbf24e3938",
            sha3_256("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha3_384() -> Result<()> {
        assert_eq!(
            "83bff28dde1b1bf5810071c6643c08e5b05bdb836effd70b403ea8ea0a634dc4997eb1053aa3593f590f9c63630dd90b",
            sha3_384("hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sha3_512() -> Result<()> {
        assert_eq!(
            "840006653e9ac9e95117a15c915caab81662918e925de9e004f774ff82d7079a40d4d27b1b372657c61d46d470304c88c788b3a4527ad074d1dccbee5dbaa99a",
            sha3_512("hello world").await?
        );
        Ok(())
    }
}

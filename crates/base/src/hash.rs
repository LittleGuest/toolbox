use crate::{Error, Result};

pub async fn md5(data: impl AsRef<[u8]>) -> Result<String> {
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha1(data: impl AsRef<[u8]>) -> Result<String> {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha256(data: impl AsRef<[u8]>) -> Result<String> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha512(data: impl AsRef<[u8]>) -> Result<String> {
    use sha2::Digest;
    let mut hasher = sha2::Sha512::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha2_224(data: impl AsRef<[u8]>) -> Result<String> {
    use sha2::Digest;
    let mut hasher = sha2::Sha224::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha2_384(data: impl AsRef<[u8]>) -> Result<String> {
    use sha2::Digest;
    let mut hasher = sha2::Sha384::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha3_256(data: impl AsRef<[u8]>) -> Result<String> {
    use sha3::Digest;
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha3_384(data: impl AsRef<[u8]>) -> Result<String> {
    use sha3::Digest;
    let mut hasher = sha3::Sha3_384::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha3_512(data: impl AsRef<[u8]>) -> Result<String> {
    use sha3::Digest;
    let mut hasher = sha3::Sha3_512::new();
    hasher.update(data);
    Ok(hex::encode(hasher.finalize()))
}

pub async fn hmac_md5(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{Hmac, KeyInit, Mac};
    type HmacMd5 = Hmac<md5::Md5>;
    let mut mac = HmacMd5::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub async fn hmac_sha1(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{Hmac, KeyInit, Mac};
    type HmacSha1 = Hmac<sha1::Sha1>;
    let mut mac = HmacSha1::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub async fn hmac_sha256(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{Hmac, KeyInit, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub async fn hmac_sha512(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{Hmac, KeyInit, Mac};
    type HmacSha512 = Hmac<sha2::Sha512>;
    let mut mac = HmacSha512::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub async fn hmac_sha3_256(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{KeyInit, Mac, SimpleHmac};
    type HmacSha3_256 = SimpleHmac<sha3::Sha3_256>;
    let mut mac = HmacSha3_256::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub async fn hmac_sha3_512(secret: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<String> {
    use hmac::{KeyInit, Mac, SimpleHmac};
    type HmacSha3_512 = SimpleHmac<sha3::Sha3_512>;
    let mut mac = HmacSha3_512::new_from_slice(secret.as_ref()).map_err(Error::from)?;
    mac.update(data.as_ref());
    Ok(hex::encode(mac.finalize().into_bytes()))
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

    #[tokio::test]
    async fn test_hmac_md5() -> Result<()> {
        assert_eq!(
            "ae92cf51adf91130130aefc2b39a7595",
            hmac_md5("key", "hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hmac_sha1() -> Result<()> {
        assert_eq!(
            "34dd234b92683593560528f6193ea68c8005f615",
            hmac_sha1("key", "hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hmac_sha256() -> Result<()> {
        assert_eq!(
            "0ba06f1f9a6300461e43454535dc3c4223e47b1d357073d7536eae90ec095be1",
            hmac_sha256("key", "hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hmac_sha512() -> Result<()> {
        assert_eq!(
            "ea0625a5ff1cd1653a327f8a4ae2f478fc51405c73ddac3a8a05a7a810310a6a14d7c8b4d284013493a6016ecadc772cfd98ed6cbe745949c5e6119fafb63b54",
            hmac_sha512("key", "hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hmac_sha3_256() -> Result<()> {
        assert_eq!(
            "fafb52a338cd0f02333d8f752b4b10383655bfff80d5c153494428d48b07d3d2",
            hmac_sha3_256("key", "hello world").await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hmac_sha3_512() -> Result<()> {
        assert_eq!(
            "98e1e412df54cf7e9fe5a9ebbc624d1b1603cff55d3bd2069bad12585d3500577f530d4d6accf0bacf6cc9a7d2904d562735720d95d3640c09de0ed8af07362f",
            hmac_sha3_512("key", "hello world").await?
        );
        Ok(())
    }
}

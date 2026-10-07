use anyhow::{Result, anyhow};
use sm2::{
    SecretKey,
    dsa::{
        Signature, SigningKey, VerifyingKey,
        signature::{Signer, Verifier},
    },
    elliptic_curve::{Generate, common::getrandom::SysRng},
    pke::{DecryptingKey, EncryptingKey},
};
use sm3::{Digest, Sm3};
use sm4::{
    Sm4,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit, array::Array},
};

pub const DEFAULT_DISTID: &str = "1234567812345678";

pub fn sm3_hex(data: &[u8]) -> String {
    let mut hasher = Sm3::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sm3_hmac_hex(key: &[u8], data: &[u8]) -> String {
    const BLOCK: usize = 64;
    let mut k = key.to_vec();
    if k.len() > BLOCK {
        let mut h = Sm3::new();
        h.update(&k);
        k = h.finalize().to_vec();
    }
    k.resize(BLOCK, 0);

    let mut ipad = Vec::with_capacity(BLOCK + data.len());
    let mut opad: Vec<u8> = Vec::with_capacity(BLOCK + 32);
    for b in &k {
        ipad.push(b ^ 0x36);
        opad.push(b ^ 0x5c);
    }
    ipad.extend_from_slice(data);

    let mut inner = Sm3::new();
    inner.update(&ipad);
    opad.extend_from_slice(&inner.finalize());

    let mut outer = Sm3::new();
    outer.update(&opad);
    hex::encode(outer.finalize())
}

fn sm4_key(key_hex: &str) -> Result<[u8; 16]> {
    let bytes =
        hex::decode(strip_ws(key_hex)).map_err(|_| anyhow!("SM4 密钥不是有效的十六进制"))?;
    if bytes.len() != 16 {
        return Err(anyhow!("SM4 密钥必须是 16 字节（32 个十六进制字符）"));
    }
    let mut key = [0u8; 16];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn sm4_iv(iv_hex: &str) -> Result<[u8; 16]> {
    let bytes = hex::decode(strip_ws(iv_hex)).map_err(|_| anyhow!("SM4 IV 不是有效的十六进制"))?;
    if bytes.len() != 16 {
        return Err(anyhow!("SM4 IV 必须是 16 字节（32 个十六进制字符）"));
    }
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&bytes);
    Ok(iv)
}

pub fn sm4_encrypt(
    plain: &[u8],
    key_hex: &str,
    mode: &str,
    iv_hex: &str,
    padding: bool,
) -> Result<Vec<u8>> {
    let key = sm4_key(key_hex)?;
    let data = pad_if(plain, padding);
    let cipher = Sm4::new(&Array::try_from(&key[..]).expect("sm4 key size"));
    let mut out = Vec::with_capacity(data.len());
    match mode {
        "CBC" => {
            let iv = sm4_iv(iv_hex)?;
            let mut prev = iv;
            for chunk in data.chunks(16) {
                let mut block = [0u8; 16];
                for i in 0..16 {
                    block[i] = chunk[i] ^ prev[i];
                }
                let mut ga = Array::from(block);
                cipher.encrypt_block(&mut ga);
                block.copy_from_slice(&ga);
                out.extend_from_slice(&block);
                prev = block;
            }
        }
        _ => {
            for chunk in data.chunks(16) {
                let mut ga = Array::try_from(chunk).expect("block size");
                cipher.encrypt_block(&mut ga);
                out.extend_from_slice(&ga);
            }
        }
    }
    Ok(out)
}

pub fn sm4_decrypt(
    cipher_data: &[u8],
    key_hex: &str,
    mode: &str,
    iv_hex: &str,
    padding: bool,
) -> Result<Vec<u8>> {
    if cipher_data.is_empty() || !cipher_data.len().is_multiple_of(16) {
        return Err(anyhow!("密文长度必须是 16 字节的整数倍"));
    }
    let key = sm4_key(key_hex)?;
    let cipher = Sm4::new(&Array::try_from(&key[..]).expect("sm4 key size"));
    let mut out = Vec::with_capacity(cipher_data.len());
    match mode {
        "CBC" => {
            let iv = sm4_iv(iv_hex)?;
            let mut prev = iv;
            for chunk in cipher_data.chunks(16) {
                let mut ga = Array::try_from(chunk).expect("block size");
                cipher.decrypt_block(&mut ga);
                let mut block = [0u8; 16];
                block.copy_from_slice(&ga);
                for i in 0..16 {
                    block[i] ^= prev[i];
                }
                out.extend_from_slice(&block);
                prev.copy_from_slice(chunk);
            }
        }
        _ => {
            for chunk in cipher_data.chunks(16) {
                let mut ga = Array::try_from(chunk).expect("block size");
                cipher.decrypt_block(&mut ga);
                out.extend_from_slice(&ga);
            }
        }
    }
    if padding {
        unpad(&mut out)?;
    }
    Ok(out)
}

fn pad_if(data: &[u8], padding: bool) -> Vec<u8> {
    let mut v = data.to_vec();
    if padding {
        let pad = 16 - (v.len() % 16);
        v.resize(v.len() + pad, pad as u8);
    } else if !v.len().is_multiple_of(16) {
        v.resize(v.len() + (16 - v.len() % 16), 0);
    }
    v
}

fn unpad(data: &mut Vec<u8>) -> Result<()> {
    let last = match data.last() {
        Some(b) => *b as usize,
        None => return Ok(()),
    };
    if last == 0 || last > 16 || last > data.len() {
        return Err(anyhow!("解密失败：填充校验不通过，请检查密钥 / IV / 模式"));
    }
    for &b in &data[data.len() - last..] {
        if b as usize != last {
            return Err(anyhow!("解密失败：填充校验不通过，请检查密钥 / IV / 模式"));
        }
    }
    data.truncate(data.len() - last);
    Ok(())
}

pub struct Sm2KeyPair {
    pub private_hex: String,
    pub public_hex: String,
}

pub fn sm2_generate_keypair() -> Result<Sm2KeyPair> {
    let secret = SecretKey::generate();
    Ok(pair_of(&secret))
}

pub fn sm2_public_from_private(private_hex: &str) -> Result<String> {
    let secret = parse_secret(private_hex)?;
    Ok(pair_of(&secret).public_hex)
}

fn pair_of(secret: &SecretKey) -> Sm2KeyPair {
    let public = secret.public_key();
    Sm2KeyPair {
        private_hex: hex::encode_upper(secret.to_bytes()),
        public_hex: hex::encode_upper(public.to_sec1_bytes()),
    }
}

fn parse_secret(private_hex: &str) -> Result<SecretKey> {
    let bytes =
        hex::decode(strip_ws(private_hex)).map_err(|_| anyhow!("私钥不是有效的十六进制"))?;
    SecretKey::from_slice(&bytes).map_err(|_| anyhow!("私钥无效：长度或取值范围不符合 SM2 要求"))
}

fn norm_public(public_hex: &str) -> Result<Vec<u8>> {
    let bytes = hex::decode(strip_ws(public_hex)).map_err(|_| anyhow!("公钥不是有效的十六进制"))?;
    match bytes.len() {
        64 | 65 => Ok(bytes),
        128 | 130 => {
            let mut v = vec![4u8];
            v.extend_from_slice(&bytes);
            Ok(v)
        }
        _ => Err(anyhow!("公钥长度无效，应为 64 / 65 字节的 SEC1 编码")),
    }
}

pub fn sm2_sign(private_hex: &str, distid: &str, message: &[u8]) -> Result<String> {
    let secret = parse_secret(private_hex)?;
    let signing = SigningKey::new(distid, &secret).map_err(|_| anyhow!("SM2 签名密钥构造失败"))?;
    let signature: Signature = signing.sign(message);
    Ok(hex::encode_upper(signature.to_bytes()))
}

pub fn sm2_verify(
    public_hex: &str,
    distid: &str,
    message: &[u8],
    signature_hex: &str,
) -> Result<bool> {
    let public = norm_public(public_hex)?;
    let verifying = VerifyingKey::from_sec1_bytes(distid, &public)
        .map_err(|_| anyhow!("公钥无效，无法构造验签密钥"))?;
    let sig = Signature::from_slice(&hex::decode(strip_ws(signature_hex))?)
        .map_err(|_| anyhow!("签名格式无效，应为 64 字节 r||s"))?;
    Ok(verifying.verify(message, &sig).is_ok())
}

pub fn sm2_encrypt(public_hex: &str, data: &[u8]) -> Result<String> {
    if data.is_empty() {
        return Err(anyhow!("待加密内容不能为空"));
    }
    let public = norm_public(public_hex)?;
    let key = EncryptingKey::from_sec1_bytes(&public).map_err(|_| anyhow!("公钥无效"))?;
    let encrypted = key
        .encrypt(&mut SysRng, data)
        .map_err(|_| anyhow!("SM2 加密失败"))?;
    Ok(hex::encode_upper(encrypted))
}

pub fn sm2_decrypt(private_hex: &str, cipher_hex: &str) -> Result<Vec<u8>> {
    let secret = parse_secret(private_hex)?;
    let bytes = hex::decode(strip_ws(cipher_hex)).map_err(|_| anyhow!("密文不是有效的十六进制"))?;
    let key = DecryptingKey::new(secret);
    key.decrypt(&bytes)
        .map_err(|_| anyhow!("SM2 解密失败，请检查私钥与密文是否匹配"))
}

pub fn sm2_private_to_pem(private_hex: &str) -> Result<String> {
    let secret = parse_secret(private_hex)?;
    use sm2::pkcs8::EncodePrivateKey as _;
    let pem = secret
        .to_sec1_pem(Default::default())
        .map_err(|_| anyhow!("PEM 编码失败"))?;
    Ok(pem.to_string())
}

pub fn sm2_private_from_pem(pem: &str) -> Result<String> {
    use sm2::pkcs8::DecodePrivateKey as _;
    let secret = SecretKey::from_sec1_pem(pem.trim()).map_err(|_| anyhow!("PEM 私钥解析失败"))?;
    Ok(hex::encode_upper(secret.to_bytes()))
}

pub fn strip_ws(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn decode_bytes(input: &str, format: &str) -> Result<Vec<u8>> {
    let text = strip_ws(input);
    match format {
        "Base64" => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(text.as_bytes())
                .map_err(|_| anyhow!("输入不是有效的 Base64"))
        }
        _ => hex::decode(&text).map_err(|_| anyhow!("输入不是有效的十六进制")),
    }
}

pub fn encode_bytes(data: &[u8], format: &str) -> String {
    match format {
        "Base64" => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(data)
        }
        _ => hex::encode_upper(data),
    }
}

pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    let mut rng = SysRng;
    use sm2::elliptic_curve::rand_core::TryRng as _;
    let _ = rng.try_fill_bytes(&mut buf);
    hex::encode_upper(buf)
}

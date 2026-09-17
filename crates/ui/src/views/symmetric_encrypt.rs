use aes::cipher::{
    block_padding::Pkcs7, generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut, KeyIvInit,
    KeyInit,
};
use crate::design;
use data_encoding::{BASE64, HEXLOWER, HEXLOWER_PERMISSIVE};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct SymmetricEncryptor {
    algorithm: String,
    mode: String,
    format: String,
    key: String,
    iv: String,
    plain: String,
    cipher: String,
    output: String,
    error: String,
    key_state: Entity<InputState>,
    iv_state: Entity<InputState>,
    plain_state: Entity<TextareaState>,
    cipher_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    algorithm_state: Entity<SelectState<Vec<String>>>,
    mode_state: Entity<SelectState<Vec<String>>>,
    format_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

/// 按目标长度截断/零填充密钥或 IV 字节
fn fit_bytes(s: &str, len: usize) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.truncate(len);
    v.resize(len, 0);
    v
}

/// rc4 crate 密钥长度需编译期常量，无法处理运行时任意长度 UTF-8 密钥，这里手写 RC4
fn rc4_apply(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut s = [0u8; 256];
    for (i, v) in s.iter_mut().enumerate() {
        *v = i as u8;
    }
    let mut j = 0usize;
    for i in 0..256 {
        j = (j + s[i] as usize + key[i % key.len()] as usize) & 0xff;
        s.swap(i, j);
    }
    let mut out = Vec::with_capacity(data.len());
    let (mut i, mut j) = (0usize, 0usize);
    for &b in data {
        i = (i + 1) & 0xff;
        j = (j + s[i] as usize) & 0xff;
        s.swap(i, j);
        out.push(b ^ s[(s[i] as usize + s[j] as usize) & 0xff]);
    }
    out
}

// Rabbit 流密码（eSTREAM 提交之一）。Vue 用 CryptoJS.Rabbit（OpenSSL 口令派生 + Salted__ 头），
// GPUI 沿用本文件手写 RC4 的 raw-key 设计，直接以密钥原始字节驱动密钥流，加解密自洽。
const RABBIT_A: [u32; 8] = [
    0x4D34D34D, 0xD34D34D3, 0x34D34D34, 0x4D34D34D, 0xD34D34D3, 0x34D34D34, 0x4D34D34D, 0xD34D34D3,
];

#[inline]
fn rabbit_g(v: u32) -> u32 {
    let s = (v as u64).wrapping_mul(v as u64);
    ((s >> 32) as u32) ^ (s as u32)
}

/// 推进一轮状态：更新 256 位计数器（带进位，进位位 b 跨轮保留，见 RFC 4503 §2.5），
/// 随后做 g 函数与混合（§2.6）。加解密共用，保证可逆。
fn rabbit_next_state(x: &mut [u32; 8], c: &mut [u32; 8], b: &mut u32) {
    for i in 0..8 {
        let t = c[i] as u64 + RABBIT_A[i] as u64 + *b as u64;
        c[i] = t as u32;
        *b = (t >> 32) as u32;
    }
    let mut gs = [0u32; 8];
    for i in 0..8 {
        gs[i] = rabbit_g(x[i].wrapping_add(c[i]));
    }
    x[0] = gs[0]
        .wrapping_add(gs[7].rotate_left(16))
        .wrapping_add(gs[6].rotate_left(16));
    x[1] = gs[1]
        .wrapping_add(gs[0].rotate_left(8))
        .wrapping_add(gs[7]);
    x[2] = gs[2]
        .wrapping_add(gs[1].rotate_left(16))
        .wrapping_add(gs[0].rotate_left(16));
    x[3] = gs[3]
        .wrapping_add(gs[2].rotate_left(8))
        .wrapping_add(gs[1]);
    x[4] = gs[4]
        .wrapping_add(gs[3].rotate_left(16))
        .wrapping_add(gs[2].rotate_left(16));
    x[5] = gs[5]
        .wrapping_add(gs[4].rotate_left(8))
        .wrapping_add(gs[3]);
    x[6] = gs[6]
        .wrapping_add(gs[5].rotate_left(16))
        .wrapping_add(gs[4].rotate_left(16));
    x[7] = gs[7]
        .wrapping_add(gs[6].rotate_left(8))
        .wrapping_add(gs[5]);
}

/// 由 128 位密钥初始化状态（RFC 4503 §2.3：8 个 16 位子密钥、4 轮扩散后 Cj ^= X(j+4)，计数器进位位 b 一并返回保留）。
fn rabbit_setup(key16: &[u8; 16]) -> ([u32; 8], [u32; 8], u32) {
    // 子密钥 K0..K7（每个 16 位）：K0 = 密钥最低 16 位，K7 = 密钥最高 16 位。
    let k = |j: usize| -> u16 {
        let hi = key16[14 - 2 * j] as u16;
        let lo = key16[15 - 2 * j] as u16;
        (hi << 8) | lo
    };
    let ksub: Vec<u16> = (0..8).map(|j| k(j)).collect();
    let hi = |i: usize| (ksub[i] as u32) << 16;
    let lo = |i: usize| ksub[i] as u32;

    let mut x = [0u32; 8];
    let mut c = [0u32; 8];
    for j in 0..8usize {
        let (j1, j4, j5) = ((j + 1) % 8, (j + 4) % 8, (j + 5) % 8);
        if j % 2 == 0 {
            x[j] = hi(j1) | lo(j);
            c[j] = hi(j4) | lo(j5);
        } else {
            x[j] = hi(j5) | lo(j4);
            c[j] = hi(j) | lo(j1);
        }
    }
    let mut b = 0u32;
    for _ in 0..4 {
        rabbit_next_state(&mut x, &mut c, &mut b);
    }
    for j in 0..8 {
        c[j] ^= x[(j + 4) % 8];
    }
    (x, c, b)
}

/// RFC 4503 §2.7 提取：由状态词半字异或得到 8 个 16 位词，按大端输出 16 字节。
fn rabbit_extract(x: &[u32; 8], out: &mut [u8; 16]) {
    let w0 = ((x[0] & 0xffff) ^ (x[5] >> 16)) as u16;
    let w1 = ((x[0] >> 16) ^ (x[3] & 0xffff)) as u16;
    let w2 = ((x[2] & 0xffff) ^ (x[7] >> 16)) as u16;
    let w3 = ((x[2] >> 16) ^ (x[5] & 0xffff)) as u16;
    let w4 = ((x[4] & 0xffff) ^ (x[1] >> 16)) as u16;
    let w5 = ((x[4] >> 16) ^ (x[7] & 0xffff)) as u16;
    let w6 = ((x[6] & 0xffff) ^ (x[3] >> 16)) as u16;
    let w7 = ((x[6] >> 16) ^ (x[1] & 0xffff)) as u16;
    let ws = [w7, w6, w5, w4, w3, w2, w1, w0];
    for (i, ww) in ws.iter().enumerate() {
        out[2 * i] = (ww >> 8) as u8;
        out[2 * i + 1] = (ww & 0xff) as u8;
    }
}

/// 流密码应用：以固定 IV=0 运行（无 UI IV 输入，与手写 RC4 一致），逐字节异或。
fn rabbit_apply(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut key16 = [0u8; 16];
    for (i, b) in key.iter().take(16).enumerate() {
        key16[i] = *b;
    }
    let (mut x, mut c, mut b) = rabbit_setup(&key16);
    let mut out = Vec::with_capacity(data.len());
    let mut block = [0u8; 16];
    for (i, &bb) in data.iter().enumerate() {
        if i % 16 == 0 {
            rabbit_next_state(&mut x, &mut c, &mut b);
            rabbit_extract(&x, &mut block);
        }
        out.push(bb ^ block[i % 16]);
    }
    out
}

impl SymmetricEncryptor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let key_state = cx.new(|cx| InputState::new(window, cx).placeholder("请输入密钥"));
        let iv_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("IV（CBC 模式可选，默认全零）")
        });
        let plain_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入明文"));
        let cipher_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入密文"));
        let output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("加密 / 解密结果"));

        let algorithm_items = vec![
            "AES".to_string(),
            "DES".to_string(),
            "3DES".to_string(),
            "RC4".to_string(),
            "Rabbit".to_string(),
        ];
        let mode_items = vec!["ECB".to_string(), "CBC".to_string()];
        let format_items = vec!["Base64".to_string(), "Hex".to_string()];

        let algorithm_state = cx.new(|cx| {
            let mut state = SelectState::new(algorithm_items, None, window, cx);
            state.set_selected_value(&"AES".to_string(), window, cx);
            state
        });
        let mode_state = cx.new(|cx| {
            let mut state = SelectState::new(mode_items, None, window, cx);
            state.set_selected_value(&"CBC".to_string(), window, cx);
            state
        });
        let format_state = cx.new(|cx| {
            let mut state = SelectState::new(format_items, None, window, cx);
            state.set_selected_value(&"Base64".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&key_state, window, {
                let key_state = key_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = key_state.read(cx).value();
                        this.key = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&iv_state, window, {
                let iv_state = iv_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = iv_state.read(cx).value();
                        this.iv = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&plain_state, window, {
                let plain_state = plain_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = plain_state.read(cx).value();
                        this.plain = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&cipher_state, window, {
                let cipher_state = cipher_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = cipher_state.read(cx).value();
                        this.cipher = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &algorithm_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.algorithm = value.clone();
                        this.error.clear();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &mode_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.mode = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.format = value.clone();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            algorithm: "AES".to_string(),
            mode: "CBC".to_string(),
            format: "Base64".to_string(),
            key: String::new(),
            iv: String::new(),
            plain: String::new(),
            cipher: String::new(),
            output: String::new(),
            error: String::new(),
            key_state,
            iv_state,
            plain_state,
            cipher_state,
            output_state,
            algorithm_state,
            mode_state,
            format_state,
            _subscriptions,
        }
    }

    fn has_mode(&self) -> bool {
        self.algorithm != "RC4" && self.algorithm != "Rabbit"
    }

    fn format_bytes(&self, bytes: &[u8]) -> String {
        match self.format.as_str() {
            "Hex" => HEXLOWER.encode(bytes),
            _ => BASE64.encode(bytes),
        }
    }

    fn parse_cipher(&self) -> Result<Vec<u8>, String> {
        let text: String = self.cipher.chars().filter(|c| !c.is_whitespace()).collect();
        match self.format.as_str() {
            "Hex" => HEXLOWER_PERMISSIVE
                .decode(text.as_bytes())
                .map_err(|_| "密文不是有效的十六进制".to_string()),
            _ => BASE64
                .decode(text.as_bytes())
                .map_err(|_| "密文不是有效的 Base64".to_string()),
        }
    }

    fn encrypt_data(&self) -> Result<Vec<u8>, String> {
        let plain = self.plain.as_bytes();
        match self.algorithm.as_str() {
            "AES" => {
                let n = self.key.as_bytes().len();
                let key16 = fit_bytes(&self.key, 16);
                let key24 = fit_bytes(&self.key, 24);
                let key32 = fit_bytes(&self.key, 32);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 16);
                    let iv = GenericArray::from_slice(&iv_buf);
                    if n <= 16 {
                        Ok(cbc::Encryptor::<aes::Aes128>::new(
                            GenericArray::from_slice(&key16),
                            iv,
                        )
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                    } else if n <= 24 {
                        Ok(cbc::Encryptor::<aes::Aes192>::new(
                            GenericArray::from_slice(&key24),
                            iv,
                        )
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                    } else {
                        Ok(cbc::Encryptor::<aes::Aes256>::new(
                            GenericArray::from_slice(&key32),
                            iv,
                        )
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                    }
                } else if n <= 16 {
                    Ok(ecb::Encryptor::<aes::Aes128>::new(
                        GenericArray::from_slice(&key16),
                    )
                    .encrypt_padded_vec_mut::<Pkcs7>(plain))
                } else if n <= 24 {
                    Ok(ecb::Encryptor::<aes::Aes192>::new(
                        GenericArray::from_slice(&key24),
                    )
                    .encrypt_padded_vec_mut::<Pkcs7>(plain))
                } else {
                    Ok(ecb::Encryptor::<aes::Aes256>::new(
                        GenericArray::from_slice(&key32),
                    )
                    .encrypt_padded_vec_mut::<Pkcs7>(plain))
                }
            }
            "DES" => {
                let key_buf = fit_bytes(&self.key, 8);
                let key = GenericArray::from_slice(&key_buf);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 8);
                    let iv = GenericArray::from_slice(&iv_buf);
                    Ok(cbc::Encryptor::<des::Des>::new(key, iv)
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                } else {
                    Ok(ecb::Encryptor::<des::Des>::new(key)
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                }
            }
            "3DES" => {
                let key_buf = fit_bytes(&self.key, 24);
                let key = GenericArray::from_slice(&key_buf);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 8);
                    let iv = GenericArray::from_slice(&iv_buf);
                    Ok(cbc::Encryptor::<des::TdesEde3>::new(key, iv)
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                } else {
                    Ok(ecb::Encryptor::<des::TdesEde3>::new(key)
                        .encrypt_padded_vec_mut::<Pkcs7>(plain))
                }
            }
            "RC4" => Ok(rc4_apply(self.key.as_bytes(), plain)),
            "Rabbit" => Ok(rabbit_apply(self.key.as_bytes(), plain)),
            _ => Err("不支持的算法".to_string()),
        }
    }

    fn decrypt_data(&self, ct: &[u8]) -> Result<Vec<u8>, String> {
        match self.algorithm.as_str() {
            "AES" => {
                let n = self.key.as_bytes().len();
                let key16 = fit_bytes(&self.key, 16);
                let key24 = fit_bytes(&self.key, 24);
                let key32 = fit_bytes(&self.key, 32);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 16);
                    let iv = GenericArray::from_slice(&iv_buf);
                    if n <= 16 {
                        cbc::Decryptor::<aes::Aes128>::new(GenericArray::from_slice(&key16), iv)
                            .decrypt_padded_vec_mut::<Pkcs7>(ct)
                    } else if n <= 24 {
                        cbc::Decryptor::<aes::Aes192>::new(GenericArray::from_slice(&key24), iv)
                            .decrypt_padded_vec_mut::<Pkcs7>(ct)
                    } else {
                        cbc::Decryptor::<aes::Aes256>::new(GenericArray::from_slice(&key32), iv)
                            .decrypt_padded_vec_mut::<Pkcs7>(ct)
                    }
                } else if n <= 16 {
                    ecb::Decryptor::<aes::Aes128>::new(GenericArray::from_slice(&key16))
                        .decrypt_padded_vec_mut::<Pkcs7>(ct)
                } else if n <= 24 {
                    ecb::Decryptor::<aes::Aes192>::new(GenericArray::from_slice(&key24))
                        .decrypt_padded_vec_mut::<Pkcs7>(ct)
                } else {
                    ecb::Decryptor::<aes::Aes256>::new(GenericArray::from_slice(&key32))
                        .decrypt_padded_vec_mut::<Pkcs7>(ct)
                }
                .map_err(|_| "解密失败，请检查密钥、IV 或密文格式".to_string())
            }
            "DES" => {
                let key_buf = fit_bytes(&self.key, 8);
                let key = GenericArray::from_slice(&key_buf);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 8);
                    let iv = GenericArray::from_slice(&iv_buf);
                    cbc::Decryptor::<des::Des>::new(key, iv).decrypt_padded_vec_mut::<Pkcs7>(ct)
                } else {
                    ecb::Decryptor::<des::Des>::new(key).decrypt_padded_vec_mut::<Pkcs7>(ct)
                }
                .map_err(|_| "解密失败，请检查密钥、IV 或密文格式".to_string())
            }
            "3DES" => {
                let key_buf = fit_bytes(&self.key, 24);
                let key = GenericArray::from_slice(&key_buf);
                if self.mode == "CBC" {
                    let iv_buf = fit_bytes(&self.iv, 8);
                    let iv = GenericArray::from_slice(&iv_buf);
                    cbc::Decryptor::<des::TdesEde3>::new(key, iv)
                        .decrypt_padded_vec_mut::<Pkcs7>(ct)
                } else {
                    ecb::Decryptor::<des::TdesEde3>::new(key).decrypt_padded_vec_mut::<Pkcs7>(ct)
                }
                .map_err(|_| "解密失败，请检查密钥、IV 或密文格式".to_string())
            }
            "RC4" => Ok(rc4_apply(self.key.as_bytes(), ct)),
            "Rabbit" => Ok(rabbit_apply(self.key.as_bytes(), ct)),
            _ => Err("不支持的算法".to_string()),
        }
    }

    fn encrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.key.is_empty() {
            self.error = "请输入密钥".to_string();
            cx.notify();
            return;
        }
        if self.plain.is_empty() {
            self.error = "请输入明文".to_string();
            cx.notify();
            return;
        }
        match self.encrypt_data() {
            Ok(bytes) => {
                self.output = self.format_bytes(&bytes);
                self.output_state.update(cx, |state, cx| {
                    state.set_value(self.output.clone(), window, cx);
                });
            }
            Err(e) => self.error = format!("加密失败: {e}"),
        }
        cx.notify();
    }

    fn decrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.key.is_empty() {
            self.error = "请输入密钥".to_string();
            cx.notify();
            return;
        }
        if self.cipher.trim().is_empty() {
            self.error = "请输入密文".to_string();
            cx.notify();
            return;
        }
        let result = self
            .parse_cipher()
            .and_then(|ct| self.decrypt_data(&ct))
            .and_then(|pt| {
                String::from_utf8(pt)
                    .map_err(|_| "解密结果不是有效的 UTF-8 文本".to_string())
            });
        match result {
            Ok(text) => {
                self.output = text.clone();
                self.output_state.update(cx, |state, cx| {
                    state.set_value(text, window, cx);
                });
            }
            Err(e) => self.error = e,
        }
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.plain.clear();
        self.cipher.clear();
        self.output.clear();
        self.error.clear();
        self.plain_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.cipher_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn paste_plain(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.plain = text.to_string();
                self.plain_state.update(cx, |state, cx| {
                    state.set_value(self.plain.clone(), window, cx);
                });
            }
        }
    }

    fn copy_plain(&mut self, cx: &mut Context<Self>) {
        if !self.plain.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.plain.clone()));
        }
    }

    fn paste_cipher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.cipher = text.to_string();
                self.cipher_state.update(cx, |state, cx| {
                    state.set_value(self.cipher.clone(), window, cx);
                });
            }
        }
    }

    fn copy_cipher(&mut self, cx: &mut Context<Self>) {
        if !self.cipher.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.cipher.clone()));
        }
    }
}

impl Render for SymmetricEncryptor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(120.0);

        design::page()
            .child(design::page_header("对称加密", "AES / DES / RC4 / Rabbit", cx))
            .child(
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("算法", cx)))
                            .child(Select::new(&self.algorithm_state)),
                    )
                    .when(self.has_mode(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().w(label_w).child(design::caption("模式", cx)))
                                .child(Select::new(&self.mode_state)),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("密钥", cx)))
                            .child(div().flex_1().child(Input::new(&self.key_state))),
                    )
                    .when(self.has_mode() && self.mode == "CBC", |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().w(label_w).child(design::caption("IV 向量", cx)))
                                .child(div().flex_1().child(Input::new(&self.iv_state))),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("输出格式", cx)))
                            .child(Select::new(&self.format_state)),
                    ),
            )
            .child(
                design::card(cx)
                    .child(
                        ButtonGroup::new("input-buttons")
                    .child(
                        Button::new("paste-plain")
                            .icon(Icon::new(IconName::File))
                            .tooltip("粘贴明文")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste_plain(window, cx);
                            })),
                    )
                    .child(
                        Button::new("copy-plain")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制明文")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_plain(cx);
                            })),
                    )
                    .child(
                        Button::new("paste-cipher")
                            .icon(Icon::new(IconName::File))
                            .tooltip("粘贴密文")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste_cipher(window, cx);
                            })),
                    )
                    .child(
                        Button::new("copy-cipher")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制密文")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_cipher(cx);
                            })),
                    )
                    .child(
                        Button::new("clear-all")
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清空")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear(window, cx);
                            })),
                    ),
                    )
                    .child(Textarea::new(&self.plain_state).h(px(120.0)).flex_1())
                    .child(Textarea::new(&self.cipher_state).h(px(120.0)).flex_1()),
            )
            .child(
                design::action_row()
                    .child(
                        Button::new("encrypt")
                            .label("加密")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encrypt(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decrypt")
                            .label("解密")
                            .icon(Icon::new(IconName::ArrowUp))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decrypt(window, cx);
                            })),
                    )
                    .when(!self.error.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .child(self.error.clone()),
                        )
                    }),
            )
            .child(
                design::card(cx)
                    .child(Textarea::new(&self.output_state).h(px(120.0)).flex_1())
                    .child(
                        ButtonGroup::new("output-buttons").child(
                    Button::new("copy-output")
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制结果")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.copy_output(cx);
                        })),
                ),
                    ),
            )
    }
}

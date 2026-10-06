use gpui_kit::{
    Stateful,
    component::{
        button::*,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

const TAB_OPTIONS: [&str; 4] = ["SM3", "SM4", "SM2 签名", "SM2 加解密"];
const SM4_MODES: [&str; 2] = ["ECB", "CBC"];
const FORMAT_OPTIONS: [&str; 2] = ["Hex", "Base64"];

pub struct SmCrypto {
    tab: String,
    error: String,
    success: String,
    info: String,

    sm3_input: String,
    sm3_hmac_key: String,
    sm3_use_hmac: bool,
    sm3_output: String,

    sm4_input: String,
    sm4_key: String,
    sm4_iv: String,
    sm4_mode: String,
    sm4_padding: bool,
    sm4_format: String,
    sm4_output: String,

    sm2_message: String,
    sm2_private: String,
    sm2_public: String,
    sm2_distid: String,
    sm2_signature: String,
    sm2_verify_result: String,

    sm2_plain: String,
    sm2_cipher: String,
    sm2_decrypted: String,

    sm3_input_state: Entity<TextareaState>,
    sm3_hmac_key_state: Entity<InputState>,
    sm3_output_state: Entity<TextareaState>,
    sm4_input_state: Entity<TextareaState>,
    sm4_key_state: Entity<InputState>,
    sm4_iv_state: Entity<InputState>,
    sm4_output_state: Entity<TextareaState>,
    sm2_message_state: Entity<TextareaState>,
    sm2_private_state: Entity<TextareaState>,
    sm2_public_state: Entity<TextareaState>,
    sm2_distid_state: Entity<InputState>,
    sm2_signature_state: Entity<TextareaState>,
    sm2_plain_state: Entity<TextareaState>,
    sm2_cipher_state: Entity<TextareaState>,
    sm2_decrypted_state: Entity<TextareaState>,
    sm4_mode_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl SmCrypto {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let sm3_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入待摘要的文本"));
        let sm3_hmac_key_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("HMAC-SM3 密钥（可留空）"));
        let sm3_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("SM3 摘要结果"));
        let sm4_input_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本"));
        let sm4_key_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("32 位十六进制密钥，如 0123456789abcdeffedcba9876543210")
        });
        let sm4_iv_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("CBC 模式 IV：32 位十六进制，默认全零")
        });
        let sm4_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("SM4 加解密结果"));
        let sm2_message_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入待签名 / 验签的原文"));
        let sm2_private_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("SM2 私钥：64 位十六进制"));
        let sm2_public_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("SM2 公钥：130 位十六进制（04 开头）或 128 位裸坐标")
        });
        let sm2_distid_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("用户标识 DistID，默认 1234567812345678")
        });
        let sm2_signature_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("签名值：128 位十六进制"));
        let sm2_plain_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("待加密明文"));
        let sm2_cipher_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("SM2 密文（十六进制）"));
        let sm2_decrypted_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("解密结果"));

        let sm4_mode_state = cx.new(|cx| {
            let items = SM4_MODES.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let mut state = SelectState::new(items, None, window, cx);
            state.set_selected_value(&"CBC".to_string(), window, cx);
            state
        });

        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe_in(&sm3_input_state, window, {
            let s = sm3_input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm3_input = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm3_hmac_key_state, window, {
            let s = sm3_hmac_key_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm3_hmac_key = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm4_input_state, window, {
            let s = sm4_input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm4_input = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm4_key_state, window, {
            let s = sm4_key_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm4_key = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm4_iv_state, window, {
            let s = sm4_iv_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm4_iv = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(
            &sm4_mode_state,
            window,
            move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                if let SelectEvent::Confirm(Some(v)) = ev {
                    this.sm4_mode = v.clone();
                    cx.notify();
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(&sm2_message_state, window, {
            let s = sm2_message_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_message = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_private_state, window, {
            let s = sm2_private_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_private = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_public_state, window, {
            let s = sm2_public_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_public = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_distid_state, window, {
            let s = sm2_distid_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_distid = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_signature_state, window, {
            let s = sm2_signature_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_signature = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_plain_state, window, {
            let s = sm2_plain_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_plain = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe_in(&sm2_cipher_state, window, {
            let s = sm2_cipher_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.sm2_cipher = s.read(cx).value().to_string();
                    cx.notify();
                }
            }
        }));

        Self {
            tab: "SM3".to_string(),
            error: String::new(),
            success: String::new(),
            info: String::new(),
            sm3_input: String::new(),
            sm3_hmac_key: String::new(),
            sm3_use_hmac: false,
            sm3_output: String::new(),
            sm4_input: String::new(),
            sm4_key: String::new(),
            sm4_iv: String::new(),
            sm4_mode: "CBC".to_string(),
            sm4_padding: true,
            sm4_format: "Hex".to_string(),
            sm4_output: String::new(),
            sm2_message: String::new(),
            sm2_private: String::new(),
            sm2_public: String::new(),
            sm2_distid: String::new(),
            sm2_signature: String::new(),
            sm2_verify_result: String::new(),
            sm2_plain: String::new(),
            sm2_cipher: String::new(),
            sm2_decrypted: String::new(),
            sm3_input_state,
            sm3_hmac_key_state,
            sm3_output_state,
            sm4_input_state,
            sm4_key_state,
            sm4_iv_state,
            sm4_output_state,
            sm2_message_state,
            sm2_private_state,
            sm2_public_state,
            sm2_distid_state,
            sm2_signature_state,
            sm2_plain_state,
            sm2_cipher_state,
            sm2_decrypted_state,
            sm4_mode_state,
            _subscriptions: subscriptions,
        }
    }

    fn reset(&mut self) {
        self.error.clear();
        self.success.clear();
        self.info.clear();
    }

    fn distid(&self) -> String {
        let v = self.sm2_distid.trim();
        if v.is_empty() {
            ::base::DEFAULT_DISTID.to_string()
        } else {
            v.to_string()
        }
    }

    fn set_textarea(
        state: &Entity<TextareaState>,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        state.update(cx, |s, cx| s.set_value(text, window, cx));
    }

    fn do_sm3(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm3_input.is_empty() {
            self.error = "请输入待摘要的文本".to_string();
            cx.notify();
            return;
        }
        let digest = if self.sm3_use_hmac {
            if self.sm3_hmac_key.is_empty() {
                self.error = "已启用 HMAC，请填写密钥".to_string();
                cx.notify();
                return;
            }
            ::base::sm3_hmac_hex(self.sm3_hmac_key.as_bytes(), self.sm3_input.as_bytes())
        } else {
            ::base::sm3_hex(self.sm3_input.as_bytes())
        };
        let pretty = digest
            .as_bytes()
            .chunks(8)
            .map(|c| String::from_utf8_lossy(c).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        self.sm3_output = pretty.clone();
        Self::set_textarea(&self.sm3_output_state, pretty, window, cx);
        self.info = format!(
            "输入 {} 字节，SM3 输出 256 位（32 字节）",
            self.sm3_input.as_bytes().len()
        );
        self.success = "计算完成".to_string();
        cx.notify();
    }

    fn do_sm4_encrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm4_key.trim().is_empty() {
            self.error = "请输入 SM4 密钥".to_string();
            cx.notify();
            return;
        }
        if self.sm4_input.is_empty() {
            self.error = "请输入待加密文本".to_string();
            cx.notify();
            return;
        }
        match ::base::sm4_encrypt(
            self.sm4_input.as_bytes(),
            &self.sm4_key,
            &self.sm4_mode,
            &self.sm4_iv,
            self.sm4_padding,
        ) {
            Ok(bytes) => {
                let rendered = ::base::encode_bytes(&bytes, &self.sm4_format);
                self.sm4_output = rendered.clone();
                Self::set_textarea(&self.sm4_output_state, rendered, window, cx);
                self.info = format!(
                    "{} / {} / 填充 {}：明文 {} 字节 → 密文 {} 字节",
                    self.sm4_mode,
                    self.sm4_format,
                    if self.sm4_padding { "PKCS#7" } else { "无" },
                    self.sm4_input.as_bytes().len(),
                    bytes.len()
                );
                self.success = "SM4 加密成功".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn do_sm4_decrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm4_key.trim().is_empty() {
            self.error = "请输入 SM4 密钥".to_string();
            cx.notify();
            return;
        }
        if self.sm4_input.trim().is_empty() {
            self.error = "请输入待解密的密文".to_string();
            cx.notify();
            return;
        }
        let result = ::base::decode_bytes(&self.sm4_input, &self.sm4_format)
            .and_then(|bytes| {
                ::base::sm4_decrypt(
                    &bytes,
                    &self.sm4_key,
                    &self.sm4_mode,
                    &self.sm4_iv,
                    self.sm4_padding,
                )
            })
            .and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| {
                    anyhow::anyhow!("解密成功但结果不是有效 UTF-8 文本，请检查密钥 / IV / 模式")
                })
            });
        match result {
            Ok(text) => {
                self.sm4_output = text.clone();
                Self::set_textarea(&self.sm4_output_state, text, window, cx);
                self.success = "SM4 解密成功".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn gen_keypair(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        match ::base::sm2_generate_keypair() {
            Ok(pair) => {
                self.sm2_private = pair.private_hex.clone();
                self.sm2_public = pair.public_hex.clone();
                Self::set_textarea(&self.sm2_private_state, pair.private_hex, window, cx);
                Self::set_textarea(&self.sm2_public_state, pair.public_hex, window, cx);
                self.success = "已生成 SM2 密钥对，请妥善保存私钥".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn public_from_private(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm2_private.trim().is_empty() {
            self.error = "请先填写私钥".to_string();
            cx.notify();
            return;
        }
        match ::base::sm2_public_from_private(&self.sm2_private) {
            Ok(pk) => {
                self.sm2_public = pk.clone();
                Self::set_textarea(&self.sm2_public_state, pk, window, cx);
                self.success = "已根据私钥推导公钥".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn do_sign(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm2_private.trim().is_empty() {
            self.error = "请填写 SM2 私钥".to_string();
            cx.notify();
            return;
        }
        if self.sm2_message.is_empty() {
            self.error = "请填写待签名原文".to_string();
            cx.notify();
            return;
        }
        let distid = self.distid();
        match ::base::sm2_sign(&self.sm2_private, &distid, self.sm2_message.as_bytes()) {
            Ok(sig) => {
                self.sm2_signature = sig.clone();
                Self::set_textarea(&self.sm2_signature_state, sig, window, cx);
                self.sm2_verify_result.clear();
                self.info = format!("DistID = {distid}");
                self.success = "签名完成".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn do_verify(&mut self, cx: &mut Context<Self>) {
        self.reset();
        if self.sm2_public.trim().is_empty() {
            self.error = "请填写 SM2 公钥".to_string();
            cx.notify();
            return;
        }
        if self.sm2_signature.trim().is_empty() {
            self.error = "请填写签名值".to_string();
            cx.notify();
            return;
        }
        let distid = self.distid();
        match ::base::sm2_verify(
            &self.sm2_public,
            &distid,
            self.sm2_message.as_bytes(),
            &self.sm2_signature,
        ) {
            Ok(true) => {
                self.sm2_verify_result = "验签通过：签名有效".to_string();
                self.success = "验签通过".to_string();
            }
            Ok(false) => {
                self.sm2_verify_result = "验签失败：签名与公钥 / 原文不匹配".to_string();
                self.error = "验签失败".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn do_sm2_encrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm2_public.trim().is_empty() {
            self.error = "请填写 SM2 公钥".to_string();
            cx.notify();
            return;
        }
        if self.sm2_plain.is_empty() {
            self.error = "请填写待加密明文".to_string();
            cx.notify();
            return;
        }
        match ::base::sm2_encrypt(&self.sm2_public, self.sm2_plain.as_bytes()) {
            Ok(ct) => {
                self.sm2_cipher = ct.clone();
                self.info = format!("密文长度 {} 字节（C1C3C2）", ct.len() / 2);
                Self::set_textarea(&self.sm2_cipher_state, ct, window, cx);
                self.success = "SM2 加密成功".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn do_sm2_decrypt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset();
        if self.sm2_private.trim().is_empty() {
            self.error = "请填写 SM2 私钥".to_string();
            cx.notify();
            return;
        }
        if self.sm2_cipher.trim().is_empty() {
            self.error = "请填写 SM2 密文".to_string();
            cx.notify();
            return;
        }
        match ::base::sm2_decrypt(&self.sm2_private, &self.sm2_cipher) {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                self.sm2_decrypted = text.clone();
                Self::set_textarea(&self.sm2_decrypted_state, text, window, cx);
                self.success = "SM2 解密成功".to_string();
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn copy(&mut self, text: String, cx: &mut Context<Self>) {
        if text.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.reset();
        self.success = "已复制到剪贴板".to_string();
        cx.notify();
    }

    fn paste_into(state: &Entity<TextareaState>, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                Self::set_textarea(state, text.to_string(), window, cx);
            }
        }
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_1()
            .children(TAB_OPTIONS.iter().enumerate().map(|(i, t)| {
                let active = *t == self.tab;
                div()
                    .id(("sm-tab", i))
                    .px_3()
                    .py_1p5()
                    .rounded(px(6.0))
                    .text_sm()
                    .cursor_pointer()
                    .bg(if active {
                        cx.theme().primary
                    } else {
                        cx.theme().background
                    })
                    .text_color(if active {
                        gpui::white()
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.tab = t.to_string();
                        this.reset();
                        cx.notify();
                    }))
                    .child(t.to_string())
            }))
    }

    fn format_toggle(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .children(FORMAT_OPTIONS.iter().enumerate().map(|(i, opt)| {
                let active = *opt == self.sm4_format;
                div()
                    .id(("sm-fmt", i))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .cursor_pointer()
                    .bg(if active {
                        cx.theme().primary
                    } else {
                        gpui::black().opacity(0.0)
                    })
                    .text_color(if active {
                        gpui::white()
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.sm4_format = opt.to_string();
                        cx.notify();
                    }))
                    .child(opt.to_string())
            }))
    }

    fn toggle(
        label: &'static str,
        active: bool,
        id: &'static str,
        target: u8,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let color = if active {
            cx.theme().primary
        } else {
            cx.theme().border
        };
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match target {
                    0 => this.sm3_use_hmac = !this.sm3_use_hmac,
                    1 => this.sm4_padding = !this.sm4_padding,
                    _ => {}
                }
                cx.notify();
            }))
            .child(
                div()
                    .w(px(14.0))
                    .h(px(14.0))
                    .rounded(px(3.0))
                    .border_1()
                    .border_color(color)
                    .bg(if active {
                        color
                    } else {
                        gpui::black().opacity(0.0)
                    }),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(cx.theme().foreground)
                    .child(label),
            )
    }

    fn label(text: &'static str, cx: &App) -> Div {
        div()
            .w(px(76.0))
            .flex_shrink_0()
            .text_size(px(12.0))
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }

    fn field(title: &'static str, state: &Entity<TextareaState>, height: f32, cx: &App) -> Div {
        div()
            .flex_col()
            .flex_1()
            .min_w(px(240.0))
            .gap_1p5()
            .child(design::editor_label(title, cx))
            .child(Textarea::new(state).h(px(height)).font_family("monospace"))
    }
}

impl Render for SmCrypto {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let success = self.success.clone();
        let info = self.info.clone();
        let tab = self.tab.clone();

        let body = match tab.as_str() {
            "SM3" => self.render_sm3(window, cx),
            "SM4" => self.render_sm4(window, cx),
            "SM2 签名" => self.render_sm2_sign(window, cx),
            _ => self.render_sm2_pke(window, cx),
        };

        design::page()
            .child(self.tab_bar(cx))
            .child(
                design::card(cx)
                    .child(body)
                    .when(!info.is_empty(), |c| {
                        c.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(info.clone()),
                        )
                    })
                    .when(!error.is_empty(), |c| {
                        c.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(error.clone()),
                        )
                    })
                    .when(!success.is_empty() && error.is_empty(), |c| {
                        c.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::OK_GREEN)))
                                .child(success.clone()),
                        )
                    })
                    .child(design::hint(
                        "国密算法遵循 GM/T 标准：SM3 为 256 位摘要，SM4 为 128 位分组密码，SM2 基于 256 位椭圆曲线。SM2 密文默认使用 C1C3C2 排布；签名与验签必须使用相同的用户标识 DistID，否则验签失败。密钥请通过安全渠道分发，勿在前端明文长期保存。",
                        cx,
                    )),
            )
    }
}

impl SmCrypto {
    fn render_sm3(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let hmac_active = self.sm3_use_hmac;
        let key_state = self.sm3_hmac_key_state.clone();
        let out_state = self.sm3_output_state.clone();
        let in_state = self.sm3_input_state.clone();
        let output = self.sm3_output.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(Self::toggle("启用 HMAC-SM3", hmac_active, "sm3-hmac", 0, cx))
                    .when(hmac_active, |row| {
                        row.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Self::label("密钥", cx))
                                .child(div().w(px(340.0)).child(Input::new(&key_state))),
                        )
                    }),
            )
            .child(Self::field("原文", &in_state, 150.0, cx))
            .child(
                design::toolbar()
                    .child(
                        Button::new("sm3-paste")
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴原文")
                            .on_click(cx.listener(|this, _, window, cx| {
                                Self::paste_into(&this.sm3_input_state.clone(), window, cx);
                            })),
                    )
                    .child(
                        Button::new("sm3-run")
                            .primary()
                            .icon(Icon::new(IconName::Check))
                            .tooltip("计算摘要")
                            .on_click(cx.listener(|this, _, window, cx| this.do_sm3(window, cx))),
                    )
                    .child(
                        Button::new("sm3-copy")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制摘要")
                            .disabled(output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm3_output.clone();
                                this.copy(t, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("摘要结果", cx))
                    .child(
                        Textarea::new(&out_state)
                            .h(px(90.0))
                            .font_family("monospace"),
                    ),
            )
            .child(design::hint(
                "SM3 适用场景：数据完整性校验、口令派生、数字签名中的消息摘要。HMAC-SM3 在密钥未知时无法伪造，适合接口签名与防篡改校验。",
                cx,
            ))
    }

    fn render_sm4(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mode_state = self.sm4_mode_state.clone();
        let key_state = self.sm4_key_state.clone();
        let iv_state = self.sm4_iv_state.clone();
        let in_state = self.sm4_input_state.clone();
        let out_state = self.sm4_output_state.clone();
        let output = self.sm4_output.clone();
        let is_cbc = self.sm4_mode == "CBC";
        let padding = self.sm4_padding;

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Self::label("模式", cx))
                            .child(div().w(px(120.0)).child(Select::new(&mode_state))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Self::label("密钥", cx))
                            .child(div().w(px(360.0)).child(Input::new(&key_state))),
                    )
                    .when(is_cbc, |row| {
                        row.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Self::label("IV", cx))
                                .child(div().w(px(360.0)).child(Input::new(&iv_state))),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Self::label("数据格式", cx))
                            .child(self.format_toggle(cx)),
                    )
                    .child(Self::toggle("PKCS#7 填充", padding, "sm4-pad", 1, cx)),
            )
            .child(Self::field("输入（明文 / 密文）", &in_state, 150.0, cx))
            .child(
                design::toolbar()
                    .child(
                        Button::new("sm4-encrypt")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("加密")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.do_sm4_encrypt(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("sm4-decrypt")
                            .primary()
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解密")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.do_sm4_decrypt(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("sm4-copy")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制结果")
                            .disabled(output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm4_output.clone();
                                this.copy(t, cx);
                            })),
                    )
                    .child(
                        Button::new("sm4-genkey")
                            .icon(Icon::new(IconName::RotateCw))
                            .tooltip("随机生成密钥")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let key = ::base::random_hex(16);
                                this.sm4_key = key.clone();
                                this.sm4_key_state
                                    .update(cx, |s, cx| s.set_value(key, window, cx));
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("结果", cx))
                    .child(
                        Textarea::new(&out_state)
                            .h(px(110.0))
                            .font_family("monospace"),
                    ),
            )
            .child(design::hint(
                "SM4 分组长度与密钥长度均为 128 位。ECB 相同明文块产生相同密文，仅建议用于调试；生产环境请使用 CBC 并配合随机 IV。解密报「填充校验不通过」通常是密钥、IV、模式或数据格式不一致。",
                cx,
            ))
    }

    fn render_sm2_sign(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let msg_state = self.sm2_message_state.clone();
        let priv_state = self.sm2_private_state.clone();
        let pub_state = self.sm2_public_state.clone();
        let distid_state = self.sm2_distid_state.clone();
        let sig_state = self.sm2_signature_state.clone();
        let result = self.sm2_verify_result.clone();
        let sig = self.sm2_signature.clone();
        let priv_key = self.sm2_private.clone();
        let pub_key = self.sm2_public.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Self::label("DistID", cx))
                            .child(div().w(px(300.0)).child(Input::new(&distid_state))),
                    )
                    .child(
                        Button::new("sm2-gen")
                            .icon(Icon::new(IconName::RotateCw))
                            .tooltip("生成密钥对")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.gen_keypair(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("sm2-derive")
                            .icon(Icon::new(IconName::ArrowRight))
                            .tooltip("由私钥推导公钥")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.public_from_private(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_4()
                    .child(Self::field("私钥", &priv_state, 96.0, cx))
                    .child(Self::field("公钥", &pub_state, 96.0, cx)),
            )
            .child(Self::field("原文", &msg_state, 96.0, cx))
            .child(
                design::toolbar()
                    .child(
                        Button::new("sm2-sign")
                            .primary()
                            .icon(Icon::new(IconName::Check))
                            .tooltip("签名")
                            .on_click(cx.listener(|this, _, window, cx| this.do_sign(window, cx))),
                    )
                    .child(
                        Button::new("sm2-verify")
                            .primary()
                            .icon(Icon::new(IconName::Search))
                            .tooltip("验签")
                            .on_click(cx.listener(|this, _, _, cx| this.do_verify(cx))),
                    )
                    .child(
                        Button::new("sm2-copy-priv")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制私钥")
                            .disabled(priv_key.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm2_private.clone();
                                this.copy(t, cx);
                            })),
                    )
                    .child(
                        Button::new("sm2-copy-pub")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制公钥")
                            .disabled(pub_key.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm2_public.clone();
                                this.copy(t, cx);
                            })),
                    ),
            )
            .child(Self::field("签名值", &sig_state, 86.0, cx))
            .when(!result.is_empty(), |c| {
                c.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(if result.starts_with("验签通过") {
                            Hsla::from(rgb(design::OK_GREEN))
                        } else {
                            Hsla::from(rgb(design::ERROR_RED))
                        })
                        .child(result.clone()),
                )
            })
            .when(!sig.is_empty(), |c| {
                c.child(
                    design::toolbar().child(
                        Button::new("sm2-copy-sig")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制签名值")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm2_signature.clone();
                                this.copy(t, cx);
                            })),
                    ),
                )
            })
            .child(design::hint(
                "SM2 签名采用「用户标识 + 原文」的哈希方式，签名方与验签方必须约定相同的 DistID（默认 1234567812345678）。签名值为 64 字节的 r || s。若验签失败，请优先核对 DistID、原文是否被修改、公钥是否与私钥匹配。",
                cx,
            ))
    }

    fn render_sm2_pke(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let priv_state = self.sm2_private_state.clone();
        let pub_state = self.sm2_public_state.clone();
        let plain_state = self.sm2_plain_state.clone();
        let cipher_state = self.sm2_cipher_state.clone();
        let out_state = self.sm2_decrypted_state.clone();
        let decrypted = self.sm2_decrypted.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                design::toolbar()
                    .child(
                        Button::new("sm2p-gen")
                            .icon(Icon::new(IconName::RotateCw))
                            .tooltip("生成密钥对")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.gen_keypair(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("sm2p-derive")
                            .icon(Icon::new(IconName::ArrowRight))
                            .tooltip("由私钥推导公钥")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.public_from_private(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_4()
                    .child(Self::field("公钥", &pub_state, 92.0, cx))
                    .child(Self::field("私钥", &priv_state, 92.0, cx)),
            )
            .child(Self::field("明文", &plain_state, 96.0, cx))
            .child(
                design::toolbar()
                    .child(
                        Button::new("sm2p-enc")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("加密")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.do_sm2_encrypt(window, cx);
                            })),
                    )
                    .child(
                        Button::new("sm2p-dec")
                            .primary()
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解密")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.do_sm2_decrypt(window, cx);
                            })),
                    ),
            )
            .child(Self::field("密文", &cipher_state, 110.0, cx))
            .child(Self::field("解密结果", &out_state, 86.0, cx))
            .when(!decrypted.is_empty(), |c| {
                c.child(
                    design::toolbar().child(
                        Button::new("sm2p-copy")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制解密结果")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let t = this.sm2_decrypted.clone();
                                this.copy(t, cx);
                            })),
                    ),
                )
            })
            .child(design::hint(
                "SM2 公钥加密、私钥解密，密文由 C1（椭圆曲线点）、C3（摘要）、C2（密文数据）组成，默认按 C1C3C2 顺序输出。SM2 加密适合小数据量场景（如密钥交换、短报文），大文件请改用 SM4 并使用 SM2 加密其会话密钥。",
                cx,
            ))
    }
}

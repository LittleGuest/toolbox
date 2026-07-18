use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{SelectEvent, SelectState},
    *,
};

pub struct HashCalculator {
    input: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    sha3_256: String,
    sha3_512: String,
    is_calculating: bool,
    generation: u64,
    uppercase: bool,
    output_type: String,
    hmac_mode: bool,
    input_state: Entity<InputState>,
    secret_state: Entity<InputState>,
    output_type_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl HashCalculator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入文本...")
                .multi_line(true)
        });
        let secret_state = cx.new(|cx| InputState::new(window, cx).placeholder("HMAC 密钥..."));

        let output_type_items = vec!["十六进制(Hex)".to_string(), "Base64".to_string()];
        let output_type_state = cx.new(|cx| {
            let mut state = SelectState::new(output_type_items, None, window, cx);
            state.set_selected_value(&"十六进制(Hex)".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    this.calculate(cx);
                    cx.notify();
                }
            }),
            cx.subscribe_in(
                &output_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.output_type = match value.as_str() {
                            "Base64" => "base64",
                            _ => "hex",
                        }
                        .to_string();
                        this.calculate(cx);
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            input: String::new(),
            md5: String::new(),
            sha1: String::new(),
            sha256: String::new(),
            sha512: String::new(),
            sha3_256: String::new(),
            sha3_512: String::new(),
            is_calculating: false,
            generation: 0,
            uppercase: false,
            output_type: "hex".to_string(),
            hmac_mode: false,
            input_state,
            secret_state,
            output_type_state,
            _subscriptions,
        }
    }

    fn calculate(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.md5.clear();
            self.sha1.clear();
            self.sha256.clear();
            self.sha512.clear();
            self.sha3_256.clear();
            self.sha3_512.clear();
            self.is_calculating = false;
            return;
        }

        self.is_calculating = true;
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let input = self.input.clone();
        let uppercase = self.uppercase;
        let output_type = self.output_type.clone();
        let hmac_mode = self.hmac_mode;
        let secret = self.secret_state.read(cx).value().to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = base::hash(
                uppercase,
                if output_type == "base64" {
                    Some("base64")
                } else {
                    None
                },
                hmac_mode,
                if hmac_mode {
                    Some(secret.as_str())
                } else {
                    None
                },
                Some(&input),
            )
            .await
            .unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.md5 = result.get("md5").cloned().unwrap_or_default();
                this.sha1 = result.get("sha1").cloned().unwrap_or_default();
                this.sha256 = result.get("sha256").cloned().unwrap_or_default();
                this.sha512 = result.get("sha512").cloned().unwrap_or_default();
                this.sha3_256 = result.get("sha3_256").cloned().unwrap_or_default();
                this.sha3_512 = result.get("sha3_512").cloned().unwrap_or_default();
                this.is_calculating = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
                self.calculate(cx);
            }
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.md5.clear();
        self.sha1.clear();
        self.sha256.clear();
        self.sha512.clear();
        self.sha3_256.clear();
        self.sha3_512.clear();
        self.is_calculating = false;
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn set_uppercase(&mut self, uppercase: bool, cx: &mut Context<Self>) {
        self.uppercase = uppercase;
        self.calculate(cx);
    }

    fn set_hmac_mode(&mut self, hmac_mode: bool, cx: &mut Context<Self>) {
        self.hmac_mode = hmac_mode;
        self.calculate(cx);
    }
}

impl Render for HashCalculator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let md5 = self.md5.clone();
        let sha1 = self.sha1.clone();
        let sha256 = self.sha256.clone();
        let sha512 = self.sha512.clone();
        let sha3_256 = self.sha3_256.clone();
        let sha3_512 = self.sha3_512.clone();

        div().child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                // label "操作" → ButtonGroup (Paste+Copy)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(80.0)).text_sm().child("操作"))
                        .child(
                            ButtonGroup::new("input-buttons")
                                .child(
                                    Button::new("paste-input")
                                        .icon(Icon::new(IconName::File))
                                        .tooltip("粘贴")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste_input(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-input")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_input(cx);
                                        })),
                                ),
                        ),
                )
                // label "输入" → textarea
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(div().w(px(80.0)).text_sm().mt_1().child("输入"))
                        .child(
                            div()
                                .flex_1()
                                .child(Input::new(&self.input_state).h(px(100.0))),
                        ),
                )
                // hash result rows
                .child(hash_row(cx, "MD5", md5, "copy_md5", |this, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.md5.clone()));
                }))
                .child(hash_row(cx, "SHA1", sha1, "copy_sha1", |this, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.sha1.clone()));
                }))
                .child(hash_row(cx, "SHA256", sha256, "copy_sha256", |this, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.sha256.clone()));
                }))
                .child(hash_row(cx, "SHA512", sha512, "copy_sha512", |this, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.sha512.clone()));
                }))
                .child(hash_row(
                    cx,
                    "SHA3 256",
                    sha3_256,
                    "copy_sha3_256",
                    |this, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(this.sha3_256.clone()));
                    },
                ))
                .child(hash_row(
                    cx,
                    "SHA3 512",
                    sha3_512,
                    "copy_sha3_512",
                    |this, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(this.sha3_512.clone()));
                    },
                )),
        )
    }
}

fn hash_row(
    cx: &mut Context<HashCalculator>,
    label: &'static str,
    value: String,
    copy_id: &'static str,
    on_copy: fn(&mut HashCalculator, &mut Context<HashCalculator>),
) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w(px(80.0)).text_sm().child(label))
        .child(
            div()
                .flex_1()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .px_2()
                .py_1()
                .text_sm()
                .font_family("monospace")
                .child(if value.is_empty() {
                    "-".to_string()
                } else {
                    value
                }),
        )
        .child(
            Button::new(copy_id)
                .icon(Icon::new(IconName::Copy))
                .tooltip("复制")
                .on_click(cx.listener(move |this, _, _, cx| on_copy(this, cx))),
        )
}

use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
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
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl HashCalculator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入文本...")
                .multi_line(true)
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, _ev: &InputEvent, _window, cx| {
                let value = input_state.read(cx).value();
                this.input = value.to_string();
                this.calculate();
                cx.notify();
            }
        })];

        Self {
            input: String::new(),
            md5: String::new(),
            sha1: String::new(),
            sha256: String::new(),
            sha512: String::new(),
            sha3_256: String::new(),
            sha3_512: String::new(),
            input_state,
            _subscriptions,
        }
    }

    fn calculate(&mut self) {
        if self.input.is_empty() {
            self.md5.clear();
            self.sha1.clear();
            self.sha256.clear();
            self.sha512.clear();
            self.sha3_256.clear();
            self.sha3_512.clear();
            return;
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt
            .block_on(base::hash(false, None, false, Some(&self.input)))
            .unwrap_or_default();

        self.md5 = result.get("md5").cloned().unwrap_or_default();
        self.sha1 = result.get("sha1").cloned().unwrap_or_default();
        self.sha256 = result.get("sha256").cloned().unwrap_or_default();
        self.sha512 = result.get("sha512").cloned().unwrap_or_default();
        self.sha3_256 = result.get("sha3_256").cloned().unwrap_or_default();
        self.sha3_512 = result.get("sha3_512").cloned().unwrap_or_default();
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

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("文本Hash"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().child(Input::new(&self.input_state).h(px(100.0)))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("MD5"))
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
                                    .child(if md5.is_empty() {
                                        "-".to_string()
                                    } else {
                                        md5.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_md5")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.md5.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA1"))
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
                                    .child(if sha1.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha1.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha1")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha1.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA256"))
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
                                    .child(if sha256.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha256.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha256")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha256.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA512"))
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
                                    .child(if sha512.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha512.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha512")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha512.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA3 256"))
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
                                    .child(if sha3_256.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha3_256.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha3_256")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha3_256.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA3 512"))
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
                                    .child(if sha3_512.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha3_512.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha3_512")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha3_512.clone(),
                                        ));
                                    })),
                            ),
                    ),
            )
    }
}

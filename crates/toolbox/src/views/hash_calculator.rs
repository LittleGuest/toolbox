use gpui_kit::{
    component::{
        button::*,
        input::{InputEvent, Textarea, TextareaState},
        *,
    },
    prelude::FluentBuilder as _,
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
    input_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl HashCalculator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本..."));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _window, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    this.calculate(cx);
                    cx.notify();
                }
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
            is_calculating: false,
            generation: 0,
            input_state,
            _subscriptions,
        }
    }

    fn clear_results(&mut self) {
        self.md5.clear();
        self.sha1.clear();
        self.sha256.clear();
        self.sha512.clear();
        self.sha3_256.clear();
        self.sha3_512.clear();
    }

    fn calculate(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.clear_results();
            self.is_calculating = false;
            return;
        }

        self.is_calculating = true;
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let input = self.input.clone();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = ::base::hash(false, None, false, None, Some(&input))
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
                    state.set_value(text.to_string(), window, cx);
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
        self.clear_results();
        self.is_calculating = false;
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_value(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }

    fn hash_row(
        &self,
        id: &'static str,
        label: &'static str,
        value: &str,
        cx: &mut Context<Self>,
    ) -> Div {
        let value = value.to_string();
        let display = if value.is_empty() {
            "-".to_string()
        } else {
            value.clone()
        };
        div()
            .flex()
            .items_center()
            .gap_2p5()
            .child(
                div()
                    .w(px(88.0))
                    .flex_shrink_0()
                    .text_size(px(13.0))
                    .font_family("monospace")
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .px_3()
                    .py_1p5()
                    .text_sm()
                    .font_family("monospace")
                    .truncate()
                    .child(display),
            )
            .child(
                Button::new(id)
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Copy))
                    .tooltip("复制")
                    .disabled(value.is_empty())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_value(&value, cx);
                    })),
            )
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

        div().w_full().flex_col().gap_4().child(
            div()
                .w_full()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("输入", cx))
                        .child(
                            Textarea::new(&self.input_state)
                                .h(px(120.0))
                                .font_family("monospace"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .mt_2()
                                .child(
                                    Button::new("paste-input")
                                        .icon(Icon::new(IconName::Inbox))
                                        .tooltip("粘贴")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste_input(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-input")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .disabled(self.input.is_empty())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_input(cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-input")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear(window, cx);
                                        })),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex_col()
                        .gap_2p5()
                        .mt_2()
                        .child(design::editor_label("结果", cx))
                        .child(self.hash_row("copy-md5", "MD5", &md5, cx))
                        .child(self.hash_row("copy-sha1", "SHA1", &sha1, cx))
                        .child(self.hash_row("copy-sha256", "SHA256", &sha256, cx))
                        .child(self.hash_row("copy-sha512", "SHA512", &sha512, cx))
                        .child(self.hash_row("copy-sha3-256", "SHA3 256", &sha3_256, cx))
                        .child(self.hash_row("copy-sha3-512", "SHA3 512", &sha3_512, cx))
                        .when(self.is_calculating, |this| {
                            this.child(design::hint("计算中…", cx))
                        }),
                ),
        )
    }
}

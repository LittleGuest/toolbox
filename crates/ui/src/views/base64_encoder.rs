use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};

pub struct Base64Encoder {
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl Base64Encoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要编码的文本...")
        });
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("编码 / 解码结果将显示在这里...")
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.input = input_state.read(cx).value().to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            output: String::new(),
            error: String::new(),
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        match ::base::encode_base64_text(&self.input) {
            Ok(value) => {
                self.output = value;
                self.error.clear();
            }
            Err(e) => self.error = e.to_string(),
        }
        self.sync_output(window, cx);
        cx.notify();
    }

    fn decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.output.is_empty() {
            return;
        }
        match ::base::decode_base64_text(&self.output) {
            Ok(value) => {
                self.input = value;
                self.error.clear();
                let input = self.input.clone();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(input, window, cx);
                });
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn sync_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let output = self.output.clone();
        self.output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.error.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn paste_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.output = text.to_string();
                self.sync_output(window, cx);
                cx.notify();
            }
        }
    }

    /// 输入 / 输出编辑器（对应 tb-editor：标签 + 文本域 + 工具行）
    fn editor(
        &self,
        label: &'static str,
        state: &Entity<TextareaState>,
        side: &'static str,
        cx: &mut Context<Self>,
    ) -> Div {
        let (paste_id, copy_id, clear_id): (&'static str, &'static str, &'static str) =
            if side == "input" {
                ("input-paste", "input-copy", "input-clear")
            } else {
                ("output-paste", "output-copy", "output-clear")
            };
        div()
            .flex_col()
            .gap_1p5()
            .child(design::editor_label(label, cx))
            .child(
                Textarea::new(state)
                    .h(design::CODE_BOX_HEIGHT)
                    .font_family("monospace"),
            )
            .child(
                design::toolbar()
                    .child(
                        Button::new(paste_id)
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if side == "input" {
                                    this.paste_input(window, cx);
                                } else {
                                    this.paste_output(window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(copy_id)
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if side == "input" {
                                    this.copy_input(cx);
                                } else {
                                    this.copy_output(cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(clear_id)
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear(window, cx);
                            })),
                    ),
            )
    }
}

impl Render for Base64Encoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();

        design::page()
            .child(design::page_header(
                "Base64 编码 / 解码",
                "文本与 Base64 互相转换",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::Asterisk,
                        "Base64 文本",
                        "文本与 Base64 互相转换",
                        cx,
                    ))
                    .child(self.editor("输入", &self.input_state, "input", cx))
                    .child(
                        design::action_row()
                            .child(
                                Button::new("encode")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("编码")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.encode(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("decode")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("解码")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.decode(window, cx);
                                    })),
                            ),
                    )
                    .child(self.editor("输出", &self.output_state, "output", cx))
                    .when(!error.is_empty(), |card| {
                        card.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(error),
                        )
                    }),
            )
    }
}

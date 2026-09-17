use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{Input, InputEvent, InputState, Textarea, TextareaState },
    *,
};

pub struct Base64Encoder {
    input: String,
    output: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl Base64Encoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("请输入文本...")
                
        });
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("输出结果...")
                
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            output: String::new(),
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        self.output = ::base::encode_base64_text(&self.input).unwrap_or_default();
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.output.is_empty() {
            return;
        }
        let decoded = ::base::decode_base64_text(&self.output).unwrap_or_default();
        self.input = decoded.clone();
        self.input_state.update(cx, |state, cx| {
            state.set_value(decoded, window, cx);
        });
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
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
                self.output_state.update(cx, |state, cx| {
                    state.set_value(self.output.clone(), window, cx);
                });
            }
        }
    }
}

impl Render for Base64Encoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex_col()
            .gap_4()
            .child(design::page_header(
                "Base64 编码 / 解码",
                "在文本与 Base64 之间互相转换",
                cx,
            ))
            .child(
                // 输入卡片
                design::card(cx)
                    .child(
                        design::toolbar()
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
                            )
                            .child(
                                Button::new("clear-input")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    )
                    .child(
                        Textarea::new(&self.input_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    ),
            )
            .child(
                // 编码/解码操作行
                design::action_row()
                    .child(
                        Button::new("encode")
                            .label("编码")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encode(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode")
                            .label("解码")
                            .icon(Icon::new(IconName::ArrowUp))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decode(window, cx);
                            })),
                    ),
            )
            .child(
                // 输出卡片
                design::card(cx)
                    .child(
                        Textarea::new(&self.output_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-output")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_output(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-output")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-output")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    ),
            )
    }
}

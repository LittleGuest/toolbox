use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    *,
};

pub struct UrlEncoder {
    input: String,
    output: String,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl UrlEncoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入URL...")
                .multi_line(true)
        });
        let output_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输出结果...")
                .multi_line(true)
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
        self.output = base::encode_url(&self.input).unwrap_or_default();
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.output.is_empty() {
            return;
        }
        let decoded = base::decode_url(&self.output).unwrap_or_default();
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

impl Render for UrlEncoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                // Row 1: 操作 → Paste + Copy + Close (input)
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().w(px(80.0)).child("操作"))
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
                            )
                            .child(
                                Button::new("clear-input")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // Row 2: 输入 → textarea
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().w(px(80.0)).child("输入"))
                    .child(Input::new(&self.input_state).h(px(250.0))),
            )
            .child(
                // Row 3: 编码/解码 → ArrowDown + ArrowUp (匹配 Tauri 非primary)
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().w(px(80.0)).child("编码/解码"))
                    .child(
                        Button::new("encode")
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("编码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encode(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode")
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decode(window, cx);
                            })),
                    ),
            )
            .child(
                // Row 4: 操作 → Paste + Copy + Close (output)
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().w(px(80.0)).child("操作"))
                    .child(
                        ButtonGroup::new("output-buttons")
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
                            ),
                    ),
            )
            .child(
                // Row 5: 输出 → textarea
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().w(px(80.0)).child("输出"))
                    .child(Input::new(&self.output_state).h(px(250.0))),
            )
    }
}

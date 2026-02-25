use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    *,
};

pub struct JsonEditor {
    input: String,
    output: String,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl JsonEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入JSON...")
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

    fn format(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            return;
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&self.input) {
            self.output = serde_json::to_string_pretty(&parsed).unwrap_or_default();
        } else {
            self.output = "无效的JSON格式".to_string();
        }

        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn minify(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            return;
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&self.input) {
            self.output = serde_json::to_string(&parsed).unwrap_or_default();
        } else {
            self.output = "无效的JSON格式".to_string();
        }

        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
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

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }
}

impl Render for JsonEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("JSON 编辑器"))
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
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_sm().child("输入"))
                                    .child(
                                        ButtonGroup::new("input-buttons")
                                            .child(
                                                Button::new("paste-input")
                                                    .icon(Icon::new(IconName::File))
                                                    .tooltip("粘贴")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.paste(window, cx);
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("copy-input")
                                                    .icon(Icon::new(IconName::Copy))
                                                    .tooltip("复制")
                                                    .on_click(cx.listener(
                                                        |this, _, _, cx| {
                                                            this.copy_input(cx);
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("clear-input")
                                                    .icon(Icon::new(IconName::Delete))
                                                    .tooltip("清空")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.clear(window, cx);
                                                        },
                                                    )),
                                            ),
                                    ),
                            )
                            .child(Input::new(&self.input_state).h(px(200.0))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                Button::new("format")
                                    .primary()
                                    .icon(Icon::new(IconName::Asterisk))
                                    .tooltip("格式化")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.format(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("minify")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("压缩")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.minify(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_sm().child("输出"))
                                    .child(
                                        ButtonGroup::new("output-buttons")
                                            .child(
                                                Button::new("copy-output")
                                                    .icon(Icon::new(IconName::Copy))
                                                    .tooltip("复制")
                                                    .on_click(cx.listener(
                                                        |this, _, _, cx| {
                                                            this.copy_output(cx);
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("clear-output")
                                                    .icon(Icon::new(IconName::Delete))
                                                    .tooltip("清空")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.clear(window, cx);
                                                        },
                                                    )),
                                            ),
                                    ),
                            )
                            .child(Input::new(&self.output_state).h(px(200.0))),
                    ),
            )
    }
}

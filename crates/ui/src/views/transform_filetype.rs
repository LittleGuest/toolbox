use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct TransformFiletype {
    file_path: String,
    input: String,
    output: String,
    from_format: String,
    to_format: String,
    indent: u8,
    is_converting: bool,
    error: String,
    from_format_state: Entity<SelectState<Vec<String>>>,
    to_format_state: Entity<SelectState<Vec<String>>>,
    indent_state: Entity<SelectState<Vec<String>>>,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl TransformFiletype {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let format_items = vec!["JSON".to_string(), "YAML".to_string(), "TOML".to_string()];

        let from_format_state = cx.new(|cx| {
            let mut state = SelectState::new(format_items.clone(), None, window, cx);
            state.set_selected_value(&"JSON".to_string(), window, cx);
            state
        });
        let to_format_state = cx.new(|cx| {
            let mut state = SelectState::new(format_items, None, window, cx);
            state.set_selected_value(&"YAML".to_string(), window, cx);
            state
        });
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入内容或选择文件...")
                .multi_line(true)
        });
        let output_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("转换结果将显示在这里...")
                .multi_line(true)
        });

        let indent_items = vec!["2".to_string(), "4".to_string()];
        let indent_state = cx.new(|cx| {
            let mut state = SelectState::new(indent_items, None, window, cx);
            state.set_selected_value(&"2".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &from_format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.from_format = value.to_lowercase();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &to_format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.to_format = value.to_lowercase();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &indent_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.indent = value.parse::<u8>().unwrap_or(2);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    cx.notify();
                }
            }),
            cx.subscribe_in(&output_state, window, {
                let output_state = output_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = output_state.read(cx).value();
                    this.output = value.to_string();
                    cx.notify();
                }
            }),
        ];

        Self {
            file_path: String::new(),
            input: String::new(),
            output: String::new(),
            from_format: "json".to_string(),
            to_format: "yaml".to_string(),
            indent: 2,
            is_converting: false,
            error: String::new(),
            from_format_state,
            to_format_state,
            indent_state,
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn select_file(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择文件")
                .add_filter("配置文件", &["json", "yaml", "yml", "toml"])
                .pick_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let path = file.path().to_string_lossy().to_string();
                let content = file.read().await;
                let content_str = String::from_utf8_lossy(&content).to_string();

                let _ = this.update(cx, |this, cx| {
                    this.file_path = path;
                    this.input = content_str;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn convert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            self.error = "请输入内容".to_string();
            cx.notify();
            return;
        }

        self.is_converting = true;
        self.error.clear();
        cx.notify();

        let from = self.from_format.clone();
        let to = self.to_format.clone();
        let input = self.input.clone();
        let indent = self.indent;

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = base::cffc(indent, &from, &to, &input);

            let _ = this.update_in(cx, |this, window, cx| {
                this.is_converting = false;
                match result {
                    Ok(output) => {
                        this.output = output.clone();
                        this.output_state.update(cx, |state, cx| {
                            state.set_value(output, window, cx);
                        });
                        this.error.clear();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.output.clear();
                        this.output_state.update(cx, |state, cx| {
                            state.set_value("".to_string(), window, cx);
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.file_path.clear();
        self.input.clear();
        self.output.clear();
        self.error.clear();
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

    fn paste_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.output = text.to_string();
                self.output_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    /// 反向转换：把输出区内容灌入输入区，交换源/目标格式，再触发一次转换
    fn reverse_convert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let output = self.output_state.read(cx).value().to_string();
        if output.trim().is_empty() {
            self.error = "输出区为空，无法反向转换".to_string();
            cx.notify();
            return;
        }
        self.error.clear();

        // 把输出灌入输入
        self.input = output.clone();
        self.input_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });

        // 交换格式
        std::mem::swap(&mut self.from_format, &mut self.to_format);
        let from_label = self.from_format.to_uppercase();
        let to_label = self.to_format.to_uppercase();
        self.from_format_state.update(cx, |state, cx| {
            state.set_selected_value(&from_label, window, cx);
        });
        self.to_format_state.update(cx, |state, cx| {
            state.set_selected_value(&to_label, window, cx);
        });

        // 触发转换
        self.convert(window, cx);
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn swap_formats(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        std::mem::swap(&mut self.from_format, &mut self.to_format);

        let from_label = self.from_format.to_uppercase();
        let to_label = self.to_format.to_uppercase();

        self.from_format_state.update(cx, |state, cx| {
            state.set_selected_value(&from_label, window, cx);
        });
        self.to_format_state.update(cx, |state, cx| {
            state.set_selected_value(&to_label, window, cx);
        });

        cx.notify();
    }
}

impl Render for TransformFiletype {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let is_converting = self.is_converting;

        div().child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                // Row: 缩进 → Select
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("缩进"))
                        .child(Select::new(&self.indent_state)),
                )
                // Row: 输入文件类型 → Select
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("输入文件类型"))
                        .child(Select::new(&self.from_format_state)),
                )
                // Row: 操作 → Paste+Copy+Close (input)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("操作"))
                        .child(
                            ButtonGroup::new("input-buttons")
                                .child(
                                    Button::new("paste-input")
                                        .icon(Icon::new(IconName::File))
                                        .tooltip("粘贴")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-input")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                this.input.clone(),
                                            ));
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
                // Row: 输入 → textarea
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("输入"))
                        .child(Input::new(&self.input_state).h(px(200.0))),
                )
                // Row: 转换 → ArrowDown + ArrowUp (separate buttons)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("转换"))
                        .child(
                            Button::new("convert")
                                .primary()
                                .icon(Icon::new(IconName::ArrowDown))
                                .tooltip("转换")
                                .disabled(is_converting)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.convert(window, cx);
                                })),
                        )
                        .child(
                            Button::new("reverse-convert")
                                .primary()
                                .icon(Icon::new(IconName::ArrowUp))
                                .tooltip("反向转换")
                                .disabled(is_converting)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.reverse_convert(window, cx);
                                })),
                        )
                        .child(if is_converting {
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("转换中...")
                        } else {
                            div()
                        })
                        .child(if !error.is_empty() {
                            div().text_sm().text_color(rgb(0xff0000)).child(error)
                        } else {
                            div()
                        }),
                )
                // Row: 输出文件类型 → Select
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("输出文件类型"))
                        .child(Select::new(&self.to_format_state)),
                )
                // Row: 操作 → Paste+Copy+Close (output)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("操作"))
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
                                        .disabled(self.output.is_empty())
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
                // Row: 输出 → textarea
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(100.0)).text_sm().child("输出"))
                        .child(Input::new(&self.output_state).h(px(200.0))),
                ),
        )
    }
}

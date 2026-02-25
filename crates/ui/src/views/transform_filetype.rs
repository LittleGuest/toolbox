use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
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
    input_state: Entity<InputState>,
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
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
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
            input_state,
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

    fn convert(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
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

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = base::cffc(indent, &from, &to, &input);

            let _ = this.update(cx, |this, cx| {
                this.is_converting = false;
                match result {
                    Ok(output) => {
                        this.output = output;
                        this.error.clear();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.output.clear();
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
        let output_text = if self.output.is_empty() {
            "转换结果将显示在这里...".to_string()
        } else {
            self.output.clone()
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("文件格式转换"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("select_file")
                                    .primary()
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("选择文件")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.select_file(cx);
                                    })),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_3()
                                    .py_2()
                                    .text_sm()
                                    .font_family("monospace")
                                    .overflow_x_scrollbar()
                                    .child(if self.file_path.is_empty() {
                                        "请选择文件...".to_string()
                                    } else {
                                        self.file_path.clone()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_16().child("源格式"))
                            .child(Select::new(&self.from_format_state))
                            .child(
                                Button::new("swap")
                                    .icon(Icon::new(IconName::ArrowRight))
                                    .tooltip("交换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.swap_formats(window, cx);
                                    })),
                            )
                            .child(div().text_sm().child("目标格式"))
                            .child(Select::new(&self.to_format_state)),
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
                                                            cx.write_to_clipboard(
                                                                ClipboardItem::new_string(
                                                                    this.input.clone(),
                                                                ),
                                                            );
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
                            .gap_2()
                            .child(
                                Button::new("convert")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowRight))
                                    .tooltip("转换")
                                    .disabled(is_converting)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.convert(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(if is_converting {
                                div().text_sm().text_color(cx.theme().muted_foreground).child("转换中...")
                            } else {
                                div()
                            })
                            .child(if !error.is_empty() {
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xff0000))
                                    .child(error)
                            } else {
                                div()
                            }),
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
                                        Button::new("copy-output")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .disabled(self.output.is_empty())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_output(cx);
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(200.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(output_text),
                            ),
                    ),
            )
    }
}

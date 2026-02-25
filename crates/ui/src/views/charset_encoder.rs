use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct CharsetEncoder {
    input: String,
    output: String,
    input_type: String,
    target_charset: String,
    output_type: String,
    delimiter_type: String,
    custom_delimiter: String,
    base_format: String,
    byte_count: usize,
    char_count: usize,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
    input_type_state: Entity<SelectState<Vec<String>>>,
    target_charset_state: Entity<SelectState<Vec<String>>>,
    output_type_state: Entity<SelectState<Vec<String>>>,
    delimiter_state: Entity<SelectState<Vec<String>>>,
    base_format_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl CharsetEncoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入文本或编码数据...")
                .multi_line(true)
        });
        let output_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("转换结果...")
                .multi_line(true)
        });

        let input_type_items = vec![
            "文本".to_string(),
            "十六进制".to_string(),
            "十进制".to_string(),
            "八进制".to_string(),
            "二进制".to_string(),
        ];
        let target_charset_items = vec![
            "UTF-8".to_string(),
            "GBK".to_string(),
            "UTF-16BE".to_string(),
            "UTF-16LE".to_string(),
            "UTF-32BE".to_string(),
            "UTF-32LE".to_string(),
            "ASCII".to_string(),
        ];
        let output_type_items = vec![
            "十六进制(Hex)".to_string(),
            "十进制(Dec)".to_string(),
            "八进制(Oct)".to_string(),
            "二进制(Bin)".to_string(),
        ];
        let delimiter_items = vec![
            "空格".to_string(),
            ", ".to_string(),
            ": ".to_string(),
            "; ".to_string(),
            "换行".to_string(),
            "自定义".to_string(),
        ];
        let base_format_items = vec![
            "无".to_string(),
            "0x前缀".to_string(),
            "0b前缀".to_string(),
            "0o前缀".to_string(),
            "h后缀".to_string(),
        ];

        let input_type_state = cx.new(|cx| {
            let mut state = SelectState::new(input_type_items, None, window, cx);
            state.set_selected_value(&"文本".to_string(), window, cx);
            state
        });
        let target_charset_state = cx.new(|cx| {
            let mut state = SelectState::new(target_charset_items, None, window, cx);
            state.set_selected_value(&"UTF-8".to_string(), window, cx);
            state
        });
        let output_type_state = cx.new(|cx| {
            let mut state = SelectState::new(output_type_items, None, window, cx);
            state.set_selected_value(&"十六进制(Hex)".to_string(), window, cx);
            state
        });
        let delimiter_state = cx.new(|cx| {
            let mut state = SelectState::new(delimiter_items, None, window, cx);
            state.set_selected_value(&"空格".to_string(), window, cx);
            state
        });
        let base_format_state = cx.new(|cx| {
            let mut state = SelectState::new(base_format_items, None, window, cx);
            state.set_selected_value(&"无".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = input_state.read(cx).value();
                        this.input = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &input_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.input_type = match value.as_str() {
                            "文本" => "text",
                            "十六进制" => "hex",
                            "十进制" => "decimal",
                            "八进制" => "octal",
                            "二进制" => "binary",
                            _ => "text",
                        }.to_string();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &target_charset_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.target_charset = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &output_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.output_type = match value.as_str() {
                            "十六进制(Hex)" => "hex",
                            "十进制(Dec)" => "decimal",
                            "八进制(Oct)" => "octal",
                            "二进制(Bin)" => "binary",
                            _ => "hex",
                        }.to_string();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &delimiter_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.delimiter_type = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &base_format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.base_format = match value.as_str() {
                            "无" => "none",
                            "0x前缀" => "0x",
                            "0b前缀" => "0b",
                            "0o前缀" => "0o",
                            "h后缀" => "h",
                            _ => "none",
                        }.to_string();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            input: String::new(),
            output: String::new(),
            input_type: "text".to_string(),
            target_charset: "UTF-8".to_string(),
            output_type: "hex".to_string(),
            delimiter_type: "空格".to_string(),
            custom_delimiter: String::new(),
            base_format: "none".to_string(),
            byte_count: 0,
            char_count: 0,
            input_state,
            output_state,
            input_type_state,
            target_charset_state,
            output_type_state,
            delimiter_state,
            base_format_state,
            _subscriptions,
        }
    }

    fn get_delimiter(&self) -> String {
        match self.delimiter_type.as_str() {
            "空格" => " ".to_string(),
            ", " => ", ".to_string(),
            ": " => ": ".to_string(),
            "; " => "; ".to_string(),
            "换行" => "\n".to_string(),
            "自定义" => self.custom_delimiter.clone(),
            _ => " ".to_string(),
        }
    }

    fn convert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            self.output.clear();
            self.byte_count = 0;
            self.char_count = 0;
            self.output_state.update(cx, |state, cx| {
                state.set_value("".to_string(), window, cx);
            });
            return;
        }

        let delimiter = self.get_delimiter();
        let result = base::charset_encode(
            &self.input,
            &self.input_type,
            &self.target_charset,
            &self.output_type,
            &delimiter,
            &self.base_format,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );

        match result {
            Ok(result) => {
                self.output = result.output;
                self.byte_count = result.byte_count;
                self.char_count = result.char_count;
            }
            Err(e) => {
                self.output = format!("转换失败: {}", e);
                self.byte_count = 0;
                self.char_count = 0;
            }
        }

        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.byte_count = 0;
        self.char_count = 0;
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

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }
}

impl Render for CharsetEncoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("字符编码转换"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_sm().w(px(80.0)).child("输入类型"))
                            .child(Select::new(&self.input_type_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_sm().w(px(80.0)).child("目标编码"))
                            .child(Select::new(&self.target_charset_state)),
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
                            .child(Input::new(&self.input_state).h(px(150.0))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                Button::new("convert")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("转换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.convert(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_sm().w(px(80.0)).child("输出类型"))
                            .child(Select::new(&self.output_type_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_sm().w(px(80.0)).child("分隔符"))
                            .child(Select::new(&self.delimiter_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_sm().w(px(80.0)).child("进制格式"))
                            .child(Select::new(&self.base_format_state)),
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
                                            ),
                                    ),
                            )
                            .child(Input::new(&self.output_state).h(px(150.0))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("字节: {}", self.byte_count)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("字符: {}", self.char_count)),
                            ),
                    ),
            )
    }
}

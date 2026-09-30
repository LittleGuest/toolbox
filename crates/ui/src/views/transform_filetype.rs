use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};
use gpui_kit::prelude::FluentBuilder;

pub struct TransformFiletype {
    input: String,
    output: String,
    from_format: String,
    to_format: String,
    indent: u8,
    error: String,
    from_format_state: Entity<SelectState<Vec<String>>>,
    to_format_state: Entity<SelectState<Vec<String>>>,
    indent_state: Entity<SelectState<Vec<String>>>,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
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
            state.set_selected_value(&"JSON".to_string(), window, cx);
            state
        });
        let input_state = cx.new(|cx| TextareaState::new(window, cx));
        let output_state = cx.new(|cx| TextareaState::new(window, cx));

        let indent_items = vec!["2".to_string(), "4".to_string()];
        let indent_state = cx.new(|cx| {
            let mut state = SelectState::new(indent_items, None, window, cx);
            state.set_selected_value(&"4".to_string(), window, cx);
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
                        this.indent = value.parse::<u8>().unwrap_or(4);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.input = input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&output_state, window, {
                let output_state = output_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.output = output_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
        ];

        Self {
            input: String::new(),
            output: String::new(),
            from_format: "json".to_string(),
            to_format: "json".to_string(),
            indent: 4,
            error: String::new(),
            from_format_state,
            to_format_state,
            indent_state,
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                cx.notify();
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
                cx.notify();
            }
        }
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
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

    fn config_item(&self, label: &str, select: &Entity<SelectState<Vec<String>>>, w: f32, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(div().w(px(w)).child(Select::new(select)))
    }
}

impl Render for TransformFiletype {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = self.input.clone();
        let output = self.output.clone();
        let error = self.error.clone();

        design::page()
            .child(design::page_header("格式转换", "JSON / YAML / TOML 配置格式互转", cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::FileText,
                        "格式转换",
                        "JSON / YAML / TOML 互转",
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_4()
                            .child(self.config_item("缩进", &self.indent_state, 100.0, cx))
                            .child(self.config_item(
                                "输入文件类型",
                                &self.from_format_state,
                                150.0,
                                cx,
                            ))
                            .child(self.config_item(
                                "输出文件类型",
                                &self.to_format_state,
                                150.0,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输入", cx))
                            .child(
                                Textarea::new(&self.input_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("paste-input")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_input(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-input")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.copy_text(&input, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear")
                                            .ghost()
                                            .compact()
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
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("convert")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("转换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.do_convert(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("reverse-convert")
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("反向转换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.do_reverse(window, cx);
                                    })),
                            )
                            .when(!error.is_empty(), |row| {
                                row.child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                        .child(error),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输出", cx))
                            .child(
                                Textarea::new(&self.output_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("paste-output")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_output(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-output")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.copy_text(&output, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear-2")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Close))
                                            .tooltip("清除")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

impl TransformFiletype {
    fn do_convert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.input.trim().is_empty() {
            self.error = "请输入内容".to_string();
            cx.notify();
            return;
        }
        match ::base::cffc(self.indent, &self.from_format, &self.to_format, &self.input) {
            Ok(output) => {
                self.output = output.clone();
                self.output_state.update(cx, |state, cx| {
                    state.set_value(output, window, cx);
                });
            }
            Err(e) => {
                self.error = e.to_string();
            }
        }
        cx.notify();
    }

    fn do_reverse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.output.trim().is_empty() {
            self.error = "请先转换得到输出".to_string();
            cx.notify();
            return;
        }
        match ::base::cffc(self.indent, &self.to_format, &self.from_format, &self.output) {
            Ok(input) => {
                self.input = input.clone();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(input, window, cx);
                });
            }
            Err(e) => {
                self.error = e.to_string();
            }
        }
        cx.notify();
    }
}

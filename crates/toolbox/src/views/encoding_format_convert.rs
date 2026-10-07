use gpui_kit::{
    component::{
        button::*,
        input::{InputEvent, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

const AUTO: &str = "自动检测";

pub struct EncodingFormatConvert {
    source: String,
    target: String,
    input: String,
    output: String,
    error: String,
    info: String,
    detected: String,
    lossy: bool,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    source_state: Entity<SelectState<Vec<String>>>,
    target_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

fn source_options() -> Vec<String> {
    let mut items = vec![AUTO.to_string()];
    items.extend(::base::CHARSETS.iter().map(|s| (*s).to_string()));
    items
}

fn target_options() -> Vec<String> {
    ::base::CHARSETS.iter().map(|s| (*s).to_string()).collect()
}

impl EncodingFormatConvert {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let source_state = cx.new(|cx| {
            let mut state = SelectState::new(source_options(), None, window, cx);
            state.set_selected_value(&AUTO.to_string(), window, cx);
            state
        });
        let target_state = cx.new(|cx| {
            let mut state = SelectState::new(target_options(), None, window, cx);
            state.set_selected_value(&"UTF-8".to_string(), window, cx);
            state
        });
        let input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入或粘贴需要转换的文字"));
        let output_state = cx.new(|cx| TextareaState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(
                &source_state,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    let SelectEvent::Confirm(value) = ev;
                    if let Some(v) = value {
                        this.source = v.clone();
                        this.detected.clear();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &target_state,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    let SelectEvent::Confirm(value) = ev;
                    if let Some(v) = value {
                        this.target = v.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&input_state, window, |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.input = this.input_state.read(cx).value().to_string();
                    cx.notify();
                }
            }),
        ];

        Self {
            source: AUTO.to_string(),
            target: "UTF-8".to_string(),
            input: String::new(),
            output: String::new(),
            error: String::new(),
            info: String::new(),
            detected: String::new(),
            lossy: false,
            input_state,
            output_state,
            source_state,
            target_state,
            _subscriptions,
        }
    }

    fn set_textarea(
        state: &Entity<TextareaState>,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        state.update(cx, |s, cx| {
            s.set_value(value, window, cx);
        });
    }

    fn resolve_source(&self) -> String {
        if self.source == AUTO {
            ::base::best_guess(&self.input).unwrap_or_else(|| "UTF-8".to_string())
        } else {
            self.source.clone()
        }
    }

    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        self.info.clear();
        self.lossy = false;
        if self.input.is_empty() {
            self.error = "请输入需要转换的文字".to_string();
            cx.notify();
            return;
        }
        let source = self.resolve_source();
        self.detected = source.clone();
        match ::base::transcode(&self.input, &source, &self.target) {
            Ok(result) => {
                self.lossy = result.lossy;
                self.output = result.text.clone();
                Self::set_textarea(&self.output_state, result.text, window, cx);
                if result.lossy {
                    self.info = format!(
                        "{} → {} 完成，部分字符在目标字符集中不存在，已替换",
                        source, self.target
                    );
                } else {
                    self.info = format!("{} → {} 转换完成", source, self.target);
                }
            }
            Err(e) => self.error = e.to_string(),
        }
        cx.notify();
    }

    fn detect(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        self.info.clear();
        if self.input.is_empty() {
            self.error = "请输入需要检测的文字".to_string();
            cx.notify();
            return;
        }
        let guesses = ::base::detect(&self.input);
        if guesses.is_empty() {
            self.error = "未能识别字符集".to_string();
            cx.notify();
            return;
        }
        let best = &guesses[0];
        self.detected = best.charset.clone();
        self.source = AUTO.to_string();
        self.source_state.update(cx, |s, cx| {
            s.set_selected_value(&AUTO.to_string(), window, cx);
        });
        let mut lines = Vec::new();
        for (i, g) in guesses.iter().take(6).enumerate() {
            let mark = if i == 0 { "★" } else { " " };
            let preview: String = g.text.chars().take(24).collect();
            lines.push(format!(
                "{mark} {}  匹配度 {:.0}%  {}",
                g.charset,
                g.ratio * 100.0,
                preview
            ));
        }
        self.info = lines.join("\n");
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.error.clear();
        self.info.clear();
        self.detected.clear();
        self.lossy = false;
        Self::set_textarea(&self.input_state, String::new(), window, cx);
        Self::set_textarea(&self.output_state, String::new(), window, cx);
        cx.notify();
    }

    fn swap(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = if self.source == AUTO {
            self.resolve_source()
        } else {
            self.source.clone()
        };
        let next_source = self.target.clone();
        let next_target = source.clone();
        self.source = next_source.clone();
        self.target = next_target.clone();
        self.source_state.update(cx, |s, cx| {
            s.set_selected_value(&next_source, window, cx);
        });
        self.target_state.update(cx, |s, cx| {
            s.set_selected_value(&next_target, window, cx);
        });
        if !self.output.is_empty() {
            let carried = self.output.clone();
            self.input = carried.clone();
            self.output.clear();
            Self::set_textarea(&self.input_state, carried, window, cx);
            Self::set_textarea(&self.output_state, String::new(), window, cx);
        }
        self.error.clear();
        self.info = format!("已交换方向：{} → {}", self.source, self.target);
        cx.notify();
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        self.info = "已复制输入内容".to_string();
        cx.notify();
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if self.output.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        self.info = "已复制转换结果".to_string();
        cx.notify();
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            self.input = text.clone();
            Self::set_textarea(&self.input_state, text, window, cx);
            cx.notify();
        }
    }

    fn paste_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            self.output = text.clone();
            Self::set_textarea(&self.output_state, text, window, cx);
            cx.notify();
        }
    }
}

impl Render for EncodingFormatConvert {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let info = self.info.clone();
        let detected = self.detected.clone();
        let lossy = self.lossy;

        let config_row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_y_3()
            .gap_x_5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(gpui::black().opacity(0.65))
                            .child("源编码"),
                    )
                    .child(div().w(px(160.0)).child(Select::new(&self.source_state))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(gpui::black().opacity(0.65))
                            .child("目标编码"),
                    )
                    .child(div().w(px(160.0)).child(Select::new(&self.target_state)))
                    .child(
                        Button::new("efc-swap")
                            .icon(Icon::new(IconName::Replace))
                            .tooltip("交换方向")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.swap(window, cx);
                            })),
                    )
                    .when(!detected.is_empty(), |row| {
                        row.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::OK_GREEN)))
                                .child(format!("识别为 {detected}")),
                        )
                    }),
            );

        design::page().child(
            design::card(cx).child(
                div()
                    .w_full()
                    .flex_col()
                    .gap_4()
                    .child(config_row)
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
                                design::toolbar()
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
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_input(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("detect-charset")
                                            .icon(Icon::new(IconName::Search))
                                            .tooltip("检测字符集")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.detect(window, cx);
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
                        design::action_row().child(
                            Button::new("efc-run")
                                .primary()
                                .icon(Icon::new(IconName::ArrowDown))
                                .tooltip("开始转换")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.run(window, cx);
                                })),
                        ),
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
                                design::toolbar()
                                    .child(
                                        Button::new("paste-output")
                                            .icon(Icon::new(IconName::Inbox))
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
                                            .tooltip("清除")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                            })),
                                    ),
                            )
                            .when(!info.is_empty(), |col| {
                                col.child(design::hint(info.clone(), cx))
                            }),
                    )
                    .when(lossy, |page| {
                        page.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                                .child("注意：部分字符在目标字符集中无法表示，已替换为占位字符"),
                        )
                    })
                    .when(!error.is_empty(), |page| {
                        page.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(error),
                        )
                    }),
            ),
        )
    }
}

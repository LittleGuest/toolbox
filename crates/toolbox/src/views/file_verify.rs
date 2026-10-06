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

/// （显示名, base 类型值）：对齐 Vue checksumOptions
const ALGORITHMS: [(&str, &str); 9] = [
    ("md5sum", "md5sum"),
    ("sha1sum", "sha1sum"),
    ("sha224sum", "sha2_224sum"),
    ("sha256sum", "sha2_256sum"),
    ("sha384sum", "sha2_384sum"),
    ("sha512sum", "sha2_512sum"),
    ("sha3_256sum", "sha3_256sum"),
    ("sha3_384sum", "sha3_384sum"),
    ("sha3_512sum", "sha3_512sum"),
];

/// 配置项：12px 灰色 label + 控件（对应 tb-config-item）
fn config_item(label: &'static str, control: Div) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(0x5b6478))
                .child(label),
        )
        .child(control)
}

pub struct FileVerify {
    file_path: String,
    checksum: String,
    valid_value: String,
    algorithm: String,
    is_calculating: bool,
    error: String,
    algorithm_state: Entity<SelectState<Vec<String>>>,
    valid_value_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl FileVerify {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let algorithm_items: Vec<String> = ALGORITHMS.iter().map(|(l, _)| l.to_string()).collect();

        let algorithm_state = cx.new(|cx| {
            let mut state = SelectState::new(algorithm_items, None, window, cx);
            state.set_selected_value(&"md5sum".to_string(), window, cx);
            state
        });

        let valid_value_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("输入对比值..."));

        let _subscriptions = vec![
            cx.subscribe_in(
                &algorithm_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        if let Some((_, v)) = ALGORITHMS.iter().find(|(l, _)| l == label) {
                            this.algorithm = v.to_string();
                            cx.notify();
                        }
                    }
                },
            ),
            cx.subscribe_in(&valid_value_state, window, {
                let valid_value_state = valid_value_state.clone();
                move |this, _, ev: &InputEvent, _window, cx| {
                    if let InputEvent::Change = ev {
                        let value = valid_value_state.read(cx).value();
                        this.valid_value = value.to_string();
                        cx.notify();
                    }
                }
            }),
        ];

        Self {
            file_path: String::new(),
            checksum: String::new(),
            valid_value: String::new(),
            algorithm: "md5sum".to_string(),
            is_calculating: false,
            error: String::new(),
            algorithm_state,
            valid_value_state,
            _subscriptions,
        }
    }

    fn select_file(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择文件")
                .pick_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let path = file.path().to_string_lossy().to_string();
                let _ = this.update(cx, |this, cx| {
                    this.file_path = path;
                    this.checksum.clear();
                    this.error.clear();
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn calculate(&mut self, cx: &mut Context<Self>) {
        if self.file_path.is_empty() {
            self.error = "请先选择文件".to_string();
            cx.notify();
            return;
        }

        self.is_calculating = true;
        self.checksum.clear();
        self.error.clear();
        cx.notify();

        let file_path = self.file_path.clone();
        let algorithm = self.algorithm.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = ::base::checksum(&algorithm, &file_path).await;

            let _ = this.update(cx, |this, cx| {
                this.is_calculating = false;
                match result {
                    Ok(sum) => this.checksum = sum,
                    Err(e) => this.error = e.to_string(),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn paste_valid_value(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.valid_value = text.trim().to_string();
                self.valid_value_state.update(cx, |state, cx| {
                    state.set_value(self.valid_value.clone(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn copy_checksum(&mut self, cx: &mut Context<Self>) {
        if !self.checksum.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.checksum.clone()));
        }
    }

    fn copy_valid_value(&mut self, cx: &mut Context<Self>) {
        if !self.valid_value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.valid_value.clone()));
        }
    }
}

impl Render for FileVerify {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let checksum = self.checksum.clone();
        let valid_value = self.valid_value.clone();
        let is_calculating = self.is_calculating;
        let error = self.error.clone();
        // 对齐 Vue checksumMatched：有对比值即比较
        let matched = checksum == valid_value;
        let show_match = !valid_value.is_empty();

        div().w_full().flex_col().gap_4().child(
            div()
                .w_full()
                .flex_col()
                .gap_4()
                // tb-config-row：文件 + 选择按钮 + 路径
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(config_item(
                            "文件",
                            div().child(
                                Button::new("select-file")
                                    .icon(Icon::new(IconName::FolderOpen))
                                    .tooltip("选择")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.select_file(cx);
                                    })),
                            ),
                        ))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(13.0))
                                .font_family("monospace")
                                .text_color(if self.file_path.is_empty() {
                                    cx.theme().muted_foreground
                                } else {
                                    cx.theme().foreground
                                })
                                .truncate()
                                .child(if self.file_path.is_empty() {
                                    "尚未选择文件".to_string()
                                } else {
                                    self.file_path.clone()
                                }),
                        ),
                )
                // tb-config-row：校验算法 + 计算
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(config_item(
                            "校验算法",
                            div().w(px(160.0)).child(Select::new(&self.algorithm_state)),
                        ))
                        .child(
                            Button::new("calculate")
                                .icon(Icon::new(IconName::Asterisk))
                                .tooltip("计算")
                                .loading(is_calculating)
                                .disabled(is_calculating)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.calculate(cx);
                                })),
                        ),
                )
                // tb-editor：校验值（只读展示） + 复制
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("校验值", cx))
                        .child(
                            div()
                                .min_h(px(72.0))
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().background)
                                .px_3()
                                .py_2()
                                .text_sm()
                                .font_family("monospace")
                                .child(if is_calculating {
                                    "计算中...".to_string()
                                } else if checksum.is_empty() {
                                    "-".to_string()
                                } else {
                                    checksum.clone()
                                }),
                        )
                        .child(
                            div().flex().items_center().gap_2().mt_2().child(
                                Button::new("copy-checksum")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .disabled(checksum.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_checksum(cx);
                                    })),
                            ),
                        ),
                )
                // tb-editor：对比值 + 粘贴 / 复制
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("对比值", cx))
                        .child(
                            Textarea::new(&self.valid_value_state)
                                .h(px(72.0))
                                .font_family("monospace"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .mt_2()
                                .child(
                                    Button::new("paste-valid")
                                        .icon(Icon::new(IconName::Inbox))
                                        .tooltip("粘贴")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste_valid_value(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-valid")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .disabled(valid_value.is_empty())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_valid_value(cx);
                                        })),
                                ),
                        ),
                )
                // tb-action-row：比对结果大字（24px 绿/红）
                .when(show_match, |card| {
                    card.child(
                        div().flex().items_center().gap_2().child(
                            div()
                                .text_size(px(24.0))
                                .font_semibold()
                                .text_color(if matched {
                                    Hsla::from(rgb(design::OK_GREEN))
                                } else {
                                    Hsla::from(rgb(design::ERROR_RED))
                                })
                                .child(if matched { "一致" } else { "不一致" }),
                        ),
                    )
                })
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

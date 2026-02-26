use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct FileVerify {
    file_path: String,
    checksum: String,
    valid_value: String,
    algorithm: String,
    is_calculating: bool,
    algorithm_state: Entity<SelectState<Vec<String>>>,
    valid_value_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl FileVerify {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let algorithm_items = vec![
            "md5sum".to_string(),
            "sha1sum".to_string(),
            "sha2_224sum".to_string(),
            "sha2_256sum".to_string(),
            "sha2_384sum".to_string(),
            "sha2_512sum".to_string(),
            "sha3_256sum".to_string(),
            "sha3_384sum".to_string(),
            "sha3_512sum".to_string(),
        ];

        let algorithm_state = cx.new(|cx| {
            let mut state = SelectState::new(algorithm_items, None, window, cx);
            state.set_selected_value(&"md5sum".to_string(), window, cx);
            state
        });

        let valid_value_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入对比值...")
                .multi_line(true)
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &algorithm_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.algorithm = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&valid_value_state, window, {
                let valid_value_state = valid_value_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = valid_value_state.read(cx).value();
                    this.valid_value = value.to_string();
                    cx.notify();
                }
            }),
        ];

        Self {
            file_path: String::new(),
            checksum: String::new(),
            valid_value: String::new(),
            algorithm: "md5sum".to_string(),
            is_calculating: false,
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
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn paste_file_path(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.file_path = text.to_string();
                cx.notify();
            }
        }
    }

    fn calculate(&mut self, cx: &mut Context<Self>) {
        if self.file_path.is_empty() {
            return;
        }

        self.is_calculating = true;
        self.checksum.clear();
        cx.notify();

        let file_path = self.file_path.clone();
        let algorithm = self.algorithm.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = base::checksum(&algorithm, &file_path).await;

            let _ = this.update(cx, |this, cx| {
                this.checksum = result.unwrap_or_default();
                this.is_calculating = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn paste_valid_value(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.valid_value = text.to_string();
                self.valid_value_state.update(cx, |state, cx| {
                    state.set_value(self.valid_value.clone(), window, cx);
                });
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

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.file_path.clear();
        self.checksum.clear();
        self.valid_value.clear();
        self.is_calculating = false;
        self.valid_value_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for FileVerify {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let checksum = self.checksum.clone();
        let valid_value = self.valid_value.clone();
        let is_calculating = self.is_calculating;
        let is_matched = !checksum.is_empty() && !valid_value.is_empty() && checksum == valid_value;
        let is_not_matched = !checksum.is_empty() && !valid_value.is_empty() && checksum != valid_value;

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("文件校验"))
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
                            .child(div().text_sm().w_20().child("校验算法"))
                            .child(Select::new(&self.algorithm_state))
                            .child(
                                Button::new("calculate")
                                    .primary()
                                    .icon(Icon::new(IconName::Asterisk))
                                    .tooltip("计算")
                                    .disabled(is_calculating || self.file_path.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.calculate(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("校验值"))
                            .child(
                                ButtonGroup::new("checksum-buttons")
                                    .child(
                                        Button::new("paste_checksum")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.paste_file_path(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy_checksum")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .disabled(checksum.is_empty())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_checksum(cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .px_3()
                            .py_2()
                            .min_h(px(80.0))
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
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("对比值"))
                            .child(
                                ButtonGroup::new("valid-value-buttons")
                                    .child(
                                        Button::new("paste_valid")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_valid_value(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy_valid")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .disabled(valid_value.is_empty())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_valid_value(cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(Input::new(&self.valid_value_state).min_h(px(80.0)))
                    .child(if is_matched {
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0x18a058))
                            .child("一致")
                    } else if is_not_matched {
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0xd03050))
                            .child("不一致")
                    } else {
                        div()
                    })
            )
    }
}

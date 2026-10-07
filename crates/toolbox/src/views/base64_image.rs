use std::collections::HashMap;

use gpui_kit::{
    component::{
        button::*,
        input::{InputEvent, Textarea, TextareaState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

fn format_size(size: &str) -> String {
    let n: usize = size.parse().unwrap_or(0);
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.2} KB", n as f64 / 1024.0)
    } else {
        format!("{:.2} MB", n as f64 / 1024.0 / 1024.0)
    }
}

fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn chunk_lines(s: &str, width: usize, max_lines: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    chars
        .chunks(width)
        .take(max_lines)
        .map(|c| c.iter().collect())
        .collect()
}

fn info_tags(mime: &str, size: &str, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(design::mini_tag(
            mime,
            design::tint(design::ACCENT, 0.12),
            Hsla::from(rgb(design::ACCENT)),
        ))
        .child(design::mini_tag(
            format_size(size),
            Hsla::from(rgb(0xeef0f5)),
            cx.theme().muted_foreground,
        ))
}

fn preview_box(children: impl IntoIterator<Item = AnyElement>, cx: &App) -> Div {
    div()
        .w_full()
        .flex_col()
        .items_center()
        .justify_center()
        .min_h(px(96.0))
        .p_2()
        .gap_0p5()
        .border_dashed()
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(6.0))
        .bg(gpui::black().opacity(0.02))
        .overflow_hidden()
        .children(children)
}

pub struct Base64ImageConverter {
    file_path: String,
    encode_result: Option<HashMap<String, String>>,
    encode_loading: bool,
    encode_error: String,
    decode_input: String,
    decode_result: Option<HashMap<String, String>>,
    status: String,
    decode_input_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl Base64ImageConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let decode_input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("粘贴 Base64 字符串或 Data URL（data:image/png;base64,...）")
        });

        let _subscriptions = vec![cx.subscribe_in(&decode_input_state, window, {
            let decode_input_state = decode_input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.decode_input = decode_input_state.read(cx).value().to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            file_path: String::new(),
            encode_result: None,
            encode_loading: false,
            encode_error: String::new(),
            decode_input: String::new(),
            decode_result: None,
            status: String::new(),
            decode_input_state,
            _subscriptions,
        }
    }

    fn select_file(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择图片")
                .add_filter(
                    "图片",
                    &[
                        "png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "avif",
                    ],
                )
                .add_filter("所有文件", &["*"])
                .pick_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let path = file.path().to_string_lossy().to_string();
                let encode_path = path.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move { ::base::encode_base64_image(&encode_path) })
                    .await;

                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(map) => {
                            this.file_path = path;
                            this.encode_result = Some(map);
                            this.encode_error.clear();
                        }
                        Err(e) => {
                            this.encode_result = None;
                            this.encode_error = e.to_string();
                        }
                    }
                    this.encode_loading = false;
                    cx.notify();
                });
            } else {
                let _ = this.update(cx, |this, cx| {
                    this.encode_loading = false;
                    cx.notify();
                });
            }
        })
        .detach();

        self.encode_loading = true;
        cx.notify();
    }

    fn clear_encode(&mut self, cx: &mut Context<Self>) {
        self.file_path.clear();
        self.encode_result = None;
        self.encode_error.clear();
        cx.notify();
    }

    fn copy_data_url(&self, map: Option<&HashMap<String, String>>, cx: &mut Context<Self>) {
        if let Some(data_url) = map.and_then(|m| m.get("dataUrl")) {
            cx.write_to_clipboard(ClipboardItem::new_string(data_url.clone()));
        }
    }

    fn decode(&mut self, cx: &mut Context<Self>) {
        let input = self.decode_input.trim().to_string();
        if input.is_empty() {
            self.status = "请输入 Base64 或 Data URL".to_string();
            cx.notify();
            return;
        }
        match ::base::decode_base64_image(&input) {
            Ok(map) => {
                self.decode_result = Some(map);
                self.status.clear();
            }
            Err(e) => {
                self.decode_result = None;
                self.status = e.to_string();
            }
        }
        cx.notify();
    }

    fn paste_decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            self.decode_input = text.to_string();
            self.decode_input_state.update(cx, |state, cx| {
                state.set_value(self.decode_input.clone(), window, cx);
            });
            self.decode(cx);
        }
    }

    fn save_image(&mut self, cx: &mut Context<Self>) {
        if self.decode_result.is_none() {
            self.status = "请先解码".to_string();
            cx.notify();
            return;
        }
        let mime = self
            .decode_result
            .as_ref()
            .and_then(|m| m.get("mime").cloned())
            .unwrap_or_else(|| "image/png".to_string());
        let ext = mime
            .rsplit('/')
            .next()
            .unwrap_or("png")
            .replace("x-", "")
            .replace("svg+xml", "svg");
        let ext = if ext == "jpeg" {
            "jpg".to_string()
        } else {
            ext
        };

        let data = self.decode_input.clone();
        if data.trim().is_empty() {
            return;
        }

        let file_name = format!("image-{}", chrono_millis());
        let ext_clone = ext.clone();
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("保存图片")
                .add_filter("图片", &[ext_clone.as_str()])
                .set_file_name(file_name)
                .save_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let path = file.path().to_string_lossy().to_string();
                let save_path = path.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move { ::base::save_base64_image(&data, &save_path) })
                    .await;

                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(_) => {
                            this.status = format!("图片已保存: {path}");
                        }
                        Err(e) => {
                            this.status = format!("保存失败: {e}");
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn clear_decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decode_input.clear();
        self.decode_result = None;
        self.status.clear();
        self.decode_input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

fn chrono_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

impl Render for Base64ImageConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let encode_mime = self
            .encode_result
            .as_ref()
            .and_then(|m| m.get("mime").cloned())
            .unwrap_or_default();
        let encode_size = self
            .encode_result
            .as_ref()
            .and_then(|m| m.get("size").cloned())
            .unwrap_or_default();
        let encode_lines = self
            .encode_result
            .as_ref()
            .and_then(|m| m.get("dataUrl").cloned())
            .map(|url| chunk_lines(&url, 72, 5));
        let encode_chars = self
            .encode_result
            .as_ref()
            .and_then(|m| m.get("dataUrl"))
            .map(|url| url.chars().count())
            .unwrap_or(0);

        let decode_mime = self
            .decode_result
            .as_ref()
            .and_then(|m| m.get("mime").cloned())
            .unwrap_or_default();
        let decode_size = self
            .decode_result
            .as_ref()
            .and_then(|m| m.get("size").cloned())
            .unwrap_or_default();

        let file_label = if self.file_path.is_empty() {
            "未选择".to_string()
        } else {
            file_name(&self.file_path)
        };
        let encode_error = self.encode_error.clone();
        let encode_loading = self.encode_loading;
        let status = self.status.clone();
        let status_error = status.contains("失败");
        let status_warn = status.starts_with("请");

        let left = div()
            .flex_col()
            .gap_2p5()
            .child(design::editor_label("图片 → Base64", cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("select-image")
                            .success()
                            .icon(Icon::new(IconName::Frame))
                            .tooltip("选择图片")
                            .loading(encode_loading)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_file(cx);
                            })),
                    )
                    .child(
                        Button::new("clear-encode")
                            .ghost()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_encode(cx);
                            })),
                    )
                    .child(
                        div()
                            .max_w(px(220.0))
                            .flex_1()
                            .min_w_0()
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(file_label),
                    ),
            )
            .when_some(encode_lines, |this, lines| {
                this.child(preview_box(
                    lines
                        .into_iter()
                        .map(|line| {
                            div()
                                .w_full()
                                .text_xs()
                                .font_family("monospace")
                                .truncate()
                                .child(line)
                                .into_any_element()
                        })
                        .chain(std::iter::once(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("……共 {encode_chars} 字符，可复制查看全部"))
                                .into_any_element(),
                        )),
                    cx,
                ))
                .child(info_tags(&encode_mime, &encode_size, cx))
                .child(
                    div().flex().items_center().gap_2().child(
                        Button::new("copy-encode")
                            .ghost()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制 Data URL")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let map = this.encode_result.clone();
                                this.copy_data_url(map.as_ref(), cx);
                            })),
                    ),
                )
            })
            .when(!encode_error.is_empty(), |this| {
                this.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                        .child(encode_error),
                )
            });

        let right = div()
            .flex_col()
            .gap_2p5()
            .child(design::editor_label("Base64 → 图片", cx))
            .child(
                Textarea::new(&self.decode_input_state)
                    .h(px(140.0))
                    .font_family("monospace"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("decode")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("解码")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.decode(cx);
                            })),
                    )
                    .child(
                        Button::new("paste-decode")
                            .ghost()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste_decode(window, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-decode")
                            .ghost()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_decode(window, cx);
                            })),
                    ),
            )
            .when(self.decode_result.is_some(), |this| {
                this.child(preview_box(
                    std::iter::once(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("图片已解码（保存后可查看）")
                            .into_any_element(),
                    ),
                    cx,
                ))
                .child(info_tags(&decode_mime, &decode_size, cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new("save-image")
                                .ghost()
                                .icon(Icon::new(IconName::ArrowDown))
                                .tooltip("保存图片")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save_image(cx);
                                })),
                        )
                        .child(
                            Button::new("copy-decode")
                                .ghost()
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制 Data URL")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let map = this.decode_result.clone();
                                    this.copy_data_url(map.as_ref(), cx);
                                })),
                        ),
                )
            });

        div().w_full().flex_col().gap_4().child(
            div()
                .w_full()
                .flex_col()
                .gap_4()
                .child(div().grid().grid_cols(2).gap_6().child(left).child(right))
                .when(!status.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(if status_error {
                                Hsla::from(rgb(design::ERROR_RED))
                            } else if status_warn {
                                Hsla::from(rgb(design::WARN_AMBER))
                            } else {
                                Hsla::from(rgb(design::OK_GREEN))
                            })
                            .child(status),
                    )
                }),
        )
    }
}

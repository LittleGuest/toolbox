use crate::design;
use std::collections::HashMap;

use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
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

/// 将长 Data URL 按 76 字符一行拆分用于预览展示
fn chunk_lines(s: &str, width: usize, max_lines: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let lines: Vec<String> = chars
        .chunks(width)
        .take(max_lines)
        .map(|c| c.iter().collect())
        .collect();
    lines
}

pub struct Base64ImageConverter {
    file_path: String,
    encode_result: Option<HashMap<String, String>>,
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
                    let value = decode_input_state.read(cx).value();
                    this.decode_input = value.to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            file_path: String::new(),
            encode_result: None,
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
                    &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "avif"],
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
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn clear_encode(&mut self, cx: &mut Context<Self>) {
        self.file_path.clear();
        self.encode_result = None;
        self.encode_error.clear();
        cx.notify();
    }

    fn copy_encode(&mut self, cx: &mut Context<Self>) {
        if let Some(map) = &self.encode_result {
            if let Some(data_url) = map.get("dataUrl") {
                cx.write_to_clipboard(ClipboardItem::new_string(data_url.clone()));
            }
        }
    }

    fn decode(&mut self, cx: &mut Context<Self>) {
        let input = self.decode_input.trim().to_string();
        if input.is_empty() {
            return;
        }
        match ::base::decode_base64_image(&input) {
            Ok(map) => {
                self.decode_result = Some(map);
                self.status.clear();
            }
            Err(e) => {
                self.decode_result = None;
                self.status = format!("解码失败: {e}");
            }
        }
        cx.notify();
    }

    fn paste_decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.decode_input = text.to_string();
                self.decode_input_state.update(cx, |state, cx| {
                    state.set_value(self.decode_input.clone(), window, cx);
                });
                self.decode(cx);
            }
        }
    }

    fn save_image(&mut self, cx: &mut Context<Self>) {
        if self.decode_result.is_none() {
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
        let ext = if ext == "jpeg" { "jpg".to_string() } else { ext };

        let data = self.decode_input.clone();
        if data.trim().is_empty() {
            return;
        }

        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("保存图片")
                .add_filter("图片", &[ext.as_str()])
                .set_file_name(format!("image.{ext}"))
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
                    this.status = match result {
                        Ok(_) => format!("图片已保存: {path}"),
                        Err(e) => format!("保存失败: {e}"),
                    };
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn copy_decode(&mut self, cx: &mut Context<Self>) {
        if let Some(map) = &self.decode_result {
            if let Some(data_url) = map.get("dataUrl") {
                cx.write_to_clipboard(ClipboardItem::new_string(data_url.clone()));
            }
        }
    }

    fn clear_decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decode_input.clear();
        self.decode_result = None;
        self.status.clear();
        self.decode_input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }
}

impl Render for Base64ImageConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let encode_info = self.encode_result.as_ref().map(|map| {
            (
                map.get("mime").cloned().unwrap_or_default(),
                map.get("size").cloned().unwrap_or_default(),
            )
        });
        let encode_lines = self
            .encode_result
            .as_ref()
            .and_then(|map| map.get("dataUrl").cloned())
            .map(|url| chunk_lines(&url, 76, 60));
        let encode_char_count = self
            .encode_result
            .as_ref()
            .and_then(|map| map.get("dataUrl"))
            .map(|url| url.chars().count())
            .unwrap_or(0);

        let decode_info = self.decode_result.as_ref().map(|map| {
            (
                map.get("mime").cloned().unwrap_or_default(),
                map.get("size").cloned().unwrap_or_default(),
            )
        });

        design::page()
            .child(design::page_header("Base64 图片", "图片与 Base64 互转", cx))
            .child(
                // 整体卡片：内部布局保持原样
                design::card(cx)
                    // ---------- 图片 → Base64 ----------
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("select-image")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("选择图片")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.select_file(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-encode")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_encode(cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if self.file_path.is_empty() {
                                        "未选择".to_string()
                                    } else {
                                        file_name(&self.file_path)
                                    }),
                            )
                            .child(div().flex_1()),
                    )
                    .when_some(encode_info.clone(), |this, (mime, size)| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_sm().child(mime))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format_size(&size)),
                                ),
                        )
                    })
                    .when_some(encode_lines, |this, lines| {
                        this.child(
                            div()
                                .flex()
                                .items_start()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .rounded_md()
                                        .p_2()
                                        .gap_0p5()
                                        .overflow_hidden()
                                        .children(lines.into_iter().map(|line| {
                                            div().text_xs().child(line)
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!(
                                                    "……共 {} 字符，可复制查看全部",
                                                    encode_char_count
                                                )),
                                        ),
                                )
                                .child(
                                    Button::new("copy-encode")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制 Data URL")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_encode(cx);
                                        })),
                                ),
                        )
                    })
                    .when(!self.encode_error.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().danger)
                                .child(self.encode_error.clone()),
                        )
                    })
                    // ---------- Base64 → 图片 ----------
                    .child(
                        Textarea::new(&self.decode_input_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("decode")
                                    .label("解码")
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.decode(cx);
                                    })),
                            )
                            .child(
                                Button::new("paste-decode")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_decode(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("save-image")
                                    .label("保存图片")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.save_image(cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-decode")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制 Data URL")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_decode(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-decode")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_decode(window, cx);
                                    })),
                            ),
                    )
                    .when_some(decode_info, |this, (mime, size)| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_sm().child(mime))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format_size(&size)),
                                ),
                        )
                    })
                    .when(!self.status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(if self.status.starts_with("保存失败")
                                            || self.status.starts_with("解码失败")
                                        {
                                            cx.theme().danger
                                        } else {
                                            cx.theme().muted_foreground
                                        })
                                        .child(self.status.clone()),
                        )
                    }),
            )
    }
}

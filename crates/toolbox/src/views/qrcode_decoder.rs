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

const MODES: [&str; 3] = ["图片文件", "Base64 图片", "矩阵文本"];

pub struct QrCodeDecoder {
    mode: String,
    source_path: String,
    base64_input: String,
    matrix_input: String,
    results: Vec<::base::QrDecodeOutput>,
    selected: usize,
    error: String,
    info: String,
    base64_state: Entity<TextareaState>,
    matrix_state: Entity<TextareaState>,
    mode_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl QrCodeDecoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let base64_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("粘贴 Base64 图片数据（可含 data:image/png;base64, 前缀）")
        });
        let matrix_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("每行一个二维码行，1/0 或 X/空格表示黑白模块")
        });

        let mode_state = cx.new(|cx| {
            let items = MODES.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let mut state = SelectState::new(items, None, window, cx);
            state.set_selected_value(&"图片文件".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&base64_state, window, {
                let s = base64_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.base64_input = s.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&matrix_state, window, {
                let s = matrix_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.matrix_input = s.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &mode_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(v)) = ev {
                        this.mode = v.clone();
                        this.results.clear();
                        this.error.clear();
                        this.info.clear();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            mode: "图片文件".to_string(),
            source_path: String::new(),
            base64_input: String::new(),
            matrix_input: String::new(),
            results: Vec::new(),
            selected: 0,
            error: String::new(),
            info: String::new(),
            base64_state,
            matrix_state,
            mode_state,
            _subscriptions,
        }
    }

    fn clear(&mut self) {
        self.results.clear();
        self.error.clear();
        self.info.clear();
        self.selected = 0;
    }

    fn accept(&mut self, found: Result<Vec<::base::QrDecodeOutput>, String>) {
        self.clear();
        match found {
            Ok(list) => {
                let total = list.len();
                self.results = list;
                self.info = format!("识别到 {total} 个二维码");
            }
            Err(e) => self.error = e,
        }
    }

    fn pick_file(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择包含二维码的图片")
                .add_filter(
                    "图片",
                    &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif"],
                )
                .pick_file()
                .await
        });
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let path = file.path().to_string_lossy().to_string();
                let decoded = ::base::decode_file(&path).map_err(|e| e.to_string());
                let _ = this.update(cx, |this, cx| {
                    this.source_path = path;
                    this.accept(decoded);
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn decode_base64(&mut self) {
        let raw = self.base64_input.trim();
        let payload = match raw.find("base64,") {
            Some(idx) => &raw[idx + 7..],
            None => raw,
        };
        let compact: String = payload.chars().filter(|c| !c.is_whitespace()).collect();
        let decoded = ::base::decode_bytes(&compact, "Base64")
            .and_then(|bytes| ::base::decode_image_bytes(&bytes));
        self.accept(decoded.map_err(|e| e.to_string()));
    }

    fn decode_matrix(&mut self) {
        let text = self.matrix_input.trim();
        if text.is_empty() {
            self.error = "请粘贴二维码矩阵".to_string();
            self.results.clear();
            return;
        }
        let mut matrix: Vec<Vec<bool>> = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let cells: Vec<bool> = trimmed
                .chars()
                .filter(|c| !c.is_whitespace() && *c != ',')
                .map(|c| matches!(c, '1' | 'X' | 'x' | '#' | '*'))
                .collect();
            if !cells.is_empty() {
                matrix.push(cells);
            }
        }
        if matrix.is_empty() {
            self.error = "未能解析出有效矩阵，请使用 1/0 或 X/空格 表示模块".to_string();
            self.results.clear();
            return;
        }
        let width = matrix[0].len();
        if matrix.iter().any(|r| r.len() != width) {
            self.error = "矩阵各行长度不一致，请检查输入".to_string();
            self.results.clear();
            return;
        }
        let found = ::base::decode_matrix(&matrix).map_err(|e| e.to_string());
        self.accept(found);
    }

    fn copy(&mut self, text: String, cx: &mut Context<Self>) {
        if text.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.error.clear();
        self.info = "已复制到剪贴板".to_string();
        cx.notify();
    }

    fn label(text: &'static str, cx: &App) -> Div {
        div()
            .w(px(76.0))
            .flex_shrink_0()
            .text_size(px(12.0))
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }
}

impl Render for QrCodeDecoder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let info = self.info.clone();
        let mode = self.mode.clone();
        let source = self.source_path.clone();

        let input_area = match mode.as_str() {
            "Base64 图片" => div()
                .flex_col()
                .gap_1p5()
                .child(design::editor_label("Base64 图片数据", cx))
                .child(
                    Textarea::new(&self.base64_state)
                        .h(px(150.0))
                        .font_family("monospace"),
                )
                .child(
                    design::toolbar().child(
                        Button::new("qrdec-b64-run")
                            .primary()
                            .icon(Icon::new(IconName::Search))
                            .tooltip("开始解码")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.decode_base64();
                                cx.notify();
                            })),
                    ),
                )
                .into_any_element(),
            "矩阵文本" => div()
                .flex_col()
                .gap_1p5()
                .child(design::editor_label("二维码矩阵", cx))
                .child(
                    Textarea::new(&self.matrix_state)
                        .h(px(150.0))
                        .font_family("monospace"),
                )
                .child(
                    design::toolbar().child(
                        Button::new("qrdec-mtx-run")
                            .primary()
                            .icon(Icon::new(IconName::Search))
                            .tooltip("开始解码")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.decode_matrix();
                                cx.notify();
                            })),
                    ),
                )
                .into_any_element(),
            _ => div()
                .flex_col()
                .gap_2()
                .child(design::editor_label("图片文件", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            Button::new("qrdec-pick")
                                .primary()
                                .icon(Icon::new(IconName::FolderOpen))
                                .tooltip("选择图片")
                                .on_click(cx.listener(|this, _, _, cx| this.pick_file(cx))),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(if source.is_empty() {
                                    "未选择文件".to_string()
                                } else {
                                    source.clone()
                                }),
                        ),
                )
                .child(design::hint(
                    "支持 PNG / JPEG / GIF / WebP / BMP。识别会自动尝试灰度化、放大与反色，以提升小尺寸、低对比度或截图类图片的识别率。",
                    cx,
                ))
                .into_any_element(),
        };

        let cards = div()
            .flex_col()
            .gap_2()
            .children(self.results.iter().enumerate().map(|(i, r)| {
                let active = i == self.selected;
                let content = r.content.clone();
                let meta = format!(
                    "版本 {} · 纠错等级 {} · 掩码 {} · 模块 {}×{}",
                    r.version,
                    ::base::ecc_name(r.ecc_level),
                    r.mask,
                    r.module_count,
                    r.module_count
                );
                let corner = r
                    .bounds
                    .first()
                    .map(|(x, y)| format!("左上角坐标 ({x}, {y})"))
                    .unwrap_or_default();
                div()
                    .id(("qr-result", i))
                    .flex_col()
                    .gap_1p5()
                    .p_3()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(if active {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .bg(cx.theme().background)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = i;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("二维码 #{}", i + 1)),
                            )
                            .child(
                                Button::new(("qr-copy", i))
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制内容")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let t = this
                                            .results
                                            .get(i)
                                            .map(|r| r.content.clone())
                                            .unwrap_or_default();
                                        this.copy(t, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().foreground)
                            .child(if content.is_empty() {
                                "（识别到二维码，但内容解码失败，可能图案受损）".to_string()
                            } else {
                                content.clone()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(cx.theme().muted_foreground)
                            .child(meta),
                    )
                    .when(!corner.is_empty(), |c| {
                        c.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(cx.theme().muted_foreground)
                                .child(corner.clone()),
                        )
                    })
            }))
            .into_any_element();

        design::page().child(
            design::card(cx)
                .child(div().flex().items_center().gap_2().child(Self::label("输入方式", cx)).child(
                    div().w(px(180.0)).child(Select::new(&self.mode_state)),
                ))
                .child(input_area)
                .when(!info.is_empty(), |c| {
                    c.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::OK_GREEN)))
                            .child(info.clone()),
                    )
                })
                .when(!error.is_empty(), |c| {
                    c.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::ERROR_RED)))
                            .child(error.clone()),
                    )
                })
                .child(cards)
                .child(design::hint(
                    "解码失败常见原因：图片过小导致模块分辨率不足、存在强透视畸变、二维码被遮挡或污损、图像对比度过低。可将图片放大后重试，或改用更清晰的原始截图。",
                    cx,
                )),
        )
    }
}

use gpui_kit::{
    component::{
        button::*,
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder,
    *,
};

const TARGET_FORMATS: [&str; 17] = [
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "tga", "ppm", "pgm", "pnm", "hdr",
    "exr", "avif", "qoi", "ff",
];

pub struct ImageConvert {
    files: Vec<String>,
    output_format: String,
    output_dir: String,
    outputs: Vec<String>,
    is_converting: bool,
    error: String,
    format_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl ImageConvert {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let format_items: Vec<String> = TARGET_FORMATS.iter().map(|s| s.to_string()).collect();
        let format_state = cx.new(|cx| {
            let mut state = SelectState::new(format_items, None, window, cx);
            state.set_selected_value(&"png".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![cx.subscribe_in(
            &format_state,
            window,
            move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = ev {
                    this.output_format = value.clone();
                    cx.notify();
                }
            },
        )];

        Self {
            files: Vec::new(),
            output_format: "png".to_string(),
            output_dir: String::new(),
            outputs: Vec::new(),
            is_converting: false,
            error: String::new(),
            format_state,
            _subscriptions,
        }
    }

    fn pick_files(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择图片")
                .add_filter(
                    "图片",
                    &[
                        "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "tga", "ppm",
                        "pgm", "pnm", "hdr", "exr", "avif", "qoi", "ff",
                    ],
                )
                .pick_files()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(files) = task.await {
                let paths: Vec<String> = files
                    .iter()
                    .map(|f| f.path().to_string_lossy().to_string())
                    .collect();
                let default_dir = paths
                    .first()
                    .and_then(|p| std::path::Path::new(p).parent().map(|d| d.to_path_buf()))
                    .map(|d| d.to_string_lossy().to_string());

                let _ = this.update(cx, |this, cx| {
                    this.files = paths;
                    this.outputs.clear();
                    this.error.clear();
                    if this.output_dir.is_empty() {
                        if let Some(dir) = default_dir {
                            this.output_dir = dir;
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn pick_dir(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择输出目录")
                .pick_folder()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(dir) = task.await {
                let path = dir.path().to_string_lossy().to_string();
                let _ = this.update(cx, |this, cx| {
                    this.output_dir = path;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn convert(&mut self, cx: &mut Context<Self>) {
        if self.files.is_empty() {
            self.error = "请先选择图片文件".to_string();
            cx.notify();
            return;
        }
        if self.output_dir.trim().is_empty() {
            self.error = "请选择输出目录".to_string();
            cx.notify();
            return;
        }

        self.is_converting = true;
        self.error.clear();
        self.outputs.clear();
        cx.notify();

        let files = self.files.clone();
        let format = self.output_format.clone();
        let dir = self.output_dir.clone();

        let task = cx
            .background_executor()
            .spawn(async move { ::base::image_convert(&files, &format, &dir, None) });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.is_converting = false;
                match result {
                    Ok(outputs) => {
                        this.outputs = outputs;
                        this.error.clear();
                    }
                    Err(e) => {
                        this.outputs.clear();
                        this.error = e.to_string();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.files.clear();
        self.outputs.clear();
        self.error.clear();
        cx.notify();
    }

    fn file_name(path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string()
    }
}

impl Render for ImageConvert {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let output_label = format!("转换为 {}", self.output_format.to_uppercase());
        let file_count = self.files.len();

        design::page().child(
            design::card(cx)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mb_4()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(cx.theme().muted_foreground)
                                .child("目标格式"),
                        )
                        .child(div().w(px(140.0)).child(Select::new(&self.format_state))),
                )
                .child(
                    div()
                        .flex_col()
                        .gap_2()
                        .child(design::editor_label("所选图片", cx))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Button::new("pick-images")
                                        .primary()
                                        .icon(Icon::new(IconName::Plus))
                                        .tooltip("添加图片")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.pick_files(cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-images")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear(cx);
                                        })),
                                )
                                .when(file_count == 0, |row| {
                                    row.child(design::hint("请选择一张或多张图片", cx))
                                }),
                        )
                        .when(file_count > 0, |block| {
                            block.child(div().mt(px(12.0)).flex_col().gap_2().children(
                                self.files.iter().enumerate().map(|(i, path)| {
                                    let name = Self::file_name(path);
                                    div()
                                        .id(("image-file-row", i))
                                        .flex()
                                        .items_center()
                                        .rounded(px(6.0))
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .px_3()
                                        .py_2()
                                        .text_sm()
                                        .child(div().min_w_0().truncate().child(name))
                                }),
                            ))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mt_4()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(cx.theme().muted_foreground)
                                .child("输出目录"),
                        )
                        .child(
                            Button::new("pick-out-dir")
                                .ghost()
                                .icon(Icon::new(IconName::FolderOpen))
                                .tooltip("选择输出目录")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.pick_dir(cx);
                                })),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(cx.theme().border)
                                .px_3()
                                .py_1p5()
                                .text_sm()
                                .font_family("monospace")
                                .truncate()
                                .child(if self.output_dir.is_empty() {
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("输出目录路径")
                                } else {
                                    div().child(self.output_dir.clone())
                                }),
                        ),
                )
                .child(
                    design::action_row().child(
                        Button::new("convert-images")
                            .primary()
                            .icon(Icon::new(IconName::Replace))
                            .tooltip(output_label.clone())
                            .loading(self.is_converting)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.convert(cx);
                            })),
                    ),
                )
                .when(!error.is_empty(), |card| {
                    card.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(rgb(design::ERROR_RED))
                            .child(error),
                    )
                })
                .when(!self.outputs.is_empty(), |card| {
                    let total = self.outputs.len();
                    card.child(
                        div()
                            .mt_4()
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("转换结果", cx))
                            .child(
                                div()
                                    .flex_col()
                                    .rounded(px(6.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .font_semibold()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .bg(cx.theme().background)
                                            .child("输出文件"),
                                    )
                                    .children(self.outputs.iter().enumerate().map(|(i, path)| {
                                        div()
                                            .id(("image-output-row", i))
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .font_family("monospace")
                                            .when(i + 1 < total, |row| {
                                                row.border_b_1().border_color(cx.theme().border)
                                            })
                                            .child(div().min_w_0().truncate().child(path.clone()))
                                    })),
                            ),
                    )
                }),
        )
    }
}

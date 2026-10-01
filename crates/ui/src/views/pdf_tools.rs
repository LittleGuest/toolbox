use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{Input, InputState, NumberInput},
    select::{Select, SelectEvent, SelectState},
    *,
};
use gpui_kit::prelude::FluentBuilder;
use std::path::Path;

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// 内联状态反馈（对齐 Vue message.success / message.error 的内联替代）
fn status_line(success: &str, error: &str, cx: &App) -> Div {
    if !error.is_empty() {
        div()
            .text_size(px(12.5))
            .text_color(Hsla::from(rgb(design::ERROR_RED)))
            .child(error.to_string())
    } else if !success.is_empty() {
        div()
            .text_size(px(12.5))
            .text_color(Hsla::from(rgb(design::OK_GREEN)))
            .child(success.to_string())
    } else {
        div()
    }
}

/// PDF 源文件磁贴（对齐 PdfInputTile.vue / tb-file-tile），由调用方附加 .id(...).on_click(...)
fn pdf_input_tile(path: &str, placeholder: &str, cx: &App) -> Div {
    let picked = !path.is_empty();
    let name = if picked {
        file_name(path)
    } else {
        placeholder.to_string()
    };
    div()
        .flex()
        .items_center()
        .gap_3()
        .px(px(14.0))
        .py(px(11.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .hover(|s| {
            s.border_color(cx.theme().primary)
                .bg(cx.theme().primary.opacity(0.1))
        })
        .child(
            div()
                .w(px(36.0))
                .h(px(36.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(9.0))
                .bg(cx.theme().primary.opacity(0.1))
                .text_color(cx.theme().primary)
                .child(Icon::new(IconName::FileText).size(px(18.0))),
        )
        .child(
            div()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(cx.theme().muted_foreground)
                        .child(if picked {
                            "源文件".to_string()
                        } else {
                            "尚未选择".to_string()
                        }),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(if picked {
                            cx.theme().foreground
                        } else {
                            cx.theme().muted_foreground
                        })
                        .truncate()
                        .child(name),
                ),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(cx.theme().muted_foreground)
                .font_family("monospace")
                .child(".pdf"),
        )
}

async fn pick_pdf_file() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .set_title("选择 PDF")
        .add_filter("PDF", &["pdf"])
        .pick_file()
        .await
        .map(|f| f.path().to_string_lossy().to_string())
}

async fn save_pdf_file(default_name: String) -> Option<String> {
    rfd::AsyncFileDialog::new()
        .set_title("保存 PDF")
        .add_filter("PDF", &["pdf"])
        .set_file_name(&default_name)
        .save_file()
        .await
        .map(|f| f.path().to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// 图片转 PDF / 合并 PDF（共用：MultiFilePicker 布局 + 保存对话框）
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum PdfFromFilesKind {
    ImagesToPdf,
    Merge,
}

impl PdfFromFilesKind {
    fn title(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "图片转 PDF",
            Self::Merge => "合并 PDF",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "每张图片自动生成一页，按列表顺序排列",
            Self::Merge => "按列表顺序合并至少 2 个文件",
        }
    }

    fn header_icon(self) -> IconName {
        match self {
            Self::ImagesToPdf => IconName::Frame,
            Self::Merge => IconName::GalleryVerticalEnd,
        }
    }

    /// MultiFilePicker 的 title 属性
    fn picker_title(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "选择图片",
            Self::Merge => "选择 PDF",
        }
    }

    fn empty_hint(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "支持 PNG / JPG / WEBP / BMP / TIFF 等格式",
            Self::Merge => "可拖拽多个 PDF，自动按列表顺序合并",
        }
    }

    /// 拖放区右侧的扩展名展示（extensions.slice(0, 4)）
    fn ext_display(self) -> &'static str {
        match self {
            Self::ImagesToPdf => ".png / .jpg / .jpeg / .webp",
            Self::Merge => ".pdf",
        }
    }

    fn is_image(self) -> bool {
        matches!(self, Self::ImagesToPdf)
    }

    fn submit_label(self, busy: bool) -> &'static str {
        match self {
            Self::ImagesToPdf => {
                if busy {
                    "正在生成 PDF…"
                } else {
                    "生成 PDF"
                }
            }
            Self::Merge => {
                if busy {
                    "正在合并…"
                } else {
                    "合并 PDF"
                }
            }
        }
    }

    fn submit_icon(self) -> IconName {
        match self {
            Self::ImagesToPdf => IconName::Check,
            Self::Merge => IconName::GalleryVerticalEnd,
        }
    }

    fn success_text(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "PDF 已生成",
            Self::Merge => "合并成功",
        }
    }

    fn min_files(self) -> usize {
        match self {
            Self::ImagesToPdf => 1,
            Self::Merge => 2,
        }
    }

    fn min_files_hint(self) -> &'static str {
        match self {
            Self::ImagesToPdf => "请先添加图片",
            Self::Merge => "请至少添加两个 PDF",
        }
    }
}

pub struct PdfFromFiles {
    kind: PdfFromFilesKind,
    files: Vec<String>,
    busy: bool,
    success: String,
    error: String,
}

impl PdfFromFiles {
    pub fn new(kind: PdfFromFilesKind) -> Self {
        Self {
            kind,
            files: Vec::new(),
            busy: false,
            success: String::new(),
            error: String::new(),
        }
    }

    fn add_files(&mut self, cx: &mut Context<Self>) {
        let is_images = self.kind.is_image();
        let task = cx.background_executor().spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new().set_title("选择文件");
            dialog = if is_images {
                dialog.add_filter(
                    "图片",
                    &[
                        "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "tga", "ppm",
                        "pgm", "pnm", "hdr", "exr", "avif", "qoi", "ff",
                    ],
                )
            } else {
                dialog.add_filter("PDF", &["pdf"])
            };
            dialog.pick_files().await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(picked) = task.await {
                let paths: Vec<String> = picked
                    .iter()
                    .map(|f| f.path().to_string_lossy().to_string())
                    .collect();
                let _ = this.update(cx, |this, cx| {
                    for p in paths {
                        if !this.files.contains(&p) {
                            this.files.push(p);
                        }
                    }
                    this.success.clear();
                    this.error.clear();
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn remove_file(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.files.len() {
            self.files.remove(index);
            cx.notify();
        }
    }

    fn move_file(&mut self, index: usize, delta: i32, cx: &mut Context<Self>) {
        let target = index as i32 + delta;
        if target >= 0 && (target as usize) < self.files.len() {
            self.files.swap(index, target as usize);
            cx.notify();
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.files.clear();
        self.success.clear();
        self.error.clear();
        cx.notify();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        if self.files.len() < self.kind.min_files() {
            self.error = self.kind.min_files_hint().to_string();
            cx.notify();
            return;
        }

        self.busy = true;
        self.error.clear();
        self.success.clear();
        cx.notify();

        let files = self.files.clone();
        let default_name = match self.kind {
            PdfFromFilesKind::ImagesToPdf => "images.pdf".to_string(),
            PdfFromFilesKind::Merge => "merged.pdf".to_string(),
        };
        let kind = self.kind;

        let save_task = {
            let name = default_name.clone();
            cx.background_executor()
                .spawn(async move { save_pdf_file(name).await })
        };
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(output) = save_task.await else {
                let _ = this.update(cx, |this, cx| {
                    this.busy = false;
                    cx.notify();
                });
                return;
            };
            let result = {
                let output = output.clone();
                cx.background_executor()
                    .spawn(async move {
                        match kind {
                            PdfFromFilesKind::ImagesToPdf => ::base::images_to_pdf(&files, &output),
                            PdfFromFilesKind::Merge => ::base::pdf_merge(&files, &output),
                        }
                    })
                    .await
            };
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        this.success = kind.success_text().to_string();
                        this.error.clear();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.success.clear();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for PdfFromFiles {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = self.kind;
        let error = self.error.clone();
        let success = self.success.clone();
        let busy = self.busy;
        let count = self.files.len();

        // 拖放区（pfp-drop）
        let drop_zone = div()
            .id("pfp-drop")
            .flex()
            .items_center()
            .gap_3p5()
            .px_5()
            .py(px(18.0))
            .rounded(px(10.0))
            .border_1()
            .border_dashed()
            .border_color(cx.theme().border)
            .hover(|s| {
                s.border_color(cx.theme().primary)
                    .bg(cx.theme().primary.opacity(0.1))
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.add_files(cx);
            }))
            .child(
                div()
                    .w(px(40.0))
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .bg(cx.theme().primary.opacity(0.1))
                    .text_color(cx.theme().primary)
                    .child(Icon::new(IconName::Inbox).size(px(20.0))),
            )
            .child(
                div()
                    .flex_col()
                    .min_w_0()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("拖拽文件到这里"),
                    )
                    .child(design::hint(format!("或点击选择文件 · {}", kind.empty_hint()), cx)),
            )
            .child(
                div()
                    .ml_auto()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .font_family("monospace")
                    .child(kind.ext_display().to_string()),
            );

        // 文件列表（pfp-item）
        let mut list = div().flex_col().gap_1p5();
        let total = self.files.len();
        for (i, path) in self.files.iter().enumerate() {
            let name = file_name(path);
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(10.0))
                    .py_2()
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .min_w(px(70.0))
                            .flex_shrink_0()
                            .child(
                                Icon::new(if kind.is_image() {
                                    IconName::Frame
                                } else {
                                    IconName::FileText
                                })
                                .size(px(20.0))
                                .text_color(if kind.is_image() {
                                    Hsla::from(rgb(design::ACCENT))
                                } else {
                                    cx.theme().primary
                                }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{}", i + 1)),
                            ),
                    )
                    .child(div().flex_1().min_w_0().text_sm().truncate().child(name))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(
                                Button::new(("file-up", i))
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("上移")
                                    .disabled(i == 0)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.move_file(i, -1, cx);
                                    })),
                            )
                            .child(
                                Button::new(("file-down", i))
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("下移")
                                    .disabled(i + 1 >= total)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.move_file(i, 1, cx);
                                    })),
                            )
                            .child(
                                Button::new(("file-remove", i))
                                    .custom(
                                        ButtonCustomVariant::new(cx)
                                            .foreground(Hsla::from(rgb(design::ERROR_RED))),
                                    )
                                    .compact()
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("移除")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove_file(i, cx);
                                    })),
                            ),
                    ),
            );
        }

        // 继续添加（dashed 全宽按钮）
        let add_more = div()
            .id("pfp-add")
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .py_2()
            .rounded(px(10.0))
            .border_1()
            .border_dashed()
            .border_color(cx.theme().border)
            .hover(|s| {
                s.border_color(cx.theme().primary)
                    .bg(cx.theme().primary.opacity(0.05))
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.add_files(cx);
            }))
            .child(
                Icon::new(IconName::Plus)
                    .size(px(16.0))
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("继续添加"),
            );

        design::page()
            .child(design::page_header(kind.title(), kind.subtitle(), cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        kind.header_icon(),
                        kind.title(),
                        kind.subtitle(),
                        cx,
                    ))
                    // pfp-head：标题 + 计数胶囊 + 清空
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .mb_3()
                            .child(design::editor_label(kind.picker_title(), cx))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2p5()
                                    .when(count > 0, |row| {
                                        row.child(design::pill(format!("{} 个文件", count), cx))
                                            .child(
                                                Button::new("clear-files")
                                                    .custom(
                                                        ButtonCustomVariant::new(cx)
                                                            .foreground(Hsla::from(rgb(design::ERROR_RED))),
                                                    )
                                                    .compact()
                                                    .label("清空")
                                                    .icon(Icon::new(IconName::Close))
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.clear(cx);
                                                    })),
                                            )
                                    }),
                            ),
                    )
                    .child(drop_zone)
                    .when(count > 0, |card| card.child(list).child(add_more))
                    .when(
                        !error.is_empty() || !success.is_empty(),
                        |card| card.child(status_line(&success, &error, cx)),
                    )
                    .child(
                        design::submit_row().child(
                            Button::new("submit-job")
                                .primary()
                                .label(kind.submit_label(busy))
                                .icon(Icon::new(kind.submit_icon()))
                                .loading(busy)
                                .w_full()
                                .h(px(44.0))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.submit(cx);
                                })),
                        ),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// 编辑 PDF：删除 / 旋转 / 调整顺序
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct PageOp {
    no: u32,
    rotate: u32,
    delete: bool,
}

pub struct PdfEditTool {
    input: String,
    pages: Vec<PageOp>,
    busy: bool,
    saved: bool,
    success: String,
    error: String,
    total_state: Entity<InputState>,
}

impl PdfEditTool {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let total_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入总页数，如 10"));
        Self {
            input: String::new(),
            pages: Vec::new(),
            busy: false,
            saved: false,
            success: String::new(),
            error: String::new(),
            total_state,
        }
    }

    fn load_pages(&mut self, cx: &mut Context<Self>) {
        let value = self.total_state.read(cx).value().to_string();
        let Ok(n) = value.trim().parse::<u32>() else {
            self.error = "请输入有效页数".to_string();
            self.pages.clear();
            cx.notify();
            return;
        };
        if n == 0 || n > 5000 {
            self.error = "页数需在 1 - 5000 之间".to_string();
            self.pages.clear();
            cx.notify();
            return;
        }
        self.error.clear();
        self.success.clear();
        self.saved = false;
        self.pages = (1..=n)
            .map(|no| PageOp {
                no,
                rotate: 0,
                delete: false,
            })
            .collect();
        cx.notify();
    }

    fn toggle_delete(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(page) = self.pages.get_mut(index) {
            page.delete = !page.delete;
            cx.notify();
        }
    }

    fn rotate_page(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(page) = self.pages.get_mut(index) {
            page.rotate = (page.rotate + 90) % 360;
            cx.notify();
        }
    }

    fn move_page(&mut self, index: usize, delta: i32, cx: &mut Context<Self>) {
        let target = index as i32 + delta;
        if target >= 0 && (target as usize) < self.pages.len() {
            self.pages.swap(index, target as usize);
            cx.notify();
        }
    }

    fn export(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.error = "请选择 PDF".to_string();
            cx.notify();
            return;
        }
        if self.pages.is_empty() {
            self.error = "请先设置总页数并载入页面".to_string();
            cx.notify();
            return;
        }

        self.busy = true;
        self.error.clear();
        self.success.clear();
        cx.notify();

        let input = self.input.clone();
        let pages = self.pages.clone();

        let save_task = cx
            .background_executor()
            .spawn(async move { save_pdf_file("edited.pdf".to_string()).await });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(output) = save_task.await else {
                let _ = this.update(cx, |this, cx| {
                    this.busy = false;
                    cx.notify();
                });
                return;
            };
            let result = {
                let output = output.clone();
                cx.background_executor()
                    .spawn(async move {
                        let delete: Vec<u32> = pages
                            .iter()
                            .filter(|p| p.delete)
                            .map(|p| p.no)
                            .collect();
                        let rotate: Vec<(u32, f32)> = pages
                            .iter()
                            .filter(|p| p.rotate % 360 != 0)
                            .map(|p| (p.no, p.rotate as f32))
                            .collect();
                        let order: Vec<u32> = pages
                            .iter()
                            .filter(|p| !p.delete)
                            .map(|p| p.no)
                            .collect();
                        ::base::pdf_edit(&input, &output, &delete, &rotate, &order)
                    })
                    .await
            };
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        this.success = "编辑完成".to_string();
                        this.saved = true;
                        this.error.clear();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.success.clear();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for PdfEditTool {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let success = self.success.clone();
        let busy = self.busy;
        let saved = self.saved;

        let del = self.pages.iter().filter(|p| p.delete).count();
        let rot = self.pages.iter().filter(|p| p.rotate % 360 != 0).count();
        let keep = self.pages.len() - del;
        let total_text_filled = !self.total_state.read(cx).value().trim().is_empty();

        // 页面操作列表（ed-item）
        let mut list = div().flex_col().gap_1p5();
        let total = self.pages.len();
        for (i, page) in self.pages.iter().enumerate() {
            let deleted = page.delete;
            let rotated = page.rotate % 360 != 0;
            let rotate_deg = page.rotate;

            let tag = if deleted {
                design::mini_tag(
                    "已删除",
                    design::tint(design::ERROR_RED, 0.12),
                    Hsla::from(rgb(design::ERROR_RED)),
                )
            } else if rotated {
                design::mini_tag(
                    format!("旋转 {}°", rotate_deg),
                    design::tint(design::WARN_AMBER, 0.15),
                    Hsla::from(rgb(design::WARN_AMBER)),
                )
            } else {
                design::mini_tag(
                    "保留",
                    cx.theme().secondary,
                    cx.theme().muted_foreground,
                )
            };

            let rotate_btn = if rotated {
                Button::new(("page-rotate", i))
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(design::tint(design::WARN_AMBER, 0.15))
                            .foreground(Hsla::from(rgb(design::WARN_AMBER))),
                    )
                    .compact()
                    .icon(Icon::new(IconName::RotateCw))
                    .tooltip(format!("旋转 90°（当前 {}°）", rotate_deg))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.rotate_page(i, cx);
                    }))
            } else {
                Button::new(("page-rotate", i))
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::RotateCw))
                    .tooltip(format!("旋转 90°（当前 {}°）", rotate_deg))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.rotate_page(i, cx);
                    }))
            };

            let delete_btn = if deleted {
                Button::new(("page-delete", i))
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(design::tint(design::ERROR_RED, 0.12))
                            .foreground(Hsla::from(rgb(design::ERROR_RED))),
                    )
                    .compact()
                    .icon(Icon::new(IconName::CircleX))
                    .tooltip("撤销删除")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_delete(i, cx);
                    }))
            } else {
                Button::new(("page-delete", i))
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::CircleX))
                    .tooltip("删除此页")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_delete(i, cx);
                    }))
            };

            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(10.0))
                    .py_2()
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(if rotated {
                        design::tint(0xc89a4c, 0.5)
                    } else {
                        cx.theme().border
                    })
                    .bg(cx.theme().background)
                    .when(deleted, |row| row.opacity(0.55))
                    // 序号徽章（ed-item-seq）
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(8.0))
                            .flex_shrink_0()
                            .bg(if deleted {
                                design::tint(design::ERROR_RED, 0.12)
                            } else {
                                cx.theme().primary.opacity(0.1)
                            })
                            .text_color(if deleted {
                                Hsla::from(rgb(design::ERROR_RED))
                            } else {
                                cx.theme().primary
                            })
                            .text_size(px(13.0))
                            .font_semibold()
                            .child(format!("{}", i + 1)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex_col()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .when(deleted, |t| t.line_through())
                                            .child(format!("第 {} 页", page.no)),
                                    )
                                    .child(tag),
                            )
                            .child(
                                div()
                                    .mt(px(2.0))
                                    .text_size(px(11.5))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("原页 {}", page.no)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .flex_shrink_0()
                            .child(
                                Button::new(("page-up", i))
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("上移")
                                    .disabled(i == 0)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.move_page(i, -1, cx);
                                    })),
                            )
                            .child(
                                Button::new(("page-down", i))
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("下移")
                                    .disabled(i + 1 >= total)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.move_page(i, 1, cx);
                                    })),
                            )
                            .child(rotate_btn)
                            .child(delete_btn),
                    ),
            );
        }

        design::page()
            .child(design::page_header("编辑 PDF", "删除 / 旋转 / 调整页面顺序", cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::FileText,
                        "编辑 PDF",
                        "删除 / 旋转 / 调整页面顺序",
                        cx,
                    ))
                    // 源文件磁贴
                    .child(
                        div()
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("源文件", cx))
                            .child(
                                pdf_input_tile(&self.input, "点击或拖拽 PDF 到此处", cx)
                                    .id("edit-source-tile")
                                    .on_click(cx.listener(Self::pick_input_handler())),
                            ),
                    )
                    // 页面数
                    .child(
                        div()
                            .mt(px(18.0))
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("页面数", cx))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_center()
                                    .gap_2p5()
                                    .child(
                                        div()
                                            .w(px(160.0))
                                            .child(Input::new(&self.total_state)),
                                    )
                                    .child(
                                        Button::new("load-pages")
                                            .custom(
                                                ButtonCustomVariant::new(cx)
                                                    .color(cx.theme().primary.opacity(0.12))
                                                    .foreground(cx.theme().primary),
                                            )
                                            .label(if self.pages.is_empty() {
                                                "载入页面"
                                            } else {
                                                "重新载入"
                                            })
                                            .disabled(!total_text_filled)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.load_pages(cx);
                                            })),
                                    )
                                    .child(design::hint(
                                        "当前工具需手动指定总页数后逐页操作",
                                        cx,
                                    )),
                            ),
                    )
                    // 统计药丸 + 页面操作
                    .when(!self.pages.is_empty(), |card| {
                        card.child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_2p5()
                                .mt_4()
                                .child(design::stat_pill(
                                    format!("{}", self.pages.len()),
                                    "总页",
                                    cx.theme().muted_foreground,
                                    cx,
                                ))
                                .child(design::stat_pill(
                                    format!("{}", del),
                                    "删除",
                                    if del > 0 {
                                        Hsla::from(rgb(design::ERROR_RED))
                                    } else {
                                        cx.theme().muted_foreground
                                    },
                                    cx,
                                ))
                                .child(design::stat_pill(
                                    format!("{}", rot),
                                    "旋转",
                                    if rot > 0 {
                                        Hsla::from(rgb(design::ACCENT))
                                    } else {
                                        cx.theme().muted_foreground
                                    },
                                    cx,
                                ))
                                .child(design::stat_pill(
                                    format!("{}", keep),
                                    "保留",
                                    Hsla::from(rgb(design::OK_GREEN)),
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .mt(px(14.0))
                                .flex_col()
                                .gap_2()
                                .child(design::editor_label("页面操作", cx))
                                .child(list),
                        )
                    })
                    .when(
                        !error.is_empty() || !success.is_empty(),
                        |card| card.child(status_line(&success, &error, cx)),
                    )
                    // 三态提交按钮
                    .child(design::submit_row().child(
                        Button::new("export-edit")
                            .primary()
                            .label(if saved {
                                "已保存 √"
                            } else if busy {
                                "正在处理…"
                            } else {
                                "导出编辑后 PDF"
                            })
                            .icon(Icon::new(IconName::Check))
                            .loading(busy)
                            .disabled(busy || self.pages.is_empty())
                            .w_full()
                            .h(px(44.0))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.export(cx);
                            })),
                    )),
            )
    }
}

impl PdfEditTool {
    fn pick_input_handler(
    ) -> impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static {
        |this, _e, _w, cx| {
            let task = cx.background_executor().spawn(pick_pdf_file());
            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                if let Some(path) = task.await {
                    let _ = this.update(cx, |this, cx| {
                        this.input = path;
                        this.pages.clear();
                        this.saved = false;
                        this.success.clear();
                        this.error.clear();
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }
}

// ---------------------------------------------------------------------------
// PDF 添加页码
// ---------------------------------------------------------------------------

const POSITIONS: [(&str, &str); 9] = [
    ("顶部靠左", "top-left"),
    ("顶部居中", "top-center"),
    ("顶部靠右", "top-right"),
    ("中部靠左", "middle-left"),
    ("中部居中", "middle-center"),
    ("中部靠右", "middle-right"),
    ("底部靠左", "bottom-left"),
    ("底部居中", "bottom-center"),
    ("底部靠右", "bottom-right"),
];

fn position_index(pos: &str) -> usize {
    let row = match pos.split('-').next() {
        Some("top") => 0,
        Some("middle") => 3,
        _ => 6,
    };
    let col = match pos.split('-').nth(1) {
        Some("left") => 0,
        Some("center") => 1,
        _ => 2,
    };
    row + col
}

pub struct PdfPageNumberTool {
    input: String,
    position: &'static str,
    busy: bool,
    success: String,
    error: String,
    position_state: Entity<SelectState<Vec<String>>>,
    font_size_state: Entity<InputState>,
    pattern_state: Entity<InputState>,
    start_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl PdfPageNumberTool {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let labels: Vec<String> = POSITIONS.iter().map(|(l, _)| l.to_string()).collect();
        let position_state = cx.new(|cx| {
            let mut state = SelectState::new(labels, None, window, cx);
            state.set_selected_value(&"底部居中".to_string(), window, cx);
            state
        });
        let font_size_state =
            cx.new(|cx| InputState::new(window, cx).default_value("12".to_string()));
        let pattern_state =
            cx.new(|cx| InputState::new(window, cx).default_value("{n}".to_string()));
        let start_state =
            cx.new(|cx| InputState::new(window, cx).default_value("1".to_string()));

        let _subscriptions = vec![cx.subscribe_in(
            &position_state,
            window,
            move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                if let SelectEvent::Confirm(Some(label)) = ev {
                    if let Some((_, value)) = POSITIONS.iter().find(|(l, _)| l == label) {
                        this.position = value;
                        cx.notify();
                    }
                }
            },
        )];

        Self {
            input: String::new(),
            position: "bottom-center",
            busy: false,
            success: String::new(),
            error: String::new(),
            position_state,
            font_size_state,
            pattern_state,
            start_state,
            _subscriptions,
        }
    }

    fn add_numbers(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.error = "请选择 PDF".to_string();
            cx.notify();
            return;
        }

        let font_size = self
            .font_size_state
            .read(cx)
            .value()
            .trim()
            .parse::<f32>()
            .unwrap_or(12.0)
            .clamp(6.0, 72.0);
        let start_at = self
            .start_state
            .read(cx)
            .value()
            .trim()
            .parse::<i32>()
            .unwrap_or(1)
            .max(1);
        let pattern = self.pattern_state.read(cx).value().trim().to_string();
        let pattern = if pattern.is_empty() {
            "{n}".to_string()
        } else {
            pattern
        };
        let position = self.position;

        self.busy = true;
        self.error.clear();
        self.success.clear();
        cx.notify();

        let input = self.input.clone();
        let save_task = cx
            .background_executor()
            .spawn(async move { save_pdf_file("numbered.pdf".to_string()).await });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(output) = save_task.await else {
                let _ = this.update(cx, |this, cx| {
                    this.busy = false;
                    cx.notify();
                });
                return;
            };
            let result = {
                let output = output.clone();
                cx.background_executor()
                    .spawn(async move {
                        ::base::pdf_add_page_numbers(
                            &input,
                            &output,
                            position,
                            font_size,
                            &pattern,
                            start_at,
                        )
                    })
                    .await
            };
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        this.success = "页码添加完成".to_string();
                        this.error.clear();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.success.clear();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for PdfPageNumberTool {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let success = self.success.clone();
        let busy = self.busy;

        // 预览参数（sampleText / 字号钳制 / 活动方位）
        let font_size = self
            .font_size_state
            .read(cx)
            .value()
            .trim()
            .parse::<f32>()
            .unwrap_or(12.0);
        let preview_size = font_size.clamp(6.0, 26.0);
        let start_at = self
            .start_state
            .read(cx)
            .value()
            .trim()
            .parse::<i32>()
            .unwrap_or(1)
            .max(1);
        let pattern = self.pattern_state.read(cx).value().to_string();
        let sample_text = pattern
            .replace("{n}", &start_at.to_string())
            .replace("{cur}", "1")
            .replace("{total}", "10");
        let active_idx = position_index(self.position);

        // 3×3 方位选择按钮（pn-slot-btn）
        let slot_buttons = div()
            .grid()
            .grid_cols(3)
            .gap_2()
            .mt(px(14.0))
            .children(POSITIONS.iter().enumerate().map(|(i, (label, value))| {
                let value: &'static str = value;
                let label: &'static str = label;
                let active = self.position == value;
                div()
                    .id(("pn-slot", i))
                    .h(px(34.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(if active {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .bg(if active {
                        cx.theme().primary.opacity(0.1)
                    } else {
                        cx.theme().background
                    })
                    .hover(|s| s.border_color(cx.theme().primary))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.position = value;
                        this.position_state.update(cx, |state, cx| {
                            state.set_selected_value(&label.to_string(), window, cx);
                        });
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded(px(2.0))
                            .bg(if active {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            }),
                    )
            }));

        // 占位符说明（pn-code 高亮）
        let placeholder_hint = div()
            .mt_2()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .text_size(px(12.5))
            .text_color(cx.theme().muted_foreground)
            .child("占位符：")
            .child(
                div()
                    .font_family("monospace")
                    .font_semibold()
                    .text_color(cx.theme().primary)
                    .child("{n}"),
            )
            .child(" 起始页递增 · ")
            .child(
                div()
                    .font_family("monospace")
                    .font_semibold()
                    .text_color(cx.theme().primary)
                    .child("{cur}"),
            )
            .child(" 文档顺序号 · ")
            .child(
                div()
                    .font_family("monospace")
                    .font_semibold()
                    .text_color(cx.theme().primary)
                    .child("{total}"),
            )
            .child(" 总页数");

        design::page()
            .child(design::page_header(
                "PDF 添加页码",
                "在指定位置添加页码或自定义文本",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::FileText,
                        "添加页码",
                        "为 PDF 每页插入页码或自定义文本",
                        cx,
                    ))
                    // 源文件磁贴
                    .child(
                        div()
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("源文件", cx))
                            .child(
                                pdf_input_tile(&self.input, "点击或拖拽 PDF 到此处", cx)
                                    .id("number-source-tile")
                                    .on_click(cx.listener(Self::pick_input_handler())),
                            ),
                    )
                    // 页码设置：pn-grid（配置列 + 预览列）
                    .child(
                        div()
                            .mt(px(18.0))
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("页码设置", cx))
                            .child(
                                div()
                                    .flex()
                                    .items_stretch()
                                    .gap(px(18.0))
                                    .child(
                                        // 配置列
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .flex_col()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2p5()
                                                    .child(pn_label("位置"))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w(px(140.0))
                                                            .child(Select::new(
                                                                &self.position_state,
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2p5()
                                                    .child(pn_label("字号"))
                                                    .child(
                                                        div()
                                                            .w(px(110.0))
                                                            .child(NumberInput::new(
                                                                &self.font_size_state,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(13.0))
                                                            .text_color(
                                                                cx.theme().muted_foreground,
                                                            )
                                                            .child("pt"),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2p5()
                                                    .child(pn_label("起始页"))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w(px(110.0))
                                                            .child(NumberInput::new(
                                                                &self.start_state,
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2p5()
                                                    .child(pn_label("格式"))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w(px(140.0))
                                                            .child(Input::new(
                                                                &self.pattern_state,
                                                            )),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // 预览列（pn-preview）
                                        div()
                                            .w(px(220.0))
                                            .flex_shrink_0()
                                            .flex_col()
                                            .gap_2p5()
                                            .p_3()
                                            .rounded(px(10.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .bg(cx.theme().background)
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_semibold()
                                                    .text_color(
                                                        cx.theme().muted_foreground,
                                                    )
                                                    .child("预览"),
                                            )
                                            .child(
                                                // 3:4 页面（pn-preview-page）
                                                div()
                                                    .w_full()
                                                    .h(px(261.0))
                                                    .rounded(px(6.0))
                                                    .border_1()
                                                    .border_color(cx.theme().border)
                                                    .bg(rgb(0xffffff))
                                                    .overflow_hidden()
                                                    .grid()
                                                    .grid_cols(3)
                                                    .grid_rows(3)
                                                    .children((0..9).map(|k| {
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .min_w_0()
                                                            .min_h_0()
                                                            .when(k == active_idx, |cell| {
                                                                cell.child(
                                                                    div()
                                                                        .text_size(px(
                                                                            preview_size,
                                                                        ))
                                                                        .text_color(rgb(
                                                                            0x1e2433,
                                                                        ))
                                                                        .truncate()
                                                                        .child(
                                                                            sample_text
                                                                                .clone(),
                                                                        ),
                                                                )
                                                            })
                                                    })),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .child(
                                                        div()
                                                            .w(px(8.0))
                                                            .h(px(8.0))
                                                            .rounded_full()
                                                            .bg(cx.theme().primary),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.5))
                                                            .text_color(
                                                                cx.theme().muted_foreground,
                                                            )
                                                            .child(
                                                                "点击下方 9 个方位选择页码位置",
                                                            ),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(slot_buttons)
                            .child(placeholder_hint),
                    )
                    .when(
                        !error.is_empty() || !success.is_empty(),
                        |card| card.child(status_line(&success, &error, cx)),
                    )
                    .child(design::submit_row().child(
                        Button::new("add-numbers")
                            .primary()
                            .label(if busy { "正在添加…" } else { "添加页码" })
                            .icon(Icon::new(IconName::Check))
                            .loading(busy)
                            .w_full()
                            .h(px(44.0))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.add_numbers(cx);
                            })),
                    )),
            )
    }
}

fn pn_label(text: &'static str) -> Div {
    div()
        .w(px(56.0))
        .flex_shrink_0()
        .text_size(px(13.0))
        .text_color(rgb(0x5b6478))
        .child(text.to_string())
}

impl PdfPageNumberTool {
    fn pick_input_handler(
    ) -> impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static {
        |this, _e, _w, cx| {
            let task = cx.background_executor().spawn(pick_pdf_file());
            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                if let Some(path) = task.await {
                    let _ = this.update(cx, |this, cx| {
                        this.input = path;
                        this.success.clear();
                        this.error.clear();
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }
}

// ---------------------------------------------------------------------------
// PDF 拆分
// ---------------------------------------------------------------------------

/// 页码范围校验：返回 Ok(范围数) 或 Err(错误信息)
fn check_ranges(ranges: &str) -> Result<usize, String> {
    if ranges.trim().is_empty() {
        return Err("请填写页码范围".to_string());
    }
    let tokens: Vec<&str> = ranges
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return Err("页码范围不能为空".to_string());
    }
    let invalid: Vec<&str> = tokens
        .iter()
        .copied()
        .filter(|t| !is_range_token(t))
        .collect();
    if !invalid.is_empty() {
        return Err(format!("无法识别的范围：{}", invalid.join("、")));
    }
    Ok(tokens.len())
}

/// 匹配 ^\d+(-\d+)?$
fn is_range_token(t: &str) -> bool {
    let mut it = t.splitn(3, '-');
    match (it.next(), it.next(), it.next()) {
        (Some(a), None, _) => !a.is_empty() && a.chars().all(|c| c.is_ascii_digit()),
        (Some(a), Some(b), None) => {
            !a.is_empty()
                && a.chars().all(|c| c.is_ascii_digit())
                && !b.is_empty()
                && b.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

pub struct PdfSplitTool {
    input: String,
    outputs: Vec<String>,
    busy: bool,
    success: String,
    error: String,
    ranges_state: Entity<InputState>,
}

impl PdfSplitTool {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ranges_state =
            cx.new(|cx| InputState::new(window, cx).default_value("1-3,5,7-9".to_string()));
        Self {
            input: String::new(),
            outputs: Vec::new(),
            busy: false,
            success: String::new(),
            error: String::new(),
            ranges_state,
        }
    }

    fn split(&mut self, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.error = "请选择 PDF".to_string();
            cx.notify();
            return;
        }
        let ranges = self.ranges_state.read(cx).value().trim().to_string();
        if let Err(msg) = check_ranges(&ranges) {
            self.error = msg;
            cx.notify();
            return;
        }

        self.busy = true;
        self.error.clear();
        self.success.clear();
        cx.notify();

        let input = self.input.clone();
        let ranges_clone = ranges.clone();

        let dir_task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("选择输出目录")
                .pick_folder()
                .await
                .map(|f| f.path().to_string_lossy().to_string())
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(output_dir) = dir_task.await else {
                let _ = this.update(cx, |this, cx| {
                    this.busy = false;
                    cx.notify();
                });
                return;
            };
            let result = cx
                .background_executor()
                .spawn(async move { ::base::pdf_split(&input, &output_dir, &ranges_clone) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(outputs) => {
                        this.success = format!("已生成 {} 个文件", outputs.len());
                        this.outputs = outputs;
                        this.error.clear();
                    }
                    Err(e) => {
                        this.outputs.clear();
                        this.error = e.to_string();
                        this.success.clear();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for PdfSplitTool {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let success = self.success.clone();
        let busy = self.busy;

        // 实时范围校验（对齐 Vue computed rangeError）
        let ranges_now = self.ranges_state.read(cx).value().trim().to_string();
        let range_check = check_ranges(&ranges_now);

        design::page()
            .child(design::page_header(
                "拆分 PDF",
                "按页码范围将 PDF 拆分为多个文件",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::Minus,
                        "拆分 PDF",
                        "按页码范围将 PDF 拆分为多个文件",
                        cx,
                    ))
                    // 源文件磁贴
                    .child(
                        div()
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("源文件", cx))
                            .child(
                                pdf_input_tile(&self.input, "点击或拖拽 PDF 到此处", cx)
                                    .id("split-source-tile")
                                    .on_click(cx.listener(Self::pick_input_handler())),
                            ),
                    )
                    // 页码范围
                    .child(
                        div()
                            .mt(px(18.0))
                            .flex_col()
                            .gap_2()
                            .child(design::editor_label("页码范围", cx))
                            .child(
                                div()
                                    .font_family("monospace")
                                    .child(Input::new(&self.ranges_state)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_center()
                                    .gap_1()
                                    .text_size(px(12.5))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("每个范围输出为一个 PDF：单页用逗号分隔，连续页用连字符。例如")
                                    .child(
                                        div()
                                            .font_family("monospace")
                                            .font_semibold()
                                            .text_color(cx.theme().primary)
                                            .child("1-3, 5, 7-9"),
                                    ),
                            )
                            .children(match &range_check {
                                Err(msg) => vec![div()
                                    .text_size(px(12.5))
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child(msg.clone())],
                                Ok(n) if !ranges_now.is_empty() => vec![div()
                                    .text_size(px(12.5))
                                    .text_color(Hsla::from(rgb(design::OK_GREEN)))
                                    .child(format!(
                                        "格式有效，将按 {} 个范围分别输出",
                                        n
                                    ))],
                                _ => vec![],
                            }),
                    )
                    // 输出文件（out-item）
                    .when(!self.outputs.is_empty(), |card| {
                        card.child(
                            div()
                                .mt(px(20.0))
                                .flex_col()
                                .gap_2()
                                .child(design::editor_label(
                                    format!("已生成 {} 个文件", self.outputs.len()),
                                    cx,
                                ))
                                .child(div().flex_col().gap_1p5().children(
                                    self.outputs.iter().enumerate().map(|(i, path)| {
                                        let name = file_name(path);
                                        div()
                                            .id(("split-output-row", i))
                                            .flex()
                                            .items_center()
                                            .gap_2p5()
                                            .px(px(10.0))
                                            .py_2()
                                            .rounded(px(10.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .bg(cx.theme().background)
                                            .child(
                                                div()
                                                    .w(px(22.0))
                                                    .h(px(22.0))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(6.0))
                                                    .bg(design::tint(design::OK_GREEN, 0.12))
                                                    .text_color(Hsla::from(rgb(design::OK_GREEN)))
                                                    .child(
                                                        Icon::new(IconName::Check)
                                                            .size(px(14.0)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.0))
                                                    .min_w_0()
                                                    .truncate()
                                                    .child(name),
                                            )
                                    }),
                                )),
                        )
                    })
                    .when(
                        !error.is_empty() || !success.is_empty(),
                        |card| card.child(status_line(&success, &error, cx)),
                    )
                    .child(design::submit_row().child(
                        Button::new("split-pdf")
                            .primary()
                            .label(if busy { "正在拆分…" } else { "拆分 PDF" })
                            .icon(Icon::new(IconName::Minus))
                            .loading(busy)
                            .w_full()
                            .h(px(44.0))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.split(cx);
                            })),
                    )),
            )
    }
}

impl PdfSplitTool {
    fn pick_input_handler(
    ) -> impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static {
        |this, _e, _w, cx| {
            let task = cx.background_executor().spawn(pick_pdf_file());
            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                if let Some(path) = task.await {
                    let _ = this.update(cx, |this, cx| {
                        this.input = path;
                        this.outputs.clear();
                        this.success.clear();
                        this.error.clear();
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }
}

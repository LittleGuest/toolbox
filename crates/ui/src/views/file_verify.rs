use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};

pub struct FileVerify {
    file_path: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    is_calculating: bool,
}

impl FileVerify {
    pub fn new() -> Self {
        Self {
            file_path: String::new(),
            md5: String::new(),
            sha1: String::new(),
            sha256: String::new(),
            sha512: String::new(),
            is_calculating: false,
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

    fn calculate(&mut self, cx: &mut Context<Self>) {
        if self.file_path.is_empty() {
            return;
        }

        self.is_calculating = true;
        self.md5.clear();
        self.sha1.clear();
        self.sha256.clear();
        self.sha512.clear();
        cx.notify();

        let file_path = self.file_path.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let md5_result = rt.block_on(base::checksum("md5sum", &file_path));
            let sha1_result = rt.block_on(base::checksum("sha1sum", &file_path));
            let sha256_result = rt.block_on(base::checksum("sha2_256sum", &file_path));
            let sha512_result = rt.block_on(base::checksum("sha2_512sum", &file_path));

            let _ = this.update(cx, |this, cx| {
                this.md5 = md5_result.unwrap_or_default();
                this.sha1 = sha1_result.unwrap_or_default();
                this.sha256 = sha256_result.unwrap_or_default();
                this.sha512 = sha512_result.unwrap_or_default();
                this.is_calculating = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.file_path.clear();
        self.md5.clear();
        self.sha1.clear();
        self.sha256.clear();
        self.sha512.clear();
        self.is_calculating = false;
        cx.notify();
    }
}

impl Render for FileVerify {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let md5 = self.md5.clone();
        let sha1 = self.sha1.clone();
        let sha256 = self.sha256.clone();
        let sha512 = self.sha512.clone();
        let is_calculating = self.is_calculating;

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
                            .child(
                                Button::new("calculate")
                                    .primary()
                                    .icon(Icon::new(IconName::Asterisk))
                                    .tooltip("计算哈希")
                                    .disabled(is_calculating || self.file_path.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.calculate(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear(cx);
                                    })),
                            )
                            .child(if is_calculating {
                                div().text_sm().text_color(cx.theme().muted_foreground).child("计算中...")
                            } else {
                                div()
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("MD5"))
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .font_family("monospace")
                                    .overflow_x_scrollbar()
                                    .child(if md5.is_empty() {
                                        "-".to_string()
                                    } else {
                                        md5.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_md5")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .disabled(md5.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.md5.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA1"))
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .font_family("monospace")
                                    .overflow_x_scrollbar()
                                    .child(if sha1.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha1.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha1")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .disabled(sha1.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha1.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA256"))
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .font_family("monospace")
                                    .overflow_x_scrollbar()
                                    .child(if sha256.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha256.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha256")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .disabled(sha256.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha256.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("SHA512"))
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .font_family("monospace")
                                    .overflow_x_scrollbar()
                                    .child(if sha512.is_empty() {
                                        "-".to_string()
                                    } else {
                                        sha512.clone()
                                    }),
                            )
                            .child(
                                Button::new("copy_sha512")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .disabled(sha512.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.sha512.clone(),
                                        ));
                                    })),
                            ),
                    ),
            )
    }
}

use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};
use md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512, Digest};

#[derive(Clone, Debug)]
pub struct HashResult {
    algorithm: String,
    hash: String,
}

pub struct FileVerify {
    input_text: String,
    results: Vec<HashResult>,
}

impl FileVerify {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
            results: Vec::new(),
        }
    }

    fn calculate_hashes(&mut self) {
        if self.input_text.trim().is_empty() {
            self.results.clear();
            return;
        }

        let text = self.input_text.as_bytes();
        let mut results = Vec::new();

        let md5_hash = {
            use md5::Digest;
            let mut hasher = md5::Md5::new();
            md5::Digest::update(&mut hasher, text);
            format!("{:x}", md5::Digest::finalize(hasher))
        };
        results.push(HashResult {
            algorithm: "MD5".to_string(),
            hash: md5_hash,
        });

        let sha1_hash = {
            use sha1::Digest;
            let mut hasher = sha1::Sha1::new();
            sha1::Digest::update(&mut hasher, text);
            format!("{:x}", sha1::Digest::finalize(hasher))
        };
        results.push(HashResult {
            algorithm: "SHA1".to_string(),
            hash: sha1_hash,
        });

        let sha256_hash = {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            sha2::Digest::update(&mut hasher, text);
            format!("{:x}", sha2::Digest::finalize(hasher))
        };
        results.push(HashResult {
            algorithm: "SHA256".to_string(),
            hash: sha256_hash,
        });

        let sha512_hash = {
            use sha2::Digest;
            let mut hasher = sha2::Sha512::new();
            sha2::Digest::update(&mut hasher, text);
            format!("{:x}", sha2::Digest::finalize(hasher))
        };
        results.push(HashResult {
            algorithm: "SHA512".to_string(),
            hash: sha512_hash,
        });

        self.results = results;
    }

    fn clear(&mut self) {
        self.input_text.clear();
        self.results.clear();
    }
}

impl Render for FileVerify {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_text = if self.input_text.is_empty() {
            "输入文本或文件内制..".to_string()
        } else {
            self.input_text.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("文件校验")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("输入")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(100.0))
                                    .max_h(px(100.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(input_text)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("计算哈希")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("清空")
                            )
                    )
                    .child(if !self.results.is_empty() {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .child("哈希结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(200.0))
                                    .max_h(px(200.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_4()
                                    .overflow_y_scrollbar()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(
                                                self.results.iter().map(|result| {
                                                    div()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_semibold()
                                                                        .child(result.algorithm.clone())
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_family("monospace")
                                                                        .child(result.hash.clone())
                                                                )
                                                        )
                                                })
                                            )
                                    )
                            )
                    } else {
                        div()
                    })
            )
    }
}
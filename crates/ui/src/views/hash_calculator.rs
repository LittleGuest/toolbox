use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};

pub struct HashCalculator {
    input: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    sha3_256: String,
    sha3_512: String,
}

impl HashCalculator {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            md5: String::new(),
            sha1: String::new(),
            sha256: String::new(),
            sha512: String::new(),
            sha3_256: String::new(),
            sha3_512: String::new(),
        }
    }

    fn calculate(&mut self) {
        if self.input.is_empty() {
            self.md5.clear();
            self.sha1.clear();
            self.sha256.clear();
            self.sha512.clear();
            self.sha3_256.clear();
            self.sha3_512.clear();
            return;
        }

        let data = self.input.as_bytes();
        
        self.md5 = self.calculate_md5(data);
        self.sha1 = self.calculate_sha1(data);
        self.sha256 = self.calculate_sha256(data);
        self.sha512 = self.calculate_sha512(data);
        self.sha3_256 = self.calculate_sha3_256(data);
        self.sha3_512 = self.calculate_sha3_512(data);
    }

    fn clear(&mut self) {
        self.input.clear();
        self.md5.clear();
        self.sha1.clear();
        self.sha256.clear();
        self.sha512.clear();
        self.sha3_256.clear();
        self.sha3_512.clear();
    }

    fn calculate_md5(&self, data: &[u8]) -> String {
        use md5::Digest;
        let mut hasher = md5::Md5::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_sha1(&self, data: &[u8]) -> String {
        use sha1::Digest;
        let mut hasher = sha1::Sha1::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_sha512(&self, data: &[u8]) -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha512::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_sha3_256(&self, data: &[u8]) -> String {
        use sha3::Digest;
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_sha3_512(&self, data: &[u8]) -> String {
        use sha3::Digest;
        let mut hasher = sha3::Sha3_512::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

impl Render for HashCalculator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let md5 = self.md5.clone();
        let sha1 = self.sha1.clone();
        let sha256 = self.sha256.clone();
        let sha512 = self.sha512.clone();
        let sha3_256 = self.sha3_256.clone();
        let sha3_512 = self.sha3_512.clone();
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("文本Hash")
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
                                    .min_h(px(100.0))
                                    .max_h(px(100.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(if self.input.is_empty() { "请输入文本...".to_string() } else { self.input.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("calculate")
                                    .primary()
                                    .label("计算")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.calculate();
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("clear")
                                    .label("清空")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear();
                                        cx.notify();
                                    }))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("MD5")
                            )
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
                                    .child(if md5.is_empty() { "-".to_string() } else { md5.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SHA1")
                            )
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
                                    .child(if sha1.is_empty() { "-".to_string() } else { sha1.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SHA256")
                            )
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
                                    .child(if sha256.is_empty() { "-".to_string() } else { sha256.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SHA512")
                            )
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
                                    .child(if sha512.is_empty() { "-".to_string() } else { sha512.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SHA3 256")
                            )
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
                                    .child(if sha3_256.is_empty() { "-".to_string() } else { sha3_256.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SHA3 512")
                            )
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
                                    .child(if sha3_512.is_empty() { "-".to_string() } else { sha3_512.clone() })
                            )
                    )
            )
    }
}

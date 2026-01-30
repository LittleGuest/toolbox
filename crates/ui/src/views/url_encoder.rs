use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};

pub struct UrlEncoder {
    input: String,
    encoded: String,
    decoded: String,
}

impl UrlEncoder {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            encoded: String::new(),
            decoded: String::new(),
        }
    }

    fn encode(&mut self) {
        if self.input.is_empty() {
            self.encoded.clear();
            self.decoded.clear();
            return;
        }
        self.encoded = base::encode_url(&self.input).unwrap_or_default();
        self.decoded.clear();
    }

    fn decode(&mut self) {
        if self.input.is_empty() {
            self.encoded.clear();
            self.decoded.clear();
            return;
        }
        self.decoded = base::decode_url(&self.input).unwrap_or_default();
        self.encoded.clear();
    }

    fn clear(&mut self) {
        self.input.clear();
        self.encoded.clear();
        self.decoded.clear();
    }
}

impl Render for UrlEncoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let encoded = self.encoded.clone();
        let decoded = self.decoded.clone();
        
        let encoded_text = if encoded.is_empty() {
            "-".to_string()
        } else {
            encoded.clone()
        };
        
        let decoded_text = if decoded.is_empty() {
            "-".to_string()
        } else {
            decoded.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("URL 编解码")
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
                                    .child(if self.input.is_empty() { "请输入URL...".to_string() } else { self.input.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("encode")
                                    .primary()
                                    .label("编码")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.encode();
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("decode")
                                    .primary()
                                    .label("解码")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.decode();
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
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("编码结果")
                            )
                            .child(
                                div()
                                    .min_h(px(80.0))
                                    .max_h(px(80.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(encoded_text)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("解码结果")
                            )
                            .child(
                                div()
                                    .min_h(px(80.0))
                                    .max_h(px(80.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(decoded_text)
                            )
                    )
            )
    }
}
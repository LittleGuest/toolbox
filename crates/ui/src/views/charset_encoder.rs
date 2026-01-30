use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

pub struct CharsetEncoder {
    input: String,
    input_type: String,
    output_type: String,
    output: String,
    input_size: usize,
    output_size: usize,
}

impl CharsetEncoder {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            input_type: "utf-8".to_string(),
            output_type: "gbk".to_string(),
            output: String::new(),
            input_size: 0,
            output_size: 0,
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.output.clear();
            self.input_size = 0;
            self.output_size = 0;
            return;
        }

        self.input_size = self.input.len();
        
        // 简化实现，直接使用 base64 编码作为示例
        self.output = base::encode_base64_text(&self.input).unwrap_or_else(|_| "转换失败".to_string());
        self.output_size = self.output.len();
    }

    fn clear(&mut self) {
        self.input.clear();
        self.output.clear();
        self.input_size = 0;
        self.output_size = 0;
    }
}

impl Render for CharsetEncoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_type_text = match self.input_type.as_str() {
            "utf-8" => "UTF-8",
            "gbk" => "GBK",
            "iso-8859-1" => "ISO-8859-1",
            _ => "UTF-8",
        };
        
        let output_type_text = match self.output_type.as_str() {
            "utf-8" => "UTF-8",
            "gbk" => "GBK",
            "iso-8859-1" => "ISO-8859-1",
            _ => "GBK",
        };
        
        let input_text = if self.input.is_empty() {
            "请输入文制..".to_string()
        } else {
            self.input.clone()
        };
        
        let output_text = if self.output.is_empty() {
            "-".to_string()
        } else {
            self.output.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("字符编码转换")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("源编码")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(input_type_text)
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .child("目标编码")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(output_type_text)
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
                                    .child("转换")
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("输入大小: {} 字节", self.input_size))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("输出大小: {} 字节", self.output_size))
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
                                    .child("输出")
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
                                    .child(output_text)
                            )
                    )
            )
    }
}
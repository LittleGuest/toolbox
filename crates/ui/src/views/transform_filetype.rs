use gpui::*;
use gpui_component::{button::*, *};

pub struct TransformFiletype {
    input: String,
    output: String,
    from_format: String,
    to_format: String,
}

impl TransformFiletype {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            from_format: "json".to_string(),
            to_format: "yaml".to_string(),
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.output.clear();
            return;
        }

        // Simple conversion logic
        // In a real implementation, you would use proper conversion libraries
        if self.from_format == "json" && self.to_format == "yaml" {
            // Simulate conversion
            self.output = format!("# Converted from JSON\n{}", self.input);
        } else if self.from_format == "yaml" && self.to_format == "json" {
            // Simulate conversion
            self.output = format!("// Converted from YAML\n{}", self.input);
        } else {
            self.output = "转换格式不支持".to_string();
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.output.clear();
    }
}

impl Render for TransformFiletype {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("文件格式转换")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .child("从")
                                    )
                                    .child(
                                        div()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_md()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .child(self.from_format.clone())
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
                                            .child("到")
                                    )
                                    .child(
                                        div()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_md()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .child(self.to_format.clone())
                                    )
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
                                    .min_h(px(200.0))
                                    .max_h(px(200.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(self.input.clone())
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                Button::new("convert")
                                    .primary()
                                    .label("转换")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.convert();
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
                                    .child("输出")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(200.0))
                                    .max_h(px(200.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(if self.output.is_empty() { "-".to_string() } else { self.output.clone() })
                            )
                    )
            )
    }
}

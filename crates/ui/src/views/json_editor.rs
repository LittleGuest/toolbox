use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};
use serde_json;

pub struct JsonEditor {
    input: String,
    formatted: String,
}

impl JsonEditor {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            formatted: String::new(),
        }
    }

    fn format(&mut self) {
        if self.input.trim().is_empty() {
            self.formatted.clear();
            return;
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&self.input) {
            self.formatted = serde_json::to_string_pretty(&parsed).unwrap_or_default();
        } else {
            self.formatted = "无效的JSON格式".to_string();
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.formatted.clear();
    }
}

impl Render for JsonEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let formatted = self.formatted.clone();
        
        let formatted_text = if formatted.is_empty() {
            "-".to_string()
        } else {
            formatted.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("JSON Editor")
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
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(if self.input.is_empty() { "请输入JSON...".to_string() } else { self.input.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("format")
                                    .primary()
                                    .label("格式化")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.format();
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
                                    .child("格式化结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(formatted_text)
                            )
                    )
            )
    }
}
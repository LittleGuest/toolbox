use gpui::*;
use gpui_component::{button::*, *};

pub struct TimestampConverter {
    input: String,
    timestamp: String,
    datetime: String,
}

impl TimestampConverter {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            timestamp: String::new(),
            datetime: String::new(),
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.timestamp.clear();
            self.datetime.clear();
            return;
        }

        if let Ok(_ts) = self.input.parse::<i64>() {
            self.timestamp = self.input.clone();
            self.datetime = base::timestamp(Some(&self.input))
                .map(|m| m.get("format").unwrap_or(&String::new()).clone())
                .unwrap_or_default();
        } else if let Ok(result) = base::timestamp(Some(&self.input)) {
            self.timestamp = result.get("format").unwrap_or(&String::new()).clone();
            self.datetime = self.input.clone();
        } else {
            self.timestamp.clear();
            self.datetime.clear();
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.timestamp.clear();
        self.datetime.clear();
    }
}

impl Render for TimestampConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let timestamp = self.timestamp.clone();
        let datetime = self.datetime.clone();
        
        let timestamp_text = if timestamp.is_empty() {
            "-".to_string()
        } else {
            timestamp.clone()
        };
        
        let datetime_text = if datetime.is_empty() {
            "-".to_string()
        } else {
            datetime.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("时间戳转换")
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
                                    .child("输入(时间戳或日期时间)")
                            )
                            .child(
                                div()
                                    .min_h(px(40.0))
                                    .max_h(px(40.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_3()
                                    .py_2()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(if self.input.is_empty() { "请输入时间戳或日期时间...".to_string() } else { self.input.clone() })
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
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
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("时间戳")
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
                                    .child(timestamp_text)
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
                                    .child("日期时间")
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
                                    .child(datetime_text)
                            )
                    )
            )
    }
}
use gpui::*;
use gpui_component::{button::*, *};

pub struct BaseConverter {
    input: String,
    binary: String,
    octal: String,
    decimal: String,
    hex: String,
}

impl BaseConverter {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            binary: String::new(),
            octal: String::new(),
            decimal: String::new(),
            hex: String::new(),
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.binary.clear();
            self.octal.clear();
            self.decimal.clear();
            self.hex.clear();
            return;
        }

        if let Ok(result) = base::number_base(Some(base::Base::Decimal), self.input.clone()) {
            self.binary = result.get("binary").unwrap_or(&String::new()).clone();
            self.octal = result.get("octal").unwrap_or(&String::new()).clone();
            self.decimal = result.get("decimal").unwrap_or(&String::new()).clone();
            self.hex = result.get("hex").unwrap_or(&String::new()).clone();
        } else {
            self.binary.clear();
            self.octal.clear();
            self.decimal.clear();
            self.hex.clear();
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.binary.clear();
        self.octal.clear();
        self.decimal.clear();
        self.hex.clear();
    }
}

impl Render for BaseConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let binary = self.binary.clone();
        let octal = self.octal.clone();
        let decimal = self.decimal.clone();
        let hex = self.hex.clone();
        
        let binary_text = if binary.is_empty() {
            "-".to_string()
        } else {
            binary.clone()
        };
        
        let octal_text = if octal.is_empty() {
            "-".to_string()
        } else {
            octal.clone()
        };
        
        let decimal_text = if decimal.is_empty() {
            "-".to_string()
        } else {
            decimal.clone()
        };
        
        let hex_text = if hex.is_empty() {
            "-".to_string()
        } else {
            hex.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("进制转换")
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
                                    .child("输入(十进制)")
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
                                    .child(if self.input.is_empty() { "请输入十进制数字...".to_string() } else { self.input.clone() })
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
                                    .child("二进制")
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
                                    .child(binary_text)
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
                                    .child("八进制")
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
                                    .child(octal_text)
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
                                    .child("十进制")
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
                                    .child(decimal_text)
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
                                    .child("十六进制")
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
                                    .child(hex_text)
                            )
                    )
            )
    }
}
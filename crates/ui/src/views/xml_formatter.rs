use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

pub struct XmlFormatter {
    xml: String,
    indent: String,
}

impl XmlFormatter {
    pub fn new() -> Self {
        Self {
            xml: String::new(),
            indent: "    ".to_string(),
        }
    }

    fn format_xml(&mut self) {
        if self.xml.trim().is_empty() {
            return;
        }

        let formatted = self.format_xml_internal(&self.xml, &self.indent);
        self.xml = formatted;
    }

    fn format_xml_internal(&self, xml: &str, indent: &str) -> String {
        let mut result = String::new();
        let mut depth = 0;
        let mut in_tag = false;
        let mut in_comment = false;
        let mut in_cdata = false;
        let mut chars = xml.chars().peekable();

        while let Some(c) = chars.next() {
            if in_cdata {
                result.push(c);
                if c == ']' {
                    if let Some(&']') = chars.peek() {
                        chars.next();
                        if let Some(&'>') = chars.peek() {
                            chars.next();
                            in_cdata = false;
                        }
                    }
                }
                continue;
            }

            if in_comment {
                result.push(c);
                if c == '-' {
                    if let Some(&'-') = chars.peek() {
                        chars.next();
                        if let Some(&'>') = chars.peek() {
                            chars.next();
                            in_comment = false;
                        }
                    }
                }
                continue;
            }

            if c == '<' {
                if let Some(&'!') = chars.peek() {
                    chars.next();
                    if let Some(&'-') = chars.peek() {
                        chars.next();
                        if let Some(&'-') = chars.peek() {
                            chars.next();
                            in_comment = true;
                            result.push_str("<!--");
                            continue;
                        }
                    }
                    if let Some(&'[') = chars.peek() {
                        chars.next();
                        if chars.next() == Some('C') && chars.next() == Some('D') && 
                           chars.next() == Some('A') && chars.next() == Some('T') && 
                           chars.next() == Some('A') && chars.next() == Some('[') {
                            in_cdata = true;
                            result.push_str("<![CDATA[");
                            continue;
                        }
                    }
                }

                if in_tag {
                    result.push_str(&format!("\n{}", indent.repeat(depth)));
                }
                in_tag = true;
                result.push(c);

                if let Some(&'/') = chars.peek() {
                    depth = depth.saturating_sub(1);
                }
            } else if c == '>' {
                result.push(c);
                in_tag = false;

                let prev_char = result.chars().rev().nth(1);
                if prev_char != Some('/') && prev_char != Some('制') && prev_char != Some('-') {
                    depth += 1;
                }
            } else if c == '/' && in_tag {
                result.push(c);
                if let Some(&'>') = chars.peek() {
                    depth = depth.saturating_sub(1);
                }
            } else if c.is_whitespace() && in_tag {
                continue;
            } else if c == '\n' || c == '\r' {
                if !in_tag {
                    continue;
                }
            } else {
                if !in_tag && !result.ends_with('\n') {
                    result.push(c);
                } else if in_tag {
                    result.push(c);
                } else {
                    result.push_str(&format!("\n{}", indent.repeat(depth)));
                    result.push(c);
                }
            }
        }

        result
    }

    fn clear(&mut self) {
        self.xml.clear();
    }
}

impl Render for XmlFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let xml_text = if self.xml.is_empty() {
            "输入XML代码...".to_string()
        } else {
            self.xml.clone()
        };
        
        let indent_text = if self.indent == "  " { "2 空格" } else { "4 空格" };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("XML格式化")
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
                                    .child("缩进")
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
                                    .child(indent_text)
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
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("粘贴")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("复制")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("格式化")
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
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("XML编辑器")
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
                                    .child(xml_text)
                            )
                    )
            )
    }
}
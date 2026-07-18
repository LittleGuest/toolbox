use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

pub struct XmlFormatter {
    input: String,
    indent: String,
    input_state: Entity<InputState>,
    indent_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl XmlFormatter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入XML...")
                .multi_line(true)
        });

        let indent_state = cx.new(|cx| {
            let mut s = SelectState::new(vec!["2".to_string(), "4".to_string()], None, window, cx);
            s.set_selected_value(&"4".to_string(), window, cx);
            s
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        let value = input_state.read(cx).value();
                        this.input = value.to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &indent_state,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        let spaces: usize = value.parse().unwrap_or(4);
                        this.indent = " ".repeat(spaces);
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            input: String::new(),
            indent: "    ".to_string(),
            input_state,
            indent_state,
            _subscriptions,
        }
    }

    fn format(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            return;
        }

        self.input = self.format_xml(&self.input.clone(), &self.indent);
        self.input_state.update(cx, |state, cx| {
            state.set_value(self.input.clone(), window, cx);
        });
    }

    fn format_xml(&self, xml: &str, indent: &str) -> String {
        let mut result = String::new();
        let mut depth = 0;
        let mut in_tag = false;
        let mut in_comment = false;
        let mut in_cdata = false;
        let mut tag_buffer = String::new();
        let mut text_buffer = String::new();
        let mut chars = xml.chars().peekable();

        fn is_self_closing(tag: &str) -> bool {
            tag.trim().ends_with('/')
        }

        fn get_tag_name(tag: &str) -> &str {
            let tag = tag.trim();
            let tag = tag.trim_start_matches('<');
            let tag = tag.trim_start_matches('/');
            let tag = tag.trim_end_matches('>');
            let tag = tag.trim_end_matches('/');
            if let Some(pos) = tag.find(|c: char| c.is_whitespace()) {
                &tag[..pos]
            } else {
                tag
            }
        }

        fn write_indent(result: &mut String, depth: usize, indent: &str) {
            for _ in 0..depth {
                result.push_str(indent);
            }
        }

        while let Some(c) = chars.next() {
            if in_cdata {
                text_buffer.push(c);
                if c == ']' {
                    if let Some(&']') = chars.peek() {
                        chars.next();
                        text_buffer.push(']');
                        if let Some(&'>') = chars.peek() {
                            chars.next();
                            text_buffer.push('>');
                            in_cdata = false;
                        }
                    }
                }
                continue;
            }

            if in_comment {
                text_buffer.push(c);
                if c == '-' {
                    if let Some(&'-') = chars.peek() {
                        chars.next();
                        text_buffer.push('-');
                        if let Some(&'>') = chars.peek() {
                            chars.next();
                            text_buffer.push('>');
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
                            text_buffer.push_str("<!--");
                            continue;
                        }
                    }
                    if let Some(&'[') = chars.peek() {
                        chars.next();
                        if chars.next() == Some('C')
                            && chars.next() == Some('D')
                            && chars.next() == Some('A')
                            && chars.next() == Some('T')
                            && chars.next() == Some('A')
                            && chars.next() == Some('[')
                        {
                            in_cdata = true;
                            text_buffer.push_str("<![CDATA[");
                            continue;
                        }
                    }
                }

                if !text_buffer.trim().is_empty() {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    write_indent(&mut result, depth, indent);
                    result.push_str(text_buffer.trim());
                    result.push('\n');
                } else if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                text_buffer.clear();

                in_tag = true;
                tag_buffer.clear();
                tag_buffer.push(c);
            } else if c == '>' {
                tag_buffer.push(c);
                in_tag = false;

                let _tag_name = get_tag_name(&tag_buffer);
                let is_closing = tag_buffer.trim().starts_with("</");
                let is_self = is_self_closing(&tag_buffer);
                let is_decl = tag_buffer.trim().starts_with("<?");
                let is_doctype = tag_buffer.trim().to_uppercase().starts_with("<!DOCTYPE");

                if is_closing {
                    depth = depth.saturating_sub(1);
                }

                write_indent(&mut result, depth, indent);
                result.push_str(&tag_buffer);
                result.push('\n');

                if !is_closing && !is_self && !is_decl && !is_doctype {
                    depth += 1;
                }
            } else if in_tag {
                tag_buffer.push(c);
            } else {
                text_buffer.push(c);
            }
        }

        if !text_buffer.trim().is_empty() {
            write_indent(&mut result, depth, indent);
            result.push_str(text_buffer.trim());
        }

        result.trim_end().to_string() + "\n"
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }
}

impl Render for XmlFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                // 缩进 → Select（匹配 Tauri n-select）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().child("缩进"))
                        .child(Select::new(&self.indent_state)),
                )
                // 操作 → Paste + Copy + 格式化（匹配 Tauri n-button-group，三个按钮在同一组内）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().child(""))
                        .child(
                            ButtonGroup::new("action-buttons")
                                .child(
                                    Button::new("paste-input")
                                        .icon(Icon::new(IconName::File))
                                        .tooltip("粘贴")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-input")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_input(cx);
                                        })),
                                )
                                .child(Button::new("format").child("格式化").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.format(window, cx);
                                    }),
                                )),
                        ),
                )
                // 单编辑器（匹配 Tauri CodeMirror，格式化后替换内容）
                .child(Input::new(&self.input_state).h(px(400.0))),
        )
    }
}

use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};

/// 对应 Vue unicodeEscape：每个字符（含代理对）转 \uXXXX
fn unicode_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let code = c as u32;
        if code > 0xFFFF {
            let v = code - 0x10000;
            let hi = 0xD800 + (v >> 10);
            let lo = 0xDC00 + (v & 0x3FF);
            out.push_str(&format!("\\u{hi:04x}\\u{lo:04x}"));
        } else {
            out.push_str(&format!("\\u{code:04x}"));
        }
    }
    out
}

/// 对应 Vue unicodeUnescape：\uXXXX → 字符，支持代理对组合
fn unicode_unescape(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\'
            && i + 6 <= chars.len()
            && chars[i + 1] == 'u'
            && chars[i + 2..i + 6].iter().all(|c| c.is_ascii_hexdigit())
        {
            let code =
                u32::from_str_radix(&chars[i + 2..i + 6].iter().collect::<String>(), 16).unwrap_or(0);
            if (0xD800..=0xDBFF).contains(&code) {
                if i + 12 <= chars.len()
                    && chars[i + 6] == '\\'
                    && chars[i + 7] == 'u'
                    && chars[i + 8..i + 12].iter().all(|c| c.is_ascii_hexdigit())
                {
                    let code2 = u32::from_str_radix(
                        &chars[i + 8..i + 12].iter().collect::<String>(),
                        16,
                    )
                    .unwrap_or(0);
                    if (0xDC00..=0xDFFF).contains(&code2) {
                        let cp = 0x10000 + ((code - 0xD800) << 10) + (code2 - 0xDC00);
                        if let Some(c) = char::from_u32(cp) {
                            out.push(c);
                        }
                        i += 12;
                        continue;
                    }
                }
                out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
            } else {
                out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
            }
            i += 6;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

pub struct Utf8UnicodeConverter {
    input: String,
    output: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl Utf8UnicodeConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要转换的文本或 \\u 转义序列")
        });
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("转换结果将显示在这里")
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.input = input_state.read(cx).value().to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            output: String::new(),
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        self.output = unicode_escape(&self.input);
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        self.output = unicode_unescape(&self.input);
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn paste_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.output = text.to_string();
                self.output_state.update(cx, |state, cx| {
                    state.set_value(self.output.clone(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for Utf8UnicodeConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header("UTF8 / Unicode", "文本与 \\u Unicode 转义序列互转", cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::FileText,
                        "UTF8 / Unicode",
                        "\\u 转义序列编解码",
                        cx,
                    ))
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输入", cx))
                            .child(
                                Textarea::new(&self.input_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("encode")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("编码")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.encode(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("decode")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("解码")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.decode(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输出", cx))
                            .child(
                                Textarea::new(&self.output_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("paste-input")
                                    .ghost()
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴输入")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_input(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("paste-output")
                                    .ghost()
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴输出")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_output(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-output")
                                    .ghost()
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .disabled(self.output.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .ghost()
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

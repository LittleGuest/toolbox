use crate::design;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{InputEvent, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct Utf8UnicodeConverter {
    mode: String,
    hex_prefix: bool,
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    mode_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

fn utf8_encode(s: &str, hex_prefix: bool) -> String {
    s.as_bytes()
        .iter()
        .map(|b| {
            if hex_prefix {
                format!("0x{b:02x}")
            } else {
                format!("{b:02x}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn utf8_decode(s: &str) -> Result<String, String> {
    let mut t: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    loop {
        let lower = t.to_ascii_lowercase();
        match lower.find("0x") {
            Some(p) => t.replace_range(p..p + 2, ""),
            None => break,
        }
    }
    if t.len() % 2 != 0 {
        return Err("十六进制长度必须为偶数".to_string());
    }
    let chars: Vec<char> = t.chars().collect();
    let mut bytes = Vec::with_capacity(chars.len() / 2);
    for pair in chars.chunks(2) {
        let hex: String = pair.iter().collect();
        let b = u8::from_str_radix(&hex, 16).map_err(|_| "无效的十六进制字符".to_string())?;
        bytes.push(b);
    }
    String::from_utf8(bytes).map_err(|_| "无效的 UTF-8 字节序列".to_string())
}

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
                out.push('\u{FFFD}');
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

fn ascii_to_codes(s: &str) -> String {
    s.chars()
        .map(|c| {
            let display = match c {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                _ => c.to_string(),
            };
            format!("{} | {} | 0x{:X}", display, c as u32, c as u32)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn codes_to_ascii(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let tokens = s
        .split(|c: char| c.is_whitespace() || c == ',' || c == '，')
        .filter(|t| !t.is_empty());
    for t in tokens {
        let parsed = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            u32::from_str_radix(h, 16)
        } else {
            t.parse::<u32>()
        };
        match parsed {
            Ok(code) => match char::from_u32(code) {
                Some(c) => out.push(c),
                None => return Err(format!("无效的码值: {t}")),
            },
            Err(_) => return Err(format!("无效的码值: {t}")),
        }
    }
    Ok(out)
}

impl Utf8UnicodeConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要编码 / 解码的文本或十六进制")
        });
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("编码 / 解码结果将显示在这里")
        });

        let mode_items = vec![
            "UTF-8 字节序列".to_string(),
            "Unicode 转义 (\\u)".to_string(),
            "ASCII 码表".to_string(),
        ];
        let mode_state = cx.new(|cx| {
            let mut state = SelectState::new(mode_items, None, window, cx);
            state.set_selected_value(&"UTF-8 字节序列".to_string(), window, cx);
            state
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
                &mode_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.mode = match value.as_str() {
                            "Unicode 转义 (\\u)" => "unicode",
                            "ASCII 码表" => "ascii",
                            _ => "utf8",
                        }
                        .to_string();
                        this.error.clear();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            mode: "utf8".to_string(),
            hex_prefix: false,
            input: String::new(),
            output: String::new(),
            error: String::new(),
            input_state,
            output_state,
            mode_state,
            _subscriptions,
        }
    }

    fn encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.input.is_empty() {
            return;
        }
        self.output = match self.mode.as_str() {
            "unicode" => unicode_escape(&self.input),
            "ascii" => ascii_to_codes(&self.input),
            _ => utf8_encode(&self.input, self.hex_prefix),
        };
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.output.is_empty() {
            return;
        }
        let result = match self.mode.as_str() {
            "unicode" => Ok(unicode_unescape(&self.output)),
            "ascii" => codes_to_ascii(&self.output),
            _ => utf8_decode(&self.output),
        };
        match result {
            Ok(text) => {
                self.input = text.clone();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text, window, cx);
                });
            }
            Err(e) => self.error = e,
        }
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.error.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
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
            }
        }
    }
}

impl Render for Utf8UnicodeConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header("UTF8 / Unicode", "UTF8 与 Unicode 互转", cx))
            .child(
                // 配置卡片：模式 + 选项
                design::card(cx)
                    .child(
                        design::toolbar()
                            .child(design::caption("模式", cx))
                            .child(Select::new(&self.mode_state)),
                    )
                    .when(self.mode == "utf8", |this| {
                        this.child(
                            design::toolbar().child(
                                Checkbox::new("hex-prefix")
                                    .label("带 0x 前缀")
                                    .checked(self.hex_prefix)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.hex_prefix = *v;
                                        cx.notify();
                                    })),
                            ),
                        )
                    }),
            )
            .child(
                // 输入卡片
                design::card(cx)
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-input")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_input(window, cx);
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
                            .child(
                                Button::new("clear-input")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    )
                    .child(
                        Textarea::new(&self.input_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    ),
            )
            .child(
                // 编码/解码操作行
                design::action_row()
                    .child(
                        Button::new("encode")
                            .label("编码")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encode(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode")
                            .label("解码")
                            .icon(Icon::new(IconName::ArrowUp))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decode(window, cx);
                            })),
                    )
                    .when(!self.error.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .child(self.error.clone()),
                        )
                    }),
            )
            .child(
                // 输出卡片
                design::card(cx)
                    .child(
                        Textarea::new(&self.output_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-output")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_output(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-output")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-output")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    ),
            )
    }
}

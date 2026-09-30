use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};

const HTML_UNESCAPE: &[(&str, &str)] = &[
    ("amp", "&"),
    ("lt", "<"),
    ("gt", ">"),
    ("quot", "\""),
    ("apos", "'"),
    ("#39", "'"),
    ("#x27", "'"),
];

const XML_UNESCAPE: &[(&str, &str)] = &[
    ("amp", "&"),
    ("lt", "<"),
    ("gt", ">"),
    ("quot", "\""),
    ("apos", "'"),
];

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_entities(s: &str, named: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        match rest[pos..].find(';') {
            Some(end_off) if end_off <= 32 => {
                let body = &rest[pos + 1..pos + end_off];
                let replaced = if let Some(num) = body.strip_prefix('#') {
                    let digits = num
                        .strip_prefix('x')
                        .or_else(|| num.strip_prefix('X'))
                        .map(|d| (d, 16u32))
                        .unwrap_or((num, 10));
                    u32::from_str_radix(digits.0, digits.1)
                        .ok()
                        .and_then(char::from_u32)
                        .map(|c| c.to_string())
                } else {
                    named
                        .iter()
                        .find(|(k, _)| *k == body)
                        .map(|(_, v)| v.to_string())
                };
                match replaced {
                    Some(rep) => {
                        out.push_str(&rest[..pos]);
                        out.push_str(&rep);
                        rest = &rest[pos + end_off + 1..];
                    }
                    None => {
                        out.push_str(&rest[..=pos]);
                        rest = &rest[pos + 1..];
                    }
                }
            }
            _ => {
                out.push_str(&rest[..=pos]);
                rest = &rest[pos + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn html_unescape(s: &str) -> String {
    unescape_entities(s, HTML_UNESCAPE)
}

fn xml_unescape(s: &str) -> String {
    unescape_entities(s, XML_UNESCAPE)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn push_char(units: &mut Vec<u16>, ch: char) {
    let mut buf = [0u16; 2];
    for u in ch.encode_utf16(&mut buf) {
        units.push(*u);
    }
}

fn json_unescape(s: &str) -> Result<String, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut units: Vec<u16> = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] != '\\' {
            push_char(&mut units, chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            return Err("无效的 JSON 转义序列".to_string());
        }
        let esc = chars[i];
        i += 1;
        match esc {
            '"' => units.push(0x22),
            '\\' => units.push(0x5c),
            '/' => units.push(0x2f),
            'n' => units.push(0x0a),
            'r' => units.push(0x0d),
            't' => units.push(0x09),
            'b' => units.push(0x08),
            'f' => units.push(0x0c),
            'u' => {
                let hex: String = chars.iter().skip(i).take(4).collect();
                if hex.len() == 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    units.push(u32::from_str_radix(&hex, 16).unwrap_or(0) as u16);
                    i += 4;
                } else {
                    return Err("无效的 JSON 转义序列".to_string());
                }
            }
            _ => return Err(format!("无效的 JSON 转义序列 \\{esc}")),
        }
    }
    Ok(String::from_utf16_lossy(&units))
}

fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\u{0b}' => out.push_str("\\v"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out
}

fn js_unescape(s: &str) -> Result<String, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut units: Vec<u16> = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] != '\\' {
            push_char(&mut units, chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            units.push(0x5c);
            break;
        }
        let esc = chars[i];
        i += 1;
        match esc {
            'n' => units.push(0x0a),
            'r' => units.push(0x0d),
            't' => units.push(0x09),
            'b' => units.push(0x08),
            'f' => units.push(0x0c),
            'v' => units.push(0x0b),
            '\\' => units.push(0x5c),
            '"' => units.push(0x22),
            '\'' => units.push(0x27),
            'x' => {
                let hex: String = chars.iter().skip(i).take(2).collect();
                if hex.len() == 2 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    units.push(u32::from_str_radix(&hex, 16).unwrap_or(0) as u16);
                    i += 2;
                } else {
                    return Err("无效的转义序列 \\x".to_string());
                }
            }
            'u' => {
                let hex: String = chars.iter().skip(i).take(4).collect();
                if hex.len() == 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    units.push(u32::from_str_radix(&hex, 16).unwrap_or(0) as u16);
                    i += 4;
                } else {
                    return Err("无效的转义序列 \\u".to_string());
                }
            }
            e if ('0'..='7').contains(&e) => {
                let mut octal = String::new();
                octal.push(e);
                while i < chars.len() && ('0'..='7').contains(&chars[i]) && octal.len() < 3 {
                    octal.push(chars[i]);
                    i += 1;
                }
                let mut val = u32::from_str_radix(&octal, 8).unwrap_or(0);
                if val > 0xff {
                    val = u32::from_str_radix(&octal[..2], 8).unwrap_or(0);
                }
                units.push(val as u16);
            }
            e => return Err(format!("无效的转义序列 \\{e}")),
        }
    }
    Ok(String::from_utf16_lossy(&units))
}

fn csv_escape_field(field: &str) -> String {
    if field.contains(|c| matches!(c, '"' | ',' | '\n' | '\r' | '\t')) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn csv_escape(s: &str) -> String {
    s.split("\r\n")
        .flat_map(|l| l.split(['\r', '\n']))
        .map(csv_escape_field)
        .collect::<Vec<_>>()
        .join(",")
}

fn csv_unescape(s: &str) -> String {
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if in_quotes {
            if ch == '"' {
                if chars.get(i + 1) == Some(&'"') {
                    field.push('"');
                    i += 2;
                } else {
                    in_quotes = false;
                    i += 1;
                }
            } else {
                field.push(ch);
                i += 1;
            }
        } else if ch == '"' {
            in_quotes = true;
            i += 1;
        } else if ch == ',' {
            fields.push(std::mem::take(&mut field));
            i += 1;
        } else {
            field.push(ch);
            i += 1;
        }
    }
    fields.push(field);
    fields.join("\n")
}

fn sql_escape(s: &str) -> String {
    s.replace('\'', "''")
}

fn sql_unescape(s: &str) -> String {
    s.replace("''", "'")
}

fn escape_by_mode(mode: &str, s: &str) -> String {
    match mode {
        "xml" => xml_escape(s),
        "json" => json_escape(s),
        "js" | "java" | "csharp" => js_escape(s),
        "csv" => csv_escape(s),
        "sql" => sql_escape(s),
        _ => html_escape(s),
    }
}

fn unescape_by_mode(mode: &str, s: &str) -> Result<String, String> {
    match mode {
        "xml" => Ok(xml_unescape(s)),
        "json" => json_unescape(s),
        "js" | "java" | "csharp" => js_unescape(s),
        "csv" => Ok(csv_unescape(s)),
        "sql" => Ok(sql_unescape(s)),
        _ => Ok(html_unescape(s)),
    }
}

pub struct EscapeTools {
    mode: String,
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    mode_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl EscapeTools {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要转义 / 反转义的内容")
        });
        let output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果将显示在这里"));

        let mode_items = vec![
            "HTML".to_string(),
            "XML".to_string(),
            "JSON".to_string(),
            "JavaScript".to_string(),
            "Java".to_string(),
            "C#".to_string(),
            "CSV".to_string(),
            "SQL".to_string(),
        ];
        let mode_state = cx.new(|cx| {
            let mut state = SelectState::new(mode_items, None, window, cx);
            state.set_selected_value(&"HTML".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.input = input_state.read(cx).value().to_string();
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
                            "XML" => "xml",
                            "JSON" => "json",
                            "JavaScript" => "js",
                            "Java" => "java",
                            "C#" => "csharp",
                            "CSV" => "csv",
                            "SQL" => "sql",
                            _ => "html",
                        }
                        .to_string();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            mode: "html".to_string(),
            input: String::new(),
            output: String::new(),
            error: String::new(),
            input_state,
            output_state,
            mode_state,
            _subscriptions,
        }
    }

    fn do_escape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        self.output = escape_by_mode(&self.mode, &self.input);
        self.error.clear();
        self.sync_output(window, cx);
        cx.notify();
    }

    fn do_unescape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        match unescape_by_mode(&self.mode, &self.input) {
            Ok(result) => {
                self.output = result;
                self.error.clear();
            }
            Err(e) => self.error = e,
        }
        self.sync_output(window, cx);
        cx.notify();
    }

    fn sync_output(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let output = self.output.clone();
        self.output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
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
                self.sync_output(window, cx);
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
        self.error.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for EscapeTools {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let output_empty = self.output.is_empty();

        design::page()
            .child(design::page_header(
                "转义工具",
                "HTML / XML / JSON / JS / CSV / SQL 转义与反转义",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::SquareTerminal,
                        "转义工具",
                        "多语言字符串转义与反转义",
                        cx,
                    ))
                    // 配置行：模式
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(gpui::black().opacity(0.65))
                                    .child("模式"),
                            )
                            .child(
                                div()
                                    .w(px(260.0))
                                    .child(Select::new(&self.mode_state)),
                            ),
                    )
                    // 输入编辑器（Vue 原版输入区无工具行）
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
                    // 转义 / 反转义动作行
                    .child(
                        design::action_row()
                            .child(
                                Button::new("escape")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("转义")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.do_escape(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("unescape")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("反转义")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.do_unescape(window, cx);
                                    })),
                            ),
                    )
                    // 输出编辑器
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输出", cx))
                            .child(
                                Textarea::new(&self.output_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(
                                design::toolbar()
                                    .child(
                                        Button::new("paste-input")
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴输入")
                                            .on_click(cx.listener(
                                                |this, _, window, cx| {
                                                    this.paste_input(window, cx);
                                                },
                                            )),
                                    )
                                    .child(
                                        Button::new("paste-output")
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴输出")
                                            .on_click(cx.listener(
                                                |this, _, window, cx| {
                                                    this.paste_output(window, cx);
                                                },
                                            )),
                                    )
                                    .child(
                                        Button::new("copy-output")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制输出")
                                            .disabled(output_empty)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_output(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear")
                                            .icon(Icon::new(IconName::Close))
                                            .tooltip("清除")
                                            .on_click(cx.listener(
                                                |this, _, window, cx| {
                                                    this.clear(window, cx);
                                                },
                                            )),
                                    ),
                            ),
                    )
                    .when(!error.is_empty(), |card| {
                        card.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(error),
                        )
                    }),
            )
    }
}

use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder,
    *,
};

const DIGITS: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
const MAX_CODEPOINT: u32 = 0x10FFFF;

fn digit_value(ch: char) -> Option<u32> {
    ch.to_lowercase()
        .next()
        .and_then(|c| DIGITS.chars().position(|d| d == c))
        .map(|p| p as u32)
}

fn is_valid_in_base(s: &str, base: u32) -> bool {
    !s.is_empty() && s.chars().all(|c| digit_value(c).is_some_and(|v| v < base))
}

fn convert_base_str(s: &str, from: u32, to: u32) -> Result<String, String> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err("请输入数值".to_string());
    }
    if !is_valid_in_base(&s, from) {
        return Err(format!("\"{s}\" 不是合法的 {from} 进制数"));
    }

    let mut digits: Vec<u8> = vec![0];
    for ch in s.chars() {
        let v = digit_value(ch).unwrap_or(0);
        let mut carry = v;
        for d in digits.iter_mut() {
            let x = (*d as u32) * from + carry;
            *d = (x % 10) as u8;
            carry = x / 10;
        }
        while carry > 0 {
            digits.push((carry % 10) as u8);
            carry /= 10;
        }
    }
    while digits.len() > 1 && *digits.last().unwrap() == 0 {
        digits.pop();
    }
    if digits.len() == 1 && digits[0] == 0 {
        return Ok("0".to_string());
    }

    let mut out = Vec::new();
    while !(digits.len() == 1 && digits[0] == 0) {
        let mut rem = 0u32;
        for d in digits.iter_mut().rev() {
            let x = rem * 10 + *d as u32;
            *d = (x / to) as u8;
            rem = x % to;
        }
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
        out.push(DIGITS.as_bytes()[rem as usize] as char);
    }
    Ok(out.iter().rev().collect::<String>().to_uppercase())
}

fn hex_encode(text: &str, no_space: bool) -> String {
    let sep = if no_space { "" } else { " " };
    text.as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(sep)
}

fn bytes_to_utf8(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|_| "解码失败：不是有效的 UTF-8 字节序列".to_string())
}

fn hex_decode(text: &str) -> Result<String, String> {
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if !clean.len().is_multiple_of(2) || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("无效的十六进制输入".to_string());
    }
    let bytes: Vec<u8> = (0..clean.len() / 2)
        .map(|i| u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16).unwrap_or(0))
        .collect();
    bytes_to_utf8(bytes)
}

fn bin_encode(text: &str) -> String {
    text.as_bytes()
        .iter()
        .map(|b| format!("{b:08b}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn bin_decode(text: &str) -> Result<String, String> {
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if !clean.len().is_multiple_of(8) || !clean.chars().all(|c| c == '0' || c == '1') {
        return Err("无效的二进制输入".to_string());
    }
    let bytes: Vec<u8> = (0..clean.len() / 8)
        .map(|i| u8::from_str_radix(&clean[i * 8..i * 8 + 8], 2).unwrap_or(0))
        .collect();
    bytes_to_utf8(bytes)
}

fn ascii_to_codes(text: &str) -> String {
    text.chars()
        .map(|ch| format!("{ch} | {} | 0x{:X}", ch as u32, ch as u32))
        .collect::<Vec<_>>()
        .join("\n")
}

fn codes_to_ascii(text: &str) -> Result<String, String> {
    let parts: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == ',' || c == '，')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Err("请输入要转换的编码".to_string());
    }
    let mut out = String::new();
    for p in parts {
        let code = if let Some(h) = p.trim().to_lowercase().strip_prefix("0x") {
            u32::from_str_radix(h, 16).map_err(|_| format!("无法识别的编码：{p}"))?
        } else {
            p.trim()
                .parse::<u32>()
                .map_err(|_| format!("无法识别的编码：{p}"))?
        };
        if code > MAX_CODEPOINT {
            return Err(format!("无效的码点：{p}"));
        }
        out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
    }
    Ok(out)
}

fn base_options() -> Vec<String> {
    (2..=36).map(|b| format!("{b} 进制")).collect()
}

pub struct BaseConverter {
    tab: usize,

    input: String,
    binary: String,
    octal: String,
    decimal: String,
    hex: String,
    input_type: ::base::Base,
    input_state: Entity<InputState>,
    input_type_state: Entity<SelectState<Vec<String>>>,

    conv_input: String,
    conv_output: String,
    conv_error: String,
    from_base: u32,
    to_base: u32,
    conv_input_state: Entity<InputState>,
    conv_output_state: Entity<TextareaState>,
    from_base_state: Entity<SelectState<Vec<String>>>,
    to_base_state: Entity<SelectState<Vec<String>>>,

    hex_input: String,
    hex_output: String,
    hex_error: String,
    hex_no_space: bool,
    hex_input_state: Entity<TextareaState>,
    hex_output_state: Entity<TextareaState>,

    bin_input: String,
    bin_output: String,
    bin_error: String,
    bin_input_state: Entity<TextareaState>,
    bin_output_state: Entity<TextareaState>,

    ascii_input: String,
    ascii_output: String,
    ascii_error: String,
    ascii_mode: usize,
    ascii_input_state: Entity<TextareaState>,
    ascii_output_state: Entity<TextareaState>,

    _subscriptions: Vec<Subscription>,
}

impl BaseConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let type_items = vec![
            "二进制".to_string(),
            "八进制".to_string(),
            "十进制".to_string(),
            "十六进制".to_string(),
        ];

        let input_type_state = cx.new(|cx| {
            let mut state = SelectState::new(type_items, None, window, cx);
            state.set_selected_value(&"十进制".to_string(), window, cx);
            state
        });
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入")
                .pattern(regex::Regex::new(r"^.{0,19}$").expect("invalid pattern"))
        });

        let options = base_options();
        let from_base_state = cx.new(|cx| {
            let mut state = SelectState::new(options.clone(), None, window, cx);
            state.set_selected_value(&"10 进制".to_string(), window, cx);
            state
        });
        let to_base_state = cx.new(|cx| {
            let mut state = SelectState::new(options, None, window, cx);
            state.set_selected_value(&"16 进制".to_string(), window, cx);
            state
        });
        let conv_input_state = cx
            .new(|cx| InputState::new(window, cx).placeholder("请输入数值，如 255、ff、11111111"));
        let conv_output_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果"));

        let hex_input_state = cx
            .new(|cx| TextareaState::new(window, cx).placeholder("请输入文本，编码后输出十六进制"));
        let hex_output_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("十六进制结果（解码时请将十六进制粘贴到此处）")
        });

        let bin_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本，编码后输出二进制"));
        let bin_output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("二进制结果（解码时请将二进制粘贴到此处）")
        });

        let ascii_input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder(
                "文本模式下输入文本；码模式下输入十进制或十六进制码（逗号/空格分隔，支持 0x 前缀）",
            )
        });
        let ascii_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果"));

        let _subscriptions = vec![
            cx.subscribe_in(
                &input_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.input_type = match value.as_str() {
                            "二进制" => ::base::Base::Binary,
                            "八进制" => ::base::Base::Octal,
                            "十进制" => ::base::Base::Decimal,
                            _ => ::base::Base::Hex,
                        };
                        this.convert();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.input = input_state.read(cx).value().to_string();
                        this.convert();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &from_base_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.from_base = value
                            .split(' ')
                            .next()
                            .and_then(|n| n.parse::<u32>().ok())
                            .unwrap_or(10);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &to_base_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.to_base = value
                            .split(' ')
                            .next()
                            .and_then(|n| n.parse::<u32>().ok())
                            .unwrap_or(16);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&conv_input_state, window, {
                let conv_input_state = conv_input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.conv_input = conv_input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&hex_input_state, window, {
                let hex_input_state = hex_input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.hex_input = hex_input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&hex_output_state, window, {
                let hex_output_state = hex_output_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.hex_output = hex_output_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&bin_input_state, window, {
                let bin_input_state = bin_input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.bin_input = bin_input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&bin_output_state, window, {
                let bin_output_state = bin_output_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.bin_output = bin_output_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&ascii_input_state, window, {
                let ascii_input_state = ascii_input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.ascii_input = ascii_input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
        ];

        Self {
            tab: 0,
            input: String::new(),
            binary: String::new(),
            octal: String::new(),
            decimal: String::new(),
            hex: String::new(),
            input_type: ::base::Base::Decimal,
            input_state,
            input_type_state,
            conv_input: String::new(),
            conv_output: String::new(),
            conv_error: String::new(),
            from_base: 10,
            to_base: 16,
            conv_input_state,
            conv_output_state,
            from_base_state,
            to_base_state,
            hex_input: String::new(),
            hex_output: String::new(),
            hex_error: String::new(),
            hex_no_space: false,
            hex_input_state,
            hex_output_state,
            bin_input: String::new(),
            bin_output: String::new(),
            bin_error: String::new(),
            bin_input_state,
            bin_output_state,
            ascii_input: String::new(),
            ascii_output: String::new(),
            ascii_error: String::new(),
            ascii_mode: 0,
            ascii_input_state,
            ascii_output_state,
            _subscriptions,
        }
    }

    fn convert(&mut self) {
        self.binary.clear();
        self.octal.clear();
        self.decimal.clear();
        self.hex.clear();
        if self.input.is_empty() {
            return;
        }
        let input_type = self.input_type;
        if let Ok(result) = ::base::number_base(Some(input_type), self.input.clone()) {
            self.binary = result.get("binary").cloned().unwrap_or_default();
            self.octal = result.get("octal").cloned().unwrap_or_default();
            self.decimal = result.get("decimal").cloned().unwrap_or_default();
            self.hex = result.get("hex").cloned().unwrap_or_default();
        }
    }

    fn convert_base(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match convert_base_str(&self.conv_input, self.from_base, self.to_base) {
            Ok(out) => {
                self.conv_output = out;
                self.conv_error.clear();
            }
            Err(e) => {
                self.conv_error = e;
                return cx.notify();
            }
        }
        self.conv_output_state.update(cx, |state, cx| {
            state.set_value(self.conv_output.clone(), window, cx);
        });
        cx.notify();
    }

    fn encode_hex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.hex_error.clear();
        if self.hex_input.is_empty() {
            return;
        }
        self.hex_output = hex_encode(&self.hex_input, self.hex_no_space);
        self.hex_output_state.update(cx, |state, cx| {
            state.set_value(self.hex_output.clone(), window, cx);
        });
        cx.notify();
    }

    fn decode_hex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match hex_decode(&self.hex_output) {
            Ok(text) => {
                self.hex_error.clear();
                self.hex_input = text;
                self.hex_input_state.update(cx, |state, cx| {
                    state.set_value(self.hex_input.clone(), window, cx);
                });
            }
            Err(e) => self.hex_error = e,
        }
        cx.notify();
    }

    fn encode_bin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.bin_error.clear();
        if self.bin_input.is_empty() {
            return;
        }
        self.bin_output = bin_encode(&self.bin_input);
        self.bin_output_state.update(cx, |state, cx| {
            state.set_value(self.bin_output.clone(), window, cx);
        });
        cx.notify();
    }

    fn decode_bin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match bin_decode(&self.bin_output) {
            Ok(text) => {
                self.bin_error.clear();
                self.bin_input = text;
                self.bin_input_state.update(cx, |state, cx| {
                    state.set_value(self.bin_input.clone(), window, cx);
                });
            }
            Err(e) => self.bin_error = e,
        }
        cx.notify();
    }

    fn convert_ascii(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.ascii_error.clear();
        let result = if self.ascii_mode == 0 {
            if self.ascii_input.is_empty() {
                Err("请输入要转换的文本".to_string())
            } else {
                Ok(ascii_to_codes(&self.ascii_input))
            }
        } else {
            codes_to_ascii(&self.ascii_input)
        };
        match result {
            Ok(out) => {
                self.ascii_output = out;
                self.ascii_output_state.update(cx, |state, cx| {
                    state.set_value(self.ascii_output.clone(), window, cx);
                });
            }
            Err(e) => self.ascii_error = e,
        }
        cx.notify();
    }

    fn paste_into_input(
        &mut self,
        state: &Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            self.input = text.to_string();
            state.update(cx, |s, cx| {
                s.set_value(self.input.clone(), window, cx);
            });
            self.convert();
            cx.notify();
        }
    }

    fn paste_into_textarea(
        &mut self,
        target: &Entity<TextareaState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> String {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            target.update(cx, |s, cx| {
                s.set_value(text.clone(), window, cx);
            });
            return text.to_string();
        }
        String::new()
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        const TABS: [&str; 5] = [
            "常见进制",
            "任意进制",
            "字符串 ↔ 十六进制",
            "字符串 ↔ 二进制",
            "文本 ↔ ASCII 码",
        ];
        div()
            .flex()
            .gap_6()
            .border_b_1()
            .border_color(cx.theme().border)
            .children(TABS.iter().enumerate().map(|(idx, label)| {
                let active = self.tab == idx;
                div()
                    .id(("tab", idx))
                    .px_1()
                    .pb_2()
                    .text_sm()
                    .font_medium()
                    .border_b_2()
                    .text_color(if active {
                        cx.theme().primary
                    } else {
                        cx.theme().muted_foreground
                    })
                    .border_color(if active {
                        cx.theme().primary
                    } else {
                        gpui::black().opacity(0.0)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.tab = idx;
                        cx.notify();
                    }))
                    .child(*label)
            }))
    }

    fn config_label(&self, text: &str, cx: &mut Context<Self>) -> Div {
        div()
            .text_size(px(13.0))
            .text_color(cx.theme().muted_foreground)
            .child(text.to_string())
    }

    fn readonly_row(
        &self,
        id: &'static str,
        label: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) -> Div {
        let value = value.to_string();
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(design::editor_label(label, cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .px_2()
                    .py_1()
                    .text_sm()
                    .font_family("monospace")
                    .truncate()
                    .child(if value.is_empty() {
                        "-".to_string()
                    } else {
                        value.clone()
                    }),
            )
            .child(
                Button::new(id)
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Copy))
                    .tooltip("复制")
                    .disabled(value.is_empty())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_text(&value, cx);
                    })),
            )
    }
}

impl Render for BaseConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = self.tab;

        design::page().child(design::card(cx).child(self.tab_bar(cx)).child(match tab {
            0 => self.render_common(cx).into_any_element(),
            1 => self.render_arbitrary(cx).into_any_element(),
            2 => self.render_hex(cx).into_any_element(),
            3 => self.render_bin(cx).into_any_element(),
            _ => self.render_ascii(cx).into_any_element(),
        }))
    }
}

impl BaseConverter {
    fn render_common(&mut self, cx: &mut Context<Self>) -> Div {
        let binary = self.binary.clone();
        let octal = self.octal.clone();
        let decimal = self.decimal.clone();
        let hex = self.hex.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.config_label("输入类型", cx))
                            .child(
                                div()
                                    .w(px(150.0))
                                    .child(Select::new(&self.input_type_state)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(240.0))
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Input::new(&self.input_state)),
                            )
                            .child(
                                Button::new("paste")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let state = this.input_state.clone();
                                        this.paste_into_input(&state, window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-input")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_text(&this.input, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .child(self.readonly_row("copy-binary", "二进制", &binary, cx))
                    .child(self.readonly_row("copy-octal", "八进制", &octal, cx))
                    .child(self.readonly_row("copy-decimal", "十进制", &decimal, cx))
                    .child(self.readonly_row("copy-hex", "十六进制", &hex, cx)),
            )
    }

    fn render_arbitrary(&mut self, cx: &mut Context<Self>) -> Div {
        let conv_output = self.conv_output.clone();
        let conv_error = self.conv_error.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.config_label("来源进制", cx))
                            .child(div().w(px(130.0)).child(Select::new(&self.from_base_state))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.config_label("目标进制", cx))
                            .child(div().w(px(130.0)).child(Select::new(&self.to_base_state))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(240.0))
                            .child(Input::new(&self.conv_input_state)),
                    )
                    .child(
                        Button::new("convert-base")
                            .primary()
                            .icon(Icon::new(IconName::Play))
                            .tooltip("转换")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.convert_base(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.conv_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("paste-conv")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴输入")
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(item) = cx.read_from_clipboard()
                                    && let Some(text) = item.text()
                                {
                                    this.conv_input = text.to_string();
                                    this.conv_input_state.update(cx, |s, cx| {
                                        s.set_value(text.to_string(), window, cx);
                                    });
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        Button::new("copy-conv")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .disabled(conv_output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_text(&this.conv_output, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-conv")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.conv_input.clear();
                                this.conv_output.clear();
                                this.conv_error.clear();
                                this.conv_input_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                this.conv_output_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                cx.notify();
                            })),
                    ),
            )
            .when(!conv_error.is_empty(), |this| {
                this.child(Self::static_error(&conv_error))
            })
    }

    fn render_hex(&mut self, cx: &mut Context<Self>) -> Div {
        let hex_output = self.hex_output.clone();
        let hex_error = self.hex_error.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.hex_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("encode-hex")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("编码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encode_hex(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode-hex")
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decode_hex(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.hex_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("hex-no-space")
                            .label("连续无空格")
                            .checked(self.hex_no_space)
                            .tooltip("勾选后编码结果不以空格分隔")
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.hex_no_space = *v;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("paste-hex")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴输入")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let state = this.hex_input_state.clone();
                                let text = this.paste_into_textarea(&state, window, cx);
                                this.hex_input = text;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("copy-hex")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .disabled(hex_output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_text(&this.hex_output, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-hex")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.hex_input.clear();
                                this.hex_output.clear();
                                this.hex_error.clear();
                                this.hex_input_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                this.hex_output_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                cx.notify();
                            })),
                    ),
            )
            .when(!hex_error.is_empty(), |this| {
                this.child(Self::static_error(&hex_error))
            })
    }

    fn render_bin(&mut self, cx: &mut Context<Self>) -> Div {
        let bin_output = self.bin_output.clone();
        let bin_error = self.bin_error.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.bin_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("encode-bin")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("编码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.encode_bin(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode-bin")
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decode_bin(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.bin_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("paste-bin")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴输入")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let state = this.bin_input_state.clone();
                                let text = this.paste_into_textarea(&state, window, cx);
                                this.bin_input = text;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("copy-bin")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .disabled(bin_output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_text(&this.bin_output, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-bin")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.bin_input.clear();
                                this.bin_output.clear();
                                this.bin_error.clear();
                                this.bin_input_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                this.bin_output_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                cx.notify();
                            })),
                    ),
            )
            .when(!bin_error.is_empty(), |this| {
                this.child(Self::static_error(&bin_error))
            })
    }

    fn render_ascii(&mut self, cx: &mut Context<Self>) -> Div {
        let ascii_output = self.ascii_output.clone();
        let ascii_error = self.ascii_error.clone();
        let mode = self.ascii_mode;

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.ascii_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(Self::segmented(mode, cx))
                    .child(
                        Button::new("convert-ascii")
                            .primary()
                            .icon(Icon::new(IconName::Play))
                            .tooltip("转换")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.convert_ascii(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.ascii_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("paste-ascii")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴输入")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let state = this.ascii_input_state.clone();
                                let text = this.paste_into_textarea(&state, window, cx);
                                this.ascii_input = text;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("copy-ascii")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .disabled(ascii_output.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_text(&this.ascii_output, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-ascii")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.ascii_input.clear();
                                this.ascii_output.clear();
                                this.ascii_error.clear();
                                this.ascii_input_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                this.ascii_output_state.update(cx, |s, cx| {
                                    s.set_value("".to_string(), window, cx);
                                });
                                cx.notify();
                            })),
                    ),
            )
            .when(!ascii_error.is_empty(), |this| {
                this.child(Self::static_error(&ascii_error))
            })
    }

    fn segmented(mode: usize, cx: &mut Context<Self>) -> Div {
        const MODES: [&str; 2] = ["文本 → 码", "码 → 文本"];
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(MODES.iter().enumerate().map(|(i, opt)| {
                let active = i == mode;
                div()
                    .id(("ascii-mode", i))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .bg(if active {
                        cx.theme().primary
                    } else {
                        gpui::black().opacity(0.0)
                    })
                    .text_color(if active {
                        gpui::white()
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.ascii_mode = i;
                        cx.notify();
                    }))
                    .child(*opt)
            }))
    }

    fn static_error(text: &str) -> Div {
        div()
            .text_size(px(12.5))
            .text_color(Hsla::from(rgb(design::ERROR_RED)))
            .child(text.to_string())
    }
}

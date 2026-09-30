use crate::design;
use data_encoding::{HEXLOWER, HEXLOWER_PERMISSIVE};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{InputEvent, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct StrHexConverter {
    radix: String,
    hex_no_space: bool,
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    radix_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl StrHexConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本，编码后输出对应进制"));
        let output_state = cx
            .new(|cx| TextareaState::new(window, cx).placeholder("编码结果（解码时请将编码内容粘贴到此处）"));

        let radix_items = vec![
            "十六进制".to_string(),
            "二进制".to_string(),
            "十进制".to_string(),
        ];
        let radix_state = cx.new(|cx| {
            let mut state = SelectState::new(radix_items, None, window, cx);
            state.set_selected_value(&"十六进制".to_string(), window, cx);
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
                &radix_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.radix = match value.as_str() {
                            "二进制" => "bin",
                            "十进制" => "dec",
                            _ => "hex",
                        }
                        .to_string();
                        this.error.clear();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            radix: "hex".to_string(),
            hex_no_space: false,
            input: String::new(),
            output: String::new(),
            error: String::new(),
            input_state,
            output_state,
            radix_state,
            _subscriptions,
        }
    }

    fn encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        if self.input.is_empty() {
            return;
        }
        let result = match self.radix.as_str() {
            "bin" => self
                .input
                .as_bytes()
                .iter()
                .map(|b| format!("{b:08b}"))
                .collect::<Vec<_>>()
                .join(" "),
            "dec" => self
                .input
                .as_bytes()
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            _ => {
                if self.hex_no_space {
                    HEXLOWER.encode(self.input.as_bytes())
                } else {
                    let hex = HEXLOWER.encode(self.input.as_bytes());
                    hex.chars()
                        .collect::<Vec<char>>()
                        .chunks(2)
                        .map(|c| c.iter().collect::<String>())
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            }
        };
        self.output = result;
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
        let cleaned: String = self.output.chars().filter(|c| !c.is_whitespace()).collect();
        let result = match self.radix.as_str() {
            "bin" => {
                if cleaned.is_empty()
                    || !cleaned.chars().all(|c| c == '0' || c == '1')
                    || cleaned.chars().count() % 8 != 0
                {
                    Err("无效的二进制输入".to_string())
                } else {
                    let bytes: Vec<u8> = cleaned
                        .as_bytes()
                        .chunks(8)
                        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap_or(""), 2).unwrap_or(0))
                        .collect();
                    String::from_utf8(bytes)
                        .map_err(|_| "解码失败：不是有效的 UTF-8 字节序列".to_string())
                }
            }
            "dec" => {
                let parts: Vec<&str> = self
                    .output
                    .split(|c: char| c.is_whitespace() || c == ',' || c == '，')
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.is_empty() {
                    Err("无效的十进制输入".to_string())
                } else {
                    let mut bytes = Vec::with_capacity(parts.len());
                    for p in &parts {
                        match p.parse::<u8>() {
                            Ok(b) => bytes.push(b),
                            Err(_) => {
                                self.error = format!("无效的字节值: {p}");
                                cx.notify();
                                return;
                            }
                        }
                    }
                    String::from_utf8(bytes)
                        .map_err(|_| "解码失败：不是有效的 UTF-8 字节序列".to_string())
                }
            }
            _ => HEXLOWER_PERMISSIVE
                .decode(cleaned.as_bytes())
                .map_err(|_| "无效的十六进制输入".to_string())
                .and_then(|bytes| {
                    String::from_utf8(bytes)
                        .map_err(|_| "解码失败：不是有效的 UTF-8 字节序列".to_string())
                }),
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

impl Render for StrHexConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();

        design::page()
            .child(design::page_header("字符串进制转换", "字符串与十六进制 / 二进制 / 十进制互转", cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::Replace,
                        "字符串进制转换",
                        "文本按所选进制编码 / 解码",
                        cx,
                    ))
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
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .text_color(cx.theme().muted_foreground)
                                            .child("进制"),
                                    )
                                    .child(div().w(px(130.0)).child(Select::new(&self.radix_state))),
                            )
                            .when(self.radix == "hex", |row| {
                                row.child(
                                    Checkbox::new("hex-no-space")
                                        .label("连续无空格")
                                        .checked(self.hex_no_space)
                                        .tooltip("勾选后编码结果不以空格分隔")
                                        .on_click(cx.listener(|this, v: &bool, _, cx| {
                                            this.hex_no_space = *v;
                                            cx.notify();
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("输入", cx))
                            .child(
                                Textarea::new(&self.input_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("paste-input")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴输入")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_input(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-input")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制输入")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_input(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear-input")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Close))
                                            .tooltip("清除")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                            })),
                                    ),
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
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("解码")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.decode(window, cx);
                                    })),
                            )
                            .when(!error.is_empty(), |row| {
                                row.child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                        .child(error),
                                )
                            }),
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
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("paste-output")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴输出")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_output(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-output")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制输出")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_output(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear-output")
                                            .ghost()
                                            .compact()
                                            .icon(Icon::new(IconName::Close))
                                            .tooltip("清除")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

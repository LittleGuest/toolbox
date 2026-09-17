use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};

use data_encoding::{BASE32, BASE32_NOPAD};

fn base32_encode(s: &str) -> String {
    BASE32.encode(s.as_bytes())
}

fn base32_decode(s: &str) -> Result<String, String> {
    let cleaned: String = s.split_whitespace().collect::<String>().to_uppercase();
    let bytes = BASE32
        .decode(cleaned.as_bytes())
        .or_else(|_| BASE32_NOPAD.decode(cleaned.trim_end_matches('=').as_bytes()))
        .map_err(|e| format!("非法 Base32 字符: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "解码结果不是有效的 UTF-8 文本".to_string())
}

fn base58_encode(s: &str) -> String {
    bs58::encode(s.as_bytes()).into_string()
}

fn base58_decode(s: &str) -> Result<String, String> {
    let bytes = bs58::decode(s.trim())
        .into_vec()
        .map_err(|e| format!("非法 Base58 字符: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "解码结果不是有效的 UTF-8 文本".to_string())
}

pub struct BaseEncodingConverter {
    algo: String,
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    algo_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl BaseEncodingConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要编码 / 解码的文本")
        });
        let output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("编码 / 解码结果将显示在这里"));

        let algo_items = vec!["Base32".to_string(), "Base58".to_string()];
        let algo_state = cx.new(|cx| {
            let mut state = SelectState::new(algo_items, None, window, cx);
            state.set_selected_value(&"Base32".to_string(), window, cx);
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
                &algo_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.algo = match value.as_str() {
                            "Base58" => "base58",
                            _ => "base32",
                        }
                        .to_string();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            algo: "base32".to_string(),
            input: String::new(),
            output: String::new(),
            error: String::new(),
            input_state,
            output_state,
            algo_state,
            _subscriptions,
        }
    }

    fn do_encode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        self.output = match self.algo.as_str() {
            "base58" => base58_encode(&self.input),
            _ => base32_encode(&self.input),
        };
        self.error.clear();
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn do_decode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            return;
        }
        let result = match self.algo.as_str() {
            "base58" => base58_decode(&self.input),
            _ => base32_decode(&self.input),
        };
        match result {
            Ok(decoded) => {
                self.output = decoded;
                self.error.clear();
            }
            Err(e) => {
                self.error = e;
            }
        }
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
    }
}

impl Render for BaseEncodingConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header("Base32 / Base58", "Base32 与 Base58 编码", cx))
            .child(
                // 输入卡片（含算法配置）
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(80.0)).text_sm().child("算法"))
                            .child(Select::new(&self.algo_state)),
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
                                this.do_encode(window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode")
                            .label("解码")
                            .icon(Icon::new(IconName::ArrowUp))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.do_decode(window, cx);
                            })),
                    ),
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
                                Button::new("paste-input")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴输入")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_input(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("paste-output")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴输出")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_output(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-output")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    ),
            )
            .when(!self.error.is_empty(), |this| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(self.error.clone()),
                )
            })
    }
}

use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};

use data_encoding::{BASE32, BASE32_NOPAD};

fn base32_encode(s: &str) -> Result<String, String> {
    Ok(BASE32.encode(s.as_bytes()))
}

fn base32_decode(s: &str) -> Result<String, String> {
    let cleaned: String = s.split_whitespace().collect::<String>().to_uppercase();
    let bytes = BASE32
        .decode(cleaned.as_bytes())
        .or_else(|_| BASE32_NOPAD.decode(cleaned.trim_end_matches('=').as_bytes()))
        .map_err(|e| format!("非法 Base32 字符: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "解码结果不是有效的 UTF-8 文本".to_string())
}

fn base58_encode(s: &str) -> Result<String, String> {
    Ok(bs58::encode(s.as_bytes()).into_string())
}

fn base58_decode(s: &str) -> Result<String, String> {
    let bytes = bs58::decode(s.trim())
        .into_vec()
        .map_err(|e| format!("非法 Base58 字符: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "解码结果不是有效的 UTF-8 文本".to_string())
}

const TABS: [&str; 4] = ["Base64 文本", "Base64 图片", "Base32", "Base58"];

/// 单个编码页的输入 / 输出对（对应 Vue 的 xxxInput / xxxOutput ref 对）
struct PairState {
    input: String,
    output: String,
    error: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
}

pub struct BaseEncodingConverter {
    tab: usize,
    b64: PairState,
    b32: PairState,
    b58: PairState,
    /// 「Base64 图片」Tab（对齐 Tauri BaseEncoding.vue 的 base64img pane）
    b64img: Option<Entity<crate::views::Base64ImageConverter>>,
    _subscriptions: Vec<Subscription>,
}

fn new_pair(window: &mut Window, cx: &mut Context<BaseEncodingConverter>) -> PairState {
    PairState {
        input: String::new(),
        output: String::new(),
        error: String::new(),
        input_state: cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入要编码 / 解码的文本")
        }),
        output_state: cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("编码 / 解码结果将显示在这里")
        }),
    }
}

impl BaseEncodingConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let b64 = new_pair(window, cx);
        let b32 = new_pair(window, cx);
        let b58 = new_pair(window, cx);

        let mut watch_input = |state: &Entity<TextareaState>| {
            let state_for_sub = state.clone();
            let state_in_fn = state.clone();
            cx.subscribe_in(&state_for_sub, window, move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = state_in_fn.read(cx).value().to_string();
                    if this.b64.input_state == state_in_fn {
                        this.b64.input = value;
                    } else if this.b32.input_state == state_in_fn {
                        this.b32.input = value;
                    } else if this.b58.input_state == state_in_fn {
                        this.b58.input = value;
                    }
                    cx.notify();
                }
            })
        };

        let _subscriptions = vec![
            watch_input(&b64.input_state),
            watch_input(&b32.input_state),
            watch_input(&b58.input_state),
        ];

        Self {
            tab: 0,
            b64,
            b32,
            b58,
            b64img: None,
            _subscriptions,
        }
    }

    fn pair(&self, tab: usize) -> &PairState {
        match tab {
            0 => &self.b64,
            2 => &self.b32,
            _ => &self.b58,
        }
    }

    fn pair_mut(&mut self, tab: usize) -> &mut PairState {
        match tab {
            0 => &mut self.b64,
            2 => &mut self.b32,
            _ => &mut self.b58,
        }
    }

    fn run_encode(&mut self, tab: usize, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.pair(tab).input.clone();
        if input.is_empty() {
            return;
        }
        let result = match tab {
            0 => ::base::encode_base64_text(&input).map_err(|e| e.to_string()),
            2 => base32_encode(&input),
            _ => base58_encode(&input),
        };
        let pair = self.pair_mut(tab);
        match result {
            Ok(value) => {
                pair.output = value;
                pair.error.clear();
            }
            Err(e) => pair.error = e,
        }
        pair.sync_output(window, cx);
        cx.notify();
    }

    fn run_decode(&mut self, tab: usize, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.pair(tab).input.clone();
        if input.is_empty() {
            return;
        }
        let result = match tab {
            0 => ::base::decode_base64_text(&input).map_err(|e| e.to_string()),
            2 => base32_decode(&input),
            _ => base58_decode(&input),
        };
        let pair = self.pair_mut(tab);
        match result {
            Ok(value) => {
                pair.output = value;
                pair.error.clear();
            }
            Err(e) => pair.error = e,
        }
        pair.sync_output(window, cx);
        cx.notify();
    }

    fn paste(&mut self, tab: usize, target_output: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                let value = text.to_string();
                let pair = self.pair_mut(tab);
                if target_output {
                    pair.output = value.clone();
                    pair.output_state.update(cx, |state, cx| {
                        state.set_value(value, window, cx);
                    });
                } else {
                    pair.input = value.clone();
                    pair.input_state.update(cx, |state, cx| {
                        state.set_value(value, window, cx);
                    });
                }
                cx.notify();
            }
        }
    }

    fn copy(&mut self, tab: usize, target_output: bool, cx: &mut Context<Self>) {
        let pair = self.pair(tab);
        let value = if target_output {
            pair.output.clone()
        } else {
            pair.input.clone()
        };
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value));
        }
    }

    fn clear(&mut self, tab: usize, window: &mut Window, cx: &mut Context<Self>) {
        let pair = self.pair_mut(tab);
        pair.input.clear();
        pair.output.clear();
        pair.error.clear();
        pair.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        pair.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    /// 线型标签页（对应 n-tabs type="line"）
    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_6()
            .border_b_1()
            .border_color(cx.theme().border)
            .mb_1()
            .children(TABS.iter().enumerate().map(|(idx, label)| {
                let active = self.tab == idx;
                div()
                    .id(("tab", idx))
                    .px_1()
                    .pb_2()
                    .mb(px(-1.0))
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
                    .child(label.to_string())
            }))
    }

    fn render_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let tab = self.tab;
        // 「Base64 图片」Tab：承载独立的 Base64ImageConverter 视图（对齐 Tauri base64img pane）
        if tab == 1 {
            if self.b64img.is_none() {
                self.b64img = Some(cx.new(|cx| {
                    crate::views::Base64ImageConverter::new(window, cx)
                }));
            }
            if let Some(ref img) = self.b64img {
                return div().child(img.clone());
            }
            return div().child("Loading...");
        }

        let input = self.pair(tab).input.clone();
        let output = self.pair(tab).output.clone();
        let error = self.pair(tab).error.clone();
        let input_state = self.pair(tab).input_state.clone();
        let output_state = self.pair(tab).output_state.clone();

        div()
            .flex_col()
            .gap_3()
            // 输入编辑器
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&input_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-in")
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴输入")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.paste(tab, false, window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-in")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输入")
                                    .disabled(input.is_empty())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.copy(tab, false, cx);
                                    })),
                            ),
                    ),
            )
            // 编码 / 解码动作行
            .child(
                design::action_row()
                    .child(
                        Button::new("encode")
                            .primary()
                            .icon(Icon::new(IconName::ArrowDown))
                            .tooltip("编码")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.run_encode(tab, window, cx);
                            })),
                    )
                    .child(
                        Button::new("decode")
                            .primary()
                            .icon(Icon::new(IconName::ArrowUp))
                            .tooltip("解码")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.run_decode(tab, window, cx);
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
                        Textarea::new(&output_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-out")
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴输出")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.paste(tab, true, window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-out")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .disabled(output.is_empty())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.copy(tab, true, cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.clear(tab, window, cx);
                                    })),
                            ),
                    ),
            )
            .when(!error.is_empty(), |this| {
                this.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                        .child(error),
                )
            })
    }
}

impl PairState {
    fn sync_output(&mut self, window: &mut Window, cx: &mut Context<BaseEncodingConverter>) {
        let output = self.output.clone();
        self.output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
    }
}

impl Render for BaseEncodingConverter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()

            .child(
                design::card(cx)

                    .child(self.tab_bar(cx))
                    .child(self.render_tab(window, cx)),
            )
    }
}

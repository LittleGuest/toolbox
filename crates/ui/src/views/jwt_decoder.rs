use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    scroll::ScrollableElement,
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

pub struct JwtDecoder {
    token: String,
    header: String,
    payload: String,
    decoded: String,
    error: String,
    token_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl JwtDecoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let token_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("粘贴 JWT Token")
        });
        let _subscriptions = vec![cx.subscribe_in(&token_state, window, {
            let token_state = token_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.token = token_state.read(cx).value().to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            token: String::new(),
            header: String::new(),
            payload: String::new(),
            decoded: String::new(),
            error: String::new(),
            token_state,
            _subscriptions,
        }
    }

    fn decode(&mut self, cx: &mut Context<Self>) {
        let token = self.token.trim().to_string();
        if token.is_empty() {
            self.error = "请输入 JWT Token".to_string();
            cx.notify();
            return;
        }
        self.error.clear();
        match ::base::decode_jwt(&token) {
            Ok(value) => {
                self.decoded = value.clone();
                match serde_json::from_str::<serde_json::Value>(&value) {
                    Ok(json) => {
                        self.header = serde_json::to_string_pretty(
                            json.get("header").unwrap_or(&serde_json::json!({})),
                        )
                        .unwrap_or_else(|_| "{}".to_string());
                        self.payload = serde_json::to_string_pretty(
                            json.get("payload").unwrap_or(&serde_json::json!({})),
                        )
                        .unwrap_or_else(|_| "{}".to_string());
                    }
                    Err(err) => {
                        self.error = err.to_string();
                    }
                }
            }
            Err(err) => {
                self.header.clear();
                self.payload.clear();
                self.decoded.clear();
                self.error = err.to_string();
            }
        }
        cx.notify();
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.token = text.to_string();
                self.token_state.update(cx, |state, cx| {
                    state.set_value(self.token.clone(), window, cx);
                });
                self.decode(cx);
            }
        }
    }

    fn copy(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }
}

impl Render for JwtDecoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let token_empty = self.token.trim().is_empty();
        let error = self.error.clone();
        let header = self.header.clone();
        let payload = self.payload.clone();
        let decoded = self.decoded.clone();

        design::page()

            .child(
                design::card(cx)

                    // Token 编辑器（tb-editor：标签 + 文本域 + 动作行）
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("Token", cx))
                            .child(
                                Textarea::new(&self.token_state)
                                    .h(px(150.0))
                                    .font_family("monospace"),
                            )
                            .child(
                                design::action_row()
                                    .child(
                                        Button::new("decode")
                                            .primary()
                                            .icon(Icon::new(IconName::Play))
                                            .tooltip("解码")
                                            .disabled(token_empty)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.decode(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("paste")
                                            .icon(Icon::new(IconName::Inbox))
                                            .tooltip("粘贴并解码")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-token")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制 Token")
                                            .disabled(token_empty)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let token = this.token.clone();
                                                this.copy(&token, cx);
                                            })),
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
                    })
                    // Header / Payload 双栏（tb-editor-grid）
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(json_panel(
                                "Header",
                                "copy-header",
                                header,
                                cx,
                            ))
                            .child(json_panel(
                                "Payload",
                                "copy-payload",
                                payload,
                                cx,
                            )),
                    )
                    // 完整解码结果
                    .child(json_panel("完整解码结果", "copy-decoded", decoded, cx)),
            )
    }
}

fn json_panel(
    title: &'static str,
    id: &'static str,
    value: String,
    cx: &mut Context<JwtDecoder>,
) -> Div {
    let display = if value.is_empty() {
        "{}".to_string()
    } else {
        value.clone()
    };
    let palette = HighlightPalette::default_light();
    let highlights = syntax_highlight::json_highlights(&display, &palette);
    let styled = syntax_highlight::styled_text(&display, highlights);
    div()
        .flex()
        .flex_col()
        .gap_2()
        .min_w_0()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(design::editor_label(title, cx))
                .child(
                    Button::new(id)
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .disabled(value.is_empty())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.copy(&value, cx);
                        })),
                ),
        )
        .child(
            div()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .p_3()
                .text_sm()
                .font_family("monospace")
                .h(px(220.0))
                .overflow_y_scrollbar()
                .child(styled),
        )
}

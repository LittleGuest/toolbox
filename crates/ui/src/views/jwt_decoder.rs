use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
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
    token_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl JwtDecoder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let token_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("粘贴 JWT Token...")
                .multi_line(true)
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
            header: "{}".to_string(),
            payload: "{}".to_string(),
            decoded: "{}".to_string(),
            error: String::new(),
            token_state,
            _subscriptions,
        }
    }

    fn decode(&mut self) {
        self.error.clear();
        match base::decode_jwt(self.token.trim()) {
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
                self.header = "{}".to_string();
                self.payload = "{}".to_string();
                self.decoded = "{}".to_string();
                self.error = err.to_string();
            }
        }
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.token = text.to_string();
                self.token_state.update(cx, |state, cx| {
                    state.set_value(self.token.clone(), window, cx);
                });
                self.decode();
            }
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.token.clear();
        self.header = "{}".to_string();
        self.payload = "{}".to_string();
        self.decoded = "{}".to_string();
        self.error.clear();
        self.token_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
    }
}

impl Render for JwtDecoder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let token_empty = self.token.trim().is_empty();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(Input::new(&self.token_state).h(px(130.0)))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("decode")
                            .primary()
                            .disabled(token_empty)
                            .label("解码")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.decode();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("paste")
                            .label("粘贴并解码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("copy-token")
                            .disabled(token_empty)
                            .label("复制 Token")
                            .on_click(cx.listener(|this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    this.token.clone(),
                                ));
                            })),
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
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(div().flex_1().child(json_panel(
                        "Header",
                        "copy-header",
                        self.header.clone(),
                        cx,
                    )))
                    .child(div().flex_1().child(json_panel(
                        "Payload",
                        "copy-payload",
                        self.payload.clone(),
                        cx,
                    ))),
            )
            .child(json_panel(
                "完整解码结果",
                "copy-decoded",
                self.decoded.clone(),
                cx,
            ))
    }
}

fn json_panel(
    title: &'static str,
    id: &'static str,
    value: String,
    cx: &mut Context<JwtDecoder>,
) -> Div {
    let palette = HighlightPalette::default_light();
    let highlights = syntax_highlight::json_highlights(&value, &palette);
    let styled = syntax_highlight::styled_text(&value, highlights);
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_sm().font_semibold().child(title))
                .child(
                    Button::new(id)
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .on_click(cx.listener({
                            let value = value.clone();
                            move |_, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
                            }
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
                .h(px(200.0))
                .overflow_y_scrollbar()
                .child(styled),
        )
}

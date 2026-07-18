use std::time::Duration;

use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    *,
};
use time::OffsetDateTime;

pub struct TimestampConverter {
    input: String,
    timestamp: String,
    datetime: String,
    current_time: String,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

fn format_now_timestamp() -> String {
    // 匹配 Tauri：显示 Unix 时间戳（秒）
    let now = OffsetDateTime::now_utc();
    now.unix_timestamp().to_string()
}

impl TimestampConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入时间戳或日期时间...")
                .multi_line(false)
        });
        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.input = input_state.read(cx).value().to_string();
                    this.convert();
                    cx.notify();
                }
            }
        })];

        let current_time = format_now_timestamp();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let _ = this.update(cx, |this, cx| {
                    this.current_time = format_now_timestamp();
                    cx.notify();
                });
            }
        })
        .detach();

        Self {
            input: String::new(),
            timestamp: String::new(),
            datetime: String::new(),
            current_time,
            input_state,
            _subscriptions,
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.timestamp.clear();
            self.datetime.clear();
            return;
        }

        if let Ok(_ts) = self.input.parse::<i64>() {
            self.timestamp = self.input.clone();
            self.datetime = base::timestamp(Some(&self.input))
                .map(|m| m.get("format").unwrap_or(&String::new()).clone())
                .unwrap_or_default();
        } else if let Ok(result) = base::timestamp(Some(&self.input)) {
            self.timestamp = result.get("format").unwrap_or(&String::new()).clone();
            self.datetime = self.input.clone();
        } else {
            self.timestamp.clear();
            self.datetime.clear();
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.timestamp.clear();
        self.datetime.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
                self.convert();
            }
        }
    }
}

impl Render for TimestampConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let datetime = self.datetime.clone();
        let current_time = self.current_time.clone();

        let datetime_text = if datetime.is_empty() {
            "-".to_string()
        } else {
            datetime.clone()
        };

        div().child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                // Row: 当前时间 → readonly + Copy
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(120.0)).text_sm().child("当前时间"))
                        .child(
                            div()
                                .flex_1()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .font_family("monospace")
                                .child(current_time),
                        )
                        .child(
                            Button::new("copy-current")
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        this.current_time.clone(),
                                    ));
                                })),
                        ),
                )
                // Row: 时间戳 → Input + Paste
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(120.0)).text_sm().child("时间戳"))
                        .child(div().flex_1().child(Input::new(&self.input_state)))
                        .child(
                            Button::new("paste")
                                .icon(Icon::new(IconName::File))
                                .tooltip("粘贴")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.paste(window, cx);
                                    cx.notify();
                                })),
                        ),
                )
                // Row: 时间 → readonly + Copy
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(120.0)).text_sm().child("时间"))
                        .child(
                            div()
                                .flex_1()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .font_family("monospace")
                                .child(datetime_text),
                        )
                        .child(
                            Button::new("copy-datetime")
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        this.datetime.clone(),
                                    ));
                                })),
                        ),
                ),
        )
    }
}

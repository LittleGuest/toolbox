use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

use crate::config_store::{self, ClipboardHistoryItem};

pub struct ClipboardManager {
    input: String,
    keyword: String,
    history: Vec<ClipboardHistoryItem>,
    status: String,
    input_state: Entity<InputState>,
    keyword_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl ClipboardManager {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入内容后复制并记录，或读取当前剪贴板加入历史")
                .multi_line(true)
        });
        let keyword_state = cx.new(|cx| InputState::new(window, cx).placeholder("搜索历史内容"));

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
            cx.subscribe_in(&keyword_state, window, {
                let keyword_state = keyword_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.keyword = keyword_state.read(cx).value().to_string();
                        this.refresh_history(cx);
                    }
                }
            }),
        ];

        Self {
            input: String::new(),
            keyword: String::new(),
            history: Vec::new(),
            status: "剪贴板历史会保存到 SQLite。".to_string(),
            input_state,
            keyword_state,
            _subscriptions,
        }
    }

    fn refresh_history(&mut self, cx: &mut Context<Self>) {
        let keyword = self.keyword.clone();
        self.status = "正在加载剪贴板历史...".to_string();
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_clipboard_history(keyword).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(history) => {
                        this.status = format!("已加载 {} 条剪贴板历史。", history.len());
                        this.history = history;
                    }
                    Err(err) => {
                        this.status = format!("加载剪贴板历史失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn read_current_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = cx.read_from_clipboard() else {
            self.status = "当前剪贴板为空。".to_string();
            cx.notify();
            return;
        };
        let Some(text) = item.text() else {
            self.status = "当前剪贴板不是文本内容。".to_string();
            cx.notify();
            return;
        };
        let text = text.to_string();
        self.input = text.clone();
        self.input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
        });
        self.add_history(text, cx);
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        let value = self.input.trim().to_string();
        if value.is_empty() {
            self.status = "请输入要复制的内容。".to_string();
            cx.notify();
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        self.add_history(value, cx);
    }

    fn copy_history(&mut self, content: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(content.clone()));
        self.add_history(content, cx);
    }

    fn add_history(&mut self, content: String, cx: &mut Context<Self>) {
        self.status = "正在保存剪贴板历史...".to_string();
        cx.notify();
        let keyword = self.keyword.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::add_clipboard_history(content).await;
            let history = match result {
                Ok(_) => config_store::load_clipboard_history(keyword).await,
                Err(err) => Err(err),
            };
            let _ = this.update(cx, |this, cx| {
                match history {
                    Ok(history) => {
                        this.status = "已复制并记录到剪贴板历史。".to_string();
                        this.history = history;
                    }
                    Err(err) => {
                        this.status = format!("保存剪贴板历史失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn delete_history(&mut self, id: i64, cx: &mut Context<Self>) {
        self.status = "正在删除剪贴板历史...".to_string();
        cx.notify();
        let keyword = self.keyword.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_clipboard_history(id).await;
            let history = match result {
                Ok(_) => config_store::load_clipboard_history(keyword).await,
                Err(err) => Err(err),
            };
            let _ = this.update(cx, |this, cx| {
                match history {
                    Ok(history) => {
                        this.status = "剪贴板历史已删除。".to_string();
                        this.history = history;
                    }
                    Err(err) => {
                        this.status = format!("删除剪贴板历史失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn clear_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let this = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, _cx| {
            let this = this.clone();
            dialog
                .title(div().text_lg().font_semibold().child("确认清空"))
                .width(px(420.))
                .child(
                    div()
                        .py_4()
                        .text_sm()
                        .child("确定要清空所有剪贴板历史吗？此操作不可撤销。"),
                )
                .confirm()
                .on_ok(move |_, _, cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.confirm_clear_history(cx);
                        });
                    }
                    true
                })
        });
    }

    fn confirm_clear_history(&mut self, cx: &mut Context<Self>) {
        self.status = "正在清空剪贴板历史...".to_string();
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::clear_clipboard_history().await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(_) => {
                        this.history.clear();
                        this.status = "剪贴板历史已清空。".to_string();
                    }
                    Err(err) => {
                        this.status = format!("清空剪贴板历史失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for ClipboardManager {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.history.is_empty() && self.status == "剪贴板历史会保存到 SQLite。" {
            self.refresh_history(cx);
        }

        let mut history_list = div()
            .h(px(360.0))
            .overflow_y_scrollbar()
            .flex()
            .flex_col()
            .gap_2();
        if self.history.is_empty() {
            history_list = history_list.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("暂无剪贴板历史"),
            );
        } else {
            for item in self.history.clone() {
                history_list = history_list.child(history_item(item, cx));
            }
        }

        div().child(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(Input::new(&self.input_state).h(px(140.0)))
                .child(
                    ButtonGroup::new("clipboard-actions")
                        .child(
                            Button::new("clipboard-copy-input")
                                .primary()
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制并记录")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.copy_input(cx);
                                })),
                        )
                        .child(
                            Button::new("clipboard-read-current")
                                .icon(Icon::new(IconName::File))
                                .tooltip("读取当前剪贴板")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.read_current_clipboard(window, cx);
                                })),
                        )
                        .child(
                            Button::new("clipboard-refresh")
                                .icon(Icon::new(IconName::Search))
                                .tooltip("刷新历史")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.refresh_history(cx);
                                })),
                        )
                        .child(
                            Button::new("clipboard-clear")
                                .icon(Icon::new(IconName::Delete))
                                .tooltip("清空历史")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.clear_history(window, cx);
                                })),
                        ),
                )
                .child(Input::new(&self.keyword_state))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.status.clone()),
                )
                .child(history_list),
        )
    }
}

fn history_item(item: ClipboardHistoryItem, cx: &mut Context<ClipboardManager>) -> Div {
    let content = item.content.clone();
    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .p_2()
        .flex()
        .items_start()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("记录时间：{}", item.created_at)),
                )
                .child(
                    div()
                        .text_sm()
                        .font_family("monospace")
                        .child(truncate_preview(&item.content)),
                ),
        )
        .child(
            ButtonGroup::new(("clipboard-history-actions", item.id as u64))
                .child(
                    Button::new(("clipboard-copy-history", item.id as u64))
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.copy_history(content.clone(), cx);
                        })),
                )
                .child(
                    Button::new(("clipboard-delete-history", item.id as u64))
                        .icon(Icon::new(IconName::Delete))
                        .tooltip("删除")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.delete_history(item.id, cx);
                        })),
                ),
        )
}

fn truncate_preview(value: &str) -> String {
    let max = 220usize;
    let mut preview = value.chars().take(max).collect::<String>();
    if value.chars().count() > max {
        preview.push_str("...");
    }
    preview
}

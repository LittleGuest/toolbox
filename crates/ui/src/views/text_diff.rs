use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    *,
};

#[derive(Clone, Copy, PartialEq)]
enum LineKind {
    Same,
    Delete,
    Insert,
}

struct DiffLine {
    kind: LineKind,
    text: String,
}

pub struct TextDiffTool {
    mode: String,
    left: String,
    right: String,
    lines: Vec<DiffLine>,
    added: usize,
    deleted: usize,
    same: usize,
    compared: bool,
    message: String,
    left_state: Entity<TextareaState>,
    right_state: Entity<TextareaState>,
    mode_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl TextDiffTool {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let left_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入旧文本（JSON 模式为 A）...")
        });
        let right_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入新文本（JSON 模式为 B）...")
        });

        let mode_items = vec!["文本差异".to_string(), "JSON 差异".to_string()];
        let mode_state = cx.new(|cx| {
            let mut state = SelectState::new(mode_items, None, window, cx);
            state.set_selected_value(&"文本差异".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&left_state, window, {
                let left_state = left_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.left = left_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&right_state, window, {
                let right_state = right_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.right = right_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &mode_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.mode = value.clone();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            mode: "文本差异".to_string(),
            left: String::new(),
            right: String::new(),
            lines: Vec::new(),
            added: 0,
            deleted: 0,
            same: 0,
            compared: false,
            message: String::new(),
            left_state,
            right_state,
            mode_state,
            _subscriptions,
        }
    }

    fn reset_result(&mut self) {
        self.lines.clear();
        self.added = 0;
        self.deleted = 0;
        self.same = 0;
        self.compared = false;
    }

    fn run_diff(&mut self, cx: &mut Context<Self>) {
        self.message.clear();

        let (old, new) = if self.mode == "JSON 差异" {
            let a = match serde_json::from_str::<serde_json::Value>(&self.left) {
                Ok(v) => v,
                Err(e) => {
                    self.reset_result();
                    self.message = format!("左侧 JSON 解析失败：{e}");
                    cx.notify();
                    return;
                }
            };
            let b = match serde_json::from_str::<serde_json::Value>(&self.right) {
                Ok(v) => v,
                Err(e) => {
                    self.reset_result();
                    self.message = format!("右侧 JSON 解析失败：{e}");
                    cx.notify();
                    return;
                }
            };
            (
                serde_json::to_string_pretty(&a).unwrap_or_default(),
                serde_json::to_string_pretty(&b).unwrap_or_default(),
            )
        } else {
            (self.left.clone(), self.right.clone())
        };

        if old.is_empty() && new.is_empty() {
            self.reset_result();
            self.message = "请输入要对比的内容".to_string();
            cx.notify();
            return;
        }

        let diff = similar::TextDiff::from_lines(old.as_str(), new.as_str());
        let mut lines = Vec::new();
        let mut added = 0usize;
        let mut deleted = 0usize;
        let mut same = 0usize;
        for change in diff.iter_all_changes() {
            let text = change
                .value()
                .trim_end_matches(|c| c == '\n' || c == '\r')
                .to_string();
            match change.tag() {
                similar::ChangeTag::Equal => {
                    same += 1;
                    lines.push(DiffLine { kind: LineKind::Same, text });
                }
                similar::ChangeTag::Delete => {
                    deleted += 1;
                    lines.push(DiffLine { kind: LineKind::Delete, text });
                }
                similar::ChangeTag::Insert => {
                    added += 1;
                    lines.push(DiffLine { kind: LineKind::Insert, text });
                }
            }
        }
        self.lines = lines;
        self.added = added;
        self.deleted = deleted;
        self.same = same;
        self.compared = true;
        cx.notify();
    }

    fn swap(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        std::mem::swap(&mut self.left, &mut self.right);
        let left = self.left.clone();
        let right = self.right.clone();
        self.left_state.update(cx, |state, cx| {
            state.set_value(left, window, cx);
        });
        self.right_state.update(cx, |state, cx| {
            state.set_value(right, window, cx);
        });
        cx.notify();
    }

    fn copy_left(&mut self, cx: &mut Context<Self>) {
        if !self.left.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.left.clone()));
        }
    }

    fn copy_right(&mut self, cx: &mut Context<Self>) {
        if !self.right.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.right.clone()));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.left.clear();
        self.right.clear();
        self.message.clear();
        self.reset_result();
        self.left_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.right_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for TextDiffTool {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(80.0);
        let show_result = self.compared && self.message.is_empty();
        let identical = show_result && self.added == 0 && self.deleted == 0;

        let diff_area = if show_result && !identical && !self.lines.is_empty() {
            let mut container = div()
                .h(px(420.0))
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .flex_1()
                .font_family("monospace")
                .text_sm();
            for line in &self.lines {
                let (prefix, color) = match line.kind {
                    LineKind::Insert => ("+ ", rgb(0x18a058).into()),
                    LineKind::Delete => ("- ", rgb(0xd03050).into()),
                    LineKind::Same => ("  ", cx.theme().muted_foreground),
                };
                container = container.child(
                    div()
                        .text_color(color)
                        .child(format!("{prefix}{}", line.text)),
                );
            }
            Some(container.into_any_element())
        } else {
            None
        };

        design::page()
            .child(design::page_header("文本 / JSON 差异", "对比文本或 JSON", cx))
            .child(
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("模式", cx)))
                            .child(Select::new(&self.mode_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.left_state)
                                    .h(px(180.0))
                                    .flex_1()
                                    .font_family("monospace"),
                            ),
                    )
                    .child(
                        design::action_row()
                            .child(
                                Button::new("run-diff")
                                    .primary()
                                    .label("对比")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.run_diff(cx);
                                    })),
                            )
                            .child(
                                Button::new("swap")
                                    .label("交换")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.swap(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.right_state)
                                    .h(px(180.0))
                                    .flex_1()
                                    .font_family("monospace"),
                            ),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("copy-left")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制左侧")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_left(cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-right")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制右侧")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_right(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    )
                    .when(show_result, |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_4()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x18a058))
                                        .child(format!("新增 {} 行", self.added)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xd03050))
                                        .child(format!("删除 {} 行", self.deleted)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("相同 {} 行", self.same)),
                                ),
                        )
                    })
                    .when(identical, |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x18a058))
                                        .child("两个内容完全相同"),
                                ),
                        )
                    })
                    .when_some(diff_area, |this, area| {
                        this.child(
                            div()
                                .flex()
                                .items_start()
                                .gap_2()
                                .child(div().flex_1().child(area)),
                        )
                    })
                    .when(!self.message.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xd03050))
                                        .child(self.message.clone()),
                                ),
                        )
                    }),
            )
    }
}

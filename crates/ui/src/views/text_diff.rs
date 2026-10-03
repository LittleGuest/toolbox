use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    scroll::ScrollableElement,
    *,
};
use serde_json::Value;

const MODE_OPTIONS: [&str; 2] = ["文本差异", "JSON 差异"];

#[derive(Clone, Copy, PartialEq)]
enum CellKind {
    Same,
    Delete,
    Add,
    Blank,
}

/// 文本差异单元格（Vue LineCell：text/type/no）
struct LineCell {
    kind: CellKind,
    text: String,
    no: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum JsonKind {
    Add,
    Del,
    Modify,
}

/// JSON 差异条目（Vue JsonDiffItem：path/type/oldText/newText）
struct JsonDiffItem {
    path: String,
    kind: JsonKind,
    old_text: String,
    new_text: String,
}

pub struct TextDiffTool {
    mode: String,
    left: String,
    right: String,
    left_cells: Vec<LineCell>,
    right_cells: Vec<LineCell>,
    added: usize,
    deleted: usize,
    same: usize,
    json_items: Vec<JsonDiffItem>,
    json_add: usize,
    json_del: usize,
    json_modify: usize,
    text_compared: bool,
    json_compared: bool,
    message: String,
    left_state: Entity<TextareaState>,
    right_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

fn truncate80(s: &str) -> String {
    if s.chars().count() > 80 {
        format!("{}…", s.chars().take(80).collect::<String>())
    } else {
        s.to_string()
    }
}

fn fmt_value(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        other => serde_json::to_string(other).unwrap_or_else(|_| other.to_string()),
    }
}

/// 递归对比两个 JSON 值（对齐 Vue diffValue）
fn diff_value(a: &Value, b: &Value, path: &str, out: &mut Vec<JsonDiffItem>) {
    let a_container = a.is_object() || a.is_array();
    let b_container = b.is_object() || b.is_array();
    let label = if path.is_empty() { "(root)" } else { path };

    if a_container && b_container {
        if a.is_array() && b.is_array() {
            let aa = a.as_array().unwrap();
            let bb = b.as_array().unwrap();
            let len = aa.len().min(bb.len());
            for i in 0..len {
                diff_value(&aa[i], &bb[i], &format!("{path}[{i}]"), out);
            }
            for i in len..aa.len() {
                out.push(JsonDiffItem {
                    path: format!("{path}[{i}]"),
                    kind: JsonKind::Del,
                    old_text: truncate80(&fmt_value(&aa[i])),
                    new_text: String::new(),
                });
            }
            for i in len..bb.len() {
                out.push(JsonDiffItem {
                    path: format!("{path}[{i}]"),
                    kind: JsonKind::Add,
                    old_text: String::new(),
                    new_text: truncate80(&fmt_value(&bb[i])),
                });
            }
        } else if a.is_array() != b.is_array() {
            out.push(JsonDiffItem {
                path: label.to_string(),
                kind: JsonKind::Modify,
                old_text: truncate80(&fmt_value(a)),
                new_text: truncate80(&fmt_value(b)),
            });
        } else {
            let ra = a.as_object().unwrap();
            let rb = b.as_object().unwrap();
            for (k, av) in ra {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match rb.get(k) {
                    None => out.push(JsonDiffItem {
                        path: p,
                        kind: JsonKind::Del,
                        old_text: truncate80(&fmt_value(av)),
                        new_text: String::new(),
                    }),
                    Some(bv) => diff_value(av, bv, &p, out),
                }
            }
            for (k, bv) in rb {
                if !ra.contains_key(k) {
                    let p = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    out.push(JsonDiffItem {
                        path: p,
                        kind: JsonKind::Add,
                        old_text: String::new(),
                        new_text: truncate80(&fmt_value(bv)),
                    });
                }
            }
        }
    } else if a == b {
        return;
    } else {
        out.push(JsonDiffItem {
            path: label.to_string(),
            kind: JsonKind::Modify,
            old_text: truncate80(&fmt_value(a)),
            new_text: truncate80(&fmt_value(b)),
        });
    }
}

impl TextDiffTool {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let left_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入旧文本（JSON 模式为 A）...")
        });
        let right_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入新文本（JSON 模式为 B）...")
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
        ];

        Self {
            mode: "文本差异".to_string(),
            left: String::new(),
            right: String::new(),
            left_cells: Vec::new(),
            right_cells: Vec::new(),
            added: 0,
            deleted: 0,
            same: 0,
            json_items: Vec::new(),
            json_add: 0,
            json_del: 0,
            json_modify: 0,
            text_compared: false,
            json_compared: false,
            message: String::new(),
            left_state,
            right_state,
            _subscriptions,
        }
    }

    fn is_json_mode(&self) -> bool {
        self.mode == "JSON 差异"
    }

    fn reset_result(&mut self) {
        self.left_cells.clear();
        self.right_cells.clear();
        self.added = 0;
        self.deleted = 0;
        self.same = 0;
        self.json_items.clear();
        self.json_add = 0;
        self.json_del = 0;
        self.json_modify = 0;
        self.text_compared = false;
        self.json_compared = false;
    }

    /// 文本差异：构建左右两栏单元格（对齐 Vue runTextDiff）
    fn run_text_diff(&mut self, cx: &mut Context<Self>) {
        let a: Vec<&str> = self.left.split('\n').collect();
        let b: Vec<&str> = self.right.split('\n').collect();
        if a.len() * b.len() > 4_000_000 {
            self.reset_result();
            self.message = "内容过大".to_string();
            cx.notify();
            return;
        }

        let diff = similar::TextDiff::from_lines(&self.left, &self.right);
        let mut left_cells = Vec::new();
        let mut right_cells = Vec::new();
        let mut oi = 0usize;
        let mut ni = 0usize;
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
                    oi += 1;
                    ni += 1;
                    same += 1;
                    left_cells.push(LineCell { kind: CellKind::Same, text: text.clone(), no: oi });
                    right_cells.push(LineCell { kind: CellKind::Same, text, no: ni });
                }
                similar::ChangeTag::Delete => {
                    oi += 1;
                    deleted += 1;
                    left_cells.push(LineCell { kind: CellKind::Delete, text, no: oi });
                    right_cells.push(LineCell { kind: CellKind::Blank, text: String::new(), no: 0 });
                }
                similar::ChangeTag::Insert => {
                    ni += 1;
                    added += 1;
                    left_cells.push(LineCell { kind: CellKind::Blank, text: String::new(), no: 0 });
                    right_cells.push(LineCell { kind: CellKind::Add, text, no: ni });
                }
            }
        }

        self.left_cells = left_cells;
        self.right_cells = right_cells;
        self.added = added;
        self.deleted = deleted;
        self.same = same;
        self.text_compared = true;
        cx.notify();
    }

    /// JSON 差异：结构化对比（对齐 Vue runJsonDiff）
    fn run_json_diff(&mut self, cx: &mut Context<Self>) {
        let a: Value = match serde_json::from_str(&self.left) {
            Ok(v) => v,
            Err(e) => {
                self.reset_result();
                self.message = format!("JSON 解析失败：{e}");
                cx.notify();
                return;
            }
        };
        let b: Value = match serde_json::from_str(&self.right) {
            Ok(v) => v,
            Err(e) => {
                self.reset_result();
                self.message = format!("JSON 解析失败：{e}");
                cx.notify();
                return;
            }
        };

        let mut items: Vec<JsonDiffItem> = Vec::new();
        diff_value(&a, &b, "", &mut items);
        self.json_add = items.iter().filter(|i| i.kind == JsonKind::Add).count();
        self.json_del = items.iter().filter(|i| i.kind == JsonKind::Del).count();
        self.json_modify = items.iter().filter(|i| i.kind == JsonKind::Modify).count();
        self.json_items = items;
        self.json_compared = true;
        cx.notify();
    }

    fn run_diff(&mut self, cx: &mut Context<Self>) {
        self.message.clear();
        if self.is_json_mode() {
            self.run_json_diff(cx);
        } else {
            self.run_text_diff(cx);
        }
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
        // 对齐 Vue：交换后若已有结果则重新对比
        let rerun = if self.is_json_mode() {
            self.json_compared
        } else {
            self.text_compared
        };
        if rerun {
            self.run_diff(cx);
        } else {
            cx.notify();
        }
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

    /// 分段选择器（对应 n-radio-button 组：文本差异/JSON 差异）
    fn mode_segmented(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(MODE_OPTIONS.iter().enumerate().map(|(i, opt)| {
                let active = self.mode == *opt;
                div()
                    .id(("mode", i))
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
                        this.mode = opt.to_string();
                        cx.notify();
                    }))
                    .child(opt.to_string())
            }))
    }

    /// 差异面板（Vue diff-pane：行号 + 前缀 + 文本 + tint 高亮）
    fn render_pane(&self, cells: &[LineCell], cx: &mut Context<Self>) -> impl IntoElement {
        let mut pane = div()
            .flex_1()
            .min_w(px(240.0))
            .max_h(px(420.0))
            .overflow_y_scrollbar()
            .flex_col()
            .rounded(px(8.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .py_1()
            .font_family("monospace")
            .text_size(px(12.0));
        for line in cells {
            let (prefix, text_color, bg) = match line.kind {
                CellKind::Delete => (
                    "-",
                    Hsla::from(rgb(0xb0302a)),
                    design::tint(design::ERROR_RED, 0.1),
                ),
                CellKind::Add => (
                    "+",
                    Hsla::from(rgb(0x15803d)),
                    design::tint(design::OK_GREEN, 0.1),
                ),
                CellKind::Same => ("", cx.theme().foreground, gpui::black().opacity(0.0)),
                CellKind::Blank => ("", cx.theme().muted_foreground, gpui::black().opacity(0.0)),
            };
            let text = line.text.clone();
            let no = if line.no > 0 {
                line.no.to_string()
            } else {
                String::new()
            };
            pane = pane.child(
                div()
                    .flex()
                    .px(px(6.0))
                    .bg(bg)
                    .text_color(text_color)
                    .child(
                        div()
                            .w(px(38.0))
                            .flex_shrink_0()
                            .text_right()
                            .mr_2()
                            .text_color(cx.theme().muted_foreground)
                            .child(no),
                    )
                    .child(
                        div()
                            .w(px(12.0))
                            .flex_shrink_0()
                            .font_semibold()
                            .child(prefix.to_string()),
                    )
                    .child(div().min_w_0().child(text)),
            );
        }
        pane
    }
}

impl Render for TextDiffTool {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let message = self.message.clone();
        let is_json = self.is_json_mode();
        let show_text = !is_json && self.text_compared && self.message.is_empty();
        let show_json = is_json && self.json_compared && self.message.is_empty();
        let json_identical = show_json && self.json_items.is_empty();

        let copy_left_label = if is_json { "复制 A" } else { "复制左侧" };
        let copy_right_label = if is_json { "复制 B" } else { "复制右侧" };

        design::page()

            .child(
                design::card(cx)

                    // 输入行（input-row：原始 | 动作列 | 对比）
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_3()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(design::editor_label("原始", cx))
                                    .child(
                                        Textarea::new(&self.left_state)
                                            .h(px(180.0))
                                            .font_family("monospace"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_col()
                                    .gap_2()
                                    .pt(px(22.0))
                                    .flex_shrink_0()
                                    .child(
                                        Button::new("run-diff")
                                            .primary()
                                            .icon(Icon::new(IconName::Search))
                                            .tooltip("对比")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.run_diff(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("swap")
                                            .icon(Icon::new(IconName::Replace))
                                            .tooltip("交换")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.swap(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(design::editor_label("对比", cx))
                                    .child(
                                        Textarea::new(&self.right_state)
                                            .h(px(180.0))
                                            .font_family("monospace"),
                                    ),
                            ),
                    )
                    // 模式条（mode-bar）
                    .child(
                        div().mt_1().child(self.mode_segmented(cx)),
                    )
                    .when(!message.is_empty(), |card| {
                        card.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(message.clone()),
                        )
                    })
                    // 文本差异结果
                    .when(show_text, |card| {
                        card.child(
                            div()
                                .mt_1()
                                .flex()
                                .flex_wrap()
                                .gap_2()
                                .child(design::mini_tag(
                                    format!("新增 {} 行", self.added),
                                    design::tint(design::OK_GREEN, 0.12),
                                    Hsla::from(rgb(design::OK_GREEN)),
                                ))
                                .child(design::mini_tag(
                                    format!("删除 {} 行", self.deleted),
                                    design::tint(design::ERROR_RED, 0.12),
                                    Hsla::from(rgb(design::ERROR_RED)),
                                ))
                                .child(design::mini_tag(
                                    format!("相同 {} 行", self.same),
                                    cx.theme().secondary,
                                    cx.theme().muted_foreground,
                                )),
                        )
                        .child(
                            div().mt_2().flex().gap_3().child(self.render_pane(
                                &self.left_cells,
                                cx,
                            ))
                            .child(self.render_pane(&self.right_cells, cx)),
                        )
                    })
                    // JSON 差异结果
                    .when(json_identical, |card| {
                        card.child(
                            div()
                                .mt_1()
                                .text_size(px(13.0))
                                .text_color(Hsla::from(rgb(design::OK_GREEN)))
                                .child("两个 JSON 完全相同"),
                        )
                    })
                    .when(show_json && !json_identical, |card| {
                        card.child(
                            div()
                                .mt_1()
                                .flex()
                                .flex_wrap()
                                .gap_2()
                                .child(design::mini_tag(
                                    format!("新增 {}", self.json_add),
                                    design::tint(design::OK_GREEN, 0.12),
                                    Hsla::from(rgb(design::OK_GREEN)),
                                ))
                                .child(design::mini_tag(
                                    format!("删除 {}", self.json_del),
                                    design::tint(design::ERROR_RED, 0.12),
                                    Hsla::from(rgb(design::ERROR_RED)),
                                ))
                                .child(design::mini_tag(
                                    format!("修改 {}", self.json_modify),
                                    design::tint(design::WARN_AMBER, 0.15),
                                    Hsla::from(rgb(design::WARN_AMBER)),
                                )),
                        )
                        .child(self.render_json_items(cx))
                    })
                    // 底部动作条（bottom-bar）
                    .child(
                        div()
                            .mt_2()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("copy-left")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip(copy_left_label)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_left(cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-right")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip(copy_right_label)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_right(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

impl TextDiffTool {
    /// JSON 差异列表（Vue diff-list / diff-item）
    fn render_json_items(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div()
            .mt_2()
            .max_h(px(460.0))
            .overflow_y_scrollbar()
            .flex_col()
            .rounded(px(8.0))
            .border_1()
            .border_color(cx.theme().border);

        for (i, item) in self.json_items.iter().enumerate() {
            let (tag_text, tag_bg, tag_fg) = match item.kind {
                JsonKind::Add => (
                    "新增",
                    design::tint(design::OK_GREEN, 0.12),
                    Hsla::from(rgb(design::OK_GREEN)),
                ),
                JsonKind::Del => (
                    "删除",
                    design::tint(design::ERROR_RED, 0.12),
                    Hsla::from(rgb(design::ERROR_RED)),
                ),
                JsonKind::Modify => (
                    "修改",
                    design::tint(design::WARN_AMBER, 0.15),
                    Hsla::from(rgb(design::WARN_AMBER)),
                ),
            };
            let old_color = Hsla::from(rgb(0xb0302a));
            let new_color = Hsla::from(rgb(0x15803d));
            let old_text = item.old_text.clone();
            let new_text = item.new_text.clone();
            let has_old = !old_text.is_empty();
            let has_new = !new_text.is_empty();

            let mut values = div().flex().flex_wrap().items_start().gap_2();
            if has_old {
                values = values.child(
                    div()
                        .font_family("monospace")
                        .text_size(px(12.0))
                        .text_color(old_color)
                        .child(old_text),
                );
            }
            if has_old && has_new {
                values = values.child(
                    div()
                        .flex_shrink_0()
                        .text_color(cx.theme().muted_foreground)
                        .child("→"),
                );
            }
            if has_new {
                values = values.child(
                    div()
                        .font_family("monospace")
                        .text_size(px(12.0))
                        .text_color(new_color)
                        .child(new_text),
                );
            }

            list = list.child(
                div()
                    .id(("json-item", i))
                    .flex_col()
                    .gap_1p5()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .child(
                                div()
                                    .font_family("monospace")
                                    .text_size(px(13.0))
                                    .font_semibold()
                                    .min_w_0()
                                    .truncate()
                                    .child(item.path.clone()),
                            )
                            .child(design::mini_tag(tag_text, tag_bg, tag_fg)),
                    )
                    .child(values),
            );
        }

        list
    }
}

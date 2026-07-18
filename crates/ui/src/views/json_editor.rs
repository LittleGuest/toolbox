use std::collections::HashSet;

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

/// 树查看器选项开关
#[derive(Clone, Copy, Default)]
struct TreeOptions {
    show_length: bool,
    show_line_number: bool,
    show_icon: bool,
}

pub struct JsonEditor {
    input: String,
    /// 解析后的 JSON 值（用于树查看器）
    parsed: Option<serde_json::Value>,
    /// 树查看器错误
    tree_error: String,
    /// 展开的节点路径集合
    expanded: HashSet<String>,
    /// 树查看器选项
    tree_options: TreeOptions,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl JsonEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入JSON...")
                .multi_line(true)
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    // 输入变化时自动解析
                    this.parse_input();
                    cx.notify();
                }
            }
        })];

        let mut this = Self {
            input: String::new(),
            parsed: None,
            tree_error: String::new(),
            expanded: HashSet::new(),
            tree_options: TreeOptions {
                show_length: true,
                show_line_number: false,
                show_icon: true,
            },
            input_state,
            _subscriptions,
        };
        this.parse_input();
        this
    }

    fn parse_input(&mut self) {
        if self.input.trim().is_empty() {
            self.parsed = None;
            self.tree_error.clear();
            return;
        }
        match serde_json::from_str::<serde_json::Value>(&self.input) {
            Ok(parsed) => {
                self.parsed = Some(parsed);
                self.tree_error.clear();
            }
            Err(_) => {
                self.parsed = None;
                self.tree_error = "无法解析为 JSON".to_string();
            }
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.parsed = None;
        self.tree_error.clear();
        self.expanded.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                self.parse_input();
            }
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    /// 切换树节点展开/折叠
    fn toggle_node(&mut self, path: &str, cx: &mut Context<Self>) {
        if self.expanded.contains(path) {
            self.expanded.remove(path);
        } else {
            self.expanded.insert(path.to_string());
        }
        cx.notify();
    }

    /// 全部展开（递归收集所有 object/array 节点路径）
    fn expand_all(&mut self, cx: &mut Context<Self>) {
        if let Some(ref value) = self.parsed {
            self.expanded.clear();
            collect_all_paths(value, "", &mut self.expanded);
        }
        cx.notify();
    }

    /// 全部折叠
    fn collapse_all(&mut self, cx: &mut Context<Self>) {
        self.expanded.clear();
        cx.notify();
    }

    fn toggle_show_length(&mut self, cx: &mut Context<Self>) {
        self.tree_options.show_length = !self.tree_options.show_length;
        cx.notify();
    }

    fn toggle_show_line_number(&mut self, cx: &mut Context<Self>) {
        self.tree_options.show_line_number = !self.tree_options.show_line_number;
        cx.notify();
    }

    fn toggle_show_icon(&mut self, cx: &mut Context<Self>) {
        self.tree_options.show_icon = !self.tree_options.show_icon;
        cx.notify();
    }
}

/// 递归收集所有 object/array 节点路径（用于「全部展开」）
fn collect_all_paths(value: &serde_json::Value, prefix: &str, paths: &mut HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if !prefix.is_empty() {
                paths.insert(prefix.to_string());
            }
            for (key, val) in map {
                let child_path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_all_paths(val, &child_path, paths);
            }
        }
        serde_json::Value::Array(arr) => {
            if !prefix.is_empty() {
                paths.insert(prefix.to_string());
            }
            for (i, val) in arr.iter().enumerate() {
                let child_path = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                collect_all_paths(val, &child_path, paths);
            }
        }
        _ => {}
    }
}

/// 判断 JSON 值是否可展开（是 object 或 array 且非空）
fn is_expandable(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(m) => !m.is_empty(),
        serde_json::Value::Array(a) => !a.is_empty(),
        _ => false,
    }
}

/// 获取 object/array 的子节点数量
fn child_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(m) => m.len(),
        serde_json::Value::Array(a) => a.len(),
        _ => 0,
    }
}

/// 获取 JSON 值的类型标签文本
fn type_label(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Null => "null",
    }
}

/// 获取叶子节点的显示值（带引号的字符串、数字、true/false/null）
fn leaf_display(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("\"{s}\""),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => String::new(),
    }
}

impl Render for JsonEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 匹配 Tauri JsonEditor.vue: ButtonGroup(Paste+Copy) + VueJsonPretty
        div().child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                // 操作按钮（匹配 Tauri n-button-group）
                .child(
                    ButtonGroup::new("input-buttons")
                        .child(
                            Button::new("paste-input")
                                .icon(Icon::new(IconName::File))
                                .tooltip("粘贴")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.paste(window, cx);
                                })),
                        )
                        .child(
                            Button::new("copy-input")
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.copy_input(cx);
                                })),
                        ),
                )
                // 树查看器（匹配 Tauri VueJsonPretty）
                .child(tree_viewer_panel(self, cx)),
        )
    }
}

/// 渲染交互式树查看器面板（匹配 Tauri VueJsonPretty：工具栏内嵌在树区域顶部）
fn tree_viewer_panel(this: &JsonEditor, cx: &mut Context<JsonEditor>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        // 工具栏（内嵌在树查看器顶部，匹配 VueJsonPretty 的内置控制）
        .child(
            div().flex().items_center().gap_2().child(
                ButtonGroup::new("tree-toolbar")
                    .child(
                        Button::new("tree-expand-all")
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("全部展开")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.expand_all(cx);
                            })),
                    )
                    .child(
                        Button::new("tree-collapse-all")
                            .icon(Icon::new(IconName::Minus))
                            .tooltip("全部折叠")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.collapse_all(cx);
                            })),
                    )
                    .child(
                        Button::new("tree-toggle-length")
                            .child("长度")
                            .selected(this.tree_options.show_length)
                            .tooltip("显示子节点数量")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_show_length(cx);
                            })),
                    )
                    .child(
                        Button::new("tree-toggle-line")
                            .child("行号")
                            .selected(this.tree_options.show_line_number)
                            .tooltip("显示行号")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_show_line_number(cx);
                            })),
                    )
                    .child(
                        Button::new("tree-toggle-icon")
                            .child("图标")
                            .selected(this.tree_options.show_icon)
                            .tooltip("显示类型图标")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_show_icon(cx);
                            })),
                    ),
            ),
        )
        // 树内容
        .child(render_tree_viewer(this, cx))
}

/// 渲染树查看器内容
fn render_tree_viewer(this: &JsonEditor, cx: &mut Context<JsonEditor>) -> Div {
    let inner = div()
        .p_3()
        .h_full()
        .overflow_y_scrollbar()
        .text_sm()
        .font_family("monospace")
        .flex()
        .flex_col()
        .gap_1();

    let inner = if !this.tree_error.is_empty() {
        inner.child(
            div()
                .text_color(cx.theme().danger)
                .child(this.tree_error.clone()),
        )
    } else if let Some(ref value) = this.parsed {
        let mut line_counter = 1usize;
        inner.child(render_node(
            value,
            "$root",
            "$root",
            0,
            &this.expanded,
            this.tree_options,
            &mut line_counter,
            cx,
        ))
    } else {
        inner.child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child("请输入 JSON 内容以显示树查看器"),
        )
    };

    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .h(px(280.0))
        .child(inner)
}

/// 递归渲染一个 JSON 节点
fn render_node(
    value: &serde_json::Value,
    key: &str,
    path: &str,
    depth: usize,
    expanded: &HashSet<String>,
    options: TreeOptions,
    line_counter: &mut usize,
    cx: &mut Context<JsonEditor>,
) -> Div {
    let expandable = is_expandable(value);
    let is_expanded = expandable && (depth == 0 || expanded.contains(path));

    // 行容器
    let mut row = div()
        .flex()
        .items_center()
        .gap_1()
        .pl(px(8.0 * depth as f32)); // 缩进：每层 8px

    // 行号
    if options.show_line_number {
        row = row.child(
            div()
                .w(px(32.0))
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", line_counter)),
        );
    }
    *line_counter += 1;

    // 展开/折叠箭头
    if expandable {
        let arrow_text = if is_expanded { "▼" } else { "▶" };
        let path_clone = path.to_string();
        row = row.child(
            div()
                .w(px(16.0))
                .cursor_pointer()
                .child(arrow_text)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.toggle_node(&path_clone, cx);
                    }),
                ),
        );
    } else {
        row = row.child(div().w(px(16.0)));
    }

    // 类型图标
    if options.show_icon {
        let icon = match value {
            serde_json::Value::Object(_) => "{}",
            serde_json::Value::Array(_) => "[]",
            serde_json::Value::String(_) => "S",
            serde_json::Value::Number(_) => "N",
            serde_json::Value::Bool(_) => "B",
            serde_json::Value::Null => "·",
        };
        let icon_color = match value {
            serde_json::Value::Object(_) | serde_json::Value::Array(_) => cx.theme().primary,
            serde_json::Value::String(_) => gpui::hsla(0.33, 0.6, 0.4, 1.0),
            serde_json::Value::Number(_) => gpui::hsla(0.08, 0.7, 0.5, 1.0),
            serde_json::Value::Bool(_) => gpui::hsla(0.0, 0.65, 0.5, 1.0),
            serde_json::Value::Null => cx.theme().muted_foreground,
        };
        row = row.child(
            div()
                .w(px(24.0))
                .text_color(icon_color)
                .child(icon.to_string()),
        );
    }

    // 键名（根节点不显示键名）
    if depth > 0 {
        row = row.child(
            div()
                .text_color(gpui::hsla(0.75, 0.55, 0.45, 1.0))
                .child(format!("\"{key}\":")),
        );
    }

    // 值或类型标签
    if expandable {
        let label = type_label(value);
        let count = child_count(value);
        row = row.child(div().text_color(cx.theme().muted_foreground).child(
            if options.show_length {
                format!("{label} ({count})")
            } else {
                label.to_string()
            },
        ));
    } else {
        // 叶子节点值
        let display = leaf_display(value);
        let color = match value {
            serde_json::Value::String(_) => gpui::hsla(0.33, 0.6, 0.4, 1.0),
            serde_json::Value::Number(_) => gpui::hsla(0.08, 0.7, 0.5, 1.0),
            serde_json::Value::Bool(_) => gpui::hsla(0.0, 0.65, 0.5, 1.0),
            serde_json::Value::Null => cx.theme().muted_foreground,
            _ => cx.theme().foreground,
        };
        row = row.child(div().text_color(color).child(display));
    }

    // 递归渲染子节点
    let mut result = div().flex().flex_col().gap_1().child(row);

    if is_expanded && expandable {
        match value {
            serde_json::Value::Object(map) => {
                for (child_key, child_value) in map {
                    let child_path = format!("{path}.{child_key}");
                    result = result.child(render_node(
                        child_value,
                        child_key,
                        &child_path,
                        depth + 1,
                        expanded,
                        options,
                        line_counter,
                        cx,
                    ));
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, child_value) in arr.iter().enumerate() {
                    let child_key = i.to_string();
                    let child_path = format!("{path}.{i}");
                    result = result.child(render_node(
                        child_value,
                        &child_key,
                        &child_path,
                        depth + 1,
                        expanded,
                        options,
                        line_counter,
                        cx,
                    ));
                }
            }
            _ => {}
        }
    }

    result
}

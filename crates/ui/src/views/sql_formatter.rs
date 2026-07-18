use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    switch::Switch,
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

pub struct SqlFormatter {
    input: String,
    indent: usize,
    keyword_case: String,
    input_state: Entity<InputState>,
    indent_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl SqlFormatter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入SQL语句...")
                .multi_line(true)
        });

        let indent_state = cx.new(|cx| {
            let mut s = SelectState::new(vec!["2".to_string(), "4".to_string()], None, window, cx);
            s.set_selected_value(&"2".to_string(), window, cx);
            s
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
                &indent_state,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.indent = value.parse().unwrap_or(2);
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            input: String::new(),
            indent: 2,
            keyword_case: "upper".to_string(),
            input_state,
            indent_state,
            _subscriptions,
        }
    }

    fn format_sql(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            return;
        }

        let keywords = [
            "SELECT",
            "FROM",
            "WHERE",
            "AND",
            "OR",
            "JOIN",
            "LEFT",
            "RIGHT",
            "INNER",
            "OUTER",
            "ON",
            "GROUP",
            "BY",
            "ORDER",
            "HAVING",
            "LIMIT",
            "OFFSET",
            "INSERT",
            "INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE",
            "CREATE",
            "TABLE",
            "DROP",
            "ALTER",
            "ADD",
            "COLUMN",
            "PRIMARY",
            "KEY",
            "FOREIGN",
            "REFERENCES",
            "UNIQUE",
            "INDEX",
            "AS",
            "DISTINCT",
            "UNION",
            "ALL",
            "EXISTS",
            "IN",
            "BETWEEN",
            "LIKE",
            "IS",
            "NULL",
            "NOT",
            "ASC",
            "DESC",
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            "WITH",
            "RECURSIVE",
        ];

        let indent_str = " ".repeat(self.indent);
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut depth = 0;

        let tokens = self.tokenize_sql(&self.input);
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];
            let upper_token = token.to_uppercase();

            if upper_token == "SELECT" && i == 0 {
                current_line.push_str(&format!("{}{}", indent_str.repeat(depth), token));
                depth += 1;
            } else if upper_token == "FROM" {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                depth = depth.saturating_sub(1);
                current_line.push_str(&format!("{}{}", indent_str.repeat(depth), token));
                depth += 1;
            } else if upper_token == "WHERE" || upper_token == "GROUP" || upper_token == "ORDER" {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                depth = depth.saturating_sub(1);
                current_line.push_str(&format!("{}{}", indent_str.repeat(depth), token));
            } else if upper_token == "AND" || upper_token == "OR" {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                current_line.push_str(&format!("{}{}", indent_str.repeat(depth), token));
            } else if upper_token == "JOIN" {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                depth = depth.saturating_sub(1);
                current_line.push_str(&format!("{}{}", indent_str.repeat(depth), token));
                depth += 1;
            } else if upper_token == "ON" {
                current_line.push(' ');
                current_line.push_str(token);
                depth += 1;
            } else if upper_token == "LEFT"
                || upper_token == "RIGHT"
                || upper_token == "INNER"
                || upper_token == "OUTER"
            {
                current_line.push(' ');
                current_line.push_str(token);
            } else if upper_token == "(" {
                current_line.push('(');
                depth += 1;
            } else if upper_token == ")" {
                depth = depth.saturating_sub(1);
                current_line.push(')');
            } else if upper_token == "," {
                current_line.push(',');
                lines.push(current_line.clone());
                current_line.clear();
            } else {
                if !current_line.is_empty() && !current_line.ends_with('(') {
                    current_line.push(' ');
                }
                current_line.push_str(token);
            }

            i += 1;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let mut formatted = lines.join("\n");

        if self.keyword_case == "upper" {
            formatted = self.apply_keyword_case(&formatted, true, &keywords);
        } else {
            formatted = self.apply_keyword_case(&formatted, false, &keywords);
        }

        self.input = formatted;
        self.input_state.update(cx, |state, cx| {
            state.set_value(self.input.clone(), window, cx);
        });
    }

    fn tokenize_sql(&self, sql: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut chars = sql.chars().peekable();

        while let Some(c) = chars.next() {
            if in_string {
                current_token.push(c);
                if c == string_char {
                    in_string = false;
                }
                continue;
            }

            if c == '\'' || c == '"' || c == '`' {
                in_string = true;
                string_char = c;
                current_token.push(c);
                continue;
            }

            if c.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                continue;
            }

            if c == '(' || c == ')' || c == ',' || c == ';' {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                tokens.push(c.to_string());
                continue;
            }

            current_token.push(c);
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    fn apply_keyword_case(&self, sql: &str, upper: bool, keywords: &[&str]) -> String {
        let mut result = sql.to_string();

        for keyword in keywords {
            let replacement = if upper {
                keyword.to_string()
            } else {
                keyword.to_lowercase()
            };

            result = result.replace(&keyword.to_lowercase(), &replacement);
            result = result.replace(*keyword, &replacement);
        }

        result
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
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
            }
        }
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }
}

impl Render for SqlFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                // 缩进 → Select（匹配 Tauri n-select）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().w(px(100.0)).child("缩进"))
                        .child(Select::new(&self.indent_state)),
                )
                // 关键字大写 → Switch（匹配 Tauri n-switch）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().w(px(100.0)).child("关键字大写"))
                        .child(
                            Switch::new("keyword-case")
                                .checked(self.keyword_case == "upper")
                                .on_click(cx.listener(|this, v: &bool, _, cx| {
                                    this.keyword_case = if *v {
                                        "upper".to_string()
                                    } else {
                                        "lower".to_string()
                                    };
                                    cx.notify();
                                })),
                        ),
                )
                // 操作 → Paste + Copy + 格式化（匹配 Tauri n-button-group，三个按钮在同一组内）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().w(px(100.0)).child(""))
                        .child(
                            ButtonGroup::new("action-buttons")
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
                                )
                                .child(Button::new("format").child("格式化").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.format_sql(window, cx);
                                    }),
                                )),
                        ),
                )
                // 单编辑器（匹配 Tauri CodeMirror，格式化后替换内容）
                .child(Input::new(&self.input_state).h(px(400.0))),
        )
    }
}

use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

pub struct SqlFormatter {
    sql: String,
    indent: usize,
    keyword_case: String,
}

impl SqlFormatter {
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            indent: 2,
            keyword_case: "upper".to_string(),
        }
    }

    fn format_sql(&mut self) {
        if self.sql.trim().is_empty() {
            return;
        }

        let keywords = vec![
            "SELECT", "FROM", "WHERE", "AND", "OR", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "INSERT", "INTO", "VALUES",
            "UPDATE", "SET", "DELETE", "CREATE", "TABLE", "DROP", "ALTER", "ADD", "COLUMN",
            "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "UNIQUE", "INDEX", "AS", "DISTINCT",
            "UNION", "ALL", "EXISTS", "IN", "BETWEEN", "LIKE", "IS", "NULL", "NOT", "ASC", "DESC",
            "CASE", "WHEN", "THEN", "ELSE", "END", "WITH", "RECURSIVE",
        ];

        let indent_str = " ".repeat(self.indent);
        let mut formatted = String::new();
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut depth = 0;

        for line in self.sql.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut tokens = self.tokenize_sql(trimmed);
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
                } else if upper_token == "LEFT" || upper_token == "RIGHT" || upper_token == "INNER" || upper_token == "OUTER" {
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
                lines.push(current_line.clone());
                current_line.clear();
            }
        }

        formatted = lines.join("\n");

        if self.keyword_case == "upper" {
            formatted = self.apply_keyword_case(&formatted, true);
        } else {
            formatted = self.apply_keyword_case(&formatted, false);
        }

        self.sql = formatted;
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

    fn apply_keyword_case(&self, sql: &str, upper: bool) -> String {
        let keywords = vec![
            "SELECT", "FROM", "WHERE", "AND", "OR", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "INSERT", "INTO", "VALUES",
            "UPDATE", "SET", "DELETE", "CREATE", "TABLE", "DROP", "ALTER", "ADD", "COLUMN",
            "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "UNIQUE", "INDEX", "AS", "DISTINCT",
            "UNION", "ALL", "EXISTS", "IN", "BETWEEN", "LIKE", "IS", "NULL", "NOT", "ASC", "DESC",
            "CASE", "WHEN", "THEN", "ELSE", "END", "WITH", "RECURSIVE",
        ];

        let mut result = sql.to_string();
        
        for keyword in &keywords {
            let pattern = keyword.to_string();
            let replacement = if upper {
                keyword.to_string()
            } else {
                keyword.to_lowercase()
            };
            
            result = result.replace(&pattern, &replacement);
            result = result.replace(&pattern.to_lowercase(), &replacement);
        }

        result
    }

    fn clear(&mut self) {
        self.sql.clear();
    }
}

impl Render for SqlFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sql_text = if self.sql.is_empty() {
            "输入SQL语句...".to_string()
        } else {
            self.sql.clone()
        };
        
        let indent_text = format!("{} 空格", self.indent);
        let case_text = if self.keyword_case == "upper" { "大写" } else { "小写" };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("SQL格式化")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("缩进")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(indent_text)
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .child("关键字大小写")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(case_text)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("粘贴")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("复制")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("格式化")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("清空")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("SQL编辑器")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(sql_text)
                            )
                    )
            )
    }
}
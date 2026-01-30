use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

#[derive(Clone, Debug)]
pub struct DiffItem {
    table_name: String,
    diff_type: String,
    sql: String,
}

pub struct DatabaseDiff {
    source_db: String,
    target_db: String,
    results: Vec<DiffItem>,
}

impl DatabaseDiff {
    pub fn new() -> Self {
        Self {
            source_db: String::new(),
            target_db: String::new(),
            results: Vec::new(),
        }
    }

    fn compare(&mut self) {
        if self.source_db.trim().is_empty() || self.target_db.trim().is_empty() {
            return;
        }

        let mut results = Vec::new();

        let source_tables = vec![
            "users", "orders", "products", "customers", "categories",
        ];

        let target_tables = vec![
            "users", "orders", "products", "customers", "payments",
        ];

        for table in &source_tables {
            if !target_tables.contains(table) {
                results.push(DiffItem {
                    table_name: table.to_string(),
                    diff_type: "仅存在于源数据库".to_string(),
                    sql: format!("-- 制{} 仅存在于源数据库", table),
                });
            }
        }

        for table in &target_tables {
            if !source_tables.contains(table) {
                results.push(DiffItem {
                    table_name: table.to_string(),
                    diff_type: "仅存在于目标数据库".to_string(),
                    sql: format!("-- 表 {} 仅存在于目标数据库", table),
                });
            }
        }

        for table in &source_tables {
            if target_tables.contains(table) {
                results.push(DiffItem {
                    table_name: table.to_string(),
                    diff_type: "结构可能不同".to_string(),
                    sql: format!("-- 制{} 在两个数据库中都存在，但结构可能不同", table),
                });
            }
        }

        self.results = results;
    }

    fn clear(&mut self) {
        self.source_db.clear();
        self.target_db.clear();
        self.results.clear();
    }
}

impl Render for DatabaseDiff {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source_text = if self.source_db.is_empty() {
            "源数据库连接...".to_string()
        } else {
            self.source_db.clone()
        };
        
        let target_text = if self.target_db.is_empty() {
            "目标数据库连制..".to_string()
        } else {
            self.target_db.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("数据库差异对比")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .child("源数据库")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h(px(40.0))
                                            .max_h(px(40.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_md()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .font_family("monospace")
                                            .child(source_text)
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
                                            .child("目标数据库")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h(px(40.0))
                                            .max_h(px(40.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_md()
                                            .px_3()
                                            .py_2()
                                            .text_sm()
                                            .font_family("monospace")
                                            .child(target_text)
                                    )
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
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("比较")
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
                    .child(if !self.results.is_empty() {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .child("差异结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_4()
                                    .overflow_y_scrollbar()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(
                                                self.results.iter().map(|item| {
                                                    div()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_2()
                                                                        .child(
                                                                            div()
                                                                                .text_sm()
                                                                                .font_semibold()
                                                                                .child(item.table_name.clone())
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .text_xs()
                                                                                .text_color(cx.theme().muted_foreground)
                                                                                .child(item.diff_type.clone())
                                                                        )
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_family("monospace")
                                                                        .text_color(cx.theme().muted_foreground)
                                                                        .child(item.sql.clone())
                                                                )
                                                        )
                                                })
                                            )
                                    )
                            )
                    } else {
                        div()
                    })
            )
    }
}
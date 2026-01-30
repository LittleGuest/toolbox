use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

pub struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
    parent_id: Option<usize>,
    created_at: i64,
}

pub struct TodoList {
    new_todo_text: String,
    todos: Vec<TodoItem>,
    filter: String,
    editing_id: Option<usize>,
    editing_text: String,
    expanded_ids: Vec<usize>,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            new_todo_text: String::new(),
            todos: Vec::new(),
            filter: "all".to_string(),
            editing_id: None,
            editing_text: String::new(),
            expanded_ids: Vec::new(),
        }
    }

    fn filtered_todos(&self) -> Vec<&TodoItem> {
        let result: Vec<&TodoItem> = match self.filter.as_str() {
            "active" => self.todos.iter().filter(|t| !t.completed).collect(),
            "completed" => self.todos.iter().filter(|t| t.completed).collect(),
            _ => self.todos.iter().collect(),
        };
        
        result
    }

    fn top_todos(&self) -> Vec<&TodoItem> {
        self.filtered_todos()
            .into_iter()
            .filter(|t| t.parent_id.is_none())
            .collect()
    }

    fn sub_todos(&self, parent_id: usize) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|t| t.parent_id == Some(parent_id))
            .collect()
    }

    fn is_expanded(&self, id: usize) -> bool {
        self.expanded_ids.contains(&id)
    }

    fn toggle_expand(&mut self, id: usize) {
        if let Some(pos) = self.expanded_ids.iter().position(|&x| x == id) {
            self.expanded_ids.remove(pos);
        } else {
            self.expanded_ids.push(id);
        }
    }

    fn has_sub_todos(&self, id: usize) -> bool {
        self.todos.iter().any(|t| t.parent_id == Some(id))
    }

    fn add_todo(&mut self) {
        if self.new_todo_text.trim().is_empty() {
            return;
        }
        
        let new_todo = TodoItem {
            id: self.todos.len(),
            text: self.new_todo_text.trim().to_string(),
            completed: false,
            parent_id: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        
        self.todos.insert(0, new_todo);
        self.new_todo_text.clear();
    }

    fn toggle_complete(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }

    fn delete_todo(&mut self, id: usize) {
        self.todos.retain(|t| t.id != id);
    }

    fn clear_completed(&mut self) {
        self.todos.retain(|t| !t.completed);
    }

    fn completed_count(&self) -> usize {
        self.todos.iter().filter(|t| t.completed).count()
    }

    fn start_edit(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter().find(|t| t.id == id) {
            self.editing_id = Some(id);
            self.editing_text = todo.text.clone();
        }
    }

    fn save_edit(&mut self) {
        if let Some(id) = self.editing_id {
            if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                todo.text = self.editing_text.clone();
            }
        }
        self.editing_id = None;
        self.editing_text.clear();
    }

    fn cancel_edit(&mut self) {
        self.editing_id = None;
        self.editing_text.clear();
    }
}

impl Render for TodoList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let top_todos = self.top_todos();
        let new_todo_text = if self.new_todo_text.is_empty() {
            "输入新的待办事项...".to_string()
        } else {
            self.new_todo_text.clone()
        };
        
        let filter_text = match self.filter.as_str() {
            "all" => "全部",
            "active" => "未完成",
            "completed" => "已完成",
            _ => "全部",
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("待办事项")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .mb_4()
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
                            .child(new_todo_text)
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(cx.theme().primary)
                            .text_color(cx.theme().primary_foreground)
                            .rounded_md()
                            .cursor_pointer()
                            .child("添加")
                    )
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .border_1()
                                    .border_color(if self.filter == "all" {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border
                                    })
                                    .rounded_md()
                                    .text_sm()
                                    .cursor_pointer()
                                    .child("全部")
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .border_1()
                                    .border_color(if self.filter == "active" {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border
                                    })
                                    .rounded_md()
                                    .text_sm()
                                    .cursor_pointer()
                                    .child("未完成")
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .border_1()
                                    .border_color(if self.filter == "completed" {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border
                                    })
                                    .rounded_md()
                                    .text_sm()
                                    .cursor_pointer()
                                    .child("已完成")
                            )
                    )
                    .child(if self.completed_count() > 0 {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .cursor_pointer()
                            .child(format!("清除已完制({})", self.completed_count()))
                    } else {
                        div()
                    })
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(400.0))
                    .max_h(px(400.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .p_4()
                    .overflow_y_scrollbar()
                    .child(if top_todos.is_empty() {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(match self.filter.as_str() {
                                "active" => "暂无未完成的待办事项",
                                "completed" => "暂无已完成的待办事项",
                                _ => "暂无待办事项",
                            })
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .children(
                                top_todos.iter().map(|todo| {
                                    let id = todo.id;
                                    let text = todo.text.clone();
                                    let completed = todo.completed;
                                    let has_sub = self.has_sub_todos(id);
                                    let is_expanded = self.is_expanded(id);
                                    let is_editing = self.editing_id == Some(id);
                                    
                                    div()
                                        .p_3()
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .rounded_md()
                                        .child(
                                            div()
                                                .flex()
                                                .items_start()
                                                .gap_2()
                                                .child(if has_sub {
                                                    div()
                                                        .w(px(24.0))
                                                        .h(px(24.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .cursor_pointer()
                                                        .child(if is_expanded { "▼" } else { "▶" })
                                                } else {
                                                    div()
                                                        .w(px(24.0))
                                                        .h(px(24.0))
                                                })
                                                .child(
                                                    div()
                                                        .w(px(16.0))
                                                        .h(px(16.0))
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_sm()
                                                        .child(if completed { "✓" } else { "" })
                                                )
                                                .child(if is_editing {
                                                    div()
                                                        .flex_1()
                                                        .min_h(px(40.0))
                                                        .max_h(px(40.0))
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .px_2()
                                                        .py_1()
                                                        .text_sm()
                                                        .font_family("monospace")
                                                        .child(self.editing_text.clone())
                                                } else {
                                                    div()
                                                        .flex_1()
                                                        .text_sm()
                                                        .child(if completed {
                                                            format!("~~{}~~", text)
                                                        } else {
                                                            text.clone()
                                                        })
                                                })
                                        )
                                })
                            )
                    })
            )
    }
}

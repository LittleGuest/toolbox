use std::collections::HashMap;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

pub struct TodoItem {
    id: usize,
    text: SharedString,
    completed: bool,
    parent_id: Option<usize>,
    created_at: i64,
}

pub struct TodoList {
    new_todo_text: SharedString,
    todos: Vec<TodoItem>,
    filter: SharedString,
    editing_id: Option<usize>,
    editing_text: SharedString,
    expanded_ids: Vec<usize>,
    input_state: Option<Entity<InputState>>,
    editing_input_states: HashMap<usize, Entity<InputState>>,
    next_id: usize,
    current_parent_id: Option<usize>,
    _subscriptions: Vec<Subscription>,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            new_todo_text: SharedString::default(),
            todos: Vec::new(),
            filter: SharedString::from("all"),
            editing_id: None,
            editing_text: SharedString::default(),
            expanded_ids: Vec::new(),
            input_state: None,
            editing_input_states: HashMap::new(),
            next_id: 0,
            current_parent_id: None,
            _subscriptions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入新的待办事项..."));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _window, cx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(cx).value();
                    this.new_todo_text = value.clone();
                    cx.notify()
                }
                _ => {}
            }
        })];

        self.input_state = Some(input_state);
        self._subscriptions = _subscriptions;
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

        let text = SharedString::from(self.new_todo_text.trim().to_string());

        let has_duplicate = self
            .todos
            .iter()
            .any(|t| t.text == text && t.parent_id == self.current_parent_id);

        if has_duplicate {
            return;
        }

        let current_depth = if let Some(parent_id) = self.current_parent_id {
            self.get_todo_depth(parent_id)
        } else {
            0
        };

        if current_depth >= 2 {
            return;
        }

        let new_todo = TodoItem {
            id: self.next_id,
            text: text.clone(),
            completed: false,
            parent_id: self.current_parent_id,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        self.next_id += 1;
        self.todos.insert(0, new_todo);
        self.new_todo_text = SharedString::default();
    }

    fn get_todo_depth(&self, id: usize) -> usize {
        let mut depth = 0;
        let mut current_id = id;

        while let Some(parent_id) = self
            .todos
            .iter()
            .find(|t| t.id == current_id)
            .and_then(|t| t.parent_id)
        {
            depth += 1;
            current_id = parent_id;
        }

        depth
    }

    fn add_sub_todo(&mut self, parent_id: usize) {
        if self.new_todo_text.trim().is_empty() {
            return;
        }

        let text = SharedString::from(self.new_todo_text.trim().to_string());

        let new_todo = TodoItem {
            id: self.next_id,
            text: text.clone(),
            completed: false,
            parent_id: Some(parent_id),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        self.next_id += 1;
        self.todos.insert(0, new_todo);
        self.new_todo_text = SharedString::default();
    }

    fn set_parent_id(&mut self, parent_id: usize, cx: &mut Context<Self>) {
        let depth = self.get_todo_depth(parent_id);
        if depth >= 2 {
            return;
        }
        self.current_parent_id = Some(parent_id);
        cx.notify();
    }

    fn clear_parent_id(&mut self) {
        self.current_parent_id = None;
    }

    fn toggle_complete(&mut self, id: usize) {
        let completed = if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
            todo.completed
        } else {
            return;
        };

        self.toggle_complete_recursive(id, completed);
    }

    fn toggle_complete_recursive(&mut self, parent_id: usize, completed: bool) {
        let mut stack = vec![parent_id];

        while let Some(current_id) = stack.pop() {
            for todo in self.todos.iter_mut() {
                if todo.parent_id == Some(current_id) {
                    todo.completed = completed;
                    stack.push(todo.id);
                }
            }
        }
    }

    fn delete_todo(&mut self, id: usize) {
        let mut ids_to_delete = vec![id];

        let mut index = 0;
        while index < ids_to_delete.len() {
            let current_id = ids_to_delete[index];
            for todo in self.todos.iter() {
                if todo.parent_id == Some(current_id) {
                    ids_to_delete.push(todo.id);
                }
            }
            index += 1;
        }

        self.todos.retain(|t| !ids_to_delete.contains(&t.id));
    }

    fn clear_completed(&mut self) {
        self.todos.retain(|t| !t.completed);
    }

    fn completed_count(&self) -> usize {
        self.todos.iter().filter(|t| t.completed).count()
    }

    fn start_edit(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(todo) = self.todos.iter().find(|t| t.id == id) {
            self.editing_id = Some(id);
            self.editing_text = todo.text.clone();

            if !self.editing_input_states.contains_key(&id) {
                let editing_input_state =
                    cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办事项..."));
                let editing_input_state_clone = editing_input_state.clone();

                let _ = cx.subscribe_in(&editing_input_state, window, {
                    move |this, _, ev: &InputEvent, _window, cx| match ev {
                        InputEvent::Change => {
                            let value = editing_input_state_clone.read(cx).value();
                            this.editing_text = value.clone();
                            cx.notify()
                        }
                        _ => {}
                    }
                });

                self.editing_input_states.insert(id, editing_input_state);
            }

            if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                editing_input_state.update(cx, |input_state, cx| {
                    input_state.replace(todo.text.clone(), window, cx);
                });
            }
        }
    }

    fn save_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.editing_id {
            if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                let value = editing_input_state.read(cx).value();
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.text = value.clone();
                }
            }
        }
        self.editing_id = None;
        self.editing_text = SharedString::default();
    }

    fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.editing_text = SharedString::default();
    }

    fn render_todo_item(
        &self,
        todo: &TodoItem,
        depth: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
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
                    .child(
                        Button::new(("expand", id))
                            .icon(if is_expanded {
                                Icon::new(IconName::ChevronDown)
                            } else {
                                Icon::new(IconName::ChevronUp)
                            })
                            .tooltip(if is_expanded { "折叠" } else { "展开" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_expand(id);
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new(("check", id))
                            .checked(completed)
                            .on_click(cx.listener(move |this, checked, _, cx| {
                                this.toggle_complete(id);
                                cx.notify();
                            })),
                    )
                    .child(if is_editing {
                        if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                            div()
                                .flex()
                                .flex_1()
                                .gap_2()
                                .child(Input::new(editing_input_state))
                                .child(
                                    ButtonGroup::new(("edit-actions", id))
                                        .child(
                                            Button::new(("save-edit", id))
                                                .icon(Icon::new(IconName::Check))
                                                .tooltip("保存")
                                                .on_click(cx.listener(
                                                    move |this, _, window, cx| {
                                                        this.save_edit(window, cx);
                                                        cx.notify();
                                                    },
                                                )),
                                        )
                                        .child(
                                            Button::new(("cancel-edit", id))
                                                .icon(Icon::new(IconName::Close))
                                                .tooltip("取消")
                                                .on_click(cx.listener(
                                                    move |this, _, window, cx| {
                                                        this.cancel_edit(window, cx);
                                                        cx.notify();
                                                    },
                                                )),
                                        ),
                                )
                        } else {
                            div()
                        }
                    } else {
                        div().flex_1().text_sm().child(if completed {
                            SharedString::from(format!("~~{}~~", text))
                        } else {
                            text
                        })
                    })
                    .when(depth < 2, |div| {
                        div.child(
                            Button::new(("add-sub", id))
                                .icon(Icon::new(IconName::Plus))
                                .tooltip("添加子项目")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.set_parent_id(id, cx);
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        ButtonGroup::new(("action-buttons", id))
                            .child(
                                Button::new(("edit", id))
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("编辑")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.start_edit(id, window, cx);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new(("delete", id))
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("删除")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.delete_todo(id);
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when(is_expanded && has_sub && depth < 2, |this_div| {
                let sub_todos = self.sub_todos(id);
                this_div.child(
                    div().flex().flex_col().gap_2().ml_6().mt_2().children(
                        sub_todos
                            .iter()
                            .map(|sub_todo| self.render_todo_item(sub_todo, depth + 1, window, cx)),
                    ),
                )
            })
    }
}

impl Render for TodoList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 确保 input_state 已初始化
        if self.input_state.is_none() {
            self.initialize(window, cx);
        }

        let top_todos = self.top_todos();

        let filter_text = match self.filter.as_str() {
            "all" => "全部",
            "active" => "未完成",
            "completed" => "已完成",
            _ => "全部",
        };

        let input_state = self.input_state.as_ref().unwrap();

        div()
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .mb_4()
                    .child(Input::new(input_state))
                    .child(
                        Button::new("add")
                            .primary()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip(if self.current_parent_id.is_some() {
                                "添加子项目"
                            } else {
                                "添加"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.add_todo();
                                cx.notify();
                            })),
                    )
                    .children(if self.current_parent_id.is_some() {
                        vec![
                            Button::new("cancel-add-sub")
                                .icon(Icon::new(IconName::Close))
                                .tooltip("取消")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clear_parent_id();
                                    cx.notify();
                                })),
                        ]
                    } else {
                        vec![]
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(
                        ButtonGroup::new("filter-buttons")
                            .child(
                                Button::new("filter-all")
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip(SharedString::from("全部"))
                                    .selected(self.filter.as_str() == "all")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.filter = SharedString::from("all");
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("filter-active")
                                    .icon(Icon::new(IconName::CircleX))
                                    .tooltip(SharedString::from("未完成"))
                                    .selected(self.filter.as_str() == "active")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.filter = SharedString::from("active");
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("filter-completed")
                                    .icon(Icon::new(IconName::CircleCheck))
                                    .tooltip(SharedString::from("已完成"))
                                    .selected(self.filter.as_str() == "completed")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.filter = SharedString::from("completed");
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        Button::new("clear-completed")
                            .icon(Icon::new(IconName::Delete))
                            .tooltip("清除已完成")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_completed();
                                cx.notify();
                            })),
                    ),
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
                        div().flex().flex_col().gap_2().children(
                            top_todos
                                .iter()
                                .map(|todo| self.render_todo_item(todo, 0, window, cx)),
                        )
                    }),
            )
    }
}

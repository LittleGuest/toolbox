use std::collections::HashMap;

use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState},
        notification::Notification,
        scroll::ScrollableElement,
        *,
    },
    prelude::FluentBuilder,
    *,
};

use crate::config_store::{self, TodoRecord};

const FILTER_OPTIONS: [(&str, &str); 3] = [
    ("all", "全部"),
    ("active", "未完成"),
    ("completed", "已完成"),
];

pub struct TodoItem {
    id: Option<i64>,
    text: SharedString,
    completed: bool,
    parent_id: Option<i64>,
    created_at: i64,
}

impl TodoItem {
    fn from_record(record: TodoRecord) -> Self {
        Self {
            id: record.id,
            text: SharedString::from(record.content),
            completed: record.completed,
            parent_id: record.parent_id,
            created_at: record.created_at,
        }
    }

    fn to_record(&self) -> TodoRecord {
        TodoRecord {
            id: self.id,
            parent_id: self.parent_id,
            content: self.text.to_string(),
            completed: self.completed,
            created_at: self.created_at,
        }
    }
}

pub struct TodoList {
    new_todo_text: SharedString,
    todos: Vec<TodoItem>,
    filter: SharedString,
    editing_id: Option<i64>,
    editing_text: SharedString,
    expanded_ids: Vec<i64>,
    adding_sub_for: Option<i64>,
    input_state: Option<Entity<InputState>>,
    sub_input_states: HashMap<i64, Entity<InputState>>,
    editing_input_states: HashMap<i64, Entity<InputState>>,
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
            adding_sub_for: None,
            input_state: None,
            sub_input_states: HashMap::new(),
            editing_input_states: HashMap::new(),
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
                InputEvent::PressEnter { .. } => {
                    this.add_todo(_window, cx);
                    cx.notify()
                }
                _ => {}
            }
        })];

        self.input_state = Some(input_state);
        self._subscriptions = _subscriptions;

        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_todos().await;
            let _ = this.update(cx, |this, cx| {
                if let Ok(records) = result {
                    this.todos = records.into_iter().map(TodoItem::from_record).collect();
                    this.sort_todos();
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn sort_todos(&mut self) {
        self.todos.sort_by(|a, b| match (a.completed, b.completed) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => b.created_at.cmp(&a.created_at),
        });
    }

    fn completed_count(&self) -> usize {
        self.todos.iter().filter(|t| t.completed).count()
    }

    fn filtered_todos(&self) -> Vec<&TodoItem> {
        match self.filter.as_str() {
            "active" => self.todos.iter().filter(|t| !t.completed).collect(),
            "completed" => self.todos.iter().filter(|t| t.completed).collect(),
            _ => self.todos.iter().collect(),
        }
    }

    fn top_todos(&self) -> Vec<&TodoItem> {
        self.filtered_todos()
            .into_iter()
            .filter(|t| t.parent_id.is_none())
            .collect()
    }

    fn sub_todos(&self, parent_id: i64) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|t| t.parent_id == Some(parent_id))
            .collect()
    }

    fn is_expanded(&self, id: i64) -> bool {
        self.expanded_ids.contains(&id)
    }

    fn toggle_expand(&mut self, id: i64) {
        if let Some(pos) = self.expanded_ids.iter().position(|&x| x == id) {
            self.expanded_ids.remove(pos);
        } else {
            self.expanded_ids.push(id);
        }
    }

    fn has_sub_todos(&self, id: i64) -> bool {
        self.todos.iter().any(|t| t.parent_id == Some(id))
    }

    fn add_todo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.new_todo_text.trim().is_empty() {
            window.push_notification(Notification::warning("请输入待办事项内容"), cx);
            return;
        }

        let text = self.new_todo_text.trim().to_string();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let record = TodoRecord {
            id: None,
            parent_id: None,
            content: text.clone(),
            completed: false,
            created_at: now,
        };
        let content_clone = text;

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_todo(record).await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(new_id) => {
                        this.todos.insert(
                            0,
                            TodoItem {
                                id: Some(new_id),
                                text: SharedString::from(content_clone),
                                completed: false,
                                parent_id: None,
                                created_at: now,
                            },
                        );
                        this.sort_todos();
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("保存待办失败：{err}")),
                            cx,
                        );
                    }
                }
                this.new_todo_text = SharedString::default();
                cx.notify();
            });
        })
        .detach();

        self.new_todo_text = SharedString::default();
    }

    /// 开始为某个待办添加子任务（对齐 Vue startAddSubTodo）
    fn start_add_sub(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.adding_sub_for = Some(id);
        if !self.expanded_ids.contains(&id) {
            self.expanded_ids.push(id);
        }
        if !self.sub_input_states.contains_key(&id) {
            let sub_state =
                cx.new(|cx| InputState::new(window, cx).placeholder("输入子任务内容..."));
            let sub_state_clone = sub_state.clone();
            let _ = cx.subscribe_in(&sub_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let _ = sub_state_clone.read(cx).value();
                        cx.notify()
                    }
                    InputEvent::PressEnter { .. } => {
                        this.add_sub_todo(id, _window, cx);
                        cx.notify()
                    }
                    _ => {}
                }
            });
            self.sub_input_states.insert(id, sub_state);
        } else if let Some(state) = self.sub_input_states.get(&id) {
            state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
            });
        }
        cx.notify();
    }

    fn cancel_add_sub(&mut self, cx: &mut Context<Self>) {
        self.adding_sub_for = None;
        cx.notify();
    }

    /// 添加子任务（对齐 Vue addSubTodo，子任务不能再有子任务）
    fn add_sub_todo(&mut self, parent_id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let text = match self.sub_input_states.get(&parent_id) {
            Some(state) => state.read(cx).value().trim().to_string(),
            None => return,
        };
        if text.is_empty() {
            window.push_notification(Notification::warning("请输入子任务内容"), cx);
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let record = TodoRecord {
            id: None,
            parent_id: Some(parent_id),
            content: text.clone(),
            completed: false,
            created_at: now,
        };

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_todo(record).await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(new_id) => {
                        this.todos.insert(
                            0,
                            TodoItem {
                                id: Some(new_id),
                                text: SharedString::from(text),
                                completed: false,
                                parent_id: Some(parent_id),
                                created_at: now,
                            },
                        );
                        this.sort_todos();
                        window.push_notification(Notification::success("子任务已添加"), cx);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("保存子待办失败：{err}")),
                            cx,
                        );
                    }
                }
                if this.adding_sub_for == Some(parent_id) {
                    this.adding_sub_for = None;
                }
                cx.notify();
            });
        })
        .detach();

        if let Some(state) = self.sub_input_states.get(&parent_id) {
            state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
            });
        }
        self.adding_sub_for = None;
    }

    fn toggle_complete(&mut self, id: i64, cx: &mut Context<Self>) {
        let completed = if let Some(todo) = self.todos.iter_mut().find(|t| t.id == Some(id)) {
            todo.completed = !todo.completed;
            todo.completed
        } else {
            return;
        };

        self.toggle_complete_recursive_down(id, completed);

        if completed {
            self.propagate_complete_upward(id);
        } else {
            self.propagate_uncomplete_upward(id);
        }

        let updates: Vec<(i64, TodoRecord)> = self
            .todos
            .iter()
            .filter(|t| {
                t.id == Some(id)
                    || t.parent_id == Some(id)
                    || self.is_ancestor(t.id.unwrap_or(0), id)
            })
            .map(|t| (t.id.unwrap_or(0), t.to_record()))
            .collect();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            for (update_id, record) in updates {
                let _ = config_store::update_todo(update_id, record).await;
            }
            let _ = this.update(cx, |this, cx| {
                this.sort_todos();
                cx.notify();
            });
        })
        .detach();

        self.sort_todos();
    }

    fn toggle_complete_recursive_down(&mut self, parent_id: i64, completed: bool) {
        let mut stack = vec![parent_id];

        while let Some(current_id) = stack.pop() {
            for todo in self.todos.iter_mut() {
                if todo.parent_id == Some(current_id) {
                    todo.completed = completed;
                    stack.push(todo.id.unwrap_or(0));
                }
            }
        }
    }

    fn propagate_complete_upward(&mut self, id: i64) {
        let mut current_id = id;
        while let Some(parent_id) = self
            .todos
            .iter()
            .find(|t| t.id == Some(current_id))
            .and_then(|t| t.parent_id)
        {
            let all_siblings_completed = self
                .todos
                .iter()
                .filter(|t| t.parent_id == Some(parent_id))
                .all(|t| t.completed);

            if all_siblings_completed {
                if let Some(parent) = self.todos.iter_mut().find(|t| t.id == Some(parent_id)) {
                    parent.completed = true;
                    current_id = parent_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn propagate_uncomplete_upward(&mut self, id: i64) {
        let mut current_id = id;
        while let Some(parent_id) = self
            .todos
            .iter()
            .find(|t| t.id == Some(current_id))
            .and_then(|t| t.parent_id)
        {
            if let Some(parent) = self.todos.iter_mut().find(|t| t.id == Some(parent_id)) {
                parent.completed = false;
                current_id = parent_id;
            } else {
                break;
            }
        }
    }

    fn is_ancestor(&self, id: i64, ancestor_id: i64) -> bool {
        let mut current_id = id;
        while let Some(parent_id) = self
            .todos
            .iter()
            .find(|t| t.id == Some(current_id))
            .and_then(|t| t.parent_id)
        {
            if parent_id == ancestor_id {
                return true;
            }
            current_id = parent_id;
        }
        false
    }

    fn delete_todo(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let this = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            let this = this.clone();
            alert
                .title(div().text_lg().font_semibold().child("确认删除"))
                .width(px(420.))
                .description(
                    div()
                        .py_4()
                        .text_sm()
                        .child("是否确认删除？将同时删除其所有子任务。"),
                )
                .confirm()
                .on_ok(move |_, window, cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.confirm_delete_todo(id, window, cx);
                        });
                    }
                    true
                })
        });
    }

    fn confirm_delete_todo(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_todo(id).await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(true) => {
                        let mut ids_to_delete = vec![id];
                        let mut index = 0;
                        while index < ids_to_delete.len() {
                            let current_id = ids_to_delete[index];
                            for todo in this.todos.iter() {
                                if todo.parent_id == Some(current_id) {
                                    if let Some(todo_id) = todo.id {
                                        ids_to_delete.push(todo_id);
                                    }
                                }
                            }
                            index += 1;
                        }
                        this.todos
                            .retain(|t| t.id.map_or(true, |tid| !ids_to_delete.contains(&tid)));
                        window.push_notification(Notification::success("待办事项已删除"), cx);
                    }
                    Ok(false) => {
                        window.push_notification(Notification::warning("未找到要删除的待办"), cx);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("删除待办失败：{err}")),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn clear_completed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let top_completed: Vec<i64> = self
            .todos
            .iter()
            .filter(|t| t.parent_id.is_none() && t.completed)
            .filter_map(|t| t.id)
            .collect();

        if top_completed.is_empty() {
            return;
        }

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let mut deleted = 0;
            for id in &top_completed {
                if config_store::delete_todo(*id).await.map_or(false, |ok| ok) {
                    deleted += 1;
                }
            }
            let _ = this.update_in(cx, |this, window, cx| {
                let mut ids_to_delete = top_completed.clone();
                let mut index = 0;
                while index < ids_to_delete.len() {
                    let current_id = ids_to_delete[index];
                    for todo in this.todos.iter() {
                        if todo.parent_id == Some(current_id) {
                            if let Some(todo_id) = todo.id {
                                ids_to_delete.push(todo_id);
                            }
                        }
                    }
                    index += 1;
                }
                this.todos
                    .retain(|t| t.id.map_or(true, |tid| !ids_to_delete.contains(&tid)));
                if deleted == top_completed.len() {
                    window.push_notification(Notification::success("已清除所有已完成的待办事项"), cx);
                } else {
                    window.push_notification(Notification::error("清除待办失败"), cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn start_edit(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.adding_sub_for = None;
        if let Some(todo) = self.todos.iter().find(|t| t.id == Some(id)) {
            self.editing_id = Some(id);
            self.editing_text = todo.text.clone();

            if !self.editing_input_states.contains_key(&id) {
                let editing_input_state =
                    cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办事项..."));
                let editing_input_state_clone = editing_input_state.clone();

                let _ = cx.subscribe_in(&editing_input_state, window, {
                    move |this, _, ev: &InputEvent, window, cx| match ev {
                        InputEvent::Change => {
                            let value = editing_input_state_clone.read(cx).value();
                            this.editing_text = value.clone();
                            cx.notify()
                        }
                        InputEvent::PressEnter { .. } => {
                            this.save_edit(window, cx);
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
        cx.notify();
    }

    fn save_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.editing_id {
            if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                let value = editing_input_state.read(cx).value().to_string();
                if value.trim().is_empty() {
                    window.push_notification(Notification::warning("待办事项内容不能为空"), cx);
                    return;
                }
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == Some(id)) {
                    todo.text = SharedString::from(value.clone());
                    let record = todo.to_record();
                    let id_for_update = id;

                    cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
                        let _ = config_store::update_todo(id_for_update, record).await;
                        let _ = this.update_in(cx, |this, window, cx| {
                            window.push_notification(Notification::success("待办事项已更新"), cx);
                            cx.notify();
                        });
                    })
                    .detach();
                }
            }
        }
        self.editing_id = None;
        self.editing_text = SharedString::default();
        let _ = window;
    }

    fn cancel_edit(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.editing_text = SharedString::default();
        cx.notify();
    }

    /// 过滤分段选择器（对应 n-radio-button 组：全部/未完成/已完成）
    fn filter_segmented(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(
                FILTER_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, (value, label))| {
                        let active = self.filter.as_str() == *value;
                        div()
                            .id(("filter", i))
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
                                this.filter = SharedString::from(*value);
                                cx.notify();
                            }))
                            .child(label.to_string())
                    }),
            )
    }

    fn render_todo_item(
        &self,
        todo: &TodoItem,
        depth: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let id = todo.id.unwrap_or(0);
        let id_usize = id as usize;
        let text = todo.text.clone();
        let completed = todo.completed;
        let has_sub = depth == 0 && self.has_sub_todos(id);
        let is_expanded = self.is_expanded(id);
        let is_editing = self.editing_id == Some(id);
        let is_adding_sub = self.adding_sub_for == Some(id);

        // 行主体（todo-item-content）
        let mut content = div().flex().items_center().w_full().gap_2();

        if depth == 0 {
            // 展开/收起按钮或占位（expand-btn / expand-placeholder）
            if has_sub {
                content = content.child(
                    div().w(px(24.0)).flex_shrink_0().child(
                        Button::new(("expand", id_usize))
                            .ghost()
                            .compact()
                            .icon(if is_expanded {
                                Icon::new(IconName::ChevronDown)
                            } else {
                                Icon::new(IconName::ArrowRight)
                            })
                            .tooltip(if is_expanded { "收起" } else { "展开" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_expand(id);
                                cx.notify();
                            })),
                    ),
                );
            } else {
                content = content.child(div().w(px(24.0)).h(px(24.0)).flex_shrink_0());
            }
        } else {
            // 子任务缩进（sub-todo-indent：虚线左边框）
            content = content.child(
                div()
                    .w(px(20.0))
                    .h(px(20.0))
                    .border_l_2()
                    .border_dashed()
                    .border_color(cx.theme().border)
                    .mr_2()
                    .flex_shrink_0(),
            );
        }

        content = content.child(
            Checkbox::new(("check", id_usize))
                .checked(completed)
                .on_click(cx.listener(move |this, _checked: &bool, _, cx| {
                    this.toggle_complete(id, cx);
                    cx.notify();
                })),
        );

        if is_editing {
            // 编辑模式
            let mut edit_row = div()
                .flex()
                .items_center()
                .flex_1()
                .min_w_0()
                .ml_3()
                .gap_2();
            if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                edit_row = edit_row.child(div().flex_1().child(Input::new(editing_input_state)));
            }
            edit_row = edit_row.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_shrink_0()
                    .child(
                        Button::new(("save-edit", id_usize))
                            .primary()
                            .compact()
                            .icon(Icon::new(IconName::Check))
                            .tooltip("保存")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.save_edit(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("cancel-edit", id_usize))
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("取消")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.cancel_edit(window, cx);
                            })),
                    ),
            );
            content = content.child(edit_row);
        } else {
            // 展示模式（display-mode）
            let mut actions = div().flex().items_center().gap(px(2.0)).flex_shrink_0();
            if depth == 0 {
                actions = actions.child(
                    Button::new(("add-sub", id_usize))
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::Plus))
                        .tooltip("添加子任务")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start_add_sub(id, window, cx);
                        })),
                );
            }
            actions = actions
                .child(
                    Button::new(("edit", id_usize))
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::File))
                        .tooltip("编辑")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start_edit(id, window, cx);
                        })),
                )
                .child(
                    Button::new(("delete", id_usize))
                        .custom(
                            ButtonCustomVariant::new(cx)
                                .foreground(Hsla::from(rgb(design::ERROR_RED))),
                        )
                        .compact()
                        .icon(Icon::new(IconName::CircleX))
                        .tooltip("删除")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.delete_todo(id, window, cx);
                        })),
                );

            content = content
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .ml_3()
                        .text_sm()
                        .when(completed, |t| {
                            t.line_through()
                                .text_color(cx.theme().muted_foreground.opacity(0.9))
                        })
                        .child(text),
                )
                .child(actions);
        }

        let mut item = div()
            .flex_col()
            .items_start()
            .w_full()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(content);

        // 行内添加子任务（add-sub-todo-container）
        if is_adding_sub && depth == 0 {
            let mut add_row = div()
                .flex()
                .items_center()
                .mt_2()
                .ml(px(60.0))
                .w_full()
                .gap_2();
            if let Some(sub_state) = self.sub_input_states.get(&id) {
                add_row = add_row.child(div().flex_1().child(Input::new(sub_state)));
            }
            add_row = add_row.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_shrink_0()
                    .child(
                        Button::new(("confirm-add-sub", id_usize))
                            .primary()
                            .compact()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("添加")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.add_sub_todo(id, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("cancel-add-sub", id_usize))
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("取消")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel_add_sub(cx);
                            })),
                    ),
            );
            item = item.child(add_row);
        }

        // 子任务列表（sub-todos-container）
        if depth == 0 && is_expanded && has_sub {
            let sub_todos = self.sub_todos(id);
            item = item.child(
                div().w_full().mt_2().flex_col().children(
                    sub_todos
                        .iter()
                        .map(|sub_todo| self.render_todo_item(sub_todo, depth + 1, window, cx)),
                ),
            );
        }

        item
    }
}

impl Render for TodoList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.input_state.is_none() {
            self.initialize(window, cx);
        }

        let top_todos = self.top_todos();
        let completed_count = self.completed_count();
        let can_add = !self.new_todo_text.trim().is_empty();

        let input_state = self.input_state.as_ref().unwrap();

        design::page()
            .child(
                design::card(cx)
                    // 添加行（add-todo）
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2p5()
                            .child(div().flex_1().child(Input::new(input_state)))
                            .child(
                                Button::new("add")
                                    .primary()
                                    .icon(Icon::new(IconName::Plus))
                                    .tooltip("添加")
                                    .disabled(!can_add)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.add_todo(window, cx);
                                        cx.notify();
                                    })),
                            ),
                    )
                    // 过滤行（filter-container）
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(self.filter_segmented(cx))
                            .when(completed_count > 0, |row| {
                                row.child(
                                    Button::new("clear-completed")
                                        .custom(
                                            ButtonCustomVariant::new(cx)
                                                .foreground(Hsla::from(rgb(design::ERROR_RED))),
                                        )
                                        .compact()
                                        .icon(Icon::new(IconName::CircleX))
                                        .tooltip("清除已完成")
                                        .label(format!("清除已完成 ({})", completed_count))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear_completed(window, cx);
                                            cx.notify();
                                        })),
                                )
                            }),
                    ),
            )
            .child(
                design::card(cx)
                    .min_h(px(300.0))
                    .max_h(px(560.0))
                    .overflow_y_scrollbar()
                    .child(if top_todos.is_empty() {
                        design::hint(
                            match self.filter.as_str() {
                                "active" => "暂无未完成的待办事项",
                                "completed" => "暂无已完成的待办事项",
                                _ => "暂无待办事项",
                            },
                            cx,
                        )
                    } else {
                        div().flex().flex_col().children(
                            top_todos
                                .iter()
                                .map(|todo| self.render_todo_item(todo, 0, window, cx)),
                        )
                    }),
            )
    }
}

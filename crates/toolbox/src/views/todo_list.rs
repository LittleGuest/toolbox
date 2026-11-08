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
            cx.new(|cx| InputState::new(window, cx).placeholder("输入新的待办事项，回车即可添加…"));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(cx).value();
                    this.new_todo_text = value;
                    cx.notify()
                }
                InputEvent::PressEnter { .. } => {
                    this.add_todo(window, cx);
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

    fn progress(&self) -> (usize, usize) {
        let done = self.todos.iter().filter(|t| t.completed).count();
        (done, self.todos.len())
    }

    fn top_level_counts(&self) -> [usize; 3] {
        let all = self.todos.iter().filter(|t| t.parent_id.is_none()).count();
        let active = self
            .todos
            .iter()
            .filter(|t| t.parent_id.is_none() && !t.completed)
            .count();
        let completed = self
            .todos
            .iter()
            .filter(|t| t.parent_id.is_none() && t.completed)
            .count();
        [all, active, completed]
    }

    fn filtered_todos(&self) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|t| match self.filter.as_str() {
                "active" => !t.completed,
                "completed" => t.completed,
                _ => true,
            })
            .collect()
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
            .filter(|t| {
                t.parent_id == Some(parent_id)
                    && match self.filter.as_str() {
                        "active" => !t.completed,
                        "completed" => t.completed,
                        _ => true,
                    }
            })
            .collect()
    }

    fn sub_progress(&self, parent_id: i64) -> (usize, usize) {
        let total = self
            .todos
            .iter()
            .filter(|t| t.parent_id == Some(parent_id))
            .count();
        let done = self
            .todos
            .iter()
            .filter(|t| t.parent_id == Some(parent_id) && t.completed)
            .count();
        (done, total)
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

        if let Some(input_state) = self.input_state.as_ref() {
            input_state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
                state.focus(window, cx);
            });
        }
        self.new_todo_text = SharedString::default();

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
                cx.notify();
            });
        })
        .detach();
    }

    fn start_add_sub(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.adding_sub_for = Some(id);
        if !self.expanded_ids.contains(&id) {
            self.expanded_ids.push(id);
        }

        let state =
            if let std::collections::hash_map::Entry::Vacant(e) = self.sub_input_states.entry(id) {
                let state = cx
                    .new(|cx| InputState::new(window, cx).placeholder("输入子任务，回车即可添加…"));
                let _ = cx.subscribe_in(
                    &state,
                    window,
                    move |this, _, ev: &InputEvent, window, cx| match ev {
                        InputEvent::Change => cx.notify(),
                        InputEvent::PressEnter { .. } => {
                            this.add_sub_todo(id, window, cx);
                            cx.notify()
                        }
                        _ => {}
                    },
                );
                e.insert(state.clone());
                state
            } else {
                self.sub_input_states.get(&id).unwrap().clone()
            };

        state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
            state.focus(window, cx);
        });
        cx.notify();
    }

    fn cancel_add_sub(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.adding_sub_for
            && let Some(state) = self.sub_input_states.get(&id)
        {
            state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
            });
        }
        self.adding_sub_for = None;
        cx.notify();
    }

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

        if let Some(state) = self.sub_input_states.get(&parent_id) {
            state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
                state.focus(window, cx);
            });
        }

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
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("保存子待办失败：{err}")),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
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
                                if todo.parent_id == Some(current_id)
                                    && let Some(todo_id) = todo.id
                                {
                                    ids_to_delete.push(todo_id);
                                }
                            }
                            index += 1;
                        }
                        this.todos
                            .retain(|t| t.id.is_none_or(|tid| !ids_to_delete.contains(&tid)));
                        this.editing_id = this
                            .editing_id
                            .filter(|editing| !ids_to_delete.contains(editing));
                        this.adding_sub_for = this
                            .adding_sub_for
                            .filter(|parent| !ids_to_delete.contains(parent));
                        this.expanded_ids
                            .retain(|expanded| !ids_to_delete.contains(expanded));
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
        let completed_ids: Vec<i64> = self
            .todos
            .iter()
            .filter(|t| t.completed)
            .filter_map(|t| t.id)
            .collect();

        if completed_ids.is_empty() {
            return;
        }

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            for id in &completed_ids {
                let _ = config_store::delete_todo(*id).await;
            }
            let records = config_store::load_todos().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match records {
                    Ok(records) => {
                        this.todos = records.into_iter().map(TodoItem::from_record).collect();
                        this.sort_todos();
                        window.push_notification(
                            Notification::success(format!(
                                "已清除 {} 项已完成的待办",
                                completed_ids.len()
                            )),
                            cx,
                        );
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("刷新待办列表失败：{err}")),
                            cx,
                        );
                    }
                }
                this.editing_id = None;
                this.adding_sub_for = None;
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

            self.editing_input_states.entry(id).or_insert_with(|| {
                let editing_input_state =
                    cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办事项，回车保存…"));
                let editing_input_state_clone = editing_input_state.clone();

                let _ = cx.subscribe_in(&editing_input_state, window, {
                    move |this, _, ev: &InputEvent, window, cx| match ev {
                        InputEvent::Change => {
                            let value = editing_input_state_clone.read(cx).value();
                            this.editing_text = value;
                            cx.notify()
                        }
                        InputEvent::PressEnter { .. } => {
                            this.save_edit(window, cx);
                            cx.notify()
                        }
                        _ => {}
                    }
                });

                editing_input_state
            });

            if let Some(editing_input_state) = self.editing_input_states.get(&id) {
                editing_input_state.update(cx, |input_state, cx| {
                    input_state.replace(todo.text.clone(), window, cx);
                    input_state.focus(window, cx);
                });
            }
        }
        cx.notify();
    }

    fn save_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.editing_id
            && let Some(editing_input_state) = self.editing_input_states.get(&id)
        {
            let value = editing_input_state.read(cx).value().to_string();
            if value.trim().is_empty() {
                window.push_notification(Notification::warning("待办事项内容不能为空"), cx);
                return;
            }
            if let Some(todo) = self.todos.iter_mut().find(|t| t.id == Some(id)) {
                todo.text = SharedString::from(value.clone());
                let record = todo.to_record();

                cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
                    let _ = config_store::update_todo(id, record).await;
                    let _ = this.update_in(cx, |_this, window, cx| {
                        window.push_notification(Notification::success("待办事项已更新"), cx);
                        cx.notify();
                    });
                })
                .detach();
            }
        }
        self.editing_id = None;
        self.editing_text = SharedString::default();
    }

    fn cancel_edit(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.editing_text = SharedString::default();
        cx.notify();
    }

    fn render_filter_tabs(&self, counts: [usize; 3], cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .children(
                FILTER_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, (value, label))| {
                        let active = self.filter.as_str() == *value;
                        div()
                            .id(("filter", i))
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(12.0))
                            .py(px(5.0))
                            .rounded_full()
                            .border_1()
                            .border_color(if active {
                                cx.theme().primary
                            } else {
                                cx.theme().border
                            })
                            .bg(if active {
                                cx.theme().primary
                            } else {
                                cx.theme().popover
                            })
                            .text_size(px(12.5))
                            .text_color(if active {
                                cx.theme().primary_foreground
                            } else {
                                cx.theme().muted_foreground
                            })
                            .cursor_pointer()
                            .when(!active, |tab| {
                                tab.hover(|s: StyleRefinement| s.bg(cx.theme().list_hover))
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.filter = SharedString::from(*value);
                                cx.notify();
                            }))
                            .child(*label)
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_semibold()
                                    .text_color(if active {
                                        cx.theme().primary_foreground.opacity(0.8)
                                    } else {
                                        cx.theme().muted_foreground
                                    })
                                    .child(counts[i].to_string()),
                            )
                    }),
            )
    }

    fn render_row_actions(
        &self,
        id: i64,
        depth: usize,
        group_id: SharedString,
        cx: &mut Context<Self>,
    ) -> Div {
        let id_usize = id as usize;

        let mut actions = div()
            .flex()
            .items_center()
            .gap(px(1.0))
            .flex_shrink_0()
            .opacity(0.45)
            .group_hover(group_id, |s: StyleRefinement| s.opacity(1.0));

        if depth == 0 {
            actions = actions.child(
                Button::new(("todo-add-sub", id_usize))
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Plus))
                    .tooltip("添加子任务")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_add_sub(id, window, cx);
                    })),
            );
        }

        actions
            .child(
                Button::new(("todo-edit", id_usize))
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::File))
                    .tooltip("编辑")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_edit(id, window, cx);
                    })),
            )
            .child(
                Button::new(("todo-delete", id_usize))
                    .custom(
                        ButtonCustomVariant::new(cx).foreground(Hsla::from(rgb(design::ERROR_RED))),
                    )
                    .compact()
                    .icon(Icon::new(IconName::Delete))
                    .tooltip("删除")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.delete_todo(id, window, cx);
                    })),
            )
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> Div {
        let (title, hint) = match self.filter.as_str() {
            "active" => ("全部待办都已完成", "干得漂亮，休息一下吧"),
            "completed" => ("还没有已完成的待办", "完成一项任务后会出现在这里"),
            _ => ("还没有待办事项", "在上方输入框写下第一件事，按回车即可添加"),
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2p5()
            .py(px(72.0))
            .child(
                div()
                    .w(px(54.0))
                    .h(px(54.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(cx.theme().primary.opacity(0.08))
                    .text_color(cx.theme().primary.opacity(0.75))
                    .child(Icon::new(IconName::CircleCheck).size(px(26.0))),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(px(14.0))
                    .font_medium()
                    .text_color(cx.theme().foreground)
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(hint),
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
        let is_adding_sub = depth == 0 && self.adding_sub_for == Some(id);
        let (sub_done, sub_total) = if has_sub {
            self.sub_progress(id)
        } else {
            (0, 0)
        };
        let group_id = SharedString::from(format!("todo-row-{id_usize}"));

        let mut row = div()
            .id(("todo-row", id_usize))
            .group(group_id.clone())
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py(px(7.0))
            .rounded(px(9.0))
            .hover(|s: StyleRefinement| s.bg(cx.theme().list_hover));

        if depth == 0 {
            if has_sub {
                row = row.child(
                    Button::new(("todo-expand", id_usize))
                        .ghost()
                        .compact()
                        .icon(Icon::new(if is_expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        }))
                        .tooltip(if is_expanded {
                            "收起子任务"
                        } else {
                            "展开子任务"
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_expand(id);
                            cx.notify();
                        })),
                );
            } else {
                row = row.child(div().w(px(20.0)).h(px(20.0)).flex_shrink_0());
            }
        }

        row = row.child(
            Checkbox::new(("todo-check", id_usize))
                .checked(completed)
                .tooltip(if completed {
                    "标记为未完成"
                } else {
                    "标记为已完成"
                })
                .on_click(cx.listener(move |this, _checked: &bool, _, cx| {
                    this.toggle_complete(id, cx);
                })),
        );

        if is_editing {
            let mut edit_area = div().flex().items_center().flex_1().min_w_0().gap_2();

            if let Some(state) = self.editing_input_states.get(&id) {
                edit_area = edit_area.child(div().flex_1().min_w_0().child(Input::new(state)));
            }

            edit_area = edit_area
                .child(
                    Button::new(("todo-save-edit", id_usize))
                        .primary()
                        .compact()
                        .icon(Icon::new(IconName::Check))
                        .tooltip("保存（回车）")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.save_edit(window, cx);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(("todo-cancel-edit", id_usize))
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::Close))
                        .tooltip("取消")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.cancel_edit(window, cx);
                        })),
                );

            row = row.child(edit_area);
        } else {
            let mut title_area = div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(3.0))
                .child(
                    div()
                        .text_size(px(if depth == 0 { 14.0 } else { 13.0 }))
                        .font_medium()
                        .truncate()
                        .text_color(if completed {
                            cx.theme().muted_foreground
                        } else {
                            cx.theme().foreground
                        })
                        .when(completed, |label| label.line_through())
                        .child(text),
                );

            if has_sub && sub_total > 0 {
                let sub_ratio = sub_done as f32 / sub_total as f32;
                title_area = title_area.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(40.0))
                                .h(px(3.0))
                                .rounded_full()
                                .bg(cx.theme().muted)
                                .overflow_hidden()
                                .child(
                                    div()
                                        .h_full()
                                        .w(relative(sub_ratio))
                                        .rounded_full()
                                        .bg(cx.theme().primary.opacity(0.7)),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{sub_done}/{sub_total} 子任务")),
                        ),
                );
            }

            row = row
                .child(title_area)
                .child(self.render_row_actions(id, depth, group_id, cx));
        }

        let mut item = div().flex_col().w_full().child(row);

        if depth == 0 && (is_adding_sub || (has_sub && is_expanded)) {
            let mut section = div()
                .flex_col()
                .ml(px(30.0))
                .pl_3()
                .pr_1()
                .pb_2()
                .border_l_1()
                .border_color(cx.theme().border);

            if is_adding_sub {
                let mut composer = div().flex().items_center().gap_2().py(px(3.0));

                if let Some(state) = self.sub_input_states.get(&id) {
                    composer = composer.child(div().flex_1().min_w_0().child(Input::new(state)));
                }

                composer = composer
                    .child(
                        Button::new(("todo-confirm-sub", id_usize))
                            .primary()
                            .compact()
                            .icon(Icon::new(IconName::Check))
                            .tooltip("添加子任务（回车）")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.add_sub_todo(id, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("todo-cancel-sub", id_usize))
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("取消")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_add_sub(window, cx);
                            })),
                    );

                section = section.child(composer);
            }

            if has_sub && is_expanded {
                let sub_todos = self.sub_todos(id);
                if !sub_todos.is_empty() {
                    section = section.child(
                        div().flex_col().gap(px(1.0)).children(
                            sub_todos
                                .iter()
                                .map(|sub| self.render_todo_item(sub, 1, window, cx)),
                        ),
                    );
                }
            }

            item = item.child(section);
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
        let (done, total) = self.progress();
        let pending = total.saturating_sub(done);
        let ratio = if total == 0 {
            0.0
        } else {
            done as f32 / total as f32
        };
        let counts = self.top_level_counts();
        let can_add = !self.new_todo_text.trim().is_empty();

        let input_state = self.input_state.as_ref().unwrap();

        let subtitle = if total == 0 {
            "把要做的事记下来，专注当下".to_string()
        } else if pending == 0 {
            "全部完成，干得漂亮".to_string()
        } else {
            format!("还有 {pending} 项未完成 · 已完成 {done} 项")
        };

        let header_row = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .w(px(42.0))
                    .h(px(42.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .rounded(px(12.0))
                    .bg(cx.theme().primary.opacity(0.1))
                    .text_color(cx.theme().primary)
                    .child(Icon::new(IconName::Inbox).size(px(22.0))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(16.5))
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("待办事项"),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(cx.theme().muted_foreground)
                            .child(subtitle),
                    ),
            )
            .when(done > 0, |header| {
                header.child(
                    Button::new("clear-completed")
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::RotateCw))
                        .label("清除已完成")
                        .tooltip("删除所有已完成的待办")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.clear_completed(window, cx);
                        })),
                )
            });

        let progress_bar = div()
            .flex_1()
            .h(px(6.0))
            .rounded_full()
            .bg(cx.theme().muted)
            .overflow_hidden();

        let progress_bar = if total > 0 {
            progress_bar.child(div().h_full().w(relative(ratio)).rounded_full().bg(
                if pending == 0 {
                    cx.theme().success
                } else {
                    cx.theme().primary
                },
            ))
        } else {
            progress_bar
        };

        let progress_row = div()
            .flex()
            .items_center()
            .gap_3()
            .child(progress_bar)
            .child(
                div()
                    .text_size(px(11.5))
                    .font_medium()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{:.0}%", ratio * 100.0)),
            );

        let composer_row = div()
            .flex()
            .items_center()
            .gap_2()
            .py(px(4.0))
            .child(div().flex_1().min_w_0().child(Input::new(input_state)))
            .child(
                Button::new("add-todo")
                    .primary()
                    .icon(Icon::new(IconName::Plus))
                    .label("添加")
                    .tooltip("添加待办（回车）")
                    .disabled(!can_add)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.add_todo(window, cx);
                        cx.notify();
                    })),
            );

        let count_label = div()
            .text_size(px(11.5))
            .text_color(cx.theme().muted_foreground)
            .child(format!("共 {} 项", counts[0]));

        let filters_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(self.render_filter_tabs(counts, cx))
            .child(count_label);

        let header_card = design::card(cx)
            .child(header_row)
            .child(progress_row)
            .child(composer_row)
            .child(filters_row);

        let list_body = if top_todos.is_empty() {
            self.render_empty_state(cx)
        } else {
            div().flex().flex_col().children(
                top_todos
                    .iter()
                    .map(|todo| self.render_todo_item(todo, 0, window, cx)),
            )
        };

        let list_card = design::card(cx)
            .p(px(10.0))
            .min_h(px(320.0))
            .max_h(px(560.0))
            .overflow_y_scrollbar()
            .child(list_body);

        design::page().child(header_card).child(list_card)
    }
}

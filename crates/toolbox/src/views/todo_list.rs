use std::collections::HashMap;

use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        notification::Notification,
        scroll::ScrollableElement,
        spinner::Spinner,
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

fn summarize(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

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
    search_query: SharedString,
    loaded: bool,
    editing_id: Option<i64>,
    editing_text: SharedString,
    expanded_ids: Vec<i64>,
    adding_sub_for: Option<i64>,
    input_state: Option<Entity<TextareaState>>,
    search_input_state: Option<Entity<InputState>>,
    sub_input_states: HashMap<i64, Entity<TextareaState>>,
    editing_input_states: HashMap<i64, Entity<TextareaState>>,
    composer_focused: bool,
    _subscriptions: Vec<Subscription>,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            new_todo_text: SharedString::default(),
            todos: Vec::new(),
            filter: SharedString::from("all"),
            search_query: SharedString::default(),
            loaded: false,
            editing_id: None,
            editing_text: SharedString::default(),
            expanded_ids: Vec::new(),
            adding_sub_for: None,
            input_state: None,
            search_input_state: None,
            sub_input_states: HashMap::new(),
            editing_input_states: HashMap::new(),
            composer_focused: false,
            _subscriptions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("写下一件要做的事；Enter 添加，Shift+Enter 换行")
                .auto_grow(1, 8)
                .submit_on_enter(true)
        });
        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("搜索待办事项…"));

        let mut subscriptions = Vec::with_capacity(2);

        subscriptions.push(cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(cx).value();
                    this.new_todo_text = value;
                    cx.notify()
                }
                InputEvent::PressEnter { shift, .. } => {
                    if !*shift {
                        this.add_todo(window, cx);
                        cx.notify()
                    }
                }
                InputEvent::Focus => {
                    this.composer_focused = true;
                    cx.notify()
                }
                InputEvent::Blur => {
                    this.composer_focused = false;
                    cx.notify()
                }
            }
        }));

        subscriptions.push(cx.subscribe_in(&search_state, window, {
            let search_state = search_state.clone();
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.search_query = search_state.read(cx).value();
                    cx.notify();
                }
            }
        }));

        self.input_state = Some(input_state);
        self.search_input_state = Some(search_state);
        self._subscriptions = subscriptions;

        cx.notify();
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_todos().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(records) => {
                        this.todos = records.into_iter().map(TodoItem::from_record).collect();
                        this.sort_todos();
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("加载待办失败：{err}")),
                            cx,
                        );
                    }
                }
                this.loaded = true;
                cx.notify();
            });
        })
        .detach();
    }

    fn sort_todos(&mut self) {
        self.todos.sort_by(|a, b| match (a.completed, b.completed) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => b
                .created_at
                .cmp(&a.created_at)
                .then(b.id.unwrap_or(0).cmp(&a.id.unwrap_or(0))),
        });
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

    fn status_matches(&self, todo: &TodoItem) -> bool {
        match self.filter.as_str() {
            "active" => !todo.completed,
            "completed" => todo.completed,
            _ => true,
        }
    }

    fn query_matches(&self, todo: &TodoItem) -> bool {
        let query = self.search_query.trim().to_lowercase();
        query.is_empty() || todo.text.to_lowercase().contains(&query)
    }

    fn has_matching_sub(&self, id: i64) -> bool {
        let query = self.search_query.trim().to_lowercase();
        !query.is_empty()
            && self
                .todos
                .iter()
                .any(|t| t.parent_id == Some(id) && t.text.to_lowercase().contains(&query))
    }

    fn top_todos(&self) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|t| t.parent_id.is_none() && self.status_matches(t))
            .filter(|t| self.query_matches(t) || self.has_matching_sub(t.id.unwrap_or(0)))
            .collect()
    }

    fn sub_todos(&self, parent_id: i64) -> Vec<&TodoItem> {
        let mut subs: Vec<&TodoItem> = self
            .todos
            .iter()
            .filter(|t| {
                t.parent_id == Some(parent_id) && self.status_matches(t) && self.query_matches(t)
            })
            .collect();
        subs.sort_by(|a, b| {
            a.completed
                .cmp(&b.completed)
                .then(a.created_at.cmp(&b.created_at))
                .then(a.id.unwrap_or(0).cmp(&b.id.unwrap_or(0)))
        });
        subs
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

    fn collect_subtree(&self, id: i64) -> Vec<i64> {
        let mut ids = vec![id];
        let mut index = 0;
        while index < ids.len() {
            let current = ids[index];
            for todo in self.todos.iter() {
                if todo.parent_id == Some(current)
                    && let Some(child) = todo.id
                    && !ids.contains(&child)
                {
                    ids.push(child);
                }
            }
            index += 1;
        }
        ids
    }

    fn remove_todos(&mut self, ids: &[i64]) {
        self.todos
            .retain(|t| t.id.is_none_or(|tid| !ids.contains(&tid)));
        self.editing_id = self.editing_id.filter(|id| !ids.contains(id));
        self.adding_sub_for = self.adding_sub_for.filter(|id| !ids.contains(id));
        self.expanded_ids.retain(|id| !ids.contains(id));
        for id in ids {
            self.editing_input_states.remove(id);
            self.sub_input_states.remove(id);
        }
    }

    fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(state) = self.search_input_state.as_ref() {
            state.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
            });
        }
        self.search_query = SharedString::default();
        cx.notify();
    }

    fn focus_composer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(state) = self.input_state.as_ref() {
            state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
        }
    }

    fn add_todo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.new_todo_text.trim().is_empty() {
            window.push_notification(Notification::warning("请输入待办事项内容"), cx);
            return;
        }

        let text = self.new_todo_text.trim().to_string();
        let text_for_restore = text.clone();

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
                        if let Some(input_state) = this.input_state.as_ref() {
                            input_state.update(cx, |state, cx| {
                                state.set_value(text_for_restore.clone(), window, cx);
                            });
                        }
                        this.new_todo_text = SharedString::from(text_for_restore);
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
                let state = cx.new(|cx| {
                    TextareaState::new(window, cx)
                        .placeholder("输入子任务；Enter 添加，Shift+Enter 换行")
                        .auto_grow(1, 3)
                        .submit_on_enter(true)
                });
                let _ = cx.subscribe_in(
                    &state,
                    window,
                    move |this, _, ev: &InputEvent, window, cx| match ev {
                        InputEvent::Change => cx.notify(),
                        InputEvent::PressEnter { shift, .. } => {
                            if !*shift {
                                this.add_sub_todo(id, window, cx);
                                cx.notify()
                            }
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
            return;
        }

        let text_for_restore = text.clone();

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

                        let ancestor_records = if this
                            .todos
                            .iter()
                            .any(|t| t.id == Some(parent_id) && t.completed)
                        {
                            this.propagate_uncomplete_upward(parent_id);
                            this.sort_todos();
                            let mut records = Vec::new();
                            let mut current = Some(parent_id);
                            while let Some(id) = current {
                                if let Some(record) = this
                                    .todos
                                    .iter()
                                    .find(|t| t.id == Some(id))
                                    .map(TodoItem::to_record)
                                {
                                    records.push(record);
                                }
                                current = this
                                    .todos
                                    .iter()
                                    .find(|t| t.id == Some(id))
                                    .and_then(|t| t.parent_id);
                            }
                            records
                        } else {
                            Vec::new()
                        };

                        if !ancestor_records.is_empty() {
                            cx.spawn_in(window, async move |this, cx| {
                                for record in ancestor_records {
                                    let Some(ancestor_id) = record.id else {
                                        continue;
                                    };
                                    if let Err(err) =
                                        config_store::update_todo(ancestor_id, record).await
                                    {
                                        let _ = this.update_in(cx, |_this, window, cx| {
                                            window.push_notification(
                                                Notification::error(format!(
                                                    "同步任务状态失败：{err}"
                                                )),
                                                cx,
                                            );
                                        });
                                    }
                                }
                            })
                            .detach();
                        }

                        if let Some(state) = this.sub_input_states.get(&parent_id) {
                            state.update(cx, |state, cx| {
                                state.focus(window, cx);
                            });
                        }
                    }
                    Err(err) => {
                        if let Some(state) = this.sub_input_states.get(&parent_id) {
                            state.update(cx, |state, cx| {
                                state.set_value(text_for_restore.clone(), window, cx);
                                state.focus(window, cx);
                            });
                        }
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

        let subtree = self.collect_subtree(id);
        let updates: Vec<(i64, TodoRecord)> = self
            .todos
            .iter()
            .filter(|t| {
                subtree.contains(&t.id.unwrap_or(0)) || self.is_ancestor(t.id.unwrap_or(0), id)
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
        let Some(todo) = self.todos.iter().find(|t| t.id == Some(id)) else {
            return;
        };
        let preview = summarize(&todo.text, 28);
        let sub_count = self
            .todos
            .iter()
            .filter(|t| t.parent_id == Some(id))
            .count();

        let this = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            let this = this.clone();
            let description = if sub_count > 0 {
                format!("将删除「{preview}」及其 {sub_count} 个子任务，此操作不可撤销。")
            } else {
                format!("将删除「{preview}」，此操作不可撤销。")
            };
            alert
                .title(div().text_lg().font_semibold().child("确认删除"))
                .width(px(440.))
                .description(div().py_4().text_sm().child(description))
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
        let targets = self.collect_subtree(id);

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let mut deleted: Vec<i64> = Vec::new();
            let mut failure: Option<String> = None;
            for target in targets.iter().copied() {
                match config_store::delete_todo(target).await {
                    Ok(_) => deleted.push(target),
                    Err(err) => {
                        failure = Some(err);
                        break;
                    }
                }
            }

            let _ = this.update_in(cx, |this, window, cx| {
                this.remove_todos(&deleted);
                match failure {
                    Some(err) => {
                        window.push_notification(
                            Notification::error(format!(
                                "删除待办失败，已删除 {} 项：{err}",
                                deleted.len()
                            )),
                            cx,
                        );
                    }
                    None => {
                        let message = if deleted.len() > 1 {
                            format!("已删除 {} 项待办（含子任务）", deleted.len())
                        } else {
                            "待办事项已删除".to_string()
                        };
                        window.push_notification(Notification::success(message), cx);
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

        let mut targets: Vec<i64> = Vec::new();
        for id in completed_ids {
            for target in self.collect_subtree(id) {
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
        let count = targets.len();

        let this = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            let this = this.clone();
            let targets = targets.clone();
            alert
                .title(div().text_lg().font_semibold().child("清除已完成待办"))
                .width(px(440.))
                .description(div().py_4().text_sm().child(format!(
                    "将永久删除 {count} 项已完成的待办（含其子任务），此操作不可撤销。"
                )))
                .confirm()
                .on_ok(move |_, window, cx| {
                    if let Some(this) = this.upgrade() {
                        let targets = targets.clone();
                        this.update(cx, |this, cx| {
                            this.do_clear_completed(targets, window, cx);
                        });
                    }
                    true
                })
        });
    }

    fn do_clear_completed(
        &mut self,
        targets: Vec<i64>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            for target in &targets {
                let _ = config_store::delete_todo(*target).await;
            }
            let records = config_store::load_todos().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match records {
                    Ok(records) => {
                        this.todos = records.into_iter().map(TodoItem::from_record).collect();
                        this.sort_todos();
                        window.push_notification(
                            Notification::success(format!(
                                "已清除 {} 项待办（含子任务）",
                                targets.len()
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
                this.expanded_ids.clear();
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
                let editing_input_state = cx.new(|cx| {
                    TextareaState::new(window, cx)
                        .placeholder("编辑待办事项；Enter 保存，Shift+Enter 换行")
                        .auto_grow(1, 8)
                        .submit_on_enter(true)
                });
                let editing_input_state_clone = editing_input_state.clone();

                let _ = cx.subscribe_in(&editing_input_state, window, {
                    move |this, _, ev: &InputEvent, window, cx| match ev {
                        InputEvent::Change => {
                            let value = editing_input_state_clone.read(cx).value();
                            this.editing_text = value;
                            cx.notify()
                        }
                        InputEvent::PressEnter { shift, .. } => {
                            if !*shift {
                                this.save_edit(window, cx);
                                cx.notify()
                            }
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
        let muted = cx.theme().muted;
        let popover = cx.theme().popover;
        let muted_foreground = cx.theme().muted_foreground;

        div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .gap(px(2.0))
            .p(px(3.0))
            .rounded_full()
            .bg(muted)
            .children(
                FILTER_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, (value, label))| {
                        let selected = self.filter.as_str() == *value;
                        let button = Button::new(("filter", i))
                            .compact()
                            .rounded_full()
                            .px(px(11.0))
                            .toggled(selected)
                            .label(format!("{label} {}", counts[i]))
                            .tooltip(format!("筛选：{label}待办"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.filter = SharedString::from(*value);
                                cx.notify();
                            }));

                        if selected {
                            button.primary()
                        } else {
                            button.custom(
                                ButtonCustomVariant::new(cx)
                                    .foreground(muted_foreground)
                                    .hover(popover),
                            )
                        }
                    }),
            )
    }

    fn render_row_actions(&self, id: i64, depth: usize, cx: &mut Context<Self>) -> Div {
        let id_usize = id as usize;
        let icon_fg = cx.theme().muted_foreground;
        let hover_bg = cx.theme().list_hover;
        let danger = cx.theme().danger;

        let mut actions = div().flex().items_center().gap(px(2.0)).flex_shrink_0();

        if depth == 0 {
            let adding = self.adding_sub_for == Some(id);
            actions = actions.child(
                Button::new(("todo-add-sub", id_usize))
                    .compact()
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .foreground(icon_fg)
                            .hover(hover_bg),
                    )
                    .icon(
                        Icon::new(if adding {
                            IconName::Close
                        } else {
                            IconName::Plus
                        })
                        .size(px(15.0)),
                    )
                    .tooltip(if adding {
                        "结束添加子任务"
                    } else {
                        "添加子任务"
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.adding_sub_for == Some(id) {
                            this.cancel_add_sub(window, cx);
                        } else {
                            this.start_add_sub(id, window, cx);
                        }
                    })),
            );
        }

        actions
            .child(
                Button::new(("todo-edit", id_usize))
                    .compact()
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .foreground(icon_fg)
                            .hover(hover_bg),
                    )
                    .icon(Icon::new(IconName::FileText).size(px(15.0)))
                    .tooltip("编辑任务（也可双击标题）")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_edit(id, window, cx);
                    })),
            )
            .child(
                Button::new(("todo-delete", id_usize))
                    .compact()
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .foreground(icon_fg)
                            .hover(danger.opacity(0.10)),
                    )
                    .icon(Icon::new(IconName::Delete).size(px(15.0)))
                    .tooltip("删除任务")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.delete_todo(id, window, cx);
                    })),
            )
    }

    fn render_empty_state(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let has_query = !self.search_query.trim().is_empty();
        let all_top = self.todos.iter().filter(|t| t.parent_id.is_none()).count();

        let (icon, title, hint) = if has_query {
            (
                IconName::Search,
                "没有匹配的待办",
                "换个关键词试试，或清除当前搜索",
            )
        } else {
            match self.filter.as_str() {
                "active" => (
                    IconName::CircleCheck,
                    "全部待办都已完成",
                    "干得漂亮，休息一下吧",
                ),
                "completed" => (
                    IconName::CircleCheck,
                    "还没有已完成的待办",
                    "勾选完成的任务会归到这里",
                ),
                _ => (
                    IconName::Inbox,
                    "还没有待办事项",
                    "在上方写下第一件事，按回车即可添加",
                ),
            }
        };

        let mut actions = div().flex().items_center().gap_2();

        if has_query {
            actions = actions.child(
                Button::new("empty-clear-search")
                    .primary()
                    .icon(Icon::new(IconName::Close))
                    .label("清除搜索")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.clear_search(window, cx);
                    })),
            );
        } else if self.filter.as_str() == "completed" && all_top > 0 {
            actions = actions.child(
                Button::new("empty-show-all")
                    .ghost()
                    .label("查看全部待办")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.filter = SharedString::from("all");
                        cx.notify();
                    })),
            );
        }

        if !has_query && self.filter.as_str() == "all" && all_top == 0 {
            actions = actions.child(
                Button::new("empty-add")
                    .primary()
                    .icon(Icon::new(IconName::Plus))
                    .label("添加第一项")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.focus_composer(window, cx);
                    })),
            );
        }

        let show_actions = has_query
            || (self.filter.as_str() == "completed" && all_top > 0)
            || (!has_query && self.filter.as_str() == "all" && all_top == 0);

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .py(px(64.0))
            .child(
                div()
                    .w(px(54.0))
                    .h(px(54.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(cx.theme().primary.opacity(0.08))
                    .text_color(cx.theme().primary.opacity(0.8))
                    .child(Icon::new(icon).size(px(26.0))),
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
            .when(show_actions, |empty| {
                empty.child(div().mt_3().child(actions))
            })
    }

    fn render_loading(&self, cx: &mut Context<Self>) -> Div {
        div()
            .w_full()
            .h(px(180.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .child(Spinner::new().color(cx.theme().primary))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(cx.theme().muted_foreground)
                    .child("正在载入待办…"),
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
        let multiline = text.contains('\n');
        let short_text = summarize(&text, 30);
        let completed = todo.completed;
        let has_sub = depth == 0 && self.has_sub_todos(id);
        let visible_subs = if depth == 0 && has_sub {
            self.sub_todos(id)
        } else {
            Vec::new()
        };
        let has_visible_sub = !visible_subs.is_empty();
        let searching = !self.search_query.trim().is_empty();
        let is_expanded =
            self.is_expanded(id) || (searching && has_sub && self.has_matching_sub(id));
        let is_editing = self.editing_id == Some(id);
        let is_adding_sub = depth == 0 && self.adding_sub_for == Some(id);
        let (sub_done, sub_total) = if has_sub {
            self.sub_progress(id)
        } else {
            (0, 0)
        };

        let mut row = div()
            .id(("todo-row", id_usize))
            .flex()
            .items_center()
            .when(multiline, |row| row.items_start())
            .gap(px(10.0))
            .px(px(10.0))
            .py(px(if depth == 0 { 9.0 } else { 6.0 }))
            .rounded(px(10.0))
            .hover(|s: StyleRefinement| s.bg(cx.theme().list_hover));

        if depth == 0 {
            if has_visible_sub {
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
                .accessibility_label(if completed {
                    format!("取消完成：{short_text}")
                } else {
                    format!("标记完成：{short_text}")
                })
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
                edit_area = edit_area.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Textarea::new(state).small().aria_label("编辑待办事项")),
                );
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
                        .tooltip("取消编辑")
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
                        .id(("todo-title", id_usize))
                        .text_size(px(if depth == 0 { 14.0 } else { 12.5 }))
                        .when(depth == 0, |title| title.font_medium())
                        .whitespace_normal()
                        .text_color(if completed {
                            cx.theme().muted_foreground
                        } else {
                            cx.theme().foreground
                        })
                        .when(completed, |label| label.line_through())
                        .child(text)
                        .on_double_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.start_edit(id, window, cx);
                        })),
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
                                .child(div().h_full().w(relative(sub_ratio)).rounded_full().bg(
                                    if sub_done == sub_total {
                                        cx.theme().success
                                    } else {
                                        cx.theme().primary.opacity(0.65)
                                    },
                                )),
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
                .child(self.render_row_actions(id, depth, cx));
        }

        let mut item = div().flex_col().w_full().child(row);

        if depth == 0 && (is_adding_sub || is_expanded) && !is_editing {
            let sub_todos = if is_expanded {
                visible_subs
            } else {
                Vec::new()
            };
            let show_section = is_adding_sub || !sub_todos.is_empty();

            if show_section {
                let mut content = div()
                    .flex_col()
                    .pl(px(14.0))
                    .border_l_1()
                    .border_color(cx.theme().primary.opacity(0.16));

                if !sub_todos.is_empty() {
                    content = content.child(
                        div().flex_col().gap(px(1.0)).children(
                            sub_todos
                                .iter()
                                .map(|sub| self.render_todo_item(sub, 1, window, cx)),
                        ),
                    );
                }

                if is_adding_sub {
                    let can_add_sub = self
                        .sub_input_states
                        .get(&id)
                        .is_some_and(|state| !state.read(cx).value().trim().is_empty());
                    let mut composer = div().flex().items_center().px(px(10.0)).py(px(5.0));

                    composer = composer.child(
                        div()
                            .w(px(16.0))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(cx.theme().muted_foreground)
                            .child(Icon::new(IconName::Plus).size(px(12.0))),
                    );

                    if let Some(state) = self.sub_input_states.get(&id) {
                        composer = composer.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .ml(px(10.0))
                                .child(Textarea::new(state).small().aria_label("新的子任务")),
                        );
                    }

                    let add_sub_button = Button::new(("todo-confirm-sub", id_usize))
                        .compact()
                        .label("添加")
                        .disabled(!can_add_sub)
                        .tooltip("添加子任务（回车继续添加）")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.add_sub_todo(id, window, cx);
                        }));

                    let add_sub_button = if can_add_sub {
                        add_sub_button.primary()
                    } else {
                        add_sub_button.custom(
                            ButtonCustomVariant::new(cx)
                                .foreground(cx.theme().muted_foreground)
                                .hover(cx.theme().popover),
                        )
                    };

                    composer = composer.child(
                        div()
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .gap(px(4.0))
                            .ml(px(8.0))
                            .child(add_sub_button)
                            .child(
                                Button::new(("todo-cancel-sub", id_usize))
                                    .compact()
                                    .custom(
                                        ButtonCustomVariant::new(cx)
                                            .foreground(cx.theme().muted_foreground)
                                            .hover(cx.theme().list_hover),
                                    )
                                    .label("结束")
                                    .tooltip("结束添加子任务")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.cancel_add_sub(window, cx);
                                    })),
                            ),
                    );

                    content = content.child(composer);
                }

                let mut section = div()
                    .flex_col()
                    .ml(px(28.0))
                    .pr(px(4.0))
                    .pb(px(8.0))
                    .child(content);

                if !is_adding_sub && self.search_query.trim().is_empty() {
                    section = section.child(
                        div().flex().pl(px(24.0)).py(px(2.0)).child(
                            Button::new(("todo-inline-add-sub", id_usize))
                                .compact()
                                .text_size(px(12.5))
                                .custom(
                                    ButtonCustomVariant::new(cx)
                                        .foreground(cx.theme().muted_foreground)
                                        .hover(cx.theme().popover),
                                )
                                .icon(Icon::new(IconName::Plus).size(px(12.0)))
                                .label("添加子任务")
                                .tooltip("在本任务下添加子任务")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.start_add_sub(id, window, cx);
                                })),
                        ),
                    );
                }

                item = item.child(section);
            }
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
        let counts = self.top_level_counts();
        let total = counts[0];
        let pending = counts[1];
        let done = counts[2];
        let done_all = self.todos.iter().filter(|t| t.completed).count();
        let can_add = !self.new_todo_text.trim().is_empty();
        let has_query = !self.search_query.trim().is_empty();

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
                    .w(px(40.0))
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .rounded(px(10.0))
                    .bg(cx.theme().muted)
                    .text_color(cx.theme().muted_foreground)
                    .child(Icon::new(IconName::Inbox).size(px(19.0))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("待办事项"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(5.0))
                            .when(pending == 0 && total > 0, |row| {
                                row.child(
                                    div()
                                        .text_color(cx.theme().success)
                                        .child(Icon::new(IconName::CircleCheck).size(px(12.0))),
                                )
                            })
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(subtitle),
                            ),
                    ),
            )
            .when(done_all > 0, |header| {
                header.child(
                    Button::new("clear-completed")
                        .compact()
                        .text_size(px(12.5))
                        .custom(
                            ButtonCustomVariant::new(cx)
                                .foreground(cx.theme().muted_foreground)
                                .hover(cx.theme().danger.opacity(0.08)),
                        )
                        .icon(Icon::new(IconName::Delete).size(px(13.0)))
                        .label("清除已完成")
                        .tooltip(format!("删除 {done_all} 项已完成的待办（含子任务）"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.clear_completed(window, cx);
                        })),
                )
            });

        let composer_border = if self.composer_focused {
            cx.theme().primary
        } else {
            cx.theme().border
        };

        let add_button = Button::new("add-todo")
            .label("添加")
            .tooltip("添加待办（回车）")
            .disabled(!can_add)
            .on_click(cx.listener(|this, _, window, cx| {
                this.add_todo(window, cx);
                cx.notify();
            }));

        let add_button = if can_add {
            add_button.primary()
        } else {
            add_button.custom(
                ButtonCustomVariant::new(cx)
                    .foreground(cx.theme().muted_foreground)
                    .hover(cx.theme().popover),
            )
        };

        let composer_row = div()
            .flex()
            .items_end()
            .gap(px(6.0))
            .min_h(px(44.0))
            .py(px(5.0))
            .pl(px(12.0))
            .pr(px(5.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(composer_border)
            .bg(cx.theme().muted)
            .child(
                div().flex_1().min_w_0().child(
                    Textarea::new(input_state)
                        .aria_label("新的待办事项")
                        .bordered(false)
                        .bg(cx.theme().transparent),
                ),
            )
            .child(add_button);

        let mut search_input = Input::new(self.search_input_state.as_ref().unwrap())
            .small()
            .aria_label("搜索待办事项")
            .cleanable(false)
            .prefix(
                div()
                    .pl_1()
                    .text_color(cx.theme().muted_foreground)
                    .child(Icon::new(IconName::Search).size(px(13.0))),
            );

        if has_query {
            search_input = search_input.suffix(
                Button::new("clear-search")
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Close).size(px(13.0)))
                    .tooltip("清除搜索")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.clear_search(window, cx);
                    })),
            );
        }

        let count_label = div()
            .text_size(px(11.5))
            .text_color(cx.theme().muted_foreground)
            .child(format!("匹配 {} 项", top_todos.len()));

        let toolbar_row = div()
            .flex()
            .items_center()
            .gap_3()
            .child(self.render_filter_tabs(counts, cx))
            .child(div().flex_1())
            .when(has_query, |row| row.child(count_label))
            .child(div().w(px(210.0)).flex_shrink_0().child(search_input));

        let composer_block = div().flex_col().gap(px(5.0)).child(composer_row).child(
            div()
                .pl(px(2.0))
                .text_size(px(11.0))
                .text_color(cx.theme().muted_foreground),
        );

        let header_card = design::card(cx)
            .gap(px(12.0))
            .child(header_row)
            .child(composer_block)
            .child(toolbar_row);

        let list_body = if !self.loaded {
            self.render_loading(cx)
        } else if top_todos.is_empty() {
            self.render_empty_state(window, cx)
        } else {
            let total = top_todos.len();
            div()
                .flex()
                .flex_col()
                .children(top_todos.iter().enumerate().map(|(index, todo)| {
                    let item = self.render_todo_item(todo, 0, window, cx);
                    let mut block = div().flex().flex_col().child(item);
                    if index + 1 < total {
                        block = block.child(div().h(px(1.0)).mx(px(10.0)).bg(cx.theme().border));
                    }
                    block
                }))
        };

        let list_card = design::card(cx)
            .flex()
            .flex_1()
            .min_h(px(0.0))
            .p(px(8.0))
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scrollbar()
                    .child(list_body),
            );

        design::page()
            .flex()
            .h(window.viewport_size().height - px(56.0))
            .child(header_card)
            .child(list_card)
    }
}

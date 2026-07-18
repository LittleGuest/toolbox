use std::collections::HashMap;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState},
    radio::{Radio, RadioGroup},
    scroll::ScrollableElement,
    *,
};

use crate::config_store::{self, TodoRecord};

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
    save_edit_pending: bool,
    expanded_ids: Vec<i64>,
    input_state: Option<Entity<InputState>>,
    editing_input_states: HashMap<i64, Entity<InputState>>,
    current_parent_id: Option<i64>,
    status: String,
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
            save_edit_pending: false,
            expanded_ids: Vec::new(),
            input_state: None,
            editing_input_states: HashMap::new(),
            current_parent_id: None,
            status: String::new(),
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
                    this.add_todo(cx);
                    cx.notify()
                }
                _ => {}
            }
        })];

        self.input_state = Some(input_state);
        self._subscriptions = _subscriptions;

        // 从 SQLite 加载已有待办
        self.status = "正在加载待办...".to_string();
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_todos().await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(records) => {
                        this.todos = records.into_iter().map(TodoItem::from_record).collect();
                        this.sort_todos();
                        this.status = format!("已加载 {} 条待办。", this.todos.len());
                    }
                    Err(err) => {
                        this.status = format!("加载待办失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn sort_todos(&mut self) {
        // 未完成在前 + created_at 倒序
        self.todos.sort_by(|a, b| match (a.completed, b.completed) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => b.created_at.cmp(&a.created_at),
        });
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

    fn add_todo(&mut self, cx: &mut Context<Self>) {
        if self.new_todo_text.trim().is_empty() {
            return;
        }

        let text = self.new_todo_text.trim().to_string();

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

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let parent_id = self.current_parent_id;
        let record = TodoRecord {
            id: None,
            parent_id,
            content: text.clone(),
            completed: false,
            created_at: now,
        };
        let content_clone = text;
        let parent_id_clone = parent_id;

        self.status = "正在保存待办...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_todo(record).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(new_id) => {
                        this.todos.insert(
                            0,
                            TodoItem {
                                id: Some(new_id),
                                text: SharedString::from(content_clone),
                                completed: false,
                                parent_id: parent_id_clone,
                                created_at: now,
                            },
                        );
                        this.sort_todos();
                        this.status = "待办已添加。".to_string();
                    }
                    Err(err) => {
                        this.status = format!("保存待办失败：{err}");
                    }
                }
                this.new_todo_text = SharedString::default();
                cx.notify();
            });
        })
        .detach();

        self.new_todo_text = SharedString::default();
    }

    fn get_todo_depth(&self, id: i64) -> usize {
        let mut depth = 0;
        let mut current_id = id;

        while let Some(parent_id) = self
            .todos
            .iter()
            .find(|t| t.id == Some(current_id))
            .and_then(|t| t.parent_id)
        {
            depth += 1;
            current_id = parent_id;
        }

        depth
    }

    fn add_sub_todo(&mut self, parent_id: i64, cx: &mut Context<Self>) {
        if self.new_todo_text.trim().is_empty() {
            return;
        }

        let text = self.new_todo_text.trim().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let record = TodoRecord {
            id: None,
            parent_id: Some(parent_id),
            content: text.clone(),
            completed: false,
            created_at: now,
        };
        let content_clone = text;

        self.status = "正在保存子待办...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_todo(record).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(new_id) => {
                        this.todos.insert(
                            0,
                            TodoItem {
                                id: Some(new_id),
                                text: SharedString::from(content_clone),
                                completed: false,
                                parent_id: Some(parent_id),
                                created_at: now,
                            },
                        );
                        this.sort_todos();
                        this.status = "子待办已添加。".to_string();
                    }
                    Err(err) => {
                        this.status = format!("保存子待办失败：{err}");
                    }
                }
                this.new_todo_text = SharedString::default();
                cx.notify();
            });
        })
        .detach();

        self.new_todo_text = SharedString::default();
    }

    fn set_parent_id(&mut self, parent_id: i64, cx: &mut Context<Self>) {
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

    fn toggle_complete(&mut self, id: i64, cx: &mut Context<Self>) {
        let completed = if let Some(todo) = self.todos.iter_mut().find(|t| t.id == Some(id)) {
            todo.completed = !todo.completed;
            todo.completed
        } else {
            return;
        };

        // 向下传播：所有子任务同步完成状态
        self.toggle_complete_recursive_down(id, completed);

        // 向上传播：如果完成，检查同级是否全完成；如果取消完成，父级也取消
        if completed {
            self.propagate_complete_upward(id);
        } else {
            self.propagate_uncomplete_upward(id);
        }

        // 收集所有需要更新的待办，异步落库
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

    /// 向上传播完成：如果同级全部完成，则父任务也标记完成
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

    /// 向上传播取消完成：任一子任务未完成，父任务也取消完成
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

    /// 检查 ancestor_id 是否是 id 的祖先
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
        window.open_dialog(cx, move |dialog, _, _cx| {
            let this = this.clone();
            dialog
                .title(div().text_lg().font_semibold().child("确认删除"))
                .width(px(420.))
                .child(
                    div()
                        .py_4()
                        .text_sm()
                        .child("确定要删除这个待办事项及其所有子项目吗？"),
                )
                .confirm()
                .on_ok(move |_, _, cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.confirm_delete_todo(id, cx);
                        });
                    }
                    true
                })
        });
    }

    fn confirm_delete_todo(&mut self, id: i64, cx: &mut Context<Self>) {
        self.status = "正在删除待办...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            // DB 的 ON DELETE CASCADE 会自动删除子任务
            let result = config_store::delete_todo(id).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(true) => {
                        // 内存中也递归删除
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
                        this.status = "待办已删除。".to_string();
                    }
                    Ok(false) => {
                        this.status = "未找到要删除的待办。".to_string();
                    }
                    Err(err) => {
                        this.status = format!("删除待办失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn clear_completed(&mut self, cx: &mut Context<Self>) {
        // 仅清除顶级已完成任务（子任务随父 ON DELETE CASCADE）
        let top_completed: Vec<i64> = self
            .todos
            .iter()
            .filter(|t| t.parent_id.is_none() && t.completed)
            .filter_map(|t| t.id)
            .collect();

        if top_completed.is_empty() {
            return;
        }

        self.status = format!("正在清除 {} 个已完成的顶级待办...", top_completed.len());
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let mut deleted = 0;
            for id in &top_completed {
                if config_store::delete_todo(*id).await.map_or(false, |ok| ok) {
                    deleted += 1;
                }
            }
            let _ = this.update(cx, |this, cx| {
                // 内存中也递归删除
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
                this.status = format!("已清除 {} 个已完成的顶级待办。", deleted);
                cx.notify();
            });
        })
        .detach();
    }

    fn start_edit(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(todo) = self.todos.iter().find(|t| t.id == Some(id)) {
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
                        InputEvent::PressEnter { .. } => {
                            // Enter 保存编辑需要 window，这里通过 spawn_in 处理
                            // 简化：直接标记需要保存
                            this.save_edit_pending = true;
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
                let value = editing_input_state.read(cx).value().to_string();
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == Some(id)) {
                    todo.text = SharedString::from(value.clone());
                    let record = todo.to_record();
                    let id_for_update = id;

                    cx.spawn(async move |this: WeakEntity<Self>, cx| {
                        let _ = config_store::update_todo(id_for_update, record).await;
                        let _ = this.update(cx, |this, cx| {
                            this.status = "待办已更新。".to_string();
                            cx.notify();
                        });
                    })
                    .detach();
                }
            }
        }
        self.editing_id = None;
        self.editing_text = SharedString::default();
        self.save_edit_pending = false;
    }

    fn cancel_edit(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.editing_id = None;
        self.editing_text = SharedString::default();
        self.save_edit_pending = false;
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
                        Button::new(("expand", id_usize))
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
                        Checkbox::new(("check", id_usize))
                            .checked(completed)
                            .on_click(cx.listener(move |this, _checked, _, cx| {
                                this.toggle_complete(id, cx);
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
                                    ButtonGroup::new(("edit-actions", id_usize))
                                        .child(
                                            Button::new(("save-edit", id_usize))
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
                                            Button::new(("cancel-edit", id_usize))
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
                            Button::new(("add-sub", id_usize))
                                .icon(Icon::new(IconName::Plus))
                                .tooltip("添加子项目")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.set_parent_id(id, cx);
                                    // 清空输入框
                                    if let Some(input_state) = &this.input_state {
                                        input_state.update(cx, |state, cx| {
                                            state.replace(SharedString::default(), window, cx);
                                        });
                                    }
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        ButtonGroup::new(("action-buttons", id_usize))
                            .child(
                                Button::new(("edit", id_usize))
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("编辑")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.start_edit(id, window, cx);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new(("delete", id_usize))
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("删除")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.delete_todo(id, window, cx);
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

        let input_state = self.input_state.as_ref().unwrap();

        let status_bar = if self.status.is_empty() {
            None
        } else {
            Some(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.status.clone()),
            )
        };

        div()
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
                                if let Some(parent_id) = this.current_parent_id {
                                    this.add_sub_todo(parent_id, cx);
                                } else {
                                    this.add_todo(cx);
                                }
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
                        RadioGroup::horizontal("filter-group")
                            .selected_index(match self.filter.as_str() {
                                "all" => Some(0),
                                "active" => Some(1),
                                "completed" => Some(2),
                                _ => Some(0),
                            })
                            .on_click(cx.listener(|this, idx: &usize, _, cx| {
                                this.filter = match idx {
                                    1 => SharedString::from("active"),
                                    2 => SharedString::from("completed"),
                                    _ => SharedString::from("all"),
                                };
                                cx.notify();
                            }))
                            .child(Radio::new("filter-all").label("全部"))
                            .child(Radio::new("filter-active").label("未完成"))
                            .child(Radio::new("filter-completed").label("已完成")),
                    )
                    .child(
                        Button::new("clear-completed")
                            .icon(Icon::new(IconName::Delete))
                            .tooltip("清除已完成的顶级待办")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_completed(cx);
                                cx.notify();
                            })),
                    ),
            )
            .children(status_bar)
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

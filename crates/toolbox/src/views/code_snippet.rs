use gpui_kit::{
    component::{
        button::*,
        input::{Input, InputEvent, InputState},
        notification::Notification,
        scroll::ScrollableElement,
        *,
    },
    prelude::FluentBuilder,
    *,
};

use crate::{
    config_store::{self, SnippetRecord},
    views::MarkdownPane,
};

pub struct Snippet {
    id: Option<i64>,
    title: SharedString,
    tags: Vec<SharedString>,
    code: SharedString,
    language: SharedString,
    created_at: i64,
    updated_at: i64,
}

impl Snippet {
    fn from_record(record: SnippetRecord) -> Self {
        Self {
            id: record.id,
            title: SharedString::from(record.title),
            tags: record.tags.into_iter().map(SharedString::from).collect(),
            code: SharedString::from(record.code),
            language: SharedString::from(record.language),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }

    fn to_record(&self) -> SnippetRecord {
        SnippetRecord {
            id: self.id,
            title: self.title.to_string(),
            code: self.code.to_string(),
            tags: self.tags.iter().map(|t| t.to_string()).collect(),
            language: self.language.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn first_line(code: &str) -> String {
    code.split('\n')
        .map(|l| l.trim_end_matches('\r'))
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

fn line_count(code: &str) -> usize {
    if code.is_empty() {
        0
    } else {
        code.split('\n').count()
    }
}

pub struct CodeSnippet {
    snippets: Vec<Snippet>,
    tags: Vec<SharedString>,
    selected_tags: Vec<SharedString>,

    selected_snippet_id: Option<i64>,

    editing_id: Option<i64>,
    search_input_state: Option<Entity<InputState>>,
    title_input_state: Option<Entity<InputState>>,

    md_editor: Option<Entity<MarkdownPane>>,
    tag_input_state: Option<Entity<InputState>>,
    search_text: SharedString,
    tag_input_text: SharedString,
    current_title: SharedString,
    current_tags: Vec<SharedString>,
    current_language: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl CodeSnippet {
    pub fn new() -> Self {
        Self {
            snippets: Vec::new(),
            tags: Vec::new(),
            selected_tags: Vec::new(),
            selected_snippet_id: None,
            editing_id: None,
            search_input_state: None,
            title_input_state: None,
            md_editor: None,
            tag_input_state: None,
            search_text: SharedString::default(),
            tag_input_text: SharedString::default(),
            current_title: SharedString::default(),
            current_tags: Vec::new(),
            current_language: SharedString::default(),
            _subscriptions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let search_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索标题或代码内容…"));

        let title_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("一句话描述"));

        let tag_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入或选择标签"));

        let search_clone = search_input_state.clone();
        let title_clone = title_input_state.clone();
        let tag_clone = tag_input_state.clone();

        let _subscriptions = vec![
            cx.subscribe_in(&search_input_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let value = search_clone.read(cx).value();
                        this.search_text = value.clone();
                        cx.notify()
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(&title_input_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let value = title_clone.read(cx).value();
                        this.current_title = value.clone();
                        cx.notify()
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(&tag_input_state, window, {
                move |this, _, ev: &InputEvent, window, cx| match ev {
                    InputEvent::Change => {
                        let value = tag_clone.read(cx).value();
                        this.tag_input_text = value.clone();
                        cx.notify()
                    }

                    InputEvent::PressEnter { .. } => {
                        this.add_tag(window, cx);
                    }
                    _ => {}
                }
            }),
        ];

        self.search_input_state = Some(search_input_state);
        self.title_input_state = Some(title_input_state);
        self.tag_input_state = Some(tag_input_state);
        self._subscriptions = _subscriptions;

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_snippets().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(records) => {
                        this.snippets = records.into_iter().map(Snippet::from_record).collect();
                        this.update_tags();
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("加载片段失败：{err}")),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn filtered_snippets(&self) -> Vec<&Snippet> {
        let mut result: Vec<&Snippet> = self.snippets.iter().collect();

        if !self.selected_tags.is_empty() {
            result = result
                .into_iter()
                .filter(|snippet| {
                    self.selected_tags
                        .iter()
                        .all(|tag| snippet.tags.contains(tag))
                })
                .collect();
        }

        if !self.search_text.is_empty() {
            let query = self.search_text.to_lowercase();
            result = result
                .into_iter()
                .filter(|snippet| {
                    snippet.title.to_lowercase().contains(&query)
                        || snippet.code.to_lowercase().contains(&query)
                })
                .collect();
        }

        result
    }

    fn toggle_tag_filter(&mut self, tag: SharedString) {
        if let Some(pos) = self.selected_tags.iter().position(|t| t == &tag) {
            self.selected_tags.remove(pos);
        } else {
            self.selected_tags.push(tag);
        }
    }

    fn reset_tag_filter(&mut self) {
        self.selected_tags.clear();
    }

    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.current_title = SharedString::default();
        self.current_tags = Vec::new();

        self.current_language = SharedString::default();
        self.tag_input_text = SharedString::default();
        self.md_editor = None;

        if let Some(title_input) = &self.title_input_state {
            title_input.update(cx, |input_state, cx| {
                input_state.set_value("".to_string(), window, cx);
            });
        }
        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.set_value("".to_string(), window, cx);
            });
        }
    }

    fn new_md_editor(
        &self,
        initial: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<MarkdownPane> {
        let pane = cx.new(|cx| {
            MarkdownPane::new(window, cx)
                .file_actions(false)
                .editor_height(290.0, 360.0)
        });
        if !initial.is_empty() {
            pane.update(cx, |pane, cx| {
                pane.set_content(initial.to_string(), window, cx);
            });
        }
        pane
    }

    fn start_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.reset_form(window, cx);
        self.md_editor = Some(self.new_md_editor("", window, cx));
        self.open_edit_sheet(window, cx);
    }

    fn start_edit(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let Some((title, code, tags, language)) =
            self.snippets.iter().find(|s| s.id == Some(id)).map(|s| {
                (
                    s.title.clone(),
                    s.code.to_string(),
                    s.tags.clone(),
                    s.language.clone(),
                )
            })
        else {
            return;
        };

        self.editing_id = Some(id);
        self.reset_form(window, cx);
        self.current_title = title.clone();
        self.current_tags = tags;
        self.current_language = language;

        if let Some(title_input) = &self.title_input_state {
            title_input.update(cx, |input_state, cx| {
                input_state.set_value(title.to_string(), window, cx);
            });
        }
        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.set_value("".to_string(), window, cx);
            });
        }
        self.tag_input_text = SharedString::default();
        self.md_editor = Some(self.new_md_editor(&code, window, cx));

        self.open_edit_sheet(window, cx);
    }

    fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_id = None;
        self.reset_form(window, cx);
        window.close_sheet(cx);
    }

    fn open_edit_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.entity().downgrade();

        let is_edit = self.editing_id.is_some();
        let title_state = self.title_input_state.clone();
        let md_editor = self.md_editor.clone();
        let tag_state = self.tag_input_state.clone();
        let all_tags = self.tags.clone();

        window.open_sheet_at(Placement::Bottom, cx, move |sheet, _, cx| {
            let this_ref = weak.upgrade();
            let form_tags = this_ref
                .as_ref()
                .map(|this| this.read(cx).current_tags.clone())
                .unwrap_or_default();

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(relative(1.0))
                .resizable(true)
                .title(if is_edit { "编辑" } else { "添加" })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .child(
                            div()
                                .flex_col()
                                .gap_1p5()
                                .child(design::editor_label("一句话", cx))
                                .child(if let Some(ref ts) = title_state {
                                    div().child(Input::new(ts))
                                } else {
                                    div()
                                }),
                        )
                        .child(
                            div()
                                .flex_col()
                                .gap_1p5()
                                .child(design::editor_label("标签", cx))
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(if let Some(ref tis) = tag_state {
                                            div().flex_1().child(Input::new(tis))
                                        } else {
                                            div()
                                        })
                                        .child({
                                            let weak = weak.clone();
                                            Button::new("sheet-add-tag")
                                                .icon(Icon::new(IconName::Plus))
                                                .tooltip("添加标签")
                                                .on_click(move |_, window, cx| {
                                                    if let Some(this) = weak.upgrade() {
                                                        this.update(cx, |this, cx| {
                                                            this.add_tag(window, cx);
                                                        });
                                                    }
                                                })
                                        }),
                                )
                                .when(!form_tags.is_empty(), |col| {
                                    col.child(div().flex().flex_wrap().gap_1p5().children(
                                        form_tags.iter().enumerate().map(|(i, tag)| {
                                            let tag = tag.clone();
                                            let weak = weak.clone();
                                            let tag_label = tag.clone();
                                            div()
                                                .id(("sheet-tag", i))
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .px_2()
                                                .py(px(2.0))
                                                .rounded_full()
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .bg(cx.theme().secondary)
                                                .text_size(px(11.5))
                                                .hover(|s: StyleRefinement| {
                                                    s.border_color(cx.theme().primary)
                                                })
                                                .on_click(move |_, _window, cx| {
                                                    if let Some(this) = weak.upgrade() {
                                                        this.update(cx, |this, cx| {
                                                            this.remove_tag(&tag);
                                                            cx.notify();
                                                        });
                                                    }
                                                })
                                                .child(tag_label)
                                                .child(
                                                    Icon::new(IconName::Close)
                                                        .size(px(10.0))
                                                        .text_color(cx.theme().muted_foreground),
                                                )
                                        }),
                                    ))
                                })
                                .when(!all_tags.is_empty(), |col| {
                                    col.child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .items_center()
                                            .gap_1p5()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("可选："),
                                            )
                                            .children(all_tags.iter().enumerate().map(
                                                |(i, tag)| {
                                                    let tag = tag.clone();
                                                    let weak = weak.clone();
                                                    let label = format!("# {tag}");
                                                    div()
                                                        .id(("sheet-suggest", i))
                                                        .px_2()
                                                        .py(px(1.0))
                                                        .rounded(px(6.0))
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .hover(|s: StyleRefinement| {
                                                            s.border_color(cx.theme().primary)
                                                                .bg(design::tint(0x4f6ef7, 0.08))
                                                        })
                                                        .on_click(move |_, window, cx| {
                                                            if let Some(this) = weak.upgrade() {
                                                                this.update(cx, |this, cx| {
                                                                    this.add_existing_tag(
                                                                        tag.clone(),
                                                                        window,
                                                                        cx,
                                                                    );
                                                                });
                                                            }
                                                        })
                                                        .child(label)
                                                },
                                            )),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .flex_col()
                                .gap_1p5()
                                .child(div().children(md_editor.clone())),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .mt_2()
                                .child({
                                    let weak = weak.clone();
                                    Button::new("sheet-cancel")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("取消")
                                        .on_click(move |_, window, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    this.cancel_edit(window, cx);
                                                });
                                            }
                                        })
                                })
                                .child({
                                    let weak = weak.clone();

                                    Button::new("sheet-save")
                                        .icon(Icon::new(IconName::Check))
                                        .tooltip("保存")
                                        .on_click(move |_, window, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    if this.save_snippet(window, cx) {
                                                        window.close_sheet(cx);
                                                    }
                                                });
                                            }
                                        })
                                }),
                        ),
                )
        });
    }

    fn add_tag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tag_str = self.tag_input_text.to_string();
        for part in tag_str.split(',') {
            let tag = part.trim();
            if tag.is_empty() {
                continue;
            }
            let tag_shared = SharedString::from(tag.to_string());
            if !self.current_tags.contains(&tag_shared) {
                self.current_tags.push(tag_shared);
            }
        }

        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.set_value("".to_string(), window, cx);
            });
        }
        self.tag_input_text = SharedString::default();
        cx.notify();
    }

    fn add_existing_tag(
        &mut self,
        tag: SharedString,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.current_tags.contains(&tag) {
            self.current_tags.push(tag);
            cx.notify();
        }
    }

    fn remove_tag(&mut self, tag: &SharedString) {
        self.current_tags.retain(|t| t != tag);
    }

    fn save_snippet(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.current_title.trim().is_empty() {
            window.push_notification(Notification::error("请一句话描述"), cx);
            return false;
        }
        if self.current_tags.is_empty() {
            window.push_notification(Notification::error("请选择或输入标签"), cx);
            return false;
        }

        let code = self
            .md_editor
            .as_ref()
            .map(|editor| editor.read(cx).content().to_string())
            .unwrap_or_default();
        if code.trim().is_empty() {
            window.push_notification(Notification::error("请输入内容"), cx);
            return false;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if let Some(id) = self.editing_id {
            if let Some(snippet) = self.snippets.iter_mut().find(|s| s.id == Some(id)) {
                snippet.title = self.current_title.clone();
                snippet.tags = self.current_tags.clone();
                snippet.code = SharedString::from(code);
                snippet.language = self.current_language.clone();
                snippet.updated_at = now;
                let record = snippet.to_record();
                self.update_tags();

                cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
                    let result = config_store::update_snippet(id, record).await;
                    let _ = this.update_in(cx, |this, window, cx| {
                        match result {
                            Ok(_) => {
                                window.push_notification(Notification::success("保存成功"), cx);
                            }
                            Err(err) => {
                                window.push_notification(
                                    Notification::error(format!("保存失败：{err}")),
                                    cx,
                                );
                            }
                        }
                        this.editing_id = None;
                        cx.notify();
                    });
                })
                .detach();
            }
        } else {
            let record = SnippetRecord {
                id: None,
                title: self.current_title.to_string(),
                code: code.clone(),
                tags: self.current_tags.iter().map(|t| t.to_string()).collect(),
                language: self.current_language.to_string(),
                created_at: now,
                updated_at: now,
            };

            let new_code = SharedString::from(code);

            cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
                let result = config_store::save_snippet(record).await;
                let _ = this.update_in(cx, move |this, window, cx| {
                    match result {
                        Ok(new_id) => {
                            this.snippets.insert(
                                0,
                                Snippet {
                                    id: Some(new_id),
                                    title: this.current_title.clone(),
                                    tags: this.current_tags.clone(),
                                    code: new_code,
                                    language: this.current_language.clone(),
                                    created_at: now,
                                    updated_at: now,
                                },
                            );
                            window.push_notification(Notification::success("保存成功"), cx);
                        }
                        Err(err) => {
                            window.push_notification(
                                Notification::error(format!("保存失败：{err}")),
                                cx,
                            );
                        }
                    }
                    this.editing_id = None;
                    this.update_tags();
                    cx.notify();
                });
            })
            .detach();
        }

        true
    }

    fn delete_snippet(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let this = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            let this = this.clone();
            alert
                .title(div().text_lg().font_semibold().child("确认删除"))
                .width(px(420.))
                .description(div().py_4().text_sm().child("是否确认删除？"))
                .confirm()
                .on_ok(move |_, window, cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.confirm_delete_snippet(id, window, cx);
                        });
                    }
                    true
                })
        });
    }

    fn confirm_delete_snippet(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_snippet(id).await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(true) => {
                        this.snippets.retain(|s| s.id != Some(id));
                        if this.selected_snippet_id == Some(id) {
                            this.selected_snippet_id = None;
                        }
                        this.update_tags();
                        window.push_notification(Notification::success("代码片段已删除"), cx);
                    }
                    Ok(false) => {
                        window
                            .push_notification(Notification::warning("未找到要删除的代码片段"), cx);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("删除片段失败：{err}")),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn import_snippets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("导入代码片段")
                .add_filter("JSON", &["json"])
                .pick_file()
                .await
        });

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let Some(file) = task.await else {
                return;
            };
            let content = file.read().await;
            let content_str = String::from_utf8_lossy(&content).to_string();

            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content_str);
            let list = match parsed {
                Ok(v) => {
                    if let Some(arr) = v.as_array() {
                        Some(arr.clone())
                    } else {
                        v.get("snippets").and_then(|s| s.as_array()).cloned()
                    }
                }
                Err(_) => None,
            };

            let Some(list) = list else {
                let _ = this.update_in(cx, |_this, window, cx| {
                    window.push_notification(Notification::error("导入文件格式不正确"), cx);
                });
                return;
            };

            let mut count = 0usize;
            for item in list {
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let code = item
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if title.is_empty() || code.is_empty() {
                    continue;
                }

                let tags_str = match item.get("tags") {
                    Some(serde_json::Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(","),
                    Some(serde_json::Value::String(s)) => s
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect::<Vec<_>>()
                        .join(","),
                    _ => String::new(),
                };
                let language = item
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let record = SnippetRecord {
                    id: None,
                    title,
                    code,
                    tags: tags_str
                        .split(',')
                        .filter(|t| !t.is_empty())
                        .map(|t| t.to_string())
                        .collect(),
                    language,
                    created_at: now,
                    updated_at: now,
                };
                if config_store::save_snippet(record).await.is_ok() {
                    count += 1;
                }
            }

            let reload = config_store::load_snippets().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match reload {
                    Ok(records) => {
                        this.snippets = records.into_iter().map(Snippet::from_record).collect();
                        this.update_tags();
                        window.push_notification(
                            Notification::success(format!("导入完成，共导入 {count} 条")),
                            cx,
                        );
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(format!("刷新列表失败：{err}")),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn export_snippets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let data: Vec<serde_json::Value> = self
            .filtered_snippets()
            .iter()
            .map(|s| {
                serde_json::json!({
                    "language": s.language.to_string(),
                    "title": s.title.to_string(),
                    "tags": s.tags.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                    "code": s.code.to_string(),
                })
            })
            .collect();

        if data.is_empty() {
            window.push_notification(Notification::warning("没有可导出的代码片段"), cx);
            return;
        }

        let count = data.len();
        let payload = serde_json::json!({ "version": 1, "snippets": data });
        let json_content = serde_json::to_string_pretty(&payload).unwrap_or_default();
        let file_name = format!(
            "code-snippets-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("导出代码片段")
                .add_filter("JSON", &["json"])
                .set_file_name(&file_name)
                .save_file()
                .await
        });

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let _ = file.write(json_content.as_bytes()).await;
                let _ = this.update_in(cx, |_this, window, cx| {
                    window.push_notification(
                        Notification::success(format!("已导出 {count} 条代码片段")),
                        cx,
                    );
                });
            }
        })
        .detach();
    }

    fn update_tags(&mut self) {
        let mut ordered: Vec<SharedString> = Vec::new();
        for snippet in &self.snippets {
            for tag in &snippet.tags {
                if !ordered.contains(tag) {
                    ordered.push(tag.clone());
                }
            }
        }
        self.tags = ordered;
    }

    fn render_tag_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let all_tags = self.tags.clone();
        let selected_tags = self.selected_tags.clone();

        let mut title_row = div()
            .flex()
            .items_center()
            .justify_between()
            .mb_3()
            .child(design::editor_label("标签", cx));
        if !selected_tags.is_empty() {
            title_row = title_row.child(
                Button::new("reset-tags")
                    .ghost()
                    .compact()
                    .label("重置")
                    .tooltip("重置标签筛选")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reset_tag_filter();
                        cx.notify();
                    })),
            );
        }

        let tag_list = div()
            .flex_col()
            .gap_1()
            .children(all_tags.iter().enumerate().map(|(idx, tag)| {
                let is_selected = selected_tags.contains(tag);
                let tag_clone = tag.clone();
                div()
                    .id(("filter-tag", idx))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1()
                    .rounded(px(6.0))
                    .text_sm()
                    .text_color(if is_selected {
                        cx.theme().primary
                    } else {
                        cx.theme().muted_foreground
                    })
                    .when(is_selected, |row| {
                        row.bg(cx.theme().primary.opacity(0.1)).font_semibold()
                    })
                    .hover(|s: StyleRefinement| s.bg(cx.theme().background))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tag_filter(tag_clone.clone());
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(if is_selected {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            })
                            .child("#"),
                    )
                    .child(tag.clone())
            }));

        div()
            .w(px(180.0))
            .flex_shrink_0()
            .border_r_1()
            .border_color(cx.theme().border)
            .pr_4()
            .child(title_row)
            .child(tag_list)
    }

    fn render_snippet_item(&self, snippet: &Snippet, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = snippet.id.unwrap_or(0);
        let id_usize = id as usize;
        let title = snippet.title.clone();
        let tags = snippet.tags.clone();
        let code = snippet.code.clone();
        let is_selected = self.selected_snippet_id == Some(id);
        let lines = line_count(&code);
        let preview = {
            let first = first_line(&code);
            if first.is_empty() {
                "（空内容）".to_string()
            } else {
                first
            }
        };

        let tags_row = div()
            .flex()
            .flex_wrap()
            .gap_1p5()
            .mt_2()
            .children(tags.iter().map(|tag| {
                div()
                    .px_2()
                    .py(px(2.0))
                    .rounded_full()
                    .text_size(px(11.5))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(if is_selected {
                        cx.theme().primary.opacity(0.12)
                    } else {
                        cx.theme().background
                    })
                    .text_color(cx.theme().muted_foreground)
                    .child(tag.clone())
            }));

        let group_id = SharedString::from(format!("snippet-item-group-{id_usize}"));

        div()
            .id(("snippet-item", id_usize))
            .group(group_id.clone())
            .flex()
            .items_start()
            .gap_3()
            .px(px(14.0))
            .py_3()
            .rounded(px(10.0))
            .border_1()
            .border_color(if is_selected {
                cx.theme().primary
            } else {
                cx.theme().border
            })
            .bg(if is_selected {
                cx.theme().primary.opacity(0.06)
            } else {
                cx.theme().popover
            })
            .hover(|s: StyleRefinement| s.border_color(cx.theme().primary.opacity(0.5)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected_snippet_id = Some(id);
                cx.notify();
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .truncate()
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(cx.theme().muted_foreground)
                                    .flex_shrink_0()
                                    .child(format!("{lines} 行")),
                            ),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .font_family("monospace")
                            .truncate()
                            .child(preview),
                    )
                    .when(!tags.is_empty(), |item| item.child(tags_row)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.0))
                    .flex_shrink_0()
                    .when(is_selected, |el| el.opacity(1.0))
                    .when(!is_selected, |el| {
                        el.opacity(0.0)
                            .group_hover(group_id, |s: StyleRefinement| s.opacity(1.0))
                    })
                    .child(
                        Button::new(("snippet-edit", id_usize))
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::File))
                            .tooltip("编辑")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.start_edit(id, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("snippet-delete", id_usize))
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Delete))
                            .tooltip("删除")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.delete_snippet(id, window, cx);
                            })),
                    ),
            )
    }
}

impl Render for CodeSnippet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.search_input_state.is_none() {
            self.initialize(window, cx);
        }

        let filtered = self.filtered_snippets();
        let has_tags = !self.tags.is_empty();

        let list_area: AnyElement = if filtered.is_empty() {
            div()
                .flex_1()
                .min_w_0()
                .h(px(360.0))
                .flex()
                .items_center()
                .justify_center()
                .child(design::hint("暂无代码片段", cx))
                .into_any_element()
        } else {
            div()
                .flex_1()
                .min_w_0()
                .min_h(px(360.0))
                .max_h(px(520.0))
                .flex_col()
                .overflow_y_scrollbar()
                .child(
                    div()
                        .flex_col()
                        .gap_2()
                        .children(filtered.iter().map(|s| self.render_snippet_item(s, cx))),
                )
                .into_any_element()
        };

        let main_content = div()
            .flex()
            .gap_4()
            .flex_1()
            .min_h_0()
            .when(has_tags, |row| row.child(self.render_tag_sidebar(cx)))
            .child(list_area);

        design::page().child(
            design::card(cx)
                .child(
                    design::card_header(IconName::FileText, "代码片段", "", cx).child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("new-snippet")
                                    .primary()
                                    .icon(Icon::new(IconName::Plus))
                                    .tooltip("新建")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.start_add(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("import-snippets")
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .tooltip("导入")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.import_snippets(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("export-snippets")
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .tooltip("导出")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.export_snippets(window, cx);
                                    })),
                            ),
                    ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .pb_3()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(div().flex_1().min_w(px(200.0)).child(
                            if let Some(search_input) = &self.search_input_state {
                                div().child(
                                    Input::new(search_input)
                                        .cleanable(true)
                                        .prefix(Icon::new(IconName::Search)),
                                )
                            } else {
                                div()
                            },
                        ))
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_size(px(12.5))
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} 条", filtered.len())),
                        ),
                )
                .child(main_content),
        )
    }
}

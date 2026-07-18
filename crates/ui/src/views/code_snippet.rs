use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

use crate::config_store::{self, SnippetRecord};

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

pub struct CodeSnippet {
    snippets: Vec<Snippet>,
    tags: Vec<SharedString>,
    selected_tags: Vec<SharedString>,
    selected_snippet_id: Option<i64>,
    is_editing: bool,
    status: String,
    search_input_state: Option<Entity<InputState>>,
    title_input_state: Option<Entity<InputState>>,
    code_input_state: Option<Entity<InputState>>,
    tag_input_state: Option<Entity<InputState>>,
    language_input_state: Option<Entity<InputState>>,
    search_text: SharedString,
    tag_input_text: SharedString,
    current_title: SharedString,
    current_code: SharedString,
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
            is_editing: false,
            status: String::new(),
            search_input_state: None,
            title_input_state: None,
            code_input_state: None,
            tag_input_state: None,
            language_input_state: None,
            search_text: SharedString::default(),
            tag_input_text: SharedString::default(),
            current_title: SharedString::default(),
            current_code: SharedString::default(),
            current_tags: Vec::new(),
            current_language: SharedString::from("text"),
            _subscriptions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let search_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索片段..."));

        let title_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入标题..."));

        let code_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入代码内容...")
                .multi_line(true)
        });

        let tag_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入标签后点击添加..."));

        let language_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("语言（如 rust, python...）"));

        let search_clone = search_input_state.clone();
        let title_clone = title_input_state.clone();
        let code_clone = code_input_state.clone();
        let tag_clone = tag_input_state.clone();
        let language_clone = language_input_state.clone();

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
            cx.subscribe_in(&code_input_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let value = code_clone.read(cx).value();
                        this.current_code = value.clone();
                        cx.notify()
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(&tag_input_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let value = tag_clone.read(cx).value();
                        this.tag_input_text = value.clone();
                        cx.notify()
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(&language_input_state, window, {
                move |this, _, ev: &InputEvent, _window, cx| match ev {
                    InputEvent::Change => {
                        let value = language_clone.read(cx).value();
                        this.current_language = value.clone();
                        cx.notify()
                    }
                    _ => {}
                }
            }),
        ];

        self.search_input_state = Some(search_input_state);
        self.title_input_state = Some(title_input_state);
        self.code_input_state = Some(code_input_state);
        self.tag_input_state = Some(tag_input_state);
        self.language_input_state = Some(language_input_state);
        self._subscriptions = _subscriptions;

        // 从 SQLite 加载已有片段
        self.status = "正在加载片段...".to_string();
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_snippets().await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(records) => {
                        this.snippets = records.into_iter().map(Snippet::from_record).collect();
                        this.update_tags();
                        this.status = format!("已加载 {} 个片段。", this.snippets.len());
                    }
                    Err(err) => {
                        this.status = format!("加载片段失败：{err}");
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

    fn start_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = true;
        self.selected_snippet_id = None;
        self.current_title = SharedString::default();
        self.current_code = SharedString::default();
        self.current_tags = Vec::new();
        self.current_language = SharedString::from("text");

        if let Some(title_input) = &self.title_input_state {
            title_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(code_input) = &self.code_input_state {
            code_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(language_input) = &self.language_input_state {
            language_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::from("text"), window, cx);
            });
        }

        self.open_edit_sheet(window, cx);
    }

    fn start_edit(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == Some(id)) {
            self.selected_snippet_id = Some(id);
            self.is_editing = true;
            self.current_title = snippet.title.clone();
            self.current_code = snippet.code.clone();
            self.current_tags = snippet.tags.clone();
            self.current_language = snippet.language.clone();

            if let Some(title_input) = &self.title_input_state {
                title_input.update(cx, |input_state, cx| {
                    input_state.replace(snippet.title.clone(), window, cx);
                });
            }
            if let Some(code_input) = &self.code_input_state {
                code_input.update(cx, |input_state, cx| {
                    input_state.replace(snippet.code.clone(), window, cx);
                });
            }
            if let Some(tag_input) = &self.tag_input_state {
                tag_input.update(cx, |input_state, cx| {
                    input_state.replace(SharedString::default(), window, cx);
                });
            }
            if let Some(language_input) = &self.language_input_state {
                language_input.update(cx, |input_state, cx| {
                    input_state.replace(snippet.language.clone(), window, cx);
                });
            }

            self.open_edit_sheet(window, cx);
        }
    }

    fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = false;
        self.selected_snippet_id = None;
        self.current_title = SharedString::default();
        self.current_code = SharedString::default();
        self.current_tags = Vec::new();
        self.current_language = SharedString::from("text");

        if let Some(title_input) = &self.title_input_state {
            title_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(code_input) = &self.code_input_state {
            code_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        if let Some(language_input) = &self.language_input_state {
            language_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::from("text"), window, cx);
            });
        }

        window.close_sheet(cx);
    }

    /// 打开底部抽屉编辑表单（对齐 Tauri 的 n-drawer placement="bottom"）
    fn open_edit_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.entity().downgrade();
        let is_edit = self.selected_snippet_id.is_some();
        let title_state = self.title_input_state.clone();
        let code_state = self.code_input_state.clone();
        let tag_state = self.tag_input_state.clone();
        let language_state = self.language_input_state.clone();
        let form_tags = self.current_tags.clone();

        window.open_sheet_at(Placement::Bottom, cx, move |sheet, _, cx| {
            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title(if is_edit {
                    "编辑片段"
                } else {
                    "添加片段"
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        // 标题
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_sm().font_semibold().child("标题"))
                                .child(if let Some(ref ts) = title_state {
                                    div().child(Input::new(ts))
                                } else {
                                    div()
                                }),
                        )
                        // 语言 + 标签
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .w(px(200.0))
                                        .child(div().text_sm().font_semibold().child("语言"))
                                        .child(if let Some(ref ls) = language_state {
                                            div().child(Input::new(ls))
                                        } else {
                                            div()
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .flex_1()
                                        .child(div().text_sm().font_semibold().child("标签"))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(if form_tags.is_empty() {
                                                    "暂无标签，输入后点击 + 添加".to_string()
                                                } else {
                                                    format!(
                                                        "已添加：{}",
                                                        form_tags
                                                            .iter()
                                                            .map(|t| t.as_ref())
                                                            .collect::<Vec<_>>()
                                                            .join(", ")
                                                    )
                                                }),
                                        )
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
                                        ),
                                ),
                        )
                        // 代码内容
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_sm().font_semibold().child("代码内容"))
                                .child(if let Some(ref cs) = code_state {
                                    div().min_h(px(200.0)).child(Input::new(cs))
                                } else {
                                    div()
                                }),
                        )
                        // 操作按钮
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
                                        .primary()
                                        .icon(Icon::new(IconName::Check))
                                        .tooltip("保存")
                                        .on_click(move |_, window, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    this.save_snippet(cx);
                                                    window.close_sheet(cx);
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
        let tag = tag_str.trim();
        if !tag.is_empty() {
            let tag_shared = SharedString::from(tag.to_string());
            if !self.current_tags.contains(&tag_shared) {
                self.current_tags.push(tag_shared);
            }
        }

        if let Some(tag_input) = &self.tag_input_state {
            tag_input.update(cx, |input_state, cx| {
                input_state.replace(SharedString::default(), window, cx);
            });
        }
        self.tag_input_text = SharedString::default();
    }

    fn remove_tag(&mut self, tag: SharedString) {
        self.current_tags.retain(|t| t != &tag);
    }

    fn save_snippet(&mut self, cx: &mut Context<Self>) {
        // 表单验证
        if self.current_title.trim().is_empty() {
            self.status = "标题不能为空".to_string();
            cx.notify();
            return;
        }
        if self.current_code.trim().is_empty() {
            self.status = "代码内容不能为空".to_string();
            cx.notify();
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if let Some(id) = self.selected_snippet_id {
            // 编辑现有片段
            if let Some(snippet) = self.snippets.iter_mut().find(|s| s.id == Some(id)) {
                snippet.title = self.current_title.clone();
                snippet.tags = self.current_tags.clone();
                snippet.code = self.current_code.clone();
                snippet.language = self.current_language.clone();
                snippet.updated_at = now;
                let record = snippet.to_record();
                self.status = "正在更新片段...".to_string();
                cx.notify();

                cx.spawn(async move |this: WeakEntity<Self>, cx| {
                    let result = config_store::update_snippet(id, record).await;
                    let _ = this.update(cx, |this, cx| {
                        this.status = match result {
                            Ok(_) => "片段已更新。".to_string(),
                            Err(err) => format!("更新片段失败：{err}"),
                        };
                        cx.notify();
                    });
                })
                .detach();
            }
        } else {
            // 新建片段
            let new_snippet = Snippet {
                id: None,
                title: self.current_title.clone(),
                tags: self.current_tags.clone(),
                code: self.current_code.clone(),
                language: self.current_language.clone(),
                created_at: now,
                updated_at: now,
            };
            let record = new_snippet.to_record();
            self.status = "正在保存片段...".to_string();
            cx.notify();

            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                let result = config_store::save_snippet(record).await;
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(new_id) => {
                            this.snippets.insert(
                                0,
                                Snippet {
                                    id: Some(new_id),
                                    title: this.current_title.clone(),
                                    tags: this.current_tags.clone(),
                                    code: this.current_code.clone(),
                                    language: this.current_language.clone(),
                                    created_at: now,
                                    updated_at: now,
                                },
                            );
                            this.update_tags();
                            this.status = "片段已保存。".to_string();
                        }
                        Err(err) => {
                            this.status = format!("保存片段失败：{err}");
                        }
                    }
                    this.is_editing = false;
                    this.selected_snippet_id = None;
                    cx.notify();
                });
            })
            .detach();
        }

        self.update_tags();
        self.is_editing = false;
        self.selected_snippet_id = None;
    }

    fn delete_snippet(&mut self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
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
                        .child("确定要删除这个代码片段吗？此操作不可撤销。"),
                )
                .confirm()
                .on_ok(move |_, _, cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.confirm_delete_snippet(id, cx);
                        });
                    }
                    true
                })
        });
    }

    fn confirm_delete_snippet(&mut self, id: i64, cx: &mut Context<Self>) {
        self.status = "正在删除片段...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_snippet(id).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(true) => {
                        this.snippets.retain(|s| s.id != Some(id));
                        if this.selected_snippet_id == Some(id) {
                            this.selected_snippet_id = None;
                            this.is_editing = false;
                        }
                        this.update_tags();
                        this.status = "片段已删除。".to_string();
                    }
                    Ok(false) => {
                        this.status = "未找到要删除的片段。".to_string();
                    }
                    Err(err) => {
                        this.status = format!("删除片段失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn import_snippets(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("导入代码片段")
                .add_filter("JSON", &["json"])
                .pick_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let content = file.read().await;
                let content_str = String::from_utf8_lossy(&content).to_string();
                match serde_json::from_str::<Vec<serde_json::Value>>(&content_str) {
                    Ok(items) => {
                        let mut count = 0;
                        for item in items {
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
                            let tags: Vec<String> = item
                                .get("tags")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let language = item
                                .get("language")
                                .and_then(|v| v.as_str())
                                .unwrap_or("text")
                                .to_string();
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64;
                            let record = SnippetRecord {
                                id: None,
                                title,
                                code,
                                tags,
                                language,
                                created_at: now,
                                updated_at: now,
                            };
                            if config_store::save_snippet(record).await.is_ok() {
                                count += 1;
                            }
                        }
                        let _ = this.update(cx, |this, cx| {
                            this.status = format!("成功导入 {} 个片段，正在刷新列表...", count);
                            cx.notify();
                            // 重新加载列表
                            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                                let result = config_store::load_snippets().await;
                                let _ = this.update(cx, |this, cx| {
                                    match result {
                                        Ok(records) => {
                                            this.snippets = records
                                                .into_iter()
                                                .map(Snippet::from_record)
                                                .collect();
                                            this.update_tags();
                                            this.status =
                                                format!("已加载 {} 个片段。", this.snippets.len());
                                        }
                                        Err(err) => {
                                            this.status = format!("刷新列表失败：{err}");
                                        }
                                    }
                                    cx.notify();
                                });
                            })
                            .detach();
                        });
                    }
                    Err(err) => {
                        let _ = this.update(cx, |this, cx| {
                            this.status = format!("解析 JSON 失败：{err}");
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();
    }

    fn export_snippets(&mut self, cx: &mut Context<Self>) {
        let snippets: Vec<serde_json::Value> = self
            .snippets
            .iter()
            .map(|s| {
                serde_json::json!({
                    "title": s.title.to_string(),
                    "code": s.code.to_string(),
                    "tags": s.tags.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                    "language": s.language.to_string(),
                })
            })
            .collect();

        let json_content = serde_json::to_string_pretty(&snippets).unwrap_or_default();
        let count = snippets.len();
        self.status = format!("正在导出 {} 个片段...", count);
        cx.notify();

        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("导出代码片段")
                .add_filter("JSON", &["json"])
                .set_file_name("snippets.json")
                .save_file()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(file) = task.await {
                let _ = file.write(json_content.as_bytes()).await;
                let _ = this.update(cx, |this, cx| {
                    this.status = format!("已导出 {} 个片段。", count);
                    cx.notify();
                });
            } else {
                let _ = this.update(cx, |this, cx| {
                    this.status = "导出已取消。".to_string();
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn update_tags(&mut self) {
        let mut tag_set = std::collections::HashSet::new();
        for snippet in &self.snippets {
            for tag in &snippet.tags {
                tag_set.insert(tag.clone());
            }
        }
        self.tags = tag_set.into_iter().collect();
    }

    fn copy_code(&self, id: i64, cx: &mut Context<Self>) {
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == Some(id)) {
            let code = snippet.code.to_string();
            if !code.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(code));
            }
        }
    }

    fn render_snippet_item(&self, snippet: &Snippet, cx: &mut Context<Self>) -> Div {
        let id = snippet.id.unwrap_or(0);
        let id_usize = id as usize;
        let title = snippet.title.clone();
        let tags = snippet.tags.clone();
        let code = snippet.code.clone();
        let is_selected = self.selected_snippet_id == Some(id);

        let tags_div = div()
            .flex()
            .flex_wrap()
            .gap_1()
            .children(tags.iter().map(|tag| {
                div()
                    .px_2()
                    .py(px(2.0))
                    .text_xs()
                    .bg(cx.theme().secondary)
                    .rounded_md()
                    .child(tag.clone())
            }));

        let code_preview = if code.len() > 200 {
            SharedString::from(format!("{}...", &code[..200]))
        } else {
            code
        };

        div()
            .p_3()
            .border_1()
            .border_color(if is_selected {
                cx.theme().primary
            } else {
                cx.theme().border
            })
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
                            .justify_between()
                            .child(div().text_sm().font_semibold().child(title))
                            .child(
                                ButtonGroup::new(("snippet-actions", id_usize))
                                    .child(
                                        Button::new(("copy", id_usize))
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制代码")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.copy_code(id, cx);
                                            })),
                                    )
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
                                                this.delete_snippet(id, window, cx);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .when(!tags.is_empty(), |this| this.child(tags_div))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .font_family("monospace")
                            .max_h(px(60.0))
                            .overflow_y_scrollbar()
                            .child(code_preview),
                    ),
            )
    }

    fn render_tag_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let all_tags = self.tags.clone();
        let selected_tags = self.selected_tags.clone();

        let reset_button = if !selected_tags.is_empty() {
            Some(
                Button::new("reset-tags")
                    .xsmall()
                    .child("重置")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reset_tag_filter();
                        cx.notify();
                    })),
            )
        } else {
            None
        };

        let tags_list = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(all_tags.iter().enumerate().map(|(idx, tag)| {
                let is_selected = selected_tags.contains(tag);
                let tag_clone = tag.clone();
                Button::new(("filter-tag", idx))
                    .xsmall()
                    .child(tag.clone())
                    .when(is_selected, |this| this.primary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tag_filter(tag_clone.clone());
                        cx.notify();
                    }))
            }));

        div()
            .w(px(200.0))
            .border_r_1()
            .border_color(cx.theme().border)
            .pr_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_2()
                    .child(div().text_sm().font_semibold().child("标签"))
                    .children(reset_button),
            )
            .child(tags_list)
    }
}

impl Render for CodeSnippet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.search_input_state.is_none() {
            self.initialize(window, cx);
        }

        let filtered = self.filtered_snippets();
        let has_tags = !self.tags.is_empty();

        let tag_sidebar = if has_tags {
            Some(self.render_tag_sidebar(cx))
        } else {
            None
        };

        let snippet_list = if filtered.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(if self.search_text.is_empty() {
                    "暂无代码片段，点击「新建」添加"
                } else {
                    "未找到匹配的代码片段"
                })
        } else {
            div().flex().flex_col().gap_2().children(
                filtered
                    .iter()
                    .map(|snippet| self.render_snippet_item(snippet, cx)),
            )
        };

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

        div().child(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new("new")
                                .primary()
                                .icon(Icon::new(IconName::Plus))
                                .tooltip("新建")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.start_add(window, cx);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("import")
                                .icon(Icon::new(IconName::ArrowUp))
                                .tooltip("导入 JSON")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.import_snippets(cx);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("export")
                                .icon(Icon::new(IconName::ArrowDown))
                                .tooltip("导出 JSON")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.export_snippets(cx);
                                    cx.notify();
                                })),
                        )
                        .child(div().flex_1().child(
                            if let Some(search_input) = &self.search_input_state {
                                div().child(Input::new(search_input))
                            } else {
                                div()
                            },
                        )),
                )
                .children(status_bar)
                .child(
                    div().flex().gap_4().flex_1().children(tag_sidebar).child(
                        div()
                            .flex_1()
                            .min_h(px(400.0))
                            .max_h(px(400.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .p_4()
                            .overflow_y_scrollbar()
                            .child(snippet_list),
                    ),
                ),
        )
    }
}

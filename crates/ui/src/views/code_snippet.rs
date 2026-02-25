use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

pub struct Snippet {
    id: usize,
    title: SharedString,
    tags: Vec<SharedString>,
    code: SharedString,
    created_at: i64,
}

pub struct CodeSnippet {
    snippets: Vec<Snippet>,
    tags: Vec<SharedString>,
    selected_tags: Vec<SharedString>,
    selected_snippet_id: Option<usize>,
    is_editing: bool,
    next_id: usize,
    search_input_state: Option<Entity<InputState>>,
    title_input_state: Option<Entity<InputState>>,
    code_input_state: Option<Entity<InputState>>,
    tag_input_state: Option<Entity<InputState>>,
    search_text: SharedString,
    tag_input_text: SharedString,
    current_title: SharedString,
    current_code: SharedString,
    current_tags: Vec<SharedString>,
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
            next_id: 0,
            search_input_state: None,
            title_input_state: None,
            code_input_state: None,
            tag_input_state: None,
            search_text: SharedString::default(),
            tag_input_text: SharedString::default(),
            current_title: SharedString::default(),
            current_code: SharedString::default(),
            current_tags: Vec::new(),
            _subscriptions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let search_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索片段..."));

        let title_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入标题..."));

        let code_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入代码内容...").multi_line(true));

        let tag_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入标签后点击添加..."));

        let search_clone = search_input_state.clone();
        let title_clone = title_input_state.clone();
        let code_clone = code_input_state.clone();
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
        ];

        self.search_input_state = Some(search_input_state);
        self.title_input_state = Some(title_input_state);
        self.code_input_state = Some(code_input_state);
        self.tag_input_state = Some(tag_input_state);
        self._subscriptions = _subscriptions;
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
    }

    fn start_edit(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
            self.selected_snippet_id = Some(id);
            self.is_editing = true;
            self.current_title = snippet.title.clone();
            self.current_code = snippet.code.clone();
            self.current_tags = snippet.tags.clone();

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
        }
    }

    fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = false;
        self.selected_snippet_id = None;
        self.current_title = SharedString::default();
        self.current_code = SharedString::default();
        self.current_tags = Vec::new();

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

    fn save_snippet(&mut self) {
        if self.current_title.trim().is_empty() {
            return;
        }

        if let Some(id) = self.selected_snippet_id {
            if let Some(snippet) = self.snippets.iter_mut().find(|s| s.id == id) {
                snippet.title = self.current_title.clone();
                snippet.tags = self.current_tags.clone();
                snippet.code = self.current_code.clone();
            }
        } else {
            let new_snippet = Snippet {
                id: self.next_id,
                title: self.current_title.clone(),
                tags: self.current_tags.clone(),
                code: self.current_code.clone(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            };

            self.next_id += 1;
            self.snippets.insert(0, new_snippet);
        }

        self.update_tags();
        self.is_editing = false;
        self.selected_snippet_id = None;
    }

    fn delete_snippet(&mut self, id: usize) {
        self.snippets.retain(|s| s.id != id);
        if self.selected_snippet_id == Some(id) {
            self.selected_snippet_id = None;
            self.is_editing = false;
        }
        self.update_tags();
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

    fn copy_code(&self, id: usize, cx: &mut Context<Self>) {
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
            let code = snippet.code.to_string();
            if !code.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(code));
            }
        }
    }

    fn render_snippet_item(&self, snippet: &Snippet, cx: &mut Context<Self>) -> Div {
        let id = snippet.id;
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
                                ButtonGroup::new(("snippet-actions", id))
                                    .child(
                                        Button::new(("copy", id))
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制代码")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.copy_code(id, cx);
                                            })),
                                    )
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
                                                this.delete_snippet(id);
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

    fn render_edit_form(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let is_edit = self.selected_snippet_id.is_some();
        let form_tags = self.current_tags.clone();

        let tags_display = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(form_tags.iter().enumerate().map(|(idx, tag)| {
                let tag_clone = tag.clone();
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .bg(cx.theme().secondary)
                    .rounded_md()
                    .text_sm()
                    .child(tag.clone())
                    .child(
                        Button::new(("remove-tag", idx))
                            .icon(Icon::new(IconName::Close))
                            .xsmall()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_tag(tag_clone.clone());
                                cx.notify();
                            })),
                    )
            }));

        div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(div().text_lg().font_semibold().child(if is_edit {
                        "编辑片段"
                    } else {
                        "添加片段"
                    }))
                    .child(
                        Button::new("cancel-edit")
                            .icon(Icon::new(IconName::Close))
                            .tooltip("取消")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_edit(window, cx);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_sm().font_semibold().child("标题"))
                                    .child(
                                        if let Some(title_input) = &self.title_input_state {
                                            div().child(Input::new(title_input))
                                        } else {
                                            div()
                                        },
                                    ),
                            )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().font_semibold().child("标签"))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(tags_display)
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            .child(if let Some(tag_input) = &self.tag_input_state {
                                                div().flex_1().child(Input::new(tag_input))
                                            } else {
                                                div()
                                            })
                                            .child(
                                                Button::new("add-tag")
                                                    .icon(Icon::new(IconName::Plus))
                                                    .tooltip("添加标签")
                                                    .on_click(cx.listener(|this, _, window, cx| {
                                                        this.add_tag(window, cx);
                                                        cx.notify();
                                                    })),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().font_semibold().child("代码内容"))
                            .child(if let Some(code_input) = &self.code_input_state {
                                div().min_h(px(150.0)).child(Input::new(code_input))
                            } else {
                                div()
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .mt_2()
                            .child(
                                Button::new("cancel-btn")
                                    .child("取消")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.cancel_edit(window, cx);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("save-btn")
                                    .primary()
                                    .child("保存")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.save_snippet();
                                        cx.notify();
                                    })),
                            ),
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

        let edit_form = if self.is_editing {
            Some(self.render_edit_form(window, cx))
        } else {
            None
        };

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
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(filtered.iter().map(|snippet| self.render_snippet_item(snippet, cx)))
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("代码片段"))
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
                                div()
                                    .flex_1()
                                    .child(
                                        if let Some(search_input) = &self.search_input_state {
                                            div().child(Input::new(search_input))
                                        } else {
                                            div()
                                        },
                                    ),
                            ),
                    )
                    .children(edit_form)
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .flex_1()
                            .children(tag_sidebar)
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
                                    .child(snippet_list),
                            ),
                    ),
            )
    }
}

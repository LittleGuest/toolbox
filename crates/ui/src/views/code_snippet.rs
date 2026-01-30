use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

pub struct CodeSnippet {
    search: String,
    selected_tags: Vec<String>,
    snippets: Vec<Snippet>,
    tags: Vec<String>,
    selected_snippet_id: Option<usize>,
    show_add_dialog: bool,
    current_form: SnippetForm,
}

#[derive(Clone)]
pub struct Snippet {
    id: usize,
    title: String,
    language: String,
    tags: Vec<String>,
    code: String,
}

#[derive(Clone)]
pub struct SnippetForm {
    id: Option<usize>,
    title: String,
    language: String,
    tags: Vec<String>,
    code: String,
}

impl CodeSnippet {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            selected_tags: Vec::new(),
            snippets: Vec::new(),
            tags: Vec::new(),
            selected_snippet_id: None,
            show_add_dialog: false,
            current_form: SnippetForm {
                id: None,
                title: String::new(),
                language: String::new(),
                tags: Vec::new(),
                code: String::new(),
            },
        }
    }

    fn filtered_snippets(&self) -> Vec<Snippet> {
        let mut result = self.snippets.clone();
        
        if !self.selected_tags.is_empty() {
            result = result
                .into_iter()
                .filter(|snippet| {
                    self.selected_tags.iter().all(|tag| snippet.tags.contains(tag))
                })
                .collect();
        }
        
        if !self.search.is_empty() {
            let query = self.search.to_lowercase();
            result = result
                .into_iter()
                .filter(|snippet| {
                    snippet.title.to_lowercase().contains(&query) || 
                    snippet.code.to_lowercase().contains(&query)
                })
                .collect();
        }
        
        result
    }

    fn select_snippet(&mut self, id: usize) {
        self.selected_snippet_id = Some(id);
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
            self.current_form = SnippetForm {
                id: Some(snippet.id),
                title: snippet.title.clone(),
                language: snippet.language.clone(),
                tags: snippet.tags.clone(),
                code: snippet.code.clone(),
            };
        }
    }

    fn toggle_tag(&mut self, tag: String) {
        if let Some(pos) = self.selected_tags.iter().position(|t| t == &tag) {
            self.selected_tags.remove(pos);
        } else {
            self.selected_tags.push(tag);
        }
    }

    fn reset_tags(&mut self) {
        self.selected_tags.clear();
    }

    fn show_add_dialog(&mut self) {
        self.current_form = SnippetForm {
            id: None,
            title: String::new(),
            language: String::new(),
            tags: Vec::new(),
            code: String::new(),
        };
        self.show_add_dialog = true;
    }

    fn save_snippet(&mut self) {
        if self.current_form.title.is_empty() {
            return;
        }
        
        if let Some(id) = self.current_form.id {
            if let Some(snippet) = self.snippets.iter_mut().find(|s| s.id == id) {
                snippet.title = self.current_form.title.clone();
                snippet.language = self.current_form.language.clone();
                snippet.tags = self.current_form.tags.clone();
                snippet.code = self.current_form.code.clone();
            }
        } else {
            let new_id = self.snippets.len();
            self.snippets.push(Snippet {
                id: new_id,
                title: self.current_form.title.clone(),
                language: self.current_form.language.clone(),
                tags: self.current_form.tags.clone(),
                code: self.current_form.code.clone(),
            });
        }
        
        self.update_tags();
        self.show_add_dialog = false;
    }

    fn delete_snippet(&mut self, id: usize) {
        self.snippets.retain(|s| s.id != id);
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
}

impl Render for CodeSnippet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filtered = self.filtered_snippets();
        let tags = self.tags.clone();
        let selected_tags = self.selected_tags.clone();
        let search_text = if self.search.is_empty() {
            "搜索片段...".to_string()
        } else {
            self.search.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("代码片段")
            )
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
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("新建")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("导入")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("导出")
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
                                    .child(search_text)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .flex_1()
                            .child(
                                div()
                                    .w(px(200.0))
                                    .border_r_1()
                                    .border_color(cx.theme().border)
                                    .pr_4()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_semibold()
                                            .mb_2()
                                            .child("标签")
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .gap_2()
                                            .children(
                                                tags.iter().map(|tag| {
                                                    let is_selected = selected_tags.contains(tag);
                                                    let tag_clone = tag.clone();
                                                    div()
                                                        .px_2()
                                                        .py_1()
                                                        .text_xs()
                                                        .border_1()
                                                        .border_color(if is_selected {
                                                            cx.theme().primary
                                                        } else {
                                                            cx.theme().border
                                                        })
                                                        .rounded_md()
                                                        .cursor_pointer()
                                                        .child(tag_clone)
                                                })
                                            )
                                    )
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
                                    .child(if filtered.is_empty() {
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("无数制")
                                    } else {
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(
                                                filtered.iter().map(|snippet| {
                                                    let is_selected = self.selected_snippet_id == Some(snippet.id);
                                                    let id = snippet.id;
                                                    let title = snippet.title.clone();
                                                    div()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(if is_selected {
                                                            cx.theme().primary
                                                        } else {
                                                            cx.theme().border
                                                        })
                                                        .rounded_md()
                                                        .cursor_pointer()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_semibold()
                                                                .child(title)
                                                        )
                                                })
                                            )
                                    })
                            )
                    )
            )
    }
}

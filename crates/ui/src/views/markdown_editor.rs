use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    scroll::ScrollableElement,
    *,
};

/// 可复用的 Markdown 编辑面板（工具栏 + 编辑/预览分栏）。
///
/// 对齐 Tauri 里两处共用的 `md-editor-v3` 的 `<MdEditor>`：
/// - `src/views/text/Markdown.vue`（独立「Markdown 编辑器」页面）
/// - `src/views/snippet/CodeSnippet.vue` 抽屉里的内容字段
pub struct MarkdownPane {
    content: String,
    show_preview: bool,
    /// 是否显示「保存为 .md」：片段抽屉里落库由外层「保存」负责，故隐藏
    file_actions: bool,
    /// 编辑器高度 = 视口高度 - height_offset，且不低于 min_height
    height_offset: f32,
    min_height: f32,
    status: String,
    error: String,
    input_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

fn find_closing_bracket(chars: &[char], open_pos: usize) -> Option<usize> {
    for i in (open_pos + 1)..chars.len() {
        if chars[i] == ']' {
            return Some(i);
        }
        if chars[i] == '[' {
            return None;
        }
    }
    None
}

fn find_closing_paren(chars: &[char], open_pos: usize) -> Option<usize> {
    for i in (open_pos + 1)..chars.len() {
        if chars[i] == ')' {
            return Some(i);
        }
        if chars[i] == '(' {
            return None;
        }
    }
    None
}

fn find_closing_marker(
    chars: &[char],
    start: usize,
    marker1: char,
    marker2: impl Into<Option<char>>,
) -> Option<usize> {
    let m2 = marker2.into();
    for i in start..chars.len() {
        if chars[i] == marker1 {
            if let Some(m) = m2 {
                if i + 1 < chars.len() && chars[i + 1] == m {
                    return Some(i);
                }
            } else {
                return Some(i);
            }
        }
    }
    None
}

fn find_closing_backtick(chars: &[char], start: usize) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == '`' {
            return Some(i);
        }
    }
    None
}

impl MarkdownPane {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 对齐 md-editor-v3 的默认 placeholder「请输入内容...」
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("请输入内容...")
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.content = value.to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            content: String::new(),
            show_preview: true,
            file_actions: true,
            height_offset: 160.0,
            min_height: 480.0,
            status: String::new(),
            error: String::new(),
            input_state,
            _subscriptions,
        }
    }

    /// 是否显示「保存为 .md」按钮（片段抽屉传 false）
    pub fn file_actions(mut self, on: bool) -> Self {
        self.file_actions = on;
        self
    }

    /// 编辑器高度策略（对齐 Tauri：页面 `calc(100vh - 160px)`、片段抽屉 `calc(100vh - 290px)`）
    pub fn editor_height(mut self, offset: f32, min_height: f32) -> Self {
        self.height_offset = offset;
        self.min_height = min_height;
        self
    }

    /// 当前 Markdown 原文（片段抽屉保存时读取）
    pub fn content(&self) -> &str {
        &self.content
    }

    /// 覆写内容（编辑已有片段时预填）
    pub fn set_content(&mut self, value: String, window: &mut Window, cx: &mut Context<Self>) {
        self.content = value.clone();
        self.input_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
        cx.notify();
    }

    fn insert_markdown(
        &mut self,
        prefix: &str,
        suffix: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.content.push_str(prefix);
        self.content.push_str(suffix);
        self.input_state.update(cx, |state, cx| {
            state.set_value(self.content.clone(), window, cx);
        });
        cx.notify();
    }

    fn add_heading1(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("# ", "\n\n", window, cx);
    }

    fn add_heading2(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("## ", "\n\n", window, cx);
    }

    fn add_heading3(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("### ", "\n\n", window, cx);
    }

    fn add_bold(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("**", "**", window, cx);
    }

    fn add_underline(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("<u>", "</u>", window, cx);
    }

    fn add_italic(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("*", "*", window, cx);
    }

    fn add_strikethrough(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("~~", "~~", window, cx);
    }

    fn add_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("`", "`", window, cx);
    }

    fn add_code_block(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("```\n", "\n```\n\n", window, cx);
    }

    fn add_link(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("[", "](url)", window, cx);
    }

    fn add_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("![", "](url)", window, cx);
    }

    fn add_quote(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("> ", "\n\n", window, cx);
    }

    fn add_bullet_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("- ", "\n", window, cx);
    }

    fn add_numbered_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("1. ", "\n", window, cx);
    }

    fn add_horizontal_rule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.insert_markdown("\n---\n\n", "", window, cx);
    }

    fn add_table(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let table = "| 列1 | 列2 | 列3 |\n|-----|-----|-----|\n| 内容 | 内容 | 内容 |\n\n";
        self.content.push_str(table);
        self.input_state.update(cx, |state, cx| {
            state.set_value(self.content.clone(), window, cx);
        });
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.content.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.content = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if !self.content.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.content.clone()));
            self.status = "复制成功".to_string();
            self.error.clear();
            cx.notify();
        } else {
            self.error = "内容为空".to_string();
            self.status.clear();
            cx.notify();
        }
    }

    /// 保存为 .md 文件（对应 md-editor-v3 工具栏的保存动作）
    fn save_to_file(&mut self, cx: &mut Context<Self>) {
        if self.content.is_empty() {
            self.error = "内容为空，无法保存".to_string();
            self.status.clear();
            cx.notify();
            return;
        }
        self.error.clear();
        self.status.clear();
        cx.notify();

        let content = self.content.clone();
        let task = cx.background_executor().spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("保存 Markdown")
                .add_filter("Markdown", &["md", "markdown", "txt"])
                .set_file_name("untitled.md")
                .save_file()
                .await
                .map(|f| f.path().to_string_lossy().to_string())
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            if let Some(path) = task.await {
                let result = std::fs::write(&path, content);
                let _ = this.update(cx, |this, cx| match result {
                    Ok(()) => {
                        this.status = format!("已保存到 {}", path);
                        this.error.clear();
                        cx.notify();
                    }
                    Err(e) => {
                        this.error = e.to_string();
                        this.status.clear();
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    fn render_preview(content: &str, cx: &mut Context<Self>) -> Div {
        let lines: Vec<&str> = content.lines().collect();
        let mut elements = Vec::new();

        let mut in_code_block = false;
        let mut code_content = String::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with("```") {
                if in_code_block {
                    elements.push(
                        div()
                            .bg(cx.theme().muted)
                            .rounded_lg()
                            .p_3()
                            .my_2()
                            .font_family("monospace")
                            .text_sm()
                            .child(code_content.clone()),
                    );
                    code_content.clear();
                    in_code_block = false;
                } else {
                    in_code_block = true;
                }
                i += 1;
                continue;
            }

            if in_code_block {
                if !code_content.is_empty() {
                    code_content.push('\n');
                }
                code_content.push_str(line);
                i += 1;
                continue;
            }

            if line.starts_with("# ") {
                let text = line.strip_prefix("# ").unwrap_or(line).to_string();
                elements.push(div().text_2xl().font_bold().mt_4().mb_2().child(text));
            } else if line.starts_with("## ") {
                let text = line.strip_prefix("## ").unwrap_or(line).to_string();
                elements.push(div().text_xl().font_bold().mt_3().mb_2().child(text));
            } else if line.starts_with("### ") {
                let text = line.strip_prefix("### ").unwrap_or(line).to_string();
                elements.push(div().text_lg().font_semibold().mt_2().mb_1().child(text));
            } else if line.starts_with("> ") {
                let text = line.strip_prefix("> ").unwrap_or(line).to_string();
                elements.push(
                    div()
                        .border_l_4()
                        .border_color(cx.theme().primary)
                        .pl_3()
                        .py_1()
                        .my_1()
                        .bg(cx.theme().muted)
                        .italic()
                        .child(text),
                );
            } else if line.starts_with("- ") || line.starts_with("* ") {
                let text = line[2..].to_string();
                elements.push(
                    div()
                        .flex()
                        .gap_2()
                        .py_0p5()
                        .child(div().child("•"))
                        .child(div().child(text)),
                );
            } else if line.starts_with("---") || line.starts_with("***") {
                elements.push(div().border_t_1().border_color(cx.theme().border).my_3());
            } else if line.starts_with("|") {
                let mut table_lines = vec![line];
                let mut j = i + 1;
                while j < lines.len() && lines[j].starts_with("|") {
                    table_lines.push(lines[j]);
                    j += 1;
                }
                i = j - 1;

                let table_div = div().w_full().my_2();
                let mut table_content = Vec::new();

                for (idx, tbl_line) in table_lines.iter().enumerate() {
                    let cells: Vec<&str> =
                        tbl_line.split('|').filter(|c| !c.is_empty()).collect();

                    if idx == 1 && cells.iter().all(|c| c.trim().matches('-').count() > 0) {
                        continue;
                    }

                    let row = div()
                        .flex()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .children(cells.iter().map(|cell| {
                            let cell_text = cell.trim().to_string();
                            div()
                                .flex_1()
                                .p_2()
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(cell_text)
                        }));
                    table_content.push(row);
                }

                elements.push(table_div.children(table_content));
            } else if !line.is_empty() {
                let rendered_line = Self::render_inline(line, cx);
                elements.push(div().py_0p5().child(rendered_line));
            }

            i += 1;
        }

        div().flex().flex_col().children(elements)
    }

    fn render_inline(text: &str, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let mut pos = 0;
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut span_index: usize = 0;
        let mut children: Vec<AnyElement> = Vec::new();

        while pos < len {
            if chars[pos] == '!' && pos + 1 < len && chars[pos + 1] == '[' {
                if let Some(end_bracket) = find_closing_bracket(&chars, pos + 1) {
                    let alt_start = pos + 2;
                    let alt: String = chars[alt_start..end_bracket].iter().collect();
                    if end_bracket + 1 < len && chars[end_bracket + 1] == '(' {
                        if let Some(end_paren) = find_closing_paren(&chars, end_bracket + 1) {
                            let url: String =
                                chars[end_bracket + 2..end_paren].iter().collect();
                            let url_for_tooltip = url.clone();
                            let idx = span_index;
                            span_index += 1;
                            children.push(
                                div()
                                    .id(ElementId::Name(format!("img-{idx}").into()))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("[img: {alt}]"))
                                    .tooltip(move |_, cx| {
                                        cx.new(|_| {
                                            gpui_kit::component::tooltip::Tooltip::new(
                                                url_for_tooltip.clone(),
                                            )
                                        })
                                        .into()
                                    })
                                    .into_any_element(),
                            );
                            pos = end_paren + 1;
                            continue;
                        }
                    }
                }
            }

            if chars[pos] == '[' {
                if let Some(end_bracket) = find_closing_bracket(&chars, pos) {
                    let link_text: String = chars[pos + 1..end_bracket].iter().collect();
                    if end_bracket + 1 < len && chars[end_bracket + 1] == '(' {
                        if let Some(end_paren) = find_closing_paren(&chars, end_bracket + 1) {
                            let url: String =
                                chars[end_bracket + 2..end_paren].iter().collect();
                            let url_for_tooltip = url.clone();
                            let idx = span_index;
                            span_index += 1;
                            children.push(
                                div()
                                    .id(ElementId::Name(format!("link-{idx}").into()))
                                    .flex()
                                    .text_color(theme.primary)
                                    .underline()
                                    .child(link_text)
                                    .tooltip(move |_, cx| {
                                        cx.new(|_| {
                                            gpui_kit::component::tooltip::Tooltip::new(
                                                url_for_tooltip.clone(),
                                            )
                                        })
                                        .into()
                                    })
                                    .into_any_element(),
                            );
                            pos = end_paren + 1;
                            continue;
                        }
                    }
                }
            }

            if chars[pos] == '*' && pos + 1 < len && chars[pos + 1] == '*' {
                if let Some(end) = find_closing_marker(&chars, pos + 2, '*', '*') {
                    let bold_text: String = chars[pos + 2..end].iter().collect();
                    children.push(div().font_bold().child(bold_text).into_any_element());
                    pos = end + 2;
                    continue;
                }
            }

            if chars[pos] == '*' {
                if let Some(end) = find_closing_marker(&chars, pos + 1, '*', None) {
                    let italic_text: String = chars[pos + 1..end].iter().collect();
                    children.push(div().italic().child(italic_text).into_any_element());
                    pos = end + 1;
                    continue;
                }
            }

            if chars[pos] == '~' && pos + 1 < len && chars[pos + 1] == '~' {
                if let Some(end) = find_closing_marker(&chars, pos + 2, '~', '~') {
                    let strike_text: String = chars[pos + 2..end].iter().collect();
                    children.push(
                        div()
                            .line_through()
                            .child(strike_text)
                            .into_any_element(),
                    );
                    pos = end + 2;
                    continue;
                }
            }

            if chars[pos] == '`' {
                if let Some(end) = find_closing_backtick(&chars, pos + 1) {
                    let code_text: String = chars[pos + 1..end].iter().collect();
                    let muted = theme.muted;
                    children.push(
                        div()
                            .flex()
                            .font_family("monospace")
                            .text_sm()
                            .bg(muted)
                            .rounded_sm()
                            .px_1()
                            .child(code_text)
                            .into_any_element(),
                    );
                    pos = end + 1;
                    continue;
                }
            }

            let start = pos;
            while pos < len {
                let c = chars[pos];
                if c == '!' || c == '[' || c == '*' || c == '~' || c == '`' {
                    break;
                }
                pos += 1;
            }
            if pos > start {
                let plain: String = chars[start..pos].iter().collect();
                children.push(div().child(plain).into_any_element());
            }

            if pos == start && pos < len {
                let ch = chars[pos];
                children.push(div().child(ch.to_string()).into_any_element());
                pos += 1;
            }
        }

        div().flex().flex_wrap().children(children)
    }
}

impl Render for MarkdownPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.content.clone();
        let status = self.status.clone();
        let error = self.error.clone();
        let show_preview = self.show_preview;
        let file_actions = self.file_actions;
        // 对齐 Tauri：`height: calc(100vh - Npx)`，并保留一个下限避免窗口过矮时挤扁
        let editor_h = (window.viewport_size().height - px(self.height_offset))
            .max(px(self.min_height));
        let char_count = content.chars().count();
        let line_count = if content.is_empty() {
            0
        } else {
            content.lines().count()
        };

        // 工具栏（对齐 md-editor-v3：加粗/下划线/斜体/删除线 · 标题 · 引用/列表 · 代码 · 链接/图片 · 表格/分割线 · 操作）
        let toolbar = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .pb_3()
            .mb_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("md-h1")
                    .compact()
                    .label("H1")
                    .tooltip("一级标题")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.add_heading1(window, cx);
                    })),
            )
            .child(
                Button::new("md-h2")
                    .compact()
                    .label("H2")
                    .tooltip("二级标题")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.add_heading2(window, cx);
                    })),
            )
            .child(
                Button::new("md-h3")
                    .compact()
                    .label("H3")
                    .tooltip("三级标题")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.add_heading3(window, cx);
                    })),
            )
            .child(
                ButtonGroup::new("md-format-group")
                    .child(
                        Button::new("md-bold")
                            .compact()
                            .label("B")
                            .font_bold()
                            .tooltip("粗体")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_bold(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-underline")
                            .compact()
                            .label("U")
                            .underline()
                            .tooltip("下划线")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_underline(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-italic")
                            .compact()
                            .label("I")
                            .italic()
                            .tooltip("斜体")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_italic(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-strike")
                            .compact()
                            .label("S")
                            .line_through()
                            .tooltip("删除线")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_strikethrough(window, cx);
                            })),
                    ),
            )
            .child(
                ButtonGroup::new("md-quote-group")
                    .child(
                        Button::new("md-quote")
                            .compact()
                            .icon(Icon::new(IconName::BookOpen))
                            .tooltip("引用")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_quote(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-ul")
                            .compact()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("无序列表")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_bullet_list(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-ol")
                            .compact()
                            .icon(Icon::new(IconName::SortAscending))
                            .tooltip("有序列表")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_numbered_list(window, cx);
                            })),
                    ),
            )
            .child(
                ButtonGroup::new("md-code-group")
                    .child(
                        Button::new("md-code")
                            .compact()
                            .icon(Icon::new(IconName::SquareTerminal))
                            .tooltip("行内代码")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_code(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-code-block")
                            .compact()
                            .icon(Icon::new(IconName::File))
                            .tooltip("代码块")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_code_block(window, cx);
                            })),
                    ),
            )
            .child(
                ButtonGroup::new("md-link-group")
                    .child(
                        Button::new("md-link")
                            .compact()
                            .icon(Icon::new(IconName::ExternalLink))
                            .tooltip("链接")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_link(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-image")
                            .compact()
                            .icon(Icon::new(IconName::Frame))
                            .tooltip("图片")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_image(window, cx);
                            })),
                    ),
            )
            .child(
                ButtonGroup::new("md-extra-group")
                    .child(
                        Button::new("md-table")
                            .compact()
                            .icon(Icon::new(IconName::LayoutDashboard))
                            .tooltip("表格")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_table(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-hr")
                            .compact()
                            .icon(Icon::new(IconName::Minus))
                            .tooltip("分割线")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_horizontal_rule(window, cx);
                            })),
                    ),
            )
            .child(
                ButtonGroup::new("md-action-group")
                    .child(
                        Button::new("md-paste")
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste(window, cx);
                            })),
                    )
                    .child(
                        Button::new("md-copy")
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy(cx);
                            })),
                    )
                    // 「保存为 .md」只在独立 Markdown 页面出现；
                    // 片段抽屉里由外层「保存」按钮负责落库，避免出现无意义的落盘入口
                    .when(file_actions, |group| {
                        group.child(
                            Button::new("md-save")
                                .compact()
                                .label("保存")
                                .tooltip("保存为 .md 文件")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save_to_file(cx);
                                })),
                        )
                    })
                    .child(
                        Button::new("md-preview")
                            .compact()
                            .icon(Icon::new(if show_preview {
                                IconName::EyeOff
                            } else {
                                IconName::Eye
                            }))
                            .tooltip(if show_preview { "隐藏预览" } else { "显示预览" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_preview = !this.show_preview;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("md-clear")
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清空")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear(window, cx);
                            })),
                    ),
            );

        // 编辑 / 预览 分栏（md-editor-v3 单卡片双栏）
        let editor_col = div()
            .flex_col()
            .gap_1p5()
            .min_w_0()
            .child(design::editor_label("编辑", cx))
            .child(
                Textarea::new(&self.input_state)
                    .h(editor_h)
                    .font_family("monospace"),
            );

        let preview_col = div()
            .flex_col()
            .gap_1p5()
            .min_w_0()
            .border_l_1()
            .border_color(cx.theme().border)
            .pl_4()
            .child(design::editor_label("预览", cx))
            .child(
                div()
                    .h(editor_h)
                    .overflow_y_scrollbar()
                    .child(if content.is_empty() {
                        design::hint("预览将显示在这里...", cx)
                    } else {
                        Self::render_preview(&content, cx)
                    }),
            );

        let body = if show_preview {
            div()
                .grid()
                .grid_cols(2)
                .gap_4()
                .child(editor_col)
                .child(preview_col)
        } else {
            editor_col
        };

        // 面板本体：不含 page/card 外框，方便被页面或抽屉直接嵌入
        div()
            .flex()
            .flex_col()
            .w_full()
            .child(toolbar)
            .when(!error.is_empty(), |this| {
                this.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                        .child(error),
                )
            })
            .when(!status.is_empty(), |this| {
                this.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::OK_GREEN)))
                        .child(status),
                )
            })
            .child(body)
            .child(
                div()
                    .mt_2()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("支持标题、粗体、斜体、删除线、引用、列表、代码、链接、图片、表格与分割线"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .font_family("monospace")
                            .child(format!("字符 {} · 行数 {}", char_count, line_count)),
                    ),
            )
    }
}

/// 「Markdown 编辑器」页面 —— 对齐 Tauri `src/views/text/Markdown.vue`：`tb-page > tb-card > MdEditor`
pub struct MarkdownEditor {
    pane: Option<Entity<MarkdownPane>>,
}

impl MarkdownEditor {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self { pane: None }
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pane.is_none() {
            // 对齐 Tauri `.md-editor { height: calc(100vh - 160px); min-height: 480px; }`
            self.pane = Some(cx.new(|cx| {
                MarkdownPane::new(window, cx).editor_height(160.0, 480.0)
            }));
        }

        design::page().child(design::card(cx).child(div().children(self.pane.clone())))
    }
}

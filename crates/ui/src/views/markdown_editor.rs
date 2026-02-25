use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

pub struct MarkdownEditor {
    content: String,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl MarkdownEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入 Markdown 内容...")
                .multi_line(true)
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
            input_state,
            _subscriptions,
        }
    }

    fn insert_markdown(&mut self, prefix: &str, suffix: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.content.push_str(prefix);
        self.content.push_str(suffix);
        self.input_state.update(cx, |state, cx| {
            state.set_value(self.content.clone(), window, cx);
        });
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
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.content.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.content = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if !self.content.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.content.clone()));
        }
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
                elements.push(
                    div()
                        .text_2xl()
                        .font_bold()
                        .mt_4()
                        .mb_2()
                        .child(text),
                );
            } else if line.starts_with("## ") {
                let text = line.strip_prefix("## ").unwrap_or(line).to_string();
                elements.push(
                    div()
                        .text_xl()
                        .font_bold()
                        .mt_3()
                        .mb_2()
                        .child(text),
                );
            } else if line.starts_with("### ") {
                let text = line.strip_prefix("### ").unwrap_or(line).to_string();
                elements.push(
                    div()
                        .text_lg()
                        .font_semibold()
                        .mt_2()
                        .mb_1()
                        .child(text),
                );
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
                    let cells: Vec<&str> = tbl_line
                        .split('|')
                        .filter(|c| !c.is_empty())
                        .collect();

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
                let rendered_line = Self::render_inline(line);
                elements.push(div().py_0p5().child(rendered_line));
            }

            i += 1;
        }

        div().flex().flex_col().children(elements)
    }

    fn render_inline(text: &str) -> String {
        text.to_string()
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.content.clone();

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("Markdown 编辑器"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                ButtonGroup::new("heading-buttons")
                                    .child(
                                        Button::new("h1")
                                            .child("H1")
                                            .tooltip("标题1")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_heading1(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("h2")
                                            .child("H2")
                                            .tooltip("标题2")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_heading2(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("h3")
                                            .child("H3")
                                            .tooltip("标题3")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_heading3(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("format-buttons")
                                    .child(
                                        Button::new("bold")
                                            .icon(Icon::new(IconName::Asterisk))
                                            .tooltip("粗体")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_bold(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("italic")
                                            .icon(Icon::new(IconName::Asterisk))
                                            .tooltip("斜体")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_italic(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("strikethrough")
                                            .child("~~")
                                            .tooltip("删除线")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_strikethrough(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("code-buttons")
                                    .child(
                                        Button::new("code")
                                            .child("</>")
                                            .tooltip("行内代码")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_code(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("code-block")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("代码块")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_code_block(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("link-buttons")
                                    .child(
                                        Button::new("link")
                                            .icon(Icon::new(IconName::ArrowRight))
                                            .tooltip("链接")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_link(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("image")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("图片")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_image(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("list-buttons")
                                    .child(
                                        Button::new("quote")
                                            .child("❝")
                                            .tooltip("引用")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_quote(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("bullet-list")
                                            .icon(Icon::new(IconName::Plus))
                                            .tooltip("无序列表")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_bullet_list(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("numbered-list")
                                            .child("1.")
                                            .tooltip("有序列表")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_numbered_list(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("extra-buttons")
                                    .child(
                                        Button::new("table")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("表格")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_table(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("hr")
                                            .child("—")
                                            .tooltip("分割线")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_horizontal_rule(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                ButtonGroup::new("action-buttons")
                                    .child(
                                        Button::new("paste")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear")
                                            .icon(Icon::new(IconName::Delete))
                                            .tooltip("清空")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().text_sm().font_medium().child("编辑"))
                                    .child(Input::new(&self.input_state).h(px(400.0))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().text_sm().font_medium().child("预览"))
                                    .child(
                                        div()
                                            .h(px(400.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_lg()
                                            .p_4()
                                            .overflow_y_scrollbar()
                                            .child(if content.is_empty() {
                                                div()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("预览将显示在这里...")
                                            } else {
                                                Self::render_preview(&content, cx)
                                            }),
                                    ),
                            ),
                    ),
            )
    }
}

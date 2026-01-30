use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};

pub struct MarkdownEditor {
    content: String,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn clear(&mut self) {
        self.content.clear();
    }

    fn add_markdown_element(&mut self, element: &str) {
        self.content.push_str(element);
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.content.clone();
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("Markdown 编辑器")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("h1")
                                    .label("H1")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("# 标题\n\n");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("h2")
                                    .label("H2")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("## 标题\n\n");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("bold")
                                    .label("粗体")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("**粗体**");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("italic")
                                    .label("斜体")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("*斜体*");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("code")
                                    .label("代码")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("`代码`");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("link")
                                    .label("链接")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("[链接文本](https://example.com)");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("image")
                                    .label("图片")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("![图片描述](https://example.com/image.jpg)");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("list")
                                    .label("列表")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_markdown_element("- 项目1\n- 项目2\n\n");
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("clear")
                                    .label("清空")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear();
                                        cx.notify();
                                    }))
                            )
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
                                    .child(
                                        div()
                                            .text_sm()
                                            .child("编辑")
                                    )
                                    .child(
                                        div()
                                            .min_h(px(400.0))
                                            .max_h(px(400.0))
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .rounded_lg()
                                            .p_2()
                                            .overflow_y_scrollbar()
                                            .text_sm()
                                            .font_family("monospace")
                                            .child(if self.content.is_empty() { "请输入Markdown内容...".to_string() } else { self.content.clone() })
                                    )
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .child("预览")
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
                                            .child(
                                                if self.content.is_empty() {
                                                    div().child("预览将显示在这里...")
                                                } else {
                                                    div().child(self.content.clone())
                                                }
                                            )
                                    )
                            )
                    )
            )
    }
}
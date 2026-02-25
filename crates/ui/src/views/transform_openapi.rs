use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct TransformOpenapi {
    api_data: String,
    output_type: String,
    error: String,
    output_type_state: Entity<SelectState<Vec<String>>>,
    api_data_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl TransformOpenapi {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let output_type_items = vec![
            "JSON".to_string(),
            "YAML".to_string(),
            "Markdown".to_string(),
            "HTML".to_string(),
        ];

        let output_type_state = cx.new(|cx| {
            let mut state = SelectState::new(output_type_items, None, window, cx);
            state.set_selected_value(&"JSON".to_string(), window, cx);
            state
        });
        let api_data_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入或粘贴 OpenAPI 数据...")
                .multi_line(true)
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &output_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.output_type = match value.as_str() {
                            "JSON" => "json",
                            "YAML" => "yaml",
                            "Markdown" => "markdown",
                            "HTML" => "html",
                            _ => "json",
                        }.to_string();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&api_data_state, window, {
                let api_data_state = api_data_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = api_data_state.read(cx).value();
                    this.api_data = value.to_string();
                    cx.notify();
                }
            }),
        ];

        Self {
            api_data: String::new(),
            output_type: "json".to_string(),
            error: String::new(),
            output_type_state,
            api_data_state,
            _subscriptions,
        }
    }

    fn convert_to_format(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.api_data.trim().is_empty() {
            self.error = "请输入 OpenAPI 数据".to_string();
            cx.notify();
            return;
        }

        self.error.clear();

        let output = match self.output_type.as_str() {
            "json" => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.api_data) {
                    serde_json::to_string_pretty(&json).unwrap_or_else(|_| self.api_data.clone())
                } else {
                    self.error = "无效的 JSON 格式".to_string();
                    self.api_data.clone()
                }
            }
            "yaml" => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.api_data) {
                    base::cffc(2, "json", "yaml", &serde_json::to_string(&json).unwrap_or_default())
                        .unwrap_or_else(|e| {
                            self.error = e.to_string();
                            self.api_data.clone()
                        })
                } else {
                    self.error = "无效的 JSON 格式".to_string();
                    self.api_data.clone()
                }
            }
            "markdown" => {
                self.generate_markdown()
            }
            "html" => {
                self.generate_html()
            }
            _ => self.api_data.clone(),
        };

        self.api_data = output.clone();
        self.api_data_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
        cx.notify();
    }

    fn generate_markdown(&self) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.api_data) {
            let mut md = String::new();

            if let Some(info) = json.get("info") {
                if let Some(title) = info.get("title").and_then(|t| t.as_str()) {
                    md.push_str(&format!("# {}\n\n", title));
                }
                if let Some(desc) = info.get("description").and_then(|d| d.as_str()) {
                    md.push_str(&format!("{}\n\n", desc));
                }
                if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
                    md.push_str(&format!("**版本:** {}\n\n", version));
                }
            }

            if let Some(paths) = json.get("paths").and_then(|p| p.as_object()) {
                md.push_str("## 接口列表\n\n");
                for (path, methods) in paths {
                    md.push_str(&format!("### `{}`\n\n", path));
                    if let Some(methods_obj) = methods.as_object() {
                        for (method, details) in methods_obj {
                            md.push_str(&format!("**{}**\n\n", method.to_uppercase()));
                            if let Some(summary) = details.get("summary").and_then(|s| s.as_str()) {
                                md.push_str(&format!("{}\n\n", summary));
                            }
                            if let Some(desc) = details.get("description").and_then(|d| d.as_str()) {
                                md.push_str(&format!("*{}*\n\n", desc));
                            }
                        }
                    }
                }
            }

            md
        } else {
            self.api_data.clone()
        }
    }

    fn generate_html(&self) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.api_data) {
            let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>OpenAPI 文档</title>\n</head>\n<body>\n");

            if let Some(info) = json.get("info") {
                if let Some(title) = info.get("title").and_then(|t| t.as_str()) {
                    html.push_str(&format!("<h1>{}</h1>\n", title));
                }
                if let Some(desc) = info.get("description").and_then(|d| d.as_str()) {
                    html.push_str(&format!("<p>{}</p>\n", desc));
                }
                if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
                    html.push_str(&format!("<p><strong>版本:</strong> {}</p>\n", version));
                }
            }

            if let Some(paths) = json.get("paths").and_then(|p| p.as_object()) {
                html.push_str("<h2>接口列表</h2>\n");
                for (path, methods) in paths {
                    html.push_str(&format!("<h3><code>{}</code></h3>\n", path));
                    if let Some(methods_obj) = methods.as_object() {
                        html.push_str("<ul>\n");
                        for (method, details) in methods_obj {
                            html.push_str(&format!("<li><strong>{}</strong> ", method.to_uppercase()));
                            if let Some(summary) = details.get("summary").and_then(|s| s.as_str()) {
                                html.push_str(summary);
                            }
                            html.push_str("</li>\n");
                        }
                        html.push_str("</ul>\n");
                    }
                }
            }

            html.push_str("</body>\n</html>");
            html
        } else {
            format!("<pre>{}</pre>", self.api_data)
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_data.clear();
        self.error.clear();
        self.api_data_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.api_data = text.to_string();
                self.api_data_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if !self.api_data.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.api_data.clone()));
        }
    }
}

impl Render for TransformOpenapi {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("OpenAPI 工具"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_24().child("输出格式"))
                            .child(Select::new(&self.output_type_state))
                            .child(
                                Button::new("convert")
                                    .primary()
                                    .icon(Icon::new(IconName::ArrowRight))
                                    .tooltip("转换")
                                    .disabled(self.api_data.trim().is_empty())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.convert_to_format(window, cx);
                                    })),
                            ),
                    )
                    .child(if !error.is_empty() {
                        div()
                            .text_sm()
                            .text_color(rgb(0xff0000))
                            .child(error)
                    } else {
                        div()
                    })
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
                                    .child(div().text_sm().child("OpenAPI 数据"))
                                    .child(
                                        ButtonGroup::new("data-buttons")
                                            .child(
                                                Button::new("paste")
                                                    .icon(Icon::new(IconName::File))
                                                    .tooltip("粘贴")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.paste(window, cx);
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("copy")
                                                    .icon(Icon::new(IconName::Copy))
                                                    .tooltip("复制")
                                                    .disabled(self.api_data.is_empty())
                                                    .on_click(cx.listener(
                                                        |this, _, _, cx| {
                                                            this.copy(cx);
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("clear")
                                                    .icon(Icon::new(IconName::Delete))
                                                    .tooltip("清空")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.clear(window, cx);
                                                        },
                                                    )),
                                            ),
                                    ),
                            )
                            .child(Input::new(&self.api_data_state).h(px(450.0))),
                    ),
            )
    }
}

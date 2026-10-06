use gpui_kit::{
    component::{
        button::*,
        input::{InputEvent, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

pub struct JsonSchemaValidator {
    schema: String,
    instance: String,
    draft: String,
    result_text: String,
    valid: bool,
    has_result: bool,
    issue_count: usize,
    error: String,
    paths: Vec<String>,
    schema_state: Entity<TextareaState>,
    instance_state: Entity<TextareaState>,
    result_state: Entity<TextareaState>,
    draft_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl JsonSchemaValidator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let schema = ::base::schema_template().to_string();
        let instance = ::base::instance_template().to_string();

        let schema_state = cx.new(|cx| {
            let mut s = TextareaState::new(window, cx).placeholder("请输入 JSON Schema");
            s.set_value(::base::schema_template().to_string(), window, cx);
            s
        });
        let instance_state = cx.new(|cx| {
            let mut s = TextareaState::new(window, cx).placeholder("请输入待校验的 JSON 实例");
            s.set_value(::base::instance_template().to_string(), window, cx);
            s
        });
        let result_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("校验结果将显示在这里"));

        let draft_state = cx.new(|cx| {
            let items = ::base::draft_options()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let mut state = SelectState::new(items, None, window, cx);
            state.set_selected_value(&"自动".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&schema_state, window, {
                let s = schema_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.schema = s.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&instance_state, window, {
                let s = instance_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.instance = s.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &draft_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(v)) = ev {
                        this.draft = v.clone();
                        this.has_result = false;
                        this.result_text.clear();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            schema,
            instance,
            draft: "自动".to_string(),
            result_text: String::new(),
            valid: false,
            has_result: false,
            issue_count: 0,
            error: String::new(),
            paths: Vec::new(),
            schema_state,
            instance_state,
            result_state,
            draft_state,
            _subscriptions,
        }
    }

    fn set_result(&self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        self.result_state
            .update(cx, |s, cx| s.set_value(text, window, cx));
    }

    fn resolve_draft(&self) -> String {
        if self.draft == "自动" {
            ::base::draft_of(&self.schema)
        } else {
            self.draft.clone()
        }
    }

    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        self.paths.clear();
        let draft = self.resolve_draft();
        match ::base::validate(&self.schema, &self.instance, &draft) {
            Ok(report) => {
                self.valid = report.valid;
                self.has_result = true;
                self.issue_count = report.total_errors;
                self.result_text = ::base::format_report_text(&report);
                let text = self.result_text.clone();
                self.set_result(text, window, cx);
                if report.valid {
                    self.paths = ::base::paths_of(&self.instance);
                }
            }
            Err(e) => {
                self.has_result = false;
                self.result_text.clear();
                self.error = e.to_string();
                self.set_result(String::new(), window, cx);
            }
        }
        cx.notify();
    }

    fn format_schema(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        match ::base::pretty_json(&self.schema) {
            Ok(text) => {
                self.schema = text.clone();
                self.schema_state
                    .update(cx, |s, cx| s.set_value(text, window, cx));
            }
            Err(e) => self.error = format!("Schema 格式化失败：{e}"),
        }
        cx.notify();
    }

    fn format_instance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error.clear();
        match ::base::pretty_json(&self.instance) {
            Ok(text) => {
                self.instance = text.clone();
                self.instance_state
                    .update(cx, |s, cx| s.set_value(text, window, cx));
            }
            Err(e) => self.error = format!("实例格式化失败：{e}"),
        }
        cx.notify();
    }

    fn load_sample(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.schema = ::base::schema_template().to_string();
        self.instance = ::base::instance_template().to_string();
        self.schema_state.update(cx, |s, cx| {
            s.set_value(::base::schema_template().to_string(), window, cx)
        });
        self.instance_state.update(cx, |s, cx| {
            s.set_value(::base::instance_template().to_string(), window, cx)
        });
        self.has_result = false;
        self.error.clear();
        self.result_text.clear();
        cx.notify();
    }

    fn copy_result(&mut self, cx: &mut Context<Self>) {
        if self.result_text.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.result_text.clone()));
        self.error.clear();
        cx.notify();
    }

    fn label(text: &'static str, cx: &App) -> Div {
        div()
            .w(px(76.0))
            .flex_shrink_0()
            .text_size(px(12.0))
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }
}

impl Render for JsonSchemaValidator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let result = self.result_text.clone();
        let valid = self.valid;
        let has_result = self.has_result;
        let issue_count = self.issue_count;
        let paths = self.paths.clone();

        let status = if has_result {
            if valid {
                div()
                    .text_size(px(12.5))
                    .text_color(Hsla::from(rgb(design::OK_GREEN)))
                    .child("校验通过：实例符合 Schema")
            } else {
                div()
                    .text_size(px(12.5))
                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                    .child(format!("校验未通过：{issue_count} 处问题"))
            }
        } else {
            div()
        };

        design::page().child(
            design::card(cx)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(Self::label("草案版本", cx))
                        .child(div().w(px(140.0)).child(Select::new(&self.draft_state)))
                        .child(
                            Button::new("js-load-sample")
                                .icon(Icon::new(IconName::FileText))
                                .tooltip("载入示例")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.load_sample(window, cx)
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_4()
                        .child(
                            div()
                                .flex_col()
                                .flex_1()
                                .min_w(px(280.0))
                                .gap_1p5()
                                .child(design::editor_label("JSON Schema", cx))
                                .child(
                                    design::toolbar().child(
                                        Button::new("js-fmt-schema")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .tooltip("格式化")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.format_schema(window, cx)
                                            })),
                                    ),
                                )
                                .child(
                                    Textarea::new(&self.schema_state)
                                        .h(px(230.0))
                                        .font_family("monospace"),
                                ),
                        )
                        .child(
                            div()
                                .flex_col()
                                .flex_1()
                                .min_w(px(280.0))
                                .gap_1p5()
                                .child(design::editor_label("JSON 实例", cx))
                                .child(
                                    design::toolbar().child(
                                        Button::new("js-fmt-instance")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .tooltip("格式化")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.format_instance(window, cx)
                                            })),
                                    ),
                                )
                                .child(
                                    Textarea::new(&self.instance_state)
                                        .h(px(230.0))
                                        .font_family("monospace"),
                                ),
                        ),
                )
                .child(
                    design::toolbar()
                        .child(
                            Button::new("js-validate")
                                .primary()
                                .icon(Icon::new(IconName::Check))
                                .tooltip("开始校验")
                                .on_click(cx.listener(|this, _, window, cx| this.run(window, cx))),
                        )
                        .child(
                            Button::new("js-copy")
                                .icon(Icon::new(IconName::Copy))
                                .tooltip("复制结果")
                                .disabled(result.is_empty())
                                .on_click(cx.listener(|this, _, _, cx| this.copy_result(cx))),
                        ),
                )
                .child(status)
                .when(!error.is_empty(), |c| {
                    c.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::ERROR_RED)))
                            .child(error.clone()),
                    )
                })
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("校验报告", cx))
                        .child(
                            Textarea::new(&self.result_state)
                                .h(px(160.0))
                                .font_family("monospace"),
                        ),
                )
                .when(!paths.is_empty(), |c| {
                    c.child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("实例数据路径", cx))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_2()
                                    .max_h(px(120.0))
                                    .overflow_hidden()
                                    .children(paths.iter().map(|p| {
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(cx.theme().muted)
                                            .text_size(px(11.5))
                                            .font_family("monospace")
                                            .text_color(cx.theme().foreground)
                                            .child(p.clone())
                                    })),
                            ),
                    )
                })
                .child(design::hint(
                    "JSON Schema 用于描述 JSON 数据的结构与约束。校验失败时会列出每处问题的数据路径，例如 $.tags[0] 表示 tags 数组的第 1 项。若 Schema 使用了 $schema 声明，识别版本会自动匹配；手动选择草案版本可用于兼容历史接口。",
                    cx,
                )),
        )
    }
}

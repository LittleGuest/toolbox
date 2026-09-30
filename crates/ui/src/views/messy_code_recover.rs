use crate::design;
use ::base::RecoverGarbledCode;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};
use gpui_kit::prelude::FluentBuilder;

pub struct MessyCodeRecover {
    input: String,
    results: Vec<RecoverGarbledCode>,
    loading: bool,
    status: String,
    status_error: bool,
    input_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl MessyCodeRecover {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("请输入乱码文本，例如：锘挎槬鐪犱笉瑙夋檽锛屽澶勯椈鍟奸笩。")
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.input = input_state.read(cx).value().to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            results: Vec::new(),
            loading: false,
            status: String::new(),
            status_error: false,
            input_state,
            _subscriptions,
        }
    }

    fn recover(&mut self, cx: &mut Context<Self>) {
        let value = self.input.trim().to_string();
        if value.is_empty() {
            self.status = "请输入乱码文本".to_string();
            self.status_error = true;
            cx.notify();
            return;
        }

        self.loading = true;
        match ::base::recover_garbled_code(&value) {
            Ok(results) => {
                self.results = results;
                if self.results.is_empty() {
                    self.status = "未找到可恢复的文本".to_string();
                    self.status_error = false;
                } else {
                    self.status.clear();
                }
            }
            Err(e) => {
                self.results.clear();
                self.status = e.to_string();
                self.status_error = true;
            }
        }
        self.loading = false;
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.results.clear();
        self.status.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_result(&mut self, text: &str, cx: &mut Context<Self>) {
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            self.status = "已复制到剪贴板".to_string();
            self.status_error = false;
            cx.notify();
        }
    }

    fn score_color(&self, score: f64) -> Hsla {
        if score >= 0.9 {
            Hsla::from(rgb(design::OK_GREEN))
        } else if score >= 0.7 {
            Hsla::from(rgb(design::WARN_AMBER))
        } else {
            Hsla::from(rgb(design::ERROR_RED))
        }
    }

    /// 结果表格（对应 n-data-table）
    fn result_table(&self, cx: &mut Context<Self>) -> Div {
        let header = |label: &'static str, width: Pixels| {
            div()
                .w(width)
                .flex_shrink_0()
                .px_2()
                .py_1p5()
                .text_xs()
                .font_semibold()
                .text_color(cx.theme().muted_foreground)
                .child(label)
        };

        div()
            .flex_col()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(6.0))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(header("序号", px(48.0)))
                    .child(header("源编码（假设）", px(110.0)))
                    .child(header("目标编码（假设）", px(110.0)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px_2()
                            .py_1p5()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().muted_foreground)
                            .child("恢复后的文本"),
                    )
                    .child(header("得分", px(64.0)))
                    .child(header("操作", px(60.0))),
            )
            .children(self.results.iter().enumerate().map(|(idx, row)| {
                let recovered = row.recovered_text.clone();
                let score_text = format!("{:.2}", row.score * 100.0);
                let score_color = self.score_color(row.score);

                div()
                    .id(("row", idx))
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .hover(|s: StyleRefinement| s.bg(cx.theme().background))
                    .child(
                        div()
                            .w(px(48.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child((idx + 1).to_string()),
                    )
                    .child(
                        div()
                            .w(px(110.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .text_sm()
                            .truncate()
                            .child(row.source_charset.clone()),
                    )
                    .child(
                        div()
                            .w(px(110.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .text_sm()
                            .truncate()
                            .child(row.target_charset.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px_2()
                            .py_1p5()
                            .text_sm()
                            .font_family("monospace")
                            .truncate()
                            .child(recovered.clone()),
                    )
                    .child(
                        div()
                            .w(px(64.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .text_sm()
                            .font_semibold()
                            .text_color(score_color)
                            .child(score_text),
                    )
                    .child(
                        div().w(px(60.0)).flex_shrink_0().px_2().py(px(3.0)).child(
                            Button::new(("copy-row", idx))
                                .success()
                                .compact()
                                .label("复制")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.copy_result(&recovered, cx);
                                })),
                        ),
                    )
            }))
    }
}

impl Render for MessyCodeRecover {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let loading = self.loading;
        let has_results = !self.results.is_empty();
        let status = self.status.clone();
        let status_error = self.status_error;

        design::page()
            .child(design::page_header("乱码恢复", "修复乱码文本", cx))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::FileText,
                        "乱码恢复",
                        "尝试恢复乱码文本",
                        cx,
                    ))
                    .child(
                        div().flex_col().gap_1p5().child(design::editor_label("输入", cx)).child(
                            Textarea::new(&self.input_state)
                                .h(px(150.0))
                                .font_family("monospace"),
                        ),
                    )
                    .child(
                        div().flex().items_center().gap_2().child(
                            Button::new("recover")
                                .primary()
                                .icon(Icon::new(IconName::Play))
                                .tooltip("恢复")
                                .loading(loading)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.recover(cx);
                                })),
                        ),
                    )
                    .when(!self.status.is_empty() || has_results, |this| {
                        this.child(
                            div().flex().items_center().gap_2().child(
                                Button::new("clear")
                                    .ghost()
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                        )
                    })
                    .child(design::hint(
                        "说明：并非所有乱码都可以被完美恢复，乱码中的问号说明该字符已经丢失，是无法恢复的。",
                        cx,
                    ))
                    .when(!status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(if status_error {
                                    Hsla::from(rgb(design::ERROR_RED))
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child(status),
                        )
                    }),
            )
            .child(
                design::card(cx)
                    .child(design::editor_label("结果", cx))
                    .child(if has_results {
                        self.result_table(cx).into_any_element()
                    } else {
                        design::hint("暂无数据", cx).into_any_element()
                    }),
            )
    }
}

use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};

pub struct MessyCodeRecover {
    input: String,
    results: Vec<base::RecoverGarbledCode>,
    loading: bool,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl MessyCodeRecover {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("请输入乱码文本，例如：锘挎槬鐪犱笉瑙夋檽锛屽澶勯椈鍟奸笩")
                .multi_line(true)
        });

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, _ev: &InputEvent, _window, cx| {
                let value = input_state.read(cx).value();
                this.input = value.to_string();
                cx.notify();
            }
        })];

        Self {
            input: String::new(),
            results: Vec::new(),
            loading: false,
            input_state,
            _subscriptions,
        }
    }

    fn recover(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        self.loading = true;
        self.results = base::recover_garbled_code(&self.input).unwrap_or_default();
        self.loading = false;
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.results.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn copy_result(&self, text: &str, cx: &mut Context<Self>) {
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
        }
    }
}

impl Render for MessyCodeRecover {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let results = self.results.clone();
        let loading = self.loading;

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("乱码恢复"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().child("输入"))
                            .child(Input::new(&self.input_state).h(px(120.0))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("recover")
                                    .primary()
                                    .icon(Icon::new(IconName::Asterisk))
                                    .tooltip("恢复")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.recover();
                                        cx.notify();
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
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "说明{}并非所有乱码都可以被完美恢复{}乱码中的问号说明该字符已经丢失{}是无法恢复的{}",
                                std::char::from_u32(0xFF1A).unwrap_or(':'),
                                std::char::from_u32(0xFF0C).unwrap_or(','),
                                std::char::from_u32(0xFF0C).unwrap_or(','),
                                std::char::from_u32(0x3002).unwrap_or('.')
                            )),
                    )
                    .child(if !results.is_empty() {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().font_semibold().child("恢复结果"))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(400.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_4()
                                    .overflow_y_scrollbar()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(results.iter().enumerate().map(
                                                |(idx, result)| {
                                                    let score_text =
                                                        format!("{:.2}%", result.score * 100.0);
                                                    let score_color =
                                                        if result.score >= 0.9 {
                                                            cx.theme().success
                                                        } else if result.score >= 0.7 {
                                                            cx.theme().warning
                                                        } else {
                                                            cx.theme().accent
                                                        };
                                                    let recovered_text =
                                                        result.recovered_text.clone();

                                                    div()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(cx.theme().border)
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
                                                                        .child(
                                                                            div()
                                                                                .flex()
                                                                                .items_center()
                                                                                .gap_4()
                                                                                .child(
                                                                                    div()
                                                                                        .text_xs()
                                                                                        .text_color(
                                                                                            cx.theme()
                                                                                                .muted_foreground,
                                                                                        )
                                                                                        .child(
                                                                                            format!(
                                                                                                "#{}",
                                                                                                idx + 1
                                                                                            ),
                                                                                        ),
                                                                                )
                                                                                .child(
                                                                                    div()
                                                                                        .text_xs()
                                                                                        .child(
                                                                                            format!(
                                                                                                "{} -> {}",
                                                                                                result
                                                                                                    .source_charset,
                                                                                                result
                                                                                                    .target_charset
                                                                                            ),
                                                                                        ),
                                                                                )
                                                                                .child(
                                                                                    div()
                                                                                        .text_xs()
                                                                                        .font_semibold()
                                                                                        .text_color(
                                                                                            score_color,
                                                                                        )
                                                                                        .child(
                                                                                            score_text,
                                                                                        ),
                                                                                ),
                                                                        )
                                                                        .child(
                                                                            Button::new((
                                                                                "copy",
                                                                                idx,
                                                                            ))
                                                                            .icon(Icon::new(
                                                                                IconName::Copy,
                                                                            ))
                                                                            .tooltip("复制")
                                                                            .on_click(
                                                                                cx.listener(
                                                                                    move |this,
                                                                                          _,
                                                                                          _,
                                                                                          cx| {
                                                                                        this.copy_result(
                                                                                            &recovered_text,
                                                                                            cx,
                                                                                        );
                                                                                    },
                                                                                ),
                                                                            ),
                                                                        ),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_family("monospace")
                                                                        .child(
                                                                            result
                                                                                .recovered_text
                                                                                .clone(),
                                                                        ),
                                                                ),
                                                        )
                                                },
                                            )),
                                    ),
                            )
                    } else if loading {
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py_8()
                            .child(div().text_sm().text_color(cx.theme().muted_foreground).child("正在恢复..."))
                    } else {
                        div()
                    }),
            )
    }
}

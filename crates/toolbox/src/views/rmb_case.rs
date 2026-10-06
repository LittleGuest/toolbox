use ::base::RmbParseResult;
use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        *,
    },
    prelude::FluentBuilder,
    *,
};

const YUAN_UNITS: [&str; 2] = ["元", "圆"];
const ZHENG_UNITS: [&str; 2] = ["整", "正"];

const AMOUNT_EXAMPLES: [&str; 5] = ["1,234.56", "6007.14", "16409.02", "0.56", "1000000.00"];
const UPPER_EXAMPLES: [&str; 4] = [
    "壹仟肆佰零玖元伍角",
    "壹万陆仟肆佰零玖元零贰分",
    "壹亿元人民币整",
    "人民币叁佰贰拾壹元零伍分",
];

pub struct RmbCase {
    mode: usize,
    yuan_index: usize,
    zheng_index: usize,
    zheng_yuan: bool,
    jiao_zheng: bool,
    amount_index: usize,
    upper_index: usize,
    amount: String,
    upper_result: String,
    upper_error: String,
    upper_input: String,
    parsed: Option<RmbParseResult>,
    parse_error: String,
    amount_state: Entity<InputState>,
    upper_result_state: Entity<TextareaState>,
    upper_input_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl RmbCase {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let amount_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("如 1234.56、¥1,234.56"));
        let upper_result_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("转换后的大写金额"));
        let upper_input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("如：壹仟贰佰叁拾肆元伍角陆分（支持壹/一、元/圆、整/正等写法）")
        });

        let _subscriptions = vec![
            cx.subscribe_in(&amount_state, window, {
                let amount_state = amount_state.clone();
                move |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        this.amount = amount_state.read(cx).value().to_string();
                        this.convert_to_upper(window, cx);
                    }
                }
            }),
            cx.subscribe_in(&upper_input_state, window, {
                let upper_input_state = upper_input_state.clone();
                move |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        this.upper_input = upper_input_state.read(cx).value().to_string();
                        this.parse_upper(cx);
                    }
                }
            }),
        ];

        Self {
            mode: 0,
            yuan_index: 0,
            zheng_index: 0,
            zheng_yuan: true,
            jiao_zheng: false,
            amount_index: 0,
            upper_index: 0,
            amount: String::new(),
            upper_result: String::new(),
            upper_error: String::new(),
            upper_input: String::new(),
            parsed: None,
            parse_error: String::new(),
            amount_state,
            upper_result_state,
            upper_input_state,
            _subscriptions,
        }
    }

    fn yuan(&self) -> &'static str {
        YUAN_UNITS.get(self.yuan_index).copied().unwrap_or("元")
    }

    fn zheng(&self) -> &'static str {
        ZHENG_UNITS.get(self.zheng_index).copied().unwrap_or("整")
    }

    fn convert_to_upper(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.amount.trim().to_string();
        if value.is_empty() {
            self.upper_result.clear();
            self.upper_error.clear();
            self.upper_result_state.update(cx, |state, cx| {
                state.set_value("".to_string(), window, cx);
            });
            cx.notify();
            return;
        }
        match ::base::rmb_to_upper(
            &value,
            Some(self.yuan()),
            Some(self.zheng()),
            self.zheng_yuan,
            self.jiao_zheng,
        ) {
            Ok(upper) => {
                self.upper_result = upper.clone();
                self.upper_error.clear();
                self.upper_result_state.update(cx, |state, cx| {
                    state.set_value(upper, window, cx);
                });
            }
            Err(e) => {
                self.upper_result.clear();
                self.upper_error = e.to_string();
                self.upper_result_state.update(cx, |state, cx| {
                    state.set_value("".to_string(), window, cx);
                });
            }
        }
        cx.notify();
    }

    fn parse_upper(&mut self, cx: &mut Context<Self>) {
        let value = self.upper_input.trim().to_string();
        if value.is_empty() {
            self.parsed = None;
            self.parse_error.clear();
            cx.notify();
            return;
        }
        match ::base::rmb_to_amount(
            &value,
            Some(self.yuan()),
            Some(self.zheng()),
            self.zheng_yuan,
        ) {
            Ok(parsed) => {
                self.parsed = Some(parsed);
                self.parse_error.clear();
            }
            Err(e) => {
                self.parsed = None;
                self.parse_error = e.to_string();
            }
        }
        cx.notify();
    }

    fn fill_amount_example(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let example = AMOUNT_EXAMPLES[self.amount_index % AMOUNT_EXAMPLES.len()].to_string();
        self.amount_index += 1;
        self.amount = example.clone();
        self.amount_state.update(cx, |state, cx| {
            state.set_value(example, window, cx);
        });
        self.convert_to_upper(window, cx);
    }

    fn fill_upper_example(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let example = UPPER_EXAMPLES[self.upper_index % UPPER_EXAMPLES.len()].to_string();
        self.upper_index += 1;
        self.upper_input = example.clone();
        self.upper_input_state.update(cx, |state, cx| {
            state.set_value(example, window, cx);
        });
        self.parse_upper(cx);
    }

    fn paste_amount(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.amount = text.to_string();
                self.amount_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                self.convert_to_upper(window, cx);
            }
        }
    }

    fn paste_upper(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.upper_input = text.to_string();
                self.upper_input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                self.parse_upper(cx);
            }
        }
    }

    fn clear_amount(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.amount.clear();
        self.upper_result.clear();
        self.upper_error.clear();
        self.amount_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.upper_result_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn clear_upper(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.upper_input.clear();
        self.parsed = None;
        self.parse_error.clear();
        self.upper_input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }

    fn segmented(
        &self,
        name: &'static str,
        options: &[&'static str],
        selected: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(options.iter().enumerate().map(|(i, opt)| {
                let active = i == selected;
                div()
                    .id((name, i))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .bg(if active {
                        cx.theme().primary
                    } else {
                        gpui::black().opacity(0.0)
                    })
                    .text_color(if active {
                        gpui::white()
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if name == "yuan" {
                            this.yuan_index = i;
                        } else {
                            this.zheng_index = i;
                        }
                        this.convert_to_upper(window, cx);
                        this.parse_upper(cx);
                    }))
                    .child(opt.to_string())
            }))
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_6()
            .border_b_1()
            .border_color(cx.theme().border)
            .mb_4()
            .child(self.tab_btn("tab-upper", "数字 → 大写", 0, cx))
            .child(self.tab_btn("tab-amount", "大写 → 数字", 1, cx))
    }

    fn tab_btn(
        &self,
        id: &'static str,
        label: &'static str,
        idx: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let active = self.mode == idx;
        div()
            .id(id)
            .px_1()
            .pb_2()
            .mb(px(-1.0))
            .text_sm()
            .font_medium()
            .border_b_2()
            .text_color(if active {
                cx.theme().primary
            } else {
                cx.theme().muted_foreground
            })
            .border_color(if active {
                cx.theme().primary
            } else {
                gpui::black().opacity(0.0)
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.mode = idx;
                cx.notify();
            }))
            .child(label.to_string())
    }

    fn config_row(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_4()
            .mb_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("元位"),
                    )
                    .child(self.segmented("yuan", &YUAN_UNITS, self.yuan_index, cx)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("结尾字"),
                    )
                    .child(self.segmented("zheng", &ZHENG_UNITS, self.zheng_index, cx)),
            )
            .child(
                Checkbox::new("zheng-yuan")
                    .label(format!("整数金额结尾加「{}」", self.zheng()))
                    .checked(self.zheng_yuan)
                    .tooltip(format!(
                        "仅当没有角、分时追加结尾字，如 100 → 壹佰元{}；取消勾选则输出 壹佰元",
                        self.zheng()
                    ))
                    .on_click(cx.listener(|this, checked: &bool, window, cx| {
                        this.zheng_yuan = *checked;
                        this.convert_to_upper(window, cx);
                        this.parse_upper(cx);
                    })),
            )
            .child(
                Checkbox::new("jiao-zheng")
                    .label(format!("角位金额后加「{}」", self.zheng()))
                    .checked(self.jiao_zheng)
                    .tooltip(format!(
                        "仅当有角无分时追加结尾字，如 100.50 → 壹佰元伍角{}",
                        self.zheng()
                    ))
                    .on_click(cx.listener(|this, checked: &bool, window, cx| {
                        this.jiao_zheng = *checked;
                        this.convert_to_upper(window, cx);
                        this.parse_upper(cx);
                    })),
            )
    }
}

impl Render for RmbCase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.mode;

        design::page().child(
            design::card(cx)
                .child(self.config_row(cx))
                .child(self.tab_bar(cx))
                .children(if mode == 0 {
                    vec![self.render_to_upper(cx).into_any_element()]
                } else {
                    vec![self.render_to_amount(cx).into_any_element()]
                }),
        )
    }
}

impl RmbCase {
    fn render_to_upper(&mut self, cx: &mut Context<Self>) -> Div {
        let upper_error = self.upper_error.clone();

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("金额"),
                    )
                    .child(div().w(px(260.0)).child(Input::new(&self.amount_state))),
            )
            .child(
                design::action_row()
                    .child(
                        Button::new("convert-upper")
                            .primary()
                            .icon(Icon::new(IconName::Replace))
                            .tooltip("转换")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.convert_to_upper(window, cx);
                            })),
                    )
                    .child(
                        Button::new("fill-amount")
                            .icon(Icon::new(IconName::Asterisk))
                            .tooltip("填入示例")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.fill_amount_example(window, cx);
                            })),
                    )
                    .child(
                        Button::new("paste-amount")
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴金额")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste_amount(window, cx);
                            })),
                    )
                    .child(
                        Button::new("copy-upper")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制大写")
                            .disabled(self.upper_result.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let value = this.upper_result.clone();
                                this.copy_text(&value, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-amount")
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_amount(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_2()
                    .child(design::editor_label("大写金额", cx))
                    .child(
                        Textarea::new(&self.upper_result_state)
                            .h(px(120.0))
                            .font_family("monospace"),
                    ),
            )
            .when(!upper_error.is_empty(), |this| {
                this.child(
                    div()
                        .mt(px(10.0))
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                        .child(upper_error),
                )
            })
    }

    fn render_to_amount(&mut self, cx: &mut Context<Self>) -> Div {
        let parse_error = self.parse_error.clone();
        let (amount_text, grouped_text, norm_text) = match &self.parsed {
            Some(p) => (p.amount.clone(), p.grouped.clone(), p.upper.clone()),
            None => (String::new(), String::new(), String::new()),
        };

        div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex_col()
                    .gap_2()
                    .child(design::editor_label("中文大写金额", cx))
                    .child(
                        Textarea::new(&self.upper_input_state)
                            .h(px(120.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                design::action_row()
                    .child(
                        Button::new("parse-upper")
                            .primary()
                            .icon(Icon::new(IconName::Replace))
                            .tooltip("转换")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.parse_upper(cx);
                            })),
                    )
                    .child(
                        Button::new("fill-upper")
                            .icon(Icon::new(IconName::Asterisk))
                            .tooltip("填入示例")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.fill_upper_example(window, cx);
                            })),
                    )
                    .child(
                        Button::new("paste-upper")
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.paste_upper(window, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-upper")
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_upper(window, cx);
                            })),
                    ),
            )
            .when(self.parsed.is_some(), |this| {
                this.child(
                    div()
                        .flex_col()
                        .gap_2p5()
                        .child(design::editor_label("解析结果", cx))
                        .child(self.result_row("copy-parsed-amount", "数字金额", &amount_text, cx))
                        .child(self.result_row("copy-parsed-grouped", "千分位", &grouped_text, cx))
                        .child(self.result_row("copy-parsed-upper", "规范大写", &norm_text, cx)),
                )
            })
            .when(!parse_error.is_empty(), |this| {
                this.child(
                    div()
                        .mt(px(10.0))
                        .text_size(px(12.5))
                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                        .child(parse_error),
                )
            })
    }

    fn result_row(
        &self,
        id: &'static str,
        label: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) -> Div {
        let value = value.to_string();
        div()
            .flex()
            .items_center()
            .gap_2p5()
            .child(
                div()
                    .w(px(88.0))
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .px_3()
                    .py_1p5()
                    .text_sm()
                    .font_family("monospace")
                    .truncate()
                    .child(value.clone()),
            )
            .child(
                Button::new(id)
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Copy))
                    .tooltip("复制")
                    .disabled(value.is_empty())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_text(&value, cx);
                    })),
            )
    }
}

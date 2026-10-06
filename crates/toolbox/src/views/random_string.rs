use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};
use rand::Rng;

const CHARSET_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const CHARSET_UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CHARSET_DIGIT: &str = "0123456789";
const CHARSET_SYMBOL: &str = "!@#$%^&*()-_=+[]{};:,.<>?";
const SIMILAR_CHARS: &[char] = &['O', '0', 'I', 'l', '1'];
const MAX_LENGTH: usize = 10000;
const MAX_COUNT: usize = 1000;

fn config_item(label: &'static str, control: Div) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(0x5b6478))
                .child(label),
        )
        .child(control)
}

pub struct RandomStringGenerator {
    length_state: Entity<InputState>,
    count_state: Entity<InputState>,
    custom_charset_state: Entity<InputState>,
    use_lower: bool,
    use_upper: bool,
    use_digit: bool,
    use_symbol: bool,
    exclude_similar: bool,
    password_mode: bool,
    output_state: Entity<TextareaState>,
    status: String,
    _subscriptions: Vec<Subscription>,
}

impl RandomStringGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let length_state = cx.new(|cx| InputState::new(window, cx).default_value("16".to_string()));
        let count_state = cx.new(|cx| InputState::new(window, cx).default_value("5".to_string()));
        let custom_charset_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("填写后覆盖上方字符集选择，如 abcXYZ0123")
        });
        let output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("生成结果（每行一个）"));

        let mut _subscriptions = Vec::new();
        for state in [&length_state, &count_state, &custom_charset_state] {
            _subscriptions.push(cx.subscribe_in(
                state,
                window,
                move |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        this.generate(window, cx);
                    }
                },
            ));
        }

        Self {
            length_state,
            count_state,
            custom_charset_state,
            use_lower: true,
            use_upper: true,
            use_digit: true,
            use_symbol: true,
            exclude_similar: false,
            password_mode: false,
            output_state,
            status: String::new(),
            _subscriptions,
        }
    }

    fn read_usize(
        &self,
        state: &Entity<InputState>,
        cx: &App,
        default: usize,
        max: usize,
    ) -> usize {
        let v = state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(default);
        v.clamp(1, max)
    }

    fn merged_charset(&self, cx: &App) -> Vec<char> {
        let custom = self
            .custom_charset_state
            .read(cx)
            .value()
            .trim()
            .to_string();
        let mut s = if !custom.is_empty() {
            custom
        } else {
            let mut s = String::new();
            if self.use_lower {
                s.push_str(CHARSET_LOWER);
            }
            if self.use_upper {
                s.push_str(CHARSET_UPPER);
            }
            if self.use_digit {
                s.push_str(CHARSET_DIGIT);
            }
            if self.use_symbol {
                s.push_str(CHARSET_SYMBOL);
            }
            s
        };
        if self.exclude_similar {
            s = s.chars().filter(|c| !SIMILAR_CHARS.contains(c)).collect();
        }
        let mut seen = std::collections::HashSet::new();
        s.chars().filter(|c| seen.insert(*c)).collect()
    }

    fn generate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let charset = self.merged_charset(cx);
        if charset.is_empty() {
            self.status = "请选择至少一种字符集或填写自定义字符集".to_string();
            self.output_state.update(cx, |state, cx| {
                state.set_value("".to_string(), window, cx);
            });
            cx.notify();
            return;
        }
        let length = self.read_usize(&self.length_state, cx, 16, MAX_LENGTH);
        let count = self.read_usize(&self.count_state, cx, 5, MAX_COUNT);
        let mut rng = rand::rng();
        let mut lines: Vec<String> = Vec::with_capacity(count);

        for _ in 0..count {
            if self.password_mode && self.custom_charset_state.read(cx).value().trim().is_empty() {
                let mut parts: Vec<char> = Vec::new();
                if self.use_lower {
                    parts.push(
                        CHARSET_LOWER.as_bytes()[rng.random_range(0..CHARSET_LOWER.len())] as char,
                    );
                }
                if self.use_upper {
                    parts.push(
                        CHARSET_UPPER.as_bytes()[rng.random_range(0..CHARSET_UPPER.len())] as char,
                    );
                }
                if self.use_digit {
                    parts.push(
                        CHARSET_DIGIT.as_bytes()[rng.random_range(0..CHARSET_DIGIT.len())] as char,
                    );
                }
                if self.use_symbol {
                    parts.push(
                        CHARSET_SYMBOL.as_bytes()[rng.random_range(0..CHARSET_SYMBOL.len())]
                            as char,
                    );
                }
                while parts.len() < length {
                    parts.push(charset[rng.random_range(0..charset.len())]);
                }
                parts.truncate(length);
                for i in (1..parts.len()).rev() {
                    let j = rng.random_range(0..=i);
                    parts.swap(i, j);
                }
                lines.push(parts.into_iter().collect());
            } else {
                let s: String = (0..length)
                    .map(|_| charset[rng.random_range(0..charset.len())])
                    .collect();
                lines.push(s);
            }
        }

        self.status.clear();
        let output = lines.join("\n");
        self.output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
        cx.notify();
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        let text = self.output_state.read(cx).value().to_string();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for RandomStringGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status.clone();

        design::page().child(
            design::card(cx)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_4()
                        .child(config_item(
                            "长度",
                            div().w(px(160.0)).child(Input::new(&self.length_state)),
                        ))
                        .child(config_item(
                            "数量",
                            div().w(px(160.0)).child(Input::new(&self.count_state)),
                        )),
                )
                .child(config_item(
                    "字符集",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_4()
                        .child(
                            Checkbox::new("use-lower")
                                .label("小写字母")
                                .checked(self.use_lower)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.use_lower = *v;
                                    this.generate(window, cx);
                                })),
                        )
                        .child(
                            Checkbox::new("use-upper")
                                .label("大写字母")
                                .checked(self.use_upper)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.use_upper = *v;
                                    this.generate(window, cx);
                                })),
                        )
                        .child(
                            Checkbox::new("use-digit")
                                .label("数字")
                                .checked(self.use_digit)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.use_digit = *v;
                                    this.generate(window, cx);
                                })),
                        )
                        .child(
                            Checkbox::new("use-symbol")
                                .label("特殊符号")
                                .checked(self.use_symbol)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.use_symbol = *v;
                                    this.generate(window, cx);
                                })),
                        ),
                ))
                .child(config_item(
                    "自定义字符集",
                    div()
                        .w(px(340.0))
                        .child(Input::new(&self.custom_charset_state)),
                ))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_4()
                        .child(
                            Checkbox::new("exclude-similar")
                                .label("排除相似字符（O0Il1）")
                                .checked(self.exclude_similar)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.exclude_similar = *v;
                                    this.generate(window, cx);
                                })),
                        )
                        .child(
                            Checkbox::new("password-mode")
                                .label("密码模式（每类至少一个字符并打乱）")
                                .checked(self.password_mode)
                                .on_click(cx.listener(|this, v: &bool, window, cx| {
                                    this.password_mode = *v;
                                    this.generate(window, cx);
                                })),
                        )
                        .child(
                            Button::new("generate")
                                .primary()
                                .icon(Icon::new(IconName::Play))
                                .tooltip("生成")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.generate(window, cx);
                                })),
                        )
                        .when(!status.is_empty(), |this| {
                            this.child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                                    .child(status),
                            )
                        }),
                )
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("输出", cx))
                        .child(
                            Textarea::new(&self.output_state)
                                .h(px(220.0))
                                .font_family("monospace"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .mt_2()
                                .child(
                                    Button::new("regen-output")
                                        .icon(Icon::new(IconName::RotateCw))
                                        .tooltip("重新生成")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.generate(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("copy-output")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制全部")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.copy_output(cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-output")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear(window, cx);
                                        })),
                                ),
                        ),
                ),
        )
    }
}

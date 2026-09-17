use crate::design;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState, Textarea, TextareaState},
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
    output: String,
    output_state: Entity<TextareaState>,
    status: String,
}

impl RandomStringGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let length_state = cx.new(|cx| InputState::new(window, cx).placeholder("1-10000"));
        let count_state = cx.new(|cx| InputState::new(window, cx).placeholder("1-1000"));
        let custom_charset_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("填写后覆盖上方字符集选择，如 abcXYZ0123"));
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("生成结果（每行一个）")
        });

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
            output: String::new(),
            output_state,
            status: String::new(),
        }
    }

    fn read_usize(&self, state: &Entity<InputState>, cx: &App, default: usize, max: usize) -> usize {
        let v = state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(default);
        v.clamp(1, max)
    }

    fn merged_charset(&self, cx: &App) -> Vec<char> {
        let custom = self.custom_charset_state.read(cx).value().trim().to_string();
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
        // 去重并保持顺序
        let mut seen = std::collections::HashSet::new();
        s.chars().filter(|c| seen.insert(*c)).collect()
    }

    fn generate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let charset = self.merged_charset(cx);
        if charset.is_empty() {
            self.status = "请选择至少一种字符集或填写自定义字符集".to_string();
            cx.notify();
            return;
        }
        let length = self.read_usize(&self.length_state, cx, 16, MAX_LENGTH);
        let count = self.read_usize(&self.count_state, cx, 5, MAX_COUNT);
        let mut rng = rand::rng();
        let mut lines: Vec<String> = Vec::with_capacity(count);

        for _ in 0..count {
            if self.password_mode && self.custom_charset_state.read(cx).value().trim().is_empty() {
                // 密码模式：每种已选字符集至少取一个字符，其余随机补齐后打乱
                let mut parts: Vec<char> = Vec::new();
                if self.use_lower {
                    parts.push(CHARSET_LOWER.as_bytes()[rng.random_range(0..CHARSET_LOWER.len())] as char);
                }
                if self.use_upper {
                    parts.push(CHARSET_UPPER.as_bytes()[rng.random_range(0..CHARSET_UPPER.len())] as char);
                }
                if self.use_digit {
                    parts.push(CHARSET_DIGIT.as_bytes()[rng.random_range(0..CHARSET_DIGIT.len())] as char);
                }
                if self.use_symbol {
                    parts.push(CHARSET_SYMBOL.as_bytes()[rng.random_range(0..CHARSET_SYMBOL.len())] as char);
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

        self.output = lines.join("\n");
        self.status.clear();
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.output.clear();
        self.status.clear();
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }
}

impl Render for RandomStringGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(80.0);

        design::page()
            .child(design::page_header("随机字符串", "生成随机字符串", cx))
            .child(
                // 配置卡片
                design::card(cx)
                    // 长度
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("长度", cx)))
                            .child(div().w(px(160.0)).child(Input::new(&self.length_state))),
                    )
                    // 数量
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("数量", cx)))
                            .child(div().w(px(160.0)).child(Input::new(&self.count_state))),
                    )
                    // 字符集
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(div().w(label_w).mt_1().child(design::caption("字符集", cx)))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_4()
                                    .child(
                                        Checkbox::new("use-lower")
                                            .label("小写字母")
                                            .checked(self.use_lower)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.use_lower = *v;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Checkbox::new("use-upper")
                                            .label("大写字母")
                                            .checked(self.use_upper)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.use_upper = *v;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Checkbox::new("use-digit")
                                            .label("数字")
                                            .checked(self.use_digit)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.use_digit = *v;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Checkbox::new("use-symbol")
                                            .label("特殊符号")
                                            .checked(self.use_symbol)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.use_symbol = *v;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    // 自定义字符集
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("自定义字符集", cx)))
                            .child(div().flex_1().child(Input::new(&self.custom_charset_state))),
                    )
                    // 选项
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(div().w(label_w).mt_1().child(design::caption("选项", cx)))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_4()
                                    .child(
                                        Checkbox::new("exclude-similar")
                                            .label("排除相似字符（O0Il1）")
                                            .checked(self.exclude_similar)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.exclude_similar = *v;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Checkbox::new("password-mode")
                                            .label("密码模式（每类至少一个字符并打乱）")
                                            .checked(self.password_mode)
                                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                                this.password_mode = *v;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            )
            // 生成操作行
            .child(
                design::action_row()
                    .child(
                        Button::new("generate")
                            .label("生成")
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.generate(window, cx);
                            })),
                    )
                    .when(!self.status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xef4444))
                                .child(self.status.clone()),
                        )
                    }),
            )
            // 输出卡片
            .child(
                design::card(cx)
                    .child(
                        Textarea::new(&self.output_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
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
                            )
                            .child(div().flex_1()),
                    ),
            )
    }
}

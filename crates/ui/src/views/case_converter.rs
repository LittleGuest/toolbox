use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};

pub struct CaseConverter {
    input: String,
    output: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// 单词首字母大写（等价 \b\w 大写 + 其余小写）
fn title_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

/// 句子首字母大写：句首（起始或 .!?。！？ 之后的第一个非空白字符）大写，其余不动
fn sentence_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut sentence_start = true;
    for c in s.chars() {
        if sentence_start && !c.is_whitespace() {
            out.extend(c.to_uppercase());
            sentence_start = false;
        } else {
            out.push(c);
            if matches!(c, '.' | '!' | '?' | '。' | '！' | '？') {
                sentence_start = true;
            }
        }
    }
    out
}

/// 小写全文，分隔符（空白/_/-）后的字母转大写；首字母小写
fn camel_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    let mut chars = out.chars();
    match chars.next() {
        Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn pascal_case(s: &str) -> String {
    let camel = camel_case(s);
    let mut chars = camel.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// 蛇形：trim + 连续空白/_/- 折叠为 _（保留原大小写）
fn snake_case(s: &str) -> String {
    join_folded(s, "_")
}

fn kebab_case(s: &str) -> String {
    join_folded(s, "-")
}

fn join_folded(s: &str, sep: &str) -> String {
    let mut out = String::new();
    let mut in_sep = false;
    for c in s.trim().chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            in_sep = true;
        } else {
            if in_sep && !out.is_empty() {
                out.push_str(sep);
            }
            out.push(c);
            in_sep = false;
        }
    }
    out
}

fn swap_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_lowercase().collect::<String>()
            }
        })
        .collect()
}

fn reverse_text(s: &str) -> String {
    s.chars().rev().collect()
}

const UPSIDE_DOWN_MAP: [(&str, &str); 48] = [
    ("a", "ɐ"), ("b", "q"), ("c", "ɔ"), ("d", "p"), ("e", "ǝ"), ("f", "ɟ"), ("g", "ƃ"),
    ("h", "ɥ"), ("i", "ᴉ"), ("j", "ɾ"), ("k", "ʞ"), ("l", "l"), ("m", "ɯ"), ("n", "u"),
    ("o", "o"), ("p", "d"), ("q", "b"), ("r", "ɹ"), ("s", "s"), ("t", "ʇ"), ("u", "n"),
    ("v", "ʌ"), ("w", "ʍ"), ("x", "x"), ("y", "ʎ"), ("z", "z"), ("0", "0"), ("1", "Ɩ"),
    ("2", "ᄅ"), ("3", "Ɛ"), ("4", "ㄣ"), ("5", "ʎ"), ("6", "9"), ("7", "ㄥ"), ("8", "8"),
    ("9", "6"), ("?", "¿"), ("!", "¡"), ("(", ")"), (")", "("), ("[", "]"), ("]", "["),
    ("{", "}"), ("}", "{"), ("<", ">"), (">", "<"), (",", "'"), ("'", ","),
];

fn upside_down(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let mut matched = false;
        // 优先匹配双字符序列（当前表内均为单字符，保留扩展性）
        let rest: String = c.to_string();
        if let Some(&next) = chars.peek() {
            let two = format!("{rest}{next}");
            if let Some((_, v)) = UPSIDE_DOWN_MAP.iter().find(|(k, _)| *k == two) {
                out.push_str(v);
                chars.next();
                matched = true;
            }
        }
        if !matched {
            if let Some((_, v)) = UPSIDE_DOWN_MAP.iter().find(|(k, _)| *k == rest) {
                out.push_str(v);
            } else {
                out.push(c);
            }
        }
    }
    out
}

fn capitalize(w: &str) -> String {
    let mut cs = w.chars();
    match cs.next() {
        Some(f) => f.to_uppercase().collect::<String>() + &cs.as_str().to_lowercase(),
        None => String::new(),
    }
}

impl CaseConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入要转换的文本"));
        let output_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果"));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            output: String::new(),
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn apply(&mut self, f: fn(&str) -> String, window: &mut Window, cx: &mut Context<Self>) {
        self.output = f(&self.input);
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                cx.notify();
            }
        }
    }

    /// 转换操作按钮（对应 Vue 的 11 个图标按钮 + tooltip）
    fn op_button(
        &self,
        id: &'static str,
        label: &'static str,
        tooltip: &'static str,
        primary: bool,
        f: fn(&str) -> String,
        cx: &mut Context<Self>,
    ) -> Button {
        let mut btn = Button::new(id).child(label).tooltip(tooltip);
        if primary {
            btn = btn.primary();
        }
        btn.on_click(cx.listener(move |this, _, window, cx| {
            this.apply(f, window, cx);
        }))
    }

    fn op_toolbar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(self.op_button("op-upper", "AA", "全部大写", false, to_upper, cx))
            .child(self.op_button("op-lower", "aa", "全部小写", false, to_lower, cx))
            .child(self.op_button(
                "op-title",
                "Title",
                "单词首字母大写",
                true,
                title_case,
                cx,
            ))
            .child(self.op_button(
                "op-sentence",
                "Sentence",
                "句子首字母大写",
                false,
                sentence_case,
                cx,
            ))
            .child(self.op_button(
                "op-camel",
                "camel",
                "驼峰 camelCase",
                false,
                camel_case,
                cx,
            ))
            .child(self.op_button(
                "op-pascal",
                "Pascal",
                "帕斯卡 PascalCase",
                false,
                pascal_case,
                cx,
            ))
            .child(self.op_button(
                "op-snake",
                "snake",
                "蛇形 snake_case",
                false,
                snake_case,
                cx,
            ))
            .child(self.op_button(
                "op-kebab",
                "kebab",
                "烤肉串 kebab-case",
                false,
                kebab_case,
                cx,
            ))
            .child(
                Button::new("op-swap")
                    .icon(Icon::new(IconName::CaseSensitive))
                    .tooltip("反转大小写")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.apply(swap_case, window, cx);
                    })),
            )
            .child(
                Button::new("op-reverse")
                    .icon(Icon::new(IconName::SortDescending))
                    .tooltip("反转文本")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.apply(reverse_text, window, cx);
                    })),
            )
            .child(
                Button::new("op-upside")
                    .icon(Icon::new(IconName::RotateCw))
                    .tooltip("字符倒置")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.apply(upside_down, window, cx);
                    })),
            )
    }
}

impl Render for CaseConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header("大小写转换", "文本大小写快速转换", cx))
            .child(
                design::card(cx)
                    .child(self.op_toolbar(cx))
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_3()
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(design::editor_label("输入", cx))
                                    .child(
                                        Textarea::new(&self.input_state)
                                            .h(design::CODE_BOX_HEIGHT)
                                            .font_family("monospace"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(design::editor_label("输出", cx))
                                    .child(
                                        Textarea::new(&self.output_state)
                                            .h(design::CODE_BOX_HEIGHT)
                                            .font_family("monospace"),
                                    ),
                            ),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-input")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Inbox))
                                    .tooltip("粘贴到输入")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_input(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-input")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输入")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let value = this.input.clone();
                                        this.copy_text(&value, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-output")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let value = this.output.clone();
                                        this.copy_text(&value, cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .ghost()
                                    .compact()
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

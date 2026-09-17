use std::collections::HashSet;

use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    *,
};

pub struct TextTools {
    stat_text: String,
    input: String,
    output: String,
    find: String,
    replace: String,
    extract_sep: String,
    is_regex: bool,
    message: String,
    stat_state: Entity<TextareaState>,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    find_state: Entity<InputState>,
    replace_state: Entity<InputState>,
    extract_state: Entity<InputState>,
    repeat_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

// ---------------- 手写纯函数 ----------------

/// 按行拆分，兼容 \r\n、\r、\n（与 Vue split(/\r\n|\r|\n/) 一致，保留尾部空行）
fn split_lines(s: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                lines.push(std::mem::take(&mut cur));
            }
            '\n' => lines.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    lines.push(cur);
    lines
}

fn is_punct_like(c: char) -> bool {
    c.is_ascii_punctuation()
        || matches!(
            c,
            '，' | '。' | '、' | '；' | '：' | '？' | '！' | '“' | '”' | '‘' | '’' | '（' | '）'
                | '《' | '》' | '【' | '】' | '…' | '—' | '·' | '｜'
        )
}

fn compute_stats(s: &str) -> Vec<(&'static str, usize)> {
    let line_count = if s.is_empty() {
        0
    } else {
        s.replace("\r\n", "\n").replace('\r', "\n").split('\n').count()
    };
    vec![
        ("字符数", s.chars().count()),
        ("单词数", s.trim().split_whitespace().count()),
        ("行数", line_count),
        ("非空白字符", s.chars().filter(|c| !c.is_whitespace()).count()),
        ("字节数", s.len()),
        ("中文字符", s.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count()),
        ("英文字母", s.chars().filter(|c| c.is_ascii_alphabetic()).count()),
        ("数字个数", s.chars().filter(|c| c.is_ascii_digit()).count()),
        ("标点符号", s.chars().filter(|c| is_punct_like(*c)).count()),
    ]
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}

/// 删除重音符号：NFD 归一化后移除组合变音符 U+0300–U+036F（对齐 Vue removeDiacritics）
fn remove_diacritics(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfd()
        .filter(|c| !(('\u{0300}'..='\u{036f}').contains(c)))
        .collect()
}

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn swap_case_char(c: char) -> char {
    if c.is_uppercase() {
        c.to_lowercase().next().unwrap_or(c)
    } else {
        c.to_uppercase().next().unwrap_or(c)
    }
}

fn add_slashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\0' => out.push_str("\\0"),
            _ => out.push(c),
        }
    }
    out
}

fn strip_slashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn upside_down_char(c: char) -> char {
    match c {
        'a' => 'ɐ',
        'b' => 'q',
        'c' => 'ɔ',
        'd' => 'p',
        'e' => 'ǝ',
        'f' => 'ɟ',
        'g' => 'ƃ',
        'h' => 'ɥ',
        'i' => 'ᴉ',
        'j' => 'ɾ',
        'k' => 'ʞ',
        'l' => 'l',
        'm' => 'ɯ',
        'n' => 'u',
        'o' => 'o',
        'p' => 'd',
        'q' => 'b',
        'r' => 'ɹ',
        's' => 's',
        't' => 'ʇ',
        'u' => 'n',
        'v' => 'ʌ',
        'w' => 'ʍ',
        'x' => 'x',
        'y' => 'ʎ',
        'z' => 'z',
        '0' => '0',
        '1' => 'Ɩ',
        '2' => 'ᄅ',
        '3' => 'Ɛ',
        '4' => 'ㄣ',
        '5' => 'ʎ',
        '6' => '9',
        '7' => 'ㄥ',
        '8' => '8',
        '9' => '6',
        '?' => '¿',
        '!' => '¡',
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        ',' => '\'',
        '\'' => ',',
        _ => c,
    }
}

impl TextTools {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stat_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("在此输入文本，统计信息将实时更新...")
        });
        let input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本..."));
        let output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("处理结果..."));
        let find_state = cx.new(|cx| InputState::new(window, cx).placeholder("查找内容"));
        let replace_state = cx.new(|cx| InputState::new(window, cx).placeholder("替换内容"));
        let extract_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("分隔符或正则表达式，如逗号 或 \\d+"));
        let repeat_state = cx.new(|cx| InputState::new(window, cx).placeholder("重复次数"));

        let _subscriptions = vec![
            cx.subscribe_in(&stat_state, window, {
                let stat_state = stat_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.stat_text = stat_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.input = input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&output_state, window, {
                let output_state = output_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.output = output_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&find_state, window, {
                let find_state = find_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.find = find_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&replace_state, window, {
                let replace_state = replace_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.replace = replace_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&extract_state, window, {
                let extract_state = extract_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.extract_sep = extract_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
        ];

        Self {
            stat_text: String::new(),
            input: String::new(),
            output: String::new(),
            find: String::new(),
            replace: String::new(),
            extract_sep: String::new(),
            is_regex: false,
            message: String::new(),
            stat_state,
            input_state,
            output_state,
            find_state,
            replace_state,
            extract_state,
            repeat_state,
            _subscriptions,
        }
    }

    fn warn(&mut self, msg: &str, cx: &mut Context<Self>) {
        self.message = msg.to_string();
        cx.notify();
    }

    fn set_output(&mut self, out: String, window: &mut Window, cx: &mut Context<Self>) {
        self.output = out;
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
        cx.notify();
    }

    fn apply_str(
        &mut self,
        f: impl FnOnce(&str) -> String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.input.is_empty() {
            self.warn("请先输入内容", cx);
            return;
        }
        self.message.clear();
        let out = f(&self.input);
        self.set_output(out, window, cx);
    }

    fn apply_lines(
        &mut self,
        f: impl FnOnce(Vec<String>) -> Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_str(|s| f(split_lines(s)).join("\n"), window, cx);
    }

    // ---------------- 清理操作 ----------------

    fn op_dedupe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(
            |lines| {
                let mut seen = HashSet::new();
                lines
                    .into_iter()
                    .filter(|l| seen.insert(l.clone()))
                    .collect()
            },
            window,
            cx,
        );
    }

    fn op_remove_empty(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(
            |lines| lines.into_iter().filter(|l| !l.trim().is_empty()).collect(),
            window,
            cx,
        );
    }

    fn op_collapse_ws(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(collapse_whitespace, window, cx);
    }

    fn op_remove_all_ws(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(|s| s.chars().filter(|c| !c.is_whitespace()).collect(), window, cx);
    }

    fn op_remove_breaks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.chars().filter(|c| *c != '\n' && *c != '\r').collect(),
            window,
            cx,
        );
    }

    fn op_swap_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.chars().map(swap_case_char).collect(),
            window,
            cx,
        );
    }

    fn op_remove_punct(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.chars().filter(|c| !is_punct_like(*c)).collect(),
            window,
            cx,
        );
    }

    fn op_remove_digits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.chars().filter(|c| !c.is_ascii_digit()).collect(),
            window,
            cx,
        );
    }

    fn op_strip_html(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(strip_html, window, cx);
    }

    fn op_remove_diacritics(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(remove_diacritics, window, cx);
    }

    // ---------------- 行操作 ----------------

    fn op_sort_asc(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(|mut lines| {
            lines.sort();
            lines
        }, window, cx);
    }

    fn op_sort_desc(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(|mut lines| {
            lines.sort();
            lines.reverse();
            lines
        }, window, cx);
    }

    fn op_shuffle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(|mut lines| {
            use rand::seq::SliceRandom;
            lines.shuffle(&mut rand::rng());
            lines
        }, window, cx);
    }

    fn op_trim_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_lines(
            |lines| lines.into_iter().map(|l| l.trim().to_string()).collect(),
            window,
            cx,
        );
    }

    fn op_extract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.extract_sep.trim().is_empty() {
            self.warn("请输入分隔符或正则表达式", cx);
            return;
        }
        if self.input.is_empty() {
            self.warn("请先输入内容", cx);
            return;
        }
        let re = match regex::Regex::new(&self.extract_sep) {
            Ok(re) => re,
            Err(_) => {
                self.warn("正则表达式无效", cx);
                return;
            }
        };
        let matches: Vec<&str> = re.find_iter(&self.input).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            self.message = "未匹配到任何内容".to_string();
        } else {
            self.message.clear();
        }
        self.set_output(matches.join("\n"), window, cx);
    }

    // ---------------- 查找替换与重复 ----------------

    fn op_replace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.find.is_empty() {
            self.warn("请输入查找内容", cx);
            return;
        }
        if self.input.is_empty() {
            self.warn("请先输入内容", cx);
            return;
        }
        let out = if self.is_regex {
            match regex::Regex::new(&self.find) {
                Ok(re) => re.replace_all(&self.input, self.replace.as_str()).to_string(),
                Err(_) => {
                    self.warn("正则表达式无效", cx);
                    return;
                }
            }
        } else {
            self.input.replace(&self.find, &self.replace)
        };
        self.message.clear();
        self.set_output(out, window, cx);
    }

    fn op_repeat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.is_empty() {
            self.warn("请先输入内容", cx);
            return;
        }
        let count = self
            .repeat_state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(3)
            .clamp(1, 10000);
        self.message.clear();
        let out = self.input.repeat(count);
        self.set_output(out, window, cx);
    }

    // ---------------- 斜线与翻转 ----------------

    fn op_add_slashes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(add_slashes, window, cx);
    }

    fn op_strip_slashes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(strip_slashes, window, cx);
    }

    fn op_reverse_string(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(|s| s.chars().rev().collect(), window, cx);
    }

    fn op_reverse_words(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.split_whitespace().rev().collect::<Vec<_>>().join(" "),
            window,
            cx,
        );
    }

    fn op_upside_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_str(
            |s| s.chars().map(upside_down_char).collect(),
            window,
            cx,
        );
    }

    // ---------------- 通用操作 ----------------

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn clear_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if self.output.is_empty() {
            self.warn("内容为空", cx);
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
    }

    fn copy_stats(&mut self, cx: &mut Context<Self>) {
        let text = compute_stats(&self.stat_text)
            .iter()
            .map(|(label, value)| format!("{label}：{value}"))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn clear_stat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stat_text.clear();
        self.stat_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }

    fn clear_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.message.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        cx.notify();
    }
}

impl Render for TextTools {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header("文本工具", "常用文本处理工具", cx))
            // 统计
            .child(
                design::card(cx)
                    .child(Textarea::new(&self.stat_state).h(px(100.0)).flex_1())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_2()
                                    .flex_1()
                                    .children(compute_stats(&self.stat_text).iter().map(
                                        |(label, value)| {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .px_3()
                                                .py_1()
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_family("monospace")
                                                        .text_color(rgb(0x18a058))
                                                        .child(value.to_string()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(*label),
                                                )
                                        },
                                    )),
                            )
                            .child(
                                Button::new("copy-stats")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制统计")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_stats(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-stats")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除统计输入")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_stat(window, cx);
                                    })),
                            ),
                    ),
            )
            // 输入
            .child(
                design::card(cx)
                    .child(Textarea::new(&self.input_state).h(px(150.0)).flex_1())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                ButtonGroup::new("input-buttons")
                                    .child(
                                        Button::new("paste-input")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste_input(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-input")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_input(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("clear-all")
                                            .icon(Icon::new(IconName::Close))
                                            .tooltip("清空输入与输出")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear_all(window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            // 清理操作
            .child(
                design::card(cx)
                    .child(design::caption("清理", cx))
                    .child(
                        div().flex().flex_wrap().gap_2().child(
                            ButtonGroup::new("clean-buttons")
                                .child(Button::new("dedupe").child("去重").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_dedupe(window, cx);
                                    }),
                                ))
                                .child(Button::new("remove-empty").child("去空行").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_remove_empty(window, cx);
                                    }),
                                ))
                                .child(Button::new("collapse-ws").child("合并空格").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_collapse_ws(window, cx);
                                    }),
                                ))
                                .child(Button::new("remove-all-ws").child("删空白").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_remove_all_ws(window, cx);
                                    }),
                                ))
                                .child(Button::new("remove-breaks").child("删换行").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_remove_breaks(window, cx);
                                    }),
                                ))
                                .child(Button::new("swap-case").child("大小写互转").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_swap_case(window, cx);
                                    }),
                                ))
                                .child(Button::new("remove-punct").child("删标点").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_remove_punct(window, cx);
                                    }),
                                ))
                                .child(Button::new("remove-digits").child("删数字").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_remove_digits(window, cx);
                                    }),
                                ))
                                .child(Button::new("strip-html").child("剥离HTML").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_strip_html(window, cx);
                                    }),
                                ))
                                .child(
                                    Button::new("remove-diacritics").child("删除重音符号").on_click(
                                        cx.listener(|this, _, window, cx| {
                                            this.op_remove_diacritics(window, cx);
                                        }),
                                    ),
                                ),
                        ),
                    ),
            )
            // 行操作
            .child(
                design::card(cx)
                    .child(design::caption("行操作", cx))
                    .child(
                        div().flex().flex_wrap().gap_2().child(
                            ButtonGroup::new("line-buttons")
                                .child(Button::new("sort-asc").child("升序排序").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_sort_asc(window, cx);
                                    }),
                                ))
                                .child(Button::new("sort-desc").child("降序排序").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_sort_desc(window, cx);
                                    }),
                                ))
                                .child(Button::new("shuffle").child("随机打乱").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_shuffle(window, cx);
                                    }),
                                ))
                                .child(Button::new("trim-lines").child("去首尾空格").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_trim_lines(window, cx);
                                    }),
                                )),
                        ),
                    ),
            )
            // 提取
            .child(
                design::card(cx)
                    .child(design::caption("提取", cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(240.0)).child(Input::new(&self.extract_state)))
                            .child(
                                Button::new("extract")
                                    .label("提取")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.op_extract(window, cx);
                                    })),
                            ),
                    ),
            )
            // 查找替换
            .child(
                design::card(cx)
                    .child(design::caption("查找替换", cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(160.0)).child(Input::new(&self.find_state)))
                            .child(div().w(px(160.0)).child(Input::new(&self.replace_state)))
                            .child(
                                Checkbox::new("opt-regex")
                                    .label("正则")
                                    .checked(self.is_regex)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.is_regex = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("replace")
                                    .label("替换")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.op_replace(window, cx);
                                    })),
                            ),
                    ),
            )
            // 重复
            .child(
                design::card(cx)
                    .child(design::caption("重复", cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(120.0)).child(Input::new(&self.repeat_state)))
                            .child(
                                Button::new("repeat")
                                    .child("重复文本")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.op_repeat(window, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("将输入内容整体重复 N 次"),
                            ),
                    ),
            )
            // 翻转
            .child(
                design::card(cx)
                    .child(design::caption("翻转", cx))
                    .child(
                        div().flex().flex_wrap().gap_2().child(
                            ButtonGroup::new("flip-buttons")
                                .child(Button::new("add-slashes").child("添加斜线").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_add_slashes(window, cx);
                                    }),
                                ))
                                .child(Button::new("strip-slashes").child("去除斜线").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_strip_slashes(window, cx);
                                    }),
                                ))
                                .child(Button::new("reverse-str").child("反向字符串").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_reverse_string(window, cx);
                                    }),
                                ))
                                .child(Button::new("reverse-words").child("单词反转").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_reverse_words(window, cx);
                                    }),
                                ))
                                .child(Button::new("upside-down").child("字符倒置").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.op_upside_down(window, cx);
                                    }),
                                )),
                        ),
                    ),
            )
            // 输出
            .child(
                design::card(cx)
                    .child(Textarea::new(&self.output_state).h(px(150.0)).flex_1()),
            )
            // 输出操作
            .child(
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("copy-output")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            ),
                    )
                    // 提示
                    .when(!self.message.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xd03050))
                                        .child(self.message.clone()),
                                ),
                        )
                    }),
            )
    }
}

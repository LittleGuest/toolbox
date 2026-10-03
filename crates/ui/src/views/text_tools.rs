use std::collections::HashSet;

use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState, NumberInput, Textarea, TextareaState},
    *,
};

// ---------------------------------------------------------------------------
// 纯文本处理函数
// ---------------------------------------------------------------------------

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
    // 对齐 Tauri 的 [\p{P}]：ASCII 下需排除属于 \p{S} 的符号字符
    if c.is_ascii() {
        return c.is_ascii_punctuation()
            && !matches!(c, '$' | '+' | '<' | '=' | '>' | '^' | '`' | '|' | '~');
    }
    matches!(
        c,
        '，' | '。' | '、' | '；' | '：' | '？' | '！' | '“' | '”' | '‘' | '’' | '（' | '）'
            | '《' | '》' | '【' | '】' | '…' | '—' | '·' | '｜'
    )
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn compute_stats(s: &str) -> Vec<(&'static str, usize)> {
    let line_count = if s.is_empty() {
        0
    } else {
        split_lines(s).len()
    };
    vec![
        ("字符数", s.chars().count()),
        ("单词数", s.trim().split_whitespace().count()),
        ("行数", line_count),
        ("非空白字符数", s.chars().filter(|c| !c.is_whitespace()).count()),
        ("字节数 (UTF-8)", s.len()),
        ("中文字符数", s.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count()),
        ("英文字母数", s.chars().filter(|c| c.is_ascii_alphabetic()).count()),
        ("数字个数", s.chars().filter(|c| c.is_ascii_digit()).count()),
        ("标点符号数", s.chars().filter(|c| is_punct_like(*c)).count()),
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

fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_word = false;
    for c in s.chars() {
        if is_word_char(c) {
            if prev_word {
                out.extend(c.to_lowercase());
            } else {
                out.extend(c.to_uppercase());
            }
        } else {
            out.push(c);
        }
        prev_word = is_word_char(c);
    }
    out
}

fn sentence_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = true;
    for c in s.chars() {
        if upper_next && !c.is_whitespace() {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
            if matches!(c, '.' | '!' | '?' | '。' | '！' | '？') {
                upper_next = true;
            }
        }
    }
    out
}

fn camel_pascal_case(s: &str, pascal: bool) -> String {
    let mut out = String::new();
    let mut cap_next = pascal;
    let mut started = false;
    for c in s.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            if started {
                cap_next = true;
            }
            continue;
        }
        started = true;
        if cap_next {
            out.extend(c.to_uppercase());
            cap_next = false;
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

fn snake_kebab_case(s: &str, sep: char) -> String {
    let mut out = String::new();
    let mut in_sep = false;
    for c in s.trim().chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            in_sep = true;
        } else {
            if in_sep && !out.is_empty() {
                out.push(sep);
            }
            in_sep = false;
            out.push(c);
        }
    }
    out
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

// ---------------------------------------------------------------------------
// 视图状态
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Success,
    Warn,
    Error,
    Info,
}

pub struct TextTools {
    tab: usize,
    is_regex: bool,
    message: String,
    tone: Tone,
    stat_state: Entity<TextareaState>,
    clean_input_state: Entity<TextareaState>,
    clean_output_state: Entity<TextareaState>,
    sort_input_state: Entity<TextareaState>,
    sort_output_state: Entity<TextareaState>,
    extract_state: Entity<InputState>,
    find_input_state: Entity<TextareaState>,
    find_output_state: Entity<TextareaState>,
    find_state: Entity<InputState>,
    replace_state: Entity<InputState>,
    repeat_state: Entity<InputState>,
    slash_input_state: Entity<TextareaState>,
    slash_output_state: Entity<TextareaState>,
    case_input_state: Entity<TextareaState>,
    case_output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

const TABS: [&str; 6] = [
    "字符统计",
    "清理工具",
    "排序与提取",
    "查找替换与重复",
    "斜线与翻转",
    "大小写转换",
];

impl TextTools {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stat_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("在此输入文本，统计信息将实时更新")
        });
        let clean_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本"));
        let clean_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("处理结果"));
        let sort_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("每行一条数据，按行处理"));
        let sort_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("处理结果"));
        let extract_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("分隔符或正则表达式，如逗号 或 \\d+")
        });
        let find_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本"));
        let find_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("处理结果"));
        let find_state = cx.new(|cx| InputState::new(window, cx).placeholder("查找内容"));
        let replace_state = cx.new(|cx| InputState::new(window, cx).placeholder("替换内容"));
        let repeat_state =
            cx.new(|cx| InputState::new(window, cx).default_value("3".to_string()));
        let slash_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入文本"));
        let slash_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("处理结果"));
        let case_input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入要转换的文本"));
        let case_output_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果"));

        // 对齐 Tauri：提取分隔符输入框支持回车直接触发提取
        let _subscriptions = vec![cx.subscribe_in(&extract_state, window, {
            move |this, _, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = ev {
                    this.op_extract(window, cx);
                }
            }
        })];

        Self {
            tab: 0,
            is_regex: false,
            message: String::new(),
            tone: Tone::Info,
            stat_state,
            clean_input_state,
            clean_output_state,
            sort_input_state,
            sort_output_state,
            extract_state,
            find_input_state,
            find_output_state,
            find_state,
            replace_state,
            repeat_state,
            slash_input_state,
            slash_output_state,
            case_input_state,
            case_output_state,
            _subscriptions,
        }
    }

    fn set_msg(&mut self, tone: Tone, msg: &str, cx: &mut Context<Self>) {
        self.tone = tone;
        self.message = msg.to_string();
        cx.notify();
    }

    fn clear_msg(&mut self, cx: &mut Context<Self>) {
        self.message.clear();
        cx.notify();
    }

    fn copy_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if value.is_empty() {
            self.set_msg(Tone::Warn, "内容为空", cx);
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        self.set_msg(Tone::Success, "复制成功", cx);
    }

    /// 统一处理：读指定输入 -> 变换 -> 写指定输出（对齐 Vue cleanApply / sortApply）
    fn apply_pair<F>(
        input: &Entity<TextareaState>,
        output: &Entity<TextareaState>,
        f: F,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        F: FnOnce(&str) -> String,
    {
        let value = input.read(cx).value().to_string();
        let result = f(&value);
        output.update(cx, |state, cx| {
            state.set_value(result, window, cx);
        });
    }

    fn apply_pair_lines<F>(
        input: &Entity<TextareaState>,
        output: &Entity<TextareaState>,
        f: F,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        F: FnOnce(Vec<String>) -> Vec<String>,
    {
        Self::apply_pair(input, output, |s| f(split_lines(s)).join("\n"), window, cx);
    }

    /// 清理工具：clean 输入 -> clean 输出
    fn clean_apply<F>(&mut self, f: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce(&str) -> String,
    {
        Self::apply_pair(&self.clean_input_state, &self.clean_output_state, f, window, cx);
        self.clear_msg(cx);
    }

    fn clean_apply_lines<F>(&mut self, f: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce(Vec<String>) -> Vec<String>,
    {
        Self::apply_pair_lines(
            &self.clean_input_state,
            &self.clean_output_state,
            f,
            window,
            cx,
        );
        self.clear_msg(cx);
    }

    /// 排序与提取：sort 输入 -> sort 输出
    fn sort_apply_lines<F>(&mut self, f: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce(Vec<String>) -> Vec<String>,
    {
        Self::apply_pair_lines(
            &self.sort_input_state,
            &self.sort_output_state,
            f,
            window,
            cx,
        );
        self.clear_msg(cx);
    }

    fn clear_pair(
        input: &Entity<TextareaState>,
        output: &Entity<TextareaState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        input.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        output.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    // -- 清理工具 -----------------------------------------------------------

    fn op_dedupe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply_lines(
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
        self.clean_apply_lines(
            |lines| lines.into_iter().filter(|l| !l.trim().is_empty()).collect(),
            window,
            cx,
        );
    }

    fn op_collapse_ws(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(collapse_whitespace, window, cx);
    }

    fn op_remove_all_ws(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(|s| s.chars().filter(|c| !c.is_whitespace()).collect(), window, cx);
    }

    fn op_remove_breaks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(
            |s| s.chars().filter(|c| *c != '\n' && *c != '\r').collect(),
            window,
            cx,
        );
    }

    fn op_remove_diacritics(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(remove_diacritics, window, cx);
    }

    fn op_remove_punct(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(|s| s.chars().filter(|c| !is_punct_like(*c)).collect(), window, cx);
    }

    fn op_remove_digits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(
            |s| s.chars().filter(|c| !c.is_ascii_digit()).collect(),
            window,
            cx,
        );
    }

    fn op_strip_html(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clean_apply(strip_html, window, cx);
    }

    // -- 排序与提取 ---------------------------------------------------------

    fn op_sort_asc(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sort_apply_lines(|mut lines| { lines.sort(); lines }, window, cx);
    }

    fn op_sort_desc(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sort_apply_lines(
            |mut lines| {
                lines.sort();
                lines.reverse();
                lines
            },
            window,
            cx,
        );
    }

    fn op_shuffle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sort_apply_lines(
            |mut lines| {
                use rand::seq::SliceRandom;
                lines.shuffle(&mut rand::rng());
                lines
            },
            window,
            cx,
        );
    }

    fn op_trim_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sort_apply_lines(
            |lines| lines.into_iter().map(|l| l.trim().to_string()).collect(),
            window,
            cx,
        );
    }

    fn op_extract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let sep = self.extract_state.read(cx).value().to_string();
        if sep.trim().is_empty() {
            self.set_msg(Tone::Warn, "请输入分隔符或正则表达式", cx);
            return;
        }
        let re = match regex::Regex::new(&sep) {
            Ok(re) => re,
            Err(_) => {
                self.set_msg(Tone::Error, "正则表达式无效", cx);
                return;
            }
        };
        let input = self.sort_input_state.read(cx).value().to_string();
        let matches: Vec<&str> = re.find_iter(&input).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            self.set_msg(Tone::Info, "未匹配到任何内容", cx);
        } else {
            self.clear_msg(cx);
        }
        let joined = matches.join("\n");
        self.sort_output_state.update(cx, |state, cx| {
            state.set_value(joined, window, cx);
        });
    }

    // -- 查找替换与重复 -----------------------------------------------------

    fn op_replace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let find = self.find_state.read(cx).value().to_string();
        if find.is_empty() {
            self.set_msg(Tone::Warn, "请输入查找内容", cx);
            return;
        }
        let replace = self.replace_state.read(cx).value().to_string();
        let input = self.find_input_state.read(cx).value().to_string();
        let out = if self.is_regex {
            match regex::Regex::new(&find) {
                Ok(re) => re.replace_all(&input, replace.as_str()).to_string(),
                Err(_) => {
                    self.set_msg(Tone::Error, "正则表达式无效", cx);
                    return;
                }
            }
        } else {
            input.replace(&find, &replace)
        };
        self.clear_msg(cx);
        self.find_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
    }

    fn op_repeat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.find_input_state.read(cx).value().to_string();
        let count = self
            .repeat_state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(1)
            .clamp(1, 10000);
        let out = input.repeat(count);
        self.clear_msg(cx);
        self.find_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
    }

    // -- 斜线与翻转 ---------------------------------------------------------

    fn op_add_slashes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.slash_input_state.read(cx).value().to_string();
        let out = add_slashes(&input);
        self.slash_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
        self.clear_msg(cx);
    }

    fn op_strip_slashes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.slash_input_state.read(cx).value().to_string();
        let out = strip_slashes(&input);
        self.slash_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
        self.clear_msg(cx);
    }

    fn op_reverse_string(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.slash_input_state.read(cx).value().to_string();
        let out: String = input.chars().rev().collect();
        self.slash_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
        self.clear_msg(cx);
    }

    fn op_reverse_words(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.slash_input_state.read(cx).value().to_string();
        let out = input.split_whitespace().rev().collect::<Vec<_>>().join(" ");
        self.slash_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
        self.clear_msg(cx);
    }

    fn op_upside_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.slash_input_state.read(cx).value().to_string();
        let out: String = input.chars().map(upside_down_char).collect();
        self.slash_output_state.update(cx, |state, cx| {
            state.set_value(out, window, cx);
        });
        self.clear_msg(cx);
    }

    // -- 大小写转换（对齐 CaseConverter.vue） --------------------------------

    fn op_case<F>(&mut self, f: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce(&str) -> String,
    {
        let input = self.case_input_state.read(cx).value().to_string();
        let output = f(&input);
        self.case_output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
        self.clear_msg(cx);
    }

    fn op_paste_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.case_input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
            }
        }
    }

    // -- 输出区复制 / 清除 ---------------------------------------------------

    fn clear_clean(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::clear_pair(&self.clean_input_state, &self.clean_output_state, window, cx);
        self.clear_msg(cx);
    }

    fn clear_sort(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::clear_pair(&self.sort_input_state, &self.sort_output_state, window, cx);
        self.extract_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.clear_msg(cx);
    }

    fn clear_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::clear_pair(&self.find_input_state, &self.find_output_state, window, cx);
        self.find_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.replace_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.repeat_state.update(cx, |state, cx| {
            state.set_value("3".to_string(), window, cx);
        });
        self.clear_msg(cx);
    }

    fn clear_slash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::clear_pair(&self.slash_input_state, &self.slash_output_state, window, cx);
        self.clear_msg(cx);
    }

    fn clear_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::clear_pair(&self.case_input_state, &self.case_output_state, window, cx);
        self.clear_msg(cx);
    }

    fn clear_stat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stat_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.clear_msg(cx);
    }

    fn copy_stats(&mut self, cx: &mut Context<Self>) {
        let text = self.stat_state.read(cx).value().to_string();
        let summary = compute_stats(&text)
            .iter()
            .map(|(label, value)| format!("{label}：{value}"))
            .collect::<Vec<_>>()
            .join("\n");
        cx.write_to_clipboard(ClipboardItem::new_string(summary));
        self.set_msg(Tone::Success, "复制成功", cx);
    }
}

// ---------------------------------------------------------------------------
// 渲染
// ---------------------------------------------------------------------------

impl TextTools {
    fn tab_btn(
        &self,
        idx: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let active = self.tab == idx;
        div()
            .id(("tt-tab", idx))
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
                this.tab = idx;
                this.clear_msg(cx);
                cx.notify();
            }))
            .child(TABS[idx].to_string())
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .border_b_1()
            .border_color(cx.theme().border)
            .mb_4()
            .children((0..TABS.len()).map(|i| self.tab_btn(i, cx)))
    }

    /// tb-toolbar：复制输出 + 清除
    fn output_toolbar(
        &self,
        id: &'static str,
        output: &Entity<TextareaState>,
        on_clear: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let output = output.clone();
        let copy_id: SharedString = format!("{id}-copy").into();
        let clear_id: SharedString = format!("{id}-clear").into();
        div()
            .flex()
            .items_center()
            .gap_2()
            .mt_2()
            .child(
                Button::new(copy_id)
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Copy))
                    .tooltip("复制输出")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let value = output.read(cx).value().to_string();
                        this.copy_value(&value, cx);
                    })),
            )
            .child(
                Button::new(clear_id)
                    .ghost()
                    .compact()
                    .icon(Icon::new(IconName::Close))
                    .tooltip("清除")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        on_clear(this, window, cx);
                    })),
            )
    }

    // -- Tab 1：字符统计 -----------------------------------------------------

    fn render_stats(&mut self, cx: &mut Context<Self>) -> Div {
        let stat_text = self.stat_state.read(cx).value().to_string();
        let stats = compute_stats(&stat_text);

        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.stat_state)
                            .h(px(180.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(5)
                    .gap_2p5()
                    .children(stats.iter().map(|(label, value)| {
                        div()
                            .flex_col()
                            .items_center()
                            .px(px(14.0))
                            .py(px(12.0))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().background)
                            .child(
                                div()
                                    .text_size(px(20.0))
                                    .font_bold()
                                    .font_family("monospace")
                                    .text_color(cx.theme().primary)
                                    .child(value.to_string()),
                            )
                            .child(
                                div()
                                    .mt(px(4.0))
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(*label),
                            )
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("copy-stats")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制统计")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_stats(cx);
                            })),
                    )
                    .child(
                        Button::new("clear-stats")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_stat(window, cx);
                            })),
                    ),
            )
    }

    // -- Tab 2：清理工具 -----------------------------------------------------

    fn render_clean(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.clean_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("dedupe")
                            .compact()
                            .label("去重")
                            .tooltip("删除重复行")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_dedupe(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-empty")
                            .compact()
                            .label("去空行")
                            .tooltip("删除空行")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_empty(window, cx);
                            })),
                    )
                    .child(
                        Button::new("collapse-ws")
                            .compact()
                            .label("合空格")
                            .tooltip("合并多余空格")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_collapse_ws(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-all-ws")
                            .compact()
                            .label("删空白")
                            .tooltip("删除全部空白")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_all_ws(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-breaks")
                            .compact()
                            .label("删换行")
                            .tooltip("删除换行符")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_breaks(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-diacritics")
                            .compact()
                            .label("删重音")
                            .tooltip("删除重音符号")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_diacritics(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-punct")
                            .compact()
                            .label("删标点")
                            .tooltip("删除标点符号")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_punct(window, cx);
                            })),
                    )
                    .child(
                        Button::new("remove-digits")
                            .compact()
                            .label("删数字")
                            .tooltip("删除数字")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_remove_digits(window, cx);
                            })),
                    )
                    .child(
                        Button::new("strip-html")
                            .compact()
                            .label("去HTML")
                            .tooltip("剥离 HTML 标签")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_strip_html(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.clean_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    )
                    .child(self.output_toolbar("tt-clean", &self.clean_output_state, |this, window, cx| {
                        this.clear_clean(window, cx);
                    }, cx)),
            )
    }

    // -- Tab 3：排序与提取 ---------------------------------------------------

    fn render_sort(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.sort_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("sort-asc")
                            .compact()
                            .label("升序")
                            .tooltip("按行升序排序")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_sort_asc(window, cx);
                            })),
                    )
                    .child(
                        Button::new("sort-desc")
                            .compact()
                            .label("降序")
                            .tooltip("按行降序排序")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_sort_desc(window, cx);
                            })),
                    )
                    .child(
                        Button::new("shuffle")
                            .compact()
                            .label("打乱")
                            .tooltip("随机打乱行")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_shuffle(window, cx);
                            })),
                    )
                    .child(
                        Button::new("trim-lines")
                            .compact()
                            .label("去首尾")
                            .tooltip("每行去首尾空格")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_trim_lines(window, cx);
                            })),
                    ),
            )
            .child(div().border_t_1().border_color(cx.theme().border))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("分隔符 / 正则"),
                    )
                    .child(
                        div()
                            .w(px(360.0))
                            .max_w_full()
                            .font_family("monospace")
                            .child(Input::new(&self.extract_state).cleanable(true)),
                    )
                    .child(
                        Button::new("extract")
                            .primary()
                            .compact()
                            .label("提取")
                            .tooltip("按分隔符 / 正则提取")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_extract(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.sort_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    )
                    .child(self.output_toolbar("tt-sort", &self.sort_output_state, |this, window, cx| {
                        this.clear_sort(window, cx);
                    }, cx)),
            )
    }

    // -- Tab 4：查找替换与重复 -----------------------------------------------

    fn render_find(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.find_input_state)
                            .h(px(130.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(220.0))
                            .font_family("monospace")
                            .child(Input::new(&self.find_state)),
                    )
                    .child(
                        div()
                            .w(px(220.0))
                            .font_family("monospace")
                            .child(Input::new(&self.replace_state)),
                    )
                    .child(
                        Checkbox::new("tt-regex")
                            .label("正则")
                            .checked(self.is_regex)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.is_regex = *checked;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("replace")
                            .primary()
                            .compact()
                            .icon(Icon::new(IconName::Replace))
                            .tooltip("替换")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_replace(window, cx);
                            })),
                    ),
            )
            .child(div().border_t_1().border_color(cx.theme().border))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(120.0))
                            .child(NumberInput::new(&self.repeat_state)),
                    )
                    .child(
                        Button::new("repeat")
                            .compact()
                            .label("重复文本")
                            .tooltip("将输入整体重复 N 次")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_repeat(window, cx);
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("将上方输入整体重复 N 次"),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.find_output_state)
                            .h(px(130.0))
                            .font_family("monospace"),
                    )
                    .child(self.output_toolbar("tt-find", &self.find_output_state, |this, window, cx| {
                        this.clear_find(window, cx);
                    }, cx)),
            )
    }

    // -- Tab 5：斜线与翻转 ---------------------------------------------------

    fn render_slash(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输入", cx))
                    .child(
                        Textarea::new(&self.slash_input_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("add-slashes")
                            .compact()
                            .label("加斜线")
                            .tooltip("添加斜线")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_add_slashes(window, cx);
                            })),
                    )
                    .child(
                        Button::new("strip-slashes")
                            .compact()
                            .label("去斜线")
                            .tooltip("去除斜线")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_strip_slashes(window, cx);
                            })),
                    )
                    .child(
                        Button::new("reverse-str")
                            .compact()
                            .label("反转")
                            .tooltip("反向字符串")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_reverse_string(window, cx);
                            })),
                    )
                    .child(
                        Button::new("reverse-words")
                            .compact()
                            .label("词序反转")
                            .tooltip("单词顺序反转")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_reverse_words(window, cx);
                            })),
                    )
                    .child(
                        Button::new("upside-down")
                            .compact()
                            .label("倒置")
                            .tooltip("字符倒置")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_upside_down(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1p5()
                    .child(design::editor_label("输出", cx))
                    .child(
                        Textarea::new(&self.slash_output_state)
                            .h(px(150.0))
                            .font_family("monospace"),
                    )
                    .child(self.output_toolbar("tt-slash", &self.slash_output_state, |this, window, cx| {
                        this.clear_slash(window, cx);
                    }, cx)),
            )
    }

    // -- Tab 6：大小写转换 ---------------------------------------------------

    fn render_case(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("case-upper")
                            .compact()
                            .label("AA")
                            .tooltip("全部大写")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| s.to_uppercase(), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-lower")
                            .compact()
                            .label("aa")
                            .tooltip("全部小写")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| s.to_lowercase(), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-title")
                            .primary()
                            .compact()
                            .label("Aa")
                            .tooltip("单词首字母大写")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(title_case, window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-sentence")
                            .compact()
                            .label("Abc")
                            .tooltip("句子首字母大写")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(sentence_case, window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-camel")
                            .compact()
                            .label("camelCase")
                            .tooltip("驼峰 camelCase")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| camel_pascal_case(s, false), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-pascal")
                            .compact()
                            .label("PascalCase")
                            .tooltip("帕斯卡 PascalCase")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| camel_pascal_case(s, true), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-snake")
                            .compact()
                            .label("snake_case")
                            .tooltip("蛇形 snake_case")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| snake_kebab_case(s, '_'), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-kebab")
                            .compact()
                            .label("kebab-case")
                            .tooltip("烤肉串 kebab-case")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| snake_kebab_case(s, '-'), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-swap")
                            .compact()
                            .label("aA")
                            .tooltip("反转大小写")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(
                                    |s| s.chars().map(swap_case_char).collect(),
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new("case-reverse")
                            .compact()
                            .label("反转")
                            .tooltip("反转文本")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(|s| s.chars().rev().collect(), window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-upside")
                            .compact()
                            .label("倒置")
                            .tooltip("字符倒置")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_case(
                                    |s| s.chars().map(upside_down_char).collect(),
                                    window,
                                    cx,
                                );
                            })),
                    ),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .min_w_0()
                            .child(design::editor_label("输入", cx))
                            .child(
                                Textarea::new(&self.case_input_state)
                                    .h(px(200.0))
                                    .font_family("monospace"),
                            ),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .min_w_0()
                            .child(design::editor_label("输出", cx))
                            .child(
                                Textarea::new(&self.case_output_state)
                                    .h(px(200.0))
                                    .font_family("monospace"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("case-paste")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Inbox))
                            .tooltip("粘贴到输入")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.op_paste_case(window, cx);
                            })),
                    )
                    .child(
                        Button::new("case-copy-in")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输入")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let value = this.case_input_state.read(cx).value().to_string();
                                this.copy_value(&value, cx);
                            })),
                    )
                    .child(
                        Button::new("case-copy-out")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let value = this.case_output_state.read(cx).value().to_string();
                                this.copy_value(&value, cx);
                            })),
                    )
                    .child(
                        Button::new("case-clear")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_case(window, cx);
                            })),
                    ),
            )
    }
}

impl Render for TextTools {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = self.tab;
        let message = self.message.clone();
        let tone = self.tone;

        design::page()

            .child(
                design::card(cx)
                    .child(self.tab_bar(cx))
                    .children(match tab {
                        0 => vec![self.render_stats(cx).into_any_element()],
                        1 => vec![self.render_clean(cx).into_any_element()],
                        2 => vec![self.render_sort(cx).into_any_element()],
                        3 => vec![self.render_find(cx).into_any_element()],
                        4 => vec![self.render_slash(cx).into_any_element()],
                        _ => vec![self.render_case(cx).into_any_element()],
                    })
                    .when(!message.is_empty(), |card| {
                        let color = match tone {
                            Tone::Success => Hsla::from(rgb(design::OK_GREEN)),
                            Tone::Warn => Hsla::from(rgb(design::WARN_AMBER)),
                            Tone::Error => Hsla::from(rgb(design::ERROR_RED)),
                            Tone::Info => cx.theme().muted_foreground,
                        };
                        card.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(color)
                                .child(message),
                        )
                    }),
            )
    }
}

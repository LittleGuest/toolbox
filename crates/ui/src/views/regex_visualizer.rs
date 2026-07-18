use std::ops::Range;

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    *,
};
use regex::Regex;

use crate::views::syntax_highlight::{self, HighlightPalette};

#[derive(Clone)]
struct RegexMatch {
    index: usize,
    start: usize,
    end: usize,
    text: String,
    groups: Vec<String>,
}

pub struct RegexVisualizer {
    pattern: String,
    test_text: String,
    explanation: Vec<String>,
    matches: Vec<RegexMatch>,
    error: String,
    pattern_state: Entity<InputState>,
    text_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl RegexVisualizer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pattern_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入正则表达式，例如 (?P<word>\\w+)")
                .multi_line(false)
                .default_value(r"\w+".to_string())
        });
        let text_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入测试文本...")
                .multi_line(true)
                .default_value("hello gpui\n123 toolbox".to_string())
        });

        let _subscriptions = vec![
            cx.subscribe_in(&pattern_state, window, {
                let pattern_state = pattern_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.pattern = pattern_state.read(cx).value().to_string();
                        this.evaluate();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&text_state, window, {
                let text_state = text_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.test_text = text_state.read(cx).value().to_string();
                        this.evaluate();
                        cx.notify();
                    }
                }
            }),
        ];

        let mut this = Self {
            pattern: r"\w+".to_string(),
            test_text: "hello gpui\n123 toolbox".to_string(),
            explanation: Vec::new(),
            matches: Vec::new(),
            error: String::new(),
            pattern_state,
            text_state,
            _subscriptions,
        };
        this.evaluate();
        this
    }

    fn evaluate(&mut self) {
        self.error.clear();
        self.matches.clear();
        self.explanation = explain_regex(&self.pattern);

        if self.pattern.is_empty() {
            return;
        }

        let regex = match Regex::new(&self.pattern) {
            Ok(regex) => regex,
            Err(err) => {
                self.error = err.to_string();
                return;
            }
        };

        for (index, captures) in regex.captures_iter(&self.test_text).enumerate() {
            if let Some(full) = captures.get(0) {
                let groups = captures
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(group_index, value)| match value {
                        Some(value) => format!("#{group_index}: {}", value.as_str()),
                        None => format!("#{group_index}: <未匹配>"),
                    })
                    .collect();
                self.matches.push(RegexMatch {
                    index: index + 1,
                    start: full.start(),
                    end: full.end(),
                    text: full.as_str().to_string(),
                    groups,
                });
            }
        }
    }

    fn paste_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.test_text = text.to_string();
                self.text_state.update(cx, |state, cx| {
                    state.set_value(self.test_text.clone(), window, cx);
                });
                self.evaluate();
            }
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pattern.clear();
        self.test_text.clear();
        self.explanation.clear();
        self.matches.clear();
        self.error.clear();
        self.pattern_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.text_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
    }
}

impl Render for RegexVisualizer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            div()
                .grid()
                .grid_cols(2)
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_sm().font_semibold().child("表达式"))
                        .child(Input::new(&self.pattern_state))
                        .child(div().text_sm().font_semibold().child("测试文本"))
                        .child(Input::new(&self.text_state).h(px(220.0)))
                        .child(
                            ButtonGroup::new("regex-actions")
                                .child(
                                    Button::new("regex-run")
                                        .primary()
                                        .icon(Icon::new(IconName::Search))
                                        .tooltip("执行匹配")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.evaluate();
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("regex-paste")
                                        .icon(Icon::new(IconName::File))
                                        .tooltip("粘贴测试文本")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.paste_text(window, cx);
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("regex-copy")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制匹配摘要")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                this.summary(),
                                            ));
                                        })),
                                )
                                .child(
                                    Button::new("regex-clear")
                                        .icon(Icon::new(IconName::Delete))
                                        .tooltip("清空")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear(window, cx);
                                            cx.notify();
                                        })),
                                ),
                        )
                        .when(!self.error.is_empty(), |this| {
                            this.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().danger)
                                    .child(self.error.clone()),
                            )
                        })
                        .child(highlight_preview_panel(&self.test_text, &self.matches, cx)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(result_panel("语法说明", self.explanation.clone(), cx))
                        .child(match_panel(self.matches.clone(), cx)),
                ),
        )
    }
}

impl RegexVisualizer {
    fn summary(&self) -> String {
        if !self.error.is_empty() {
            return format!("正则错误：{}", self.error);
        }

        let mut lines = vec![format!("匹配数量：{}", self.matches.len())];
        for item in &self.matches {
            lines.push(format!(
                "#{} [{}..{}] {}",
                item.index, item.start, item.end, item.text
            ));
            lines.extend(item.groups.iter().map(|group| format!("  {group}")));
        }
        lines.join("\n")
    }
}

fn result_panel(title: &'static str, lines: Vec<String>, cx: &mut Context<RegexVisualizer>) -> Div {
    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().font_semibold().child(title))
        .child(
            div()
                .h(px(150.0))
                .overflow_y_scrollbar()
                .text_sm()
                .children(if lines.is_empty() {
                    vec![div().child("暂无说明")]
                } else {
                    lines
                        .into_iter()
                        .map(|line| div().mb_1().child(line))
                        .collect::<Vec<_>>()
                }),
        )
}

fn match_panel(matches: Vec<RegexMatch>, cx: &mut Context<RegexVisualizer>) -> Div {
    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().font_semibold().child("匹配结果"))
        .child(
            div()
                .h(px(290.0))
                .overflow_y_scrollbar()
                .children(if matches.is_empty() {
                    vec![div().text_sm().child("暂无匹配")]
                } else {
                    matches
                        .into_iter()
                        .map(|item| {
                            div()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .p_2()
                                .mb_2()
                                .child(div().text_sm().font_semibold().child(format!(
                                    "#{} [{}..{}] {}",
                                    item.index, item.start, item.end, item.text
                                )))
                                .children(item.groups.into_iter().map(|group| {
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(group)
                                }))
                        })
                        .collect::<Vec<_>>()
                }),
        )
}

fn explain_regex(pattern: &str) -> Vec<String> {
    if pattern.is_empty() {
        return Vec::new();
    }

    let rules = [
        (r"\d", r"\d：数字字符"),
        (r"\w", r"\w：字母、数字或下划线"),
        (r"\s", r"\s：空白字符"),
        (".", ".：任意字符（默认不含换行）"),
        ("*", "*：重复 0 次或多次"),
        ("+", "+：重复 1 次或多次"),
        ("?", "?：可选或非贪婪修饰"),
        ("|", "|：或分支"),
        ("^", "^：文本开始或行开始"),
        ("$", "$：文本结束或行结束"),
        ("[", "[]：字符集合"),
        ("(", "()：捕获组或分组"),
        ("(?P<", "(?P<name>...)：命名捕获组"),
    ];

    let mut lines = Vec::new();
    for (token, description) in rules {
        if pattern.contains(token) {
            lines.push(description.to_string());
        }
    }

    if lines.is_empty() {
        lines.push("普通字符：按字面值匹配".to_string());
    }

    lines
}

/// 渲染匹配高亮预览面板，将匹配的文本区间用高亮色着色
fn highlight_preview_panel(
    text: &str,
    matches: &[RegexMatch],
    cx: &mut Context<RegexVisualizer>,
) -> Div {
    // 构建高亮区间：匹配部分用强调色，非匹配部分用默认前景色
    let palette = HighlightPalette::default_light();
    let highlight_color = palette.boolean; // 红色突出匹配
    let mut ranges: Vec<Range<usize>> = matches.iter().map(|m| m.start..m.end).collect();
    // 合并重叠区间并排序，确保高亮渲染正确
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for r in ranges {
        if let Some(last) = merged.last_mut() {
            if r.start <= last.end {
                last.end = last.end.max(r.end);
                continue;
            }
        }
        merged.push(r);
    }

    let highlight_ranges: Vec<syntax_highlight::HighlightRange> = merged
        .into_iter()
        .map(|r| syntax_highlight::HighlightRange {
            range: r,
            color: highlight_color,
        })
        .collect();

    let styled = syntax_highlight::styled_text(text, highlight_ranges);

    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().font_semibold().child("匹配高亮预览"))
        .child(
            div()
                .h(px(120.0))
                .overflow_y_scrollbar()
                .text_sm()
                .font_family("monospace")
                .when(text.is_empty(), |this| {
                    this.text_color(cx.theme().muted_foreground)
                        .child("暂无测试文本")
                })
                .when(!text.is_empty(), |this| this.child(styled)),
        )
}

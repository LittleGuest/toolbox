use crate::design;
use std::ops::Range;

use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    scroll::ScrollableElement,
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

/// 轨道图节点（由 regex_syntax AST 构建）
enum RRNode {
    Literal(String),
    Class(String),
    Dot,
    Assertion(String),
    Group {
        index: Option<u32>,
        child: Box<RRNode>,
    },
    Repeat {
        child: Box<RRNode>,
        min: u32,
        max: Option<u32>,
    },
    Concat(Vec<RRNode>),
    Alternate(Vec<RRNode>),
    Empty,
}

#[derive(Clone)]
struct RegexMatch {
    index: usize,
    start: usize,
    end: usize,
    text: String,
    groups: Vec<String>,
}

/// flags 与 regex crate RegexBuilder 的对应：i m s U x
const FLAGS: [(usize, char, &'static str, usize); 5] = [
    (0, 'i', "忽略大小写", 0),
    (1, 'm', "多行模式（^ $ 匹配行首行尾）", 1),
    (2, 's', "点号匹配换行符", 2),
    (3, 'U', "交换贪婪（* 变懒惰）", 3),
    (4, 'x', "扩展模式（忽略空白与 # 注释）", 4),
];

const RR_LINE: f32 = 16.0;

pub struct RegexVisualizer {
    pattern: String,
    test_text: String,
    flags: [bool; 5],
    diagram: Option<RRNode>,
    matches: Vec<RegexMatch>,
    error: String,
    pattern_state: Entity<InputState>,
    text_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl RegexVisualizer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pattern_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入正则表达式，例如 (?P<word>\\w+)")
                .default_value(r"\w+".to_string())
        });
        let text_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("输入测试文本...")
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
            flags: [false; 5],
            diagram: None,
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
        self.diagram = parse_diagram(&self.pattern);

        if self.pattern.is_empty() {
            return;
        }

        let regex = match regex::RegexBuilder::new(&self.pattern)
            .case_insensitive(self.flags[0])
            .multi_line(self.flags[1])
            .dot_matches_new_line(self.flags[2])
            .swap_greed(self.flags[3])
            .ignore_whitespace(self.flags[4])
            .build()
        {
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

    fn toggle_flag(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(slot) = self.flags.get_mut(index) {
            *slot = !*slot;
        }
        self.evaluate();
        cx.notify();
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

    fn copy_pattern(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.pattern.clone()));
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pattern.clear();
        self.test_text.clear();
        self.diagram = None;
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

fn parse_diagram(pattern: &str) -> Option<RRNode> {
    use regex_syntax::ast;
    match ast::parse::Parser::new().parse(pattern) {
        Ok(ast) => Some(build_rr(&ast, pattern)),
        Err(_) => None,
    }
}

fn ast_text(pattern: &str, span: &regex_syntax::ast::Span) -> String {
    pattern
        .get(span.start.offset..span.end.offset)
        .unwrap_or("")
        .to_string()
}

fn build_rr(ast: &regex_syntax::ast::Ast, pattern: &str) -> RRNode {
    use regex_syntax::ast::{Ast, GroupKind, RepetitionKind, RepetitionRange};
    match ast {
        Ast::Empty(_) => RRNode::Empty,
        Ast::Flags(_) => RRNode::Assertion("(?flags)".to_string()),
        Ast::Literal(lit) => RRNode::Literal(ast_text(pattern, &lit.span)),
        Ast::Dot(_) => RRNode::Dot,
        Ast::Assertion(a) => RRNode::Assertion(ast_text(pattern, &a.span)),
        Ast::ClassUnicode(c) => RRNode::Class(ast_text(pattern, &c.span)),
        Ast::ClassPerl(c) => RRNode::Class(ast_text(pattern, &c.span)),
        Ast::ClassBracketed(c) => RRNode::Class(ast_text(pattern, &c.span)),
        Ast::Repetition(rep) => {
            let (min, max) = match &rep.op.kind {
                RepetitionKind::ZeroOrMore => (0, None),
                RepetitionKind::OneOrMore => (1, None),
                RepetitionKind::ZeroOrOne => (0, Some(1)),
                RepetitionKind::Range(range) => match range {
                    RepetitionRange::Exactly(n) => (*n, Some(*n)),
                    RepetitionRange::AtLeast(n) => (*n, None),
                    RepetitionRange::Bounded(a, b) => (*a, Some(*b)),
                },
            };
            RRNode::Repeat {
                child: Box::new(build_rr(&rep.ast, pattern)),
                min,
                max,
            }
        }
        Ast::Group(g) => RRNode::Group {
            index: match &g.kind {
                GroupKind::CaptureIndex(index) => Some(*index),
                GroupKind::CaptureName { name, .. } => Some(name.index),
                GroupKind::NonCapturing(_) => None,
            },
            child: Box::new(build_rr(&g.ast, pattern)),
        },
        Ast::Alternation(a) => RRNode::Alternate(
            a.asts
                .iter()
                .filter(|c| !matches!(c, Ast::Empty(_)))
                .map(|c| build_rr(c, pattern))
                .collect(),
        ),
        Ast::Concat(c) => RRNode::Concat(
            c.asts
                .iter()
                .filter(|c| !matches!(c, Ast::Empty(_)))
                .map(|c| build_rr(c, pattern))
                .collect(),
        ),
    }
}

// ---------- 轨道图渲染 ----------

fn rr_hline(cx: &App) -> Div {
    div().w(px(RR_LINE)).h(px(1.0)).bg(cx.theme().border)
}

fn rr_hline_flex(cx: &App) -> Div {
    div().flex_1().h(px(1.0)).bg(cx.theme().border)
}

fn rr_rail(cx: &App) -> Div {
    // 分支左右连接竖线列
    div()
        .w(px(RR_LINE))
        .relative()
        .child(
            div()
                .absolute()
                .left(px((RR_LINE - 1.0) / 2.0))
                .top_0()
                .bottom_0()
                .w(px(1.0))
                .bg(cx.theme().border),
        )
}

fn rr_repeat_label(label: String, cx: &App) -> Div {
    div()
        .px(px(6.0))
        .py(px(1.0))
        .rounded_full()
        .bg(design::tint(0x4f6ef7, 0.1))
        .text_size(px(10.0))
        .font_family("monospace")
        .text_color(cx.theme().primary)
        .child(label)
}

fn rr_terminal(text: String, color: Hsla, cx: &App) -> Div {
    div()
        .max_w(px(260.0))
        .truncate()
        .px(px(8.0))
        .py(px(3.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(color)
        .bg(color.opacity(0.08))
        .font_family("monospace")
        .text_size(px(13.0))
        .text_color(cx.theme().foreground)
        .child(text)
}

fn rr_render(node: &RRNode, cx: &App) -> Div {
    match node {
        RRNode::Empty => div(),
        RRNode::Literal(text) => rr_terminal(text.clone(), cx.theme().primary, cx),
        RRNode::Class(text) => {
            rr_terminal(text.clone(), Hsla::from(rgb(design::ACCENT)), cx)
        }
        RRNode::Dot => rr_terminal("•".to_string(), Hsla::from(rgb(design::OK_GREEN)), cx),
        RRNode::Assertion(text) => {
            rr_terminal(text.clone(), Hsla::from(rgb(design::WARN_AMBER)), cx)
        }
        RRNode::Group { index, child } => {
            let inner = rr_render(child, cx);
            match index {
                Some(group_index) => div()
                    .relative()
                    .child(
                        div()
                            .px(px(4.0))
                            .py(px(3.0))
                            .rounded(px(8.0))
                            .border_dashed()
                            .border_1()
                            .border_color(design::tint(design::ACCENT, 0.45))
                            .child(inner),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(-7.0))
                            .left(px(8.0))
                            .px_1()
                            .text_size(px(9.0))
                            .text_color(design::tint(design::ACCENT, 0.9))
                            .bg(cx.theme().popover)
                            .child(format!("组{group_index}")),
                    ),
                None => inner,
            }
        }
        RRNode::Repeat { child, min, max } => {
            let label = match (min, max) {
                (0, Some(1)) => "?".to_string(),
                (0, None) => "*".to_string(),
                (1, None) => "+".to_string(),
                (a, Some(b)) => format!("{{{a},{b}}}"),
                (a, None) => format!("{{{a},}}"),
            };
            let repeatable = max.is_none() || *max > Some(1);
            div()
                .flex()
                .flex_col()
                .when(repeatable, |col| {
                    col.child(
                        div()
                            .flex()
                            .items_center()
                            .child(rr_hline(cx))
                            .child(rr_hline_flex(cx))
                            .child(rr_repeat_label(label, cx))
                            .child(div().w(px(RR_LINE))),
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(rr_hline(cx))
                        .child(rr_render(child, cx))
                        .child(rr_hline(cx)),
                )
                .when(*min == 0, |col| {
                    col.child(
                        div()
                            .flex()
                            .items_center()
                            .child(rr_hline(cx))
                            .child(rr_hline_flex(cx))
                            .child(rr_repeat_label("0".to_string(), cx))
                            .child(div().w(px(RR_LINE))),
                    )
                })
        }
        RRNode::Concat(items) => {
            let mut row = div().flex().items_center();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    row = row.child(rr_hline(cx));
                }
                row = row.child(rr_render(item, cx));
            }
            row
        }
        RRNode::Alternate(branches) => {
            let mut column = div().flex().flex_col().gap(px(10.0));
            if branches.is_empty() {
                column = column.child(div().w(px(RR_LINE)).h(px(1.0)).bg(cx.theme().border));
            }
            for branch in branches {
                column = column.child(
                    div()
                        .flex()
                        .items_center()
                        .child(rr_hline(cx))
                        .child(rr_render(branch, cx))
                        .child(rr_hline(cx)),
                );
            }
            div()
                .flex()
                .child(rr_rail(cx))
                .child(column)
                .child(rr_rail(cx))
        }
    }
}

impl Render for RegexVisualizer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let matches = self.matches.clone();
        let match_count = self.matches.len();
        let error = self.error.clone();
        let diagram = self.diagram.as_ref();

        let mut flag_row = div().flex().items_center().gap_1();
        for (index, label, tooltip, flag_index) in FLAGS {
            let active = self.flags[flag_index];
            flag_row = flag_row.child(
                Button::new(("regex-flag", index))
                    .label(label.to_string())
                    .compact()
                    .when(active, |btn| btn.selected(true))
                    .tooltip(tooltip)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_flag(flag_index, cx);
                    })),
            );
        }

        // 对齐 regex-vis 应用式三段布局 + Vue 全高卡片（tb-page 最后一个子元素撑满剩余高度）：
        // 顶段 输入+flags → 中段 轨道图(flex_1) → 底段 测试区
        div().h_full().flex_col().child(
            design::card(cx)
                .flex_1()
                .min_h_0()
                .p(px(8.0))
                .gap_2()
                // 顶段：/ pattern / + flags（regex-vis 顶栏）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_lg()
                                .font_family("monospace")
                                .text_color(cx.theme().muted_foreground)
                                .child("/"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .child(Input::new(&self.pattern_state).h(px(38.0))),
                        )
                        .child(
                            div()
                                .text_lg()
                                .font_family("monospace")
                                .text_color(cx.theme().muted_foreground)
                                .child("/"),
                        )
                        .child(flag_row)
                        .child(
                            Button::new("regex-copy")
                                .icon(Icon::new(IconName::Copy))
                                .ghost()
                                .compact()
                                .tooltip("复制正则")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.copy_pattern(window, cx);
                                })),
                        ),
                )
                .when(!error.is_empty(), |card| {
                    card.child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2()
                            .py(px(4.0))
                            .rounded(px(6.0))
                            .bg(design::tint(design::ERROR_RED, 0.08))
                            .child(
                                Icon::new(IconName::TriangleAlert)
                                    .size(px(14.0))
                                    .text_color(Hsla::from(rgb(design::ERROR_RED))),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child(error.clone()),
                            ),
                    )
                })
                // 中段：轨道图占满剩余高度
                .child(
                    div()
                        .flex_1()
                        .min_h(px(180.0))
                        .overflow_scrollbar()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().background)
                        .p_4()
                        .child(
                            div()
                                .min_w_full()
                                .flex()
                                .justify_center()
                                .items_center()
                                .py_2()
                                .children(match diagram {
                                    Some(node) => vec![rr_render(node, cx)],
                                    None => vec![design::hint("输入有效正则后显示轨道图", cx)],
                                }),
                        ),
                )
                // 底段：测试区（固定高度）
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(design::editor_label("测试文本", cx))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(design::pill(format!("{match_count} 个匹配"), cx))
                                        .child(
                                            Button::new("regex-paste")
                                                .icon(Icon::new(IconName::File))
                                                .ghost()
                                                .compact()
                                                .tooltip("粘贴测试文本")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.paste_text(window, cx);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Button::new("regex-clear")
                                                .icon(Icon::new(IconName::Delete))
                                                .ghost()
                                                .compact()
                                                .tooltip("清空全部")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.clear(window, cx);
                                                    cx.notify();
                                                })),
                                        ),
                                ),
                        )
                        .child(Textarea::new(&self.text_state).h(px(90.0)))
                        .child(highlight_preview_panel(&self.test_text, &matches, cx))
                        .child(match_panel(matches, cx)),
                ),
        )
    }
}

fn match_panel(matches: Vec<RegexMatch>, cx: &mut Context<RegexVisualizer>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(design::editor_label("匹配结果", cx))
        .child(
            div()
                .max_h(px(220.0))
                .overflow_y_scrollbar()
                .children(if matches.is_empty() {
                    vec![design::hint("暂无匹配", cx)]
                } else {
                    matches
                        .into_iter()
                        .map(|item| {
                            div()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded(px(8.0))
                                .px_2()
                                .py_1p5()
                                .mb_1p5()
                                .flex()
                                .items_baseline()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_family("monospace")
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("#{} [{}..{}]", item.index, item.start, item.end)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_family("monospace")
                                        .font_semibold()
                                        .child(item.text),
                                )
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

fn highlight_preview_panel(
    text: &str,
    matches: &[RegexMatch],
    cx: &mut Context<RegexVisualizer>,
) -> Div {
    let palette = HighlightPalette::default_light();
    let highlight_color = palette.boolean;
    let mut ranges: Vec<Range<usize>> = matches.iter().map(|m| m.start..m.end).collect();
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
        .flex()
        .flex_col()
        .gap_1p5()
        .child(design::editor_label("匹配高亮", cx))
        .child(
            div()
                .max_h(px(140.0))
                .overflow_y_scrollbar()
                .rounded(px(8.0))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_2()
                .text_sm()
                .font_family("monospace")
                .when(text.is_empty(), |this| {
                    this.text_color(cx.theme().muted_foreground)
                        .child("暂无测试文本")
                })
                .when(!text.is_empty(), |this| this.child(styled)),
        )
}

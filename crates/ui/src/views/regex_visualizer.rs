use std::ops::Range;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    tab::{Tab, TabBar},
    *,
};

use crate::views::syntax_highlight::{self, HighlightPalette};

// ===========================================================================
// 轨道图几何常量
//
// 全部取自 regex-vis 官方 SVG 的实测值（见 .workbuddy/memory 内对齐规格笔记）。
// 单位与官方一致，均为 CSS px。
// ===========================================================================

/// 节点框高（官方 `rect height="28"`）
const NODE_H: f32 = 28.0;
/// 节点框圆角（官方 `rx="5"`）
const NODE_R: f32 = 5.0;
/// 节点文字左右内边距（官方 rect x=85 / foreignObject x=95）
const NODE_PAD_X: f32 = 10.0;
/// 节点字号（官方 `font-size="16"`）
const NODE_FS: f32 = 16.0;
/// 装饰标签字号（官方 `font-size="14"`）
const LABEL_FS: f32 = 14.0;
/// 装饰标签的绘制行高（官方 1.5 × 14，foreignObject height="21"）
const LABEL_H: f32 = 21.0;
/// 装饰标签/量词行在布局盒里占的高度（官方常量 `uj` / `cj`）
const LABEL_BOX: f32 = 16.0;
/// 重复标签两侧图标宽度（官方 18×18）
const ICON_W: f32 = 18.0;
/// 相邻节点之间的连接线长度（官方 137.8 → 162.8）
const LINK: f32 = 25.0;
/// 分支相对容器的水平缩进（官方 85 → 110）
const BRANCH_INDENT: f32 = 25.0;
/// 分支转弯圆角半径（官方 `A5 5 0 0 0`）
const BRANCH_R: f32 = 5.0;
/// 分支行间距（官方 119 → 134）
const ROW_GAP: f32 = 15.0;
/// 分支容器上下内边距（官方 81 → 91）
const BRANCH_PAD_V: f32 = 10.0;
/// 分组框左右内边距（官方 85 → 110）
const GROUP_PAD_H: f32 = 25.0;
/// 分组框上下内边距（官方 66 → 81）
const GROUP_PAD_V: f32 = 15.0;
/// 画布根内边距（官方根布局：上/下各 50，左/右各 50 + 标记直径 + 连接线）
const ROOT_PAD: f32 = 50.0;
/// 叶子节点的最小内容尺寸（官方 `F0`：min width 20 / min height 26）
const MIN_W: f32 = 20.0;
const MIN_H: f32 = 26.0;
/// 起点/终点圆环半径（官方 `circle r="5"`）
const MARK_R: f32 = 5.0;
/// 描边宽度（官方 `stroke-[1.5]`）
const STROKE: f32 = 1.5;
/// 官方 CSS 变量 --graph
const GRAPH: u32 = 0x3f3f46;
/// 官方 CSS 变量 --graph-group
const GRAPH_GROUP: u32 = 0xa1a1aa;
/// 官方 CSS 变量 --graph-bg
const GRAPH_BG: u32 = 0xfafafa;
/// 节点文字色（官方 text-foreground）
const GRAPH_FG: u32 = 0x09090b;
/// 取反字符类描边色（官方 tailwind `stroke-red-500`）
const GRAPH_RED: u32 = 0xef4444;
/// 选中高亮色（官方 tailwind `fill-blue-500/30`）
const SELECT_BLUE: u32 = 0x3b82f6;
/// 选中高亮的透明度（tailwind 的 `/30`）
const SELECT_ALPHA: f32 = 0.3;

/// 官方 `hd`：选中时在节点矩形上再叠一层 `fill-blue-500/30` 的圆角矩形
fn select_highlight(x: f32, y: f32, w: f32, h: f32) -> Prim {
    Prim::Highlight {
        x,
        y,
        w,
        h,
        r: NODE_R,
    }
}

/// 对齐 regex-vis yA：输入行下方的复选框组（g/i/m/s）
/// 中文列对齐官网 cn 语言包（Global search / Case-insensitive / Multi-line / Allows . to match newline）
const FLAGS: [(usize, char, &'static str, &'static str); 4] = [
    (0, 'g', "全局搜索", "Global search"),
    (1, 'i', "忽略大小写", "Case-insensitive"),
    (2, 'm', "多行", "Multi-line"),
    (3, 's', "允许 . 匹配换行符", "Allows . to match newline"),
];

/// 对齐 regex-vis HB 样本列表（右面板 Samples 页签，共 6 项）
const SAMPLES: [(&str, &str, &str); 6] = [
    ("1. 整数", "1. Whole Numbers", r"^\d+$"),
    ("2. 小数", "2. Decimal Numbers", r"^\d*\.\d+$"),
    ("3. 整数 + 小数", "3. Whole + Decimal Numbers", r"^\d*(\.\d+)?$"),
    (
        "4. 正负 整数 + 小数",
        "4. Negative, Positive Whole + Decimal Numbers",
        r"^-?\d*(\.\d+)?$",
    ),
    (
        "5. Url",
        "5. Url",
        r"^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#()?&//=]*)$",
    ),
    (
        "6. 日期格式 YYYY-MM-dd",
        "6. Date Format YYYY-MM-dd",
        r"^[12]\d{3}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$",
    ),
];

/// 右面板页签顺序，对齐 regex-vis zB：Legends / Edit / Test / Samples
const PANEL_TABS: [(&str, &str); 4] = [
    ("图例", "Legends"),
    ("编辑", "Edit"),
    ("测试", "Test"),
    ("样例", "Samples"),
];
const TAB_LEGEND: usize = 0;
const TAB_EDIT: usize = 1;
const TAB_TEST: usize = 2;

// ===========================================================================
// 语言
// ===========================================================================

/// 界面语言。官方用 i18next 支持 en/cn/ru/jp/fr/tr 六种，这里只保留中英两种：
/// 中文文案逐条对齐官网 `/locales/cn/translation.json`（115 条），
/// 英文文案就是官方 i18next 的 key 本身（官方 fallbackLng="en" 且没有 en 资源文件，
/// 所以英文态下 `t("Any digit")` 返回的就是 "Any digit"）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Cn,
    En,
}

impl Lang {
    fn of(self, cn: &'static str, en: &'static str) -> &'static str {
        match self {
            Lang::Cn => cn,
            Lang::En => en,
        }
    }

    fn toggled(self) -> Self {
        match self {
            Lang::Cn => Lang::En,
            Lang::En => Lang::Cn,
        }
    }

    /// 切换按钮上的文字：显示的是**当前**语言
    fn badge(self) -> &'static str {
        match self {
            Lang::Cn => "中",
            Lang::En => "EN",
        }
    }

    fn tooltip(self) -> &'static str {
        match self {
            Lang::Cn => "切换为 English",
            Lang::En => "切换为中文",
        }
    }
}

/// 官方 jn 映射表（字符类 → 说明文案），中文列取自官网 cn 语言包。
/// `\f` 官方未给中文译名，回退成英文原文，与官网行为一致。
const CLASS_LABELS: [(&str, &str, &str); 13] = [
    (".", "任意字符", "Any character"),
    (r"\d", "任意数字", "Any digit"),
    (r"\D", "任意非数字", "Non-digit"),
    (r"\w", "任意基本拉丁字母数字", "Any alphanumeric"),
    (r"\W", "任意非基本拉丁字母数字", "Non-alphanumeric"),
    (r"\s", "任意空白字符", "White space"),
    (r"\S", "任意非空白字符", "Non-white space"),
    (r"\t", "制表符", "Horizontal tab"),
    (r"\r", "回车符", "Carriage return"),
    (r"\n", "换行符", "Linefeed"),
    (r"\v", "垂直制表符", "Vertical tab"),
    (r"\f", "Form-feed", "Form-feed"),
    (r"\0", "NUL", "NUL"),
];

fn class_label(raw: &str, lang: Lang) -> Option<&'static str> {
    CLASS_LABELS
        .iter()
        .find(|(key, _, _)| *key == raw)
        .map(|(_, cn, en)| lang.of(cn, en))
}

/// 字面量转义字符 → 文案
const LITERAL_LABELS: [(char, &str, &str); 6] = [
    ('\n', "换行符", "Linefeed"),
    ('\r', "回车符", "Carriage return"),
    ('\t', "制表符", "Horizontal tab"),
    ('\u{0b}', "垂直制表符", "Vertical tab"),
    ('\u{0c}', "Form-feed", "Form-feed"),
    ('\0', "NUL", "NUL"),
];

/// 分组标签用的两种前缀
fn group_word(lang: Lang) -> &'static str {
    lang.of("组", "Group")
}

/// 零宽断言的文案。官方 vh：
/// `{beginning:"Begins with", end:"Ends with", lookahead:["Followed by:","Not followed by:"],
///   lookbehind:["Preceded by:","Not preceded by:"], word:["WordBoundary","NonWordBoundary"]}`
///
/// 注意 `WordBoundary` / `NonWordBoundary` 在英文态下就是这两个 key 本身
/// （官网 cn 语言包给了「单词边界」「非单词边界」，英文无资源文件故回退为 key）。
fn boundary_label(lang: Lang, negated: bool) -> &'static str {
    match (lang, negated) {
        (Lang::Cn, false) => "单词边界",
        (Lang::Cn, true) => "非单词边界",
        (Lang::En, false) => "WordBoundary",
        (Lang::En, true) => "NonWordBoundary",
    }
}

fn beginning_label(lang: Lang) -> &'static str {
    lang.of("以...开始", "Begins with")
}

fn ending_label(lang: Lang) -> &'static str {
    lang.of("以...结束", "Ends with")
}

/// 前瞻 / 后顾断言的分组标签（官方把环视渲染成**一个带标签的普通分组**）
fn lookaround_label(lang: Lang, ahead: bool, negated: bool) -> &'static str {
    match (lang, ahead, negated) {
        (Lang::Cn, true, false) => "接着:",
        (Lang::Cn, true, true) => "不接着:",
        (Lang::Cn, false, false) => "前面是:",
        (Lang::Cn, false, true) => "前面不是:",
        (Lang::En, true, false) => "Followed by:",
        (Lang::En, true, true) => "Not followed by:",
        (Lang::En, false, false) => "Preceded by:",
        (Lang::En, false, true) => "Not preceded by:",
    }
}

fn one_of_label(lang: Lang) -> &'static str {
    lang.of("其一", "One of")
}

fn none_of_label(lang: Lang) -> &'static str {
    lang.of("没有其一", "None of")
}

fn empty_label(lang: Lang) -> &'static str {
    lang.of("空", "Empty")
}

// ===========================================================================
// 可编辑 AST
//
// 官方 regex-vis 的编辑能力建立在它**自己的** AST 上（不是解析库的），共七种节点：
//
//   regex                —— 根：body + flags
//   choice               —— 分支：branches: [[...], [...]]
//   group                —— capturing / nonCapturing / namedCapturing
//   lookAroundAssertion  —— lookahead / lookbehind，带 negate
//   character            —— string / class / ranges
//   backReference        —— \1 或 \k<name>
//   boundaryAssertion    —— beginning / end / word
//
// 三处与「包一层修饰节点」写法不同的关键点：
//
//  1. **量词挂在节点上**（`quantifier` 字段），不额外套 repeat 节点。
//     官方 `y0(node)` 只有 character / group / backReference 能带量词。
//  2. **没有 concat 节点**：序列就是数组本身（body / children / branches[i]）。
//  3. 每个节点带稳定 `id`；选中、表达式高亮、所有变更操作都按 id 定位。
//
// `regex-syntax 0.8.11` 的 AST 没有 backReference / lookAround 变体
// （官方用的是 JS 版解析器），所以这两个变体只会由图例示意手工构造出来。
// ===========================================================================

/// 节点 id。0 =「无 id」——图例里手工构造的示意节点用。
type NodeId = u32;

/// 自增 id 发生器，对应官方 `Yt()`
#[derive(Default)]
struct IdGen(u32);

impl IdGen {
    fn next(&mut self) -> NodeId {
        self.0 += 1;
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CharKind {
    String,
    Class,
    Ranges,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EGroupKind {
    Capturing,
    NonCapturing,
    NamedCapturing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LookKind {
    Lookahead,
    Lookbehind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BoundaryKind {
    Beginning,
    End,
    Word,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum QuantKind {
    Star,
    Plus,
    Question,
    Custom,
}

/// 量词，对应官方 `{kind, min, max, greedy}`。`max == u32::MAX` 即无上界（`Infinity`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Quantifier {
    kind: QuantKind,
    min: u32,
    max: u32,
    greedy: bool,
}

impl Quantifier {
    const INF: u32 = u32::MAX;

    fn infinite(&self) -> bool {
        self.max == Self::INF
    }
}

#[derive(Clone, Debug)]
enum EKind {
    Character {
        kind: CharKind,
        value: String,
        ranges: Vec<(String, String)>,
        negate: bool,
    },
    BackReference {
        reference: String,
    },
    Boundary {
        kind: BoundaryKind,
        negate: bool,
    },
    Group {
        kind: EGroupKind,
        name: String,
        children: Vec<ENode>,
    },
    LookAround {
        kind: LookKind,
        negate: bool,
        children: Vec<ENode>,
    },
    Choice {
        branches: Vec<Vec<ENode>>,
    },
}

#[derive(Clone, Debug)]
struct ENode {
    id: NodeId,
    quantifier: Option<Quantifier>,
    kind: EKind,
}

impl ENode {
    fn new(id: NodeId, kind: EKind) -> Self {
        Self {
            id,
            quantifier: None,
            kind,
        }
    }

    /// 官方 `y0(node)`：只有这三种节点能带量词
    fn accepts_quantifier(&self) -> bool {
        matches!(
            self.kind,
            EKind::Character { .. } | EKind::Group { .. } | EKind::BackReference { .. }
        )
    }

    fn children(&self) -> Option<&Vec<ENode>> {
        match &self.kind {
            EKind::Group { children, .. } | EKind::LookAround { children, .. } => Some(children),
            _ => None,
        }
    }

    /// 官方 `H8` 的判定：节点带 `negate` 字段且为真 → 红色虚线描边
    /// （字符范围、单词边界断言、环视断言三种）
    fn dashed(&self) -> bool {
        match &self.kind {
            EKind::Character { negate, .. } => *negate,
            EKind::Boundary { kind, negate } => matches!(kind, BoundaryKind::Word) && *negate,
            EKind::LookAround { negate, .. } => *negate,
            _ => false,
        }
    }
}

// —— 构造器 ——

fn e_string(id: NodeId, value: impl Into<String>) -> ENode {
    ENode::new(
        id,
        EKind::Character {
            kind: CharKind::String,
            value: value.into(),
            ranges: Vec::new(),
            negate: false,
        },
    )
}

fn e_class(id: NodeId, value: impl Into<String>) -> ENode {
    ENode::new(
        id,
        EKind::Character {
            kind: CharKind::Class,
            value: value.into(),
            ranges: Vec::new(),
            negate: false,
        },
    )
}

fn e_ranges(id: NodeId, ranges: Vec<(String, String)>, negate: bool) -> ENode {
    ENode::new(
        id,
        EKind::Character {
            kind: CharKind::Ranges,
            value: String::new(),
            ranges,
            negate,
        },
    )
}

#[allow(dead_code)] // 反向引用暂不可解析（regex-syntax 的 AST 里没有这种节点），先留着模型
fn e_backref(id: NodeId, reference: impl Into<String>) -> ENode {
    ENode::new(
        id,
        EKind::BackReference {
            reference: reference.into(),
        },
    )
}

fn e_boundary(id: NodeId, kind: BoundaryKind, negate: bool) -> ENode {
    ENode::new(id, EKind::Boundary { kind, negate })
}

fn e_group(
    id: NodeId,
    kind: EGroupKind,
    name: impl Into<String>,
    children: Vec<ENode>,
) -> ENode {
    ENode::new(
        id,
        EKind::Group {
            kind,
            name: name.into(),
            children,
        },
    )
}

fn e_look(id: NodeId, kind: LookKind, negate: bool, children: Vec<ENode>) -> ENode {
    ENode::new(
        id,
        EKind::LookAround {
            kind,
            negate,
            children,
        },
    )
}

fn e_choice(id: NodeId, branches: Vec<Vec<ENode>>) -> ENode {
    ENode::new(id, EKind::Choice { branches })
}

/// 根。对应官方 `He` store 的 `{id:"", type:"regex", body:[], flags:[]}`。
/// `flags` / `literal` / `escapeBackslash` 三个字段这套应用用不到，故省略
/// （输入行的 g/i/m/s 是**测试**开关，与官方那个 `/pattern/flags` 不是一回事）。
#[derive(Clone, Debug, Default)]
struct ERoot {
    body: Vec<ENode>,
}

// ===========================================================================
// regex_syntax::Ast → 可编辑 AST
// ===========================================================================

/// 官方 `jn` 表里没有的字符就是「可合并的裸字符」（渲染成 `"abc"` 这种引号串）；
/// 表里的（`\n`、`\t` …）各自渲染成对应文案，且不能与相邻裸字符合并。
fn is_bare_literal(c: char) -> bool {
    !LITERAL_LABELS.iter().any(|(ch, _, _)| *ch == c)
}

/// 把一个**节点位**转成可编辑节点。`Concat` / `Alternation` 之外的变体都在这里处理。
fn convert_ast(ast: &regex_syntax::ast::Ast, src: &str, id_gen: &mut IdGen) -> ENode {
    use regex_syntax::ast::Ast;

    match ast {
        // 官方没有 Empty 节点：空串就是 `character/string` 且 value 为 ""
        // （`V8` 里 `value===""` 时渲染 `t("Empty")`）
        Ast::Empty(_) => e_string(id_gen.next(), ""),
        Ast::Flags(_) => e_class(id_gen.next(), ast_text(src, ast_span(ast))),
        Ast::Literal(lit) => {
            if is_bare_literal(lit.c) {
                e_string(id_gen.next(), lit.c.to_string())
            } else {
                // 非裸字面量（`\n` 等）保留**源码写法**，这样序列化能原样还原
                e_class(id_gen.next(), ast_text(src, &lit.span))
            }
        }
        Ast::Dot(_) => e_class(id_gen.next(), "."),
        Ast::Assertion(a) => {
            use regex_syntax::ast::AssertionKind as K;
            let (kind, negate) = match a.kind {
                K::StartLine | K::StartText => (BoundaryKind::Beginning, false),
                K::EndLine | K::EndText => (BoundaryKind::End, false),
                K::WordBoundary | K::WordBoundaryStart | K::WordBoundaryStartAngle => {
                    (BoundaryKind::Word, false)
                }
                K::NotWordBoundary | K::WordBoundaryEnd | K::WordBoundaryEndAngle => {
                    (BoundaryKind::Word, true)
                }
                _ => (BoundaryKind::Word, false),
            };
            e_boundary(id_gen.next(), kind, negate)
        }
        Ast::ClassUnicode(c) => e_class(id_gen.next(), ast_text(src, &c.span)),
        Ast::ClassPerl(c) => e_class(id_gen.next(), ast_text(src, &c.span)),
        Ast::ClassBracketed(c) => {
            let mut ranges: Vec<(String, String)> = Vec::new();
            collect_ranges(&c.kind, &mut ranges);
            e_ranges(id_gen.next(), ranges, c.negated)
        }
        Ast::Repetition(rep) => {
            let mut node = convert_ast(&rep.ast, src, id_gen);
            // `rep.greedy` 就是末尾那个 `?` 标记（`a+?` 是懒惰的）。写死 `true`
            // 会让 `a+?` 往返成 `a+`，所以这里必须跟着解析结果走。
            let greedy = rep.greedy;
            node.quantifier = Some(match &rep.op.kind {
                regex_syntax::ast::RepetitionKind::ZeroOrMore => Quantifier {
                    kind: QuantKind::Star,
                    min: 0,
                    max: Quantifier::INF,
                    greedy,
                },
                regex_syntax::ast::RepetitionKind::OneOrMore => Quantifier {
                    kind: QuantKind::Plus,
                    min: 1,
                    max: Quantifier::INF,
                    greedy,
                },
                regex_syntax::ast::RepetitionKind::ZeroOrOne => Quantifier {
                    kind: QuantKind::Question,
                    min: 0,
                    max: 1,
                    greedy,
                },
                regex_syntax::ast::RepetitionKind::Range(range) => {
                    let (min, max) = match range {
                        regex_syntax::ast::RepetitionRange::Exactly(n) => (*n, *n),
                        regex_syntax::ast::RepetitionRange::AtLeast(n) => (*n, Quantifier::INF),
                        regex_syntax::ast::RepetitionRange::Bounded(a, b) => (*a, *b),
                    };
                    Quantifier {
                        kind: QuantKind::Custom,
                        min,
                        max,
                        greedy,
                    }
                }
            });
            node
        }
        Ast::Group(g) => {
            let children = convert_seq(&g.ast, src, id_gen);
            match &g.kind {
                regex_syntax::ast::GroupKind::CaptureIndex(_) => {
                    e_group(id_gen.next(), EGroupKind::Capturing, "", children)
                }
                // 官方 `v8`：具名分组的标签用**名字**，不是索引
                regex_syntax::ast::GroupKind::CaptureName { name, .. } => {
                    e_group(id_gen.next(), EGroupKind::NamedCapturing, &name.name, children)
                }
                regex_syntax::ast::GroupKind::NonCapturing(_) => {
                    e_group(id_gen.next(), EGroupKind::NonCapturing, "", children)
                }
            }
        }
        Ast::Alternation(a) => {
            let branches = a.asts.iter().map(|c| convert_seq(c, src, id_gen)).collect();
            e_choice(id_gen.next(), branches)
        }
        // 不会出现：序列位一律走 convert_seq。留个兜底，包成非捕获组以免丢内容。
        Ast::Concat(c) => {
            let children = if c.asts.len() == 1 {
                convert_seq(&c.asts[0], src, id_gen)
            } else {
                let mut all = Vec::new();
                for child in &c.asts {
                    all.extend(convert_seq(child, src, id_gen));
                }
                all
            };
            e_group(id_gen.next(), EGroupKind::NonCapturing, "", children)
        }
    }
}

/// 把一个**序列位**转成节点列表。官方没有 concat 节点，所以这里就地铺平：
/// · `Concat` → 逐项转换，`Empty` 丢弃，**连续裸字面量合并**成一个 `string`
/// · 其它     → 单元素列表
fn convert_seq(ast: &regex_syntax::ast::Ast, src: &str, id_gen: &mut IdGen) -> Vec<ENode> {
    use regex_syntax::ast::Ast;

    if let Ast::Concat(c) = ast {
        let mut out: Vec<ENode> = Vec::new();
        let mut pending = String::new();
        for child in &c.asts {
            match child {
                Ast::Empty(_) => {}
                Ast::Literal(lit) if is_bare_literal(lit.c) => pending.push(lit.c),
                _ => {
                    if !pending.is_empty() {
                        out.push(e_string(id_gen.next(), std::mem::take(&mut pending)));
                    }
                    out.push(convert_ast(child, src, id_gen));
                }
            }
        }
        if !pending.is_empty() {
            out.push(e_string(id_gen.next(), pending));
        }
        return out;
    }

    vec![convert_ast(ast, src, id_gen)]
}

fn ast_span(ast: &regex_syntax::ast::Ast) -> &regex_syntax::ast::Span {
    use regex_syntax::ast::Ast;
    match ast {
        Ast::Empty(s) | Ast::Dot(s) => s,
        Ast::Flags(f) => &f.span,
        Ast::Literal(l) => &l.span,
        Ast::Assertion(l) => &l.span,
        Ast::ClassUnicode(l) => &l.span,
        Ast::ClassPerl(l) => &l.span,
        Ast::ClassBracketed(l) => &l.span,
        Ast::Repetition(r) => &r.span,
        Ast::Group(g) => &g.span,
        Ast::Alternation(a) => &a.span,
        Ast::Concat(c) => &c.span,
    }
}

/// 字符组 → 结构化区间列表（`(from, to)`；单字符时 from == to）。
/// 复刻官方解析器的产物：`[abc]` → 三段 `a-a`、`b-b`、`c-c`。
fn collect_ranges(node: &regex_syntax::ast::ClassSet, out: &mut Vec<(String, String)>) {
    use regex_syntax::ast::ClassSet;
    match node {
        ClassSet::Item(item) => collect_ranges_item(item, out),
        ClassSet::BinaryOp(op) => {
            collect_ranges(&op.lhs, out);
            collect_ranges(&op.rhs, out);
        }
    }
}

fn collect_ranges_item(item: &regex_syntax::ast::ClassSetItem, out: &mut Vec<(String, String)>) {
    use regex_syntax::ast::ClassSetItem;
    match item {
        ClassSetItem::Literal(lit) => out.push((lit.c.to_string(), lit.c.to_string())),
        ClassSetItem::Range(r) => {
            out.push((r.start.c.to_string(), r.end.c.to_string()));
        }
        ClassSetItem::Bracketed(b) => collect_ranges(&b.kind, out),
        ClassSetItem::Perl(p) => {
            use regex_syntax::ast::ClassPerlKind as K;
            let raw = match p.kind {
                K::Digit => r"\d",
                K::Space => r"\s",
                K::Word => r"\w",
            };
            out.push((raw.to_string(), raw.to_string()));
        }
        ClassSetItem::Unicode(u) => {
            let name = format!("\\p{}", unicode_class_name(&u.kind));
            out.push((name.clone(), name));
        }
        ClassSetItem::Ascii(a) => {
            use regex_syntax::ast::ClassAsciiKind as K;
            let raw = match a.kind {
                K::Alnum | K::Word => r"\w",
                K::Digit => r"\d",
                K::Space => r"\s",
                _ => "class",
            };
            out.push((raw.to_string(), raw.to_string()));
        }
        ClassSetItem::Union(u) => {
            for i in &u.items {
                collect_ranges_item(i, out);
            }
        }
        ClassSetItem::Empty(_) => {}
    }
}

// ===========================================================================
// 序列化：可编辑 AST → 正则串
//
// 对齐官方 `M4.id_gen()`，取这套应用实际用到的默认项
// （`literal = false`、`escapeBackslash = false`，因此反斜杠就是单个 `\`）。
// 带 `head`/`tail` 时顺带记录这两点的字符下标，用于表达式段的高亮
// （官方用一个子类 `qA` 覆盖 `genNode` 来做这件事）。
// ===========================================================================

/// 官方 `prefix()` 的转义集合：`/[|\\{}()[\]^$+*?.]/g`
const LITERAL_ESCAPES: [char; 14] = [
    '|', '\\', '{', '}', '(', ')', '[', ']', '^', '$', '+', '*', '?', '.',
];

struct Ser {
    out: String,
    head: NodeId,
    tail: NodeId,
    start: usize,
    end: usize,
}

impl Ser {
    fn plain() -> Self {
        Self {
            out: String::new(),
            head: 0,
            tail: 0,
            start: 0,
            end: 0,
        }
    }

    fn tracking(head: NodeId, tail: NodeId) -> Self {
        Self {
            head,
            tail,
            ..Self::plain()
        }
    }

    fn nodes(&mut self, nodes: &[ENode]) {
        for node in nodes {
            self.node(node);
        }
    }

    /// 官方 `qA.genNode`：进入节点前记 start，写完量词后记 end
    fn node(&mut self, node: &ENode) {
        if node.id != 0 && node.id == self.head {
            self.start = self.out.len();
        }
        self.kind(node);
        if node.accepts_quantifier() {
            if let Some(q) = &node.quantifier {
                self.quantifier(q);
            }
        }
        if node.id != 0 && node.id == self.tail {
            self.end = self.out.len();
        }
    }

    fn kind(&mut self, node: &ENode) {
        match &node.kind {
            EKind::Character {
                kind,
                value,
                ranges,
                negate,
            } => match kind {
                CharKind::String => {
                    for c in value.chars() {
                        if LITERAL_ESCAPES.contains(&c) {
                            self.out.push('\\');
                        }
                        self.out.push(c);
                    }
                }
                // 官方 `genCharacter` 的 class 分支：
                // `regex += value === "." ? value : this.characterClassPrefix + value`
                // 而本应用 `escapeBackslash = false`，故 `characterClassPrefix` 求值为
                // **空串** —— 也就是说取值原样输出。取值本身已自带反斜杠
                // （`\d` / `\w` / `\p{Greek}`，与 `jn` 表的 key 一致），
                // 这里再补一个就变成 `\\d` 了。
                CharKind::Class => self.out.push_str(value),
                CharKind::Ranges => {
                    self.out.push_str(if *negate { "[^" } else { "[" });
                    let last = ranges.len().saturating_sub(1);
                    for (i, (from, to)) in ranges.iter().enumerate() {
                        let mut f = escape_range_edge(from);
                        let mut t = escape_range_edge(to);
                        // 官方：只有**非首尾**两项上的 `-` 才需要再转义一次
                        if i != 0 && i != last {
                            if f == "-" {
                                f = "\\-".to_string();
                            }
                            if t == "-" {
                                t = "\\-".to_string();
                            }
                        }
                        if f != t {
                            self.out.push_str(&f);
                            self.out.push('-');
                            self.out.push_str(&t);
                        } else {
                            self.out.push_str(&f);
                        }
                    }
                    self.out.push(']');
                }
            },
            EKind::BackReference { reference } => {
                let numeric =
                    !reference.is_empty() && reference.chars().all(|c| c.is_ascii_digit());
                if numeric {
                    self.out.push('\\');
                    self.out.push_str(reference);
                } else {
                    self.out.push_str("\\k<");
                    self.out.push_str(reference);
                    self.out.push('>');
                }
            }
            EKind::Boundary { kind, negate } => match kind {
                BoundaryKind::Beginning => self.out.push('^'),
                BoundaryKind::End => self.out.push('$'),
                BoundaryKind::Word => self.out.push_str(if *negate { "\\B" } else { "\\b" }),
            },
            EKind::Group { kind, name, children } => {
                match kind {
                    EGroupKind::Capturing => self.out.push('('),
                    EGroupKind::NonCapturing => self.out.push_str("(?:"),
                    EGroupKind::NamedCapturing => {
                        self.out.push_str("(?<");
                        self.out.push_str(name);
                        self.out.push('>');
                    }
                }
                self.nodes(children);
                self.out.push(')');
            }
            EKind::LookAround {
                kind,
                negate,
                children,
            } => {
                self.out.push_str(match (kind, negate) {
                    (LookKind::Lookahead, false) => "(?=",
                    (LookKind::Lookahead, true) => "(?!",
                    (LookKind::Lookbehind, false) => "(?<=",
                    (LookKind::Lookbehind, true) => "(?<!",
                });
                self.nodes(children);
                self.out.push(')');
            }
            EKind::Choice { branches } => {
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 {
                        self.out.push('|');
                    }
                    self.nodes(branch);
                }
            }
        }
    }

    fn quantifier(&mut self, q: &Quantifier) {
        match q.kind {
            QuantKind::Star => self.out.push('*'),
            QuantKind::Plus => self.out.push('+'),
            QuantKind::Question => self.out.push('?'),
            QuantKind::Custom => {
                if q.min == q.max {
                    self.out.push_str(&format!("{{{}}}", q.min));
                } else if q.infinite() {
                    self.out.push_str(&format!("{{{},}}", q.min));
                } else {
                    self.out.push_str(&format!("{{{},{}}}", q.min, q.max));
                }
            }
        }
        if !q.greedy {
            self.out.push('?');
        }
    }
}

/// 官方 `genCharacter` 的 ranges 分支：`]` 与 `\` 永远转义
fn escape_range_edge(s: &str) -> String {
    if s == "]" || s == "\\" {
        format!("\\{s}")
    } else {
        s.to_string()
    }
}

impl ERoot {
    /// 生成正则串（不带区间）
    fn to_pattern(&self) -> String {
        let mut ser = Ser::plain();
        ser.nodes(&self.body);
        ser.out
    }

    /// 生成正则串并给出 `[head, tail]` 这段节点覆盖的字符区间
    fn pattern_with_span(&self, head: NodeId, tail: NodeId) -> (String, usize, usize) {
        let mut ser = Ser::tracking(head, tail);
        ser.nodes(&self.body);
        (ser.out, ser.start, ser.end)
    }

    // -----------------------------------------------------------------------
    // 定位
    //
    // 官方 `ls()` 返回 `{node, parent, nodeList, index}` —— 同时持有不可变与
    // 可变引用的写法过不了 Rust 借用检查，所以这里改成「路径」：
    // 一串「进入哪条子序列」的步骤，外加最终序列内的起始下标。
    // -----------------------------------------------------------------------

    fn find(&self, id: NodeId) -> Option<(Vec<Step>, usize)> {
        find_path(&self.body, id)
    }

    fn node(&self, id: NodeId) -> Option<&ENode> {
        self.find(id).map(|(path, index)| &seq_at(&self.body, &path)[index])
    }

    /// 选中集合的 `[首 id, 尾 id]` 在同一条序列里对应的切片。
    /// 官方 `ad(ast, ids)` 只按首尾 id 切片，不校验中间是否连续。
    #[allow(dead_code)] // 官方 `ad()` 的等价物，插入操作目前走路径切片
    fn slice(&self, ids: &[NodeId]) -> Option<(&[ENode], usize, usize)> {
        if ids.is_empty() {
            return None;
        }
        let (path, index) = self.find(ids[0])?;
        let seq = seq_at(&self.body, &path);
        let len = ids.len().min(seq.len().saturating_sub(index));
        Some((seq, index, len))
    }

    /// 官方：`root.body[0].id === id`（只认根序列的第一个，不认嵌套）
    fn is_first(&self, id: NodeId) -> bool {
        self.body.first().is_some_and(|n| n.id == id)
    }

    fn is_last(&self, id: NodeId) -> bool {
        self.body.last().is_some_and(|n| n.id == id)
    }

    // -----------------------------------------------------------------------
    // 变更操作
    // -----------------------------------------------------------------------

    /// 官方 `FA`：向前 / 向后 / 并行插入一个空字符串节点
    fn insert_around(&mut self, ids: &[NodeId], mode: InsertMode, id_gen: &mut IdGen) {
        if ids.is_empty() {
            return;
        }
        let Some((path, index)) = self.find(ids[0]) else {
            return;
        };
        let fresh = e_string(id_gen.next(), "");
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        let len = ids.len().min(seq.len().saturating_sub(index));

        match mode {
            InsertMode::Before => seq.insert(index, fresh),
            InsertMode::After => seq.insert(index + len, fresh),
            InsertMode::Parallel => {
                let single_choice = len == 1 && matches!(seq[index].kind, EKind::Choice { .. });
                let whole_branch = index == 0
                    && len == seq.len()
                    && matches!(path.last(), Some(Step::Branch(..)));
                if single_choice {
                    if let EKind::Choice { branches } = &mut seq[index].kind {
                        branches.push(vec![fresh]);
                    }
                } else if whole_branch {
                    let parent_path = &path[..path.len() - 1];
                    let ci = match path[path.len() - 1] {
                        Step::Branch(ci, _) => ci,
                        Step::Children(_) => unreachable!(),
                    };
                    let parent = seq_at_mut(&mut self.body, parent_path);
                    if let EKind::Choice { branches } = &mut parent[ci].kind {
                        branches.push(vec![fresh]);
                    }
                } else {
                    // 用一个新的 choice 把选中段包起来
                    let taken: Vec<ENode> = seq.splice(index..index + len, []).collect();
                    let choice = e_choice(id_gen.next(), vec![taken, vec![fresh]]);
                    seq.insert(index, choice);
                }
            }
        }
    }

    /// 官方 `VA`：替换节点的内容，保留量词。
    /// 多字符字符串 + 已有量词 → 自动包一个非捕获组，量词挪到组上。
    /// 返回**新的选中 id**（自动加组时是组的 id）。
    fn set_content(&mut self, id: NodeId, spec: &ContentSpec, id_gen: &mut IdGen) -> NodeId {
        let Some((path, index)) = self.find(id) else {
            return id;
        };
        let fresh = e_string(id_gen.next(), "");
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        let quantifier = seq[index].quantifier;
        let new_kind = spec.to_kind();

        if let ContentSpec::String { value } = spec {
            if value.chars().count() > 1 && quantifier.is_some() {
                let inner = ENode {
                    id,
                    quantifier: None,
                    kind: new_kind,
                };
                let group_id = id_gen.next();
                seq[index] = ENode {
                    id: group_id,
                    quantifier,
                    kind: EKind::Group {
                        kind: EGroupKind::NonCapturing,
                        name: String::new(),
                        children: vec![inner],
                    },
                };
                return group_id;
            }
        }
        let _ = fresh;
        seq[index].kind = new_kind;
        id
    }

    /// 官方 `HA`：设置量词。多字符字符串 → 自动包非捕获组并把量词挂到组上。
    /// 返回新的选中 id。
    fn set_quantifier(
        &mut self,
        id: NodeId,
        quantifier: Option<Quantifier>,
        id_gen: &mut IdGen,
    ) -> NodeId {
        let Some((path, index)) = self.find(id) else {
            return id;
        };
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        let multi = matches!(
            &seq[index].kind,
            EKind::Character { kind: CharKind::String, value, .. } if value.chars().count() > 1
        );
        if multi {
            let child = seq[index].clone();
            let group_id = id_gen.next();
            seq[index] = ENode {
                id: group_id,
                quantifier,
                kind: EKind::Group {
                    kind: EGroupKind::NonCapturing,
                    name: String::new(),
                    children: vec![child],
                },
            };
            group_id
        } else if seq[index].accepts_quantifier() {
            seq[index].quantifier = quantifier;
            id
        } else {
            id
        }
    }

    /// 官方 `BA` / `zA`：切换分组类型；`None` 表示解组（子节点原地展开）
    fn set_group_kind(&mut self, id: NodeId, kind: Option<EGroupKind>) -> Vec<NodeId> {
        let Some((path, index)) = self.find(id) else {
            return vec![id];
        };
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        if !matches!(seq[index].kind, EKind::Group { .. }) {
            return vec![id];
        }
        match kind {
            None => {
                let children = seq[index].children().cloned().unwrap_or_default();
                let ids = children.iter().map(|c| c.id).collect();
                seq.splice(index..index + 1, children);
                ids
            }
            Some(next) => {
                if let EKind::Group { kind, name, .. } = &mut seq[index].kind {
                    match next {
                        EGroupKind::Capturing => {
                            *kind = next;
                            name.clear();
                        }
                        EGroupKind::NamedCapturing => {
                            *kind = next;
                            if name.is_empty() {
                                *name = "name".to_string();
                            }
                        }
                        EGroupKind::NonCapturing => *kind = next,
                    }
                }
                vec![id]
            }
        }
    }

    /// 官方 `WA` / `GA`：切换环视类型与否定；`None` 表示取消断言（子节点原地展开）
    fn set_lookaround(&mut self, id: NodeId, spec: Option<(LookKind, bool)>) -> Vec<NodeId> {
        let Some((path, index)) = self.find(id) else {
            return vec![id];
        };
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        if !matches!(seq[index].kind, EKind::LookAround { .. }) {
            return vec![id];
        }
        match spec {
            Some((kind, negate)) => {
                if let EKind::LookAround {
                    kind: k, negate: n, ..
                } = &mut seq[index].kind
                {
                    *k = kind;
                    *n = negate;
                }
                vec![id]
            }
            None => {
                let children = seq[index].children().cloned().unwrap_or_default();
                let ids = children.iter().map(|c| c.id).collect();
                seq.splice(index..index + 1, children);
                ids
            }
        }
    }

    /// 官方 `MA`：删除选中节点，并自底向上清理空壳。
    ///
    /// 清理规则与官方一致：
    ///   · 分组 / 环视断言空了 → 连壳一起删
    ///   · 分支空了 → 丢掉该分支
    ///   · 只剩一条分支 → 用这条分支替换掉 choice 本身
    ///
    /// 两阶段执行。**不能**只按 `ids[0]` 的路径摘一次：`[a|b|c]` 里同时删掉 `b` 和
    /// `c` 时，两者分别位于 `branches[1]` / `branches[2]` 两条不同的子序列，
    /// 单条路径覆盖不到。
    fn remove_nodes(&mut self, ids: &[NodeId]) {
        if ids.is_empty() {
            return;
        }
        // ① 逐个定位所属序列再摘除。每次重新 `find` —— 上一次摘除会让同层下标漂移。
        let mut paths: Vec<Vec<Step>> = Vec::new();
        for id in ids {
            if let Some((path, _)) = self.find(*id) {
                seq_at_mut(&mut self.body, &path).retain(|n| n.id != *id);
                paths.push(path);
            }
        }
        // ② 自底向上清理空壳。清理会删节点、摊平 choice，使路径过期，
        //    所以要循环到不动点（每轮摘空一层壳，轮数不会超过路径深度）。
        let rounds = paths.iter().map(Vec::len).max().unwrap_or(0) + 1;
        for _ in 0..rounds {
            let mut changed = false;
            for path in &paths {
                if self.cleanup_along(path) {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// 沿一条路径自底向上摘掉空壳，返回是否发生了改动。
    /// 路径可能在清理过程中过期，所以每步都用 `try_seq_at_mut` 校验。
    fn cleanup_along(&mut self, path: &[Step]) -> bool {
        let mut changed = false;
        for depth in (0..path.len()).rev() {
            let (in_children, index) = match path[depth] {
                Step::Children(i) => (true, i),
                Step::Branch(i, _) => (false, i),
            };
            let Some(seq) = try_seq_at_mut(&mut self.body, &path[..depth]) else {
                break;
            };
            if index >= seq.len() {
                continue;
            }
            if in_children
                && matches!(
                    &seq[index].kind,
                    EKind::Group { children, .. } | EKind::LookAround { children, .. }
                        if children.is_empty()
                )
            {
                seq.remove(index);
                changed = true;
                continue;
            }
            let mut flatten: Option<Vec<ENode>> = None;
            if let EKind::Choice { branches } = &mut seq[index].kind {
                let before = branches.len();
                branches.retain(|b| !b.is_empty());
                if branches.len() != before {
                    changed = true;
                }
                if branches.len() == 1 {
                    flatten = Some(std::mem::take(&mut branches[0]));
                }
            }
            if let Some(single) = flatten {
                seq.splice(index..index + 1, single);
                changed = true;
            }
        }
        changed
    }

    /// 官方 `ZA` / `UA`：把选中段就地包进分组 / 环视断言
    fn wrap(&mut self, ids: &[NodeId], kind: WrapKind, id_gen: &mut IdGen) -> Vec<NodeId> {
        if ids.is_empty() {
            return Vec::new();
        }
        let Some((path, index)) = self.find(ids[0]) else {
            return Vec::new();
        };
        let new_id = id_gen.next();
        let body = &mut self.body;
        let seq = seq_at_mut(body, &path);
        let len = ids.len().min(seq.len().saturating_sub(index));
        let taken: Vec<ENode> = seq.splice(index..index + len, []).collect();
        let node = match kind {
            WrapKind::Group(k) => e_group(
                new_id,
                k,
                if matches!(k, EGroupKind::NamedCapturing) {
                    "name"
                } else {
                    ""
                },
                taken,
            ),
            WrapKind::LookAround(k) => e_look(new_id, k, false, taken),
        };
        seq.insert(index, node);
        vec![new_id]
    }

    /// 树里所有捕获组的标识（具名用名字，普通用序号）。反向引用下拉用。
    fn capture_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut index = 0usize;
        fn walk(nodes: &[ENode], index: &mut usize, out: &mut Vec<String>) {
            for node in nodes {
                match &node.kind {
                    EKind::Group { kind, name, children } => {
                        match kind {
                            EGroupKind::Capturing => {
                                *index += 1;
                                out.push(index.to_string());
                            }
                            EGroupKind::NamedCapturing => {
                                *index += 1;
                                out.push(name.clone());
                            }
                            EGroupKind::NonCapturing => {}
                        }
                        walk(children, index, out);
                    }
                    EKind::LookAround { children, .. } => walk(children, index, out),
                    EKind::Choice { branches } => {
                        for branch in branches {
                            walk(branch, index, out);
                        }
                    }
                    _ => {}
                }
            }
        }
        walk(&self.body, &mut index, &mut out);
        out
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InsertMode {
    Before,
    Parallel,
    After,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WrapKind {
    Group(EGroupKind),
    LookAround(LookKind),
}

#[derive(Clone, Copy, Debug)]
enum Step {
    /// 进入第 i 个节点的 children（group / lookAroundAssertion）
    Children(usize),
    /// 进入第 i 个节点（choice）的第 j 条分支
    Branch(usize, usize),
}

fn find_path(nodes: &[ENode], id: NodeId) -> Option<(Vec<Step>, usize)> {
    for (i, node) in nodes.iter().enumerate() {
        if node.id == id {
            return Some((Vec::new(), i));
        }
        match &node.kind {
            EKind::Group { children, .. } | EKind::LookAround { children, .. } => {
                if let Some((mut path, index)) = find_path(children, id) {
                    path.insert(0, Step::Children(i));
                    return Some((path, index));
                }
            }
            EKind::Choice { branches } => {
                for (j, branch) in branches.iter().enumerate() {
                    if let Some((mut path, index)) = find_path(branch, id) {
                        path.insert(0, Step::Branch(i, j));
                        return Some((path, index));
                    }
                }
            }
            EKind::Character { .. } | EKind::BackReference { .. } | EKind::Boundary { .. } => {}
        }
    }
    None
}

fn seq_at<'a>(nodes: &'a [ENode], path: &[Step]) -> &'a [ENode] {
    match path.split_first() {
        // 空路径 = 就是这一层序列本身。顶层节点（选中集合落在根序列上）会走到这里，
        // 必须返回 `nodes`，不能 panic —— 与 `seq_at_mut` 的 `None => nodes` 保持一致。
        None => nodes,
        Some((step, rest)) => {
            if rest.is_empty() {
                match step {
                    Step::Children(i) => match &nodes[*i].kind {
                        EKind::Group { children, .. } | EKind::LookAround { children, .. } => {
                            children
                        }
                        _ => unreachable!(),
                    },
                    Step::Branch(i, j) => match &nodes[*i].kind {
                        EKind::Choice { branches } => &branches[*j],
                        _ => unreachable!(),
                    },
                }
            } else {
                match step {
                    Step::Children(i) => match &nodes[*i].kind {
                        EKind::Group { children, .. } | EKind::LookAround { children, .. } => {
                            seq_at(children, rest)
                        }
                        _ => unreachable!(),
                    },
                    Step::Branch(i, j) => match &nodes[*i].kind {
                        EKind::Choice { branches } => seq_at(&branches[*j], rest),
                        _ => unreachable!(),
                    },
                }
            }
        }
    }
}

fn seq_at_mut<'a>(nodes: &'a mut Vec<ENode>, path: &[Step]) -> &'a mut Vec<ENode> {
    match path.split_first() {
        None => nodes,
        Some((step, rest)) => {
            let next = match step {
                Step::Children(i) => match &mut nodes[*i].kind {
                    EKind::Group { children, .. } | EKind::LookAround { children, .. } => children,
                    _ => unreachable!(),
                },
                Step::Branch(i, j) => match &mut nodes[*i].kind {
                    EKind::Choice { branches } => &mut branches[*j],
                    _ => unreachable!(),
                },
            };
            seq_at_mut(next, rest)
        }
    }
}

/// `seq_at_mut` 的安全版本：路径上任何一环过期就返回 `None`。
///
/// 清理空壳时会删节点、摊平 choice，早先算出的路径可能因此指不到东西了。
/// 这时候必须安静退出，不能越界 panic。
fn try_seq_at_mut<'a>(nodes: &'a mut Vec<ENode>, path: &[Step]) -> Option<&'a mut Vec<ENode>> {
    match path.split_first() {
        None => Some(nodes),
        Some((step, rest)) => {
            let next = match step {
                Step::Children(i) => match nodes.get_mut(*i).map(|n| &mut n.kind) {
                    Some(EKind::Group { children, .. } | EKind::LookAround { children, .. }) => {
                        children
                    }
                    _ => return None,
                },
                Step::Branch(i, j) => match nodes.get_mut(*i).map(|n| &mut n.kind) {
                    Some(EKind::Choice { branches }) => branches.get_mut(*j)?,
                    _ => return None,
                },
            };
            try_seq_at_mut(next, rest)
        }
    }
}

// ===========================================================================
// 节点文案（对齐官方 `V8` / `v8`）
// ===========================================================================

fn backref_label(lang: Lang) -> &'static str {
    lang.of("反向引用", "Back reference")
}

/// 官方 `kM`：`from == to` 的单字符区间合并成一段裸串，其余按 `"a" - "z"` 渲染。
/// 合并串**放在最后**（官方先把区间 push 进数组，循环结束后才 push 合并串），
/// 且用 Set 去重、保持首次出现的顺序。
fn ranges_text(ranges: &[(String, String)], lang: Lang) -> String {
    let mut merged: Vec<char> = Vec::new();
    let mut parts: Vec<String> = Vec::new();
    for (from, to) in ranges {
        if from == to {
            let chars: Vec<char> = from.chars().collect();
            if chars.len() == 1 {
                if !merged.contains(&chars[0]) {
                    merged.push(chars[0]);
                }
            } else {
                parts.push(from.clone());
            }
        } else if from.chars().count() == 1 {
            parts.push(format!("\"{from}\" - \"{to}\""));
        } else {
            parts.push(format!("{from} - {to}"));
        }
    }
    if !merged.is_empty() {
        parts.push(format!("\"{}\"", merged.iter().collect::<String>()));
    }
    if parts.is_empty() {
        empty_label(lang).to_string()
    } else {
        parts.join(", ")
    }
}

/// 节点主文案。分组和分支没有主文案（返回空串）。
fn node_text(node: &ENode, lang: Lang) -> String {
    match &node.kind {
        EKind::Character { kind, value, ranges, .. } => match kind {
            // 官方 `V8`：空串渲染成 `t("Empty")`
            CharKind::String => {
                if value.is_empty() {
                    empty_label(lang).to_string()
                } else {
                    format!("\"{value}\"")
                }
            }
            CharKind::Class => class_label(value, lang)
                .map(str::to_string)
                .unwrap_or_else(|| value.clone()),
            CharKind::Ranges => ranges_text(ranges, lang),
        },
        EKind::BackReference { reference } => {
            format!("{} #{reference}", backref_label(lang))
        }
        EKind::Boundary { kind, negate } => match kind {
            BoundaryKind::Beginning => beginning_label(lang).to_string(),
            BoundaryKind::End => ending_label(lang).to_string(),
            BoundaryKind::Word => boundary_label(lang, *negate).to_string(),
        },
        EKind::Group { .. } | EKind::LookAround { .. } | EKind::Choice { .. } => String::new(),
    }
}

/// 节点上方的装饰标签。对齐官方 `v8`：
/// 只有字符范围（One of / None of）、捕获组（组 #名字）和环视（接着: 等）三种。
fn node_label(node: &ENode, lang: Lang) -> Option<String> {
    match &node.kind {
        EKind::Character {
            kind: CharKind::Ranges,
            negate,
            ..
        } => Some(if *negate {
            none_of_label(lang)
        } else {
            one_of_label(lang)
        }
        .to_string()),
        EKind::Group {
            kind: EGroupKind::Capturing,
            ..
        } => Some(format!("{} #{}", group_word(lang), capture_index_of(node, lang))),
        EKind::Group {
            kind: EGroupKind::NamedCapturing,
            name,
            ..
        } => Some(format!("{} #{name}", group_word(lang))),
        EKind::LookAround { kind, negate, .. } => Some(
            lookaround_label(lang, matches!(kind, LookKind::Lookahead), *negate).to_string(),
        ),
        _ => None,
    }
}

/// 普通捕获组的显示序号（从 1 开始）。因为可编辑 AST 不再保存解析器的索引，
/// 这里按树的先序重新数一遍——与 `regex-syntax` 的编号规则一致。
fn capture_index_of(target: &ENode, _lang: Lang) -> usize {
    CAPTURE_INDEX.with(|cell| {
        let map = cell.borrow();
        map.get(&target.id).copied().unwrap_or(0)
    })
}

thread_local! {
    /// 最近一次构建图时算出的「捕获组 id → 序号」映射
    static CAPTURE_INDEX: std::cell::RefCell<std::collections::HashMap<NodeId, usize>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// 先序遍历，登记每个捕获组的序号
fn index_captures(nodes: &[ENode], counter: &mut usize, out: &mut std::collections::HashMap<NodeId, usize>) {
    for node in nodes {
        match &node.kind {
            EKind::Group { kind, children, .. } => {
                if !matches!(kind, EGroupKind::NonCapturing) {
                    *counter += 1;
                    out.insert(node.id, *counter);
                }
                index_captures(children, counter, out);
            }
            EKind::LookAround { children, .. } => index_captures(children, counter, out),
            EKind::Choice { branches } => {
                for branch in branches {
                    index_captures(branch, counter, out);
                }
            }
            _ => {}
        }
    }
}

fn refresh_capture_index(root: &ERoot) {
    let mut counter = 0usize;
    let mut map = std::collections::HashMap::new();
    index_captures(&root.body, &mut counter, &mut map);
    CAPTURE_INDEX.with(|cell| *cell.borrow_mut() = map);
}

// ===========================================================================
// 编辑面板用的取值模型
// ===========================================================================

/// 内容段可取的类型。官方 `sF` / `uF` / `aF` / `lF` / `cF` 的并集。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ContentSpec {
    String { value: String },
    Class { value: String },
    Ranges {
        ranges: Vec<(String, String)>,
        negate: bool,
    },
    BackReference { reference: String },
    WordBoundary { negate: bool },
    Beginning,
    End,
}

/// 官方 `Gz` 里的类型键（也是下拉的取值）
const CONTENT_TYPES: [(&str, &str, &str); 7] = [
    ("string", "简单字符串", "Simple string"),
    ("class", "字符类", "Character class"),
    ("ranges", "字符范围", "Character range"),
    ("backReference", "反向引用", "Back reference"),
    ("beginningAssertion", "开始断言", "Beginning Assertion"),
    ("endAssertion", "结束断言", "End Assertion"),
    ("wordBoundaryAssertion", "单词边界断言", "Word Boundary Assertion"),
];

impl ContentSpec {
    fn key(&self) -> &'static str {
        match self {
            ContentSpec::String { .. } => "string",
            ContentSpec::Class { .. } => "class",
            ContentSpec::Ranges { .. } => "ranges",
            ContentSpec::BackReference { .. } => "backReference",
            ContentSpec::WordBoundary { .. } => "wordBoundaryAssertion",
            ContentSpec::Beginning => "beginningAssertion",
            ContentSpec::End => "endAssertion",
        }
    }

    /// 官方 `c(u)`：切换类型时给出的初值
    fn for_key(key: &str) -> ContentSpec {
        match key {
            "string" => ContentSpec::String {
                value: String::new(),
            },
            "class" => ContentSpec::Class {
                value: String::new(),
            },
            "ranges" => ContentSpec::Ranges {
                ranges: vec![(String::new(), String::new())],
                negate: false,
            },
            "backReference" => ContentSpec::BackReference {
                reference: "1".to_string(),
            },
            "beginningAssertion" => ContentSpec::Beginning,
            "endAssertion" => ContentSpec::End,
            _ => ContentSpec::WordBoundary { negate: false },
        }
    }

    fn to_kind(&self) -> EKind {
        match self {
            ContentSpec::String { value } => EKind::Character {
                kind: CharKind::String,
                value: value.clone(),
                ranges: Vec::new(),
                negate: false,
            },
            ContentSpec::Class { value } => EKind::Character {
                kind: CharKind::Class,
                value: value.clone(),
                ranges: Vec::new(),
                negate: false,
            },
            ContentSpec::Ranges { ranges, negate } => EKind::Character {
                kind: CharKind::Ranges,
                value: String::new(),
                ranges: ranges.clone(),
                negate: *negate,
            },
            ContentSpec::BackReference { reference } => EKind::BackReference {
                reference: reference.clone(),
            },
            ContentSpec::WordBoundary { negate } => EKind::Boundary {
                kind: BoundaryKind::Word,
                negate: *negate,
            },
            ContentSpec::Beginning => EKind::Boundary {
                kind: BoundaryKind::Beginning,
                negate: false,
            },
            ContentSpec::End => EKind::Boundary {
                kind: BoundaryKind::End,
                negate: false,
            },
        }
    }

    /// 官方 `lB`：只有单选、且节点属于这七类时才给出内容信息
    fn from_node(node: &ENode) -> Option<ContentSpec> {
        match &node.kind {
            EKind::Character {
                kind,
                value,
                ranges,
                negate,
            } => Some(match kind {
                CharKind::String => ContentSpec::String {
                    value: value.clone(),
                },
                CharKind::Class => ContentSpec::Class {
                    value: value.clone(),
                },
                CharKind::Ranges => ContentSpec::Ranges {
                    ranges: ranges.clone(),
                    negate: *negate,
                },
            }),
            EKind::BackReference { reference } => Some(ContentSpec::BackReference {
                reference: reference.clone(),
            }),
            EKind::Boundary { kind, negate } => Some(match kind {
                BoundaryKind::Word => ContentSpec::WordBoundary { negate: *negate },
                BoundaryKind::Beginning => ContentSpec::Beginning,
                BoundaryKind::End => ContentSpec::End,
            }),
            _ => None,
        }
    }
}

/// 官方 `Gz` 的类型下拉可见项：基础四项 + 三个条件项。
///
/// ```js
/// const u = [...sF, uF];
/// (lookArounds.length !== 0 || a === "backReference") && u.push(aF);
/// (root.body[0].id === t || a === "beginningAssertion") && u.push(lF);
/// (root.body[last].id === t || a === "endAssertion") && u.push(cF);
/// ```
fn content_type_keys(info: &SelectionInfo, capture_count: usize) -> Vec<&'static str> {
    let current = info.content.as_ref().map(ContentSpec::key);
    let mut keys: Vec<&'static str> = vec!["string", "class", "ranges", "wordBoundaryAssertion"];
    if capture_count > 0 || current == Some("backReference") {
        keys.push("backReference");
    }
    if info.first || current == Some("beginningAssertion") {
        keys.push("beginningAssertion");
    }
    if info.last || current == Some("endAssertion") {
        keys.push("endAssertion");
    }
    keys
}

/// 官方 `qz`：量词下拉。CN 语言包缺 "0 or 1" / "0 or more" / "1 or more"，
/// 与官网一致地回退成英文 key。
const QUANT_OPTIONS: [(&str, &str, &str, &str); 5] = [
    ("non", "1 (默认)", "1 (default)", ""),
    ("?", "0 or 1", "0 or 1", "?"),
    ("*", "0 or more", "0 or more", "*"),
    ("+", "1 or more", "1 or more", "+"),
    ("custom", "自定义", "custom", "{min,max}"),
];

fn quant_key(q: Option<&Quantifier>) -> &'static str {
    match q.map(|q| q.kind) {
        None => "non",
        Some(QuantKind::Question) => "?",
        Some(QuantKind::Star) => "*",
        Some(QuantKind::Plus) => "+",
        Some(QuantKind::Custom) => "custom",
    }
}

fn quant_for_key(key: &str, greedy: bool) -> Option<Quantifier> {
    Some(match key {
        "?" => Quantifier {
            kind: QuantKind::Question,
            min: 0,
            max: 1,
            greedy,
        },
        "*" => Quantifier {
            kind: QuantKind::Star,
            min: 0,
            max: Quantifier::INF,
            greedy,
        },
        "+" => Quantifier {
            kind: QuantKind::Plus,
            min: 1,
            max: Quantifier::INF,
            greedy,
        },
        "custom" => Quantifier {
            kind: QuantKind::Custom,
            min: 1,
            max: 1,
            greedy,
        },
        _ => return None,
    })
}

/// 一次选中对应的全部面板信息。对应官方 `fB`。
#[derive(Clone, Debug)]
struct SelectionInfo {
    ids: Vec<NodeId>,
    id: NodeId,
    /// 整条正则串
    pattern: String,
    /// 选中区间在 `pattern` 里的字符下标（官方 `qA` 的结果）
    start: usize,
    end: usize,
    content: Option<ContentSpec>,
    group: Option<(EGroupKind, String)>,
    /// 节点自身是否允许带量词（官方 `y0`）
    has_quantifier: bool,
    quantifier: Option<Quantifier>,
    lookaround: Option<(LookKind, bool)>,
    /// 是否根序列的第一个 / 最后一个节点（决定能否加「开始/结束断言」）
    first: bool,
    last: bool,
    single: bool,
}

impl ERoot {
    /// 官方 `fB(root, nodes)`
    fn selection(&self, ids: &[NodeId]) -> SelectionInfo {
        let id = ids.first().copied().unwrap_or(0);
        let (pattern, start, end) = match (ids.first(), ids.last()) {
            (Some(head), Some(tail)) => self.pattern_with_span(*head, *tail),
            _ => (self.to_pattern(), 0, 0),
        };
        let single = ids.len() == 1;
        let node = if single { self.node(id) } else { None };
        let group = node.and_then(|n| match &n.kind {
            EKind::Group { kind, name, .. } => Some((*kind, name.clone())),
            _ => None,
        });
        let lookaround = node.and_then(|n| match &n.kind {
            EKind::LookAround { kind, negate, .. } => Some((*kind, *negate)),
            _ => None,
        });
        SelectionInfo {
            ids: ids.to_vec(),
            id,
            pattern,
            start,
            end,
            content: node.and_then(ContentSpec::from_node),
            group,
            has_quantifier: node.is_some_and(|n| n.accepts_quantifier()),
            quantifier: node.and_then(|n| n.quantifier),
            lookaround,
            first: self.is_first(id),
            last: self.is_last(id),
            single,
        }
    }
}

// ===========================================================================
// 编辑面板控件模型
// ===========================================================================

/// 撤销栈上限。官方 `Bj` 没设上限，这里给个保守值免得长会话越攒越大。
const UNDO_LIMIT: usize = 100;

/// 编辑面板上一个控件的槽位。
///
/// key 里带节点 id，所以「换选中节点」天生就是换了一批槽位：旧控件会被 `retain`
/// 丢掉，新节点拿到全新的 `InputState`，不会串值。
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum EditSlot {
    ContentType(NodeId),
    ClassKind(NodeId),
    Backref(NodeId),
    QuantKind(NodeId),
    GroupKind(NodeId),
    LookKind(NodeId),
    Value(NodeId),
    RangeFrom(NodeId, usize),
    RangeTo(NodeId, usize),
    QuantMin(NodeId),
    QuantMax(NodeId),
    GroupName(NodeId),
}

/// 下拉项：显示文案与取值分离。
///
/// 官方是 `<SelectItem value={key}>{渲染内容}</SelectItem>`；gpui 这边对应
/// `SelectState<Vec<LabeledItem>>` + `SelectEvent::Confirm(Option<String>)`。
#[derive(Clone)]
struct LabeledItem {
    key: String,
    label: SharedString,
}

impl LabeledItem {
    fn new(key: impl Into<String>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

impl gpui_kit::component::searchable_list::SearchableListItem for LabeledItem {
    type Value = String;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.key
    }
}

/// 官方 `jn` 全表 —— 「字符类」下拉的候选项（`J6` 就是遍历它建出来的）。
///
/// 中文列取自官网 cn 语言包；`\xhh` / `\uhhhh` 两条官网没有译文，按 i18next 的
/// fallbackLng 规则回退成英文 key。节点上显示的名字走 `CLASS_LABELS`，
/// 那张表只覆盖图里画得出来的 13 项，和官方一致地分工。
const CLASS_OPTIONS: [(&str, &str, &str); 22] = [
    (".", "任意字符", "Any character"),
    (r"\d", "任意数字", "Any digit"),
    (r"\D", "任意非数字", "Non-digit"),
    (r"\w", "任意基本拉丁字母数字", "Any alphanumeric"),
    (r"\W", "任意非基本拉丁字母数字", "Non-alphanumeric"),
    (r"\s", "任意空白字符", "White space"),
    (r"\S", "任意非空白字符", "Non-white space"),
    (r"\t", "制表符", "Horizontal tab"),
    (r"\r", "回车符", "Carriage return"),
    (r"\n", "换行符", "Linefeed"),
    (r"\v", "垂直制表符", "Vertical tab"),
    (r"\f", "Form-feed", "Form-feed"),
    (r"[\b]", "退格", "Backspace"),
    (r"\0", "NUL", "NUL"),
    (r"\cH", r"\b 退格", r"\b Backspace"),
    (r"\cI", r"\t 制表符", r"\t Horizontal Tab"),
    (r"\cJ", r"\n 换行符", r"\n Line Feed"),
    (r"\cK", r"\v 垂直制表符", r"\v Vertical Tab"),
    (r"\cL", r"\f Form Feed", r"\f Form Feed"),
    (r"\cM", r"\r 回车符", r"\r Carriage Return"),
    (r"\xhh", "ASCII symbol", "ASCII symbol"),
    (r"\uhhhh", "Unicode symbol", "Unicode symbol"),
];

/// 官方 `rF` 里的 `r = useMemo(...)`：把 `\xNN` / `\uNNNN` 形式的取值归一到
/// 下拉用的伪 key，其它取值原样返回。
fn class_kind_key(value: &str) -> String {
    let bytes: Vec<char> = value.chars().collect();
    if bytes.len() == 4 && bytes[0] == '\\' && bytes[1] == 'x' {
        if bytes[2].is_ascii_hexdigit() && bytes[3].is_ascii_hexdigit() {
            return r"\xhh".to_string();
        }
    }
    if bytes.len() == 6 && bytes[0] == '\\' && bytes[1] == 'u' {
        if bytes[2..].iter().all(|c| c.is_ascii_hexdigit()) {
            return r"\uhhhh".to_string();
        }
    }
    value.to_string()
}

/// 字符类下拉的候选项。官方渲染成「青色等宽的 value + 译名」，这里并成一条文案。
fn class_items(lang: Lang) -> Vec<LabeledItem> {
    CLASS_OPTIONS
        .iter()
        .map(|(key, cn, en)| {
            let label = lang.of(cn, en);
            LabeledItem::new(*key, format!("{key}  {label}"))
        })
        .collect()
}

/// 官方 `Yz` 里 `i = l => o(r, l)`：改具名分组的分组名。
///
/// 名字为空时官方强制回落成 `"name"`（否则 `(?<>...)` 是非法正则）。
fn set_group_name(tree: &mut ERoot, id: NodeId, name: &str) {
    let Some((path, index)) = tree.find(id) else {
        return;
    };
    let body = &mut tree.body;
    let seq = seq_at_mut(body, &path);
    if let Some(node) = seq.get_mut(index) {
        if let EKind::Group {
            kind, name: slot, ..
        } = &mut node.kind
        {
            *kind = EGroupKind::NamedCapturing;
            *slot = if name.is_empty() {
                "name".to_string()
            } else {
                name.to_string()
            };
        }
    }
}

fn group_kind_key(kind: EGroupKind) -> &'static str {
    match kind {
        EGroupKind::Capturing => "capturing",
        EGroupKind::NonCapturing => "nonCapturing",
        EGroupKind::NamedCapturing => "namedCapturing",
    }
}

fn look_kind_key(kind: LookKind) -> &'static str {
    match kind {
        LookKind::Lookahead => "lookahead",
        LookKind::Lookbehind => "lookbehind",
    }
}

// ===========================================================================
// 编辑面板的通用零件
// ===========================================================================

/// 官方 `it`：面板小节 —— 12px 加粗次要色标题 + 内容，右上角可挂一个动作按钮
fn panel_section(
    label: &'static str,
    action: Option<AnyElement>,
    body: AnyElement,
    cx: &App,
) -> Div {
    let mut head = div().flex().items_center().justify_between();
    head = head.child(design::editor_label(label.to_string(), cx));
    if let Some(action) = action {
        head = head.child(action);
    }
    div().flex().flex_col().gap_2().child(head).child(body)
}

/// 小节里的子项（官方 `it.Item`）：13px 常规色标题 + 内容
fn panel_item(label: &'static str, body: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_size(px(13.0)).child(label))
        .child(body)
}

/// 官方 `Lf`：一串 outline 按钮拼成一组
fn button_row(buttons: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_1()
        .children(buttons)
}

/// 官方 `sF` 里的「插入」按钮
fn insert_button(
    id: &'static str,
    label: &'static str,
    mode: InsertMode,
    ids: &[NodeId],
    cx: &Context<RegexVisualizer>,
) -> AnyElement {
    let ids = ids.to_vec();
    Button::new(id)
        .label(label)
        .outline()
        .compact()
        .on_click(cx.listener(move |this, _, window, cx| {
            let ids = ids.clone();
            this.edit(window, cx, move |tree, id_gen| {
                tree.insert_around(&ids, mode, id_gen);
                Vec::new()
            });
        }))
        .into_any_element()
}

/// 「否定」勾选框。官方是 `<h6>Negate</h6>` 在勾选框**前面**。
fn negate_row<F>(
    id: &'static str,
    checked: bool,
    cx: &Context<RegexVisualizer>,
    handler: F,
) -> Div
where
    F: Fn(&mut RegexVisualizer, bool, &mut Window, &mut Context<RegexVisualizer>) + 'static,
{
    div()
        .flex()
        .items_center()
        .gap_2()
        .text_size(px(13.0))
        .text_color(cx.theme().muted_foreground)
        .child("Negate")
        .child(
            Checkbox::new(id)
                .checked(checked)
                .on_click(cx.listener(move |this, value: &bool, window, cx| {
                    handler(this, *value, window, cx);
                })),
        )
}

/// 官方 `Xz`：正则全文 + 选中区间高亮（`bg-blue-500/50`）
fn expression_body(pattern: &str, start: usize, end: usize, family: &SharedString) -> AnyElement {
    let (head, mid, tail) = if start <= end
        && end <= pattern.len()
        && pattern.is_char_boundary(start)
        && pattern.is_char_boundary(end)
    {
        (&pattern[..start], &pattern[start..end], &pattern[end..])
    } else {
        (pattern, "", "")
    };
    let highlight = Hsla {
        a: 0.5,
        ..Hsla::from(rgb(SELECT_BLUE))
    };
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .font_family(family.clone())
        .text_sm()
        .child(head.to_string())
        .child(div().rounded(px(4.0)).py(px(2.0)).bg(highlight).child(mid.to_string()))
        .child(tail.to_string())
        .into_any_element()
}

// ===========================================================================
// 绘制基元
// ===========================================================================

#[derive(Clone)]
enum Cmd {
    Move(f32, f32),
    Line(f32, f32),
    /// 对应官方 SVG 的 `A rx ry rot large sweep x y`（rx == ry、rot == 0）
    Arc {
        r: f32,
        large: bool,
        sweep: bool,
        to: (f32, f32),
    },
    /// 闭合子路径（官方的 `Z`）
    Close,
}

#[derive(Clone)]
enum Prim {
    RoundRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        stroke: u32,
        dash: bool,
    },
    /// 官方分支容器的定位盒：`class="fill-transparent"`，既无描边也无圆角，
    /// 在页面上不可见，仅承载分支的排布与命中区域。这里保留以便与官方结构一致。
    Frame {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// 选中节点的高亮覆盖层（官方 `hd` 里那层 `class="fill-blue-500/30"` 的矩形）
    Highlight {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stroke: u32,
    },
    Curve {
        cmds: Vec<Cmd>,
        stroke: u32,
        /// true 时按填充绘制（官方图标用 `fill="currentColor"`）
        filled: bool,
    },
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
        stroke: u32,
    },
    Text {
        x: f32,
        y: f32,
        w: f32,
        text: String,
        fs: f32,
        color: u32,
        center: bool,
    },
}

/// 一个可点击的节点区域。
///
/// 官方在渲染时把每个节点**内容盒**塞进 `gh` 这个 Map（见 `Z8` 的 `useEffect`），
/// 拖拽框选时按这些矩形做命中。这里改为布局期直接登记。
#[derive(Clone, Copy)]
struct HitBox {
    id: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    /// 嵌套深度。命中多个时取最深的那个 —— 等价于官方靠 DOM 事件
    /// `stopPropagation` 实现的「最内层优先」
    depth: u16,
}

impl HitBox {
    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

#[derive(Clone, Default)]
struct Diagram {
    prims: Vec<Prim>,
    /// 可点击的节点矩形（命中测试用）
    hits: Vec<HitBox>,
    width: f32,
    height: f32,
}

// ===========================================================================
// 语法树 → 布局节点
//
// 严格对齐官方 regex-vis 的两套尺寸（见其 `us` store 的 `{box, content}`）：
//
//   content —— 真正画出来的那个矩形（含装饰标签/量词行之外的纯内容）
//   box     —— 父容器排布时使用的占位尺寸
//
// 官方规则：
//   · 叶子（字符/断言）：content = 文本 + (10,2) 内边距；box 高 = content 高 + 32
//     （装饰标签 16 与量词行 16 各占一侧，两者同时存在也只加 32）
//   · 分组：content = 子序列 box + (50,30)；带标签时 box 高 = content 高 + 32
//   · 分支：box == content，高 = 20 + Σ行高 + 15×(行数-1)，宽 = 最宽行 + 50
//   · 拼接：box == content，宽 = Σ宽 + 25×(项数-1)，高 = 最高项
//
// 构建期用窗口文本系统实测文字宽度。
// ===========================================================================

struct GNode {
    kind: GKind,
    /// 对应的可编辑 AST 节点 id；0 = 不可选中（拼接容器、图例示意节点）
    id: NodeId,
    /// 布局盒（官方 `box`）宽
    w: f32,
    /// 布局盒（官方 `box`）高
    h: f32,
    /// 绘制盒（官方 `content`）宽 —— 实际画出来的矩形
    cw: f32,
    /// 绘制盒（官方 `content`）高 —— 实际画出来的矩形
    ch: f32,
}

enum GKind {
    /// 单节点：框内文字 + 可选的上方装饰标签
    Token {
        text: String,
        label: Option<String>,
        dash: bool,
    },
    /// 量词：包裹一个子节点，标记画在内容盒下方
    Repeat {
        child: Box<GNode>,
        label: String,
        /// 标签文字本身宽度（用于摆放 ∞ 图标）
        label_text_w: f32,
        /// 标签整体宽度（图标 + 文字 + 图标）
        label_w: f32,
        infinite: bool,
    },
    /// 分组：内容盒四周各留 (25, 15)，标签画在内容盒上方
    Group {
        child: Box<GNode>,
        label: Option<String>,
    },
    Concat(Vec<GNode>),
    /// 分支：行水平居中于容器，容器上下各留 10，行间距 15
    Alternate(Vec<GNode>),
}

fn shape_mono(
    text: &str,
    fs: f32,
    color: Hsla,
    window: &mut Window,
    family: &SharedString,
) -> ShapedLine {
    let font = Font {
        family: family.clone(),
        ..window.text_style().font()
    };
    let run = TextRun {
        len: text.len(),
        font,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(SharedString::from(text.to_string()), px(fs), &[run], None)
}

fn mono_text_width(text: &str, fs: f32, window: &mut Window, family: &SharedString) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    shape_mono(text, fs, Hsla::from(rgb(GRAPH_FG)), window, family)
        .width()
        .as_f32()
}

/// 官方把字面量渲染为 `"` + 2px + 字符 + 2px + `"`（span 的 padding），
/// 因此每对引号在实测文字宽度之外还要多占 4px。
fn quote_pad(text: &str) -> f32 {
    (text.matches('"').count() / 2) as f32 * 4.0
}

/// 装饰标签占用的宽度。
/// 注意官方 `oL` 走的是 `_r(t)`，而 `_r` 固定用 `ba`（16px）测量，
/// 与标签实际渲染的 14px 不同——必须按 16px 量。
fn label_node_w(text: &str, window: &mut Window, family: &SharedString) -> f32 {
    mono_text_width(text, NODE_FS, window, family) + NODE_PAD_X * 2.0
}

fn token_node(
    text: String,
    label: Option<String>,
    dash: bool,
    window: &mut Window,
    family: &SharedString,
) -> GNode {
    let cw = mono_text_width(&text, NODE_FS, window, family) + quote_pad(&text) + NODE_PAD_X * 2.0;
    let cw = cw.max(MIN_W);
    let ch = NODE_H.max(MIN_H);
    // 装饰标签独占内容盒上方的一行
    let label_w = match &label {
        Some(l) => label_node_w(l, window, family),
        None => 0.0,
    };
    let dh = if label_w > 0.0 { LABEL_BOX * 2.0 } else { 0.0 };
    GNode {
        id: 0,
        kind: GKind::Token { text, label, dash },
        w: cw.max(label_w),
        h: ch + dh,
        cw,
        ch,
    }
}/// 重复标签，对齐官方 y8 + JI：
/// - min == max → " n"
/// - 有上界    → " min - max"
/// - 无上界    → " min - " 再补一个 ∞ 图标
fn repeat_label(
    min: u32,
    max: Option<u32>,
    window: &mut Window,
    family: &SharedString,
) -> (String, f32, f32, bool) {
    let (text, infinite) = match max {
        Some(m) if m == min => (format!(" {min}"), false),
        Some(m) => (format!(" {min} - {m}"), false),
        None => (format!(" {min} - "), true),
    };
    let text_w = mono_text_width(&text, LABEL_FS, window, family);
    let total = ICON_W + text_w + if infinite { ICON_W } else { 0.0 };
    (text, text_w, total, infinite)
}

// ===========================================================================
// 布局：place() 把节点画到绝对坐标并返回其布局盒宽度
//
// 入参 (x, cy) 是该节点**布局盒**的左边界与竖直中线；绘制用的内容盒在盒内居中。
// ===========================================================================

#[allow(clippy::too_many_arguments)]
fn place(
    node: &GNode,
    x: f32,
    cy: f32,
    depth: u16,
    out: &mut Vec<Prim>,
    hits: &mut Vec<HitBox>,
    selected: &[NodeId],
    inherited_sel: bool,
) -> f32 {
    // 内容盒在布局盒内居中（官方 `l = x + (box[0]-content[0])/2`）
    let cx = x + (node.w - node.cw) / 2.0;
    let top = cy - node.ch / 2.0;

    // 选中态。量词外壳（Repeat）自己画不出矩形，由它把选中态转交给子节点，
    // 这样蓝色覆盖层正好落在子节点那条矩形之上、文字之下。
    let is_selected = inherited_sel || (node.id != 0 && selected.contains(&node.id));
    let mark = |hits: &mut Vec<HitBox>, id: NodeId, cx: f32, top: f32, w: f32, h: f32| {
        if id != 0 {
            hits.push(HitBox {
                id,
                x: cx,
                y: top,
                w,
                h,
                depth,
            });
        }
    };

    // 官方 B8：布局盒比内容盒宽时，用两段水平线把两侧补齐
    if node.w > node.cw {
        out.push(Prim::Line {
            x1: x,
            y1: cy,
            x2: cx,
            y2: cy,
            stroke: GRAPH,
        });
        out.push(Prim::Line {
            x1: cx + node.cw,
            y1: cy,
            x2: x + node.w,
            y2: cy,
            stroke: GRAPH,
        });
    }

    match &node.kind {
        GKind::Token { text, label, dash } => {
            if let Some(label) = label {
                out.push(Prim::Text {
                    x,
                    y: top - LABEL_H,
                    w: node.w,
                    text: label.clone(),
                    fs: LABEL_FS,
                    color: GRAPH_FG,
                    center: true,
                });
            }
            out.push(Prim::RoundRect {
                x: cx,
                y: top,
                w: node.cw,
                h: node.ch,
                r: NODE_R,
                // 取反字符类官方用红色虚线（stroke-red-500 + dasharray 4 2）
                stroke: if *dash { GRAPH_RED } else { GRAPH },
                dash: *dash,
            });
            if is_selected {
                out.push(select_highlight(cx, top, node.cw, node.ch));
            }
            mark(hits, node.id, cx, top, node.cw, node.ch);
            out.push(Prim::Text {
                x: cx + NODE_PAD_X,
                y: top + (node.ch - NODE_FS * 1.5) / 2.0,
                w: node.cw - NODE_PAD_X * 2.0,
                text: text.clone(),
                fs: NODE_FS,
                color: GRAPH_FG,
                center: true,
            });
        }
        GKind::Repeat {
            child,
            label,
            label_text_w,
            label_w,
            infinite,
        } => {
            // 量词外壳与子节点共用同一块内容盒，命中区因此也只有一个
            mark(hits, node.id, cx, top, node.cw, node.ch);
            // 内容盒即子节点的布局盒，子节点按自己的盒中线对齐
            place(child, cx, cy, depth + 1, out, hits, selected, is_selected);

            // 量词行画在内容盒正下方：重复图标 + 文字 +（无上界时）∞ 图标
            let ty = top + node.ch + (LABEL_H - LABEL_FS * 1.5) / 2.0;
            let mut cursor = x + (node.w - label_w) / 2.0;
            let icon_oy = ty + LABEL_FS * 0.75 - ICON_W / 2.0;
            out.push(Prim::Curve {
                cmds: repeat_icon_cmds(cursor, icon_oy, ICON_W / 24.0),
                stroke: GRAPH,
                filled: false,
            });
            cursor += ICON_W;
            out.push(Prim::Text {
                x: cursor,
                y: ty,
                w: *label_text_w,
                text: label.clone(),
                fs: LABEL_FS,
                color: GRAPH_FG,
                center: false,
            });
            if *infinite {
                out.push(Prim::Curve {
                    cmds: infinity_cmds(
                        cursor + label_text_w + ICON_W / 2.0,
                        ty + LABEL_FS * 0.75,
                        ICON_W,
                    ),
                    stroke: GRAPH,
                    filled: true,
                });
            }
        }
        GKind::Group { child, label } => {
            if let Some(label) = label {
                out.push(Prim::Text {
                    x,
                    y: top - LABEL_H,
                    w: node.w,
                    text: label.clone(),
                    fs: LABEL_FS,
                    color: GRAPH_FG,
                    center: true,
                });
            }
            out.push(Prim::RoundRect {
                x: cx,
                y: top,
                w: node.cw,
                h: node.ch,
                r: NODE_R,
                stroke: GRAPH_GROUP,
                dash: false,
            });
            if is_selected {
                out.push(select_highlight(cx, top, node.cw, node.ch));
            }
            mark(hits, node.id, cx, top, node.cw, node.ch);
            let inner = cx + GROUP_PAD_H;
            out.push(Prim::Line {
                x1: cx,
                y1: cy,
                x2: inner,
                y2: cy,
                stroke: GRAPH,
            });
            out.push(Prim::Line {
                x1: inner + child.w,
                y1: cy,
                x2: cx + node.cw,
                y2: cy,
                stroke: GRAPH,
            });
            place(child, inner, cy, depth + 1, out, hits, selected, is_selected);
        }
        GKind::Concat(items) => {
            let mut cursor = x;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(Prim::Line {
                        x1: cursor,
                        y1: cy,
                        x2: cursor + LINK,
                        y2: cy,
                        stroke: GRAPH,
                    });
                    cursor += LINK;
                }
                place(item, cursor, cy, depth + 1, out, hits, selected, inherited_sel);
                cursor += item.w;
            }
        }
        GKind::Alternate(rows) => {
            let top = cy - node.h / 2.0;
            let right = x + node.w;
            out.push(Prim::Frame {
                x,
                y: top,
                w: node.w,
                h: node.h,
            });
            // 官方 `T8`（ChoiceName）同样用 `hd` 包住整个分支容器，
            // 所以分支整体也是一个可选中节点
            if is_selected {
                out.push(select_highlight(x, top, node.w, node.h));
            }
            mark(hits, node.id, x, top, node.w, node.h);
            let mut row_top = top + BRANCH_PAD_V;
            for row in rows {
                let row_x = x + (node.w - row.w) / 2.0;
                let row_cy = row_top + row.h / 2.0;
                out.push(Prim::Curve {
                    cmds: split_cmds(x, cy, row_cy, row_x, BRANCH_R),
                    stroke: GRAPH,
                    filled: false,
                });
                place(row, row_x, row_cy, depth + 1, out, hits, selected, inherited_sel);
                out.push(Prim::Curve {
                    cmds: merge_cmds(row_x + row.w, row_cy, cy, right, BRANCH_R),
                    stroke: GRAPH,
                    filled: false,
                });
                row_top += row.h + ROW_GAP;
            }
        }
    }
    node.w
}

/// 干线 → 分支行，对齐官方 `StartConnect`：
///
/// ```text
/// M x,cy  L x+10,cy  A5→(x+15, cy±5)  L (x+15, row_cy∓5)  A5→(x+20, row_cy)  L end_x,row_cy
/// ```
///
/// 竖段恒定落在 `x+15`；`end_x` 是分支行**布局盒**的左边界（行水平居中，
/// 因此不一定等于 `x+25`）。两端等高时官方退化成一条直线。
fn split_cmds(x: f32, cy: f32, row_cy: f32, end_x: f32, r: f32) -> Vec<Cmd> {
    if (row_cy - cy).abs() < 0.5 {
        return vec![Cmd::Move(x, cy), Cmd::Line(end_x, row_cy)];
    }
    let down = row_cy > cy;
    let mid = if down { cy + r } else { cy - r };
    let near = if down { row_cy - r } else { row_cy + r };
    vec![
        Cmd::Move(x, cy),
        Cmd::Line(x + 10.0, cy),
        Cmd::Arc {
            r,
            large: false,
            sweep: down,
            to: (x + 15.0, mid),
        },
        Cmd::Line(x + 15.0, near),
        Cmd::Arc {
            r,
            large: false,
            sweep: !down,
            to: (x + 20.0, row_cy),
        },
        Cmd::Line(end_x, row_cy),
    ]
}

/// 分支行 → 干线，对齐官方 `EndConnect`：
///
/// ```text
/// M row_right,row_cy  L right-20,row_cy  A5→(right-15, row_cy∓5)  L (right-15, cy±5)  A5→(right-10, cy)  L right,cy
/// ```
///
/// 关键：竖段锚在**容器右缘**左侧 15px，而不是行右缘右侧 5px——
/// 行越窄这个差别越明显。
fn merge_cmds(row_right: f32, row_cy: f32, cy: f32, container_right: f32, r: f32) -> Vec<Cmd> {
    if (row_cy - cy).abs() < 0.5 {
        return vec![Cmd::Move(row_right, row_cy), Cmd::Line(container_right, cy)];
    }
    // 官方判据 `end.y > start.y`：干线上方（row_cy < cy）时首段弧向顺时针
    let above = cy > row_cy;
    let near = if above { row_cy + r } else { row_cy - r };
    let mid = if above { cy - r } else { cy + r };
    let ax = container_right - 15.0;
    vec![
        Cmd::Move(row_right, row_cy),
        Cmd::Line(container_right - 20.0, row_cy),
        Cmd::Arc {
            r,
            large: false,
            sweep: above,
            to: (ax, near),
        },
        Cmd::Line(ax, mid),
        Cmd::Arc {
            r,
            large: false,
            sweep: !above,
            to: (container_right - 10.0, cy),
        },
        Cmd::Line(container_right, cy),
    ]
}

fn mv(p: (f32, f32)) -> Cmd {
    Cmd::Move(p.0, p.1)
}

fn ln(p: (f32, f32)) -> Cmd {
    Cmd::Line(p.0, p.1)
}

/// 官方重复图标（lucide repeat，24×24 viewBox），按 s 缩放
fn repeat_icon_cmds(ox: f32, oy: f32, s: f32) -> Vec<Cmd> {
    let p = |x: f32, y: f32| (ox + x * s, oy + y * s);
    let r = 4.0 * s;
    vec![
        mv(p(17.0, 1.0)),
        ln(p(21.0, 5.0)),
        ln(p(17.0, 9.0)),
        mv(p(3.0, 11.0)),
        ln(p(3.0, 9.0)),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: p(7.0, 5.0),
        },
        ln(p(21.0, 5.0)),
        mv(p(21.0, 13.0)),
        ln(p(21.0, 15.0)),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: p(17.0, 19.0),
        },
        ln(p(3.0, 19.0)),
        mv(p(7.0, 23.0)),
        ln(p(3.0, 19.0)),
        ln(p(7.0, 15.0)),
    ]
}

/// 官方 ∞ 图标：Phosphor「infinity」16 进制路径（`viewBox="0 0 256 256"`，
/// `fill="currentColor"`）。官方把它渲染成 18×18 的**填充**图形，这里按
/// `size / 256` 缩放后照样填充。
fn infinity_cmds(cx: f32, cy: f32, size: f32) -> Vec<Cmd> {
    let s = size / 256.0;
    // 官方把图标放在 18×18 盒里，路径本身居中于 (128,128)
    let ox = cx - 128.0 * s;
    let oy = cy - 128.0 * s;
    let p = |x: f32, y: f32| (ox + x * s, oy + y * s);
    let r = |v: f32| v * s;
    vec![
        mv(p(248.0, 128.0)),
        Cmd::Arc {
            r: r(56.0),
            large: false,
            sweep: true,
            to: p(152.4, 167.6),
        },
        ln(p(152.07, 167.25)),
        ln(p(92.12, 99.55)),
        Cmd::Arc {
            r: r(40.0),
            large: true,
            sweep: false,
            to: p(92.12, 156.45),
        },
        ln(p(100.64, 146.83)),
        Cmd::Arc {
            r: r(8.0),
            large: true,
            sweep: true,
            to: p(112.64, 157.44),
        },
        ln(p(103.95, 167.25)),
        ln(p(103.62, 167.6)),
        Cmd::Arc {
            r: r(56.0),
            large: true,
            sweep: true,
            to: p(103.62, 88.4),
        },
        ln(p(103.95, 88.75)),
        ln(p(163.9, 156.45)),
        Cmd::Arc {
            r: r(40.0),
            large: true,
            sweep: false,
            to: p(163.9, 99.55),
        },
        ln(p(155.38, 109.17)),
        Cmd::Arc {
            r: r(8.0),
            large: true,
            sweep: true,
            to: p(143.38, 98.56),
        },
        ln(p(152.07, 88.75)),
        ln(p(152.4, 88.4)),
        Cmd::Arc {
            r: r(56.0),
            large: false,
            sweep: true,
            to: p(248.0, 128.0),
        },
        Cmd::Close,
    ]
}

// ===========================================================================
// 语法树 → GNode
// ===========================================================================

// ===========================================================================
// 可编辑 AST → 布局节点
// ===========================================================================

/// 序列 → 布局节点。官方没有 concat 节点：0 项渲染 Empty，1 项就是它本身，
/// 多项才套一层拼接（官方 `Ch`）。
fn gnode_seq(nodes: &[ENode], lang: Lang, window: &mut Window, family: &SharedString) -> GNode {
    match nodes.len() {
        0 => token_node(empty_label(lang).to_string(), None, false, window, family),
        1 => gnode(&nodes[0], lang, window, family),
        _ => build_concat(
            nodes
                .iter()
                .map(|n| gnode(n, lang, window, family))
                .collect(),
        ),
    }
}

/// 节点 → 布局节点。
///
/// 官方把 `quantifier` 挂在节点**自己身上**，只有 character / group /
/// backReference 三种能带（`y0`）。布局上等价于套一层量词外壳，但 id 属于
/// 外层：内层要清成 0，免得同一块矩形登记出两个命中区。
fn gnode(node: &ENode, lang: Lang, window: &mut Window, family: &SharedString) -> GNode {
    let mut layout = match &node.kind {
        EKind::Group { children, .. } | EKind::LookAround { children, .. } => {
            let child = gnode_seq(children, lang, window, family);
            group_node(node_label(node, lang), child, window, family)
        }
        EKind::Choice { branches } => build_alternate(
            branches
                .iter()
                .map(|b| gnode_seq(b, lang, window, family))
                .collect(),
        ),
        _ => token_node(
            node_text(node, lang),
            node_label(node, lang),
            node.dashed(),
            window,
            family,
        ),
    };

    match node.quantifier {
        Some(q) if node.accepts_quantifier() => {
            let (min, max) = match q.kind {
                QuantKind::Star => (0u32, None),
                QuantKind::Plus => (1u32, None),
                QuantKind::Question => (0u32, Some(1u32)),
                QuantKind::Custom => (q.min, if q.infinite() { None } else { Some(q.max) }),
            };
            let mut outer = repeat_node(layout, min, max, window, family);
            outer.id = node.id;
            outer
        }
        _ => {
            layout.id = node.id;
            layout
        }
    }
}

fn build_concat(items: Vec<GNode>) -> GNode {
    let w = items.iter().map(|i| i.w).sum::<f32>()
        + if items.is_empty() {
            0.0
        } else {
            LINK * (items.len() as f32 - 1.0)
        };
    let h = items.iter().map(|i| i.h).fold(0.0f32, f32::max);
    // 官方 Ch() 对拼接结果套一层最小尺寸约束
    let w = w.max(MIN_W);
    let h = h.max(MIN_H);
    GNode {
        id: 0,
        kind: GKind::Concat(items),
        w,
        h,
        cw: w,
        ch: h,
    }
}

fn build_alternate(rows: Vec<GNode>) -> GNode {
    let max_w = rows.iter().map(|r| r.w).fold(0.0f32, f32::max);
    let h = rows.iter().map(|r| r.h).sum::<f32>()
        + if rows.is_empty() {
            0.0
        } else {
            ROW_GAP * (rows.len() as f32 - 1.0)
        }
        + BRANCH_PAD_V * 2.0;
    // 官方 OM()：宽 = 最宽行 + 左右各 25
    let w = (max_w + BRANCH_INDENT * 2.0).max(MIN_W);
    let h = h.max(MIN_H);
    GNode {
        id: 0,
        kind: GKind::Alternate(rows),
        w,
        h,
        cw: w,
        ch: h,
    }
}

/// 官方 O8 的分组外壳：内容盒四边各留 (GROUP_PAD_H, GROUP_PAD_V)，
/// 有标签时上方再加一行装饰（LABEL_BOX * 2）。具名/普通分组、以及**环视断言**
/// 在官方渲染里用的是同一个外壳，区别只在标签文案。
///
/// （环视自己不走这里——`regex-syntax` 的 AST 没有 lookaround 变体，
///   我的解析器拿不到它；这里只服务真分组和图例里的示意节点。）
fn group_node(
    label: Option<String>,
    child: GNode,
    window: &mut Window,
    family: &SharedString,
) -> GNode {
    let label_w = match &label {
        Some(l) => label_node_w(l, window, family),
        None => 0.0,
    };
    let cw = child.w + GROUP_PAD_H * 2.0;
    let ch = child.h + GROUP_PAD_V * 2.0;
    let dh = if label_w > 0.0 { LABEL_BOX * 2.0 } else { 0.0 };
    GNode {
        id: 0,
        kind: GKind::Group {
            child: Box::new(child),
            label,
        },
        w: cw.max(label_w),
        h: ch + dh,
        cw,
        ch,
    }
}

/// 量词外壳。内容盒取子节点的**绘制盒**，上下各留一行（标签行 + 量词行）。
fn repeat_node(
    child: GNode,
    min: u32,
    max: Option<u32>,
    window: &mut Window,
    family: &SharedString,
) -> GNode {
    let (label, label_text_w, label_w, infinite) = repeat_label(min, max, window, family);
    let (cw, ch) = (child.cw, child.ch);
    let child_w = child.w;
    GNode {
        id: 0,
        kind: GKind::Repeat {
            child: Box::new(child),
            label,
            label_text_w,
            label_w,
            infinite,
        },
        w: child_w.max(label_w),
        h: ch + LABEL_BOX * 2.0,
        cw,
        ch,
    }
}

fn ast_text(src: &str, span: &regex_syntax::ast::Span) -> String {
    src.get(span.start.offset..span.end.offset)
        .unwrap_or("")
        .to_string()
}

fn unicode_class_name(kind: &regex_syntax::ast::ClassUnicodeKind) -> String {
    use regex_syntax::ast::ClassUnicodeKind;
    match kind {
        ClassUnicodeKind::OneLetter(c) => c.to_string(),
        ClassUnicodeKind::Named(n) => n.to_string(),
        ClassUnicodeKind::NamedValue { name, value, .. } => format!("{name}={value}"),
    }
}

// ===========================================================================
// 根布局
// ===========================================================================

/// 无选中态的轨道图（测试与图例预览用）
fn layout_diagram(root: &GNode) -> Diagram {
    layout_diagram_selected(root, &[])
}

/// 带选中态的完整轨道图
fn layout_diagram_selected(root: &GNode, selected: &[NodeId]) -> Diagram {
    let left = ROOT_PAD + MARK_R * 2.0 + LINK;
    let width = root.w + left * 2.0;
    let height = root.h + ROOT_PAD * 2.0;
    let cy = height / 2.0;
    let mut prims = Vec::new();
    let mut hits = Vec::new();
    prims.push(Prim::Circle {
        cx: ROOT_PAD + MARK_R,
        cy,
        r: MARK_R,
        stroke: GRAPH,
    });
    prims.push(Prim::Line {
        x1: ROOT_PAD + MARK_R * 2.0,
        y1: cy,
        x2: left,
        y2: cy,
        stroke: GRAPH,
    });
    place(root, left, cy, 0, &mut prims, &mut hits, selected, false);
    prims.push(Prim::Line {
        x1: width - left,
        y1: cy,
        x2: width - ROOT_PAD - MARK_R * 2.0,
        y2: cy,
        stroke: GRAPH,
    });
    prims.push(Prim::Circle {
        cx: width - ROOT_PAD - MARK_R,
        cy,
        r: MARK_R,
        stroke: GRAPH,
    });
    Diagram {
        prims,
        hits,
        width,
        height,
    }
}

fn prims_bbox(prims: &[Prim]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for prim in prims {
        let (x0, y0, x1, y1) = match prim {
            Prim::RoundRect { x, y, w, h, .. } => (*x, *y, x + w, y + h),
            Prim::Frame { x, y, w, h } => (*x, *y, x + w, y + h),
            Prim::Highlight { x, y, w, h, .. } => (*x, *y, x + w, y + h),
            Prim::Line { x1, y1, x2, y2, .. } => (x1.min(*x2), y1.min(*y2), x1.max(*x2), y1.max(*y2)),
            Prim::Curve { cmds, .. } => {
                let mut b = (
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                );
                for c in cmds {
                    let (cx, cy) = match c {
                        Cmd::Move(x, y) | Cmd::Line(x, y) => (*x, *y),
                        Cmd::Arc { to, .. } => *to,
                        Cmd::Close => continue,
                    };
                    b.0 = b.0.min(cx);
                    b.1 = b.1.min(cy);
                    b.2 = b.2.max(cx);
                    b.3 = b.3.max(cy);
                }
                b
            }
            Prim::Circle { cx, cy, r, .. } => (cx - r, cy - r, cx + r, cy + r),
            Prim::Text { x, y, w, fs, .. } => (*x, *y, x + w, y + fs * 1.5),
        };
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    }
    (min_x, min_y, max_x, max_y)
}

fn translate_prim(prim: &mut Prim, dx: f32, dy: f32) {
    match prim {
        Prim::RoundRect { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Prim::Frame { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Prim::Highlight { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Prim::Line { x1, y1, x2, y2, .. } => {
            *x1 += dx;
            *y1 += dy;
            *x2 += dx;
            *y2 += dy;
        }
        Prim::Curve { cmds, .. } => {
            for c in cmds.iter_mut() {
                match c {
                    Cmd::Move(x, y) | Cmd::Line(x, y) => {
                        *x += dx;
                        *y += dy;
                    }
                    Cmd::Arc { to, .. } => {
                        to.0 += dx;
                        to.1 += dy;
                    }
                    Cmd::Close => {}
                }
            }
        }
        Prim::Circle { cx, cy, .. } => {
            *cx += dx;
            *cy += dy;
        }
        Prim::Text { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
    }
}

/// 图例/预览用：只画内容，不带起止圆环，四周留 10px
fn layout_content(root: &GNode) -> Diagram {
    let mut prims = Vec::new();
    let mut hits = Vec::new();
    place(root, 0.0, 0.0, 0, &mut prims, &mut hits, &[], false);
    let (min_x, min_y, max_x, max_y) = prims_bbox(&prims);
    let dx = 10.0 - min_x;
    let dy = 10.0 - min_y;
    for prim in prims.iter_mut() {
        translate_prim(prim, dx, dy);
    }
    for hit in hits.iter_mut() {
        hit.x += dx;
        hit.y += dy;
    }
    Diagram {
        prims,
        hits,
        width: (max_x - min_x) + 20.0,
        height: (max_y - min_y) + 20.0,
    }
}

// ===========================================================================
// canvas 绘制
// ===========================================================================

fn build_path(cmds: &[Cmd], origin: Point<Pixels>, dash: bool, filled: bool) -> Option<Path<Pixels>> {
    let mut builder = if filled {
        PathBuilder::fill()
    } else {
        PathBuilder::stroke(px(STROKE))
    };
    if dash {
        builder = builder.dash_array(&[px(4.0), px(2.0)]);
    }
    for cmd in cmds {
        match cmd {
            Cmd::Move(x, y) => builder.move_to(point(origin.x + px(*x), origin.y + px(*y))),
            Cmd::Line(x, y) => builder.line_to(point(origin.x + px(*x), origin.y + px(*y))),
            Cmd::Arc {
                r,
                large,
                sweep,
                to,
            } => builder.arc_to(
                point(px(*r), px(*r)),
                px(0.0),
                *large,
                *sweep,
                point(origin.x + px(to.0), origin.y + px(to.1)),
            ),
            Cmd::Close => builder.close(),
        }
    }
    builder.build().ok()
}

fn rounded_rect_cmds(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Cmd> {
    let r = r.min(w / 2.0).min(h / 2.0);
    vec![
        Cmd::Move(x + r, y),
        Cmd::Line(x + w - r, y),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (x + w, y + r),
        },
        Cmd::Line(x + w, y + h - r),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (x + w - r, y + h),
        },
        Cmd::Line(x + r, y + h),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (x, y + h - r),
        },
        Cmd::Line(x, y + r),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (x + r, y),
        },
    ]
}

fn circle_cmds(cx: f32, cy: f32, r: f32) -> Vec<Cmd> {
    vec![
        Cmd::Move(cx - r, cy),
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (cx + r, cy),
        },
        Cmd::Arc {
            r,
            large: false,
            sweep: true,
            to: (cx - r, cy),
        },
    ]
}

fn paint_diagram(
    prims: &[Prim],
    origin: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
    family: &SharedString,
) {
    for prim in prims {
        match prim {
            Prim::RoundRect {
                x,
                y,
                w,
                h,
                r,
                stroke,
                dash,
            } => {
                let cmds = rounded_rect_cmds(*x, *y, *w, *h, *r);
                if let Some(path) = build_path(&cmds, origin, *dash, false) {
                    window.paint_path(path, Hsla::from(rgb(*stroke)));
                }
            }
            // 官方分支容器为 `fill-transparent` 且无描边，页面上不可见
            Prim::Frame { .. } => {}
            Prim::Highlight { x, y, w, h, r } => {
                let cmds = rounded_rect_cmds(*x, *y, *w, *h, *r);
                if let Some(path) = build_path(&cmds, origin, false, true) {
                    let color = Hsla {
                        a: SELECT_ALPHA,
                        ..Hsla::from(rgb(SELECT_BLUE))
                    };
                    window.paint_path(path, color);
                }
            }
            Prim::Line {
                x1,
                y1,
                x2,
                y2,
                stroke,
            } => {
                let cmds = [Cmd::Move(*x1, *y1), Cmd::Line(*x2, *y2)];
                if let Some(path) = build_path(&cmds, origin, false, false) {
                    window.paint_path(path, Hsla::from(rgb(*stroke)));
                }
            }
            Prim::Curve {
                cmds,
                stroke,
                filled,
            } => {
                if let Some(path) = build_path(cmds, origin, false, *filled) {
                    window.paint_path(path, Hsla::from(rgb(*stroke)));
                }
            }
            Prim::Circle {
                cx: ccx,
                cy: ccy,
                r,
                stroke,
            } => {
                if let Some(path) = build_path(&circle_cmds(*ccx, *ccy, *r), origin, false, false) {
                    window.paint_path(path, Hsla::from(rgb(*stroke)));
                }
            }
            Prim::Text {
                x,
                y,
                w,
                text,
                fs,
                color,
                center,
            } => {
                if text.is_empty() {
                    continue;
                }
                let line = shape_mono(text, *fs, Hsla::from(rgb(*color)), window, family);
                let width = line.width().as_f32();
                let ox = if *center {
                    origin.x + px(*x + (*w - width) / 2.0)
                } else {
                    origin.x + px(*x)
                };
                let _ = line.paint(
                    point(ox, origin.y + px(*y)),
                    px(*fs),
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
        }
    }
}

/// 轨道图画布：官方 svg 带 class="rounded-lg border"
fn diagram_canvas(diagram: &Diagram, cx: &App) -> Div {
    let prims = diagram.prims.clone();
    let width = diagram.width;
    let height = diagram.height;
    div()
        .w(px(width))
        .h(px(height))
        .flex_shrink_0()
        .rounded(px(8.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(Hsla::from(rgb(GRAPH_BG)))
        .child(
            canvas(
                move |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window: &mut Window, cx: &mut App| {
                    let family: SharedString = cx.theme().mono_font_family.clone();
                    paint_diagram(&prims, bounds.origin, window, cx, &family);
                },
            )
            .size_full(),
        )
}

/// 主轨道图：在 `diagram_canvas` 基础上挂上「点选 / 框选」交互。
///
/// 画布的绘制原点要靠 `canvas` 的 paint 回调才能拿到（布局居中 + 内边距都不固定），
/// 所以这里用一个 `Rc<Cell>` 把 paint 期的原点转交给鼠标事件处理。
fn interactive_diagram_canvas(
    diagram: &Diagram,
    marquee: Option<((f32, f32), (f32, f32))>,
    cx: &Context<RegexVisualizer>,
) -> Div {
    let prims = diagram.prims.clone();
    let width = diagram.width;
    let height = diagram.height;

    let origin: std::rc::Rc<std::cell::Cell<(f32, f32)>> =
        std::rc::Rc::new(std::cell::Cell::new((0.0, 0.0)));
    let paint_origin = origin.clone();
    let down_origin = origin.clone();
    let move_origin = origin.clone();
    let up_origin = origin.clone();

    let mut wrapper = div()
        .w(px(width))
        .h(px(height))
        .flex_shrink_0()
        .relative()
        .rounded(px(8.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(Hsla::from(rgb(GRAPH_BG)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                let (ox, oy) = down_origin.get();
                let p = (
                    f32::from(ev.position.x) - ox,
                    f32::from(ev.position.y) - oy,
                );
                // 命中节点就直接单选；空白处按下开始框选
                match deepest_hit(&this.diagram, p.0, p.1) {
                    Some(id) => {
                        this.marquee = None;
                        this.select_node(id, window, cx);
                    }
                    None => {
                        this.marquee = Some((p, p));
                        cx.notify();
                    }
                }
            }),
        )
        .on_mouse_move(cx.listener(move |this, ev: &MouseMoveEvent, _, cx| {
            if let Some((start, _)) = this.marquee {
                let (ox, oy) = move_origin.get();
                let p = (
                    f32::from(ev.position.x) - ox,
                    f32::from(ev.position.y) - oy,
                );
                this.marquee = Some((start, p));
                cx.notify();
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, ev: &MouseUpEvent, window, cx| {
                let Some((start, _)) = this.marquee.take() else {
                    return;
                };
                let (ox, oy) = up_origin.get();
                let end = (
                    f32::from(ev.position.x) - ox,
                    f32::from(ev.position.y) - oy,
                );
                let (x0, x1) = (start.0.min(end.0), start.0.max(end.0));
                let (y0, y1) = (start.1.min(end.1), start.1.max(end.1));
                // 位移太小当作「点空白」——清空选中
                if x1 - x0 < 3.0 && y1 - y0 < 3.0 {
                    this.clear_selection(window, cx);
                    return;
                }
                let ids: Vec<NodeId> = this
                    .diagram
                    .as_ref()
                    .map(|d| {
                        d.hits
                            .iter()
                            .filter(|h| {
                                h.x < x1 && h.x + h.w > x0 && h.y < y1 && h.y + h.h > y0
                            })
                            .map(|h| h.id)
                            .collect()
                    })
                    .unwrap_or_default();
                this.set_selection(ids, window, cx);
            }),
        )
        .child(
            canvas(
                move |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window: &mut Window, cx: &mut App| {
                    paint_origin.set((f32::from(bounds.origin.x), f32::from(bounds.origin.y)));
                    let family: SharedString = cx.theme().mono_font_family.clone();
                    paint_diagram(&prims, bounds.origin, window, cx, &family);
                },
            )
            .size_full(),
        );

    // 橡皮筋矩形：官方用的是半透明蓝底 + 蓝边
    if let Some((start, current)) = marquee {
        let x = start.0.min(current.0);
        let y = start.1.min(current.1);
        let w = (current.0 - start.0).abs();
        let h = (current.1 - start.1).abs();
        wrapper = wrapper.child(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(w))
                .h(px(h))
                .border_1()
                .border_color(Hsla::from(rgb(SELECT_BLUE)))
                .bg(Hsla {
                    a: 0.15,
                    ..Hsla::from(rgb(SELECT_BLUE))
                }),
        );
    }

    wrapper
}

/// 命中测试：取最深的那个（等价于官方靠 DOM `stopPropagation` 的最内层优先）
fn deepest_hit(diagram: &Option<Diagram>, x: f32, y: f32) -> Option<NodeId> {
    diagram
        .as_ref()?
        .hits
        .iter()
        .filter(|h| h.contains(x, y))
        .max_by_key(|h| h.depth)
        .map(|h| h.id)
}

// ===========================================================================
// 视图
// ===========================================================================

#[derive(Clone)]
struct RegexMatch {
    index: usize,
    start: usize,
    end: usize,
    text: String,
    groups: Vec<String>,
}

/// 图例中的单项：一个小轨道图 + 一句说明。
/// 对齐官方 yB 的 `{Icon, desc}`。
struct LegendEntry {
    desc: &'static str,
    diagram: Diagram,
}

/// 图例分组。官方 yB 共 8 组 / 13 项：
/// Characters / Character classes / Ranges(2) / Choice / Quantifier /
/// Group(3) / Back reference(2) / Assertion(2)。
struct LegendGroup {
    title: &'static str,
    entries: Vec<LegendEntry>,
}

pub struct RegexVisualizer {
    pattern: String,
    test_text: String,
    flags: [bool; 4],
    lang: Lang,
    panel_tab: usize,
    panel_collapsed: bool,
    diagram: Option<Diagram>,
    legend: Vec<LegendGroup>,
    matches: Vec<RegexMatch>,
    error: String,
    mono_family: SharedString,
    pattern_state: Entity<InputState>,
    text_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,

    // —— 可编辑 AST ——
    /// 当前树。`evaluate` 只在原文变化时重建它，所以编辑期 id 保持稳定。
    tree: ERoot,
    /// 建出 `tree` 的那一版原文。和 `pattern` 相等说明树是本轮最新的。
    tree_source: String,
    /// 选中节点（官方 `je`）
    selected: Vec<NodeId>,
    /// id 发生器水位（官方 `Yt` 的全局计数器）
    next_id: u32,
    undo_stack: Vec<ERoot>,
    redo_stack: Vec<ERoot>,
    /// 编辑面板输入框池（只补不覆盖，避免每敲一个字控件都被重置）
    edit_inputs: std::collections::HashMap<EditSlot, (Entity<InputState>, Subscription)>,
    /// 编辑面板下拉框池
    edit_selects: std::collections::HashMap<
        EditSlot,
        (Entity<SelectState<Vec<LabeledItem>>>, Subscription),
    >,
    /// 官方 `iB` 的折叠开关（`useLocalStorage(id, false)`，默认收起）
    show_lookaround: bool,
    /// 正在框选时的橡皮筋矩形（画布局部坐标，`(起点, 当前点)`）
    marquee: Option<((f32, f32), (f32, f32))>,
}

impl RegexVisualizer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mono_family: SharedString = cx.theme().mono_font_family.clone();

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
                move |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        this.pattern = pattern_state.read(cx).value().to_string();
                        // 编辑器回写造成的回声也走这里，但那时 `tree_source == pattern`，
                        // `evaluate` 不会重新解析，节点 id 与选中态都保得住。
                        this.evaluate(window, cx);
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&text_state, window, {
                let text_state = text_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.test_text = text_state.read(cx).value().to_string();
                        this.re_match();
                        cx.notify();
                    }
                }
            }),
        ];

        let mut this = Self {
            pattern: r"\w+".to_string(),
            test_text: "hello gpui\n123 toolbox".to_string(),
            flags: [false; 4],
            lang: Lang::Cn,
            panel_tab: TAB_LEGEND,
            panel_collapsed: false,
            diagram: None,
            legend: Vec::new(),
            matches: Vec::new(),
            error: String::new(),
            mono_family,
            pattern_state,
            text_state,
            _subscriptions,
            tree: ERoot::default(),
            tree_source: String::new(),
            selected: Vec::new(),
            next_id: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            edit_inputs: std::collections::HashMap::new(),
            edit_selects: std::collections::HashMap::new(),
            show_lookaround: false,
            marquee: None,
        };
        this.legend = build_legend(window, &this.mono_family, this.lang);
        this.evaluate(window, cx);
        this
    }

    /// 重建可编辑 AST 与轨道图。
    ///
    /// **只有正则原文变化时才重新解析**：编辑器的每次改动都会先把结果回写到
    /// `self.pattern` 并同步 `tree_source`，所以节点 id 能跨操作保持稳定，
    /// 选中态与撤销栈也不会被误清。
    fn evaluate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let parsed_ok = if self.tree_source != self.pattern {
            match regex_syntax::ast::parse::Parser::new().parse(&self.pattern) {
                Ok(ast) => {
                    let mut id_gen = IdGen(self.next_id);
                    let body = convert_seq(&ast, &self.pattern, &mut id_gen);
                    self.next_id = id_gen.0;
                    self.tree = ERoot { body };
                    true
                }
                Err(_) => {
                    self.tree = ERoot::default();
                    false
                }
            }
        } else {
            // 树是本轮的原文构建出来的；编辑器仍可能产出**语义上**非法的正则
            // （典型是重名分组），所以再验一次。空串是合法正则（图里渲染成
            // Empty 节点），因此这里不加 `is_empty` 短路 —— 否则首尾两次
            // `evaluate` 会对同一个空串给出不一致的结果。
            regex_syntax::ast::parse::Parser::new()
                .parse(&self.pattern)
                .is_ok()
        };
        self.tree_source = self.pattern.clone();
        if parsed_ok {
            // 原文被外部改动过（不是编辑器回写）时才清历史
        } else {
            self.undo_stack.clear();
            self.redo_stack.clear();
        }

        refresh_capture_index(&self.tree);

        // 节点可能已被替换 / 删除，丢掉失效的选中 id
        self.selected.retain(|id| self.tree.node(*id).is_some());

        self.diagram = if parsed_ok {
            let root = gnode_seq(&self.tree.body, self.lang, window, &self.mono_family);
            Some(layout_diagram_selected(&root, &self.selected))
        } else {
            None
        };

        self.re_match();
        self.sync_edit_widgets(window, cx);
    }

    /// 编辑器改完树之后回写正则。
    ///
    /// 同时把 `tree_source` 置成新原文，这样 `evaluate` 不会重新解析 —— id 得以保留。
    /// `InputState::set_value` 内部会关掉事件派发，所以这里不会引发输入框的 Change 回声。
    fn sync_pattern_from_tree(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pattern = self.tree.to_pattern();
        self.tree_source = self.pattern.clone();
        let value = self.pattern.clone();
        self.pattern_state
            .update(cx, |state, cx| state.set_value(value, window, cx));
        self.evaluate(window, cx);
        cx.notify();
    }

    /// 所有编辑操作的统一入口：压撤销栈 → 改树 → 回写正则 → 重建。
    ///
    /// 闭包返回新的选中 id；返回空则沿用原选中（对应官方插入操作不动 `je`）。
    fn edit<F>(&mut self, window: &mut Window, cx: &mut Context<Self>, f: F)
    where
        F: FnOnce(&mut ERoot, &mut IdGen) -> Vec<NodeId>,
    {
        let before = self.tree.clone();
        let mut id_gen = IdGen(self.next_id);
        let selection = f(&mut self.tree, &mut id_gen);
        self.next_id = id_gen.0;
        self.undo_stack.push(before);
        if self.undo_stack.len() > UNDO_LIMIT {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        if !selection.is_empty() {
            self.selected = selection;
        }
        self.sync_pattern_from_tree(window, cx);
    }

    /// 官方 `hd` 的 onClick（`Rj`）：再点同一个节点取消选中，否则单选它
    fn select_node(&mut self, id: NodeId, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.len() == 1 && self.selected[0] == id {
            self.selected.clear();
        } else {
            self.selected = vec![id];
        }
        self.show_lookaround = false;
        self.relayout(window, cx);
    }

    /// 官方 `zj`：删掉当前选中的节点
    fn delete_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            return;
        }
        let ids = self.selected.clone();
        self.edit(window, cx, move |tree, _| {
            tree.remove_nodes(&ids);
            Vec::new()
        });
    }

    /// 框选：直接整批替换选中
    fn set_selection(&mut self, ids: Vec<NodeId>, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected == ids {
            self.relayout(window, cx);
            return;
        }
        self.selected = ids;
        self.show_lookaround = false;
        self.relayout(window, cx);
    }

    /// 官方 `fd`：清空选中
    fn clear_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            return;
        }
        self.selected.clear();
        self.show_lookaround = false;
        self.relayout(window, cx);
    }

    /// 只重排（不改 AST）——选中态变了但几何没变
    fn relayout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() || self.diagram.is_some() {
            let root = gnode_seq(&self.tree.body, self.lang, window, &self.mono_family);
            self.diagram = Some(layout_diagram_selected(&root, &self.selected));
        }
        self.sync_edit_widgets(window, cx);
        cx.notify();
    }

    /// 官方 `Bj` / `Uj`
    fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.tree.clone());
            self.tree = previous;
            self.selected.clear();
            // 控件池是「只补不覆盖」的，历史回退会让它们的值变陈旧，整批丢重建
            self.edit_inputs.clear();
            self.edit_selects.clear();
            self.sync_pattern_from_tree(window, cx);
        }
    }

    fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.tree.clone());
            self.tree = next;
            self.selected.clear();
            self.edit_inputs.clear();
            self.edit_selects.clear();
            self.sync_pattern_from_tree(window, cx);
        }
    }

    fn content_of(&self, id: NodeId) -> Option<ContentSpec> {
        self.tree.node(id).and_then(ContentSpec::from_node)
    }

    /// 当前选中对应的面板信息
    fn selection(&self) -> SelectionInfo {
        self.tree.selection(&self.selected)
    }

    // -----------------------------------------------------------------------
    // 控件池
    // -----------------------------------------------------------------------

    /// 按当前选中算出需要哪些控件，补建缺失的、丢掉多余的。
    ///
    /// **只补不覆盖**：已存在的槽位原样保留，否则用户每敲一个字控件都会被
    /// 重置、光标跳回开头。槽位 key 里带节点 id，所以换选中节点时自然是一批新控件。
    fn sync_edit_widgets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (inputs, selects) = self.edit_field_plan();

        for (slot, value) in inputs {
            if self.edit_inputs.contains_key(&slot) {
                continue;
            }
            let state: Entity<InputState> =
                cx.new(|cx| InputState::new(window, cx).default_value(value));
            let sub = cx.subscribe_in(&state, window, {
                let slot = slot.clone();
                move |this: &mut RegexVisualizer, input, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        let value = input.read(cx).value().to_string();
                        this.on_edit_input(&slot, &value, window, cx);
                    }
                }
            });
            self.edit_inputs.insert(slot, (state, sub));
        }

        for (slot, items, selected) in selects {
            if self.edit_selects.contains_key(&slot) {
                continue;
            }
            let state: Entity<SelectState<Vec<LabeledItem>>> = cx.new(|cx| {
                let mut state = SelectState::new(items, None, window, cx);
                state.set_selected_value(&selected, window, cx);
                state
            });
            let sub = cx.subscribe_in(&state, window, {
                let slot = slot.clone();
                move |this: &mut RegexVisualizer, _, ev: &SelectEvent<Vec<LabeledItem>>, window, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        let value = value.clone();
                        this.on_edit_select(&slot, &value, window, cx);
                    }
                }
            });
            self.edit_selects.insert(slot, (state, sub));
        }

        let (wanted_inputs, wanted_selects) = self.edit_field_plan();
        let input_keys: Vec<EditSlot> = wanted_inputs.into_iter().map(|(s, _)| s).collect();
        let select_keys: Vec<EditSlot> = wanted_selects.into_iter().map(|(s, _, _)| s).collect();
        self.edit_inputs.retain(|k, _| input_keys.contains(k));
        self.edit_selects.retain(|k, _| select_keys.contains(k));
    }

    /// 面板需要哪些控件。返回（输入框，下拉框）。
    fn edit_field_plan(
        &self,
    ) -> (
        Vec<(EditSlot, String)>,
        Vec<(EditSlot, Vec<LabeledItem>, String)>,
    ) {
        let mut inputs: Vec<(EditSlot, String)> = Vec::new();
        let mut selects: Vec<(EditSlot, Vec<LabeledItem>, String)> = Vec::new();
        if self.panel_tab != TAB_EDIT || self.selected.is_empty() {
            return (inputs, selects);
        }
        let info = self.selection();
        let id = info.id;

        if let Some(content) = &info.content {
            let keys = content_type_keys(&info, self.tree.capture_names().len());
            let items: Vec<LabeledItem> = keys
                .iter()
                .filter_map(|key| {
                    CONTENT_TYPES
                        .iter()
                        .find(|(k, _, _)| k == key)
                        .map(|(k, cn, en)| LabeledItem::new(*k, self.lang.of(cn, en)))
                })
                .collect();
            selects.push((
                EditSlot::ContentType(id),
                items,
                content.key().to_string(),
            ));
            match content {
                ContentSpec::String { value } => {
                    inputs.push((EditSlot::Value(id), value.clone()));
                }
                ContentSpec::Class { value } => {
                    let kind = class_kind_key(value);
                    selects.push((
                        EditSlot::ClassKind(id),
                        class_items(self.lang),
                        kind.clone(),
                    ));
                    if kind != *value {
                        inputs.push((EditSlot::Value(id), value.clone()));
                    }
                }
                ContentSpec::Ranges { ranges, .. } => {
                    for (i, (from, to)) in ranges.iter().enumerate() {
                        inputs.push((EditSlot::RangeFrom(id, i), from.clone()));
                        inputs.push((EditSlot::RangeTo(id, i), to.clone()));
                    }
                }
                ContentSpec::BackReference { reference } => {
                    let mut names = self.tree.capture_names();
                    if !names.contains(reference) {
                        names.insert(0, reference.clone());
                    }
                    let items: Vec<LabeledItem> = names
                        .iter()
                        .map(|name| {
                            LabeledItem::new(
                                name.clone(),
                                format!("{} #{name}", group_word(self.lang)),
                            )
                        })
                        .collect();
                    selects.push((EditSlot::Backref(id), items, reference.clone()));
                }
                _ => {}
            }
        }

        if info.has_quantifier {
            let items: Vec<LabeledItem> = QUANT_OPTIONS
                .iter()
                .map(|(key, cn, en, _)| LabeledItem::new(*key, self.lang.of(cn, en)))
                .collect();
            selects.push((
                EditSlot::QuantKind(id),
                items,
                quant_key(info.quantifier.as_ref()).to_string(),
            ));
            if let Some(q) = &info.quantifier {
                if q.kind == QuantKind::Custom {
                    inputs.push((EditSlot::QuantMin(id), q.min.to_string()));
                    inputs.push((
                        EditSlot::QuantMax(id),
                        if q.infinite() {
                            String::new()
                        } else {
                            q.max.to_string()
                        },
                    ));
                }
            }
        }

        if let Some((EGroupKind::NamedCapturing, name)) = &info.group {
            inputs.push((EditSlot::GroupName(id), name.clone()));
        }

        // 分组详情：类型下拉（官方 `Kz`）
        if let Some((kind, _)) = &info.group {
            let items = vec![
                LabeledItem::new("capturing", self.lang.of("捕获组", "Capturing group")),
                LabeledItem::new("nonCapturing", self.lang.of("非捕获组", "Non-capturing group")),
                LabeledItem::new("namedCapturing", self.lang.of("具名捕获组", "Named capturing group")),
            ];
            selects.push((
                EditSlot::GroupKind(id),
                items,
                group_kind_key(*kind).to_string(),
            ));
        }

        // 断言详情：类型下拉（官方 `oB` 里的两项）
        if let Some((kind, _)) = info.lookaround {
            let items = vec![
                LabeledItem::new("lookahead", self.lang.of("向前断言", "Lookahead assertion")),
                LabeledItem::new("lookbehind", self.lang.of("向后断言", "Lookbehind assertion")),
            ];
            selects.push((
                EditSlot::LookKind(id),
                items,
                look_kind_key(kind).to_string(),
            ));
        }

        (inputs, selects)
    }

    /// 输入框改动。`value` 来自控件，先与树里的现值比对，相同就是回写造成的回声。
    fn on_edit_input(
        &mut self,
        slot: &EditSlot,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match slot.clone() {
            EditSlot::Value(id) => {
                let next = match self.content_of(id) {
                    Some(ContentSpec::String { .. }) => ContentSpec::String {
                        value: value.to_string(),
                    },
                    Some(ContentSpec::Class { .. }) => ContentSpec::Class {
                        value: value.to_string(),
                    },
                    _ => return,
                };
                if self.content_of(id).as_ref() == Some(&next) {
                    return;
                }
                self.edit(window, cx, |tree, id_gen| vec![tree.set_content(id, &next, id_gen)]);
            }
            EditSlot::RangeFrom(id, i) => self.update_range(id, i, true, value, window, cx),
            EditSlot::RangeTo(id, i) => self.update_range(id, i, false, value, window, cx),
            EditSlot::QuantMin(id) | EditSlot::QuantMax(id) => {
                let is_min = matches!(slot, EditSlot::QuantMin(_));
                let Some(Some(q)) = self.content_of(id).map(|_| self.quantifier_of(id)) else {
                    return;
                };
                let parsed = match value.trim() {
                    "" => {
                        if is_min {
                            0
                        } else {
                            Quantifier::INF
                        }
                    }
                    "Infinity" | "∞" => Quantifier::INF,
                    other => match other.parse::<u32>() {
                        Ok(n) => n,
                        Err(_) => return,
                    },
                };
                let (min, max) = if is_min {
                    (parsed, q.max)
                } else {
                    (q.min, parsed)
                };
                if min > max {
                    return;
                }
                let next = Quantifier {
                    kind: QuantKind::Custom,
                    min,
                    max,
                    greedy: q.greedy,
                };
                if q.kind == QuantKind::Custom && q.min == next.min && q.max == next.max {
                    return;
                }
                self.edit(window, cx, |tree, id_gen| {
                    vec![tree.set_quantifier(id, Some(next), id_gen)]
                });
            }
            EditSlot::GroupName(id) => {
                if self
                    .tree
                    .node(id)
                    .is_some_and(|n| matches!(&n.kind, EKind::Group { name, .. } if name == value))
                {
                    return;
                }
                let name = value.to_string();
                self.edit(window, cx, |tree, id_gen| {
                    let gen_ref = &mut *id_gen;
                    set_group_name(tree, id, &name);
                    let _ = gen_ref;
                    Vec::new()
                });
            }
            _ => {}
        }
    }

    fn quantifier_of(&self, id: NodeId) -> Option<Quantifier> {
        self.tree.node(id).and_then(|n| n.quantifier)
    }

    fn update_range(
        &mut self,
        id: NodeId,
        index: usize,
        is_from: bool,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ContentSpec::Ranges { ranges, negate }) = self.content_of(id) else {
            return;
        };
        let mut next = ranges.clone();
        let Some(slot) = next.get_mut(index) else {
            return;
        };
        if is_from {
            slot.0 = value.to_string();
        } else {
            slot.1 = value.to_string();
        }
        if next == ranges {
            return;
        }
        let spec = ContentSpec::Ranges {
            ranges: next,
            negate,
        };
        self.edit(window, cx, |tree, id_gen| vec![tree.set_content(id, &spec, id_gen)]);
    }

    /// 下拉框改动
    fn on_edit_select(
        &mut self,
        slot: &EditSlot,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match slot.clone() {
            EditSlot::ContentType(id) => {
                if self
                    .content_of(id)
                    .is_some_and(|c| c.key() == value)
                {
                    return;
                }
                let spec = ContentSpec::for_key(value);
                self.edit(window, cx, |tree, id_gen| vec![tree.set_content(id, &spec, id_gen)]);
            }
            EditSlot::ClassKind(id) => {
                let raw = match value {
                    "\\xhh" => "\\x00".to_string(),
                    "\\uhhhh" => "\\u0000".to_string(),
                    other => other.to_string(),
                };
                let spec = ContentSpec::Class { value: raw };
                if self.content_of(id).as_ref() == Some(&spec) {
                    return;
                }
                self.edit(window, cx, |tree, id_gen| vec![tree.set_content(id, &spec, id_gen)]);
            }
            EditSlot::Backref(id) => {
                let spec = ContentSpec::BackReference {
                    reference: value.to_string(),
                };
                if self.content_of(id).as_ref() == Some(&spec) {
                    return;
                }
                self.edit(window, cx, |tree, id_gen| vec![tree.set_content(id, &spec, id_gen)]);
            }
            EditSlot::QuantKind(id) => {
                let greedy = self.quantifier_of(id).map(|q| q.greedy).unwrap_or(true);
                let next = quant_for_key(value, greedy);
                self.edit(window, cx, |tree, id_gen| {
                    vec![tree.set_quantifier(id, next, id_gen)]
                });
            }
            EditSlot::GroupKind(id) => {
                let next = match value {
                    "capturing" => EGroupKind::Capturing,
                    "nonCapturing" => EGroupKind::NonCapturing,
                    _ => EGroupKind::NamedCapturing,
                };
                if self
                    .tree
                    .node(id)
                    .is_some_and(|n| matches!(&n.kind, EKind::Group { kind, .. } if *kind == next))
                {
                    return;
                }
                self.edit(window, cx, move |tree, _| {
                    tree.set_group_kind(id, Some(next))
                });
            }
            EditSlot::LookKind(id) => {
                let negate = self
                    .tree
                    .node(id)
                    .and_then(|n| match &n.kind {
                        EKind::LookAround { negate, .. } => Some(*negate),
                        _ => None,
                    })
                    .unwrap_or(false);
                let kind = if value == "lookbehind" {
                    LookKind::Lookbehind
                } else {
                    LookKind::Lookahead
                };
                self.edit(window, cx, move |tree, _| {
                    tree.set_lookaround(id, Some((kind, negate)))
                });
            }
            _ => {}
        }
    }

    /// 切换中/英。节点标签文字宽度会变，所以图例和轨道图都必须重建。
    fn toggle_lang(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.lang = self.lang.toggled();
        self.legend = build_legend(window, &self.mono_family, self.lang);
        self.evaluate(window, cx);
        cx.notify();
    }

    fn re_match(&mut self) {
        self.matches.clear();
        self.error.clear();
        if self.pattern.is_empty() {
            return;
        }
        let regex = match regex::RegexBuilder::new(&self.pattern)
            .case_insensitive(self.flags[1])
            .multi_line(self.flags[2])
            .dot_matches_new_line(self.flags[3])
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
                        None => if self.lang == Lang::Cn {
                            format!("#{group_index}: <未匹配>")
                        } else {
                            format!("#{group_index}: <no match>")
                        },
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
        self.re_match();
        cx.notify();
    }

    /// 对齐 regex-vis：输入框右侧拼接显示已勾选的 flags（如 "gi"）
    fn flags_string(&self) -> String {
        FLAGS
            .iter()
            .filter(|(index, _, _, _)| self.flags[*index])
            .map(|(_, ch, _, _)| ch.to_string())
            .collect()
    }

    fn apply_sample(&mut self, pattern: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.pattern = pattern.to_string();
        self.pattern_state.update(cx, |state, cx| {
            state.set_value(pattern.to_string(), window, cx);
        });
        self.evaluate(window, cx);
        self.panel_tab = TAB_TEST;
        self.panel_collapsed = false;
        cx.notify();
    }

    /// 切换右侧页签。切到「编辑」时要先把控件池准备好（`render` 里不该建实体）。
    fn set_panel_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.panel_tab = index;
        if index == TAB_EDIT {
            self.sync_edit_widgets(window, cx);
        }
        cx.notify();
    }

    // -----------------------------------------------------------------------
    // 编辑面板（官方 `pB`）
    // -----------------------------------------------------------------------

    /// 面板根。段落顺序与官方一致。
    ///
    /// 官方 `pB` 只渲染「插入/分组/断言按钮组 + 表达式 + 内容 + 分组 + 量词 + 断言详情」，
    /// 每段都有前置条件；没选中任何节点时整块基本是空的。
    fn edit_panel(&self, cx: &Context<Self>) -> AnyElement {
        let lang = self.lang;
        let mut root = div().flex().flex_col().gap_6();

        if self.selected.is_empty() {
            // 官方此处只剩一个空表达式段，等于白板；补一句引导
            return root
                .child(design::hint(
                    lang.of(
                        "在轨道图里点击一个节点，即可编辑它的内容、分组与量词",
                        "Click a node in the graph to edit its content, group and quantifier",
                    ),
                    cx,
                ))
                .into_any_element();
        }

        let info = self.selection();
        let ids = info.ids.clone();
        let id = info.id;

        root = root.child(self.insert_section(&info, &ids, cx));
        root = root.child(panel_section(
            lang.of("表达式", "Expression"),
            None,
            expression_body(&info.pattern, info.start, info.end, &self.mono_family),
            cx,
        ));

        if info.content.is_some() {
            root = root.child(self.content_section(&info, cx));
        }
        if let Some((kind, _name)) = info.group.clone() {
            root = root.child(self.group_section(id, kind, cx));
        }
        if info.has_quantifier {
            root = root.child(self.quantifier_section(&info, cx));
        }
        if let Some((kind, negate)) = info.lookaround {
            root = root.child(self.lookaround_section(id, kind, negate, cx));
        }

        root.into_any_element()
    }

    /// 官方 `sB`：三组「插入 / 分组 / 断言」按钮
    fn insert_section(&self, info: &SelectionInfo, ids: &[NodeId], cx: &Context<Self>) -> Div {
        let lang = self.lang;
        let mut block = div().flex().flex_col().gap_6();

        // —— 1) 插入节点。首节点是 `^` 时不能再往前插，尾节点是 `$` 时不能再往后插 ——
        let ids_owned = ids.to_vec();
        let first_is_begin = self.tree.node(ids[0]).is_some_and(|n| {
            matches!(
                &n.kind,
                EKind::Boundary {
                    kind: BoundaryKind::Beginning,
                    ..
                }
            )
        });
        let last_is_end = self
            .tree
            .node(*ids.last().unwrap_or(&ids[0]))
            .is_some_and(|n| {
                matches!(
                    &n.kind,
                    EKind::Boundary {
                        kind: BoundaryKind::End,
                        ..
                    }
                )
            });

        let mut insert_buttons: Vec<AnyElement> = Vec::new();
        if !first_is_begin {
            insert_buttons.push(insert_button(
                "regex-insert-prev",
                lang.of("向前插入", "Before"),
                InsertMode::Before,
                &ids_owned,
                cx,
            ));
        }
        insert_buttons.push(insert_button(
            "regex-insert-parallel",
            lang.of("插入或", "Parallel"),
            InsertMode::Parallel,
            &ids_owned,
            cx,
        ));
        if !last_is_end {
            insert_buttons.push(insert_button(
                "regex-insert-next",
                lang.of("向后插入", "After"),
                InsertMode::After,
                &ids_owned,
                cx,
            ));
        }
        block = block.child(panel_section(
            lang.of("插入节点", "Insert around"),
            None,
            button_row(insert_buttons).into_any_element(),
            cx,
        ));

        // —— 2) 分组。选中的本身就是一个分组时官方不给这组按钮 ——
        if !(info.single && info.group.is_some()) {
            let mut buttons: Vec<AnyElement> = Vec::new();
            for (index, (kind, cn, en)) in [
                (EGroupKind::Capturing, "捕获组", "Capturing"),
                (EGroupKind::NonCapturing, "非捕获组", "Non-cap"),
                (EGroupKind::NamedCapturing, "具名捕获组", "Named cap"),
            ]
            .into_iter()
            .enumerate()
            {
                buttons.push(
                    Button::new(("regex-wrap-group", index))
                        .label(lang.of(cn, en))
                        .outline()
                        .compact()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let ids = this.selected.clone();
                            this.edit(window, cx, |tree, id_gen| {
                                tree.wrap(&ids, WrapKind::Group(kind), id_gen)
                            });
                        }))
                        .into_any_element(),
                );
            }
            block = block.child(panel_section(
                lang.of("分组", "Group selection"),
                None,
                button_row(buttons).into_any_element(),
                cx,
            ));
        }

        // —— 3) 向前/向后断言。官方把它收在「显示更多」药丸后面（默认收起） ——
        if !(info.single && info.lookaround.is_some()) {
            if self.show_lookaround {
                let mut buttons: Vec<AnyElement> = Vec::new();
                for (index, (kind, cn, en)) in [
                    (LookKind::Lookahead, "向前断言", "Lookahead"),
                    (LookKind::Lookbehind, "向后断言", "Lookbehind"),
                ]
                .into_iter()
                .enumerate()
                {
                    buttons.push(
                        Button::new(("regex-wrap-look", index))
                            .label(lang.of(cn, en))
                            .outline()
                            .compact()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let ids = this.selected.clone();
                                this.edit(window, cx, |tree, id_gen| {
                                    tree.wrap(&ids, WrapKind::LookAround(kind), id_gen)
                                });
                            }))
                            .into_any_element(),
                    );
                }
                block = block.child(panel_section(
                    lang.of("向前/向后断言", "Lookaround assertion"),
                    None,
                    button_row(buttons).into_any_element(),
                    cx,
                ));
            }
            block = block.child(
                div().flex().justify_center().child(
                    div()
                        .id("regex-lookaround-toggle")
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py(px(2.0))
                        .rounded_full()
                        .border_1()
                        .border_color(cx.theme().border)
                        .text_size(px(12.0))
                        .text_color(cx.theme().muted_foreground)
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().muted))
                        .child(lang.of("显示更多", "show more"))
                        .child(
                            Icon::new(IconName::ChevronDown)
                                .size(px(12.0))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_lookaround = true;
                            cx.notify();
                        })),
                ),
            );
        }

        block
    }

    /// 官方 `Gz`：内容段。类型下拉 + 对应子编辑器
    fn content_section(&self, info: &SelectionInfo, cx: &Context<Self>) -> Div {
        let lang = self.lang;
        let id = info.id;
        let Some(content) = info.content.clone() else {
            return div();
        };

        let mut body = div().flex().flex_col().gap_6();

        let mut type_row = div().flex().items_center().gap_2();
        if let Some((state, _)) = self.edit_selects.get(&EditSlot::ContentType(id)) {
            type_row = type_row.child(div().w(px(208.0)).child(Select::new(state)));
        }
        body = body.child(panel_item(lang.of("类型", "Type"), type_row));

        match &content {
            ContentSpec::String { .. } => {
                let mut column = div().flex().flex_col().gap_2();
                // 官方这一段带的提示条：输入会被自动转义
                column = column.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .p_2()
                        .rounded(px(6.0))
                        .bg(cx.theme().muted)
                        .child(
                            Icon::new(IconName::Info)
                                .size(px(16.0))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(lang.of(
                                    "输入将会被自动转义",
                                    "The input will be escaped automatically.",
                                )),
                        ),
                );
                if let Some((state, _)) = self.edit_inputs.get(&EditSlot::Value(id)) {
                    column = column.child(Input::new(state));
                }
                body = body.child(panel_item(" 值", column));
            }
            ContentSpec::Class { value } => {
                let mut row = div().flex().items_center().gap_2();
                if let Some((state, _)) = self.edit_selects.get(&EditSlot::ClassKind(id)) {
                    row = row.child(div().w(px(208.0)).child(Select::new(state)));
                }
                // 只有 `\xhh` / `\uhhhh` 才额外给一个手填框（官方 `r !== e`）
                if class_kind_key(value) != *value {
                    if let Some((state, _)) = self.edit_inputs.get(&EditSlot::Value(id)) {
                        row = row.child(div().w(px(208.0)).child(Input::new(state)));
                    }
                }
                body = body.child(panel_item(lang.of("类", "Class"), row));
            }
            ContentSpec::Ranges { ranges, negate } => {
                body = body.child(panel_item(
                    lang.of("范围", "Ranges"),
                    self.ranges_body(id, ranges, *negate, cx),
                ));
            }
            ContentSpec::BackReference { .. } => {
                let mut row = div().flex().items_center().gap_2();
                if let Some((state, _)) = self.edit_selects.get(&EditSlot::Backref(id)) {
                    row = row.child(div().w(px(208.0)).child(Select::new(state)));
                }
                body = body.child(panel_item(lang.of("反向引用", "Back Reference"), row));
            }
            ContentSpec::WordBoundary { negate } => {
                body = body.child(panel_item(
                    lang.of("否定", "Negate"),
                    negate_row(
                        "regex-negate-word",
                        *negate,
                        cx,
                        move |view, value, window, cx| {
                            view.set_negate(id, value, window, cx);
                        },
                    ),
                ));
            }
            _ => {}
        }

        panel_section(
            lang.of("内容", "Content"),
            None,
            body.into_any_element(),
            cx,
        )
    }

    /// 官方 `Zz`：字符范围列表 + 「空范围」+ 常用预设 + 否定
    fn ranges_body(
        &self,
        id: NodeId,
        ranges: &[(String, String)],
        negate: bool,
        cx: &Context<Self>,
    ) -> Div {
        let lang = self.lang;
        let mut body = div().flex().flex_col().gap_4();

        let mut rows = div().flex().flex_col().gap(px(10.0));
        for index in 0..ranges.len() {
            let mut row = div().flex().items_center().gap_2();
            if let Some((state, _)) = self.edit_inputs.get(&EditSlot::RangeFrom(id, index)) {
                row = row.child(div().flex_1().child(Input::new(state)));
            }
            row = row.child(div().text_sm().child("-"));
            if let Some((state, _)) = self.edit_inputs.get(&EditSlot::RangeTo(id, index)) {
                row = row.child(div().flex_1().child(Input::new(state)));
            }
            row = row.child(
                Button::new(("regex-range-remove", index))
                    .icon(Icon::new(IconName::Close))
                    .ghost()
                    .compact()
                    .tooltip(lang.of("删除该范围", "Remove this range"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.remove_range(id, index, window, cx);
                    })),
            );
            rows = rows.child(row);
        }
        body = body.child(rows);

        let mut actions = div().flex().flex_wrap().items_center().gap_2();
        actions = actions.child(
            Button::new("regex-range-add")
                .icon(Icon::new(IconName::Plus))
                .label("An Empty Range")
                .outline()
                .compact()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.add_range(id, "", "", window, cx);
                })),
        );
        // 官方那三个预设（`Uz`）跟在按钮后面，这里收成三个小按钮
        for (index, (from, to)) in [("0", "9"), ("a", "z"), ("A", "Z")].into_iter().enumerate() {
            actions = actions.child(
                Button::new(("regex-range-preset", index))
                    .label(format!("{from} - {to}"))
                    .outline()
                    .compact()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.add_range(id, from, to, window, cx);
                    })),
            );
        }
        body = body.child(actions);

        body.child(negate_row(
            "regex-negate-ranges",
            negate,
            cx,
            move |view, value, window, cx| {
                view.set_negate(id, value, window, cx);
            },
        ))
    }

    /// 官方 `Yz`：分组详情（类型下拉 + 具名分组的名字 + 取消组）
    fn group_section(&self, id: NodeId, kind: EGroupKind, cx: &Context<Self>) -> Div {
        let lang = self.lang;
        let mut body = div().flex().flex_col().gap_2();

        let mut row = div().flex().items_center().gap_2();
        if let Some((state, _)) = self.edit_selects.get(&EditSlot::GroupKind(id)) {
            row = row.child(div().w(px(192.0)).child(Select::new(state)));
        }
        body = body.child(row);

        if kind == EGroupKind::NamedCapturing {
            let mut name_row = div().flex().items_center();
            name_row = name_row.child(
                div()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .px_2()
                    .text_size(px(12.0))
                    .text_color(cx.theme().muted_foreground)
                    .bg(cx.theme().muted)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_l(px(6.0))
                    .child(lang.of("组名", "Group's name")),
            );
            if let Some((state, _)) = self.edit_inputs.get(&EditSlot::GroupName(id)) {
                name_row = name_row.child(div().flex_1().min_w_0().child(Input::new(state)));
            }
            body = body.child(name_row);
        }

        panel_section(
            lang.of("组", "Group"),
            Some(
                Button::new("regex-ungroup")
                    .icon(Icon::new(IconName::Close))
                    .ghost()
                    .compact()
                    .tooltip(lang.of("取消组", "UnGroup"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.edit(window, cx, |tree, _| tree.set_group_kind(id, None));
                    }))
                    .into_any_element(),
            ),
            body.into_any_element(),
            cx,
        )
    }

    /// 官方 `rB`：量词段
    fn quantifier_section(&self, info: &SelectionInfo, cx: &Context<Self>) -> Div {
        let lang = self.lang;
        let id = info.id;
        let mut body = div().flex().flex_col().gap(px(10.0));

        let mut row = div().flex().items_center().gap_2();
        if let Some((state, _)) = self.edit_selects.get(&EditSlot::QuantKind(id)) {
            row = row.child(div().w(px(208.0)).child(Select::new(state)));
        }
        body = body.child(row);

        // 自定义量词才给 min/max 两个输入框（官方占位是 0 与 Infinity）
        if info.quantifier.is_some_and(|q| q.kind == QuantKind::Custom) {
            let mut custom = div().flex().items_center().gap_2();
            if let Some((state, _)) = self.edit_inputs.get(&EditSlot::QuantMin(id)) {
                custom = custom.child(div().flex_1().child(Input::new(state)));
            }
            custom = custom.child(div().text_sm().child("-"));
            if let Some((state, _)) = self.edit_inputs.get(&EditSlot::QuantMax(id)) {
                custom = custom.child(div().flex_1().child(Input::new(state)));
            }
            body = body.child(custom);
        }

        if let Some(quantifier) = info.quantifier {
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(13.0))
                    .text_color(cx.theme().muted_foreground)
                    .child("Greedy")
                    .child(
                        Checkbox::new("regex-greedy")
                            .checked(quantifier.greedy)
                            .on_click(cx.listener(move |this, value: &bool, window, cx| {
                                this.set_greedy(id, *value, window, cx);
                            })),
                    ),
            );
        }

        panel_section(
            lang.of("量词", "Quantifier"),
            None,
            panel_section(lang.of("次数", "times"), None, body.into_any_element(), cx)
                .into_any_element(),
            cx,
        )
    }

    /// 官方 `oB`：断言详情（类型下拉 + 否定 + 取消断言）
    fn lookaround_section(
        &self,
        id: NodeId,
        kind: LookKind,
        negate: bool,
        cx: &Context<Self>,
    ) -> Div {
        let lang = self.lang;
        let mut body = div().flex().flex_col().gap_4();

        let mut row = div().flex().items_center().gap_2();
        if let Some((state, _)) = self.edit_selects.get(&EditSlot::LookKind(id)) {
            row = row.child(div().w(px(208.0)).child(Select::new(state)));
        }
        body = body.child(row);
        body = body.child(negate_row(
            "regex-negate-look",
            negate,
            cx,
            move |view, value, window, cx| {
                view.set_lookaround_negate(id, value, window, cx);
            },
        ));
        let _ = kind;

        panel_section(
            lang.of("向前/向后断言", "Lookaround assertion"),
            Some(
                Button::new("regex-cancel-look")
                    .icon(Icon::new(IconName::Close))
                    .ghost()
                    .compact()
                    .tooltip(lang.of("取消断言", "Cancel assertion"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.edit(window, cx, |tree, _| tree.set_lookaround(id, None));
                    }))
                    .into_any_element(),
            ),
            body.into_any_element(),
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // 面板动作
    // -----------------------------------------------------------------------

    fn add_range(
        &mut self,
        id: NodeId,
        from: &str,
        to: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ContentSpec::Ranges { mut ranges, negate }) = self.content_of(id) else {
            return;
        };
        ranges.push((from.to_string(), to.to_string()));
        let spec = ContentSpec::Ranges { ranges, negate };
        self.edit(window, cx, move |tree, id_gen| {
            vec![tree.set_content(id, &spec, id_gen)]
        });
    }

    fn remove_range(
        &mut self,
        id: NodeId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ContentSpec::Ranges { mut ranges, negate }) = self.content_of(id) else {
            return;
        };
        if index >= ranges.len() {
            return;
        }
        ranges.remove(index);
        let spec = ContentSpec::Ranges { ranges, negate };
        self.edit(window, cx, move |tree, id_gen| {
            vec![tree.set_content(id, &spec, id_gen)]
        });
    }

    /// 字符范围 / 单词边界的「否定」开关
    fn set_negate(&mut self, id: NodeId, negate: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(content) = self.content_of(id) else {
            return;
        };
        let spec = match content {
            ContentSpec::Ranges { ranges, .. } => ContentSpec::Ranges { ranges, negate },
            ContentSpec::WordBoundary { .. } => ContentSpec::WordBoundary { negate },
            _ => return,
        };
        self.edit(window, cx, move |tree, id_gen| {
            vec![tree.set_content(id, &spec, id_gen)]
        });
    }

    fn set_lookaround_negate(
        &mut self,
        id: NodeId,
        negate: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(kind) = self.tree.node(id).and_then(|n| match &n.kind {
            EKind::LookAround { kind, .. } => Some(*kind),
            _ => None,
        }) else {
            return;
        };
        self.edit(window, cx, move |tree, _| {
            tree.set_lookaround(id, Some((kind, negate)))
        });
    }

    fn set_greedy(&mut self, id: NodeId, greedy: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(mut quantifier) = self.quantifier_of(id) else {
            return;
        };
        if quantifier.greedy == greedy {
            return;
        }
        quantifier.greedy = greedy;
        self.edit(window, cx, move |tree, id_gen| {
            vec![tree.set_quantifier(id, Some(quantifier), id_gen)]
        });
    }

    fn paste_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.test_text = text.to_string();
                self.text_state.update(cx, |state, cx| {
                    state.set_value(self.test_text.clone(), window, cx);
                });
                self.re_match();
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
        self.tree = ERoot::default();
        self.tree_source.clear();
        self.selected.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.edit_inputs.clear();
        self.edit_selects.clear();
        self.marquee = None;
        self.pattern_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.text_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
    }
}

/// 图例文案表：外层 8 组，内层是各组条目。
/// 逐项对齐官方 `yB` 的分组顺序与 `/locales/cn` 的译法。
/// **顺序必须与 `build_legend` 构建示意图的顺序一致**（两者按位置 zip）。
const LEGEND_TEXT: [(
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
); 8] = [
    (
        "字符",
        "Characters",
        &[("直接匹配字符串", "Direct match characters")],
    ),
    (
        "字符类",
        "Character classes",
        &[(
            "区分不同类型的字符",
            "Distinguish different types of characters",
        )],
    ),
    (
        "范围",
        "Ranges",
        &[
            (
                "匹配任何一个包含的字符",
                "Matches any one of the enclosed characters",
            ),
            (
                "匹配任何没有包含在括号中的字符",
                "Matches anything that is not enclosed in the brackets",
            ),
        ],
    ),
    (
        "或",
        "Choice",
        &[("匹配 “x” 或者 “y”", "Matches either \"x\" or \"y\"")],
    ),
    (
        "量词",
        "Quantifier",
        &[(
            "表示要匹配的字符或表达式的数量",
            "Indicate numbers of characters or expressions to match",
        )],
    ),
    (
        "组",
        "Group",
        &[
            ("匹配x并记住匹配项", "Matches x and remembers the match"),
            (
                "匹配 “x”，但不记得匹配",
                "Matches \"x\" but does not remember the match",
            ),
            (
                "匹配 “x” 并将其存储在返回的匹配项的groups属性中，该属性位于 <Name> 指定的名称下",
                "Matches \"x\" and stores it on the groups property of the returned matches under the name specified by <Name>",
            ),
        ],
    ),
    (
        "反向引用",
        "Back reference",
        &[
            (
                "匹配组 #1 的反向引用",
                "A back reference to match group #1",
            ),
            (
                "匹配组 #Name 的反向引用",
                "A back reference to match group #Name",
            ),
        ],
    ),
    (
        "断言",
        "Assertion",
        &[
            ("匹配输入的开头", "Matches the beginning of input"),
            (
                "x 被 y 跟随时匹配 x",
                "Matches \"x\" only if \"x\" is followed by \"y\"",
            ),
        ],
    ),
];

/// 构建右面板「图例」页签。逐项对齐官方 `yB`：8 组 / 13 项。
///
/// 官方的每一项就是这个 `yt` 组件喂一个正则串渲染出来的**完整小轨道图**，
/// 这里同样直接构造节点（不走解析），保证与主图渲染完全同源。
/// 文案取自 `LEGEND_TEXT`，示意图按同样顺序现构建后与之 zip。
fn build_legend(window: &mut Window, family: &SharedString, lang: Lang) -> Vec<LegendGroup> {
    let t = |text: &str, window: &mut Window| {
        token_node(text.to_string(), None, false, window, family)
    };
    let range = |label: &'static str, negated: bool, window: &mut Window| {
        token_node(
            "\"a\" - \"z\"".to_string(),
            Some(label.to_string()),
            negated,
            window,
            family,
        )
    };
    let class_text = class_label(r"\d", lang).unwrap_or(r"\d").to_string();
    let backref = lang.of("反向引用", "Back Reference");

    let diagrams: Vec<Vec<Diagram>> = vec![
        vec![layout_content(&t("\"abc\"", window))],
        vec![layout_content(&t(&class_text, window))],
        vec![
            layout_content(&range(one_of_label(lang), false, window)),
            layout_content(&range(none_of_label(lang), true, window)),
        ],
        vec![layout_content(&build_alternate(vec![
            t("\"x\"", window),
            t("\"y\"", window),
        ]))],
        vec![layout_content(&repeat_node(
            t("\"a\"", window),
            0,
            None,
            window,
            family,
        ))],
        vec![
            layout_content(&group_node(
                Some(format!("{} #1", group_word(lang))),
                t("\"x\"", window),
                window,
                family,
            )),
            layout_content(&group_node(None, t("\"x\"", window), window, family)),
            layout_content(&group_node(
                Some(format!("{} #Name", group_word(lang))),
                t("\"x\"", window),
                window,
                family,
            )),
        ],
        vec![
            layout_content(&t(&format!("{backref} #1"), window)),
            layout_content(&t(&format!("{backref} #Name"), window)),
        ],
        vec![
            layout_content(&t(beginning_label(lang), window)),
            layout_content(&build_concat(vec![
                t("\"x\"", window),
                group_node(
                    Some(lookaround_label(lang, true, false).to_string()),
                    t("\"y\"", window),
                    window,
                    family,
                ),
            ])),
        ],
    ];

    LEGEND_TEXT
        .iter()
        .zip(diagrams)
        .map(|((cn_title, en_title, entries), group_diagrams)| LegendGroup {
            title: lang.of(cn_title, en_title),
            entries: entries
                .iter()
                .zip(group_diagrams)
                .map(|((cn_desc, en_desc), diagram)| LegendEntry {
                    desc: lang.of(cn_desc, en_desc),
                    diagram,
                })
                .collect(),
        })
        .collect()
}

// ===========================================================================
// 渲染
// ===========================================================================

/// `regex::Error` 的 Display 是**多行**的（原文 / `^` 指示符 / `error: 描述`）。
/// 顶部浮层只放一行，这里把换行折成空格并压掉连续空白。
fn error_one_line(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for word in raw.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

impl Render for RegexVisualizer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let matches = self.matches.clone();
        let match_count = self.matches.len();
        let error = self.error.clone();
        let has_error = !error.is_empty();
        let error_text = error_one_line(&error);
        let pattern_empty = self.pattern.is_empty();
        let flags_str = self.flags_string();
        let panel_tab = self.panel_tab;
        let lang = self.lang;
        let flags_state = self.flags;
        let mono_family = self.mono_family.clone();
        let show_hint = self.diagram.is_some() && error.is_empty();
        let can_undo = !self.undo_stack.is_empty();
        let can_redo = !self.redo_stack.is_empty();

        // —— 图区：官方 flex-1 relative flex flex-col min-w-0 bg-graph-bg ——
        let diagram_area = div()
            .flex_1()
            .min_h_0()
            .relative()
            .overflow_scrollbar()
            .bg(Hsla::from(rgb(GRAPH_BG)))
            .child(
                // 官方：flex items-center justify-center p-8 h-full
                div()
                    .min_h_full()
                    .min_w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_8()
                    .children(match self.diagram.as_ref() {
                        Some(diagram) => {
                            vec![interactive_diagram_canvas(diagram, self.marquee, cx)
                                .into_any_element()]
                        }
                        // 有校验错误时不再显示中性的占位提示，顶部浮层已经说明原因
                        None if has_error => Vec::new(),
                        None => {
                            vec![design::hint(
                                lang.of("输入有效正则后显示轨道图", "Enter a valid regex to show the graph"),
                                cx,
                            )
                            .into_any_element()]
                        }
                    }),
            )
            .when(show_hint, |area| {
                area.child(
                    // 官方：absolute bg-graph-bg bottom-0 left-1/2 -translate-x-1/2 z-10 text-sm
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .bg(Hsla::from(rgb(GRAPH_BG)))
                                .py_1()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(lang.of(
                                    "↑ 可以通过点击或拖拽选中节点",
                                    "↑ You can select nodes by dragging or clicking on the graph",
                                )),
                        ),
                )
            })
            .when(has_error, |area| {
                area.child(
                    // 校验错误钉在图区顶部：绝对定位不参与流式布局，
                    // 输入框区（输入框 + Flags 行）不会被顶下去，逐字输入时不再抖动。
                    div()
                        .absolute()
                        .top_3()
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .px_4()
                        .child(
                            div()
                                .max_w(px(560.0))
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_2()
                                .rounded(px(8.0))
                                .border_1()
                                .border_color(Hsla::from(rgb(design::ERROR_RED)))
                                .bg(Hsla::from(rgb(GRAPH_BG)))
                                .child(
                                    Icon::new(IconName::TriangleAlert)
                                        .size(px(14.0))
                                        .flex_shrink_0()
                                        .text_color(Hsla::from(rgb(design::ERROR_RED))),
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .text_size(px(12.5))
                                        .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                        .child(error_text.clone()),
                                ),
                        ),
                )
            })
            .child(
                // 官方：侧栏折叠开关，绝对定位在右上角；这里顺带放撤销/重做
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Button::new("regex-undo")
                            .icon(Icon::new(IconName::Undo2))
                            .ghost()
                            .compact()
                            .disabled(!can_undo)
                            .tooltip(lang.of("撤销", "Undo"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.undo(window, cx);
                            })),
                    )
                    .child(
                        Button::new("regex-redo")
                            .icon(Icon::new(IconName::Redo2))
                            .ghost()
                            .compact()
                            .disabled(!can_redo)
                            .tooltip(lang.of("重做", "Redo"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.redo(window, cx);
                            })),
                    )
                    .child(
                        Button::new("regex-panel-toggle")
                            .icon(Icon::new(if self.panel_collapsed {
                                IconName::PanelLeftOpen
                            } else {
                                IconName::PanelLeftClose
                            }))
                            .ghost()
                            .compact()
                            .tooltip(lang.of("显示/隐藏右侧面板", "Show / hide the side panel"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.panel_collapsed = !this.panel_collapsed;
                                cx.notify();
                            })),
                    ),
            );

        // —— 输入区：官方 xA，border-t + px-4 py-8 + 内容 max-w-4xl + 纵向 space-y-4 ——
        let mut input_block = div()
            .max_w(px(896.0))
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.pattern_state).h(px(36.0))),
                    )
                    .when(!pattern_empty && !flags_str.is_empty(), |row| {
                        row.child(
                            // 官方：h-9 px-2 border border-l-0 rounded-r-md text-sm
                            div()
                                .h(px(36.0))
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .px_2()
                                .border_t_1()
                                .border_r_1()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .rounded_tr(px(6.0))
                                .rounded_br(px(6.0))
                                .font_family(mono_family.clone())
                                .text_sm()
                                .child(flags_str.clone()),
                        )
                    })
                    .child(
                        div()
                            .ml_4()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                Button::new("regex-copy")
                                    .icon(Icon::new(IconName::Copy))
                                    .ghost()
                                    .compact()
                                    .tooltip(lang.of("复制正则", "Copy regex"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.copy_pattern(window, cx);
                                    })),
                            )
                            .child(
                                // 官方把语言选择器放在顶栏；这里收成输入行右侧一个紧凑按钮
                                Button::new("regex-lang")
                                    .label(lang.badge())
                                    .ghost()
                                    .compact()
                                    .tooltip(lang.tooltip())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_lang(window, cx);
                                    })),
                            ),
                    ),
            );

        if !pattern_empty {
            // Flags 行：官方 flex items-center space-x-3，"Flags: " + 4 组勾选框
            let mut flag_row = div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().mr_2().text_sm().child(lang.of("标志: ", "Flags: ")));
            for &(flag_index, _, cn, en) in FLAGS.iter() {
                let label = lang.of(cn, en);
                let active = flags_state[flag_index];
                flag_row = flag_row.child(
                    div()
                        .id(("regex-flag", flag_index))
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_flag(flag_index, cx);
                        }))
                        .child(
                            div()
                                .size(px(16.0))
                                .flex_shrink_0()
                                .rounded(px(4.0))
                                .border_1()
                                .border_color(if active {
                                    Hsla::from(rgb(GRAPH))
                                } else {
                                    cx.theme().border
                                })
                                .bg(if active {
                                    Hsla::from(rgb(GRAPH))
                                } else {
                                    cx.theme().background
                                })
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(active, |box_el| {
                                    box_el.child(
                                        Icon::new(IconName::Check)
                                            .size(px(11.0))
                                            .text_color(cx.theme().background),
                                    )
                                }),
                        )
                        .child(div().text_sm().child(label)),
                );
            }
            input_block = input_block.child(flag_row);
        }

        let left_column = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .child(diagram_area)
            .child(
                div()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .px_4()
                    .py_8()
                    .flex()
                    .justify_center()
                    .child(input_block),
            );

        // —— 右面板：官方 zB，305px + py-4 + border-l，页签 mx-4 mb-6，内容 p-4 pt-0 ——
        let mut tab_bar = TabBar::new("regex-panel-tabs")
            .segmented()
            .selected_index(panel_tab)
            .on_click(cx.listener(move |this, index: &usize, window, cx| {
                this.set_panel_tab(*index, window, cx);
            }));
        for (cn, en) in PANEL_TABS.iter() {
            tab_bar = tab_bar.child(Tab::new().label(self.lang.of(cn, en)));
        }

        let panel_body: AnyElement = match panel_tab {
            TAB_LEGEND => {
                // 官方 xB：外层 divide-y，每组「h5 标题 + space-y-6 的条目列表」
                let mut list = div().flex().flex_col();
                for (gi, group) in self.legend.iter().enumerate() {
                    list = list
                        .when(gi > 0, |l| {
                            l.child(div().my_4().h(px(1.0)).w_full().bg(cx.theme().border))
                        })
                        .child(
                            div()
                                .text_size(px(15.0))
                                .font_semibold()
                                .mb_2()
                                .child(group.title),
                        );
                    for entry in group.entries.iter() {
                        list = list
                            .child(div().mb_3().child(diagram_canvas(&entry.diagram, cx)))
                            .child(
                                div()
                                    .mb_2()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(entry.desc),
                            );
                    }
                }
                list.into_any_element()
            }
            TAB_EDIT => self.edit_panel(cx),
            TAB_TEST => {
                let (status_icon, status_color, status_text) = if !error.is_empty() {
                    (
                        IconName::CircleX,
                        Hsla::from(rgb(design::ERROR_RED)),
                        error_text.clone(),
                    )
                } else if pattern_empty {
                    (
                        IconName::CircleCheck,
                        cx.theme().muted_foreground,
                        lang.of("输入正则后测试", "Enter a regex to test").to_string(),
                    )
                } else if match_count > 0 {
                    (
                        IconName::CircleCheck,
                        Hsla::from(rgb(design::OK_GREEN)),
                        match lang {
                            Lang::Cn => format!("{match_count} 个匹配"),
                            Lang::En => format!("{match_count} matches"),
                        },
                    )
                } else {
                    (
                        IconName::CircleX,
                        cx.theme().muted_foreground,
                        lang.of("无匹配", "No match").to_string(),
                    )
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(design::editor_label(
                                lang.of("测试文本", "Test text"),
                                cx,
                            ))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Button::new("regex-paste")
                                            .icon(Icon::new(IconName::File))
                                            .ghost()
                                            .compact()
                                            .tooltip(lang.of("粘贴测试文本", "Paste test text"))
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
                                            .tooltip(lang.of("清空全部", "Clear all"))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.clear(window, cx);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(Textarea::new(&self.text_state).h(px(96.0)))
                    .child(
                        // 官方 bB：textarea 下方 bg-muted 圆角状态条
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pl_3()
                            .pr_2()
                            .py_1p5()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded(px(8.0))
                            .bg(cx.theme().muted)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .min_w_0()
                                    .child(
                                        Icon::new(status_icon)
                                            .size(px(14.0))
                                            .flex_shrink_0()
                                            .text_color(status_color),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(status_text),
                                    ),
                            )
                            .child(design::pill(format!("{match_count}"), cx)),
                    )
                    .child(highlight_preview_panel(&self.test_text, &matches, cx, lang))
                    .child(match_panel(matches, cx, lang))
                    .into_any_element()
            }
            _ => {
                let mut list = div().flex().flex_col().gap_1p5();
                for (i, (cn, en, sample)) in SAMPLES.iter().enumerate() {
                    let name = lang.of(cn, en);
                    let sample: &'static str = sample;
                    list = list.child(
                        div()
                            .id(("regex-sample", i))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .p_3()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.apply_sample(sample, window, cx);
                            }))
                            .child(div().text_sm().font_medium().child(name.to_string()))
                            .child(
                                div()
                                    .mt_1()
                                    .font_family(mono_family.clone())
                                    .text_size(px(12.0))
                                    .text_color(Hsla::from(rgb(0x2dd4bf)))
                                    .child(sample.to_string()),
                            ),
                    );
                }
                list.into_any_element()
            }
        };

        let right_panel = div()
            .w(px(305.0))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .py_4()
            .border_l_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .child(div().mx_4().mb_6().child(tab_bar))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(div().px_4().pb_4().child(panel_body)),
            );

        // —— 根布局：顶栏 + 左图/输入 + 右面板 ——
        div()
            .h_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .rounded(px(8.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            // 官方把快捷键挂在 window 上，且聚焦在 INPUT/TEXTAREA 时直接跳过 ——
            // gpui 里输入框会消费掉按键，效果一致。
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                let modifiers = &ev.keystroke.modifiers;
                match ev.keystroke.key.as_str() {
                    "z" if modifiers.control || modifiers.platform => {
                        if modifiers.shift {
                            this.redo(window, cx);
                        } else {
                            this.undo(window, cx);
                        }
                        cx.stop_propagation();
                    }
                    "backspace" | "delete" if !this.selected.is_empty() => {
                        this.delete_selected(window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if !this.selected.is_empty() => {
                        this.clear_selection(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .child(header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(left_column)
                    .when(!self.panel_collapsed, |row| row.child(right_panel)),
            )
    }
}

fn header(cx: &App) -> Div {
    div()
        .h(px(64.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(cx.theme().border)
        .px_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .size(px(32.0))
                        .rounded(px(8.0))
                        .bg(cx.theme().primary)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .font_family("monospace")
                                .text_size(px(13.0))
                                .font_semibold()
                                .text_color(cx.theme().background)
                                .child(".*"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_base().font_semibold().child("正则可视化"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Regex Visualization"),
                        ),
                ),
        )
        .child(
            Button::new("regex-home")
                .icon(Icon::new(IconName::ExternalLink))
                .ghost()
                .compact()
                .tooltip("打开 regex-vis.com")
                .on_click(|_, _, cx| cx.open_url("https://regex-vis.com")),
        )
}


fn match_panel(matches: Vec<RegexMatch>, cx: &mut Context<RegexVisualizer>, lang: Lang) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(design::editor_label(
            lang.of("匹配结果", "Matches"),
            cx,
        ))
        .child(
            div()
                .max_h(px(220.0))
                .overflow_y_scrollbar()
                .children(if matches.is_empty() {
                    vec![design::hint(lang.of("暂无匹配", "No matches yet"), cx)]
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
                                        .child(format!(
                                            "#{} [{}..{}]",
                                            item.index, item.start, item.end
                                        )),
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

/// 官方等宽字体的字符步进恒为 0.6em，测试用它替代窗口文本系统。
///
/// 注意：这里必须显式列举导入项。`gpui_kit::*` 在 test-support 特性下会带入
/// `gpui_kit::test`，用 `use super::*` 会遮蔽内置的 `#[test]` 属性宏。
#[cfg(test)]
mod align_tests {
    use super::{
        build_alternate, build_concat, layout_diagram, merge_cmds, quote_pad, split_cmds, Cmd,
        Diagram, GKind, GNode, Prim, GROUP_PAD_H, GROUP_PAD_V, ICON_W, LABEL_BOX, LABEL_FS,
        NODE_FS, NODE_H, NODE_PAD_X,
    };

    const ADVANCE: f32 = 0.6;

    fn tok(text: &str) -> GNode {
        let w = text.chars().count() as f32 * NODE_FS * ADVANCE + quote_pad(text) + NODE_PAD_X * 2.0;
        GNode {
            id: 0,
            kind: GKind::Token {
                text: text.to_string(),
                label: None,
                dash: false,
            },
            w,
            h: NODE_H,
            cw: w,
            ch: NODE_H,
        }
    }

    fn lab(text: &str, label: &str) -> GNode {
        let mut node = tok(text);
        if let GKind::Token { label: l, .. } = &mut node.kind {
            *l = Some(label.to_string());
        }
        // 官方 `_r` 固定用 16px 量标签宽度，与标签实际渲染的 14px 不同
        let label_w = label.chars().count() as f32 * NODE_FS * ADVANCE + NODE_PAD_X * 2.0;
        node.w = node.cw.max(label_w);
        node.h = node.ch + LABEL_BOX * 2.0;
        node
    }

    fn rep(child: GNode, min: u32, max: Option<u32>) -> GNode {
        let (text, infinite) = match max {
            Some(m) if m == min => (format!(" {min}"), false),
            Some(m) => (format!(" {min} - {m}"), false),
            None => (format!(" {min} - "), true),
        };
        let label_text_w = text.chars().count() as f32 * LABEL_FS * ADVANCE;
        let label_w = ICON_W + label_text_w + if infinite { ICON_W } else { 0.0 };
        let (cw, ch, child_w) = (child.cw, child.ch, child.w);
        GNode {
            id: 0,
            kind: GKind::Repeat {
                child: Box::new(child),
                label: text,
                label_text_w,
                label_w,
                infinite,
            },
            w: child_w.max(label_w),
            h: ch + LABEL_BOX * 2.0,
            cw,
            ch,
        }
    }

    fn group(child: GNode, label: Option<&str>) -> GNode {
        let cw = child.w + GROUP_PAD_H * 2.0;
        let ch = child.h + GROUP_PAD_V * 2.0;
        let label_w = label
            .map(|l| l.chars().count() as f32 * NODE_FS * ADVANCE + NODE_PAD_X * 2.0)
            .unwrap_or(0.0);
        let dh = if label_w > 0.0 { LABEL_BOX * 2.0 } else { 0.0 };
        GNode {
            id: 0,
            kind: GKind::Group {
                child: Box::new(child),
                label: label.map(str::to_string),
            },
            w: cw.max(label_w),
            h: ch + dh,
            cw,
            ch,
        }
    }

    fn assert_close(actual: f32, expected: f32, what: &str) {
        assert!(
            (actual - expected).abs() < 0.05,
            "{what}: 期望 {expected}, 实际 {actual}"
        );
    }

    /// 所有可见矩形（跳过官方那个不可见的分支容器）
    fn rects(prims: &[Prim]) -> Vec<(f32, f32, f32, f32)> {
        prims
            .iter()
            .filter_map(|p| match p {
                Prim::RoundRect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
                _ => None,
            })
            .collect()
    }

    /// 官方那个 `fill-transparent` 的分支容器
    fn frames(prims: &[Prim]) -> Vec<(f32, f32, f32, f32)> {
        prims
            .iter()
            .filter_map(|p| match p {
                Prim::Frame { x, y, w, h } => Some((*x, *y, *w, *h)),
                _ => None,
            })
            .collect()
    }

    /// 最外层的可见矩形（分组/重复节点的子框会排在它后面）
    fn rect_at(prims: &[Prim]) -> (f32, f32, f32, f32) {
        let v = rects(prims);
        assert!(!v.is_empty(), "未找到可见矩形");
        v[0]
    }

    fn circles(prims: &[Prim]) -> Vec<(f32, f32, f32)> {
        prims
            .iter()
            .filter_map(|p| match p {
                Prim::Circle { cx, cy, r, .. } => Some((*cx, *cy, *r)),
                _ => None,
            })
            .collect()
    }

    /// 官方 SVG 实测：`/a/` → 222.8×128，节点 (85,50,52.8,28)，圆环 cx 55 / 167.8
    #[test]
    fn plain_literal_matches_official() {
        let d = layout_diagram(&tok("\"a\""));
        assert_close(d.width, 222.8, "画布宽");
        assert_close(d.height, 128.0, "画布高");
        let (x, y, w, h) = rect_at(&d.prims);
        assert_close(x, 85.0, "节点 x");
        assert_close(y, 50.0, "节点 y");
        assert_close(w, 52.8, "节点宽");
        assert_close(h, 28.0, "节点高");
        let c = circles(&d.prims);
        assert_close(c[0].0, 55.0, "起点圆心 x");
        assert_close(c[0].1, 64.0, "起点圆心 y");
        assert_close(c[1].0, 167.8, "终点圆心 x");
    }

    /// 官方 SVG 实测：`/a*/` → 248×160，包裹盒 78 宽，节点居中缩进 12.6
    #[test]
    fn repeat_wrapper_matches_official() {
        let d = layout_diagram(&rep(tok("\"a\""), 0, None));
        assert_close(d.width, 248.0, "画布宽");
        assert_close(d.height, 160.0, "画布高");
        assert_close(rect_at(&d.prims).0, 97.6, "节点 x（包裹盒内居中）");
        let c = circles(&d.prims);
        assert_close(c[1].0, 193.0, "终点圆心 x");
    }

    /// 官方 SVG 实测：`/a?/` → 238.4×160，标签 " 0 - 1" 宽 68.4 → 缩进 7.8
    #[test]
    fn zero_or_one_label_width_matches_official() {
        let d = layout_diagram(&rep(tok("\"a\""), 0, Some(1)));
        assert_close(d.width, 238.4, "画布宽");
        assert_close(rect_at(&d.prims).0, 92.8, "节点 x");
    }

    /// 官方 SVG 实测：`/\d+/` → 276.4×160，节点宽 106.4（标签 78 更窄，取节点宽）
    #[test]
    fn perl_class_repeat_matches_official() {
        let d = layout_diagram(&rep(tok("Any digit"), 1, None));
        assert_close(d.width, 276.4, "画布宽");
        assert_close(d.height, 160.0, "画布高");
        let (x, _, w, _) = rect_at(&d.prims);
        assert_close(x, 85.0, "节点 x");
        assert_close(w, 106.4, "节点宽");
        let c = circles(&d.prims);
        assert_close(c[1].0, 221.4, "终点圆心 x");
    }

    /// 官方 SVG 实测：`/a|b/` → 272.8×191，分支容器 102.8×91，行中心 74 / 117
    #[test]
    fn branch_matches_official() {
        let d = layout_diagram(&build_alternate(vec![tok("\"a\""), tok("\"b\"")]));
        assert_close(d.width, 272.8, "画布宽");
        assert_close(d.height, 191.0, "画布高");
        let c = circles(&d.prims);
        assert_close(c[0].1, 95.5, "干线 y");
        // 第一行节点 y = 60，第二行 y = 103
        let mut ys: Vec<f32> = d
            .prims
            .iter()
            .filter_map(|p| match p {
                Prim::RoundRect { y, .. } => Some(*y),
                _ => None,
            })
            .collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_close(ys[0], 60.0, "首行节点 y");
        assert_close(ys[1], 103.0, "次行节点 y");
    }

    /// 官方 SVG 实测：`/(abc)/` → 292×190，分组框 122×58，标签抬高 21
    #[test]
    fn group_matches_official() {
        let d = layout_diagram(&group(tok("\"abc\""), Some("Group #1")));
        assert_close(d.width, 292.0, "画布宽");
        assert_close(d.height, 190.0, "画布高");
        let (x, y, w, h) = rect_at(&d.prims);
        assert_close(x, 85.0, "分组框 x");
        assert_close(y, 66.0, "分组框 y");
        assert_close(w, 122.0, "分组框宽");
        assert_close(h, 58.0, "分组框高");
    }

    /// 官方 SVG 实测：`/(?:a)/` → 272.8×158，无标签时布局盒等于内容盒
    #[test]
    fn non_capturing_group_matches_official() {
        let d = layout_diagram(&group(tok("\"a\""), None));
        assert_close(d.width, 272.8, "画布宽");
        assert_close(d.height, 158.0, "画布高");
        let (x, y, w, h) = rect_at(&d.prims);
        assert_close(x, 85.0, "分组框 x");
        assert_close(y, 50.0, "分组框 y");
        assert_close(w, 102.8, "分组框宽");
        assert_close(h, 58.0, "分组框高");
    }

    /// 官方 SVG 实测：`/[^a]/` → 257.2×160，标签 "None of" 比内容宽 → 内容水平居中
    #[test]
    fn wide_label_centers_content() {
        let d = layout_diagram(&lab("\"a\"", "None of"));
        assert_close(d.width, 257.2, "画布宽");
        assert_close(d.height, 160.0, "画布高");
        let (x, _, w, _) = rect_at(&d.prims);
        assert_close(x, 102.2, "内容盒 x");
        assert_close(w, 52.8, "内容盒宽");
    }

    /// 官方 SVG 实测：`/x|(a)/` → 322.8×253
    /// 分支容器 152.8×153（不可见），首行 y=60，分组内容框 y=119
    #[test]
    fn choice_with_group_matches_official() {
        let d = layout_diagram(&build_alternate(vec![
            tok("\"x\""),
            group(tok("\"a\""), Some("Group #1")),
        ]));
        assert_close(d.width, 322.8, "画布宽");
        assert_close(d.height, 253.0, "画布高");
        let f = frames(&d.prims);
        assert_eq!(f.len(), 1, "分支容器数量");
        assert_close(f[0].0, 85.0, "分支容器 x");
        assert_close(f[0].1, 50.0, "分支容器 y");
        assert_close(f[0].2, 152.8, "分支容器宽");
        assert_close(f[0].3, 153.0, "分支容器高");
        let mut v = rects(&d.prims);
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        // 首行 token "x"：分支盒高 28，容器顶 50 + 10
        assert_close(v[0].0, 135.0, "首行 x");
        assert_close(v[0].1, 60.0, "首行 y");
        // 次行分组内容框：分组布局盒高 90，起点 60+28+15 = 103，内容再下移 16
        assert_close(v[1].0, 110.0, "分组框 x");
        assert_close(v[1].1, 119.0, "分组框 y");
        assert_close(v[1].2, 102.8, "分组框宽");
    }

    /// 官方 SVG 实测：`/a\d+/` → 354.2×160，拼接盒高取重复节点（60），
    /// 短节点纵向居中到 y=66
    #[test]
    fn concat_with_repeat_matches_official() {
        let d = layout_diagram(&build_concat(vec![tok("\"a\""), rep(tok("Any digit"), 1, None)]));
        assert_close(d.width, 354.2, "画布宽");
        assert_close(d.height, 160.0, "画布高");
        let mut v = rects(&d.prims);
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        assert_close(v[0].0, 85.0, "首项 x");
        assert_close(v[0].1, 66.0, "首项 y（在 60 高的拼接盒里居中）");
        assert_close(v[1].0, 162.8, "重复项 x");
        assert_close(v[1].1, 66.0, "重复项 y");
    }

    /// 官方 SVG 实测：`/(a)+/` → 272.8×190，布局盒 = 内容高 58 + 32
    #[test]
    fn quantified_group_matches_official() {
        let d = layout_diagram(&rep(group(tok("\"a\""), Some("Group #1")), 1, None));
        assert_close(d.width, 272.8, "画布宽");
        assert_close(d.height, 190.0, "画布高");
        let (_, y, _, h) = rect_at(&d.prims);
        assert_close(y, 66.0, "分组框 y");
        assert_close(h, 58.0, "分组框高");
    }

    /// 官方 SVG 实测：`/^a$/` → 504.8×128，三段拼接 125.6 + 25 + 52.8 + 25 + 106.4
    #[test]
    fn assertion_concat_matches_official() {
        let concat = build_concat(vec![
            tok("Begins with"),
            tok("\"a\""),
            tok("Ends with"),
        ]);
        let d = layout_diagram(&concat);
        assert_close(d.width, 504.8, "画布宽");
        assert_close(d.height, 128.0, "画布高");
    }

    /// 分支分/汇曲线端点应落在官方给出的绝对坐标上，且弧的旋向要一致
    #[test]
    fn branch_curves_match_official_paths() {
        let m = |c: &Cmd| match c {
            Cmd::Move(x, y) | Cmd::Line(x, y) => (*x, *y),
            Cmd::Arc { to, .. } => *to,
            Cmd::Close => panic!("曲线测试不应出现 Close"),
        };
        let sweep = |c: &Cmd| match c {
            Cmd::Arc { sweep, .. } => Some(*sweep),
            _ => None,
        };

        // 官方：M85,95.5 L95,95.5 A5 5 0 0 0, 100,90.5 L100,79 A5 5 0 0 1, 105,74 L110,74
        let split = split_cmds(85.0, 95.5, 74.0, 110.0, 5.0);
        assert_close(m(&split[0]).0, 85.0, "split 起点 x");
        assert_close(m(&split[0]).1, 95.5, "split 起点 y");
        assert_close(m(&split[1]).0, 95.0, "split 直线段 x");
        assert_close(m(&split[2]).1, 90.5, "split 首弧终点 y");
        assert_close(m(&split[3]).1, 79.0, "split 竖直段终点 y");
        assert_close(m(&split[4]).0, 105.0, "split 次弧终点 x");
        assert_close(m(&split[5]).0, 110.0, "split 收尾 x");
        assert_eq!(sweep(&split[2]), Some(false), "split 首弧旋向");
        assert_eq!(sweep(&split[4]), Some(true), "split 次弧旋向");

        // 行在干线上方：官方 M162.8,74 L167.8,74 A5 5 0 0 1, 172.8,79 L172.8,90.5
        //                    A5 5 0 0 0, 177.8,95.5 L187.8,95.5
        let merge = merge_cmds(162.8, 74.0, 95.5, 187.8, 5.0);
        assert_close(m(&merge[1]).0, 167.8, "merge 直线段 x");
        assert_close(m(&merge[2]).1, 79.0, "merge 首弧终点 y");
        assert_close(m(&merge[4]).0, 177.8, "merge 次弧终点 x");
        assert_close(m(&merge[5]).0, 187.8, "merge 收尾 x");
        assert_close(m(&merge[5]).1, 95.5, "merge 收尾 y");
        assert_eq!(sweep(&merge[2]), Some(true), "merge 首弧旋向");
        assert_eq!(sweep(&merge[4]), Some(false), "merge 次弧旋向");

        // 行在干线下方：官方 M162.8,117 L167.8,117 A5 5 0 0 0, 172.8,112
        //                    L172.8,100.5 A5 5 0 0 1, 177.8,95.5 L187.8,95.5
        let merge2 = merge_cmds(162.8, 117.0, 95.5, 187.8, 5.0);
        assert_close(m(&merge2[2]).1, 112.0, "下行 merge 首弧终点 y");
        assert_eq!(sweep(&merge2[2]), Some(false), "下行 merge 首弧旋向");
        assert_eq!(sweep(&merge2[4]), Some(true), "下行 merge 次弧旋向");
    }

    /// 窄行汇入宽容器时，竖段锚在容器右缘而非行右缘
    /// （官方 `/x|(a)/`：M187.8,74 L217.8,74 A5.. 222.8,79）
    #[test]
    fn merge_anchors_to_container_edge() {
        let m = |c: &Cmd| match c {
            Cmd::Move(x, y) | Cmd::Line(x, y) => (*x, *y),
            Cmd::Arc { to, .. } => *to,
            Cmd::Close => panic!("曲线测试不应出现 Close"),
        };
        let merge = merge_cmds(187.8, 74.0, 126.5, 237.8, 5.0);
        assert_close(m(&merge[1]).0, 217.8, "merge 直线段 x = 容器右缘 - 20");
        assert_close(m(&merge[2]).0, 222.8, "merge 竖段 x = 容器右缘 - 15");
        assert_close(m(&merge[4]).0, 227.8, "merge 次弧终点 x = 容器右缘 - 10");
        assert_close(m(&merge[5]).0, 237.8, "merge 收尾到容器右缘");
    }

    /// 节点宽度的引号留白规则
    #[test]
    fn quote_padding_matches_official() {
        assert_close(quote_pad("\"a\""), 4.0, "\"a\"");
        assert_close(quote_pad("\"a\" - \"z\""), 8.0, "区间");
        assert_close(quote_pad("Any digit"), 0.0, "字符类");
    }

    /// 把布局结果导出为 SVG，便于与官方渲染做像素级目视比对。
    /// `cargo test -p ui --bin toolbox emit_svg -- --ignored`
    #[test]
    #[ignore]
    fn emit_svg() {
        fn esc(s: &str) -> String {
            s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        }
        fn to_svg(d: &Diagram) -> String {
            let mut out = format!(
                "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}' viewBox='0 0 {} {}'>",
                d.width, d.height, d.width, d.height
            );
            for prim in &d.prims {
                match prim {
                    Prim::RoundRect {
                        x,
                        y,
                        w,
                        h,
                        r,
                        stroke,
                        dash,
                    } => out.push_str(&format!(
                        "<rect x='{x}' y='{y}' width='{w}' height='{h}' rx='{r}' ry='{r}' fill='none' \
                         stroke='#{stroke:06x}' stroke-width='1.5'{} />",
                        if *dash { " stroke-dasharray='4 2'" } else { "" }
                    )),
                    Prim::Frame { x, y, w, h } => out.push_str(&format!(
                        "<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='transparent' />"
                    )),
                    // 选中高亮层：对官方 SVG 的几何比对里选中集恒为空，不会出现
                    Prim::Highlight { .. } => {}
                    Prim::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        stroke,
                    } => out.push_str(&format!(
                        "<path d='M{x1},{y1}L{x2},{y2}' fill='none' stroke='#{stroke:06x}' stroke-width='1.5' />"
                    )),
                    Prim::Curve {
                        cmds,
                        stroke,
                        filled,
                    } => {
                        let mut d_attr = String::new();
                        for c in cmds {
                            match c {
                                Cmd::Move(x, y) => d_attr.push_str(&format!("M{x},{y}")),
                                Cmd::Line(x, y) => d_attr.push_str(&format!("L{x},{y}")),
                                Cmd::Arc {
                                    r,
                                    large,
                                    sweep,
                                    to,
                                } => d_attr.push_str(&format!(
                                    "A{r},{r} 0 {},{} {},{}",
                                    *large as u8,
                                    *sweep as u8,
                                    to.0,
                                    to.1
                                )),
                                Cmd::Close => d_attr.push('Z'),
                            }
                        }
                        out.push_str(&format!(
                            "<path d='{d_attr}' {} />",
                            if *filled {
                                format!("fill='#{stroke:06x}'")
                            } else {
                                format!("fill='none' stroke='#{stroke:06x}' stroke-width='1.5'")
                            }
                        ));
                    }
                    Prim::Circle { cx, cy, r, stroke } => out.push_str(&format!(
                        "<circle cx='{cx}' cy='{cy}' r='{r}' fill='none' stroke='#{stroke:06x}' stroke-width='1.5' />"
                    )),
                    Prim::Text {
                        x,
                        y,
                        w,
                        text,
                        fs,
                        color,
                        center,
                    } => {
                        let (tx, anchor) = if *center {
                            (x + w / 2.0, "middle")
                        } else {
                            (*x, "start")
                        };
                        out.push_str(&format!(
                            "<text x='{tx}' y='{}' font-size='{fs}' font-family='monospace' \
                             text-anchor='{anchor}' fill='#{color:06x}'>{}</text>",
                            y + fs * 0.78,
                            esc(text)
                        ));
                    }
                }
            }
            out.push_str("</svg>");
            out
        }

        // 用例名与 .workbuddy/shots/off/<name>.svg 一一对应（官方基准）
        let neg = {
            let mut n = lab("\"a\"", "None of");
            if let GKind::Token { dash, .. } = &mut n.kind {
                *dash = true;
            }
            n
        };
        let cases: Vec<(&str, Diagram)> = vec![
            ("a", layout_diagram(&tok("\"a\""))),
            ("a_star", layout_diagram(&rep(tok("\"a\""), 0, None))),
            ("a_opt", layout_diagram(&rep(tok("\"a\""), 0, Some(1)))),
            ("d_plus", layout_diagram(&rep(tok("Any digit"), 1, None))),
            ("range", layout_diagram(&lab("\"a\" - \"z\"", "One of"))),
            ("neg_class", layout_diagram(&neg)),
            ("group_abc", layout_diagram(&group(tok("\"abc\""), Some("Group #1")))),
            ("group_nc", layout_diagram(&group(tok("\"a\""), None))),
            (
                "group_plus",
                layout_diagram(&rep(group(tok("\"a\""), Some("Group #1")), 1, None)),
            ),
            (
                "choice_group",
                layout_diagram(&build_alternate(vec![
                    tok("\"x\""),
                    group(tok("\"a\""), Some("Group #1")),
                ])),
            ),
            (
                "choice_group2",
                layout_diagram(&build_alternate(vec![
                    tok("\"a\""),
                    build_concat(vec![tok("\"b\""), group(tok("\"c\""), Some("Group #1"))]),
                ])),
            ),
            (
                "concat_repeat",
                layout_diagram(&build_concat(vec![tok("\"a\""), rep(tok("Any digit"), 1, None)])),
            ),
        ];

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.workbuddy/shots");
        std::fs::create_dir_all(&dir).unwrap();
        for (name, diagram) in &cases {
            let path = dir.join(format!("mine_{name}.svg"));
            std::fs::write(&path, to_svg(diagram)).unwrap();
            println!("wrote {}", path.display());
        }
    }
}

fn highlight_preview_panel(
    text: &str,
    matches: &[RegexMatch],
    cx: &mut Context<RegexVisualizer>,
    lang: Lang,
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
                        .child(lang.of("暂无测试文本", "No test text"))
                })
                .when(!text.is_empty(), |this| this.child(styled)),
        )
}


/// 文案与图例结构的校验。
///
/// 中文标签的**排版**（字宽、换行）需要真实窗口的文本系统才能测，
/// 沙箱里拿不到（无 Wayland），所以这里只校验**文案内容与结构**——
/// 每个字符串都逐条对齐官网 `/locales/cn/translation.json`（115 条）。
#[cfg(test)]
mod i18n_tests {
    use super::{
        beginning_label, boundary_label, class_label, empty_label, ending_label, group_word,
        lookaround_label, none_of_label, one_of_label, Lang, LEGEND_TEXT, PANEL_TABS, SAMPLES,
    };

    /// 官方 jn 表 + cn 语言包。`\f` 官方没给中文译名，回退英文原文。
    #[test]
    fn class_labels_match_official_locale() {
        let expect = [
            (".", "任意字符", "Any character"),
            (r"\d", "任意数字", "Any digit"),
            (r"\D", "任意非数字", "Non-digit"),
            (r"\w", "任意基本拉丁字母数字", "Any alphanumeric"),
            (r"\W", "任意非基本拉丁字母数字", "Non-alphanumeric"),
            (r"\s", "任意空白字符", "White space"),
            (r"\S", "任意非空白字符", "Non-white space"),
            (r"\t", "制表符", "Horizontal tab"),
            (r"\r", "回车符", "Carriage return"),
            (r"\n", "换行符", "Linefeed"),
            (r"\v", "垂直制表符", "Vertical tab"),
            (r"\f", "Form-feed", "Form-feed"),
            (r"\0", "NUL", "NUL"),
        ];
        for (raw, cn, en) in expect {
            assert_eq!(class_label(raw, Lang::Cn), Some(cn), "中文 {raw}");
            assert_eq!(class_label(raw, Lang::En), Some(en), "英文 {raw}");
        }
        // 表外的写法不翻译，回退成原文（官方也是这样）
        assert_eq!(class_label(r"\p{Greek}", Lang::Cn), None);
        assert_eq!(class_label("[a-z]", Lang::Cn), None);
    }

    #[test]
    fn misc_labels_match_official_locale() {
        assert_eq!(beginning_label(Lang::Cn), "以...开始");
        assert_eq!(beginning_label(Lang::En), "Begins with");
        assert_eq!(ending_label(Lang::Cn), "以...结束");
        assert_eq!(ending_label(Lang::En), "Ends with");

        // 官方 vh.word 的英文态没有资源文件，直接回退成 key 本身
        assert_eq!(boundary_label(Lang::Cn, false), "单词边界");
        assert_eq!(boundary_label(Lang::Cn, true), "非单词边界");
        assert_eq!(boundary_label(Lang::En, false), "WordBoundary");
        assert_eq!(boundary_label(Lang::En, true), "NonWordBoundary");

        assert_eq!(one_of_label(Lang::Cn), "其一");
        assert_eq!(none_of_label(Lang::Cn), "没有其一");
        assert_eq!(one_of_label(Lang::En), "One of");
        assert_eq!(none_of_label(Lang::En), "None of");
        assert_eq!(empty_label(Lang::Cn), "空");
        assert_eq!(empty_label(Lang::En), "Empty");

        // 官方 v8：`${t("Group")} #${e.name}`
        assert_eq!(group_word(Lang::Cn), "组");
        assert_eq!(group_word(Lang::En), "Group");

        // 官方 vh.lookahead / vh.lookbehind
        assert_eq!(lookaround_label(Lang::Cn, true, false), "接着:");
        assert_eq!(lookaround_label(Lang::Cn, true, true), "不接着:");
        assert_eq!(lookaround_label(Lang::Cn, false, false), "前面是:");
        assert_eq!(lookaround_label(Lang::Cn, false, true), "前面不是:");
        assert_eq!(lookaround_label(Lang::En, true, false), "Followed by:");
        assert_eq!(lookaround_label(Lang::En, true, true), "Not followed by:");
        assert_eq!(lookaround_label(Lang::En, false, false), "Preceded by:");
        assert_eq!(lookaround_label(Lang::En, false, true), "Not preceded by:");
    }

    /// 官方 yB：8 组、共 13 项
    #[test]
    fn legend_shape_matches_official_yb() {
        let zh: Vec<&str> = LEGEND_TEXT.iter().map(|(cn, _, _)| *cn).collect();
        assert_eq!(
            zh,
            ["字符", "字符类", "范围", "或", "量词", "组", "反向引用", "断言"]
        );
        let en: Vec<&str> = LEGEND_TEXT.iter().map(|(_, en, _)| *en).collect();
        assert_eq!(
            en,
            [
                "Characters",
                "Character classes",
                "Ranges",
                "Choice",
                "Quantifier",
                "Group",
                "Back reference",
                "Assertion",
            ]
        );
        // 各组条目数
        let counts: Vec<usize> = LEGEND_TEXT.iter().map(|(_, _, e)| e.len()).collect();
        assert_eq!(counts, [1, 1, 2, 1, 1, 3, 2, 2]);
        assert_eq!(counts.iter().sum::<usize>(), 13);
    }

    /// 右面板页签与样本列表对齐官网 cn 语言包
    #[test]
    fn panel_tabs_and_samples_match_official_locale() {
        let zh: Vec<&str> = PANEL_TABS.iter().map(|(cn, _)| *cn).collect();
        assert_eq!(zh, ["图例", "编辑", "测试", "样例"]);
        let en: Vec<&str> = PANEL_TABS.iter().map(|(_, en)| *en).collect();
        assert_eq!(en, ["Legends", "Edit", "Test", "Samples"]);

        assert_eq!(SAMPLES.len(), 6);
        assert_eq!(SAMPLES[0].0, "1. 整数");
        assert_eq!(SAMPLES[0].1, "1. Whole Numbers");
        assert_eq!(SAMPLES[3].0, "4. 正负 整数 + 小数");
        assert_eq!(SAMPLES[5].0, "6. 日期格式 YYYY-MM-dd");
        assert_eq!(SAMPLES[5].1, "6. Date Format YYYY-MM-dd");
    }
}

#[cfg(test)]
mod edit_tests {
    use super::{
        class_kind_key, content_type_keys, convert_seq, group_kind_key, look_kind_key, set_group_name,
        BoundaryKind, CharKind, ContentSpec, EGroupKind, EKind, ERoot, IdGen, InsertMode, Lang,
        LookKind, NodeId, QuantKind, Quantifier, Step, WrapKind,
    };

    fn lang() -> Lang {
        Lang::Cn
    }

    /// 按原文字符串建树（和视图里 `evaluate` 的解析路径一致）
    fn build(pattern: &str) -> ERoot {
        let ast = regex_syntax::ast::parse::Parser::new()
            .parse(pattern)
            .unwrap_or_else(|e| panic!("{pattern} 解析失败: {e}"));
        let mut id_gen = IdGen(0);
        ERoot {
            body: convert_seq(&ast, pattern, &mut id_gen),
        }
    }

    fn ids(tree: &ERoot) -> Vec<NodeId> {
        tree.body.iter().map(|n| n.id).collect()
    }

    fn first_body_child(tree: &ERoot, path: &[Step], index: usize) -> NodeId {
        super::seq_at(&tree.body, path)[index].id
    }

    // ------------------------------------------------------------------
    // 序列化器：解析 → 建树 → 回写必须逐字还原
    // ------------------------------------------------------------------

    #[test]
    fn pattern_round_trips_through_the_tree() {
        let cases = [
            "",
            "a",
            "abc",
            r"\w+",
            r"\d{2,}",
            r"\d{2,4}",
            r"[a-z0-9]",
            r"[abc]",
            r"[^a-z]",
            r"x|y",
            r"a|b|c",
            r"(abc)",
            r"(?:abc)",
            r"(?<name>abc)",
            r"(abc)+",
            r"(?:x)*",
            r"^ab$",
            r"\bword\b",
            r"\Bx",
            r"a+?",
            r"a??",
            r"a*?",
            r"a{3}",
            r"a{3,}",
            r"a{3,5}?",
            r"[a-z]*x",
            r"(a|b)+c",
            r"\.\*",
            r"\n\t",
            r"[\d\s]",
        ];
        for pattern in cases {
            let tree = build(pattern);
            assert_eq!(tree.to_pattern(), pattern, "round-trip 失败：{pattern}");
        }

        // 例外：字符类里的转义写法会被**规范化**掉，官方也一样。
        // 官方 `parseRanges`：`l = i === EscapedChar ? regex.slice(s+1, a) : regex.slice(s, a)`
        // —— `\-` 读进来就被剥掉反斜杠存成 `-`，序列化时又只有**非首尾**的 `-`
        // 才补转义，所以开头那个 `-` 原样输出，`[\-a]` 稳定收敛成 `[-a]`。
        for (input, expected) in [
            (r"[\-a]", "[-a]"),
            (r"[a\-z]", "[a\\-z]"), // 中间位置的 `-` 官方才会补回转义
        ] {
            let tree = build(input);
            assert_eq!(tree.to_pattern(), expected, "规范化结果应稳定：{input}");
            // 收敛：再跑一遍不再变化
            let again = build(&tree.to_pattern());
            assert_eq!(again.to_pattern(), expected, "不是不动点：{input}");
        }
    }

    #[test]
    fn consecutive_literals_merge_into_one_string_node() {
        let tree = build("abc");
        assert_eq!(tree.body.len(), 1, "连续裸字面量应合并成一个 string 节点");
        match &tree.body[0].kind {
            EKind::Character { kind, value, .. } => {
                assert!(matches!(kind, CharKind::String));
                assert_eq!(value, "abc");
            }
            other => panic!("期望 string 节点，得到 {other:?}"),
        }
    }

    #[test]
    fn escaped_literal_breaks_the_merge() {
        let tree = build("a\\nb");
        assert_eq!(tree.body.len(), 3);
        assert!(matches!(
            &tree.body[1].kind,
            EKind::Character { kind: CharKind::Class, value, .. } if value == "\\n"
        ));
    }

    // ------------------------------------------------------------------
    // 选中区间（官方 `qA`，用于表达式段的高亮）
    // ------------------------------------------------------------------

    #[test]
    fn span_points_at_the_selected_slice() {
        let tree = build(r"(abc)+\d{2,3}");
        let group = tree.body[0].id;
        let digit = tree.body[1].id;
        let (pattern, start, end) = tree.pattern_with_span(group, group);
        assert_eq!(pattern, r"(abc)+\d{2,3}");
        assert_eq!(&pattern[start..end], "(abc)+");

        let (_, start, end) = tree.pattern_with_span(digit, digit);
        assert_eq!(&pattern[start..end], r"\d{2,3}");

        // 跨节点区间：首尾 id 之间的整段
        let (_, start, end) = tree.pattern_with_span(group, digit);
        assert_eq!(&pattern[start..end], r"(abc)+\d{2,3}");
    }

    // ------------------------------------------------------------------
    // 选中信息（官方 `fB`）
    // ------------------------------------------------------------------

    #[test]
    fn selection_reports_quantifier_and_group_details() {
        let tree = build(r"(?<n>ab)+");
        let id = tree.body[0].id;
        let info = tree.selection(&[id]);
        assert!(info.single);
        assert!(info.has_quantifier);
        assert_eq!(info.quantifier.map(|q| q.kind), Some(QuantKind::Plus));
        assert_eq!(
            info.group.as_ref().map(|(k, _)| *k),
            Some(EGroupKind::NamedCapturing)
        );
        assert_eq!(info.group.as_ref().map(|(_, n)| n.clone()), Some("n".into()));
        assert!(info.first && info.last);
        assert!(info.content.is_none(), "分组没有「内容」段");
    }

    #[test]
    fn selection_reports_content_types() {
        let tree = build(r"[a-z]");
        let id = tree.body[0].id;
        let info = tree.selection(&[id]);
        assert_eq!(
            info.content,
            Some(ContentSpec::Ranges {
                ranges: vec![("a".into(), "z".into())],
                negate: false,
            })
        );
        // `has_quantifier` 是「允许带量词」（官方 `y0`），不是「已经带了」。
        // `[a-z]` 属于 character/ranges，`y0` 认它 → true；但它身上并没有量词。
        assert!(info.has_quantifier, "字符范围允许带量词（[a-z]+ 合法）");
        assert!(info.quantifier.is_none(), "但本身没有量词");
    }

    #[test]
    fn first_and_last_only_look_at_the_root_sequence() {
        let tree = build(r"^(a)$");
        // 手动取嵌套在组里的那个 `a`。不能用「先序铺平的最后一个」——
        // 那是根序列末尾的 `$`，恰好 `is_last` 为真，断言会自相矛盾。
        let inner = match &tree.body[1].kind {
            EKind::Group { children, .. } => children[0].id,
            other => panic!("body[1] 应是分组，实际是 {other:?}"),
        };
        assert!(!tree.is_first(inner), "嵌套在组里的节点不算根序列首项");
        assert!(!tree.is_last(inner), "嵌套在组里的节点不算根序列末项");
        // 根序列：`^`、(a)、`$`
        assert!(tree.is_first(tree.body[0].id));
        assert!(tree.is_last(tree.body[2].id));
        assert!(tree.body[0].id != tree.body[2].id);
    }

    #[test]
    fn content_type_options_follow_the_visibility_rules() {
        // 无捕获组、既非首也非尾 → 只有四项
        // 无捕获组、既非首也非尾 → 只有四项。
        // 选中项必须是**中间**节点：`a[bc]` 里 `body[1]` 已是末项，会多出
        // endAssertion，所以这里用 `a[bc]d` 把它夹在中间。
        let tree = build(r"a[bc]d");
        assert_eq!(tree.body.len(), 3, "应是 a / [bc] / d 三个节点");
        let picked = tree.body[1].id;
        assert!(!tree.is_first(picked) && !tree.is_last(picked), "选取的应是中间节点");
        let info = tree.selection(&[picked]);
        assert_eq!(
            content_type_keys(&info, 0),
            ["string", "class", "ranges", "wordBoundaryAssertion"]
        );

        // 首项 + 有捕获组 → 追加 backReference / beginningAssertion
        let tree = build(r"(a)b");
        let info = tree.selection(&[tree.body[0].id]);
        assert_eq!(
            content_type_keys(&info, 1),
            [
                "string",
                "class",
                "ranges",
                "wordBoundaryAssertion",
                "backReference",
                "beginningAssertion",
            ]
        );

        // 尾项 → 追加 endAssertion
        let tree = build(r"(a)b");
        let info = tree.selection(&[tree.body[1].id]);
        assert_eq!(
            content_type_keys(&info, 1),
            [
                "string",
                "class",
                "ranges",
                "wordBoundaryAssertion",
                "backReference",
                "endAssertion",
            ]
        );
    }

    // ------------------------------------------------------------------
    // 变更操作
    // ------------------------------------------------------------------

    #[test]
    fn insert_before_parallel_after() {
        let mut tree = build("abc");
        let id = tree.body[0].id;
        let mut id_gen = IdGen(100);

        tree.insert_around(&[id], InsertMode::Before, &mut id_gen);
        assert_eq!(tree.to_pattern(), "abc");
        assert_eq!(tree.body.len(), 2);
        assert_eq!(tree.body[1].id, id, "原有节点仍在原位之后");

        let before = tree.body[0].id;
        assert!(matches!(
            &tree.body[0].kind,
            EKind::Character { kind: CharKind::String, value, .. } if value.is_empty()
        ));
        assert!(before != id);

        tree.insert_around(&[id], InsertMode::After, &mut id_gen);
        assert_eq!(tree.body.len(), 3);

        // Parallel：单选一个 choice 时直接给它补一条分支
        let mut tree = build("a|b");
        let choice = tree.body[0].id;
        tree.insert_around(&[choice], InsertMode::Parallel, &mut id_gen);
        match &tree.body[0].kind {
            EKind::Choice { branches } => assert_eq!(branches.len(), 3),
            other => panic!("期望 choice，得到 {other:?}"),
        }
    }

    #[test]
    fn wrap_and_unwrap_group() {
        let mut tree = build("abc");
        let ids = ids(&tree);

        let wrapped = tree.wrap(&ids, WrapKind::Group(EGroupKind::NonCapturing), &mut IdGen(100));
        assert_eq!(tree.to_pattern(), "(?:abc)");
        assert_eq!(wrapped.len(), 1);

        // 解组后子节点原地展开，id 不变
        let group = wrapped[0];
        let restored = tree.set_group_kind(group, None);
        assert_eq!(tree.to_pattern(), "abc");
        assert_eq!(restored, ids);
    }

    #[test]
    fn named_group_with_empty_name_falls_back_to_name() {
        let mut tree = build("abc");
        let ids = ids(&tree);
        let wrapped = tree.wrap(
            &ids,
            WrapKind::Group(EGroupKind::NamedCapturing),
            &mut IdGen(100),
        );
        assert_eq!(tree.to_pattern(), "(?<name>abc)");

        set_group_name(&mut tree, wrapped[0], "");
        assert_eq!(tree.to_pattern(), "(?<name>abc)", "空名字回落成 name");

        set_group_name(&mut tree, wrapped[0], "word");
        assert_eq!(tree.to_pattern(), "(?<word>abc)");
    }

    #[test]
    fn group_kind_switch_rewrites_the_pattern() {
        let mut tree = build("(abc)");
        let group = tree.body[0].id;
        tree.set_group_kind(group, Some(EGroupKind::NonCapturing));
        assert_eq!(tree.to_pattern(), "(?:abc)");
        tree.set_group_kind(group, Some(EGroupKind::NamedCapturing));
        assert_eq!(tree.to_pattern(), "(?<name>abc)");
        tree.set_group_kind(group, Some(EGroupKind::Capturing));
        assert_eq!(tree.to_pattern(), "(abc)");
        assert_eq!(group_kind_key(EGroupKind::Capturing), "capturing");
        assert_eq!(group_kind_key(EGroupKind::NonCapturing), "nonCapturing");
        assert_eq!(group_kind_key(EGroupKind::NamedCapturing), "namedCapturing");
    }

    #[test]
    fn wrap_and_cancel_lookaround() {
        let mut tree = build("ab");
        let ids = ids(&tree);
        let wrapped = tree.wrap(&ids, WrapKind::LookAround(LookKind::Lookahead), &mut IdGen(100));
        assert_eq!(tree.to_pattern(), "(?=ab)");

        tree.set_lookaround(wrapped[0], Some((LookKind::Lookahead, true)));
        assert_eq!(tree.to_pattern(), "(?!ab)");

        tree.set_lookaround(wrapped[0], Some((LookKind::Lookbehind, true)));
        assert_eq!(tree.to_pattern(), "(?<!ab)");

        let restored = tree.set_lookaround(wrapped[0], None);
        assert_eq!(tree.to_pattern(), "ab");
        assert_eq!(restored, ids);
        assert_eq!(look_kind_key(LookKind::Lookahead), "lookahead");
        assert_eq!(look_kind_key(LookKind::Lookbehind), "lookbehind");
    }

    #[test]
    fn set_content_switches_between_every_kind() {
        let mut tree = build("a");
        let id = tree.body[0].id;
        let mut id_gen = IdGen(100);

        let same = tree.set_content(
            id,
            &ContentSpec::String {
                value: "xy".into(),
            },
            &mut id_gen,
        );
        assert_eq!(same, id);
        assert_eq!(tree.to_pattern(), "xy");

        tree.set_content(
            id,
            &ContentSpec::Class {
                value: "\\d".into(),
            },
            &mut id_gen,
        );
        assert_eq!(tree.to_pattern(), r"\d");

        tree.set_content(
            id,
            &ContentSpec::Ranges {
                ranges: vec![("a".into(), "z".into()), ("0".into(), "9".into())],
                negate: true,
            },
            &mut id_gen,
        );
        assert_eq!(tree.to_pattern(), "[^a-z0-9]");

        tree.set_content(id, &ContentSpec::WordBoundary { negate: false }, &mut id_gen);
        assert_eq!(tree.to_pattern(), r"\b");
        tree.set_content(id, &ContentSpec::WordBoundary { negate: true }, &mut id_gen);
        assert_eq!(tree.to_pattern(), r"\B");

        tree.set_content(id, &ContentSpec::Beginning, &mut id_gen);
        assert_eq!(tree.to_pattern(), "^");
        tree.set_content(id, &ContentSpec::End, &mut id_gen);
        assert_eq!(tree.to_pattern(), "$");
    }

    #[test]
    fn multi_char_string_gets_an_implicit_non_capturing_group_when_quantified() {
        let mut tree = build("a*");
        let id = tree.body[0].id;
        let mut id_gen = IdGen(100);
        let new_id = tree.set_content(
            id,
            &ContentSpec::String {
                value: "ab".into(),
            },
            &mut id_gen,
        );
        assert_eq!(tree.to_pattern(), "(?:ab)*", "多字符 + 量词要自动套非捕获组");
        assert_ne!(new_id, id, "包了一层组，返回的是新组的 id");
    }

    #[test]
    fn quantifier_switching_preserves_greediness() {
        let mut tree = build("a+?");
        let id = tree.body[0].id;
        let mut id_gen = IdGen(100);

        let q = Quantifier {
            kind: QuantKind::Star,
            min: 0,
            max: Quantifier::INF,
            greedy: false,
        };
        tree.set_quantifier(id, Some(q), &mut id_gen);
        assert_eq!(tree.to_pattern(), "a*?");

        tree.set_quantifier(
            id,
            Some(Quantifier {
                kind: QuantKind::Custom,
                min: 2,
                max: 3,
                greedy: false,
            }),
            &mut id_gen,
        );
        assert_eq!(tree.to_pattern(), "a{2,3}?");

        tree.set_quantifier(id, None, &mut id_gen);
        assert_eq!(tree.to_pattern(), "a");
    }

    #[test]
    fn remove_nodes_cleans_up_empty_shells() {
        let mut tree = build("(?:a)bc");
        let inner = first_body_child(&tree, &[Step::Children(0)], 0);
        tree.remove_nodes(&[inner]);
        assert_eq!(tree.to_pattern(), "bc", "空分组连壳一起删");

        // 分支空了就丢掉；只剩一条时把 choice 摊平
        let mut tree = build("a|b|c");
        let choice = tree.body[0].id;
        let b = match &tree.body[0].kind {
            EKind::Choice { branches } => branches[1][0].id,
            _ => unreachable!(),
        };
        let c = match &tree.body[0].kind {
            EKind::Choice { branches } => branches[2][0].id,
            _ => unreachable!(),
        };
        tree.remove_nodes(&[b, c]);
        assert_eq!(tree.to_pattern(), "a");
        assert_eq!(tree.body.len(), 1);
        assert!(
            !matches!(&tree.body[0].kind, EKind::Choice { .. }),
            "只剩一条分支后 choice 应被摊平"
        );
        assert_ne!(tree.body[0].id, choice);
    }

    #[test]
    fn remove_nodes_drops_one_branch_of_a_three_way_choice() {
        let mut tree = build("a|b|c");
        let b = match &tree.body[0].kind {
            EKind::Choice { branches } => branches[1][0].id,
            _ => unreachable!(),
        };
        tree.remove_nodes(&[b]);
        assert_eq!(tree.to_pattern(), "a|c");
    }

    // ------------------------------------------------------------------
    // 面板用到的小工具
    // ------------------------------------------------------------------

    #[test]
    fn class_kind_key_normalises_hex_escapes() {
        assert_eq!(class_kind_key(r"\x1b"), r"\xhh");
        assert_eq!(class_kind_key(r"\u00e9"), r"\uhhhh");
        assert_eq!(class_kind_key(r"\d"), r"\d");
        assert_eq!(class_kind_key(r"\xZZ"), r"\xZZ");
        assert_eq!(class_kind_key(r"\u00e"), r"\u00e");
        assert_eq!(class_kind_key("."), ".");
    }

    #[test]
    fn class_options_cover_every_official_entry() {
        let items = super::class_items(lang());
        assert_eq!(items.len(), 22, "官方 `jn` 全表 22 项");
        let keys: Vec<&str> = items
            .iter()
            .map(|i| i.key.as_str())
            .collect();
        assert!(keys.contains(&r"\xhh"));
        assert!(keys.contains(&r"\uhhhh"));
        assert_eq!(keys[0], ".");
    }

    #[test]
    fn boundary_kinds_map_to_the_right_serialisation() {
        let mut tree = build("a");
        let id = tree.body[0].id;
        let mut id_gen = IdGen(100);
        for (kind, negate, want) in [
            (BoundaryKind::Beginning, false, "^"),
            (BoundaryKind::End, false, "$"),
            (BoundaryKind::Word, false, r"\b"),
            (BoundaryKind::Word, true, r"\B"),
        ] {
            tree.set_content(
                id,
                &ContentSpec::from_node(&super::ENode {
                    id,
                    quantifier: None,
                    kind: EKind::Boundary { kind, negate },
                })
                .unwrap(),
                &mut id_gen,
            );
            assert_eq!(tree.to_pattern(), want);
        }
    }
}

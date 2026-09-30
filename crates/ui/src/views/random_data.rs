use crate::design;
use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{Input, InputState, Textarea, TextareaState},
    *,
};
use rand::Rng;
use time::{Date, Month};

const TABS: [&str; 5] = ["JSON", "XML", "CSV / TSV", "正则随机数据", "文本随机排序"];

const SURNAMES: &str = "赵钱孙李周吴郑王冯陈褚卫蒋沈韩杨朱秦许何吕张孔曹严华金魏陶姜戚谢邹喻柏水窦章云苏潘葛奚范彭郎鲁韦昌马苗凤花方俞任袁柳酆鲍史唐费廉岑薛雷贺倪汤滕殷罗毕郝邬安常乐于时傅皮卞齐康伍余元卜顾孟平黄和穆萧尹姚邵湛汪祁毛禹狄米贝明臧计伏成戴谈宋茅庞熊纪舒屈项祝董梁杜阮蓝闵席季麻强贾路娄危江童颜郭梅盛林刁钟徐邱骆高夏蔡田樊胡凌霍虞万支柯昝管卢莫经房裘缪干解应宗丁宣贲邓郁单杭洪包诸左石崔吉钮龚程嵇邢滑裴陆荣翁荀羊於惠甄麹家封芮羿储靳汲邴糜松井段富巫乌焦巴弓牧隗山谷车侯宓蓬全郗班仰秋仲伊宫宁仇栾暴甘钭厉戎祖武符刘景詹束龙叶幸司韶郜黎蓟薄印宿白怀蒲邰从鄂索咸籍赖卓蔺屠蒙池乔阴欎胥能苍双闻莘党翟谭贡劳逄姬申扶堵冉宰郦雍舄璩桑桂濮牛寿通边扈燕冀郏浦尚农温别庄晏柴瞿阎充慕连茹习宦艾鱼容向古易慎戈廖庾终暨居衡步都耿满弘匡国文寇广禄阙东欧殳沃利蔚越夔隆师巩厍聂晁勾敖融冷訾辛阚那简饶空曾毋沙乜养鞠须丰巢关蒯相查后荆红游竺权逯盖益桓公";
const GIVEN_CHARS: &str = "伟刚勇毅俊峰强军平保东文辉力明永健世广志义兴良海山仁波宁贵福生龙元全国胜学祥才发武新利清飞彬富顺信子杰涛昌成康星光天达安岩中茂进林有坚和彪博诚先敬震振壮会思群豪心邦承乐绍功松善厚庆磊民友裕河哲江超浩亮政谦亨奇固之轮翰朗伯宏言若鸣朋斌梁栋维启克伦翔旭鹏泽晨辰士以建家致树炎德行时泰盛雄琛钧冠策腾楠榕风航弘";
const EMAIL_DOMAINS: [&str; 7] = [
    "example.com",
    "mail.com",
    "test.org",
    "demo.net",
    "qq.com",
    "163.com",
    "gmail.com",
];
const PHONE_SECONDS: [&str; 5] = ["3", "5", "7", "8", "9"];
const WORD_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const EMAIL_USER_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789";
const WS_CHARS: &str = " \t\n\r\u{c}\u{b}";
const PRINTABLE: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~ \t";
const ANY_CHARS: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const MAX_REGEN_LEN: usize = 20000;

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('\t') || s.contains('\r') || s.contains('\n') || s.contains('"')
    {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn rand_char(s: &str, rng: &mut impl Rng) -> char {
    let len = s.chars().count();
    s.chars().nth(rng.random_range(0..len)).unwrap_or('a')
}

// ---------------------------------------------------------------------------
// 假数据字段推断（对齐 Vue fieldValue）
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Str,
    Num,
    Bool,
}

fn chinese_name(rng: &mut impl Rng) -> String {
    let given_len = if rng.random::<bool>() { 1 } else { 2 };
    let mut name = String::new();
    name.push(rand_char(SURNAMES, rng));
    for _ in 0..given_len {
        name.push(rand_char(GIVEN_CHARS, rng));
    }
    name
}

fn random_email(rng: &mut impl Rng) -> String {
    let user_len = rng.random_range(5..=10usize);
    let user: String = (0..user_len).map(|_| rand_char(EMAIL_USER_CHARS, rng)).collect();
    let domain = EMAIL_DOMAINS[rng.random_range(0..EMAIL_DOMAINS.len())];
    format!("{}@{}", user, domain)
}

fn random_phone(rng: &mut impl Rng) -> String {
    let mut s = String::from("1");
    s.push_str(PHONE_SECONDS[rng.random_range(0..PHONE_SECONDS.len())]);
    for _ in 0..9 {
        s.push_str(&rng.random_range(0..=9i64).to_string());
    }
    s
}

fn random_date(rng: &mut impl Rng) -> String {
    let lo = Date::from_calendar_date(1970, Month::January, 1)
        .unwrap()
        .to_julian_day();
    let hi = Date::from_calendar_date(2030, Month::December, 31)
        .unwrap()
        .to_julian_day();
    let d = Date::from_julian_day(rng.random_range(lo..=hi))
        .unwrap_or(Date::from_calendar_date(1970, Month::January, 1).unwrap());
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

fn random_word(rng: &mut impl Rng) -> String {
    let len = rng.random_range(6..=12usize);
    (0..len).map(|_| rand_char(WORD_CHARS, rng)).collect()
}

fn field_value(name: &str, rng: &mut impl Rng) -> (String, Kind) {
    match name.trim().to_lowercase().as_str() {
        "name" | "username" | "author" => (chinese_name(rng), Kind::Str),
        "age" => (rng.random_range(18..=80i64).to_string(), Kind::Num),
        "email" | "mail" => (random_email(rng), Kind::Str),
        "phone" | "tel" | "mobile" => (random_phone(rng), Kind::Str),
        "active" | "enabled" | "isactive" | "status" => (
            if rng.random::<bool>() {
                "true".to_string()
            } else {
                "false".to_string()
            },
            Kind::Bool,
        ),
        "joined" | "date" | "created" | "birthday" | "time" => (random_date(rng), Kind::Str),
        "id" | "no" | "num" | "count" => (rng.random_range(1..=99999i64).to_string(), Kind::Num),
        _ => (random_word(rng), Kind::Str),
    }
}

fn parse_fields(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

type Row = Vec<(String, String, Kind)>;

fn build_row(fields: &[String], rng: &mut impl Rng) -> Row {
    fields
        .iter()
        .map(|f| {
            let (value, kind) = field_value(f, rng);
            (f.clone(), value, kind)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 正则随机数据（对齐 Vue RegexParser）
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum ClassPart {
    Set(String),
    Range(char, char),
}

#[derive(Clone)]
enum Node {
    Lit(char),
    Any,
    Class { negated: bool, parts: Vec<ClassPart> },
    Group(Vec<Vec<Node>>),
    Quant { node: Box<Node>, min: usize, max: Option<usize> },
}

fn not_of(set: &str) -> String {
    PRINTABLE.chars().filter(|c| !set.contains(*c)).collect()
}

fn range_chars(from: char, to: char) -> String {
    let (a, b) = if from <= to { (from, to) } else { (to, from) };
    (a as u32..=b as u32)
        .filter_map(char::from_u32)
        .collect()
}

fn class_chars(negated: bool, parts: &[ClassPart]) -> String {
    let mut s = String::new();
    for p in parts {
        match p {
            ClassPart::Set(chars) => s.push_str(chars),
            ClassPart::Range(from, to) => s.push_str(&range_chars(*from, *to)),
        }
    }
    let mut seen = std::collections::HashSet::new();
    let deduped: String = s.chars().filter(|c| seen.insert(*c)).collect();
    if negated {
        not_of(&deduped)
    } else {
        deduped
    }
}

struct RegexParser {
    src: Vec<char>,
    pos: usize,
}

impl RegexParser {
    fn new(src: &str) -> Self {
        Self {
            src: src.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    /// 判断 self.pos 处的 "{...}" 是否为量词（不消费）
    fn is_quant_ahead(&self) -> bool {
        if self.peek() != Some('{') {
            return false;
        }
        let mut i = self.pos + 1;
        let mut digits = 0;
        while i < self.src.len() && self.src[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
        if digits == 0 {
            return false;
        }
        if i < self.src.len() && self.src[i] == '}' {
            return true;
        }
        if i < self.src.len() && self.src[i] == ',' {
            i += 1;
            while i < self.src.len() && self.src[i].is_ascii_digit() {
                i += 1;
            }
            return i < self.src.len() && self.src[i] == '}';
        }
        false
    }

    /// 消费 self.pos 处的量词
    fn parse_quant(&mut self) -> Option<(usize, Option<usize>)> {
        match self.peek() {
            Some('*') => {
                self.pos += 1;
                Some((0, None))
            }
            Some('+') => {
                self.pos += 1;
                Some((1, None))
            }
            Some('?') => {
                self.pos += 1;
                Some((0, Some(1)))
            }
            Some('{') => {
                let mut i = self.pos + 1;
                let mut min_str = String::new();
                while i < self.src.len() && self.src[i].is_ascii_digit() {
                    min_str.push(self.src[i]);
                    i += 1;
                }
                if min_str.is_empty() {
                    return None;
                }
                let min: usize = min_str.parse().ok()?;
                if i < self.src.len() && self.src[i] == '}' {
                    self.pos = i + 1;
                    return Some((min, Some(min)));
                }
                if i < self.src.len() && self.src[i] == ',' {
                    i += 1;
                    let mut max_str = String::new();
                    while i < self.src.len() && self.src[i].is_ascii_digit() {
                        max_str.push(self.src[i]);
                        i += 1;
                    }
                    if i < self.src.len() && self.src[i] == '}' {
                        self.pos = i + 1;
                        if max_str.is_empty() {
                            return Some((min, None));
                        }
                        let max: usize = max_str.parse().ok()?;
                        if max < min {
                            return None;
                        }
                        return Some((min, Some(max)));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn parse(&mut self) -> Result<Vec<Vec<Node>>, String> {
        let alts = self.parse_alts()?;
        if self.pos < self.src.len() {
            return Err("invalid pattern".to_string());
        }
        Ok(alts)
    }

    fn parse_alts(&mut self) -> Result<Vec<Vec<Node>>, String> {
        let mut alts = vec![self.parse_seq()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            alts.push(self.parse_seq()?);
        }
        Ok(alts)
    }

    fn parse_seq(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            nodes.push(self.parse_atom()?);
        }
        Ok(nodes)
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        let c = self.peek().ok_or("unexpected end")?;
        let node = match c {
            '(' => {
                self.pos += 1;
                if self.peek() == Some('?') {
                    if self.src.get(self.pos + 1) == Some(&':') {
                        self.pos += 2;
                    } else {
                        return Err("unsupported lookahead".to_string());
                    }
                }
                let alts = self.parse_alts()?;
                if self.peek() != Some(')') {
                    return Err("unbalanced paren".to_string());
                }
                self.pos += 1;
                Node::Group(alts)
            }
            '[' => self.parse_class()?,
            '\\' => self.parse_escape()?,
            '.' => {
                self.pos += 1;
                Node::Any
            }
            '^' | '$' => return Err("anchors unsupported".to_string()),
            '*' | '+' | '?' => return Err("lone quantifier".to_string()),
            '{' => {
                if self.is_quant_ahead() {
                    return Err("lone quantifier".to_string());
                }
                self.pos += 1;
                Node::Lit('{')
            }
            _ => {
                self.pos += 1;
                Node::Lit(c)
            }
        };
        if let Some((min, max)) = self.parse_quant() {
            return Ok(Node::Quant {
                node: Box::new(node),
                min,
                max,
            });
        }
        Ok(node)
    }

    fn parse_escape(&mut self) -> Result<Node, String> {
        self.pos += 1;
        let c = self.src.get(self.pos).copied().ok_or("trailing backslash")?;
        self.pos += 1;
        Ok(match c {
            'd' => Node::Class {
                negated: false,
                parts: vec![ClassPart::Set("0123456789".to_string())],
            },
            'w' => Node::Class {
                negated: false,
                parts: vec![ClassPart::Set(format!("{}_", WORD_CHARS))],
            },
            's' => Node::Class {
                negated: false,
                parts: vec![ClassPart::Set(WS_CHARS.to_string())],
            },
            'D' => Node::Class {
                negated: true,
                parts: vec![ClassPart::Set("0123456789".to_string())],
            },
            'W' => Node::Class {
                negated: true,
                parts: vec![ClassPart::Set(format!("{}_", WORD_CHARS))],
            },
            'S' => Node::Class {
                negated: true,
                parts: vec![ClassPart::Set(WS_CHARS.to_string())],
            },
            'n' => Node::Lit('\n'),
            't' => Node::Lit('\t'),
            'r' => Node::Lit('\r'),
            c => Node::Lit(c),
        })
    }

    fn parse_class(&mut self) -> Result<Node, String> {
        self.pos += 1; // [
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.pos += 1;
        }
        let mut parts: Vec<ClassPart> = Vec::new();
        let mut chars: Vec<char> = Vec::new();
        while self.pos < self.src.len() && self.src[self.pos] != ']' {
            let c = self.src[self.pos];
            if c == '\\' {
                self.pos += 1;
                let e = self.src.get(self.pos).copied().ok_or("bad class escape")?;
                self.pos += 1;
                if !chars.is_empty() {
                    parts.push(ClassPart::Set(chars.iter().collect()));
                    chars.clear();
                }
                match e {
                    'd' => parts.push(ClassPart::Set("0123456789".to_string())),
                    'w' => parts.push(ClassPart::Set(format!("{}_", WORD_CHARS))),
                    's' => parts.push(ClassPart::Set(WS_CHARS.to_string())),
                    'D' => parts.push(ClassPart::Set(not_of("0123456789"))),
                    'W' => parts.push(ClassPart::Set(not_of(&format!("{}_", WORD_CHARS)))),
                    'S' => parts.push(ClassPart::Set(not_of(WS_CHARS))),
                    'n' => parts.push(ClassPart::Set("\n".to_string())),
                    't' => parts.push(ClassPart::Set("\t".to_string())),
                    'r' => parts.push(ClassPart::Set("\r".to_string())),
                    e => chars.push(e),
                }
            } else if c == '-'
                && !chars.is_empty()
                && self.pos + 1 < self.src.len()
                && self.src[self.pos + 1] != ']'
            {
                let from = chars.pop().unwrap();
                self.pos += 1;
                let to = self.src[self.pos];
                self.pos += 1;
                parts.push(ClassPart::Range(from, to));
            } else {
                chars.push(c);
                self.pos += 1;
            }
        }
        if self.pos >= self.src.len() {
            return Err("unbalanced bracket".to_string());
        }
        self.pos += 1; // ]
        if !chars.is_empty() {
            parts.push(ClassPart::Set(chars.iter().collect()));
        }
        Ok(Node::Class { negated, parts })
    }
}

fn quant_count(min: usize, max: Option<usize>, rng: &mut impl Rng) -> usize {
    let lo = min.min(10000);
    let hi = match max {
        None => (lo + 10).min(10000),
        Some(m) => m.min(10000),
    };
    let hi = hi.max(lo);
    rng.random_range(lo..=hi)
}

fn gen_from_node(node: &Node, out: &mut String, left: &mut usize, rng: &mut impl Rng) {
    if *left == 0 {
        return;
    }
    match node {
        Node::Quant { node, min, max } => {
            let count = quant_count(*min, *max, rng);
            for _ in 0..count {
                if *left == 0 {
                    return;
                }
                gen_from_node(node, out, left, rng);
            }
        }
        Node::Lit(c) => {
            out.push(*c);
            *left -= 1;
        }
        Node::Any => {
            out.push(rand_char(ANY_CHARS, rng));
            *left -= 1;
        }
        Node::Class { negated, parts } => {
            let cs = class_chars(*negated, parts);
            if !cs.is_empty() {
                out.push(rand_char(&cs, rng));
                *left -= 1;
            }
        }
        Node::Group(alts) => {
            if alts.is_empty() {
                return;
            }
            let alt = &alts[rng.random_range(0..alts.len())];
            for sub in alt {
                gen_from_node(sub, out, left, rng);
                if *left == 0 {
                    return;
                }
            }
        }
    }
}

fn generate_regex_line(alts: &[Vec<Node>], rng: &mut impl Rng) -> String {
    let mut out = String::new();
    let mut left = MAX_REGEN_LEN;
    let alt = &alts[rng.random_range(0..alts.len())];
    for node in alt {
        gen_from_node(node, &mut out, &mut left, rng);
        if left == 0 {
            break;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 视图
// ---------------------------------------------------------------------------

/// 配置项：12px 灰色 label + 控件（对应 tb-config-item）
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

pub struct RandomDataGenerator {
    mode: usize,

    json_fields_state: Entity<InputState>,
    json_records_state: Entity<InputState>,
    json_out_state: Entity<TextareaState>,

    xml_root_state: Entity<InputState>,
    xml_fields_state: Entity<InputState>,
    xml_records_state: Entity<InputState>,
    xml_out_state: Entity<TextareaState>,

    csv_separator_comma: bool,
    csv_columns_state: Entity<InputState>,
    csv_rows_state: Entity<InputState>,
    csv_out_state: Entity<TextareaState>,

    regex_pattern_state: Entity<InputState>,
    regex_count_state: Entity<InputState>,
    regex_out_state: Entity<TextareaState>,

    shuffle_input_state: Entity<TextareaState>,
    shuffle_out_state: Entity<TextareaState>,

    status: String,
}

impl RandomDataGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |window: &mut Window, cx: &mut Context<Self>, default: &str| {
            cx.new(|cx| InputState::new(window, cx).default_value(default.to_string()))
        };
        let input_ph = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        let textarea = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder))
        };

        Self {
            mode: 0,
            json_fields_state: input(
                window,
                cx,
                "name,age,email,phone,active,joined",
            ),
            json_records_state: input(window, cx, "5"),
            json_out_state: textarea(window, cx, "生成的 JSON 数组"),
            xml_root_state: input(window, cx, "items"),
            xml_fields_state: input(
                window,
                cx,
                "name,age,email,phone,active,joined",
            ),
            xml_records_state: input(window, cx, "5"),
            xml_out_state: textarea(window, cx, "生成的 XML 文档"),
            csv_separator_comma: true,
            csv_columns_state: input(window, cx, "name,age,email"),
            csv_rows_state: input(window, cx, "5"),
            csv_out_state: textarea(window, cx, "生成的数据（首行为表头）"),
            regex_pattern_state: input_ph(window, cx, "如 [a-z]{5}\\d{2}"),
            regex_count_state: input(window, cx, "10"),
            regex_out_state: textarea(window, cx, "匹配正则的随机字符串（每行一个）"),
            shuffle_input_state: textarea(
                window,
                cx,
                "输入多行文本，每行将被随机打乱顺序",
            ),
            shuffle_out_state: textarea(window, cx, "打乱行序后的文本"),
            status: String::new(),
        }
    }

    fn read_usize(&self, state: &Entity<InputState>, cx: &App, default: usize, max: usize) -> usize {
        state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(default)
            .clamp(1, max)
    }

    fn read_str(&self, state: &Entity<InputState>, cx: &App) -> String {
        state.read(cx).value().trim().to_string()
    }

    fn set_output(&self, state: &Entity<TextareaState>, text: String, window: &mut Window, cx: &mut Context<Self>) {
        state.update(cx, |s, cx| {
            s.set_value(text, window, cx);
        });
    }

    fn copy_output(&mut self, state: &Entity<TextareaState>, cx: &mut Context<Self>) {
        let text = state.read(cx).value().to_string();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn clear_output(&mut self, state: &Entity<TextareaState>, window: &mut Window, cx: &mut Context<Self>) {
        state.update(cx, |s, cx| {
            s.set_value("".to_string(), window, cx);
        });
    }

    fn gen_json(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let fields = parse_fields(&self.read_str(&self.json_fields_state, cx));
        if fields.is_empty() {
            self.status = "请填写至少一个字段".to_string();
            cx.notify();
            return;
        }
        let records = self.read_usize(&self.json_records_state, cx, 5, 10000);
        let mut rng = rand::rng();
        let mut text = String::from("[\n");
        for i in 0..records {
            let row = build_row(&fields, &mut rng);
            text.push_str("  {\n");
            for (j, (name, value, kind)) in row.iter().enumerate() {
                let comma = if j + 1 < row.len() { "," } else { "" };
                match kind {
                    Kind::Num => text.push_str(&format!("    \"{}\": {}{}\n", name, value, comma)),
                    Kind::Bool => {
                        text.push_str(&format!("    \"{}\": {}{}\n", name, value, comma))
                    }
                    Kind::Str => text.push_str(&format!(
                        "    \"{}\": \"{}\"{}\n",
                        name,
                        json_escape(value),
                        comma
                    )),
                }
            }
            text.push_str("  }");
            text.push_str(if i + 1 < records { ",\n" } else { "\n" });
        }
        text.push(']');
        self.set_output(&self.json_out_state, text, window, cx);
        cx.notify();
    }

    fn gen_xml(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let root = {
            let r = self.read_str(&self.xml_root_state, cx);
            if r.is_empty() {
                "items".to_string()
            } else {
                r
            }
        };
        let fields = parse_fields(&self.read_str(&self.xml_fields_state, cx));
        if fields.is_empty() {
            self.status = "请填写至少一个字段".to_string();
            cx.notify();
            return;
        }
        let records = self.read_usize(&self.xml_records_state, cx, 5, 10000);
        let mut rng = rand::rng();
        let mut item_xml: Vec<String> = Vec::with_capacity(records);
        for _ in 0..records {
            let row = build_row(&fields, &mut rng);
            let inner: Vec<String> = row
                .iter()
                .map(|(f, v, _)| format!("    <{}>{}</{}>", f, xml_escape(v), f))
                .collect();
            item_xml.push(format!("  <item>\n{}\n  </item>", inner.join("\n")));
        }
        let text = format!("<{}>\n{}\n</{}>", root, item_xml.join("\n"), root);
        self.set_output(&self.xml_out_state, text, window, cx);
        cx.notify();
    }

    fn gen_csv(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let sep = if self.csv_separator_comma { ',' } else { '\t' };
        let cols = parse_fields(&self.read_str(&self.csv_columns_state, cx));
        if cols.is_empty() {
            self.status = "请填写至少一个列名".to_string();
            cx.notify();
            return;
        }
        let rows = self.read_usize(&self.csv_rows_state, cx, 5, 10000);
        let mut rng = rand::rng();
        let sep_s = sep.to_string();
        let header: Vec<String> = cols.iter().map(|c| csv_escape(c)).collect();
        let mut lines = vec![header.join(&sep_s)];
        for _ in 0..rows {
            let cells: Vec<String> = cols
                .iter()
                .map(|c| {
                    let (v, _) = field_value(c, &mut rng);
                    csv_escape(&v)
                })
                .collect();
            lines.push(cells.join(&sep_s));
        }
        self.set_output(&self.csv_out_state, lines.join("\n"), window, cx);
        cx.notify();
    }

    fn gen_regex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let pattern = self.read_str(&self.regex_pattern_state, cx);
        if pattern.is_empty() {
            self.status = "请输入正则表达式".to_string();
            cx.notify();
            return;
        }
        let alts = match RegexParser::new(&pattern).parse() {
            Ok(alts) => alts,
            Err(_) => {
                self.status = "暂不支持该正则".to_string();
                cx.notify();
                return;
            }
        };
        let count = self.read_usize(&self.regex_count_state, cx, 10, 10000);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| generate_regex_line(&alts, &mut rng))
            .collect();
        self.set_output(&self.regex_out_state, lines.join("\n"), window, cx);
        cx.notify();
    }

    fn shuffle_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let input = self.shuffle_input_state.read(cx).value().to_string();
        if input.trim().is_empty() {
            self.status = "请输入要打乱的文本".to_string();
            cx.notify();
            return;
        }
        let mut lines: Vec<String> = input.split('\n').map(|s| s.to_string()).collect();
        let mut rng = rand::rng();
        for i in (1..lines.len()).rev() {
            let j = rng.random_range(0..=i);
            lines.swap(i, j);
        }
        self.set_output(&self.shuffle_out_state, lines.join("\n"), window, cx);
        cx.notify();
    }

    // ------------------------------------------------------------------
    // 渲染
    // ------------------------------------------------------------------

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
                this.status.clear();
                cx.notify();
            }))
            .child(label.to_string())
    }

    fn output_editor(
        &self,
        copy_id: &'static str,
        clear_id: &'static str,
        state: &Entity<TextareaState>,
        h: f32,
        cx: &mut Context<Self>,
    ) -> Div {
        let state_copy = state.clone();
        let state_clear = state.clone();
        div()
            .flex_col()
            .gap_1p5()
            .child(design::editor_label("输出", cx))
            .child(Textarea::new(state).h(px(h)).font_family("monospace"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mt_2()
                    .child(
                        Button::new(copy_id)
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制输出")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.copy_output(&state_copy, cx);
                            })),
                    )
                    .child(
                        Button::new(clear_id)
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.clear_output(&state_clear, window, cx);
                            })),
                    ),
            )
    }

    fn gen_button(&self, id: &'static str) -> Button {
        Button::new(id)
            .primary()
            .icon(Icon::new(IconName::Play))
            .tooltip("生成")
    }

    fn render_tab(&mut self, cx: &mut Context<Self>) -> Div {
        let status = self.status.clone();
        let status_line = || {
            div()
                .text_size(px(12.5))
                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                .child(status.clone())
        };

        match self.mode {
            0 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(config_item(
                            "字段",
                            div()
                                .w(px(380.0))
                                .child(Input::new(&self.json_fields_state)),
                        ))
                        .child(config_item(
                            "记录数",
                            div().w(px(120.0)).child(Input::new(&self.json_records_state)),
                        ))
                        .child(
                            self.gen_button("gen-json").on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.gen_json(window, cx);
                                },
                            )),
                        ),
                )
                .when(!status.is_empty(), |this| this.child(status_line()))
                .child(self.output_editor("copy-json", "clear-json", &self.json_out_state, 240.0, cx)),
            1 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(config_item(
                            "根元素",
                            div().w(px(140.0)).child(Input::new(&self.xml_root_state)),
                        ))
                        .child(config_item(
                            "字段",
                            div()
                                .w(px(340.0))
                                .child(Input::new(&self.xml_fields_state)),
                        ))
                        .child(config_item(
                            "记录数",
                            div().w(px(120.0)).child(Input::new(&self.xml_records_state)),
                        ))
                        .child(
                            self.gen_button("gen-xml").on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.gen_xml(window, cx);
                                },
                            )),
                        ),
                )
                .when(!status.is_empty(), |this| this.child(status_line()))
                .child(self.output_editor("copy-xml", "clear-xml", &self.xml_out_state, 240.0, cx)),
            2 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            // n-radio-button 组：逗号 / 制表符
                            div()
                                .flex()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(cx.theme().border)
                                .overflow_hidden()
                                .child(
                                    div()
                                        .id("csv-sep-comma")
                                        .px_3()
                                        .py_1()
                                        .text_sm()
                                        .bg(if self.csv_separator_comma {
                                            cx.theme().primary
                                        } else {
                                            gpui::black().opacity(0.0)
                                        })
                                        .text_color(if self.csv_separator_comma {
                                            gpui::white()
                                        } else {
                                            cx.theme().muted_foreground
                                        })
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.csv_separator_comma = true;
                                            cx.notify();
                                        }))
                                        .child("逗号分隔"),
                                )
                                .child(
                                    div()
                                        .id("csv-sep-tab")
                                        .px_3()
                                        .py_1()
                                        .text_sm()
                                        .bg(if !self.csv_separator_comma {
                                            cx.theme().primary
                                        } else {
                                            gpui::black().opacity(0.0)
                                        })
                                        .text_color(if !self.csv_separator_comma {
                                            gpui::white()
                                        } else {
                                            cx.theme().muted_foreground
                                        })
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.csv_separator_comma = false;
                                            cx.notify();
                                        }))
                                        .child("制表符分隔"),
                                ),
                        )
                        .child(config_item(
                            "列名",
                            div()
                                .w(px(240.0))
                                .child(Input::new(&self.csv_columns_state)),
                        ))
                        .child(config_item(
                            "行数",
                            div().w(px(120.0)).child(Input::new(&self.csv_rows_state)),
                        ))
                        .child(
                            self.gen_button("gen-csv").on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.gen_csv(window, cx);
                                },
                            )),
                        ),
                )
                .when(!status.is_empty(), |this| this.child(status_line()))
                .child(self.output_editor("copy-csv", "clear-csv", &self.csv_out_state, 240.0, cx)),
            3 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(config_item(
                            "正则",
                            div()
                                .w(px(300.0))
                                .font_family("monospace")
                                .child(Input::new(&self.regex_pattern_state)),
                        ))
                        .child(config_item(
                            "数量",
                            div().w(px(120.0)).child(Input::new(&self.regex_count_state)),
                        ))
                        .child(
                            self.gen_button("gen-regex").on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.gen_regex(window, cx);
                                },
                            )),
                        ),
                )
                .when(!status.is_empty(), |this| this.child(status_line()))
                .child(self.output_editor("copy-regex", "clear-regex", &self.regex_out_state, 240.0, cx)),
            _ => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("输入", cx))
                        .child(
                            Textarea::new(&self.shuffle_input_state)
                                .h(px(180.0))
                                .font_family("monospace"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new("shuffle-lines")
                                .primary()
                                .icon(Icon::new(IconName::RotateCw))
                                .tooltip("打乱")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.shuffle_lines(window, cx);
                                })),
                        ),
                )
                .when(!status.is_empty(), |this| this.child(status_line()))
                .child(self.output_editor("copy-shuffle", "clear-shuffle", &self.shuffle_out_state, 180.0, cx)),
        }
    }
}

impl Render for RandomDataGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header(
                "随机数据",
                "JSON / XML / CSV / 正则随机数据 / 文本随机排序",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::Inbox,
                        "随机数据",
                        "批量假数据与正则随机串生成",
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_6()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .mb_4()
                            .children(TABS.iter().enumerate().map(|(i, label)| {
                                self.tab_btn(
                                    match i {
                                        0 => "tab-json",
                                        1 => "tab-xml",
                                        2 => "tab-csv",
                                        3 => "tab-regex",
                                        _ => "tab-shuffle",
                                    },
                                    label,
                                    i,
                                    cx,
                                )
                            })),
                    )
                    .child(self.render_tab(cx)),
            )
    }
}

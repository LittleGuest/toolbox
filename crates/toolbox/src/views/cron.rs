use gpui_kit::{
    component::{
        button::*,
        input::{Input, InputEvent, InputState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

const DEFAULT_LINUX_EXPR: &str = "*/5 * * * *";
const DEFAULT_SPRING_EXPR: &str = "0 */5 * * * *";
const DEFAULT_QUARTZ_EXPR: &str = "0 */5 * * * * ?";

const LINUX_PRESETS: &[(&str, &str)] = &[
    ("每分钟", "* * * * *"),
    ("每 5 分钟", "*/5 * * * *"),
    ("每小时", "0 * * * *"),
    ("每天 08:00", "0 8 * * *"),
    ("每周一 09:00", "0 9 * * 1"),
    ("工作日 09:00", "0 9 * * 1-5"),
    ("每月 1 日 00:00", "0 0 1 * *"),
    ("每年 1 月 1 日", "0 0 1 1 *"),
    ("@daily", "@daily"),
    ("@hourly", "@hourly"),
];

const SPRING_PRESETS: &[(&str, &str)] = &[
    ("每秒", "* * * * * *"),
    ("每 10 秒", "*/10 * * * * *"),
    ("每分钟", "0 * * * * *"),
    ("每 5 分钟", "0 */5 * * * *"),
    ("每小时整点", "0 0 * * * *"),
    ("每天 08:00", "0 0 8 * * *"),
    ("每周一 09:00", "0 0 9 * * 1"),
    ("工作日 09:00", "0 0 9 * * 1-5"),
    ("每月 1 日", "0 0 0 1 * ?"),
    ("每月最后一天", "0 0 0 L * ?"),
    ("@daily", "@daily"),
];

const QUARTZ_PRESETS: &[(&str, &str)] = &[
    ("每秒", "* * * * * *"),
    ("每 10 秒", "*/10 * * * * *"),
    ("每分钟", "0 * * * * *"),
    ("每 5 分钟", "0 */5 * * * *"),
    ("每小时整点", "0 0 * * * *"),
    ("每天 08:00", "0 0 8 * * *"),
    ("每周一 09:00", "0 0 9 ? * 2"),
    ("每月 1 日", "0 0 0 1 * ?"),
    ("每月最后一天", "0 0 0 L * ?"),
    ("每月第三个周五", "0 0 9 ? * 6#3"),
    ("每年 2 月 29 日", "0 0 0 29 2 ?"),
];

const MODE_ANY: &str = "任意 (*)";
const MODE_INTERVAL: &str = "间隔 (*/N)";
const MODE_SPECIFIC: &str = "指定";
const MODE_YEAR_SPECIFIC: &str = "指定年份";
const MODE_DOM_SPECIAL: &str = "特殊 (L/W)";
const MODE_DOW_SPECIAL: &str = "特殊 (L/#)";

const DOM_SPECIAL_LABELS: &[&str] = &[
    "L · 每月最后一天",
    "LW · 每月最后一个工作日",
    "NW · 离 N 号最近的工作日",
    "L-N · 每月倒数第 N 天",
];

const DOW_SPECIAL_LABELS: &[&str] = &["NL · 每月最后一个周 N", "N#M · 每月第 M 个周 N"];

const DOW_NAMES: [&str; 7] = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Parse,
    Generate,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GenFieldKey {
    Sec = 0,
    Min = 1,
    Hour = 2,
    Dom = 3,
    Month = 4,
    Dow = 5,
    Year = 6,
}

const GEN_KEYS: [GenFieldKey; 7] = [
    GenFieldKey::Sec,
    GenFieldKey::Min,
    GenFieldKey::Hour,
    GenFieldKey::Dom,
    GenFieldKey::Month,
    GenFieldKey::Dow,
    GenFieldKey::Year,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Num,
    Dom,
    Dow,
    Year,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GenMode {
    Any,
    Interval,
    Specific,
    Special,
}

struct FieldGen {
    key: GenFieldKey,
    label: &'static str,
    kind: FieldKind,
    interval_default: u32,
    mode: GenMode,
    mode_state: Entity<SelectState<Vec<String>>>,
    interval: Option<u32>,
    interval_state: Option<Entity<InputState>>,
    values_text: String,
    values_state: Option<Entity<InputState>>,
    special: &'static str,
    special_state: Option<Entity<SelectState<Vec<String>>>>,
    special_dow: u32,
    special_dow_state: Option<Entity<SelectState<Vec<String>>>>,
    w_day: Option<u32>,
    w_day_state: Option<Entity<InputState>>,
    offset: Option<u32>,
    offset_state: Option<Entity<InputState>>,
    nth: Option<u32>,
    nth_state: Option<Entity<InputState>>,
    year_text: String,
    year_state: Option<Entity<InputState>>,
}

fn mode_from_label(label: &str) -> GenMode {
    if label.starts_with("任意") {
        GenMode::Any
    } else if label.starts_with("间隔") {
        GenMode::Interval
    } else if label.starts_with("特殊") {
        GenMode::Special
    } else {
        GenMode::Specific
    }
}

fn mode_label(mode: GenMode, kind: FieldKind) -> &'static str {
    match mode {
        GenMode::Any => MODE_ANY,
        GenMode::Interval => MODE_INTERVAL,
        GenMode::Specific => {
            if kind == FieldKind::Year {
                MODE_YEAR_SPECIFIC
            } else {
                MODE_SPECIFIC
            }
        }
        GenMode::Special => {
            if kind == FieldKind::Dow {
                MODE_DOW_SPECIAL
            } else {
                MODE_DOM_SPECIAL
            }
        }
    }
}

fn mode_items(kind: FieldKind, cron_type: &str) -> Vec<String> {
    let base = vec![
        MODE_ANY.to_string(),
        MODE_INTERVAL.to_string(),
        MODE_SPECIFIC.to_string(),
    ];
    match kind {
        FieldKind::Num => base,
        FieldKind::Dom => {
            if cron_type == "linux" {
                base
            } else {
                let mut items = base;
                items.push(MODE_DOM_SPECIAL.to_string());
                items
            }
        }
        FieldKind::Dow => {
            if cron_type == "linux" {
                vec![MODE_ANY.to_string(), MODE_SPECIFIC.to_string()]
            } else {
                vec![
                    MODE_ANY.to_string(),
                    MODE_SPECIFIC.to_string(),
                    MODE_DOW_SPECIAL.to_string(),
                ]
            }
        }
        FieldKind::Year => vec![MODE_ANY.to_string(), MODE_YEAR_SPECIFIC.to_string()],
    }
}

fn dow_items(cron_type: &str) -> Vec<String> {
    if cron_type == "quartz" {
        (0..7)
            .map(|i| format!("{}({})", DOW_NAMES[i], i + 1))
            .collect()
    } else {
        let mut items: Vec<String> = (0..7).map(|i| format!("{}({})", DOW_NAMES[i], i)).collect();
        items.push("周日(7)".to_string());
        items
    }
}

fn parse_dow_number(label: &str) -> Option<u32> {
    let start = label.find('(')? + 1;
    let end = label[start..].find(')')? + start;
    label[start..end].parse::<u32>().ok()
}

fn dom_special_from_label(label: &str) -> &'static str {
    if label.starts_with("LW") {
        "LW"
    } else if label.starts_with("NW") {
        "W"
    } else if label.starts_with("L-N") {
        "L-n"
    } else {
        "L"
    }
}

fn dow_special_from_label(label: &str) -> &'static str {
    if label.starts_with("N#M") { "#" } else { "L" }
}

fn parse_values(text: &str, min: u32, max: u32) -> Vec<u32> {
    let mut values: Vec<u32> = text
        .split([',', '，', ' ', '\t'])
        .filter_map(|part| part.trim().parse::<u32>().ok())
        .filter(|&v| v >= min && v <= max)
        .collect();
    values.sort_unstable();
    values.dedup();
    values
}

fn compose_num(f: &FieldGen, min: u32, max: u32) -> String {
    match f.mode {
        GenMode::Any | GenMode::Special => "*".to_string(),
        GenMode::Interval => match f.interval {
            Some(n) if n >= 1 => format!("*/{}", n.clamp(1, max)),
            _ => "*".to_string(),
        },
        GenMode::Specific => {
            let values = parse_values(&f.values_text, min, max);
            if values.is_empty() {
                "*".to_string()
            } else {
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }
        }
    }
}

fn compose_dom(f: &FieldGen) -> String {
    if f.mode == GenMode::Special {
        return match f.special {
            "LW" => "LW".to_string(),
            "W" => format!("{}W", f.w_day.unwrap_or(15).clamp(1, 31)),
            "L-n" => format!("L-{}", f.offset.unwrap_or(3).clamp(1, 30)),
            _ => "L".to_string(),
        };
    }
    compose_num(f, 1, 31)
}

fn compose_dow(f: &FieldGen, dow_min: u32) -> String {
    if f.mode == GenMode::Special {
        let dow = f.special_dow.clamp(1, 7);
        return match f.special {
            "#" => format!("{}#{}", dow, f.nth.unwrap_or(3).clamp(1, 5)),
            _ => format!("{}L", dow),
        };
    }
    compose_num(f, dow_min, 7)
}

impl FieldGen {
    fn new(key: GenFieldKey, window: &mut Window, cx: &mut Context<CronConverter>) -> Self {
        let cron_type = "linux";
        let (label, kind, interval_default, values_placeholder) = match key {
            GenFieldKey::Sec => ("秒", FieldKind::Num, 10u32, "如 0,15,30"),
            GenFieldKey::Min => ("分", FieldKind::Num, 5, "如 0,15,30"),
            GenFieldKey::Hour => ("时", FieldKind::Num, 2, "如 0,8,22"),
            GenFieldKey::Dom => ("日", FieldKind::Dom, 5, "如 1,15,28（1-31）"),
            GenFieldKey::Month => ("月", FieldKind::Num, 2, "如 1,3,12（1-12）"),
            GenFieldKey::Dow => ("周", FieldKind::Dow, 0, "如 0,1,5"),
            GenFieldKey::Year => ("年", FieldKind::Year, 0, "如 2024-2026 / 2024,2026"),
        };

        let mode_state = cx.new(|cx| {
            let mut state = SelectState::new(mode_items(kind, cron_type), None, window, cx);
            state.set_selected_value(&MODE_ANY.to_string(), window, cx);
            state
        });

        let interval_state = if matches!(kind, FieldKind::Num | FieldKind::Dom) {
            let default_text = interval_default.to_string();
            Some(cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder("N");
                state.set_value(default_text, window, cx);
                state
            }))
        } else {
            None
        };

        let values_state = if kind != FieldKind::Year {
            let placeholder = values_placeholder;
            Some(cx.new(|cx| InputState::new(window, cx).placeholder(placeholder)))
        } else {
            None
        };

        let special_state = if matches!(kind, FieldKind::Dom | FieldKind::Dow) {
            let items: Vec<String> = if kind == FieldKind::Dom {
                DOM_SPECIAL_LABELS.iter().map(|s| s.to_string()).collect()
            } else {
                DOW_SPECIAL_LABELS.iter().map(|s| s.to_string()).collect()
            };
            let selected = if kind == FieldKind::Dom {
                DOM_SPECIAL_LABELS[0]
            } else {
                DOW_SPECIAL_LABELS[0]
            };
            Some(cx.new(|cx| {
                let mut state = SelectState::new(items, None, window, cx);
                state.set_selected_value(&selected.to_string(), window, cx);
                state
            }))
        } else {
            None
        };

        let special_dow_state = if kind == FieldKind::Dow {
            Some(cx.new(|cx| {
                let mut state = SelectState::new(dow_items(cron_type), None, window, cx);
                state.set_selected_value(&"周一(1)".to_string(), window, cx);
                state
            }))
        } else {
            None
        };

        let w_day_state = if kind == FieldKind::Dom {
            Some(cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder("1-31");
                state.set_value("15".to_string(), window, cx);
                state
            }))
        } else {
            None
        };

        let offset_state = if kind == FieldKind::Dom {
            Some(cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder("1-30");
                state.set_value("3".to_string(), window, cx);
                state
            }))
        } else {
            None
        };

        let nth_state = if kind == FieldKind::Dow {
            Some(cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder("1-5");
                state.set_value("3".to_string(), window, cx);
                state
            }))
        } else {
            None
        };

        let year_state = if kind == FieldKind::Year {
            Some(cx.new(|cx| InputState::new(window, cx).placeholder("如 2024-2026 / 2024,2026")))
        } else {
            None
        };

        Self {
            key,
            label,
            kind,
            interval_default,
            mode: GenMode::Any,
            mode_state,
            interval: if interval_default > 0 {
                Some(interval_default)
            } else {
                None
            },
            interval_state,
            values_text: String::new(),
            values_state,
            special: "L",
            special_state,
            special_dow: 1,
            special_dow_state,
            w_day: Some(15),
            w_day_state,
            offset: Some(3),
            offset_state,
            nth: Some(3),
            nth_state,
            year_text: String::new(),
            year_state,
        }
    }
}

pub struct CronConverter {
    expr: String,
    cron_type: String,
    active_tab: Tab,
    result: Option<::base::CronParseResult>,
    error: String,
    expr_state: Entity<InputState>,
    gen_fields: Vec<FieldGen>,
    _subscriptions: Vec<Subscription>,
}

impl CronConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let expr_state = cx.new(|cx| {
            let mut state =
                InputState::new(window, cx).placeholder("如 */5 * * * *（分 时 日 月 周）");
            state.set_value(DEFAULT_LINUX_EXPR.to_string(), window, cx);
            state
        });

        let mut gen_fields = Vec::new();
        for key in GEN_KEYS {
            gen_fields.push(FieldGen::new(key, window, cx));
        }

        let mut _subscriptions = vec![cx.subscribe_in(&expr_state, window, {
            let expr_state = expr_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = expr_state.read(cx).value();
                    this.expr = value.to_string();
                    cx.notify();
                }
            }
        })];

        for i in 0..GEN_KEYS.len() {
            let key = GEN_KEYS[i];
            let field = &gen_fields[i];

            let mode_state = field.mode_state.clone();
            _subscriptions.push(cx.subscribe_in(
                &mode_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        let mode = mode_from_label(label);
                        let field = this.field_mut(key);
                        if field.mode != mode {
                            field.mode = mode;
                            cx.notify();
                        }
                    }
                },
            ));

            if let Some(entity) = &field.interval_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).interval = text.trim().parse::<u32>().ok();
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(entity) = &field.values_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).values_text = text;
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(state) = &field.special_state {
                let state = state.clone();
                let kind = field.kind;
                _subscriptions.push(cx.subscribe_in(
                    &state,
                    window,
                    move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                        if let SelectEvent::Confirm(Some(label)) = ev {
                            let field = this.field_mut(key);
                            match kind {
                                FieldKind::Dom => field.special = dom_special_from_label(label),
                                FieldKind::Dow => field.special = dow_special_from_label(label),
                                _ => {}
                            }
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(state) = &field.special_dow_state {
                let state = state.clone();
                _subscriptions.push(cx.subscribe_in(
                    &state,
                    window,
                    move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                        if let SelectEvent::Confirm(Some(label)) = ev
                            && let Some(n) = parse_dow_number(label)
                        {
                            this.field_mut(key).special_dow = n;
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(entity) = &field.w_day_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).w_day = text.trim().parse::<u32>().ok();
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(entity) = &field.offset_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).offset = text.trim().parse::<u32>().ok();
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(entity) = &field.nth_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).nth = text.trim().parse::<u32>().ok();
                            cx.notify();
                        }
                    },
                ));
            }

            if let Some(entity) = &field.year_state {
                let state = entity.clone();
                _subscriptions.push(cx.subscribe_in(
                    entity,
                    window,
                    move |this, _, ev: &InputEvent, _, cx| {
                        if let InputEvent::Change = ev {
                            let text = state.read(cx).value().to_string();
                            this.field_mut(key).year_text = text;
                            cx.notify();
                        }
                    },
                ));
            }
        }

        Self {
            expr: DEFAULT_LINUX_EXPR.to_string(),
            cron_type: "linux".to_string(),
            active_tab: Tab::Parse,
            result: None,
            error: String::new(),
            expr_state,
            gen_fields,
            _subscriptions,
        }
    }

    fn field(&self, key: GenFieldKey) -> &FieldGen {
        &self.gen_fields[key as usize]
    }

    fn field_mut(&mut self, key: GenFieldKey) -> &mut FieldGen {
        &mut self.gen_fields[key as usize]
    }

    fn visible_keys(&self) -> Vec<GenFieldKey> {
        let mut keys = vec![
            GenFieldKey::Min,
            GenFieldKey::Hour,
            GenFieldKey::Dom,
            GenFieldKey::Month,
            GenFieldKey::Dow,
        ];
        if self.cron_type != "linux" {
            keys.insert(0, GenFieldKey::Sec);
        }
        if self.cron_type == "quartz" {
            keys.push(GenFieldKey::Year);
        }
        keys
    }

    fn generated_expr(&self) -> String {
        let cron_type = self.cron_type.as_str();
        let mut parts: Vec<String> = Vec::new();
        if cron_type != "linux" {
            parts.push(compose_num(self.field(GenFieldKey::Sec), 0, 59));
        }
        parts.push(compose_num(self.field(GenFieldKey::Min), 0, 59));
        parts.push(compose_num(self.field(GenFieldKey::Hour), 0, 23));
        parts.push(compose_dom(self.field(GenFieldKey::Dom)));
        parts.push(compose_num(self.field(GenFieldKey::Month), 1, 12));
        let dow_min = if cron_type == "quartz" { 1 } else { 0 };
        parts.push(compose_dow(self.field(GenFieldKey::Dow), dow_min));
        if cron_type == "quartz" {
            let year = self.field(GenFieldKey::Year);
            if year.mode == GenMode::Specific {
                let text = year.year_text.trim();
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }
        parts.join(" ")
    }

    #[allow(dead_code)]
    fn type_label(&self) -> &'static str {
        match self.cron_type.as_str() {
            "spring" => "Java Spring",
            "quartz" => "Java Quartz",
            _ => "Linux",
        }
    }

    fn on_cron_type_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let cron_type = self.cron_type.clone();

        let default_expr = match cron_type.as_str() {
            "spring" => DEFAULT_SPRING_EXPR,
            "quartz" => DEFAULT_QUARTZ_EXPR,
            _ => DEFAULT_LINUX_EXPR,
        };
        self.expr = default_expr.to_string();
        self.expr_state.update(cx, |state, cx| {
            state.set_value(default_expr.to_string(), window, cx);
        });

        for key in [GenFieldKey::Dom, GenFieldKey::Dow] {
            let field = self.field_mut(key);
            if cron_type == "linux" && field.mode == GenMode::Special {
                field.mode = GenMode::Any;
            }
            let items = mode_items(field.kind, &cron_type);
            let label = mode_label(field.mode, field.kind).to_string();
            let state = field.mode_state.clone();
            state.update(cx, |state, cx| {
                state.set_items(items, window, cx);
                state.set_selected_value(&label, window, cx);
            });
        }

        let field = self.field_mut(GenFieldKey::Dow);
        if let Some(state) = field.special_dow_state.clone() {
            let items = dow_items(&cron_type);
            let wanted = field.special_dow;
            let label = items
                .iter()
                .find(|item| parse_dow_number(item) == Some(wanted))
                .cloned();
            state.update(cx, |state, cx| {
                state.set_items(items, window, cx);
                if let Some(label) = label {
                    state.set_selected_value(&label, window, cx);
                }
            });
        }

        cx.notify();
    }

    fn hint(&self) -> &'static str {
        match self.cron_type.as_str() {
            "linux" => "5 个字段；周 0-7（0 和 7 均为周日）；支持 @ 宏",
            "spring" => "6 个字段；周 0-7（0 和 7 均为周日）；支持 ? L W # 与 @ 宏",
            "quartz" => "6-7 个字段；周 1-7（1 为周日）；支持 ? L W # 与年份字段",
            _ => "5 个字段；周 0-7（0 和 7 均为周日）；支持 @ 宏",
        }
    }

    fn presets(&self) -> &'static [(&'static str, &'static str)] {
        match self.cron_type.as_str() {
            "spring" => SPRING_PRESETS,
            "quartz" => QUARTZ_PRESETS,
            _ => LINUX_PRESETS,
        }
    }

    fn parse(&mut self, cx: &mut Context<Self>) {
        let expression = self.expr.trim().to_string();
        if expression.is_empty() {
            self.error = "请输入 Cron 表达式".to_string();
            self.result = None;
            cx.notify();
            return;
        }
        match ::base::cron_parse(&expression, 10, &self.cron_type) {
            Ok(result) => {
                self.error.clear();
                self.result = Some(result);
            }
            Err(e) => {
                self.error = e.to_string();
                self.result = None;
            }
        }
        cx.notify();
    }

    fn apply_preset(&mut self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.expr = value.to_string();
        self.expr_state.update(cx, |state, cx| {
            state.set_value(value.to_string(), window, cx);
        });
        self.parse(cx);
    }

    fn parse_generated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let expression = self.generated_expr();
        self.expr = expression.clone();
        self.active_tab = Tab::Parse;
        self.expr_state.update(cx, |state, cx| {
            state.set_value(expression, window, cx);
        });
        self.parse(cx);
    }

    fn copy_generated(&mut self, cx: &mut Context<Self>) {
        let expression = self.generated_expr();
        if !expression.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(expression));
        }
    }

    fn copy_expression(&mut self, cx: &mut Context<Self>) {
        if let Some(result) = &self.result {
            cx.write_to_clipboard(ClipboardItem::new_string(result.expression.clone()));
        }
    }

    fn clear_result(&mut self, cx: &mut Context<Self>) {
        self.result = None;
        self.error.clear();
        cx.notify();
    }

    fn unit_of(key: GenFieldKey) -> &'static str {
        match key {
            GenFieldKey::Sec => "秒",
            GenFieldKey::Min => "分钟",
            GenFieldKey::Hour => "小时",
            GenFieldKey::Dom => "天",
            GenFieldKey::Month => "个月",
            _ => "",
        }
    }

    fn any_symbol(cx: &App) -> Div {
        div()
            .px_3()
            .py_1()
            .border_dashed()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_sm()
            .font_family("monospace")
            .text_size(px(16.0))
            .text_color(cx.theme().muted_foreground)
            .child("*")
    }

    fn render_gen_card(&self, key: GenFieldKey, cx: &App) -> Div {
        let field = self.field(key);

        let head = div()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .mb_2()
            .child(
                div()
                    .text_size(px(13.0))
                    .font_semibold()
                    .child(field.label.to_string()),
            )
            .child(div().w(px(132.0)).child(Select::new(&field.mode_state)));

        let body = match field.mode {
            GenMode::Any => Self::any_symbol(cx).into_any_element(),
            GenMode::Interval => {
                let n = field.interval.unwrap_or(field.interval_default).max(1);
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .when_some(field.interval_state.clone(), |this, state| {
                        this.child(div().w(px(90.0)).child(Input::new(&state)))
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("每 {} {}", n, Self::unit_of(field.key))),
                    )
                    .into_any_element()
            }
            GenMode::Specific => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("逗号分隔，自动排序去重，越界值忽略"),
                )
                .when_some(field.values_state.clone(), |this, state| {
                    this.child(Input::new(&state))
                })
                .into_any_element(),
            GenMode::Special => {
                if field.kind == FieldKind::Dom {
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .when_some(field.special_state.clone(), |this, state| {
                            this.child(div().flex_1().min_w(px(140.0)).child(Select::new(&state)))
                        })
                        .when(field.special == "W", |this| {
                            this.when_some(field.w_day_state.clone(), |this, state| {
                                this.child(div().w(px(70.0)).child(Input::new(&state)))
                            })
                        })
                        .when(field.special == "L-n", |this| {
                            this.when_some(field.offset_state.clone(), |this, state| {
                                this.child(div().w(px(70.0)).child(Input::new(&state)))
                            })
                        })
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .when_some(field.special_state.clone(), |this, state| {
                            this.child(div().flex_1().min_w(px(130.0)).child(Select::new(&state)))
                        })
                        .when_some(field.special_dow_state.clone(), |this, state| {
                            this.child(div().w(px(108.0)).child(Select::new(&state)))
                        })
                        .when(field.special == "#", |this| {
                            this.when_some(field.nth_state.clone(), |this, state| {
                                this.child(div().w(px(64.0)).child(Input::new(&state)))
                            })
                        })
                        .into_any_element()
                }
            }
        };

        div()
            .w(px(320.0))
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .flex()
            .flex_col()
            .gap_1()
            .child(head)
            .child(body)
    }

    fn render_result(&self, cx: &Context<Self>) -> Div {
        let mut container = div().flex().flex_col().gap_3().mt_2();

        if !self.error.is_empty() {
            return container.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(Hsla::from(rgb(design::ERROR_RED)))
                    .bg(design::tint(design::ERROR_RED, 0.06))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child("表达式无效"),
                            )
                            .child(
                                Button::new("clear-error")
                                    .icon(Icon::new(IconName::Close))
                                    .compact()
                                    .tooltip("关闭")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_result(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(Hsla::from(rgb(design::ERROR_RED)))
                            .child(self.error.clone()),
                    ),
            );
        }

        let Some(result) = self.result.as_ref() else {
            return container;
        };

        if !result.valid {
            let error_text = result
                .error
                .clone()
                .unwrap_or_else(|| "表达式无效".to_string());
            return container.child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(Hsla::from(rgb(design::ERROR_RED)))
                    .bg(design::tint(design::ERROR_RED, 0.06))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child("表达式无效"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child(error_text),
                            ),
                    )
                    .child(
                        Button::new("clear-invalid")
                            .icon(Icon::new(IconName::Close))
                            .compact()
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_result(cx);
                            })),
                    ),
            );
        }

        let type_label = match result.cron_type.as_str() {
            "linux" => "Linux",
            "spring" => "Java Spring",
            _ => "Java Quartz",
        };
        let ok_green = Hsla::from(rgb(design::OK_GREEN));
        let banner = div()
            .w_full()
            .flex()
            .items_start()
            .justify_between()
            .flex_wrap()
            .gap_4()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .min_w_0()
                    .child(design::mini_tag(
                        type_label,
                        design::tint(design::OK_GREEN, 0.12),
                        ok_green,
                    ))
                    .child(
                        div()
                            .font_family("monospace")
                            .text_size(px(16.0))
                            .mt_1()
                            .child(result.expression.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                result
                                    .description
                                    .clone()
                                    .unwrap_or_else(|| "无描述".to_string()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("copy-expr")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_expression(cx);
                            })),
                    )
                    .child(
                        Button::new("clear-result")
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_result(cx);
                            })),
                    ),
            );
        container = container.child(banner);

        container = container.child(design::editor_label("接下来 10 次执行时间", cx));
        let time_chips = result
            .next_times
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let first = i == 0;
                let mut chip = div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .font_family("monospace")
                    .text_size(px(13.0))
                    .border_color(if first {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .text_color(if first {
                        cx.theme().primary
                    } else {
                        cx.theme().foreground
                    });
                if first {
                    chip = chip.font_semibold().child(
                        div()
                            .px_2()
                            .py(px(1.0))
                            .rounded_full()
                            .bg(cx.theme().primary)
                            .text_color(gpui::white())
                            .text_size(px(11.0))
                            .font_semibold()
                            .child("下次"),
                    );
                }
                chip.child(t.clone())
            })
            .collect::<Vec<_>>();
        container.child(div().grid().grid_cols(2).gap_2().children(time_chips))
    }
}

impl Render for CronConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_generate = self.active_tab == Tab::Generate;

        let type_cards: [(&str, &str, &str); 3] = [
            ("linux", "Linux", "5 字段 · 无秒"),
            ("spring", "Java Spring", "6 字段 · 秒级"),
            ("quartz", "Java Quartz", "6-7 字段 · 可含年"),
        ];
        let kind_switch =
            div()
                .grid()
                .grid_cols(3)
                .gap_3()
                .children(type_cards.iter().enumerate().map(|(i, (key, title, sub))| {
                    let active = self.cron_type == *key;
                    let key = key.to_string();
                    div()
                        .id(("kind-card", i))
                        .flex()
                        .items_center()
                        .gap_2()
                        .pl_3()
                        .pr_4()
                        .py(px(14.0))
                        .rounded(px(12.0))
                        .border_1()
                        .bg(if active {
                            design::tint(0x4f6ef7, 0.05)
                        } else {
                            cx.theme().background
                        })
                        .border_color(if active {
                            cx.theme().primary
                        } else {
                            cx.theme().border
                        })
                        .hover(|s| s.border_color(cx.theme().primary))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if this.cron_type != key {
                                this.cron_type = key.clone();
                                this.on_cron_type_changed(window, cx);
                            }
                        }))
                        .child(div().w(px(3.0)).h(px(30.0)).rounded_full().bg(if active {
                            cx.theme().primary
                        } else {
                            gpui::black().opacity(0.0)
                        }))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_size(px(15.0))
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child(title.to_string()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(sub.to_string()),
                                ),
                        )
                }));

        let kind_hint = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .mt_1()
            .child(
                Icon::new(IconName::Info)
                    .size(px(15.0))
                    .text_color(cx.theme().primary)
                    .opacity(0.8),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.hint()),
            );

        let presets = self.presets();
        let preset_grid =
            div()
                .grid()
                .grid_cols(3)
                .gap_2()
                .children(presets.iter().enumerate().map(|(i, (label, value))| {
                    let active = self.expr.trim() == *value;
                    let value = value.to_string();
                    let value_for_click = value.clone();
                    div()
                        .id(("preset", i))
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap(px(4.0))
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .border_1()
                        .bg(if active {
                            design::tint(0x4f6ef7, 0.06)
                        } else {
                            gpui::black().opacity(0.0)
                        })
                        .border_color(if active {
                            cx.theme().primary
                        } else {
                            cx.theme().border
                        })
                        .hover(|s| s.border_color(cx.theme().primary))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.apply_preset(&value_for_click, window, cx);
                        }))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_semibold()
                                .child(label.to_string()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family("monospace")
                                .text_color(cx.theme().muted_foreground)
                                .child(value.clone()),
                        )
                }));

        let gen_cards = self
            .visible_keys()
            .iter()
            .map(|&key| self.render_gen_card(key, cx).into_any_element())
            .collect::<Vec<_>>();

        let generated = self.generated_expr();

        let expr_bar = div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_3()
            .mt_2()
            .px(px(14.0))
            .py_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .min_w(px(200.0))
                    .child(
                        Icon::new(IconName::Frame)
                            .size(px(16.0))
                            .text_color(cx.theme().primary),
                    )
                    .child(
                        div()
                            .font_family("monospace")
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().primary)
                            .child(generated.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("parse-preview")
                            .primary()
                            .icon(Icon::new(IconName::Play))
                            .tooltip("解析预览")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.parse_generated(window, cx);
                            })),
                    )
                    .child(
                        Button::new("copy-generated")
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_generated(cx);
                            })),
                    ),
            );

        let result_panel = self.render_result(cx);

        design::page().child(kind_switch).child(kind_hint).child(
            design::card(cx)
                .p(px(22.0))
                .child(
                    ButtonGroup::new("cron-tabs")
                        .child(
                            Button::new("tab-parse")
                                .label("解析")
                                .selected(!is_generate)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.active_tab = Tab::Parse;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("tab-generate")
                                .label("生成")
                                .selected(is_generate)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.active_tab = Tab::Generate;
                                    cx.notify();
                                })),
                        ),
                )
                .when(self.active_tab == Tab::Parse, |this| {
                    this.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(div().flex_1().child(Input::new(&self.expr_state)))
                            .child(
                                Button::new("parse")
                                    .primary()
                                    .icon(Icon::new(IconName::Play))
                                    .label("解析")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.parse(cx);
                                    })),
                            ),
                    )
                    .child(design::editor_label("常用预设", cx).mt_3())
                    .child(preset_grid)
                })
                .when(self.active_tab == Tab::Generate, |this| {
                    this.child(div().flex().flex_wrap().gap_3().children(gen_cards))
                        .child(expr_bar)
                })
                .child(result_panel),
        )
    }
}

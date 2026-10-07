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

const FORMATS: [&str; 6] = ["IPv4", "十进制", "十六进制", "二进制", "八进制", "IPv6"];

const PLACEHOLDERS: [&str; 6] = [
    "如 192.168.1.1",
    "如 3232235777（0 ~ 4294967295）",
    "如 0xC0A80101（0x 前缀，8 位以内）",
    "如 11000000101010000000000100000001（32 位以内）",
    "如 0177（0 前缀八进制整数）",
    "如 2001:0db8:85a3::8a2e:0370:7334",
];

pub struct IpConverter {
    format: usize,
    error: String,
    result: Option<IpResultView>,
    input_state: Entity<InputState>,
    format_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Default)]
struct IpResultView {
    ipv4: String,
    decimal: String,
    hex: String,
    binary: String,
    octal: String,
    ipv6_binary: String,
}

impl IpConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder(PLACEHOLDERS[0]));
        let format_items: Vec<String> = FORMATS.iter().map(|s| s.to_string()).collect();
        let format_state = cx.new(|cx| {
            let mut state = SelectState::new(format_items, None, window, cx);
            state.set_selected_value(&"IPv4".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &input_state,
                window,
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::PressEnter { .. } = ev {
                        this.convert(cx);
                    }
                },
            ),
            cx.subscribe_in(
                &format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, window, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        this.format = FORMATS.iter().position(|f| f == label).unwrap_or(0);
                        this.input_state.update(cx, |state, cx| {
                            state.set_placeholder(PLACEHOLDERS[this.format], window, cx);
                        });
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            format: 0,
            error: String::new(),
            result: None,
            input_state,
            format_state,
            _subscriptions,
        }
    }

    fn set_error(&mut self, msg: &str, cx: &mut Context<Self>) {
        self.error = msg.to_string();
        self.result = None;
        cx.notify();
    }

    fn convert(&mut self, cx: &mut Context<Self>) {
        let raw = self.input_state.read(cx).value().trim().to_string();
        if raw.is_empty() {
            self.set_error("请输入内容", cx);
            return;
        }

        let mut is_v6 = false;
        let parsed: Result<u128, String> = match self.format {
            0 => match ::base::ip_to_number("v4", Some(raw.clone())) {
                Ok(map) if !map.is_empty() => map
                    .get("decimal")
                    .and_then(|d| d.parse::<u128>().ok())
                    .ok_or_else(|| "IPv4 格式非法（应为 4 段 0-255）".to_string()),
                _ => Err("IPv4 格式非法（应为 4 段 0-255）".to_string()),
            },
            1 => {
                if raw.len() > 10 || !raw.chars().all(|c| c.is_ascii_digit()) {
                    Err("十进制格式非法".to_string())
                } else {
                    match raw.parse::<u128>() {
                        Ok(v) if v <= 0xffffffff => Ok(v),
                        _ => Err("十进制超出范围（0 ~ 4294967295）".to_string()),
                    }
                }
            }
            2 => {
                let h = raw
                    .strip_prefix("0x")
                    .or_else(|| raw.strip_prefix("0X"))
                    .unwrap_or(&raw);
                if h.is_empty() || h.len() > 8 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                    Err("十六进制格式非法（0x 前缀，8 位以内）".to_string())
                } else {
                    Ok(u128::from_str_radix(h, 16).unwrap_or(0))
                }
            }
            3 => {
                let b = raw
                    .strip_prefix("0b")
                    .or_else(|| raw.strip_prefix("0B"))
                    .unwrap_or(&raw);
                if b.is_empty() || b.len() > 32 || !b.chars().all(|c| c == '0' || c == '1') {
                    Err("二进制格式非法（32 位以内 0/1）".to_string())
                } else {
                    Ok(u128::from_str_radix(b, 2).unwrap_or(0))
                }
            }
            4 => {
                let o = raw.strip_prefix('0').unwrap_or(&raw);
                if o.is_empty() {
                    Ok(0)
                } else if o.len() > 11 || !o.chars().all(|c| ('0'..='7').contains(&c)) {
                    Err("八进制格式非法（0 前缀 + 八进制数字）".to_string())
                } else {
                    Ok(u128::from_str_radix(o, 8).unwrap_or(0))
                }
            }
            _ => {
                is_v6 = true;
                parse_ipv6(&raw)
                    .ok_or_else(|| "IPv6 格式非法（8 组 4 位 hex，支持 :: 压缩）".to_string())
            }
        };

        let value = match parsed {
            Ok(v) => v,
            Err(msg) => {
                self.set_error(&msg, cx);
                return;
            }
        };

        let bit_len = if is_v6 { 128 } else { 32 };
        let hex_digits = if is_v6 { 32 } else { 8 };
        let ipv4 = if is_v6 {
            "—".to_string()
        } else {
            int_to_ipv4(value)
        };
        self.error.clear();
        self.result = Some(IpResultView {
            ipv4,
            decimal: value.to_string(),
            hex: format!("0x{:0width$x}", value, width = hex_digits),
            binary: format!("{:0width$b}", value, width = bit_len),
            octal: format!("0{:o}", value),
            ipv6_binary: if is_v6 {
                (0..8)
                    .map(|i| format!("{:016b}", (value >> ((7 - i) * 16)) & 0xffff))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                String::new()
            },
        });
        cx.notify();
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }
}

fn int_to_ipv4(v: u128) -> String {
    format!(
        "{}.{}.{}.{}",
        (v >> 24) & 0xff,
        (v >> 16) & 0xff,
        (v >> 8) & 0xff,
        v & 0xff
    )
}

fn parse_ipv6(s: &str) -> Option<u128> {
    let s = s.trim().to_lowercase();
    if s.is_empty() || s.matches("::").count() > 1 {
        return None;
    }
    let groups: Vec<String> = if let Some((left, right)) = s.split_once("::") {
        let left_parts: Vec<&str> = if left.is_empty() {
            Vec::new()
        } else {
            left.split(':').collect()
        };
        let right_parts: Vec<&str> = if right.is_empty() {
            Vec::new()
        } else {
            right.split(':').collect()
        };
        let parts_len = left_parts.len() + right_parts.len();
        if parts_len >= 8 {
            return None;
        }
        let mut groups = left_parts
            .iter()
            .copied()
            .map(str::to_string)
            .collect::<Vec<_>>();
        for _ in 0..(8 - parts_len) {
            groups.push("0".to_string());
        }
        groups.extend(right_parts.iter().copied().map(str::to_string));
        groups
    } else {
        let groups: Vec<&str> = s.split(':').collect();
        if groups.len() != 8 {
            return None;
        }
        groups.iter().copied().map(str::to_string).collect()
    };
    let mut value: u128 = 0;
    for g in &groups {
        if g.is_empty() || g.len() > 4 || !g.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        value = (value << 16) | u128::from_str_radix(g, 16).ok()?;
    }
    Some(value)
}

impl Render for IpConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let result = self.result.clone();

        let mut card = design::card(cx).child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(240.0))
                        .child(Input::new(&self.input_state)),
                )
                .child(div().w(px(130.0)).child(Select::new(&self.format_state)))
                .child(
                    Button::new("ip-convert")
                        .primary()
                        .icon(Icon::new(IconName::RotateCw))
                        .tooltip("转换")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.convert(cx);
                        })),
                ),
        );

        if let Some(r) = result {
            let mut items: Vec<(&'static str, String)> = vec![
                ("IPv4 点分形式", r.ipv4),
                ("十进制整数", r.decimal),
                ("十六进制", r.hex),
                ("二进制", r.binary),
                ("八进制", r.octal),
            ];
            if !r.ipv6_binary.is_empty() {
                items.push(("二进制展开（16 位 × 8 段）", r.ipv6_binary));
            }
            let rows = items
                .into_iter()
                .enumerate()
                .map(|(i, (label, value))| {
                    result_row(
                        ElementId::Name(format!("ip-copy-{}", i).into()),
                        label,
                        value,
                        cx,
                    )
                })
                .collect::<Vec<_>>();

            card = card
                .child(div().mt_2().border_t_1().border_color(cx.theme().border))
                .child(
                    div()
                        .mt_3()
                        .flex_col()
                        .gap_0()
                        .child(
                            div()
                                .flex()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().muted_foreground)
                                .child(div().w(px(200.0)).px_2().py_1p5().child("项目"))
                                .child(div().px_2().py_1p5().child("值")),
                        )
                        .children(rows),
                );
        }

        design::page().child(card).when(!error.is_empty(), |this| {
            this.child(
                div()
                    .text_size(px(12.5))
                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                    .child(error),
            )
        })
    }
}

fn result_row(
    id: ElementId,
    label: &'static str,
    value: String,
    cx: &mut Context<IpConverter>,
) -> Div {
    let copy_value = value.clone();
    div()
        .flex()
        .items_center()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .w(px(200.0))
                .flex_shrink_0()
                .px_2()
                .py_1p5()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .px_2()
                .py_1p5()
                .child(
                    div()
                        .min_w_0()
                        .text_size(px(13.0))
                        .font_family("monospace")
                        .child(value.clone()),
                )
                .child(
                    Button::new(id)
                        .ghost()
                        .compact()
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .disabled(value.is_empty() || value == "—")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let v = copy_value.clone();
                            this.copy_text(&v, cx);
                        })),
                ),
        )
}

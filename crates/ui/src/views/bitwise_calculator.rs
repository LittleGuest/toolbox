use crate::design;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{Input, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

fn parse_operand(text: &str, base: &str, width: u32) -> Option<u64> {
    let s = text.trim();
    if s.is_empty() {
        return None;
    }
    let max: u128 = 1u128 << width;
    let v: u128 = match base {
        "hex" => {
            let h = s
                .strip_prefix("0x")
                .or_else(|| s.strip_prefix("0X"))
                .unwrap_or(s);
            u128::from_str_radix(h, 16).ok()?
        }
        "bin" => {
            let b = s
                .strip_prefix("0b")
                .or_else(|| s.strip_prefix("0B"))
                .unwrap_or(s);
            u128::from_str_radix(b, 2).ok()?
        }
        _ => {
            let n: i128 = s.parse().ok()?;
            if n < 0 {
                let m = n.unsigned_abs();
                if m > max {
                    return None;
                }
                max - m
            } else {
                n as u128
            }
        }
    };
    if v >= max {
        return None;
    }
    Some(v as u64)
}

pub struct BitwiseCalculator {
    a_state: Entity<InputState>,
    b_state: Entity<InputState>,
    shift_state: Entity<InputState>,
    bits_state: Entity<SelectState<Vec<String>>>,
    a_base_state: Entity<SelectState<Vec<String>>>,
    b_base_state: Entity<SelectState<Vec<String>>>,
    bits: u32,
    a_base: String,
    b_base: String,
    res_expr: String,
    res_dec: String,
    res_hex: String,
    res_bin: String,
    status: String,
    _subscriptions: Vec<Subscription>,
}

impl BitwiseCalculator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let a_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("如 255、0xFF、0b1111")
        });
        let b_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("单目运算（NOT、移位）可留空")
        });
        let shift_state = cx.new(|cx| InputState::new(window, cx).placeholder("移位位数"));

        let base_items = vec![
            "十进制".to_string(),
            "十六进制".to_string(),
            "二进制".to_string(),
        ];
        let bits_items = vec![
            "8 位".to_string(),
            "16 位".to_string(),
            "32 位".to_string(),
            "64 位".to_string(),
        ];

        let a_base_state = cx.new(|cx| {
            let mut state = SelectState::new(base_items.clone(), None, window, cx);
            state.set_selected_value(&"十进制".to_string(), window, cx);
            state
        });
        let b_base_state = cx.new(|cx| {
            let mut state = SelectState::new(base_items, None, window, cx);
            state.set_selected_value(&"十进制".to_string(), window, cx);
            state
        });
        let bits_state = cx.new(|cx| {
            let mut state = SelectState::new(bits_items, None, window, cx);
            state.set_selected_value(&"32 位".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &bits_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.bits = match value.trim_end_matches(" 位").parse::<u32>() {
                            Ok(v) => v,
                            Err(_) => 32,
                        };
                        this.res_expr.clear();
                        this.res_dec.clear();
                        this.res_hex.clear();
                        this.res_bin.clear();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &a_base_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.a_base = match value.as_str() {
                            "十六进制" => "hex",
                            "二进制" => "bin",
                            _ => "dec",
                        }
                        .to_string();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &b_base_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.b_base = match value.as_str() {
                            "十六进制" => "hex",
                            "二进制" => "bin",
                            _ => "dec",
                        }
                        .to_string();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            a_state,
            b_state,
            shift_state,
            bits_state,
            a_base_state,
            b_base_state,
            bits: 32,
            a_base: "dec".to_string(),
            b_base: "dec".to_string(),
            res_expr: String::new(),
            res_dec: String::new(),
            res_hex: String::new(),
            res_bin: String::new(),
            status: String::new(),
            _subscriptions,
        }
    }

    fn read_shift(&self, cx: &App) -> u64 {
        self.shift_state
            .read(cx)
            .value()
            .trim()
            .parse::<u64>()
            .unwrap_or(1)
            .max(1)
    }

    fn calc(&mut self, op: &str, cx: &mut Context<Self>) {
        let width = self.bits;
        let mask: u64 = if width >= 64 { u64::MAX } else { (1u64 << width) - 1 };

        let a_text = self.a_state.read(cx).value().to_string();
        let Some(a) = parse_operand(&a_text, &self.a_base.as_str(), width) else {
            self.status = "操作数无效".to_string();
            self.res_expr.clear();
            cx.notify();
            return;
        };

        let needs_b = matches!(op, "and" | "or" | "xor" | "nand" | "nor" | "xnor");
        let b = if needs_b {
            let b_text = self.b_state.read(cx).value().to_string();
            match parse_operand(&b_text, self.b_base.as_str(), width) {
                Some(v) => v,
                None => {
                    self.status = "操作数无效".to_string();
                    self.res_expr.clear();
                    cx.notify();
                    return;
                }
            }
        } else {
            0
        };

        let value: u64 = match op {
            "and" => a & b,
            "or" => a | b,
            "xor" => a ^ b,
            "nand" => !(a & b) & mask,
            "nor" => !(a | b) & mask,
            "xnor" => !(a ^ b) & mask,
            "not" => a ^ mask,
            "shl" => {
                let n = self.read_shift(cx).min(127);
                (((a as u128) << n) & (mask as u128)) as u64
            }
            "shr" => {
                let n = self.read_shift(cx);
                if n >= 64 {
                    0
                } else {
                    a >> n
                }
            }
            _ => return,
        };

        let expr = match op {
            "and" => "A & B".to_string(),
            "or" => "A | B".to_string(),
            "xor" => "A ^ B".to_string(),
            "nand" => "~(A & B)".to_string(),
            "nor" => "~(A | B)".to_string(),
            "xnor" => "~(A ^ B)".to_string(),
            "not" => "~A".to_string(),
            "shl" => format!("A << {}", self.read_shift(cx)),
            "shr" => format!("A >> {}", self.read_shift(cx)),
            _ => String::new(),
        };

        let hex_digits = (width as usize + 3) / 4;
        self.res_expr = expr;
        self.res_dec = value.to_string();
        self.res_hex = format!("0x{:0w$X}", value, w = hex_digits);
        self.res_bin = format!("{:0w$b}", value, w = width as usize);
        self.status.clear();
        cx.notify();
    }

    fn copy(&mut self, text: String, cx: &mut Context<Self>) {
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }
}

impl Render for BitwiseCalculator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(80.0);

        let op_button =
            |id: &'static str, symbol: &'static str, tooltip: &'static str, cx: &mut Context<Self>| {
                Button::new(id)
                    .child(
                        div()
                            .font_family("monospace")
                            .text_sm()
                            .font_medium()
                            .child(symbol),
                    )
                    .tooltip(tooltip)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.calc(op_for_id(id), cx);
                    }))
            };

        let result_row = |label: &'static str,
                          value: String,
                          copy_id: &'static str,
                          cx: &mut Context<Self>| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().w(label_w).text_sm().child(label))
                .child(
                    div()
                        .flex_1()
                        .font_family("monospace")
                        .text_sm()
                        .child(value.clone()),
                )
                .child(
                    Button::new(copy_id)
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let text = value.clone();
                            this.copy(text, cx);
                        })),
                )
        };

        design::page()
            .child(design::page_header("按位计算器", "二进制位运算", cx))
            .child(
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("位宽", cx)))
                            .child(Select::new(&self.bits_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("操作数 A", cx)))
                            .child(div().flex_1().child(Input::new(&self.a_state)))
                            .child(div().w(px(110.0)).child(Select::new(&self.a_base_state))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("操作数 B", cx)))
                            .child(div().flex_1().child(Input::new(&self.b_state)))
                            .child(div().w(px(110.0)).child(Select::new(&self.b_base_state))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_center()
                                    .gap_2()
                                    .child(op_button("op-and", "&", "AND", cx))
                                    .child(op_button("op-or", "|", "OR", cx))
                                    .child(op_button("op-xor", "^", "XOR", cx))
                                    .child(op_button("op-nand", "~&", "NAND", cx))
                                    .child(op_button("op-nor", "~|", "NOR", cx))
                                    .child(op_button("op-xnor", "~^", "XNOR", cx))
                                    .child(op_button("op-not", "~", "NOT(A)", cx))
                                    .child(op_button("op-shl", "<<", "左移", cx))
                                    .child(op_button("op-shr", ">>", "右移", cx))
                                    .child(
                                        div()
                                            .w(px(90.0))
                                            .ml_2()
                                            .child(Input::new(&self.shift_state)),
                                    )
                                    .child(
                                        div().text_xs().text_color(cx.theme().muted_foreground).child("移位位数"),
                                    ),
                            ),
                    )
                    .when(!self.status.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().w(label_w))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xef4444))
                                        .child(self.status.clone()),
                                ),
                        )
                    }),
            )
            .when(!self.res_expr.is_empty(), |this| {
                this.child(
                    design::card(cx)
                        .child(result_row("说明", self.res_expr.clone(), "copy-expr", cx))
                        .child(result_row("十进制", self.res_dec.clone(), "copy-dec", cx))
                        .child(result_row("十六进制", self.res_hex.clone(), "copy-hex", cx))
                        .child(result_row("二进制", self.res_bin.clone(), "copy-bin", cx)),
                )
            })
    }
}

fn op_for_id(id: &str) -> &'static str {
    match id {
        "op-and" => "and",
        "op-or" => "or",
        "op-xor" => "xor",
        "op-nand" => "nand",
        "op-nor" => "nor",
        "op-xnor" => "xnor",
        "op-not" => "not",
        "op-shl" => "shl",
        "op-shr" => "shr",
        _ => "and",
    }
}

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

const BASE_OPTIONS: [&str; 3] = ["十进制", "十六进制", "二进制"];
const BIT_WIDTHS: [&str; 4] = ["8 位", "16 位", "32 位", "64 位"];
/// 与 BIT_WIDTHS 一一对应的位宽数值
const BIT_WIDTH_VALUES: [u32; 4] = [8, 16, 32, 64];

const OPERATIONS: [(&str, &str, &str, bool); 9] = [
    // (op, symbol, label, needs_b)
    ("and", "&", "AND", true),
    ("or", "|", "OR", true),
    ("xor", "^", "XOR", true),
    ("nand", "~&", "NAND", true),
    ("nor", "~|", "NOR", true),
    ("xnor", "~^", "XNOR", true),
    ("not", "~", "NOT(A)", false),
    ("shl", "<<", "左移", false),
    ("shr", ">>", "右移", false),
];

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
        let shift_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("1")
                .default_value("1".to_string())
        });

        let base_items: Vec<String> = BASE_OPTIONS.iter().map(|s| s.to_string()).collect();
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

        let _subscriptions = vec![
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
            cx.subscribe_in(&a_state, window, move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::PressEnter { .. } = ev {
                    this.calc("and", cx);
                }
            }),
        ];

        Self {
            a_state,
            b_state,
            shift_state,
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

    /// 分段选择器（对应 n-radio-button 组）
    fn segmented(
        &self,
        options: &[&'static str],
        selected: u32,
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(options.iter().enumerate().map(|(i, opt)| {
                let active = BIT_WIDTH_VALUES.get(i).copied().unwrap_or(0) == selected;
                div()
                    .id(("bits", i))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .bg(if active {
                        cx.theme().primary
                    } else {
                        gpui::black().opacity(0.0)
                    })
                    .text_color(if active {
                        gpui::white()
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.bits = BIT_WIDTH_VALUES.get(i).copied().unwrap_or(32);
                        this.res_expr.clear();
                        this.res_dec.clear();
                        this.res_hex.clear();
                        this.res_bin.clear();
                        cx.notify();
                    }))
                    .child(opt.to_string())
            }))
    }
}

impl Render for BitwiseCalculator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(80.0);
        let status = self.status.clone();
        let res_expr = self.res_expr.clone();
        let res_dec = self.res_dec.clone();
        let res_hex = self.res_hex.clone();
        let res_bin = self.res_bin.clone();

        let op_buttons = OPERATIONS
            .iter()
            .map(|(op, symbol, label, _)| {
                let op = op.to_string();
                Button::new(ElementId::Name(format!("op-{}", op).into()))
                    .primary()
                    .child(
                        div()
                            .font_family("monospace")
                            .text_sm()
                            .font_medium()
                            .child(symbol.to_string()),
                    )
                    .tooltip(label.to_string())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let op = op.clone();
                        this.calc(&op, cx);
                    }))
            })
            .collect::<Vec<_>>();

        let result_row = |id: &'static str,
                          label: &'static str,
                          value: String,
                          cx: &mut Context<Self>| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(label_w)
                        .flex_shrink_0()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(
                    div()
                        .flex_1()
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
                        .disabled(value.is_empty())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let text = value.clone();
                            this.copy(text, cx);
                        })),
                )
        };

        design::page()

            .child(
                design::card(cx)
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(label_w)
                                    .text_size(px(13.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("位宽"),
                            )
                            .child(self.segmented(&BIT_WIDTHS, self.bits, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(label_w)
                                    .text_size(px(13.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("操作数 A"),
                            )
                            .child(div().flex_1().min_w(px(220.0)).child(Input::new(&self.a_state)))
                            .child(
                                div()
                                    .w(px(110.0))
                                    .child(Select::new(&self.a_base_state)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(label_w)
                                    .text_size(px(13.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("操作数 B"),
                            )
                            .child(div().flex_1().min_w(px(220.0)).child(Input::new(&self.b_state)))
                            .child(
                                div()
                                    .w(px(110.0))
                                    .child(Select::new(&self.b_base_state)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .mt_1()
                            .children(op_buttons)
                            .child(
                                div()
                                    .w(px(90.0))
                                    .ml_2()
                                    .child(Input::new(&self.shift_state)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("移位位数"),
                            ),
                    )
                    .when(!status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                .child(status),
                        )
                    })
                    .when(!res_expr.is_empty(), |this| {
                        this.child(
                            div()
                                .mt_2()
                                .border_t_1()
                                .border_color(cx.theme().border),
                        )
                        .child(
                            div()
                                .mt_3()
                                .flex_col()
                                .gap_2()
                                .child(design::editor_label("结果", cx))
                                .child(result_row("copy-expr", "说明", res_expr, cx))
                                .child(result_row("copy-dec", "十进制", res_dec, cx))
                                .child(result_row("copy-hex", "十六进制", res_hex, cx))
                                .child(result_row("copy-bin", "二进制", res_bin, cx)),
                        )
                    }),
            )
    }
}

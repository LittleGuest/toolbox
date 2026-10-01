use gpui_kit::{prelude::FluentBuilder as _, *};
use gpui_kit::component::{
    button::*,
    input::{Input, InputState, Textarea, TextareaState},
    *,
};
use rand::Rng;

const MAX_COUNT: usize = 10000;
const MAX_TEXT_LEN: usize = 4096;
const HEX_CHARS: &[u8] = b"0123456789abcdef";

const TABS: [&str; 6] = ["整数", "小数", "素数", "十六进制", "二进制", "字节"];

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    if n % 3 == 0 {
        return n == 3;
    }
    let mut i: u64 = 5;
    while (i as u128) * (i as u128) <= n as u128 {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

pub struct RandomNumberGenerator {
    mode: usize,

    int_min_state: Entity<InputState>,
    int_max_state: Entity<InputState>,
    int_count_state: Entity<InputState>,
    int_out_state: Entity<TextareaState>,

    float_min_state: Entity<InputState>,
    float_max_state: Entity<InputState>,
    float_dec_state: Entity<InputState>,
    float_count_state: Entity<InputState>,
    float_out_state: Entity<TextareaState>,

    prime_lo_state: Entity<InputState>,
    prime_hi_state: Entity<InputState>,
    prime_count_state: Entity<InputState>,
    prime_out_state: Entity<TextareaState>,

    hex_len_state: Entity<InputState>,
    hex_count_state: Entity<InputState>,
    hex_out_state: Entity<TextareaState>,

    bin_bits_state: Entity<InputState>,
    bin_count_state: Entity<InputState>,
    bin_out_state: Entity<TextareaState>,

    byte_per_line_state: Entity<InputState>,
    byte_lines_state: Entity<InputState>,
    byte_out_state: Entity<TextareaState>,

    status: String,
}

impl RandomNumberGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |window: &mut Window, cx: &mut Context<Self>, default: &str| {
            cx.new(|cx| InputState::new(window, cx).default_value(default.to_string()))
        };
        let textarea = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder))
        };

        Self {
            mode: 0,
            int_min_state: input(window, cx, "1"),
            int_max_state: input(window, cx, "100"),
            int_count_state: input(window, cx, "10"),
            int_out_state: textarea(window, cx, "区间内随机整数（每行一个）"),
            float_min_state: input(window, cx, "0"),
            float_max_state: input(window, cx, "1"),
            float_dec_state: input(window, cx, "2"),
            float_count_state: input(window, cx, "10"),
            float_out_state: textarea(window, cx, "区间内随机小数（每行一个）"),
            prime_lo_state: input(window, cx, "1"),
            prime_hi_state: input(window, cx, "100"),
            prime_count_state: input(window, cx, "10"),
            prime_out_state: textarea(window, cx, "区间内随机素数（每行一个）"),
            hex_len_state: input(window, cx, "8"),
            hex_count_state: input(window, cx, "10"),
            hex_out_state: textarea(window, cx, "随机十六进制字符串（每行一个，不含 0x 前缀）"),
            bin_bits_state: input(window, cx, "8"),
            bin_count_state: input(window, cx, "10"),
            bin_out_state: textarea(window, cx, "随机二进制字符串（每行一个，首字符不为 0）"),
            byte_per_line_state: input(window, cx, "8"),
            byte_lines_state: input(window, cx, "10"),
            byte_out_state: textarea(window, cx, "随机字节序列（每行逗号分隔的 0-255 十进制数）"),
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

    fn read_i64(&self, state: &Entity<InputState>, cx: &App, default: i64) -> i64 {
        state.read(cx).value().trim().parse::<i64>().unwrap_or(default)
    }

    fn read_f64(&self, state: &Entity<InputState>, cx: &App, default: f64) -> f64 {
        state.read(cx).value().trim().parse::<f64>().unwrap_or(default)
    }

    fn set_output(&self, state: &Entity<TextareaState>, lines: Vec<String>, window: &mut Window, cx: &mut Context<Self>) {
        let text = lines.join("\n");
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

    fn gen_int(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let lo = self.read_i64(&self.int_min_state, cx, 1);
        let hi = self.read_i64(&self.int_max_state, cx, 100);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        let count = self.read_usize(&self.int_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| rng.random_range(lo..=hi).to_string())
            .collect();
        self.set_output(&self.int_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_float(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let lo = self.read_f64(&self.float_min_state, cx, 0.0);
        let hi = self.read_f64(&self.float_max_state, cx, 1.0);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        let decimals = self
            .float_dec_state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(2)
            .min(10);
        let count = self.read_usize(&self.float_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let v = lo + rng.random::<f64>() * (hi - lo);
                format!("{:.*}", decimals, v)
            })
            .collect();
        self.set_output(&self.float_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_prime(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let lo_raw = self.read_i64(&self.prime_lo_state, cx, 1);
        let hi_raw = self.read_i64(&self.prime_hi_state, cx, 100);
        let (mut lo, hi) = (lo_raw.min(hi_raw), lo_raw.max(hi_raw));
        let count = self.read_usize(&self.prime_count_state, cx, 10, MAX_COUNT);
        lo = lo.max(2);
        if hi < 2 {
            self.status = "区间上限需 ≥ 2".to_string();
            self.set_output(&self.prime_out_state, vec![], window, cx);
            cx.notify();
            return;
        }
        if lo > hi {
            self.status = "区间内没有素数".to_string();
            self.set_output(&self.prime_out_state, vec![], window, cx);
            cx.notify();
            return;
        }
        let mut rng = rand::rng();
        let attempts = ((hi - lo + 1) as usize).min(2000);
        let mut lines: Vec<String> = Vec::new();
        for _ in 0..count {
            let mut found = false;
            for _ in 0..attempts {
                let candidate = rng.random_range(lo..=hi) as u64;
                if is_prime(candidate) {
                    lines.push(candidate.to_string());
                    found = true;
                    break;
                }
            }
            if !found {
                self.status = "在指定区间内未找到素数".to_string();
                break;
            }
        }
        if !lines.is_empty() && lines.len() == count {
            self.status.clear();
        }
        self.set_output(&self.prime_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_hex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let len = self.read_usize(&self.hex_len_state, cx, 8, MAX_TEXT_LEN);
        let count = self.read_usize(&self.hex_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                (0..len)
                    .map(|_| HEX_CHARS[rng.random_range(0..HEX_CHARS.len())] as char)
                    .collect()
            })
            .collect();
        self.set_output(&self.hex_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_bin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let bits = self.read_usize(&self.bin_bits_state, cx, 8, MAX_TEXT_LEN);
        let count = self.read_usize(&self.bin_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let s: String = (0..bits)
                    .map(|_| if rng.random::<bool>() { '1' } else { '0' })
                    .collect();
                if bits > 1 {
                    format!("1{}", &s[1..])
                } else {
                    s
                }
            })
            .collect();
        self.set_output(&self.bin_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_byte(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        let per_line = self.read_usize(&self.byte_per_line_state, cx, 8, MAX_TEXT_LEN);
        let line_count = self.read_usize(&self.byte_lines_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..line_count)
            .map(|_| {
                (0..per_line)
                    .map(|_| rng.random_range(0..=255u32).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect();
        self.set_output(&self.byte_out_state, lines, window, cx);
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

    /// 每个标签页共用的输出编辑器：editor_label + textarea + 复制/清除
    fn output_editor(
        &self,
        copy_id: &'static str,
        clear_id: &'static str,
        state: &Entity<TextareaState>,
        cx: &mut Context<Self>,
    ) -> Div {
        let state_copy = state.clone();
        let state_clear = state.clone();
        div()
            .flex_col()
            .gap_1p5()
            .child(design::editor_label("输出", cx))
            .child(
                Textarea::new(state)
                    .h(px(220.0))
                    .font_family("monospace"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mt_2()
                    .child(
                        Button::new(copy_id)
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
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

    fn render_tab(&mut self, cx: &mut Context<Self>) -> Div {
        let label = |text: &'static str| {
            div()
                .text_size(px(12.0))
                .text_color(rgb(0x5b6478))
                .child(text)
        };
        let num = |state: &Entity<InputState>, w: f32| div().w(px(w)).child(Input::new(state));

        let gen_btn = |id: &'static str| {
            Button::new(id)
                .primary()
                .icon(Icon::new(IconName::Play))
                .tooltip("生成")
        };

        let status = self.status.clone();

        let body = match self.mode {
            0 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("最小值"))
                                .child(num(&self.int_min_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("最大值"))
                                .child(num(&self.int_max_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("数量"))
                                .child(num(&self.int_count_state, 120.0)),
                        )
                        .child(gen_btn("gen-int").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_int(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("copy-int", "clear-int", &self.int_out_state, cx)),
            1 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("最小值"))
                                .child(num(&self.float_min_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("最大值"))
                                .child(num(&self.float_max_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("小数位数"))
                                .child(num(&self.float_dec_state, 120.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("数量"))
                                .child(num(&self.float_count_state, 120.0)),
                        )
                        .child(gen_btn("gen-float").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_float(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("copy-float", "clear-float", &self.float_out_state, cx)),
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
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("下限"))
                                .child(num(&self.prime_lo_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("上限"))
                                .child(num(&self.prime_hi_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("数量"))
                                .child(num(&self.prime_count_state, 120.0)),
                        )
                        .child(gen_btn("gen-prime").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_prime(window, cx);
                            },
                        ))),
                )
                .when(!status.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                            .child(status),
                    )
                })
                .child(self.output_editor("copy-prime", "clear-prime", &self.prime_out_state, cx)),
            3 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("长度"))
                                .child(num(&self.hex_len_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("数量"))
                                .child(num(&self.hex_count_state, 120.0)),
                        )
                        .child(gen_btn("gen-hex").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_hex(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("copy-hex", "clear-hex", &self.hex_out_state, cx)),
            4 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("位数"))
                                .child(num(&self.bin_bits_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("数量"))
                                .child(num(&self.bin_count_state, 120.0)),
                        )
                        .child(gen_btn("gen-bin").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_bin(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("copy-bin", "clear-bin", &self.bin_out_state, cx)),
            _ => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("每行字节数"))
                                .child(num(&self.byte_per_line_state, 140.0)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(label("行数"))
                                .child(num(&self.byte_lines_state, 120.0)),
                        )
                        .child(gen_btn("gen-byte").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_byte(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("copy-byte", "clear-byte", &self.byte_out_state, cx)),
        };

        body
    }
}

impl Render for RandomNumberGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        design::page()
            .child(design::page_header(
                "随机数字",
                "整数 / 小数 / 素数 / 十六进制 / 二进制 / 字节",
                cx,
            ))
            .child(
                design::card(cx)
                    .child(design::card_header(
                        IconName::Play,
                        "随机数字",
                        "六类随机数字一键生成",
                        cx,
                    ))
                    // n-tabs type="line"
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
                                        0 => "tab-int",
                                        1 => "tab-float",
                                        2 => "tab-prime",
                                        3 => "tab-hex",
                                        4 => "tab-bin",
                                        _ => "tab-byte",
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

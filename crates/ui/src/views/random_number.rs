use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState, Textarea, TextareaState},
    *,
};
use rand::Rng;

const MAX_COUNT: usize = 10000;
const MAX_TEXT_LEN: usize = 4096;
const HEX_CHARS: &[u8] = b"0123456789abcdef";

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
    int_min_state: Entity<InputState>,
    int_max_state: Entity<InputState>,
    int_count_state: Entity<InputState>,
    int_allow_dup: bool,
    int_out_state: Entity<TextareaState>,

    float_min_state: Entity<InputState>,
    float_max_state: Entity<InputState>,
    float_dec_state: Entity<InputState>,
    float_count_state: Entity<InputState>,
    float_allow_dup: bool,
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
        let input = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        let textarea = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder))
        };

        Self {
            int_min_state: input(window, cx, ""),
            int_max_state: input(window, cx, ""),
            int_count_state: input(window, cx, ""),
            int_allow_dup: true,
            int_out_state: textarea(window, cx, "区间内随机整数（每行一个）"),
            float_min_state: input(window, cx, ""),
            float_max_state: input(window, cx, ""),
            float_dec_state: input(window, cx, ""),
            float_count_state: input(window, cx, ""),
            float_allow_dup: true,
            float_out_state: textarea(window, cx, "区间内随机小数（每行一个）"),
            prime_lo_state: input(window, cx, ""),
            prime_hi_state: input(window, cx, ""),
            prime_count_state: input(window, cx, ""),
            prime_out_state: textarea(window, cx, "区间内随机素数（每行一个）"),
            hex_len_state: input(window, cx, ""),
            hex_count_state: input(window, cx, ""),
            hex_out_state: textarea(window, cx, "随机十六进制字符串（每行一个，不含 0x 前缀）"),
            bin_bits_state: input(window, cx, ""),
            bin_count_state: input(window, cx, ""),
            bin_out_state: textarea(window, cx, "随机二进制字符串（每行一个，首字符不为 0）"),
            byte_per_line_state: input(window, cx, ""),
            byte_lines_state: input(window, cx, ""),
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

    fn set_output(
        &self,
        state: &Entity<TextareaState>,
        lines: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        let lo = self.read_i64(&self.int_min_state, cx, 1);
        let hi = self.read_i64(&self.int_max_state, cx, 100);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        let count = self.read_usize(&self.int_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let mut lines: Vec<String> = Vec::with_capacity(count);
        if self.int_allow_dup {
            for _ in 0..count {
                lines.push(rng.random_range(lo..=hi).to_string());
            }
        } else {
            let mut seen = std::collections::HashSet::new();
            let mut attempts = count * 20 + 1000;
            while lines.len() < count && attempts > 0 {
                attempts -= 1;
                let v = rng.random_range(lo..=hi);
                if seen.insert(v) {
                    lines.push(v.to_string());
                }
            }
            if lines.len() < count {
                self.status = "区间内不重复的整数不足，仅生成部分结果".to_string();
            }
        }
        self.set_output(&self.int_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_float(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        let mut lines: Vec<String> = Vec::with_capacity(count);
        let mut seen = std::collections::HashSet::new();
        let mut attempts = if self.float_allow_dup { count } else { count * 20 + 1000 };
        while lines.len() < count && attempts > 0 {
            attempts -= 1;
            let v = lo + rng.random::<f64>() * (hi - lo);
            let s = format!("{:.*}", decimals, v);
            if self.float_allow_dup || seen.insert(s.clone()) {
                lines.push(s);
            }
        }
        if !self.float_allow_dup && lines.len() < count {
            self.status = "区间内不重复的小数不足，仅生成部分结果".to_string();
        }
        self.set_output(&self.float_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_prime(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let lo = self.read_i64(&self.prime_lo_state, cx, 1).max(2);
        let hi = self.read_i64(&self.prime_hi_state, cx, 100);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        let count = self.read_usize(&self.prime_count_state, cx, 10, MAX_COUNT);
        if hi < 2 {
            self.status = "区间上限需 ≥ 2".to_string();
            cx.notify();
            return;
        }
        let lo = lo.max(2);
        if lo > hi {
            self.status = "区间内没有素数".to_string();
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
                break;
            }
        }
        if lines.is_empty() {
            self.status = "区间内没有素数".to_string();
        } else {
            self.status.clear();
        }
        self.set_output(&self.prime_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_hex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        let bits = self.read_usize(&self.bin_bits_state, cx, 8, MAX_TEXT_LEN);
        let count = self.read_usize(&self.bin_count_state, cx, 10, MAX_COUNT);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let s: String = (0..bits)
                    .map(|_| if rng.random::<bool>() { '1' } else { '0' })
                    .collect();
                if bits > 1 {
                    // 首字符不能为 0
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
}

impl Render for RandomNumberGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(64.0);

        let num_input = |state: &Entity<InputState>, w: f32| {
            div().w(px(w)).child(Input::new(state))
        };

        design::page()
            .child(design::page_header("随机数字", "生成随机数字", cx))
            // 整数
            .child(
                design::card(cx)
                    .child(design::caption("整数", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("最小值"))
                        .child(num_input(&self.int_min_state, 140.0))
                        .child(div().text_sm().child("最大值"))
                        .child(num_input(&self.int_max_state, 140.0))
                        .child(div().text_sm().child("数量"))
                        .child(num_input(&self.int_count_state, 100.0))
                        .child(
                            Checkbox::new("int-dup")
                                .label("允许重复")
                                .checked(self.int_allow_dup)
                                .on_click(cx.listener(|this, v: &bool, _, cx| {
                                    this.int_allow_dup = *v;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("gen-int")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_int(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.int_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-int")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.int_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-int")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.int_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
            // 小数
            .child(
                design::card(cx)
                    .child(design::caption("小数", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("最小值"))
                        .child(num_input(&self.float_min_state, 140.0))
                        .child(div().text_sm().child("最大值"))
                        .child(num_input(&self.float_max_state, 140.0))
                        .child(div().text_sm().child("小数位数"))
                        .child(num_input(&self.float_dec_state, 80.0))
                        .child(div().text_sm().child("数量"))
                        .child(num_input(&self.float_count_state, 100.0))
                        .child(
                            Checkbox::new("float-dup")
                                .label("允许重复")
                                .checked(self.float_allow_dup)
                                .on_click(cx.listener(|this, v: &bool, _, cx| {
                                    this.float_allow_dup = *v;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("gen-float")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_float(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.float_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-float")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.float_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-float")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.float_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
            // 素数
            .child(
                design::card(cx)
                    .child(design::caption("素数", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("下限"))
                        .child(num_input(&self.prime_lo_state, 140.0))
                        .child(div().text_sm().child("上限"))
                        .child(num_input(&self.prime_hi_state, 140.0))
                        .child(div().text_sm().child("数量"))
                        .child(num_input(&self.prime_count_state, 100.0))
                        .child(
                            Button::new("gen-prime")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_prime(window, cx);
                                })),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xef4444))
                                .child(self.status.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.prime_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-prime")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.prime_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-prime")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.prime_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
            // 十六进制
            .child(
                design::card(cx)
                    .child(design::caption("十六进制", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("长度"))
                        .child(num_input(&self.hex_len_state, 140.0))
                        .child(div().text_sm().child("数量"))
                        .child(num_input(&self.hex_count_state, 100.0))
                        .child(
                            Button::new("gen-hex")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_hex(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.hex_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-hex")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.hex_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-hex")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.hex_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
            // 二进制
            .child(
                design::card(cx)
                    .child(design::caption("二进制", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("位数"))
                        .child(num_input(&self.bin_bits_state, 140.0))
                        .child(div().text_sm().child("数量"))
                        .child(num_input(&self.bin_count_state, 100.0))
                        .child(
                            Button::new("gen-bin")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_bin(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.bin_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-bin")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.bin_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-bin")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.bin_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
            // 字节
            .child(
                design::card(cx)
                    .child(design::caption("字节", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .child(div().w(label_w).text_sm().child("每行字节数"))
                        .child(num_input(&self.byte_per_line_state, 140.0))
                        .child(div().text_sm().child("行数"))
                        .child(num_input(&self.byte_lines_state, 100.0))
                        .child(
                            Button::new("gen-byte")
                                .label("生成")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.gen_byte(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(Textarea::new(&self.byte_out_state).h(px(100.0)).flex_1())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    Button::new("copy-byte")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            let state = this.byte_out_state.clone();
                                            this.copy_output(&state, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-byte")
                                        .icon(Icon::new(IconName::Close))
                                        .tooltip("清除")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let state = this.byte_out_state.clone();
                                            this.clear_output(&state, window, cx);
                                        })),
                                ),
                        ),
                )
            )
    }
}

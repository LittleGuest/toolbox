use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    input::{Input, InputEvent, InputState},
    *,
};
use rand::Rng;

type Rgb = (u8, u8, u8);

const INIT_HEX: &str = "#3498db";

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn fmt_num(v: f64) -> String {
    let r = round1(v);
    if (r - r.trunc()).abs() < f64::EPSILON {
        format!("{}", r as i64)
    } else {
        format!("{:.1}", r)
    }
}

fn hex_to_rgb(input: &str) -> Option<Rgb> {
    let h = input.trim().trim_start_matches('#');
    let full = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => h.to_string(),
        _ => return None,
    };
    if !full.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some((
        u8::from_str_radix(&full[0..2], 16).ok()?,
        u8::from_str_radix(&full[2..4], 16).ok()?,
        u8::from_str_radix(&full[4..6], 16).ok()?,
    ))
}

fn rgb_to_hex((r, g, b): Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Hsla → "#rrggbb"（取色器回调使用）
fn hsla_to_hex(color: Hsla) -> String {
    let v = u32::from(Rgba::from(color));
    format!("#{:06x}", v & 0xffffff)
}

fn rgb_to_hsv((r, g, b): Rgb) -> (f64, f64, f64) {
    let (rr, gg, bb) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = rr.max(gg).max(bb);
    let min = rr.min(gg).min(bb);
    let d = max - min;
    let mut h = 0.0;
    if d != 0.0 {
        if max == rr {
            h = 60.0 * (((gg - bb) / d) % 6.0);
        } else if max == gg {
            h = 60.0 * ((bb - rr) / d + 2.0);
        } else {
            h = 60.0 * ((rr - gg) / d + 4.0);
        }
    }
    if h < 0.0 {
        h += 360.0;
    }
    let s = if max == 0.0 { 0.0 } else { (d / max) * 100.0 };
    (h, s, max * 100.0)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgb {
    let hh = ((h % 360.0) + 360.0) % 360.0;
    let ss = clamp(s, 0.0, 100.0) / 100.0;
    let vv = clamp(v, 0.0, 100.0) / 100.0;
    let c = vv * ss;
    let x = c * (1.0 - (((hh / 60.0) % 2.0) - 1.0).abs());
    let m = vv - c;
    let (r, g, b) = if hh < 60.0 {
        (c, x, 0.0)
    } else if hh < 120.0 {
        (x, c, 0.0)
    } else if hh < 180.0 {
        (0.0, c, x)
    } else if hh < 240.0 {
        (0.0, x, c)
    } else if hh < 300.0 {
        (c, 0.0, x)
    } else {
        (x, 0.0, c)
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

fn rgb_to_cmyk((r, g, b): Rgb) -> (f64, f64, f64, f64) {
    let (rr, gg, bb) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let k = 1.0 - rr.max(gg).max(bb);
    if k >= 1.0 {
        return (0.0, 0.0, 0.0, 100.0);
    }
    (
        ((1.0 - rr - k) / (1.0 - k)) * 100.0,
        ((1.0 - gg - k) / (1.0 - k)) * 100.0,
        ((1.0 - bb - k) / (1.0 - k)) * 100.0,
        k * 100.0,
    )
}

fn cmyk_to_rgb(c: f64, m: f64, y: f64, k: f64) -> Rgb {
    let (cc, mm, yy, kk) = (
        clamp(c, 0.0, 100.0) / 100.0,
        clamp(m, 0.0, 100.0) / 100.0,
        clamp(y, 0.0, 100.0) / 100.0,
        clamp(k, 0.0, 100.0) / 100.0,
    );
    (
        (255.0 * (1.0 - cc) * (1.0 - kk)).round() as u8,
        (255.0 * (1.0 - mm) * (1.0 - kk)).round() as u8,
        (255.0 * (1.0 - yy) * (1.0 - kk)).round() as u8,
    )
}

fn parse_list(input: &str, count: usize) -> Option<Vec<f64>> {
    let parts: Vec<&str> = input.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    if parts.len() != count {
        return None;
    }
    parts.iter().map(|p| p.parse::<f64>().ok()).collect()
}

fn parse_rgb(input: &str) -> Option<Rgb> {
    let nums = parse_list(input, 3)?;
    let (r, g, b) = (nums[0], nums[1], nums[2]);
    if !([r, g, b].iter().all(|n| (0.0..=255.0).contains(n))) {
        return None;
    }
    Some((r.round() as u8, g.round() as u8, b.round() as u8))
}

fn parse_hsv(input: &str) -> Option<(f64, f64, f64)> {
    let nums = parse_list(input, 3)?;
    let (h, s, v) = (nums[0], nums[1], nums[2]);
    if !(0.0..=360.0).contains(&h) || !(0.0..=100.0).contains(&s) || !(0.0..=100.0).contains(&v) {
        return None;
    }
    Some((h, s, v))
}

fn parse_cmyk(input: &str) -> Option<(f64, f64, f64, f64)> {
    let nums = parse_list(input, 4)?;
    if !nums.iter().all(|n| (0.0..=100.0).contains(n)) {
        return None;
    }
    Some((nums[0], nums[1], nums[2], nums[3]))
}

pub struct ColorConverter {
    hex: String,
    rgb: String,
    hsv: String,
    cmyk: String,
    current_rgb: Option<Rgb>,
    invalid: Option<&'static str>,
    hex_state: Entity<InputState>,
    rgb_state: Entity<InputState>,
    hsv_state: Entity<InputState>,
    cmyk_state: Entity<InputState>,
    picker_state: Entity<ColorPickerState>,
    _subscriptions: Vec<Subscription>,
}

impl ColorConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        let hex_state = input(window, cx, "#rrggbb 或 #rgb");
        let rgb_state = input(window, cx, "r, g, b（0-255）");
        let hsv_state = input(window, cx, "h, s, v（0-360, 0-100, 0-100）");
        let cmyk_state = input(window, cx, "c, m, y, k（0-100）");

        let picker_state = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_value(Hsla::from(rgb(0x3498db)))
        });

        let sources: Vec<(&'static str, Entity<InputState>)> = vec![
            ("hex", hex_state.clone()),
            ("rgb", rgb_state.clone()),
            ("hsv", hsv_state.clone()),
            ("cmyk", cmyk_state.clone()),
        ];
        let mut _subscriptions = sources
            .into_iter()
            .map(|(source, state)| {
                cx.subscribe_in(&state, window, move |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::Change = ev {
                        this.recompute(source, window, cx);
                    }
                })
            })
            .collect::<Vec<_>>();

        _subscriptions.push(cx.subscribe_in(
            &picker_state,
            window,
            move |this, _, ev: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = ev {
                    let hex = hsla_to_hex(*color);
                    this.hex_state.update(cx, |state, cx| {
                        state.set_value(hex, window, cx);
                    });
                    this.recompute("hex", window, cx);
                }
            },
        ));

        let mut this = Self {
            hex: String::new(),
            rgb: String::new(),
            hsv: String::new(),
            cmyk: String::new(),
            current_rgb: None,
            invalid: None,
            hex_state,
            rgb_state,
            hsv_state,
            cmyk_state,
            picker_state,
            _subscriptions,
        };
        if let Some(rgb) = hex_to_rgb(INIT_HEX) {
            this.fill_all(rgb, window, cx);
        }
        this
    }

    fn fill_all(&mut self, rgb: Rgb, window: &mut Window, cx: &mut Context<Self>) {
        self.current_rgb = Some(rgb);
        self.invalid = None;
        self.hex = rgb_to_hex(rgb);
        self.rgb = format!("{}, {}, {}", rgb.0, rgb.1, rgb.2);
        let (h, s, v) = rgb_to_hsv(rgb);
        self.hsv = format!("{}, {}, {}", fmt_num(h), fmt_num(s), fmt_num(v));
        let (c, m, y, k) = rgb_to_cmyk(rgb);
        self.cmyk = format!("{}, {}, {}, {}", fmt_num(c), fmt_num(m), fmt_num(y), fmt_num(k));

        let values = [
            (self.hex_state.clone(), self.hex.clone()),
            (self.rgb_state.clone(), self.rgb.clone()),
            (self.hsv_state.clone(), self.hsv.clone()),
            (self.cmyk_state.clone(), self.cmyk.clone()),
        ];
        for (state, value) in values {
            state.update(cx, |s, cx| {
                s.set_value(value.clone(), window, cx);
            });
        }
        cx.notify();
    }

    fn recompute(&mut self, source: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        let read = |state: &Entity<InputState>| state.read(cx).value().to_string();
        let rgb: Option<Rgb> = match source {
            "hex" => hex_to_rgb(&read(&self.hex_state)),
            "rgb" => parse_rgb(&read(&self.rgb_state)),
            "hsv" => parse_hsv(&read(&self.hsv_state)).map(|(h, s, v)| hsv_to_rgb(h, s, v)),
            "cmyk" => parse_cmyk(&read(&self.cmyk_state)).map(|(c, m, y, k)| cmyk_to_rgb(c, m, y, k)),
            _ => None,
        };
        match rgb {
            Some(rgb) => self.fill_all(rgb, window, cx),
            None => {
                self.invalid = Some(source);
                cx.notify();
            }
        }
    }

    fn random_color(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut rng = rand::rng();
        let rgb = (
            rng.random_range(0..=255u8),
            rng.random_range(0..=255u8),
            rng.random_range(0..=255u8),
        );
        self.fill_all(rgb, window, cx);
    }

    fn copy_hex(&mut self, cx: &mut Context<Self>) {
        if !self.hex.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.hex.clone()));
        }
    }
}

impl Render for ColorConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(56.0);
        let current = self.current_rgb.unwrap_or((255, 255, 255));
        let (r, g, b) = current;
        let luminance = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
        let text_color = if luminance > 128.0 {
            rgb(0x111111)
        } else {
            rgb(0xffffff)
        };
        let preview_hex = if self.hex.is_empty() {
            "#ffffff".to_string()
        } else {
            self.hex.clone()
        };

        let fmt_row = |label: &'static str,
                       state: &Entity<InputState>,
                       field: &'static str,
                       cx: &mut Context<Self>| {
            let invalid = self.invalid == Some(field);
            div()
                .flex()
                .items_start()
                .gap_3()
                .child(design::editor_label(label, cx).w(label_w))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex_col()
                        .gap_1()
                        .child(Input::new(state))
                        .when(invalid, |this| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                                    .child("格式无效"),
                            )
                        }),
                )
        };

        design::page()

            .child(
                design::card(cx)
                    // 预览色块（preview-block）
                    .child(
                        div()
                            .h(px(110.0))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(text_color)
                            .font_family("monospace")
                            .text_size(px(16.0))
                            .font_semibold()
                            .child(preview_hex),
                    )
                    // 取色器行（preview-actions）
                    .child(
                        design::action_row()
                            .mt_3()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("取色器"),
                            )
                            .child(ColorPicker::new(&self.picker_state))
                            .child(
                                Button::new("random-color")
                                    .primary()
                                    .icon(Icon::new(IconName::RotateCw))
                                    .tooltip("随机颜色")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.random_color(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-hex")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制 HEX")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_hex(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .mt_3()
                            .flex_col()
                            .gap_2p5()
                            .child(fmt_row("HEX", &self.hex_state, "hex", cx))
                            .child(fmt_row("RGB", &self.rgb_state, "rgb", cx))
                            .child(fmt_row("HSV", &self.hsv_state, "hsv", cx))
                            .child(fmt_row("CMYK", &self.cmyk_state, "cmyk", cx)),
                    ),
            )
    }
}

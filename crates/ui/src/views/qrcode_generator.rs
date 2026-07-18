use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    radio::{Radio, RadioGroup},
    slider::{Slider, SliderEvent, SliderState},
    *,
};
use qrcode_generator::QrCodeEcc;

/// 纠错级别
#[derive(Clone, Copy, Debug, PartialEq)]
enum EccLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl EccLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::Quartile => "Q",
            Self::High => "H",
        }
    }

    fn to_ecc(self) -> QrCodeEcc {
        match self {
            Self::Low => QrCodeEcc::Low,
            Self::Medium => QrCodeEcc::Medium,
            Self::Quartile => QrCodeEcc::Quartile,
            Self::High => QrCodeEcc::High,
        }
    }
}

const ECC_LEVELS: [EccLevel; 4] = [
    EccLevel::Low,
    EccLevel::Medium,
    EccLevel::Quartile,
    EccLevel::High,
];

pub struct QrCodeGenerator {
    text: String,
    size: usize,
    margin: usize,
    ecc_level: EccLevel,
    dark_color: String,
    light_color: String,
    svg: String,
    png_data: Option<Vec<u8>>,
    error: String,
    input_state: Entity<InputState>,
    size_slider: Entity<SliderState>,
    margin_state: Entity<InputState>,
    dark_color_state: Entity<InputState>,
    light_color_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl QrCodeGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入文本、链接或其它需要编码的内容...")
                .default_value("https://github.com/")
                .multi_line(true)
        });

        let size_slider = cx.new(|_| {
            SliderState::new()
                .max(720.0)
                .min(120.0)
                .step(20.0)
                .default_value(260.0)
        });

        let margin_state = cx.new(|cx| InputState::new(window, cx).default_value("2".to_string()));

        let dark_color_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("#000000")
                .default_value("#000000".to_string())
        });

        let light_color_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("#ffffff")
                .default_value("#ffffff".to_string())
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.text = input_state.read(cx).value().to_string();
                        this.generate();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&size_slider, window, |this, _, _: &SliderEvent, _, cx| {
                this.size = this.size_slider.read(cx).value().end() as usize;
                this.size = (this.size / 20) * 20; // 对齐步长
                this.generate();
                cx.notify();
            }),
            cx.subscribe_in(&margin_state, window, {
                let margin_state = margin_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        if let Ok(val) = margin_state.read(cx).value().parse::<usize>() {
                            this.margin = val.clamp(0, 10);
                        }
                        this.generate();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&margin_state, window, {
                let margin_state = margin_state.clone();
                move |this, state, ev: &NumberInputEvent, window, cx| {
                    if let NumberInputEvent::Step(action) = ev {
                        let text = state.read(cx).value();
                        let mut val = text.parse::<usize>().unwrap_or(2);
                        match action {
                            StepAction::Increment => val = val.saturating_add(1).min(10),
                            StepAction::Decrement => val = val.saturating_sub(1),
                        }
                        state.update(cx, |state, cx| {
                            state.set_value(val.to_string(), window, cx);
                        });
                        this.margin = val;
                        this.generate();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&dark_color_state, window, {
                let dark_color_state = dark_color_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.dark_color = dark_color_state.read(cx).value().to_string();
                        this.generate();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&light_color_state, window, {
                let light_color_state = light_color_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.light_color = light_color_state.read(cx).value().to_string();
                        this.generate();
                        cx.notify();
                    }
                }
            }),
        ];

        let mut this = Self {
            text: "https://github.com/".to_string(),
            size: 260,
            margin: 2,
            ecc_level: EccLevel::Medium,
            dark_color: "#000000".to_string(),
            light_color: "#ffffff".to_string(),
            svg: String::new(),
            png_data: None,
            error: String::new(),
            input_state,
            size_slider,
            margin_state,
            dark_color_state,
            light_color_state,
            _subscriptions,
        };
        this.generate();
        this
    }

    fn generate(&mut self) {
        self.error.clear();
        if self.text.trim().is_empty() {
            self.svg.clear();
            self.png_data = None;
            return;
        }

        let ecc = self.ecc_level.to_ecc();
        let size = self.size.max(120);

        // 生成 SVG
        match qrcode_generator::to_svg_to_string_from_str(&self.text, ecc, size, None::<&str>) {
            Ok(svg) => self.svg = svg,
            Err(err) => {
                self.svg.clear();
                self.png_data = None;
                self.error = err.to_string();
                return;
            }
        }

        // 生成 PNG
        match qrcode_generator::to_png_to_vec_from_str(&self.text, ecc, size) {
            Ok(data) => self.png_data = Some(data),
            Err(err) => {
                self.png_data = None;
                self.error = format!("PNG 生成失败：{err}");
            }
        }
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.text = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.text.clone(), window, cx);
                });
                self.generate();
                cx.notify();
            }
        }
    }

    fn copy_data_url(&mut self, cx: &mut Context<Self>) {
        if let Some(ref png_data) = self.png_data {
            let data_url = format!("data:image/png;base64,{}", base64_encode(png_data));
            cx.write_to_clipboard(ClipboardItem::new_string(data_url));
        }
    }

    fn copy_svg(&mut self, cx: &mut Context<Self>) {
        if !self.svg.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.svg.clone()));
        }
    }

    fn download_png(&mut self, cx: &mut Context<Self>) {
        let Some(ref png_data) = self.png_data else {
            return;
        };
        let png_data = png_data.clone();

        cx.spawn(async move |_, cx| {
            let file_path = cx
                .background_executor()
                .spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("qrcode.png")
                        .save_file()
                        .await
                })
                .await;

            if let Some(path) = file_path {
                let _ = cx
                    .background_executor()
                    .spawn(async move { std::fs::write(path.path(), png_data) })
                    .await;
            }
        })
        .detach();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.text.clear();
        self.svg.clear();
        self.png_data = None;
        self.error.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
    }

    fn set_ecc_level(&mut self, level: EccLevel, cx: &mut Context<Self>) {
        self.ecc_level = level;
        self.generate();
        cx.notify();
    }
}

/// 简易 base64 编码
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// 将 hex 颜色（#rrggbb）转换为 Hsla
fn hex_to_hsla(hex: &str) -> Option<Hsla> {
    let hex = hex.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-6 {
        return Some(hsla(0.0, 0.0, l, 1.0));
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < 1e-6 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    Some(hsla(h, s, l, 1.0))
}

impl Render for QrCodeGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ecc_index = ECC_LEVELS.iter().position(|&l| l == self.ecc_level);

        // 预览面板：用 SVG 渲染二维码图形
        let preview = if self.svg.is_empty() {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("点击生成按钮生成二维码...")
        } else {
            // 使用 canvas 绘制 SVG 内容
            let svg = self.svg.clone();
            let dark_color = hex_to_hsla(&self.dark_color).unwrap_or(gpui::black());
            let light_color = hex_to_hsla(&self.light_color).unwrap_or(gpui::white());
            let qr_size = self.size;

            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(
                    canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, _window, window, _cx| {
                            // 绘制背景
                            window.paint_quad(fill(bounds, light_color));

                            // 解析 SVG 中 QR 码模块坐标并绘制
                            // qrcode-generator SVG 使用 rect 元素，我们直接解析 viewBox 和 rect
                            let viewbox_size = qr_size as f32;
                            let scale = f32::from(bounds.size.width) / viewbox_size;

                            // 从 SVG 提取 rect 的 x,y,width,height
                            for line in svg.lines() {
                                let line = line.trim();
                                if !line.starts_with('<') || !line.contains("rect") {
                                    continue;
                                }
                                let mut x: Option<f32> = None;
                                let mut y: Option<f32> = None;
                                let mut w: Option<f32> = None;
                                let mut h: Option<f32> = None;

                                // 简易属性解析
                                for part in line.split_whitespace() {
                                    if let Some(val) = part.strip_prefix("x=\"") {
                                        x = val.trim_end_matches('"').parse().ok();
                                    } else if let Some(val) = part.strip_prefix("y=\"") {
                                        y = val.trim_end_matches('"').parse().ok();
                                    } else if let Some(val) = part.strip_prefix("width=\"") {
                                        w = val.trim_end_matches('"').parse().ok();
                                    } else if let Some(val) = part.strip_prefix("height=\"") {
                                        h = val.trim_end_matches('"').parse().ok();
                                    }
                                }

                                if let (Some(rx), Some(ry), Some(rw), Some(rh)) = (x, y, w, h) {
                                    let px_x = f32::from(bounds.origin.x) + rx * scale;
                                    let px_y = f32::from(bounds.origin.y) + ry * scale;
                                    let px_w = rw * scale;
                                    let px_h = rh * scale;
                                    let rect_bounds = Bounds::new(
                                        point(px(px_x), px(px_y)),
                                        size(px(px_w), px(px_h)),
                                    );
                                    window.paint_quad(fill(rect_bounds, dark_color));
                                }
                            }
                        },
                    )
                    .size_full(),
                )
        };

        let error_msg = if self.error.is_empty() {
            None
        } else {
            Some(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(self.error.clone()),
            )
        };

        div()
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    // 左栏：参数表单
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .min_w(px(320.))
                            // 内容
                            .child(
                                div()
                                    .flex()
                                    .items_start()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("内容"))
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(Input::new(&self.input_state).h(px(180.0))),
                                    ),
                            )
                            // 尺寸
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("尺寸"))
                                    .child(div().flex_1().child(Slider::new(&self.size_slider))),
                            )
                            // 边距 → NumberInput（匹配 Tauri n-input-number）
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("边距"))
                                    .child(
                                        div().flex_1().child(NumberInput::new(&self.margin_state)),
                                    ),
                            )
                            // 纠错级别
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("纠错级别"))
                                    .child(
                                        RadioGroup::horizontal("ecc-group")
                                            .selected_index(ecc_index)
                                            .on_click(cx.listener(|this, idx: &usize, _, cx| {
                                                if let Some(level) = ECC_LEVELS.get(*idx) {
                                                    this.set_ecc_level(*level, cx);
                                                }
                                            }))
                                            .children(
                                                ECC_LEVELS.iter().map(|l| Radio::new(l.label())),
                                            ),
                                    ),
                            )
                            // 前景色
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("前景色"))
                                    .child(
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .rounded_md()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .bg(hex_to_hsla(&self.dark_color)
                                                .unwrap_or(gpui::black())),
                                    )
                                    .child(
                                        div().flex_1().child(Input::new(&self.dark_color_state)),
                                    ),
                            )
                            // 背景色
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(px(110.0)).text_sm().child("背景色"))
                                    .child(
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .rounded_md()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .bg(hex_to_hsla(&self.light_color)
                                                .unwrap_or(gpui::white())),
                                    )
                                    .child(
                                        div().flex_1().child(Input::new(&self.light_color_state)),
                                    ),
                            )
                            // 操作按钮（匹配 Tauri n-space: 生成 + 粘贴内容 + 复制 Data URL + 下载 PNG）
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("generate")
                                            .primary()
                                            .icon(Icon::new(IconName::Plus))
                                            .tooltip("生成")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.generate();
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("paste")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴内容")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.paste(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-data-url")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制 Data URL")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_data_url(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("download-png")
                                            .icon(Icon::new(IconName::ArrowDown))
                                            .tooltip("下载 PNG")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.download_png(cx);
                                            })),
                                    ),
                            )
                            .children(error_msg),
                    )
                    // 右栏：图形化预览
                    .child(
                        div()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .p_3()
                            .h(px(360.0))
                            .child(preview),
                    ),
            )
    }
}

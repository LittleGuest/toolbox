use gpui_kit::{
    component::{
        button::*,
        input::{
            Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, Textarea,
            TextareaState,
        },
        slider::{Slider, SliderEvent, SliderState},
        *,
    },
    *,
};
use qrcode_generator::QrCodeEcc;

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
    matrix: Vec<Vec<bool>>,
    png_data: Option<Vec<u8>>,
    error: String,
    status: String,
    input_state: Entity<TextareaState>,
    size_slider: Entity<SliderState>,
    margin_state: Entity<InputState>,
    dark_color_state: Entity<InputState>,
    light_color_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl QrCodeGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("输入文本、链接或其它需要编码的内容...")
                .default_value("https://github.com/")
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
                this.size = (this.size / 20) * 20;
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
                move |this, state, ev: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = ev;
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
            matrix: Vec::new(),
            png_data: None,
            error: String::new(),
            status: String::new(),
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
        self.status.clear();
        if self.text.trim().is_empty() {
            self.matrix.clear();
            self.png_data = None;
            return;
        }

        let ecc = self.ecc_level.to_ecc();

        match qrcode_generator::to_matrix_from_str(&self.text, ecc) {
            Ok(matrix) => self.matrix = matrix,
            Err(err) => {
                self.matrix.clear();
                self.png_data = None;
                self.error = err.to_string();
                return;
            }
        }

        let size = self.size.max(120);
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
        match &self.png_data {
            Some(png_data) => {
                let data_url = format!("data:image/png;base64,{}", base64_encode(png_data));
                cx.write_to_clipboard(ClipboardItem::new_string(data_url));
                self.status = "已复制二维码 Data URL".to_string();
            }
            None => {
                self.status = "请先生成二维码".to_string();
            }
        }
        cx.notify();
    }

    fn download_png(&mut self, cx: &mut Context<Self>) {
        let Some(ref png_data) = self.png_data else {
            self.status = "请先生成二维码".to_string();
            cx.notify();
            return;
        };
        let png_data = png_data.clone();
        let default_name = format!(
            "qrcode-{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );

        cx.spawn(async move |_, cx| {
            let file_path = cx
                .background_executor()
                .spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name(&default_name)
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
        self.matrix.clear();
        self.png_data = None;
        self.error.clear();
        self.status.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.generate();
        cx.notify();
    }

    fn set_ecc_level(&mut self, level: EccLevel, cx: &mut Context<Self>) {
        self.ecc_level = level;
        self.generate();
        cx.notify();
    }

    /// 分段选择器（对应 n-radio-button 组）
    fn segmented(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(ECC_LEVELS.iter().enumerate().map(|(i, level)| {
                let active = *level == self.ecc_level;
                let level = *level;
                div()
                    .id(("ecc", i))
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
                        this.set_ecc_level(level, cx);
                    }))
                    .child(level.label().to_string())
            }))
    }
}

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
        let status_color = if self.error.is_empty() && !self.status.is_empty() {
            Hsla::from(rgb(design::OK_GREEN))
        } else if !self.status.is_empty() {
            Hsla::from(rgb(design::WARN_AMBER))
        } else {
            Hsla::from(rgb(design::ERROR_RED))
        };
        let feedback = if !self.error.is_empty() {
            Some(self.error.clone())
        } else if !self.status.is_empty() {
            Some(self.status.clone())
        } else {
            None
        };

        // 预览：矩阵自绘（支持边距与前景/背景色）
        let preview = if self.matrix.is_empty() {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("输入内容后自动生成二维码")
        } else {
            let matrix = self.matrix.clone();
            let qr_size = self.size.max(120) as f32;
            let margin = self.margin as f32;
            let dark_color = hex_to_hsla(&self.dark_color).unwrap_or(gpui_kit::black());
            let light_color = hex_to_hsla(&self.light_color).unwrap_or(gpui_kit::white());

            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(
                    canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, _window, window, _cx| {
                            window.paint_quad(fill(bounds, light_color));

                            let modules = matrix.len() as f32;
                            if modules <= 0.0 {
                                return;
                            }
                            let total = modules + margin * 2.0;
                            let module_vu = qr_size / total;
                            let scale = f32::from(bounds.size.width) / qr_size;

                            for (i, row) in matrix.iter().enumerate() {
                                for (j, &on) in row.iter().enumerate() {
                                    if !on {
                                        continue;
                                    }
                                    let vx = (margin + j as f32) * module_vu;
                                    let vy = (margin + i as f32) * module_vu;
                                    let rect = Bounds::new(
                                        point(
                                            px(f32::from(bounds.origin.x) + vx * scale),
                                            px(f32::from(bounds.origin.y) + vy * scale),
                                        ),
                                        size(px(module_vu * scale), px(module_vu * scale)),
                                    );
                                    window.paint_quad(fill(rect, dark_color));
                                }
                            }
                        },
                    )
                    .size_full(),
                )
        };

        // 表单列（对应 qr-form）
        let form = div()
            .flex()
            .flex_col()
            .gap_3()
            .min_w(px(300.0))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(design::caption("内容", cx).w(px(80.0)).pt(px(6.0)))
                    .child(
                        div().flex_1().child(
                            Textarea::new(&self.input_state)
                                .h(px(160.0))
                                .font_family("monospace"),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("尺寸", cx).w(px(80.0)))
                    .child(div().flex_1().child(Slider::new(&self.size_slider))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("边距", cx).w(px(80.0)))
                    .child(
                        div()
                            .w(px(120.0))
                            .child(NumberInput::new(&self.margin_state)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("纠错级别", cx).w(px(80.0)))
                    .child(self.segmented(cx)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("前景色", cx).w(px(80.0)))
                    .child(
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(hex_to_hsla(&self.dark_color).unwrap_or(gpui_kit::black())),
                    )
                    .child(div().flex_1().child(Input::new(&self.dark_color_state))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("背景色", cx).w(px(80.0)))
                    .child(
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(hex_to_hsla(&self.light_color).unwrap_or(gpui_kit::white())),
                    )
                    .child(div().flex_1().child(Input::new(&self.light_color_state))),
            );

        design::page().child(
            design::card(cx)
                .child(design::card_header(
                    IconName::Frame,
                    "二维码生成",
                    "生成二维码",
                    cx,
                ))
                // 头部动作行（tb-card-header-actions）
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("generate")
                                .primary()
                                .icon(Icon::new(IconName::Asterisk))
                                .tooltip("生成")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.generate();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("paste")
                                .icon(Icon::new(IconName::Inbox))
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
                        )
                        .child(
                            Button::new("clear")
                                .icon(Icon::new(IconName::Close))
                                .tooltip("清除")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.clear(window, cx);
                                })),
                        ),
                )
                // 双栏主体（qr-body：表单 | 预览）
                .child(
                    div().flex().items_start().gap_4().child(form).child(
                        div()
                            .flex_1()
                            .min_w(px(280.0))
                            .flex_col()
                            .min_h(px(360.0))
                            .border_1()
                            .border_dashed()
                            .border_color(cx.theme().border)
                            .rounded(px(10.0))
                            .bg(cx.theme().background)
                            .p(px(14.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(cx.theme().muted_foreground)
                                    .mb_2()
                                    .child("预览"),
                            )
                            .child(div().flex_1().min_h(px(300.0)).child(preview)),
                    ),
                )
                .children(feedback.map(|text| {
                    div()
                        .text_size(px(12.5))
                        .text_color(status_color)
                        .child(text)
                })),
        )
    }
}

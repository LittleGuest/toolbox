use gpui::*;
use gpui_component::{button::*, *};

pub struct SystemMonitor {
    memory_usage: f32,
    cpu_usage: Vec<f32>,
    is_loaded: bool,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            memory_usage: 0.0,
            cpu_usage: Vec::new(),
            is_loaded: false,
        }
    }

    fn refresh(&mut self) {
        // Simulate system info loading
        self.memory_usage = 45.5;
        self.cpu_usage = vec![10.2, 15.5, 8.7, 12.3];
        self.is_loaded = true;
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("系统监控")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        Button::new("refresh")
                            .primary()
                            .label("刷新")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh();
                                cx.notify();
                            }))
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("内存使用")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(format!("{:.2}%", self.memory_usage))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("CPU 使用")
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .children(
                                        self.cpu_usage.iter().enumerate().map(|(i, usage)| {
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .child(format!("CPU {}:", i))
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .px_2()
                                                        .py_1()
                                                        .text_sm()
                                                        .font_family("monospace")
                                                        .child(format!("{:.2}%", usage))
                                                )
                                        }).collect::<Vec<_>>()
                                    )
                            )
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                if self.is_loaded {
                                    div().child("系统信息已加载")
                                } else {
                                    div().child("点击刷新按钮加载系统信息")
                                }
                            )
                    )
            )
    }
}

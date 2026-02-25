use gpui::*;
use gpui_component::{button::*, *};
use std::sync::{LazyLock, Mutex};
use sysinfo::{Disks, MemoryRefreshKind, System, Networks};

static SYS: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new_all()));
static DISKS: LazyLock<Mutex<Disks>> = LazyLock::new(|| Mutex::new(Disks::new_with_refreshed_list()));
static NETWORKS: LazyLock<Mutex<Networks>> = LazyLock::new(|| Mutex::new(Networks::new_with_refreshed_list()));

pub struct SystemMonitor {
    cpu_usage: Vec<f32>,
    total_memory: u64,
    used_memory: u64,
    memory_usage_percent: f32,
    total_swap: u64,
    used_swap: u64,
    swap_usage_percent: f32,
    disks: Vec<DiskInfo>,
    networks: Vec<NetworkInfo>,
    system_name: String,
    kernel_version: String,
    os_version: String,
}

#[derive(Clone)]
struct DiskInfo {
    name: String,
    total: u64,
    used: u64,
    usage_percent: f32,
    mount_point: String,
}

#[derive(Clone)]
struct NetworkInfo {
    name: String,
    received: u64,
    transmitted: u64,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            cpu_usage: Vec::new(),
            total_memory: 0,
            used_memory: 0,
            memory_usage_percent: 0.0,
            total_swap: 0,
            used_swap: 0,
            swap_usage_percent: 0.0,
            disks: Vec::new(),
            networks: Vec::new(),
            system_name: String::new(),
            kernel_version: String::new(),
            os_version: String::new(),
        };
        monitor.refresh();
        monitor
    }

    fn refresh(&mut self) {
        {
            let mut sys = SYS.lock().unwrap();
            sys.refresh_all();
            sys.refresh_memory_specifics(MemoryRefreshKind::everything());
            
            self.cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            self.total_memory = sys.total_memory();
            self.used_memory = sys.used_memory();
            self.memory_usage_percent = if self.total_memory > 0 {
                (self.used_memory as f64 / self.total_memory as f64 * 100.0) as f32
            } else {
                0.0
            };
            self.total_swap = sys.total_swap();
            self.used_swap = sys.used_swap();
            self.swap_usage_percent = if self.total_swap > 0 {
                (self.used_swap as f64 / self.total_swap as f64 * 100.0) as f32
            } else {
                0.0
            };
        }

        {
            let mut disks = DISKS.lock().unwrap();
            disks.refresh(true);
            self.disks = disks
                .iter()
                .map(|disk| {
                    let total = disk.total_space();
                    let available = disk.available_space();
                    let used = total.saturating_sub(available);
                    let usage_percent = if total > 0 {
                        (used as f64 / total as f64 * 100.0) as f32
                    } else {
                        0.0
                    };
                    DiskInfo {
                        name: disk.name().to_string_lossy().to_string(),
                        total,
                        used,
                        usage_percent,
                        mount_point: disk.mount_point().to_string_lossy().to_string(),
                    }
                })
                .collect();
        }

        {
            let mut networks = NETWORKS.lock().unwrap();
            networks.refresh(true);
            self.networks = networks
                .iter()
                .map(|(name, data)| NetworkInfo {
                    name: name.clone(),
                    received: data.received(),
                    transmitted: data.transmitted(),
                })
                .collect();
        }

        self.system_name = System::name().unwrap_or_default();
        self.kernel_version = System::kernel_version().unwrap_or_default();
        self.os_version = System::os_version().unwrap_or_default();
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    fn render_progress_bar(&self, percent: f32, cx: &mut Context<Self>) -> Div {
        let bar_color = if percent < 50.0 {
            cx.theme().accent
        } else if percent < 80.0 {
            gpui::rgb(0xf59e0b).into()
        } else {
            gpui::rgb(0xef4444).into()
        };

        div()
            .flex_1()
            .h_2()
            .rounded_md()
            .bg(cx.theme().border)
            .child(
                div()
                    .h_full()
                    .w(relative(percent / 100.0))
                    .rounded_md()
                    .bg(bar_color),
            )
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cpu_usage = self.cpu_usage.clone();
        let total_memory = self.total_memory;
        let used_memory = self.used_memory;
        let memory_usage_percent = self.memory_usage_percent;
        let total_swap = self.total_swap;
        let used_swap = self.used_swap;
        let swap_usage_percent = self.swap_usage_percent;
        let disks = self.disks.clone();
        let networks = self.networks.clone();
        let system_name = self.system_name.clone();
        let kernel_version = self.kernel_version.clone();
        let os_version = self.os_version.clone();

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("系统监控"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        Button::new("refresh")
                            .primary()
                            .icon(Icon::new(IconName::Asterisk))
                            .tooltip("刷新")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh();
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().font_semibold().mb_2().child("系统信息"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("系统名称"))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_family("monospace")
                                                    .child(if system_name.is_empty() {
                                                        "-".to_string()
                                                    } else {
                                                        system_name.clone()
                                                    }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("内核版本"))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_family("monospace")
                                                    .child(if kernel_version.is_empty() {
                                                        "-".to_string()
                                                    } else {
                                                        kernel_version.clone()
                                                    }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("系统版本"))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_family("monospace")
                                                    .child(if os_version.is_empty() {
                                                        "-".to_string()
                                                    } else {
                                                        os_version.clone()
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().font_semibold().mb_2().child("CPU 使用率"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("总体"))
                                            .child(self.render_progress_bar(
                                                if cpu_usage.is_empty() {
                                                    0.0
                                                } else {
                                                    cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
                                                },
                                                cx,
                                            ))
                                            .child(
                                                div()
                                                    .w(px(80.0))
                                                    .text_sm()
                                                    .text_right()
                                                    .font_family("monospace")
                                                    .child(format!(
                                                        "{:.1}%",
                                                        if cpu_usage.is_empty() {
                                                            0.0
                                                        } else {
                                                            cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
                                                        }
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .gap_2()
                                            .children(
                                                cpu_usage
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(i, usage)| {
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_1()
                                                            .w(px(150.0))
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .w(px(40.0))
                                                                    .child(format!("核心 {}:", i)),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex_1()
                                                                    .h_1()
                                                                    .rounded_sm()
                                                                    .bg(cx.theme().border)
                                                                    .child(
                                                                        div()
                                                                            .h_full()
                                                                            .w(relative(*usage / 100.0))
                                                                            .rounded_sm()
                                                                            .bg(if *usage < 50.0 {
                                                                                cx.theme().accent
                                                                            } else if *usage < 80.0 {
                                                                                gpui::rgb(0xf59e0b).into()
                                                                            } else {
                                                                                gpui::rgb(0xef4444).into()
                                                                            }),
                                                                    ),
                                                            )
                                                            .child(
                                                                div()
                                                                    .w(px(50.0))
                                                                    .text_xs()
                                                                    .text_right()
                                                                    .font_family("monospace")
                                                                    .child(format!("{:.1}%", usage)),
                                                            )
                                                    })
                                                    .collect::<Vec<_>>(),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().font_semibold().mb_2().child("内存使用"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("内存"))
                                            .child(self.render_progress_bar(memory_usage_percent, cx))
                                            .child(
                                                div()
                                                    .w(px(150.0))
                                                    .text_sm()
                                                    .text_right()
                                                    .font_family("monospace")
                                                    .child(format!(
                                                        "{} / {}",
                                                        Self::format_bytes(used_memory),
                                                        Self::format_bytes(total_memory)
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("使用率"))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_family("monospace")
                                                    .child(format!("{:.1}%", memory_usage_percent)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().w_24().child("交换分区"))
                                            .child(self.render_progress_bar(swap_usage_percent, cx))
                                            .child(
                                                div()
                                                    .w(px(150.0))
                                                    .text_sm()
                                                    .text_right()
                                                    .font_family("monospace")
                                                    .child(format!(
                                                        "{} / {}",
                                                        Self::format_bytes(used_swap),
                                                        Self::format_bytes(total_swap)
                                                    )),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().font_semibold().mb_2().child("磁盘使用"))
                                    .children(
                                        disks
                                            .iter()
                                            .map(|disk| {
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_1()
                                                    .pb_2()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_semibold()
                                                                    .child(disk.name.clone()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(cx.theme().muted_foreground)
                                                                    .child(disk.mount_point.clone()),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(self.render_progress_bar(disk.usage_percent, cx))
                                                            .child(
                                                                div()
                                                                    .w(px(150.0))
                                                                    .text_sm()
                                                                    .text_right()
                                                                    .font_family("monospace")
                                                                    .child(format!(
                                                                        "{} / {}",
                                                                        Self::format_bytes(disk.used),
                                                                        Self::format_bytes(disk.total)
                                                                    )),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(format!("使用率: {:.1}%", disk.usage_percent)),
                                                    )
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().font_semibold().mb_2().child("网络信息"))
                                    .children(
                                        networks
                                            .iter()
                                            .map(|net| {
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_4()
                                                    .py_1()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_semibold()
                                                            .w(px(100.0))
                                                            .child(net.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(cx.theme().muted_foreground)
                                                                    .child("接收:"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_family("monospace")
                                                                    .child(Self::format_bytes(net.received)),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(cx.theme().muted_foreground)
                                                                    .child("发送:"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_family("monospace")
                                                                    .child(Self::format_bytes(net.transmitted)),
                                                            ),
                                                    )
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                            ),
                    ),
            )
    }
}

use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    WindowExt,
    *,
};
use std::sync::{LazyLock, Mutex};
use sysinfo::{Disks, MemoryRefreshKind, System, Networks, ProcessesToUpdate};

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
    processes: Vec<ProcessInfo>,
    search_text: String,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct DiskInfo {
    name: String,
    total: u64,
    available: u64,
    used: u64,
    usage_percent: f32,
    mount_point: String,
    file_system: String,
}

#[derive(Clone)]
struct ProcessInfo {
    name: String,
    pid: u32,
    memory: u64,
    cpu: f32,
}

impl SystemMonitor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("搜索进程名称..."));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, _ev: &InputEvent, _window, cx| {
                let value = input_state.read(cx).value();
                this.search_text = value.to_string();
                cx.notify();
            }
        })];

        let mut monitor = Self {
            cpu_usage: Vec::new(),
            total_memory: 0,
            used_memory: 0,
            memory_usage_percent: 0.0,
            total_swap: 0,
            used_swap: 0,
            swap_usage_percent: 0.0,
            disks: Vec::new(),
            processes: Vec::new(),
            search_text: String::new(),
            input_state,
            _subscriptions,
        };
        monitor.refresh();
        monitor
    }

    fn refresh(&mut self) {
        {
            let mut sys = SYS.lock().unwrap();
            sys.refresh_all();
            sys.refresh_memory_specifics(MemoryRefreshKind::everything());
            sys.refresh_processes(ProcessesToUpdate::All, true);

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

            self.processes = sys
                .processes()
                .iter()
                .map(|(pid, process)| ProcessInfo {
                    name: process.name().to_string_lossy().to_string(),
                    pid: pid.as_u32(),
                    memory: process.memory(),
                    cpu: process.cpu_usage(),
                })
                .collect();
            self.processes.sort_by(|a, b| b.memory.cmp(&a.memory));
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
                        available,
                        used,
                        usage_percent,
                        mount_point: disk.mount_point().to_string_lossy().to_string(),
                        file_system: disk.file_system().to_string_lossy().to_string(),
                    }
                })
                .collect();
        }
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

    fn render_progress_bar(percent: f32, cx: &App) -> Div {
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

    fn filtered_processes(&self) -> Vec<&ProcessInfo> {
        if self.search_text.is_empty() {
            return self.processes.iter().collect();
        }
        self.processes
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&self.search_text.to_lowercase()))
            .collect()
    }

    fn kill_process(&mut self, pid: u32, cx: &mut Context<Self>) {
        {
            let sys = SYS.lock().unwrap();
            if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                process.kill();
            }
        }
        self.refresh();
        cx.notify();
    }

    fn open_disk_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let disks = self.disks.clone();
        window.open_sheet_at(Placement::Right, cx, move |this, _, cx| {
            this.overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title("磁盘详情")
                .child(
                    div()
                        .size_full()
                        .overflow_y_scrollbar()
                        .gap_3()
                        .children(disks.iter().map(|disk| {
                            div()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .child(
                                    div()
                                        .font_semibold()
                                        .mb_2()
                                        .child(disk.mount_point.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("文件系统:"))
                                        .child(disk.file_system.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("总空间:"))
                                        .child(Self::format_bytes(disk.total)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("可用空间:"))
                                        .child(Self::format_bytes(disk.available)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("已用空间:"))
                                        .child(Self::format_bytes(disk.used)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("使用率:"))
                                        .child(format!("{:.1}%", disk.usage_percent)),
                                )
                                .child(
                                    div()
                                        .mt_2()
                                        .child(Self::render_progress_bar(disk.usage_percent, cx)),
                                )
                        })),
                )
        });
    }

    fn open_cpu_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let cpu_usage = self.cpu_usage.clone();
        let global_usage = if cpu_usage.is_empty() {
            0.0
        } else {
            cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
        };
        let system_name = System::name().unwrap_or_default();
        let physical_core_count = System::physical_core_count().unwrap_or(0);

        window.open_sheet_at(Placement::Right, cx, move |this, _, cx| {
            this.overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title("CPU详情")
                .child(
                    div()
                        .size_full()
                        .overflow_y_scrollbar()
                        .gap_3()
                        .child(
                            div()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("芯片名称:"))
                                        .child(system_name.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("物理核心数:"))
                                        .child(format!("{}", physical_core_count)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child("全局使用率:"))
                                        .child(format!("{:.1}%", global_usage)),
                                ),
                        )
                        .child(div().font_semibold().mb_2().child("核心详情"))
                        .children(cpu_usage.iter().enumerate().map(|(i, usage)| {
                            div()
                                .p_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .mb_2()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .mb_1()
                                        .child(div().text_color(cx.theme().muted_foreground).child(format!("核心 {}:", i + 1)))
                                        .child(format!("{:.1}%", usage)),
                                )
                                .child(Self::render_progress_bar(*usage, cx))
                        })),
                )
        });
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cpu_usage = self.cpu_usage.clone();
        let global_cpu_usage = if cpu_usage.is_empty() {
            0.0
        } else {
            cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
        };
        let memory_usage_percent = self.memory_usage_percent;
        let swap_usage_percent = self.swap_usage_percent;
        let disks = self.disks.clone();
        let filtered_processes = self.filtered_processes();
        let total_disk_used: u64 = disks.iter().map(|d| d.used).sum();
        let total_disk: u64 = disks.iter().map(|d| d.total).sum();
        let disk_usage_percent = if total_disk > 0 {
            (total_disk_used as f64 / total_disk as f64 * 100.0) as f32
        } else {
            0.0
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("系统监控"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .p_4()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(div().font_semibold().child("CPU使用率"))
                                                    .child(
                                                        Button::new("cpu_detail")
                                                            .icon(Icon::new(IconName::ChevronRight))
                                                            .tooltip("查看详情")
                                                            .on_click(cx.listener(|this, _, window, cx| {
                                                                this.open_cpu_drawer(window, cx);
                                                            })),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(global_cpu_usage, cx))
                                                    .child(
                                                        div()
                                                            .w(px(60.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!("{:.1}%", global_cpu_usage)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(div().font_semibold().child("磁盘使用率"))
                                                    .child(
                                                        Button::new("disk_detail")
                                                            .icon(Icon::new(IconName::ChevronRight))
                                                            .tooltip("查看详情")
                                                            .on_click(cx.listener(|this, _, window, cx| {
                                                                this.open_disk_drawer(window, cx);
                                                            })),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(disk_usage_percent, cx))
                                                    .child(
                                                        div()
                                                            .w(px(60.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!("{:.1}%", disk_usage_percent)),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .p_4()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_3()
                                            .child(div().font_semibold().child("物理内存"))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(memory_usage_percent, cx))
                                                    .child(
                                                        div()
                                                            .w(px(100.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!("{:.1}%", memory_usage_percent)),
                                                    ),
                                            )
                                            .child(div().font_semibold().child("交换内存"))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(swap_usage_percent, cx))
                                                    .child(
                                                        div()
                                                            .w(px(100.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!("{:.1}%", swap_usage_percent)),
                                                    ),
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
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .child(div().font_semibold().child("进程列表"))
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .child(Input::new(&self.input_state).w(px(200.0)))
                                                    .child(
                                                        Button::new("refresh")
                                                            .icon(Icon::new(IconName::Asterisk))
                                                            .tooltip("刷新")
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.refresh();
                                                                cx.notify();
                                                            })),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .text_sm()
                                            .font_semibold()
                                            .py_2()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .child(div().w(px(300.0)).child("名称"))
                                            .child(div().w(px(80.0)).child("PID"))
                                            .child(div().w(px(120.0)).child("内存"))
                                            .child(div().w(px(80.0)).child("CPU(%)"))
                                            .child(div().w(px(80.0)).child("操作")),
                                    )
                                    .child(
                                        div()
                                            .max_h(px(400.0))
                                            .overflow_y_scrollbar()
                                            .children(filtered_processes.iter().map(|process| {
                                                let pid = process.pid;
                                                div()
                                                    .flex()
                                                    .text_sm()
                                                    .py_1()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .child(
                                                        div()
                                                            .w(px(300.0))
                                                            .overflow_x_hidden()
                                                            .child(process.name.clone()),
                                                    )
                                                    .child(div().w(px(80.0)).child(format!("{}", pid)))
                                                    .child(div().w(px(120.0)).child(Self::format_bytes(process.memory)))
                                                    .child(div().w(px(80.0)).child(format!("{:.2}", process.cpu)))
                                                    .child(
                                                        div().w(px(80.0)).child(
                                                            Button::new(("kill", pid))
                                                                .with_variant(ButtonVariant::Danger)
                                                                .child("终止")
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    this.kill_process(pid, cx);
                                                                })),
                                                        ),
                                                    )
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

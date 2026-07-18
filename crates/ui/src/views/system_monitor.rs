use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    WindowExt,
    button::*,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    switch::Switch,
    *,
};
use sysinfo::{Components, Disks, MemoryRefreshKind, ProcessesToUpdate, System};

static SYS: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new_all()));
static DISKS: LazyLock<Mutex<Disks>> =
    LazyLock::new(|| Mutex::new(Disks::new_with_refreshed_list()));
static COMPONENTS: LazyLock<Mutex<Components>> =
    LazyLock::new(|| Mutex::new(Components::new_with_refreshed_list()));

const PAGE_SIZE: usize = 20;

pub struct SystemMonitor {
    cpu_usage: Vec<f32>,
    cpu_temperature: f32,
    cpu_chip_name: String,
    physical_core_count: usize,
    total_memory: u64,
    used_memory: u64,
    memory_usage_percent: f32,
    total_swap: u64,
    used_swap: u64,
    swap_usage_percent: f32,
    disks: Vec<DiskInfo>,
    processes: Vec<ProcessInfo>,
    search_text: String,
    monitoring_enabled: bool,
    current_page: usize,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct DiskInfo {
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
                this.current_page = 0;
                cx.notify();
            }
        })];

        let mut monitor = Self {
            cpu_usage: Vec::new(),
            cpu_temperature: 0.0,
            cpu_chip_name: String::new(),
            physical_core_count: 0,
            total_memory: 0,
            used_memory: 0,
            memory_usage_percent: 0.0,
            total_swap: 0,
            used_swap: 0,
            swap_usage_percent: 0.0,
            disks: Vec::new(),
            processes: Vec::new(),
            search_text: String::new(),
            monitoring_enabled: false,
            current_page: 0,
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
            self.cpu_chip_name = sys
                .cpus()
                .first()
                .map(|c| c.brand().to_string())
                .unwrap_or_default();
            self.physical_core_count = System::physical_core_count().unwrap_or(0);
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

        // 读取 CPU 温度传感器
        {
            let mut components = COMPONENTS.lock().unwrap();
            components.refresh(true);
            let cpu_temp = components.iter().find_map(|c| {
                let label = c.label().to_lowercase();
                if label.contains("cpu") || label.contains("core") || label.contains("tctl") {
                    Some(c.temperature().unwrap_or(0.0))
                } else {
                    None
                }
            });
            self.cpu_temperature = cpu_temp.unwrap_or(0.0);
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
        let result: Vec<&ProcessInfo> = if self.search_text.is_empty() {
            self.processes.iter().collect()
        } else {
            self.processes
                .iter()
                .filter(|p| {
                    p.name
                        .to_lowercase()
                        .contains(&self.search_text.to_lowercase())
                })
                .collect()
        };
        result
    }

    /// 当前页的进程列表
    fn paginated_processes(&self) -> Vec<&ProcessInfo> {
        let filtered = self.filtered_processes();
        let start = self.current_page * PAGE_SIZE;
        if start >= filtered.len() {
            return Vec::new();
        }
        let end = (start + PAGE_SIZE).min(filtered.len());
        filtered[start..end].to_vec()
    }

    fn total_pages(&self) -> usize {
        let total = self.filtered_processes().len();
        if total == 0 {
            1
        } else {
            (total + PAGE_SIZE - 1) / PAGE_SIZE
        }
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

    /// 弹出确认对话框，确认后终止进程
    fn confirm_kill_process(
        &mut self,
        pid: u32,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, _, _cx| {
            let weak = weak.clone();
            let name = name.clone();
            dialog
                .title("确认终止进程")
                .child(div().child(format!("确定要终止进程 {} (PID: {}) 吗？", name, pid)))
                .confirm()
                .on_ok(move |_, _, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.kill_process(pid, cx);
                    });
                    true
                })
        });
    }

    fn toggle_monitoring(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.monitoring_enabled = enabled;
        if enabled {
            self.refresh();
            // 启动自动刷新定时器：CPU/内存 2s，进程 10s
            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_secs(2)).await;
                    let should_stop = this.update(cx, |this, cx| {
                        if !this.monitoring_enabled {
                            return true;
                        }
                        this.refresh();
                        cx.notify();
                        false
                    });
                    if should_stop.unwrap_or(true) {
                        break;
                    }
                }
            })
            .detach();

            // 进程列表刷新定时器 10s
            cx.spawn(async move |this: WeakEntity<Self>, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_secs(10))
                        .await;
                    let should_stop = this.update(cx, |this, cx| {
                        if !this.monitoring_enabled {
                            return true;
                        }
                        // 进程数据已在 refresh() 中刷新
                        false
                    });
                    if should_stop.unwrap_or(true) {
                        break;
                    }
                }
            })
            .detach();
        }
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
                                .child(div().font_semibold().mb_2().child(disk.mount_point.clone()))
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("文件系统:"),
                                        )
                                        .child(disk.file_system.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("总空间:"),
                                        )
                                        .child(Self::format_bytes(disk.total)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("可用空间:"),
                                        )
                                        .child(Self::format_bytes(disk.available)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("已用空间:"),
                                        )
                                        .child(Self::format_bytes(disk.used)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("使用率:"),
                                        )
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
        let system_name = self.cpu_chip_name.clone();
        let physical_core_count = self.physical_core_count;
        let cpu_temperature = self.cpu_temperature;

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
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("芯片名称:"),
                                        )
                                        .child(system_name.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("物理核心数:"),
                                        )
                                        .child(format!("{}", physical_core_count)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("全局使用率:"),
                                        )
                                        .child(format!("{:.1}%", global_usage)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_sm()
                                        .py_1()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("CPU温度:"),
                                        )
                                        .child(format!("{:.1}°C", cpu_temperature)),
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
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("核心 {}:", i + 1)),
                                        )
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
        let monitoring_enabled = self.monitoring_enabled;

        // 未开启监控时的空状态
        if !monitoring_enabled {
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .p_8()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        .child(
                            Switch::new("monitor-toggle")
                                .checked(false)
                                .on_click(cx.listener(|this, v: &bool, _, cx| {
                                    this.toggle_monitoring(*v, cx);
                                })),
                        )
                        .child(div().text_lg().font_semibold().child("系统监控已关闭"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("打开开关后开始采集 CPU、内存、磁盘和进程信息"),
                        ),
                )
                .into_any_element();
        }

        let cpu_usage = self.cpu_usage.clone();
        let global_cpu_usage = if cpu_usage.is_empty() {
            0.0
        } else {
            cpu_usage.iter().sum::<f32>() / cpu_usage.len() as f32
        };
        let memory_usage_percent = self.memory_usage_percent;
        let swap_usage_percent = self.swap_usage_percent;
        let cpu_temperature = self.cpu_temperature;
        let disks = self.disks.clone();
        let paginated = self.paginated_processes();
        let total_disk_used: u64 = disks.iter().map(|d| d.used).sum();
        let total_disk: u64 = disks.iter().map(|d| d.total).sum();
        let disk_usage_percent = if total_disk > 0 {
            (total_disk_used as f64 / total_disk as f64 * 100.0) as f32
        } else {
            0.0
        };
        let current_page = self.current_page;
        let total_pages = self.total_pages();
        let total_filtered = self.filtered_processes().len();

        div()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    // CPU 和 内存 卡片
                    .child(
                        h_flex()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .p_4()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .child(
                                        v_flex()
                                            .gap_3()
                                            .child(
                                                h_flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(div().font_semibold().child("CPU使用率"))
                                                    .child(
                                                        h_flex()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(
                                                                        cx.theme().muted_foreground,
                                                                    )
                                                                    .child(format!(
                                                                        "🌡 {:.1}°C",
                                                                        cpu_temperature
                                                                    )),
                                                            )
                                                            .child(
                                                                Button::new("cpu_detail")
                                                                    .icon(Icon::new(
                                                                        IconName::ChevronRight,
                                                                    ))
                                                                    .tooltip("查看详情")
                                                                    .on_click(cx.listener(
                                                                        |this, _, window, cx| {
                                                                            this.open_cpu_drawer(
                                                                                window, cx,
                                                                            );
                                                                        },
                                                                    )),
                                                            ),
                                                    ),
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(
                                                        global_cpu_usage,
                                                        cx,
                                                    ))
                                                    .child(
                                                        div()
                                                            .w(px(60.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!(
                                                                "{:.1}%",
                                                                global_cpu_usage
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                h_flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(
                                                        div().font_semibold().child("磁盘使用率"),
                                                    )
                                                    .child(
                                                        Button::new("disk_detail")
                                                            .icon(Icon::new(IconName::ChevronRight))
                                                            .tooltip("查看详情")
                                                            .on_click(cx.listener(
                                                                |this, _, window, cx| {
                                                                    this.open_disk_drawer(
                                                                        window, cx,
                                                                    );
                                                                },
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(
                                                        disk_usage_percent,
                                                        cx,
                                                    ))
                                                    .child(
                                                        div()
                                                            .w(px(60.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!(
                                                                "{:.1}%",
                                                                disk_usage_percent
                                                            )),
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
                                        v_flex()
                                            .gap_3()
                                            .child(div().font_semibold().child("物理内存"))
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(
                                                        memory_usage_percent,
                                                        cx,
                                                    ))
                                                    .child(
                                                        div()
                                                            .w(px(100.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!(
                                                                "{:.1}%",
                                                                memory_usage_percent
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!(
                                                        "{} / {}",
                                                        Self::format_bytes(self.used_memory),
                                                        Self::format_bytes(self.total_memory)
                                                    )),
                                            )
                                            .child(div().font_semibold().child("交换内存"))
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Self::render_progress_bar(
                                                        swap_usage_percent,
                                                        cx,
                                                    ))
                                                    .child(
                                                        div()
                                                            .w(px(100.0))
                                                            .text_sm()
                                                            .text_right()
                                                            .font_family("monospace")
                                                            .child(format!(
                                                                "{:.1}%",
                                                                swap_usage_percent
                                                            )),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                    // 进程列表
                    .child(
                        div()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                v_flex()
                                    .gap_3()
                                    .child(
                                        h_flex()
                                            .justify_between()
                                            .items_center()
                                            .child(div().font_semibold().child("进程列表"))
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .child(
                                                        Input::new(&self.input_state).w(px(200.0)),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(format!(
                                                                "共 {} 个进程",
                                                                total_filtered
                                                            )),
                                                    ),
                                            ),
                                    )
                                    // 表头
                                    .child(
                                        h_flex()
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
                                    .child(div().max_h(px(400.0)).overflow_y_scrollbar().children(
                                        paginated.iter().map(|process| {
                                            let pid = process.pid;
                                            let process_name = process.name.clone();
                                            h_flex()
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
                                                .child(
                                                    div()
                                                        .w(px(120.0))
                                                        .child(Self::format_bytes(process.memory)),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(80.0))
                                                        .child(format!("{:.2}", process.cpu)),
                                                )
                                                .child(
                                                    div().w(px(80.0)).child(
                                                        Button::new(("kill", pid as usize))
                                                            .with_variant(ButtonVariant::Danger)
                                                            .child("终止")
                                                            .on_click(cx.listener(
                                                                move |this, _, window, cx| {
                                                                    this.confirm_kill_process(
                                                                        pid,
                                                                        process_name.clone(),
                                                                        window,
                                                                        cx,
                                                                    );
                                                                },
                                                            )),
                                                    ),
                                                )
                                        }),
                                    ))
                                    // 分页控件
                                    .child(
                                        h_flex()
                                            .justify_between()
                                            .items_center()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!(
                                                        "第 {} / {} 页",
                                                        current_page + 1,
                                                        total_pages
                                                    )),
                                            )
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .child(
                                                        Button::new("prev-page")
                                                            .icon(Icon::new(IconName::ArrowLeft))
                                                            .tooltip("上一页")
                                                            .disabled(current_page == 0)
                                                            .on_click(cx.listener(
                                                                |this, _, _, cx| {
                                                                    if this.current_page > 0 {
                                                                        this.current_page -= 1;
                                                                        cx.notify();
                                                                    }
                                                                },
                                                            )),
                                                    )
                                                    .child(
                                                        Button::new("next-page")
                                                            .icon(Icon::new(IconName::ArrowRight))
                                                            .tooltip("下一页")
                                                            .disabled(
                                                                current_page >= total_pages - 1,
                                                            )
                                                            .on_click(cx.listener(
                                                                |this, _, _, cx| {
                                                                    if this.current_page
                                                                        < this.total_pages() - 1
                                                                    {
                                                                        this.current_page += 1;
                                                                        cx.notify();
                                                                    }
                                                                },
                                                            )),
                                                    ),
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}

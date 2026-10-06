use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use gpui_kit::{
    component::{
        button::*,
        input::{Input, InputEvent, InputState},
        scroll::ScrollableElement,
        switch::Switch,
        *,
    },
    prelude::FluentBuilder,
    *,
};
use sysinfo::{Components, Disks, MemoryRefreshKind, ProcessesToUpdate, System};

static SYS: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new_all()));
static DISKS: LazyLock<Mutex<Disks>> =
    LazyLock::new(|| Mutex::new(Disks::new_with_refreshed_list()));
static COMPONENTS: LazyLock<Mutex<Components>> =
    LazyLock::new(|| Mutex::new(Components::new_with_refreshed_list()));

const PAGE_SIZE: usize = 20;

// 与 Vue 版 n-progress 颜色对齐
const COLOR_CPU: u32 = 0x4caf50;
const COLOR_DISK: u32 = 0x9c27b0;
const COLOR_MEM: u32 = 0x2196f3;
const COLOR_SWAP: u32 = 0xff9800;

pub struct SystemMonitor {
    cpu_cores: Vec<CpuCore>,
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
struct CpuCore {
    usage: f32,
    frequency: u64,
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
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("搜索进程名称"));

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
            cpu_cores: Vec::new(),
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

            self.cpu_cores = sys
                .cpus()
                .iter()
                .map(|cpu| CpuCore {
                    usage: cpu.cpu_usage(),
                    frequency: cpu.frequency(),
                })
                .collect();
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

    /// n-progress line：底槽 + 固定颜色填充（height 单位 px）
    fn render_progress_bar(percent: f32, color: u32, height: f32, cx: &App) -> Div {
        let percent = percent.clamp(0.0, 100.0);
        div()
            .flex_1()
            .h(px(height))
            .rounded_md()
            .bg(cx.theme().border)
            .overflow_hidden()
            .child(
                div()
                    .h_full()
                    .w(relative(percent / 100.0))
                    .rounded_md()
                    .bg(Hsla::from(rgb(color))),
            )
    }

    /// metric 行（item-content .metric：label 左 灰 14px + 值右 16px 加粗）
    fn render_metric(label: &str, value: String, cx: &App) -> Div {
        div()
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(div().text_size(px(16.0)).font_semibold().child(value))
    }

    /// 可点击 metric 行（点击打开对应抽屉）
    fn render_clickable_metric(
        id: &'static str,
        label: &str,
        value: String,
        cx: &App,
    ) -> Stateful<Div> {
        div()
            .id(id)
            .flex()
            .justify_between()
            .items_center()
            .px_1()
            .py(px(2.0))
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(cx.theme().background))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_size(px(16.0)).font_semibold().child(value))
                    .child(
                        Icon::new(IconName::ChevronRight)
                            .size(px(14.0))
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
    }

    fn global_cpu_usage(&self) -> f32 {
        if self.cpu_cores.is_empty() {
            0.0
        } else {
            self.cpu_cores.iter().map(|c| c.usage).sum::<f32>() / self.cpu_cores.len() as f32
        }
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

    fn confirm_kill_process(
        &mut self,
        pid: u32,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let weak = cx.weak_entity();
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            let weak = weak.clone();
            let name = name.clone();
            alert
                .title("确认终止进程")
                .description(div().child(format!("确定要终止进程 {} (PID: {}) 吗？", name, pid)))
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
            // Vue：cpuMemoryTimer 2s / diskTimer 30s / processTimer 10s，此处统一刷新
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
        }
        cx.notify();
    }

    fn open_disk_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // 同 CPU 抽屉：builder 每帧读取实时磁盘数据
        let this = cx.entity().downgrade();
        window.open_sheet_at(Placement::Right, cx, move |sheet, _, cx| {
            let disks = this
                .upgrade()
                .map(|m| m.read(cx).disks.clone())
                .unwrap_or_default();
            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title("磁盘详情")
                .child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .overflow_y_scrollbar()
                        .children(disks.iter().map(|disk| {
                            div()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .mb_1()
                                        .child(disk.mount_point.clone()),
                                )
                                .child(Self::drawer_row("文件系统:", disk.file_system.clone(), cx))
                                .child(Self::drawer_row(
                                    "总空间:",
                                    Self::format_bytes(disk.total),
                                    cx,
                                ))
                                .child(Self::drawer_row(
                                    "可用空间:",
                                    Self::format_bytes(disk.available),
                                    cx,
                                ))
                                .child(Self::drawer_row(
                                    "已用空间:",
                                    Self::format_bytes(disk.used),
                                    cx,
                                ))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(Self::drawer_row(
                                            "使用率:",
                                            format!("{:.2}%", disk.usage_percent),
                                            cx,
                                        ))
                                        .child(Self::render_progress_bar(
                                            disk.usage_percent,
                                            COLOR_DISK,
                                            8.0,
                                            cx,
                                        )),
                                )
                        })),
                )
        });
    }

    fn drawer_row(label: &str, value: String, cx: &App) -> Div {
        div()
            .flex()
            .justify_between()
            .text_size(px(13.0))
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(value)
    }

    fn open_cpu_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // 捕获实体弱引用，builder 每帧重执行时读取实时数据（对齐 Tauri 抽屉内容随采集刷新）
        let this = cx.entity().downgrade();
        window.open_sheet_at(Placement::Right, cx, move |sheet, _, cx| {
            let monitor = this.upgrade();
            let (cpu_cores, chip_name, physical_core_count) = monitor
                .as_ref()
                .map(|m| {
                    let m = m.read(cx);
                    (
                        m.cpu_cores.clone(),
                        m.cpu_chip_name.clone(),
                        m.physical_core_count,
                    )
                })
                .unwrap_or_default();
            let global_usage = if cpu_cores.is_empty() {
                0.0
            } else {
                cpu_cores.iter().map(|c| c.usage).sum::<f32>() / cpu_cores.len() as f32
            };

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title("CPU详情")
                .child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .overflow_y_scrollbar()
                        .child(
                            div()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(Self::drawer_row("芯片名称:", chip_name.clone(), cx))
                                .child(Self::drawer_row(
                                    "物理核心数:",
                                    format!("{}", physical_core_count),
                                    cx,
                                ))
                                .child(Self::drawer_row(
                                    "全局使用率:",
                                    format!("{:.2}%", global_usage),
                                    cx,
                                )),
                        )
                        .child(div().text_sm().font_semibold().mb_1().child("核心详情"))
                        .children(cpu_cores.iter().enumerate().map(|(i, core)| {
                            div()
                                .p_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .mb_2()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_size(px(13.0))
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("核心 {}:", i + 1)),
                                        )
                                        .child(format!("{:.2}%", core.usage)),
                                )
                                .child(Self::render_progress_bar(core.usage, COLOR_CPU, 8.0, cx))
                                .child(Self::drawer_row(
                                    "频率:",
                                    format!("{} MHz", core.frequency),
                                    cx,
                                ))
                        })),
                )
        });
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let monitoring_enabled = self.monitoring_enabled;

        // 监控关闭态：居中开关面板（monitor-switch-panel）
        if !monitoring_enabled {
            // 对齐 Tauri 的 min-height: calc(100vh - 160px)：
            // 顶栏 56 + 页面 padding 40 + 余量，保证面板在视口中垂直居中
            let min_h_center = (window.viewport_size().height - px(160.0)).max(px(320.0));
            return design::page()
                .child(
                    div()
                        .flex_1()
                        .min_h(min_h_center)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_3()
                                .px(px(40.0))
                                .py_8()
                                .rounded(px(16.0))
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().popover)
                                .shadow_sm()
                                .child(
                                    div()
                                        .text_size(px(18.0))
                                        .font_semibold()
                                        .child("系统监控已关闭"),
                                )
                                .child(Switch::new("monitor-toggle").checked(false).on_click(
                                    cx.listener(|this, v: &bool, _, cx| {
                                        this.toggle_monitoring(*v, cx);
                                    }),
                                )),
                        ),
                )
                .into_any_element();
        }

        let global_cpu_usage = self.global_cpu_usage();
        let memory_usage_percent = self.memory_usage_percent;
        let swap_usage_percent = self.swap_usage_percent;

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

        // monitor-row：两张监控卡并排（Tauri：display:flex; gap:20px; 两卡 flex:1 等宽）
        let monitor_row = div()
            .flex()
            .gap_5()
            .child(
                div().flex_1().child(
                    design::card(cx).child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(15.0))
                            // CPU使用率（点击打开 CPU 详情抽屉）
                            .child(
                                Self::render_clickable_metric(
                                    "cpu-metric",
                                    "CPU使用率",
                                    format!("{:.2}%", global_cpu_usage),
                                    cx,
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.open_cpu_drawer(window, cx);
                                    },
                                )),
                            )
                            .child(Self::render_progress_bar(
                                global_cpu_usage,
                                COLOR_CPU,
                                12.0,
                                cx,
                            ))
                            // 磁盘使用率（点击打开磁盘详情抽屉）
                            .child(
                                Self::render_clickable_metric(
                                    "disk-metric",
                                    "磁盘使用率",
                                    format!("{:.2}%", disk_usage_percent),
                                    cx,
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.open_disk_drawer(window, cx);
                                    },
                                )),
                            )
                            .child(Self::render_progress_bar(
                                disk_usage_percent,
                                COLOR_DISK,
                                12.0,
                                cx,
                            )),
                    ),
                ),
            )
            .child(
                div().flex_1().child(
                    design::card(cx).child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(15.0))
                            // 物理内存
                            .child(Self::render_metric(
                                "物理内存",
                                format!("{:.2}%", memory_usage_percent),
                                cx,
                            ))
                            .child(Self::render_progress_bar(
                                memory_usage_percent,
                                COLOR_MEM,
                                10.0,
                                cx,
                            ))
                            // 交换内存
                            .child(Self::render_metric(
                                "交换内存",
                                format!("{:.2}%", swap_usage_percent),
                                cx,
                            ))
                            .child(Self::render_progress_bar(
                                swap_usage_percent,
                                COLOR_SWAP,
                                10.0,
                                cx,
                            )),
                    ),
                ),
            );

        // 进程表头（n-data-table 表头：xs 加粗 次要色）
        let table_header = div()
            .flex()
            .text_xs()
            .font_semibold()
            .text_color(cx.theme().muted_foreground)
            .px_2()
            .py_1p5()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().flex_1().min_w_0().child("名称"))
            .child(div().w(px(90.0)).child("PID"))
            .child(div().w(px(110.0)).child("内存"))
            .child(div().w(px(80.0)).child("CPU(%)"))
            .child(div().w(px(76.0)).child("操作"));

        // 进程行
        let table_rows = paginated
            .iter()
            .map(|process| {
                let pid = process.pid;
                let process_name = process.name.clone();
                div()
                    .flex()
                    .items_center()
                    .text_sm()
                    .px_2()
                    .py_1p5()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .hover(|s| s.bg(cx.theme().background))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(process.name.clone()),
                    )
                    .child(
                        div()
                            .w(px(90.0))
                            .font_family("monospace")
                            .child(format!("{}", pid)),
                    )
                    .child(div().w(px(110.0)).child(Self::format_bytes(process.memory)))
                    .child(
                        div()
                            .w(px(80.0))
                            .font_family("monospace")
                            .child(format!("{:.2}", process.cpu)),
                    )
                    .child(
                        div().w(px(76.0)).child(
                            Button::new(("kill", pid as usize))
                                .with_variant(ButtonVariant::Danger)
                                .compact()
                                .child("终止")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.confirm_kill_process(
                                        pid,
                                        process_name.clone(),
                                        window,
                                        cx,
                                    );
                                })),
                        ),
                    )
            })
            .collect::<Vec<_>>();

        let process_card = design::card(cx)
            .flex_1()
            .child(
                // 卡片头：标题 + 右侧搜索框（Vue：header 内右对齐搜索）
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .pb(px(14.0))
                    .mb_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(design::editor_label("进程列表", cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Input::new(&self.input_state).w(px(200.0)))
                            .child(design::stat_pill(
                                format!("{}", total_filtered),
                                "个进程",
                                cx.theme().primary,
                                cx,
                            )),
                    ),
            )
            .child(table_header)
            .child(
                div()
                    .max_h(px(420.0))
                    .overflow_y_scrollbar()
                    .when(table_rows.is_empty(), |this| {
                        this.child(
                            div()
                                .py_6()
                                .text_center()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("暂无进程数据"),
                        )
                    })
                    .children(table_rows),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .pt_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("第 {} / {} 页", current_page + 1, total_pages)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("prev-page")
                                    .icon(Icon::new(IconName::ArrowLeft))
                                    .compact()
                                    .tooltip("上一页")
                                    .disabled(current_page == 0)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.current_page > 0 {
                                            this.current_page -= 1;
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("next-page")
                                    .icon(Icon::new(IconName::ArrowRight))
                                    .compact()
                                    .tooltip("下一页")
                                    .disabled(current_page >= total_pages - 1)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.current_page < this.total_pages() - 1 {
                                            this.current_page += 1;
                                            cx.notify();
                                        }
                                    })),
                            ),
                    ),
            );

        design::page()
            .child(monitor_row)
            .child(process_card)
            .flex_1()
            .into_any_element()
    }
}

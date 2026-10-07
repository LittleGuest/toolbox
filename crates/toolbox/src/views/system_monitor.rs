use std::{
    sync::{Arc, LazyLock, Mutex},
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
use monitor::{LineageNode, ProcessLineage, process_lineage};
use sysinfo::{Components, Disks, MemoryRefreshKind, ProcessesToUpdate, System};

static SYS: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new_all()));
static DISKS: LazyLock<Mutex<Disks>> =
    LazyLock::new(|| Mutex::new(Disks::new_with_refreshed_list()));
static COMPONENTS: LazyLock<Mutex<Components>> =
    LazyLock::new(|| Mutex::new(Components::new_with_refreshed_list()));

const PAGE_SIZE: usize = 20;

const COLOR_CPU: u32 = 0x4caf50;
const COLOR_DISK: u32 = 0x9c27b0;
const COLOR_MEM: u32 = 0x2196f3;
const COLOR_SWAP: u32 = 0xff9800;
const COLOR_CYAN: u32 = 0x0891b2;

const ACTION_COLUMN_WIDTH: f32 = 80.0;

const ROW_HEIGHT: f32 = 45.0;
const LIST_CHROME_HEIGHT: Pixels = px(505.0);
const MIN_LIST_HEIGHT: Pixels = px(180.0);

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

    fn open_process_lineage(
        &mut self,
        pid: u32,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let lineage = Arc::new(process_lineage(pid));
        let title = format!("{name} · 来龙去脉");
        window.open_sheet_at(Placement::Right, cx, move |sheet, _, cx| {
            let data = lineage.clone();
            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(580.))
                .title(title.clone())
                .child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .gap(px(22.0))
                        .overflow_y_scrollbar()
                        .child(Self::render_lineage_hero(&data, cx))
                        .child(Self::render_lineage_chain(&data, cx))
                        .child(Self::render_lineage_source(&data, cx))
                        .child(Self::render_lineage_fields(
                            "进程信息",
                            IconName::Cpu,
                            &data.details,
                            cx,
                        ))
                        .child(Self::render_lineage_context(&data, cx))
                        .child(Self::render_lineage_warnings(&data, cx)),
                )
        });
    }

    fn role_color(role: &str, cx: &App) -> Hsla {
        match role {
            "init" => Hsla::from(rgb(COLOR_DISK)),
            "systemd" => Hsla::from(rgb(COLOR_MEM)),
            "container" | "terminal" => Hsla::from(rgb(COLOR_CYAN)),
            "shell" => Hsla::from(rgb(COLOR_CPU)),
            "ssh" => Hsla::from(rgb(COLOR_SWAP)),
            "cron" => Hsla::from(rgb(design::ACCENT)),
            "desktop" => Hsla::from(rgb(COLOR_DISK)),
            "login" => cx.theme().muted_foreground,
            _ => cx.theme().muted_foreground,
        }
    }

    fn source_icon(kind: &str) -> IconName {
        match kind {
            "systemd" => IconName::FileText,
            "container" => IconName::Frame,
            "terminal" | "shell" => IconName::SquareTerminal,
            "desktop" => IconName::LayoutDashboard,
            "ssh" => IconName::Globe,
            "cron" => IconName::Calendar,
            "adopted" => IconName::TriangleAlert,
            "parent" => IconName::User,
            _ => IconName::Info,
        }
    }

    fn source_color(kind: &str, cx: &App) -> Hsla {
        match kind {
            "systemd" => Hsla::from(rgb(COLOR_MEM)),
            "container" => Hsla::from(rgb(COLOR_CYAN)),
            "terminal" | "shell" => Hsla::from(rgb(COLOR_CPU)),
            "desktop" => Hsla::from(rgb(COLOR_DISK)),
            "ssh" => Hsla::from(rgb(COLOR_SWAP)),
            "cron" => Hsla::from(rgb(design::ACCENT)),
            "adopted" | "unknown" => Hsla::from(rgb(design::WARN_AMBER)),
            _ => cx.theme().muted_foreground,
        }
    }

    fn render_lineage_hero(lineage: &ProcessLineage, cx: &App) -> Div {
        let target = &lineage.target;
        let accent = Self::role_color(&target.role, cx);
        let meta = format!(
            "PID {} · {} · {} · 已运行 {}",
            target.pid, target.user, target.status, target.run_time
        );
        div()
            .w_full()
            .p(px(16.0))
            .rounded(px(12.0))
            .border_1()
            .border_color(accent.opacity(0.28))
            .bg(accent.opacity(0.07))
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .w(px(42.0))
                    .h(px(42.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(11.0))
                    .bg(accent.opacity(0.16))
                    .text_color(accent)
                    .child(Icon::new(IconName::Network).size(px(21.0))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_semibold()
                                    .child(target.name.clone()),
                            )
                            .child(design::mini_tag(
                                target.role_label.clone(),
                                accent.opacity(0.16),
                                accent,
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(meta),
                    ),
            )
    }

    fn render_lineage_chain(lineage: &ProcessLineage, cx: &App) -> Div {
        let adopted_gap = lineage.source.kind == "adopted";
        let total = lineage.chain.len();
        let mut rows: Vec<AnyElement> = Vec::with_capacity(total + 2);
        if let Some(broken) = lineage.broken_at {
            rows.push(Self::chain_broken_note(broken, cx).into_any_element());
        }
        for (index, node) in lineage.chain.iter().enumerate() {
            let is_last = index + 1 == total;
            rows.push(Self::chain_row(node, is_last, cx).into_any_element());
            if adopted_gap && index == 0 && total > 1 {
                rows.push(Self::chain_gap_note(cx).into_any_element());
            }
        }
        let note = if adopted_gap {
            "链路底部由 init 收养，起始过程未知"
        } else if lineage.broken_at.is_some() {
            "链路在追溯途中断裂"
        } else {
            "从系统根部到目标进程的完整因果链"
        };
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(Self::lineage_section(
                "为什么它在运行",
                IconName::Network,
                Some(format!("{} 层", total)),
                note,
                cx,
            ))
            .child(div().flex().flex_col().children(rows))
    }

    fn chain_row(node: &LineageNode, is_last: bool, cx: &App) -> Div {
        let is_target = node.is_target;
        let color = if is_target {
            cx.theme().primary
        } else {
            Self::role_color(&node.role, cx)
        };
        let dot_top = px(10.0);
        let line_top = px(24.0);

        let marker = {
            let mut col = div().w(px(18.0)).flex_shrink_0().relative().child(
                div()
                    .absolute()
                    .top(dot_top)
                    .left(px(4.0))
                    .size(px(10.0))
                    .rounded_full()
                    .bg(color),
            );
            if is_target {
                col = col.child(
                    div()
                        .absolute()
                        .top(px(8.0))
                        .left(px(1.0))
                        .size(px(16.0))
                        .rounded_full()
                        .border_1()
                        .border_color(color.opacity(0.45)),
                );
            }
            if !is_last {
                col = col.child(
                    div()
                        .absolute()
                        .top(line_top)
                        .bottom(px(-14.0))
                        .left(px(8.0))
                        .w(px(2.0))
                        .rounded_full()
                        .bg(cx.theme().border),
                );
            }
            col
        };

        let mut head = div().flex().items_center().gap_2().flex_wrap();
        head = head
            .child(
                div()
                    .text_size(px(13.5))
                    .font_semibold()
                    .text_color(if is_target {
                        color
                    } else {
                        cx.theme().foreground
                    })
                    .child(node.name.clone()),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("PID {}", node.pid)),
            )
            .child(design::mini_tag(
                node.role_label.clone(),
                color.opacity(0.14),
                color,
            ));
        if is_target {
            head = head.child(design::mini_tag(
                "当前进程",
                cx.theme().primary.opacity(0.16),
                cx.theme().primary,
            ));
        }

        let detail = if node.cmdline.is_empty() {
            node.exe.clone()
        } else {
            node.cmdline.clone()
        };

        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(3.0))
            .px(px(10.0))
            .py(px(5.0))
            .rounded(px(9.0))
            .child(head)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(11.5))
                            .flex_shrink_0()
                            .text_color(cx.theme().muted_foreground)
                            .child(node.user.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_family("monospace")
                            .text_size(px(11.5))
                            .text_color(cx.theme().muted_foreground)
                            .child(detail),
                    ),
            );
        if is_target {
            body = body
                .bg(cx.theme().primary.opacity(0.07))
                .border_1()
                .border_color(cx.theme().primary.opacity(0.22));
        }

        let mut row = div().flex().gap_3().child(marker).child(body);
        if !is_last {
            row = row.pb(px(14.0));
        }
        row
    }

    fn chain_gap_note(cx: &App) -> Div {
        div()
            .flex()
            .gap_3()
            .pb(px(14.0))
            .items_start()
            .child(
                div()
                    .w(px(18.0))
                    .flex_shrink_0()
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .bottom(px(-14.0))
                            .left(px(8.0))
                            .w(px(2.0))
                            .rounded_full()
                            .bg(Hsla::from(rgb(design::WARN_AMBER)).opacity(0.35)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(2.0))
                            .left(px(2.0))
                            .size(px(14.0))
                            .rounded_full()
                            .bg(cx.theme().popover)
                            .border_1()
                            .border_color(Hsla::from(rgb(design::WARN_AMBER)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(10.0))
                            .font_semibold()
                            .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                            .child("?"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_medium()
                            .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                            .child("原始父进程已退出"),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("此处为收养关系，不代表它真的由 init 启动"),
                    ),
            )
    }

    fn chain_broken_note(pid: u32, cx: &App) -> Div {
        div()
            .flex()
            .gap_3()
            .items_start()
            .pb(px(6.0))
            .child(
                div()
                    .w(px(18.0))
                    .flex_shrink_0()
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top(px(20.0))
                            .bottom(px(-18.0))
                            .left(px(8.0))
                            .w(px(2.0))
                            .rounded_full()
                            .bg(Hsla::from(rgb(design::ERROR_RED)).opacity(0.4)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(4.0))
                            .left(px(2.0))
                            .size(px(14.0))
                            .rounded_full()
                            .bg(cx.theme().popover)
                            .border_1()
                            .border_color(Hsla::from(rgb(design::ERROR_RED)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(Hsla::from(rgb(design::ERROR_RED)))
                            .child(Icon::new(IconName::TriangleAlert).size(px(9.0))),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pt(px(3.0))
                    .text_size(px(11.5))
                    .text_color(Hsla::from(rgb(design::ERROR_RED)))
                    .child(format!("再往上追溯时父进程 PID {pid} 已不存在")),
            )
    }

    fn render_lineage_source(lineage: &ProcessLineage, cx: &App) -> Div {
        let source = &lineage.source;
        let color = Self::source_color(&source.kind, cx);
        let mut card = div()
            .w_full()
            .p(px(14.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(color.opacity(0.3))
            .bg(color.opacity(0.06))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(7.0))
                            .bg(color.opacity(0.18))
                            .text_color(color)
                            .child(Icon::new(Self::source_icon(&source.kind)).size(px(15.0))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_semibold()
                                    .text_color(color)
                                    .child(source.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().foreground)
                                    .child(source.summary.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.0))
                    .py(px(7.0))
                    .rounded(px(8.0))
                    .bg(cx.theme().background)
                    .child(
                        Icon::new(IconName::Info)
                            .size(px(13.0))
                            .flex_shrink_0()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(source.supervisor.clone()),
                    ),
            );

        for field in &source.details {
            card = card.child(Self::lineage_kv(&field.label, &field.value, true, cx));
        }

        div()
            .w_full()
            .flex()
            .flex_col()
            .child(Self::lineage_section(
                "启动来源",
                IconName::Inspector,
                None,
                "谁把它拉起来的",
                cx,
            ))
            .child(card)
    }

    fn render_lineage_fields(
        title: &str,
        icon: IconName,
        fields: &[monitor::LineageField],
        cx: &App,
    ) -> Div {
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(Self::lineage_section(title, icon, None, "进程自身的信息", cx))
            .child(
                div()
                    .w_full()
                    .p(px(14.0))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .flex()
                    .flex_col()
                    .gap(px(9.0))
                    .children(
                        fields
                            .iter()
                            .map(|f| Self::lineage_kv(&f.label, &f.value, false, cx)),
                    ),
            )
    }

    fn render_lineage_context(lineage: &ProcessLineage, cx: &App) -> Div {
        let mut body = div()
            .w_full()
            .p(px(14.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .flex()
            .flex_col()
            .gap(px(9.0))
            .children(
                lineage
                    .context
                    .iter()
                    .map(|f| Self::lineage_kv(&f.label, &f.value, false, cx)),
            );

        if !lineage.sockets.is_empty() {
            body = body.child(
                div()
                    .text_size(px(11.5))
                    .font_semibold()
                    .text_color(cx.theme().muted_foreground)
                    .pt(px(2.0))
                    .child("监听端口"),
            );
            body = body.child(div().flex().flex_wrap().gap(px(6.0)).children(
                lineage.sockets.iter().map(|socket| {
                    let color = if socket.public_bind {
                        Hsla::from(rgb(design::WARN_AMBER))
                    } else {
                        Hsla::from(rgb(COLOR_MEM))
                    };
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py(px(3.0))
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(color.opacity(0.35))
                        .bg(color.opacity(0.08))
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_semibold()
                                .text_color(color)
                                .child(socket.proto.clone()),
                        )
                        .child(
                            div()
                                .font_family("monospace")
                                .text_size(px(11.5))
                                .child(socket.local.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(socket.state.clone()),
                        )
                }),
            ));
        }

        if !lineage.children.is_empty() {
            body = body.child(
                div()
                    .text_size(px(11.5))
                    .font_semibold()
                    .text_color(cx.theme().muted_foreground)
                    .pt(px(2.0))
                    .child(format!("子进程（{}）", lineage.children.len())),
            );
            body = body.child(
                div().flex().flex_wrap().gap(px(6.0)).children(
                    lineage.children.iter().take(12).map(|child| {
                        let color = Self::role_color(&child.role, cx);
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2()
                            .py(px(3.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().popover)
                            .child(
                                div()
                                    .w(px(7.0))
                                    .h(px(7.0))
                                    .rounded_full()
                                    .bg(color),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .child(child.name.clone()),
                            )
                            .child(
                                div()
                                    .font_family("monospace")
                                    .text_size(px(10.5))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{}", child.pid)),
                            )
                    }),
                ),
            );
        }

        if let Some(count) = lineage.env_count {
            let mut env_row = div()
                .flex()
                .flex_col()
                .gap_1p5()
                .pt(px(2.0))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child(if count > 0 {
                            format!("环境变量（{count} 项）")
                        } else {
                            "环境变量".to_string()
                        }),
                );
            if count == 0 {
                env_row = env_row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(cx.theme().muted_foreground)
                        .child("进程已清空自身环境变量（Chromium / Electron 等常见）"),
                );
            } else if lineage.env_highlights.is_empty() {
                env_row = env_row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(cx.theme().muted_foreground)
                        .child("没有值得注意的变量"),
                );
            } else {
                env_row = env_row.children(
                    lineage
                        .env_highlights
                        .iter()
                        .map(|f| Self::lineage_kv(&f.label, &f.value, true, cx)),
                );
            }
            body = body.child(env_row);
        }

        div()
            .w_full()
            .flex()
            .flex_col()
            .child(Self::lineage_section(
                "运行上下文",
                IconName::FolderOpen,
                None,
                "它此刻处在什么环境里",
                cx,
            ))
            .child(body)
    }

    fn render_lineage_warnings(lineage: &ProcessLineage, cx: &App) -> Div {
        let total = lineage.warnings.len();
        let mut list = div().w_full().flex().flex_col().gap(px(7.0));

        if lineage.warnings.is_empty() {
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(Hsla::from(rgb(design::OK_GREEN)).opacity(0.3))
                    .bg(Hsla::from(rgb(design::OK_GREEN)).opacity(0.07))
                    .child(
                        Icon::new(IconName::CircleCheck)
                            .size(px(14.0))
                            .flex_shrink_0()
                            .text_color(Hsla::from(rgb(design::OK_GREEN))),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::OK_GREEN)))
                            .child("未发现明显风险"),
                    ),
            );
        } else {
            for warning in &lineage.warnings {
                let (color, icon) = match warning.level.as_str() {
                    "danger" => (Hsla::from(rgb(design::ERROR_RED)), IconName::TriangleAlert),
                    "warning" => (Hsla::from(rgb(design::WARN_AMBER)), IconName::TriangleAlert),
                    _ => (cx.theme().muted_foreground, IconName::Info),
                };
                list = list.child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(color.opacity(0.3))
                        .bg(color.opacity(0.07))
                        .child(
                            Icon::new(icon)
                                .size(px(14.0))
                                .flex_shrink_0()
                                .text_color(color),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(12.5))
                                .text_color(color)
                                .child(warning.text.clone()),
                        ),
                );
            }
        }

        div()
            .w_full()
            .flex()
            .flex_col()
            .child(Self::lineage_section(
                "风险提示",
                IconName::TriangleAlert,
                if total > 0 {
                    Some(format!("{total} 条"))
                } else {
                    None
                },
                "需要留意的点",
                cx,
            ))
            .child(list)
    }

    fn lineage_section(
        title: &str,
        icon: IconName,
        count: Option<String>,
        note: &str,
        cx: &App,
    ) -> Div {
        let mut left = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(px(22.0))
                    .h(px(22.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .bg(cx.theme().primary.opacity(0.1))
                    .text_color(cx.theme().primary)
                    .child(Icon::new(icon).size(px(13.0))),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_semibold()
                    .child(title.to_string()),
            );
        if let Some(count) = count {
            left = left.child(design::pill(count, cx));
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .mb(px(10.0))
            .child(left)
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(note.to_string()),
            )
    }

    fn lineage_kv(label: &str, value: &str, mono: bool, cx: &App) -> Div {
        let mut value_div = div()
            .flex_1()
            .min_w_0()
            .text_size(px(12.5))
            .text_color(cx.theme().foreground);
        let mut label_div = div()
            .w(px(96.0))
            .flex_shrink_0()
            .text_size(px(12.0))
            .text_color(cx.theme().muted_foreground)
            .child(label.to_string());
        if mono {
            value_div = value_div.font_family("monospace").text_size(px(12.0));
            label_div = label_div
                .w(px(124.0))
                .font_family("monospace")
                .text_size(px(11.0));
        }
        div()
            .flex()
            .items_start()
            .gap_3()
            .child(label_div)
            .child(value_div.child(value.to_string()))
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

        if !monitoring_enabled {
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

        let list_space = (window.viewport_size().height - LIST_CHROME_HEIGHT).max(MIN_LIST_HEIGHT);
        let rows_fit = (list_space.as_f32() / ROW_HEIGHT).floor().max(1.0);
        let list_max_h = px(rows_fit * ROW_HEIGHT);

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
            .child(div().w(px(ACTION_COLUMN_WIDTH)).child("操作"));

        let table_rows = paginated
            .iter()
            .map(|process| {
                let pid = process.pid;
                let process_name = process.name.clone();
                let lineage_name = process_name.clone();
                div()
                    .flex()
                    .items_center()
                    .h(px(ROW_HEIGHT))
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
                        div()
                            .w(px(ACTION_COLUMN_WIDTH))
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new(("lineage", pid as usize))
                                    .icon(Icon::new(IconName::Network))
                                    .compact()
                                    .tooltip("来龙去脉")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_process_lineage(
                                            pid,
                                            lineage_name.clone(),
                                            window,
                                            cx,
                                        );
                                    })),
                            )
                            .child(
                                Button::new(("kill", pid as usize))
                                    .with_variant(ButtonVariant::Danger)
                                    .icon(Icon::new(IconName::CircleX))
                                    .compact()
                                    .tooltip("终止进程")
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
                    .max_h(list_max_h)
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
            .into_any_element()
    }
}

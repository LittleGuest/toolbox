use std::{collections::HashSet, fs};

use serde::Serialize;
use sysinfo::{
    Pid, Process, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, UpdateKind, Users,
};
use time::{OffsetDateTime, UtcOffset};

use crate::get_system;

const MAX_DEPTH: usize = 64;
const MAX_CHILDREN: usize = 40;
const HIGH_MEMORY: u64 = 1024 * 1024 * 1024;
const LONG_RUNNING_DAYS: u64 = 90;

const SHELLS: &[&str] = &[
    "bash", "zsh", "sh", "fish", "dash", "ksh", "ksh93", "tcsh", "csh", "mksh", "nu", "xonsh",
    "elvish", "yash", "ash",
];

const TERMINALS: &[&str] = &[
    "terminal",
    "vte-spawn",
    "kgx",
    "konsole",
    "kitty",
    "alacritty",
    "wezterm",
    "ghostty",
    "foot",
    "xterm",
    "urxvt",
    "tmux",
    "screen",
    "terminator",
    "tilix",
    "guake",
    "ptyxis",
];

const CONTAINER_TOKENS: &[&str] = &[
    "kubepods",
    "docker-",
    "libpod-",
    "crio-",
    "cri-containerd-",
    "lxc.payload",
    "lxc-",
    "containerd-",
    "podman-",
];

const DESKTOPS: &[&str] = &[
    "niri",
    "sway",
    "hyprland",
    "gnome-shell",
    "plasmashell",
    "kwin",
    "mutter",
    "xfwm",
    "i3",
    "river",
    "labwc",
    "wayfire",
    "cosmic-comp",
    "xfdesktop",
    "marco",
    "fuzzel",
    "rofi",
    "wofi",
    "tofi",
    "bemenu",
    "walker",
    "ulauncher",
    "dmenu",
    "wofi",
];

#[derive(Serialize, Clone, Debug, Default)]
pub struct LineageNode {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub exe: String,
    pub cmdline: String,
    pub user: String,
    pub role: String,
    pub role_label: String,
    pub start_time: String,
    pub run_time: String,
    pub status: String,
    pub is_target: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct LineageField {
    pub label: String,
    pub value: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct LineageSource {
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub supervisor: String,
    pub details: Vec<LineageField>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct LineageSocket {
    pub proto: String,
    pub local: String,
    pub state: String,
    pub public_bind: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct LineageWarning {
    pub level: String,
    pub text: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ProcessLineage {
    pub pid_not_found: bool,
    pub target: LineageNode,
    pub chain: Vec<LineageNode>,
    pub broken_at: Option<u32>,
    pub source: LineageSource,
    pub details: Vec<LineageField>,
    pub context: Vec<LineageField>,
    pub sockets: Vec<LineageSocket>,
    pub children: Vec<LineageNode>,
    pub warnings: Vec<LineageWarning>,
    pub env_count: Option<usize>,
    pub env_highlights: Vec<LineageField>,
}

pub fn process_lineage(pid: u32) -> ProcessLineage {
    let target_pid = Pid::from_u32(pid);
    let (chain, broken_at, children, not_found) = {
        let mut sys = get_system().lock().unwrap();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_user(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_root(UpdateKind::Always)
                .with_environ(UpdateKind::Always)
                .with_cmd(UpdateKind::Always)
                .with_exe(UpdateKind::Always),
        );

        if sys.process(target_pid).is_none() {
            return ProcessLineage {
                pid_not_found: true,
                ..Default::default()
            };
        }

        let users = Users::new_with_refreshed_list();
        let mut visited: HashSet<u32> = HashSet::new();
        let mut rev: Vec<LineageNode> = Vec::new();
        let mut cur = Some(target_pid);
        let mut broken_at = None;

        while let Some(p) = cur {
            if rev.len() >= MAX_DEPTH || !visited.insert(p.as_u32()) {
                break;
            }
            match sys.process(p) {
                Some(proc) => {
                    let node = node_of(proc, &users, p == target_pid);
                    cur = node.ppid.map(Pid::from_u32);
                    rev.push(node);
                }
                None => {
                    broken_at = Some(p.as_u32());
                    break;
                }
            }
        }
        rev.reverse();

        let mut children: Vec<LineageNode> = sys
            .processes()
            .values()
            .filter(|p| {
                p.pid() != target_pid
                    && p.thread_kind().is_none()
                    && p.parent() == Some(target_pid)
            })
            .map(|p| node_of(p, &users, false))
            .collect();
        children.sort_by_key(|n| n.pid);
        children.truncate(MAX_CHILDREN);

        (rev, broken_at, children, false)
    };

    if not_found || chain.is_empty() {
        return ProcessLineage {
            pid_not_found: true,
            ..Default::default()
        };
    }

    let mut lineage = ProcessLineage {
        chain,
        broken_at,
        children,
        target: Default::default(),
        ..Default::default()
    };
    lineage.target = lineage.chain.last().cloned().unwrap_or_default();

    let ancestors: Vec<LineageNode> = lineage.chain[..lineage.chain.len() - 1].to_vec();

    lineage.source = detect_source(pid, &ancestors, &lineage.target, broken_at);
    lineage.details = collect_details(pid, &lineage.target);
    lineage.context = collect_context(pid);
    lineage.sockets = collect_sockets(pid);
    let (env_count, env_highlights) = collect_environ(pid);
    lineage.env_count = env_count;
    lineage.env_highlights = env_highlights;
    lineage.warnings = collect_warnings(&lineage);
    lineage
}

fn node_of(process: &Process, users: &Users, is_target: bool) -> LineageNode {
    let name = process.name().to_string_lossy().to_string();
    let user = process
        .user_id()
        .map(|uid| {
            let raw: u32 = **uid;
            users
                .get_user_by_id(uid)
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| raw.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let (role, role_label) = classify(&name, process.pid().as_u32());
    LineageNode {
        pid: process.pid().as_u32(),
        ppid: process.parent().map(|p| p.as_u32()),
        name,
        exe: process
            .exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        cmdline: process
            .cmd()
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" "),
        user,
        role,
        role_label,
        start_time: fmt_epoch(process.start_time()),
        run_time: fmt_duration(process.run_time()),
        status: status_label(process.status()),
        is_target,
    }
}

fn classify(name: &str, pid: u32) -> (String, String) {
    if pid == 1 {
        return ("init".to_string(), "PID 1 · init".to_string());
    }
    let lower = name.trim_start_matches('-').to_lowercase();
    if lower == "systemd" || lower.starts_with("systemd ") {
        return ("systemd".to_string(), "systemd".to_string());
    }
    if SHELLS.contains(&lower.as_str()) {
        return ("shell".to_string(), "交互式 Shell".to_string());
    }
    if lower.starts_with("sshd") {
        return ("ssh".to_string(), "SSH 服务".to_string());
    }
    if lower.contains("cron") {
        return ("cron".to_string(), "定时任务".to_string());
    }
    if lower.contains("dockerd")
        || lower.contains("containerd")
        || lower.contains("libpod")
        || lower.contains("podman")
        || lower == "runc"
        || lower.contains("crio")
    {
        return ("container".to_string(), "容器运行时".to_string());
    }
    if lower == "login"
        || lower.contains("agetty")
        || lower.contains("sddm")
        || lower.contains("gdm")
        || lower.contains("lightdm")
        || lower.contains("greetd")
    {
        return ("login".to_string(), "登录会话".to_string());
    }
    if DESKTOPS.iter().any(|d| lower.contains(d)) {
        return ("desktop".to_string(), "桌面会话".to_string());
    }
    ("process".to_string(), "普通进程".to_string())
}

fn detect_source(
    pid: u32,
    ancestors: &[LineageNode],
    target: &LineageNode,
    broken_at: Option<u32>,
) -> LineageSource {
    let cgroup = read_cgroup(pid);
    let units = cgroup.as_deref().map(unit_chain).unwrap_or_default();

    if let Some(container) = units
        .iter()
        .rev()
        .find(|u| {
            let lower = u.to_lowercase();
            CONTAINER_TOKENS.iter().any(|t| lower.contains(t))
        })
    {
        return LineageSource {
            kind: "container".to_string(),
            title: "容器".to_string(),
            summary: format!("由容器运行时启动并监管：{container}"),
            supervisor: "由容器运行时监管，容器停止后该进程随之结束".to_string(),
            details: vec![
                field("容器单元", container.clone()),
                field("控制组", cgroup.clone().unwrap_or_default()),
            ],
        };
    }

    if let Some(unit) = units
        .iter()
        .rev()
        .find(|u| u.ends_with(".service") && !is_session_unit(u))
    {
        let (path, description, restart) = read_unit(unit);
        let mut details = vec![field("服务单元", unit.clone())];
        if !description.is_empty() {
            details.push(field("单元描述", description));
        }
        if !path.is_empty() {
            details.push(field("单元文件", path));
        }
        if !restart.is_empty() {
            details.push(field("重启策略", format!("Restart={restart}")));
        }
        return LineageSource {
            kind: "systemd".to_string(),
            title: "systemd 服务".to_string(),
            summary: format!("由 systemd 单元 {unit} 启动"),
            supervisor: if restart.is_empty() {
                "由 systemd 监管，unit 停止时进程一并结束".to_string()
            } else {
                format!("由 systemd 监管，异常退出按 Restart={restart} 处理")
            },
            details,
        };
    }

    if let Some(terminal) = units
        .iter()
        .rev()
        .find(|u| {
            u.ends_with(".scope") && {
                let lower = u.to_lowercase();
                TERMINALS.iter().any(|t| lower.contains(t))
            }
        })
    {
        return LineageSource {
            kind: "terminal".to_string(),
            title: "终端会话".to_string(),
            summary: format!("从图形终端或复用会话中启动：{terminal}"),
            supervisor: "随终端会话关闭而结束".to_string(),
            details: vec![
                field("终端作用域", terminal.clone()),
                field("控制组", cgroup.clone().unwrap_or_default()),
            ],
        };
    }

    let has_user_unit = units.iter().any(|u| is_session_unit(u));

    if ancestors.iter().any(|a| a.role == "ssh") {
        let connection = env_value(pid, "SSH_CONNECTION").unwrap_or_default();
        let tty = env_value(pid, "SSH_TTY").unwrap_or_default();
        let mut details = Vec::new();
        let parts: Vec<&str> = connection.split_whitespace().collect();
        if parts.len() >= 4 {
            details.push(field("远程地址", format!("{}:{}", parts[0], parts[1])));
            details.push(field("对端地址", format!("{}:{}", parts[2], parts[3])));
        }
        if !tty.is_empty() {
            details.push(field("终端", tty));
        }
        if details.is_empty() {
            details.push(field("父进程", "sshd".to_string()));
        }
        return LineageSource {
            kind: "ssh".to_string(),
            title: "SSH 远程会话".to_string(),
            summary: "由远程 SSH 登录派生".to_string(),
            supervisor: "SSH 连接断开后进程通常被终止".to_string(),
            details,
        };
    }

    if let Some(cron) = ancestors.iter().rev().find(|a| a.role == "cron") {
        return LineageSource {
            kind: "cron".to_string(),
            title: "定时任务".to_string(),
            summary: format!("由 {} 的调度触发", cron.name),
            supervisor: "由 cron 触发启动，任务结束后进程自行退出".to_string(),
            details: vec![field("调度进程", format!("{} (PID {})", cron.name, cron.pid))],
        };
    }

    if let Some(shell) = ancestors.iter().rev().find(|a| a.role == "shell") {
        let mut details = vec![field(
            "启动 shell",
            format!("{} (PID {})", shell.name, shell.pid),
        )];
        if !shell.cmdline.is_empty() {
            details.push(field("shell 进程", truncate(&shell.cmdline, 140)));
        }
        if has_user_unit {
            details.push(field(
                "登录会话",
                "存在，但进程已脱离会话的直接父子关系".to_string(),
            ));
        }
        return LineageSource {
            kind: "shell".to_string(),
            title: "交互式 Shell".to_string(),
            summary: format!("由 {} 手工启动", shell.name),
            supervisor: "随启动它的 shell 会话结束而终止".to_string(),
            details,
        };
    }

    if let Some(app) = units
        .iter()
        .rev()
        .find(|u| u.ends_with(".scope") && u.starts_with("app-"))
    {
        return LineageSource {
            kind: "desktop".to_string(),
            title: "桌面应用".to_string(),
            summary: format!("由桌面会话以应用作用域 {app} 启动"),
            supervisor: "随图形会话退出而结束，不随终端关闭".to_string(),
            details: vec![
                field("应用作用域", app.clone()),
                field(
                    "控制组",
                    cgroup.clone().unwrap_or_else(|| "不可读".to_string()),
                ),
            ],
        };
    }

    if let Some(desktop) = ancestors.iter().rev().find(|a| {
        let lower = a.name.to_lowercase();
        DESKTOPS.iter().any(|d| lower.contains(d))
    }) {
        return LineageSource {
            kind: "desktop".to_string(),
            title: "桌面会话".to_string(),
            summary: format!("由图形会话 {} (PID {}) 启动", desktop.name, desktop.pid),
            supervisor: "随图形会话退出而结束".to_string(),
            details: vec![
                field("桌面进程", format!("{} (PID {})", desktop.name, desktop.pid)),
                field(
                    "控制组",
                    cgroup.clone().unwrap_or_else(|| "不可读".to_string()),
                ),
            ],
        };
    }

    if broken_at.is_some() {
        return LineageSource {
            kind: "unknown".to_string(),
            title: "起源已丢失".to_string(),
            summary: format!(
                "原始父进程（PID {}）已退出，无法继续往上追溯",
                broken_at.unwrap_or_default()
            ),
            supervisor: "无人监管，或监管者已退出".to_string(),
            details: vec![
                field("断链 PID", broken_at.unwrap_or_default().to_string()),
                field(
                    "现有父进程",
                    target.ppid.map(|p| p.to_string()).unwrap_or_else(|| "无".to_string()),
                ),
            ],
        };
    }

    if let Some(parent) = ancestors.last() {
        if parent.pid == 1 {
            return LineageSource {
                kind: "adopted".to_string(),
                title: "被 init 收养".to_string(),
                summary: format!(
                    "父进程是 PID 1（{}），原始启动者已退出，只能追溯到 init",
                    parent.name
                ),
                supervisor: "无人监管，原始父进程退出后由 init 收养".to_string(),
                details: vec![
                    field("收养者", format!("{} (PID 1)", parent.name)),
                    field("含义", "进程已 daemon 化，或启动它的会话已经结束".to_string()),
                ],
            };
        }
        return LineageSource {
            kind: "parent".to_string(),
            title: "父进程".to_string(),
            summary: format!("由 {} (PID {}) 直接启动", parent.name, parent.pid),
            supervisor: "父进程退出后本进程会成为孤儿并被 init 收养".to_string(),
            details: vec![field("父进程", format!("{} (PID {})", parent.name, parent.pid))],
        };
    }

    LineageSource {
        kind: "unknown".to_string(),
        title: "系统最上层进程".to_string(),
        summary: "该进程没有父进程，它就是整条链的起点".to_string(),
        supervisor: "系统初始化进程".to_string(),
        details: vec![field("名称", target.name.clone())],
    }
}

fn collect_details(pid: u32, target: &LineageNode) -> Vec<LineageField> {
    let unreadable = "不可读（需要 root 权限）".to_string();
    vec![
        field("进程名", target.name.clone()),
        field("PID", target.pid.to_string()),
        field(
            "父进程 PID",
            target
                .ppid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "无".to_string()),
        ),
        field("运行用户", target.user.clone()),
        field("进程角色", target.role_label.clone()),
        field(
            "可执行文件",
            if target.exe.is_empty() {
                unreadable.clone()
            } else {
                target.exe.clone()
            },
        ),
        field(
            "命令行",
            if target.cmdline.is_empty() {
                unreadable
            } else {
                truncate(&target.cmdline, 300)
            },
        ),
        field("启动时间", target.start_time.clone()),
        field("已运行", target.run_time.clone()),
        field("状态", target.status.clone()),
    ]
    .into_iter()
    .chain(
        read_link(&format!("/proc/{pid}/root"))
            .map(|root| field("根目录", root)),
    )
    .collect()
}

fn collect_context(pid: u32) -> Vec<LineageField> {
    let mut rows = Vec::new();
    let cwd = read_link(&format!("/proc/{pid}/cwd"))
        .filter(|c| c.starts_with('/'))
        .unwrap_or_default();
    rows.push(field(
        "工作目录",
        if cwd.is_empty() {
            "不可读（需要 root 权限）".to_string()
        } else {
            cwd.clone()
        },
    ));

    match git_info(&cwd) {
        Some((repo, branch)) => {
            rows.push(field("Git 仓库", repo));
            rows.push(field(
                "Git 分支",
                if branch.is_empty() {
                    "未知".to_string()
                } else {
                    branch
                },
            ));
        }
        None => rows.push(field(
            "Git 仓库",
            if cwd.is_empty() {
                "无法判断（工作目录不可读）".to_string()
            } else {
                "不在 Git 仓库内".to_string()
            },
        )),
    }

    rows.push(field(
        "打开的文件描述符",
        match count_fds(pid) {
            Some(count) => count.to_string(),
            None => "不可读（需要 root 权限）".to_string(),
        },
    ));

    if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
        for (key, label) in [
            ("Threads", "线程数"),
            ("VmRSS", "常驻内存"),
            ("VmSize", "虚拟内存"),
            ("voluntary_ctxt_switches", "主动上下文切换"),
        ] {
            if let Some(value) = status
                .lines()
                .find(|l| l.starts_with(&format!("{key}:")))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
            {
                rows.push(field(label, value));
            }
        }
    }

    if let Some(sec) = read_security_context(pid) {
        rows.push(field("安全上下文", sec));
    }

    rows
}

fn collect_environ(pid: u32) -> (Option<usize>, Vec<LineageField>) {
    let Ok(raw) = fs::read(format!("/proc/{pid}/environ")) else {
        return (None, Vec::new());
    };
    let mut count = 0usize;
    let mut highlights = Vec::new();
    for entry in raw.split(|b| *b == 0) {
        if entry.is_empty() {
            continue;
        }
        count += 1;
        let text = String::from_utf8_lossy(entry).to_string();
        let Some((key, value)) = text.split_once('=') else {
            continue;
        };
        let interesting = key.starts_with("LD_")
            || matches!(
                key,
                "SSH_CONNECTION"
                    | "SSH_TTY"
                    | "DISPLAY"
                    | "WAYLAND_DISPLAY"
                    | "XDG_SESSION_TYPE"
                    | "WSL_DISTRO_NAME"
                    | "PYTHONPATH"
                    | "NODE_OPTIONS"
            );
        if interesting {
            highlights.push(field(key, truncate(value, 120)));
        }
    }
    (Some(count), highlights)
}

fn collect_warnings(lineage: &ProcessLineage) -> Vec<LineageWarning> {
    let mut out = Vec::new();

    if lineage.target.user == "root" {
        out.push(warn("danger", "以 root 身份运行"));
    }

    for socket in lineage
        .sockets
        .iter()
        .filter(|s| s.public_bind && s.state == "LISTEN")
    {
        out.push(warn(
            "danger",
            &format!("监听在公网地址 {}，对外暴露", socket.local),
        ));
    }

    if let Some(entry) = lineage
        .env_highlights
        .iter()
        .find(|f| f.label == "LD_PRELOAD")
    {
        out.push(warn(
            "danger",
            &format!("检测到 LD_PRELOAD 注入：{}", entry.value),
        ));
    }

    if lineage.target.exe.contains("(deleted)") {
        out.push(warn("danger", "可执行文件已被删除，但进程仍在运行"));
    }

    if lineage.target.status == "僵尸" {
        out.push(warn("danger", "僵尸进程，未被父进程回收"));
    }

    if matches!(lineage.source.kind.as_str(), "adopted" | "unknown") {
        out.push(warn("warning", "原始父进程已退出，无法确定是谁启动了它"));
    }

    if lineage.broken_at.is_some() && lineage.source.kind != "unknown" {
        out.push(warn("warning", "父进程链在追溯过程中断裂，完整启动来源不可得"));
    }

    let resident = lineage
        .context
        .iter()
        .find(|f| f.label == "常驻内存")
        .and_then(|f| parse_kb(&f.value));
    if resident.is_some_and(|memory| memory > HIGH_MEMORY) {
        out.push(warn(
            "warning",
            &format!(
                "常驻内存超过 1 GB（{}）",
                human_bytes(resident.unwrap_or_default())
            ),
        ));
    }

    let running_days = lineage
        .target
        .run_time
        .split(' ')
        .next()
        .filter(|_| lineage.target.run_time.contains(" 天"))
        .and_then(|d| d.parse::<u64>().ok());
    if running_days.is_some_and(|days| days >= LONG_RUNNING_DAYS) {
        out.push(warn(
            "warning",
            &format!("已持续运行 {}，建议确认是否仍需要", lineage.target.run_time),
        ));
    }

    if lineage.target.status == "不可中断等待" {
        out.push(warn("warning", "处于不可中断的磁盘等待状态"));
    }

    if lineage.chain.len() > 12 {
        out.push(warn("info", "启动链路很长，可能经过多层脚本或包装器"));
    }

    if !lineage.children.is_empty() {
        out.push(warn(
            "info",
            &format!("当前有 {} 个子进程正在运行", lineage.children.len()),
        ));
    }

    out
}

fn warn(level: &str, text: &str) -> LineageWarning {
    LineageWarning {
        level: level.to_string(),
        text: text.to_string(),
    }
}

fn field(label: &str, value: impl Into<String>) -> LineageField {
    LineageField {
        label: label.to_string(),
        value: value.into(),
    }
}

fn is_session_unit(unit: &str) -> bool {
    unit == "init.scope"
        || unit.starts_with("user@")
        || unit.starts_with("session-")
        || unit.starts_with("user-")
}

fn read_cgroup(pid: u32) -> Option<String> {
    let raw = fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?;
    raw.lines()
        .filter_map(|line| line.split(':').nth(2))
        .map(|p| p.trim())
        .find(|p| !p.is_empty() && *p != "/")
        .map(|p| p.to_string())
}

fn unit_chain(cgroup: &str) -> Vec<String> {
    cgroup
        .split('/')
        .filter(|c| c.ends_with(".service") || c.ends_with(".scope"))
        .map(|c| c.to_string())
        .collect()
}

fn read_unit(unit: &str) -> (String, String, String) {
    let candidates = [
        format!("/etc/systemd/system/{unit}"),
        format!("/run/systemd/system/{unit}"),
        format!("/usr/lib/systemd/system/{unit}"),
        format!("/lib/systemd/system/{unit}"),
        format!("/usr/local/lib/systemd/system/{unit}"),
        format!("/usr/lib/systemd/user/{unit}"),
        format!("/usr/share/systemd/user/{unit}"),
    ];
    for path in candidates {
        if let Ok(content) = fs::read_to_string(&path) {
            let description = ini_value(&content, "Description");
            let restart = ini_value(&content, "Restart");
            return (path, description, restart);
        }
    }
    (String::new(), String::new(), String::new())
}

fn ini_value(content: &str, key: &str) -> String {
    let prefix = format!("{key}=");
    content
        .lines()
        .map(|l| l.trim())
        .find(|l| l.starts_with(&prefix))
        .and_then(|l| l.split_once('='))
        .map(|(_, v)| v.trim().to_string())
        .unwrap_or_default()
}

fn read_link(path: &str) -> Option<String> {
    fs::read_link(path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

fn count_fds(pid: u32) -> Option<usize> {
    fs::read_dir(format!("/proc/{pid}/fd"))
        .ok()
        .map(|entries| entries.filter_map(|e| e.ok()).count())
}

fn read_security_context(pid: u32) -> Option<String> {
    let attr = fs::read_to_string(format!("/proc/{pid}/attr/current")).ok()?;
    let value = attr.trim();
    if value.is_empty() || value.starts_with("unconfined") {
        None
    } else {
        Some(value.to_string())
    }
}

fn git_info(cwd: &str) -> Option<(String, String)> {
    if !cwd.starts_with('/') {
        return None;
    }
    let mut dir = std::path::PathBuf::from(cwd);
    for _ in 0..24 {
        let git = dir.join(".git");
        if git.exists() {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.to_string_lossy().to_string());
            let head_path = if git.is_dir() {
                git.join("HEAD")
            } else {
                let content = fs::read_to_string(&git).ok()?;
                let target = content.trim().strip_prefix("gitdir: ").unwrap_or("").trim();
                if target.is_empty() {
                    return Some((name, String::new()));
                }
                dir.join(target).join("HEAD")
            };
            let branch = fs::read_to_string(head_path)
                .ok()
                .map(|h| {
                    let h = h.trim().to_string();
                    h.strip_prefix("ref: refs/heads/")
                        .map(|b| b.to_string())
                        .unwrap_or_else(|| {
                            if h.len() >= 8 && h.chars().all(|c| c.is_ascii_hexdigit()) {
                                format!("detached@{}", &h[..8])
                            } else {
                                h
                            }
                        })
                })
                .unwrap_or_default();
            return Some((name, branch));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn collect_sockets(pid: u32) -> Vec<LineageSocket> {
    let inodes = socket_inodes(pid);
    if inodes.is_empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (file, proto) in [
        ("/proc/net/tcp", "TCP"),
        ("/proc/net/tcp6", "TCP6"),
        ("/proc/net/udp", "UDP"),
        ("/proc/net/udp6", "UDP6"),
    ] {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        for line in content.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 10 {
                continue;
            }
            let Ok(inode) = cols[9].parse::<u64>() else {
                continue;
            };
            if !inodes.contains(&inode) {
                continue;
            }
            let Some((ip, port)) = decode_addr(cols[1], proto.ends_with('6')) else {
                continue;
            };
            let state_code = u8::from_str_radix(cols[3], 16).unwrap_or(0);
            let state = state_name(state_code, proto.starts_with("UDP"));
            if proto.starts_with("TCP") && state != "LISTEN" {
                continue;
            }
            found.push(LineageSocket {
                proto: proto.to_string(),
                local: format!("{ip}:{port}"),
                state: state.to_string(),
                public_bind: ip == "0.0.0.0" || ip == "::",
            });
        }
    }
    found.sort_by(|a, b| a.local.cmp(&b.local));
    found.dedup_by(|a, b| a.local == b.local && a.proto == b.proto);
    found
}

fn socket_inodes(pid: u32) -> HashSet<u64> {
    let mut set = HashSet::new();
    let Ok(entries) = fs::read_dir(format!("/proc/{pid}/fd")) else {
        return set;
    };
    for entry in entries.flatten() {
        let Ok(target) = fs::read_link(entry.path()) else {
            continue;
        };
        let target = target.to_string_lossy();
        let Some(rest) = target.strip_prefix("socket:[") else {
            continue;
        };
        let Some(num) = rest.strip_suffix(']') else {
            continue;
        };
        if let Ok(inode) = num.parse::<u64>() {
            set.insert(inode);
        }
    }
    set
}

fn decode_addr(hex: &str, v6: bool) -> Option<(String, u16)> {
    let (addr, port) = hex.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    if v6 {
        if addr.len() != 32 {
            return None;
        }
        let mut groups = Vec::with_capacity(4);
        for chunk in 0..4 {
            let part = &addr[chunk * 8..chunk * 8 + 8];
            groups.push(format!(
                "{}{}{}{}",
                &part[6..8],
                &part[4..6],
                &part[2..4],
                &part[0..2]
            ));
        }
        let joined = groups.join(":");
        let ip = match joined.as_str() {
            "0000:0000:0000:0000:0000:0000:0000:0000" => "::".to_string(),
            "0000:0000:0000:0000:0000:0000:0000:0001" => "::1".to_string(),
            _ => joined,
        };
        Some((ip, port))
    } else {
        if addr.len() != 8 {
            return None;
        }
        let byte = |i: usize| u32::from_str_radix(&addr[i * 2..i * 2 + 2], 16).unwrap_or(0);
        Some((
            format!("{}.{}.{}.{}", byte(3), byte(2), byte(1), byte(0)),
            port,
        ))
    }
}

fn state_name(code: u8, udp: bool) -> &'static str {
    if udp {
        return if code == 0x07 { "UNCONN" } else { "CONN" };
    }
    match code {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RECV",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSE",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
}

fn env_value(pid: u32, key: &str) -> Option<String> {
    let raw = fs::read(format!("/proc/{pid}/environ")).ok()?;
    for entry in raw.split(|b| *b == 0) {
        let text = String::from_utf8_lossy(entry);
        let Some((k, v)) = text.split_once('=') else {
            continue;
        };
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

fn status_label(status: ProcessStatus) -> String {
    match status {
        ProcessStatus::Idle => "空闲",
        ProcessStatus::Run => "运行中",
        ProcessStatus::Sleep => "休眠",
        ProcessStatus::Stop => "已停止",
        ProcessStatus::Zombie => "僵尸",
        ProcessStatus::Tracing => "调试暂停",
        ProcessStatus::Dead => "不可中断等待",
        ProcessStatus::Wakekill => "唤醒被中断",
        ProcessStatus::Waking => "唤醒中",
        ProcessStatus::Parked => "已挂起",
        ProcessStatus::LockBlocked => "等待内核锁",
        ProcessStatus::UninterruptibleDiskSleep => "不可中断等待",
        ProcessStatus::Unknown(_) => "未知",
    }
    .to_string()
}

fn fmt_epoch(secs: u64) -> String {
    if secs == 0 {
        return "未知".to_string();
    }
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    match OffsetDateTime::from_unix_timestamp(secs as i64) {
        Ok(dt) => {
            let dt = dt.to_offset(offset);
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                dt.year(),
                dt.month() as u8,
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )
        }
        Err(_) => secs.to_string(),
    }
}

fn fmt_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs} 秒")
    } else if secs < 3600 {
        format!("{} 分 {} 秒", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{} 小时 {} 分", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{} 天 {} 小时", secs / 86400, (secs % 86400) / 3600)
    }
}

fn parse_kb(text: &str) -> Option<u64> {
    let value = text.split_whitespace().next()?.parse::<u64>().ok()?;
    if text.contains("kB") {
        Some(value * 1024)
    } else {
        Some(value)
    }
}

fn human_bytes(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lineage_of_self_is_complete() {
        let me = std::process::id();
        let l = process_lineage(me);
        assert!(!l.pid_not_found);
        assert_eq!(l.target.pid, me);
        assert!(!l.chain.is_empty());
        assert_eq!(l.chain.last().unwrap().pid, me);
        assert!(l.chain.iter().all(|n| !n.name.is_empty()));
        assert!(l.chain.iter().filter(|n| n.is_target).count() == 1);
        assert!(l.context.iter().any(|f| f.label == "工作目录"));
        assert!(!l.source.kind.is_empty());
        assert!(!l.source.title.is_empty());
        assert!(!l.source.supervisor.is_empty());
    }

    #[test]
    fn unknown_pid_is_reported() {
        let l = process_lineage(4_000_000);
        assert!(l.pid_not_found);
        assert!(l.chain.is_empty());
        assert!(l.warnings.is_empty());
        assert!(l.children.is_empty());
    }

    #[test]
    fn init_is_chain_root() {
        let l = process_lineage(1);
        assert!(!l.pid_not_found);
        assert_eq!(l.chain.len(), 1);
        assert_eq!(l.target.pid, 1);
        assert_eq!(l.target.role, "init");
    }

    #[test]
    fn children_exclude_self_and_threads() {
        let me = std::process::id();
        let l = process_lineage(me);
        assert!(l.children.iter().all(|c| c.pid != me));
        assert!(l.chain.len() <= MAX_DEPTH);
    }
}

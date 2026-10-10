use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ::base::blank_line::{
    self, BlankMode, FileReport, FileState, OutputMode, TEXT_EXTENSIONS, VcsKind,
};
use gpui_kit::{
    component::{
        button::*,
        checkbox::Checkbox,
        progress::Progress,
        radio::{Radio, RadioGroup},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Success,
    Warn,
    Error,
    Info,
}

struct Entry {
    path: String,
    checked: bool,
    vcs: Option<VcsKind>,
}

pub struct BlankLineTool {
    entries: Vec<Entry>,
    mode: BlankMode,
    output_mode: OutputMode,
    busy: bool,
    progress: f32,
    reports: Vec<FileReport>,
    ignored: usize,
    output_dir: Option<String>,
    message: String,
    tone: Tone,
    cancel: Arc<AtomicBool>,
    file_list_scroll: ScrollHandle,
    result_list_scroll: ScrollHandle,
}

impl BlankLineTool {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            entries: Vec::new(),
            mode: BlankMode::RemoveAll,
            output_mode: OutputMode::NewFile,
            busy: false,
            progress: 0.0,
            reports: Vec::new(),
            ignored: 0,
            output_dir: None,
            message: String::new(),
            tone: Tone::Info,
            cancel: Arc::new(AtomicBool::new(false)),
            file_list_scroll: ScrollHandle::new(),
            result_list_scroll: ScrollHandle::new(),
        }
    }

    fn set_msg(&mut self, tone: Tone, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.tone = tone;
        self.message = msg.into();
        cx.notify();
    }

    fn clear_msg(&mut self, cx: &mut Context<Self>) {
        self.message.clear();
        cx.notify();
    }

    fn pick_files(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async {
            rfd::AsyncFileDialog::new()
                .set_title("选择文件")
                .add_filter("文本 / 代码 / 配置", TEXT_EXTENSIONS)
                .add_filter("全部文件（自动识别文本）", &["*"])
                .pick_files()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(picked) = task.await else {
                return;
            };
            let paths: Vec<String> = picked
                .iter()
                .map(|f| f.path().to_string_lossy().to_string())
                .collect();
            let _ = this.update(cx, |this, cx| {
                this.add_paths(paths, cx);
            });
        })
        .detach();
    }

    fn pick_folder(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_executor().spawn(async {
            rfd::AsyncFileDialog::new()
                .set_title("选择文件夹")
                .pick_folder()
                .await
        });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let Some(dir) = task.await else {
                return;
            };
            let path = dir.path().to_string_lossy().to_string();
            let _ = this.update(cx, |this, cx| {
                this.add_paths(vec![path], cx);
            });
        })
        .detach();
    }

    fn add_paths(&mut self, paths: Vec<String>, cx: &mut Context<Self>) {
        let mut added = 0usize;
        for path in paths {
            let is_dir = Path::new(&path).is_dir();
            if !is_dir && !blank_line::is_processable(Path::new(&path)) {
                continue;
            }
            if self.entries.iter().any(|entry| entry.path == path) {
                continue;
            }
            let vcs = is_dir
                .then(|| blank_line::detect_vcs(Path::new(&path)).map(|info| info.kind))
                .flatten();
            self.entries.push(Entry {
                path,
                checked: true,
                vcs,
            });
            added += 1;
        }

        if added == 0 {
            self.set_msg(Tone::Info, "没有新增可处理的文本文件", cx);
        } else {
            self.set_msg(
                Tone::Info,
                format!("已添加 {added} 项，共 {} 项", self.entries.len()),
                cx,
            );
        }
        self.reports.clear();
        self.file_list_scroll.scroll_to_bottom();
        cx.notify();
    }

    fn remove_at(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.reports.clear();
            cx.notify();
        }
    }

    fn remove_checked(&mut self, cx: &mut Context<Self>) {
        let before = self.entries.len();
        self.entries.retain(|entry| !entry.checked);
        let removed = before - self.entries.len();
        if removed == 0 {
            self.set_msg(Tone::Warn, "请先勾选要移除的条目", cx);
        } else {
            self.reports.clear();
            self.set_msg(Tone::Info, format!("已移除 {removed} 项"), cx);
        }
    }

    fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.entries.clear();
        self.reports.clear();
        self.output_dir = None;
        self.progress = 0.0;
        self.clear_msg(cx);
    }

    fn toggle_all(&mut self, cx: &mut Context<Self>) {
        if self.entries.is_empty() {
            return;
        }
        let all_selected = self.entries.iter().all(|entry| entry.checked);
        for entry in self.entries.iter_mut() {
            entry.checked = !all_selected;
        }
        cx.notify();
    }

    fn targets(&self) -> Vec<String> {
        let checked: Vec<String> = self
            .entries
            .iter()
            .filter(|entry| entry.checked)
            .map(|entry| entry.path.clone())
            .collect();
        if checked.is_empty() {
            self.entries
                .iter()
                .map(|entry| entry.path.clone())
                .collect()
        } else {
            checked
        }
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let targets = self.targets();
        if targets.is_empty() {
            self.set_msg(Tone::Warn, "请先添加要处理的文件或文件夹", cx);
            return;
        }

        self.busy = true;
        self.progress = 0.0;
        self.reports.clear();
        self.output_dir = None;
        self.message.clear();
        self.cancel.store(false, Ordering::SeqCst);
        cx.notify();

        let mode = self.mode;
        let output_mode = self.output_mode;
        let cancel = self.cancel.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let scanned = cx
                .background_executor()
                .spawn(async move { blank_line::scan(&targets) })
                .await;

            let ignored = scanned.ignored_total();

            if scanned.files.is_empty() {
                let _ = this.update(cx, |this, cx| {
                    this.busy = false;
                    this.progress = 0.0;
                    this.ignored = ignored;
                    let msg = if ignored > 0 {
                        format!("没有可处理的文本文件，已按忽略规则跳过 {ignored} 项")
                    } else {
                        "没有找到可处理的文本文件".to_string()
                    };
                    this.set_msg(Tone::Warn, msg, cx);
                });
                return;
            }

            let files = scanned.files;
            let total = files.len();
            let mut reports: Vec<FileReport> = Vec::with_capacity(total);
            let mut canceled = false;

            for (index, path) in files.iter().enumerate() {
                if cancel.load(Ordering::SeqCst) {
                    canceled = true;
                    break;
                }

                let path = path.clone();
                let report = cx
                    .background_executor()
                    .spawn(async move {
                        blank_line::process_one(Path::new(&path), mode, output_mode)
                    })
                    .await;
                reports.push(report);

                let snapshot = reports.clone();
                let progress = (index + 1) as f32 / total as f32 * 100.0;
                let _ = this.update(cx, |this, cx| {
                    this.progress = progress;
                    this.reports = snapshot;
                    this.result_list_scroll.scroll_to_bottom();
                    cx.notify();
                });
            }

            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                this.ignored = ignored;
                this.output_dir = reports.iter().find_map(|report| {
                    report.output.as_deref().and_then(|output| {
                        Path::new(output)
                            .parent()
                            .map(|dir| dir.to_string_lossy().to_string())
                    })
                });

                let done = reports
                    .iter()
                    .filter(|r| r.state == FileState::Done)
                    .count();
                let unchanged = reports
                    .iter()
                    .filter(|r| r.state == FileState::Unchanged)
                    .count();
                let failed = reports
                    .iter()
                    .filter(|r| r.state == FileState::Failed)
                    .count();
                let removed: usize = reports.iter().map(|r| r.removed).sum();
                let processed = reports.len();

                let tone = if failed > 0 && done == 0 {
                    Tone::Error
                } else if failed > 0 {
                    Tone::Warn
                } else if done == 0 {
                    Tone::Info
                } else {
                    Tone::Success
                };
                let msg = format!(
                    "处理 {processed} 个文件：已清理 {done} · 无需处理 {unchanged} · 失败 {failed} · 共删除 {removed} 行空行"
                );

                this.reports = reports;

                if canceled {
                    this.set_msg(
                        Tone::Warn,
                        format!(
                            "已终止：完成 {processed} 个，剩余 {} 个未处理",
                            total - processed
                        ),
                        cx,
                    );
                    return;
                }

                this.set_msg(tone, msg, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn cancel_run(&mut self, cx: &mut Context<Self>) {
        if !self.busy {
            return;
        }
        self.cancel.store(true, Ordering::SeqCst);
        self.set_msg(Tone::Warn, "正在终止，当前文件处理完即停止…", cx);
    }

    fn open_output_dir(&mut self, cx: &mut Context<Self>) {
        let Some(dir) = self.output_dir.clone() else {
            self.set_msg(Tone::Warn, "还没有输出位置，请先开始处理", cx);
            return;
        };
        cx.background_executor()
            .spawn(async move { open_in_file_manager(&dir) })
            .detach();
        self.set_msg(Tone::Info, "已在文件管理器中打开输出位置", cx);
    }

    fn picker_zone(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let count = self.entries.len();
        div()
            .id("bl-picker")
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_3()
            .py(px(22.0))
            .rounded(px(12.0))
            .border_1()
            .border_dashed()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .cursor_pointer()
            .hover(|this| this.border_color(cx.theme().primary.opacity(0.5)))
            .on_click(cx.listener(|this, _, _, cx| {
                this.pick_files(cx);
            }))
            .child(
                div()
                    .w(px(38.0))
                    .h(px(38.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(11.0))
                    .bg(cx.theme().primary.opacity(0.1))
                    .text_color(cx.theme().primary)
                    .child(Icon::new(IconName::FileText).size(px(19.0))),
            )
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(13.5))
                            .font_medium()
                            .text_color(cx.theme().foreground)
                            .child(if count == 0 {
                                "点击这里选择文本文件"
                            } else {
                                "继续添加文件或文件夹"
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "支持 md / txt / html / xml / json / yaml / rs / js 等文本文件，二进制与 .gitignore 命中的文件自动跳过",
                            ),
                    ),
            )
    }

    fn file_row(&self, index: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let entry = &self.entries[index];
        let path = Path::new(&entry.path);
        let is_dir = path.is_dir();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.path.clone());
        let location = path
            .parent()
            .map(|dir| dir.to_string_lossy().to_string())
            .filter(|dir| !dir.is_empty())
            .unwrap_or_else(|| "/".to_string());
        let checked = entry.checked;
        let kind_label = if is_dir {
            entry
                .vcs
                .map(|kind| kind.tag().to_string())
                .unwrap_or_else(|| "文件夹".to_string())
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_uppercase())
                .unwrap_or_else(|| "文本".to_string())
        };

        div()
            .id(("bl-row", index))
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .when(index > 0, |row| {
                row.border_t_1().border_color(cx.theme().border)
            })
            .child(
                Checkbox::new(("bl-check", index))
                    .checked(checked)
                    .on_click(cx.listener(move |this, value, _, cx| {
                        if let Some(entry) = this.entries.get_mut(index) {
                            entry.checked = *value;
                        }
                        this.reports.clear();
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().foreground)
                            .truncate()
                            .child(name),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_family("monospace")
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(location),
                    ),
            )
            .child(design::mini_tag(
                kind_label,
                cx.theme().muted_foreground.opacity(0.12),
                cx.theme().muted_foreground,
            ))
            .child(
                Button::new(("bl-remove", index))
                    .ghost()
                    .icon(Icon::new(IconName::Close))
                    .tooltip("从列表移除")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.remove_at(index, cx);
                    })),
            )
    }

    fn options_block(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let mode_index = match self.mode {
            BlankMode::RemoveAll => 0usize,
            BlankMode::Collapse => 1usize,
        };

        let mode_radio = RadioGroup::horizontal("bl-mode")
            .selected_index(Some(mode_index))
            .on_click({
                let weak = weak.clone();
                move |index: &usize, _, cx: &mut App| {
                    let mode = if *index == 1 {
                        BlankMode::Collapse
                    } else {
                        BlankMode::RemoveAll
                    };
                    if let Some(this) = weak.upgrade() {
                        this.update(cx, |this, cx| {
                            this.mode = mode;
                            cx.notify();
                        });
                    }
                }
            })
            .child(Radio::new("bl-mode-remove").label("删除所有空行"))
            .child(Radio::new("bl-mode-collapse").label("连续空行合并为一个"));

        let output_index = match self.output_mode {
            OutputMode::NewFile => 0usize,
            OutputMode::Overwrite => 1usize,
        };

        let output_radio = RadioGroup::horizontal("bl-output")
            .selected_index(Some(output_index))
            .on_click({
                let weak = weak.clone();
                move |index: &usize, _, cx: &mut App| {
                    let output_mode = if *index == 1 {
                        OutputMode::Overwrite
                    } else {
                        OutputMode::NewFile
                    };
                    if let Some(this) = weak.upgrade() {
                        this.update(cx, |this, cx| {
                            this.output_mode = output_mode;
                            cx.notify();
                        });
                    }
                }
            })
            .child(Radio::new("bl-out-new").label("另存为新文件（推荐）"))
            .child(Radio::new("bl-out-overwrite").label("覆盖原文件（自动 .bak 备份）"));

        div()
            .flex()
            .flex_wrap()
            .gap_3()
            .w_full()
            .child(self.option_panel("空行处理", mode_radio, cx))
            .child(self.option_panel("输出方式", output_radio, cx))
    }

    fn option_panel(&self, title: &'static str, radio: RadioGroup, cx: &mut Context<Self>) -> Div {
        div()
            .flex_1()
            .min_w(px(280.0))
            .flex_col()
            .gap_2()
            .p(px(14.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(design::editor_label(title.to_string(), cx))
            .child(div().flex().w_full().child(radio))
    }

    fn result_row(
        &self,
        index: usize,
        report: &FileReport,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (label, bg, fg) = match report.state {
            FileState::Done => (
                "已清理",
                design::tint(design::OK_GREEN, 0.14),
                Hsla::from(rgb(design::OK_GREEN)),
            ),
            FileState::Unchanged => (
                "无空行",
                cx.theme().muted_foreground.opacity(0.12),
                cx.theme().muted_foreground,
            ),
            FileState::Failed => (
                "失败",
                design::tint(design::ERROR_RED, 0.14),
                Hsla::from(rgb(design::ERROR_RED)),
            ),
        };

        let name = Path::new(&report.path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| report.path.clone());

        let detail = match report.state {
            FileState::Done => format!("删除 {} 行空行", report.removed),
            FileState::Unchanged => "没有空行需要处理".to_string(),
            FileState::Failed => report
                .detail
                .clone()
                .unwrap_or_else(|| "处理失败".to_string()),
        };

        div()
            .id(("bl-result", index))
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .when(index > 0, |row| {
                row.border_t_1().border_color(cx.theme().border)
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(cx.theme().foreground)
                            .truncate()
                            .child(name),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_family("monospace")
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(match report.output.as_deref() {
                                Some(output) => output.to_string(),
                                None => report.path.clone(),
                            }),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(cx.theme().muted_foreground)
                    .child(detail),
            )
            .child(design::mini_tag(label, bg, fg))
    }

    fn results_block(&self, cx: &mut Context<Self>) -> Div {
        let done = self
            .reports
            .iter()
            .filter(|r| r.state == FileState::Done)
            .count();
        let unchanged = self
            .reports
            .iter()
            .filter(|r| r.state == FileState::Unchanged)
            .count();
        let failed = self
            .reports
            .iter()
            .filter(|r| r.state == FileState::Failed)
            .count();
        let removed: usize = self.reports.iter().map(|r| r.removed).sum();
        let ignored = self.ignored;

        div()
            .flex_col()
            .gap_3()
            .w_full()
            .mt_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(design::stat_pill(
                        done.to_string(),
                        "已清理",
                        Hsla::from(rgb(design::OK_GREEN)),
                        cx,
                    ))
                    .child(design::stat_pill(
                        unchanged.to_string(),
                        "无需处理",
                        cx.theme().muted_foreground,
                        cx,
                    ))
                    .child(design::stat_pill(
                        failed.to_string(),
                        "失败",
                        Hsla::from(rgb(design::ERROR_RED)),
                        cx,
                    ))
                    .child(design::stat_pill(
                        removed.to_string(),
                        "行空行已删除",
                        cx.theme().primary,
                        cx,
                    ))
                    .when(ignored > 0, |row| {
                        row.child(design::stat_pill(
                            ignored.to_string(),
                            "已忽略（.gitignore）",
                            cx.theme().muted_foreground,
                            cx,
                        ))
                    }),
            )
            .child(
                div()
                    .id("bl-result-list")
                    .flex_col()
                    .max_h(px(280.0))
                    .overflow_y_scroll()
                    .track_scroll(&self.result_list_scroll)
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .children(
                        self.reports
                            .iter()
                            .enumerate()
                            .map(|(index, report)| self.result_row(index, report, cx)),
                    ),
            )
    }
}

fn open_in_file_manager(path: &str) {
    #[cfg(target_os = "windows")]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";

    let _ = std::process::Command::new(program).arg(path).spawn();
}

impl Render for BlankLineTool {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.entries.len();
        let checked = self.entries.iter().filter(|entry| entry.checked).count();
        let all_selected = count > 0 && checked == count;
        let busy = self.busy;
        let has_entries = count > 0;
        let has_reports = !self.reports.is_empty();

        let tone_color = match self.tone {
            Tone::Success => Hsla::from(rgb(design::OK_GREEN)),
            Tone::Warn => Hsla::from(rgb(design::WARN_AMBER)),
            Tone::Error => Hsla::from(rgb(design::ERROR_RED)),
            Tone::Info => cx.theme().muted_foreground,
        };
        let message = self.message.clone();
        let output_dir = self.output_dir.clone();

        let action_button = if busy {
            Button::new("bl-cancel")
                .danger()
                .icon(Icon::new(IconName::CircleX))
                .tooltip("终止处理（当前文件完成后停止）")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.cancel_run(cx);
                }))
        } else {
            Button::new("bl-start")
                .primary()
                .icon(Icon::new(IconName::Play))
                .disabled(!has_entries)
                .tooltip("按当前规则开始处理")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.start(cx);
                }))
        };

        let mut list = div()
            .id("bl-file-list")
            .flex_col()
            .max_h(px(280.0))
            .overflow_y_scroll()
            .track_scroll(&self.file_list_scroll)
            .rounded(px(10.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background);
        for index in 0..count {
            list = list.child(self.file_row(index, cx));
        }

        design::page()
            .child(
                design::card(cx)
                    .child(self.picker_zone(cx))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("bl-add-files")
                                    .primary()
                                    .icon(Icon::new(IconName::Plus))
                                    .tooltip("选择文件（可多选）")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.pick_files(cx);
                                    })),
                            )
                            .child(
                                Button::new("bl-add-folder")
                                    .icon(Icon::new(IconName::FolderOpen))
                                    .tooltip("选择文件夹：Git / SVN 工程会按 .gitignore 忽略文件")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.pick_folder(cx);
                                    })),
                            )
                            .child(div().flex_1())
                            .when(has_entries, |row| {
                                row.child(design::pill(format!("{checked} / {count} 已选"), cx))
                            })
                            .child(
                                Button::new("bl-toggle-all")
                                    .icon(Icon::new(if all_selected {
                                        IconName::CircleCheck
                                    } else {
                                        IconName::Square
                                    }))
                                    .tooltip(if all_selected {
                                        "取消全选"
                                    } else {
                                        "全选"
                                    })
                                    .disabled(!has_entries)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_all(cx);
                                    })),
                            )
                            .child(
                                Button::new("bl-remove-checked")
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("移除已勾选的条目")
                                    .disabled(!has_entries)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.remove_checked(cx);
                                    })),
                            )
                            .child(
                                Button::new("bl-clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空列表与处理结果")
                                    .disabled(!has_entries)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_all(cx);
                                    })),
                            ),
                    )
                    .when(has_entries, |card| card.child(list))
                    .when(!has_entries, |card| {
                        card.child(
                            div()
                                .py(px(26.0))
                                .flex()
                                .justify_center()
                                .text_size(px(12.5))
                                .text_color(cx.theme().muted_foreground)
                                .child("列表为空，添加文件或文件夹后即可开始处理"),
                        )
                    }),
            )
            .child(
                design::card(cx)
                    .child(self.options_block(cx))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .mt_4()
                            .child(action_button)
                            .child(
                                Button::new("bl-open-dir")
                                    .icon(Icon::new(IconName::ExternalLink))
                                    .disabled(output_dir.is_none())
                                    .tooltip("在文件管理器中打开输出目录")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_output_dir(cx);
                                    })),
                            )
                            .when(busy, |row| {
                                row.child(
                                    div()
                                        .flex_1()
                                        .min_w(px(160.0))
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(div().flex_1().child(
                                            Progress::new("bl-progress").value(self.progress),
                                        ))
                                        .child(
                                            div()
                                                .text_size(px(11.5))
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("{:.0}%", self.progress)),
                                        ),
                                )
                            }),
                    )
                    .when(!message.is_empty(), |card| {
                        card.child(
                            div()
                                .mt_3()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Icon::new(IconName::Info)
                                        .size(px(14.0))
                                        .text_color(tone_color),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(tone_color)
                                        .child(message),
                                ),
                        )
                    })
                    .when(has_reports, |card| card.child(self.results_block(cx))),
            )
    }
}

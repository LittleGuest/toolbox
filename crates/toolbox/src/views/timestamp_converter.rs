use std::time::Duration;

use ::base::TimestampRow;
use gpui_kit::{
    component::{
        button::*,
        input::{InputEvent, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};
use time::OffsetDateTime;

const TZ_LABELS: [&str; 25] = [
    "UTC-12", "UTC-11", "UTC-10", "UTC-9", "UTC-8", "UTC-7", "UTC-6", "UTC-5", "UTC-4", "UTC-3",
    "UTC-2", "UTC-1", "UTC", "UTC+1", "UTC+2", "UTC+3", "UTC+4", "UTC+5", "UTC+6", "UTC+7",
    "UTC+8", "UTC+9", "UTC+10", "UTC+11", "UTC+12",
];
const TZ_OFFSETS: [i32; 25] = [
    -43200, -39600, -36000, -32400, -28800, -25200, -21600, -18000, -14400, -10800, -7200, -3600,
    0, 3600, 7200, 10800, 14400, 18000, 21600, 25200, 28800, 32400, 36000, 39600, 43200,
];

const MODES: [&str; 2] = ["时间戳 → 时间", "时间 → 时间戳"];
const UNITS: [&str; 2] = ["秒", "毫秒"];

pub struct TimestampConverter {
    mode: usize,
    unit: usize,
    tz_index: usize,
    input: String,
    results: Vec<TimestampRow>,
    error: String,
    loading: bool,
    current_time: String,
    input_state: Entity<TextareaState>,
    tz_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

fn format_now_timestamp() -> String {
    let now = OffsetDateTime::now_utc();
    now.unix_timestamp().to_string()
}

impl TimestampConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("每行一个时间戳或时间，如：\n1700000000\n2023-11-14 22:13:20")
        });

        let tz_items: Vec<String> = TZ_LABELS.iter().map(|s| s.to_string()).collect();
        let tz_state = cx.new(|cx| {
            let mut s = SelectState::new(tz_items, None, window, cx);
            s.set_selected_value(&"UTC+8".to_string(), window, cx);
            s
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.input = input_state.read(cx).value().to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(
                &tz_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.tz_index = TZ_LABELS
                            .iter()
                            .position(|&l| l == value.as_str())
                            .unwrap_or(20);
                        cx.notify();
                    }
                },
            ),
        ];

        let current_time = format_now_timestamp();
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let _ = this.update(cx, |this, cx| {
                    this.current_time = format_now_timestamp();
                    cx.notify();
                });
            }
        })
        .detach();

        Self {
            mode: 0,
            unit: 0,
            tz_index: 20,
            input: String::new(),
            results: Vec::new(),
            error: String::new(),
            loading: false,
            current_time,
            input_state,
            tz_state,
            _subscriptions,
        }
    }

    fn mode_str(&self) -> &'static str {
        if self.mode == 0 {
            "ts_to_dt"
        } else {
            "dt_to_ts"
        }
    }

    fn unit_str(&self) -> &'static str {
        if self.unit == 0 { "s" } else { "ms" }
    }

    fn tz_offset(&self) -> i32 {
        TZ_OFFSETS.get(self.tz_index).copied().unwrap_or(28800)
    }

    fn collect_values(&self) -> Vec<String> {
        self.input
            .split('\n')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    fn convert(&mut self, cx: &mut Context<Self>) {
        let values = self.collect_values();
        if values.is_empty() {
            self.error = "请输入时间戳或时间".to_string();
            self.results.clear();
            cx.notify();
            return;
        }
        self.loading = true;
        self.error.clear();
        cx.notify();

        let mode = self.mode_str().to_string();
        let unit = self.unit_str().to_string();
        let tz = self.tz_offset();

        let task = cx
            .background_executor()
            .spawn(async move { ::base::timestamp_convert(&mode, &unit, tz, &values) });

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(rows) => {
                        this.results = rows;
                        this.error.clear();
                    }
                    Err(e) => {
                        this.results.clear();
                        this.error = e.to_string();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn fill_example(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let example = if self.mode == 0 {
            "1700000000\n1700000000123"
        } else {
            "2023-11-14 22:13:20\n2023-11-14 22:13:20.123"
        };
        self.input = example.to_string();
        self.input_state.update(cx, |state, cx| {
            state.set_value(example.to_string(), window, cx);
        });
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.results.clear();
        self.error.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        cx.notify();
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(text.to_string(), window, cx);
                });
                cx.notify();
            }
        }
    }

    fn copy_text(&self, value: &str, cx: &mut Context<Self>) {
        if !value.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        }
    }

    /// 分段选择器（对应 n-radio-button 组）
    fn segmented(
        &self,
        name: &'static str,
        options: &[&'static str],
        selected: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .flex()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(cx.theme().background)
            .children(options.iter().enumerate().map(|(i, opt)| {
                let active = i == selected;
                div()
                    .id((name, i))
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
                        if name == "mode" {
                            this.mode = i;
                        } else {
                            this.unit = i;
                        }
                        cx.notify();
                    }))
                    .child(opt.to_string())
            }))
    }

    fn config_row(&self, cx: &mut Context<Self>) -> Div {
        let current = self.current_time.clone();

        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("当前秒级", cx))
                    .child(
                        div()
                            .w(px(130.0))
                            .flex()
                            .items_center()
                            .gap_1()
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().background)
                            .px_2()
                            .py_1()
                            .font_family("monospace")
                            .child(div().flex_1().text_sm().truncate().child(current.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("s"),
                            ),
                    )
                    .child(
                        Button::new("copy-current")
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制当前时间戳")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let value = this.current_time.clone();
                                this.copy_text(&value, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("方向", cx))
                    .child(self.segmented("mode", &MODES, self.mode, cx)),
            )
            .when(self.mode == 0, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(design::caption("单位", cx))
                        .child(self.segmented("unit", &UNITS, self.unit, cx)),
                )
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(design::caption("时区", cx))
                    .child(div().w(px(140.0)).child(Select::new(&self.tz_state))),
            )
    }

    /// 结果表格（对应 n-data-table）
    fn result_table(&self, cx: &mut Context<Self>) -> Div {
        let header = div()
            .flex()
            .items_center()
            .text_xs()
            .font_semibold()
            .text_color(cx.theme().muted_foreground)
            .border_b_1()
            .border_color(cx.theme().border)
            .pb_1()
            .child(self.cell("输入", px(200.0)))
            .child(self.cell("秒时间戳", px(130.0)))
            .child(self.cell("毫秒时间戳", px(150.0)))
            .child(div().flex_1().min_w_0().px_2().py_1p5().child("时间"))
            .child(self.cell("操作", px(140.0)));

        let rows: Vec<Div> = self
            .results
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let input = row.input.clone();
                let second = row.second_ts.map(|v| v.to_string());
                let milli = row.milli_ts.map(|v| v.to_string());
                let datetime = row.datetime.clone();

                div()
                    .flex()
                    .items_center()
                    .text_sm()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .w(px(200.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .font_family("monospace")
                            .truncate()
                            .child(input),
                    )
                    .child(
                        div()
                            .w(px(130.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .font_family("monospace")
                            .truncate()
                            .child(second.clone().unwrap_or_else(|| "-".to_string())),
                    )
                    .child(
                        div()
                            .w(px(150.0))
                            .flex_shrink_0()
                            .px_2()
                            .py_1p5()
                            .font_family("monospace")
                            .truncate()
                            .child(milli.clone().unwrap_or_else(|| "-".to_string())),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px_2()
                            .py_1p5()
                            .font_family("monospace")
                            .truncate()
                            .child(datetime.clone()),
                    )
                    .child(
                        div()
                            .w(px(140.0))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_1p5()
                            .child(self.copy_btn(("copy-second", idx), second, "复制秒时间戳", cx))
                            .child(self.copy_btn(("copy-milli", idx), milli, "复制毫秒时间戳", cx))
                            .child(self.copy_btn(("copy-dt", idx), Some(datetime), "复制时间", cx)),
                    )
            })
            .collect();

        div()
            .flex_col()
            .rounded(px(6.0))
            .border_1()
            .border_color(cx.theme().border)
            .px_1()
            .py_1()
            .child(header)
            .children(rows)
    }

    fn cell(&self, text: &'static str, width: Pixels) -> Div {
        div().w(width).flex_shrink_0().px_2().py_1p5().child(text)
    }

    fn copy_btn(
        &self,
        id: (&'static str, usize),
        value: Option<String>,
        tooltip: &'static str,
        cx: &mut Context<Self>,
    ) -> Button {
        Button::new(id)
            .ghost()
            .compact()
            .icon(Icon::new(IconName::Copy))
            .tooltip(if value.is_some() {
                tooltip
            } else {
                "值无效"
            })
            .disabled(value.is_none())
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(v) = value.clone() {
                    this.copy_text(&v, cx);
                }
            }))
    }
}

impl Render for TimestampConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let loading = self.loading;
        let error = self.error.clone();
        let has_result = !self.results.is_empty();

        design::page().child(
            design::card(cx)
                .child(self.config_row(cx))
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("输入", cx))
                        .child(
                            Textarea::new(&self.input_state)
                                .h(px(140.0))
                                .font_family("monospace"),
                        ),
                )
                .child(
                    design::action_row()
                        .child(
                            Button::new("convert")
                                .primary()
                                .icon(Icon::new(IconName::Replace))
                                .tooltip("转换")
                                .loading(loading)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.convert(cx);
                                })),
                        )
                        .child(
                            Button::new("fill-example")
                                .icon(Icon::new(IconName::Asterisk))
                                .tooltip("填入示例")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.fill_example(window, cx);
                                })),
                        )
                        .child(
                            Button::new("paste-input")
                                .icon(Icon::new(IconName::Inbox))
                                .tooltip("粘贴")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.paste(window, cx);
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
                .when(has_result, |this| {
                    this.child(
                        div()
                            .flex_col()
                            .gap_1p5()
                            .child(design::editor_label("结果", cx))
                            .child(self.result_table(cx)),
                    )
                })
                .when(!error.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::ERROR_RED)))
                            .child(error),
                    )
                }),
        )
    }
}

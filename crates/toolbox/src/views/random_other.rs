use gpui_kit::{
    component::{
        button::*,
        calendar::Date as PickerDate,
        checkbox::Checkbox,
        date_picker::{DatePicker, DatePickerState},
        input::{Input, InputState, Textarea, TextareaState},
        select::{Select, SelectEvent, SelectState},
        *,
    },
    prelude::FluentBuilder as _,
    *,
};
use rand::RngExt as _;
use time::{Date, Month};

const TABS: [&str; 4] = ["IP 地址", "MAC 地址", "时间", "日期"];

fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

fn parse_date(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u8 = parts[1].parse().ok()?;
    let day: u8 = parts[2].parse().ok()?;
    Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()
}

fn picker_date_string(date: &PickerDate) -> Option<String> {
    match date {
        PickerDate::Single(Some(d)) => Some(d.to_string()),
        _ => None,
    }
}

pub struct RandomOtherGenerator {
    mode: usize,
    ip_type: usize,
    ip_count_state: Entity<InputState>,
    ip_out_state: Entity<TextareaState>,

    mac_format: String,
    mac_format_state: Entity<SelectState<Vec<String>>>,
    mac_unicast: bool,
    mac_count_state: Entity<InputState>,
    mac_out_state: Entity<TextareaState>,

    time_format: usize,
    time_count_state: Entity<InputState>,
    time_out_state: Entity<TextareaState>,

    date_start_state: Entity<DatePickerState>,
    date_end_state: Entity<DatePickerState>,
    date_count_state: Entity<InputState>,
    date_out_state: Entity<TextareaState>,

    status: String,
    _subscriptions: Vec<Subscription>,
}

impl RandomOtherGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let count_input = |window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("10")
                    .default_value("10".to_string())
            })
        };
        let textarea = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder))
        };

        let mac_format_items = vec![
            "冒号分隔 (AA:BB:CC:DD:EE:FF)".to_string(),
            "短横分隔 (AA-BB-CC-DD-EE-FF)".to_string(),
            "无分隔".to_string(),
        ];
        let mac_format_state = cx.new(|cx| {
            let mut state = SelectState::new(mac_format_items, None, window, cx);
            state.set_selected_value(&"冒号分隔 (AA:BB:CC:DD:EE:FF)".to_string(), window, cx);
            state
        });

        let date_start_state = cx.new(|cx| DatePickerState::new(window, cx));
        let date_end_state = cx.new(|cx| DatePickerState::new(window, cx));

        let _subscriptions = vec![cx.subscribe_in(
            &mac_format_state,
            window,
            move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = ev {
                    this.mac_format = match value.as_str() {
                        "短横分隔 (AA-BB-CC-DD-EE-FF)" => "dash",
                        "无分隔" => "none",
                        _ => "colon",
                    }
                    .to_string();
                    cx.notify();
                }
            },
        )];

        Self {
            mode: 0,
            ip_type: 0,
            ip_count_state: count_input(window, cx),
            ip_out_state: textarea(window, cx, "随机 IP 地址（每行一个）"),
            mac_format: "colon".to_string(),
            mac_format_state,
            mac_unicast: false,
            mac_count_state: count_input(window, cx),
            mac_out_state: textarea(window, cx, "随机 MAC 地址（每行一个）"),
            time_format: 0,
            time_count_state: count_input(window, cx),
            time_out_state: textarea(window, cx, "随机时间（每行一个）"),
            date_start_state,
            date_end_state,
            date_count_state: count_input(window, cx),
            date_out_state: textarea(window, cx, "区间内随机日期 YYYY-MM-DD（每行一个）"),
            status: String::new(),
            _subscriptions,
        }
    }

    fn read_usize(
        &self,
        state: &Entity<InputState>,
        cx: &App,
        default: usize,
        max: usize,
    ) -> usize {
        state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(default)
            .clamp(1, max)
    }

    fn set_output(
        &self,
        state: &Entity<TextareaState>,
        lines: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = lines.join("\n");
        state.update(cx, |s, cx| {
            s.set_value(text, window, cx);
        });
    }

    fn copy_output(&mut self, state: &Entity<TextareaState>, cx: &mut Context<Self>) {
        let text = state.read(cx).value().to_string();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn clear_output(
        &mut self,
        state: &Entity<TextareaState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        state.update(cx, |s, cx| {
            s.set_value("".to_string(), window, cx);
        });
    }

    fn gen_ip(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.read_usize(&self.ip_count_state, cx, 10, 10000);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                if self.ip_type == 1 {
                    (0..8)
                        .map(|_| format!("{:x}", rng.random_range(0..0x10000u32)))
                        .collect::<Vec<_>>()
                        .join(":")
                } else {
                    (0..4)
                        .map(|_| rng.random_range(0..=255u32).to_string())
                        .collect::<Vec<_>>()
                        .join(".")
                }
            })
            .collect();
        self.set_output(&self.ip_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_mac(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.read_usize(&self.mac_count_state, cx, 10, 10000);
        let mut rng = rand::rng();
        let sep = match self.mac_format.as_str() {
            "dash" => "-",
            "none" => "",
            _ => ":",
        };
        let unicast_hi = ['0', '2', '4', '6', '8', 'A', 'C', 'E'];
        let unicast = self.mac_unicast;
        let lines: Vec<String> = (0..count)
            .map(|_| {
                (0..6)
                    .map(|j| {
                        if j == 1 && unicast {
                            let hi = unicast_hi[rng.random_range(0..unicast_hi.len())];
                            let lo = format!("{:X}", rng.random_range(0..16));
                            format!("{hi}{lo}")
                        } else {
                            format!("{:02X}", rng.random_range(0..=255u32))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(sep)
            })
            .collect();
        self.set_output(&self.mac_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_time(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.read_usize(&self.time_count_state, cx, 10, 10000);
        let is_12h = self.time_format == 1;
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let h = rng.random_range(0..24u32);
                let m = rng.random_range(0..60u32);
                let s = rng.random_range(0..60u32);
                if is_12h {
                    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
                    let ampm = if h < 12 { "AM" } else { "PM" };
                    format!("{:02}:{:02}:{:02} {}", h12, m, s, ampm)
                } else {
                    format!("{:02}:{:02}:{:02}", h, m, s)
                }
            })
            .collect();
        self.set_output(&self.time_out_state, lines, window, cx);
        cx.notify();
    }

    fn gen_date(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let start_s = picker_date_string(&self.date_start_state.read(cx).date());
        let end_s = picker_date_string(&self.date_end_state.read(cx).date());
        let (Some(start_s), Some(end_s)) = (start_s, end_s) else {
            self.status = "请选择开始和结束日期".to_string();
            cx.notify();
            return;
        };
        let (Some(start), Some(end)) = (parse_date(&start_s), parse_date(&end_s)) else {
            self.status = "请选择有效的开始和结束日期".to_string();
            cx.notify();
            return;
        };
        let count = self.read_usize(&self.date_count_state, cx, 10, 10000);
        let (lo, hi) = {
            let (a, b) = (start.to_julian_day(), end.to_julian_day());
            (a.min(b), a.max(b))
        };
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let day = rng.random_range(lo..=hi);
                Date::from_julian_day(day)
                    .map(format_date)
                    .unwrap_or_default()
            })
            .collect();
        self.status.clear();
        self.set_output(&self.date_out_state, lines, window, cx);
        cx.notify();
    }

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
                        match name {
                            "ip-type" => this.ip_type = i,
                            "time-format" => this.time_format = i,
                            _ => {}
                        }
                        cx.notify();
                    }))
                    .child(opt.to_string())
            }))
    }

    fn config_label(&self, text: &'static str, cx: &mut Context<Self>) -> Div {
        div()
            .text_size(px(13.0))
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_6()
            .border_b_1()
            .border_color(cx.theme().border)
            .mb_4()
            .children(
                TABS.iter()
                    .enumerate()
                    .map(|(i, label)| self.tab_btn(i, label, cx)),
            )
    }

    fn tab_btn(&self, idx: usize, label: &str, cx: &mut Context<Self>) -> Stateful<Div> {
        let active = self.mode == idx;
        div()
            .id(("random-tab", idx))
            .px_1()
            .pb_2()
            .mb(px(-1.0))
            .text_sm()
            .font_medium()
            .border_b_2()
            .text_color(if active {
                cx.theme().primary
            } else {
                cx.theme().muted_foreground
            })
            .border_color(if active {
                cx.theme().primary
            } else {
                gpui::black().opacity(0.0)
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.mode = idx;
                this.status.clear();
                cx.notify();
            }))
            .child(label.to_string())
    }

    fn output_editor(
        &self,
        id: &'static str,
        state: &Entity<TextareaState>,
        cx: &mut Context<Self>,
    ) -> Div {
        let state = state.clone();
        let copy_state = state.clone();
        let clear_state = state.clone();
        div()
            .flex_col()
            .gap_1p5()
            .child(design::editor_label("输出", cx))
            .child(
                Textarea::new(&state)
                    .h(design::CODE_BOX_HEIGHT)
                    .font_family("monospace"),
            )
            .child(
                design::action_row()
                    .child(
                        Button::new(ElementId::Name(format!("{id}-copy").into()))
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Copy))
                            .tooltip("复制")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let s = copy_state.clone();
                                this.copy_output(&s, cx);
                            })),
                    )
                    .child(
                        Button::new(ElementId::Name(format!("{id}-clear").into()))
                            .ghost()
                            .compact()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("清除")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let s = clear_state.clone();
                                this.clear_output(&s, window, cx);
                            })),
                    ),
            )
    }
}

impl Render for RandomOtherGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.mode;
        let status = self.status.clone();

        let gen_button = |id: &'static str, _cx: &mut Context<Self>| {
            Button::new(id)
                .primary()
                .icon(Icon::new(IconName::Play))
                .tooltip("生成")
        };

        let card = design::card(cx).child(self.tab_bar(cx)).child(match mode {
            0 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(self.segmented("ip-type", &["IPv4", "IPv6"], self.ip_type, cx))
                        .child(self.config_label("数量", cx))
                        .child(div().w(px(120.0)).child(Input::new(&self.ip_count_state)))
                        .child(gen_button("gen-ip", cx).on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_ip(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("ip", &self.ip_out_state, cx))
                .into_any_element(),
            1 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(self.config_label("格式", cx))
                        .child(
                            div()
                                .w(px(260.0))
                                .child(Select::new(&self.mac_format_state)),
                        )
                        .child(
                            Checkbox::new("mac-unicast")
                                .label("仅单播（第二段首字符为偶数）")
                                .checked(self.mac_unicast)
                                .on_click(cx.listener(|this, v: &bool, _, cx| {
                                    this.mac_unicast = *v;
                                    cx.notify();
                                })),
                        )
                        .child(self.config_label("数量", cx))
                        .child(div().w(px(120.0)).child(Input::new(&self.mac_count_state)))
                        .child(gen_button("gen-mac", cx).on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_mac(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("mac", &self.mac_out_state, cx))
                .into_any_element(),
            2 => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(self.segmented(
                            "time-format",
                            &["24 小时制 HH:MM:SS", "12 小时制 hh:mm:ss AM|PM"],
                            self.time_format,
                            cx,
                        ))
                        .child(self.config_label("数量", cx))
                        .child(div().w(px(120.0)).child(Input::new(&self.time_count_state)))
                        .child(gen_button("gen-time", cx).on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_time(window, cx);
                            },
                        ))),
                )
                .child(self.output_editor("time", &self.time_out_state, cx))
                .into_any_element(),
            _ => div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(self.config_label("开始日期", cx))
                        .child(
                            div()
                                .w(px(160.0))
                                .child(DatePicker::new(&self.date_start_state)),
                        )
                        .child(self.config_label("结束日期", cx))
                        .child(
                            div()
                                .w(px(160.0))
                                .child(DatePicker::new(&self.date_end_state)),
                        )
                        .child(self.config_label("数量", cx))
                        .child(div().w(px(120.0)).child(Input::new(&self.date_count_state)))
                        .child(gen_button("gen-date", cx).on_click(cx.listener(
                            |this, _, window, cx| {
                                this.gen_date(window, cx);
                            },
                        ))),
                )
                .when(!status.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(Hsla::from(rgb(design::WARN_AMBER)))
                            .child(status),
                    )
                })
                .child(self.output_editor("date", &self.date_out_state, cx))
                .into_any_element(),
        });

        design::page().child(card)
    }
}

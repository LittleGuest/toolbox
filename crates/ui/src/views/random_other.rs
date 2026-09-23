use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};
use rand::Rng;
use time::{Date, Month};

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

pub struct RandomOtherGenerator {
    ip_type: String,
    ip_type_state: Entity<SelectState<Vec<String>>>,
    ip_count_state: Entity<InputState>,
    ip_out_state: Entity<TextareaState>,

    mac_format: String,
    mac_format_state: Entity<SelectState<Vec<String>>>,
    mac_unicast: bool,
    mac_count_state: Entity<InputState>,
    mac_out_state: Entity<TextareaState>,

    time_format: String,
    time_format_state: Entity<SelectState<Vec<String>>>,
    time_count_state: Entity<InputState>,
    time_out_state: Entity<TextareaState>,

    date_start_state: Entity<InputState>,
    date_end_state: Entity<InputState>,
    date_count_state: Entity<InputState>,
    date_out_state: Entity<TextareaState>,

    status: String,
    _subscriptions: Vec<Subscription>,
}

impl RandomOtherGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        let textarea = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str| {
            cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder))
        };

        let ip_type_items = vec!["IPv4".to_string(), "IPv6".to_string()];
        let mac_format_items = vec![
            "冒号分隔 (AA:BB:CC:DD:EE:FF)".to_string(),
            "短横分隔 (AA-BB-CC-DD-EE-FF)".to_string(),
            "无分隔".to_string(),
        ];
        let time_format_items = vec![
            "24 小时制 HH:MM:SS".to_string(),
            "12 小时制 hh:mm:ss AM|PM".to_string(),
        ];

        let ip_type_state = cx.new(|cx| {
            let mut state = SelectState::new(ip_type_items, None, window, cx);
            state.set_selected_value(&"IPv4".to_string(), window, cx);
            state
        });
        let mac_format_state = cx.new(|cx| {
            let mut state = SelectState::new(mac_format_items, None, window, cx);
            state.set_selected_value(&"冒号分隔 (AA:BB:CC:DD:EE:FF)".to_string(), window, cx);
            state
        });
        let time_format_state = cx.new(|cx| {
            let mut state = SelectState::new(time_format_items, None, window, cx);
            state.set_selected_value(&"24 小时制 HH:MM:SS".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &ip_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.ip_type = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
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
            ),
            cx.subscribe_in(
                &time_format_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.time_format = if value.starts_with("12") { "12" } else { "24" }
                            .to_string();
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            ip_type: "v4".to_string(),
            ip_type_state,
            ip_count_state: input(window, cx, ""),
            ip_out_state: textarea(window, cx, "随机 IP 地址（每行一个）"),
            mac_format: "colon".to_string(),
            mac_format_state,
            mac_unicast: false,
            mac_count_state: input(window, cx, ""),
            mac_out_state: textarea(window, cx, "随机 MAC 地址（每行一个）"),
            time_format: "24".to_string(),
            time_format_state,
            time_count_state: input(window, cx, ""),
            time_out_state: textarea(window, cx, "随机时间（每行一个）"),
            date_start_state: input(window, cx, "YYYY-MM-DD"),
            date_end_state: input(window, cx, "YYYY-MM-DD"),
            date_count_state: input(window, cx, ""),
            date_out_state: textarea(window, cx, "区间内随机日期 YYYY-MM-DD（每行一个）"),
            status: String::new(),
            _subscriptions,
        }
    }

    fn read_usize(&self, state: &Entity<InputState>, cx: &App, default: usize, max: usize) -> usize {
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

    fn clear_output(&mut self, state: &Entity<TextareaState>, window: &mut Window, cx: &mut Context<Self>) {
        state.update(cx, |s, cx| {
            s.set_value("".to_string(), window, cx);
        });
    }

    fn gen_ip(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.read_usize(&self.ip_count_state, cx, 10, 10000);
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                if self.ip_type == "IPv6" {
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
        let lines: Vec<String> = (0..count)
            .map(|_| {
                (0..6)
                    .map(|j| {
                        if j == 1 && self.mac_unicast {
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
        let mut rng = rand::rng();
        let lines: Vec<String> = (0..count)
            .map(|_| {
                let h = rng.random_range(0..24u32);
                let m = rng.random_range(0..60u32);
                let s = rng.random_range(0..60u32);
                if self.time_format == "12" {
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
        let start_str = self.date_start_state.read(cx).value().to_string();
        let end_str = self.date_end_state.read(cx).value().to_string();
        let (Some(start), Some(end)) = (parse_date(&start_str), parse_date(&end_str)) else {
            self.status = "请输入有效的开始和结束日期（YYYY-MM-DD）".to_string();
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
}

impl Render for RandomOtherGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(64.0);

        let num_input = |state: &Entity<InputState>, w: f32| div().w(px(w)).child(Input::new(state));

        let action_buttons = |copy_id: &'static str,
                              clear_id: &'static str,
                              state: &Entity<TextareaState>,
                              cx: &mut Context<Self>| {
            let copy_state = state.clone();
            let clear_state = state.clone();
            design::toolbar()
                .child(
                    Button::new(copy_id)
                        .icon(Icon::new(IconName::Copy))
                        .tooltip("复制")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.copy_output(&copy_state, cx);
                        })),
                )
                .child(
                    Button::new(clear_id)
                        .icon(Icon::new(IconName::Close))
                        .tooltip("清除")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.clear_output(&clear_state, window, cx);
                        })),
                )
        };

        design::page()
            .child(design::page_header("随机 IP / MAC / 时间", "生成随机 IP / MAC / 时间", cx))
            .child(
                design::card(cx)
                    .child(design::caption("IP 地址", cx))
                    .child(
                        design::toolbar()
                            .flex_wrap()
                            .child(design::caption("类型", cx).w(label_w))
                            .child(Select::new(&self.ip_type_state))
                            .child(design::caption("数量", cx))
                            .child(num_input(&self.ip_count_state, 100.0))
                            .child(
                                Button::new("gen-ip")
                                    .label("生成")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.gen_ip(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.ip_out_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(action_buttons("copy-ip", "clear-ip", &self.ip_out_state, cx)),
                    ),
            )
            .child(
                design::card(cx)
                    .child(design::caption("MAC 地址", cx))
                    .child(
                        design::toolbar()
                            .flex_wrap()
                            .child(design::caption("格式", cx).w(label_w))
                            .child(Select::new(&self.mac_format_state))
                            .child(
                                Checkbox::new("mac-unicast")
                                    .label("仅单播（第二段首字符为偶数）")
                                    .checked(self.mac_unicast)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.mac_unicast = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(design::caption("数量", cx))
                            .child(num_input(&self.mac_count_state, 100.0))
                            .child(
                                Button::new("gen-mac")
                                    .label("生成")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.gen_mac(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.mac_out_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(action_buttons("copy-mac", "clear-mac", &self.mac_out_state, cx)),
                    ),
            )
            .child(
                design::card(cx)
                    .child(design::caption("时间", cx))
                    .child(
                        design::toolbar()
                            .flex_wrap()
                            .child(design::caption("格式", cx).w(label_w))
                            .child(Select::new(&self.time_format_state))
                            .child(design::caption("数量", cx))
                            .child(num_input(&self.time_count_state, 100.0))
                            .child(
                                Button::new("gen-time")
                                    .label("生成")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.gen_time(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.time_out_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(action_buttons("copy-time", "clear-time", &self.time_out_state, cx)),
                    ),
            )
            .child(
                design::card(cx)
                    .child(design::caption("日期", cx))
                    .child(
                        design::toolbar()
                            .flex_wrap()
                            .child(design::caption("开始日期", cx).w(label_w))
                            .child(num_input(&self.date_start_state, 160.0))
                            .child(design::caption("结束日期", cx))
                            .child(num_input(&self.date_end_state, 160.0))
                            .child(design::caption("数量", cx))
                            .child(num_input(&self.date_count_state, 100.0))
                            .child(
                                Button::new("gen-date")
                                    .label("生成")
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.gen_date(window, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xef4444))
                                    .child(self.status.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                Textarea::new(&self.date_out_state)
                                    .h(design::CODE_BOX_HEIGHT)
                                    .font_family("monospace"),
                            )
                            .child(action_buttons("copy-date", "clear-date", &self.date_out_state, cx)),
                    ),
            )
    }
}

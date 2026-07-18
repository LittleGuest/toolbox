use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    *,
};

#[derive(Default, Clone)]
struct IpResult {
    binary: String,
    octal: String,
    decimal: String,
    hex: String,
}

pub struct IpConverter {
    ipv4: String,
    ipv6: String,
    ipv4_result: IpResult,
    ipv6_result: IpResult,
    error: String,
    ipv4_state: Entity<InputState>,
    ipv6_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl IpConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ipv4_state = cx.new(|cx| InputState::new(window, cx).placeholder("例如 192.168.1.1"));
        let ipv6_state = cx.new(|cx| InputState::new(window, cx).placeholder("例如 2001:db8::1"));

        let _subscriptions = vec![
            cx.subscribe_in(&ipv4_state, window, {
                let ipv4_state = ipv4_state.clone();
                move |this, _, ev: &InputEvent, _, cx| match ev {
                    InputEvent::Change => {
                        this.ipv4 = ipv4_state.read(cx).value().to_string();
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.convert_ipv4();
                        cx.notify();
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(&ipv6_state, window, {
                let ipv6_state = ipv6_state.clone();
                move |this, _, ev: &InputEvent, _, cx| match ev {
                    InputEvent::Change => {
                        this.ipv6 = ipv6_state.read(cx).value().to_string();
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.convert_ipv6();
                        cx.notify();
                    }
                    _ => {}
                }
            }),
        ];

        Self {
            ipv4: String::new(),
            ipv6: String::new(),
            ipv4_result: IpResult::default(),
            ipv6_result: IpResult::default(),
            error: String::new(),
            ipv4_state,
            ipv6_state,
            _subscriptions,
        }
    }

    fn convert_ipv4(&mut self) {
        self.ipv4_result = self.convert("v4", self.ipv4.clone());
    }

    fn convert_ipv6(&mut self) {
        self.ipv6_result = self.convert("v6", self.ipv6.clone());
    }

    fn convert(&mut self, version: &str, ip: String) -> IpResult {
        self.error.clear();
        match base::ip_to_number(version, Some(ip)) {
            Ok(map) => IpResult {
                binary: map.get("binary").cloned().unwrap_or_default(),
                octal: map.get("octal").cloned().unwrap_or_default(),
                decimal: map.get("decimal").cloned().unwrap_or_default(),
                hex: map.get("hex").cloned().unwrap_or_default(),
            },
            Err(err) => {
                self.error = err.to_string();
                IpResult::default()
            }
        }
    }

    fn paste_ipv4(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.ipv4 = text.to_string();
                self.ipv4_state.update(cx, |state, cx| {
                    state.set_value(self.ipv4.clone(), window, cx);
                });
                self.convert_ipv4();
            }
        }
    }

    fn paste_ipv6(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.ipv6 = text.to_string();
                self.ipv6_state.update(cx, |state, cx| {
                    state.set_value(self.ipv6.clone(), window, cx);
                });
                self.convert_ipv6();
            }
        }
    }
}

impl Render for IpConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(ip_panel(
                    "IPv4 转换",
                    &self.ipv4_state,
                    self.ipv4_result.clone(),
                    true,
                    cx,
                ))
                .child(ip_panel(
                    "IPv6 转换",
                    &self.ipv6_state,
                    self.ipv6_result.clone(),
                    false,
                    cx,
                ))
                .when(!self.error.is_empty(), |this| {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().danger)
                            .child(self.error.clone()),
                    )
                }),
        )
    }
}

fn ip_panel(
    title: &'static str,
    input_state: &Entity<InputState>,
    result: IpResult,
    is_ipv4: bool,
    cx: &mut Context<IpConverter>,
) -> Div {
    let convert_id = if is_ipv4 {
        "convert-ipv4"
    } else {
        "convert-ipv6"
    };
    let paste_id = if is_ipv4 { "paste-ipv4" } else { "paste-ipv6" };

    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_4()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_lg().font_semibold().child(title))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_1().child(Input::new(input_state)))
                .child(
                    Button::new(convert_id)
                        .primary()
                        .icon(Icon::new(IconName::ArrowRight))
                        .tooltip("转换")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if is_ipv4 {
                                this.convert_ipv4();
                            } else {
                                this.convert_ipv6();
                            }
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(paste_id)
                        .icon(Icon::new(IconName::File))
                        .tooltip("粘贴")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if is_ipv4 {
                                this.paste_ipv4(window, cx);
                            } else {
                                this.paste_ipv6(window, cx);
                            }
                            cx.notify();
                        })),
                ),
        )
        .children([
            result_row(
                "二进制",
                if is_ipv4 {
                    "copy-ipv4-binary"
                } else {
                    "copy-ipv6-binary"
                },
                result.binary,
                cx,
            ),
            result_row(
                "八进制",
                if is_ipv4 {
                    "copy-ipv4-octal"
                } else {
                    "copy-ipv6-octal"
                },
                result.octal,
                cx,
            ),
            result_row(
                "十进制",
                if is_ipv4 {
                    "copy-ipv4-decimal"
                } else {
                    "copy-ipv6-decimal"
                },
                result.decimal,
                cx,
            ),
            result_row(
                "十六进制",
                if is_ipv4 {
                    "copy-ipv4-hex"
                } else {
                    "copy-ipv6-hex"
                },
                result.hex,
                cx,
            ),
        ])
}

fn result_row(
    label: &'static str,
    id: &'static str,
    value: String,
    cx: &mut Context<IpConverter>,
) -> Div {
    let display = if value.is_empty() {
        "-".to_string()
    } else {
        value.clone()
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w_20().text_sm().child(label))
        .child(
            div()
                .flex_1()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .px_2()
                .py_1()
                .text_sm()
                .font_family("monospace")
                .child(display),
        )
        .child(
            Button::new(id)
                .icon(Icon::new(IconName::Copy))
                .tooltip("复制")
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
                })),
        )
}

use gpui_kit::{
    component::{
        button::*,
        input::{
            InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, Textarea,
            TextareaState,
        },
        select::{Select, SelectEvent, SelectState},
        switch::Switch,
        *,
    },
    *,
};

const VERSIONS: [(&str, u8); 7] = [
    ("v1", 1),
    ("v3", 3),
    ("v4", 4),
    ("v5", 5),
    ("v6", 6),
    ("v7", 7),
    ("v8", 8),
];

/// 配置项：12px 灰色 label + 控件（对应 tb-config-item）
fn config_item(label: &'static str, control: Div) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(0x5b6478))
                .child(label),
        )
        .child(control)
}

pub struct UuidGenerator {
    uppercase: bool,
    remove_connector: bool,
    version: u8,
    version_state: Entity<SelectState<Vec<String>>>,
    number_state: Entity<InputState>,
    uuids_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl UuidGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let version_items: Vec<String> = VERSIONS.iter().map(|(l, _)| l.to_string()).collect();

        let version_state = cx.new(|cx| {
            let mut state = SelectState::new(version_items, None, window, cx);
            state.set_selected_value(&"v4".to_string(), window, cx);
            state
        });
        let number_state = cx.new(|cx| InputState::new(window, cx).default_value("5".to_string()));
        let uuids_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("点击上方生成按钮生成 UUID"));

        let _subscriptions = vec![
            cx.subscribe_in(
                &version_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        if let Some((_, v)) = VERSIONS.iter().find(|(l, _)| l == label) {
                            this.version = *v;
                            cx.notify();
                        }
                    }
                },
            ),
            cx.subscribe_in(
                &number_state,
                window,
                move |_this, state, ev: &InputEvent, window, cx| {
                    if let InputEvent::Blur = ev {
                        let text = state.read(cx).value().trim().to_string();
                        let value = text.parse::<u16>().unwrap_or(5).max(1);
                        if text != value.to_string() {
                            state.update(cx, |state, cx| {
                                state.set_value(value.to_string(), window, cx);
                            });
                        }
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &number_state,
                window,
                move |_this, state, ev: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = ev;
                    let text = state.read(cx).value();
                    let mut value = text.parse::<u16>().unwrap_or(5);
                    match action {
                        StepAction::Increment => value = value.saturating_add(1),
                        StepAction::Decrement => value = value.saturating_sub(1).max(1),
                    }
                    state.update(cx, |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    });
                    cx.notify();
                },
            ),
        ];

        Self {
            uppercase: false,
            remove_connector: false,
            version: 4,
            version_state,
            number_state,
            uuids_state,
            _subscriptions,
        }
    }

    fn generate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let number = self
            .number_state
            .read(cx)
            .value()
            .trim()
            .parse::<u16>()
            .unwrap_or(5)
            .max(1);

        match ::base::uuid(
            None,
            self.uppercase,
            self.remove_connector,
            self.version,
            number,
        ) {
            Ok(uuids) => {
                let text = uuids.join("\n");
                self.uuids_state.update(cx, |state, cx| {
                    state.set_value(text, window, cx);
                });
            }
            Err(e) => {
                self.uuids_state.update(cx, |state, cx| {
                    state.set_value(e.to_string(), window, cx);
                });
            }
        }
        cx.notify();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        let text = self.uuids_state.read(cx).value().to_string();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }
}

impl Render for UuidGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let uppercase = self.uppercase;
        let remove_connector = self.remove_connector;

        design::page().child(
            design::card(cx)
                // tb-config-row：大写 / 去掉连接符 / 版本 / 生成数量
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_4()
                        .child(config_item(
                            "大写",
                            div().child(Switch::new("uppercase").checked(uppercase).on_click(
                                cx.listener(|this, checked: &bool, _, cx| {
                                    this.uppercase = *checked;
                                    cx.notify();
                                }),
                            )),
                        ))
                        .child(config_item(
                            "去掉连接符",
                            div().child(
                                Switch::new("remove-connector")
                                    .checked(remove_connector)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.remove_connector = *checked;
                                        cx.notify();
                                    })),
                            ),
                        ))
                        .child(config_item(
                            "UUID版本",
                            div().w(px(140.0)).child(Select::new(&self.version_state)),
                        ))
                        .child(config_item(
                            "生成数量",
                            div()
                                .w(px(140.0))
                                .child(NumberInput::new(&self.number_state)),
                        )),
                )
                // tb-action-row：生成（主色图标按钮）
                .child(
                    div().flex().items_center().gap_2().child(
                        Button::new("generate")
                            .primary()
                            .icon(Icon::new(IconName::Asterisk))
                            .tooltip("生成")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.generate(window, cx);
                            })),
                    ),
                )
                // tb-editor：UUID 列表 + 复制
                .child(
                    div()
                        .flex_col()
                        .gap_1p5()
                        .child(design::editor_label("UUID 列表", cx))
                        .child(
                            Textarea::new(&self.uuids_state)
                                .h(px(220.0))
                                .font_family("monospace"),
                        )
                        .child(
                            div().flex().items_center().gap_2().mt_2().child(
                                Button::new("copy-uuids")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy(cx);
                                    })),
                            ),
                        ),
                ),
        )
    }
}

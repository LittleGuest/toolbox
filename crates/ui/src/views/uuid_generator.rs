use gpui::*;
use gpui_component::{
    button::*,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    switch::Switch,
    *,
};

pub struct UuidGenerator {
    uppercase: bool,
    remove_connector: bool,
    version: u32,
    number: u32,
    uuids: String,
    version_state: Entity<SelectState<Vec<String>>>,
    number_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl UuidGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let version_items = vec![
            "V1".to_string(),
            "V3".to_string(),
            "V4".to_string(),
            "V5".to_string(),
            "V6".to_string(),
            "V7".to_string(),
            "V8".to_string(),
        ];

        let version_state = cx.new(|cx| {
            let mut state = SelectState::new(version_items, None, window, cx);
            state.set_selected_value(&"V4".to_string(), window, cx);
            state
        });
        let number_state = cx.new(|cx| InputState::new(window, cx).default_value("5"));

        let _subscriptions = vec![
            cx.subscribe_in(
                &version_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        let version = match value.as_str() {
                            "V1" => 1,
                            "V3" => 3,
                            "V4" => 4,
                            "V5" => 5,
                            "V6" => 6,
                            "V7" => 7,
                            "V8" => 8,
                            _ => 4,
                        };
                        this.set_version(version, cx);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &number_state,
                window,
                move |this, state, ev: &InputEvent, _, cx| {
                    if let InputEvent::Blur = ev {
                        let text = state.read(cx).value();
                        let value = text.parse::<u32>().unwrap_or(5);
                        this.set_number(value);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &number_state,
                window,
                move |this, state, ev: &NumberInputEvent, window, cx| {
                    if let NumberInputEvent::Step(action) = ev {
                        let text = state.read(cx).value();
                        let mut value = text.parse::<u32>().unwrap_or(5);
                        match action {
                            StepAction::Increment => value = value.saturating_add(1),
                            StepAction::Decrement => value = value.saturating_sub(1),
                        }
                        state.update(cx, |state, cx| {
                            state.set_value(value.to_string(), window, cx);
                        });
                        this.set_number(value);
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            uppercase: false,
            remove_connector: false,
            version: 4,
            number: 5,
            uuids: String::new(),
            version_state,
            number_state,
            _subscriptions,
        }
    }

    fn generate(&mut self, cx: &mut Context<Self>) {
        self.number = self
            .number_state
            .read(cx)
            .value()
            .parse::<u32>()
            .unwrap_or(5);

        let mut results = Vec::new();
        for _ in 0..self.number {
            let uuid_str = match self.version {
                1 => base::uuid::uuid_v1().unwrap_or_default(),
                3 => base::uuid::uuid_v3("namespace", "name").unwrap_or_default(),
                4 => base::uuid::uuid_v4().unwrap_or_default(),
                5 => base::uuid::uuid_v5("namespace", "name").unwrap_or_default(),
                6 => base::uuid::uuid_v6().unwrap_or_default(),
                7 => base::uuid::uuid_v7().unwrap_or_default(),
                8 => base::uuid::uuid_v8().unwrap_or_default(),
                _ => base::uuid::uuid_v4().unwrap_or_default(),
            };

            let mut uuid_str = uuid_str;
            if self.uppercase {
                uuid_str = uuid_str.to_uppercase();
            }
            if self.remove_connector {
                uuid_str = uuid_str.replace("-", "");
            }
            results.push(uuid_str);
        }

        self.uuids = results.join("\n");
    }

    fn clear(&mut self) {
        self.uuids.clear();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.uuids.clone()));
    }

    fn set_uppercase(&mut self, uppercase: bool) {
        self.uppercase = uppercase;
    }

    fn set_remove_connector(&mut self, remove_connector: bool) {
        self.remove_connector = remove_connector;
    }

    fn set_version(&mut self, version: u32, cx: &mut Context<Self>) {
        self.version = version;
        self.generate(cx);
    }

    fn set_number(&mut self, number: u32) {
        self.number = number;
    }
}

impl Render for UuidGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let uppercase = self.uppercase;
        let remove_connector = self.remove_connector;

        let uuids_text = if self.uuids.is_empty() {
            "点击生成按钮生成UUID...".to_string()
        } else {
            self.uuids.clone()
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("UUID 生成器"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("大写"))
                            .child(Switch::new("uppercase").checked(uppercase).on_click(
                                cx.listener(|this, checked: &bool, _, cx| {
                                    this.set_uppercase(*checked);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("去掉连接符"))
                            .child(
                                Switch::new("remove-connector")
                                    .checked(remove_connector)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.set_remove_connector(*checked);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("UUID版本"))
                            .child(Select::new(&self.version_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("生成数量UUID(s) x"))
                            .child(NumberInput::new(&self.number_state)),
                    )
                    .child(
                        ButtonGroup::new("buttons")
                            .child(
                                Button::new("generate")
                                    .primary()
                                    .icon(Icon::new(IconName::Plus))
                                    .tooltip("生成")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.generate(cx);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("copy")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy(cx);
                                    })),
                            ),
                    )
                    .child(
                        div().flex().flex_col().gap_2().child(
                            div()
                                .flex_1()
                                .min_h(px(300.0))
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .p_2()
                                .overflow_y_scrollbar()
                                .text_sm()
                                .font_family("monospace")
                                .child(uuids_text),
                        ),
                    ),
            )
    }
}

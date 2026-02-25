use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    *,
};

pub struct BaseConverter {
    input: String,
    binary: String,
    octal: String,
    decimal: String,
    hex: String,
    input_type: base::Base,
    input_state: Entity<InputState>,
    input_type_state: Entity<SelectState<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl BaseConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let type_items = vec![
            "二进制".to_string(),
            "八进制".to_string(),
            "十进制".to_string(),
            "十六进制".to_string(),
        ];

        let input_type_state = cx.new(|cx| {
            let mut state = SelectState::new(type_items, None, window, cx);
            state.set_selected_value(&"十进制".to_string(), window, cx);
            state
        });
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("请输入数字..."));

        let _subscriptions = vec![
            cx.subscribe_in(
                &input_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        let input_type = match value.as_str() {
                            "二进制" => base::Base::Binary,
                            "八进制" => base::Base::Octal,
                            "十进制" => base::Base::Decimal,
                            "十六进制" => base::Base::Hex,
                            _ => base::Base::Decimal,
                        };
                        this.input_type = input_type;
                        this.convert();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(&input_state, window, {
                let input_state = input_state.clone();
                move |this, _, _ev: &InputEvent, _window, cx| {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    this.convert();
                    cx.notify();
                }
            }),
        ];

        Self {
            input: String::new(),
            binary: String::new(),
            octal: String::new(),
            decimal: String::new(),
            hex: String::new(),
            input_type: base::Base::Decimal,
            input_state,
            input_type_state,
            _subscriptions,
        }
    }

    fn convert(&mut self) {
        if self.input.is_empty() {
            self.binary.clear();
            self.octal.clear();
            self.decimal.clear();
            self.hex.clear();
            return;
        }

        let input_type = match self.input_type {
            base::Base::Binary => base::Base::Binary,
            base::Base::Octal => base::Base::Octal,
            base::Base::Decimal => base::Base::Decimal,
            base::Base::Hex => base::Base::Hex,
        };
        if let Ok(result) = base::number_base(Some(input_type), self.input.clone()) {
            self.binary = result.get("binary").cloned().unwrap_or_default();
            self.octal = result.get("octal").cloned().unwrap_or_default();
            self.decimal = result.get("decimal").cloned().unwrap_or_default();
            self.hex = result.get("hex").cloned().unwrap_or_default();
        } else {
            self.binary.clear();
            self.octal.clear();
            self.decimal.clear();
            self.hex.clear();
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.input.clear();
        self.binary.clear();
        self.octal.clear();
        self.decimal.clear();
        self.hex.clear();
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard() {
            if let Some(text) = text.text() {
                self.input = text.to_string();
                self.convert();
                cx.notify();
            }
        }
    }
}

impl Render for BaseConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let binary = self.binary.clone();
        let octal = self.octal.clone();
        let decimal = self.decimal.clone();
        let hex = self.hex.clone();

        let binary_text = if binary.is_empty() {
            "-".to_string()
        } else {
            binary.clone()
        };

        let octal_text = if octal.is_empty() {
            "-".to_string()
        } else {
            octal.clone()
        };

        let decimal_text = if decimal.is_empty() {
            "-".to_string()
        } else {
            decimal.clone()
        };

        let hex_text = if hex.is_empty() {
            "-".to_string()
        } else {
            hex.clone()
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("进制转换"))
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
                            .child(div().text_sm().w_20().child("输入类型"))
                            .child(Select::new(&self.input_type_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("输入"))
                            .child(
                                ButtonGroup::new("input-buttons")
                                    .child(
                                        Button::new("paste")
                                            .icon(Icon::new(IconName::File))
                                            .tooltip("粘贴")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.paste(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("copy_input")
                                            .icon(Icon::new(IconName::Copy))
                                            .tooltip("复制")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(
                                                    this.input.clone(),
                                                ));
                                            })),
                                    ),
                            )
                            .child(div().flex_1().child(Input::new(&self.input_state))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Delete))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("二进制"))
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
                                    .child(binary_text),
                            )
                            .child(
                                Button::new("copy_binary")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.binary.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("八进制"))
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
                                    .child(octal_text),
                            )
                            .child(
                                Button::new("copy_octal")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.octal.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("十进制"))
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
                                    .child(decimal_text),
                            )
                            .child(
                                Button::new("copy_decimal")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.decimal.clone(),
                                        ));
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().w_20().child("十六进制"))
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
                                    .child(hex_text),
                            )
                            .child(
                                Button::new("copy_hex")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            this.hex.clone(),
                                        ));
                                    })),
                            ),
                    ),
            )
    }
}

use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState, Textarea, TextareaState},
    select::{Select, SelectEvent, SelectState},
    *,
};
use datafaker::{Faker, Locale};
use rand::Rng;
use time::{Date, Month};

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

pub struct RandomDataGenerator {
    field_name: bool,
    field_email: bool,
    field_phone: bool,
    field_address: bool,
    field_uuid: bool,
    field_age: bool,
    field_date: bool,
    count_state: Entity<InputState>,
    format_is_json_array: bool,
    format_state: Entity<SelectState<Vec<String>>>,
    output_state: Entity<TextareaState>,
    status: String,
    _subscriptions: Vec<Subscription>,
}

impl RandomDataGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let count_state = cx.new(|cx| InputState::new(window, cx).placeholder("1-10000"));
        let output_state = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("生成的假数据")
        });
        let format_state = cx.new(|cx| {
            let mut state = SelectState::new(
                vec!["JSON 数组".to_string(), "每行一条".to_string()],
                None,
                window,
                cx,
            );
            state.set_selected_value(&"JSON 数组".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![cx.subscribe_in(
            &format_state,
            window,
            move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = ev {
                    this.format_is_json_array = value == "JSON 数组";
                    cx.notify();
                }
            },
        )];

        Self {
            field_name: true,
            field_email: true,
            field_phone: true,
            field_address: false,
            field_uuid: false,
            field_age: true,
            field_date: false,
            count_state,
            format_is_json_array: true,
            format_state,
            output_state,
            status: String::new(),
            _subscriptions,
        }
    }

    fn field_list(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.field_name {
            fields.push("name");
        }
        if self.field_email {
            fields.push("email");
        }
        if self.field_phone {
            fields.push("phone");
        }
        if self.field_address {
            fields.push("address");
        }
        if self.field_uuid {
            fields.push("uuid");
        }
        if self.field_age {
            fields.push("age");
        }
        if self.field_date {
            fields.push("date");
        }
        fields
    }

    fn build_row(
        &self,
        fields: &[&'static str],
        faker: &Faker,
        rng: &mut impl Rng,
    ) -> Vec<(&'static str, String, bool)> {
        fields
            .iter()
            .map(|f| {
                let value = match *f {
                    "name" => (faker.name().name(), false),
                    "email" => (faker.internet().standard_public_email(), false),
                    "phone" => (faker.person().mobile(), false),
                    "address" => (faker.address().full_address(), false),
                    "uuid" => (faker.uuid().uuid_v4(), false),
                    "age" => (rng.random_range(18..=80i64).to_string(), true),
                    // 1970-01-01 ~ 2030-12-31 之间的随机日期
                    "date" => {
                        let lo = Date::from_calendar_date(1970, Month::January, 1)
                            .unwrap()
                            .to_julian_day();
                        let hi = Date::from_calendar_date(2030, Month::December, 31)
                            .unwrap()
                            .to_julian_day();
                        let day = rng.random_range(lo..=hi);
                        let d = Date::from_julian_day(day)
                            .unwrap_or(Date::from_calendar_date(1970, Month::January, 1).unwrap());
                        (
                            format!(
                                "{:04}-{:02}-{:02}",
                                d.year(),
                                u8::from(d.month()),
                                d.day()
                            ),
                            false,
                        )
                    }
                    _ => (String::new(), false),
                };
                (*f, value.0, value.1)
            })
            .collect()
    }

    fn generate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let fields = self.field_list();
        if fields.is_empty() {
            self.status = "请至少选择一个字段".to_string();
            cx.notify();
            return;
        }
        let count = self
            .count_state
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .unwrap_or(5)
            .clamp(1, 10000);

        let mut faker = Faker::new();
        faker.locale = Locale::ZhCn;
        let mut rng = rand::rng();

        let mut text = String::new();
        if self.format_is_json_array {
            text.push_str("[\n");
        }
        for i in 0..count {
            let row = self.build_row(&fields, &faker, &mut rng);
            if self.format_is_json_array {
                text.push_str("  {\n");
                for (j, (name, value, is_num)) in row.iter().enumerate() {
                    let comma = if j + 1 < row.len() { "," } else { "" };
                    if *is_num {
                        text.push_str(&format!("    \"{}\": {}{}\n", name, value, comma));
                    } else {
                        text.push_str(&format!(
                            "    \"{}\": \"{}\"{}\n",
                            name,
                            json_escape(value),
                            comma
                        ));
                    }
                }
                text.push_str("  }");
                text.push_str(if i + 1 < count { ",\n" } else { "\n" });
            } else {
                let pairs: Vec<String> = row
                    .iter()
                    .map(|(name, value, is_num)| {
                        if *is_num {
                            format!("\"{}\": {}", name, value)
                        } else {
                            format!("\"{}\": \"{}\"", name, json_escape(value))
                        }
                    })
                    .collect();
                text.push_str(&format!("{{{}}}\n", pairs.join(", ")));
            }
        }
        if self.format_is_json_array {
            text.push_str("]");
        }

        self.status.clear();
        let output = text;
        self.output_state.update(cx, |state, cx| {
            state.set_value(output, window, cx);
        });
        cx.notify();
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        let text = self.output_state.read(cx).value().to_string();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status.clear();
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }
}

impl Render for RandomDataGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label_w = px(80.0);

        design::page()
            .child(design::page_header("随机数据", "生成随机数据记录", cx))
            .child(
                // 字段与格式配置卡片
                design::card(cx)
                    .child(design::caption("字段", cx))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_4()
                            .child(
                                Checkbox::new("field-name")
                                    .label("姓名")
                                    .checked(self.field_name)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_name = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-email")
                                    .label("邮箱")
                                    .checked(self.field_email)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_email = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-phone")
                                    .label("手机号")
                                    .checked(self.field_phone)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_phone = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-address")
                                    .label("地址")
                                    .checked(self.field_address)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_address = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-uuid")
                                    .label("UUID")
                                    .checked(self.field_uuid)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_uuid = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-age")
                                    .label("年龄")
                                    .checked(self.field_age)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_age = *v;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("field-date")
                                    .label("日期")
                                    .checked(self.field_date)
                                    .on_click(cx.listener(|this, v: &bool, _, cx| {
                                        this.field_date = *v;
                                        cx.notify();
                                    })),
                            ),
                    )
                    // 数量
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("数量", cx)))
                            .child(div().w(px(160.0)).child(Input::new(&self.count_state))),
                    )
                    // 输出格式
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(label_w).child(design::caption("输出格式", cx)))
                            .child(Select::new(&self.format_state)),
                    ),
            )
            // 生成操作行
            .child(
                design::action_row()
                    .child(
                        Button::new("generate")
                            .label("生成")
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.generate(window, cx);
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xef4444))
                            .child(self.status.clone()),
                    ),
            )
            // 输出卡片
            .child(
                design::card(cx)
                    .child(
                        Textarea::new(&self.output_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    )
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("copy-output")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输出")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_output(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-output")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清除")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    ),
            )
    }
}

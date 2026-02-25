use gpui::*;
use gpui_component::{
    button::*,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    *,
};
use rand::Rng;

pub struct FakeDataGenerator {
    data_type: String,
    count: usize,
    output: String,
    data_type_state: Entity<SelectState<Vec<String>>>,
    count_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl FakeDataGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data_type_items = vec![
            "姓名".to_string(),
            "邮箱".to_string(),
            "电话".to_string(),
            "地址".to_string(),
            "公司".to_string(),
            "UUID".to_string(),
            "数字".to_string(),
            "日期".to_string(),
        ];

        let data_type_state = cx.new(|cx| {
            let mut state = SelectState::new(data_type_items, None, window, cx);
            state.set_selected_value(&"姓名".to_string(), window, cx);
            state
        });
        let count_state = cx.new(|cx| InputState::new(window, cx).default_value("10"));

        let _subscriptions = vec![
            cx.subscribe_in(
                &data_type_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        let data_type = match value.as_str() {
                            "姓名" => "name",
                            "邮箱" => "email",
                            "电话" => "phone",
                            "地址" => "address",
                            "公司" => "company",
                            "UUID" => "uuid",
                            "数字" => "number",
                            "日期" => "date",
                            _ => "name",
                        };
                        this.data_type = data_type.to_string();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &count_state,
                window,
                move |this, state, ev: &InputEvent, _, cx| {
                    if let InputEvent::Blur = ev {
                        let text = state.read(cx).value();
                        let value = text.parse::<usize>().unwrap_or(10);
                        this.count = value;
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &count_state,
                window,
                move |this, state, ev: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = ev;
                    let text = state.read(cx).value();
                    let mut value = text.parse::<usize>().unwrap_or(10);
                    match action {
                        StepAction::Increment => value = value.saturating_add(1),
                        StepAction::Decrement => value = value.saturating_sub(1),
                    }
                    state.update(cx, |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    });
                    this.count = value;
                    cx.notify();
                },
            ),
        ];

        Self {
            data_type: "name".to_string(),
            count: 10,
            output: String::new(),
            data_type_state,
            count_state,
            _subscriptions,
        }
    }

    fn generate(&mut self, cx: &mut Context<Self>) {
        self.count = self
            .count_state
            .read(cx)
            .value()
            .parse::<usize>()
            .unwrap_or(10);

        let mut rng = rand::rng();
        let mut result = Vec::new();

        for _ in 0..self.count {
            let data = match self.data_type.as_str() {
                "name" => self.generate_name(&mut rng),
                "email" => self.generate_email(&mut rng),
                "phone" => self.generate_phone(&mut rng),
                "address" => self.generate_address(&mut rng),
                "company" => self.generate_company(&mut rng),
                "uuid" => self.generate_uuid(),
                "number" => self.generate_number(&mut rng),
                "date" => self.generate_date(&mut rng),
                _ => String::new(),
            };
            result.push(data);
        }

        self.output = result.join("\n");
    }

    fn generate_name(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let first_names = vec![
            "张", "王", "李", "赵", "刘", "陈", "杨", "黄", "周", "吴", "徐", "孙", "马", "朱",
            "胡", "郭", "何", "高", "林", "罗",
        ];

        let last_names = vec![
            "伟", "芳", "娜", "敏", "静", "丽", "强", "磊", "军", "洋", "勇", "艳", "杰", "娟",
            "涛", "明", "秀英", "平", "刚", "桂英",
        ];

        let first = first_names[rng.random_range(0..first_names.len())];
        let last = last_names[rng.random_range(0..last_names.len())];

        format!("{}{}", first, last)
    }

    fn generate_email(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let domains = vec![
            "gmail.com",
            "qq.com",
            "163.com",
            "126.com",
            "hotmail.com",
            "outlook.com",
            "yahoo.com",
            "sina.com",
            "sohu.com",
            "aliyun.com",
        ];

        let username = self.generate_random_string(rng, 8);
        let domain = domains[rng.random_range(0..domains.len())];

        format!("{}@{}", username, domain)
    }

    fn generate_phone(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let prefixes = vec![
            "138", "139", "136", "137", "158", "159", "188", "189", "150", "151", "152", "153",
            "155", "156", "157", "186",
        ];

        let prefix = prefixes[rng.random_range(0..prefixes.len())];
        let suffix = rng.random_range(0..100_000_000);

        format!("{}{:08}", prefix, suffix)
    }

    fn generate_address(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let cities = vec![
            "北京市",
            "上海市",
            "广州市",
            "深圳市",
            "杭州市",
            "成都市",
            "武汉市",
            "西安市",
            "南京市",
            "重庆市",
            "天津市",
            "苏州市",
        ];

        let districts = vec![
            "朝阳区",
            "海淀区",
            "西城区",
            "东城区",
            "浦东新区",
            "黄浦区",
            "天河区",
            "越秀区",
            "南山区",
            "福田区",
            "西湖区",
            "江干区",
        ];

        let streets = vec![
            "人民路",
            "建设路",
            "解放路",
            "中山路",
            "和平路",
            "胜利路",
            "文化路",
            "科技路",
            "花园路",
            "朝阳路",
            "光明路",
            "新华路",
        ];

        let city = cities[rng.random_range(0..cities.len())];
        let district = districts[rng.random_range(0..districts.len())];
        let street = streets[rng.random_range(0..streets.len())];
        let number = rng.random_range(1..1000);

        format!("{}{}{}{}号", city, district, street, number)
    }

    fn generate_company(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let prefixes = vec![
            "北京", "上海", "广州", "深圳", "杭州", "成都", "武汉", "西安",
        ];

        let types = vec![
            "科技有限公司",
            "信息技术有限公司",
            "网络科技有限公司",
            "软件有限公司",
            "电子科技有限公司",
            "数码科技有限公司",
            "智能科技有限公司",
            "数据科技有限公司",
        ];

        let names = vec![
            "创新", "科技", "未来", "智能", "云端", "数据", "网络", "信息", "数码", "电子", "软件",
            "系统", "平台", "服务", "咨询", "管理",
        ];

        let prefix = prefixes[rng.random_range(0..prefixes.len())];
        let name = names[rng.random_range(0..names.len())];
        let type_suffix = types[rng.random_range(0..types.len())];

        format!("{}{}{}", prefix, name, type_suffix)
    }

    fn generate_uuid(&self) -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
    }

    fn generate_number(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let num: u32 = rng.random();
        num.to_string()
    }

    fn generate_date(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let timestamp = duration.as_secs() - (rng.random_range(0..365) * 24 * 60 * 60);

        let dt = time::OffsetDateTime::from_unix_timestamp(timestamp as i64).unwrap();
        dt.format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap()
    }

    fn generate_random_string(&self, rng: &mut rand::rngs::ThreadRng, length: usize) -> String {
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .chars()
            .collect();

        (0..length)
            .map(|_| chars[rng.random_range(0..chars.len())])
            .collect()
    }

    fn clear(&mut self) {
        self.output.clear();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
    }
}

impl Render for FakeDataGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let output_text = if self.output.is_empty() {
            "点击生成按钮生成假数据...".to_string()
        } else {
            self.output.clone()
        };

        div()
            .p_4()
            .child(div().text_xl().font_semibold().mb_4().child("假数据生成器"))
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
                            .child(div().text_sm().child("数据类型"))
                            .child(Select::new(&self.data_type_state)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("生成数量"))
                            .child(NumberInput::new(&self.count_state)),
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
                                .child(output_text),
                        ),
                    ),
            )
    }
}

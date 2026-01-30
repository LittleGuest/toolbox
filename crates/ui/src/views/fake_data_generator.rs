use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};
use rand::Rng;

pub struct FakeDataGenerator {
    data_type: String,
    count: usize,
    output: String,
}

impl FakeDataGenerator {
    pub fn new() -> Self {
        Self {
            data_type: "name".to_string(),
            count: 10,
            output: String::new(),
        }
    }

    fn generate(&mut self) {
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
            "张", "王", "李", "赵", "刘", "陈", "杨", "黄", "周", "吴",
            "徐", "孙", "马", "朱", "胡", "郭", "何", "高", "林", "罗",
        ];

        let last_names = vec![
            "伟", "芳", "娜", "敏", "静", "丽", "强", "磊", "军", "洋",
            "勇", "艳", "杰", "娟", "涛", "明", "秀英", "平", "刚", "桂英",
        ];

        let first = first_names[rng.random_range(0..first_names.len())];
        let last = last_names[rng.random_range(0..last_names.len())];
        
        format!("{}{}", first, last)
    }

    fn generate_email(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let domains = vec![
            "gmail.com", "qq.com", "163.com", "126.com", "hotmail.com",
            "outlook.com", "yahoo.com", "sina.com", "sohu.com", "aliyun.com",
        ];
        
        let username = self.generate_random_string(rng, 8);
        let domain = domains[rng.random_range(0..domains.len())];
        
        format!("{}@{}", username, domain)
    }

    fn generate_phone(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let prefixes = vec![
            "138", "139", "136", "137", "158", "159", "188", "189",
            "150", "151", "152", "153", "155", "156", "157", "186",
        ];
        
        let prefix = prefixes[rng.random_range(0..prefixes.len())];
        let suffix = rng.random_range(0..100_000_000);
        
        format!("{}{:08}", prefix, suffix)
    }

    fn generate_address(&self, rng: &mut rand::rngs::ThreadRng) -> String {
        let cities = vec![
            "北京市", "上海市", "广州市", "深圳市", "杭州市", "成都市",
            "武汉市", "西安市", "南京市", "重庆市", "天津市", "苏州市",
        ];

        let districts = vec![
            "朝阳区", "海淀区", "西城区", "东城区", "浦东新区", "黄浦区",
            "天河区", "越秀区", "南山区", "福田区", "西湖区", "江干区",
        ];

        let streets = vec![
            "人民路", "建设路", "解放路", "中山路", "和平路", "胜利路",
            "文化路", "科技路", "花园路", "朝阳路", "光明路", "新华路",
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
            "科技有限公司", "信息技术有限公司", "网络科技有限公司", "软件有限公司",
            "电子科技有限公司", "数码科技有限公司", "智能科技有限公司", "数据科技有限公司",
        ];

        let names = vec![
            "创新", "科技", "未来", "智能", "云端", "数据", "网络", "信息",
            "数码", "电子", "软件", "系统", "平台", "服务", "咨询", "管理",
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
        dt.format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap()
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
}

impl Render for FakeDataGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let data_type_text = match self.data_type.as_str() {
            "name" => "姓名",
            "email" => "邮箱",
            "phone" => "电话",
            "address" => "地址",
            "company" => "公司",
            "uuid" => "UUID",
            "number" => "数字",
            "date" => "日期",
            _ => "姓名",
        };
        
        let count_text = format!("{} 条", self.count);

        let output_text = if self.output.is_empty() {
            "点击生成按钮生成假数据...".to_string()
        } else {
            self.output.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("假数据生成器")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("数据类型")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(data_type_text)
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .child("数量")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_2()
                                    .py_1()
                                    .child(count_text)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("生成")
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("清空")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("生成结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(output_text)
                            )
                    )
            )
    }
}
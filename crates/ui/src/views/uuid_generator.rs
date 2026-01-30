use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, *};

pub struct UuidGenerator {
    uppercase: bool,
    remove_connector: bool,
    version: u32,
    number: u32,
    uuids: String,
}

impl UuidGenerator {
    pub fn new() -> Self {
        Self {
            uppercase: false,
            remove_connector: false,
            version: 4,
            number: 5,
            uuids: String::new(),
        }
    }

    fn generate(&mut self) {
        use uuid::Uuid;
        use uuid::Timestamp;
        
        let mut results = Vec::new();
        for _ in 0..self.number {
            let uuid = match self.version {
                1 => {
                    let ts = Timestamp::now(uuid::timestamp::context::NoContext);
                    let node_id: [u8; 6] = [0, 0, 0, 0, 0, 1];
                    Uuid::new_v1(ts, &node_id)
                },
                3 => Uuid::new_v3(&Uuid::nil(), b"namespace"),
                4 => Uuid::new_v4(),
                5 => Uuid::new_v5(&Uuid::nil(), b"namespace"),
                6 => {
                    let ts = Timestamp::now(uuid::timestamp::context::NoContext);
                    let node_id: [u8; 6] = [0, 0, 0, 0, 0, 1];
                    Uuid::new_v6(ts, &node_id)
                },
                7 => {
                    let ts = Timestamp::now(uuid::timestamp::context::NoContext);
                    Uuid::new_v7(ts)
                },
                8 => {
                    let buf: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
                    Uuid::new_v8(buf)
                },
                _ => Uuid::new_v4(),
            };

            let mut uuid_str = uuid.to_string();
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

    fn toggle_uppercase(&mut self) {
        self.uppercase = !self.uppercase;
        self.generate();
    }

    fn toggle_remove_connector(&mut self) {
        self.remove_connector = !self.remove_connector;
        self.generate();
    }

    fn set_version(&mut self, version: u32) {
        self.version = version;
        self.generate();
    }

    fn set_number(&mut self, number: u32) {
        self.number = number;
        self.generate();
    }
}

impl Render for UuidGenerator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let uppercase = self.uppercase;
        let remove_connector = self.remove_connector;
        let version = self.version;
        let number = self.number;
        
        let uuids_text = if self.uuids.is_empty() {
            "点击生成按钮生成UUID...".to_string()
        } else {
            self.uuids.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("UUID 生成器")
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
                                    .child("大写")
                            )
                            .child(
                                div()
                                    .w_12()
                                    .h_6()
                                    .bg(if uppercase { cx.theme().primary } else { cx.theme().border })
                                    .rounded_full()
                                    .cursor_pointer()
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("去掉连接符")
                            )
                            .child(
                                div()
                                    .w_12()
                                    .h_6()
                                    .bg(if remove_connector { cx.theme().primary } else { cx.theme().border })
                                    .rounded_full()
                                    .cursor_pointer()
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .child("版本")
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
                                    .cursor_pointer()
                                    .child(format!("V{}", version))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
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
                                    .cursor_pointer()
                                    .child(number.to_string())
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("generate")
                                    .primary()
                                    .label("生成")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.generate();
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("clear")
                                    .label("清空")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear();
                                        cx.notify();
                                    }))
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
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(uuids_text)
                            )
                    )
            )
    }
}
use gpui::*;
use gpui_component::{button::*, *};

pub struct TransformOpenapi {
    input: String,
    output: String,
}

impl TransformOpenapi {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
        }
    }

    fn generate(&mut self) {
        if self.input.is_empty() {
            self.output.clear();
            return;
        }

        // Simple OpenAPI generation logic
        // In a real implementation, you would use proper OpenAPI libraries
        self.output = format!(
            r#"openapi: 3.0.0
info:
  title: Generated API
  version: 1.0.0
paths:
  /api/example:
    get:
      summary: Example endpoint
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                type: object
                properties:
                  message:
                    type: string
                    example: "Hello from OpenAPI"
                  input:
                    type: string
                    example: "{}"
"#, 
            self.input
        );
    }

    fn clear(&mut self) {
        self.input.clear();
        self.output.clear();
    }
}

impl Render for TransformOpenapi {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("OpenAPI 工具")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child("API 名称/描述")
                            )
                            .child(
                                div()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_md()
                                    .px_3()
                                    .py_2()
                                    .text_sm()
                                    .child(self.input.clone())
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                Button::new("generate")
                                    .primary()
                                    .label("生成 OpenAPI 规范")
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
                                    .child("OpenAPI 规范")
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
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(if self.output.is_empty() { "-".to_string() } else { self.output.clone() })
                            )
                    )
            )
    }
}

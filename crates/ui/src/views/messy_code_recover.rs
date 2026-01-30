use gpui::*;
use gpui_component::{scroll::ScrollableElement, *};

#[derive(Clone, Debug)]
pub struct RecoveryResult {
    index: usize,
    source_charset: String,
    target_charset: String,
    recovered_text: String,
    score: f64,
}

pub struct MessyCodeRecover {
    input: String,
    results: Vec<RecoveryResult>,
    loading: bool,
}

impl MessyCodeRecover {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            results: Vec::new(),
            loading: false,
        }
    }

    fn recover(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        self.loading = true;
        
        let input_text = self.input.clone();
        
        let mut results = Vec::new();
        
        let charsets = vec![
            ("UTF-8", "GBK"),
            ("GBK", "UTF-8"),
            ("UTF-8", "ISO-8859-1"),
            ("ISO-8859-1", "UTF-8"),
            ("UTF-8", "BIG5"),
            ("BIG5", "UTF-8"),
            ("GBK", "BIG5"),
            ("BIG5", "GBK"),
        ];

        let mut index = 0;
        for (source, target) in charsets {
            if let Some(recovered) = self.try_recover(&input_text, source, target) {
                let score = self.calculate_score(&recovered);
                results.push(RecoveryResult {
                    index,
                    source_charset: source.to_string(),
                    target_charset: target.to_string(),
                    recovered_text: recovered,
                    score,
                });
                index += 1;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        self.results = results;
        self.loading = false;
    }

    fn try_recover(&self, input: &str, source_charset: &str, target_charset: &str) -> Option<String> {
        let bytes = input.as_bytes();
        
        let recovered = match (source_charset, target_charset) {
            ("UTF-8", "GBK") | ("UTF-8", "BIG5") => {
                let mut result = String::new();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] < 0x80 {
                        result.push(bytes[i] as char);
                        i += 1;
                    } else if i + 1 < bytes.len() {
                        let b1 = bytes[i] as u16;
                        let b2 = bytes[i + 1] as u16;
                        let code = (b1 << 8) | b2;
                        if let Some(c) = std::char::from_u32(code as u32) {
                            result.push(c);
                        } else {
                            result.push('制');
                        }
                        i += 2;
                    } else {
                        result.push('制');
                        i += 1;
                    }
                }
                result
            },
            ("GBK", "UTF-8") | ("BIG5", "UTF-8") => {
                String::from_utf8_lossy(bytes).to_string()
            },
            ("UTF-8", "ISO-8859-1") => {
                bytes.iter().map(|&b| b as char).collect()
            },
            ("ISO-8859-1", "UTF-8") => {
                String::from_utf8_lossy(bytes).to_string()
            },
            _ => input.to_string(),
        };

        if recovered.contains('制') || recovered.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            None
        } else {
            Some(recovered)
        }
    }

    fn calculate_score(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        let mut valid_chars = 0;
        let mut total_chars = 0;

        for c in text.chars() {
            total_chars += 1;
            if c.is_alphabetic() || c.is_numeric() || c.is_whitespace() || 
               "，。！？、；：\"\'()()【】《制".contains(c) {
                valid_chars += 1;
            }
        }

        if total_chars == 0 {
            return 0.0;
        }

        valid_chars as f64 / total_chars as f64
    }

    fn clear(&mut self) {
        self.input.clear();
        self.results.clear();
    }
}

impl Render for MessyCodeRecover {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_text = if self.input.is_empty() {
            format!("请输入乱码文本{}例如{}锘挎槬鐪犱笉瑙夋檽锛屽澶勯椈鍟奸笩 ",
                std::char::from_u32(0xFF0C).unwrap_or(','),
                std::char::from_u32(0xFF1A).unwrap_or(':'))
        } else {
            self.input.clone()
        };
        
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .mb_4()
                    .child("乱码恢复")
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
                                    .child("输入")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(120.0))
                                    .max_h(px(120.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .text_sm()
                                    .font_family("monospace")
                                    .child(input_text)
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
                                    .child(if self.loading { "恢复中..." } else { "恢复" })
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
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("说明{}并非所有乱码都可以被完美恢复{}乱码中的问号说明该字符已经丢失{}是无法恢复的{}",
                std::char::from_u32(0xFF1A).unwrap_or(':'),
                std::char::from_u32(0xFF0C).unwrap_or(','),
                std::char::from_u32(0xFF0C).unwrap_or(','),
                std::char::from_u32(0x3002).unwrap_or('.')))
                    )
                    .child(if !self.results.is_empty() {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .child("恢复结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(300.0))
                                    .max_h(px(300.0))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p_4()
                                    .overflow_y_scrollbar()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(
                                                self.results.iter().map(|result| {
                                                    let score_text = format!("{:.2}%", result.score * 100.0);
                                                    let score_color = if result.score >= 0.9 {
                                                        cx.theme().success
                                                    } else if result.score >= 0.7 {
                                                        cx.theme().warning
                                                    } else {
                                                        cx.theme().accent
                                                    };
                                                    
                                                    div()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_4()
                                                                        .child(
                                                                            div()
                                                                                .text_xs()
                                                                                .child(format!("{} -> {}", result.source_charset, result.target_charset))
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .text_xs()
                                                                                .font_semibold()
                                                                                .text_color(score_color)
                                                                                .child(score_text)
                                                                        )
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_family("monospace")
                                                                        .child(result.recovered_text.clone())
                                                                )
                                                        )
                                                })
                                            )
                                    )
                            )
                    } else {
                        div()
                    })
            )
    }
}
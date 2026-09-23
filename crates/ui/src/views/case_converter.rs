use crate::design;
use gpui_kit::*;
use gpui_kit::component::{
    button::*,
    input::{InputEvent, Textarea, TextareaState},
    *,
};

pub struct CaseConverter {
    input: String,
    output: String,
    input_state: Entity<TextareaState>,
    output_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

fn to_upper(s: &str) -> anyhow::Result<String> {
    Ok(s.to_uppercase())
}

fn to_lower(s: &str) -> anyhow::Result<String> {
    Ok(s.to_lowercase())
}

fn split_words(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let boundary = if cur.is_empty() {
            false
        } else {
            let prev = chars[i - 1];
            let next_lower = chars.get(i + 1).map(|n| n.is_lowercase()).unwrap_or(false);
            (c.is_uppercase() && (prev.is_lowercase() || prev.is_numeric()))
                || (c.is_uppercase() && prev.is_uppercase() && next_lower)
        };
        if boundary {
            words.push(std::mem::take(&mut cur));
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn capitalize(w: &str) -> String {
    let mut cs = w.chars();
    match cs.next() {
        Some(f) => f.to_uppercase().collect::<String>() + &cs.as_str().to_lowercase(),
        None => String::new(),
    }
}

fn join_words(words: &[String], sep: &str, f: fn(&str) -> String) -> String {
    words.iter().map(|w| f(w)).collect::<Vec<_>>().join(sep)
}

fn sentence_case(s: &str) -> anyhow::Result<String> {
    let mut chars = s.trim().chars();
    Ok(match chars.next() {
        Some(f) => f.to_uppercase().chain(chars.flat_map(char::to_lowercase)).collect(),
        None => String::new(),
    })
}

fn title_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), " ", capitalize))
}

fn camel_case(s: &str) -> anyhow::Result<String> {
    let words = split_words(s);
    Ok(match words.split_first() {
        Some((first, rest)) => {
            first.to_lowercase() + &join_words(rest, "", capitalize)
        }
        None => String::new(),
    })
}

fn pascal_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "", capitalize))
}

fn lower_word(w: &str) -> String {
    w.to_lowercase()
}

fn upper_word(w: &str) -> String {
    w.to_uppercase()
}

fn snake_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "_", lower_word))
}

fn constant_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "_", upper_word))
}

fn kebab_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "-", lower_word))
}

fn cobol_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "-", upper_word))
}

fn train_case(s: &str) -> anyhow::Result<String> {
    Ok(join_words(&split_words(s), "-", capitalize))
}

fn alternating_case(s: &str) -> anyhow::Result<String> {
    let mut uppercase = false;
    Ok(s.chars()
        .map(|c| {
            if c.is_alphabetic() {
                uppercase = !uppercase;
                if uppercase {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            } else {
                c.to_string()
            }
        })
        .collect())
}

fn inverse_case(s: &str) -> anyhow::Result<String> {
    Ok(s.chars()
        .flat_map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<Vec<_>>()
            } else if c.is_uppercase() {
                c.to_lowercase().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect())
}

fn reverse_text(s: &str) -> anyhow::Result<String> {
    Ok(s.chars().rev().collect())
}

impl CaseConverter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| TextareaState::new(window, cx).placeholder("请输入要转换的文本"));
        let output_state = cx.new(|cx| TextareaState::new(window, cx).placeholder("转换结果"));

        let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let value = input_state.read(cx).value();
                    this.input = value.to_string();
                    cx.notify();
                }
            }
        })];

        Self {
            input: String::new(),
            output: String::new(),
            input_state,
            output_state,
            _subscriptions,
        }
    }

    fn apply(
        &mut self,
        f: fn(&str) -> anyhow::Result<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.input.is_empty() {
            return;
        }
        self.output = match f(&self.input) {
            Ok(s) => s,
            Err(e) => format!("转换失败: {e}"),
        };
        self.output_state.update(cx, |state, cx| {
            state.set_value(self.output.clone(), window, cx);
        });
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.clear();
        self.output.clear();
        self.input_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("".to_string(), window, cx);
        });
    }

    fn copy_input(&mut self, cx: &mut Context<Self>) {
        if !self.input.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.input.clone()));
        }
    }

    fn copy_output(&mut self, cx: &mut Context<Self>) {
        if !self.output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
        }
    }

    fn paste_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.input = text.to_string();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(self.input.clone(), window, cx);
                });
            }
        }
    }
}

impl Render for CaseConverter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        const CASES: [(&str, &str, fn(&str) -> anyhow::Result<String>); 14] = [
            ("upper", "全部大写", to_upper),
            ("lower", "全部小写", to_lower),
            ("sentence", "句首大写", sentence_case),
            ("title", "标题大写", title_case),
            ("camel", "驼峰 camelCase", camel_case),
            ("pascal", "帕斯卡 PascalCase", pascal_case),
            ("snake", "蛇形 snake_case", snake_case),
            ("constant", "常量 CONSTANT_CASE", constant_case),
            ("kebab", "烤肉串 kebab-case", kebab_case),
            ("cobol", "COBOL-CASE", cobol_case),
            ("train", "Train-Case", train_case),
            ("alternating", "交替 aLtErNaTiNg", alternating_case),
            ("inverse", "反转大小写", inverse_case),
            ("reverse", "反转文本", reverse_text),
        ];
        let buttons: Vec<Button> = CASES
            .iter()
            .map(|(id, label, f)| {
                let f = *f;
                Button::new(*id).child(*label).on_click(cx.listener(
                    move |this, _, window, cx| {
                        this.apply(f, window, cx);
                    },
                ))
            })
            .collect();

        design::page()
            .child(design::page_header("大小写转换", "文本大小写快速转换", cx))
            .child(
                design::card(cx)
                    .child(
                        design::toolbar()
                            .child(
                                Button::new("paste-input")
                                    .icon(Icon::new(IconName::File))
                                    .tooltip("粘贴")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.paste_input(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("copy-input")
                                    .icon(Icon::new(IconName::Copy))
                                    .tooltip("复制输入")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_input(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear")
                                    .icon(Icon::new(IconName::Close))
                                    .tooltip("清空")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear(window, cx);
                                    })),
                            )
                            .child(div().flex_1()),
                    )
                    .child(
                        Textarea::new(&self.input_state)
                            .h(design::CODE_BOX_HEIGHT)
                            .font_family("monospace"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(div().flex().flex_wrap().gap_2().children(buttons)),
            )
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
                            .child(div().flex_1()),
                    ),
            )
    }
}

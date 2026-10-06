use gpui_kit::{
    component::{Theme, ThemeMode, button::*, select::*, switch::Switch, *},
    prelude::FluentBuilder,
    *,
};

use crate::config_store;

const DEFAULT_LANGUAGE: &str = "zh_cn";
const DEFAULT_THEME: &str = "light";
const DEFAULT_FONT: &str = "system";

pub struct SettingsView {
    theme_label: String,
    theme_value: String,
    language_value: String,
    font_value: String,
    compact_mode: bool,
    smart_detect: bool,
    status: String,
    theme_state: Option<Entity<SelectState<Vec<String>>>>,
    language_state: Option<Entity<SelectState<Vec<String>>>>,
    font_state: Option<Entity<SelectState<Vec<String>>>>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let theme_items = vec!["浅色".to_string(), "深色".to_string()];
        let language_items = vec!["简体中文".to_string(), "English".to_string()];
        let font_items = vec![
            "系统默认".to_string(),
            "等宽字体".to_string(),
            "苹方/微软雅黑优先".to_string(),
        ];

        let theme_state = cx.new(|cx| {
            let mut state = SelectState::new(theme_items, None, window, cx);
            state.set_selected_value(&"浅色".to_string(), window, cx);
            state
        });
        let language_state = cx.new(|cx| {
            let mut state = SelectState::new(language_items, None, window, cx);
            state.set_selected_value(&"简体中文".to_string(), window, cx);
            state
        });
        let font_state = cx.new(|cx| {
            let mut state = SelectState::new(font_items, None, window, cx);
            state.set_selected_value(&"系统默认".to_string(), window, cx);
            state
        });

        let _subscriptions = vec![
            cx.subscribe_in(&theme_state, window, {
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.theme_value = match value.as_str() {
                            "深色" => "dark",
                            _ => "light",
                        }
                        .to_string();
                        this.apply_theme(cx);
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&language_state, window, {
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.language_value = match value.as_str() {
                            "English" => "en_us",
                            _ => "zh_cn",
                        }
                        .to_string();
                        cx.notify();
                    }
                }
            }),
            cx.subscribe_in(&font_state, window, {
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.font_value = match value.as_str() {
                            "等宽字体" => "monospace",
                            "苹方/微软雅黑优先" => "sans_cn",
                            _ => "system",
                        }
                        .to_string();
                        cx.notify();
                    }
                }
            }),
        ];

        let mut view = Self {
            theme_label: cx.theme().mode.name().to_string(),
            theme_value: DEFAULT_THEME.to_string(),
            language_value: DEFAULT_LANGUAGE.to_string(),
            font_value: DEFAULT_FONT.to_string(),
            compact_mode: false,
            smart_detect: false,
            status: String::new(),
            theme_state: Some(theme_state),
            language_state: Some(language_state),
            font_state: Some(font_state),
            _subscriptions,
        };

        view.load_settings(window, cx);
        view
    }

    fn load_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status = "正在加载设置...".to_string();
        cx.notify();
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let theme = config_store::get_setting("theme")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| DEFAULT_THEME.to_string());
            let language = config_store::get_setting("language")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
            let font = config_store::get_setting("font")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| DEFAULT_FONT.to_string());
            let compact = config_store::get_setting("compact_mode")
                .await
                .ok()
                .flatten()
                .map(|v| v == "true")
                .unwrap_or(false);
            let smart_detect = config_store::get_setting("smart_detect")
                .await
                .ok()
                .flatten()
                .map(|v| v == "true")
                .unwrap_or(false);

            let _ = this.update_in(cx, |this, window, cx| {
                this.theme_value = theme.clone();
                this.language_value = language.clone();
                this.font_value = font.clone();
                this.compact_mode = compact;
                this.smart_detect = smart_detect;

                if let Some(state) = &this.theme_state {
                    let label = if theme == "dark" { "深色" } else { "浅色" }.to_string();
                    state.update(cx, |s, cx| {
                        s.set_selected_value(&label, window, cx);
                    });
                }
                if let Some(state) = &this.language_state {
                    let label = if language == "en_us" {
                        "English"
                    } else {
                        "简体中文"
                    }
                    .to_string();
                    state.update(cx, |s, cx| {
                        s.set_selected_value(&label, window, cx);
                    });
                }
                if let Some(state) = &this.font_state {
                    let label = match font.as_str() {
                        "monospace" => "等宽字体",
                        "sans_cn" => "苹方/微软雅黑优先",
                        _ => "系统默认",
                    }
                    .to_string();
                    state.update(cx, |s, cx| {
                        s.set_selected_value(&label, window, cx);
                    });
                }

                this.apply_theme(cx);
                this.status = "设置已加载。".to_string();
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_theme(&mut self, cx: &mut Context<Self>) {
        let mode = if self.theme_value == "dark" {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        Theme::change(mode, None, cx);
        self.theme_label = cx.theme().mode.name().to_string();
    }

    fn set_compact_mode(&mut self, value: bool, cx: &mut Context<Self>) {
        self.compact_mode = value;
        cx.notify();
    }

    fn set_smart_detect(&mut self, value: bool, cx: &mut Context<Self>) {
        self.smart_detect = value;
        cx.notify();
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let theme = self.theme_value.clone();
        let language = self.language_value.clone();
        let font = self.font_value.clone();
        let compact = self.compact_mode.to_string();
        let smart_detect = self.smart_detect.to_string();

        self.status = "正在保存设置...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let r1 = config_store::set_setting("theme", &theme).await;
            let r2 = config_store::set_setting("language", &language).await;
            let r3 = config_store::set_setting("font", &font).await;
            let r4 = config_store::set_setting("compact_mode", &compact).await;
            let r5 = config_store::set_setting("smart_detect", &smart_detect).await;

            let has_err = r1.is_err() || r2.is_err() || r3.is_err() || r4.is_err() || r5.is_err();
            let _ = this.update(cx, |this, cx| {
                if has_err {
                    this.status = "保存设置失败，请重试。".to_string();
                } else {
                    this.status = "设置已保存。".to_string();
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn reset_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.theme_value = DEFAULT_THEME.to_string();
        self.language_value = DEFAULT_LANGUAGE.to_string();
        self.font_value = DEFAULT_FONT.to_string();
        self.compact_mode = false;
        self.smart_detect = false;

        if let Some(state) = &self.theme_state {
            state.update(cx, |s, cx| {
                s.set_selected_value(&"浅色".to_string(), window, cx);
            });
        }
        if let Some(state) = &self.language_state {
            state.update(cx, |s, cx| {
                s.set_selected_value(&"简体中文".to_string(), window, cx);
            });
        }
        if let Some(state) = &self.font_state {
            state.update(cx, |s, cx| {
                s.set_selected_value(&"系统默认".to_string(), window, cx);
            });
        }

        self.apply_theme(cx);
        self.status = "已恢复默认设置，请点击「保存设置」以持久化。".to_string();
        cx.notify();
    }

    fn config_row(label: &'static str, cx: &App) -> Div {
        div().flex().items_center().gap_3().child(
            div()
                .w(px(100.0))
                .flex_shrink_0()
                .text_size(px(13.0))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = self.theme_state.clone();
        let language_state = self.language_state.clone();
        let font_state = self.font_state.clone();
        let status_ok = self.status.contains("失败");

        design::page().child(
            design::card(cx)
                .child(
                    Self::config_row("语言", cx).child(
                        div()
                            .w(px(220.0))
                            .children(language_state.map(|s| Select::new(&s))),
                    ),
                )
                .child(
                    Self::config_row("智能检测", cx).child(
                        Switch::new("smart-detect")
                            .checked(self.smart_detect)
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.set_smart_detect(*v, cx);
                            })),
                    ),
                )
                .child(
                    Self::config_row("字体", cx).child(
                        div()
                            .w(px(220.0))
                            .children(font_state.map(|s| Select::new(&s))),
                    ),
                )
                .child(
                    Self::config_row("紧凑模式", cx).child(
                        Switch::new("compact-mode")
                            .checked(self.compact_mode)
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.set_compact_mode(*v, cx);
                            })),
                    ),
                )
                .child(
                    Self::config_row("主题", cx).child(
                        div()
                            .w(px(220.0))
                            .children(theme_state.map(|s| Select::new(&s))),
                    ),
                )
                .child(
                    design::action_row()
                        .child(
                            Button::new("save-settings")
                                .primary()
                                .icon(Icon::new(IconName::Check))
                                .tooltip("保存设置")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save_settings(cx);
                                })),
                        )
                        .child(
                            Button::new("reset-settings")
                                .icon(Icon::new(IconName::RotateCw))
                                .tooltip("恢复默认")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.reset_settings(window, cx);
                                })),
                        ),
                )
                .when(!self.status.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(if status_ok {
                                Hsla::from(rgb(design::ERROR_RED))
                            } else {
                                cx.theme().muted_foreground
                            })
                            .child(self.status.clone()),
                    )
                }),
        )
    }
}
